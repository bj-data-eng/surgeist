Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Filter Effects Module Level 1](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/).

Original copyright notice: Copyright © 2018 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Filter Effects Module Level 1

Source snapshot: https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/

Snapshot SHA-256: b73408da406802d4aa405c8ab1c0591732ff1403bae74f7267d53dc74b0dfb3d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 14 MathML expressions are represented as portable fenced TeX. Independent round-trip checks cover mathematical tokens, matrix shape/order, scripts, fractions and root structure; exact source MathML is retained in verification metadata. Visual equivalence is not certified.
- 34 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>Filter Effects Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2018 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

Filter effects are a way of processing an element’s rendering before it is displayed in the document. Typically, rendering an element via CSS or SVG can conceptually be described as if the element, including its children, are drawn into a buffer (such as a raster image) and then that buffer is composited into the elements parent. Filters apply an effect before the compositing stage. Examples of such effects are blurring, changing color intensity and warping the image.

<a id="ref-for-propdef-filter"></a>

<a id="ref-for-image-type"></a>

Although originally designed for use in SVG, filter effects are a set of operations to apply on an image buffer and therefore can be applied to nearly any presentational environment, including CSS. They are triggered by a style instruction (the [filter](#propdef-filter) property). This specification describes filters in a manner that allows them to be used in content styled by CSS, such as HTML and SVG. It also defines a CSS property value function that produces a CSS [\<image\>](https://www.w3.org/TR/css3-images/#image-type) value.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of&#xA;   its publication. Other documents may supersede this document. A list of&#xA;   current W3C publications and the latest revision of this technical report&#xA;   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports&#xA;   index at https://www.w3.org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “filter-effects” in the title, preferably like this: “\[filter-effects\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) (part of the [Style Activity](https://www.w3.org/Style/)).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 February 2018 W3C Process Document](https://www.w3.org/2018/Process-20180201/).

## <a id="intro"></a>1. Introduction

<em>This section is not normative</em>

A filter effect is a graphical operation that is applied to an element as it is drawn into the document. It is an image-based effect, in that it takes zero or more images as input, a number of parameters specific to the effect, and then produces an image as output. The output image is either rendered into the document instead of the original element, used as an input image to another filter effect, or provided as a CSS image value.

A simple example of a filter effect is a “flood”. It takes no image inputs but has a parameter defining a color. The effect produces an output image that is completely filled with the given color. A slightly more complex example is an “inversion” which takes a single image input (typically an image of the element as it would normally be rendered into its parent) and adjusts each pixel such that they have the opposite color values.

Filter effects are exposed with two levels of complexity:

1.  A small set of canned filter functions that are given by name. While not particularly powerful, these are convenient and easily understood and provide a simple approach to achieving common effects, such as blurring. The canned filters can also be animated by [\[CSS3-ANIMATIONS\]](#biblio-css3-animations).

2.  A graph of individual filter effects described in markup that define an overall effect. The graph is agnostic to its input in that the effect can be applied to any content. While such graphs are the combination of effects that may be simple in isolation, the graph as a whole can produce complex effects. An example is given below.

<a id="ref-for-funcdef-filter-grayscale"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6d81ccb9"></a> In this example, an image is filtered with the [\<grayscale()\>](#funcdef-filter-grayscale) filter function.
>
> ```text
> #image {
>     filter: grayscale(100%);
> }
> ```
>
> ![Example for grayscale filter applied to image](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/images/grayscale.svg)
>
> An image without filter (left) and the same filter with a 100% grayscale filter (right).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f7874f10"></a> The following shows an example of graph of individual filter effects.
>
> ![Example Filter](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01.png)
>
> Initial example for a filtered object.
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01.svg)
>
> The filter effect used in the example above is repeated here with reference numbers in the left column before each of the six filter primitives:
>
> <strong>Table 1 — structured row/cell transcription</strong>
>
> <strong>Row 1</strong>
>
> <strong>Column 1 (data cell):</strong>
>
>   
>   
> 1  
> 2  
> 3  
>   
>   
>   
>   
> 4  
> 5  
>   
> 6  
>   
>   
>
> <strong>Column 2 (data cell):</strong>
>
> ```text
> <filter id="MyFilter" filterUnits="userSpaceOnUse" x="0" y="0" width="200" height="120">
>   <desc>Produces a 3D lighting effect.</desc>
>   <feGaussianBlur in="SourceAlpha" stdDeviation="4" result="blur"/>
>   <feOffset in="blur" dx="4" dy="4" result="offsetBlur"/>
>   <feSpecularLighting in="blur" surfaceScale="5" specularConstant=".75"
>                       specularExponent="20" lighting-color="#bbbbbb"
>                       result="specOut">
>     <fePointLight x="-5000" y="-10000" z="20000"/>
>   </feSpecularLighting>
>   <feComposite in="specOut" in2="SourceAlpha" operator="in" result="specOut"/>
>   <feComposite in="SourceGraphic" in2="specOut" operator="arithmetic"
>                k1="0" k2="1" k3="1" k4="0" result="litPaint"/>
>   <feMerge>
>     <feMergeNode in="offsetBlur"/>
>     <feMergeNode in="litPaint"/>
>   </feMerge>
> </filter>
> ```
>
> The following pictures show the intermediate image results from each of the six filter elements:
>
> ![filters01 - original source graphic](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-0.png)
>
> Source graphic
>
> ![filters01 - after filter element 1](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-1.png)
>
> After filter primitive 1
>
> ![filters01 - after filter element 2](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-2.png)
>
> After filter primitive 2
>
> ![filters01 - after filter element 3](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-3.png)
>
> After filter primitive 3
>
> ![filters01 - after filter element 4](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-4.png)
>
> After filter primitive 4
>
> ![filters01 - after filter element 5](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-5.png)
>
> After filter primitive 5
>
> ![filters01 - after filter element 6](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filters01-6.png)
>
> After filter primitive 6
>
> 1.  <a id="ref-for-elementdef-fegaussianblur"></a>
>
>     <a id="ref-for-attr-valuedef-in-sourcealpha"></a>
>
>     Filter primitive [feGaussianBlur](#elementdef-fegaussianblur) takes input [SourceAlpha](#attr-valuedef-in-sourcealpha), which is the alpha channel of the source graphic. The result is stored in a temporary buffer named "blur". Note that "blur" is used as input to both filter primitives 2 and 3.
>
> 2.  <a id="ref-for-elementdef-feoffset"></a>
>
>     Filter primitive [feOffset](#elementdef-feoffset) takes buffer "blur", shifts the result in a positive direction in both x and y, and creates a new buffer named "offsetBlur". The effect is that of a drop shadow.
>
> 3.  <a id="ref-for-elementdef-fespecularlighting"></a>
>
>     Filter primitive [feSpecularLighting](#elementdef-fespecularlighting), uses buffer "blur" as a model of a surface elevation and generates a lighting effect from a single point source. The result is stored in buffer "specOut".
>
> 4.  <a id="ref-for-elementdef-fecomposite"></a>
>
>     Filter primitive [feComposite](#elementdef-fecomposite) masks out the result of filter primitive 3 by the original source graphics alpha channel so that the intermediate result is no bigger than the original source graphic.
>
> 5.  <a id="ref-for-elementdef-fecomposite①"></a>
>
>     Filter primitive [feComposite](#elementdef-fecomposite) composites the result of the specular lighting with the original source graphic.
>
> 6.  <a id="ref-for-elementdef-femerge"></a>
>
>     Filter primitive [feMerge](#elementdef-femerge) composites two layers together. The lower layer consists of the drop shadow result from filter primitive 2. The upper layer consists of the specular lighting result from filter primitive 5.

## <a id="placement"></a>2. Module interactions

<a id="ref-for-containing-block"></a>

<a id="ref-for-stacking-context"></a>

This specification defines a set of CSS properties that affect the visual rendering of elements to which those properties are applied; these effects are applied after elements have been sized and positioned according to the [Visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) from [\[CSS21\]](#biblio-css21). Some values of these properties result in the creation of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block), and/or the creation of a [stacking context](https://www.w3.org/TR/css3-positioning/#stacking-context).

<a id="ref-for-propdef-border"></a>

The compositing model follows the SVG compositing model [\[SVG11\]](#biblio-svg11): first any filter effect is applied, then any clipping, masking and opacity [\[CSS3COLOR\]](#biblio-css3color). These effects all apply after any other CSS effects such as [border](https://www.w3.org/TR/css3-background/#propdef-border) [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-propdef-color-interpolation-filters"></a>

<a id="ref-for-propdef-flood-color"></a>

<a id="ref-for-propdef-flood-opacity"></a>

<a id="ref-for-propdef-lighting-color"></a>

<a id="ref-for-elementdef-filter"></a>

<a id="ref-for-elementdef-femergenode"></a>

<a id="ref-for-transfer-function-element"></a>

<a id="ref-for-filter-primitive"></a>

Some property and element definitions in this specification require an SVG 1.1 implementation [\[SVG11\]](#biblio-svg11). UAs without support for SVG must not implement the [color-interpolation-filters](#propdef-color-interpolation-filters), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity) and [lighting-color](#propdef-lighting-color) properties as well as the [filter](#elementdef-filter) element, the [feMergeNode](#elementdef-femergenode) element, the [transfer function elements](#transfer-function-element) and the [filter primitive](#filter-primitive) elements.

## <a id="values"></a>3. Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#biblio-css21). Value types not defined in these specifications are defined in CSS Values and Units Module Level 3 [\[CSS3VAL\]](#biblio-css3val).

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [inherit](https://www.w3.org/TR/CSS21/cascade.html#value-def-inherit) keyword as their property value. For readability it has not been repeated explicitly.

## <a id="definitions"></a>4. Terminology

When used in this specification, terms have the meanings assigned in this section.

<a id="filter-primitive"></a>filter primitive, <a id="elementdef-filter-primitive"></a>`filter-primitive`  
<a id="ref-for-elementdef-filter①"></a>

<a id="ref-for-elementdef-fespotlight"></a>

<a id="ref-for-elementdef-feblend"></a>

<a id="ref-for-elementdef-fecolormatrix"></a>

<a id="ref-for-elementdef-fecomponenttransfer"></a>

<a id="ref-for-elementdef-fecomposite②"></a>

<a id="ref-for-elementdef-feconvolvematrix"></a>

<a id="ref-for-elementdef-fediffuselighting"></a>

<a id="ref-for-elementdef-fedisplacementmap"></a>

<a id="ref-for-elementdef-fedropshadow"></a>

<a id="ref-for-elementdef-feflood"></a>

<a id="ref-for-elementdef-fegaussianblur①"></a>

<a id="ref-for-elementdef-feimage"></a>

<a id="ref-for-elementdef-femerge①"></a>

<a id="ref-for-elementdef-femorphology"></a>

<a id="ref-for-elementdef-feoffset①"></a>

<a id="ref-for-elementdef-fespecularlighting①"></a>

<a id="ref-for-elementdef-fetile"></a>

<a id="ref-for-elementdef-feturbulence"></a>

The set of elements that control the output of a [filter](#elementdef-filter) element, particularly: [feSpotLight](#elementdef-fespotlight), [feBlend](#elementdef-feblend), [feColorMatrix](#elementdef-fecolormatrix), [feComponentTransfer](#elementdef-fecomponenttransfer), [feComposite](#elementdef-fecomposite), [feConvolveMatrix](#elementdef-feconvolvematrix), [feDiffuseLighting](#elementdef-fediffuselighting), [feDisplacementMap](#elementdef-fedisplacementmap), [feDropShadow](#elementdef-fedropshadow), [feFlood](#elementdef-feflood), [feGaussianBlur](#elementdef-fegaussianblur), [feImage](#elementdef-feimage), [feMerge](#elementdef-femerge), [feMorphology](#elementdef-femorphology), [feOffset](#elementdef-feoffset), [feSpecularLighting](#elementdef-fespecularlighting), [feTile](#elementdef-fetile), [feTurbulence](#elementdef-feturbulence).

<a id="pass-through-filter"></a>pass through filter  
The pass through filter output is equal to the primary input of the filter primitive.

<a id="ref-for-propdef-filter①"></a>

## <a id="FilterProperty"></a>5. Graphic filters: the [filter](#propdef-filter) property

<a id="ref-for-propdef-filter②"></a>

The description of the [filter](#propdef-filter) property is as follows:

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-filter"></a>filter

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-filter-value-list"></a>

<a id="ref-for-comb-one"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<filter-value-list\>](#typedef-filter-value-list)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-use"></a>

<a id="ref-for-graphics-element"></a>

<a id="ref-for-elementdef-defs"></a>

<a id="ref-for-container-element"></a>

All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/svg2/struct.html#container-element) without the [defs](https://www.w3.org/TR/svg2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/svg2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/svg2/struct.html#elementdef-use) element.

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Media:

<strong>Column 2 (data cell):</strong>

visual

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

See prose in [Animation of Filters](#animation-of-filters).

<a id="typedef-filter-value-list"></a>

<a id="ref-for-typedef-filter-function"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-typedef-filter-url"></a>

<a id="ref-for-mult-one-plus"></a>

```text
<filter-value-list> = [ <filter-function> | <url> ]+
```
<a id="typedef-filter-url"></a>\<url\>

<a id="ref-for-elementdef-filter②"></a>

A filter reference to a [filter](#elementdef-filter) element. For example url(commonfilters.svg#filter). If the filter references a non-existent object or the referenced object is not a <a id="ref-for-elementdef-filter③"></a>filter element, then the whole filter chain is ignored. No filter is applied to the object.

<a id="ref-for-typedef-filter-function①"></a>

[\<filter-function\>](#typedef-filter-function)

See [Filter Functions](#filter-functions).

none

No filter effect gets applied.

<a id="ref-for-propdef-filter③"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-browsing-context"></a>

A value other than none for the [filter](#propdef-filter) property results in the creation of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for absolute and fixed positioned descendants unless the element it applies to is a document root element in the current [browsing context](https://html.spec.whatwg.org/multipage/browsers.html#browsing-context). The list of functions are applied in the order provided.

<a id="ref-for-elementdef-filter④"></a>

<a id="ref-for-attr-valuedef-in-sourcegraphic"></a>

The first filter function or [filter](#elementdef-filter) reference in the list takes the element ([SourceGraphic](#attr-valuedef-in-sourcegraphic)) as the input image. Subsequent operations take the output from the previous filter function or <a id="ref-for-elementdef-filter⑤"></a>filter reference as the input image. <a id="ref-for-elementdef-filter⑥"></a>filter element reference functions can specify an alternate input, but still uses the previous output as its <a id="ref-for-attr-valuedef-in-sourcegraphic①"></a>SourceGraphic.

<a id="ref-for-propdef-color-interpolation-filters①"></a>

[color-interpolation-filters](#propdef-color-interpolation-filters) has no affect for Filter Functions. Filter Functions must operate in the sRGB color space.

<a id="ref-for-propdef-opacity"></a>

A computed value of other than none results in the creation of a [stacking context](https://www.w3.org/TR/CSS21/zindex.html) [\[CSS21\]](#biblio-css21) the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) does. All the elements descendants are rendered together as a group with the filter effect applied to the group as a whole.

<a id="ref-for-propdef-filter④"></a>

The [filter](#propdef-filter) property has no effect on the geometry of the target element’s CSS boxes, even though <a id="ref-for-propdef-filter⑤"></a>filter can cause painting outside of an element’s border box.

<a id="ref-for-local-coordinate-system"></a>

Conceptually, any parts of the drawing are effected by filter operations. This includes any content, background, borders, text decoration, outline and visible scrolling mechanism of the element to which the filter is applied, and those of its descendants. The filter operations are applied in the element’s [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system).

<a id="ref-for-propdef-filter⑥"></a>

The compositing model follows the [SVG compositing model](https://www.w3.org/TR/SVG11/render.html#Introduction) [\[SVG11\]](#biblio-svg11): first any filter effect is applied, then any clipping, masking and opacity. As per SVG, the application of [filter](#propdef-filter) has no effect on hit-testing.

<a id="ref-for-propdef-filter⑦"></a>

The [filter](#propdef-filter) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-95bcf4be"></a> How does filter behave on fixed background images? [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;238&#x3E;](https://github.com/w3c/csswg-drafts/issues/238)

## <a id="filter-functions"></a>6. Filter Functions

### <a id="supported-filter-functions"></a>6.1. Supported Filter Functions

<a id="typedef-filter-function"></a>

<a id="ref-for-funcdef-filter-blur"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-funcdef-filter-brightness"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-funcdef-filter-contrast"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-funcdef-filter-drop-shadow"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-funcdef-filter-grayscale①"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-funcdef-filter-hue-rotate"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-funcdef-filter-invert"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-funcdef-filter-opacity"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-funcdef-filter-sepia"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-funcdef-filter-saturate"></a>

```text
<filter-function> = <blur()> | <brightness()> | <contrast()> | <drop-shadow()> |    <grayscale()> | <hue-rotate()> | <invert()> | <opacity()> | <sepia()> | <saturate()>
```
<a id="ref-for-TermInitialValue"></a>

Unless defined otherwise, omitted values default to the [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation.

<a id="ref-for-TermInitialValue①"></a>

<a id="ref-for-funcdef-filter-grayscale②"></a>

<a id="ref-for-funcdef-filter-sepia①"></a>

<a id="ref-for-funcdef-filter-invert①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For some filter functions the default value for omitted values differes from their [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation. For the convenience of content creators, the default value for omitted values for [\<grayscale()\>](#funcdef-filter-grayscale), [\<sepia()\>](#funcdef-filter-sepia) and [\<invert()\>](#funcdef-filter-invert) is 1 (apply the effect to 100%) while the <a id="ref-for-TermInitialValue②"></a>initial value for interpolation is 0 (no effect).

<a id="FilterFunction"></a>

<a id="funcdef-filter-blur"></a>

<a id="ref-for-length-value"></a>

<a id="ref-for-mult-opt"></a>

```text
blur() = blur( <length>? )
```
Applies a Gaussian blur to the input image. The passed parameter defines the value of the standard deviation to the Gaussian function. The parameter is specified a CSS length, but does not accept percentage values. The markup equivalent of this function is [given below](#blurEquivalent).

Negative values are not allowed.

Default value when omitted is 0px.

<a id="ref-for-TermInitialValue③"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 0px.

<a id="ref-for-propdef-box-shadow"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Standard deviation is different to [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow) s blur radius.

<a id="funcdef-filter-brightness"></a>

<a id="ref-for-typedef-number-percentage"></a>

<a id="ref-for-mult-opt①"></a>

```text
brightness() = brightness( <number-percentage>? )
```
Applies a linear multiplier to input image, making it appear more or less bright. A value of 0% will create an image that is completely black. A value of 100% leaves the input unchanged. Other values are linear multipliers on the effect. Values of amount over 100% are allowed, providing brighter results. The markup equivalent of this function is [given below](#brightnessEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue④"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 1.

<a id="funcdef-filter-contrast"></a>

<a id="ref-for-typedef-number-percentage①"></a>

<a id="ref-for-mult-opt②"></a>

```text
contrast() = contrast( <number-percentage>? )
```
Adjusts the contrast of the input. A value of 0% will create an image that is completely gray. A value of 100% leaves the input unchanged. Values of amount over 100% are allowed, providing results with more contrast. The markup equivalent of this function is [given below](#contrastEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue⑤"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 1.

<a id="funcdef-filter-drop-shadow"></a>

<a id="ref-for-valuea-def-color"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-length-value①"></a>

<a id="ref-for-mult-num-range"></a>

```text
drop-shadow() = drop-shadow( <color>? && <length>{2,3} )
```
<a id="ref-for-propdef-box-shadow①"></a>

<a id="ref-for-length-value②"></a>

Applies a drop shadow effect to the input image. A drop shadow is effectively a blurred, offset version of the input image’s alpha mask drawn in a particular color, composited below the image. Values are interpreted as for [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow) [\[CSS3BG\]](#biblio-css3bg) but with the optional 3rd [\<length\>](https://www.w3.org/TR/css3-values/#length-value) value being the standard deviation instead of blur radius. The markup equivalent of this function is [given below](#dropshadowEquivalent).

The default value for omitted values is missing length values set to 0 and the missing used color is taken from the color property.

<a id="ref-for-TermInitialValue⑥"></a>

<a id="ref-for-valdef-color-transparent"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is all length values set to 0 and the used color set to [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Spread values or multiple shadows are not accepted for this level of the specification.

<a id="ref-for-propdef-box-shadow②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Standard deviation is different to [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow) s blur radius.

<a id="funcdef-filter-grayscale"></a>

<a id="ref-for-typedef-number-percentage②"></a>

<a id="ref-for-mult-opt④"></a>

```text
grayscale() = grayscale( <number-percentage>? )
```
Converts the input image to grayscale. The passed parameter defines the proportion of the conversion. A value of 100% is completely grayscale. A value of 0% leaves the input unchanged. Values between 0% and 100% are linear multipliers on the effect. Values of amount over 100% are allowed but UAs must clamp the values to 1. The markup equivalent of this function is [given below](#grayscaleEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue⑦"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 0.

<a id="funcdef-filter-hue-rotate"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-zero-value"></a>

<a id="ref-for-mult-opt⑤"></a>

```text
hue-rotate() = hue-rotate( [ <angle> | <zero> ]? )
```
Applies a hue rotation on the input image. The passed parameter defines the number of degrees around the color circle the input samples will be adjusted. A value of 0deg leaves the input unchanged. Implementations must not normalize this value in order to allow animations beyond 360deg. The markup equivalent of this function is [given below](#huerotateEquivalent).

<a id="ref-for-angle-value①"></a>

The unit identifier may be omitted if the [\<angle\>](https://www.w3.org/TR/css3-values/#angle-value) is zero.

Default value when omitted is 0deg.

<a id="ref-for-TermInitialValue⑧"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 0deg.

<a id="funcdef-filter-invert"></a>

<a id="ref-for-typedef-number-percentage③"></a>

<a id="ref-for-mult-opt⑥"></a>

```text
invert() = invert( <number-percentage>? )
```
Inverts the samples in the input image. The passed parameter defines the proportion of the conversion. A value of 100% is completely inverted. A value of 0% leaves the input unchanged. Values between 0% and 100% are linear multipliers on the effect. Values of amount over 100% are allowed but UAs must clamp the values to 1. The markup equivalent of this function is [given below](#invertEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue⑨"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 0.

<a id="funcdef-filter-opacity"></a>

<a id="ref-for-typedef-number-percentage④"></a>

<a id="ref-for-mult-opt⑦"></a>

```text
opacity() = opacity( <number-percentage>? )
```
Applies transparency to the samples in the input image. The passed parameter defines the proportion of the conversion. A value of 0% is completely transparent. A value of 100% leaves the input unchanged. Values between 0% and 100% are linear multipliers on the effect. This is equivalent to multiplying the input image samples by amount. Values of amount over 100% are allowed but UAs must clamp the values to 1. The markup equivalent of this function is [given below](#opacityEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue①⓪"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 1.

<a id="ref-for-propdef-opacity①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The opacity filter function is not meant to be a shorthand of the [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) property. Furthermore, it allows setting the transparency of intermediate filter primitive results before passing to the next filter primitive. If the opacity filter function is set as last filter primitive, the value of the <a id="ref-for-propdef-opacity②"></a>opacity property is multiplied on top of the value of the filter function, which may result in a more transparent content.

<a id="funcdef-filter-saturate"></a>

<a id="ref-for-typedef-number-percentage⑤"></a>

<a id="ref-for-mult-opt⑧"></a>

```text
saturate() = saturate( <number-percentage>? )
```
Saturates the input image. The passed parameter defines the proportion of the conversion. A value of 0% is completely un-saturated. A value of 100% leaves the input unchanged. Other values are linear multipliers on the effect. Values of amount over 100% are allowed, providing super-saturated results. The markup equivalent of this function is [given below](#saturateEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue①①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 1.

<a id="funcdef-filter-sepia"></a>

<a id="ref-for-typedef-number-percentage⑥"></a>

<a id="ref-for-mult-opt⑨"></a>

```text
sepia() = sepia( <number-percentage>? )
```
Converts the input image to sepia. The passed parameter defines the proportion of the conversion. A value of 100% is completely sepia. A value of 0% leaves the input unchanged. Values between 0% and 100% are linear multipliers on the effect. Values of amount over 100% are allowed but UAs must clamp the values to 1. The markup equivalent of this function is [given below](#sepiaEquivalent).

Negative values are not allowed.

Default value when omitted is 1.

<a id="ref-for-TermInitialValue①②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for interpolation is 0.

### <a id="computed-values-of-filter-functions"></a>6.2. Computed Values of Filter Functions

<a id="ref-for-typedef-filter-function②"></a>

The values in a [\<filter-function\>](#typedef-filter-function) are computed as specified, with these exceptions:

- Omitted values are included and compute to their defaults.

- <a id="ref-for-funcdef-filter-drop-shadow①"></a>

  <a id="ref-for-valuea-def-color①"></a>

  <a id="ref-for-length-value③"></a>

  [\<drop-shadow()\>](#funcdef-filter-drop-shadow) starts with the computed value of [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) followed by the computed value of the [\<length\>](https://www.w3.org/TR/css3-values/#length-value) values.

### <a id="serialization-of-filter-functions"></a>6.3. Serialization of Filter Functions

<a id="ref-for-typedef-filter-function③"></a>

<a id="ref-for-funcdef-calc"></a>

To serialize the [\<filter-function\>](#typedef-filter-function), serialize as per their individual grammars, in the order the grammars are written in, avoiding [\<calc()\>](https://www.w3.org/TR/css-values-4/#funcdef-calc) expressions where possible, serialize filter arguments as specified, avoiding <a id="ref-for-funcdef-calc①"></a>\<calc()\> transformations, joining space-separated tokens with a single space, and following each serialized comma with a single space.

### <a id="interpolation-of-filter-functions"></a>6.4. Interpolation of Filter Functions

<a id="ref-for-typedef-filter-function④"></a>

For interpolation of values in [\<filter-function\>](#typedef-filter-function)s, the steps corresponding to the first matching condition in the following list must be run:

<a id="ref-for-funcdef-filter-blur①"></a>

[\<blur()\>](#funcdef-filter-blur)

Interpolate values as length [by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value).

<a id="ref-for-funcdef-filter-brightness①"></a>

[\<brightness()\>](#funcdef-filter-brightness)

<a id="ref-for-funcdef-filter-contrast①"></a>

[\<contrast()\>](#funcdef-filter-contrast)

<a id="ref-for-funcdef-filter-grayscale③"></a>

[\<grayscale()\>](#funcdef-filter-grayscale)

<a id="ref-for-funcdef-filter-invert②"></a>

[\<invert()\>](#funcdef-filter-invert)

<a id="ref-for-funcdef-filter-opacity①"></a>

[\<opacity()\>](#funcdef-filter-opacity)

<a id="ref-for-funcdef-filter-saturate①"></a>

[\<saturate()\>](#funcdef-filter-saturate)

<a id="ref-for-funcdef-filter-sepia②"></a>

[\<sepia()\>](#funcdef-filter-sepia)

Convert percentage values to numbers with 0% being relative to 0 and 100% relative to 1. Interpolate values as number [by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value) .

<a id="ref-for-funcdef-filter-hue-rotate①"></a>

[\<hue-rotate()\>](#funcdef-filter-hue-rotate)

Interpolate values as number [by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value).

<a id="ref-for-funcdef-filter-drop-shadow②"></a>

[\<drop-shadow()\>](#funcdef-filter-drop-shadow)

Interpolate values as shadow list as [repeatable list](https://drafts.csswg.org/web-animations-1/#repeatable-list).

<a id="ref-for-elementdef-filter⑦"></a>

## <a id="FilterElement"></a>7. SVG Filter Sources: the [filter](#elementdef-filter) element

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-filter"></a>`filter`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

None.

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

Any number of the following elements, in any order:

- <a id="ref-for-elementdef-metadata"></a>

  <a id="ref-for-elementdef-title"></a>

  <a id="ref-for-elementdef-desc"></a>

  [descriptive](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermDescriptiveElement) — [desc](https://www.w3.org/TR/svg2/struct.html#elementdef-desc), [title](https://www.w3.org/TR/svg2/struct.html#elementdef-title), [metadata](https://www.w3.org/TR/svg2/struct.html#elementdef-metadata)

- <a id="ref-for-elementdef-feturbulence①"></a>

  <a id="ref-for-elementdef-fetile①"></a>

  <a id="ref-for-elementdef-fespecularlighting②"></a>

  <a id="ref-for-elementdef-feoffset②"></a>

  <a id="ref-for-elementdef-femorphology①"></a>

  <a id="ref-for-elementdef-femerge②"></a>

  <a id="ref-for-elementdef-feimage①"></a>

  <a id="ref-for-elementdef-fegaussianblur②"></a>

  <a id="ref-for-elementdef-fedropshadow①"></a>

  <a id="ref-for-elementdef-fedisplacementmap①"></a>

  <a id="ref-for-elementdef-fediffuselighting①"></a>

  <a id="ref-for-elementdef-feconvolvematrix①"></a>

  <a id="ref-for-elementdef-fecomposite③"></a>

  <a id="ref-for-elementdef-fecomponenttransfer①"></a>

  <a id="ref-for-elementdef-fecolormatrix①"></a>

  <a id="ref-for-elementdef-feflood①"></a>

  <a id="ref-for-elementdef-feblend①"></a>

  <a id="ref-for-elementdef-filter-primitive"></a>

  [filter primitive](#elementdef-filter-primitive) — [feBlend](#elementdef-feblend), [feFlood](#elementdef-feflood), [feColorMatrix](#elementdef-fecolormatrix), [feComponentTransfer](#elementdef-fecomponenttransfer), [feComposite](#elementdef-fecomposite), [feConvolveMatrix](#elementdef-feconvolvematrix), [feDiffuseLighting](#elementdef-fediffuselighting), [feDisplacementMap](#elementdef-fedisplacementmap), [feDropShadow](#elementdef-fedropshadow), [feGaussianBlur](#elementdef-fegaussianblur), [feImage](#elementdef-feimage), [feMerge](#elementdef-femerge), [feMorphology](#elementdef-femorphology), [feOffset](#elementdef-feoffset), [feSpecularLighting](#elementdef-fespecularlighting), [feTile](#elementdef-fetile), [feTurbulence](#elementdef-feturbulence)

- <a id="ref-for-AnimateElement"></a>

  [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement)

- <a id="ref-for-elementdef-script"></a>

  [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script)

- <a id="ref-for-SetElement"></a>

  [set](https://www.w3.org/TR/SVG11/animate.html#SetElement)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode"></a>

  <a id="ref-for-propdef-word-spacing"></a>

  <a id="ref-for-propdef-visibility"></a>

  <a id="ref-for-propdef-unicode-bidi"></a>

  <a id="ref-for-TextRenderingProperty"></a>

  <a id="ref-for-propdef-text-decoration"></a>

  <a id="ref-for-TextAnchorProperty"></a>

  <a id="ref-for-StrokeWidthProperty"></a>

  <a id="ref-for-StrokeOpacityProperty"></a>

  <a id="ref-for-StrokeMiterlimitProperty"></a>

  <a id="ref-for-StrokeLinejoinProperty"></a>

  <a id="ref-for-StrokeLinecapProperty"></a>

  <a id="ref-for-StrokeDashoffsetProperty"></a>

  <a id="ref-for-StrokeDasharrayProperty"></a>

  <a id="ref-for-StrokeProperty"></a>

  <a id="ref-for-StopOpacityProperty"></a>

  <a id="ref-for-StopColorProperty"></a>

  <a id="ref-for-ShapeRenderingProperty"></a>

  <a id="ref-for-PointerEventsProperty"></a>

  <a id="ref-for-propdef-overflow"></a>

  <a id="ref-for-propdef-opacity③"></a>

  <a id="ref-for-propdef-mask"></a>

  <a id="ref-for-MarkerStartProperty"></a>

  <a id="ref-for-MarkerMidProperty"></a>

  <a id="ref-for-MarkerEndProperty"></a>

  <a id="ref-for-MarkerProperty"></a>

  <a id="ref-for-propdef-lighting-color①"></a>

  <a id="ref-for-propdef-letter-spacing"></a>

  <a id="ref-for-KerningProperty"></a>

  <a id="ref-for-propdef-isolation"></a>

  <a id="ref-for-propdef-image-rendering"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty"></a>

  <a id="ref-for-propdef-font-weight"></a>

  <a id="ref-for-propdef-font-variant"></a>

  <a id="ref-for-propdef-font-style"></a>

  <a id="ref-for-propdef-font-stretch"></a>

  <a id="ref-for-propdef-font-size-adjust"></a>

  <a id="ref-for-propdef-font-size"></a>

  <a id="ref-for-propdef-font-family"></a>

  <a id="ref-for-propdef-font"></a>

  <a id="ref-for-propdef-flood-opacity①"></a>

  <a id="ref-for-propdef-flood-color①"></a>

  <a id="ref-for-propdef-filter⑧"></a>

  <a id="ref-for-FillRuleProperty"></a>

  <a id="ref-for-FillOpacityProperty"></a>

  <a id="ref-for-FillProperty"></a>

  <a id="ref-for-EnableBackgroundProperty"></a>

  <a id="ref-for-DominantBaselineProperty"></a>

  <a id="ref-for-propdef-display"></a>

  <a id="ref-for-propdef-direction"></a>

  <a id="ref-for-propdef-cursor"></a>

  <a id="ref-for-ColorRenderingProperty"></a>

  <a id="ref-for-propdef-color-interpolation-filters②"></a>

  <a id="ref-for-ColorInterpolationProperty"></a>

  <a id="ref-for-color0"></a>

  <a id="ref-for-propdef-clip-rule"></a>

  <a id="ref-for-propdef-clip-path"></a>

  <a id="ref-for-propdef-clip"></a>

  <a id="ref-for-BaselineShiftProperty"></a>

  <a id="ref-for-AlignmentBaselineProperty"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- [externalResourcesRequired](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#ExternalResourcesRequiredAttribute)

- <a id="ref-for-element-attrdef-filter-x"></a>

  [x](#element-attrdef-filter-x)

- <a id="ref-for-element-attrdef-filter-y"></a>

  [y](#element-attrdef-filter-y)

- <a id="ref-for-element-attrdef-filter-width"></a>

  [width](#element-attrdef-filter-width)

- <a id="ref-for-element-attrdef-filter-height"></a>

  [height](#element-attrdef-filter-height)

- <a id="ref-for-element-attrdef-filter-filterunits"></a>

  [filterUnits](#element-attrdef-filter-filterunits)

- <a id="ref-for-element-attrdef-filter-primitiveunits"></a>

  [primitiveUnits](#element-attrdef-filter-primitiveunits)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFilterElement](#InterfaceSVGFilterElement)

<a id="ref-for-elementdef-filter⑧"></a>

The description of the [filter](#elementdef-filter) element follows:

<a id="ref-for-number-value"></a>

<a id="typedef-number-optional-number"></a>\<number-optional-number\> = [\<number\>](https://www.w3.org/TR/css3-values/#number-value) <a id="ref-for-number-value①"></a>\<number\>?

<em>Attribute definitions:</em>

<a id="element-attrdef-filter-filterunits"></a>`filterUnits` = "<a id="attr-valuedef-filterunits-userspaceonuse"></a>`userSpaceOnUse` \| <a id="attr-valuedef-filterunits-objectboundingbox"></a>`objectBoundingBox`"

<a id="ref-for-filter-region"></a>

See [filter region](#filter-region).

<a id="element-attrdef-filter-primitiveunits"></a>`primitiveUnits` = "<a id="attr-valuedef-primitiveunits-userspaceonuse"></a>`userSpaceOnUse` \| <a id="attr-valuedef-primitiveunits-objectboundingbox"></a>`objectBoundingBox`"

<a id="ref-for-filter-primitive①"></a>

<a id="ref-for-filter-primitive-subregion"></a>

Specifies the coordinate system for the various length values within the [filter primitives](#filter-primitive) and for the attributes that define the [filter primitive subregion](#filter-primitive-subregion).

<a id="ref-for-element-attrdef-filter-primitiveunits①"></a>

<a id="ref-for-attr-valuedef-primitiveunits-userspaceonuse"></a>

<a id="ref-for-local-coordinate-system①"></a>

<a id="ref-for-elementdef-filter⑨"></a>

<a id="ref-for-propdef-filter⑨"></a>

If [primitiveUnits](#element-attrdef-filter-primitiveunits) is equal to [userSpaceOnUse](#attr-valuedef-primitiveunits-userspaceonuse), any length values within the filter definitions represent values in the current [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system) in place at the time when the [filter](#elementdef-filter) element is referenced (i.e., the user coordinate system for the element referencing the <a id="ref-for-elementdef-filter①⓪"></a>filter element via a [filter](#propdef-filter) property).

<a id="ref-for-element-attrdef-filter-primitiveunits②"></a>

<a id="ref-for-attr-valuedef-primitiveunits-objectboundingbox"></a>

<a id="ref-for-ObjectBoundingBoxUnits"></a>

<a id="ref-for-typedef-number-optional-number"></a>

If [primitiveUnits](#element-attrdef-filter-primitiveunits) is equal to [objectBoundingBox](#attr-valuedef-primitiveunits-objectboundingbox), then any length values within the filter definitions represent fractions or percentages of the bounding box on the referencing element (see [object bounding box units](https://svgwg.org/svg2-draft/coords.html#ObjectBoundingBoxUnits)). Note that if only one number was specified in a [\<number-optional-number\>](#typedef-number-optional-number) value this number is expanded out before the <a id="ref-for-element-attrdef-filter-primitiveunits③"></a>primitiveUnits computation takes place.

<a id="ref-for-TermInitialValue①③"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits④"></a>

<a id="ref-for-attr-valuedef-primitiveunits-userspaceonuse①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [primitiveUnits](#element-attrdef-filter-primitiveunits) is [userSpaceOnUse](#attr-valuedef-primitiveunits-userspaceonuse).

Animatable: yes.

<a id="ref-for-typedef-length-percentage"></a>

<a id="element-attrdef-filter-x"></a>`x` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-region①"></a>

See [filter region](#filter-region).

<a id="ref-for-typedef-length-percentage①"></a>

<a id="element-attrdef-filter-y"></a>`y` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-region②"></a>

See [filter region](#filter-region).

<a id="ref-for-typedef-length-percentage②"></a>

<a id="element-attrdef-filter-width"></a>`width` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-region③"></a>

See [filter region](#filter-region).

<a id="ref-for-typedef-length-percentage③"></a>

<a id="element-attrdef-filter-height"></a>`height` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-region④"></a>

See [filter region](#filter-region).

<a id="ref-for-typedef-number-optional-number①"></a>

<a id="element-attrdef-filter-filterres"></a>`filterRes` = "[\<number-optional-number\>](#typedef-number-optional-number)"

The <em>filterRes</em> attribute was removed from the specification. See SVG 1.1 specification for the defintion [\[SVG11\]](#biblio-svg11).

<a id="ref-for-elementdef-filter①①"></a>

Properties inherit into the [filter](#elementdef-filter) element from its ancestors; properties do <em>not</em> inherit from the element referencing the <a id="ref-for-elementdef-filter①②"></a>filter element.

<a id="ref-for-elementdef-filter①③"></a>

<a id="ref-for-propdef-filter①⓪"></a>

<a id="ref-for-propdef-display①"></a>

[filter](#elementdef-filter) elements are never rendered directly; their only usage is as something that can be referenced using the [filter](#propdef-filter) property. The [display](https://www.w3.org/TR/css-display-3/#propdef-display) property does not apply to the <a id="ref-for-elementdef-filter①④"></a>filter element; thus, <a id="ref-for-elementdef-filter①⑤"></a>filter elements are not directly rendered even if the <a id="ref-for-propdef-display②"></a>display property is set to a value other than none, and <a id="ref-for-elementdef-filter①⑥"></a>filter elements are available for referencing even when the <a id="ref-for-propdef-display③"></a>display property on the <a id="ref-for-elementdef-filter①⑦"></a>filter element or any of its ancestors is set to none.

## <a id="FilterEffectsRegion"></a>8. Filter Region

<a id="ref-for-elementdef-filter①⑧"></a>

<a id="ref-for-filter-primitive②"></a>

A [filter](#elementdef-filter) element can define a <a id="filter-region"></a>filter region on the canvas to which a given filter effect applies and can provide a resolution for any intermediate continuous tone images used to process any raster-based [filter primitives](#filter-primitive). The <a id="ref-for-elementdef-filter①⑨"></a>filter element has the following attributes which work together to define the filter region:

<a id="ref-for-element-attrdef-filter-filterunits①"></a>

[filterUnits](#element-attrdef-filter-filterunits)

<a id="ref-for-element-attrdef-filter-x①"></a>

<a id="ref-for-element-attrdef-filter-y①"></a>

<a id="ref-for-element-attrdef-filter-width①"></a>

<a id="ref-for-element-attrdef-filter-height①"></a>

Defines the coordinate system for attributes [x](#element-attrdef-filter-x), [y](#element-attrdef-filter-y), [width](#element-attrdef-filter-width), [height](#element-attrdef-filter-height).

<a id="ref-for-element-attrdef-filter-filterunits②"></a>

<a id="ref-for-attr-valuedef-filterunits-userspaceonuse"></a>

<a id="ref-for-element-attrdef-filter-x②"></a>

<a id="ref-for-element-attrdef-filter-y②"></a>

<a id="ref-for-element-attrdef-filter-width②"></a>

<a id="ref-for-element-attrdef-filter-height②"></a>

<a id="ref-for-elementdef-filter②⓪"></a>

<a id="ref-for-propdef-filter①①"></a>

If [filterUnits](#element-attrdef-filter-filterunits) is equal to [userSpaceOnUse](#attr-valuedef-filterunits-userspaceonuse), [x](#element-attrdef-filter-x), [y](#element-attrdef-filter-y), [width](#element-attrdef-filter-width), [height](#element-attrdef-filter-height) represent values in the current user coordinate system in place at the time when the [filter](#elementdef-filter) element is referenced (i.e., the user coordinate system for the element referencing the <a id="ref-for-elementdef-filter②①"></a>filter element via a [filter](#propdef-filter) property).

<a id="ref-for-element-attrdef-filter-filterunits③"></a>

<a id="ref-for-attr-valuedef-filterunits-objectboundingbox"></a>

<a id="ref-for-element-attrdef-filter-x③"></a>

<a id="ref-for-element-attrdef-filter-y③"></a>

<a id="ref-for-element-attrdef-filter-width③"></a>

<a id="ref-for-element-attrdef-filter-height③"></a>

<a id="ref-for-ObjectBoundingBoxUnits①"></a>

If [filterUnits](#element-attrdef-filter-filterunits) is equal to [objectBoundingBox](#attr-valuedef-filterunits-objectboundingbox), then [x](#element-attrdef-filter-x), [y](#element-attrdef-filter-y), [width](#element-attrdef-filter-width), [height](#element-attrdef-filter-height) represent fractions or percentages of the bounding box on the referencing element (see [object bounding box units](https://svgwg.org/svg2-draft/coords.html#ObjectBoundingBoxUnits)).

<a id="ref-for-TermInitialValue①④"></a>

<a id="ref-for-element-attrdef-filter-filterunits④"></a>

<a id="ref-for-attr-valuedef-filterunits-objectboundingbox①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [filterUnits](#element-attrdef-filter-filterunits) is [objectBoundingBox](#attr-valuedef-filterunits-objectboundingbox).

Animatable: yes.

<a id="ref-for-element-attrdef-filter-height④"></a>

<a id="ref-for-element-attrdef-filter-width④"></a>

<a id="ref-for-element-attrdef-filter-y④"></a>

<a id="ref-for-element-attrdef-filter-x④"></a>

[x](#element-attrdef-filter-x), [y](#element-attrdef-filter-y), [width](#element-attrdef-filter-width), [height](#element-attrdef-filter-height)

These attributes define a rectangular region on the canvas to which this filter applies.

<a id="ref-for-element-attrdef-filter-filterunits⑤"></a>

The coordinate system for these attributes depends on the value for attribute [filterUnits](#element-attrdef-filter-filterunits).

<a id="ref-for-filter-primitive③"></a>

<a id="ref-for-elementdef-filter②②"></a>

<a id="ref-for-elementdef-fegaussianblur③"></a>

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation"></a>

The bounds of this rectangle act as a hard clipping region for each [filter primitive](#filter-primitive) included with a given [filter](#elementdef-filter) element; thus, if the effect of a given filter primitive would extend beyond the bounds of the rectangle (this sometimes happens when using a [feGaussianBlur](#elementdef-fegaussianblur) filter primitive with a very large [stdDeviation](#element-attrdef-fegaussianblur-stddeviation)), parts of the effect will get clipped.

<a id="ref-for-TermInitialValue①⑤"></a>

<a id="ref-for-element-attrdef-filter-x⑤"></a>

<a id="ref-for-element-attrdef-filter-y⑤"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [x](#element-attrdef-filter-x) and [y](#element-attrdef-filter-y) is -10%.

<a id="ref-for-TermInitialValue①⑥"></a>

<a id="ref-for-element-attrdef-filter-width⑤"></a>

<a id="ref-for-element-attrdef-filter-height⑤"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [width](#element-attrdef-filter-width) and [height](#element-attrdef-filter-height) is 120%.

ng of the element which referenced the filter.

Animatable: yes.

<a id="ref-for-element-attrdef-filter-filterunits⑥"></a>

<a id="ref-for-attr-valuedef-filterunits-objectboundingbox②"></a>

<a id="ref-for-attr-valuedef-filterunits-userspaceonuse①"></a>

<a id="ref-for-filter-region⑤"></a>

<a id="ref-for-local-coordinate-system②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Both of the two possible value for [filterUnits](#element-attrdef-filter-filterunits) (i.e., [objectBoundingBox](#attr-valuedef-filterunits-objectboundingbox) and [userSpaceOnUse](#attr-valuedef-filterunits-userspaceonuse)) result in a [filter region](#filter-region) whose coordinate system has its X-axis and Y-axis each parallel to the X-axis and Y-axis, respectively, of the [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system) for the element to which the filter will be applied.

<a id="ref-for-filter-region⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Sometimes implementers can achieve faster performance when the filter region can be mapped directly to device pixels; thus, for best performance on display devices, it is suggested that authors define their region such that the user agent can align the [filter region](#filter-region) pixel-for-pixel with the background. In particular, for best filter effects performance, avoid rotating or skewing the user coordinate system.

<a id="ref-for-bounding-box"></a>

<a id="ref-for-element-attrdef-filter-x⑥"></a>

<a id="ref-for-element-attrdef-filter-y⑥"></a>

<a id="ref-for-element-attrdef-filter-width⑥"></a>

<a id="ref-for-element-attrdef-filter-height⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is often necessary to provide padding space because the filter effect might impact bits slightly outside the tight-fitting [bounding box](https://www.w3.org/TR/svg2/coords.html#bounding-box) on a given object. For these purposes, it is possible to provide negative percentage values for [x](#element-attrdef-filter-x), [y](#element-attrdef-filter-y) and percentages values greater than 100% for [width](#element-attrdef-filter-width), [height](#element-attrdef-filter-height). This, for example, is why the defaults for the filter region are `x="-10%" y="-10%" width="120%" height="120%"`.

## <a id="FilterPrimitivesOverview"></a>9. Filter primitives

### <a id="FilterPrimitivesOverviewIntro"></a>9.1. Overview

This section describes the various filter primitives that can be assembled to achieve a particular filter effect.

<a id="ref-for-elementdef-fecolormatrix②"></a>

<a id="ref-for-elementdef-fecomponenttransfer②"></a>

Unless otherwise stated, all image filters operate on premultiplied RGBA samples. Some filters like [feColorMatrix](#elementdef-fecolormatrix) and [feComponentTransfer](#elementdef-fecomponenttransfer) work more naturally on non-premultiplied data. For the time of the filter operation, all color values must temporarily be transformed to the required color multiplication of the current filter.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All input images are assumed to be in premultiplied RGBA. User agents may optimize performance by using non-premultiplied data buffering.

All raster effect filtering operations take 1 to N input RGBA images, additional attributes as parameters, and produce a single output RGBA image.

<a id="ref-for-filter-primitive④"></a>

The RGBA result from each filter primitive will be clamped into the allowable ranges for colors and opacity values. Thus, for example, the result from a given [filter primitive](#filter-primitive) will have any negative color values or opacity values adjusted up to color/opacity of zero.

<a id="ref-for-filter-primitive⑤"></a>

<a id="ref-for-propdef-color-interpolation-filters③"></a>

<a id="ref-for-ColorInterpolationProperty①"></a>

<a id="ref-for-valdef-color-interpolation-filters-linearrgb"></a>

<a id="ref-for-valdef-color-interpolation-filters-srgb"></a>

<a id="filtersColorSpace"></a>The color space in which a particular [filter primitive](#filter-primitive) performs its operations is determined by the value of the property [color-interpolation-filters](#propdef-color-interpolation-filters) on the given <a id="ref-for-filter-primitive⑥"></a>filter primitive. A different property, [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty) determines the color space for other color operations. Because these two properties have different initial values (<a id="ref-for-propdef-color-interpolation-filters④"></a>color-interpolation-filters has an initial value of [linearRGB](#valdef-color-interpolation-filters-linearrgb) whereas <a id="ref-for-ColorInterpolationProperty②"></a>color-interpolation has an initial value of [sRGB](#valdef-color-interpolation-filters-srgb)), in some cases to achieve certain results (e.g., when coordinating gradient interpolation with a filtering operation) it will be necessary to explicitly set <a id="ref-for-ColorInterpolationProperty③"></a>color-interpolation to <a id="ref-for-valdef-color-interpolation-filters-linearrgb①"></a>linearRGB or <a id="ref-for-propdef-color-interpolation-filters⑤"></a>color-interpolation-filters to <a id="ref-for-valdef-color-interpolation-filters-srgb①"></a>sRGB on particular elements. Note that the examples below do not explicitly set either <a id="ref-for-ColorInterpolationProperty④"></a>color-interpolation or <a id="ref-for-propdef-color-interpolation-filters⑥"></a>color-interpolation-filters, so the initial values for these properties apply to the examples.

<a id="ref-for-filter-primitive⑦"></a>

<a id="ref-for-elementdef-feoffset③"></a>

Sometimes [filter primitives](#filter-primitive) result in undefined pixels. For example, filter primitive [feOffset](#elementdef-feoffset) can shift an image down and to the right, leaving undefined pixels at the top and left. In these cases, the undefined pixels are set to transparent black.

<a id="ref-for-element-attrdef-filter-primitiveunits⑤"></a>

<a id="ref-for-operating-coordinate-space"></a>

To provide high quality rendering, all filter primitives should operate in a device dependent coordinate space, the <a id="operating-coordinate-space"></a>operating coordinate space, taking device pixel density, user space transformations and zooming into account. To provide a platform independent alignment, attribute and property values are often relative to a coordinate system described by the [primitiveUnits](#element-attrdef-filter-primitiveunits) attribute. User agents must scale these relative attributes and properties to the [operating coordinate space](#operating-coordinate-space).

<a id="ref-for-element-attrdef-filter-primitiveunits⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: On high resolution devices, attribute and property values that are relative to the [primitiveUnits](#element-attrdef-filter-primitiveunits) usually need to be scaled up. User agents may reduce the resolution of filter primitives on limited platform resources.

<a id="ref-for-elementdef-feconvolvematrix②"></a>

<a id="ref-for-light-source"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits⑦"></a>

<a id="ref-for-operating-coordinate-space①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some attribute or property values from the filter primitives [feConvolveMatrix](#elementdef-feconvolvematrix) and [light sources](#light-source) can not be mapped from the coordinate space defined by the [primitiveUnits](#element-attrdef-filter-primitiveunits) attribute to the [operating coordinate space](#operating-coordinate-space).

### <a id="CommonAttributes"></a>9.2. Common filter primitive attributes

The following <a id="filter-primitive-attributes"></a>filter primitive attributes are available for all filter primitives:

<em>Attribute definitions:</em>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="element-attrdef-filter-primitive-x"></a>`x` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-primitive⑧"></a>

<a id="ref-for-filter-primitive-subregion①"></a>

The minimum x coordinate for the subregion which restricts calculation and rendering of the given [filter primitive](#filter-primitive). See [filter primitive subregion](#filter-primitive-subregion).

<a id="ref-for-TermInitialValue①⑦"></a>

<a id="ref-for-element-attrdef-filter-primitive-x"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [x](#element-attrdef-filter-primitive-x) is 0%.

Animatable: yes.

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="element-attrdef-filter-primitive-y"></a>`y` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-primitive⑨"></a>

<a id="ref-for-filter-primitive-subregion②"></a>

The minimum y coordinate for the subregion which restricts calculation and rendering of the given [filter primitive](#filter-primitive). See [filter primitive subregion](#filter-primitive-subregion).

<a id="ref-for-TermInitialValue①⑧"></a>

<a id="ref-for-element-attrdef-filter-primitive-y"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [y](#element-attrdef-filter-primitive-y) is 0%.

Animatable: yes.

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="element-attrdef-filter-primitive-width"></a>`width` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-primitive①⓪"></a>

<a id="ref-for-filter-primitive-subregion③"></a>

The width of the subregion which restricts calculation and rendering of the given [filter primitive](#filter-primitive). See [filter primitive subregion](#filter-primitive-subregion).

A negative or zero value disables the effect of the given filter primitive (i.e., the result is a transparent black image).

<a id="ref-for-TermInitialValue①⑨"></a>

<a id="ref-for-element-attrdef-filter-primitive-width"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [width](#element-attrdef-filter-primitive-width) is 100%.

Animatable: yes.

<a id="ref-for-typedef-length-percentage⑦"></a>

<a id="element-attrdef-filter-primitive-height"></a>`height` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-filter-primitive①①"></a>

<a id="ref-for-filter-primitive-subregion④"></a>

The height of the subregion which restricts calculation and rendering of the given [filter primitive](#filter-primitive). See [filter primitive subregion](#filter-primitive-subregion).

A negative or zero value must disable the effect of the given filter primitive (i.e., the result is a transparent black image).

<a id="ref-for-TermInitialValue②⓪"></a>

<a id="ref-for-element-attrdef-filter-primitive-height"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [height](#element-attrdef-filter-primitive-height) is 100%.

Animatable: yes.

<a id="element-attrdef-filter-primitive-result"></a>`result` = "<em><dfn><span><a id="typedef-result-filter-primitive-reference"></a></span>&lt;filter-primitive-reference&gt;</dfn></em>"

<a id="ref-for-typedef-result-filter-primitive-reference"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-filter-primitive①②"></a>

<a id="ref-for-element-attrdef-filter-primitive-in"></a>

<a id="ref-for-elementdef-filter②③"></a>

[\<filter-primitive-reference\>](#typedef-result-filter-primitive-reference) is an [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) [\[CSS3VAL\]](#biblio-css3val) and an assigned name for this [filter primitive](#filter-primitive). If supplied, then graphics that result from processing this <a id="ref-for-filter-primitive①③"></a>filter primitive can be referenced by an [in](#element-attrdef-filter-primitive-in) attribute on a subsequent filter primitive within the same [filter](#elementdef-filter) element. If no value is provided, the output will only be available for re-use as the implicit input into the next <a id="ref-for-filter-primitive①④"></a>filter primitive if that <a id="ref-for-filter-primitive①⑤"></a>filter primitive provides no value for its <a id="ref-for-element-attrdef-filter-primitive-in①"></a>in attribute.

Most filter primitives take other filter primitives as input. The following attribute is representative for all input attributes to reference other filter primitives:

<em>Attribute definitions:</em>

<a id="ref-for-typedef-result-filter-primitive-reference①"></a>

<a id="ref-for-attr-valuedef-in-strokepaint"></a>

<a id="ref-for-attr-valuedef-in-fillpaint"></a>

<a id="ref-for-attr-valuedef-in-backgroundalpha"></a>

<a id="ref-for-attr-valuedef-in-backgroundimage"></a>

<a id="ref-for-attr-valuedef-in-sourcealpha①"></a>

<a id="ref-for-attr-valuedef-in-sourcegraphic②"></a>

<a id="element-attrdef-filter-primitive-in"></a>`in` = "<em><a href="#attr-valuedef-in-sourcegraphic">SourceGraphic</a> | <a href="#attr-valuedef-in-sourcealpha">SourceAlpha</a> | <a href="#attr-valuedef-in-backgroundimage">BackgroundImage</a> | <a href="#attr-valuedef-in-backgroundalpha">BackgroundAlpha</a> | <a href="#attr-valuedef-in-fillpaint">FillPaint</a> | <a href="#attr-valuedef-in-strokepaint">StrokePaint</a> | <a href="#typedef-result-filter-primitive-reference">&lt;filter-primitive-reference&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitive-result"></a>

<a id="ref-for-elementdef-filter②④"></a>

<a id="ref-for-filter-primitive①⑥"></a>

<a id="ref-for-attr-valuedef-in-sourcegraphic③"></a>

Identifies input for the given filter primitive. The value can be either one of six keywords or can be a string which matches a previous [result](#element-attrdef-filter-primitive-result) attribute value within the same [filter](#elementdef-filter) element. If no value is provided and this is the first [filter primitive](#filter-primitive), then this <a id="ref-for-filter-primitive①⑦"></a>filter primitive will use [SourceGraphic](#attr-valuedef-in-sourcegraphic) as its input. If no value is provided and this is a subsequent <a id="ref-for-filter-primitive①⑧"></a>filter primitive, then this <a id="ref-for-filter-primitive①⑨"></a>filter primitive will use the result from the previous <a id="ref-for-filter-primitive②⓪"></a>filter primitive as its input.

<a id="ref-for-element-attrdef-filter-primitive-result①"></a>

<a id="ref-for-elementdef-filter②⑤"></a>

<a id="ref-for-filter-primitive②①"></a>

If the value for [result](#element-attrdef-filter-primitive-result) appears multiple times within a given [filter](#elementdef-filter) element, then a reference to that result will use the closest preceding [filter primitive](#filter-primitive) with the given value for attribute <a id="ref-for-element-attrdef-filter-primitive-result②"></a>result.

Forward references to results are not allowed, and will be treated as if no result was specified.

References to non-existent results will be treated as if no result was specified.

Definitions for the six keywords:

<a id="attr-valuedef-in-sourcegraphic"></a>`SourceGraphic`  
<a id="ref-for-elementdef-filter②⑥"></a>

<a id="ref-for-filter-primitive②②"></a>

This keyword represents the graphics elements that were the original input into the [filter](#elementdef-filter) element. For raster effects [filter primitives](#filter-primitive), the graphics elements will be rasterized into an initially clear RGBA raster in image space. Pixels left untouched by the original graphic will be left clear. The image is specified to be rendered in linear RGBA pixels. The alpha channel of this image captures any anti-aliasing specified by SVG. (Since the raster is linear, the alpha channel of this image will represent the exact percent coverage of each pixel.)

<a id="attr-valuedef-in-sourcealpha"></a>`SourceAlpha`  
<a id="ref-for-elementdef-filter②⑦"></a>

<a id="ref-for-attr-valuedef-in-sourcealpha②"></a>

<a id="ref-for-attr-valuedef-in-sourcegraphic④"></a>

This keyword represents the graphics elements that were the original input into the [filter](#elementdef-filter) element. [SourceAlpha](#attr-valuedef-in-sourcealpha) has all of the same rules as [SourceGraphic](#attr-valuedef-in-sourcegraphic) except that only the alpha channel is used. The input image is an RGBA image consisting of implicitly black color values for the RGB channels, but whose alpha channel is the same as <a id="ref-for-attr-valuedef-in-sourcegraphic⑤"></a>SourceGraphic.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If this option is used, then some implementations might need to rasterize the graphics elements in order to extract the alpha channel.

<a id="attr-valuedef-in-backgroundimage"></a>`BackgroundImage`  
<a id="ref-for-filter-region⑦"></a>

<a id="ref-for-elementdef-filter②⑧"></a>

<a id="ref-for-propdef-isolation①"></a>

This keyword represents the back drop defined by the current isolation group behind the [filter region](#filter-region) at the time that the [filter](#elementdef-filter) element was invoked. See [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation) property [\[COMPOSITING-1\]](#biblio-compositing-1).

<a id="attr-valuedef-in-backgroundalpha"></a>`BackgroundAlpha`  
<a id="ref-for-attr-valuedef-in-backgroundimage①"></a>

<a id="ref-for-attr-valuedef-in-sourcealpha③"></a>

<a id="ref-for-propdef-isolation②"></a>

Same as [BackgroundImage](#attr-valuedef-in-backgroundimage) except only the alpha channel is used. See [SourceAlpha](#attr-valuedef-in-sourcealpha) and the [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation) property [\[COMPOSITING-1\]](#biblio-compositing-1).

<a id="attr-valuedef-in-fillpaint"></a>`FillPaint`  
<a id="ref-for-FillProperty①"></a>

<a id="ref-for-attr-valuedef-in-fillpaint①"></a>

<a id="ref-for-attr-valuedef-primitiveunits-objectboundingbox①"></a>

<a id="ref-for-attr-valuedef-primitiveunits-userspaceonuse②"></a>

<a id="ref-for-local-coordinate-system③"></a>

This keyword represents the value of the [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty) property on the target element for the filter effect. The [FillPaint](#attr-valuedef-in-fillpaint) image has conceptually infinite extent. Frequently this image is opaque everywhere, but it might not be if the "paint" itself has alpha, as in the case of a gradient or pattern which itself includes transparent or semi-transparent parts. If <a id="ref-for-FillProperty②"></a>fill references a paint server, then the coordinate space of the paint server is the coordinate space defined for the filtered object. E.g if the paint server requires to use the [objectBoundingBox](#attr-valuedef-primitiveunits-objectboundingbox) of the object, the object bounding box of the filtered object defines the reference size of the paint server. If the paint server requires to use the [userSpaceOnUse](#attr-valuedef-primitiveunits-userspaceonuse), the nearest viewport in the [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system) of the filtered object defines the reference size of the paint server.

<a id="attr-valuedef-in-strokepaint"></a>`StrokePaint`  
<a id="ref-for-StrokeProperty①"></a>

<a id="ref-for-attr-valuedef-in-strokepaint①"></a>

<a id="ref-for-attr-valuedef-in-fillpaint②"></a>

This keyword represents the value of the [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty) property on the target element for the filter effect. The [StrokePaint](#attr-valuedef-in-strokepaint) image has conceptually infinite extent. See [FillPaint](#attr-valuedef-in-fillpaint) above for more details.

Animatable: yes.

### <a id="FilterPrimitiveTree"></a>9.3. Filter primitive tree

<a id="ref-for-typedef-filter-value-list①"></a>

<a id="ref-for-typedef-filter-function⑤"></a>

Filter primitives with no or one filter primitive input can be linked together to a filter chain. E.g. the filter primitive representation of a [\<filter-value-list\>](#typedef-filter-value-list) with two or more [\<filter-function\>](#typedef-filter-function)s is an example of a filter chain. Every filter primitive takes the result of the previous filter primitive as input.

<a id="ref-for-elementdef-filter②⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fd38116b"></a> A simple example of a [filter](#elementdef-filter) element with its filter primitive children.
>
> ```text
> <filter id="filter">
>   <feColorMatrix type="hueRotate" values="45"/>
>   <feOffset dx="10" dy="10"/>
>   <feGaussianBlur stdDeviation="3"/>
> </filter>
> ```
>
> <a id="ref-for-elementdef-fecolormatrix③"></a>
>
> <a id="ref-for-elementdef-feoffset④"></a>
>
> <a id="ref-for-elementdef-fegaussianblur④"></a>
>
> [feColorMatrix](#elementdef-fecolormatrix), [feOffset](#elementdef-feoffset) and [feGaussianBlur](#elementdef-fegaussianblur) create a filter chain.
>
> <a id="ref-for-elementdef-fecolormatrix④"></a>
>
> <a id="ref-for-elementdef-feoffset⑤"></a>
>
> <a id="ref-for-elementdef-fegaussianblur⑤"></a>
>
> [feColorMatrix](#elementdef-fecolormatrix) takes SourceGraphic as input. The result is the input of [feOffset](#elementdef-feoffset) with its result being the input of [feGaussianBlur](#elementdef-fegaussianblur).

<a id="ref-for-element-attrdef-filter-primitive-in②"></a>

<a id="ref-for-element-attrdef-filter-primitive-result③"></a>

<a id="ref-for-elementdef-filter③⓪"></a>

Some filter primitives may have more than one filter primitive inputs. With the use of the [in](#element-attrdef-filter-primitive-in) and [result](#element-attrdef-filter-primitive-result) attributes it is possible to combine multiple filter primitives to a complex filter structure. Due to the non-forward reference restriction of filter primitives, every filter structure can be represented as a tree, the <a id="filter-primitive-tree"></a>filter primitive tree. The root filter primitive of the filter primitive tree is the most subsequential primitive of [filter](#elementdef-filter) elements filter primitive children.

A filter chain is one possible filter structure that can also be represented in a filter primitive tree. Therefore, filter chains are referred to as filter primitive trees onwards as well.

<a id="ref-for-elementdef-filter③①"></a>

A [filter](#elementdef-filter) element may have one or more filter primitive trees. The filter primitive tree whose subsequent filter primitive is the last filter primitive child of the <a id="ref-for-elementdef-filter③②"></a>filter elements is the <a id="primary-filter-primitive-tree"></a>primary filter primitive tree.

Only the primary filter primitive tree contributes to the filter process. Implementations may chose to ignore all other possible filter primitive trees.

<a id="ref-for-elementdef-filter③③"></a>

If a [filter](#elementdef-filter) element has no filter primitive tree then the element the filter applies to does not get rendered.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2c693d1d"></a> An example of multiple filter primitive trees:
>
> ```text
> <filter id="filter">
>   <-- The first filter primitive tree. Ignored for filter process. -->
>   <feColorMatrix type="hueRotate" values="45"/>
>   <feOffset dx="10" dy="10"/>
>   <feGaussianBlur stdDeviation="3"/>
>   <-- The primary filter primitive tree. -->
>   <feFlood flood-color="green" result="flood"/>
>   <feComposite operator="in" in="SourceAlpha" in2="flood"/>
> </filter>
> ```
>
> The above filter has 2 filter primitive trees with the filter primitives:
>
> 1.  <a id="ref-for-elementdef-fecolormatrix⑤"></a>
>
>     <a id="ref-for-elementdef-feoffset⑥"></a>
>
>     <a id="ref-for-elementdef-fegaussianblur⑥"></a>
>
>     [feColorMatrix](#elementdef-fecolormatrix), [feOffset](#elementdef-feoffset) and [feGaussianBlur](#elementdef-fegaussianblur) (with <a id="ref-for-elementdef-fegaussianblur⑦"></a>feGaussianBlur being the root filter primitive of the tree) as well as
>
> 2.  <a id="ref-for-elementdef-feflood②"></a>
>
>     <a id="ref-for-elementdef-fecomposite④"></a>
>
>     [feFlood](#elementdef-feflood) and [feComposite](#elementdef-fecomposite) (with <a id="ref-for-elementdef-fecomposite⑤"></a>feComposite as the root filter primitive of the tree).
>
> Both filter primitive trees are not connected. Only the 2nd, the primary filter primitive tree contributes to the filter process. The first tree can get ignored by implementations.

### <a id="FilterPrimitiveSubRegion"></a>9.4. Filter primitive subregion

<a id="ref-for-filter-primitive②③"></a>

<a id="ref-for-element-attrdef-filter-primitive-x①"></a>

<a id="ref-for-element-attrdef-filter-primitive-y①"></a>

<a id="ref-for-element-attrdef-filter-primitive-width①"></a>

<a id="ref-for-element-attrdef-filter-primitive-height①"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits⑧"></a>

<a id="ref-for-elementdef-filter③④"></a>

All [filter primitives](#filter-primitive) have attributes [x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width) and [height](#element-attrdef-filter-primitive-height) which together identify a <a id="filter-primitive-subregion"></a>filter primitive subregion which restricts calculation and rendering of the given <a id="ref-for-filter-primitive②④"></a>filter primitive. The <a id="ref-for-element-attrdef-filter-primitive-x②"></a>x, <a id="ref-for-element-attrdef-filter-primitive-y②"></a>y, <a id="ref-for-element-attrdef-filter-primitive-width②"></a>width and <a id="ref-for-element-attrdef-filter-primitive-height②"></a>height attributes are defined according to the same rules as other <a id="ref-for-filter-primitive②⑤"></a>filter primitives coordinate and length attributes and thus represent values in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-element-attrdef-filter-primitive-x③"></a>

<a id="ref-for-element-attrdef-filter-primitive-y③"></a>

<a id="ref-for-element-attrdef-filter-primitive-width③"></a>

<a id="ref-for-element-attrdef-filter-primitive-height③"></a>

<a id="ref-for-elementdef-feimage②"></a>

<a id="ref-for-elementdef-feturbulence②"></a>

<a id="ref-for-attr-valuedef-in-sourcegraphic⑥"></a>

<a id="ref-for-attr-valuedef-in-sourcealpha④"></a>

<a id="ref-for-attr-valuedef-in-backgroundimage②"></a>

<a id="ref-for-attr-valuedef-in-backgroundalpha①"></a>

<a id="ref-for-attr-valuedef-in-fillpaint③"></a>

<a id="ref-for-attr-valuedef-in-strokepaint②"></a>

<a id="ref-for-elementdef-fetile②"></a>

<a id="ref-for-filter-region⑧"></a>

<a id="ref-for-filter-primitive-subregion⑤"></a>

[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width) and [height](#element-attrdef-filter-primitive-height) default to the union (i.e., tightest fitting bounding box) of the subregions defined for all referenced nodes. If there are no referenced nodes (e.g., for [feImage](#elementdef-feimage) or [feTurbulence](#elementdef-feturbulence)), or one or more of the referenced nodes is a standard input (one of [SourceGraphic](#attr-valuedef-in-sourcegraphic), [SourceAlpha](#attr-valuedef-in-sourcealpha), [BackgroundImage](#attr-valuedef-in-backgroundimage), [BackgroundAlpha](#attr-valuedef-in-backgroundalpha), [FillPaint](#attr-valuedef-in-fillpaint) or [StrokePaint](#attr-valuedef-in-strokepaint)), or for [feTile](#elementdef-fetile) (which is special because its principal function is to replicate the referenced node in X and Y and thereby produce a usually larger result), the default subregion is 0%, 0%, 100%, 100%, where as a special-case the percentages are relative to the dimensions of the [filter region](#filter-region), thus making the default [filter primitive subregion](#filter-primitive-subregion) equal to the <a id="ref-for-filter-region⑨"></a>filter region.

<a id="ref-for-filter-primitive-subregion⑥"></a>

If the [filter primitive subregion](#filter-primitive-subregion) has a negative or zero width or height, the effect of the filter primitive is disabled.

<a id="ref-for-filter-region①⓪"></a>

The [filter region](#filter-region) acts as a hard clip clipping rectangle on the filter primitive’s input image(s).

<a id="ref-for-filter-primitive-subregion⑦"></a>

The [filter primitive subregion](#filter-primitive-subregion) acts as a hard clip clipping rectangle on the filter primitive result.

<a id="ref-for-filter-primitive-subregion⑧"></a>

<a id="ref-for-filter-region①①"></a>

All intermediate offscreens are defined to not exceed the intersection of the [filter primitive subregion](#filter-primitive-subregion) with the [filter region](#filter-region). The <a id="ref-for-filter-region①②"></a>filter region and any of the filter primitive subregions are to be set up such that all offscreens are made big enough to accommodate any pixels which even partly intersect with either the <a id="ref-for-filter-region①③"></a>filter region or the filter primitive subregions.

<a id="ref-for-elementdef-fetile③"></a>

<a id="ref-for-filter-primitive-subregion⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ee9442c9"></a> [feTile](#elementdef-fetile) references a previous filter primitive and then stitches the tiles together based on the [filter primitive subregion](#filter-primitive-subregion) of the referenced filter primitive in order to fill its own <a id="ref-for-filter-primitive-subregion①⓪"></a>filter primitive subregion.
>
> ```text
> <svg width="400" height="400" xmlns="http://www.w3.org/2000/svg">
>   <defs>
>     <filter id="flood" x="0" y="0" width="100%" height="100%" primitiveUnits="objectBoundingBox">
>        <feFlood x="25%" y="25%" width="50%" height="50%"
>           flood-color="green" flood-opacity="0.75"/>
>     </filter>
>     <filter id="blend" primitiveUnits="objectBoundingBox">
>        <feBlend x="25%" y="25%" width="50%" height="50%"
>           in2="SourceGraphic" mode="multiply"/>
>     </filter>
>     <filter id="merge" primitiveUnits="objectBoundingBox">
>        <feMerge x="25%" y="25%" width="50%" height="50%">
>         <feMergeNode in="SourceGraphic"/>
>         <feMergeNode in="FillPaint"/>
>        </feMerge>
>     </filter>
>   </defs>
> 
>   <g fill="none" stroke="blue" stroke-width="4">
>      <rect width="200" height="200"/>
>      <line x2="200" y2="200"/>
>      <line x1="200" y2="200"/>
>   </g>
>   <circle fill="green" filter="url(#flood)" cx="100" cy="100" r="90"/>
> 
>   <g transform="translate(200 0)">
>     <g fill="none" stroke="blue" stroke-width="4">
>        <rect width="200" height="200"/>
>        <line x2="200" y2="200"/>
>        <line x1="200" y2="200"/>
>     </g>
>     <circle fill="green" filter="url(#blend)" cx="100" cy="100" r="90"/>
>   </g>
> 
>   <g transform="translate(0 200)">
>     <g fill="none" stroke="blue" stroke-width="4">
>        <rect width="200" height="200"/>
>        <line x2="200" y2="200"/>
>        <line x1="200" y2="200"/>
>     </g>
>     <circle fill="green" fill-opacity="0.5" filter="url(#merge)" cx="100" cy="100" r="90"/>
>   </g>
> </svg>
> ```
>
> ![Example for subregions](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filtersubregion00.png)
>
> Example for subregions
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/filtersubregion00.svg)
>
> <a id="ref-for-filter-primitive-subregion①①"></a>
>
> In the example above there are three rectangles that each have a cross and a circle in them. The circle element in each one has a different filter applied, but with the same [filter primitive subregion](#filter-primitive-subregion). The filter output should be limited to the <a id="ref-for-filter-primitive-subregion①②"></a>filter primitive subregion so you should never see the circles themselves, just the rectangles that make up the <a id="ref-for-filter-primitive-subregion①③"></a>filter primitive subregion.
>
> - <a id="ref-for-elementdef-feflood③"></a>
>
>   <a id="ref-for-propdef-flood-opacity②"></a>
>
>   The upper left rectangle shows an [feFlood](#elementdef-feflood) with [flood-opacity: 75%](#propdef-flood-opacity) so the cross should be visible through the green rect in the middle.
>
> - <a id="ref-for-elementdef-femerge③"></a>
>
>   <a id="ref-for-attr-valuedef-in-sourcegraphic⑦"></a>
>
>   <a id="ref-for-attr-valuedef-in-fillpaint④"></a>
>
>   The lower left rectangle shows an [feMerge](#elementdef-femerge) that merges [SourceGraphic](#attr-valuedef-in-sourcegraphic) with [FillPaint](#attr-valuedef-in-fillpaint). Since the circle has `fill-opacity="0.5"` it will also be transparent so that the cross is visible through the green rect in the middle.
>
> - <a id="ref-for-elementdef-feblend②"></a>
>
>   The upper right rectangle shows an [feBlend](#elementdef-feblend) that has `mode="multiply"`. Since the circle in this case isn’t transparent the result is totally opaque. The rect should be dark green and the cross should not be visible through it.

<a id="ref-for-elementdef-feblend③"></a>

### <a id="feBlendElement"></a>9.5. Filter primitive [feBlend](#elementdef-feblend)

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feblend"></a>`feBlend`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive②⑥"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①"></a>

<a id="ref-for-elementdef-script①"></a>

<a id="ref-for-AnimateElement①"></a>

<a id="ref-for-TermDescriptiveElement"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①"></a>

  <a id="ref-for-propdef-word-spacing①"></a>

  <a id="ref-for-propdef-visibility①"></a>

  <a id="ref-for-propdef-unicode-bidi①"></a>

  <a id="ref-for-TextRenderingProperty①"></a>

  <a id="ref-for-propdef-text-decoration①"></a>

  <a id="ref-for-TextAnchorProperty①"></a>

  <a id="ref-for-StrokeWidthProperty①"></a>

  <a id="ref-for-StrokeOpacityProperty①"></a>

  <a id="ref-for-StrokeMiterlimitProperty①"></a>

  <a id="ref-for-StrokeLinejoinProperty①"></a>

  <a id="ref-for-StrokeLinecapProperty①"></a>

  <a id="ref-for-StrokeDashoffsetProperty①"></a>

  <a id="ref-for-StrokeDasharrayProperty①"></a>

  <a id="ref-for-StrokeProperty②"></a>

  <a id="ref-for-StopOpacityProperty①"></a>

  <a id="ref-for-StopColorProperty①"></a>

  <a id="ref-for-ShapeRenderingProperty①"></a>

  <a id="ref-for-PointerEventsProperty①"></a>

  <a id="ref-for-propdef-overflow①"></a>

  <a id="ref-for-propdef-opacity④"></a>

  <a id="ref-for-propdef-mask①"></a>

  <a id="ref-for-MarkerStartProperty①"></a>

  <a id="ref-for-MarkerMidProperty①"></a>

  <a id="ref-for-MarkerEndProperty①"></a>

  <a id="ref-for-MarkerProperty①"></a>

  <a id="ref-for-propdef-lighting-color②"></a>

  <a id="ref-for-propdef-letter-spacing①"></a>

  <a id="ref-for-KerningProperty①"></a>

  <a id="ref-for-propdef-isolation③"></a>

  <a id="ref-for-propdef-image-rendering①"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①"></a>

  <a id="ref-for-propdef-font-weight①"></a>

  <a id="ref-for-propdef-font-variant①"></a>

  <a id="ref-for-propdef-font-style①"></a>

  <a id="ref-for-propdef-font-stretch①"></a>

  <a id="ref-for-propdef-font-size-adjust①"></a>

  <a id="ref-for-propdef-font-size①"></a>

  <a id="ref-for-propdef-font-family①"></a>

  <a id="ref-for-propdef-font①"></a>

  <a id="ref-for-propdef-flood-opacity③"></a>

  <a id="ref-for-propdef-flood-color②"></a>

  <a id="ref-for-propdef-filter①②"></a>

  <a id="ref-for-FillRuleProperty①"></a>

  <a id="ref-for-FillOpacityProperty①"></a>

  <a id="ref-for-FillProperty③"></a>

  <a id="ref-for-EnableBackgroundProperty①"></a>

  <a id="ref-for-DominantBaselineProperty①"></a>

  <a id="ref-for-propdef-display④"></a>

  <a id="ref-for-propdef-direction①"></a>

  <a id="ref-for-propdef-cursor①"></a>

  <a id="ref-for-ColorRenderingProperty①"></a>

  <a id="ref-for-propdef-color-interpolation-filters⑦"></a>

  <a id="ref-for-ColorInterpolationProperty⑤"></a>

  <a id="ref-for-color0①"></a>

  <a id="ref-for-propdef-clip-rule①"></a>

  <a id="ref-for-propdef-clip-path①"></a>

  <a id="ref-for-propdef-clip①"></a>

  <a id="ref-for-BaselineShiftProperty①"></a>

  <a id="ref-for-AlignmentBaselineProperty①"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x④"></a>

  <a id="ref-for-filter-primitive-attributes"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in③"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-feblend-in2"></a>

  [in2](#element-attrdef-feblend-in2)

- <a id="ref-for-element-attrdef-feblend-mode"></a>

  [mode](#element-attrdef-feblend-mode)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEBlendElement](#InterfaceSVGFEBlendElement)

This filter blends two objects together using commonly used imaging software blending modes. It performs a pixel-wise combination of two input images. (See [\[COMPOSITING-1\]](#biblio-compositing-1).)

<em>Attribute definitions:</em>

<a id="ref-for-ltblendmodegt"></a>

<a id="element-attrdef-feblend-mode"></a>`mode` = "[\<blend-mode\>](https://www.w3.org/TR/compositing-1/#ltblendmodegt)"

<a id="ref-for-element-attrdef-filter-primitive-in④"></a>

<a id="ref-for-element-attrdef-feblend-in2①"></a>

<a id="ref-for-backdrop"></a>

One of the blend modes defined by “Compositing and Blending Level 1” [\[COMPOSITING-1\]](#biblio-compositing-1) with the input [in](#element-attrdef-filter-primitive-in) representing the source `Cs` and the second input [in2](#element-attrdef-feblend-in2) representing the [backdrop](https://www.w3.org/TR/compositing-1/#backdrop) `Cb`. The output of this filter primitive `Cm` is the result of blending `Cs` with `Cb`.

<a id="ref-for-TermInitialValue②①"></a>

<a id="ref-for-element-attrdef-feblend-mode①"></a>

<a id="ref-for-valdef-blend-mode-normal"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [mode](#element-attrdef-feblend-mode) is [normal](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-normal).

Animatable: yes.

<a id="element-attrdef-feblend-no-composite"></a>`no-composite` = "<a id="attr-valuedef-no-composite-no-composite"></a>`no-composite`"

<a id="ref-for-element-attrdef-feblend-no-composite"></a>

<a id="ref-for-element-attrdef-feblend-mode②"></a>

If the [no-composite](#element-attrdef-feblend-no-composite) attribute is present, the specified blend mode must not apply alpha compositing. See [Blending](https://www.w3.org/TR/compositing-1/#blending) [\[COMPOSITING-1\]](#biblio-compositing-1) for the "mixing" formula without compositing. Otherwise, implementations must combine the blend mode specified by [mode](#element-attrdef-feblend-mode) with the [Source Over](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcover) composite operator. See [Blending](https://www.w3.org/TR/compositing-1/#blending) \[COMPOSITING-1\] for the "mixing" formula with compositing.

<a id="ref-for-elementdef-feblend④"></a>

<a id="ref-for-element-attrdef-feblend-no-composite①"></a>

<a id="ref-for-attr-valuedef-in-backgroundimage③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This attribute is an addition to the [feBlend](#elementdef-feblend) element defintion in SVG 1.1. [no-composite](#element-attrdef-feblend-no-composite), when specified, is meant to avoid "double-compositing" effects when blending an input source with the backdrop of the filtered object (E.g. using the [BackgroundImage](#attr-valuedef-in-backgroundimage) filter primitive). For the majority of use cases authors will not need to specify the <a id="ref-for-element-attrdef-feblend-no-composite②"></a>no-composite attribute.

Animatable: no.

<a id="ref-for-element-attrdef-filter-primitive-in⑤"></a>

<a id="element-attrdef-feblend-in2"></a>`in2` = "<em>(see <a href="#element-attrdef-filter-primitive-in">in</a> attribute)</em>"

The second input image to the blending operation.

Animatable: yes.

<a id="ref-for-valdef-blend-mode-normal①"></a>

<a id="ref-for-elementdef-fecomposite⑥"></a>

<a id="ref-for-elementdef-femerge④"></a>

<a id="ref-for-SimpleAlphaBlending"></a>

The [normal](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-normal) blend mode with alpha compositing is equivalent to `operator="over"` on the [feComposite](#elementdef-fecomposite) filter primitive, matches the blending method used by [feMerge](#elementdef-femerge) and matches the [simple alpha compositing](https://www.w3.org/TR/SVG11/masking.html#SimpleAlphaBlending) technique used in SVG for all compositing outside of filter effects.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-50c05d26"></a>
>
> ```text
> <svg width="5cm" height="5cm" viewBox="0 0 500 500"
>      xmlns="http://www.w3.org/2000/svg">
>   <title>Example feBlend - Examples of feBlend modes</title>
>   <desc>Five text strings blended into a gradient,
>         with one text string for each of the five feBlend modes.</desc>
>   <defs>
>     <linearGradient id="MyGradient" gradientUnits="userSpaceOnUse"
>             x1="100" y1="0" x2="300" y2="0">
>       <stop offset="0" stop-color="#000000" />
>       <stop offset=".33" stop-color="#ffffff" />
>       <stop offset=".67" stop-color="#ff0000" />
>       <stop offset="1" stop-color="#808080" />
>     </linearGradient>
>     <filter id="Normal">
>       <feBlend mode="normal" in2="BackgroundImage" in="SourceGraphic"/>
>     </filter>
>     <filter id="Multiply">
>       <feBlend mode="multiply" in2="BackgroundImage" in="SourceGraphic"/>
>     </filter>
>     <filter id="Screen">
>       <feBlend mode="screen" in2="BackgroundImage" in="SourceGraphic"/>
>     </filter>
>     <filter id="Darken">
>       <feBlend mode="darken" in2="BackgroundImage" in="SourceGraphic"/>
>     </filter>
>     <filter id="Lighten">
>       <feBlend mode="lighten" in2="BackgroundImage" in="SourceGraphic"/>
>     </filter>
>   </defs>
>   <rect fill="none" stroke="blue"
>         x="1" y="1" width="498" height="498"/>
>   <g isolation="isolate" >
>     <rect x="100" y="20" width="300" height="460" fill="url(#MyGradient)" />
>     <g font-family="Verdana" font-size="75" fill="#888888" fill-opacity=".6" >
>       <text x="50" y="90" filter="url(#Normal)" >Normal</text>
>       <text x="50" y="180" filter="url(#Multiply)" >Multiply</text>
>       <text x="50" y="270" filter="url(#Screen)" >Screen</text>
>       <text x="50" y="360" filter="url(#Darken)" >Darken</text>
>       <text x="50" y="450" filter="url(#Lighten)" >Lighten</text>
>     </g>
>   </g>
> </svg>
> ```
>
> ![Example of feBlend](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feBlend.png)
>
> Example of feBlend
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feBlend.svg)

<a id="ref-for-elementdef-fecolormatrix⑥"></a>

### <a id="feColorMatrixElement"></a>9.6. Filter primitive [feColorMatrix](#elementdef-fecolormatrix)

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fecolormatrix"></a>`feColorMatrix`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive②⑦"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement②"></a>

<a id="ref-for-elementdef-script②"></a>

<a id="ref-for-AnimateElement②"></a>

<a id="ref-for-TermDescriptiveElement①"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode②"></a>

  <a id="ref-for-propdef-word-spacing②"></a>

  <a id="ref-for-propdef-visibility②"></a>

  <a id="ref-for-propdef-unicode-bidi②"></a>

  <a id="ref-for-TextRenderingProperty②"></a>

  <a id="ref-for-propdef-text-decoration②"></a>

  <a id="ref-for-TextAnchorProperty②"></a>

  <a id="ref-for-StrokeWidthProperty②"></a>

  <a id="ref-for-StrokeOpacityProperty②"></a>

  <a id="ref-for-StrokeMiterlimitProperty②"></a>

  <a id="ref-for-StrokeLinejoinProperty②"></a>

  <a id="ref-for-StrokeLinecapProperty②"></a>

  <a id="ref-for-StrokeDashoffsetProperty②"></a>

  <a id="ref-for-StrokeDasharrayProperty②"></a>

  <a id="ref-for-StrokeProperty③"></a>

  <a id="ref-for-StopOpacityProperty②"></a>

  <a id="ref-for-StopColorProperty②"></a>

  <a id="ref-for-ShapeRenderingProperty②"></a>

  <a id="ref-for-PointerEventsProperty②"></a>

  <a id="ref-for-propdef-overflow②"></a>

  <a id="ref-for-propdef-opacity⑤"></a>

  <a id="ref-for-propdef-mask②"></a>

  <a id="ref-for-MarkerStartProperty②"></a>

  <a id="ref-for-MarkerMidProperty②"></a>

  <a id="ref-for-MarkerEndProperty②"></a>

  <a id="ref-for-MarkerProperty②"></a>

  <a id="ref-for-propdef-lighting-color③"></a>

  <a id="ref-for-propdef-letter-spacing②"></a>

  <a id="ref-for-KerningProperty②"></a>

  <a id="ref-for-propdef-isolation④"></a>

  <a id="ref-for-propdef-image-rendering②"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty②"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty②"></a>

  <a id="ref-for-propdef-font-weight②"></a>

  <a id="ref-for-propdef-font-variant②"></a>

  <a id="ref-for-propdef-font-style②"></a>

  <a id="ref-for-propdef-font-stretch②"></a>

  <a id="ref-for-propdef-font-size-adjust②"></a>

  <a id="ref-for-propdef-font-size②"></a>

  <a id="ref-for-propdef-font-family②"></a>

  <a id="ref-for-propdef-font②"></a>

  <a id="ref-for-propdef-flood-opacity④"></a>

  <a id="ref-for-propdef-flood-color③"></a>

  <a id="ref-for-propdef-filter①③"></a>

  <a id="ref-for-FillRuleProperty②"></a>

  <a id="ref-for-FillOpacityProperty②"></a>

  <a id="ref-for-FillProperty④"></a>

  <a id="ref-for-EnableBackgroundProperty②"></a>

  <a id="ref-for-DominantBaselineProperty②"></a>

  <a id="ref-for-propdef-display⑤"></a>

  <a id="ref-for-propdef-direction②"></a>

  <a id="ref-for-propdef-cursor②"></a>

  <a id="ref-for-ColorRenderingProperty②"></a>

  <a id="ref-for-propdef-color-interpolation-filters⑧"></a>

  <a id="ref-for-ColorInterpolationProperty⑥"></a>

  <a id="ref-for-color0②"></a>

  <a id="ref-for-propdef-clip-rule②"></a>

  <a id="ref-for-propdef-clip-path②"></a>

  <a id="ref-for-propdef-clip②"></a>

  <a id="ref-for-BaselineShiftProperty②"></a>

  <a id="ref-for-AlignmentBaselineProperty②"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x⑤"></a>

  <a id="ref-for-filter-primitive-attributes①"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in⑥"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fecolormatrix-type"></a>

  [type](#element-attrdef-fecolormatrix-type)

- <a id="ref-for-element-attrdef-fecolormatrix-values"></a>

  [values](#element-attrdef-fecolormatrix-values)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEColorMatrixElement](#InterfaceSVGFEColorMatrixElement)

This filter applies a matrix transformation:

<strong>Mathematical expression 1</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
{R'} \\
{G'} \\
{B'} \\
{A'} \\
1
\end{bmatrix} = {\begin{bmatrix}
a_{00} & a_{01} & a_{02} & a_{03} & a_{04} \\
a_{10} & a_{11} & a_{12} & a_{13} & a_{14} \\
a_{20} & a_{21} & a_{22} & a_{23} & a_{24} \\
a_{30} & a_{31} & a_{32} & a_{33} & a_{34} \\
0 & 0 & 0 & 0 & 1
\end{bmatrix} \cdot \begin{bmatrix}
R \\
G \\
B \\
A \\
1
\end{bmatrix}}
```

Source mathematical tokens: R ' G ' B ' A ' 1 = a 00 a 01 a 02 a 03 a 04 a 10 a 11 a 12 a 13 a 14 a 20 a 21 a 22 a 23 a 24 a 30 a 31 a 32 a 33 a 34 0 0 0 0 1 ⋅ R G B A 1 left \[ stack {R' \# G' \# B' \# A' \# 1} right \] = left \[ matrix { a_00 \# a_01 \# a_02 \# a_03 \# a_04 \## a_10 \# a_11 \# a_12 \# a_13 \# a_14 \## a_20 \# a_21 \# a_22 \# a_23 \# a_24 \## a_30 \# a_31 \# a_32 \# a_33 \# a_34 \## 0 \# 0 \# 0 \# 0 \# 1 } right \] cdot left\[ stack { R \# G \# B \# A \# 1 } right\]

on the RGBA color and alpha values of every pixel on the input graphics to produce a result with a new set of RGBA color and alpha values.

The calculations are performed on non-premultiplied color values.

<em>Attribute definitions:</em>

<a id="element-attrdef-fecolormatrix-type"></a>`type` = "<a id="attr-valuedef-type-matrix"></a>`matrix` \| <a id="attr-valuedef-type-saturate"></a>`saturate` \| <a id="attr-valuedef-type-huerotate"></a>`hueRotate` \| <a id="attr-valuedef-type-luminancetoalpha"></a>`luminanceToAlpha`"

<a id="ref-for-attr-valuedef-type-matrix"></a>

Indicates the type of matrix operation. The keyword [matrix](#attr-valuedef-type-matrix) indicates that a full 5x4 matrix of values will be provided. The other keywords represent convenience shortcuts to allow commonly used color operations to be performed without specifying a complete matrix.

<a id="ref-for-TermInitialValue②②"></a>

<a id="ref-for-element-attrdef-fecolormatrix-type①"></a>

<a id="ref-for-attr-valuedef-type-matrix①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [type](#element-attrdef-fecolormatrix-type) is [matrix](#attr-valuedef-type-matrix).

Animatable: yes.

<a id="ref-for-number-value②"></a>

<a id="element-attrdef-fecolormatrix-values"></a>`values` = "<em>list of <a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a>s</em>"

<a id="ref-for-element-attrdef-fecolormatrix-values①"></a>

<a id="ref-for-element-attrdef-fecolormatrix-type②"></a>

The contents of [values](#element-attrdef-fecolormatrix-values) depends on the value of attribute [type](#element-attrdef-fecolormatrix-type):

- <a id="ref-for-element-attrdef-fecolormatrix-values②"></a>

  For `type="matrix"`, [values](#element-attrdef-fecolormatrix-values) is a list of 20 matrix values (a00 a01 a02 a03 a04 a10 a11 ... a34), separated by whitespace and/or a comma. For example, the identity matrix could be expressed as:

  ```text
  type="matrix"
  values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 1 0"
  ```
- <a id="ref-for-element-attrdef-fecolormatrix-values③"></a>

  <a id="ref-for-attr-valuedef-type-saturate"></a>

  For `type="saturate"`, [values](#element-attrdef-fecolormatrix-values) is a single real number value. A [saturate](#attr-valuedef-type-saturate) operation is equivalent to the following matrix operation:

  <strong>Mathematical expression 2</strong>

  TeX transcription (renderer-independent source notation):

  ``` language-tex
  \begin{bmatrix}
  {R'} \\
  {G'} \\
  {B'} \\
  {A'} \\
  1
  \end{bmatrix} = {\begin{bmatrix}
  {0.213 + 0.787s} & {0.715 - 0.715s} & {0.072 - 0.072s} & 0 & 0 \\
  {0.213 - 0.213s} & {0.715 + 0.285s} & {0.072 - 0.072s} & 0 & 0 \\
  {0.213 - 0.213s} & {0.715 - 0.715s} & {0.072 + 0.928s} & 0 & 0 \\
  0 & 0 & 0 & 1 & 0 \\
  0 & 0 & 0 & 0 & 1
  \end{bmatrix} \cdot \begin{bmatrix}
  R \\
  G \\
  B \\
  A \\
  1
  \end{bmatrix}}
  ```

  Source mathematical tokens: R ' G ' B ' A ' 1 = 0.213 + 0.787s 0.715 − 0.715s 0.072 − 0.072s 0 0 0.213 − 0.213s 0.715 + 0.285s 0.072 − 0.072s 0 0 0.213 − 0.213s 0.715 − 0.715s 0.072 + 0.928s 0 0 0 0 0 1 0 0 0 0 0 1 ⋅ R G B A 1 left \[ stack {R' \# G' \# B' \# A' \# 1} right \] = left \[ matrix { 0.213 + 0.787s \# 0.715 - 0.715s \# 0.072 - 0.072s \# 0 \# 0 \## 0.213 - 0.213s \# 0.715 + 0.285s \# 0.072 - 0.072s \# 0 \# 0 \## 0.213 - 0.213s \# 0.715 - 0.715s \# 0.072 + 0.928s \# 0 \# 0 \## 0 \# 0 \# 0 \# 1 \# 0 \## 0 \# 0 \# 0 \# 0 \# 1 } right \] cdot left\[ stack { R \# G \# B \# A \# 1 } right\]

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: A value of 0 produces a fully desaturated (grayscale) filter result, while a value of 1 passes the filter input image through unchanged. Values outside the 0..1 range under- or oversaturates the filter input image respectively.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The precision of the luminance coefficients increased in comparison to previous specification texts [\[Cmam\]](#biblio-cmam).

- <a id="ref-for-element-attrdef-fecolormatrix-values④"></a>

  <a id="ref-for-attr-valuedef-type-huerotate"></a>

  For `type="hueRotate"`, [values](#element-attrdef-fecolormatrix-values) is a single one real number value (degrees). A [hueRotate](#attr-valuedef-type-huerotate) operation is equivalent to the following matrix operation:

  <strong>Mathematical expression 3</strong>

  TeX transcription (renderer-independent source notation):

  ``` language-tex
  \begin{bmatrix}
  {R'} \\
  {G'} \\
  {B'} \\
  {A'} \\
  1
  \end{bmatrix} = {\begin{bmatrix}
  a_{00} & a_{01} & a_{02} & 0 & 0 \\
  a_{10} & a_{11} & a_{12} & 0 & 0 \\
  a_{20} & a_{21} & a_{22} & 0 & 0 \\
  0 & 0 & 0 & 1 & 0 \\
  0 & 0 & 0 & 0 & 1
  \end{bmatrix} \cdot \begin{bmatrix}
  R \\
  G \\
  B \\
  A \\
  1
  \end{bmatrix}}
  ```

  Source mathematical tokens: R ' G ' B ' A ' 1 = a 00 a 01 a 02 0 0 a 10 a 11 a 12 0 0 a 20 a 21 a 22 0 0 0 0 0 1 0 0 0 0 0 1 ⋅ R G B A 1 left \[ stack {R' \# G' \# B' \# A' \# 1} right \] = left \[ matrix { a_00 \# a_01 \# a_02 \# 0 \# 0 \## a_10 \# a_11 \# a_12 \# 0 \# 0 \## a_20 \# a_21 \# a_22 \# 0 \# 0 \## 0 \# 0 \# 0 \# 1 \# 0 \## 0 \# 0 \# 0 \# 0 \# 1 } right \] cdot left\[ stack { R \# G \# B \# A \# 1 } right\]

  where the terms a00, a01, etc. are calculated as follows:

  <strong>Mathematical expression 4</strong>

  TeX transcription (renderer-independent source notation):

  ``` language-tex
  {\begin{bmatrix}
  a_{00} & a_{01} & a_{02} \\
  a_{10} & a_{11} & a_{12} \\
  a_{20} & a_{21} & a_{22}
  \end{bmatrix} = {\begin{bmatrix}
  {+ 0.213} & {+ 0.715} & {+ 0.072} \\
  {+ 0.213} & {+ 0.715} & {+ 0.072} \\
  {+ 0.213} & {+ 0.715} & {+ 0.072}
  \end{bmatrix} + \cos}}{{{({\mathit{hueRotate}\mathit{value}})} \cdot \begin{bmatrix}
  {+ 0.787} & {- 0.715} & {- 0.072} \\
  {- 0.213} & {+ 0.285} & {- 0.072} \\
  {- 0.213} & {- 0.715} & {+ 0.928}
  \end{bmatrix}} + \sin}{{({\mathit{hueRotate}\mathit{value}})} \cdot \begin{bmatrix}
  {- 0.213} & {- 0.715} & {+ 0.928} \\
  {+ 0.143} & {+ 0.140} & {- 0.283} \\
  {- 0.787} & {+ 0.715} & {+ 0.072}
  \end{bmatrix}}
  ```

  Source mathematical tokens: a 00 a 01 a 02 a 10 a 11 a 12 a 20 a 21 a 22 = + 0.213 + 0.715 + 0.072 + 0.213 + 0.715 + 0.072 + 0.213 + 0.715 + 0.072 + cos ( hueRotate value ) ⋅ + 0.787 − 0.715 − 0.072 − 0.213 + 0.285 − 0.072 − 0.213 − 0.715 + 0.928 + sin ( hueRotate value ) ⋅ − 0.213 − 0.715 + 0.928 + 0.143 + 0.140 − 0.283 − 0.787 + 0.715 + 0.072 left\[ matrix { a_00 \# a_01 \# a_02 \## a_10 \# a_11 \# a_12 \## a_20 \# a_21 \# a_22 } right\] = left\[ matrix { +0.213 \# +0.715 \# +0.072 \## +0.213 \# +0.715 \# +0.072 \## +0.213 \# +0.715 \# +0.072 } right\] + cos(hueRotate value)cdot left\[ matrix { +0.787 \# -0.715 \# -0.072 \## -0.213 \# +0.285 \# -0.072 \## -0.213 \# -0.715 \# +0.928 } right\] + sin(hueRotate value) cdot left\[ matrix { -0.213 \# -0.715 \# +0.928 \## +0.143 \# +0.140 \# -0.283 \## -0.787 \# +0.715 \# +0.072 } right\]

  Thus, the upper left term of the hue matrix turns out to be:

  <strong>Mathematical expression 5</strong>

  TeX transcription (renderer-independent source notation):

  ``` language-tex
  {a_{00} = {0.2127 + \cos}}{{{({\mathit{hueRotate}\mathit{value}})} \cdot 0.7873} - \sin}{{({\mathit{hueRotate}\mathit{value}})} \cdot 0.2127}
  ```

  Source mathematical tokens: a 00 = 0.2127 + cos ( hueRotate value ) ⋅ 0.7873 − sin ( hueRotate value ) ⋅ 0.2127 a_00 = 0.2127 + cos(hueRotate value) cdot 0.7873 - sin(hueRotate value) cdot 0.2127

- <a id="ref-for-element-attrdef-fecolormatrix-values⑤"></a>

  <a id="ref-for-attr-valuedef-type-luminancetoalpha"></a>

  For `type="luminanceToAlpha"`, [values](#element-attrdef-fecolormatrix-values) is not applicable. A [luminanceToAlpha](#attr-valuedef-type-luminancetoalpha) operation is equivalent to the following matrix operation:

  <strong>Mathematical expression 6</strong>

  TeX transcription (renderer-independent source notation):

  ``` language-tex
  \begin{bmatrix}
  {R'} \\
  {G'} \\
  {B'} \\
  {A'} \\
  1
  \end{bmatrix} = {\begin{bmatrix}
  0 & 0 & 0 & 0 & 0 \\
  0 & 0 & 0 & 0 & 0 \\
  0 & 0 & 0 & 0 & 0 \\
  0.2126 & 0.7152 & 0.0722 & 0 & 0 \\
  0 & 0 & 0 & 0 & 1
  \end{bmatrix} \cdot \begin{bmatrix}
  R \\
  G \\
  B \\
  A \\
  1
  \end{bmatrix}}
  ```

  Source mathematical tokens: R ' G ' B ' A ' 1 = 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.2126 0.7152 0.0722 0 0 0 0 0 0 1 ⋅ R G B A 1 left \[ stack {R' \# G' \# B' \# A' \# 1} right \] = left \[ matrix { 0 \# 0 \# 0 \# 0 \# 0 \## 0 \# 0 \# 0 \# 0 \# 0 \## 0 \# 0 \# 0 \# 0 \# 0 \## 0.2126 \# 0.7152 \# 0.0722 \# 0 \# 0 \## 0 \# 0 \# 0 \# 0 \# 1 } right \] cdot left\[ stack { R \# G \# B \# A \# 1 } right\]

<a id="ref-for-TermInitialValue②③"></a>

<a id="ref-for-element-attrdef-fecolormatrix-values⑥"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [values](#element-attrdef-fecolormatrix-values)

if `type="matrix"`  
defaults to the identity matrix

if `type="saturate"`  
defaults to the value 1

if `type="hueRotate"`  
defaults to the value 0 which results in the identity matrix.

<a id="ref-for-element-attrdef-fecolormatrix-values⑦"></a>

<a id="ref-for-element-attrdef-fecolormatrix-type③"></a>

<a id="ref-for-pass-through-filter"></a>

If the number of entries in the [values](#element-attrdef-fecolormatrix-values) list does not match the required number of entries by the [type](#element-attrdef-fecolormatrix-type), the filter primitive acts as a [pass through filter](#pass-through-filter).

Animatable: yes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-05772ccd"></a>
>
> ```text
> <svg width="8cm" height="5cm" viewBox="0 0 800 500"
>      xmlns="http://www.w3.org/2000/svg">
>   <title>Example feColorMatrix - Examples of feColorMatrix operations</title>
>   <desc>Five text strings showing the effects of feColorMatrix:
>         an unfiltered text string acting as a reference,
>         use of the feColorMatrix matrix option to convert to grayscale,
>         use of the feColorMatrix saturate option,
>         use of the feColorMatrix hueRotate option,
>         and use of the feColorMatrix luminanceToAlpha option.</desc>
>   <defs>
>     <linearGradient id="MyGradient" gradientUnits="userSpaceOnUse"
>             x1="100" y1="0" x2="500" y2="0">
>       <stop offset="0" stop-color="#ff00ff" />
>       <stop offset=".33" stop-color="#88ff88" />
>       <stop offset=".67" stop-color="#2020ff" />
>       <stop offset="1" stop-color="#d00000" />
>     </linearGradient>
>     <filter id="Matrix" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feColorMatrix type="matrix" in="SourceGraphic"
>            values=".33 .33 .33 0 0
>                    .33 .33 .33 0 0
>                    .33 .33 .33 0 0
>                    .33 .33 .33 0 0"/>
>     </filter>
>     <filter id="Saturate40" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feColorMatrix type="saturate" in="SourceGraphic" values="0.4"/>
>     </filter>
>     <filter id="HueRotate90" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feColorMatrix type="hueRotate" in="SourceGraphic" values="90"/>
>     </filter>
>     <filter id="LuminanceToAlpha" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feColorMatrix type="luminanceToAlpha" in="SourceGraphic" result="a"/>
>       <feComposite in="SourceGraphic" in2="a" operator="in" />
>     </filter>
>   </defs>
>   <rect fill="none" stroke="blue"
>         x="1" y="1" width="798" height="498"/>
>   <g font-family="Verdana" font-size="75"
>             font-weight="bold" fill="url(#MyGradient)" >
>     <rect x="100" y="0" width="500" height="20" />
>     <text x="100" y="90">Unfiltered</text>
>     <text x="100" y="190" filter="url(#Matrix)" >Matrix</text>
>     <text x="100" y="290" filter="url(#Saturate40)" >Saturate</text>
>     <text x="100" y="390" filter="url(#HueRotate90)" >HueRotate</text>
>     <text x="100" y="490" filter="url(#LuminanceToAlpha)" >Luminance</text>
>   </g>
> </svg>
> ```
>
> ![Example ](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feColorMatrix.png)
>
> Example of feColorMatrix
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feColorMatrix.svg)

<a id="ref-for-elementdef-fecomponenttransfer③"></a>

### <a id="feComponentTransferElement"></a>9.7. Filter primitive [feComponentTransfer](#elementdef-fecomponenttransfer)

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fecomponenttransfer"></a>`feComponentTransfer`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive②⑧"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-script③"></a>

<a id="ref-for-elementdef-fefunca"></a>

<a id="ref-for-elementdef-fefuncb"></a>

<a id="ref-for-elementdef-fefuncg"></a>

<a id="ref-for-elementdef-fefuncr"></a>

<a id="ref-for-TermDescriptiveElement②"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [feFuncR](#elementdef-fefuncr), [feFuncG](#elementdef-fefuncg), [feFuncB](#elementdef-fefuncb), [feFuncA](#elementdef-fefunca), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode③"></a>

  <a id="ref-for-propdef-word-spacing③"></a>

  <a id="ref-for-propdef-visibility③"></a>

  <a id="ref-for-propdef-unicode-bidi③"></a>

  <a id="ref-for-TextRenderingProperty③"></a>

  <a id="ref-for-propdef-text-decoration③"></a>

  <a id="ref-for-TextAnchorProperty③"></a>

  <a id="ref-for-StrokeWidthProperty③"></a>

  <a id="ref-for-StrokeOpacityProperty③"></a>

  <a id="ref-for-StrokeMiterlimitProperty③"></a>

  <a id="ref-for-StrokeLinejoinProperty③"></a>

  <a id="ref-for-StrokeLinecapProperty③"></a>

  <a id="ref-for-StrokeDashoffsetProperty③"></a>

  <a id="ref-for-StrokeDasharrayProperty③"></a>

  <a id="ref-for-StrokeProperty④"></a>

  <a id="ref-for-StopOpacityProperty③"></a>

  <a id="ref-for-StopColorProperty③"></a>

  <a id="ref-for-ShapeRenderingProperty③"></a>

  <a id="ref-for-PointerEventsProperty③"></a>

  <a id="ref-for-propdef-overflow③"></a>

  <a id="ref-for-propdef-opacity⑥"></a>

  <a id="ref-for-propdef-mask③"></a>

  <a id="ref-for-MarkerStartProperty③"></a>

  <a id="ref-for-MarkerMidProperty③"></a>

  <a id="ref-for-MarkerEndProperty③"></a>

  <a id="ref-for-MarkerProperty③"></a>

  <a id="ref-for-propdef-lighting-color④"></a>

  <a id="ref-for-propdef-letter-spacing③"></a>

  <a id="ref-for-KerningProperty③"></a>

  <a id="ref-for-propdef-isolation⑤"></a>

  <a id="ref-for-propdef-image-rendering③"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty③"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty③"></a>

  <a id="ref-for-propdef-font-weight③"></a>

  <a id="ref-for-propdef-font-variant③"></a>

  <a id="ref-for-propdef-font-style③"></a>

  <a id="ref-for-propdef-font-stretch③"></a>

  <a id="ref-for-propdef-font-size-adjust③"></a>

  <a id="ref-for-propdef-font-size③"></a>

  <a id="ref-for-propdef-font-family③"></a>

  <a id="ref-for-propdef-font③"></a>

  <a id="ref-for-propdef-flood-opacity⑤"></a>

  <a id="ref-for-propdef-flood-color④"></a>

  <a id="ref-for-propdef-filter①④"></a>

  <a id="ref-for-FillRuleProperty③"></a>

  <a id="ref-for-FillOpacityProperty③"></a>

  <a id="ref-for-FillProperty⑤"></a>

  <a id="ref-for-EnableBackgroundProperty③"></a>

  <a id="ref-for-DominantBaselineProperty③"></a>

  <a id="ref-for-propdef-display⑥"></a>

  <a id="ref-for-propdef-direction③"></a>

  <a id="ref-for-propdef-cursor③"></a>

  <a id="ref-for-ColorRenderingProperty③"></a>

  <a id="ref-for-propdef-color-interpolation-filters⑨"></a>

  <a id="ref-for-ColorInterpolationProperty⑦"></a>

  <a id="ref-for-color0③"></a>

  <a id="ref-for-propdef-clip-rule③"></a>

  <a id="ref-for-propdef-clip-path③"></a>

  <a id="ref-for-propdef-clip③"></a>

  <a id="ref-for-BaselineShiftProperty③"></a>

  <a id="ref-for-AlignmentBaselineProperty③"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x⑥"></a>

  <a id="ref-for-filter-primitive-attributes②"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in⑦"></a>

  [in](#element-attrdef-filter-primitive-in)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEComponentTransferElement](#InterfaceSVGFEComponentTransferElement)

This filter primitive performs component-wise remapping of data as follows:

<a id="ref-for-elementdef-fefuncr①"></a>

<a id="ref-for-elementdef-fefuncg①"></a>

<a id="ref-for-elementdef-fefuncb①"></a>

<a id="ref-for-elementdef-fefunca①"></a>

```text
R' = feFuncR( R )G' = feFuncG( G )
B' = feFuncB( B )
A' = feFuncA( A )
```
for every pixel. It allows operations like brightness adjustment, contrast adjustment, color balance or thresholding.

The calculations are performed on non-premultiplied color values.

<a id="ref-for-elementdef-fecomponenttransfer④"></a>

The child elements of a [feComponentTransfer](#elementdef-fecomponenttransfer) element specify the transfer functions for the four channels:

- <a id="ref-for-elementdef-fefuncr②"></a>

  [feFuncR](#elementdef-fefuncr) - transfer function for the red component of the input graphic

- <a id="ref-for-elementdef-fefuncg②"></a>

  [feFuncG](#elementdef-fefuncg) - transfer function for the green component of the input graphic

- <a id="ref-for-elementdef-fefuncb②"></a>

  [feFuncB](#elementdef-fefuncb) - transfer function for the blue component of the input graphic

- <a id="ref-for-elementdef-fefunca②"></a>

  [feFuncA](#elementdef-fefunca) - transfer function for the alpha component of the input graphic

<a id="ref-for-elementdef-fefuncr③"></a>

<a id="ref-for-elementdef-fefuncg③"></a>

<a id="ref-for-elementdef-fefuncb③"></a>

<a id="ref-for-elementdef-fefunca③"></a>

The set of [feFuncR](#elementdef-fefuncr), [feFuncG](#elementdef-fefuncg), [feFuncB](#elementdef-fefuncb), [feFuncA](#elementdef-fefunca) elements are also called <a id="transfer-function-element"></a>transfer function elements.

<a id="ref-for-elementdef-fecomponenttransfer⑤"></a>

The following rules apply to the processing of the [feComponentTransfer](#elementdef-fecomponenttransfer) element:

- <a id="ref-for-transfer-function-element①"></a>

  If more than one [transfer function element](#transfer-function-element) of the same kind is specified, the last occurrence is to be used.

- <a id="ref-for-transfer-function-element②"></a>

  <a id="ref-for-elementdef-fecomponenttransfer⑥"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-type"></a>

  If any of the [transfer function elements](#transfer-function-element) are unspecified, the [feComponentTransfer](#elementdef-fecomponenttransfer) must be processed as if those <a id="ref-for-transfer-function-element③"></a>transfer function elements were specified with their [type](#element-attrdef-fecomponenttransfer-type) attributes set to identity.

<a id="ref-for-elementdef-fefuncr④"></a>

#### <a id="feFuncRElement"></a>9.7.1. Transfer function [feFuncR](#elementdef-fefuncr)

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fefuncr"></a>`feFuncR`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transfer-function-element④"></a>

[transfer function element](#transfer-function-element)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement③"></a>

<a id="ref-for-elementdef-script④"></a>

<a id="ref-for-AnimateElement③"></a>

<a id="ref-for-TermDescriptiveElement③"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fecomponenttransfer-offset"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-exponent"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-amplitude"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-intercept"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-slope"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-type①"></a>

  <a id="ref-for-transfer-function-element-attributes"></a>

  [transfer function element attributes](#transfer-function-element-attributes) — [type](#element-attrdef-fecomponenttransfer-type), [tableValues](#element-attrdef-fecomponenttransfer-tablevalues), [slope](#element-attrdef-fecomponenttransfer-slope), [intercept](#element-attrdef-fecomponenttransfer-intercept), [amplitude](#element-attrdef-fecomponenttransfer-amplitude), [exponent](#element-attrdef-fecomponenttransfer-exponent), [offset](#element-attrdef-fecomponenttransfer-offset)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEFuncRElement](#InterfaceSVGFEFuncRElement)

<a id="ref-for-transfer-function-element⑤"></a>

The attributes below are the <a id="transfer-function-element-attributes"></a>transfer function element attributes, which apply to the [transfer function elements](#transfer-function-element).

<em>Attribute definitions:</em>

<a id="ref-for-attr-valuedef-type-gamma"></a>

<a id="ref-for-attr-valuedef-type-linear"></a>

<a id="ref-for-attr-valuedef-type-discrete"></a>

<a id="ref-for-attr-valuedef-type-table"></a>

<a id="ref-for-attr-valuedef-type-identity"></a>

<a id="element-attrdef-fecomponenttransfer-type"></a>`type` = "[identity](#attr-valuedef-type-identity) \| [table](#attr-valuedef-type-table) \| [discrete](#attr-valuedef-type-discrete) \| [linear](#attr-valuedef-type-linear) \| [gamma](#attr-valuedef-type-gamma)"

Indicates the type of component transfer function. The type of function determines the applicability of the other attributes.

<a id="ref-for-elementdef-fefuncr⑤"></a>

In the following, C is the initial component (e.g., [feFuncR](#elementdef-fefuncr)), C' is the remapped component; both in the closed interval \[0,1\].

- For <a id="attr-valuedef-type-identity"></a>`identity`:

  ```text
  C' = C
  ```
- <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues①"></a>

  For <a id="attr-valuedef-type-table"></a>`table`, the function is defined by linear interpolation between values given in the attribute [tableValues](#element-attrdef-fecomponenttransfer-tablevalues). The table has <em>n+1</em> values (i.e., v<sub>0</sub> to v<sub>n</sub>) specifying the start and end values for <em>n</em> evenly sized interpolation regions. Interpolations use the following formula:

  For a value `C < 1` find `k` such that:

  `k/n <= C < (k+1)/n`

  The result `C'` is given by:

  <code>C' = v<sub>k</sub> + (C - k/n)&#x2A;n &#x2A; (v<sub>k+1</sub> - v<sub>k</sub>)</code>

  If `C = 1` then:

  <code>C' = v<sub>n</sub>.</code>

- <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues②"></a>

  For <a id="attr-valuedef-type-discrete"></a>`discrete`, the function is defined by the step function given in the attribute [tableValues](#element-attrdef-fecomponenttransfer-tablevalues), which provides a list of <em>n</em> values (i.e., v<sub>0</sub> to v<sub>n-1</sub>) in order to identify a step function consisting of <em>n</em> steps. The step function is defined by the following formula:

  For a value `C < 1` find `k` such that:

  `k/n <= C < (k+1)/n`

  The result `C'` is given by:

  <code>C' = v<sub>k</sub></code>

  If `C = 1` then:

  <code>C' = v<sub>n-1</sub>.</code>

- For <a id="attr-valuedef-type-linear"></a>`linear`, the function is defined by the following linear equation:

  <a id="ref-for-element-attrdef-fecomponenttransfer-slope①"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-intercept①"></a>

  <code>C' = <a href="#element-attrdef-fecomponenttransfer-slope">slope</a> &#x2A; C + <a href="#element-attrdef-fecomponenttransfer-intercept">intercept</a></code>

- For <a id="attr-valuedef-type-gamma"></a>`gamma`, the function is defined by the following exponential function:

  <a id="ref-for-element-attrdef-fecomponenttransfer-amplitude①"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-exponent①"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-offset①"></a>

  <code>C' = <a href="#element-attrdef-fecomponenttransfer-amplitude">amplitude</a> &#x2A; pow(C, <a href="#element-attrdef-fecomponenttransfer-exponent">exponent</a>) + <a href="#element-attrdef-fecomponenttransfer-offset">offset</a></code>

<a id="ref-for-TermInitialValue②④"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-type②"></a>

<a id="ref-for-attr-valuedef-type-identity①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [type](#element-attrdef-fecomponenttransfer-type) is [identity](#attr-valuedef-type-identity).

Animatable: yes.

<a id="ref-for-number-value③"></a>

<a id="element-attrdef-fecomponenttransfer-tablevalues"></a>`tableValues` = "<em>(list of <a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a>s)</em>"

<a id="ref-for-number-value④"></a>

When `type="table"`, the list of [\<number\>](https://www.w3.org/TR/css3-values/#number-value) s <em>v0,v1,...vn</em>, separated by white space and/or a comma, which define the lookup table. An empty list results in an identity transfer function.

If the attribute is not specified, then the effect is as if an empty list were provided.

Animatable: yes.

<a id="ref-for-number-value⑤"></a>

<a id="element-attrdef-fecomponenttransfer-slope"></a>`slope` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

When `type="linear"`, the slope of the linear function.

<a id="ref-for-TermInitialValue②⑤"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-slope②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [slope](#element-attrdef-fecomponenttransfer-slope) is 1.

Animatable: yes.

<a id="ref-for-number-value⑥"></a>

<a id="element-attrdef-fecomponenttransfer-intercept"></a>`intercept` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

When `type="linear"`, the intercept of the linear function.

<a id="ref-for-TermInitialValue②⑥"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-intercept②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [intercept](#element-attrdef-fecomponenttransfer-intercept) is 0.

Animatable: yes.

<a id="ref-for-number-value⑦"></a>

<a id="element-attrdef-fecomponenttransfer-amplitude"></a>`amplitude` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

When `type="gamma"`, the amplitude of the gamma function.

<a id="ref-for-TermInitialValue②⑦"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-amplitude②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [amplitude](#element-attrdef-fecomponenttransfer-amplitude) is 1.

Animatable: yes.

<a id="ref-for-number-value⑧"></a>

<a id="element-attrdef-fecomponenttransfer-exponent"></a>`exponent` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

When `type="gamma"`, the exponent of the gamma function.

<a id="ref-for-TermInitialValue②⑧"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-exponent②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [exponent](#element-attrdef-fecomponenttransfer-exponent) is 1.

Animatable: yes.

<a id="ref-for-number-value⑨"></a>

<a id="element-attrdef-fecomponenttransfer-offset"></a>`offset` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

When `type="gamma"`, the offset of the gamma function.

<a id="ref-for-TermInitialValue②⑨"></a>

<a id="ref-for-element-attrdef-fecomponenttransfer-offset②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [offset](#element-attrdef-fecomponenttransfer-offset) is 0.

Animatable: yes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-49a7ba07"></a>
>
> ```text
> <svg width="8cm" height="4cm" viewBox="0 0 800 400"
>      xmlns="http://www.w3.org/2000/svg">
>   <title>Example feComponentTransfer - Examples of feComponentTransfer operations</title>
>   <desc>Four text strings showing the effects of feComponentTransfer:
>         an identity function acting as a reference,
>         use of the feComponentTransfer table option,
>         use of the feComponentTransfer linear option,
>         and use of the feComponentTransfer gamma option.</desc>
>   <defs>
>     <linearGradient id="MyGradient" gradientUnits="userSpaceOnUse"
>             x1="100" y1="0" x2="600" y2="0">
>       <stop offset="0" stop-color="#ff0000" />
>       <stop offset=".33" stop-color="#00ff00" />
>       <stop offset=".67" stop-color="#0000ff" />
>       <stop offset="1" stop-color="#000000" />
>     </linearGradient>
>     <filter id="Identity" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feComponentTransfer>
>         <feFuncR type="identity"/>
>         <feFuncG type="identity"/>
>         <feFuncB type="identity"/>
>         <feFuncA type="identity"/>
>       </feComponentTransfer>
>     </filter>
>     <filter id="Table" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feComponentTransfer>
>         <feFuncR type="table" tableValues="0 0 1 1"/>
>         <feFuncG type="table" tableValues="1 1 0 0"/>
>         <feFuncB type="table" tableValues="0 1 1 0"/>
>       </feComponentTransfer>
>     </filter>
>     <filter id="Linear" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feComponentTransfer>
>         <feFuncR type="linear" slope=".5" intercept=".25"/>
>         <feFuncG type="linear" slope=".5" intercept="0"/>
>         <feFuncB type="linear" slope=".5" intercept=".5"/>
>       </feComponentTransfer>
>     </filter>
>     <filter id="Gamma" filterUnits="objectBoundingBox"
>             x="0%" y="0%" width="100%" height="100%">
>       <feComponentTransfer>
>         <feFuncR type="gamma" amplitude="2" exponent="5" offset="0"/>
>         <feFuncG type="gamma" amplitude="2" exponent="3" offset="0"/>
>         <feFuncB type="gamma" amplitude="2" exponent="1" offset="0"/>
>       </feComponentTransfer>
>     </filter>
>   </defs>
>   <rect fill="none" stroke="blue"
>         x="1" y="1" width="798" height="398"/>
>   <g font-family="Verdana" font-size="75"
>             font-weight="bold" fill="url(#MyGradient)" >
>     <rect x="100" y="0" width="600" height="20" />
>     <text x="100" y="90">Identity</text>
>     <text x="100" y="190" filter="url(#Table)" >TableLookup</text>
>     <text x="100" y="290" filter="url(#Linear)" >LinearFunc</text>
>     <text x="100" y="390" filter="url(#Gamma)" >GammaFunc</text>
>   </g>
> </svg>
> ```
>
> ![Example for feComponentTransfer](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feComponentTransfer.png)
>
> Example for feComponentTransfer
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feComponentTransfer.svg)

<a id="ref-for-elementdef-fefuncg④"></a>

#### <a id="feFuncGElement"></a>9.7.2. Transfer function [feFuncG](#elementdef-fefuncg)

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fefuncg"></a>`feFuncG`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transfer-function-element⑥"></a>

[transfer function element](#transfer-function-element)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement④"></a>

<a id="ref-for-elementdef-script⑤"></a>

<a id="ref-for-AnimateElement④"></a>

<a id="ref-for-TermDescriptiveElement④"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fecomponenttransfer-offset③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-exponent③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-amplitude③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-intercept③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-slope③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues③"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-type③"></a>

  <a id="ref-for-transfer-function-element-attributes①"></a>

  [transfer function element attributes](#transfer-function-element-attributes) — [type](#element-attrdef-fecomponenttransfer-type), [tableValues](#element-attrdef-fecomponenttransfer-tablevalues), [slope](#element-attrdef-fecomponenttransfer-slope), [intercept](#element-attrdef-fecomponenttransfer-intercept), [amplitude](#element-attrdef-fecomponenttransfer-amplitude), [exponent](#element-attrdef-fecomponenttransfer-exponent), [offset](#element-attrdef-fecomponenttransfer-offset)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEFuncGElement](#InterfaceSVGFEFuncGElement)

<a id="ref-for-elementdef-fefuncr⑥"></a>

See [feFuncR](#elementdef-fefuncr) for the definitions of the attribute values.

<a id="ref-for-elementdef-fefuncb④"></a>

#### <a id="feFuncBElement"></a>9.7.3. Transfer function [feFuncB](#elementdef-fefuncb)

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fefuncb"></a>`feFuncB`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transfer-function-element⑦"></a>

[transfer function element](#transfer-function-element)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement⑤"></a>

<a id="ref-for-elementdef-script⑥"></a>

<a id="ref-for-AnimateElement⑤"></a>

<a id="ref-for-TermDescriptiveElement⑤"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fecomponenttransfer-offset④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-exponent④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-amplitude④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-intercept④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-slope④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues④"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-type④"></a>

  <a id="ref-for-transfer-function-element-attributes②"></a>

  [transfer function element attributes](#transfer-function-element-attributes) — [type](#element-attrdef-fecomponenttransfer-type), [tableValues](#element-attrdef-fecomponenttransfer-tablevalues), [slope](#element-attrdef-fecomponenttransfer-slope), [intercept](#element-attrdef-fecomponenttransfer-intercept), [amplitude](#element-attrdef-fecomponenttransfer-amplitude), [exponent](#element-attrdef-fecomponenttransfer-exponent), [offset](#element-attrdef-fecomponenttransfer-offset)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEFuncBElement](#InterfaceSVGFEFuncBElement)

<a id="ref-for-elementdef-fefuncr⑦"></a>

See [feFuncR](#elementdef-fefuncr) for the definitions of the attribute values.

<a id="ref-for-elementdef-fefunca④"></a>

#### <a id="feFuncAElement"></a>9.7.4. Transfer function [feFuncA](#elementdef-fefunca)

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fefunca"></a>`feFuncA`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transfer-function-element⑧"></a>

[transfer function element](#transfer-function-element)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement⑥"></a>

<a id="ref-for-elementdef-script⑦"></a>

<a id="ref-for-AnimateElement⑥"></a>

<a id="ref-for-TermDescriptiveElement⑥"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fecomponenttransfer-offset⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-exponent⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-amplitude⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-intercept⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-slope⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues⑤"></a>

  <a id="ref-for-element-attrdef-fecomponenttransfer-type⑤"></a>

  <a id="ref-for-transfer-function-element-attributes③"></a>

  [transfer function element attributes](#transfer-function-element-attributes) — [type](#element-attrdef-fecomponenttransfer-type), [tableValues](#element-attrdef-fecomponenttransfer-tablevalues), [slope](#element-attrdef-fecomponenttransfer-slope), [intercept](#element-attrdef-fecomponenttransfer-intercept), [amplitude](#element-attrdef-fecomponenttransfer-amplitude), [exponent](#element-attrdef-fecomponenttransfer-exponent), [offset](#element-attrdef-fecomponenttransfer-offset)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEFuncAElement](#InterfaceSVGFEFuncAElement)

<a id="ref-for-elementdef-fefuncr⑧"></a>

See [feFuncR](#elementdef-fefuncr) for the definitions of the attribute values.

<a id="ref-for-elementdef-fecomposite⑦"></a>

### <a id="feCompositeElement"></a>9.8. Filter primitive [feComposite](#elementdef-fecomposite)

<strong>Table 11 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fecomposite"></a>`feComposite`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive②⑨"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement⑦"></a>

<a id="ref-for-elementdef-script⑧"></a>

<a id="ref-for-AnimateElement⑦"></a>

<a id="ref-for-TermDescriptiveElement⑦"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode④"></a>

  <a id="ref-for-propdef-word-spacing④"></a>

  <a id="ref-for-propdef-visibility④"></a>

  <a id="ref-for-propdef-unicode-bidi④"></a>

  <a id="ref-for-TextRenderingProperty④"></a>

  <a id="ref-for-propdef-text-decoration④"></a>

  <a id="ref-for-TextAnchorProperty④"></a>

  <a id="ref-for-StrokeWidthProperty④"></a>

  <a id="ref-for-StrokeOpacityProperty④"></a>

  <a id="ref-for-StrokeMiterlimitProperty④"></a>

  <a id="ref-for-StrokeLinejoinProperty④"></a>

  <a id="ref-for-StrokeLinecapProperty④"></a>

  <a id="ref-for-StrokeDashoffsetProperty④"></a>

  <a id="ref-for-StrokeDasharrayProperty④"></a>

  <a id="ref-for-StrokeProperty⑤"></a>

  <a id="ref-for-StopOpacityProperty④"></a>

  <a id="ref-for-StopColorProperty④"></a>

  <a id="ref-for-ShapeRenderingProperty④"></a>

  <a id="ref-for-PointerEventsProperty④"></a>

  <a id="ref-for-propdef-overflow④"></a>

  <a id="ref-for-propdef-opacity⑦"></a>

  <a id="ref-for-propdef-mask④"></a>

  <a id="ref-for-MarkerStartProperty④"></a>

  <a id="ref-for-MarkerMidProperty④"></a>

  <a id="ref-for-MarkerEndProperty④"></a>

  <a id="ref-for-MarkerProperty④"></a>

  <a id="ref-for-propdef-lighting-color⑤"></a>

  <a id="ref-for-propdef-letter-spacing④"></a>

  <a id="ref-for-KerningProperty④"></a>

  <a id="ref-for-propdef-isolation⑥"></a>

  <a id="ref-for-propdef-image-rendering④"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty④"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty④"></a>

  <a id="ref-for-propdef-font-weight④"></a>

  <a id="ref-for-propdef-font-variant④"></a>

  <a id="ref-for-propdef-font-style④"></a>

  <a id="ref-for-propdef-font-stretch④"></a>

  <a id="ref-for-propdef-font-size-adjust④"></a>

  <a id="ref-for-propdef-font-size④"></a>

  <a id="ref-for-propdef-font-family④"></a>

  <a id="ref-for-propdef-font④"></a>

  <a id="ref-for-propdef-flood-opacity⑥"></a>

  <a id="ref-for-propdef-flood-color⑤"></a>

  <a id="ref-for-propdef-filter①⑤"></a>

  <a id="ref-for-FillRuleProperty④"></a>

  <a id="ref-for-FillOpacityProperty④"></a>

  <a id="ref-for-FillProperty⑥"></a>

  <a id="ref-for-EnableBackgroundProperty④"></a>

  <a id="ref-for-DominantBaselineProperty④"></a>

  <a id="ref-for-propdef-display⑦"></a>

  <a id="ref-for-propdef-direction④"></a>

  <a id="ref-for-propdef-cursor④"></a>

  <a id="ref-for-ColorRenderingProperty④"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⓪"></a>

  <a id="ref-for-ColorInterpolationProperty⑧"></a>

  <a id="ref-for-color0④"></a>

  <a id="ref-for-propdef-clip-rule④"></a>

  <a id="ref-for-propdef-clip-path④"></a>

  <a id="ref-for-propdef-clip④"></a>

  <a id="ref-for-BaselineShiftProperty④"></a>

  <a id="ref-for-AlignmentBaselineProperty④"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x⑦"></a>

  <a id="ref-for-filter-primitive-attributes③"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in⑧"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fecomposite-in2"></a>

  [in2](#element-attrdef-fecomposite-in2)

- <a id="ref-for-element-attrdef-fecomposite-operator"></a>

  [operator](#element-attrdef-fecomposite-operator)

- <a id="ref-for-element-attrdef-fecomposite-k1"></a>

  [k1](#element-attrdef-fecomposite-k1)

- <a id="ref-for-element-attrdef-fecomposite-k2"></a>

  [k2](#element-attrdef-fecomposite-k2)

- <a id="ref-for-element-attrdef-fecomposite-k3"></a>

  [k3](#element-attrdef-fecomposite-k3)

- <a id="ref-for-element-attrdef-fecomposite-k4"></a>

  [k4](#element-attrdef-fecomposite-k4)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFECompositeElement](#InterfaceSVGFECompositeElement)

<a id="ref-for-attr-valuedef-operator-over"></a>

This filter performs the combination of the two input images pixel-wise in image space using one of the Porter-Duff [\[PORTERDUFF\]](#biblio-porterduff) compositing operations: [over](#attr-valuedef-operator-over), in, atop, out, xor, lighter [\[COMPOSITING-1\]](#biblio-compositing-1). Additionally, a component-wise <em>arithmetic</em> operation (with the result clamped between \[0..1\]) can be applied.

<a id="ref-for-elementdef-fediffuselighting②"></a>

<a id="ref-for-elementdef-fespecularlighting③"></a>

The <em>arithmetic</em> operation is useful for combining the output from the [feDiffuseLighting](#elementdef-fediffuselighting) and [feSpecularLighting](#elementdef-fespecularlighting) filters with texture data. It is also useful for implementing <em>dissolve</em>. If the <em>arithmetic</em> operation is chosen, each result pixel is computed using the following formula:

```text
result = k1*i1*i2 + k2*i1 + k3*i2 + k4
```
where:

- <a id="ref-for-element-attrdef-filter-primitive-in⑨"></a>

  <a id="ref-for-element-attrdef-fecomposite-in2①"></a>

  `i1` and `i2` indicate the corresponding pixel channel values of the input image, which map to [in](#element-attrdef-filter-primitive-in) and [in2](#element-attrdef-fecomposite-in2) respectively

- `k1, k2, k3` and `k4` indicate the values of the attributes with the same name

<a id="ref-for-filter-primitive-subregion①④"></a>

For this filter primitive, the extent of the resulting image might grow as described in the section that describes the [filter primitive subregion](#filter-primitive-subregion).

<em>Attribute definitions:</em>

<a id="element-attrdef-fecomposite-operator"></a>`operator` = "<a id="attr-valuedef-operator-over"></a>`over` \| <a id="attr-valuedef-operator-in"></a>`in` \| <a id="attr-valuedef-operator-out"></a>`out` \| <a id="attr-valuedef-operator-atop"></a>`atop` \| <a id="attr-valuedef-operator-xor"></a>`xor` \| <a id="attr-valuedef-operator-lighter"></a>`lighter` \| <a id="attr-valuedef-operator-arithmetic"></a>`arithmetic`"

<a id="ref-for-element-attrdef-fecomposite-operator①"></a>

<a id="ref-for-element-attrdef-filter-primitive-in①⓪"></a>

<a id="ref-for-element-attrdef-fecomposite-in2②"></a>

The compositing operation that is to be performed. All of the [operator](#element-attrdef-fecomposite-operator) types except arithmetic match the corresponding operation as described in [\[COMPOSITING-1\]](#biblio-compositing-1) with [in](#element-attrdef-filter-primitive-in) representing the source and [in2](#element-attrdef-fecomposite-in2) representing the destination. The arithmetic operator is described above.

<a id="ref-for-TermInitialValue③⓪"></a>

<a id="ref-for-element-attrdef-fecomposite-operator②"></a>

<a id="ref-for-attr-valuedef-operator-over①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [operator](#element-attrdef-fecomposite-operator) is [over](#attr-valuedef-operator-over).

Animatable: yes.

<a id="ref-for-number-value①⓪"></a>

<a id="element-attrdef-fecomposite-k1"></a>`k1` = "[\<number\>](https://www.w3.org/TR/css3-values/#number-value)"

Only applicable if `operator="arithmetic"`.

<a id="ref-for-TermInitialValue③①"></a>

<a id="ref-for-element-attrdef-fecomposite-k1①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [k1](#element-attrdef-fecomposite-k1) is 0.

Animatable: yes.

<a id="ref-for-number-value①①"></a>

<a id="element-attrdef-fecomposite-k2"></a>`k2` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

Only applicable if `operator="arithmetic"`.

<a id="ref-for-TermInitialValue③②"></a>

<a id="ref-for-element-attrdef-fecomposite-k2①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [k2](#element-attrdef-fecomposite-k2) is 0.

Animatable: yes.

<a id="ref-for-number-value①②"></a>

<a id="element-attrdef-fecomposite-k3"></a>`k3` = "[\<number\>](https://www.w3.org/TR/css3-values/#number-value)"

Only applicable if `operator="arithmetic"`.

<a id="ref-for-TermInitialValue③③"></a>

<a id="ref-for-element-attrdef-fecomposite-k3①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [k3](#element-attrdef-fecomposite-k3) is 0.

Animatable: yes.

<a id="ref-for-number-value①③"></a>

<a id="element-attrdef-fecomposite-k4"></a>`k4` = "[\<number\>](https://www.w3.org/TR/css3-values/#number-value)"

Only applicable if `operator="arithmetic"`.

<a id="ref-for-TermInitialValue③④"></a>

<a id="ref-for-element-attrdef-fecomposite-k4①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [k4](#element-attrdef-fecomposite-k4) is 0.

Animatable: yes.

<a id="ref-for-element-attrdef-filter-primitive-in①①"></a>

<a id="element-attrdef-fecomposite-in2"></a>`in2` = "<em>(see <a href="#element-attrdef-filter-primitive-in">in</a> attribute)</em>"

The second input image to the compositing operation.

Animatable: yes.

<a id="ref-for-element-attrdef-filter-primitive-in①②"></a>

<a id="ref-for-element-attrdef-fecomposite-in2③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Compositing and Blending [\[COMPOSITING-1\]](#biblio-compositing-1) defines more compositing keywords. The functionality of the additional keywords can be archived by switching the input filter primitives [in](#element-attrdef-filter-primitive-in) and [in2](#element-attrdef-fecomposite-in2).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5847e5f6"></a>
>
> ```text
> <svg width="330" height="195" viewBox="0 0 1100 650"
>      xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
>   <title>Example feComposite - Examples of feComposite operations</title>
>   <desc>Four rows of six pairs of overlapping triangles depicting
>         the six different feComposite operators under different
>         opacity values and different clearing of the background.</desc>
>   <defs>
>     <desc>Define two sets of six filters for each of the six compositing operators.
>           The first set wipes out the background image by flooding with opaque white.
>           The second set does not wipe out the background, with the result
>           that the background sometimes shines through and is other cases
>           is blended into itself (i.e., "double-counting").</desc>
>     <filter id="overFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="over" result="comp"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="inFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="in" result="comp"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="outFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="out" result="comp"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="atopFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="atop" result="comp"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="xorFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="xor" result="comp"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="arithmeticFlood" filterUnits="objectBoundingBox"
>             x="-5%" y="-5%" width="110%" height="110%">
>       <feFlood flood-color="#ffffff" flood-opacity="1" result="flood"/>
>       <feComposite in="SourceGraphic" in2="BackgroundImage" result="comp"
>                    operator="arithmetic" k1=".5" k2=".5" k3=".5" k4=".5"/>
>       <feMerge> <feMergeNode in="flood"/> <feMergeNode in="comp"/> </feMerge>
>     </filter>
>     <filter id="overNoFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="over" result="comp"/>
>     </filter>
>     <filter id="inNoFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="in" result="comp"/>
>     </filter>
>     <filter id="outNoFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="out" result="comp"/>
>     </filter>
>     <filter id="atopNoFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="atop" result="comp"/>
>     </filter>
>     <filter id="xorNoFlood" filterUnits="objectBoundingBox" x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" operator="xor" result="comp"/>
>     </filter>
>     <filter id="arithmeticNoFlood" filterUnits="objectBoundingBox"
>             x="-5%" y="-5%" width="110%" height="110%">
>       <feComposite in="SourceGraphic" in2="BackgroundImage" result="comp"
>                    operator="arithmetic" k1=".5" k2=".5" k3=".5" k4=".5"/>
>     </filter>
>     <path id="Blue100" d="M 0 0 L 100 0 L 100 100 z" fill="#00ffff" />
>     <path id="Red100" d="M 0 0 L 0 100 L 100 0 z" fill="#ff00ff" />
>     <path id="Blue50" d="M 0 125 L 100 125 L 100 225 z" fill="#00ffff" fill-opacity=".5" />
>     <path id="Red50" d="M 0 125 L 0 225 L 100 125 z" fill="#ff00ff" fill-opacity=".5" />
>     <g id="TwoBlueTriangles">
>       <use xlink:href="#Blue100"/>
>       <use xlink:href="#Blue50"/>
>     </g>
>     <g id="BlueTriangles">
>       <use transform="translate(275,25)" xlink:href="#TwoBlueTriangles"/>
>       <use transform="translate(400,25)" xlink:href="#TwoBlueTriangles"/>
>       <use transform="translate(525,25)" xlink:href="#TwoBlueTriangles"/>
>       <use transform="translate(650,25)" xlink:href="#TwoBlueTriangles"/>
>       <use transform="translate(775,25)" xlink:href="#TwoBlueTriangles"/>
>       <use transform="translate(900,25)" xlink:href="#TwoBlueTriangles"/>
>     </g>
>   </defs>
> 
>   <rect fill="none" stroke="blue" x="1" y="1" width="1098" height="648"/>
>   <g font-family="Verdana" font-size="40" shape-rendering="crispEdges">
>     <desc>Render the examples using the filters that draw on top of
>           an opaque white surface, thus obliterating the background.</desc>
>     <g isolation="isolate">
>       <text x="15" y="75">opacity 1.0</text>
>       <text x="15" y="115" font-size="27">(with feFlood)</text>
>       <text x="15" y="200">opacity 0.5</text>
>       <text x="15" y="240" font-size="27">(with feFlood)</text>
>       <use xlink:href="#BlueTriangles"/>
>       <g transform="translate(275,25)">
>         <use xlink:href="#Red100" filter="url(#overFlood)" />
>         <use xlink:href="#Red50" filter="url(#overFlood)" />
>         <text x="5" y="275">over</text>
>       </g>
>       <g transform="translate(400,25)">
>         <use xlink:href="#Red100" filter="url(#inFlood)" />
>         <use xlink:href="#Red50" filter="url(#inFlood)" />
>         <text x="35" y="275">in</text>
>       </g>
>       <g transform="translate(525,25)">
>         <use xlink:href="#Red100" filter="url(#outFlood)" />
>         <use xlink:href="#Red50" filter="url(#outFlood)" />
>         <text x="15" y="275">out</text>
>       </g>
>       <g transform="translate(650,25)">
>         <use xlink:href="#Red100" filter="url(#atopFlood)" />
>         <use xlink:href="#Red50" filter="url(#atopFlood)" />
>         <text x="10" y="275">atop</text>
>       </g>
>       <g transform="translate(775,25)">
>         <use xlink:href="#Red100" filter="url(#xorFlood)" />
>         <use xlink:href="#Red50" filter="url(#xorFlood)" />
>         <text x="15" y="275">xor</text>
>       </g>
>       <g transform="translate(900,25)">
>         <use xlink:href="#Red100" filter="url(#arithmeticFlood)" />
>         <use xlink:href="#Red50" filter="url(#arithmeticFlood)" />
>         <text x="-25" y="275">arithmetic</text>
>       </g>
>     </g>
>     <g transform="translate(0,325)" isolation="isolate">
>     <desc>Render the examples using the filters that do not obliterate
>           the background, thus sometimes causing the background to continue
>           to appear in some cases, and in other cases the background
>           image blends into itself ("double-counting").</desc>
>       <text x="15" y="75">opacity 1.0</text>
>       <text x="15" y="115" font-size="27">(without feFlood)</text>
>       <text x="15" y="200">opacity 0.5</text>
>       <text x="15" y="240" font-size="27">(without feFlood)</text>
>       <use xlink:href="#BlueTriangles"/>
>       <g transform="translate(275,25)">
>         <use xlink:href="#Red100" filter="url(#overNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#overNoFlood)" />
>         <text x="5" y="275">over</text>
>       </g>
>       <g transform="translate(400,25)">
>         <use xlink:href="#Red100" filter="url(#inNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#inNoFlood)" />
>         <text x="35" y="275">in</text>
>       </g>
>       <g transform="translate(525,25)">
>         <use xlink:href="#Red100" filter="url(#outNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#outNoFlood)" />
>         <text x="15" y="275">out</text>
>       </g>
>       <g transform="translate(650,25)">
>         <use xlink:href="#Red100" filter="url(#atopNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#atopNoFlood)" />
>         <text x="10" y="275">atop</text>
>       </g>
>       <g transform="translate(775,25)">
>         <use xlink:href="#Red100" filter="url(#xorNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#xorNoFlood)" />
>         <text x="15" y="275">xor</text>
>       </g>
>       <g transform="translate(900,25)">
>         <use xlink:href="#Red100" filter="url(#arithmeticNoFlood)" />
>         <use xlink:href="#Red50" filter="url(#arithmeticNoFlood)" />
>         <text x="-25" y="275">arithmetic</text>
>       </g>
>     </g>
>   </g>
> </svg>
> ```
>
> ![Example of feComposite](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feComposite.png)
>
> Example of feComposite
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feComposite.svg)

<a id="ref-for-elementdef-feconvolvematrix③"></a>

### <a id="feConvolveMatrixElement"></a>9.9. Filter primitive [feConvolveMatrix](#elementdef-feconvolvematrix)

<strong>Table 12 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feconvolvematrix"></a>`feConvolveMatrix`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⓪"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement⑧"></a>

<a id="ref-for-elementdef-script⑨"></a>

<a id="ref-for-AnimateElement⑧"></a>

<a id="ref-for-TermDescriptiveElement⑧"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode⑤"></a>

  <a id="ref-for-propdef-word-spacing⑤"></a>

  <a id="ref-for-propdef-visibility⑤"></a>

  <a id="ref-for-propdef-unicode-bidi⑤"></a>

  <a id="ref-for-TextRenderingProperty⑤"></a>

  <a id="ref-for-propdef-text-decoration⑤"></a>

  <a id="ref-for-TextAnchorProperty⑤"></a>

  <a id="ref-for-StrokeWidthProperty⑤"></a>

  <a id="ref-for-StrokeOpacityProperty⑤"></a>

  <a id="ref-for-StrokeMiterlimitProperty⑤"></a>

  <a id="ref-for-StrokeLinejoinProperty⑤"></a>

  <a id="ref-for-StrokeLinecapProperty⑤"></a>

  <a id="ref-for-StrokeDashoffsetProperty⑤"></a>

  <a id="ref-for-StrokeDasharrayProperty⑤"></a>

  <a id="ref-for-StrokeProperty⑥"></a>

  <a id="ref-for-StopOpacityProperty⑤"></a>

  <a id="ref-for-StopColorProperty⑤"></a>

  <a id="ref-for-ShapeRenderingProperty⑤"></a>

  <a id="ref-for-PointerEventsProperty⑤"></a>

  <a id="ref-for-propdef-overflow⑤"></a>

  <a id="ref-for-propdef-opacity⑧"></a>

  <a id="ref-for-propdef-mask⑤"></a>

  <a id="ref-for-MarkerStartProperty⑤"></a>

  <a id="ref-for-MarkerMidProperty⑤"></a>

  <a id="ref-for-MarkerEndProperty⑤"></a>

  <a id="ref-for-MarkerProperty⑤"></a>

  <a id="ref-for-propdef-lighting-color⑥"></a>

  <a id="ref-for-propdef-letter-spacing⑤"></a>

  <a id="ref-for-KerningProperty⑤"></a>

  <a id="ref-for-propdef-isolation⑦"></a>

  <a id="ref-for-propdef-image-rendering⑤"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty⑤"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty⑤"></a>

  <a id="ref-for-propdef-font-weight⑤"></a>

  <a id="ref-for-propdef-font-variant⑤"></a>

  <a id="ref-for-propdef-font-style⑤"></a>

  <a id="ref-for-propdef-font-stretch⑤"></a>

  <a id="ref-for-propdef-font-size-adjust⑤"></a>

  <a id="ref-for-propdef-font-size⑤"></a>

  <a id="ref-for-propdef-font-family⑤"></a>

  <a id="ref-for-propdef-font⑤"></a>

  <a id="ref-for-propdef-flood-opacity⑦"></a>

  <a id="ref-for-propdef-flood-color⑥"></a>

  <a id="ref-for-propdef-filter①⑥"></a>

  <a id="ref-for-FillRuleProperty⑤"></a>

  <a id="ref-for-FillOpacityProperty⑤"></a>

  <a id="ref-for-FillProperty⑦"></a>

  <a id="ref-for-EnableBackgroundProperty⑤"></a>

  <a id="ref-for-DominantBaselineProperty⑤"></a>

  <a id="ref-for-propdef-display⑧"></a>

  <a id="ref-for-propdef-direction⑤"></a>

  <a id="ref-for-propdef-cursor⑤"></a>

  <a id="ref-for-ColorRenderingProperty⑤"></a>

  <a id="ref-for-propdef-color-interpolation-filters①①"></a>

  <a id="ref-for-ColorInterpolationProperty⑨"></a>

  <a id="ref-for-color0⑤"></a>

  <a id="ref-for-propdef-clip-rule⑤"></a>

  <a id="ref-for-propdef-clip-path⑤"></a>

  <a id="ref-for-propdef-clip⑤"></a>

  <a id="ref-for-BaselineShiftProperty⑤"></a>

  <a id="ref-for-AlignmentBaselineProperty⑤"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x⑧"></a>

  <a id="ref-for-filter-primitive-attributes④"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in①③"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-order"></a>

  [order](#element-attrdef-order)

- <a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix"></a>

  [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix)

- <a id="ref-for-element-attrdef-feconvolvematrix-divisor"></a>

  [divisor](#element-attrdef-feconvolvematrix-divisor)

- <a id="ref-for-element-attrdef-feconvolvematrix-bias"></a>

  [bias](#element-attrdef-feconvolvematrix-bias)

- <a id="ref-for-element-attrdef-feconvolvematrix-targetx"></a>

  [targetX](#element-attrdef-feconvolvematrix-targetx)

- <a id="ref-for-element-attrdef-feconvolvematrix-targety"></a>

  [targetY](#element-attrdef-feconvolvematrix-targety)

- <a id="ref-for-element-attrdef-feconvolvematrix-edgemode"></a>

  [edgeMode](#element-attrdef-feconvolvematrix-edgemode)

- <a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength"></a>

  [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength)

- <a id="ref-for-element-attrdef-feconvolvematrix-preservealpha"></a>

  [preserveAlpha](#element-attrdef-feconvolvematrix-preservealpha)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEConvolveMatrixElement](#InterfaceSVGFEConvolveMatrixElement)

feConvolveMatrix applies a matrix convolution filter effect. A convolution combines pixels in the input image with neighboring pixels to produce a resulting image. A wide variety of imaging operations can be achieved through convolutions, including blurring, edge detection, sharpening, embossing and beveling.

A matrix convolution is based on an n-by-m matrix (the convolution kernel) which describes how a given pixel value in the input image is combined with its neighboring pixel values to produce a resulting pixel value. Each result pixel is determined by applying the kernel matrix to the corresponding source pixel and its neighboring pixels. The basic convolution formula which is applied to each color value for a given pixel is:

<a id="feConvolveMatrixElementFormula"></a>

<strong>Mathematical expression 7</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
{color}_{X,Y} = {\frac{{\sum\limits_{i = 0}^{{orderY} - 1}{\sum\limits_{j = 0}^{{orderX} - 1}{source}_{{{x - {targetX}} + j},{{y - \mathit{targetY}} + i}}}} \cdot {kernalMatrix}_{{{{orderX} - j} - 1,}{{{orderY} - i} - 1}}}{divisor} + {{bias} \cdot {alpha}_{x,y}}}
```

Source mathematical tokens: color X , Y = ∑ i = 0 orderY − 1 ∑ j = 0 orderX − 1 source x − targetX + j , y − targetY + i ⋅ kernalMatrix orderX − j − 1, orderY − i − 1 divisor + bias ⋅ alpha x , y func color\_{X , Y} = { sum from{i=0} to{func orderY -1} sum from{j=0} to{func orderX -1} func source\_{x - func targetX + j , y - targetY + i} cdot func kernalMatrix\_{func orderX - j - 1, func orderY - i - 1 }} over {func divisor} + func bias cdot func alpha\_{x,y}

<a id="ref-for-element-attrdef-order①"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targetx①"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targety①"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix①"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-divisor①"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-bias①"></a>

where "orderX" and "orderY" represent the X and Y values for the [order](#element-attrdef-order) attribute, "targetX" represents the value of the [targetX](#element-attrdef-feconvolvematrix-targetx) attribute, "targetY" represents the value of the [targetY](#element-attrdef-feconvolvematrix-targety) attribute, "kernelMatrix" represents the value of the [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix) attribute, "divisor" represents the value of the [divisor](#element-attrdef-feconvolvematrix-divisor) attribute, and "bias" represents the value of the [bias](#element-attrdef-feconvolvematrix-bias) attribute.

In the above formulas the values in the kernel matrix are applied such that the kernel matrix is rotated 180 degrees relative to the source and destination images in order to match convolution theory as described in many computer graphics textbooks.

To illustrate, suppose you have a input image which is 5 pixels by 5 pixels, whose color values for one of the color channels are as follows:

<strong>Mathematical expression 8</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
0 & 20 & 40 & 235 & 235 \\
100 & 120 & 140 & 235 & 235 \\
200 & 220 & 240 & 235 & 235 \\
255 & 255 & 255 & 255 & 255 \\
255 & 255 & 255 & 255 & 255
\end{bmatrix}
```

Source mathematical tokens: 0 20 40 235 235 100 120 140 235 235 200 220 240 235 235 255 255 255 255 255 255 255 255 255 255 left\[ matrix {0 \# 20 \# 40 \# 235 \# 235 \## 100 \# 120 \# 140 \# 235 \# 235 \## 200 \# 220 \# 240 \# 235 \# 235 \## 255 \# 255 \# 255 \# 255 \# 255 \## 255 \# 255 \# 255 \# 255 \# 255} right\]

and you define a 3-by-3 convolution kernel as follows:

<strong>Mathematical expression 9</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
1 & 2 & 3 \\
4 & 5 & 6 \\
7 & 8 & 9
\end{bmatrix}
```

Source mathematical tokens: 1 2 3 4 5 6 7 8 9 left\[ matrix {1 \# 2 \# 3 \## 4 \# 5 \# 6 \## 7 \# 8 \# 9} right\]

<a id="ref-for-element-attrdef-feconvolvematrix-divisor②"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targetx②"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targety②"></a>

Let’s focus on the color value at the second row and second column of the image (source pixel value is 120). Assuming the simplest case (where the input image’s pixel grid aligns perfectly with the kernel’s pixel grid) and assuming default values for attributes [divisor](#element-attrdef-feconvolvematrix-divisor), [targetX](#element-attrdef-feconvolvematrix-targetx) and [targetY](#element-attrdef-feconvolvematrix-targety), then resulting color value will be:

<strong>Mathematical expression 10</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
{resultChannel}_{2,2} = \frac{{{{{{{{{9 \cdot 0} + {8 \cdot 20}} + {7 \cdot 40}} + {6 \cdot 100}} + {5 \cdot 120}} + {4 \cdot 140}} + {3 \cdot 200}} + {2 \cdot 220}} + {1 \cdot 240}}{{{{{{{{9 + 8} + 7} + 6} + 5} + 4} + 3} + 2} + 1}
```

Source mathematical tokens: resultChannel 2,2 = 9 ⋅ 0 + 8 ⋅ 20 + 7 ⋅ 40 + 6 ⋅ 100 + 5 ⋅ 120 + 4 ⋅ 140 + 3 ⋅ 200 + 2 ⋅ 220 + 1 ⋅ 240 9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1 func resultChannel_2,2 = {{9 cdot 0} + {8 cdot 20} + {7 cdot 40} + {6 cdot 100} + {5 cdot 120} + {4 cdot 140} + {3 cdot 200} + {2 cdot 220} + {1 cdot 240}} over {9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1}

<a id="ref-for-elementdef-feconvolvematrix④"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength①"></a>

Because they operate on pixels, matrix convolutions are inherently resolution-dependent. To make [feConvolveMatrix](#elementdef-feconvolvematrix) produce resolution-independent results, an explicit value should be provided for the attribute [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength).

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength②"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits⑨"></a>

[kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength), in combination with the other attributes, defines an implicit pixel grid in the filter effects coordinate system (i.e., the coordinate system established by the [primitiveUnits](#element-attrdef-filter-primitiveunits) attribute). The input image will be temporarily rescaled to match its pixels with <a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength③"></a>kernelUnitLength. The convolution happens on the resampled image. After applying the convolution, the image is resampled back to the original resolution.

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength④"></a>

<a id="ref-for-propdef-image-rendering⑥"></a>

When the image must be resampled to match the coordinate system defined by [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength) prior to convolution, or resampled to match the device coordinate system after convolution, it is recommended that high quality viewers make use of appropriate interpolation techniques, for example bilinear or bicubic. Depending on the speed of the available interpolents, this choice may be affected by the [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting. Note that implementations might choose approaches that minimize or eliminate resampling when not necessary to produce proper results, such as when the document is zoomed out such that <a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength⑤"></a>kernelUnitLength is considerably smaller than a device pixel.

<em>Attribute definitions:</em>

<a id="ref-for-typedef-number-optional-number②"></a>

<a id="element-attrdef-order"></a>`order` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix②"></a>

<a id="ref-for-integer-value"></a>

Indicates the number of cells in each dimension for [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix). The values provided must be [\<integer\>](https://www.w3.org/TR/css3-values/#integer-value) s greater than zero. Values that are not integers will be truncated, i.e. rounded to the closest integer value towards zero. The first number, \<orderX\>, indicates the number of columns in the matrix. The second number, \<orderY\>, indicates the number of rows in the matrix. If \<orderY\> is not provided, it defaults to \<orderX\>.

It is recommended that only small values (e.g., 3) be used; higher values may result in very high CPU overhead and usually do not produce results that justify the impact on performance.

<a id="ref-for-TermInitialValue③⑤"></a>

<a id="ref-for-element-attrdef-order②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [order](#element-attrdef-order) is 3.

Animatable: yes.

<a id="element-attrdef-feconvolvematrix-kernelmatrix"></a>`kernelMatrix` = "<em>&lt;list of numbers&gt;</em>"

<a id="ref-for-number-value①④"></a>

The list of [\<number\>](https://www.w3.org/TR/css3-values/#number-value) s that make up the kernel matrix for the convolution. Values are separated by space characters and/or a comma. The number of entries in the list must equal \<orderX\> times \<orderY\>.

<a id="ref-for-pass-through-filter①"></a>

If the result of `orderX * orderY` is not equal to the the number of entries in the value list, the filter primitive acts as a [pass through filter](#pass-through-filter).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4fd9d6f6"></a> How to behave on invalid number of entries in the value list? [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;237&#x3E;](https://github.com/w3c/csswg-drafts/issues/237)

Animatable: yes.

<a id="ref-for-number-value①⑤"></a>

<a id="element-attrdef-feconvolvematrix-divisor"></a>`divisor` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix③"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-divisor③"></a>

After applying the [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix) to the input image to yield a number, that number is divided by [divisor](#element-attrdef-feconvolvematrix-divisor) to yield the final destination color value. A divisor that is the sum of all the matrix values tends to have an evening effect on the overall color intensity of the result. If the specified divisor is 0 then the default value will be used instead.

<a id="ref-for-TermInitialValue③⑥"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix④"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) is the sum of all values in [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix), with the exception that if the sum is zero, then the divisor is set to 1.

Animatable: yes.

<a id="ref-for-number-value①⑥"></a>

<a id="element-attrdef-feconvolvematrix-bias"></a>`bias` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix⑤"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-divisor④"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-bias②"></a>

After applying the [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix) to the input image to yield a number and applying the [divisor](#element-attrdef-feconvolvematrix-divisor), the [bias](#element-attrdef-feconvolvematrix-bias) attribute is added to each component. One application of <a id="ref-for-element-attrdef-feconvolvematrix-bias③"></a>bias is when it is desirable to have .5 gray value be the zero response of the filter. The bias property shifts the range of the filter. This allows representation of values that would otherwise be clamped to 0 or 1.

<a id="ref-for-TermInitialValue③⑦"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-bias④"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [bias](#element-attrdef-feconvolvematrix-bias) is 0.

Animatable: yes.

<a id="ref-for-integer-value①"></a>

<a id="element-attrdef-feconvolvematrix-targetx"></a>`targetX` = "[\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)"

Determines the positioning in X of the convolution matrix relative to a given target pixel in the input image. The leftmost column of the matrix is column number zero. The value must be such that: 0 \<= targetX \< orderX. By default, the convolution matrix is centered in X over each pixel of the input image (i.e., targetX = floor ( orderX / 2 )).

Animatable: yes.

<a id="ref-for-integer-value②"></a>

<a id="element-attrdef-feconvolvematrix-targety"></a>`targetY` = "[\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)"

Determines the positioning in Y of the convolution matrix relative to a given target pixel in the input image. The topmost row of the matrix is row number zero. The value must be such that: 0 \<= targetY \< orderY. By default, the convolution matrix is centered in Y over each pixel of the input image (i.e., targetY = floor ( orderY / 2 )).

Animatable: yes.

<a id="ref-for-attr-valuedef-edgemode-wrap"></a>

<a id="ref-for-attr-valuedef-edgemode-duplicate"></a>

<a id="element-attrdef-feconvolvematrix-edgemode"></a>`edgeMode` = "[duplicate](#attr-valuedef-edgemode-duplicate) \| [wrap](#attr-valuedef-edgemode-wrap) \| none"

Determines how to extend the input image as necessary with color values so that the matrix operations can be applied when the kernel is positioned at or near the edge of the input image.

duplicate indicates that the input image is extended along each of its borders as necessary by duplicating the color values at the given edge of the input image.

Original N-by-M image, where m=M-1 and n=N-1:

<strong>Mathematical expression 11</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
11 & 12 & {\text{.}\text{.}\text{.}} & {1m} & {1M} \\
21 & 22 & {\text{.}\text{.}\text{.}} & {2m} & {2M} \\
{\text{.}\text{.}\text{.}} & {\text{.}\text{.}\text{.}} & {\text{.}\text{.}\text{.}} & {\text{.}\text{.}\text{.}} & {\text{.}\text{.}\text{.}} \\
{n1} & {n2} & {\text{.}\text{.}\text{.}} & {nm} & {nM} \\
{N1} & {N2} & {\text{.}\text{.}\text{.}} & {Nm} & {NM}
\end{bmatrix}
```

Source mathematical tokens: 11 12 . . . 1 m 1 M 21 22 . . . 2 m 2 M . . . . . . . . . . . . . . . n 1 n 2 . . . nm nM N 1 N 2 . . . Nm NM left\[ matrix {11 \# 12 \# { "." "." "."} \# {1 m} \# {1 M} \## 21 \# 22 \# { "." "." "."} \# {2 m} \# {2 M} \## { "." "." "."} \# { "." "." "."} \# { "." "." "."} \# { "." "." "."} \# { "." "." "."} \## {n 1} \# {n 2} \# { "." "." "."} \# func nm \# func nM \## {N 1} \# {N 2} \# { "." "." "."} \# func Nm \# func NM} right\]

Extended by two pixels using duplicate:

<strong>Mathematical expression 12</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
11 & 11 & 11 & 12 & \ldots & {1m} & {1M} & {1M} & {1M} \\
11 & 11 & 11 & 12 & \ldots & {1m} & {1M} & {1M} & {1M} \\
11 & 11 & 11 & 12 & \ldots & {1m} & {1M} & {1M} & {1M} \\
21 & 21 & 21 & 22 & \ldots & {2m} & {2M} & {2M} & {2M} \\
\ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots \\
{n1} & {n1} & {n1} & {n2} & \ldots & {nm} & {nM} & {nM} & {nM} \\
{N1} & {N1} & {N1} & {N2} & \ldots & {Nm} & {NM} & {NM} & {NM} \\
{N1} & {N1} & {N1} & {N2} & \ldots & {Nm} & {NM} & {NM} & {NM} \\
{N1} & {N1} & {N1} & {N2} & \ldots & {Nm} & {NM} & {NM} & {NM}
\end{bmatrix}
```

Source mathematical tokens: 11 11 11 12 … 1 m 1 M 1 M 1 M 11 11 11 12 … 1 m 1 M 1 M 1 M 11 11 11 12 … 1 m 1 M 1 M 1 M 21 21 21 22 … 2 m 2 M 2 M 2 M … … … … … … … … … n 1 n 1 n 1 n 2 … nm nM nM nM N 1 N 1 N 1 N 2 … Nm NM NM NM N 1 N 1 N 1 N 2 … Nm NM NM NM N 1 N 1 N 1 N 2 … Nm NM NM NM left\[ matrix {11 \# 11 \# 11 \# 12 \# dotslow \# {1 m} \# {1 M} \# {1 M} \# {1 M} \## 11 \# 11 \# 11 \# 12 \# dotslow \# {1 m} \# {1 M} \# {1 M} \# {1 M} \## 11 \# 11 \# 11 \# 12 \# dotslow \# {1 m} \# {1 M} \# {1 M} \# {1 M} \## 21 \# 21 \# 21 \# 22 \# dotslow \# {2 m} \# {2 M} \# {2 M} \# {2 M} \## dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \## {n 1} \# {n 1} \# {n 1} \# {n 2} \# dotslow \# func nm \# func nM \# func nM \# func nM \## {N 1} \# {N 1} \# {N 1} \# {N 2} \# dotslow \# func Nm \# func NM \# func NM \# func NM \## {N 1} \# {N 1} \# {N 1} \# {N 2} \# dotslow \# func Nm \# func NM \# func NM \# func NM \## {N 1} \# {N 1} \# {N 1} \# {N 2} \# dotslow \# func Nm \# func NM \# func NM \# func NM} right\]

<a id="ref-for-attr-valuedef-edgemode-wrap①"></a>

[wrap](#attr-valuedef-edgemode-wrap) indicates that the input image is extended by taking the color values from the opposite edge of the image.

<a id="ref-for-attr-valuedef-edgemode-wrap②"></a>

Extended by two pixels using [wrap](#attr-valuedef-edgemode-wrap):

<strong>Mathematical expression 13</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\begin{bmatrix}
\mathit{nm} & \mathit{nM} & \mathit{n1} & \mathit{n2} & \ldots & \mathit{nm} & \mathit{nM} & \mathit{n1} & \mathit{n2} \\
\mathit{Nm} & \mathit{NM} & \mathit{N1} & \mathit{N2} & \ldots & \mathit{Nm} & \mathit{NM} & \mathit{N1} & \mathit{N2} \\
1m & 1M & 11 & 12 & \ldots & 1m & 1M & 11 & 12 \\
2m & 2M & 21 & 22 & \ldots & 2m & 2M & 21 & 22 \\
\ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots & \ldots \\
\mathit{nm} & \mathit{nM} & \mathit{n1} & \mathit{n2} & \ldots & \mathit{nm} & \mathit{nM} & \mathit{n1} & \mathit{n2} \\
\mathit{Nm} & \mathit{NM} & \mathit{N1} & \mathit{N2} & \ldots & \mathit{Nm} & \mathit{NM} & \mathit{N1} & \mathit{N2} \\
1m & 1M & 11 & 12 & \ldots & 1m & 1M & 11 & 12 \\
2m & 2M & 21 & 22 & \ldots & 2m & 2M & 21 & 22
\end{bmatrix}
```

Source mathematical tokens: nm nM n1 n2 … nm nM n1 n2 Nm NM N1 N2 … Nm NM N1 N2 1m 1M 11 12 … 1m 1M 11 12 2m 2M 21 22 … 2m 2M 21 22 … … … … … … … … … nm nM n1 n2 … nm nM n1 n2 Nm NM N1 N2 … Nm NM N1 N2 1m 1M 11 12 … 1m 1M 11 12 2m 2M 21 22 … 2m 2M 21 22 left\[ matrix { nm \# nM \# n1 \# n2 \# dotslow \# nm \# nM \# n1 \# n2 \## Nm \# NM \# N1 \# N2 \# dotslow \# Nm \# NM \# N1 \# N2 \## 1m \# 1M \# 11 \# 12 \# dotslow \# 1m \# 1M \# 11 \# 12 \## 2m \# 2M \# 21 \# 22 \# dotslow \# 2m \# 2M \# 21 \# 22 \## dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \# dotslow \## nm \# nM \# n1 \# n2 \# dotslow \# nm \# nM \# n1 \# n2 \## Nm \# NM \# N1 \# N2 \# dotslow \# Nm \# NM \# N1 \# N2 \## 1m \# 1M \# 11 \# 12 \# dotslow \# 1m \# 1M \# 11 \# 12 \## 2m \# 2M \# 21 \# 22 \# dotslow \# 2m \# 2M \# 21 \# 22 } right\]

The value none indicates that the input image is extended with pixel values of zero for R, G, B and A.

<a id="ref-for-TermInitialValue③⑧"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-edgemode①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [edgeMode](#element-attrdef-feconvolvematrix-edgemode) is duplicate.

Animatable: yes.

<a id="ref-for-typedef-number-optional-number③"></a>

<a id="element-attrdef-feconvolvematrix-kernelunitlength"></a>`kernelUnitLength` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits①⓪"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix⑥"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength⑥"></a>

The first number is the \<dx\> value. The second number is the \<dy\> value. If the \<dy\> value is not specified, it defaults to the same value as \<dx\>. Indicates the intended distance in current filter units (i.e., units as determined by the value of attribute [primitiveUnits](#element-attrdef-filter-primitiveunits)) between successive columns and rows, respectively, in the [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix). By specifying value(s) for [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength), the kernel becomes defined in a scalable, abstract coordinate system. If <a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength⑦"></a>kernelUnitLength is not specified, the default value is one pixel in the offscreen bitmap, which is a pixel-based coordinate system, and thus potentially not scalable. For some level of consistency across display media and user agents, it is necessary that a value be provided for <a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength⑧"></a>kernelUnitLength. In some implementations, the most consistent results and the fastest performance will be achieved if the pixel grid of the temporary off-screen images aligns with the pixel grid of the kernel.

If a negative or zero value is specified the default value will be used instead.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This attribute is deprecated and will be removed. It does not provide a reliable way to create platform independent results. Future versions of this specification will cover this use case.

Animatable: yes.

<a id="element-attrdef-feconvolvematrix-preservealpha"></a>`preserveAlpha` = "<em>false | true</em>"

<a id="ref-for-valdef-custom-media-false"></a>

A value of [false](https://drafts.csswg.org/mediaqueries-5/#valdef-custom-media-false) indicates that the convolution will apply to all channels, including the alpha channel. In this case the <code>ALPHA<sub>X,Y</sub></code> of the [convolution formula](#feConvolveMatrixElementFormula) for a given pixel is:

<a id="ref-for-element-attrdef-order⑤"></a>

<a id="ref-for-element-attrdef-order⑥"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targetx③"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-targety③"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix⑦"></a>

<a id="ref-for-element-attrdef-order⑦"></a>

<a id="ref-for-element-attrdef-order⑧"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-divisor⑤"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-bias⑤"></a>

<code> ALPHA<sub>X,Y</sub>&#xA0;=&#xA0;(&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;SUM <sub>I=0&#xA0;to&#xA0;&#x5B;<a href="#element-attrdef-order">orderY</a>-1&#x5D;</sub>&#xA0;{&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;SUM <sub>J=0&#xA0;to&#xA0;&#x5B;<a href="#element-attrdef-order">orderX</a>-1&#x5D;</sub>&#xA0;{&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;SOURCE <sub>X-<a href="#element-attrdef-feconvolvematrix-targetx">targetX</a>+J,&#xA0;Y-<a href="#element-attrdef-feconvolvematrix-targety">targetY</a>+I</sub>&#xA0;&#x2A;&#xA0; <a href="#element-attrdef-feconvolvematrix-kernelmatrix">kernelMatrix</a><sub><a href="#element-attrdef-order">orderX</a>-J-1,&#xA0; <a href="#element-attrdef-order">orderY</a>-I-1</sub>&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;}&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;}&#xA0;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;)&#xA0;/&#xA0; <a href="#element-attrdef-feconvolvematrix-divisor">divisor</a>&#xA0;+&#xA0; <a href="#element-attrdef-feconvolvematrix-bias">bias</a>&#xA0;<br> </code>

A value of "true" indicates that the convolution will only apply to the color channels. In this case, the filter will temporarily unpremultiply the color component values and apply the kernel. In this case the <code>ALPHA<sub>X,Y</sub></code> of the [convolution formula](#feConvolveMatrixElementFormula) for a given pixel is:

<code>ALPHA<sub>X,Y</sub>&#xA0;=&#xA0;SOURCE<sub>X,Y</sub></code>

<a id="ref-for-TermInitialValue③⑨"></a>

<a id="ref-for-element-attrdef-feconvolvematrix-preservealpha①"></a>

<a id="ref-for-valdef-custom-media-false①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [preserveAlpha](#element-attrdef-feconvolvematrix-preservealpha) is [false](https://drafts.csswg.org/mediaqueries-5/#valdef-custom-media-false).

Animatable: yes.

<a id="ref-for-elementdef-fediffuselighting③"></a>

### <a id="feDiffuseLightingElement"></a>9.10. Filter primitive [feDiffuseLighting](#elementdef-fediffuselighting)

<strong>Table 13 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fediffuselighting"></a>`feDiffuseLighting`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③①"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-light-source①"></a>

<a id="ref-for-elementdef-script①⓪"></a>

<a id="ref-for-TermDescriptiveElement⑨"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script) and exactly one [light sources](#light-source) element, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode⑥"></a>

  <a id="ref-for-propdef-word-spacing⑥"></a>

  <a id="ref-for-propdef-visibility⑥"></a>

  <a id="ref-for-propdef-unicode-bidi⑥"></a>

  <a id="ref-for-TextRenderingProperty⑥"></a>

  <a id="ref-for-propdef-text-decoration⑥"></a>

  <a id="ref-for-TextAnchorProperty⑥"></a>

  <a id="ref-for-StrokeWidthProperty⑥"></a>

  <a id="ref-for-StrokeOpacityProperty⑥"></a>

  <a id="ref-for-StrokeMiterlimitProperty⑥"></a>

  <a id="ref-for-StrokeLinejoinProperty⑥"></a>

  <a id="ref-for-StrokeLinecapProperty⑥"></a>

  <a id="ref-for-StrokeDashoffsetProperty⑥"></a>

  <a id="ref-for-StrokeDasharrayProperty⑥"></a>

  <a id="ref-for-StrokeProperty⑦"></a>

  <a id="ref-for-StopOpacityProperty⑥"></a>

  <a id="ref-for-StopColorProperty⑥"></a>

  <a id="ref-for-ShapeRenderingProperty⑥"></a>

  <a id="ref-for-PointerEventsProperty⑥"></a>

  <a id="ref-for-propdef-overflow⑥"></a>

  <a id="ref-for-propdef-opacity⑨"></a>

  <a id="ref-for-propdef-mask⑥"></a>

  <a id="ref-for-MarkerStartProperty⑥"></a>

  <a id="ref-for-MarkerMidProperty⑥"></a>

  <a id="ref-for-MarkerEndProperty⑥"></a>

  <a id="ref-for-MarkerProperty⑥"></a>

  <a id="ref-for-propdef-lighting-color⑦"></a>

  <a id="ref-for-propdef-letter-spacing⑥"></a>

  <a id="ref-for-KerningProperty⑥"></a>

  <a id="ref-for-propdef-isolation⑧"></a>

  <a id="ref-for-propdef-image-rendering⑦"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty⑥"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty⑥"></a>

  <a id="ref-for-propdef-font-weight⑥"></a>

  <a id="ref-for-propdef-font-variant⑥"></a>

  <a id="ref-for-propdef-font-style⑥"></a>

  <a id="ref-for-propdef-font-stretch⑥"></a>

  <a id="ref-for-propdef-font-size-adjust⑥"></a>

  <a id="ref-for-propdef-font-size⑥"></a>

  <a id="ref-for-propdef-font-family⑥"></a>

  <a id="ref-for-propdef-font⑥"></a>

  <a id="ref-for-propdef-flood-opacity⑧"></a>

  <a id="ref-for-propdef-flood-color⑦"></a>

  <a id="ref-for-propdef-filter①⑦"></a>

  <a id="ref-for-FillRuleProperty⑥"></a>

  <a id="ref-for-FillOpacityProperty⑥"></a>

  <a id="ref-for-FillProperty⑧"></a>

  <a id="ref-for-EnableBackgroundProperty⑥"></a>

  <a id="ref-for-DominantBaselineProperty⑥"></a>

  <a id="ref-for-propdef-display⑨"></a>

  <a id="ref-for-propdef-direction⑥"></a>

  <a id="ref-for-propdef-cursor⑥"></a>

  <a id="ref-for-ColorRenderingProperty⑥"></a>

  <a id="ref-for-propdef-color-interpolation-filters①②"></a>

  <a id="ref-for-ColorInterpolationProperty①⓪"></a>

  <a id="ref-for-color0⑥"></a>

  <a id="ref-for-propdef-clip-rule⑥"></a>

  <a id="ref-for-propdef-clip-path⑥"></a>

  <a id="ref-for-propdef-clip⑥"></a>

  <a id="ref-for-BaselineShiftProperty⑥"></a>

  <a id="ref-for-AlignmentBaselineProperty⑥"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x⑨"></a>

  <a id="ref-for-filter-primitive-attributes⑤"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in①④"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fediffuselighting-surfacescale"></a>

  [surfaceScale](#element-attrdef-fediffuselighting-surfacescale)

- <a id="ref-for-element-attrdef-fediffuselighting-diffuseconstant"></a>

  [diffuseConstant](#element-attrdef-fediffuselighting-diffuseconstant)

- <a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength"></a>

  [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEDiffuseLightingElement](#InterfaceSVGFEDiffuseLightingElement)

This filter primitive lights an image using the alpha channel as a bump map. The resulting image is an RGBA opaque image based on the light color with alpha = 1.0 everywhere. The lighting calculation follows the standard diffuse component of the Phong lighting model. The resulting image depends on the light color, light position and surface geometry of the input bump map.

<a id="ref-for-elementdef-fecomposite⑧"></a>

<a id="ref-for-light-source②"></a>

The light map produced by this filter primitive can be combined with a texture image using the multiply term of the <em>arithmetic</em> [feComposite](#elementdef-fecomposite) compositing method. Multiple [light sources](#light-source) can be simulated by adding several of these light maps together before applying it to the texture image.

<a id="ref-for-elementdef-fediffuselighting④"></a>

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength①"></a>

The formulas below make use of 3x3 filters. Because they operate on pixels, such filters are inherently resolution-dependent. To make [feDiffuseLighting](#elementdef-fediffuselighting) produce resolution-independent results, an explicit value should be provided for the attribute [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength).

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength②"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits①①"></a>

[kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength), in combination with the other attributes, defines an implicit pixel grid in the filter effects coordinate system (i.e., the coordinate system established by the [primitiveUnits](#element-attrdef-filter-primitiveunits) attribute). The input image will be temporarily rescaled to match its pixels with <a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength③"></a>kernelUnitLength. The 3x3 filters are applied to the resampled image. After applying the filter, the image is resampled back to its original resolution.

<a id="ref-for-propdef-image-rendering⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the speed of the available interpolates, this choice may be affected by the [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting.

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementations might choose approaches that minimize or eliminate resampling when not necessary to produce proper results, such as when the document is zoomed out such that [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength) is considerably smaller than a device pixel.

For the formulas that follow, the <code>Norm(A<sub>x</sub>,A<sub>y</sub>,A<sub>z</sub>)</code> function is defined as:

<strong>Mathematical expression 14</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
\mathit{Norm}{{({A_{x},A_{y},A_{z}})} = \sqrt{{A_{x}^{2} + A_{y}^{2}} + A_{z}^{2}}}
```

Source mathematical tokens: Norm ( A x , A y , A z ) = A x 2 + A y 2 + A z 2 Norm(A_x,A_y,A_z) = sqrt { A_x^2+A_y^2+A_z^2}

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents may use the the "fast inverse square root" to optimize the equation and avoid time differences on extrema color values. See [Privacy and Security Considerations](#priv-sec) section for more details about timing attacks.

The resulting RGBA image is computed as follows:

<code> D<sub>r</sub> = k<sub>d</sub> &#x2A; N.L &#x2A; L<sub>r</sub><br> D<sub>g</sub> = k<sub>d</sub> &#x2A; N.L &#x2A; L<sub>g</sub><br> D<sub>b</sub> = k<sub>d</sub> &#x2A; N.L &#x2A; L<sub>b</sub><br> D<sub>a</sub> = 1.0 </code>

where

k<sub>d</sub> = diffuse lighting constant  
N = surface normal unit vector, a function of x and y  
L = unit vector pointing from surface to light, a function of x and y in the point and spot light cases  
L<sub>r</sub>,L<sub>g</sub>,L<sub>b</sub> = RGB components of light, a function of x and y in the spot light case

N is a function of x and y and depends on the surface gradient as follows:

The surface described by the input alpha image I(x,y) is:

`Z (x,y) = surfaceScale * I(x,y)`

<a id="SurfaceNormalCalculations"></a>Surface normal is calculated using the Sobel gradient 3x3 filter. Different filter kernels are used depending on whether the given pixel is on the interior or an edge. For each case, the formula is:

<code> N<sub>x</sub> (x,y) = - surfaceScale &#x2A; FACTOR<sub>x</sub> &#x2A;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;(K<sub>x</sub>(0,0)&#x2A;I(x-dx,y-dy)&#xA0;+&#xA;  K<sub>x</sub>(1,0)&#x2A;I(x,y-dy)&#xA0;+ K<sub>x</sub>(2,0)&#x2A;I(x+dx,y-dy)&#xA0;+<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;K<sub>x</sub>(0,1)&#x2A;I(x-dx,y)&#xA0;&#xA0;&#xA0;&#xA0;+&#xA;  K<sub>x</sub>(1,1)&#x2A;I(x,y)&#xA0;&#xA0;&#xA0;&#xA0;+&#xA;  K<sub>x</sub>(2,1)&#x2A;I(x+dx,y)&#xA0;&#xA0;&#xA0;&#xA0;+<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;K<sub>x</sub>(0,2)&#x2A;I(x-dx,y+dy)&#xA0;+&#xA;  K<sub>x</sub>(1,2)&#x2A;I(x,y+dy)&#xA0;+ K<sub>x</sub>(2,2)&#x2A;I(x+dx,y+dy))<br> N<sub>y</sub> (x,y) = - surfaceScale &#x2A; FACTOR<sub>y</sub> &#x2A;<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;(K<sub>y</sub>(0,0)&#x2A;I(x-dx,y-dy)&#xA0;+&#xA;  K<sub>y</sub>(1,0)&#x2A;I(x,y-dy)&#xA0;+ K<sub>y</sub>(2,0)&#x2A;I(x+dx,y-dy)&#xA0;+<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;K<sub>y</sub>(0,1)&#x2A;I(x-dx,y)&#xA0;&#xA0;&#xA0;&#xA0;+&#xA;  K<sub>y</sub>(1,1)&#x2A;I(x,y)&#xA0;&#xA0;&#xA0;&#xA0;+&#xA;  K<sub>y</sub>(2,1)&#x2A;I(x+dx,y)&#xA0;&#xA0;&#xA0;&#xA0;+<br> &#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;&#xA0;K<sub>y</sub>(0,2)&#x2A;I(x-dx,y+dy)&#xA0;+&#xA;  K<sub>y</sub>(1,2)&#x2A;I(x,y+dy)&#xA0;+ K<sub>y</sub>(2,2)&#x2A;I(x+dx,y+dy))<br> N<sub>z</sub> (x,y) = 1.0<br> <br> N = (N<sub>x</sub>, N<sub>y</sub>, N<sub>z</sub>) /&#xA;  Norm((N<sub>x</sub>,N<sub>y</sub>,N<sub>z</sub>)) </code>

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength⑤"></a>

In these formulas, the `dx` and `dy` values (e.g., `I(x-dx,y-dy)`), represent deltas relative to a given `(x,y)` position for the purpose of estimating the slope of the surface at that point. These deltas are determined by the value (explicit or implicit) of attribute [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength).

<strong>Table 14 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Top/left corner:

<strong>Column 2 (header cell):</strong>

Top row:

<strong>Column 3 (header cell):</strong>

Top/right corner:

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

FACTOR<sub>x</sub>=2/(3\*dx)  
K<sub>x</sub> =  
    \|  0  0  0 \|  
    \|  0 -2  2 \|  
    \|  0 -1  1 \|  
  
FACTOR<sub>y</sub>=2/(3\*dy)  
K<sub>y</sub> =    
    \|  0  0  0 \|  
    \|  0 -2 -1 \|  
    \|  0  2  1 \|

<strong>Column 2 (data cell):</strong>

FACTOR<sub>x</sub>=1/(3\*dx)  
K<sub>x</sub> =  
    \|  0  0  0 \|  
    \| -2  0  2 \|  
    \| -1  0  1 \|  
  
FACTOR<sub>y</sub>=1/(2\*dy)  
K<sub>y</sub> =    
    \|  0  0  0 \|  
    \| -1 -2 -1 \|  
    \|  1  2  1 \|

<strong>Column 3 (data cell):</strong>

FACTOR<sub>x</sub>=2/(3\*dx)  
K<sub>x</sub> =  
    \|  0  0  0 \|  
    \| -2  2  0 \|  
    \| -1  1  0 \|  
  
FACTOR<sub>y</sub>=2/(3\*dy)  
K<sub>y</sub> =    
    \|  0  0  0 \|  
    \| -1 -2  0 \|  
    \|  1  2  0 \|

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Left column:

<strong>Column 2 (header cell):</strong>

Interior pixels:

<strong>Column 3 (header cell):</strong>

Right column:

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

FACTOR<sub>x</sub>=1/(2\*dx)  
K<sub>x</sub> =  
    \| 0 -1  1 \|  
    \| 0 -2  2 \|  
    \| 0 -1  1 \|  
  
FACTOR<sub>y</sub>=1/(3\*dy)  
K<sub>y</sub> =    
    \|  0 -2 -1 \|  
    \|  0  0  0 \|  
    \|  0  2  1 \|

<strong>Column 2 (data cell):</strong>

FACTOR<sub>x</sub>=1/(4\*dx)  
K<sub>x</sub> =  
    \| -1  0  1 \|  
    \| -2  0  2 \|  
    \| -1  0  1 \|  
  
FACTOR<sub>y</sub>=1/(4\*dy)  
K<sub>y</sub> =    
    \| -1 -2 -1 \|  
    \|  0  0  0 \|  
    \|  1  2  1 \|

<strong>Column 3 (data cell):</strong>

FACTOR<sub>x</sub>=1/(2\*dx)  
K<sub>x</sub> =  
    \| -1  1  0\|  
    \| -2  2  0\|  
    \| -1  1  0\|  
  
FACTOR<sub>y</sub>=1/(3\*dy)  
K<sub>y</sub> =    
    \| -1 -2  0 \|  
    \|  0  0  0 \|  
    \|  1  2  0 \|

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

Bottom/left corner:

<strong>Column 2 (header cell):</strong>

Bottom row:

<strong>Column 3 (header cell):</strong>

Bottom/right corner:

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

FACTOR<sub>x</sub>=2/(3\*dx)  
K<sub>x</sub> =  
    \| 0 -1  1 \|  
    \| 0 -2  2 \|  
    \| 0  0  0 \|  
  
FACTOR<sub>y</sub>=2/(3\*dy)  
K<sub>y</sub> =    
    \|  0 -2 -1 \|  
    \|  0  2  1 \|  
    \|  0  0  0 \|

<strong>Column 2 (data cell):</strong>

FACTOR<sub>x</sub>=1/(3\*dx)  
K<sub>x</sub> =  
    \| -1  0  1 \|  
    \| -2  0  2 \|  
    \|  0  0  0 \|  
  
FACTOR<sub>y</sub>=1/(2\*dy)  
K<sub>y</sub> =    
    \| -1 -2 -1 \|  
    \|  1  2  1 \|  
    \|  0  0  0 \|

<strong>Column 3 (data cell):</strong>

FACTOR<sub>x</sub>=2/(3\*dx)  
K<sub>x</sub> =  
    \| -1  1  0 \|  
    \| -2  2  0 \|  
    \|  0  0  0 \|  
  
FACTOR<sub>y</sub>=2/(3\*dy)  
K<sub>y</sub> =    
    \| -1 -2  0 \|  
    \|  1  2  0 \|  
    \|  0  0  0 \|

L, the unit vector from the image sample to the light, is calculated as follows:

<a id="ref-for-light-source③"></a>

For Infinite [light sources](#light-source) it is constant:

<code> L<sub>x</sub> = cos(azimuth)&#x2A;cos(elevation)<br> L<sub>y</sub> = sin(azimuth)&#x2A;cos(elevation)<br> L<sub>z</sub> = sin(elevation) </code>

For Point and spot lights it is a function of position:

<code> L<sub>x</sub> = Light<sub>x</sub> - x<br> L<sub>y</sub> = Light<sub>y</sub> - y<br> L<sub>z</sub> = Light<sub>z</sub> - Z(x,y)<br> <br> L = (L<sub>x</sub>, L<sub>y</sub>, L<sub>z</sub>) / Norm(L<sub>x</sub>,&#xA;  L<sub>y</sub>, L<sub>z</sub>) </code>

where Light<sub>x</sub>, Light<sub>y</sub>, and Light<sub>z</sub> are the input light position.

L<sub>r</sub>,L<sub>g</sub>,L<sub>b</sub>, the light color vector, is a function of position in the spot light case only:

<code> L<sub>r</sub> = Light<sub>r</sub>&#x2A;pow((-L.S),specularExponent)<br> L<sub>g</sub> = Light<sub>g</sub>&#x2A;pow((-L.S),specularExponent)<br> L<sub>b</sub> = Light<sub>b</sub>&#x2A;pow((-L.S),specularExponent) </code>

where S is the unit vector pointing from the light to the point (pointsAtX, pointsAtY, pointsAtZ) in the x-y plane:

<code> S<sub>x</sub> = pointsAtX - Light<sub>x</sub><br> S<sub>y</sub> = pointsAtY - Light<sub>y</sub><br> S<sub>z</sub> = pointsAtZ - Light<sub>z</sub><br> <br> S = (S<sub>x</sub>, S<sub>y</sub>, S<sub>z</sub>) / Norm(S<sub>x</sub>,&#xA;  S<sub>y</sub>, S<sub>z</sub>) </code>

<a id="ref-for-element-attrdef-fespotlight-limitingconeangle"></a>

If L.S is positive, no light is present. (L<sub>r</sub> = L<sub>g</sub> = L<sub>b</sub> = 0). If [limitingConeAngle](#element-attrdef-fespotlight-limitingconeangle) is specified, -L.S \< cos(limitingConeAngle) also indicates that no light is present.

<em>Attribute definitions:</em>

<a id="ref-for-number-value①⑦"></a>

<a id="element-attrdef-fediffuselighting-surfacescale"></a>`surfaceScale` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

height of surface when A<sub>in</sub> = 1.

If the attribute is not specified, then the effect is as if a value of 1 were specified.

Animatable: yes.

<a id="ref-for-number-value①⑧"></a>

<a id="element-attrdef-fediffuselighting-diffuseconstant"></a>`diffuseConstant` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

kd in Phong lighting model. In SVG, this can be any non-negative number.

If the attribute is not specified, then the effect is as if a value of 1 were specified.

Animatable: yes.

<a id="ref-for-typedef-number-optional-number④"></a>

<a id="element-attrdef-fediffuselighting-kernelunitlength"></a>`kernelUnitLength` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits①②"></a>

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength⑥"></a>

The first number is the \<dx\> value. The second number is the \<dy\> value. If the \<dy\> value is not specified, it defaults to the same value as \<dx\>. Indicates the intended distance in current filter units (i.e., units as determined by the value of attribute [primitiveUnits](#element-attrdef-filter-primitiveunits)) for `dx` and `dy`, respectively, in the [surface normal calculation formulas](#SurfaceNormalCalculations). By specifying value(s) for [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength), the kernel becomes defined in a scalable, abstract coordinate system. If <a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength⑦"></a>kernelUnitLength is not specified, the `dx` and `dy` values should represent very small deltas relative to a given `(x,y)` position, which might be implemented in some cases as one pixel in the intermediate image offscreen bitmap, which is a pixel-based coordinate system, and thus potentially not scalable. For some level of consistency across display media and user agents, it is necessary that a value be provided for <a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength⑧"></a>kernelUnitLength.

If a negative or zero value is specified the default value will be used instead.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This attribute is deprecated and will be removed. It does not provide a reliable way to create platform independent results. Future versions of this specification will cover this use case.

Animatable: yes.

<a id="ref-for-light-source④"></a>

<a id="ref-for-elementdef-fedistantlight"></a>

<a id="ref-for-elementdef-fepointlight"></a>

<a id="ref-for-elementdef-fespotlight①"></a>

<a id="ref-for-propdef-lighting-color⑧"></a>

The [light source](#light-source) is defined by one of the child elements [feDistantLight](#elementdef-fedistantlight), [fePointLight](#elementdef-fepointlight) or [feSpotLight](#elementdef-fespotlight). The light color is specified by property [lighting-color](#propdef-lighting-color).

<a id="ref-for-elementdef-fedisplacementmap②"></a>

### <a id="feDisplacementMapElement"></a>9.11. Filter primitive [feDisplacementMap](#elementdef-fedisplacementmap)

<strong>Table 15 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fedisplacementmap"></a>`feDisplacementMap`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③②"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement⑨"></a>

<a id="ref-for-elementdef-script①①"></a>

<a id="ref-for-AnimateElement⑨"></a>

<a id="ref-for-TermDescriptiveElement①⓪"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode⑦"></a>

  <a id="ref-for-propdef-word-spacing⑦"></a>

  <a id="ref-for-propdef-visibility⑦"></a>

  <a id="ref-for-propdef-unicode-bidi⑦"></a>

  <a id="ref-for-TextRenderingProperty⑦"></a>

  <a id="ref-for-propdef-text-decoration⑦"></a>

  <a id="ref-for-TextAnchorProperty⑦"></a>

  <a id="ref-for-StrokeWidthProperty⑦"></a>

  <a id="ref-for-StrokeOpacityProperty⑦"></a>

  <a id="ref-for-StrokeMiterlimitProperty⑦"></a>

  <a id="ref-for-StrokeLinejoinProperty⑦"></a>

  <a id="ref-for-StrokeLinecapProperty⑦"></a>

  <a id="ref-for-StrokeDashoffsetProperty⑦"></a>

  <a id="ref-for-StrokeDasharrayProperty⑦"></a>

  <a id="ref-for-StrokeProperty⑧"></a>

  <a id="ref-for-StopOpacityProperty⑦"></a>

  <a id="ref-for-StopColorProperty⑦"></a>

  <a id="ref-for-ShapeRenderingProperty⑦"></a>

  <a id="ref-for-PointerEventsProperty⑦"></a>

  <a id="ref-for-propdef-overflow⑦"></a>

  <a id="ref-for-propdef-opacity①⓪"></a>

  <a id="ref-for-propdef-mask⑦"></a>

  <a id="ref-for-MarkerStartProperty⑦"></a>

  <a id="ref-for-MarkerMidProperty⑦"></a>

  <a id="ref-for-MarkerEndProperty⑦"></a>

  <a id="ref-for-MarkerProperty⑦"></a>

  <a id="ref-for-propdef-lighting-color⑨"></a>

  <a id="ref-for-propdef-letter-spacing⑦"></a>

  <a id="ref-for-KerningProperty⑦"></a>

  <a id="ref-for-propdef-isolation⑨"></a>

  <a id="ref-for-propdef-image-rendering⑨"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty⑦"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty⑦"></a>

  <a id="ref-for-propdef-font-weight⑦"></a>

  <a id="ref-for-propdef-font-variant⑦"></a>

  <a id="ref-for-propdef-font-style⑦"></a>

  <a id="ref-for-propdef-font-stretch⑦"></a>

  <a id="ref-for-propdef-font-size-adjust⑦"></a>

  <a id="ref-for-propdef-font-size⑦"></a>

  <a id="ref-for-propdef-font-family⑦"></a>

  <a id="ref-for-propdef-font⑦"></a>

  <a id="ref-for-propdef-flood-opacity⑨"></a>

  <a id="ref-for-propdef-flood-color⑧"></a>

  <a id="ref-for-propdef-filter①⑧"></a>

  <a id="ref-for-FillRuleProperty⑦"></a>

  <a id="ref-for-FillOpacityProperty⑦"></a>

  <a id="ref-for-FillProperty⑨"></a>

  <a id="ref-for-EnableBackgroundProperty⑦"></a>

  <a id="ref-for-DominantBaselineProperty⑦"></a>

  <a id="ref-for-propdef-display①⓪"></a>

  <a id="ref-for-propdef-direction⑦"></a>

  <a id="ref-for-propdef-cursor⑦"></a>

  <a id="ref-for-ColorRenderingProperty⑦"></a>

  <a id="ref-for-propdef-color-interpolation-filters①③"></a>

  <a id="ref-for-ColorInterpolationProperty①①"></a>

  <a id="ref-for-color0⑦"></a>

  <a id="ref-for-propdef-clip-rule⑦"></a>

  <a id="ref-for-propdef-clip-path⑦"></a>

  <a id="ref-for-propdef-clip⑦"></a>

  <a id="ref-for-BaselineShiftProperty⑦"></a>

  <a id="ref-for-AlignmentBaselineProperty⑦"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①⓪"></a>

  <a id="ref-for-filter-primitive-attributes⑥"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in①⑤"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fedisplacementmap-in2"></a>

  [in2](#element-attrdef-fedisplacementmap-in2)

- <a id="ref-for-element-attrdef-fedisplacementmap-scale"></a>

  [scale](#element-attrdef-fedisplacementmap-scale)

- <a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector"></a>

  [xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector)

- <a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector"></a>

  [yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEDisplacementMapElement](#InterfaceSVGFEDisplacementMapElement)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1d845580"></a> Implementations do not match specification. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;113&#x3E;](https://github.com/w3c/csswg-drafts/issues/113)

<a id="ref-for-element-attrdef-fedisplacementmap-in2①"></a>

<a id="ref-for-element-attrdef-filter-primitive-in①⑥"></a>

This filter primitive uses the pixels values from the image from [in2](#element-attrdef-fedisplacementmap-in2) to spatially displace the image from [in](#element-attrdef-filter-primitive-in). This is the transformation to be performed:

```text
P'(x,y) ← P( x + scale * (XC(x,y) - .5), y + scale * (YC(x,y) - .5))
```
<a id="ref-for-element-attrdef-filter-primitive-in①⑦"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector①"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector①"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-in2②"></a>

where P(x,y) is the input image, [in](#element-attrdef-filter-primitive-in), and P'(x,y) is the destination. XC(x,y) and YC(x,y) are the component values of the channel designated by the [xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector) and [yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector). For example, to use the R component of [in2](#element-attrdef-fedisplacementmap-in2) to control displacement in x and the G component of Image2 to control displacement in y, set <a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector②"></a>xChannelSelector to "R" and <a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector②"></a>yChannelSelector to "G".

<a id="ref-for-element-attrdef-fedisplacementmap-in2③"></a>

The displacement map, [in2](#element-attrdef-fedisplacementmap-in2), defines the inverse of the mapping performed.

<a id="ref-for-element-attrdef-filter-primitive-in①⑧"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-in2④"></a>

The input image [in](#element-attrdef-filter-primitive-in) is to remain premultiplied for this filter primitive. The calculations using the pixel values from [in2](#element-attrdef-fedisplacementmap-in2) are performed using non-premultiplied color values.

<a id="ref-for-element-attrdef-fedisplacementmap-scale①"></a>

This filter can have arbitrary non-localized effect on the input which might require substantial buffering in the processing pipeline. However with this formulation, any intermediate buffering needs can be determined by [scale](#element-attrdef-fedisplacementmap-scale) which represents the maximum range of displacement in either x or y.

When applying this filter, the source pixel location will often lie between several source pixels.

<a id="ref-for-propdef-image-rendering①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the speed of the available interpolents, this choice may be affected by the [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future version of this spec will define the interpolation method to be used when distorting the source image making UAs rendering result more interoperable.

<a id="ref-for-propdef-color-interpolation-filters①④"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-in2⑤"></a>

<a id="ref-for-element-attrdef-filter-primitive-in①⑨"></a>

The [color-interpolation-filters](#propdef-color-interpolation-filters) property only applies to the [in2](#element-attrdef-fedisplacementmap-in2) source image and does not apply to the [in](#element-attrdef-filter-primitive-in) source image. The <a id="ref-for-element-attrdef-filter-primitive-in②⓪"></a>in source image must remain in its current color space.

<em>Attribute definitions:</em>

<a id="ref-for-number-value①⑨"></a>

<a id="element-attrdef-fedisplacementmap-scale"></a>`scale` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits①③"></a>

<a id="ref-for-elementdef-filter③⑤"></a>

Displacement scale factor. The amount is expressed in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

When the value of this attribute is 0, this operation has no effect on the source image.

<a id="ref-for-TermInitialValue④⓪"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-scale②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [scale](#element-attrdef-fedisplacementmap-scale) is 0.

Animatable: yes.

<a id="element-attrdef-fedisplacementmap-xchannelselector"></a>`xChannelSelector` = "<em>R | G | B | A</em>"

<a id="ref-for-element-attrdef-fedisplacementmap-in2⑥"></a>

<a id="ref-for-element-attrdef-filter-primitive-in②①"></a>

Indicates which channel from [in2](#element-attrdef-fedisplacementmap-in2) to use to displace the pixels in [in](#element-attrdef-filter-primitive-in) along the x-axis.

<a id="ref-for-TermInitialValue④①"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector③"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector) is A.

Animatable: yes.

<a id="element-attrdef-fedisplacementmap-ychannelselector"></a>`yChannelSelector` = "<em>R | G | B | A</em>"

<a id="ref-for-element-attrdef-fedisplacementmap-in2⑦"></a>

<a id="ref-for-element-attrdef-filter-primitive-in②②"></a>

Indicates which channel from [in2](#element-attrdef-fedisplacementmap-in2) to use to displace the pixels in [in](#element-attrdef-filter-primitive-in) along the y-axis.

<a id="ref-for-TermInitialValue④②"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector③"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector) is A.

Animatable: yes.

<a id="ref-for-element-attrdef-filter-primitive-in②③"></a>

<a id="element-attrdef-fedisplacementmap-in2"></a>`in2` = "<em>(see <a href="#element-attrdef-filter-primitive-in">in</a> attribute)</em>"

<a id="ref-for-element-attrdef-filter-primitive-in②④"></a>

The second input image, which is used to displace the pixels in the image from attribute [in](#element-attrdef-filter-primitive-in). See defintion for <a id="ref-for-element-attrdef-filter-primitive-in②⑤"></a>in attribute.

Animatable: yes.

<a id="ref-for-elementdef-fedropshadow②"></a>

### <a id="feDropShadowElement"></a>9.12. Filter primitive [feDropShadow](#elementdef-fedropshadow)

<strong>Table 16 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fedropshadow"></a>`feDropShadow`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③③"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⓪"></a>

<a id="ref-for-elementdef-script①②"></a>

<a id="ref-for-AnimateElement①⓪"></a>

<a id="ref-for-TermDescriptiveElement①①"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode⑧"></a>

  <a id="ref-for-propdef-word-spacing⑧"></a>

  <a id="ref-for-propdef-visibility⑧"></a>

  <a id="ref-for-propdef-unicode-bidi⑧"></a>

  <a id="ref-for-TextRenderingProperty⑧"></a>

  <a id="ref-for-propdef-text-decoration⑧"></a>

  <a id="ref-for-TextAnchorProperty⑧"></a>

  <a id="ref-for-StrokeWidthProperty⑧"></a>

  <a id="ref-for-StrokeOpacityProperty⑧"></a>

  <a id="ref-for-StrokeMiterlimitProperty⑧"></a>

  <a id="ref-for-StrokeLinejoinProperty⑧"></a>

  <a id="ref-for-StrokeLinecapProperty⑧"></a>

  <a id="ref-for-StrokeDashoffsetProperty⑧"></a>

  <a id="ref-for-StrokeDasharrayProperty⑧"></a>

  <a id="ref-for-StrokeProperty⑨"></a>

  <a id="ref-for-StopOpacityProperty⑧"></a>

  <a id="ref-for-StopColorProperty⑧"></a>

  <a id="ref-for-ShapeRenderingProperty⑧"></a>

  <a id="ref-for-PointerEventsProperty⑧"></a>

  <a id="ref-for-propdef-overflow⑧"></a>

  <a id="ref-for-propdef-opacity①①"></a>

  <a id="ref-for-propdef-mask⑧"></a>

  <a id="ref-for-MarkerStartProperty⑧"></a>

  <a id="ref-for-MarkerMidProperty⑧"></a>

  <a id="ref-for-MarkerEndProperty⑧"></a>

  <a id="ref-for-MarkerProperty⑧"></a>

  <a id="ref-for-propdef-lighting-color①⓪"></a>

  <a id="ref-for-propdef-letter-spacing⑧"></a>

  <a id="ref-for-KerningProperty⑧"></a>

  <a id="ref-for-propdef-isolation①⓪"></a>

  <a id="ref-for-propdef-image-rendering①①"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty⑧"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty⑧"></a>

  <a id="ref-for-propdef-font-weight⑧"></a>

  <a id="ref-for-propdef-font-variant⑧"></a>

  <a id="ref-for-propdef-font-style⑧"></a>

  <a id="ref-for-propdef-font-stretch⑧"></a>

  <a id="ref-for-propdef-font-size-adjust⑧"></a>

  <a id="ref-for-propdef-font-size⑧"></a>

  <a id="ref-for-propdef-font-family⑧"></a>

  <a id="ref-for-propdef-font⑧"></a>

  <a id="ref-for-propdef-flood-opacity①⓪"></a>

  <a id="ref-for-propdef-flood-color⑨"></a>

  <a id="ref-for-propdef-filter①⑨"></a>

  <a id="ref-for-FillRuleProperty⑧"></a>

  <a id="ref-for-FillOpacityProperty⑧"></a>

  <a id="ref-for-FillProperty①⓪"></a>

  <a id="ref-for-EnableBackgroundProperty⑧"></a>

  <a id="ref-for-DominantBaselineProperty⑧"></a>

  <a id="ref-for-propdef-display①①"></a>

  <a id="ref-for-propdef-direction⑧"></a>

  <a id="ref-for-propdef-cursor⑧"></a>

  <a id="ref-for-ColorRenderingProperty⑧"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⑤"></a>

  <a id="ref-for-ColorInterpolationProperty①②"></a>

  <a id="ref-for-color0⑧"></a>

  <a id="ref-for-propdef-clip-rule⑧"></a>

  <a id="ref-for-propdef-clip-path⑧"></a>

  <a id="ref-for-propdef-clip⑧"></a>

  <a id="ref-for-BaselineShiftProperty⑧"></a>

  <a id="ref-for-AlignmentBaselineProperty⑧"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①①"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①①"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①①"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①①"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①①"></a>

  <a id="ref-for-filter-primitive-attributes⑦"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in②⑥"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fedropshadow-stddeviation"></a>

  [stdDeviation](#element-attrdef-fedropshadow-stddeviation)

- <a id="ref-for-element-attrdef-fedropshadow-dx"></a>

  [dx](#element-attrdef-fedropshadow-dx)

- <a id="ref-for-element-attrdef-fedropshadow-dy"></a>

  [dy](#element-attrdef-fedropshadow-dy)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEDropShadowElement](#InterfaceSVGFEDropShadowElement)

<a id="ref-for-filter-primitive③④"></a>

This filter creates a drop shadow of the input image. It is a shorthand filter, and is defined in terms of combinations of other [filter primitives](#filter-primitive). The expectation is that it can be optimized more easily by implementations.

<a id="ref-for-elementdef-fedropshadow③"></a>

The result of a [feDropShadow](#elementdef-fedropshadow) filter primitive is equivalent to the following:

```text
<feGaussianBlur in="alpha-channel-of-feDropShadow-in"   stdDeviation="stdDeviation-of-feDropShadow"/>
<feOffset dx="dx-of-feDropShadow"   dy="dy-of-feDropShadow" result="offsetblur"/>
<feFlood flood-color="flood-color-of-feDropShadow"  flood-opacity="flood-opacity-of-feDropShadow"/>
<feComposite in2="offsetblur" operator="in"/>
<feMerge>
  <feMergeNode/>
  <feMergeNode in="in-of-feDropShadow"/>
</feMerge>
```
The above divided into steps:

1.  <a id="ref-for-elementdef-fedropshadow④"></a>

    <a id="ref-for-element-attrdef-fedropshadow-stddeviation①"></a>

    <a id="ref-for-elementdef-fegaussianblur⑧"></a>

    Take the alpha channel of the input to the [feDropShadow](#elementdef-fedropshadow) filter primitive and the [stdDeviation](#element-attrdef-fedropshadow-stddeviation) on the <a id="ref-for-elementdef-fedropshadow⑤"></a>feDropShadow and do processing as if the following [feGaussianBlur](#elementdef-fegaussianblur) was applied:

    ```text
    <feGaussianBlur in="alpha-channel-of-feDropShadow-in" stdDeviation="stdDeviation-of-feDropShadow"/>
    ```
2.  <a id="ref-for-element-attrdef-fedropshadow-dx①"></a>

    <a id="ref-for-element-attrdef-fedropshadow-dy①"></a>

    <a id="ref-for-elementdef-fedropshadow⑥"></a>

    <a id="ref-for-elementdef-feoffset⑦"></a>

    Offset the result of step 1 by [dx](#element-attrdef-fedropshadow-dx) and [dy](#element-attrdef-fedropshadow-dy) as specified on the [feDropShadow](#elementdef-fedropshadow) element, equivalent to applying an [feOffset](#elementdef-feoffset) with these parameters:

    ```text
    <feOffset dx="dx-of-feDropShadow" dy="dy-of-feDropShadow" result="offsetblur"/>
    ```
3.  <a id="ref-for-elementdef-feflood④"></a>

    <a id="ref-for-propdef-flood-color①⓪"></a>

    <a id="ref-for-propdef-flood-opacity①①"></a>

    <a id="ref-for-elementdef-fedropshadow⑦"></a>

    Do processing as if an [feFlood](#elementdef-feflood) element with [flood-color](#propdef-flood-color) and [flood-opacity](#propdef-flood-opacity) as specified on the [feDropShadow](#elementdef-fedropshadow) was applied:

    ```text
    <feFlood flood-color="flood-color-of-feDropShadow" flood-opacity="flood-opacity-of-feDropShadow"/>
    ```
4.  <a id="ref-for-elementdef-feflood⑤"></a>

    <a id="ref-for-elementdef-feoffset⑧"></a>

    <a id="ref-for-elementdef-fecomposite⑨"></a>

    Composite the result of the [feFlood](#elementdef-feflood) in step 3 with the result of the [feOffset](#elementdef-feoffset) in step 2 as if an [feComposite](#elementdef-fecomposite) filter primitive with `operator="in"` was applied:

    ```text
    <feComposite in2="offsetblur" operator="in"/>
    ```
5.  <a id="ref-for-elementdef-femerge⑤"></a>

    Finally merge the result of the previous step, doing processing as if the following [feMerge](#elementdef-femerge) was performed:

    ```text
    <feMerge>
      <feMergeNode/>
      <feMergeNode in="in-of-feDropShadow"/>
    </feMerge>
    ```
<a id="ref-for-elementdef-fedropshadow⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: that while the definition of the [feDropShadow](#elementdef-fedropshadow) filter primitive says that it can be expanded into an equivalent tree it is not required that it is implemented like that. The expectation is that user agents can optimize the handling by not having to do all the steps separately.

<a id="ref-for-elementdef-fedropshadow⑨"></a>

Beyond the DOM interface [SVGFEDropShadowElement](#InterfaceSVGFEDropShadowElement) there is no way of accessing the internals of the [feDropShadow](#elementdef-fedropshadow) filter primitive, meaning <a id="assert_dropShadowShadowTrees"></a>if the filter primitive is implemented as an equivalent tree then that tree must not be exposed to the DOM.

<em>Attribute definitions:</em>

<a id="ref-for-number-value②⓪"></a>

<a id="element-attrdef-fedropshadow-dx"></a>`dx` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

The x offset of the drop shadow.

<a id="ref-for-TermInitialValue④③"></a>

<a id="ref-for-element-attrdef-fedropshadow-dx②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [dx](#element-attrdef-fedropshadow-dx) is 2.

<a id="ref-for-element-attrdef-feoffset-dx"></a>

<a id="ref-for-elementdef-feoffset⑨"></a>

This attribute is then forwarded to the [dx](#element-attrdef-feoffset-dx) attribute of the internal [feOffset](#elementdef-feoffset) element.

Animatable: yes.

<a id="ref-for-number-value②①"></a>

<a id="element-attrdef-fedropshadow-dy"></a>`dy` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

The y offset of the drop shadow.

<a id="ref-for-TermInitialValue④④"></a>

<a id="ref-for-element-attrdef-fedropshadow-dy②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [dy](#element-attrdef-fedropshadow-dy) is 2.

<a id="ref-for-element-attrdef-feoffset-dy"></a>

<a id="ref-for-elementdef-feoffset①⓪"></a>

This attribute is then forwarded to the [dy](#element-attrdef-feoffset-dy) attribute of the internal [feOffset](#elementdef-feoffset) element.

Animatable: yes.

<a id="ref-for-typedef-number-optional-number⑤"></a>

<a id="element-attrdef-fedropshadow-stddeviation"></a>`stdDeviation` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

The standard deviation for the blur operation in the drop shadow.

<a id="ref-for-TermInitialValue④⑤"></a>

<a id="ref-for-element-attrdef-fedropshadow-stddeviation②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [stdDeviation](#element-attrdef-fedropshadow-stddeviation) is 2.

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation①"></a>

<a id="ref-for-elementdef-fegaussianblur⑨"></a>

This attribute is then forwarded to the [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) attribute of the internal [feGaussianBlur](#elementdef-fegaussianblur) element.

Animatable: yes.

<a id="ref-for-elementdef-feflood⑥"></a>

### <a id="feFloodElement"></a>9.13. Filter primitive [feFlood](#elementdef-feflood)

<strong>Table 17 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feflood"></a>`feFlood`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⑤"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①①"></a>

<a id="ref-for-elementdef-script①③"></a>

<a id="ref-for-AnimateElement①①"></a>

<a id="ref-for-TermDescriptiveElement①②"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode⑨"></a>

  <a id="ref-for-propdef-word-spacing⑨"></a>

  <a id="ref-for-propdef-visibility⑨"></a>

  <a id="ref-for-propdef-unicode-bidi⑨"></a>

  <a id="ref-for-TextRenderingProperty⑨"></a>

  <a id="ref-for-propdef-text-decoration⑨"></a>

  <a id="ref-for-TextAnchorProperty⑨"></a>

  <a id="ref-for-StrokeWidthProperty⑨"></a>

  <a id="ref-for-StrokeOpacityProperty⑨"></a>

  <a id="ref-for-StrokeMiterlimitProperty⑨"></a>

  <a id="ref-for-StrokeLinejoinProperty⑨"></a>

  <a id="ref-for-StrokeLinecapProperty⑨"></a>

  <a id="ref-for-StrokeDashoffsetProperty⑨"></a>

  <a id="ref-for-StrokeDasharrayProperty⑨"></a>

  <a id="ref-for-StrokeProperty①⓪"></a>

  <a id="ref-for-StopOpacityProperty⑨"></a>

  <a id="ref-for-StopColorProperty⑨"></a>

  <a id="ref-for-ShapeRenderingProperty⑨"></a>

  <a id="ref-for-PointerEventsProperty⑨"></a>

  <a id="ref-for-propdef-overflow⑨"></a>

  <a id="ref-for-propdef-opacity①②"></a>

  <a id="ref-for-propdef-mask⑨"></a>

  <a id="ref-for-MarkerStartProperty⑨"></a>

  <a id="ref-for-MarkerMidProperty⑨"></a>

  <a id="ref-for-MarkerEndProperty⑨"></a>

  <a id="ref-for-MarkerProperty⑨"></a>

  <a id="ref-for-propdef-lighting-color①①"></a>

  <a id="ref-for-propdef-letter-spacing⑨"></a>

  <a id="ref-for-KerningProperty⑨"></a>

  <a id="ref-for-propdef-isolation①①"></a>

  <a id="ref-for-propdef-image-rendering①②"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty⑨"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty⑨"></a>

  <a id="ref-for-propdef-font-weight⑨"></a>

  <a id="ref-for-propdef-font-variant⑨"></a>

  <a id="ref-for-propdef-font-style⑨"></a>

  <a id="ref-for-propdef-font-stretch⑨"></a>

  <a id="ref-for-propdef-font-size-adjust⑨"></a>

  <a id="ref-for-propdef-font-size⑨"></a>

  <a id="ref-for-propdef-font-family⑨"></a>

  <a id="ref-for-propdef-font⑨"></a>

  <a id="ref-for-propdef-flood-opacity①②"></a>

  <a id="ref-for-propdef-flood-color①①"></a>

  <a id="ref-for-propdef-filter②⓪"></a>

  <a id="ref-for-FillRuleProperty⑨"></a>

  <a id="ref-for-FillOpacityProperty⑨"></a>

  <a id="ref-for-FillProperty①①"></a>

  <a id="ref-for-EnableBackgroundProperty⑨"></a>

  <a id="ref-for-DominantBaselineProperty⑨"></a>

  <a id="ref-for-propdef-display①②"></a>

  <a id="ref-for-propdef-direction⑨"></a>

  <a id="ref-for-propdef-cursor⑨"></a>

  <a id="ref-for-ColorRenderingProperty⑨"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⑥"></a>

  <a id="ref-for-ColorInterpolationProperty①③"></a>

  <a id="ref-for-color0⑨"></a>

  <a id="ref-for-propdef-clip-rule⑨"></a>

  <a id="ref-for-propdef-clip-path⑨"></a>

  <a id="ref-for-propdef-clip⑨"></a>

  <a id="ref-for-BaselineShiftProperty⑨"></a>

  <a id="ref-for-AlignmentBaselineProperty⑨"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①②"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①②"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①②"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①②"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①②"></a>

  <a id="ref-for-filter-primitive-attributes⑧"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEFloodElement](#InterfaceSVGFEFloodElement)

<a id="ref-for-propdef-flood-color①②"></a>

<a id="ref-for-propdef-flood-opacity①③"></a>

<a id="ref-for-filter-primitive-subregion①⑤"></a>

<a id="ref-for-elementdef-feflood⑦"></a>

This filter primitive creates a rectangle filled with the color and opacity values from properties [flood-color](#propdef-flood-color) and [flood-opacity](#propdef-flood-opacity). The rectangle is as large as the [filter primitive subregion](#filter-primitive-subregion) established by the [feFlood](#elementdef-feflood) element.

<a id="ref-for-propdef-flood-color①③"></a>

#### <a id="FloodColorProperty"></a>9.13.1. The [flood-color](#propdef-flood-color) property

<strong>Table 18 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-flood-color"></a>flood-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valuea-def-color②"></a>

[\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

black

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-fedropshadow①⓪"></a>

<a id="ref-for-elementdef-feflood⑧"></a>

[feFlood](#elementdef-feflood) and [feDropShadow](#elementdef-fedropshadow) elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Media:

<strong>Column 2 (data cell):</strong>

visual

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

as [by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value)

<a id="ref-for-propdef-flood-color①④"></a>

<a id="ref-for-filter-primitive-subregion①⑥"></a>

The [flood-color](#propdef-flood-color) property indicates what color to used to flood the current [filter primitive subregion](#filter-primitive-subregion).

<a id="ref-for-propdef-flood-color①⑤"></a>

The [flood-color](#propdef-flood-color) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

<a id="ref-for-propdef-flood-opacity①④"></a>

#### <a id="FloodOpacityProperty"></a>9.13.2. The [flood-opacity](#propdef-flood-opacity) property

<strong>Table 19 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-flood-opacity"></a>flood-opacity

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-alpha-value"></a>

[\<alpha-value\>](https://www.w3.org/TR/css-color-4/#typedef-alpha-value)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

1

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-fedropshadow①①"></a>

<a id="ref-for-elementdef-feflood⑨"></a>

[feFlood](#elementdef-feflood) and [feDropShadow](#elementdef-fedropshadow) elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

the specified value converted to a number, clamped to the range \[0,1\]

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Media:

<strong>Column 2 (data cell):</strong>

visual

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

[by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value)

<a id="ref-for-propdef-flood-opacity①⑤"></a>

<a id="ref-for-filter-primitive-subregion①⑦"></a>

<a id="ref-for-propdef-flood-color①⑥"></a>

The [flood-opacity](#propdef-flood-opacity) property defines the opacity value to use across the entire [filter primitive subregion](#filter-primitive-subregion). If the [flood-color](#propdef-flood-color) value includes an alpha channel, the alpha channel gets multiplied with the computed value of the <a id="ref-for-propdef-flood-opacity①⑥"></a>flood-opacity property.

<a id="ref-for-propdef-flood-opacity①⑦"></a>

The [flood-opacity](#propdef-flood-opacity) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

<a id="ref-for-elementdef-fegaussianblur①⓪"></a>

### <a id="feGaussianBlurElement"></a>9.14. Filter primitive [feGaussianBlur](#elementdef-fegaussianblur)

<strong>Table 20 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fegaussianblur"></a>`feGaussianBlur`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⑥"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①②"></a>

<a id="ref-for-elementdef-script①④"></a>

<a id="ref-for-AnimateElement①②"></a>

<a id="ref-for-TermDescriptiveElement①③"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①⓪"></a>

  <a id="ref-for-propdef-word-spacing①⓪"></a>

  <a id="ref-for-propdef-visibility①⓪"></a>

  <a id="ref-for-propdef-unicode-bidi①⓪"></a>

  <a id="ref-for-TextRenderingProperty①⓪"></a>

  <a id="ref-for-propdef-text-decoration①⓪"></a>

  <a id="ref-for-TextAnchorProperty①⓪"></a>

  <a id="ref-for-StrokeWidthProperty①⓪"></a>

  <a id="ref-for-StrokeOpacityProperty①⓪"></a>

  <a id="ref-for-StrokeMiterlimitProperty①⓪"></a>

  <a id="ref-for-StrokeLinejoinProperty①⓪"></a>

  <a id="ref-for-StrokeLinecapProperty①⓪"></a>

  <a id="ref-for-StrokeDashoffsetProperty①⓪"></a>

  <a id="ref-for-StrokeDasharrayProperty①⓪"></a>

  <a id="ref-for-StrokeProperty①①"></a>

  <a id="ref-for-StopOpacityProperty①⓪"></a>

  <a id="ref-for-StopColorProperty①⓪"></a>

  <a id="ref-for-ShapeRenderingProperty①⓪"></a>

  <a id="ref-for-PointerEventsProperty①⓪"></a>

  <a id="ref-for-propdef-overflow①⓪"></a>

  <a id="ref-for-propdef-opacity①③"></a>

  <a id="ref-for-propdef-mask①⓪"></a>

  <a id="ref-for-MarkerStartProperty①⓪"></a>

  <a id="ref-for-MarkerMidProperty①⓪"></a>

  <a id="ref-for-MarkerEndProperty①⓪"></a>

  <a id="ref-for-MarkerProperty①⓪"></a>

  <a id="ref-for-propdef-lighting-color①②"></a>

  <a id="ref-for-propdef-letter-spacing①⓪"></a>

  <a id="ref-for-KerningProperty①⓪"></a>

  <a id="ref-for-propdef-isolation①②"></a>

  <a id="ref-for-propdef-image-rendering①③"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①⓪"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①⓪"></a>

  <a id="ref-for-propdef-font-weight①⓪"></a>

  <a id="ref-for-propdef-font-variant①⓪"></a>

  <a id="ref-for-propdef-font-style①⓪"></a>

  <a id="ref-for-propdef-font-stretch①⓪"></a>

  <a id="ref-for-propdef-font-size-adjust①⓪"></a>

  <a id="ref-for-propdef-font-size①⓪"></a>

  <a id="ref-for-propdef-font-family①⓪"></a>

  <a id="ref-for-propdef-font①⓪"></a>

  <a id="ref-for-propdef-flood-opacity①⑧"></a>

  <a id="ref-for-propdef-flood-color①⑦"></a>

  <a id="ref-for-propdef-filter②①"></a>

  <a id="ref-for-FillRuleProperty①⓪"></a>

  <a id="ref-for-FillOpacityProperty①⓪"></a>

  <a id="ref-for-FillProperty①②"></a>

  <a id="ref-for-EnableBackgroundProperty①⓪"></a>

  <a id="ref-for-DominantBaselineProperty①⓪"></a>

  <a id="ref-for-propdef-display①③"></a>

  <a id="ref-for-propdef-direction①⓪"></a>

  <a id="ref-for-propdef-cursor①⓪"></a>

  <a id="ref-for-ColorRenderingProperty①⓪"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⑦"></a>

  <a id="ref-for-ColorInterpolationProperty①④"></a>

  <a id="ref-for-color0①⓪"></a>

  <a id="ref-for-propdef-clip-rule①⓪"></a>

  <a id="ref-for-propdef-clip-path①⓪"></a>

  <a id="ref-for-propdef-clip①⓪"></a>

  <a id="ref-for-BaselineShiftProperty①⓪"></a>

  <a id="ref-for-AlignmentBaselineProperty①⓪"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①③"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①③"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①③"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①③"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①③"></a>

  <a id="ref-for-filter-primitive-attributes⑨"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in②⑦"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fegaussianblur-stddeviation②"></a>

  [stdDeviation](#element-attrdef-fegaussianblur-stddeviation)

- <a id="ref-for-element-attrdef-fegaussianblur-edgemode"></a>

  [edgeMode](#element-attrdef-fegaussianblur-edgemode)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEGaussianBlurElement](#InterfaceSVGFEGaussianBlurElement)

This filter primitive performs a Gaussian blur on the input image.

The Gaussian blur kernel is an approximation of the normalized convolution:

`G(x,y) = H(x)I(y)`

where

<code>H(x) = exp(-x<sup>2</sup>/ (2s<sup>2</sup>)) / sqrt(2π &#x2A; s<sup>2</sup>)</code>

and

<code>I(y) = exp(-y<sup>2</sup>/ (2t<sup>2</sup>)) / sqrt(2π &#x2A; t<sup>2</sup>)</code>

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation③"></a>

with "s" being the standard deviation in the x direction and "t" being the standard deviation in the y direction, as specified by [stdDeviation](#element-attrdef-fegaussianblur-stddeviation).

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation④"></a>

The value of [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) can be either one or two numbers. If two numbers are provided, the first number represents a standard deviation value along the x-axis of the current coordinate system and the second value represents a standard deviation in Y. If one number is provided, then that value is used for both X and Y.

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation⑤"></a>

Even if only one value is provided for [stdDeviation](#element-attrdef-fegaussianblur-stddeviation), this can be implemented as a separable convolution.

For larger values of "s" (s \>= 2.0), an approximation can be used: Three successive box-blurs build a piece-wise quadratic convolution kernel, which approximates the Gaussian kernel to within roughly 3%.

`let d = floor(s * 3 * sqrt(2 * π) / 4 + 0.5)`

... if d is odd, use three box-blurs of size "d", centered on the output pixel.

... if d is even, two box-blurs of size "d" (the first one centered on the pixel boundary between the output pixel and the one to the left, the second one centered on the pixel boundary between the output pixel and the one to the right) and one box blur of size "d+1" centered on the output pixel.

The approximation formula also applies correspondingly to "t".

<a id="ref-for-attr-valuedef-in-sourcealpha⑤"></a>

<a id="ref-for-attr-valuedef-in-fillpaint⑤"></a>

<a id="ref-for-elementdef-fetile④"></a>

Frequently this operation will take place on alpha-only images, such as that produced by the built-in input, [SourceAlpha](#attr-valuedef-in-sourcealpha). The implementation may notice this and optimize the single channel case. This optimization must be omitted if it leads to privacy concerns of any matter. (See section [Privacy and Security Considerations](#priv-sec) for more details about timing attacks.) If the input has infinite extent and is constant (e.g [FillPaint](#attr-valuedef-in-fillpaint) where the fill is a solid color), this operation has no effect. If the input has infinite extent and the filter result where the fill is a solid color) is the input to an [feTile](#elementdef-fetile), the filter is evaluated with [periodic boundary conditions](https://en.wikipedia.org/wiki/Periodic_boundary_conditions).

<em>Attribute definitions:</em>

<a id="ref-for-typedef-number-optional-number⑥"></a>

<a id="element-attrdef-fegaussianblur-stddeviation"></a>`stdDeviation` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-number-value②②"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits①④"></a>

<a id="ref-for-elementdef-filter③⑥"></a>

The standard deviation for the blur operation. If two [\<number\>](https://www.w3.org/TR/css3-values/#number-value) s are provided, the first number represents a standard deviation value along the x-axis of the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element. The second value represents a standard deviation in Y. If one number is provided, then that value is used for both X and Y.

A negative value or a value of zero disables the effect of the given filter primitive (i.e., the result is the filter input image).

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation⑥"></a>

If [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) is 0 in only one of X or Y, then the effect is that the blur is only applied in the direction that has a non-zero value.

<a id="ref-for-TermInitialValue④⑥"></a>

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation⑦"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) is 0.

Animatable: yes.

<a id="ref-for-attr-valuedef-edgemode-wrap③"></a>

<a id="ref-for-attr-valuedef-edgemode-duplicate①"></a>

<a id="element-attrdef-fegaussianblur-edgemode"></a>`edgeMode` = "[duplicate](#attr-valuedef-edgemode-duplicate) \| [wrap](#attr-valuedef-edgemode-wrap) \| none"

Determines how to extend the input image as necessary with color values so that the matrix operations can be applied when the kernel is positioned at or near the edge of the input image.

<a id="attr-valuedef-edgemode-duplicate"></a>`duplicate` indicates that the input image is extended along each of its borders as necessary by duplicating the color values at the given edge of the input image.

Original N-by-M image, where m=M-1 and n=N-1:

<a id="attr-valuedef-edgemode-wrap"></a>`wrap` indicates that the input image is extended by taking the color values from the opposite edge of the image.

The value none indicates that the input image is extended with pixel values of zero for R, G, B and A.

<a id="ref-for-TermInitialValue④⑦"></a>

<a id="ref-for-element-attrdef-fegaussianblur-edgemode①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [edgeMode](#element-attrdef-fegaussianblur-edgemode) is none.

Animatable: yes.

<a id="ref-for-elementdef-fegaussianblur①①"></a>

[The example](#intro) at the start of this chapter makes use of the [feGaussianBlur](#elementdef-fegaussianblur) filter primitive to create a drop shadow effect.

<a id="ref-for-elementdef-feimage③"></a>

### <a id="feImageElement"></a>9.15. Filter primitive [feImage](#elementdef-feimage)

<strong>Table 21 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feimage"></a>`feImage`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⑦"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①③"></a>

<a id="ref-for-elementdef-script①⑤"></a>

<a id="ref-for-AnimateTransformElement"></a>

<a id="ref-for-AnimateElement①③"></a>

<a id="ref-for-TermDescriptiveElement①④"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [animateTransform](https://www.w3.org/TR/SVG11/animate.html#AnimateTransformElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①①"></a>

  <a id="ref-for-propdef-word-spacing①①"></a>

  <a id="ref-for-propdef-visibility①①"></a>

  <a id="ref-for-propdef-unicode-bidi①①"></a>

  <a id="ref-for-TextRenderingProperty①①"></a>

  <a id="ref-for-propdef-text-decoration①①"></a>

  <a id="ref-for-TextAnchorProperty①①"></a>

  <a id="ref-for-StrokeWidthProperty①①"></a>

  <a id="ref-for-StrokeOpacityProperty①①"></a>

  <a id="ref-for-StrokeMiterlimitProperty①①"></a>

  <a id="ref-for-StrokeLinejoinProperty①①"></a>

  <a id="ref-for-StrokeLinecapProperty①①"></a>

  <a id="ref-for-StrokeDashoffsetProperty①①"></a>

  <a id="ref-for-StrokeDasharrayProperty①①"></a>

  <a id="ref-for-StrokeProperty①②"></a>

  <a id="ref-for-StopOpacityProperty①①"></a>

  <a id="ref-for-StopColorProperty①①"></a>

  <a id="ref-for-ShapeRenderingProperty①①"></a>

  <a id="ref-for-PointerEventsProperty①①"></a>

  <a id="ref-for-propdef-overflow①①"></a>

  <a id="ref-for-propdef-opacity①④"></a>

  <a id="ref-for-propdef-mask①①"></a>

  <a id="ref-for-MarkerStartProperty①①"></a>

  <a id="ref-for-MarkerMidProperty①①"></a>

  <a id="ref-for-MarkerEndProperty①①"></a>

  <a id="ref-for-MarkerProperty①①"></a>

  <a id="ref-for-propdef-lighting-color①③"></a>

  <a id="ref-for-propdef-letter-spacing①①"></a>

  <a id="ref-for-KerningProperty①①"></a>

  <a id="ref-for-propdef-isolation①③"></a>

  <a id="ref-for-propdef-image-rendering①④"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①①"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①①"></a>

  <a id="ref-for-propdef-font-weight①①"></a>

  <a id="ref-for-propdef-font-variant①①"></a>

  <a id="ref-for-propdef-font-style①①"></a>

  <a id="ref-for-propdef-font-stretch①①"></a>

  <a id="ref-for-propdef-font-size-adjust①①"></a>

  <a id="ref-for-propdef-font-size①①"></a>

  <a id="ref-for-propdef-font-family①①"></a>

  <a id="ref-for-propdef-font①①"></a>

  <a id="ref-for-propdef-flood-opacity①⑨"></a>

  <a id="ref-for-propdef-flood-color①⑧"></a>

  <a id="ref-for-propdef-filter②②"></a>

  <a id="ref-for-FillRuleProperty①①"></a>

  <a id="ref-for-FillOpacityProperty①①"></a>

  <a id="ref-for-FillProperty①③"></a>

  <a id="ref-for-EnableBackgroundProperty①①"></a>

  <a id="ref-for-DominantBaselineProperty①①"></a>

  <a id="ref-for-propdef-display①④"></a>

  <a id="ref-for-propdef-direction①①"></a>

  <a id="ref-for-propdef-cursor①①"></a>

  <a id="ref-for-ColorRenderingProperty①①"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⑧"></a>

  <a id="ref-for-ColorInterpolationProperty①⑤"></a>

  <a id="ref-for-color0①①"></a>

  <a id="ref-for-propdef-clip-rule①①"></a>

  <a id="ref-for-propdef-clip-path①①"></a>

  <a id="ref-for-propdef-clip①①"></a>

  <a id="ref-for-BaselineShiftProperty①①"></a>

  <a id="ref-for-AlignmentBaselineProperty①①"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①④"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①④"></a>

  <a id="ref-for-filter-primitive-attributes①⓪"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- [externalResourcesRequired](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#ExternalResourcesRequiredAttribute)

- <a id="ref-for-element-attrdef-feimage-preserveaspectratio"></a>

  [preserveAspectRatio](#element-attrdef-feimage-preserveaspectratio)

- <a id="ref-for-element-attrdef-feimage-xlinkhref"></a>

  [xlink:href](#element-attrdef-feimage-xlinkhref)

- <a id="ref-for-element-attrdef-feimage-href"></a>

  [href](#element-attrdef-feimage-href)

- <a id="ref-for-element-attrdef-feimage-crossorigin"></a>

  [crossorigin](#element-attrdef-feimage-crossorigin)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEImageElement](#InterfaceSVGFEImageElement)

This filter primitive refers to a graphic external to this filter element, which is loaded or rendered into an RGBA raster and becomes the result of the filter primitive.

<a id="ref-for-attr-valuedef-in-sourcegraphic⑧"></a>

This filter primitive can refer to an external image or can be a reference to another piece of SVG. It produces an image similar to the built-in image source [SourceGraphic](#attr-valuedef-in-sourcegraphic) except that the graphic comes from an external source.

<a id="ref-for-element-attrdef-feimage-href①"></a>

<a id="ref-for-elementdef-image"></a>

<a id="ref-for-elementdef-use①"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits①⑤"></a>

<a id="ref-for-elementdef-filter③⑦"></a>

<a id="ref-for-element-attrdef-feimage-preserveaspectratio①"></a>

<a id="ref-for-elementdef-feimage④"></a>

If the [href](#element-attrdef-feimage-href) references a stand-alone image resource such as a JPEG, PNG or SVG file, then the image resource is rendered according to the behavior of the [image](https://www.w3.org/TR/svg2/embedded.html#elementdef-image) element; otherwise, the referenced resource is rendered according to the behavior of the [use](https://www.w3.org/TR/svg2/struct.html#elementdef-use) element. In either case, the current user coordinate system depends on the value of attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element. The processing of the [preserveAspectRatio](#element-attrdef-feimage-preserveaspectratio) attribute on the [feImage](#elementdef-feimage) element is identical to that of the <a id="ref-for-elementdef-image①"></a>image element.

<a id="ref-for-element-attrdef-feimage-href②"></a>

A [href](#element-attrdef-feimage-href) reference that is an empty image (zero width or zero height), that fails to download, is non-existent, or that cannot be displayed (e.g. because it is not in a supported image format) fills the filter primitive subregion with transparent black.

<a id="ref-for-propdef-image-rendering①⑤"></a>

<a id="assert_hqImageResampling"></a>When the referenced image must be resampled to match the device coordinate system, it is recommended that high quality viewers make use of appropriate interpolation techniques, for example bilinear or bicubic. Depending on the speed of the available interpolents, this choice may be affected by the [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting.

<em>Attribute definitions:</em>

<a id="ref-for-typedef-filter-url①"></a>

<a id="element-attrdef-feimage-xlinkhref"></a>`xlink:href` = "[\<url\>](#typedef-filter-url)"

<a id="ref-for-element-attrdef-feimage-href③"></a>

See [href](#element-attrdef-feimage-href) attribute.

Animatable: yes.

<a id="ref-for-element-attrdef-feimage-href④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This <em>xlink:href</em> attribute is deprecated and should not be used in new content, it’s included for backwards compatibility reasons only. Authors should use the [href](#element-attrdef-feimage-href) attribute instead.

<a id="ref-for-typedef-filter-url②"></a>

<a id="element-attrdef-feimage-href"></a>`href` = "[\<url\>](#typedef-filter-url)"

<a id="ref-for-typedef-filter-url③"></a>

<a id="ref-for-element-attrdef-feimage-xlinkhref①"></a>

<a id="ref-for-element-attrdef-feimage-href⑤"></a>

An [\<url\>](#typedef-filter-url) to an image resource or to an element. If both, the [xlink:href](#element-attrdef-feimage-xlinkhref) and the [href](#element-attrdef-feimage-href) attribute are specified, the latter overrides the first definition.

Animatable: yes.

<a id="element-attrdef-feimage-preserveaspectratio"></a>`preserveAspectRatio` = "<em>&#x5B;defer&#x5D; &lt;align&gt; &#x5B;&lt;meetOrSlice&gt;&#x5D;</em>"

<a id="ref-for-element-attrdef-feimage-preserveaspectratio②"></a>

See [preserveAspectRatio](#element-attrdef-feimage-preserveaspectratio).

<a id="ref-for-TermInitialValue④⑧"></a>

<a id="ref-for-element-attrdef-feimage-preserveaspectratio③"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [preserveAspectRatio](#element-attrdef-feimage-preserveaspectratio) is xMidYMid meet.

Animatable: yes.

<a id="element-attrdef-feimage-crossorigin"></a>`crossorigin` = "<em>anonymous</em> \| <em>use-credentials</em>"

<a id="ref-for-elementdef-fedisplacementmap③"></a>

<a id="ref-for-the-img-element"></a>

The [crossorigin](https://www.w3.org/TR/html5/infrastructure.html#cors-settings-attribute) attribute is a CORS settings attribute. Its purpose is to allow images from third-party sites that allow cross-origin access to be used with [feDisplacementMap](#elementdef-fedisplacementmap). For the defintion see [crossorigin](https://www.w3.org/TR/html5/infrastructure.html#cors-settings-attribute) attribute for the [img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element) tag [\[HTML5\]](#biblio-html5) and the [Privacy and Security Considerations](#priv-sec) section in this specification.

Animatable: no.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d975b48d"></a> The following example illustrates how images are placed relative to an object. From left to right:
>
> - <a id="ref-for-filter-region①④"></a>
>
>   The default placement of an image. Note that the image is centered in the [filter region](#filter-region) and has the maximum size that will fit in the region consistent with preserving the aspect ratio.
>
> - The image stretched to fit the bounding box of an object.
>
> - <a id="ref-for-filter-region①⑤"></a>
>
>   <a id="ref-for-element-attrdef-filter-primitive-x①⑤"></a>
>
>   <a id="ref-for-element-attrdef-filter-primitive-y①⑤"></a>
>
>   The image placed using user coordinates. Note that the image is first centered in a box the size of the [filter region](#filter-region) and has the maximum size that will fit in the box consistent with preserving the aspect ratio. This box is then shifted by the given [x](#element-attrdef-filter-primitive-x) and [y](#element-attrdef-filter-primitive-y) values relative to the viewport the object is in.
>
> ```text
> <svg width="600" height="250" viewBox="0 0 600 250"
>      xmlns="http://www.w3.org/2000/svg"
>      xmlns:xlink="http://www.w3.org/1999/xlink">
>   <title>Example feImage - Examples of feImage use</title>
>   <desc>Three examples of using feImage, the first showing the
>         default rendering, the second showing the image fit
>         to a box and the third showing the image
>         shifted and clipped.</desc>
>   <defs>
>     <filter id="Default">
>       <feImage xlink:href="smiley.png" />
>     </filter>
>     <filter id="Fitted" primitiveUnits="objectBoundingBox">
>       <feImage xlink:href="smiley.png"
>          x="0" y="0" width="100%" height="100%"
>          preserveAspectRatio="none"/>
>     </filter>
>     <filter id="Shifted">
>       <feImage xlink:href="smiley.png"
>          x="500" y="5"/>
>     </filter>
>   </defs>
>   <rect fill="none" stroke="blue"
>         x="1" y="1" width="598" height="248"/>
>   <g>
>     <rect x="50"  y="25" width="100" height="200" filter="url(#Default)"/>
>     <rect x="50"  y="25" width="100" height="200" fill="none" stroke="green"/>
>     <rect x="250" y="25" width="100" height="200" filter="url(#Fitted)"/>
>     <rect x="250" y="25" width="100" height="200" fill="none" stroke="green"/>
>     <rect x="450" y="25" width="100" height="200" filter="url(#Shifted)"/>
>     <rect x="450" y="25" width="100" height="200" fill="none" stroke="green"/>
>   </g>
> </svg>
> ```
>
> ![Example feImage — Examples of feImage use](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feImage-01.png)
>
> Example of feImage
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feImage-01.svg)

<a id="ref-for-elementdef-femerge⑥"></a>

### <a id="feMergeElement"></a>9.16. Filter primitive [feMerge](#elementdef-femerge)

<strong>Table 22 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-femerge"></a>`feMerge`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⑧"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-script①⑥"></a>

<a id="ref-for-elementdef-femergenode①"></a>

<a id="ref-for-TermDescriptiveElement①⑤"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [feMergeNode](#elementdef-femergenode), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①②"></a>

  <a id="ref-for-propdef-word-spacing①②"></a>

  <a id="ref-for-propdef-visibility①②"></a>

  <a id="ref-for-propdef-unicode-bidi①②"></a>

  <a id="ref-for-TextRenderingProperty①②"></a>

  <a id="ref-for-propdef-text-decoration①②"></a>

  <a id="ref-for-TextAnchorProperty①②"></a>

  <a id="ref-for-StrokeWidthProperty①②"></a>

  <a id="ref-for-StrokeOpacityProperty①②"></a>

  <a id="ref-for-StrokeMiterlimitProperty①②"></a>

  <a id="ref-for-StrokeLinejoinProperty①②"></a>

  <a id="ref-for-StrokeLinecapProperty①②"></a>

  <a id="ref-for-StrokeDashoffsetProperty①②"></a>

  <a id="ref-for-StrokeDasharrayProperty①②"></a>

  <a id="ref-for-StrokeProperty①③"></a>

  <a id="ref-for-StopOpacityProperty①②"></a>

  <a id="ref-for-StopColorProperty①②"></a>

  <a id="ref-for-ShapeRenderingProperty①②"></a>

  <a id="ref-for-PointerEventsProperty①②"></a>

  <a id="ref-for-propdef-overflow①②"></a>

  <a id="ref-for-propdef-opacity①⑤"></a>

  <a id="ref-for-propdef-mask①②"></a>

  <a id="ref-for-MarkerStartProperty①②"></a>

  <a id="ref-for-MarkerMidProperty①②"></a>

  <a id="ref-for-MarkerEndProperty①②"></a>

  <a id="ref-for-MarkerProperty①②"></a>

  <a id="ref-for-propdef-lighting-color①④"></a>

  <a id="ref-for-propdef-letter-spacing①②"></a>

  <a id="ref-for-KerningProperty①②"></a>

  <a id="ref-for-propdef-isolation①④"></a>

  <a id="ref-for-propdef-image-rendering①⑥"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①②"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①②"></a>

  <a id="ref-for-propdef-font-weight①②"></a>

  <a id="ref-for-propdef-font-variant①②"></a>

  <a id="ref-for-propdef-font-style①②"></a>

  <a id="ref-for-propdef-font-stretch①②"></a>

  <a id="ref-for-propdef-font-size-adjust①②"></a>

  <a id="ref-for-propdef-font-size①②"></a>

  <a id="ref-for-propdef-font-family①②"></a>

  <a id="ref-for-propdef-font①②"></a>

  <a id="ref-for-propdef-flood-opacity②⓪"></a>

  <a id="ref-for-propdef-flood-color①⑨"></a>

  <a id="ref-for-propdef-filter②③"></a>

  <a id="ref-for-FillRuleProperty①②"></a>

  <a id="ref-for-FillOpacityProperty①②"></a>

  <a id="ref-for-FillProperty①④"></a>

  <a id="ref-for-EnableBackgroundProperty①②"></a>

  <a id="ref-for-DominantBaselineProperty①②"></a>

  <a id="ref-for-propdef-display①⑤"></a>

  <a id="ref-for-propdef-direction①②"></a>

  <a id="ref-for-propdef-cursor①②"></a>

  <a id="ref-for-ColorRenderingProperty①②"></a>

  <a id="ref-for-propdef-color-interpolation-filters①⑨"></a>

  <a id="ref-for-ColorInterpolationProperty①⑥"></a>

  <a id="ref-for-color0①②"></a>

  <a id="ref-for-propdef-clip-rule①②"></a>

  <a id="ref-for-propdef-clip-path①②"></a>

  <a id="ref-for-propdef-clip①②"></a>

  <a id="ref-for-BaselineShiftProperty①②"></a>

  <a id="ref-for-AlignmentBaselineProperty①②"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⑤"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①⑥"></a>

  <a id="ref-for-filter-primitive-attributes①①"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEMergeElement](#InterfaceSVGFEMergeElement)

<a id="ref-for-elementdef-femergenode②"></a>

This filter primitive composites input image layers on top of each other using the <em>over</em> operator with <em>Input1</em> (corresponding to the first [feMergeNode](#elementdef-femergenode) child element) on the bottom and the last specified input, <em>InputN</em> (corresponding to the last <a id="ref-for-elementdef-femergenode③"></a>feMergeNode child element), on top.

Many effects produce a number of intermediate layers in order to create the final output image. This filter allows us to collapse those into a single image. Although this could be done by using n-1 Composite-filters, it is more convenient to have this common operation available in this form, and offers the implementation some additional flexibility.

<a id="ref-for-elementdef-femerge⑦"></a>

<a id="ref-for-elementdef-femergenode④"></a>

<a id="ref-for-element-attrdef-filter-primitive-in②⑧"></a>

Each [feMerge](#elementdef-femerge) element can have any number of [feMergeNode](#elementdef-femergenode) subelements, each of which has an [in](#element-attrdef-filter-primitive-in) attribute.

The canonical implementation of feMerge is to render the entire effect into one RGBA layer, and then render the resulting layer on the output device. In certain cases (in particular if the output device itself is a continuous tone device), and since merging is associative, it might be a sufficient approximation to evaluate the effect one layer at a time and render each layer individually onto the output device bottom to top.

<a id="ref-for-attr-valuedef-in-sourcegraphic⑨"></a>

<a id="ref-for-elementdef-femerge⑧"></a>

If the topmost image input is [SourceGraphic](#attr-valuedef-in-sourcegraphic) and this [feMerge](#elementdef-femerge) is the last filter primitive in the filter, the implementation is encouraged to render the layers up to that point, and then render the <a id="ref-for-attr-valuedef-in-sourcegraphic①⓪"></a>SourceGraphic directly from its vector description on top.

<a id="ref-for-elementdef-femerge⑨"></a>

[The example](#intro) at the start of this chapter makes use of the [feMerge](#elementdef-femerge) filter primitive to composite two intermediate filter results together.

<a id="ref-for-elementdef-femergenode⑤"></a>

#### <a id="feMergeNodeElement"></a>9.16.1. Merge node [feMergeNode](#elementdef-femergenode)

<strong>Table 23 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-femergenode"></a>`feMergeNode`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

None.

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①④"></a>

<a id="ref-for-elementdef-script①⑦"></a>

<a id="ref-for-AnimateElement①④"></a>

<a id="ref-for-TermDescriptiveElement①⑥"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in②⑨"></a>

  [in](#element-attrdef-filter-primitive-in)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEMergeNodeElement](#InterfaceSVGFEMergeNodeElement)

<a id="ref-for-elementdef-femorphology②"></a>

### <a id="feMorphologyElement"></a>9.17. Filter primitive [feMorphology](#elementdef-femorphology)

<strong>Table 24 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-femorphology"></a>`feMorphology`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive③⑨"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⑤"></a>

<a id="ref-for-elementdef-script①⑧"></a>

<a id="ref-for-AnimateElement①⑤"></a>

<a id="ref-for-TermDescriptiveElement①⑦"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①③"></a>

  <a id="ref-for-propdef-word-spacing①③"></a>

  <a id="ref-for-propdef-visibility①③"></a>

  <a id="ref-for-propdef-unicode-bidi①③"></a>

  <a id="ref-for-TextRenderingProperty①③"></a>

  <a id="ref-for-propdef-text-decoration①③"></a>

  <a id="ref-for-TextAnchorProperty①③"></a>

  <a id="ref-for-StrokeWidthProperty①③"></a>

  <a id="ref-for-StrokeOpacityProperty①③"></a>

  <a id="ref-for-StrokeMiterlimitProperty①③"></a>

  <a id="ref-for-StrokeLinejoinProperty①③"></a>

  <a id="ref-for-StrokeLinecapProperty①③"></a>

  <a id="ref-for-StrokeDashoffsetProperty①③"></a>

  <a id="ref-for-StrokeDasharrayProperty①③"></a>

  <a id="ref-for-StrokeProperty①④"></a>

  <a id="ref-for-StopOpacityProperty①③"></a>

  <a id="ref-for-StopColorProperty①③"></a>

  <a id="ref-for-ShapeRenderingProperty①③"></a>

  <a id="ref-for-PointerEventsProperty①③"></a>

  <a id="ref-for-propdef-overflow①③"></a>

  <a id="ref-for-propdef-opacity①⑥"></a>

  <a id="ref-for-propdef-mask①③"></a>

  <a id="ref-for-MarkerStartProperty①③"></a>

  <a id="ref-for-MarkerMidProperty①③"></a>

  <a id="ref-for-MarkerEndProperty①③"></a>

  <a id="ref-for-MarkerProperty①③"></a>

  <a id="ref-for-propdef-lighting-color①⑤"></a>

  <a id="ref-for-propdef-letter-spacing①③"></a>

  <a id="ref-for-KerningProperty①③"></a>

  <a id="ref-for-propdef-isolation①⑤"></a>

  <a id="ref-for-propdef-image-rendering①⑦"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①③"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①③"></a>

  <a id="ref-for-propdef-font-weight①③"></a>

  <a id="ref-for-propdef-font-variant①③"></a>

  <a id="ref-for-propdef-font-style①③"></a>

  <a id="ref-for-propdef-font-stretch①③"></a>

  <a id="ref-for-propdef-font-size-adjust①③"></a>

  <a id="ref-for-propdef-font-size①③"></a>

  <a id="ref-for-propdef-font-family①③"></a>

  <a id="ref-for-propdef-font①③"></a>

  <a id="ref-for-propdef-flood-opacity②①"></a>

  <a id="ref-for-propdef-flood-color②⓪"></a>

  <a id="ref-for-propdef-filter②④"></a>

  <a id="ref-for-FillRuleProperty①③"></a>

  <a id="ref-for-FillOpacityProperty①③"></a>

  <a id="ref-for-FillProperty①⑤"></a>

  <a id="ref-for-EnableBackgroundProperty①③"></a>

  <a id="ref-for-DominantBaselineProperty①③"></a>

  <a id="ref-for-propdef-display①⑥"></a>

  <a id="ref-for-propdef-direction①③"></a>

  <a id="ref-for-propdef-cursor①③"></a>

  <a id="ref-for-ColorRenderingProperty①③"></a>

  <a id="ref-for-propdef-color-interpolation-filters②⓪"></a>

  <a id="ref-for-ColorInterpolationProperty①⑦"></a>

  <a id="ref-for-color0①③"></a>

  <a id="ref-for-propdef-clip-rule①③"></a>

  <a id="ref-for-propdef-clip-path①③"></a>

  <a id="ref-for-propdef-clip①③"></a>

  <a id="ref-for-BaselineShiftProperty①③"></a>

  <a id="ref-for-AlignmentBaselineProperty①③"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⑥"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①⑦"></a>

  <a id="ref-for-filter-primitive-attributes①②"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in③⓪"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-femorphology-operator"></a>

  [operator](#element-attrdef-femorphology-operator)

- <a id="ref-for-element-attrdef-femorphology-radius"></a>

  [radius](#element-attrdef-femorphology-radius)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEMorphologyElement](#InterfaceSVGFEMorphologyElement)

This filter primitive performs "fattening" or "thinning" of artwork. It is particularly useful for fattening or thinning an alpha channel.

The dilation (or erosion) kernel is a rectangle with a width of 2\*<em>x-radius</em> and a height of 2\*<em>y-radius</em>. In dilation, the output pixel is the individual component-wise maximum of the corresponding R,G,B,A values in the input image’s kernel rectangle. In erosion, the output pixel is the individual component-wise minimum of the corresponding R,G,B,A values in the input image’s kernel rectangle.

<a id="ref-for-attr-valuedef-in-sourcealpha⑥"></a>

Frequently this operation will take place on alpha-only images, such as that produced by the built-in input, [SourceAlpha](#attr-valuedef-in-sourcealpha). In that case, the implementation might want to optimize the single channel case. This optimization must be omitted if it leads to privacy concerns of any matter. (See section [Privacy and Security Considerations](#priv-sec) for more details.)

<a id="ref-for-attr-valuedef-in-fillpaint⑥"></a>

<a id="ref-for-elementdef-fetile⑤"></a>

If the input has infinite extent and is constant (e.g [FillPaint](#attr-valuedef-in-fillpaint) where the fill is a solid color), this operation has no effect. If the input has infinite extent and the filter result is the input to an [feTile](#elementdef-fetile), the filter is evaluated with [periodic boundary conditions](https://en.wikipedia.org/wiki/Periodic_boundary_conditions).

<a id="ref-for-elementdef-femorphology③"></a>

Because [feMorphology](#elementdef-femorphology) operates on premultipied color values, it will always result in color values less than or equal to the alpha channel.

<em>Attribute definitions:</em>

<a id="element-attrdef-femorphology-operator"></a>`operator` = "<em>erode | dilate</em>"

A keyword indicating whether to erode (i.e., thin) or dilate (fatten) the source graphic.

<a id="ref-for-TermInitialValue④⑨"></a>

<a id="ref-for-element-attrdef-femorphology-operator①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [operator](#element-attrdef-femorphology-operator) is erode.

Animatable: yes.

<a id="ref-for-typedef-number-optional-number⑦"></a>

<a id="element-attrdef-femorphology-radius"></a>`radius` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-number-value②③"></a>

<a id="ref-for-element-attrdef-filter-primitiveunits①⑥"></a>

<a id="ref-for-elementdef-filter③⑧"></a>

The radius (or radii) for the operation. If two [\<number\>](https://www.w3.org/TR/css3-values/#number-value) s are provided, the first number represents a x-radius and the second value represents a y-radius. If one number is provided, then that value is used for both X and Y. The values are in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

A negative or zero value disables the effect of the given filter primitive (i.e., the result is the filter input image).

<a id="ref-for-TermInitialValue⑤⓪"></a>

<a id="ref-for-element-attrdef-femorphology-radius①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [radius](#element-attrdef-femorphology-radius) is 0.

Animatable: yes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-131071e3"></a>
>
> ```text
> <svg width="5cm" height="7cm" viewBox="0 0 700 500"
>      xmlns="http://www.w3.org/2000/svg">
>   <title>Example feMorphology - Examples of erode and dilate</title>
>   <desc>Five text strings drawn as outlines.
>         The first is unfiltered. The second and third use 'erode'.
>         The fourth and fifth use 'dilate'.</desc>
>   <defs>
>     <filter id="Erode3">
>       <feMorphology operator="erode" in="SourceGraphic" radius="3" />
>     </filter>
>     <filter id="Erode6">
>       <feMorphology operator="erode" in="SourceGraphic" radius="6" />
>     </filter>
>     <filter id="Dilate3">
>       <feMorphology operator="dilate" in="SourceGraphic" radius="3" />
>     </filter>
>     <filter id="Dilate6">
>       <feMorphology operator="dilate" in="SourceGraphic" radius="6" />
>     </filter>
>   </defs>
>   <rect fill="none" stroke="blue" stroke-width="2"
>         x="1" y="1" width="698" height="498"/>
>   <g isolation="isolate" >
>     <g font-family="Verdana" font-size="75"
>               fill="none" stroke="black" stroke-width="6" >
>       <text x="50" y="90">Unfiltered</text>
>       <text x="50" y="180" filter="url(#Erode3)" >Erode radius 3</text>
>       <text x="50" y="270" filter="url(#Erode6)" >Erode radius 6</text>
>       <text x="50" y="360" filter="url(#Dilate3)" >Dilate radius 3</text>
>       <text x="50" y="450" filter="url(#Dilate6)" >Dilate radius 6</text>
>     </g>
>   </g>
> </svg>
> ```
>
> ![Example of feMorphology](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feMorphology.png)
>
> Example of feMorphology
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feMorphology.svg)

<a id="ref-for-elementdef-feoffset①①"></a>

### <a id="feOffsetElement"></a>9.18. Filter primitive [feOffset](#elementdef-feoffset)

<strong>Table 25 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feoffset"></a>`feOffset`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive④⓪"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⑥"></a>

<a id="ref-for-elementdef-script①⑨"></a>

<a id="ref-for-AnimateElement①⑥"></a>

<a id="ref-for-TermDescriptiveElement①⑧"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①④"></a>

  <a id="ref-for-propdef-word-spacing①④"></a>

  <a id="ref-for-propdef-visibility①④"></a>

  <a id="ref-for-propdef-unicode-bidi①④"></a>

  <a id="ref-for-TextRenderingProperty①④"></a>

  <a id="ref-for-propdef-text-decoration①④"></a>

  <a id="ref-for-TextAnchorProperty①④"></a>

  <a id="ref-for-StrokeWidthProperty①④"></a>

  <a id="ref-for-StrokeOpacityProperty①④"></a>

  <a id="ref-for-StrokeMiterlimitProperty①④"></a>

  <a id="ref-for-StrokeLinejoinProperty①④"></a>

  <a id="ref-for-StrokeLinecapProperty①④"></a>

  <a id="ref-for-StrokeDashoffsetProperty①④"></a>

  <a id="ref-for-StrokeDasharrayProperty①④"></a>

  <a id="ref-for-StrokeProperty①⑤"></a>

  <a id="ref-for-StopOpacityProperty①④"></a>

  <a id="ref-for-StopColorProperty①④"></a>

  <a id="ref-for-ShapeRenderingProperty①④"></a>

  <a id="ref-for-PointerEventsProperty①④"></a>

  <a id="ref-for-propdef-overflow①④"></a>

  <a id="ref-for-propdef-opacity①⑦"></a>

  <a id="ref-for-propdef-mask①④"></a>

  <a id="ref-for-MarkerStartProperty①④"></a>

  <a id="ref-for-MarkerMidProperty①④"></a>

  <a id="ref-for-MarkerEndProperty①④"></a>

  <a id="ref-for-MarkerProperty①④"></a>

  <a id="ref-for-propdef-lighting-color①⑥"></a>

  <a id="ref-for-propdef-letter-spacing①④"></a>

  <a id="ref-for-KerningProperty①④"></a>

  <a id="ref-for-propdef-isolation①⑥"></a>

  <a id="ref-for-propdef-image-rendering①⑧"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①④"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①④"></a>

  <a id="ref-for-propdef-font-weight①④"></a>

  <a id="ref-for-propdef-font-variant①④"></a>

  <a id="ref-for-propdef-font-style①④"></a>

  <a id="ref-for-propdef-font-stretch①④"></a>

  <a id="ref-for-propdef-font-size-adjust①④"></a>

  <a id="ref-for-propdef-font-size①④"></a>

  <a id="ref-for-propdef-font-family①④"></a>

  <a id="ref-for-propdef-font①④"></a>

  <a id="ref-for-propdef-flood-opacity②②"></a>

  <a id="ref-for-propdef-flood-color②①"></a>

  <a id="ref-for-propdef-filter②⑤"></a>

  <a id="ref-for-FillRuleProperty①④"></a>

  <a id="ref-for-FillOpacityProperty①④"></a>

  <a id="ref-for-FillProperty①⑥"></a>

  <a id="ref-for-EnableBackgroundProperty①④"></a>

  <a id="ref-for-DominantBaselineProperty①④"></a>

  <a id="ref-for-propdef-display①⑦"></a>

  <a id="ref-for-propdef-direction①④"></a>

  <a id="ref-for-propdef-cursor①④"></a>

  <a id="ref-for-ColorRenderingProperty①④"></a>

  <a id="ref-for-propdef-color-interpolation-filters②①"></a>

  <a id="ref-for-ColorInterpolationProperty①⑧"></a>

  <a id="ref-for-color0①④"></a>

  <a id="ref-for-propdef-clip-rule①④"></a>

  <a id="ref-for-propdef-clip-path①④"></a>

  <a id="ref-for-propdef-clip①④"></a>

  <a id="ref-for-BaselineShiftProperty①④"></a>

  <a id="ref-for-AlignmentBaselineProperty①④"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⑦"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①⑧"></a>

  <a id="ref-for-filter-primitive-attributes①③"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in③①"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-feoffset-dx①"></a>

  [dx](#element-attrdef-feoffset-dx)

- <a id="ref-for-element-attrdef-feoffset-dy①"></a>

  [dy](#element-attrdef-feoffset-dy)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEOffsetElement](#InterfaceSVGFEOffsetElement)

This filter primitive offsets the input image relative to its current position in the image space by the specified vector.

This is important for effects like drop shadows.

<a id="ref-for-propdef-image-rendering①⑨"></a>

When applying this filter, the destination location may be offset by a fraction of a pixel in device space. <a id="assert_hqFeOffsetInterpolation"></a>In this case a high quality viewer should make use of appropriate interpolation techniques, for example bilinear or bicubic. This is especially recommended for dynamic viewers where this interpolation provides visually smoother movement of images. For static viewers this is less of a concern. Close attention should be made to the [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering) property setting to determine the authors intent.

<em>Attribute definitions:</em>

<a id="ref-for-number-value②④"></a>

<a id="element-attrdef-feoffset-dx"></a>`dx` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits①⑦"></a>

<a id="ref-for-elementdef-filter③⑨"></a>

The amount to offset the input graphic along the x-axis. The offset amount is expressed in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑤①"></a>

<a id="ref-for-element-attrdef-feoffset-dx②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [dx](#element-attrdef-feoffset-dx) is 0.

Animatable: yes.

<a id="ref-for-number-value②⑤"></a>

<a id="element-attrdef-feoffset-dy"></a>`dy` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits①⑧"></a>

<a id="ref-for-elementdef-filter④⓪"></a>

The amount to offset the input graphic along the y-axis. The offset amount is expressed in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑤②"></a>

<a id="ref-for-element-attrdef-feoffset-dy②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [dy](#element-attrdef-feoffset-dy) is 0.

Animatable: yes.

<a id="ref-for-elementdef-feoffset①②"></a>

[The example](#intro) at the start of this chapter makes use of the [feOffset](#elementdef-feoffset) filter primitive to offset the drop shadow from the original source graphic.

<a id="ref-for-elementdef-fespecularlighting④"></a>

### <a id="feSpecularLightingElement"></a>9.19. Filter primitive [feSpecularLighting](#elementdef-fespecularlighting)

<strong>Table 26 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fespecularlighting"></a>`feSpecularLighting`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive④①"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-light-source⑤"></a>

<a id="ref-for-elementdef-script②⓪"></a>

<a id="ref-for-TermDescriptiveElement①⑨"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script) and exactly one [light sources](#light-source) element, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①⑤"></a>

  <a id="ref-for-propdef-word-spacing①⑤"></a>

  <a id="ref-for-propdef-visibility①⑤"></a>

  <a id="ref-for-propdef-unicode-bidi①⑤"></a>

  <a id="ref-for-TextRenderingProperty①⑤"></a>

  <a id="ref-for-propdef-text-decoration①⑤"></a>

  <a id="ref-for-TextAnchorProperty①⑤"></a>

  <a id="ref-for-StrokeWidthProperty①⑤"></a>

  <a id="ref-for-StrokeOpacityProperty①⑤"></a>

  <a id="ref-for-StrokeMiterlimitProperty①⑤"></a>

  <a id="ref-for-StrokeLinejoinProperty①⑤"></a>

  <a id="ref-for-StrokeLinecapProperty①⑤"></a>

  <a id="ref-for-StrokeDashoffsetProperty①⑤"></a>

  <a id="ref-for-StrokeDasharrayProperty①⑤"></a>

  <a id="ref-for-StrokeProperty①⑥"></a>

  <a id="ref-for-StopOpacityProperty①⑤"></a>

  <a id="ref-for-StopColorProperty①⑤"></a>

  <a id="ref-for-ShapeRenderingProperty①⑤"></a>

  <a id="ref-for-PointerEventsProperty①⑤"></a>

  <a id="ref-for-propdef-overflow①⑤"></a>

  <a id="ref-for-propdef-opacity①⑧"></a>

  <a id="ref-for-propdef-mask①⑤"></a>

  <a id="ref-for-MarkerStartProperty①⑤"></a>

  <a id="ref-for-MarkerMidProperty①⑤"></a>

  <a id="ref-for-MarkerEndProperty①⑤"></a>

  <a id="ref-for-MarkerProperty①⑤"></a>

  <a id="ref-for-propdef-lighting-color①⑦"></a>

  <a id="ref-for-propdef-letter-spacing①⑤"></a>

  <a id="ref-for-KerningProperty①⑤"></a>

  <a id="ref-for-propdef-isolation①⑦"></a>

  <a id="ref-for-propdef-image-rendering②⓪"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①⑤"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①⑤"></a>

  <a id="ref-for-propdef-font-weight①⑤"></a>

  <a id="ref-for-propdef-font-variant①⑤"></a>

  <a id="ref-for-propdef-font-style①⑤"></a>

  <a id="ref-for-propdef-font-stretch①⑤"></a>

  <a id="ref-for-propdef-font-size-adjust①⑤"></a>

  <a id="ref-for-propdef-font-size①⑤"></a>

  <a id="ref-for-propdef-font-family①⑤"></a>

  <a id="ref-for-propdef-font①⑤"></a>

  <a id="ref-for-propdef-flood-opacity②③"></a>

  <a id="ref-for-propdef-flood-color②②"></a>

  <a id="ref-for-propdef-filter②⑥"></a>

  <a id="ref-for-FillRuleProperty①⑤"></a>

  <a id="ref-for-FillOpacityProperty①⑤"></a>

  <a id="ref-for-FillProperty①⑦"></a>

  <a id="ref-for-EnableBackgroundProperty①⑤"></a>

  <a id="ref-for-DominantBaselineProperty①⑤"></a>

  <a id="ref-for-propdef-display①⑧"></a>

  <a id="ref-for-propdef-direction①⑤"></a>

  <a id="ref-for-propdef-cursor①⑤"></a>

  <a id="ref-for-ColorRenderingProperty①⑤"></a>

  <a id="ref-for-propdef-color-interpolation-filters②②"></a>

  <a id="ref-for-ColorInterpolationProperty①⑨"></a>

  <a id="ref-for-color0①⑤"></a>

  <a id="ref-for-propdef-clip-rule①⑤"></a>

  <a id="ref-for-propdef-clip-path①⑤"></a>

  <a id="ref-for-propdef-clip①⑤"></a>

  <a id="ref-for-BaselineShiftProperty①⑤"></a>

  <a id="ref-for-AlignmentBaselineProperty①⑤"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⑧"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y①⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x①⑨"></a>

  <a id="ref-for-filter-primitive-attributes①④"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in③②"></a>

  [in](#element-attrdef-filter-primitive-in)

- <a id="ref-for-element-attrdef-fespecularlighting-surfacescale"></a>

  [surfaceScale](#element-attrdef-fespecularlighting-surfacescale)

- <a id="ref-for-element-attrdef-fespecularlighting-specularconstant"></a>

  [specularConstant](#element-attrdef-fespecularlighting-specularconstant)

- <a id="ref-for-element-attrdef-fespecularlighting-specularexponent"></a>

  [specularExponent](#element-attrdef-fespecularlighting-specularexponent)

- <a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength"></a>

  [kernelUnitLength](#element-attrdef-fespecularlighting-kernelunitlength)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFESpecularLightingElement](#InterfaceSVGFESpecularLightingElement)

This filter primitive lights a source graphic using the alpha channel as a bump map. The resulting image is an RGBA image based on the light color. The lighting calculation follows the standard specular component of the Blinn-Phong lighting model. The resulting image depends on the light color, light position and surface geometry of the input bump map. The result of the lighting calculation is added. The filter primitive assumes that the viewer is at infinity in the z direction (i.e., the unit vector in the eye direction is (0,0,1) everywhere).

<a id="ref-for-elementdef-fecomposite①⓪"></a>

<a id="ref-for-light-source⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This filter primitive produces an image which contains the specular reflection part of the lighting calculation. Such a map is intended to be combined with a texture from a second filter primitive using the <em>add</em> term of the <em>arithmetic</em> [feComposite](#elementdef-fecomposite) method. Multiple [light sources](#light-source) can be simulated by adding several of these light maps before applying it to the texture image.

The resulting RGBA image is computed as follows:

<code> S<sub>r</sub> = k<sub>s</sub> &#x2A; pow(N.H, specularExponent) &#x2A; L<sub>r<br></sub> S<sub>g</sub> = k<sub>s</sub> &#x2A; pow(N.H, specularExponent) &#x2A; L<sub>g<br></sub> S<sub>b</sub> = k<sub>s</sub> &#x2A; pow(N.H, specularExponent) &#x2A; L<sub>b<br></sub> S<sub>a</sub> = max(S<sub>r,</sub> S<sub>g,</sub> S<sub>b</sub>) </code>

where

k<sub>s</sub> = specular lighting constant  
N = surface normal unit vector, a function of x and y  
H = "halfway" unit vector between eye unit vector and light unit vector  
  
L<sub>r</sub>,L<sub>g</sub>,L<sub>b</sub> = RGB components of light

<a id="ref-for-elementdef-fediffuselighting⑤"></a>

See [feDiffuseLighting](#elementdef-fediffuselighting) for definition of N and (L<sub>r</sub>, L<sub>g</sub>, L<sub>b</sub>).

The definition of H reflects our assumption of the constant eye vector E = (0,0,1):

`  H = (L + E) / Norm(L+E)  `

where L is the light unit vector.

<a id="ref-for-elementdef-fediffuselighting⑥"></a>

<a id="ref-for-elementdef-fespecularlighting⑤"></a>

Unlike the [feDiffuseLighting](#elementdef-fediffuselighting), the [feSpecularLighting](#elementdef-fespecularlighting) filter produces a non-opaque image. This is due to the fact that the specular result (S<sub>r</sub>,S<sub>g</sub>,S<sub>b</sub>,S<sub>a</sub>) is meant to be added to the textured image. The alpha channel of the result is the max of the color components, so that where the specular light is zero, no additional coverage is added to the image and a fully white highlight will add opacity.

<a id="ref-for-elementdef-fediffuselighting⑦"></a>

<a id="ref-for-elementdef-fespecularlighting⑥"></a>

The [feDiffuseLighting](#elementdef-fediffuselighting) and [feSpecularLighting](#elementdef-fespecularlighting) filters will often be applied together. An implementation may detect this and calculate both maps in one pass, instead of two.

<em>Attribute definitions:</em>

<a id="ref-for-number-value②⑥"></a>

<a id="element-attrdef-fespecularlighting-surfacescale"></a>`surfaceScale` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

height of surface when A<sub>in</sub> = 1.

<a id="ref-for-TermInitialValue⑤③"></a>

<a id="ref-for-element-attrdef-fespecularlighting-surfacescale①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [surfaceScale](#element-attrdef-fespecularlighting-surfacescale) is 1.

Animatable: yes.

<a id="ref-for-number-value②⑦"></a>

<a id="element-attrdef-fespecularlighting-specularconstant"></a>`specularConstant` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

ks in Phong lighting model. In SVG, this can be any non-negative number.

<a id="ref-for-TermInitialValue⑤④"></a>

<a id="ref-for-element-attrdef-fespecularlighting-specularconstant①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [specularConstant](#element-attrdef-fespecularlighting-specularconstant) is 1.

Animatable: yes.

<a id="ref-for-number-value②⑧"></a>

<a id="element-attrdef-fespecularlighting-specularexponent"></a>`specularExponent` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

Exponent for specular term, larger is more "shiny".

<a id="ref-for-TermInitialValue⑤⑤"></a>

<a id="ref-for-element-attrdef-fespecularlighting-specularexponent①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [specularExponent](#element-attrdef-fespecularlighting-specularexponent) is 1.

Animatable: yes.

<a id="ref-for-typedef-number-optional-number⑧"></a>

<a id="element-attrdef-fespecularlighting-kernelunitlength"></a>`kernelUnitLength` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

The first number is the \<dx\> value. The second number is the \<dy\> value.

<a id="ref-for-element-attrdef-filter-primitiveunits①⑨"></a>

<a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength①"></a>

If the \<dy\> value is not specified, it defaults to the same value as \<dx\>. Indicates the intended distance in current filter units (i.e., units as determined by the value of attribute [primitiveUnits](#element-attrdef-filter-primitiveunits)) for `dx` and `dy`, respectively, in the [surface normal calculation formulas](#SurfaceNormalCalculations). By specifying value(s) for [kernelUnitLength](#element-attrdef-fespecularlighting-kernelunitlength), the kernel becomes defined in a scalable, abstract coordinate system.

<a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength②"></a>

If [kernelUnitLength](#element-attrdef-fespecularlighting-kernelunitlength) is not specified, the `dx` and `dy` values should represent very small deltas relative to a given `(x,y)` position, which might be implemented in some cases as one pixel in the intermediate image offscreen bitmap, which is a pixel-based coordinate system, and thus potentially not scalable. For some level of consistency across display media and user agents, it is necessary that a value be provided for <a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength③"></a>kernelUnitLength.

Animatable: yes.

<a id="ref-for-elementdef-fedistantlight①"></a>

<a id="ref-for-elementdef-fepointlight①"></a>

<a id="ref-for-elementdef-fespotlight②"></a>

<a id="ref-for-propdef-lighting-color①⑧"></a>

The light source is defined by one of the child elements [feDistantLight](#elementdef-fedistantlight), [fePointLight](#elementdef-fepointlight) or [feSpotLight](#elementdef-fespotlight). The light color is specified by property [lighting-color](#propdef-lighting-color).

<a id="ref-for-elementdef-fespecularlighting⑦"></a>

[The example](#intro) at the start of this chapter makes use of the [feSpecularLighting](#elementdef-fespecularlighting) filter primitive to achieve a highly reflective, 3D glowing effect.

<a id="ref-for-elementdef-fetile⑥"></a>

### <a id="feTileElement"></a>9.20. Filter primitive [feTile](#elementdef-fetile)

<strong>Table 27 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fetile"></a>`feTile`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive④②"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⑦"></a>

<a id="ref-for-elementdef-script②①"></a>

<a id="ref-for-AnimateElement①⑦"></a>

<a id="ref-for-TermDescriptiveElement②⓪"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①⑥"></a>

  <a id="ref-for-propdef-word-spacing①⑥"></a>

  <a id="ref-for-propdef-visibility①⑥"></a>

  <a id="ref-for-propdef-unicode-bidi①⑥"></a>

  <a id="ref-for-TextRenderingProperty①⑥"></a>

  <a id="ref-for-propdef-text-decoration①⑥"></a>

  <a id="ref-for-TextAnchorProperty①⑥"></a>

  <a id="ref-for-StrokeWidthProperty①⑥"></a>

  <a id="ref-for-StrokeOpacityProperty①⑥"></a>

  <a id="ref-for-StrokeMiterlimitProperty①⑥"></a>

  <a id="ref-for-StrokeLinejoinProperty①⑥"></a>

  <a id="ref-for-StrokeLinecapProperty①⑥"></a>

  <a id="ref-for-StrokeDashoffsetProperty①⑥"></a>

  <a id="ref-for-StrokeDasharrayProperty①⑥"></a>

  <a id="ref-for-StrokeProperty①⑦"></a>

  <a id="ref-for-StopOpacityProperty①⑥"></a>

  <a id="ref-for-StopColorProperty①⑥"></a>

  <a id="ref-for-ShapeRenderingProperty①⑥"></a>

  <a id="ref-for-PointerEventsProperty①⑥"></a>

  <a id="ref-for-propdef-overflow①⑥"></a>

  <a id="ref-for-propdef-opacity①⑨"></a>

  <a id="ref-for-propdef-mask①⑥"></a>

  <a id="ref-for-MarkerStartProperty①⑥"></a>

  <a id="ref-for-MarkerMidProperty①⑥"></a>

  <a id="ref-for-MarkerEndProperty①⑥"></a>

  <a id="ref-for-MarkerProperty①⑥"></a>

  <a id="ref-for-propdef-lighting-color①⑨"></a>

  <a id="ref-for-propdef-letter-spacing①⑥"></a>

  <a id="ref-for-KerningProperty①⑥"></a>

  <a id="ref-for-propdef-isolation①⑧"></a>

  <a id="ref-for-propdef-image-rendering②①"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①⑥"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①⑥"></a>

  <a id="ref-for-propdef-font-weight①⑥"></a>

  <a id="ref-for-propdef-font-variant①⑥"></a>

  <a id="ref-for-propdef-font-style①⑥"></a>

  <a id="ref-for-propdef-font-stretch①⑥"></a>

  <a id="ref-for-propdef-font-size-adjust①⑥"></a>

  <a id="ref-for-propdef-font-size①⑥"></a>

  <a id="ref-for-propdef-font-family①⑥"></a>

  <a id="ref-for-propdef-font①⑥"></a>

  <a id="ref-for-propdef-flood-opacity②④"></a>

  <a id="ref-for-propdef-flood-color②③"></a>

  <a id="ref-for-propdef-filter②⑦"></a>

  <a id="ref-for-FillRuleProperty①⑥"></a>

  <a id="ref-for-FillOpacityProperty①⑥"></a>

  <a id="ref-for-FillProperty①⑧"></a>

  <a id="ref-for-EnableBackgroundProperty①⑥"></a>

  <a id="ref-for-DominantBaselineProperty①⑥"></a>

  <a id="ref-for-propdef-display①⑨"></a>

  <a id="ref-for-propdef-direction①⑥"></a>

  <a id="ref-for-propdef-cursor①⑥"></a>

  <a id="ref-for-ColorRenderingProperty①⑥"></a>

  <a id="ref-for-propdef-color-interpolation-filters②③"></a>

  <a id="ref-for-ColorInterpolationProperty②⓪"></a>

  <a id="ref-for-color0①⑥"></a>

  <a id="ref-for-propdef-clip-rule①⑥"></a>

  <a id="ref-for-propdef-clip-path①⑥"></a>

  <a id="ref-for-propdef-clip①⑥"></a>

  <a id="ref-for-BaselineShiftProperty①⑥"></a>

  <a id="ref-for-AlignmentBaselineProperty①⑥"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result①⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height①⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width①⑨"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y②⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x②⓪"></a>

  <a id="ref-for-filter-primitive-attributes①⑤"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-filter-primitive-in③③"></a>

  [in](#element-attrdef-filter-primitive-in)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFETileElement](#InterfaceSVGFETileElement)

<a id="ref-for-filter-primitive-subregion①⑧"></a>

<a id="ref-for-elementdef-fetile⑦"></a>

This filter primitive fills a target rectangle with a repeated, tiled pattern of an input image. The target rectangle is as large as the [filter primitive subregion](#filter-primitive-subregion) established by the [feTile](#elementdef-fetile) element.

<a id="ref-for-filter-primitive-subregion①⑨"></a>

<a id="ref-for-elementdef-fetile⑧"></a>

Typically, the input image has been defined with its own [filter primitive subregion](#filter-primitive-subregion) in order to define a reference tile. [feTile](#elementdef-fetile) replicates the reference tile in both X and Y to completely fill the target rectangle. The top/left corner of each given tile is at location `(x + i*width, y + j*height)`, where `(x,y)` represents the top/left of the input image’s <a id="ref-for-filter-primitive-subregion②⓪"></a>filter primitive subregion, `width` and `height` represent the width and height of the input image’s <a id="ref-for-filter-primitive-subregion②①"></a>filter primitive subregion, and `i` and `j` can be any integer value. In most cases, the input image will have a smaller <a id="ref-for-filter-primitive-subregion②②"></a>filter primitive subregion than the <a id="ref-for-elementdef-fetile⑨"></a>feTile in order to achieve a repeated pattern effect.

Implementers must take appropriate measures in constructing the tiled image to avoid artifacts between tiles, particularly in situations where the user to device transform includes shear and/or rotation. Unless care is taken, interpolation can lead to edge pixels in the tile having opacity values lower or higher than expected due to the interaction of painting adjacent tiles which each have partial overlap with particular pixels.

<a id="ref-for-elementdef-feturbulence③"></a>

### <a id="feTurbulenceElement"></a>9.21. Filter primitive [feTurbulence](#elementdef-feturbulence)

<strong>Table 28 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-feturbulence"></a>`feTurbulence`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive④③"></a>

[filter primitive](#filter-primitive)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⑧"></a>

<a id="ref-for-elementdef-script②②"></a>

<a id="ref-for-AnimateElement①⑧"></a>

<a id="ref-for-TermDescriptiveElement②①"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①⑦"></a>

  <a id="ref-for-propdef-word-spacing①⑦"></a>

  <a id="ref-for-propdef-visibility①⑦"></a>

  <a id="ref-for-propdef-unicode-bidi①⑦"></a>

  <a id="ref-for-TextRenderingProperty①⑦"></a>

  <a id="ref-for-propdef-text-decoration①⑦"></a>

  <a id="ref-for-TextAnchorProperty①⑦"></a>

  <a id="ref-for-StrokeWidthProperty①⑦"></a>

  <a id="ref-for-StrokeOpacityProperty①⑦"></a>

  <a id="ref-for-StrokeMiterlimitProperty①⑦"></a>

  <a id="ref-for-StrokeLinejoinProperty①⑦"></a>

  <a id="ref-for-StrokeLinecapProperty①⑦"></a>

  <a id="ref-for-StrokeDashoffsetProperty①⑦"></a>

  <a id="ref-for-StrokeDasharrayProperty①⑦"></a>

  <a id="ref-for-StrokeProperty①⑧"></a>

  <a id="ref-for-StopOpacityProperty①⑦"></a>

  <a id="ref-for-StopColorProperty①⑦"></a>

  <a id="ref-for-ShapeRenderingProperty①⑦"></a>

  <a id="ref-for-PointerEventsProperty①⑦"></a>

  <a id="ref-for-propdef-overflow①⑦"></a>

  <a id="ref-for-propdef-opacity②⓪"></a>

  <a id="ref-for-propdef-mask①⑦"></a>

  <a id="ref-for-MarkerStartProperty①⑦"></a>

  <a id="ref-for-MarkerMidProperty①⑦"></a>

  <a id="ref-for-MarkerEndProperty①⑦"></a>

  <a id="ref-for-MarkerProperty①⑦"></a>

  <a id="ref-for-propdef-lighting-color②⓪"></a>

  <a id="ref-for-propdef-letter-spacing①⑦"></a>

  <a id="ref-for-KerningProperty①⑦"></a>

  <a id="ref-for-propdef-isolation①⑨"></a>

  <a id="ref-for-propdef-image-rendering②②"></a>

  <a id="ref-for-GlyphOrientationVerticalProperty①⑦"></a>

  <a id="ref-for-GlyphOrientationHorizontalProperty①⑦"></a>

  <a id="ref-for-propdef-font-weight①⑦"></a>

  <a id="ref-for-propdef-font-variant①⑦"></a>

  <a id="ref-for-propdef-font-style①⑦"></a>

  <a id="ref-for-propdef-font-stretch①⑦"></a>

  <a id="ref-for-propdef-font-size-adjust①⑦"></a>

  <a id="ref-for-propdef-font-size①⑦"></a>

  <a id="ref-for-propdef-font-family①⑦"></a>

  <a id="ref-for-propdef-font①⑦"></a>

  <a id="ref-for-propdef-flood-opacity②⑤"></a>

  <a id="ref-for-propdef-flood-color②④"></a>

  <a id="ref-for-propdef-filter②⑧"></a>

  <a id="ref-for-FillRuleProperty①⑦"></a>

  <a id="ref-for-FillOpacityProperty①⑦"></a>

  <a id="ref-for-FillProperty①⑨"></a>

  <a id="ref-for-EnableBackgroundProperty①⑦"></a>

  <a id="ref-for-DominantBaselineProperty①⑦"></a>

  <a id="ref-for-propdef-display②⓪"></a>

  <a id="ref-for-propdef-direction①⑦"></a>

  <a id="ref-for-propdef-cursor①⑦"></a>

  <a id="ref-for-ColorRenderingProperty①⑦"></a>

  <a id="ref-for-propdef-color-interpolation-filters②④"></a>

  <a id="ref-for-ColorInterpolationProperty②①"></a>

  <a id="ref-for-color0①⑦"></a>

  <a id="ref-for-propdef-clip-rule①⑦"></a>

  <a id="ref-for-propdef-clip-path①⑦"></a>

  <a id="ref-for-propdef-clip①⑦"></a>

  <a id="ref-for-BaselineShiftProperty①⑦"></a>

  <a id="ref-for-AlignmentBaselineProperty①⑦"></a>

  [presentation attributes](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/SVG11/text.html#AlignmentBaselineProperty), [baseline-shift](https://www.w3.org/TR/SVG11/text.html#BaselineShiftProperty), [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip), [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), [clip-rule](https://www.w3.org/TR/css-masking-1/#propdef-clip-rule), [color](https://www.w3.org/TR/css3-color/#color0), [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](#propdef-color-interpolation-filters), [color-rendering](https://www.w3.org/TR/svg2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css3-ui/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/svg2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/svg2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/svg2/painting.html#FillRuleProperty), [filter](#propdef-filter), [flood-color](#propdef-flood-color), [flood-opacity](#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-3/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-3/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-4/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-3/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-3/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationVerticalProperty), [image-rendering](https://drafts.csswg.org/css-images-3/#propdef-image-rendering), [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](#propdef-lighting-color), [marker](https://www.w3.org/TR/svg2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/svg2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/svg2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/svg2/painting.html#MarkerStartProperty), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/svg2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/svg2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/svg2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/svg2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/svg2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/svg2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/svg2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/svg2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/svg2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/svg2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/svg2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/svg2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/svg2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/svg2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- <a id="ref-for-element-attrdef-filter-primitive-result②⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-height②⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-width②⓪"></a>

  <a id="ref-for-element-attrdef-filter-primitive-y②①"></a>

  <a id="ref-for-element-attrdef-filter-primitive-x②①"></a>

  <a id="ref-for-filter-primitive-attributes①⑥"></a>

  [filter primitive attributes](#filter-primitive-attributes) —[x](#element-attrdef-filter-primitive-x), [y](#element-attrdef-filter-primitive-y), [width](#element-attrdef-filter-primitive-width), [height](#element-attrdef-filter-primitive-height), [result](#element-attrdef-filter-primitive-result)

- [class](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [style](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-feturbulence-basefrequency"></a>

  [baseFrequency](#element-attrdef-feturbulence-basefrequency)

- <a id="ref-for-element-attrdef-feturbulence-numoctaves"></a>

  [numOctaves](#element-attrdef-feturbulence-numoctaves)

- <a id="ref-for-element-attrdef-feturbulence-seed"></a>

  [seed](#element-attrdef-feturbulence-seed)

- <a id="ref-for-element-attrdef-feturbulence-stitchtiles"></a>

  [stitchTiles](#element-attrdef-feturbulence-stitchtiles)

- <a id="ref-for-element-attrdef-feturbulence-type"></a>

  [type](#element-attrdef-feturbulence-type)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFETurbulenceElement](#InterfaceSVGFETurbulenceElement)

<a id="ref-for-filter-primitive-subregion②③"></a>

This filter primitive creates an image using the Perlin turbulence function. It allows the synthesis of artificial textures like clouds or marble. For a detailed description the of the Perlin turbulence function, see "Texturing and Modeling" [\[TaM\]](#biblio-tam). The resulting image will fill the entire [filter primitive subregion](#filter-primitive-subregion) for this filter primitive.

It is possible to create bandwidth-limited noise by synthesizing only one octave.

<a id="ref-for-filter-primitive-subregion②④"></a>

The C code below shows the exact algorithm used for this filter effect. The [filter primitive subregion](#filter-primitive-subregion) is to be passed as the arguments fTileX, fTileY, fTileWidth and fTileHeight.

For fractalSum, you get a turbFunctionResult that is aimed at a range of -1 to 1 (the actual result might exceed this range in some cases). To convert to a color or alpha value, use the formula `colorValue = (turbFunctionResult + 1) / 2`, then clamp to the range 0 to 1.

For turbulence, you get a turbFunctionResult that is aimed at a range of 0 to 1 (the actual result might exceed this range in some cases). To convert to a color or alpha value, use the formula `colorValue = turbFunctionResult`, then clamp to the range 0 to 1.

<a id="ref-for-element-attrdef-feturbulence-seed①"></a>

The following order is used for applying the pseudo random numbers. An initial seed value is computed based on the [seed](#element-attrdef-feturbulence-seed) attribute. Then the implementation computes the lattice points for R, then continues getting additional pseudo random numbers relative to the last generated pseudo random number and computes the lattice points for G, and so on for B and A.

<a id="ref-for-propdef-color-interpolation-filters②⑤"></a>

The generated color and alpha values are in the color space determined by the [color-interpolation-filters](#propdef-color-interpolation-filters) property:

```text
/* Produces results in the range [1, 2**31 - 2].
Algorithm is: r = (a * r) mod m
where a = 16807 and m = 2**31 - 1 = 2147483647
See [Park & Miller], CACM vol. 31 no. 10 p. 1195, Oct. 1988
To test: the algorithm should produce the result 1043618065
as the 10,000th generated number if the original seed is 1.
*/
#define RAND_m 2147483647 /* 2**31 - 1 */
#define RAND_a 16807 /* 7**5; primitive root of m */
#define RAND_q 127773 /* m / a */
#define RAND_r 2836 /* m % a */
long setup_seed(long lSeed)
{
  if (lSeed <= 0) lSeed = -(lSeed % (RAND_m - 1)) + 1;
  if (lSeed > RAND_m - 1) lSeed = RAND_m - 1;
  return lSeed;
}
long random(long lSeed)
{
  long result;
  result = RAND_a * (lSeed % RAND_q) - RAND_r * (lSeed / RAND_q);
  if (result <= 0) result += RAND_m;
  return result;
}
#define BSize 0x100
#define BM 0xff
#define PerlinN 0x1000
#define NP 12 /* 2^PerlinN */
#define NM 0xfff
static uLatticeSelector[BSize + BSize + 2];
static double fGradient[4][BSize + BSize + 2][2];
struct StitchInfo
{
  int nWidth; // How much to subtract to wrap for stitching.
  int nHeight;
  int nWrapX; // Minimum value to wrap.
  int nWrapY;
};
static void init(long lSeed)
{
  double s;
  int i, j, k;
  lSeed = setup_seed(lSeed);
  for(k = 0; k < 4; k++)
  {
    for(i = 0; i < BSize; i++)
    {
      uLatticeSelector[i] = i;
      do {
         for (j = 0; j < 2; j++)
           fGradient[k][i][j] = (double)(((lSeed = random(lSeed)) % (BSize + BSize)) - BSize) / BSize;
      } while(fGradient[k][i][0] == 0 && fGradient[k][i][1] == 0);
      s = double(sqrt(fGradient[k][i][0] * fGradient[k][i][0] + fGradient[k][i][1] * fGradient[k][i][1]));
      if (s > 1) {
          i--; // discard the current random vector; try it again.
          continue;
      }
      fGradient[k][i][0] /= s;
      fGradient[k][i][1] /= s;
    }
  }
  while(--i)
  {
    k = uLatticeSelector[i];
    uLatticeSelector[i] = uLatticeSelector[j = (lSeed = random(lSeed)) % BSize];
    uLatticeSelector[j] = k;
  }
  for(i = 0; i < BSize + 2; i++)
  {
    uLatticeSelector[BSize + i] = uLatticeSelector[i];
    for(k = 0; k < 4; k++)
      for(j = 0; j < 2; j++)
        fGradient[k][BSize + i][j] = fGradient[k][i][j];
  }
}
#define s_curve(t) ( t * t * (3. - 2. * t) )
#define lerp(t, a, b) ( a + t * (b - a) )
double noise2(int nColorChannel, double vec[2], StitchInfo *pStitchInfo)
{
  int bx0, bx1, by0, by1, b00, b10, b01, b11;
  double rx0, rx1, ry0, ry1, *q, sx, sy, a, b, t, u, v;
  register i, j;
  t = vec[0] + PerlinN;
  bx0 = (int)t;
  bx1 = bx0+1;
  rx0 = t - (int)t;
  rx1 = rx0 - 1.0f;
  t = vec[1] + PerlinN;
  by0 = (int)t;
  by1 = by0+1;
  ry0 = t - (int)t;
  ry1 = ry0 - 1.0f;
  // If stitching, adjust lattice points accordingly.
  if(pStitchInfo != NULL)
  {
    if(bx0 >= pStitchInfo->nWrapX)
      bx0 -= pStitchInfo->nWidth;
    if(bx1 >= pStitchInfo->nWrapX)
      bx1 -= pStitchInfo->nWidth;
    if(by0 >= pStitchInfo->nWrapY)
      by0 -= pStitchInfo->nHeight;
    if(by1 >= pStitchInfo->nWrapY)
      by1 -= pStitchInfo->nHeight;
  }
  bx0 &= BM;
  bx1 &= BM;
  by0 &= BM;
  by1 &= BM;
  i = uLatticeSelector[bx0];
  j = uLatticeSelector[bx1];
  b00 = uLatticeSelector[i + by0];
  b10 = uLatticeSelector[j + by0];
  b01 = uLatticeSelector[i + by1];
  b11 = uLatticeSelector[j + by1];
  sx = double(s_curve(rx0));
  sy = double(s_curve(ry0));
  q = fGradient[nColorChannel][b00]; u = rx0 * q[0] + ry0 * q[1];
  q = fGradient[nColorChannel][b10]; v = rx1 * q[0] + ry0 * q[1];
  a = lerp(sx, u, v);
  q = fGradient[nColorChannel][b01]; u = rx0 * q[0] + ry1 * q[1];
  q = fGradient[nColorChannel][b11]; v = rx1 * q[0] + ry1 * q[1];
  b = lerp(sx, u, v);
  return lerp(sy, a, b);
}
double turbulence(int nColorChannel, double *point, double fBaseFreqX, double fBaseFreqY,
          int nNumOctaves, bool bFractalSum, bool bDoStitching,
          double fTileX, double fTileY, double fTileWidth, double fTileHeight)
{
  StitchInfo stitch;
  StitchInfo *pStitchInfo = NULL; // Not stitching when NULL.
  // Adjust the base frequencies if necessary for stitching.
  if(bDoStitching)
  {
    // When stitching tiled turbulence, the frequencies must be adjusted
    // so that the tile borders will be continuous.
    if(fBaseFreqX != 0.0)
    {
      double fLoFreq = double(floor(fTileWidth * fBaseFreqX)) / fTileWidth;
      double fHiFreq = double(ceil(fTileWidth * fBaseFreqX)) / fTileWidth;
      if(fBaseFreqX / fLoFreq < fHiFreq / fBaseFreqX)
        fBaseFreqX = fLoFreq;
      else
        fBaseFreqX = fHiFreq;
    }
    if(fBaseFreqY != 0.0)
    {
      double fLoFreq = double(floor(fTileHeight * fBaseFreqY)) / fTileHeight;
      double fHiFreq = double(ceil(fTileHeight * fBaseFreqY)) / fTileHeight;
      if(fBaseFreqY / fLoFreq < fHiFreq / fBaseFreqY)
        fBaseFreqY = fLoFreq;
      else
        fBaseFreqY = fHiFreq;
    }
    // Set up initial stitch values.
    pStitchInfo = &stitch;
    stitch.nWidth = int(fTileWidth * fBaseFreqX + 0.5f);
    stitch.nWrapX = fTileX * fBaseFreqX + PerlinN + stitch.nWidth;
    stitch.nHeight = int(fTileHeight * fBaseFreqY + 0.5f);
    stitch.nWrapY = fTileY * fBaseFreqY + PerlinN + stitch.nHeight;
  }
  double fSum = 0.0f;
  double vec[2];
  vec[0] = point[0] * fBaseFreqX;
  vec[1] = point[1] * fBaseFreqY;
  double ratio = 1;
  for(int nOctave = 0; nOctave < nNumOctaves; nOctave++)
  {
    if(bFractalSum)
      fSum += double(noise2(nColorChannel, vec, pStitchInfo) / ratio);
    else
      fSum += double(fabs(noise2(nColorChannel, vec, pStitchInfo)) / ratio);
    vec[0] *= 2;
    vec[1] *= 2;
    ratio *= 2;
    if(pStitchInfo != NULL)
    {
      // Update stitch values. Subtracting PerlinN before the multiplication and
      // adding it afterward simplifies to subtracting it once.
      stitch.nWidth *= 2;
      stitch.nWrapX = 2 * stitch.nWrapX - PerlinN;
      stitch.nHeight *= 2;
      stitch.nWrapY = 2 * stitch.nWrapY - PerlinN;
    }
  }
  return fSum;
}
```
<em>Attribute definitions:</em>

<a id="ref-for-typedef-number-optional-number⑨"></a>

<a id="element-attrdef-feturbulence-basefrequency"></a>`baseFrequency` = "<em><a href="#typedef-number-optional-number">&lt;number-optional-number&gt;</a></em>"

<a id="ref-for-number-value②⑨"></a>

The base frequency (frequencies) parameter(s) for the noise function. If two [\<number\>](https://www.w3.org/TR/css3-values/#number-value)s are provided, the first number represents a base frequency in the X direction and the second value represents a base frequency in the Y direction. If one number is provided, then that value is used for both X and Y.

<a id="ref-for-TermInitialValue⑤⑥"></a>

<a id="ref-for-element-attrdef-feturbulence-basefrequency①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [baseFrequency](#element-attrdef-feturbulence-basefrequency) is 0.

<a id="ref-for-TermUnsupportedValue"></a>

Negative values are [unsupported](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermUnsupportedValue).

Animatable: yes.

<a id="ref-for-integer-value③"></a>

<a id="element-attrdef-feturbulence-numoctaves"></a>`numOctaves` = "<em><a href="https://www.w3.org/TR/css3-values/#integer-value">&lt;integer&gt;</a></em>"

The numOctaves parameter for the noise function.

<a id="ref-for-TermInitialValue⑤⑦"></a>

<a id="ref-for-element-attrdef-feturbulence-numoctaves①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [numOctaves](#element-attrdef-feturbulence-numoctaves) is 1.

<a id="ref-for-TermUnsupportedValue①"></a>

Negative values are [unsupported](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermUnsupportedValue).

Animatable: yes.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The contribution of each additional octave to the color and alpha values in the final image is one-half of the preceding octave. At some point, the contribution of additional octaves becomes smaller than the color resolution for a given color depth. UAs may clamp the specified value for numOctaves during the processing depending on the supported color depth. (For example: For a color depth of 8 bits per channel, the UA may clamp the value of numOctaves to 9.)

<a id="ref-for-number-value③⓪"></a>

<a id="element-attrdef-feturbulence-seed"></a>`seed` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

The starting number for the pseudo random number generator.

<a id="ref-for-TermInitialValue⑤⑧"></a>

<a id="ref-for-element-attrdef-feturbulence-seed②"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [seed](#element-attrdef-feturbulence-seed) is 0.

When the seed number is handed over to the algorithm above it must first be truncated, i.e. rounded to the closest integer value towards zero.

Animatable: yes.

<a id="element-attrdef-feturbulence-stitchtiles"></a>`stitchTiles` = "<em>stitch | noStitch</em>"

If `stitchTiles="noStitch"`, no attempt is made to achieve smooth transitions at the border of tiles which contain a turbulence function. Sometimes the result will show clear discontinuities at the tile borders.

<a id="ref-for-elementdef-feturbulence④"></a>

If `stitchTiles="stitch"`, then the user agent will automatically adjust baseFrequency-x and baseFrequency-y values such that the [feTurbulence](#elementdef-feturbulence) node’s width and height (i.e., the width and height of the current subregion) contains an integral number of the Perlin tile width and height for the first octave. The baseFrequency will be adjusted up or down depending on which way has the smallest relative (not absolute) change as follows: Given the frequency, calculate `lowFreq=floor(width*frequency)/width` and `hiFreq=ceil(width*frequency)/width`. If frequency/lowFreq \< hiFreq/frequency then use lowFreq, else use hiFreq. While generating turbulence values, generate lattice vectors as normal for Perlin Noise, except for those lattice points that lie on the right or bottom edges of the active area (the size of the resulting tile). In those cases, copy the lattice vector from the opposite edge of the active area.

<a id="ref-for-TermInitialValue⑤⑨"></a>

<a id="ref-for-element-attrdef-feturbulence-stitchtiles①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [stitchTiles](#element-attrdef-feturbulence-stitchtiles) is noStitch.

Animatable: yes.

<a id="element-attrdef-feturbulence-type"></a>`type` = "<em>fractalNoise | turbulence</em>"

Indicates whether the filter primitive should perform a noise or turbulence function.

<a id="ref-for-TermInitialValue⑥⓪"></a>

<a id="ref-for-element-attrdef-feturbulence-type①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [type](#element-attrdef-feturbulence-type) is turbulence.

Animatable: yes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a47584c3"></a>
>
> ```text
> <svg width="450px" height="325px" viewBox="0 0 450 325"
>      xmlns="http://www.w3.org/2000/svg">
>   <title>Example feTurbulence - Examples of feTurbulence operations</title>
>   <desc>Six rectangular areas showing the effects of
>         various parameter settings for feTurbulence.</desc>
>   <g  font-family="Verdana" text-anchor="middle" font-size="10" >
>     <defs>
>       <filter id="Turb1" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="turbulence" baseFrequency="0.05" numOctaves="2"/>
>       </filter>
>       <filter id="Turb2" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="turbulence" baseFrequency="0.1" numOctaves="2"/>
>       </filter>
>       <filter id="Turb3" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="turbulence" baseFrequency="0.05" numOctaves="8"/>
>       </filter>
>       <filter id="Turb4" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="fractalNoise" baseFrequency="0.1" numOctaves="4"/>
>       </filter>
>       <filter id="Turb5" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="fractalNoise" baseFrequency="0.4" numOctaves="4"/>
>       </filter>
>       <filter id="Turb6" filterUnits="objectBoundingBox"
>               x="0%" y="0%" width="100%" height="100%">
>         <feTurbulence type="fractalNoise" baseFrequency="0.1" numOctaves="1"/>
>       </filter>
>     </defs>
> 
>     <rect x="1" y="1" width="448" height="323"
>           fill="none" stroke="blue" stroke-width="1"  />
> 
>     <rect x="25" y="25" width="100" height="75" filter="url(#Turb1)"  />
>     <text x="75" y="117">type=turbulence</text>
>     <text x="75" y="129">baseFrequency=0.05</text>
>     <text x="75" y="141">numOctaves=2</text>
> 
>     <rect x="175" y="25" width="100" height="75" filter="url(#Turb2)"  />
>     <text x="225" y="117">type=turbulence</text>
>     <text x="225" y="129">baseFrequency=0.1</text>
>     <text x="225" y="141">numOctaves=2</text>
> 
>     <rect x="325" y="25" width="100" height="75" filter="url(#Turb3)"  />
>     <text x="375" y="117">type=turbulence</text>
>     <text x="375" y="129">baseFrequency=0.05</text>
>     <text x="375" y="141">numOctaves=8</text>
> 
>     <rect x="25" y="180" width="100" height="75" filter="url(#Turb4)"  />
>     <text x="75" y="272">type=fractalNoise</text>
>     <text x="75" y="284">baseFrequency=0.1</text>
>     <text x="75" y="296">numOctaves=4</text>
> 
>     <rect x="175" y="180" width="100" height="75" filter="url(#Turb5)"  />
>     <text x="225" y="272">type=fractalNoise</text>
>     <text x="225" y="284">baseFrequency=0.4</text>
>     <text x="225" y="296">numOctaves=4</text>
> 
>     <rect x="325" y="180" width="100" height="75" filter="url(#Turb6)"  />
>     <text x="375" y="272">type=fractalNoise</text>
>     <text x="375" y="284">baseFrequency=0.1</text>
>     <text x="375" y="296">numOctaves=1</text>
>   </g>
> </svg>
> ```
>
> ![Example of feTurbulence](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feTurbulence.png)
>
> Example of feTurbulence
>
> [View this example as SVG](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/feTurbulence.svg)

<a id="ref-for-propdef-color-interpolation-filters②⑥"></a>

## <a id="ColorInterpolationFiltersProperty"></a>10. The [color-interpolation-filters](#propdef-color-interpolation-filters) property

<a id="ref-for-propdef-color-interpolation-filters②⑦"></a>

The description of the [color-interpolation-filters](#propdef-color-interpolation-filters) property is as follows:

<strong>Table 29 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-color-interpolation-filters"></a>color-interpolation-filters

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①②"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) sRGB <a id="ref-for-comb-one①③"></a>\| linearRGB

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

linearRGB

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-filter-primitive④④"></a>

All [filter primitives](#filter-primitive)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Media:

<strong>Column 2 (data cell):</strong>

visual

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

no

<a id="valdef-color-interpolation-filters-auto"></a>auto  
<a id="ref-for-valdef-color-interpolation-filters-srgb②"></a>

<a id="ref-for-valdef-color-interpolation-filters-linearrgb②"></a>

Indicates that the user agent can choose either the [sRGB](#valdef-color-interpolation-filters-srgb) or [linearRGB](#valdef-color-interpolation-filters-linearrgb) spaces for filter effects color operations. This option indicates that the author doesn’t require that color operations occur in a particular color space.

<a id="valdef-color-interpolation-filters-srgb"></a>sRGB  
Indicates that filter effects color operations should occur in the sRGB color space.

<a id="valdef-color-interpolation-filters-linearrgb"></a>linearRGB  
Indicates that filter effects color operations should occur in the linearized RGB color space.

<a id="ref-for-propdef-color-interpolation-filters②⑧"></a>

The [color-interpolation-filters](#propdef-color-interpolation-filters) property specifies the color space for imaging operations performed via filter effects.

<a id="ref-for-propdef-color-interpolation-filters②⑨"></a>

<a id="ref-for-elementdef-feoffset①③"></a>

<a id="ref-for-elementdef-feimage⑤"></a>

<a id="ref-for-elementdef-fetile①⓪"></a>

<a id="ref-for-elementdef-feflood①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [color-interpolation-filters](#propdef-color-interpolation-filters) property just has an affect on filter operations. Therefore, it has no effect on filter primitives like [feOffset](#elementdef-feoffset), [feImage](#elementdef-feimage), [feTile](#elementdef-fetile) or [feFlood](#elementdef-feflood).

<a id="ref-for-propdef-color-interpolation-filters③⓪"></a>

<a id="ref-for-ColorInterpolationProperty②②"></a>

<a id="ref-for-valdef-color-interpolation-filters-linearrgb③"></a>

<a id="ref-for-valdef-color-interpolation-filters-srgb③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [color-interpolation-filters](#propdef-color-interpolation-filters) has a different initial value than [color-interpolation](https://www.w3.org/TR/svg2/painting.html#ColorInterpolationProperty). <a id="ref-for-propdef-color-interpolation-filters③①"></a>color-interpolation-filters has an initial value of [linearRGB](#valdef-color-interpolation-filters-linearrgb), where as <a id="ref-for-ColorInterpolationProperty②③"></a>color-interpolation has an initial value of [sRGB](#valdef-color-interpolation-filters-srgb). Thus, in the default case, filter effects operations occur in the linearRGB color space, whereas all other color interpolations occur by default in the sRGB color space.

<a id="ref-for-propdef-color-interpolation-filters③②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [color-interpolation-filters](#propdef-color-interpolation-filters) property has no affect on Filter Functions, which operate in the sRGB color space.

<a id="ref-for-propdef-color-interpolation-filters③③"></a>

The [color-interpolation-filters](#propdef-color-interpolation-filters) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

## <a id="LightSourceDefinitions"></a>11. Light source elements and properties

### <a id="LightSourceIntro"></a>11.1. Introduction

<a id="ref-for-elementdef-fedistantlight②"></a>

<a id="ref-for-elementdef-fepointlight②"></a>

<a id="ref-for-elementdef-fespotlight③"></a>

<a id="ref-for-propdef-lighting-color②①"></a>

The following sections define the elements that define a <a id="light-source"></a>light source, [feDistantLight](#elementdef-fedistantlight), [fePointLight](#elementdef-fepointlight) and [feSpotLight](#elementdef-fespotlight), and property [lighting-color](#propdef-lighting-color), which defines the color of the light.

<a id="ref-for-elementdef-fedistantlight③"></a>

### <a id="feDistantLightElement"></a>11.2. Light source [feDistantLight](#elementdef-fedistantlight)

<strong>Table 30 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fedistantlight"></a>`feDistantLight`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-light-source⑦"></a>

[light source](#light-source)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement①⑨"></a>

<a id="ref-for-elementdef-script②③"></a>

<a id="ref-for-AnimateElement①⑨"></a>

<a id="ref-for-TermDescriptiveElement②②"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fedistantlight-azimuth"></a>

  [azimuth](#element-attrdef-fedistantlight-azimuth)

- <a id="ref-for-element-attrdef-fedistantlight-elevation"></a>

  [elevation](#element-attrdef-fedistantlight-elevation)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEDistantLightElement](#InterfaceSVGFEDistantLightElement)

<em>Attribute definitions:</em>

<a id="ref-for-number-value③①"></a>

<a id="element-attrdef-fedistantlight-azimuth"></a>`azimuth` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

Direction angle for the light source on the XY plane (clockwise), in degrees from the x axis.

<a id="ref-for-TermInitialValue⑥①"></a>

<a id="ref-for-element-attrdef-fedistantlight-azimuth①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [azimuth](#element-attrdef-fedistantlight-azimuth) is 0.

Animatable: yes.

<a id="ref-for-number-value③②"></a>

<a id="element-attrdef-fedistantlight-elevation"></a>`elevation` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

Direction angle for the light source from the XY plane towards the Z-axis, in degrees. Note that the positive Z-axis points towards the viewer.

<a id="ref-for-TermInitialValue⑥②"></a>

<a id="ref-for-element-attrdef-fedistantlight-elevation①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [elevation](#element-attrdef-fedistantlight-elevation) is 0.

Animatable: yes.

<a id="ref-for-element-attrdef-fedistantlight-azimuth②"></a>

<a id="ref-for-element-attrdef-fedistantlight-elevation②"></a>

The following diagram illustrates the angles which [azimuth](#element-attrdef-fedistantlight-azimuth) and [elevation](#element-attrdef-fedistantlight-elevation) represent in an XYZ coordinate system.

![Angles which azimuth and elevation represent](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/examples/azimuth-elevation.svg)

Angles which azimuth and elevation represent

<a id="ref-for-elementdef-fepointlight③"></a>

### <a id="fePointLightElement"></a>11.3. Light source [fePointLight](#elementdef-fepointlight)

<strong>Table 31 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fepointlight"></a>`fePointLight`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-light-source⑧"></a>

[light source](#light-source)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement②⓪"></a>

<a id="ref-for-elementdef-script②④"></a>

<a id="ref-for-AnimateElement②⓪"></a>

<a id="ref-for-TermDescriptiveElement②③"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fepointlight-x"></a>

  [x](#element-attrdef-fepointlight-x)

- <a id="ref-for-element-attrdef-fepointlight-y"></a>

  [y](#element-attrdef-fepointlight-y)

- <a id="ref-for-element-attrdef-fepointlight-z"></a>

  [z](#element-attrdef-fepointlight-z)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFEPointLightElement](#InterfaceSVGFEPointLightElement)

<em>Attribute definitions:</em>

<a id="ref-for-number-value③③"></a>

<a id="element-attrdef-fepointlight-x"></a>`x` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②⓪"></a>

<a id="ref-for-elementdef-filter④①"></a>

X location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑥③"></a>

<a id="ref-for-element-attrdef-fepointlight-x①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [x](#element-attrdef-fepointlight-x) is 0.

Animatable: yes.

<a id="ref-for-number-value③④"></a>

<a id="element-attrdef-fepointlight-y"></a>`y` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②①"></a>

<a id="ref-for-elementdef-filter④②"></a>

Y location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑥④"></a>

<a id="ref-for-element-attrdef-fepointlight-y①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [y](#element-attrdef-fepointlight-y) is 0.

Animatable: yes.

<a id="ref-for-number-value③⑤"></a>

<a id="element-attrdef-fepointlight-z"></a>`z` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②②"></a>

<a id="ref-for-elementdef-filter④③"></a>

<a id="ref-for-local-coordinate-system④"></a>

Z location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element, assuming that, in the initial [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system) , the positive Z-axis comes out towards the person viewing the content and assuming that one unit along the Z-axis equals [one unit in X and Y](https://www.w3.org/TR/SVG11/coords.html#Units_viewport_percentage).

<a id="ref-for-TermInitialValue⑥⑤"></a>

<a id="ref-for-element-attrdef-fepointlight-z①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [z](#element-attrdef-fepointlight-z) is 0.

Animatable: yes.

<a id="ref-for-elementdef-fespotlight④"></a>

### <a id="feSpotLightElement"></a>11.4. Light source [feSpotLight](#elementdef-fespotlight)

<strong>Table 32 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="elementdef-fespotlight"></a>`feSpotLight`

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Categories:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-light-source⑨"></a>

[light source](#light-source)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Content model:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-SetElement②①"></a>

<a id="ref-for-elementdef-script②⑤"></a>

<a id="ref-for-AnimateElement②①"></a>

<a id="ref-for-TermDescriptiveElement②④"></a>

Any number of [descriptive elements](https://www.w3.org/TR/svg2/struct.html#TermDescriptiveElement), [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement) elements, in any order.

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Attributes:

<strong>Column 2 (data cell):</strong>

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [id](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [xml:base](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [xml:lang](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [xml:space](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-element-attrdef-fespotlight-x"></a>

  [x](#element-attrdef-fespotlight-x)

- <a id="ref-for-element-attrdef-fespotlight-y"></a>

  [y](#element-attrdef-fespotlight-y)

- <a id="ref-for-element-attrdef-fespotlight-z"></a>

  [z](#element-attrdef-fespotlight-z)

- <a id="ref-for-element-attrdef-fespotlight-pointsatx"></a>

  [pointsAtX](#element-attrdef-fespotlight-pointsatx)

- <a id="ref-for-element-attrdef-fespotlight-pointsaty"></a>

  [pointsAtY](#element-attrdef-fespotlight-pointsaty)

- <a id="ref-for-element-attrdef-fespotlight-pointsatz"></a>

  [pointsAtZ](#element-attrdef-fespotlight-pointsatz)

- <a id="ref-for-element-attrdef-fespotlight-specularexponent"></a>

  [specularExponent](#element-attrdef-fespotlight-specularexponent)

- <a id="ref-for-element-attrdef-fespotlight-limitingconeangle①"></a>

  [limitingConeAngle](#element-attrdef-fespotlight-limitingconeangle)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

DOM Interfaces:

<strong>Column 2 (data cell):</strong>

[SVGFESpotLightElement](#InterfaceSVGFESpotLightElement)

<em>Attribute definitions:</em>

<a id="ref-for-number-value③⑥"></a>

<a id="element-attrdef-fespotlight-x"></a>`x` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②③"></a>

<a id="ref-for-elementdef-filter④④"></a>

X location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑥⑥"></a>

<a id="ref-for-element-attrdef-fespotlight-x①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [x](#element-attrdef-fespotlight-x) is 0.

Animatable: yes.

<a id="ref-for-number-value③⑦"></a>

<a id="element-attrdef-fespotlight-y"></a>`y` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②④"></a>

<a id="ref-for-elementdef-filter④⑤"></a>

Y location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element.

<a id="ref-for-TermInitialValue⑥⑦"></a>

<a id="ref-for-element-attrdef-fespotlight-y①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [y](#element-attrdef-fespotlight-y) is 0.

Animatable: yes.

<a id="ref-for-number-value③⑧"></a>

<a id="element-attrdef-fespotlight-z"></a>`z` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②⑤"></a>

<a id="ref-for-elementdef-filter④⑥"></a>

<a id="ref-for-local-coordinate-system⑤"></a>

Z location for the light source in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element, assuming that, in the initial [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system), the positive Z-axis comes out towards the person viewing the content and assuming that one unit along the Z-axis equals [one unit in X and Y](https://www.w3.org/TR/SVG11/coords.html#Units_viewport_percentage).

<a id="ref-for-TermInitialValue⑥⑧"></a>

<a id="ref-for-element-attrdef-fespotlight-z①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [z](#element-attrdef-fespotlight-z) is 0.

Animatable: yes.

<a id="ref-for-number-value③⑨"></a>

<a id="element-attrdef-fespotlight-pointsatx"></a>`pointsAtX` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②⑥"></a>

<a id="ref-for-elementdef-filter④⑦"></a>

X location in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element of the point at which the light source is pointing.

<a id="ref-for-TermInitialValue⑥⑨"></a>

<a id="ref-for-element-attrdef-fespotlight-pointsatx①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [pointsAtX](#element-attrdef-fespotlight-pointsatx) is 0.

Animatable: yes.

<a id="ref-for-number-value④⓪"></a>

<a id="element-attrdef-fespotlight-pointsaty"></a>`pointsAtY` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②⑦"></a>

<a id="ref-for-elementdef-filter④⑧"></a>

Y location in the coordinate system established by attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element of the point at which the light source is pointing.

<a id="ref-for-TermInitialValue⑦⓪"></a>

<a id="ref-for-element-attrdef-fespotlight-pointsaty①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [pointsAtY](#element-attrdef-fespotlight-pointsaty) is 0.

Animatable: yes.

<a id="ref-for-number-value④①"></a>

<a id="element-attrdef-fespotlight-pointsatz"></a>`pointsAtZ` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-filter-primitiveunits②⑧"></a>

<a id="ref-for-elementdef-filter④⑨"></a>

<a id="ref-for-local-coordinate-system⑥"></a>

Z location in the coordinate system established by the attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the [filter](#elementdef-filter) element of the point at which the light source is pointing, assuming that, in the initial [local coordinate system](https://www.w3.org/TR/css-transforms-1/#local-coordinate-system), the positive Z-axis comes out towards the person viewing the content and assuming that one unit along the Z-axis equals [one unit in X and Y](https://www.w3.org/TR/SVG11/coords.html#Units_viewport_percentage).

<a id="ref-for-TermInitialValue⑦①"></a>

<a id="ref-for-element-attrdef-fespotlight-pointsatz①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [pointsAtZ](#element-attrdef-fespotlight-pointsatz) is 0.

Animatable: yes.

<a id="ref-for-number-value④②"></a>

<a id="element-attrdef-fespotlight-specularexponent"></a>`specularExponent` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

Exponent value controlling the focus for the light source.

<a id="ref-for-TermInitialValue⑦②"></a>

<a id="ref-for-element-attrdef-fespotlight-specularexponent①"></a>

The [initial value](https://svgwg.org/svg2-draft/types.html#TermInitialValue) for [specularExponent](#element-attrdef-fespotlight-specularexponent) is 1.

<a id="ref-for-element-attrdef-fespotlight-specularexponent②"></a>

See section [Filter primitive \<feDiffuseLighting\>](#feDiffuseLightingElement) for how to use [specularExponent](#element-attrdef-fespotlight-specularexponent).

<a id="ref-for-element-attrdef-fespotlight-specularexponent③"></a>

<a id="ref-for-elementdef-fespotlight⑤"></a>

<a id="ref-for-element-attrdef-fespecularlighting-specularexponent②"></a>

<a id="ref-for-elementdef-fespecularlighting⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [specularExponent](#element-attrdef-fespotlight-specularexponent) for [feSpotLight](#elementdef-fespotlight) serves a different use case than [specularExponent](#element-attrdef-fespecularlighting-specularexponent) for [feSpecularLighting](#elementdef-fespecularlighting).

Animatable: yes.

<a id="ref-for-number-value④③"></a>

<a id="element-attrdef-fespotlight-limitingconeangle"></a>`limitingConeAngle` = "<em><a href="https://www.w3.org/TR/css3-values/#number-value">&lt;number&gt;</a></em>"

<a id="ref-for-element-attrdef-fespotlight-limitingconeangle②"></a>

A limiting cone which restricts the region where the light is projected. No light is projected outside the cone. [limitingConeAngle](#element-attrdef-fespotlight-limitingconeangle) represents the angle in degrees between the spot light axis (i.e. the axis between the light source and the point to which it is pointing at) and the spot light cone. <a id="assert_userAgentLightingConeSmoothing"></a>User agents should apply a smoothing technique such as anti-aliasing at the boundary of the cone.

If no value is specified, then no limiting cone will be applied.

Animatable: yes.

<a id="ref-for-propdef-lighting-color②②"></a>

### <a id="LightingColorProperty"></a>11.5. The [lighting-color](#propdef-lighting-color) property

<strong>Table 33 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-lighting-color"></a>lighting-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valuea-def-color③"></a>

[\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

white

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-elementdef-fespecularlighting⑨"></a>

<a id="ref-for-elementdef-fediffuselighting⑧"></a>

[feDiffuseLighting](#elementdef-fediffuselighting) and [feSpecularLighting](#elementdef-fespecularlighting) elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Media:

<strong>Column 2 (data cell):</strong>

visual

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

as [by computed value](https://drafts.csswg.org/web-animations-1/#by-computed-value)

<a id="ref-for-propdef-lighting-color②③"></a>

<a id="ref-for-filter-primitive④⑤"></a>

<a id="ref-for-elementdef-fediffuselighting⑨"></a>

<a id="ref-for-elementdef-fespecularlighting①⓪"></a>

The [lighting-color](#propdef-lighting-color) property defines the color of the light source for [filter primitives](#filter-primitive) [feDiffuseLighting](#elementdef-fediffuselighting) and [feSpecularLighting](#elementdef-fespecularlighting).

<a id="ref-for-propdef-lighting-color②④"></a>

The [lighting-color](#propdef-lighting-color) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

<a id="ref-for-image-type①"></a>

## <a id="FilterCSSImageValue"></a>12. Filter CSS [\<image\>](https://www.w3.org/TR/css3-images/#image-type) values

<a id="ref-for-image-type②"></a>

<a id="ref-for-propdef-filter②⑨"></a>

<a id="ref-for-funcdef-filter"></a>

CSS [\<image\>](https://www.w3.org/TR/css3-images/#image-type) values can be filtered with the filter functions specified for the CSS [filter](#propdef-filter) property. This specification introduces the <a id="ref-for-image-type③"></a>\<image\> [\<filter()\>](#funcdef-filter) function with the following syntax:

<a id="funcdef-filter"></a>

<a id="ref-for-image-type④"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-filter-value-list②"></a>

```text
filter() = filter( [ <image> | <string> ], <filter-value-list> )
```
<a id="ref-for-funcdef-filter①"></a>

<a id="ref-for-image-type⑤"></a>

<a id="ref-for-propdef-filter③⓪"></a>

<a id="ref-for-concrete-object-size"></a>

The [\<filter()\>](#funcdef-filter) function takes two arguments. The first argument is an [\<image\>](https://www.w3.org/TR/css3-images/#image-type). The second is a filter function list as specified for the CSS [filter](#propdef-filter) property. The function takes the <a id="ref-for-image-type⑥"></a>\<image\> parameter and applies the filter rules, returning a processing image. Filter- and filter effect regions are sized according to the [concrete object size](https://drafts.csswg.org/css-images-3/#concrete-object-size) of the input <a id="ref-for-image-type⑦"></a>\<image\>.

<a id="ref-for-funcdef-filter-drop-shadow③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the dimension and origin of the original image must be preserved, some filter effects like [\<drop-shadow()\>](#funcdef-filter-drop-shadow) on a fully opaque image may not have any affect.

<a id="ref-for-funcdef-filter-blur②"></a>

<a id="ref-for-element-attrdef-fegaussianblur-edgemode②"></a>

<a id="ref-for-elementdef-fegaussianblur①②"></a>

For the [\<blur()\>](#funcdef-filter-blur) function the [edgeMode](#element-attrdef-fegaussianblur-edgemode) attribute on the [feGaussianBlur](#elementdef-fegaussianblur) element is set to duplicate. This produces more pleasant results on the edges of the filtered input image.

### <a id="interpolating-filter-image"></a>12.1. Interpolating filter()

<a id="ref-for-funcdef-filter②"></a>

<a id="ref-for-image-type⑧"></a>

If both the starting and ending image are [\<filter()\>](#funcdef-filter)s which may only differ by their used filter functions, they must be interpolated by interpolating their filter function lists as described in section [Interpolation of Filters](#interpolation-of-filters). Otherwise, they must be interpolated as generic [\<image\>](https://www.w3.org/TR/css3-images/#image-type)s. If the filter function interpolation can not be performed, the images must be interpolated as generic <a id="ref-for-image-type⑨"></a>\<image\>s.

<a id="ref-for-elementdef-filter⑤⓪"></a>

## <a id="ShorthandEquivalents"></a>13. Shorthands defined in terms of the [filter](#elementdef-filter) element

### <a id="FilterPrimitiveRepresentation"></a>13.1. Filter primitive representation

Below are the equivalents for each of the filter functions expressed in terms of the 'filter element' element. The parameters from the function are labeled with brackets in the following style: \[amount\]. In the case of parameters that are percentage values, they are converted to real numbers.

#### <a id="grayscaleEquivalent"></a>13.1.1. grayscale

```text
<filter id="grayscale">
  <feColorMatrix type="matrix"
             values="
    (0.2126 + 0.7874 * [1 - amount]) (0.7152 - 0.7152  * [1 - amount]) (0.0722 - 0.0722 * [1 - amount]) 0 0
    (0.2126 - 0.2126 * [1 - amount]) (0.7152 + 0.2848  * [1 - amount]) (0.0722 - 0.0722 * [1 - amount]) 0 0
    (0.2126 - 0.2126 * [1 - amount]) (0.7152 - 0.7152  * [1 - amount]) (0.0722 + 0.9278 * [1 - amount]) 0 0
    0 0 0 1 0"/>
</filter>
```
#### <a id="sepiaEquivalent"></a>13.1.2. sepia

```text
<filter id="sepia">
  <feColorMatrix type="matrix"
             values="
    (0.393 + 0.607 * [1 - amount]) (0.769 - 0.769 * [1 - amount]) (0.189 - 0.189 * [1 - amount]) 0 0
    (0.349 - 0.349 * [1 - amount]) (0.686 + 0.314 * [1 - amount]) (0.168 - 0.168 * [1 - amount]) 0 0
    (0.272 - 0.272 * [1 - amount]) (0.534 - 0.534 * [1 - amount]) (0.131 + 0.869 * [1 - amount]) 0 0
    0 0 0 1 0"/>
</filter>
```
#### <a id="saturateEquivalent"></a>13.1.3. saturate

```text
<filter id="saturate">
  <feColorMatrix type="saturate" values="[amount]"/>
</filter>
```
#### <a id="huerotateEquivalent"></a>13.1.4. hue-rotate

```text
<filter id="hue-rotate">
  <feColorMatrix type="hueRotate" values="[angle]"/>
</filter>
```
#### <a id="invertEquivalent"></a>13.1.5. invert

```text
<filter id="invert">
  <feComponentTransfer>
      <feFuncR type="table" tableValues="[amount] (1 - [amount])"/>
      <feFuncG type="table" tableValues="[amount] (1 - [amount])"/>
      <feFuncB type="table" tableValues="[amount] (1 - [amount])"/>
  </feComponentTransfer>
</filter>
```
#### <a id="opacityEquivalent"></a>13.1.6. opacity

```text
<filter id="opacity">
  <feComponentTransfer>
      <feFuncA type="table" tableValues="0 [amount]"/>
  </feComponentTransfer>
</filter>
```
#### <a id="brightnessEquivalent"></a>13.1.7. brightness

```text
<filter id="brightness">
  <feComponentTransfer>
      <feFuncR type="linear" slope="[amount]"/>
      <feFuncG type="linear" slope="[amount]"/>
      <feFuncB type="linear" slope="[amount]"/>
  </feComponentTransfer>
</filter>
```
#### <a id="contrastEquivalent"></a>13.1.8. contrast

```text
<filter id="contrast">
  <feComponentTransfer>
      <feFuncR type="linear" slope="[amount]" intercept="-(0.5 * [amount]) + 0.5"/>
      <feFuncG type="linear" slope="[amount]" intercept="-(0.5 * [amount]) + 0.5"/>
      <feFuncB type="linear" slope="[amount]" intercept="-(0.5 * [amount]) + 0.5"/>
  </feComponentTransfer>
</filter>
```
#### <a id="blurEquivalent"></a>13.1.9. blur

```text
<filter id="blur">
  <feGaussianBlur stdDeviation="[radius radius]" edgeMode="[edge mode]" >
</filter>
```
<a id="ref-for-propdef-filter③①"></a>

<a id="ref-for-funcdef-filter③"></a>

Where edge mode computes to none for the [filter](#propdef-filter) property and to duplicate for the CSS Image [\<filter()\>](#funcdef-filter) function.

<a id="ref-for-funcdef-filter-blur③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<blur()\>](#funcdef-filter-blur) function may increase the UA defined filter region. See [Filter region for shorthands](#filter-region-for-shorthands).

#### <a id="dropshadowEquivalent"></a>13.1.10. drop-shadow

```text
<filter id="drop-shadow">
  <feGaussianBlur in="[alpha-channel-of-input]" stdDeviation="[radius]"/>
  <feOffset dx="[offset-x]" dy="[offset-y]" result="offsetblur"/>
  <feFlood flood-color="[color]"/>
  <feComposite in2="offsetblur" operator="in"/>
  <feMerge>
    <feMergeNode/>
    <feMergeNode in="[input-image]"/>
  </feMerge>
</filter>
```
<a id="ref-for-funcdef-filter-drop-shadow④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<drop-shadow()\>](#funcdef-filter-drop-shadow) function may increase the UA defined filter region. See [Filter region for shorthands](#filter-region-for-shorthands).

### <a id="filter-region-for-shorthands"></a>13.2. Filter region for shorthands

<a id="ref-for-filter-region①⑥"></a>

<a id="ref-for-propdef-border①"></a>

<a id="ref-for-propdef-border-image"></a>

<a id="ref-for-propdef-box-shadow③"></a>

<a id="ref-for-propdef-text-shadow"></a>

<a id="ref-for-propdef-outline"></a>

<a id="ref-for-funcdef-filter-blur④"></a>

<a id="ref-for-funcdef-filter-drop-shadow⑤"></a>

All shorthand filters implemented with filter primitives in the previous subsection must have a UA defined [filter region](#filter-region). The filter region must cover the visual content of an element including overflowing content, graphical control elements such as scrollbars, [border](https://www.w3.org/TR/css3-background/#propdef-border)/[border-image](https://www.w3.org/TR/css3-background/#propdef-border-image), [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow), [text-shadow](https://www.w3.org/TR/css-text-decor-3/#propdef-text-shadow) and [outline](https://www.w3.org/TR/css3-ui/#propdef-outline). Furthermore, if a shorthand filter expands this visible area like it is the case for [\<blur()\>](#funcdef-filter-blur) or [\<drop-shadow()\>](#funcdef-filter-drop-shadow) the filter region must cover this area as well.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the handling of [filter sources](#FilterElement) see section [Filter region](#FilterEffectsRegion).

## <a id="animation-of-filters"></a>14. Animation of Filters

### <a id="interpolation-of-filters"></a>14.1. Interpolation of Filter Function Lists

For interpolation between one filter and a second, the steps corresponding to the first matching condition in the following list must be run:

<a id="ref-for-typedef-filter-function⑥"></a>

<a id="ref-for-typedef-filter-url④"></a>

<a id="ref-for-typedef-filter-value-list③"></a>

<a id="from-to-animation-equal"></a>If both filters have a [\<filter-value-list\>](#typedef-filter-value-list) of same length without [\<url\>](#typedef-filter-url) and for each [\<filter-function\>](#typedef-filter-function) for which there is a corresponding item in each list

<a id="ref-for-typedef-filter-function⑦"></a>

Interpolate each [\<filter-function\>](#typedef-filter-function) pair following the rules in section [Interpolation of Filter Functions](#interpolation-of-filter-functions).

<a id="ref-for-typedef-filter-function⑧"></a>

<a id="ref-for-typedef-filter-url⑤"></a>

<a id="ref-for-typedef-filter-value-list④"></a>

<a id="from-to-animation-unequal"></a>If both filters have a [\<filter-value-list\>](#typedef-filter-value-list) of different length without [\<url\>](#typedef-filter-url) and for each [\<filter-function\>](#typedef-filter-function) for which there is a corresponding item in each list

1.  <a id="ref-for-typedef-filter-function⑨"></a>

    Append the missing equivalent [\<filter-function\>](#typedef-filter-function)s from the longer list to the end of the shorter list. The new added <a id="ref-for-typedef-filter-function①⓪"></a>\<filter-function\>s must be initialized to their initial values for interpolation.

2.  <a id="ref-for-typedef-filter-function①①"></a>

    Interpolate each [\<filter-function\>](#typedef-filter-function) pair following the rules in section [Interpolation of Filter Functions](#interpolation-of-filter-functions).

<a id="ref-for-typedef-filter-url⑥"></a>

<a id="ref-for-typedef-filter-value-list⑤"></a>

<a id="interpolation-none-filter-functions"></a>If one filter is none and the other is a [\<filter-value-list\>](#typedef-filter-value-list) without [\<url\>](#typedef-filter-url)

1.  <a id="ref-for-typedef-filter-function①②"></a>

    <a id="ref-for-typedef-filter-value-list⑥"></a>

    Replace none with the corresponding [\<filter-value-list\>](#typedef-filter-value-list) of the other filter. The new [\<filter-function\>](#typedef-filter-function)s must be initialized to their initial values for interpolation.

2.  <a id="ref-for-typedef-filter-function①③"></a>

    Interpolate each [\<filter-function\>](#typedef-filter-function) pair following the rules in section [Interpolation of Filter Functions](#interpolation-of-filter-functions).

<a id="no-interpolation"></a>Otherwise

Use discrete interpolation.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f7e128e9"></a> Compute distance of filter functions. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;91&#x3E;](https://github.com/w3c/csswg-drafts/issues/91)

### <a id="addition"></a>14.2. Addition

<a id="ref-for-typedef-filter-value-list⑦"></a>

It is possible to combine independent animations of [\<filter-value-list\>](#typedef-filter-value-list)s [\[SVG11\]](#biblio-svg11).

Given two filter values representing an base value (<var>base filter list</var>) and a value to add (<var>added filter list</var>), returns the concatenation of the the two lists: ‘<var>base filter list</var> <var>added filter list</var>’.

<a id="ref-for-AnimateElement②②"></a>

<a id="ref-for-propdef-filter③②"></a>

<a id="ref-for-elementdef-rect"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-39a9a3b0"></a> The following SVG animation has two [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement) elements animating [filter](#propdef-filter) property of the [rect](https://www.w3.org/TR/svg2/shapes.html#elementdef-rect) element. Both specified animations are additive and have a duration of <var>10s</var>.
>
> ```text
> <rect width="200px" filter="none" ...>
>   <animate attributeName="filter" from="blur(0px)" to="blur(10px)" dur="10s"
>            additive="sum"/>
>   <animate attributeName="filter" from="sepia(0)" to="sepia(1)" dur="10s"
>            additive="sum"/>
> </rect>
> ```
>
> <a id="ref-for-used-value"></a>
>
> <a id="ref-for-propdef-filter③③"></a>
>
> After <var>5s</var>, the [used value](https://www.w3.org/TR/css-cascade-4/#used-value) of [filter](#propdef-filter) is blur(5px) sepia(0.5).

### <a id="accumulation"></a>14.3. Accumulation

Given two filter values <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var>, returns the filter value, <var>V<sub>b</sub></var>.

## <a id="priv-sec"></a>15. Privacy and Security Considerations

### <a id="tainted-filter-primitives"></a>15.1. Tainted Filter Primitives

It is important that the timing of any filter operation is independent of pixel values derived from the filtered content or other sources potentially containing privacy-sensitive information.

<a id="ref-for-filter-primitive④⑥"></a>

The following [filter primitives](#filter-primitive) may have access to pixel values that potentially contain privacy-sensitive information, either from the filtered object itself or other sources such as CSS styling. These primitives must be flagged as "tainted".

1.  <a id="ref-for-elementdef-feflood①①"></a>

    <a id="ref-for-specified-value"></a>

    <a id="ref-for-propdef-flood-color②⑤"></a>

    [feFlood](#elementdef-feflood) when the [specified value](https://www.w3.org/TR/css-cascade-4/#specified-value) of the [flood-color](#propdef-flood-color) property computes to currentColor,

2.  <a id="ref-for-elementdef-fedropshadow①②"></a>

    <a id="ref-for-specified-value①"></a>

    <a id="ref-for-propdef-flood-color②⑥"></a>

    [feDropShadow](#elementdef-fedropshadow) when the [specified value](https://www.w3.org/TR/css-cascade-4/#specified-value) value of the [flood-color](#propdef-flood-color) property computes to currentColor,

3.  <a id="ref-for-elementdef-fediffuselighting①⓪"></a>

    <a id="ref-for-specified-value②"></a>

    <a id="ref-for-propdef-lighting-color②⑤"></a>

    [feDiffuseLighting](#elementdef-fediffuselighting), when the [specified value](https://www.w3.org/TR/css-cascade-4/#specified-value) value of the [lighting-color](#propdef-lighting-color) property computes to currentColor

4.  <a id="ref-for-elementdef-fespecularlighting①①"></a>

    <a id="ref-for-specified-value③"></a>

    <a id="ref-for-propdef-lighting-color②⑥"></a>

    [feSpecularLighting](#elementdef-fespecularlighting) when the [specified value](https://www.w3.org/TR/css-cascade-4/#specified-value) value of the [lighting-color](#propdef-lighting-color) property computes to currentColor,

5.  <a id="ref-for-elementdef-feimage⑥"></a>

    <a id="ref-for-typedef-filter-url⑦"></a>

    [feImage](#elementdef-feimage), when the [\<url\>](#typedef-filter-url) reference points to an element or fetches a resource with the fetching mode <em>No-CORS</em> and

6.  <a id="ref-for-attr-valuedef-in-sourcegraphic①①"></a>

    <a id="ref-for-attr-valuedef-in-sourcealpha⑦"></a>

    <a id="ref-for-attr-valuedef-in-backgroundimage④"></a>

    <a id="ref-for-attr-valuedef-in-backgroundalpha②"></a>

    <a id="ref-for-attr-valuedef-in-fillpaint⑦"></a>

    <a id="ref-for-attr-valuedef-in-strokepaint③"></a>

    the filter primitives: [SourceGraphic](#attr-valuedef-in-sourcegraphic), [SourceAlpha](#attr-valuedef-in-sourcealpha), [BackgroundImage](#attr-valuedef-in-backgroundimage), [BackgroundAlpha](#attr-valuedef-in-backgroundalpha), [FillPaint](#attr-valuedef-in-fillpaint) and [StrokePaint](#attr-valuedef-in-strokepaint).

<a id="ref-for-elementdef-feflood①②"></a>

<a id="ref-for-elementdef-fedropshadow①③"></a>

<a id="ref-for-elementdef-fediffuselighting①①"></a>

<a id="ref-for-elementdef-fespecularlighting①②"></a>

<a id="ref-for-valuea-def-color④"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-color0①⑧"></a>

<a id="ref-for-visited-pseudo"></a>

[feFlood](#elementdef-feflood), [feDropShadow](#elementdef-fedropshadow), [feDiffuseLighting](#elementdef-fediffuselighting) and [feSpecularLighting](#elementdef-fespecularlighting) are primitives with one or more CSS properties that take [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) as property value. <a id="ref-for-valuea-def-color⑤"></a>\<color\> consists of (amongst others) the currentColor keyword. The [used value](https://www.w3.org/TR/css-cascade-4/#used-value) for currentColor derives from the [color](https://www.w3.org/TR/css3-color/#color0) property. Since <a id="ref-for-color0①⑨"></a>color can be set by the [:visited](https://www.w3.org/TR/selectors4/#visited-pseudo) pseudo selector, it potentially contains privacy-sensitive information and therefore these primitives must be marked as tainted.

<a id="ref-for-elementdef-feimage⑦"></a>

<a id="ref-for-graphics-element①"></a>

[feImage](#elementdef-feimage) can reference cross-domain images as well as document fragments such as SVG [graphics elements](https://www.w3.org/TR/svg2/struct.html#graphics-element). These references potentially contain privacy-sensitive information and therefore the primitive must be marked as tainted.

<a id="ref-for-attr-valuedef-in-sourcegraphic①②"></a>

<a id="ref-for-attr-valuedef-in-sourcealpha⑧"></a>

<a id="ref-for-attr-valuedef-in-backgroundimage⑤"></a>

<a id="ref-for-attr-valuedef-in-backgroundalpha③"></a>

<a id="ref-for-attr-valuedef-in-fillpaint⑧"></a>

<a id="ref-for-attr-valuedef-in-strokepaint④"></a>

<a id="ref-for-graphics-element②"></a>

<a id="ref-for-color0②⓪"></a>

The filter primitives [SourceGraphic](#attr-valuedef-in-sourcegraphic), [SourceAlpha](#attr-valuedef-in-sourcealpha), [BackgroundImage](#attr-valuedef-in-backgroundimage), [BackgroundAlpha](#attr-valuedef-in-backgroundalpha), [FillPaint](#attr-valuedef-in-fillpaint) and [StrokePaint](#attr-valuedef-in-strokepaint) either reference document fragments such as SVG [graphics elements](https://www.w3.org/TR/svg2/struct.html#graphics-element) or style information that may derive directly or indirectly from the [color](https://www.w3.org/TR/css3-color/#color0) property. Therefore these primitives must be marked as tainted.

<a id="ref-for-filter-primitive④⑦"></a>

Every [filter primitive](#filter-primitive) that has a "tainted" flagged <a id="ref-for-filter-primitive④⑧"></a>filter primitive as input must be flagged as "tainted" as well.

Filter operations must be implemented in such a way that they always take the same amount of time regardless of the pixel values if one of the input filter primitives is flagged as "tainted".

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification aggravates the restrictions to filter primitives based on implementation feedback from user agents.

<a id="ref-for-elementdef-fedisplacementmap④"></a>

### <a id="fedisplacemnentmap-restrictions"></a>15.2. [feDisplacementMap](#elementdef-fedisplacementmap) Restrictions

<a id="ref-for-elementdef-fedisplacementmap⑤"></a>

<a id="ref-for-element-attrdef-fedisplacementmap-in2⑧"></a>

<a id="ref-for-pass-through-filter②"></a>

If [feDisplacementMap](#elementdef-fedisplacementmap) has a "tainted" flagged filter primitive as input and this input filter primitive is used as displacement map (referenced by [in2](#element-attrdef-fedisplacementmap-in2)), then <a id="ref-for-elementdef-fedisplacementmap⑥"></a>feDisplacementMap must not proceed with the filter operation and acts as a [pass through filter](#pass-through-filter).

### <a id="origin-restrictions"></a>15.3. Origin Restrictions

<a id="ref-for-propdef-filter③④"></a>

User agents must use the [potentially CORS-enabled fetch](https://fetch.spec.whatwg.org/#cors-request) method defined by the [\[HTML5\]](#biblio-html5) specification for the [filter](#propdef-filter) property. When fetching, user agents must use "Anonymous" mode, set the referrer source to the stylesheet’s URL and set the origin to the URL of the containing document. If this results in network errors, the effect is as if the value none had been specified.

### <a id="timing-attack"></a>15.4. Timing Attacks

If any of the above rules are not followed, an attacker could infer information and mount a timing attack.

A timing attack is a method of obtaining information about content that is otherwise protected, based on studying the amount of time it takes for an operation to occur. If, for example, red pixels took longer to draw than green pixels, one might be able to reconstruct a rough image of the element being rendered, without ever having access to the content of the element. Security studies show that timing differences on arithmetic operations can be caused by the hardware architecture or compiler [\[ArTD\]](#biblio-artd).

<a id="ref-for-EnableBackgroundProperty①⑧"></a>

## <a id="AccessBackgroundImage"></a>Appendix A: The deprecated [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty) property

<a id="ref-for-EnableBackgroundProperty①⑨"></a>

<a id="ref-for-filter-region①⑦"></a>

<a id="ref-for-elementdef-filter⑤①"></a>

SVG 1.1 introduced the [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty) property [\[SVG11\]](#biblio-svg11). The property defined the back drop under the [filter region](#filter-region) at the time that the [filter](#elementdef-filter) element was invoked. The concept defined by this property was identified to be incompatible with the model of stacking context in CSS at the time writing this specification. UAs can choose to implement the <a id="ref-for-EnableBackgroundProperty②⓪"></a>enable-background property as defined in SVG 1.1 but will not be compatible to this specification or to CSS Compositing and Blending [\[COMPOSITING-1\]](#biblio-compositing-1).

<a id="ref-for-EnableBackgroundProperty②①"></a>

<a id="ref-for-propdef-isolation②⓪"></a>

This specification does not support the [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty) property. UAs must support the [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation) property instead [\[COMPOSITING-1\]](#biblio-compositing-1).

## <a id="DOMInterfaces"></a>Appendix B: DOM interfaces

### <a id="InterfaceSVGFilterElement"></a>Interface SVGFilterElement

<a id="ref-for-elementdef-filter⑤②"></a>

The <a id="svgfilterelement"></a>`SVGFilterElement` interface corresponds to the [filter](#elementdef-filter) element.

<a id="ref-for-svgfilterelement"></a>

<a id="ref-for-InterfaceSVGElement"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration"></a>

<a id="ref-for-dom-svgfilterelement-filterunits"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①"></a>

<a id="ref-for-dom-svgfilterelement-primitiveunits"></a>

<a id="ref-for-InterfaceSVGAnimatedLength"></a>

<a id="ref-for-dom-svgfilterelement-x"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①"></a>

<a id="ref-for-dom-svgfilterelement-y"></a>

<a id="ref-for-InterfaceSVGAnimatedLength②"></a>

<a id="ref-for-dom-svgfilterelement-width"></a>

<a id="ref-for-InterfaceSVGAnimatedLength③"></a>

<a id="ref-for-dom-svgfilterelement-height"></a>

<a id="ref-for-svgfilterelement①"></a>

<a id="ref-for-InterfaceSVGURIReference"></a>

```text
interface SVGFilterElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration filterUnits;
  readonly attribute SVGAnimatedEnumeration primitiveUnits;
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
};

SVGFilterElement includes SVGURIReference;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedEnumeration②"></a>

<a id="dom-svgfilterelement-filterunits"></a>`filterUnits`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-filter-filterunits⑦"></a>

<a id="ref-for-elementdef-filter⑤③"></a>

<a id="ref-for-InterfaceSVGUnitTypes"></a>

Corresponds to attribute [filterUnits](#element-attrdef-filter-filterunits) on the given [filter](#elementdef-filter) element. Takes one of the constants defined in [SVGUnitTypes](https://www.w3.org/TR/svg2/types.html#InterfaceSVGUnitTypes).

<a id="ref-for-InterfaceSVGAnimatedEnumeration③"></a>

<a id="dom-svgfilterelement-primitiveunits"></a>`primitiveUnits`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-filter-primitiveunits②⑨"></a>

<a id="ref-for-elementdef-filter⑤④"></a>

<a id="ref-for-InterfaceSVGUnitTypes①"></a>

Corresponds to attribute [primitiveUnits](#element-attrdef-filter-primitiveunits) on the given [filter](#elementdef-filter) element. Takes one of the constants defined in [SVGUnitTypes](https://www.w3.org/TR/svg2/types.html#InterfaceSVGUnitTypes).

<a id="ref-for-InterfaceSVGAnimatedLength④"></a>

<a id="dom-svgfilterelement-x"></a>`x`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-x⑦"></a>

<a id="ref-for-elementdef-filter⑤⑤"></a>

Corresponds to attribute [x](#element-attrdef-filter-x) on the given [filter](#elementdef-filter) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑤"></a>

<a id="dom-svgfilterelement-y"></a>`y`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-y⑦"></a>

<a id="ref-for-elementdef-filter⑤⑥"></a>

Corresponds to attribute [y](#element-attrdef-filter-y) on the given [filter](#elementdef-filter) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑥"></a>

<a id="dom-svgfilterelement-width"></a>`width`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-width⑦"></a>

<a id="ref-for-elementdef-filter⑤⑦"></a>

Corresponds to attribute [width](#element-attrdef-filter-width) on the given [filter](#elementdef-filter) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑦"></a>

<a id="dom-svgfilterelement-height"></a>`height`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-height⑦"></a>

<a id="ref-for-elementdef-filter⑤⑧"></a>

Corresponds to attribute [height](#element-attrdef-filter-height) on the given [filter](#elementdef-filter) element.

### <a id="InterfaceSVGFilterPrimitiveStandardAttributes"></a>Interface SVGFilterPrimitiveStandardAttributes

<a id="svgfilterprimitivestandardattributes"></a>

<a id="ref-for-InterfaceSVGAnimatedLength⑧"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-x"></a>

<a id="ref-for-InterfaceSVGAnimatedLength⑨"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-y"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①⓪"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-width"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①①"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-height"></a>

<a id="ref-for-InterfaceSVGAnimatedString"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-result"></a>

```text
interface mixin SVGFilterPrimitiveStandardAttributes {
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
  readonly attribute SVGAnimatedString result;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedLength①②"></a>

<a id="dom-svgfilterprimitivestandardattributes-x"></a>`x`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-primitive-x②②"></a>

Corresponds to attribute [x](#element-attrdef-filter-primitive-x) on the given element.

<a id="ref-for-InterfaceSVGAnimatedLength①③"></a>

<a id="dom-svgfilterprimitivestandardattributes-y"></a>`y`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-primitive-y②②"></a>

Corresponds to attribute [y](#element-attrdef-filter-primitive-y) on the given element.

<a id="ref-for-InterfaceSVGAnimatedLength①④"></a>

<a id="dom-svgfilterprimitivestandardattributes-width"></a>`width`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-primitive-width②①"></a>

Corresponds to attribute [width](#element-attrdef-filter-primitive-width) on the given element.

<a id="ref-for-InterfaceSVGAnimatedLength①⑤"></a>

<a id="dom-svgfilterprimitivestandardattributes-height"></a>`height`, of type [SVGAnimatedLength](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-filter-primitive-height②①"></a>

Corresponds to attribute [height](#element-attrdef-filter-primitive-height) on the given element.

<a id="ref-for-InterfaceSVGAnimatedString①"></a>

<a id="dom-svgfilterprimitivestandardattributes-result"></a>`result`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-result②①"></a>

Corresponds to attribute [result](#element-attrdef-filter-primitive-result) on the given element.

### <a id="InterfaceSVGFEBlendElement"></a>Interface SVGFEBlendElement

<a id="ref-for-elementdef-feblend⑤"></a>

The <a id="svgfeblendelement"></a>`SVGFEBlendElement` interface corresponds to the [feBlend](#elementdef-feblend) element.

<a id="ref-for-svgfeblendelement"></a>

<a id="ref-for-InterfaceSVGElement①"></a>

<a id="ref-for-idl-unsigned-short"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_unknown"></a>

<a id="ref-for-idl-unsigned-short①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_normal"></a>

<a id="ref-for-idl-unsigned-short②"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_multiply"></a>

<a id="ref-for-idl-unsigned-short③"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_screen"></a>

<a id="ref-for-idl-unsigned-short④"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_darken"></a>

<a id="ref-for-idl-unsigned-short⑤"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_lighten"></a>

<a id="ref-for-idl-unsigned-short⑥"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_overlay"></a>

<a id="ref-for-idl-unsigned-short⑦"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color_dodge"></a>

<a id="ref-for-idl-unsigned-short⑧"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color_burn"></a>

<a id="ref-for-idl-unsigned-short⑨"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_hard_light"></a>

<a id="ref-for-idl-unsigned-short①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_soft_light"></a>

<a id="ref-for-idl-unsigned-short①①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_difference"></a>

<a id="ref-for-idl-unsigned-short①②"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_exclusion"></a>

<a id="ref-for-idl-unsigned-short①③"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_hue"></a>

<a id="ref-for-idl-unsigned-short①④"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_saturation"></a>

<a id="ref-for-idl-unsigned-short①⑤"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color"></a>

<a id="ref-for-idl-unsigned-short①⑥"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_luminosity"></a>

<a id="ref-for-InterfaceSVGAnimatedString②"></a>

<a id="ref-for-dom-svgfeblendelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedString③"></a>

<a id="ref-for-dom-svgfeblendelement-in2"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration④"></a>

<a id="ref-for-dom-svgfeblendelement-mode"></a>

<a id="ref-for-svgfeblendelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes"></a>

```text
interface SVGFEBlendElement : SVGElement {

  // Blend Mode Types
  const unsigned short SVG_FEBLEND_MODE_UNKNOWN = 0;
  const unsigned short SVG_FEBLEND_MODE_NORMAL = 1;
  const unsigned short SVG_FEBLEND_MODE_MULTIPLY = 2;
  const unsigned short SVG_FEBLEND_MODE_SCREEN = 3;
  const unsigned short SVG_FEBLEND_MODE_DARKEN = 4;
  const unsigned short SVG_FEBLEND_MODE_LIGHTEN = 5;
  const unsigned short SVG_FEBLEND_MODE_OVERLAY = 6;
  const unsigned short SVG_FEBLEND_MODE_COLOR_DODGE = 7;
  const unsigned short SVG_FEBLEND_MODE_COLOR_BURN = 8;
  const unsigned short SVG_FEBLEND_MODE_HARD_LIGHT = 9;
  const unsigned short SVG_FEBLEND_MODE_SOFT_LIGHT = 10;
  const unsigned short SVG_FEBLEND_MODE_DIFFERENCE = 11;
  const unsigned short SVG_FEBLEND_MODE_EXCLUSION = 12;
  const unsigned short SVG_FEBLEND_MODE_HUE = 13;
  const unsigned short SVG_FEBLEND_MODE_SATURATION = 14;
  const unsigned short SVG_FEBLEND_MODE_COLOR = 15;
  const unsigned short SVG_FEBLEND_MODE_LUMINOSITY = 16;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedEnumeration mode;
};

SVGFEBlendElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Blend Mode Types”:  
<a id="dom-svgfeblendelement-svg_feblend_mode_unknown"></a>`SVG_FEBLEND_MODE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfeblendelement-svg_feblend_mode_normal"></a>`SVG_FEBLEND_MODE_NORMAL`  
<a id="ref-for-valdef-blend-mode-normal②"></a>

Corresponds to value [normal](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-normal).

<a id="dom-svgfeblendelement-svg_feblend_mode_multiply"></a>`SVG_FEBLEND_MODE_MULTIPLY`  
<a id="ref-for-valdef-blend-mode-multiply"></a>

Corresponds to value [multiply](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-multiply).

<a id="dom-svgfeblendelement-svg_feblend_mode_screen"></a>`SVG_FEBLEND_MODE_SCREEN`  
<a id="ref-for-valdef-blend-mode-screen"></a>

Corresponds to value [screen](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-screen).

<a id="dom-svgfeblendelement-svg_feblend_mode_darken"></a>`SVG_FEBLEND_MODE_DARKEN`  
<a id="ref-for-valdef-blend-mode-darken"></a>

Corresponds to value [darken](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-darken).

<a id="dom-svgfeblendelement-svg_feblend_mode_lighten"></a>`SVG_FEBLEND_MODE_LIGHTEN`  
<a id="ref-for-valdef-blend-mode-lighten"></a>

Corresponds to value [lighten](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-lighten).

<a id="dom-svgfeblendelement-svg_feblend_mode_overlay"></a>`SVG_FEBLEND_MODE_OVERLAY`  
<a id="ref-for-valdef-blend-mode-overlay"></a>

Corresponds to value [overlay](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-overlay).

<a id="dom-svgfeblendelement-svg_feblend_mode_color_dodge"></a>`SVG_FEBLEND_MODE_COLOR_DODGE`  
<a id="ref-for-valdef-blend-mode-color-dodge"></a>

Corresponds to value [color-dodge](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-color-dodge).

<a id="dom-svgfeblendelement-svg_feblend_mode_color_burn"></a>`SVG_FEBLEND_MODE_COLOR_BURN`  
<a id="ref-for-valdef-blend-mode-color-burn"></a>

Corresponds to value [color-burn](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-color-burn).

<a id="dom-svgfeblendelement-svg_feblend_mode_hard_light"></a>`SVG_FEBLEND_MODE_HARD_LIGHT`  
<a id="ref-for-valdef-blend-mode-hard-light"></a>

Corresponds to value [hard-light](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-hard-light).

<a id="dom-svgfeblendelement-svg_feblend_mode_soft_light"></a>`SVG_FEBLEND_MODE_SOFT_LIGHT`  
<a id="ref-for-valdef-blend-mode-soft-light"></a>

Corresponds to value [soft-light](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-soft-light).

<a id="dom-svgfeblendelement-svg_feblend_mode_difference"></a>`SVG_FEBLEND_MODE_DIFFERENCE`  
<a id="ref-for-valdef-blend-mode-difference"></a>

Corresponds to value [difference](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-difference).

<a id="dom-svgfeblendelement-svg_feblend_mode_exclusion"></a>`SVG_FEBLEND_MODE_EXCLUSION`  
<a id="ref-for-valdef-blend-mode-exclusion"></a>

Corresponds to value [exclusion](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-exclusion).

<a id="dom-svgfeblendelement-svg_feblend_mode_hue"></a>`SVG_FEBLEND_MODE_HUE`  
<a id="ref-for-valdef-blend-mode-hue"></a>

Corresponds to value [hue](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-hue).

<a id="dom-svgfeblendelement-svg_feblend_mode_saturation"></a>`SVG_FEBLEND_MODE_SATURATION`  
<a id="ref-for-valdef-blend-mode-saturation"></a>

Corresponds to value [saturation](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-saturation).

<a id="dom-svgfeblendelement-svg_feblend_mode_color"></a>`SVG_FEBLEND_MODE_COLOR`  
<a id="ref-for-valdef-blend-mode-color"></a>

Corresponds to value [color](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-color).

<a id="dom-svgfeblendelement-svg_feblend_mode_luminosity"></a>`SVG_FEBLEND_MODE_LUMINOSITY`  
<a id="ref-for-valdef-blend-mode-luminosity"></a>

Corresponds to value [luminosity](https://www.w3.org/TR/compositing-1/#valdef-blend-mode-luminosity).

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString④"></a>

<a id="dom-svgfeblendelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③④"></a>

<a id="ref-for-elementdef-feblend⑥"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feBlend](#elementdef-feblend) element.

<a id="ref-for-InterfaceSVGAnimatedString⑤"></a>

<a id="dom-svgfeblendelement-in2"></a>`in2`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-feblend-in2②"></a>

<a id="ref-for-elementdef-feblend⑦"></a>

Corresponds to attribute [in2](#element-attrdef-feblend-in2) on the given [feBlend](#elementdef-feblend) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑤"></a>

<a id="dom-svgfeblendelement-mode"></a>`mode`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-feblend-mode③"></a>

<a id="ref-for-elementdef-feblend⑧"></a>

Corresponds to attribute [mode](#element-attrdef-feblend-mode) on the given [feBlend](#elementdef-feblend) element. Takes one of the SVG_FEBLEND_MODE\_\* constants defined on this interface.

### <a id="InterfaceSVGFEColorMatrixElement"></a>Interface SVGFEColorMatrixElement

<a id="ref-for-elementdef-fecolormatrix⑦"></a>

The <a id="svgfecolormatrixelement"></a>`SVGFEColorMatrixElement` interface corresponds to the [feColorMatrix](#elementdef-fecolormatrix) element.

<a id="ref-for-svgfecolormatrixelement"></a>

<a id="ref-for-InterfaceSVGElement②"></a>

<a id="ref-for-idl-unsigned-short①⑦"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_unknown"></a>

<a id="ref-for-idl-unsigned-short①⑧"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_matrix"></a>

<a id="ref-for-idl-unsigned-short①⑨"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_saturate"></a>

<a id="ref-for-idl-unsigned-short②⓪"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_huerotate"></a>

<a id="ref-for-idl-unsigned-short②①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_luminancetoalpha"></a>

<a id="ref-for-InterfaceSVGAnimatedString⑥"></a>

<a id="ref-for-dom-svgfecolormatrixelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑥"></a>

<a id="ref-for-dom-svgfecolormatrixelement-type"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList"></a>

<a id="ref-for-dom-svgfecolormatrixelement-values"></a>

<a id="ref-for-svgfecolormatrixelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①"></a>

```text
interface SVGFEColorMatrixElement : SVGElement {

  // Color Matrix Types
  const unsigned short SVG_FECOLORMATRIX_TYPE_UNKNOWN = 0;
  const unsigned short SVG_FECOLORMATRIX_TYPE_MATRIX = 1;
  const unsigned short SVG_FECOLORMATRIX_TYPE_SATURATE = 2;
  const unsigned short SVG_FECOLORMATRIX_TYPE_HUEROTATE = 3;
  const unsigned short SVG_FECOLORMATRIX_TYPE_LUMINANCETOALPHA = 4;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedEnumeration type;
  readonly attribute SVGAnimatedNumberList values;
};

SVGFEColorMatrixElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Color Matrix Types”:  
<a id="dom-svgfecolormatrixelement-svg_fecolormatrix_type_unknown"></a>`SVG_FECOLORMATRIX_TYPE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfecolormatrixelement-svg_fecolormatrix_type_matrix"></a>`SVG_FECOLORMATRIX_TYPE_MATRIX`  
Corresponds to value matrix.

<a id="dom-svgfecolormatrixelement-svg_fecolormatrix_type_saturate"></a>`SVG_FECOLORMATRIX_TYPE_SATURATE`  
Corresponds to value saturate.

<a id="dom-svgfecolormatrixelement-svg_fecolormatrix_type_huerotate"></a>`SVG_FECOLORMATRIX_TYPE_HUEROTATE`  
Corresponds to value hueRotate.

<a id="dom-svgfecolormatrixelement-svg_fecolormatrix_type_luminancetoalpha"></a>`SVG_FECOLORMATRIX_TYPE_LUMINANCETOALPHA`  
Corresponds to value luminanceToAlpha.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString⑦"></a>

<a id="dom-svgfecolormatrixelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③⑤"></a>

<a id="ref-for-elementdef-fecolormatrix⑧"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feColorMatrix](#elementdef-fecolormatrix) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑦"></a>

<a id="dom-svgfecolormatrixelement-type"></a>`type`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-fecolormatrix-type④"></a>

<a id="ref-for-elementdef-fecolormatrix⑨"></a>

Corresponds to attribute [type](#element-attrdef-fecolormatrix-type) on the given [feColorMatrix](#elementdef-fecolormatrix) element. Takes one of the SVG_FECOLORMATRIX_TYPE\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedNumberList①"></a>

<a id="dom-svgfecolormatrixelement-values"></a>`values`, of type [SVGAnimatedNumberList](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumberList), readonly

<a id="ref-for-element-attrdef-fecolormatrix-values⑧"></a>

<a id="ref-for-elementdef-fecolormatrix①⓪"></a>

Corresponds to attribute [values](#element-attrdef-fecolormatrix-values) on the given [feColorMatrix](#elementdef-fecolormatrix) element.

### <a id="InterfaceSVGFEComponentTransferElement"></a>Interface SVGFEComponentTransferElement

<a id="ref-for-elementdef-fecomponenttransfer⑦"></a>

The <a id="svgfecomponenttransferelement"></a>`SVGFEComponentTransferElement` interface corresponds to the [feComponentTransfer](#elementdef-fecomponenttransfer) element.

<a id="ref-for-svgfecomponenttransferelement"></a>

<a id="ref-for-InterfaceSVGElement③"></a>

<a id="ref-for-InterfaceSVGAnimatedString⑧"></a>

<a id="ref-for-dom-svgfecomponenttransferelement-in1"></a>

<a id="ref-for-svgfecomponenttransferelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes②"></a>

```text
interface SVGFEComponentTransferElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};

SVGFEComponentTransferElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString⑨"></a>

<a id="dom-svgfecomponenttransferelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer⑧"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

### <a id="InterfaceSVGComponentTransferFunctionElement"></a>Interface SVGComponentTransferFunctionElement

This interface defines a base interface used by the component transfer function interfaces.

<a id="svgcomponenttransferfunctionelement"></a>

<a id="ref-for-InterfaceSVGElement④"></a>

<a id="ref-for-idl-unsigned-short②②"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_unknown"></a>

<a id="ref-for-idl-unsigned-short②③"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_identity"></a>

<a id="ref-for-idl-unsigned-short②④"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_table"></a>

<a id="ref-for-idl-unsigned-short②⑤"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_discrete"></a>

<a id="ref-for-idl-unsigned-short②⑥"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_linear"></a>

<a id="ref-for-idl-unsigned-short②⑦"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_gamma"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑧"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-type"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList②"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-tablevalues"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-slope"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-intercept"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-amplitude"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-exponent"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-offset"></a>

```text
interface SVGComponentTransferFunctionElement : SVGElement {

  // Component Transfer Types
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN = 0;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY = 1;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_TABLE = 2;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE = 3;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_LINEAR = 4;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_GAMMA = 5;

  readonly attribute SVGAnimatedEnumeration type;
  readonly attribute SVGAnimatedNumberList tableValues;
  readonly attribute SVGAnimatedNumber slope;
  readonly attribute SVGAnimatedNumber intercept;
  readonly attribute SVGAnimatedNumber amplitude;
  readonly attribute SVGAnimatedNumber exponent;
  readonly attribute SVGAnimatedNumber offset;
};
```
Constants in group “Component Transfer Types”:  
<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_unknown"></a>`SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_identity"></a>`SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY`  
Corresponds to value identity.

<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_table"></a>`SVG_FECOMPONENTTRANSFER_TYPE_TABLE`  
Corresponds to value table.

<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_discrete"></a>`SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE`  
Corresponds to value discrete.

<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_linear"></a>`SVG_FECOMPONENTTRANSFER_TYPE_LINEAR`  
Corresponds to value linear.

<a id="dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_gamma"></a>`SVG_FECOMPONENTTRANSFER_TYPE_GAMMA`  
Corresponds to value gamma.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑨"></a>

<a id="dom-svgcomponenttransferfunctionelement-type"></a>`type`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③⑦"></a>

<a id="ref-for-elementdef-fecomponenttransfer⑨"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element. Takes one of the SVG_FECOMPONENTTRANSFER_TYPE\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedNumberList③"></a>

<a id="dom-svgcomponenttransferfunctionelement-tablevalues"></a>`tableValues`, of type [SVGAnimatedNumberList](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumberList), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-tablevalues⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①⓪"></a>

Corresponds to attribute [tableValues](#element-attrdef-fecomponenttransfer-tablevalues) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element. Takes one of the SVG_FECOLORMATRIX_TYPE\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤"></a>

<a id="dom-svgcomponenttransferfunctionelement-slope"></a>`slope`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-slope⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①①"></a>

Corresponds to attribute [slope](#element-attrdef-fecomponenttransfer-slope) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥"></a>

<a id="dom-svgcomponenttransferfunctionelement-intercept"></a>`intercept`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-intercept⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①②"></a>

Corresponds to attribute [intercept](#element-attrdef-fecomponenttransfer-intercept) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑦"></a>

<a id="dom-svgcomponenttransferfunctionelement-amplitude"></a>`amplitude`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-amplitude⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①③"></a>

Corresponds to attribute [amplitude](#element-attrdef-fecomponenttransfer-amplitude) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧"></a>

<a id="dom-svgcomponenttransferfunctionelement-exponent"></a>`exponent`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-exponent⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①④"></a>

Corresponds to attribute [exponent](#element-attrdef-fecomponenttransfer-exponent) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑨"></a>

<a id="dom-svgcomponenttransferfunctionelement-offset"></a>`offset`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomponenttransfer-offset⑥"></a>

<a id="ref-for-elementdef-fecomponenttransfer①⑤"></a>

Corresponds to attribute [offset](#element-attrdef-fecomponenttransfer-offset) on the given [feComponentTransfer](#elementdef-fecomponenttransfer) element.

### <a id="InterfaceSVGFEFuncRElement"></a>Interface SVGFEFuncRElement

<a id="ref-for-elementdef-fefuncr⑨"></a>

The <a id="svgfefuncrelement"></a>`SVGFEFuncRElement` interface corresponds to the [feFuncR](#elementdef-fefuncr) element.

<a id="ref-for-svgfefuncrelement"></a>

<a id="ref-for-svgcomponenttransferfunctionelement"></a>

```text
interface SVGFEFuncRElement : SVGComponentTransferFunctionElement {
};
```
### <a id="InterfaceSVGFEFuncGElement"></a>Interface SVGFEFuncGElement

<a id="ref-for-elementdef-fefuncg⑤"></a>

The <a id="svgfefuncgelement"></a>`SVGFEFuncGElement` interface corresponds to the [feFuncG](#elementdef-fefuncg) element.

<a id="ref-for-svgfefuncgelement"></a>

<a id="ref-for-svgcomponenttransferfunctionelement①"></a>

```text
interface SVGFEFuncGElement : SVGComponentTransferFunctionElement {
};
```
### <a id="InterfaceSVGFEFuncBElement"></a>Interface SVGFEFuncBElement

<a id="ref-for-elementdef-fefuncb⑤"></a>

The <a id="svgfefuncbelement"></a>`SVGFEFuncBElement` interface corresponds to the [feFuncB](#elementdef-fefuncb) element.

<a id="ref-for-svgfefuncbelement"></a>

<a id="ref-for-svgcomponenttransferfunctionelement②"></a>

```text
interface SVGFEFuncBElement : SVGComponentTransferFunctionElement {
};
```
### <a id="InterfaceSVGFEFuncAElement"></a>Interface SVGFEFuncAElement

<a id="ref-for-elementdef-fefunca⑤"></a>

The <a id="svgfefuncaelement"></a>`SVGFEFuncAElement` interface corresponds to the [feFuncA](#elementdef-fefunca) element.

<a id="ref-for-svgfefuncaelement"></a>

<a id="ref-for-svgcomponenttransferfunctionelement③"></a>

```text
interface SVGFEFuncAElement : SVGComponentTransferFunctionElement {
};
```
### <a id="InterfaceSVGFECompositeElement"></a>Interface SVGFECompositeElement

<a id="ref-for-elementdef-fecomposite①①"></a>

The <a id="svgfecompositeelement"></a>`SVGFECompositeElement` interface corresponds to the [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-svgfecompositeelement"></a>

<a id="ref-for-InterfaceSVGElement⑤"></a>

<a id="ref-for-idl-unsigned-short②⑧"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_unknown"></a>

<a id="ref-for-idl-unsigned-short②⑨"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_over"></a>

<a id="ref-for-idl-unsigned-short③⓪"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_in"></a>

<a id="ref-for-idl-unsigned-short③①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_out"></a>

<a id="ref-for-idl-unsigned-short③②"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_atop"></a>

<a id="ref-for-idl-unsigned-short③③"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_xor"></a>

<a id="ref-for-idl-unsigned-short③④"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_arithmetic"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⓪"></a>

<a id="ref-for-dom-svgfecompositeelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedString①①"></a>

<a id="ref-for-dom-svgfecompositeelement-in2"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⓪"></a>

<a id="ref-for-dom-svgfecompositeelement-operator"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⓪"></a>

<a id="ref-for-dom-svgfecompositeelement-k1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①①"></a>

<a id="ref-for-dom-svgfecompositeelement-k2"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①②"></a>

<a id="ref-for-dom-svgfecompositeelement-k3"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①③"></a>

<a id="ref-for-dom-svgfecompositeelement-k4"></a>

<a id="ref-for-svgfecompositeelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes③"></a>

```text
interface SVGFECompositeElement : SVGElement {

  // Composite Operators
  const unsigned short SVG_FECOMPOSITE_OPERATOR_UNKNOWN = 0;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OVER = 1;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_IN = 2;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OUT = 3;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ATOP = 4;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_XOR = 5;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ARITHMETIC = 6;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedEnumeration operator;
  readonly attribute SVGAnimatedNumber k1;
  readonly attribute SVGAnimatedNumber k2;
  readonly attribute SVGAnimatedNumber k3;
  readonly attribute SVGAnimatedNumber k4;
};

SVGFECompositeElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Composite Operators”:  
<a id="dom-svgfecompositeelement-svg_fecomposite_operator_unknown"></a>`SVG_FECOMPOSITE_OPERATOR_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_over"></a>`SVG_FECOMPOSITE_OPERATOR_OVER`  
<a id="ref-for-attr-valuedef-operator-over②"></a>

Corresponds to value [over](#attr-valuedef-operator-over).

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_in"></a>`SVG_FECOMPOSITE_OPERATOR_IN`  
<a id="ref-for-attr-valuedef-operator-in"></a>

Corresponds to value [in](#attr-valuedef-operator-in).

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_out"></a>`SVG_FECOMPOSITE_OPERATOR_OUT`  
<a id="ref-for-attr-valuedef-operator-out"></a>

Corresponds to value [out](#attr-valuedef-operator-out).

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_atop"></a>`SVG_FECOMPOSITE_OPERATOR_ATOP`  
<a id="ref-for-attr-valuedef-operator-atop"></a>

Corresponds to value [atop](#attr-valuedef-operator-atop).

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_xor"></a>`SVG_FECOMPOSITE_OPERATOR_XOR`  
<a id="ref-for-attr-valuedef-operator-xor"></a>

Corresponds to value [xor](#attr-valuedef-operator-xor).

<a id="dom-svgfecompositeelement-svg_fecomposite_operator_arithmetic"></a>`SVG_FECOMPOSITE_OPERATOR_ARITHMETIC`  
<a id="ref-for-attr-valuedef-operator-arithmetic"></a>

Corresponds to value [arithmetic](#attr-valuedef-operator-arithmetic).

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString①②"></a>

<a id="dom-svgfecompositeelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③⑧"></a>

<a id="ref-for-elementdef-fecomposite①②"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedString①③"></a>

<a id="dom-svgfecompositeelement-in2"></a>`in2`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-fecomposite-in2④"></a>

<a id="ref-for-elementdef-fecomposite①③"></a>

Corresponds to attribute [in2](#element-attrdef-fecomposite-in2) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration①①"></a>

<a id="dom-svgfecompositeelement-operator"></a>`operator`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-fecomposite-operator③"></a>

<a id="ref-for-elementdef-fecomposite①④"></a>

Corresponds to attribute [operator](#element-attrdef-fecomposite-operator) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedNumber①④"></a>

<a id="dom-svgfecompositeelement-k1"></a>`k1`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomposite-k1②"></a>

<a id="ref-for-elementdef-fecomposite①⑤"></a>

Corresponds to attribute [k1](#element-attrdef-fecomposite-k1) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedNumber①⑤"></a>

<a id="dom-svgfecompositeelement-k2"></a>`k2`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomposite-k2②"></a>

<a id="ref-for-elementdef-fecomposite①⑥"></a>

Corresponds to attribute [k2](#element-attrdef-fecomposite-k2) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedNumber①⑥"></a>

<a id="dom-svgfecompositeelement-k3"></a>`k3`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomposite-k3②"></a>

<a id="ref-for-elementdef-fecomposite①⑦"></a>

Corresponds to attribute [k3](#element-attrdef-fecomposite-k3) on the given [feComposite](#elementdef-fecomposite) element.

<a id="ref-for-InterfaceSVGAnimatedNumber①⑦"></a>

<a id="dom-svgfecompositeelement-k4"></a>`k4`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fecomposite-k4②"></a>

<a id="ref-for-elementdef-fecomposite①⑧"></a>

Corresponds to attribute [k4](#element-attrdef-fecomposite-k4) on the given [feComposite](#elementdef-fecomposite) element.

### <a id="InterfaceSVGFEConvolveMatrixElement"></a>Interface SVGFEConvolveMatrixElement

<a id="ref-for-elementdef-feconvolvematrix⑤"></a>

The <a id="svgfeconvolvematrixelement"></a>`SVGFEConvolveMatrixElement` interface corresponds to the [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-svgfeconvolvematrixelement"></a>

<a id="ref-for-InterfaceSVGElement⑥"></a>

<a id="ref-for-idl-unsigned-short③⑤"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_unknown"></a>

<a id="ref-for-idl-unsigned-short③⑥"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_duplicate"></a>

<a id="ref-for-idl-unsigned-short③⑦"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_wrap"></a>

<a id="ref-for-idl-unsigned-short③⑧"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_none"></a>

<a id="ref-for-InterfaceSVGAnimatedString①④"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-orderx"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-ordery"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList④"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelmatrix"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⑧"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-divisor"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⑨"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-bias"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger②"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-targetx"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger③"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-targety"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①②"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-edgemode"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⓪"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelunitlengthx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelunitlengthy"></a>

<a id="ref-for-InterfaceSVGAnimatedBoolean"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-preservealpha"></a>

<a id="ref-for-svgfeconvolvematrixelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes④"></a>

```text
interface SVGFEConvolveMatrixElement : SVGElement {

  // Edge Mode Values
  const unsigned short SVG_EDGEMODE_UNKNOWN = 0;
  const unsigned short SVG_EDGEMODE_DUPLICATE = 1;
  const unsigned short SVG_EDGEMODE_WRAP = 2;
  const unsigned short SVG_EDGEMODE_NONE = 3;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedInteger orderX;
  readonly attribute SVGAnimatedInteger orderY;
  readonly attribute SVGAnimatedNumberList kernelMatrix;
  readonly attribute SVGAnimatedNumber divisor;
  readonly attribute SVGAnimatedNumber bias;
  readonly attribute SVGAnimatedInteger targetX;
  readonly attribute SVGAnimatedInteger targetY;
  readonly attribute SVGAnimatedEnumeration edgeMode;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
  readonly attribute SVGAnimatedBoolean preserveAlpha;
};

SVGFEConvolveMatrixElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Edge Mode Values”:  
<a id="dom-svgfeconvolvematrixelement-svg_edgemode_unknown"></a>`SVG_EDGEMODE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfeconvolvematrixelement-svg_edgemode_duplicate"></a>`SVG_EDGEMODE_DUPLICATE`  
<a id="ref-for-attr-valuedef-edgemode-duplicate②"></a>

Corresponds to value [duplicate](#attr-valuedef-edgemode-duplicate).

<a id="dom-svgfeconvolvematrixelement-svg_edgemode_wrap"></a>`SVG_EDGEMODE_WRAP`  
<a id="ref-for-attr-valuedef-edgemode-wrap④"></a>

Corresponds to value [wrap](#attr-valuedef-edgemode-wrap).

<a id="dom-svgfeconvolvematrixelement-svg_edgemode_none"></a>`SVG_EDGEMODE_NONE`  
Corresponds to value none.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString①⑤"></a>

<a id="dom-svgfeconvolvematrixelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in③⑨"></a>

<a id="ref-for-elementdef-feconvolvematrix⑥"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedInteger④"></a>

<a id="dom-svgfeconvolvematrixelement-orderx"></a>`orderX`, of type [SVGAnimatedInteger](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedInteger), readonly

<a id="ref-for-element-attrdef-order③"></a>

<a id="ref-for-elementdef-feconvolvematrix⑦"></a>

Corresponds to attribute [order](#element-attrdef-order) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedInteger⑤"></a>

<a id="dom-svgfeconvolvematrixelement-ordery"></a>`orderY`, of type [SVGAnimatedInteger](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedInteger), readonly

<a id="ref-for-element-attrdef-order④"></a>

<a id="ref-for-elementdef-feconvolvematrix⑧"></a>

Corresponds to attribute [order](#element-attrdef-order) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedNumberList⑤"></a>

<a id="dom-svgfeconvolvematrixelement-kernelmatrix"></a>`kernelMatrix`, of type [SVGAnimatedNumberList](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumberList), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-kernelmatrix⑧"></a>

<a id="ref-for-elementdef-feconvolvematrix⑨"></a>

Corresponds to attribute [kernelMatrix](#element-attrdef-feconvolvematrix-kernelmatrix) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedNumber②②"></a>

<a id="dom-svgfeconvolvematrixelement-divisor"></a>`divisor`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-divisor⑥"></a>

<a id="ref-for-elementdef-feconvolvematrix①⓪"></a>

Corresponds to attribute [divisor](#element-attrdef-feconvolvematrix-divisor) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedNumber②③"></a>

<a id="dom-svgfeconvolvematrixelement-bias"></a>`bias`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-bias⑥"></a>

<a id="ref-for-elementdef-feconvolvematrix①①"></a>

Corresponds to attribute [bias](#element-attrdef-feconvolvematrix-bias) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedInteger⑥"></a>

<a id="dom-svgfeconvolvematrixelement-targetx"></a>`targetX`, of type [SVGAnimatedInteger](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedInteger), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-targetx④"></a>

<a id="ref-for-elementdef-feconvolvematrix①②"></a>

Corresponds to attribute [targetX](#element-attrdef-feconvolvematrix-targetx) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedInteger⑦"></a>

<a id="dom-svgfeconvolvematrixelement-targety"></a>`targetY`, of type [SVGAnimatedInteger](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedInteger), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-targety④"></a>

<a id="ref-for-elementdef-feconvolvematrix①③"></a>

Corresponds to attribute [targetY](#element-attrdef-feconvolvematrix-targety) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration①③"></a>

<a id="dom-svgfeconvolvematrixelement-edgemode"></a>`edgeMode`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-edgemode②"></a>

<a id="ref-for-elementdef-feconvolvematrix①④"></a>

Corresponds to attribute [edgeMode](#element-attrdef-feconvolvematrix-edgemode) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedNumber②④"></a>

<a id="dom-svgfeconvolvematrixelement-kernelunitlengthx"></a>`kernelUnitLengthX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength⑨"></a>

<a id="ref-for-elementdef-feconvolvematrix①⑤"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedNumber②⑤"></a>

<a id="dom-svgfeconvolvematrixelement-kernelunitlengthy"></a>`kernelUnitLengthY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-kernelunitlength①⓪"></a>

<a id="ref-for-elementdef-feconvolvematrix①⑥"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-feconvolvematrix-kernelunitlength) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

<a id="ref-for-InterfaceSVGAnimatedBoolean①"></a>

<a id="dom-svgfeconvolvematrixelement-preservealpha"></a>`preserveAlpha`, of type [SVGAnimatedBoolean](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedBoolean), readonly

<a id="ref-for-element-attrdef-feconvolvematrix-preservealpha②"></a>

<a id="ref-for-elementdef-feconvolvematrix①⑦"></a>

Corresponds to attribute [preserveAlpha](#element-attrdef-feconvolvematrix-preservealpha) on the given [feConvolveMatrix](#elementdef-feconvolvematrix) element.

### <a id="InterfaceSVGFEDiffuseLightingElement"></a>Interface SVGFEDiffuseLightingElement

<a id="ref-for-elementdef-fediffuselighting①②"></a>

The <a id="svgfediffuselightingelement"></a>`SVGFEDiffuseLightingElement` interface corresponds to the [feDiffuseLighting](#elementdef-fediffuselighting) element.

<a id="ref-for-svgfediffuselightingelement"></a>

<a id="ref-for-InterfaceSVGElement⑦"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑥"></a>

<a id="ref-for-dom-svgfediffuselightingelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑥"></a>

<a id="ref-for-dom-svgfediffuselightingelement-surfacescale"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑦"></a>

<a id="ref-for-dom-svgfediffuselightingelement-diffuseconstant"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑧"></a>

<a id="ref-for-dom-svgfediffuselightingelement-kernelunitlengthx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑨"></a>

<a id="ref-for-dom-svgfediffuselightingelement-kernelunitlengthy"></a>

<a id="ref-for-svgfediffuselightingelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑤"></a>

```text
interface SVGFEDiffuseLightingElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber surfaceScale;
  readonly attribute SVGAnimatedNumber diffuseConstant;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
};

SVGFEDiffuseLightingElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString①⑦"></a>

<a id="dom-svgfediffuselightingelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④⓪"></a>

<a id="ref-for-elementdef-fediffuselighting①③"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feDiffuseLighting](#elementdef-fediffuselighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber③⓪"></a>

<a id="dom-svgfediffuselightingelement-surfacescale"></a>`surfaceScale`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fediffuselighting-surfacescale①"></a>

<a id="ref-for-elementdef-fediffuselighting①④"></a>

Corresponds to attribute [surfaceScale](#element-attrdef-fediffuselighting-surfacescale) on the given [feDiffuseLighting](#elementdef-fediffuselighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber③①"></a>

<a id="dom-svgfediffuselightingelement-diffuseconstant"></a>`diffuseConstant`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fediffuselighting-diffuseconstant①"></a>

<a id="ref-for-elementdef-fediffuselighting①⑤"></a>

Corresponds to attribute [diffuseConstant](#element-attrdef-fediffuselighting-diffuseconstant) on the given [feDiffuseLighting](#elementdef-fediffuselighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber③②"></a>

<a id="dom-svgfediffuselightingelement-kernelunitlengthx"></a>`kernelUnitLengthX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength⑨"></a>

<a id="ref-for-elementdef-fediffuselighting①⑥"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength) on the given [feDiffuseLighting](#elementdef-fediffuselighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber③③"></a>

<a id="dom-svgfediffuselightingelement-kernelunitlengthy"></a>`kernelUnitLengthY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fediffuselighting-kernelunitlength①⓪"></a>

<a id="ref-for-elementdef-fediffuselighting①⑦"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-fediffuselighting-kernelunitlength) on the given [feDiffuseLighting](#elementdef-fediffuselighting) element.

### <a id="InterfaceSVGFEDistantLightElement"></a>Interface SVGFEDistantLightElement

<a id="ref-for-elementdef-fedistantlight④"></a>

The <a id="svgfedistantlightelement"></a>`SVGFEDistantLightElement` interface corresponds to the [feDistantLight](#elementdef-fedistantlight) element.

<a id="ref-for-svgfedistantlightelement"></a>

<a id="ref-for-InterfaceSVGElement⑧"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③④"></a>

<a id="ref-for-dom-svgfedistantlightelement-azimuth"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑤"></a>

<a id="ref-for-dom-svgfedistantlightelement-elevation"></a>

```text
interface SVGFEDistantLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber azimuth;
  readonly attribute SVGAnimatedNumber elevation;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedNumber③⑥"></a>

<a id="dom-svgfedistantlightelement-azimuth"></a>`azimuth`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedistantlight-azimuth③"></a>

<a id="ref-for-elementdef-fedistantlight⑤"></a>

Corresponds to attribute [azimuth](#element-attrdef-fedistantlight-azimuth) on the given [feDistantLight](#elementdef-fedistantlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber③⑦"></a>

<a id="dom-svgfedistantlightelement-elevation"></a>`elevation`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedistantlight-elevation③"></a>

<a id="ref-for-elementdef-fedistantlight⑥"></a>

Corresponds to attribute [elevation](#element-attrdef-fedistantlight-elevation) on the given [feDistantLight](#elementdef-fedistantlight) element.

### <a id="InterfaceSVGFEPointLightElement"></a>Interface SVGFEPointLightElement

<a id="ref-for-elementdef-fepointlight④"></a>

The <a id="svgfepointlightelement"></a>`SVGFEPointLightElement` interface corresponds to the [fePointLight](#elementdef-fepointlight) element.

<a id="ref-for-svgfepointlightelement"></a>

<a id="ref-for-InterfaceSVGElement⑨"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑧"></a>

<a id="ref-for-dom-svgfepointlightelement-x"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑨"></a>

<a id="ref-for-dom-svgfepointlightelement-y"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⓪"></a>

<a id="ref-for-dom-svgfepointlightelement-z"></a>

```text
interface SVGFEPointLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber x;
  readonly attribute SVGAnimatedNumber y;
  readonly attribute SVGAnimatedNumber z;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedNumber④①"></a>

<a id="dom-svgfepointlightelement-x"></a>`x`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fepointlight-x②"></a>

<a id="ref-for-elementdef-fepointlight⑤"></a>

Corresponds to attribute [x](#element-attrdef-fepointlight-x) on the given [fePointLight](#elementdef-fepointlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber④②"></a>

<a id="dom-svgfepointlightelement-y"></a>`y`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fepointlight-y②"></a>

<a id="ref-for-elementdef-fepointlight⑥"></a>

Corresponds to attribute [y](#element-attrdef-fepointlight-y) on the given [fePointLight](#elementdef-fepointlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber④③"></a>

<a id="dom-svgfepointlightelement-z"></a>`z`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fepointlight-z②"></a>

<a id="ref-for-elementdef-fepointlight⑦"></a>

Corresponds to attribute [z](#element-attrdef-fepointlight-z) on the given [fePointLight](#elementdef-fepointlight) element.

### <a id="InterfaceSVGFESpotLightElement"></a>Interface SVGFESpotLightElement

<a id="ref-for-elementdef-fespotlight⑥"></a>

The <a id="svgfespotlightelement"></a>`SVGFESpotLightElement` interface corresponds to the [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-svgfespotlightelement"></a>

<a id="ref-for-InterfaceSVGElement①⓪"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④④"></a>

<a id="ref-for-dom-svgfespotlightelement-x"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑤"></a>

<a id="ref-for-dom-svgfespotlightelement-y"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑥"></a>

<a id="ref-for-dom-svgfespotlightelement-z"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑦"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsatx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑧"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsaty"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑨"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsatz"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⓪"></a>

<a id="ref-for-dom-svgfespotlightelement-specularexponent"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑤①"></a>

<a id="ref-for-dom-svgfespotlightelement-limitingconeangle"></a>

```text
interface SVGFESpotLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber x;
  readonly attribute SVGAnimatedNumber y;
  readonly attribute SVGAnimatedNumber z;
  readonly attribute SVGAnimatedNumber pointsAtX;
  readonly attribute SVGAnimatedNumber pointsAtY;
  readonly attribute SVGAnimatedNumber pointsAtZ;
  readonly attribute SVGAnimatedNumber specularExponent;
  readonly attribute SVGAnimatedNumber limitingConeAngle;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedNumber⑤②"></a>

<a id="dom-svgfespotlightelement-x"></a>`x`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-x②"></a>

<a id="ref-for-elementdef-fespotlight⑦"></a>

Corresponds to attribute [x](#element-attrdef-fespotlight-x) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤③"></a>

<a id="dom-svgfespotlightelement-y"></a>`y`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-y②"></a>

<a id="ref-for-elementdef-fespotlight⑧"></a>

Corresponds to attribute [y](#element-attrdef-fespotlight-y) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤④"></a>

<a id="dom-svgfespotlightelement-z"></a>`z`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-z②"></a>

<a id="ref-for-elementdef-fespotlight⑨"></a>

Corresponds to attribute [z](#element-attrdef-fespotlight-z) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⑤"></a>

<a id="dom-svgfespotlightelement-pointsatx"></a>`pointsAtX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-pointsatx②"></a>

<a id="ref-for-elementdef-fespotlight①⓪"></a>

Corresponds to attribute [pointsAtX](#element-attrdef-fespotlight-pointsatx) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⑥"></a>

<a id="dom-svgfespotlightelement-pointsaty"></a>`pointsAtY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-pointsaty②"></a>

<a id="ref-for-elementdef-fespotlight①①"></a>

Corresponds to attribute [pointsAtY](#element-attrdef-fespotlight-pointsaty) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⑦"></a>

<a id="dom-svgfespotlightelement-pointsatz"></a>`pointsAtZ`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-pointsatz②"></a>

<a id="ref-for-elementdef-fespotlight①②"></a>

Corresponds to attribute [pointsAtZ](#element-attrdef-fespotlight-pointsatz) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⑧"></a>

<a id="dom-svgfespotlightelement-specularexponent"></a>`specularExponent`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-specularexponent④"></a>

<a id="ref-for-elementdef-fespotlight①③"></a>

Corresponds to attribute [specularExponent](#element-attrdef-fespotlight-specularexponent) on the given [feSpotLight](#elementdef-fespotlight) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⑨"></a>

<a id="dom-svgfespotlightelement-limitingconeangle"></a>`limitingConeAngle`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespotlight-limitingconeangle③"></a>

<a id="ref-for-elementdef-fespotlight①④"></a>

Corresponds to attribute [limitingConeAngle](#element-attrdef-fespotlight-limitingconeangle) on the given [feSpotLight](#elementdef-fespotlight) element.

### <a id="InterfaceSVGFEDisplacementMapElement"></a>Interface SVGFEDisplacementMapElement

<a id="ref-for-elementdef-fedisplacementmap⑦"></a>

The <a id="svgfedisplacementmapelement"></a>`SVGFEDisplacementMapElement` interface corresponds to the [feDisplacementMap](#elementdef-fedisplacementmap) element.

<a id="ref-for-svgfedisplacementmapelement"></a>

<a id="ref-for-InterfaceSVGElement①①"></a>

<a id="ref-for-idl-unsigned-short③⑨"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_unknown"></a>

<a id="ref-for-idl-unsigned-short④⓪"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_r"></a>

<a id="ref-for-idl-unsigned-short④①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_g"></a>

<a id="ref-for-idl-unsigned-short④②"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_b"></a>

<a id="ref-for-idl-unsigned-short④③"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_a"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑧"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑨"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-in2"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⓪"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-scale"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①④"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-xchannelselector"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑤"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-ychannelselector"></a>

<a id="ref-for-svgfedisplacementmapelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑥"></a>

```text
interface SVGFEDisplacementMapElement : SVGElement {

  // Channel Selectors
  const unsigned short SVG_CHANNEL_UNKNOWN = 0;
  const unsigned short SVG_CHANNEL_R = 1;
  const unsigned short SVG_CHANNEL_G = 2;
  const unsigned short SVG_CHANNEL_B = 3;
  const unsigned short SVG_CHANNEL_A = 4;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedNumber scale;
  readonly attribute SVGAnimatedEnumeration xChannelSelector;
  readonly attribute SVGAnimatedEnumeration yChannelSelector;
};

SVGFEDisplacementMapElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Channel Selectors”:  
<a id="dom-svgfedisplacementmapelement-svg_channel_unknown"></a>`SVG_CHANNEL_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfedisplacementmapelement-svg_channel_r"></a>`SVG_CHANNEL_R`  
Corresponds to value R.

<a id="dom-svgfedisplacementmapelement-svg_channel_g"></a>`SVG_CHANNEL_G`  
Corresponds to value G.

<a id="dom-svgfedisplacementmapelement-svg_channel_b"></a>`SVG_CHANNEL_B`  
Corresponds to value B.

<a id="dom-svgfedisplacementmapelement-svg_channel_a"></a>`SVG_CHANNEL_A`  
Corresponds to value A.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString②⓪"></a>

<a id="dom-svgfedisplacementmapelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④①"></a>

<a id="ref-for-elementdef-fedisplacementmap⑧"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feDisplacementMap](#elementdef-fedisplacementmap) element.

<a id="ref-for-InterfaceSVGAnimatedString②①"></a>

<a id="dom-svgfedisplacementmapelement-in2"></a>`in2`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-fedisplacementmap-in2⑨"></a>

<a id="ref-for-elementdef-fedisplacementmap⑨"></a>

Corresponds to attribute [in2](#element-attrdef-fedisplacementmap-in2) on the given [feDisplacementMap](#elementdef-fedisplacementmap) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥①"></a>

<a id="dom-svgfedisplacementmapelement-scale"></a>`scale`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedisplacementmap-scale③"></a>

<a id="ref-for-elementdef-fedisplacementmap①⓪"></a>

Corresponds to attribute [scale](#element-attrdef-fedisplacementmap-scale) on the given [feDisplacementMap](#elementdef-fedisplacementmap) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑥"></a>

<a id="dom-svgfedisplacementmapelement-xchannelselector"></a>`xChannelSelector`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-fedisplacementmap-xchannelselector④"></a>

<a id="ref-for-elementdef-fedisplacementmap①①"></a>

Corresponds to attribute [xChannelSelector](#element-attrdef-fedisplacementmap-xchannelselector) on the given [feDisplacementMap](#elementdef-fedisplacementmap) element. Takes one of the SVG_CHANNEL\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑦"></a>

<a id="dom-svgfedisplacementmapelement-ychannelselector"></a>`yChannelSelector`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-fedisplacementmap-ychannelselector④"></a>

<a id="ref-for-elementdef-fedisplacementmap①②"></a>

Corresponds to attribute [yChannelSelector](#element-attrdef-fedisplacementmap-ychannelselector) on the given [feDisplacementMap](#elementdef-fedisplacementmap) element. Takes one of the SVG_CHANNEL\_\* constants defined on this interface.

### <a id="InterfaceSVGFEDropShadowElement"></a>Interface SVGFEDropShadowElement

<a id="ref-for-elementdef-fedropshadow①④"></a>

The <a id="svgfedropshadowelement"></a>`SVGFEDropShadowElement` interface corresponds to the [feDropShadow](#elementdef-fedropshadow) element.

<a id="ref-for-svgfedropshadowelement"></a>

<a id="ref-for-InterfaceSVGElement①②"></a>

<a id="ref-for-InterfaceSVGAnimatedString②②"></a>

<a id="ref-for-dom-svgfedropshadowelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥②"></a>

<a id="ref-for-dom-svgfedropshadowelement-dx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥③"></a>

<a id="ref-for-dom-svgfedropshadowelement-dy"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥④"></a>

<a id="ref-for-dom-svgfedropshadowelement-stddeviationx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑤"></a>

<a id="ref-for-dom-svgfedropshadowelement-stddeviationy"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation"></a>

<a id="ref-for-idl-float"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx"></a>

<a id="ref-for-idl-float①"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy"></a>

<a id="ref-for-svgfedropshadowelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑦"></a>

```text
interface SVGFEDropShadowElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber dx;
  readonly attribute SVGAnimatedNumber dy;
  readonly attribute SVGAnimatedNumber stdDeviationX;
  readonly attribute SVGAnimatedNumber stdDeviationY;

  void setStdDeviation(float stdDeviationX, float stdDeviationY);
};

SVGFEDropShadowElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString②③"></a>

<a id="dom-svgfedropshadowelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④②"></a>

<a id="ref-for-elementdef-fedropshadow①⑤"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feDropShadow](#elementdef-fedropshadow) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑥"></a>

<a id="dom-svgfedropshadowelement-dx"></a>`dx`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedropshadow-dx③"></a>

<a id="ref-for-elementdef-fedropshadow①⑥"></a>

Corresponds to attribute [dx](#element-attrdef-fedropshadow-dx) on the given [feDropShadow](#elementdef-fedropshadow) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑦"></a>

<a id="dom-svgfedropshadowelement-dy"></a>`dy`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedropshadow-dy③"></a>

<a id="ref-for-elementdef-fedropshadow①⑦"></a>

Corresponds to attribute [dy](#element-attrdef-fedropshadow-dy) on the given [feDropShadow](#elementdef-fedropshadow) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑧"></a>

<a id="dom-svgfedropshadowelement-stddeviationx"></a>`stdDeviationX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedropshadow-stddeviation③"></a>

<a id="ref-for-elementdef-fedropshadow①⑧"></a>

Corresponds to attribute [stdDeviation](#element-attrdef-fedropshadow-stddeviation) on the given [feDropShadow](#elementdef-fedropshadow) element. Contains the X component of attribute <a id="ref-for-element-attrdef-fedropshadow-stddeviation④"></a>stdDeviation.

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑨"></a>

<a id="dom-svgfedropshadowelement-stddeviationy"></a>`stdDeviationY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fedropshadow-stddeviation⑤"></a>

<a id="ref-for-elementdef-fedropshadow①⑨"></a>

Corresponds to attribute [stdDeviation](#element-attrdef-fedropshadow-stddeviation) on the given [feDropShadow](#elementdef-fedropshadow) element. Contains the Y component of attribute <a id="ref-for-element-attrdef-fedropshadow-stddeviation⑥"></a>stdDeviation.

Methods:  
<a id="dom-svgfedropshadowelement-setstddeviation"></a>`setStdDeviation(stdDeviationX, stdDeviationY)`  
<a id="ref-for-element-attrdef-fedropshadow-stddeviation⑦"></a>

Sets the values for attribute [stdDeviation](#element-attrdef-fedropshadow-stddeviation).

<a id="dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx"></a>`stdDeviationX`  
<a id="ref-for-element-attrdef-fedropshadow-stddeviation⑧"></a>

The X component of attribute [stdDeviation](#element-attrdef-fedropshadow-stddeviation).

<a id="dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy"></a>`stdDeviationY`  
<a id="ref-for-element-attrdef-fedropshadow-stddeviation⑨"></a>

The Y component of attribute [stdDeviation](#element-attrdef-fedropshadow-stddeviation).

### <a id="InterfaceSVGFEFloodElement"></a>Interface SVGFEFloodElement

<a id="ref-for-elementdef-feflood①③"></a>

The <a id="svgfefloodelement"></a>`SVGFEFloodElement` interface corresponds to the [feFlood](#elementdef-feflood) element.

<a id="ref-for-svgfefloodelement"></a>

<a id="ref-for-InterfaceSVGElement①③"></a>

<a id="ref-for-svgfefloodelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑧"></a>

```text
interface SVGFEFloodElement : SVGElement {
};

SVGFEFloodElement includes SVGFilterPrimitiveStandardAttributes;
```
### <a id="InterfaceSVGFEGaussianBlurElement"></a>Interface SVGFEGaussianBlurElement

<a id="ref-for-elementdef-fegaussianblur①③"></a>

The <a id="svgfegaussianblurelement"></a>`SVGFEGaussianBlurElement` interface corresponds to the [feGaussianBlur](#elementdef-fegaussianblur) element.

<a id="ref-for-svgfegaussianblurelement"></a>

<a id="ref-for-InterfaceSVGElement①④"></a>

<a id="ref-for-idl-unsigned-short④④"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_unknown"></a>

<a id="ref-for-idl-unsigned-short④⑤"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_duplicate"></a>

<a id="ref-for-idl-unsigned-short④⑥"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_wrap"></a>

<a id="ref-for-idl-unsigned-short④⑦"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_none"></a>

<a id="ref-for-InterfaceSVGAnimatedString②④"></a>

<a id="ref-for-dom-svgfegaussianblurelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⓪"></a>

<a id="ref-for-dom-svgfegaussianblurelement-stddeviationx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-stddeviationy"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑧"></a>

<a id="ref-for-dom-svgfegaussianblurelement-edgemode"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation"></a>

<a id="ref-for-idl-float②"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx"></a>

<a id="ref-for-idl-float③"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy"></a>

<a id="ref-for-svgfegaussianblurelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑨"></a>

```text
interface SVGFEGaussianBlurElement : SVGElement {

  // Edge Mode Values
  const unsigned short SVG_EDGEMODE_UNKNOWN = 0;
  const unsigned short SVG_EDGEMODE_DUPLICATE = 1;
  const unsigned short SVG_EDGEMODE_WRAP = 2;
  const unsigned short SVG_EDGEMODE_NONE = 3;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber stdDeviationX;
  readonly attribute SVGAnimatedNumber stdDeviationY;
  readonly attribute SVGAnimatedEnumeration edgeMode;

  void setStdDeviation(float stdDeviationX, float stdDeviationY);
};

SVGFEGaussianBlurElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Edge Mode Values”:  
<a id="dom-svgfegaussianblurelement-svg_edgemode_unknown"></a>`SVG_EDGEMODE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfegaussianblurelement-svg_edgemode_duplicate"></a>`SVG_EDGEMODE_DUPLICATE`  
Corresponds to value duplicate.

<a id="dom-svgfegaussianblurelement-svg_edgemode_wrap"></a>`SVG_EDGEMODE_WRAP`  
<a id="ref-for-attr-valuedef-edgemode-wrap⑤"></a>

Corresponds to value [wrap](#attr-valuedef-edgemode-wrap).

<a id="dom-svgfegaussianblurelement-svg_edgemode_none"></a>`SVG_EDGEMODE_NONE`  
Corresponds to value none.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString②⑤"></a>

<a id="dom-svgfegaussianblurelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④③"></a>

<a id="ref-for-elementdef-fegaussianblur①④"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feGaussianBlur](#elementdef-fegaussianblur) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑦②"></a>

<a id="dom-svgfegaussianblurelement-stddeviationx"></a>`stdDeviationX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation⑧"></a>

<a id="ref-for-elementdef-fegaussianblur①⑤"></a>

Corresponds to attribute [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) on the given [feGaussianBlur](#elementdef-fegaussianblur) element. Contains the X component of attribute <a id="ref-for-element-attrdef-fegaussianblur-stddeviation⑨"></a>stdDeviation.

<a id="ref-for-InterfaceSVGAnimatedNumber⑦③"></a>

<a id="dom-svgfegaussianblurelement-stddeviationy"></a>`stdDeviationY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fegaussianblur-stddeviation①⓪"></a>

<a id="ref-for-elementdef-fegaussianblur①⑥"></a>

Corresponds to attribute [stdDeviation](#element-attrdef-fegaussianblur-stddeviation) on the given [feGaussianBlur](#elementdef-fegaussianblur) element. Contains the Y component of attribute <a id="ref-for-element-attrdef-fegaussianblur-stddeviation①①"></a>stdDeviation.

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑨"></a>

<a id="dom-svgfegaussianblurelement-edgemode"></a>`edgeMode`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-fegaussianblur-edgemode③"></a>

<a id="ref-for-elementdef-fegaussianblur①⑦"></a>

Corresponds to attribute [edgeMode](#element-attrdef-fegaussianblur-edgemode) on the given [feGaussianBlur](#elementdef-fegaussianblur) element. Takes one of the SVG_EDGEMODE\_\* constants defined on this interface.

Methods:  
<a id="dom-svgfegaussianblurelement-setstddeviation"></a>`setStdDeviation(stdDeviationX, stdDeviationY)`  
<a id="ref-for-element-attrdef-fegaussianblur-stddeviation①②"></a>

Sets the values for attribute [stdDeviation](#element-attrdef-fegaussianblur-stddeviation).

<a id="dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx"></a>`stdDeviationX`  
<a id="ref-for-element-attrdef-fegaussianblur-stddeviation①③"></a>

The X component of attribute [stdDeviation](#element-attrdef-fegaussianblur-stddeviation).

<a id="dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy"></a>`stdDeviationY`  
<a id="ref-for-element-attrdef-fegaussianblur-stddeviation①④"></a>

The Y component of attribute [stdDeviation](#element-attrdef-fegaussianblur-stddeviation).

### <a id="InterfaceSVGFEImageElement"></a>Interface SVGFEImageElement

<a id="ref-for-elementdef-feimage⑧"></a>

The <a id="svgfeimageelement"></a>`SVGFEImageElement` interface corresponds to the [feImage](#elementdef-feimage) element.

<a id="ref-for-svgfeimageelement"></a>

<a id="ref-for-InterfaceSVGElement①⑤"></a>

<a id="ref-for-InterfaceSVGAnimatedPreserveAspectRatio"></a>

<a id="ref-for-dom-svgfeimageelement-preserveaspectratio"></a>

<a id="ref-for-InterfaceSVGAnimatedString②⑥"></a>

<a id="ref-for-dom-svgfeimageelement-crossorigin"></a>

<a id="ref-for-svgfeimageelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⓪"></a>

<a id="ref-for-svgfeimageelement②"></a>

<a id="ref-for-InterfaceSVGURIReference①"></a>

```text
interface SVGFEImageElement : SVGElement {
  readonly attribute SVGAnimatedPreserveAspectRatio preserveAspectRatio;
  readonly attribute SVGAnimatedString crossOrigin;
};

SVGFEImageElement includes SVGFilterPrimitiveStandardAttributes;
SVGFEImageElement includes SVGURIReference;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedPreserveAspectRatio①"></a>

<a id="dom-svgfeimageelement-preserveaspectratio"></a>`preserveAspectRatio`, of type [SVGAnimatedPreserveAspectRatio](https://www.w3.org/TR/svg2/coords.html#InterfaceSVGAnimatedPreserveAspectRatio), readonly

<a id="ref-for-element-attrdef-feimage-preserveaspectratio④"></a>

<a id="ref-for-elementdef-feimage⑨"></a>

Corresponds to attribute [preserveAspectRatio](#element-attrdef-feimage-preserveaspectratio) on the given [feImage](#elementdef-feimage) element.

<a id="ref-for-InterfaceSVGAnimatedString②⑦"></a>

<a id="dom-svgfeimageelement-crossorigin"></a>`crossOrigin`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-feimage-crossorigin①"></a>

The crossOrigin IDL attribute must reflect the [crossorigin](#element-attrdef-feimage-crossorigin) content attribute, limited to only known values.

### <a id="InterfaceSVGFEMergeElement"></a>Interface SVGFEMergeElement

<a id="ref-for-elementdef-femerge①⓪"></a>

The <a id="svgfemergeelement"></a>`SVGFEMergeElement` interface corresponds to the [feMerge](#elementdef-femerge) element.

<a id="ref-for-svgfemergeelement"></a>

<a id="ref-for-InterfaceSVGElement①⑥"></a>

<a id="ref-for-svgfemergeelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①①"></a>

```text
interface SVGFEMergeElement : SVGElement {
};

SVGFEMergeElement includes SVGFilterPrimitiveStandardAttributes;
```
### <a id="InterfaceSVGFEMergeNodeElement"></a>Interface SVGFEMergeNodeElement

<a id="ref-for-elementdef-femergenode⑥"></a>

The <a id="svgfemergenodeelement"></a>`SVGFEMergeNodeElement` interface corresponds to the [feMergeNode](#elementdef-femergenode) element.

<a id="ref-for-svgfemergenodeelement"></a>

<a id="ref-for-InterfaceSVGElement①⑦"></a>

<a id="ref-for-InterfaceSVGAnimatedString②⑧"></a>

<a id="ref-for-dom-svgfemergenodeelement-in1"></a>

```text
interface SVGFEMergeNodeElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString②⑨"></a>

<a id="dom-svgfemergenodeelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④④"></a>

<a id="ref-for-elementdef-femergenode⑦"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feMergeNode](#elementdef-femergenode) element.

### <a id="InterfaceSVGFEMorphologyElement"></a>Interface SVGFEMorphologyElement

<a id="ref-for-elementdef-femorphology④"></a>

The <a id="svgfemorphologyelement"></a>`SVGFEMorphologyElement` interface corresponds to the [feMorphology](#elementdef-femorphology) element.

<a id="ref-for-svgfemorphologyelement"></a>

<a id="ref-for-InterfaceSVGElement①⑧"></a>

<a id="ref-for-idl-unsigned-short④⑧"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_unknown"></a>

<a id="ref-for-idl-unsigned-short④⑨"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_erode"></a>

<a id="ref-for-idl-unsigned-short⑤⓪"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_dilate"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⓪"></a>

<a id="ref-for-dom-svgfemorphologyelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②⓪"></a>

<a id="ref-for-dom-svgfemorphologyelement-operator"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦④"></a>

<a id="ref-for-dom-svgfemorphologyelement-radiusx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑤"></a>

<a id="ref-for-dom-svgfemorphologyelement-radiusy"></a>

<a id="ref-for-svgfemorphologyelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①②"></a>

```text
interface SVGFEMorphologyElement : SVGElement {

  // Morphology Operators
  const unsigned short SVG_MORPHOLOGY_OPERATOR_UNKNOWN = 0;
  const unsigned short SVG_MORPHOLOGY_OPERATOR_ERODE = 1;
  const unsigned short SVG_MORPHOLOGY_OPERATOR_DILATE = 2;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedEnumeration operator;
  readonly attribute SVGAnimatedNumber radiusX;
  readonly attribute SVGAnimatedNumber radiusY;
};

SVGFEMorphologyElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Morphology Operators”:  
<a id="dom-svgfemorphologyelement-svg_morphology_operator_unknown"></a>`SVG_MORPHOLOGY_OPERATOR_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfemorphologyelement-svg_morphology_operator_erode"></a>`SVG_MORPHOLOGY_OPERATOR_ERODE`  
Corresponds to value erode.

<a id="dom-svgfemorphologyelement-svg_morphology_operator_dilate"></a>`SVG_MORPHOLOGY_OPERATOR_DILATE`  
Corresponds to value dilate.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedString③①"></a>

<a id="dom-svgfemorphologyelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④⑤"></a>

<a id="ref-for-elementdef-femorphology⑤"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feMorphology](#elementdef-femorphology) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration②①"></a>

<a id="dom-svgfemorphologyelement-operator"></a>`operator`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-femorphology-operator②"></a>

<a id="ref-for-elementdef-femorphology⑥"></a>

Corresponds to attribute [operator](#element-attrdef-femorphology-operator) on the given [feMorphology](#elementdef-femorphology) element. Takes one of the SVG_MORPHOLOGY_OPERATOR\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑥"></a>

<a id="dom-svgfemorphologyelement-radiusx"></a>`radiusX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-femorphology-radius②"></a>

<a id="ref-for-elementdef-femorphology⑦"></a>

Corresponds to attribute [radius](#element-attrdef-femorphology-radius) on the given [feMorphology](#elementdef-femorphology) element. Contains the X component of attribute <a id="ref-for-element-attrdef-femorphology-radius③"></a>radius.

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑦"></a>

<a id="dom-svgfemorphologyelement-radiusy"></a>`radiusY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-femorphology-radius④"></a>

<a id="ref-for-elementdef-femorphology⑧"></a>

Corresponds to attribute [radius](#element-attrdef-femorphology-radius) on the given [feMorphology](#elementdef-femorphology) element. Contains the Y component of attribute <a id="ref-for-element-attrdef-femorphology-radius⑤"></a>radius.

### <a id="InterfaceSVGFEOffsetElement"></a>Interface SVGFEOffsetElement

<a id="ref-for-elementdef-feoffset①④"></a>

The <a id="svgfeoffsetelement"></a>`SVGFEOffsetElement` interface corresponds to the [feOffset](#elementdef-feoffset) element.

<a id="ref-for-svgfeoffsetelement"></a>

<a id="ref-for-InterfaceSVGElement①⑨"></a>

<a id="ref-for-InterfaceSVGAnimatedString③②"></a>

<a id="ref-for-dom-svgfeoffsetelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑧"></a>

<a id="ref-for-dom-svgfeoffsetelement-dx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑨"></a>

<a id="ref-for-dom-svgfeoffsetelement-dy"></a>

<a id="ref-for-svgfeoffsetelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①③"></a>

```text
interface SVGFEOffsetElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber dx;
  readonly attribute SVGAnimatedNumber dy;
};

SVGFEOffsetElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString③③"></a>

<a id="dom-svgfeoffsetelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④⑥"></a>

<a id="ref-for-elementdef-feoffset①⑤"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feOffset](#elementdef-feoffset) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⓪"></a>

<a id="dom-svgfeoffsetelement-dx"></a>`dx`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feoffset-dx③"></a>

<a id="ref-for-elementdef-feoffset①⑥"></a>

Corresponds to attribute [dx](#element-attrdef-feoffset-dx) on the given [feOffset](#elementdef-feoffset) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧①"></a>

<a id="dom-svgfeoffsetelement-dy"></a>`dy`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feoffset-dy③"></a>

<a id="ref-for-elementdef-feoffset①⑦"></a>

Corresponds to attribute [dy](#element-attrdef-feoffset-dy) on the given [feOffset](#elementdef-feoffset) element.

### <a id="InterfaceSVGFESpecularLightingElement"></a>Interface SVGFESpecularLightingElement

<a id="ref-for-elementdef-fespecularlighting①③"></a>

The <a id="svgfespecularlightingelement"></a>`SVGFESpecularLightingElement` interface corresponds to the [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-svgfespecularlightingelement"></a>

<a id="ref-for-InterfaceSVGElement②⓪"></a>

<a id="ref-for-InterfaceSVGAnimatedString③④"></a>

<a id="ref-for-dom-svgfespecularlightingelement-in1"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧②"></a>

<a id="ref-for-dom-svgfespecularlightingelement-surfacescale"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧③"></a>

<a id="ref-for-dom-svgfespecularlightingelement-specularconstant"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧④"></a>

<a id="ref-for-dom-svgfespecularlightingelement-specularexponent"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑤"></a>

<a id="ref-for-dom-svgfespecularlightingelement-kernelunitlengthx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑥"></a>

<a id="ref-for-dom-svgfespecularlightingelement-kernelunitlengthy"></a>

<a id="ref-for-svgfespecularlightingelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①④"></a>

```text
interface SVGFESpecularLightingElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber surfaceScale;
  readonly attribute SVGAnimatedNumber specularConstant;
  readonly attribute SVGAnimatedNumber specularExponent;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
};

SVGFESpecularLightingElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString③⑤"></a>

<a id="dom-svgfespecularlightingelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④⑦"></a>

<a id="ref-for-elementdef-fespecularlighting①④"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑦"></a>

<a id="dom-svgfespecularlightingelement-surfacescale"></a>`surfaceScale`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespecularlighting-surfacescale②"></a>

<a id="ref-for-elementdef-fespecularlighting①⑤"></a>

Corresponds to attribute [surfaceScale](#element-attrdef-fespecularlighting-surfacescale) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑧"></a>

<a id="dom-svgfespecularlightingelement-specularconstant"></a>`specularConstant`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespecularlighting-specularconstant②"></a>

<a id="ref-for-elementdef-fespecularlighting①⑥"></a>

Corresponds to attribute [specularConstant](#element-attrdef-fespecularlighting-specularconstant) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑨"></a>

<a id="dom-svgfespecularlightingelement-specularexponent"></a>`specularExponent`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespecularlighting-specularexponent③"></a>

<a id="ref-for-elementdef-fespecularlighting①⑦"></a>

Corresponds to attribute [specularExponent](#element-attrdef-fespecularlighting-specularexponent) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑨⓪"></a>

<a id="dom-svgfespecularlightingelement-kernelunitlengthx"></a>`kernelUnitLengthX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength④"></a>

<a id="ref-for-elementdef-fespecularlighting①⑧"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-fespecularlighting-kernelunitlength) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑨①"></a>

<a id="dom-svgfespecularlightingelement-kernelunitlengthy"></a>`kernelUnitLengthY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-fespecularlighting-kernelunitlength⑤"></a>

<a id="ref-for-elementdef-fespecularlighting①⑨"></a>

Corresponds to attribute [kernelUnitLength](#element-attrdef-fespecularlighting-kernelunitlength) on the given [feSpecularLighting](#elementdef-fespecularlighting) element.

### <a id="InterfaceSVGFETileElement"></a>Interface SVGFETileElement

<a id="ref-for-elementdef-fetile①①"></a>

The <a id="svgfetileelement"></a>`SVGFETileElement` interface corresponds to the [feTile](#elementdef-fetile) element.

<a id="ref-for-svgfetileelement"></a>

<a id="ref-for-InterfaceSVGElement②①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⑥"></a>

<a id="ref-for-dom-svgfetileelement-in1"></a>

<a id="ref-for-svgfetileelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑤"></a>

```text
interface SVGFETileElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};

SVGFETileElement includes SVGFilterPrimitiveStandardAttributes;
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedString③⑦"></a>

<a id="dom-svgfetileelement-in1"></a>`in1`, of type [SVGAnimatedString](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedString), readonly

<a id="ref-for-element-attrdef-filter-primitive-in④⑧"></a>

<a id="ref-for-elementdef-fetile①②"></a>

Corresponds to attribute [in](#element-attrdef-filter-primitive-in) on the given [feTile](#elementdef-fetile) element.

### <a id="InterfaceSVGFETurbulenceElement"></a>Interface SVGFETurbulenceElement

<a id="ref-for-elementdef-feturbulence⑤"></a>

The <a id="svgfeturbulenceelement"></a>`SVGFETurbulenceElement` interface corresponds to the [feTurbulence](#elementdef-feturbulence) element.

<a id="ref-for-svgfeturbulenceelement"></a>

<a id="ref-for-InterfaceSVGElement②②"></a>

<a id="ref-for-idl-unsigned-short⑤①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_unknown"></a>

<a id="ref-for-idl-unsigned-short⑤②"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_fractalnoise"></a>

<a id="ref-for-idl-unsigned-short⑤③"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_turbulence"></a>

<a id="ref-for-idl-unsigned-short⑤④"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_unknown"></a>

<a id="ref-for-idl-unsigned-short⑤⑤"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_stitch"></a>

<a id="ref-for-idl-unsigned-short⑤⑥"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_nostitch"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨②"></a>

<a id="ref-for-dom-svgfeturbulenceelement-basefrequencyx"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨③"></a>

<a id="ref-for-dom-svgfeturbulenceelement-basefrequencyy"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger⑧"></a>

<a id="ref-for-dom-svgfeturbulenceelement-numoctaves"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨④"></a>

<a id="ref-for-dom-svgfeturbulenceelement-seed"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②②"></a>

<a id="ref-for-dom-svgfeturbulenceelement-stitchtiles"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②③"></a>

<a id="ref-for-dom-svgfeturbulenceelement-type"></a>

<a id="ref-for-svgfeturbulenceelement①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑥"></a>

```text
interface SVGFETurbulenceElement : SVGElement {

  // Turbulence Types
  const unsigned short SVG_TURBULENCE_TYPE_UNKNOWN = 0;
  const unsigned short SVG_TURBULENCE_TYPE_FRACTALNOISE = 1;
  const unsigned short SVG_TURBULENCE_TYPE_TURBULENCE = 2;

  // Stitch Options
  const unsigned short SVG_STITCHTYPE_UNKNOWN = 0;
  const unsigned short SVG_STITCHTYPE_STITCH = 1;
  const unsigned short SVG_STITCHTYPE_NOSTITCH = 2;

  readonly attribute SVGAnimatedNumber baseFrequencyX;
  readonly attribute SVGAnimatedNumber baseFrequencyY;
  readonly attribute SVGAnimatedInteger numOctaves;
  readonly attribute SVGAnimatedNumber seed;
  readonly attribute SVGAnimatedEnumeration stitchTiles;
  readonly attribute SVGAnimatedEnumeration type;
};

SVGFETurbulenceElement includes SVGFilterPrimitiveStandardAttributes;
```
Constants in group “Turbulence Types”:  
<a id="dom-svgfeturbulenceelement-svg_turbulence_type_unknown"></a>`SVG_TURBULENCE_TYPE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfeturbulenceelement-svg_turbulence_type_fractalnoise"></a>`SVG_TURBULENCE_TYPE_FRACTALNOISE`  
Corresponds to value fractalNoise.

<a id="dom-svgfeturbulenceelement-svg_turbulence_type_turbulence"></a>`SVG_TURBULENCE_TYPE_TURBULENCE`  
Corresponds to value turbulence.

Constants in group “Stitch Options”:  
<a id="dom-svgfeturbulenceelement-svg_stitchtype_unknown"></a>`SVG_STITCHTYPE_UNKNOWN`  
The type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="dom-svgfeturbulenceelement-svg_stitchtype_stitch"></a>`SVG_STITCHTYPE_STITCH`  
Corresponds to value stitch.

<a id="dom-svgfeturbulenceelement-svg_stitchtype_nostitch"></a>`SVG_STITCHTYPE_NOSTITCH`  
Corresponds to value noStitch.

Attributes:

<a id="ref-for-InterfaceSVGAnimatedNumber⑨⑤"></a>

<a id="dom-svgfeturbulenceelement-basefrequencyx"></a>`baseFrequencyX`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feturbulence-basefrequency②"></a>

<a id="ref-for-elementdef-feturbulence⑥"></a>

Corresponds to attribute [baseFrequency](#element-attrdef-feturbulence-basefrequency) on the given [feTurbulence](#elementdef-feturbulence) element. Contains the X component of the <a id="ref-for-element-attrdef-feturbulence-basefrequency③"></a>baseFrequency attribute.

<a id="ref-for-InterfaceSVGAnimatedNumber⑨⑥"></a>

<a id="dom-svgfeturbulenceelement-basefrequencyy"></a>`baseFrequencyY`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feturbulence-basefrequency④"></a>

<a id="ref-for-elementdef-feturbulence⑦"></a>

Corresponds to attribute [baseFrequency](#element-attrdef-feturbulence-basefrequency) on the given [feTurbulence](#elementdef-feturbulence) element. Contains the Y component of the <a id="ref-for-element-attrdef-feturbulence-basefrequency⑤"></a>baseFrequency attribute.

<a id="ref-for-InterfaceSVGAnimatedInteger⑨"></a>

<a id="dom-svgfeturbulenceelement-numoctaves"></a>`numOctaves`, of type [SVGAnimatedInteger](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedInteger), readonly

<a id="ref-for-element-attrdef-feturbulence-numoctaves②"></a>

<a id="ref-for-elementdef-feturbulence⑧"></a>

Corresponds to attribute [numOctaves](#element-attrdef-feturbulence-numoctaves) on the given [feTurbulence](#elementdef-feturbulence) element.

<a id="ref-for-InterfaceSVGAnimatedNumber⑨⑦"></a>

<a id="dom-svgfeturbulenceelement-seed"></a>`seed`, of type [SVGAnimatedNumber](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedNumber), readonly

<a id="ref-for-element-attrdef-feturbulence-seed③"></a>

<a id="ref-for-elementdef-feturbulence⑨"></a>

Corresponds to attribute [seed](#element-attrdef-feturbulence-seed) on the given [feTurbulence](#elementdef-feturbulence) element.

<a id="ref-for-InterfaceSVGAnimatedEnumeration②④"></a>

<a id="dom-svgfeturbulenceelement-stitchtiles"></a>`stitchTiles`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-feturbulence-stitchtiles②"></a>

<a id="ref-for-elementdef-feturbulence①⓪"></a>

Corresponds to attribute [stitchTiles](#element-attrdef-feturbulence-stitchtiles) on the given [feTurbulence](#elementdef-feturbulence) element. Takes one of the SVG_TURBULENCE_TYPE\_\* constants defined on this interface.

<a id="ref-for-InterfaceSVGAnimatedEnumeration②⑤"></a>

<a id="dom-svgfeturbulenceelement-type"></a>`type`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/svg2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-feturbulence-type②"></a>

<a id="ref-for-elementdef-feturbulence①①"></a>

Corresponds to attribute [type](#element-attrdef-feturbulence-type) on the given [feTurbulence](#elementdef-feturbulence) element. Takes one of the SVG_STITCHTYPE\_\* constants defined on this interface.

## <a id="changes"></a>Changes

The following significant changes were made since the [25 November 2014 Working Draft](https://www.w3.org/TR/2014/WD-filter-effects-1-20141125/).

- Editorial changes.

- Add description elements to content model of all elements.

- Clarify that crossorigin is not animatable.

- <a id="ref-for-funcdef-filter-hue-rotate②"></a>

  [\<hue-rotate()\>](#funcdef-filter-hue-rotate) takes unitless zero.

- <a id="ref-for-valuea-def-color⑥"></a>

  <a id="ref-for-length-value④"></a>

  <a id="ref-for-funcdef-filter-drop-shadow⑥"></a>

  Allow author to change order of [\<color\>](https://www.w3.org/TR/css3-color/#valuea-def-color) and [\<length\>](https://www.w3.org/TR/css3-values/#length-value) values of [\<drop-shadow()\>](#funcdef-filter-drop-shadow). Change grammar to put <a id="ref-for-valuea-def-color⑦"></a>\<color\> first. Differentiate between initial value for interpolation and default values for omitted values.

- Define animation type of CSS properties.

- <a id="ref-for-elementdef-fecolormatrix①①"></a>

  <a id="ref-for-elementdef-feconvolvematrix①⑧"></a>

  Make [feColorMatrix](#elementdef-fecolormatrix) and [feConvolveMatrix](#elementdef-feconvolvematrix) a pass through on unfulfilled pre-conditions.

- <a id="ref-for-elementdef-feturbulence①②"></a>

  Make [feTurbulence](#elementdef-feturbulence) algorithms respect uniformity.

- Apply properties that apply to all graphics elements to the use element as well.

- <a id="ref-for-funcdef-filter-saturate②"></a>

  Corrected filter primitive representation of [\<saturate()\>](#funcdef-filter-saturate).

- Extend SVG DOM SVGFEBlendElement enumerations with new blend modes.

The following significant changes were made since the [26 November 2013 Working Draft](https://www.w3.org/TR/2013/WD-filter-effects-1-20131126/).

- Removed Custom Filters.

- <a id="ref-for-elementdef-script②⑥"></a>

  Allow the [script](https://www.w3.org/TR/svg2/interact.html#elementdef-script) element in the content model everywhere.

- <a id="ref-for-elementdef-feblend⑨"></a>

  Support all blend modes from CSS Blending specification for [feBlend](#elementdef-feblend).

- <a id="ref-for-elementdef-fecomposite①⑨"></a>

  Support all non-duplicated compositing modes from CSS Blending specification for [feComposite](#elementdef-fecomposite).

- <a id="ref-for-elementdef-feblend①⓪"></a>

  Added no-composite attribute to [feBlend](#elementdef-feblend) to avoid double compositing.

- Corrections on shorthands syntax.

- Added definition for shorthand filter regions.

The following significant changes were made since the [25 October 2012 Working Draft](https://www.w3.org/TR/2012/WD-filter-effects-20121025/).

- Correction of brightness short hand filter.

- New syntax for Custom Filter function.

- Add at-function rule for Custom Filters.

- Allow Custom Filter function to be used as extension for future filter features.

- Remove unnecessary attributes and uniforms on shaders.

- Redefine origin of shader coordinate space to bottom left.

- Remove now unnecessary filter-margin properties.

See more detailed and longterm changes in the [ChangeLog](https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/ChangeLog).

## <a id="acknowledgments"></a>Acknowledgments

The editors would like to thank Robert O’Callahan, Coralie Mercier, Chris Lilley, Nikos Andronikos, Stephen Chenney, Simon Fraser, Tavmjong Bah, Robert Longson, Cameron McCormack, Brad Kemper, Tab Atkins, Brian Birtles, Michael Mullany, Rik Cabanier, Anne van Kesteren, Boris Zbarsky, Kristopher Giesing, Stephen White, Jasper van de Gronde, Kang-Hao Lu, Paul LeBeau, Debarshi Ray, Jarek Foksa, Sebastian Zartner, Yuqian Li, Amelia Bellamy-Royds and Max Vujovic for their careful reviews, comments, and corrections.

## <a id="conformance"></a> Conformance

### <a id="document-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conform-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="conform-responsible"></a> Requirements for Responsible Implementation of CSS

The following sections define several conformance requirements for implementing CSS responsibly, in a way that promotes interoperability in the present and future.

#### <a id="conform-partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid&#xA;&#x9;&#x9;(and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)&#xA;&#x9;&#x9;any at-rules, properties, property values, keywords, and other syntactic constructs&#xA;&#x9;&#x9;for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- amplitude
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-amplitude), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-amplitude), in §9.7.1
- [arithmetic](#attr-valuedef-operator-arithmetic), in §9.8
- [atop](#attr-valuedef-operator-atop), in §9.8
- [auto](#valdef-color-interpolation-filters-auto), in §10
- azimuth
  - [attribute for SVGFEDistantLightElement](#dom-svgfedistantlightelement-azimuth), in §Unnumbered section
  - [element-attr for feDistantLight](#element-attrdef-fedistantlight-azimuth), in §11.2
- [BackgroundAlpha](#attr-valuedef-in-backgroundalpha), in §9.2
- [BackgroundImage](#attr-valuedef-in-backgroundimage), in §9.2
- [baseFrequency](#element-attrdef-feturbulence-basefrequency), in §9.21
- [baseFrequencyX](#dom-svgfeturbulenceelement-basefrequencyx), in §Unnumbered section
- [baseFrequencyY](#dom-svgfeturbulenceelement-basefrequencyy), in §Unnumbered section
- bias
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-bias), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-bias), in §9.9
- [blur()](#funcdef-filter-blur), in §6.1
- [brightness()](#funcdef-filter-brightness), in §6.1
- [color-interpolation-filters](#propdef-color-interpolation-filters), in §10
- [contrast()](#funcdef-filter-contrast), in §6.1
- [crossOrigin](#dom-svgfeimageelement-crossorigin), in §Unnumbered section
- [crossorigin](#element-attrdef-feimage-crossorigin), in §9.15
- diffuseConstant
  - [attribute for SVGFEDiffuseLightingElement](#dom-svgfediffuselightingelement-diffuseconstant), in §Unnumbered section
  - [element-attr for feDiffuseLighting](#element-attrdef-fediffuselighting-diffuseconstant), in §9.10
- [discrete](#attr-valuedef-type-discrete), in §9.7.1
- divisor
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-divisor), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-divisor), in §9.9
- [drop-shadow()](#funcdef-filter-drop-shadow), in §6.1
- [duplicate](#attr-valuedef-edgemode-duplicate), in §9.14
- dx
  - [attribute for SVGFEDropShadowElement](#dom-svgfedropshadowelement-dx), in §Unnumbered section
  - [attribute for SVGFEOffsetElement](#dom-svgfeoffsetelement-dx), in §Unnumbered section
  - [element-attr for feDropShadow](#element-attrdef-fedropshadow-dx), in §9.12
  - [element-attr for feOffset](#element-attrdef-feoffset-dx), in §9.18
- dy
  - [attribute for SVGFEDropShadowElement](#dom-svgfedropshadowelement-dy), in §Unnumbered section
  - [attribute for SVGFEOffsetElement](#dom-svgfeoffsetelement-dy), in §Unnumbered section
  - [element-attr for feDropShadow](#element-attrdef-fedropshadow-dy), in §9.12
  - [element-attr for feOffset](#element-attrdef-feoffset-dy), in §9.18
- edgeMode
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-edgemode), in §Unnumbered section
  - [attribute for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-edgemode), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-edgemode), in §9.9
  - [element-attr for feGaussianBlur](#element-attrdef-fegaussianblur-edgemode), in §9.14
- elevation
  - [attribute for SVGFEDistantLightElement](#dom-svgfedistantlightelement-elevation), in §Unnumbered section
  - [element-attr for feDistantLight](#element-attrdef-fedistantlight-elevation), in §11.2
- exponent
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-exponent), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-exponent), in §9.7.1
- [feBlend](#elementdef-feblend), in §9.5
- [feColorMatrix](#elementdef-fecolormatrix), in §9.6
- [feComponentTransfer](#elementdef-fecomponenttransfer), in §9.7
- [feComposite](#elementdef-fecomposite), in §9.8
- [feConvolveMatrix](#elementdef-feconvolvematrix), in §9.9
- [feDiffuseLighting](#elementdef-fediffuselighting), in §9.10
- [feDisplacementMap](#elementdef-fedisplacementmap), in §9.11
- [feDistantLight](#elementdef-fedistantlight), in §11.2
- [feDropShadow](#elementdef-fedropshadow), in §9.12
- [feFlood](#elementdef-feflood), in §9.13
- [feFuncA](#elementdef-fefunca), in §9.7.4
- [feFuncB](#elementdef-fefuncb), in §9.7.3
- [feFuncG](#elementdef-fefuncg), in §9.7.2
- [feFuncR](#elementdef-fefuncr), in §9.7.1
- [feGaussianBlur](#elementdef-fegaussianblur), in §9.14
- [feImage](#elementdef-feimage), in §9.15
- [feMerge](#elementdef-femerge), in §9.16
- [feMergeNode](#elementdef-femergenode), in §9.16.1
- [feMorphology](#elementdef-femorphology), in §9.17
- [feOffset](#elementdef-feoffset), in §9.18
- [fePointLight](#elementdef-fepointlight), in §11.3
- [feSpecularLighting](#elementdef-fespecularlighting), in §9.19
- [feSpotLight](#elementdef-fespotlight), in §11.4
- [feTile](#elementdef-fetile), in §9.20
- [feTurbulence](#elementdef-feturbulence), in §9.21
- [FillPaint](#attr-valuedef-in-fillpaint), in §9.2
- filter
  - [(element)](#elementdef-filter), in §7
  - [(property)](#propdef-filter), in §5
- [filter()](#funcdef-filter), in §12
- [\<filter-function\>](#typedef-filter-function), in §6.1
- [filter primitive](#filter-primitive), in §4
- [filter-primitive](#elementdef-filter-primitive), in §4
- [filter primitive attributes](#filter-primitive-attributes), in §9.2
- [\<filter-primitive-reference\>](#typedef-result-filter-primitive-reference), in §9.2
- [filter primitive subregion](#filter-primitive-subregion), in §9.4
- [filter primitive tree](#filter-primitive-tree), in §9.3
- [filter region](#filter-region), in §8
- [filterRes](#element-attrdef-filter-filterres), in §7
- filterUnits
  - [attribute for SVGFilterElement](#dom-svgfilterelement-filterunits), in §Unnumbered section
  - [element-attr for filter](#element-attrdef-filter-filterunits), in §7
- [\<filter-value-list\>](#typedef-filter-value-list), in §5
- [flood-color](#propdef-flood-color), in §9.13.1
- [flood-opacity](#propdef-flood-opacity), in §9.13.2
- [gamma](#attr-valuedef-type-gamma), in §9.7.1
- [grayscale()](#funcdef-filter-grayscale), in §6.1
- height
  - [attribute for SVGFilterElement](#dom-svgfilterelement-height), in §Unnumbered section
  - [attribute for SVGFilterPrimitiveStandardAttributes](#dom-svgfilterprimitivestandardattributes-height), in §Unnumbered section
  - [element-attr for filter](#element-attrdef-filter-height), in §7
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-height), in §9.2
- [href](#element-attrdef-feimage-href), in §9.15
- [hueRotate](#attr-valuedef-type-huerotate), in §9.6
- [hue-rotate()](#funcdef-filter-hue-rotate), in §6.1
- [identity](#attr-valuedef-type-identity), in §9.7.1
- in
  - [attr-value for operator](#attr-valuedef-operator-in), in §9.8
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-in), in §9.2
- in1
  - [attribute for SVGFEBlendElement](#dom-svgfeblendelement-in1), in §Unnumbered section
  - [attribute for SVGFEColorMatrixElement](#dom-svgfecolormatrixelement-in1), in §Unnumbered section
  - [attribute for SVGFEComponentTransferElement](#dom-svgfecomponenttransferelement-in1), in §Unnumbered section
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-in1), in §Unnumbered section
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-in1), in §Unnumbered section
  - [attribute for SVGFEDiffuseLightingElement](#dom-svgfediffuselightingelement-in1), in §Unnumbered section
  - [attribute for SVGFEDisplacementMapElement](#dom-svgfedisplacementmapelement-in1), in §Unnumbered section
  - [attribute for SVGFEDropShadowElement](#dom-svgfedropshadowelement-in1), in §Unnumbered section
  - [attribute for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-in1), in §Unnumbered section
  - [attribute for SVGFEMergeNodeElement](#dom-svgfemergenodeelement-in1), in §Unnumbered section
  - [attribute for SVGFEMorphologyElement](#dom-svgfemorphologyelement-in1), in §Unnumbered section
  - [attribute for SVGFEOffsetElement](#dom-svgfeoffsetelement-in1), in §Unnumbered section
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-in1), in §Unnumbered section
  - [attribute for SVGFETileElement](#dom-svgfetileelement-in1), in §Unnumbered section
- in2
  - [attribute for SVGFEBlendElement](#dom-svgfeblendelement-in2), in §Unnumbered section
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-in2), in §Unnumbered section
  - [attribute for SVGFEDisplacementMapElement](#dom-svgfedisplacementmapelement-in2), in §Unnumbered section
  - [element-attr for feBlend](#element-attrdef-feblend-in2), in §9.5
  - [element-attr for feComposite](#element-attrdef-fecomposite-in2), in §9.8
  - [element-attr for feDisplacementMap](#element-attrdef-fedisplacementmap-in2), in §9.11
- intercept
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-intercept), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-intercept), in §9.7.1
- [invert()](#funcdef-filter-invert), in §6.1
- k1
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-k1), in §Unnumbered section
  - [element-attr for feComposite](#element-attrdef-fecomposite-k1), in §9.8
- k2
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-k2), in §Unnumbered section
  - [element-attr for feComposite](#element-attrdef-fecomposite-k2), in §9.8
- k3
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-k3), in §Unnumbered section
  - [element-attr for feComposite](#element-attrdef-fecomposite-k3), in §9.8
- k4
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-k4), in §Unnumbered section
  - [element-attr for feComposite](#element-attrdef-fecomposite-k4), in §9.8
- kernelMatrix
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-kernelmatrix), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-kernelmatrix), in §9.9
- kernelUnitLength
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-kernelunitlength), in §9.9
  - [element-attr for feDiffuseLighting](#element-attrdef-fediffuselighting-kernelunitlength), in §9.10
  - [element-attr for feSpecularLighting](#element-attrdef-fespecularlighting-kernelunitlength), in §9.19
- kernelUnitLengthX
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-kernelunitlengthx), in §Unnumbered section
  - [attribute for SVGFEDiffuseLightingElement](#dom-svgfediffuselightingelement-kernelunitlengthx), in §Unnumbered section
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-kernelunitlengthx), in §Unnumbered section
- kernelUnitLengthY
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-kernelunitlengthy), in §Unnumbered section
  - [attribute for SVGFEDiffuseLightingElement](#dom-svgfediffuselightingelement-kernelunitlengthy), in §Unnumbered section
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-kernelunitlengthy), in §Unnumbered section
- [lighter](#attr-valuedef-operator-lighter), in §9.8
- [lighting-color](#propdef-lighting-color), in §11.5
- [light source](#light-source), in §11.1
- limitingConeAngle
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-limitingconeangle), in §Unnumbered section
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-limitingconeangle), in §11.4
- [linear](#attr-valuedef-type-linear), in §9.7.1
- [linearRGB](#valdef-color-interpolation-filters-linearrgb), in §10
- [luminanceToAlpha](#attr-valuedef-type-luminancetoalpha), in §9.6
- [matrix](#attr-valuedef-type-matrix), in §9.6
- mode
  - [attribute for SVGFEBlendElement](#dom-svgfeblendelement-mode), in §Unnumbered section
  - [element-attr for feBlend](#element-attrdef-feblend-mode), in §9.5
- no-composite
  - [attr-value for no-composite](#attr-valuedef-no-composite-no-composite), in §9.5
  - [element-attr for feBlend](#element-attrdef-feblend-no-composite), in §9.5
- [\<number-optional-number\>](#typedef-number-optional-number), in §7
- numOctaves
  - [attribute for SVGFETurbulenceElement](#dom-svgfeturbulenceelement-numoctaves), in §Unnumbered section
  - [element-attr for feTurbulence](#element-attrdef-feturbulence-numoctaves), in §9.21
- objectBoundingBox
  - [attr-value for filterUnits](#attr-valuedef-filterunits-objectboundingbox), in §7
  - [attr-value for primitiveUnits](#attr-valuedef-primitiveunits-objectboundingbox), in §7
- offset
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-offset), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-offset), in §9.7.1
- [opacity()](#funcdef-filter-opacity), in §6.1
- [operating coordinate space](#operating-coordinate-space), in §9.1
- operator
  - [attribute for SVGFECompositeElement](#dom-svgfecompositeelement-operator), in §Unnumbered section
  - [attribute for SVGFEMorphologyElement](#dom-svgfemorphologyelement-operator), in §Unnumbered section
  - [element-attr for feComposite](#element-attrdef-fecomposite-operator), in §9.8
  - [element-attr for feMorphology](#element-attrdef-femorphology-operator), in §9.17
- [order](#element-attrdef-order), in §9.9
- [orderX](#dom-svgfeconvolvematrixelement-orderx), in §Unnumbered section
- [orderY](#dom-svgfeconvolvematrixelement-ordery), in §Unnumbered section
- [out](#attr-valuedef-operator-out), in §9.8
- [over](#attr-valuedef-operator-over), in §9.8
- [pass through filter](#pass-through-filter), in §4
- pointsAtX
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-pointsatx), in §Unnumbered section
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-pointsatx), in §11.4
- pointsAtY
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-pointsaty), in §Unnumbered section
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-pointsaty), in §11.4
- pointsAtZ
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-pointsatz), in §Unnumbered section
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-pointsatz), in §11.4
- preserveAlpha
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-preservealpha), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-preservealpha), in §9.9
- preserveAspectRatio
  - [attribute for SVGFEImageElement](#dom-svgfeimageelement-preserveaspectratio), in §Unnumbered section
  - [element-attr for feImage](#element-attrdef-feimage-preserveaspectratio), in §9.15
- [primary filter primitive tree](#primary-filter-primitive-tree), in §9.3
- primitiveUnits
  - [attribute for SVGFilterElement](#dom-svgfilterelement-primitiveunits), in §Unnumbered section
  - [element-attr for filter](#element-attrdef-filter-primitiveunits), in §7
- [radius](#element-attrdef-femorphology-radius), in §9.17
- [radiusX](#dom-svgfemorphologyelement-radiusx), in §Unnumbered section
- [radiusY](#dom-svgfemorphologyelement-radiusy), in §Unnumbered section
- result
  - [attribute for SVGFilterPrimitiveStandardAttributes](#dom-svgfilterprimitivestandardattributes-result), in §Unnumbered section
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-result), in §9.2
- [saturate()](#funcdef-filter-saturate), in §6.1
- [saturate](#attr-valuedef-type-saturate), in §9.6
- scale
  - [attribute for SVGFEDisplacementMapElement](#dom-svgfedisplacementmapelement-scale), in §Unnumbered section
  - [element-attr for feDisplacementMap](#element-attrdef-fedisplacementmap-scale), in §9.11
- seed
  - [attribute for SVGFETurbulenceElement](#dom-svgfeturbulenceelement-seed), in §Unnumbered section
  - [element-attr for feTurbulence](#element-attrdef-feturbulence-seed), in §9.21
- [sepia()](#funcdef-filter-sepia), in §6.1
- setStdDeviation(stdDeviationX, stdDeviationY)
  - [method for SVGFEDropShadowElement](#dom-svgfedropshadowelement-setstddeviation), in §Unnumbered section
  - [method for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-setstddeviation), in §Unnumbered section
- slope
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-slope), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-slope), in §9.7.1
- [SourceAlpha](#attr-valuedef-in-sourcealpha), in §9.2
- [SourceGraphic](#attr-valuedef-in-sourcegraphic), in §9.2
- specularConstant
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-specularconstant), in §Unnumbered section
  - [element-attr for feSpecularLighting](#element-attrdef-fespecularlighting-specularconstant), in §9.19
- specularExponent
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-specularexponent), in §Unnumbered section
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-specularexponent), in §Unnumbered section
  - [element-attr for feSpecularLighting](#element-attrdef-fespecularlighting-specularexponent), in §9.19
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-specularexponent), in §11.4
- [sRGB](#valdef-color-interpolation-filters-srgb), in §10
- stdDeviation
  - [element-attr for feDropShadow](#element-attrdef-fedropshadow-stddeviation), in §9.12
  - [element-attr for feGaussianBlur](#element-attrdef-fegaussianblur-stddeviation), in §9.14
- stdDeviationX
  - [attribute for SVGFEDropShadowElement](#dom-svgfedropshadowelement-stddeviationx), in §Unnumbered section
  - [attribute for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-stddeviationx), in §Unnumbered section
- stdDeviationY
  - [attribute for SVGFEDropShadowElement](#dom-svgfedropshadowelement-stddeviationy), in §Unnumbered section
  - [attribute for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-stddeviationy), in §Unnumbered section
- stitchTiles
  - [attribute for SVGFETurbulenceElement](#dom-svgfeturbulenceelement-stitchtiles), in §Unnumbered section
  - [element-attr for feTurbulence](#element-attrdef-feturbulence-stitchtiles), in §9.21
- [StrokePaint](#attr-valuedef-in-strokepaint), in §9.2
- surfaceScale
  - [attribute for SVGFEDiffuseLightingElement](#dom-svgfediffuselightingelement-surfacescale), in §Unnumbered section
  - [attribute for SVGFESpecularLightingElement](#dom-svgfespecularlightingelement-surfacescale), in §Unnumbered section
  - [element-attr for feDiffuseLighting](#element-attrdef-fediffuselighting-surfacescale), in §9.10
  - [element-attr for feSpecularLighting](#element-attrdef-fespecularlighting-surfacescale), in §9.19
- [SVG_CHANNEL_A](#dom-svgfedisplacementmapelement-svg_channel_a), in §Unnumbered section
- [SVG_CHANNEL_B](#dom-svgfedisplacementmapelement-svg_channel_b), in §Unnumbered section
- [SVG_CHANNEL_G](#dom-svgfedisplacementmapelement-svg_channel_g), in §Unnumbered section
- [SVG_CHANNEL_R](#dom-svgfedisplacementmapelement-svg_channel_r), in §Unnumbered section
- [SVG_CHANNEL_UNKNOWN](#dom-svgfedisplacementmapelement-svg_channel_unknown), in §Unnumbered section
- [SVGComponentTransferFunctionElement](#svgcomponenttransferfunctionelement), in §Unnumbered section
- SVG_EDGEMODE_DUPLICATE
  - [const for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-svg_edgemode_duplicate), in §Unnumbered section
  - [const for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-svg_edgemode_duplicate), in §Unnumbered section
- SVG_EDGEMODE_NONE
  - [const for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-svg_edgemode_none), in §Unnumbered section
  - [const for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-svg_edgemode_none), in §Unnumbered section
- SVG_EDGEMODE_UNKNOWN
  - [const for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-svg_edgemode_unknown), in §Unnumbered section
  - [const for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-svg_edgemode_unknown), in §Unnumbered section
- SVG_EDGEMODE_WRAP
  - [const for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-svg_edgemode_wrap), in §Unnumbered section
  - [const for SVGFEGaussianBlurElement](#dom-svgfegaussianblurelement-svg_edgemode_wrap), in §Unnumbered section
- [SVGFEBlendElement](#svgfeblendelement), in §Unnumbered section
- [SVG_FEBLEND_MODE_COLOR](#dom-svgfeblendelement-svg_feblend_mode_color), in §Unnumbered section
- [SVG_FEBLEND_MODE_COLOR_BURN](#dom-svgfeblendelement-svg_feblend_mode_color_burn), in §Unnumbered section
- [SVG_FEBLEND_MODE_COLOR_DODGE](#dom-svgfeblendelement-svg_feblend_mode_color_dodge), in §Unnumbered section
- [SVG_FEBLEND_MODE_DARKEN](#dom-svgfeblendelement-svg_feblend_mode_darken), in §Unnumbered section
- [SVG_FEBLEND_MODE_DIFFERENCE](#dom-svgfeblendelement-svg_feblend_mode_difference), in §Unnumbered section
- [SVG_FEBLEND_MODE_EXCLUSION](#dom-svgfeblendelement-svg_feblend_mode_exclusion), in §Unnumbered section
- [SVG_FEBLEND_MODE_HARD_LIGHT](#dom-svgfeblendelement-svg_feblend_mode_hard_light), in §Unnumbered section
- [SVG_FEBLEND_MODE_HUE](#dom-svgfeblendelement-svg_feblend_mode_hue), in §Unnumbered section
- [SVG_FEBLEND_MODE_LIGHTEN](#dom-svgfeblendelement-svg_feblend_mode_lighten), in §Unnumbered section
- [SVG_FEBLEND_MODE_LUMINOSITY](#dom-svgfeblendelement-svg_feblend_mode_luminosity), in §Unnumbered section
- [SVG_FEBLEND_MODE_MULTIPLY](#dom-svgfeblendelement-svg_feblend_mode_multiply), in §Unnumbered section
- [SVG_FEBLEND_MODE_NORMAL](#dom-svgfeblendelement-svg_feblend_mode_normal), in §Unnumbered section
- [SVG_FEBLEND_MODE_OVERLAY](#dom-svgfeblendelement-svg_feblend_mode_overlay), in §Unnumbered section
- [SVG_FEBLEND_MODE_SATURATION](#dom-svgfeblendelement-svg_feblend_mode_saturation), in §Unnumbered section
- [SVG_FEBLEND_MODE_SCREEN](#dom-svgfeblendelement-svg_feblend_mode_screen), in §Unnumbered section
- [SVG_FEBLEND_MODE_SOFT_LIGHT](#dom-svgfeblendelement-svg_feblend_mode_soft_light), in §Unnumbered section
- [SVG_FEBLEND_MODE_UNKNOWN](#dom-svgfeblendelement-svg_feblend_mode_unknown), in §Unnumbered section
- [SVGFEColorMatrixElement](#svgfecolormatrixelement), in §Unnumbered section
- [SVG_FECOLORMATRIX_TYPE_HUEROTATE](#dom-svgfecolormatrixelement-svg_fecolormatrix_type_huerotate), in §Unnumbered section
- [SVG_FECOLORMATRIX_TYPE_LUMINANCETOALPHA](#dom-svgfecolormatrixelement-svg_fecolormatrix_type_luminancetoalpha), in §Unnumbered section
- [SVG_FECOLORMATRIX_TYPE_MATRIX](#dom-svgfecolormatrixelement-svg_fecolormatrix_type_matrix), in §Unnumbered section
- [SVG_FECOLORMATRIX_TYPE_SATURATE](#dom-svgfecolormatrixelement-svg_fecolormatrix_type_saturate), in §Unnumbered section
- [SVG_FECOLORMATRIX_TYPE_UNKNOWN](#dom-svgfecolormatrixelement-svg_fecolormatrix_type_unknown), in §Unnumbered section
- [SVGFEComponentTransferElement](#svgfecomponenttransferelement), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_discrete), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_GAMMA](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_gamma), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_identity), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_LINEAR](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_linear), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_TABLE](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_table), in §Unnumbered section
- [SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN](#dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_unknown), in §Unnumbered section
- [SVGFECompositeElement](#svgfecompositeelement), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_ARITHMETIC](#dom-svgfecompositeelement-svg_fecomposite_operator_arithmetic), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_ATOP](#dom-svgfecompositeelement-svg_fecomposite_operator_atop), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_IN](#dom-svgfecompositeelement-svg_fecomposite_operator_in), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_OUT](#dom-svgfecompositeelement-svg_fecomposite_operator_out), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_OVER](#dom-svgfecompositeelement-svg_fecomposite_operator_over), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_UNKNOWN](#dom-svgfecompositeelement-svg_fecomposite_operator_unknown), in §Unnumbered section
- [SVG_FECOMPOSITE_OPERATOR_XOR](#dom-svgfecompositeelement-svg_fecomposite_operator_xor), in §Unnumbered section
- [SVGFEConvolveMatrixElement](#svgfeconvolvematrixelement), in §Unnumbered section
- [SVGFEDiffuseLightingElement](#svgfediffuselightingelement), in §Unnumbered section
- [SVGFEDisplacementMapElement](#svgfedisplacementmapelement), in §Unnumbered section
- [SVGFEDistantLightElement](#svgfedistantlightelement), in §Unnumbered section
- [SVGFEDropShadowElement](#svgfedropshadowelement), in §Unnumbered section
- [SVGFEFloodElement](#svgfefloodelement), in §Unnumbered section
- [SVGFEFuncAElement](#svgfefuncaelement), in §Unnumbered section
- [SVGFEFuncBElement](#svgfefuncbelement), in §Unnumbered section
- [SVGFEFuncGElement](#svgfefuncgelement), in §Unnumbered section
- [SVGFEFuncRElement](#svgfefuncrelement), in §Unnumbered section
- [SVGFEGaussianBlurElement](#svgfegaussianblurelement), in §Unnumbered section
- [SVGFEImageElement](#svgfeimageelement), in §Unnumbered section
- [SVGFEMergeElement](#svgfemergeelement), in §Unnumbered section
- [SVGFEMergeNodeElement](#svgfemergenodeelement), in §Unnumbered section
- [SVGFEMorphologyElement](#svgfemorphologyelement), in §Unnumbered section
- [SVGFEOffsetElement](#svgfeoffsetelement), in §Unnumbered section
- [SVGFEPointLightElement](#svgfepointlightelement), in §Unnumbered section
- [SVGFESpecularLightingElement](#svgfespecularlightingelement), in §Unnumbered section
- [SVGFESpotLightElement](#svgfespotlightelement), in §Unnumbered section
- [SVGFETileElement](#svgfetileelement), in §Unnumbered section
- [SVGFETurbulenceElement](#svgfeturbulenceelement), in §Unnumbered section
- [SVGFilterElement](#svgfilterelement), in §Unnumbered section
- [SVGFilterPrimitiveStandardAttributes](#svgfilterprimitivestandardattributes), in §Unnumbered section
- [SVG_MORPHOLOGY_OPERATOR_DILATE](#dom-svgfemorphologyelement-svg_morphology_operator_dilate), in §Unnumbered section
- [SVG_MORPHOLOGY_OPERATOR_ERODE](#dom-svgfemorphologyelement-svg_morphology_operator_erode), in §Unnumbered section
- [SVG_MORPHOLOGY_OPERATOR_UNKNOWN](#dom-svgfemorphologyelement-svg_morphology_operator_unknown), in §Unnumbered section
- [SVG_STITCHTYPE_NOSTITCH](#dom-svgfeturbulenceelement-svg_stitchtype_nostitch), in §Unnumbered section
- [SVG_STITCHTYPE_STITCH](#dom-svgfeturbulenceelement-svg_stitchtype_stitch), in §Unnumbered section
- [SVG_STITCHTYPE_UNKNOWN](#dom-svgfeturbulenceelement-svg_stitchtype_unknown), in §Unnumbered section
- [SVG_TURBULENCE_TYPE_FRACTALNOISE](#dom-svgfeturbulenceelement-svg_turbulence_type_fractalnoise), in §Unnumbered section
- [SVG_TURBULENCE_TYPE_TURBULENCE](#dom-svgfeturbulenceelement-svg_turbulence_type_turbulence), in §Unnumbered section
- [SVG_TURBULENCE_TYPE_UNKNOWN](#dom-svgfeturbulenceelement-svg_turbulence_type_unknown), in §Unnumbered section
- [table](#attr-valuedef-type-table), in §9.7.1
- tableValues
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-tablevalues), in §Unnumbered section
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-tablevalues), in §9.7.1
- targetX
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-targetx), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-targetx), in §9.9
- targetY
  - [attribute for SVGFEConvolveMatrixElement](#dom-svgfeconvolvematrixelement-targety), in §Unnumbered section
  - [element-attr for feConvolveMatrix](#element-attrdef-feconvolvematrix-targety), in §9.9
- [transfer function element](#transfer-function-element), in §9.7
- [transfer function element attributes](#transfer-function-element-attributes), in §9.7.1
- type
  - [attribute for SVGComponentTransferFunctionElement](#dom-svgcomponenttransferfunctionelement-type), in §Unnumbered section
  - [attribute for SVGFEColorMatrixElement](#dom-svgfecolormatrixelement-type), in §Unnumbered section
  - [attribute for SVGFETurbulenceElement](#dom-svgfeturbulenceelement-type), in §Unnumbered section
  - [element-attr for feColorMatrix](#element-attrdef-fecolormatrix-type), in §9.6
  - [element-attr for feComponentTransfer](#element-attrdef-fecomponenttransfer-type), in §9.7.1
  - [element-attr for feTurbulence](#element-attrdef-feturbulence-type), in §9.21
- [\<url\>](#typedef-filter-url), in §5
- userSpaceOnUse
  - [attr-value for filterUnits](#attr-valuedef-filterunits-userspaceonuse), in §7
  - [attr-value for primitiveUnits](#attr-valuedef-primitiveunits-userspaceonuse), in §7
- values
  - [attribute for SVGFEColorMatrixElement](#dom-svgfecolormatrixelement-values), in §Unnumbered section
  - [element-attr for feColorMatrix](#element-attrdef-fecolormatrix-values), in §9.6
- width
  - [attribute for SVGFilterElement](#dom-svgfilterelement-width), in §Unnumbered section
  - [attribute for SVGFilterPrimitiveStandardAttributes](#dom-svgfilterprimitivestandardattributes-width), in §Unnumbered section
  - [element-attr for filter](#element-attrdef-filter-width), in §7
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-width), in §9.2
- [wrap](#attr-valuedef-edgemode-wrap), in §9.14
- x
  - [attribute for SVGFEPointLightElement](#dom-svgfepointlightelement-x), in §Unnumbered section
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-x), in §Unnumbered section
  - [attribute for SVGFilterElement](#dom-svgfilterelement-x), in §Unnumbered section
  - [attribute for SVGFilterPrimitiveStandardAttributes](#dom-svgfilterprimitivestandardattributes-x), in §Unnumbered section
  - [element-attr for fePointLight](#element-attrdef-fepointlight-x), in §11.3
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-x), in §11.4
  - [element-attr for filter](#element-attrdef-filter-x), in §7
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-x), in §9.2
- xChannelSelector
  - [attribute for SVGFEDisplacementMapElement](#dom-svgfedisplacementmapelement-xchannelselector), in §Unnumbered section
  - [element-attr for feDisplacementMap](#element-attrdef-fedisplacementmap-xchannelselector), in §9.11
- [xlink:href](#element-attrdef-feimage-xlinkhref), in §9.15
- [xor](#attr-valuedef-operator-xor), in §9.8
- y
  - [attribute for SVGFEPointLightElement](#dom-svgfepointlightelement-y), in §Unnumbered section
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-y), in §Unnumbered section
  - [attribute for SVGFilterElement](#dom-svgfilterelement-y), in §Unnumbered section
  - [attribute for SVGFilterPrimitiveStandardAttributes](#dom-svgfilterprimitivestandardattributes-y), in §Unnumbered section
  - [element-attr for fePointLight](#element-attrdef-fepointlight-y), in §11.3
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-y), in §11.4
  - [element-attr for filter](#element-attrdef-filter-y), in §7
  - [element-attr for filter-primitive](#element-attrdef-filter-primitive-y), in §9.2
- yChannelSelector
  - [attribute for SVGFEDisplacementMapElement](#dom-svgfedisplacementmapelement-ychannelselector), in §Unnumbered section
  - [element-attr for feDisplacementMap](#element-attrdef-fedisplacementmap-ychannelselector), in §9.11
- z
  - [attribute for SVGFEPointLightElement](#dom-svgfepointlightelement-z), in §Unnumbered section
  - [attribute for SVGFESpotLightElement](#dom-svgfespotlightelement-z), in §Unnumbered section
  - [element-attr for fePointLight](#element-attrdef-fepointlight-z), in §11.3
  - [element-attr for feSpotLight](#element-attrdef-fespotlight-z), in §11.4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[COMPOSITING-1\] defines the following terms:
  - <a id="term-for-ltblendmodegt"></a>\<blend-mode\>
  - <a id="term-for-backdrop"></a>backdrop
  - <a id="term-for-valdef-blend-mode-color"></a>color
  - <a id="term-for-valdef-blend-mode-color-burn"></a>color-burn
  - <a id="term-for-valdef-blend-mode-color-dodge"></a>color-dodge
  - <a id="term-for-valdef-blend-mode-darken"></a>darken
  - <a id="term-for-valdef-blend-mode-difference"></a>difference
  - <a id="term-for-valdef-blend-mode-exclusion"></a>exclusion
  - <a id="term-for-valdef-blend-mode-hard-light"></a>hard-light
  - <a id="term-for-valdef-blend-mode-hue"></a>hue
  - <a id="term-for-propdef-isolation"></a>isolation
  - <a id="term-for-valdef-blend-mode-lighten"></a>lighten
  - <a id="term-for-valdef-blend-mode-luminosity"></a>luminosity
  - <a id="term-for-valdef-blend-mode-multiply"></a>multiply
  - <a id="term-for-valdef-blend-mode-normal"></a>normal
  - <a id="term-for-valdef-blend-mode-overlay"></a>overlay
  - <a id="term-for-valdef-blend-mode-saturation"></a>saturation
  - <a id="term-for-valdef-blend-mode-screen"></a>screen
  - <a id="term-for-valdef-blend-mode-soft-light"></a>soft-light
- \[css-cascade-4\] defines the following terms:
  - <a id="term-for-specified-value"></a>specified value
  - <a id="term-for-used-value"></a>used value
- \[css-color-4\] defines the following terms:
  - <a id="term-for-typedef-alpha-value"></a>\<alpha-value\>
  - <a id="term-for-propdef-opacity"></a>opacity
  - <a id="term-for-valdef-color-transparent"></a>transparent
- \[css-display-3\] defines the following terms:
  - <a id="term-for-containing-block"></a>containing block
  - <a id="term-for-propdef-display"></a>display
- \[css-fonts-3\] defines the following terms:
  - <a id="term-for-propdef-font"></a>font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-propdef-font-size"></a>font-size
  - <a id="term-for-propdef-font-stretch"></a>font-stretch
  - <a id="term-for-propdef-font-style"></a>font-style
  - <a id="term-for-propdef-font-variant"></a>font-variant
  - <a id="term-for-propdef-font-weight"></a>font-weight
- \[css-fonts-4\] defines the following terms:
  - <a id="term-for-propdef-font-size-adjust"></a>font-size-adjust
- \[css-masking-1\] defines the following terms:
  - <a id="term-for-propdef-clip"></a>clip
  - <a id="term-for-propdef-clip-path"></a>clip-path
  - <a id="term-for-propdef-clip-rule"></a>clip-rule
  - <a id="term-for-propdef-mask"></a>mask
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
- \[css-position-3\] defines the following terms:
  - <a id="term-for-stacking-context"></a>stacking context
- \[css-text-3\] defines the following terms:
  - <a id="term-for-propdef-letter-spacing"></a>letter-spacing
  - <a id="term-for-propdef-word-spacing"></a>word-spacing
- \[css-text-decor-3\] defines the following terms:
  - <a id="term-for-propdef-text-decoration"></a>text-decoration
  - <a id="term-for-propdef-text-shadow"></a>text-shadow
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-local-coordinate-system"></a>local coordinate system
- \[css-ui-3\] defines the following terms:
  - <a id="term-for-propdef-cursor"></a>cursor
  - <a id="term-for-propdef-outline"></a>outline
- \[css-values-4\] defines the following terms:
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-mult-one-plus"></a>+
  - <a id="term-for-comb-comma"></a>,
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-zero-value"></a>\<zero\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-funcdef-calc"></a>calc()
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
  - <a id="term-for-propdef-unicode-bidi"></a>unicode-bidi
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="term-for-propdef-visibility"></a>visibility
- \[css3-images\] defines the following terms:
  - <a id="term-for-image-type"></a>\<image\>
  - <a id="term-for-concrete-object-size"></a>concrete object size
  - <a id="term-for-propdef-image-rendering"></a>image-rendering
- \[CSS3BG\] defines the following terms:
  - <a id="term-for-propdef-border"></a>border
  - <a id="term-for-propdef-border-image"></a>border-image
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
- \[CSS3COLOR\] defines the following terms:
  - <a id="term-for-valuea-def-color"></a>\<color\>
  - <a id="term-for-color0"></a>color
- \[CSS3VAL\] defines the following terms:
  - <a id="term-for-angle-value"></a>\<angle\>
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-typedef-number-percentage"></a>\<number-percentage\>
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-string-value"></a>\<string\>
- \[HTML\] defines the following terms:
  - <a id="term-for-browsing-context"></a>browsing context
  - <a id="term-for-the-img-element"></a>img
- \[mediaqueries-5\] defines the following terms:
  - <a id="term-for-valdef-custom-media-false"></a>false
- \[selectors-4\] defines the following terms:
  - <a id="term-for-visited-pseudo"></a>:visited
- \[SVG11\] defines the following terms:
  - <a id="term-for-AlignmentBaselineProperty"></a>alignment-baseline
  - <a id="term-for-AnimateElement"></a>animate
  - <a id="term-for-AnimateTransformElement"></a>animatetransform
  - <a id="term-for-BaselineShiftProperty"></a>baseline-shift
  - <a id="term-for-DominantBaselineProperty"></a>dominant-baseline
  - <a id="term-for-EnableBackgroundProperty"></a>enable-background
  - <a id="term-for-GlyphOrientationHorizontalProperty"></a>glyph-orientation-horizontal
  - <a id="term-for-GlyphOrientationVerticalProperty"></a>glyph-orientation-vertical
  - <a id="term-for-KerningProperty"></a>kerning
  - <a id="term-for-SetElement"></a>set
  - <a id="term-for-SimpleAlphaBlending"></a>simple alpha compositing
- \[svg12t\] defines the following terms:
  - <a id="term-for-TermUnsupportedValue"></a>unsupported
- \[SVG2\] defines the following terms:
  - <a id="term-for-InterfaceSVGAnimatedBoolean"></a>SVGAnimatedBoolean
  - <a id="term-for-InterfaceSVGAnimatedEnumeration"></a>SVGAnimatedEnumeration
  - <a id="term-for-InterfaceSVGAnimatedInteger"></a>SVGAnimatedInteger
  - <a id="term-for-InterfaceSVGAnimatedLength"></a>SVGAnimatedLength
  - <a id="term-for-InterfaceSVGAnimatedNumber"></a>SVGAnimatedNumber
  - <a id="term-for-InterfaceSVGAnimatedNumberList"></a>SVGAnimatedNumberList
  - <a id="term-for-InterfaceSVGAnimatedPreserveAspectRatio"></a>SVGAnimatedPreserveAspectRatio
  - <a id="term-for-InterfaceSVGAnimatedString"></a>SVGAnimatedString
  - <a id="term-for-InterfaceSVGElement"></a>SVGElement
  - <a id="term-for-InterfaceSVGURIReference"></a>SVGURIReference
  - <a id="term-for-InterfaceSVGUnitTypes"></a>SVGUnitTypes
  - <a id="term-for-bounding-box"></a>bounding box
  - <a id="term-for-ColorInterpolationProperty"></a>color-interpolation
  - <a id="term-for-ColorRenderingProperty"></a>color-rendering
  - <a id="term-for-container-element"></a>container element
  - <a id="term-for-elementdef-defs"></a>defs
  - <a id="term-for-elementdef-desc"></a>desc
  - <a id="term-for-TermDescriptiveElement"></a>descriptive element
  - <a id="term-for-FillProperty"></a>fill
  - <a id="term-for-FillOpacityProperty"></a>fill-opacity
  - <a id="term-for-FillRuleProperty"></a>fill-rule
  - <a id="term-for-graphics-element"></a>graphics element
  - <a id="term-for-elementdef-image"></a>image
  - <a id="term-for-TermInitialValue"></a>initial value
  - <a id="term-for-MarkerProperty"></a>marker
  - <a id="term-for-MarkerEndProperty"></a>marker-end
  - <a id="term-for-MarkerMidProperty"></a>marker-mid
  - <a id="term-for-MarkerStartProperty"></a>marker-start
  - <a id="term-for-elementdef-metadata"></a>metadata
  - <a id="term-for-ObjectBoundingBoxUnits"></a>object bounding box units
  - <a id="term-for-PointerEventsProperty"></a>pointer-events
  - <a id="term-for-elementdef-rect"></a>rect
  - <a id="term-for-elementdef-script"></a>script
  - <a id="term-for-ShapeRenderingProperty"></a>shape-rendering
  - <a id="term-for-StopColorProperty"></a>stop-color
  - <a id="term-for-StopOpacityProperty"></a>stop-opacity
  - <a id="term-for-StrokeProperty"></a>stroke
  - <a id="term-for-StrokeDasharrayProperty"></a>stroke-dasharray
  - <a id="term-for-StrokeDashoffsetProperty"></a>stroke-dashoffset
  - <a id="term-for-StrokeLinecapProperty"></a>stroke-linecap
  - <a id="term-for-StrokeLinejoinProperty"></a>stroke-linejoin
  - <a id="term-for-StrokeMiterlimitProperty"></a>stroke-miterlimit
  - <a id="term-for-StrokeOpacityProperty"></a>stroke-opacity
  - <a id="term-for-StrokeWidthProperty"></a>stroke-width
  - <a id="term-for-TextAnchorProperty"></a>text-anchor
  - <a id="term-for-TextRenderingProperty"></a>text-rendering
  - <a id="term-for-elementdef-title"></a>title
  - <a id="term-for-elementdef-use"></a>use
- \[WebIDL\] defines the following terms:
  - <a id="term-for-idl-float"></a>float
  - <a id="term-for-idl-unsigned-short"></a>unsigned short

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-compositing-1"></a>\[COMPOSITING-1\]  
Rik Cabanier; Nikos Andronikos. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 13 January 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 5 July 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 20 September 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 26 August 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 31 July 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Rossen Atanassov; Arron Eicholz. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 17 May 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 12 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 3 July 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 30 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 10 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Elika Etemad; Tab Atkins Jr.. [CSS Image Values and Replaced Content Module Level 3](https://www.w3.org/TR/css3-images/). 17 April 2012. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-images&#x2F;](https://www.w3.org/TR/css3-images/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 17 October 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3color"></a>\[CSS3COLOR\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 19 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css3val"></a>\[CSS3VAL\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 14 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-html5"></a>\[HTML5\]  
Ian Hickson; et al. [HTML5](https://www.w3.org/TR/html5/). 27 March 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;html5&#x2F;](https://www.w3.org/TR/html5/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Media Queries Level 5 URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;mediaqueries-5&#x2F;](https://drafts.csswg.org/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-webidl"></a>\[WebIDL\]  
Cameron McCormack; Boris Zbarsky; Tobie Langel. [Web IDL](https://heycam.github.io/webidl/). 15 December 2016. ED. URL: [https&#x3A;&#x2F;&#x2F;heycam&#x2E;github&#x2E;io&#x2F;webidl&#x2F;](https://heycam.github.io/webidl/)

### <a id="informative"></a>Informative References

<a id="biblio-artd"></a>\[ArTD\]  
B. Jacob. [Arithmetic Timing Differences](https://wiki.mozilla.org/User:Bjacob/ArithmeticTimingDifferences). URL: [https&#x3A;&#x2F;&#x2F;wiki&#x2E;mozilla&#x2E;org&#x2F;User&#x3A;Bjacob&#x2F;ArithmeticTimingDifferences](https://wiki.mozilla.org/User:Bjacob/ArithmeticTimingDifferences)

<a id="biblio-cmam"></a>\[Cmam\]  
IEC. [IEC 61966-2-1:1999 Colour measurement and management - Part 2-1: Colour management - Default RGB colour space - sRGB](https://webstore.iec.ch/publication/6169). URL: [https&#x3A;&#x2F;&#x2F;webstore&#x2E;iec&#x2E;ch&#x2F;publication&#x2F;6169](https://webstore.iec.ch/publication/6169)

<a id="biblio-css3-animations"></a>\[CSS3-ANIMATIONS\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-porterduff"></a>\[PORTERDUFF\]  
Thomas Porter; Tom Duff. Compositing digital images.

<a id="biblio-tam"></a>\[TaM\]  
Ebert et al, AP Professional. Texturing and Modeling. 1994.

## <a id="property-index"></a>Property Index

<strong>Table 34 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Applies to

<strong>Column 5 (header cell; scope col):</strong>

Inh.

<strong>Column 6 (header cell; scope col):</strong>

%ages

<strong>Column 7 (header cell; scope col):</strong>

Ani­mat­able

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Column 10 (header cell; scope col):</strong>

Media

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-color-interpolation-filters③④"></a>

[color-interpolation-filters](#propdef-color-interpolation-filters)

<strong>Column 2 (data cell):</strong>

auto \| sRGB \| linearRGB

<strong>Column 3 (data cell):</strong>

linearRGB

<strong>Column 4 (data cell):</strong>

All filter primitives

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

no

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Column 10 (data cell):</strong>

visual

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-filter③⑤"></a>

[filter](#propdef-filter)

<strong>Column 2 (data cell):</strong>

none \| \<filter-value-list\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

All elements. In SVG, it applies to container elements without the defs element, all graphics elements and the use element.

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

See prose in Animation of Filters.

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Column 10 (data cell):</strong>

visual

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-flood-color②⑦"></a>

[flood-color](#propdef-flood-color)

<strong>Column 2 (data cell):</strong>

\<color\>

<strong>Column 3 (data cell):</strong>

black

<strong>Column 4 (data cell):</strong>

feFlood and feDropShadow elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

as color

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Column 10 (data cell):</strong>

visual

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-flood-opacity②⑥"></a>

[flood-opacity](#propdef-flood-opacity)

<strong>Column 2 (data cell):</strong>

\<alpha-value\>

<strong>Column 3 (data cell):</strong>

1

<strong>Column 4 (data cell):</strong>

feFlood and feDropShadow elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

as number or percentage

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the specified value converted to a number, clamped to the range \[0,1\]

<strong>Column 10 (data cell):</strong>

visual

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-lighting-color②⑦"></a>

[lighting-color](#propdef-lighting-color)

<strong>Column 2 (data cell):</strong>

\<color\>

<strong>Column 3 (data cell):</strong>

white

<strong>Column 4 (data cell):</strong>

feDiffuseLighting and feSpecularLighting elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

as color

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Column 10 (data cell):</strong>

visual

## <a id="idl-index"></a>IDL Index

<a id="ref-for-svgfilterelement②"></a>

<a id="ref-for-InterfaceSVGElement②③"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②⑥"></a>

<a id="ref-for-dom-svgfilterelement-filterunits①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①①⓪"></a>

<a id="ref-for-dom-svgfilterelement-primitiveunits①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①⑥"></a>

<a id="ref-for-dom-svgfilterelement-x①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①⑦"></a>

<a id="ref-for-dom-svgfilterelement-y①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength②①"></a>

<a id="ref-for-dom-svgfilterelement-width①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength③①"></a>

<a id="ref-for-dom-svgfilterelement-height①"></a>

<a id="ref-for-svgfilterelement①①"></a>

<a id="ref-for-InterfaceSVGURIReference②"></a>

<a id="ref-for-InterfaceSVGAnimatedLength⑧①"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-x①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength⑨①"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-y①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①⓪①"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-width①"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①①①"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-height①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⑧"></a>

<a id="ref-for-dom-svgfilterprimitivestandardattributes-result①"></a>

<a id="ref-for-svgfeblendelement②"></a>

<a id="ref-for-InterfaceSVGElement①①⓪"></a>

<a id="ref-for-idl-unsigned-short⑤⑧"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_unknown①"></a>

<a id="ref-for-idl-unsigned-short①①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_normal①"></a>

<a id="ref-for-idl-unsigned-short②①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_multiply①"></a>

<a id="ref-for-idl-unsigned-short③①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_screen①"></a>

<a id="ref-for-idl-unsigned-short④①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_darken①"></a>

<a id="ref-for-idl-unsigned-short⑤⑦"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_lighten①"></a>

<a id="ref-for-idl-unsigned-short⑥①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_overlay①"></a>

<a id="ref-for-idl-unsigned-short⑦①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color_dodge①"></a>

<a id="ref-for-idl-unsigned-short⑧①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color_burn①"></a>

<a id="ref-for-idl-unsigned-short⑨①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_hard_light①"></a>

<a id="ref-for-idl-unsigned-short①⓪①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_soft_light①"></a>

<a id="ref-for-idl-unsigned-short①①①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_difference①"></a>

<a id="ref-for-idl-unsigned-short①②①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_exclusion①"></a>

<a id="ref-for-idl-unsigned-short①③①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_hue①"></a>

<a id="ref-for-idl-unsigned-short①④①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_saturation①"></a>

<a id="ref-for-idl-unsigned-short①⑤①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_color①"></a>

<a id="ref-for-idl-unsigned-short①⑥①"></a>

<a id="ref-for-dom-svgfeblendelement-svg_feblend_mode_luminosity①"></a>

<a id="ref-for-InterfaceSVGAnimatedString②①⓪"></a>

<a id="ref-for-dom-svgfeblendelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⑨"></a>

<a id="ref-for-dom-svgfeblendelement-in2①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration④①"></a>

<a id="ref-for-dom-svgfeblendelement-mode①"></a>

<a id="ref-for-svgfeblendelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑦"></a>

<a id="ref-for-svgfecolormatrixelement②"></a>

<a id="ref-for-InterfaceSVGElement②④"></a>

<a id="ref-for-idl-unsigned-short①⑦①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_unknown①"></a>

<a id="ref-for-idl-unsigned-short①⑧①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_matrix①"></a>

<a id="ref-for-idl-unsigned-short①⑨①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_saturate①"></a>

<a id="ref-for-idl-unsigned-short②⓪①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_huerotate①"></a>

<a id="ref-for-idl-unsigned-short②①①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-svg_fecolormatrix_type_luminancetoalpha①"></a>

<a id="ref-for-InterfaceSVGAnimatedString⑥①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑥①"></a>

<a id="ref-for-dom-svgfecolormatrixelement-type①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList⑥"></a>

<a id="ref-for-dom-svgfecolormatrixelement-values①"></a>

<a id="ref-for-svgfecolormatrixelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑧"></a>

<a id="ref-for-svgfecomponenttransferelement②"></a>

<a id="ref-for-InterfaceSVGElement③①"></a>

<a id="ref-for-InterfaceSVGAnimatedString⑧①"></a>

<a id="ref-for-dom-svgfecomponenttransferelement-in1①"></a>

<a id="ref-for-svgfecomponenttransferelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes②①"></a>

<a id="ref-for-InterfaceSVGElement④①"></a>

<a id="ref-for-idl-unsigned-short②②①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_unknown①"></a>

<a id="ref-for-idl-unsigned-short②③①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_identity①"></a>

<a id="ref-for-idl-unsigned-short②④①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_table①"></a>

<a id="ref-for-idl-unsigned-short②⑤①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_discrete①"></a>

<a id="ref-for-idl-unsigned-short②⑥①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_linear①"></a>

<a id="ref-for-idl-unsigned-short②⑦①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-svg_fecomponenttransfer_type_gamma①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑧①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-type①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList②①"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-tablevalues①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨⑧"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-slope①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①①⓪"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-intercept①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②①⓪"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-amplitude①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③①⓪"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-exponent①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④①⓪"></a>

<a id="ref-for-dom-svgcomponenttransferfunctionelement-offset①"></a>

<a id="ref-for-svgfefuncrelement①"></a>

<a id="ref-for-svgcomponenttransferfunctionelement④"></a>

<a id="ref-for-svgfefuncgelement①"></a>

<a id="ref-for-svgcomponenttransferfunctionelement①①"></a>

<a id="ref-for-svgfefuncbelement①"></a>

<a id="ref-for-svgcomponenttransferfunctionelement②①"></a>

<a id="ref-for-svgfefuncaelement①"></a>

<a id="ref-for-svgcomponenttransferfunctionelement③①"></a>

<a id="ref-for-svgfecompositeelement②"></a>

<a id="ref-for-InterfaceSVGElement⑤①"></a>

<a id="ref-for-idl-unsigned-short②⑧①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_unknown①"></a>

<a id="ref-for-idl-unsigned-short②⑨①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_over①"></a>

<a id="ref-for-idl-unsigned-short③⓪①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_in①"></a>

<a id="ref-for-idl-unsigned-short③①①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_out①"></a>

<a id="ref-for-idl-unsigned-short③②①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_atop①"></a>

<a id="ref-for-idl-unsigned-short③③①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_xor①"></a>

<a id="ref-for-idl-unsigned-short③④①"></a>

<a id="ref-for-dom-svgfecompositeelement-svg_fecomposite_operator_arithmetic①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⓪①"></a>

<a id="ref-for-dom-svgfecompositeelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①①①"></a>

<a id="ref-for-dom-svgfecompositeelement-in2①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⓪①"></a>

<a id="ref-for-dom-svgfecompositeelement-operator①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⓪①"></a>

<a id="ref-for-dom-svgfecompositeelement-k1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①①①"></a>

<a id="ref-for-dom-svgfecompositeelement-k2①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①②①"></a>

<a id="ref-for-dom-svgfecompositeelement-k3①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①③①"></a>

<a id="ref-for-dom-svgfecompositeelement-k4①"></a>

<a id="ref-for-svgfecompositeelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes③①"></a>

<a id="ref-for-svgfeconvolvematrixelement②"></a>

<a id="ref-for-InterfaceSVGElement⑥①"></a>

<a id="ref-for-idl-unsigned-short③⑤①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_unknown①"></a>

<a id="ref-for-idl-unsigned-short③⑥①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_duplicate①"></a>

<a id="ref-for-idl-unsigned-short③⑦①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_wrap①"></a>

<a id="ref-for-idl-unsigned-short③⑧①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-svg_edgemode_none①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①④①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger①⓪"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-orderx①"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger①①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-ordery①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumberList④①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelmatrix①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⑧①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-divisor①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber①⑨①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-bias①"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger②①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-targetx①"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger③①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-targety①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①②①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-edgemode①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⓪①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelunitlengthx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②①①"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-kernelunitlengthy①"></a>

<a id="ref-for-InterfaceSVGAnimatedBoolean②"></a>

<a id="ref-for-dom-svgfeconvolvematrixelement-preservealpha①"></a>

<a id="ref-for-svgfeconvolvematrixelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes④①"></a>

<a id="ref-for-svgfediffuselightingelement②"></a>

<a id="ref-for-InterfaceSVGElement⑦①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑥①"></a>

<a id="ref-for-dom-svgfediffuselightingelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑥①"></a>

<a id="ref-for-dom-svgfediffuselightingelement-surfacescale①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑦①"></a>

<a id="ref-for-dom-svgfediffuselightingelement-diffuseconstant①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑧①"></a>

<a id="ref-for-dom-svgfediffuselightingelement-kernelunitlengthx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber②⑨①"></a>

<a id="ref-for-dom-svgfediffuselightingelement-kernelunitlengthy①"></a>

<a id="ref-for-svgfediffuselightingelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑤①"></a>

<a id="ref-for-svgfedistantlightelement①"></a>

<a id="ref-for-InterfaceSVGElement⑧①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③④①"></a>

<a id="ref-for-dom-svgfedistantlightelement-azimuth①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑤①"></a>

<a id="ref-for-dom-svgfedistantlightelement-elevation①"></a>

<a id="ref-for-svgfepointlightelement①"></a>

<a id="ref-for-InterfaceSVGElement⑨①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑧①"></a>

<a id="ref-for-dom-svgfepointlightelement-x①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber③⑨①"></a>

<a id="ref-for-dom-svgfepointlightelement-y①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⓪①"></a>

<a id="ref-for-dom-svgfepointlightelement-z①"></a>

<a id="ref-for-svgfespotlightelement①"></a>

<a id="ref-for-InterfaceSVGElement①⓪①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④④①"></a>

<a id="ref-for-dom-svgfespotlightelement-x①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑤①"></a>

<a id="ref-for-dom-svgfespotlightelement-y①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑥①"></a>

<a id="ref-for-dom-svgfespotlightelement-z①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑦①"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsatx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑧①"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsaty①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber④⑨①"></a>

<a id="ref-for-dom-svgfespotlightelement-pointsatz①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑤⓪①"></a>

<a id="ref-for-dom-svgfespotlightelement-specularexponent①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑤①①"></a>

<a id="ref-for-dom-svgfespotlightelement-limitingconeangle①"></a>

<a id="ref-for-svgfedisplacementmapelement②"></a>

<a id="ref-for-InterfaceSVGElement①①①"></a>

<a id="ref-for-idl-unsigned-short③⑨①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_unknown①"></a>

<a id="ref-for-idl-unsigned-short④⓪①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_r①"></a>

<a id="ref-for-idl-unsigned-short④①①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_g①"></a>

<a id="ref-for-idl-unsigned-short④②①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_b①"></a>

<a id="ref-for-idl-unsigned-short④③①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-svg_channel_a①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑧①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedString①⑨①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-in2①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⓪①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-scale①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①④①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-xchannelselector①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑤①"></a>

<a id="ref-for-dom-svgfedisplacementmapelement-ychannelselector①"></a>

<a id="ref-for-svgfedisplacementmapelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑥①"></a>

<a id="ref-for-svgfedropshadowelement②"></a>

<a id="ref-for-InterfaceSVGElement①②①"></a>

<a id="ref-for-InterfaceSVGAnimatedString②②①"></a>

<a id="ref-for-dom-svgfedropshadowelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥②①"></a>

<a id="ref-for-dom-svgfedropshadowelement-dx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥③①"></a>

<a id="ref-for-dom-svgfedropshadowelement-dy①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥④①"></a>

<a id="ref-for-dom-svgfedropshadowelement-stddeviationx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑥⑤①"></a>

<a id="ref-for-dom-svgfedropshadowelement-stddeviationy①"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation①"></a>

<a id="ref-for-idl-float④"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx①"></a>

<a id="ref-for-idl-float①①"></a>

<a id="ref-for-dom-svgfedropshadowelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy①"></a>

<a id="ref-for-svgfedropshadowelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑦①"></a>

<a id="ref-for-svgfefloodelement②"></a>

<a id="ref-for-InterfaceSVGElement①③①"></a>

<a id="ref-for-svgfefloodelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑧①"></a>

<a id="ref-for-svgfegaussianblurelement②"></a>

<a id="ref-for-InterfaceSVGElement①④①"></a>

<a id="ref-for-idl-unsigned-short④④①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_unknown①"></a>

<a id="ref-for-idl-unsigned-short④⑤①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_duplicate①"></a>

<a id="ref-for-idl-unsigned-short④⑥①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_wrap①"></a>

<a id="ref-for-idl-unsigned-short④⑦①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-svg_edgemode_none①"></a>

<a id="ref-for-InterfaceSVGAnimatedString②④①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⓪①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-stddeviationx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦①①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-stddeviationy①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration①⑧①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-edgemode①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation①"></a>

<a id="ref-for-idl-float②①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationx①"></a>

<a id="ref-for-idl-float③①"></a>

<a id="ref-for-dom-svgfegaussianblurelement-setstddeviation-stddeviationx-stddeviationy-stddeviationy①"></a>

<a id="ref-for-svgfegaussianblurelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes⑨①"></a>

<a id="ref-for-svgfeimageelement③"></a>

<a id="ref-for-InterfaceSVGElement①⑤①"></a>

<a id="ref-for-InterfaceSVGAnimatedPreserveAspectRatio②"></a>

<a id="ref-for-dom-svgfeimageelement-preserveaspectratio①"></a>

<a id="ref-for-InterfaceSVGAnimatedString②⑥①"></a>

<a id="ref-for-dom-svgfeimageelement-crossorigin①"></a>

<a id="ref-for-svgfeimageelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⓪①"></a>

<a id="ref-for-svgfeimageelement②①"></a>

<a id="ref-for-InterfaceSVGURIReference①①"></a>

<a id="ref-for-svgfemergeelement②"></a>

<a id="ref-for-InterfaceSVGElement①⑥①"></a>

<a id="ref-for-svgfemergeelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①①①"></a>

<a id="ref-for-svgfemergenodeelement①"></a>

<a id="ref-for-InterfaceSVGElement①⑦①"></a>

<a id="ref-for-InterfaceSVGAnimatedString②⑧①"></a>

<a id="ref-for-dom-svgfemergenodeelement-in1①"></a>

<a id="ref-for-svgfemorphologyelement②"></a>

<a id="ref-for-InterfaceSVGElement①⑧①"></a>

<a id="ref-for-idl-unsigned-short④⑧①"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_unknown①"></a>

<a id="ref-for-idl-unsigned-short④⑨①"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_erode①"></a>

<a id="ref-for-idl-unsigned-short⑤⓪①"></a>

<a id="ref-for-dom-svgfemorphologyelement-svg_morphology_operator_dilate①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⓪①"></a>

<a id="ref-for-dom-svgfemorphologyelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②⓪①"></a>

<a id="ref-for-dom-svgfemorphologyelement-operator①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦④①"></a>

<a id="ref-for-dom-svgfemorphologyelement-radiusx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑤①"></a>

<a id="ref-for-dom-svgfemorphologyelement-radiusy①"></a>

<a id="ref-for-svgfemorphologyelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①②①"></a>

<a id="ref-for-svgfeoffsetelement②"></a>

<a id="ref-for-InterfaceSVGElement①⑨①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③②①"></a>

<a id="ref-for-dom-svgfeoffsetelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑧①"></a>

<a id="ref-for-dom-svgfeoffsetelement-dx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑦⑨①"></a>

<a id="ref-for-dom-svgfeoffsetelement-dy①"></a>

<a id="ref-for-svgfeoffsetelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①③①"></a>

<a id="ref-for-svgfespecularlightingelement②"></a>

<a id="ref-for-InterfaceSVGElement②⓪①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③④①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-in1①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧②①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-surfacescale①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧③①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-specularconstant①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧④①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-specularexponent①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑤①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-kernelunitlengthx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑧⑥①"></a>

<a id="ref-for-dom-svgfespecularlightingelement-kernelunitlengthy①"></a>

<a id="ref-for-svgfespecularlightingelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①④①"></a>

<a id="ref-for-svgfetileelement②"></a>

<a id="ref-for-InterfaceSVGElement②①①"></a>

<a id="ref-for-InterfaceSVGAnimatedString③⑥①"></a>

<a id="ref-for-dom-svgfetileelement-in1①"></a>

<a id="ref-for-svgfetileelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑤①"></a>

<a id="ref-for-svgfeturbulenceelement②"></a>

<a id="ref-for-InterfaceSVGElement②②①"></a>

<a id="ref-for-idl-unsigned-short⑤①①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_unknown①"></a>

<a id="ref-for-idl-unsigned-short⑤②①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_fractalnoise①"></a>

<a id="ref-for-idl-unsigned-short⑤③①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_turbulence_type_turbulence①"></a>

<a id="ref-for-idl-unsigned-short⑤④①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_unknown①"></a>

<a id="ref-for-idl-unsigned-short⑤⑤①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_stitch①"></a>

<a id="ref-for-idl-unsigned-short⑤⑥①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-svg_stitchtype_nostitch①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨②①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-basefrequencyx①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨③①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-basefrequencyy①"></a>

<a id="ref-for-InterfaceSVGAnimatedInteger⑧①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-numoctaves①"></a>

<a id="ref-for-InterfaceSVGAnimatedNumber⑨④①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-seed①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②②①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-stitchtiles①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②③①"></a>

<a id="ref-for-dom-svgfeturbulenceelement-type①"></a>

<a id="ref-for-svgfeturbulenceelement①①"></a>

<a id="ref-for-svgfilterprimitivestandardattributes①⑥①"></a>

```text
interface SVGFilterElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration filterUnits;
  readonly attribute SVGAnimatedEnumeration primitiveUnits;
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
};

SVGFilterElement includes SVGURIReference;

interface mixin SVGFilterPrimitiveStandardAttributes {
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
  readonly attribute SVGAnimatedString result;
};

interface SVGFEBlendElement : SVGElement {

  // Blend Mode Types
  const unsigned short SVG_FEBLEND_MODE_UNKNOWN = 0;
  const unsigned short SVG_FEBLEND_MODE_NORMAL = 1;
  const unsigned short SVG_FEBLEND_MODE_MULTIPLY = 2;
  const unsigned short SVG_FEBLEND_MODE_SCREEN = 3;
  const unsigned short SVG_FEBLEND_MODE_DARKEN = 4;
  const unsigned short SVG_FEBLEND_MODE_LIGHTEN = 5;
  const unsigned short SVG_FEBLEND_MODE_OVERLAY = 6;
  const unsigned short SVG_FEBLEND_MODE_COLOR_DODGE = 7;
  const unsigned short SVG_FEBLEND_MODE_COLOR_BURN = 8;
  const unsigned short SVG_FEBLEND_MODE_HARD_LIGHT = 9;
  const unsigned short SVG_FEBLEND_MODE_SOFT_LIGHT = 10;
  const unsigned short SVG_FEBLEND_MODE_DIFFERENCE = 11;
  const unsigned short SVG_FEBLEND_MODE_EXCLUSION = 12;
  const unsigned short SVG_FEBLEND_MODE_HUE = 13;
  const unsigned short SVG_FEBLEND_MODE_SATURATION = 14;
  const unsigned short SVG_FEBLEND_MODE_COLOR = 15;
  const unsigned short SVG_FEBLEND_MODE_LUMINOSITY = 16;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedEnumeration mode;
};

SVGFEBlendElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEColorMatrixElement : SVGElement {

  // Color Matrix Types
  const unsigned short SVG_FECOLORMATRIX_TYPE_UNKNOWN = 0;
  const unsigned short SVG_FECOLORMATRIX_TYPE_MATRIX = 1;
  const unsigned short SVG_FECOLORMATRIX_TYPE_SATURATE = 2;
  const unsigned short SVG_FECOLORMATRIX_TYPE_HUEROTATE = 3;
  const unsigned short SVG_FECOLORMATRIX_TYPE_LUMINANCETOALPHA = 4;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedEnumeration type;
  readonly attribute SVGAnimatedNumberList values;
};

SVGFEColorMatrixElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEComponentTransferElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};

SVGFEComponentTransferElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGComponentTransferFunctionElement : SVGElement {

  // Component Transfer Types
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN = 0;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY = 1;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_TABLE = 2;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE = 3;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_LINEAR = 4;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_GAMMA = 5;

  readonly attribute SVGAnimatedEnumeration type;
  readonly attribute SVGAnimatedNumberList tableValues;
  readonly attribute SVGAnimatedNumber slope;
  readonly attribute SVGAnimatedNumber intercept;
  readonly attribute SVGAnimatedNumber amplitude;
  readonly attribute SVGAnimatedNumber exponent;
  readonly attribute SVGAnimatedNumber offset;
};

interface SVGFEFuncRElement : SVGComponentTransferFunctionElement {
};

interface SVGFEFuncGElement : SVGComponentTransferFunctionElement {
};

interface SVGFEFuncBElement : SVGComponentTransferFunctionElement {
};

interface SVGFEFuncAElement : SVGComponentTransferFunctionElement {
};

interface SVGFECompositeElement : SVGElement {

  // Composite Operators
  const unsigned short SVG_FECOMPOSITE_OPERATOR_UNKNOWN = 0;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OVER = 1;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_IN = 2;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OUT = 3;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ATOP = 4;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_XOR = 5;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ARITHMETIC = 6;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedEnumeration operator;
  readonly attribute SVGAnimatedNumber k1;
  readonly attribute SVGAnimatedNumber k2;
  readonly attribute SVGAnimatedNumber k3;
  readonly attribute SVGAnimatedNumber k4;
};

SVGFECompositeElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEConvolveMatrixElement : SVGElement {

  // Edge Mode Values
  const unsigned short SVG_EDGEMODE_UNKNOWN = 0;
  const unsigned short SVG_EDGEMODE_DUPLICATE = 1;
  const unsigned short SVG_EDGEMODE_WRAP = 2;
  const unsigned short SVG_EDGEMODE_NONE = 3;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedInteger orderX;
  readonly attribute SVGAnimatedInteger orderY;
  readonly attribute SVGAnimatedNumberList kernelMatrix;
  readonly attribute SVGAnimatedNumber divisor;
  readonly attribute SVGAnimatedNumber bias;
  readonly attribute SVGAnimatedInteger targetX;
  readonly attribute SVGAnimatedInteger targetY;
  readonly attribute SVGAnimatedEnumeration edgeMode;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
  readonly attribute SVGAnimatedBoolean preserveAlpha;
};

SVGFEConvolveMatrixElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEDiffuseLightingElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber surfaceScale;
  readonly attribute SVGAnimatedNumber diffuseConstant;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
};

SVGFEDiffuseLightingElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEDistantLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber azimuth;
  readonly attribute SVGAnimatedNumber elevation;
};

interface SVGFEPointLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber x;
  readonly attribute SVGAnimatedNumber y;
  readonly attribute SVGAnimatedNumber z;
};

interface SVGFESpotLightElement : SVGElement {
  readonly attribute SVGAnimatedNumber x;
  readonly attribute SVGAnimatedNumber y;
  readonly attribute SVGAnimatedNumber z;
  readonly attribute SVGAnimatedNumber pointsAtX;
  readonly attribute SVGAnimatedNumber pointsAtY;
  readonly attribute SVGAnimatedNumber pointsAtZ;
  readonly attribute SVGAnimatedNumber specularExponent;
  readonly attribute SVGAnimatedNumber limitingConeAngle;
};

interface SVGFEDisplacementMapElement : SVGElement {

  // Channel Selectors
  const unsigned short SVG_CHANNEL_UNKNOWN = 0;
  const unsigned short SVG_CHANNEL_R = 1;
  const unsigned short SVG_CHANNEL_G = 2;
  const unsigned short SVG_CHANNEL_B = 3;
  const unsigned short SVG_CHANNEL_A = 4;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedString in2;
  readonly attribute SVGAnimatedNumber scale;
  readonly attribute SVGAnimatedEnumeration xChannelSelector;
  readonly attribute SVGAnimatedEnumeration yChannelSelector;
};

SVGFEDisplacementMapElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEDropShadowElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber dx;
  readonly attribute SVGAnimatedNumber dy;
  readonly attribute SVGAnimatedNumber stdDeviationX;
  readonly attribute SVGAnimatedNumber stdDeviationY;

  void setStdDeviation(float stdDeviationX, float stdDeviationY);
};

SVGFEDropShadowElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEFloodElement : SVGElement {
};

SVGFEFloodElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEGaussianBlurElement : SVGElement {

  // Edge Mode Values
  const unsigned short SVG_EDGEMODE_UNKNOWN = 0;
  const unsigned short SVG_EDGEMODE_DUPLICATE = 1;
  const unsigned short SVG_EDGEMODE_WRAP = 2;
  const unsigned short SVG_EDGEMODE_NONE = 3;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber stdDeviationX;
  readonly attribute SVGAnimatedNumber stdDeviationY;
  readonly attribute SVGAnimatedEnumeration edgeMode;

  void setStdDeviation(float stdDeviationX, float stdDeviationY);
};

SVGFEGaussianBlurElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEImageElement : SVGElement {
  readonly attribute SVGAnimatedPreserveAspectRatio preserveAspectRatio;
  readonly attribute SVGAnimatedString crossOrigin;
};

SVGFEImageElement includes SVGFilterPrimitiveStandardAttributes;
SVGFEImageElement includes SVGURIReference;

interface SVGFEMergeElement : SVGElement {
};

SVGFEMergeElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEMergeNodeElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};

interface SVGFEMorphologyElement : SVGElement {

  // Morphology Operators
  const unsigned short SVG_MORPHOLOGY_OPERATOR_UNKNOWN = 0;
  const unsigned short SVG_MORPHOLOGY_OPERATOR_ERODE = 1;
  const unsigned short SVG_MORPHOLOGY_OPERATOR_DILATE = 2;

  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedEnumeration operator;
  readonly attribute SVGAnimatedNumber radiusX;
  readonly attribute SVGAnimatedNumber radiusY;
};

SVGFEMorphologyElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFEOffsetElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber dx;
  readonly attribute SVGAnimatedNumber dy;
};

SVGFEOffsetElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFESpecularLightingElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
  readonly attribute SVGAnimatedNumber surfaceScale;
  readonly attribute SVGAnimatedNumber specularConstant;
  readonly attribute SVGAnimatedNumber specularExponent;
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
};

SVGFESpecularLightingElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFETileElement : SVGElement {
  readonly attribute SVGAnimatedString in1;
};

SVGFETileElement includes SVGFilterPrimitiveStandardAttributes;

interface SVGFETurbulenceElement : SVGElement {

  // Turbulence Types
  const unsigned short SVG_TURBULENCE_TYPE_UNKNOWN = 0;
  const unsigned short SVG_TURBULENCE_TYPE_FRACTALNOISE = 1;
  const unsigned short SVG_TURBULENCE_TYPE_TURBULENCE = 2;

  // Stitch Options
  const unsigned short SVG_STITCHTYPE_UNKNOWN = 0;
  const unsigned short SVG_STITCHTYPE_STITCH = 1;
  const unsigned short SVG_STITCHTYPE_NOSTITCH = 2;

  readonly attribute SVGAnimatedNumber baseFrequencyX;
  readonly attribute SVGAnimatedNumber baseFrequencyY;
  readonly attribute SVGAnimatedInteger numOctaves;
  readonly attribute SVGAnimatedNumber seed;
  readonly attribute SVGAnimatedEnumeration stitchTiles;
  readonly attribute SVGAnimatedEnumeration type;
};

SVGFETurbulenceElement includes SVGFilterPrimitiveStandardAttributes;

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> How does filter behave on fixed background images? [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;238&#x3E;](https://github.com/w3c/csswg-drafts/issues/238) [↵](#issue-95bcf4be)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> How to behave on invalid number of entries in the value list? [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;237&#x3E;](https://github.com/w3c/csswg-drafts/issues/237) [↵](#issue-4fd9d6f6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Implementations do not match specification. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;113&#x3E;](https://github.com/w3c/csswg-drafts/issues/113) [↵](#issue-1d845580)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Compute distance of filter functions. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;91&#x3E;](https://github.com/w3c/csswg-drafts/issues/91) [↵](#issue-f7e128e9)
