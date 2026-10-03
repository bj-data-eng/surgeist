Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Scroll Snap Module Level 1

Source snapshot: https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/

Snapshot SHA-256: 9e5f5353d73d6df89aedafb11258fb44b8e00cd2b37332a658f417093a3b1e64

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 12 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Scroll Snap Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module contains features to control panning and scrolling behavior with “snap positions”.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	Other documents may supersede this document.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Candidate Recommendation Snapshot</strong>. Publication as a Candidate Recommendation does not imply endorsement by the W3C Membership. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2020/Process-20200915/#dfn-wide-review) and is intended to gather implementation experience. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 11 May 2021 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-scroll-snap” in the title, like this: “\[css-scroll-snap\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-scroll-snap%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

A test suite and an implementation report will be produced during the CR period.

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-scroll-snap-stop"></a>

  [scroll-snap-stop](#propdef-scroll-snap-stop)

- <a id="ref-for-captures-snap-positions"></a>

  <a id="ref-for-scroll-container"></a>

  <a id="ref-for-propdef-scroll-snap-type"></a>

  whether [scroll-snap-type](#propdef-scroll-snap-type) can cause boxes that are not [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) to [capture snap positions](#captures-snap-positions) (see [discussion](https://github.com/w3c/csswg-drafts/issues/4496))

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1. Introduction

<em>This section is not normative.</em>

Popular UX paradigms for scrollable content frequently employ paging through content, or sectioning into logical divisions. This is especially true for touch interactions where it is quicker and easier for users to quickly pan through a flatly-arranged breadth of content rather than delving into a hierarchical structure through tap navigation. For example, it is easier for a user to view many photos in a photo album by panning through a photo slideshow view rather than tapping on individual photos in an album.

However, given the imprecise nature of scrolling inputs like touch panning and mousewheel scrolling, it is difficult for web developers to guarantee a well-controlled scrolling experience, in particular creating the effect of paging through content. For instance, it is easy for a user to land at an awkward scroll position which leaves an item partially on-screen when panning.

<a id="ref-for-scroll-snap-position"></a>

<a id="ref-for-scroll-container①"></a>

To this end, this module introduces [scroll snap positions](#scroll-snap-position) which enforce the scroll positions that a [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) scrollport may end at after a scrolling operation has completed.

<a id="ref-for-propdef-scroll-padding"></a>

<a id="ref-for-scroll-container②"></a>

<a id="ref-for-optimal-viewing-region"></a>

<a id="ref-for-propdef-scroll-margin"></a>

Also, to offer better control over paging and scroll positioning even when snapping is off, this module defines the [scroll-padding](#propdef-scroll-padding) property for use on all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container), to adjust the <a id="ref-for-scroll-container③"></a>scroll container’s [optimal viewing region](#optimal-viewing-region) for the purpose of paging and scroll-into-view operations. Similarly the [scroll-margin](#propdef-scroll-margin) property can be used on any box to adjust its visual area for the purpose of scroll-into-view operations.

### <a id="placement"></a>1.1. Module interactions

This module extends the scrolling user interface features defined in [\[CSS2\]](#biblio-css2) section 11.1.

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-selectordef-first-letter"></a>

None of the properties in this module apply to the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements.

### <a id="values"></a>1.2. Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="examples"></a>2. Motivating Examples

<a id="ref-for-scroll-container④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b9bef3ed"></a> In this example, a series of images arranged in a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) are used to build a photo gallery. In this example the <a id="ref-for-scroll-container⑤"></a>scroll container is larger than the photos contained within (such that multiple images may be seen simultaneously), and the image sizes vary. Using mandatory element-based snap positions, scrolling will always complete with an image centered in the <a id="ref-for-scroll-container⑥"></a>scroll container’s scrollport.
>
> ```css
> img {
>     /* Specifies that the center of each photo
>        should align with the center of the scroll
>        container in the X axis when snapping */
>     scroll-snap-align: none center;
> }
> .photoGallery {
>     width: 500px;
>     overflow-x: auto;
>     overflow-y: hidden;
>     white-space: nowrap;
>     /* Requires that the scroll position always be
>        at a snap position when the scrolling
>        operation completes. */
>     scroll-snap-type: x mandatory;
> }
> ```
>
> ```html
> <div class="photoGallery">
>     <img src="img1.jpg">
>     <img src="img2.jpg">
>     <img src="img3.jpg">
>     <img src="img4.jpg">
>     <img src="img5.jpg">
> </div>
> ```
>
> ![](https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/images/element_snap_positions.png)
>
> The layout of the scroll container’s contents in the example. The snapport is represented by the red rectangle, and the snap area is represented by the yellow rectangle. Since the scroll-snap-align is “center” in the inline (horizontal) axis, a snap position is established at each scroll position which aligns the X-center of the snapport (represented by a red dotted line) with the X-center of a snap area (represented by a yellow dotted line).

<a id="ref-for-scroll-container⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a6210411"></a> This example builds a paginated document that aligns each page near to (but not exactly on) the edge of the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container). This allows the previous page to “peek” in from above in order to make the user aware that they are not yet at the top of the document. Using proximity snap positions instead of mandatory snap positions allows the user to stop halfway through a page (rather than forcing them to snap one page at a time). However, if a scrolling operation would finish near a snap position, then the scroll will be adjusted to align the page as specified.
>
> ```css
> .page {
>     /* Defines the top of each page as the
>        edge that should be used for snapping */
>     scroll-snap-align: start none;
> }
> .docScroller {
>     width: 500px;
>     overflow-x: hidden;
>     overflow-y: auto;
>     /* Specifies that each element’s snap area should
>        align with a 100px offset from the top edge. */
>     scroll-padding: 100px 0 0;
>     /* Encourages scrolling to end at a snap position when the
>         operation completes, if it is near a snap position */
>     scroll-snap-type: y proximity;
> }
> ```
>
> ```html
> <div class="docScroller">
>     <div class="page">Page 1</div>
>     <div class="page">Page 2</div>
>     <div class="page">Page 3</div>
>     <div class="page">Page 4</div>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/images/element_snap_positions_offset.png)
>
> The layout of the scroll container’s contents in the example. The snapport is represented by the red rectangle (inset from the top by 100px due to the scroll-padding), and the snap area is represented by the yellow rectangle. Since the scroll-snap-align is “start” in the Y axis, a snap position is established at each scroll position which aligns the Y-start of the snapport (represented by a red dotted line) with the Y-start of a snap area (represented by a yellow dotted line).

## <a id="overview"></a>3. Scroll Snap Model

<a id="ref-for-propdef-scroll-snap-type①"></a>

<a id="ref-for-scroll-container⑧"></a>

<a id="ref-for-scroll-snap-position①"></a>

<a id="ref-for-dom-window-scrollto"></a>

This module defines controls for <a id="scroll-snap-position"></a>scroll snap positions, which are scroll positions that produce particular alignments of content within a scroll container. Using the [scroll-snap-type](#propdef-scroll-snap-type) property on the relevant [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the author can request a particular bias for the scrollport to land on a [snap position](#scroll-snap-position) after scrolling operations (including programmatic scrolls such as the <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-scrollto">scrollTo()</a></code> method).

<a id="ref-for-scroll-snap-position②"></a>

<a id="ref-for-propdef-scroll-snap-align"></a>

<a id="ref-for-scroll-snap-area"></a>

<a id="ref-for-propdef-scroll-margin①"></a>

<a id="ref-for-scroll-container⑨"></a>

<a id="ref-for-scroll-snapport"></a>

<a id="ref-for-propdef-scroll-padding①"></a>

<a id="ref-for-alignment-subject"></a>

<a id="ref-for-alignment-container"></a>

[Snap positions](#scroll-snap-position) are specified as a particular alignment ([scroll-snap-align](#propdef-scroll-snap-align)) of an element’s [scroll snap area](#scroll-snap-area) (its border bounding box, as modified by [scroll-margin](#propdef-scroll-margin)) within the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s [snapport](#scroll-snapport) (its scrollport, as reduced by [scroll-padding](#propdef-scroll-padding)). This is conceptually equivalent to specifying the alignment of an [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) within an [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container). A scroll position that satisfies the specified alignment is a <a id="ref-for-scroll-snap-position③"></a>snap position.

<a id="ref-for-scroll-container①⓪"></a>

<a id="ref-for-scroll-snap"></a>

<a id="ref-for-scroll-snap-position④"></a>

The act of adjusting the scroll position of a scroll container’s scrollport such that it is aligned to a snap position is called <a id="scroll-snap"></a>snapping, and a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) is said to be [snapped](#scroll-snap) to a [snap position](#scroll-snap-position) if its scrollport’s scroll position is that <a id="ref-for-scroll-snap-position⑤"></a>snap position and there is no active scrolling operation. The CSS Scroll Snap Module intentionally does not specify nor mandate any precise animations or physics used to enforce <a id="ref-for-scroll-snap-position⑥"></a>snap positions; this is left up to the user agent.

<a id="ref-for-scroll-snap-position⑦"></a>

<a id="ref-for-scroll-container①①"></a>

<a id="ref-for-containing-block-chain"></a>

[Snap positions](#scroll-snap-position) only affect the nearest ancestor [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) on the element’s [containing block chain](https://www.w3.org/TR/css-display-3/#containing-block-chain).

## <a id="properties-on-the-scroll-container"></a>4. Capturing Scroll Snap Areas: Properties on the scroll container

<a id="ref-for-propdef-scroll-snap-type②"></a>

### <a id="scroll-snap-type"></a>4.1. Scroll Snapping Rules: the [scroll-snap-type](#propdef-scroll-snap-type) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-snap-type"></a>scroll-snap-type

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-one"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ x <a id="ref-for-comb-one①"></a>\| y <a id="ref-for-comb-one②"></a>\| block <a id="ref-for-comb-one③"></a>\| inline <a id="ref-for-comb-one④"></a>\| both \] \[ mandatory <a id="ref-for-comb-one⑤"></a>\| proximity \][?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword(s)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-propdef-scroll-snap-type③"></a>

<a id="ref-for-scroll-container①②"></a>

<a id="ref-for-scroll-snap-container"></a>

<a id="ref-for-scroll-snap①"></a>

<a id="ref-for-valdef-scroll-snap-type-proximity"></a>

The [scroll-snap-type](#propdef-scroll-snap-type) property specifies whether a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) is a [scroll snap container](#scroll-snap-container), how [strictly](#snap-strictness) it [snaps](#scroll-snap), and [which axes](#snap-axis) are considered. If no strictness value is specified, [proximity](#valdef-scroll-snap-type-proximity) is assumed.

<a id="ref-for-block-axis"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-35961710"></a> In this example, snapping to headings is enabled in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) (the y axis for horizontal writing, x axis for vertical writing):
>
> ```text
> html {
>   scroll-snap-type: block;   /* applied to main document scroller */
> }
> h1, h2, h3, h4, h5, h6 {
>   scroll-snap-align: start;  /* snap to the start (top) of the viewport */
> }
> ```
<a id="ref-for-propdef-scroll-snap-type④"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-the-body-element"></a>

UAs must apply the [scroll-snap-type](#propdef-scroll-snap-type) value set on the root element to the document viewport. Note that, unlike [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), <a id="ref-for-propdef-scroll-snap-type⑤"></a>scroll-snap-type values are <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.

<a id="ref-for-valdef-scroll-snap-type-x"></a>

<a id="ref-for-valdef-scroll-snap-type-y"></a>

<a id="ref-for-valdef-scroll-snap-type-block"></a>

<a id="ref-for-valdef-scroll-snap-type-inline"></a>

<a id="ref-for-valdef-scroll-snap-type-both"></a>

#### <a id="snap-axis"></a>4.1.1.  Scroll Snap Axis: the [x](#valdef-scroll-snap-type-x), [y](#valdef-scroll-snap-type-y), [block](#valdef-scroll-snap-type-block), [inline](#valdef-scroll-snap-type-inline), and [both](#valdef-scroll-snap-type-both) values

<a id="ref-for-scroll-snap-position⑧"></a>

The <a id="axis-value"></a>axis values specify what axis(es) are affected by [snap positions](#scroll-snap-position), and whether <a id="ref-for-scroll-snap-position⑨"></a>snap positions are evaluated independently per axis, or together as a 2D point. Values are defined as follows:

<a id="valdef-scroll-snap-type-x"></a>x  
<a id="ref-for-scroll-snap-position①⓪"></a>

<a id="ref-for-scroll-snap②"></a>

<a id="ref-for-scroll-container①③"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) [snaps](#scroll-snap) to [snap positions](#scroll-snap-position) in its horizontal axis only.

<a id="valdef-scroll-snap-type-y"></a>y  
<a id="ref-for-scroll-snap-position①①"></a>

<a id="ref-for-scroll-snap③"></a>

<a id="ref-for-scroll-container①④"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) [snaps](#scroll-snap) to [snap positions](#scroll-snap-position) in its vertical axis only.

<a id="valdef-scroll-snap-type-block"></a>block  
<a id="ref-for-scroll-snap-position①②"></a>

<a id="ref-for-scroll-snap④"></a>

<a id="ref-for-scroll-container①⑤"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) [snaps](#scroll-snap) to [snap positions](#scroll-snap-position) in its block axis only.

<a id="valdef-scroll-snap-type-inline"></a>inline  
<a id="ref-for-scroll-snap-position①③"></a>

<a id="ref-for-scroll-snap⑤"></a>

<a id="ref-for-scroll-container①⑥"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) [snaps](#scroll-snap) to [snap positions](#scroll-snap-position) in its inline axis only.

<a id="valdef-scroll-snap-type-both"></a>both  
<a id="ref-for-scroll-snap-position①④"></a>

<a id="ref-for-scroll-snap⑥"></a>

<a id="ref-for-scroll-container①⑦"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) [snaps](#scroll-snap) to [snap positions](#scroll-snap-position) in both of its axes independently (potentially snapping to different elements in each axis).

<a id="ref-for-valdef-scroll-snap-type-none"></a>

<a id="ref-for-valdef-scroll-snap-type-proximity①"></a>

<a id="ref-for-valdef-scroll-snap-type-mandatory"></a>

#### <a id="snap-strictness"></a>4.1.2.  Scroll Snap Strictness: the [none](#valdef-scroll-snap-type-none), [proximity](#valdef-scroll-snap-type-proximity), and [mandatory](#valdef-scroll-snap-type-mandatory) values

<a id="ref-for-valdef-scroll-snap-type-none①"></a>

<a id="ref-for-valdef-scroll-snap-type-proximity②"></a>

<a id="ref-for-valdef-scroll-snap-type-mandatory①"></a>

<a id="ref-for-scroll-snap-position①⑤"></a>

<a id="ref-for-scroll-container①⑧"></a>

The <a id="strictness-value"></a>strictness values ([none](#valdef-scroll-snap-type-none), [proximity](#valdef-scroll-snap-type-proximity), [mandatory](#valdef-scroll-snap-type-mandatory)) specify how strictly [snap positions](#scroll-snap-position) are enforced on the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) (by forcing an adjustment to the scroll position). Values are defined as follows:

<a id="valdef-scroll-snap-type-none"></a>none  
<a id="ref-for-scroll-snap⑦"></a>

<a id="ref-for-scroll-container①⑨"></a>

If specified on a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the <a id="ref-for-scroll-container②⓪"></a>scroll container must not [snap](#scroll-snap).

<a id="valdef-scroll-snap-type-mandatory"></a>mandatory  
<a id="ref-for-scroll-snap-position①⑥"></a>

<a id="ref-for-scroll-snap⑧"></a>

<a id="ref-for-scroll-container②①"></a>

If specified on a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the <a id="ref-for-scroll-container②②"></a>scroll container is required to be [snapped](#scroll-snap) to a snap position when there are no active scrolling operations. If a valid [snap position](#scroll-snap-position) exists then the scroll container must <a id="ref-for-scroll-snap⑨"></a>snap at the termination of a scroll (if none exist then no <a id="ref-for-scroll-snap①⓪"></a>snapping occurs).

<a id="valdef-scroll-snap-type-proximity"></a>proximity  
<a id="ref-for-scroll-snap①①"></a>

<a id="ref-for-scroll-container②③"></a>

If specified on a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the <a id="ref-for-scroll-container②④"></a>scroll container may [snap](#scroll-snap) to a snap position at the termination of a scroll, at the discretion of the UA given the parameters of the scroll.

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors should use mandatory snap positions with consideration of
    varyingly-sized screens and (if applicable) varying-sized content.
    In particular, although access to snapped elements larger than the scrollport
    is handled by the UA,
    if authors assign mandatory snapping to non-adjacent siblings,
    content in between can become inaccessible
    in cases where it is longer than the screen.</strong>

<a id="ref-for-scroll-container②⑤"></a>

<a id="ref-for-valdef-scroll-snap-type-none②"></a>

<a id="ref-for-propdef-scroll-snap-type⑥"></a>

<a id="ref-for-captures-snap-positions①"></a>

<a id="ref-for-containing-block-chain①"></a>

<a id="ref-for-scroll-snap-container①"></a>

<a id="ref-for-scroll-snap-position①⑦"></a>

<a id="ref-for-scroll-snap①②"></a>

A box <a id="captures-snap-positions"></a>captures snap positions if it is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) <em>or</em> has a value other than [none](#valdef-scroll-snap-type-none) for [scroll-snap-type](#propdef-scroll-snap-type). If a box’s nearest [snap-position capturing](#captures-snap-positions) ancestor on its [containing block chain](https://www.w3.org/TR/css-display-3/#containing-block-chain) is a <a id="ref-for-scroll-container②⑥"></a>scroll container with a non-<a id="ref-for-valdef-scroll-snap-type-none③"></a>none value for <a id="ref-for-propdef-scroll-snap-type⑦"></a>scroll-snap-type, that is the box’s <a id="scroll-snap-container"></a>scroll snap container. Otherwise, the box has no [scroll snap container](#scroll-snap-container), and its [snap positions](#scroll-snap-position) do not trigger [snapping](#scroll-snap).

#### <a id="re-snap"></a>4.1.3.  Re-snapping After Layout Changes

<a id="ref-for-scroll-snapport①"></a>

<a id="ref-for-scroll-container②⑦"></a>

<a id="ref-for-scroll-snap①③"></a>

<a id="ref-for-scroll-snap-position①⑧"></a>

If the content or layout of the document changes (e.g. content is added, moved, deleted, resized) such that the content of a [snapport](#scroll-snapport) changes, the UA must re-evaluate the resulting scroll position, and re-snap if required. If the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) was [snapped](#scroll-snap) before the content change and that same [snap position](#scroll-snap-position) still exists (e.g. its associated element was not deleted), the scroll container must be re-snapped to that same snap position after the content change. If multiple boxes were <a id="ref-for-scroll-snap①④"></a>snapped before and their <a id="ref-for-scroll-snap-position①⑨"></a>snap positions no longer coincide, then if one of them is focused or targeted, the <a id="ref-for-scroll-container②⑧"></a>scroll container must re-snap to that one and otherwise which one to re-snap to is UA-defined. (The UA may, for example, track which element is snapped as layout shifts align and de-align the <a id="ref-for-scroll-snap-position②⓪"></a>snap positions of other elements.)

<a id="ref-for-propdef-scroll-behavior"></a>

Scrolling required by a re-snap operation to a new or different box must behave and animate the same way as any other scroll-into-view operation, including honoring controls such as [scroll-behavior](https://www.w3.org/TR/cssom-view/#propdef-scroll-behavior). Scrolling behavior for re-snapping to the same box as before however, is UA-defined. The UA may, for example, when snapped to the start of a section, choose not to animation the scroll to the section’s new position as content is dynamically added earlier in the document in order to create the illusion of not scrolling.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-542ab8d7"></a> In the following example, the log console, when initially loaded and as each message is added to the bottom, remains snapped to the bottom of the content unless the user has scrolled away from that edge:
>
> ```css
> .log {
>   scroll-snap-type: proximity;
>   align-content: end;
> }
> .log::after {
>   display: block;
>   content: "";
>   scroll-snap-align: end;
> }
> ```
>
> <a id="ref-for-scroll-snap-area①"></a>
>
> <a id="ref-for-selectordef-after"></a>
>
> <a id="ref-for-scroll-snap-container②"></a>
>
> The rules create a single [scroll snap area](#scroll-snap-area) represented by the [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-element, positioned at the very bottom of a [scroll snap container](#scroll-snap-container). If the user scrolls “near” the bottom, the container will snap to it. If more content is dynamically added to the container, it’ll remain snapped to it (because scroll containers are required to re-snap to the same scroll snap area if it still exists after any changes). However, if the user has scrolled to somewhere else in the logs, it won’t do anything at all.

<a id="ref-for-propdef-scroll-padding②"></a>

### <a id="scroll-padding"></a>4.2. Scroll Snapport: the [scroll-padding](#propdef-scroll-padding) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-padding"></a>scroll-padding

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-comb-one⑥"></a>

\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container②⑨"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

relative to the corresponding dimension of the scroll container’s scrollport

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-valdef-scroll-padding-auto"></a>

per side, either the keyword [auto](#valdef-scroll-padding-auto) or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-scroll-container③⓪"></a>

<a id="ref-for-scroll-snap-container③"></a>

<a id="ref-for-scrollport"></a>

This property specifies (for all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container), not just [scroll snap containers](#scroll-snap-container)) offsets that define the <a id="optimal-viewing-region"></a>optimal viewing region of a scrollport: the region used as the target region for placing things in view of the user. This allows the author to exclude regions of the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) that are obscured by other content (such as fixed-positioned toolbars or sidebars) or simply to put more breathing room between a targeted element and the edges of the scrollport.

<a id="ref-for-propdef-scroll-padding③"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-propdef-padding"></a>

The [scroll-padding](#propdef-scroll-padding) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets all of the [scroll-padding-\* longhands](#longhands) in one declaration, assigning values to the longhands representing each side exactly as the [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) property does for its longhands. Values have the following meanings:

<a id="ref-for-typedef-length-percentage②"></a>

<a id="valdef-scroll-padding-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-scrollport①"></a>

<a id="ref-for-inset-properties"></a>

<a id="ref-for-fixed-position"></a>

<a id="ref-for-optimal-viewing-region①"></a>

Defines an inward offset from the corresponding edge of the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport). When applied to the root viewport, the offset is calculated and applied relative to the layout viewport (rather than the visual viewport) the same way as the corresponding [inset properties](https://drafts.csswg.org/css-logical-1/#inset-properties) on [fixed-positioned boxes](https://www.w3.org/TR/css-position-3/#fixed-position); the [optimal viewing region](#optimal-viewing-region) is the remaining area that intersects with the visual viewport.

<a id="valdef-scroll-padding-auto"></a>auto

<a id="ref-for-scrollport②"></a>

Indicates that the offset for the corresponding edge of the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) is UA-determined. This should generally default to a used length of 0px, but UAs may use heuristics to detect when a non-zero value is more appropriate.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-71c18b16"></a> For example, a UA could detect when a position:fixed element is being used as an opaque unscrollable “header” that obscures the content below it, and resolve the top offset to the height of that element so that a “page down” operation (such as pressing `PgDn`) automatically scrolls by one “visible page” of content.

<a id="ref-for-scrollport③"></a>

<a id="ref-for-optimal-viewing-region②"></a>

These offsets reduce the region of the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) that is considered “viewable” <em>for scrolling operations</em>: they have no effect on layout, on the scroll origin or initial position, or on whether or not an element is considered actually <em>visible</em>, but should affect whether an element or the caret is considered scrolled into view (e.g. for targeting or focusing operations), and reduce the amount of scrolling for paging operations (such as using the `PgUp` and `PgDn` keys or triggering equivalent operations from the scrollbar) so that within the [optimal viewing region](#optimal-viewing-region) of the <a id="ref-for-scrollport④"></a>scrollport the user sees a continuous stream of content.

<a id="ref-for-scroll-snap-container④"></a>

<a id="ref-for-alignment-container①"></a>

<a id="ref-for-scroll-snap-area②"></a>

<a id="ref-for-scroll-snap-position②①"></a>

For a [scroll snap container](#scroll-snap-container) this region also defines the <a id="scroll-snapport"></a>scroll snapport—the area of the scrollport that is used as the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) for the [scroll snap areas](#scroll-snap-area) when calculating [snap positions](#scroll-snap-position).

<a id="ref-for-propdef-scroll-padding④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-22bb626b"></a> In this example, [scroll-padding](#propdef-scroll-padding) is used to center slideshow images within the portion of the scrollport that is not obscured by a fixed-position toolbar.
>
> ```css
> html {
>     overflow-x: auto;
>     overflow-y: hidden;
>     scroll-snap-type: x mandatory;
>     scroll-padding: 0 500px 0 0;
> }
> .toolbar {
>     position: fixed;
>     height: 100%;
>     width: 500px;
>     right: 0;
> }
> img {
>     scroll-snap-align: none center;
> }
> ```
<a id="ref-for-propdef-scroll-padding⑤"></a>

<a id="ref-for-propdef-overflow①"></a>

<a id="ref-for-the-body-element①"></a>

UAs must apply the [scroll-padding](#propdef-scroll-padding) values set on the root element to the document viewport. (Note that, unlike [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), <a id="ref-for-propdef-scroll-padding⑥"></a>scroll-padding values are <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.)

## <a id="properties-on-the-elements"></a>5. Aligning Scroll Snap Areas: Properties on the elements

<a id="ref-for-propdef-scroll-margin②"></a>

### <a id="scroll-margin"></a>5.1. Scroll Snapping Area: the [scroll-margin](#propdef-scroll-margin) property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-margin"></a>scroll-margin

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range①"></a>

<a id="ref-for-length-value"></a>

[\<length\>](https://www.w3.org/TR/css-values-3/#length-value)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

per side, an absolute length

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-margin"></a>

This property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets all of the [scroll-margin-\* longhands](#longhands) in one declaration, assigning values to the longhands representing each side exactly as the [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) property does for its longhands.

<a id="ref-for-scroll-snap-area③"></a>

<a id="ref-for-scroll-container③①"></a>

Values represent outsets defining the <a id="scroll-snap-area"></a>scroll snap area that is used for snapping this box to the snapport. The [scroll snap area](#scroll-snap-area) is determined by taking the transformed border box, finding its rectangular bounding box (axis-aligned in the [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) coordinate space), then adding the specified outsets.

<a id="ref-for-scroll-snap-area④"></a>

<a id="ref-for-scroll-container③②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This ensures that the [scroll snap area](#scroll-snap-area) is always rectangular and axis-aligned to the [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) coordinate space.

<a id="ref-for-target-pseudo"></a>

<a id="ref-for-dom-element-scrollintoview"></a>

<a id="ref-for-scroll-snap-area⑤"></a>

<a id="ref-for-scrollable-overflow-region"></a>

If a page is navigated to a fragment that defines a target element (one that would be matched by [:target](https://www.w3.org/TR/selectors-4/#target-pseudo), or the target of <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>), the UA should use the element’s [scroll snap area](#scroll-snap-area), rather than just its border box, to determine which area of the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) to bring into view, <em>even when snapping is off
    or not applied on this element</em>.

<a id="ref-for-propdef-scroll-snap-align①"></a>

### <a id="scroll-snap-align"></a>5.2. Scroll Snapping Alignment: the [scroll-snap-align](#propdef-scroll-snap-align) property

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-snap-align"></a>scroll-snap-align

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range②"></a>

<a id="ref-for-comb-one⑦"></a>

\[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) start <a id="ref-for-comb-one⑧"></a>\| end <a id="ref-for-comb-one⑨"></a>\| center \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

two keywords

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-propdef-scroll-snap-align②"></a>

<a id="ref-for-scroll-snap-position②②"></a>

<a id="ref-for-scroll-snap-area⑥"></a>

<a id="ref-for-alignment-subject①"></a>

<a id="ref-for-scroll-snap-container⑤"></a>

<a id="ref-for-scroll-snapport②"></a>

<a id="ref-for-alignment-container②"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-writing-mode"></a>

The [scroll-snap-align](#propdef-scroll-snap-align) property specifies the box’s [snap position](#scroll-snap-position) as an alignment of its [snap area](#scroll-snap-area) (as the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject)) within its [snap container’s](#scroll-snap-container) [snapport](#scroll-snapport) (as the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container)). The two values specify the snapping alignment in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) and [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), respectively, as determined by the <a id="ref-for-scroll-snap-container⑥"></a>snap container’s [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode). If only one value is specified, the second value defaults to the same value.

Values are defined as follows:

<a id="valdef-scroll-snap-align-none"></a>none  
<a id="ref-for-scroll-snap-position②③"></a>

This box does not define a [snap position](#scroll-snap-position) in the specified axis.

<a id="valdef-scroll-snap-align-start"></a>start  
<a id="ref-for-scroll-snap-position②④"></a>

<a id="ref-for-scroll-snapport③"></a>

<a id="ref-for-scroll-container③③"></a>

<a id="ref-for-scroll-snap-area⑦"></a>

Start alignment of this box’s [scroll snap area](#scroll-snap-area) within the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s [snapport](#scroll-snapport) is a [snap position](#scroll-snap-position) in the specified axis.

<a id="valdef-scroll-snap-align-end"></a>end  
<a id="ref-for-scroll-snap-position②⑤"></a>

<a id="ref-for-scroll-snapport④"></a>

<a id="ref-for-scroll-container③④"></a>

<a id="ref-for-scroll-snap-area⑧"></a>

End alignment of this box’s [scroll snap area](#scroll-snap-area) within the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s [snapport](#scroll-snapport) is a [snap position](#scroll-snap-position) in the specified axis.

<a id="valdef-scroll-snap-align-center"></a>center  
<a id="ref-for-scroll-snap-position②⑥"></a>

<a id="ref-for-scroll-snapport⑤"></a>

<a id="ref-for-scroll-container③⑤"></a>

<a id="ref-for-scroll-snap-area⑨"></a>

Center alignment of this box’s [scroll snap area](#scroll-snap-area) within the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s [snapport](#scroll-snapport) is a [snap position](#scroll-snap-position) in the specified axis.

<a id="ref-for-writing-mode①"></a>

<a id="ref-for-scroll-snap-container⑦"></a>

<a id="ref-for-scroll-snap-area①⓪"></a>

<a id="ref-for-scroll-snapport⑥"></a>

<a id="ref-for-valdef-scroll-snap-align-start"></a>

Start and end alignments are resolved with respect to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [snap container](#scroll-snap-container) unless the [scroll snap area](#scroll-snap-area) is larger than the [snapport](#scroll-snapport), in which case they are resolved with respect to the <a id="ref-for-writing-mode②"></a>writing mode of the box itself. (This allows items in a container to have consistent snap alignment in general, while ensuring that [start](#valdef-scroll-snap-align-start) always aligns the item to allow reading its contents from the beginning.)

#### <a id="snap-scope"></a>5.2.1.  Scoping Valid Snap Positions to Visible Boxes

<a id="ref-for-scrollport⑤"></a>

<a id="ref-for-scroll-snap-position②⑦"></a>

<a id="ref-for-scroll-snap①⑤"></a>

<a id="ref-for-scroll-snap-area①①"></a>

<a id="ref-for-scroll-snapport⑦"></a>

Since the purpose of scroll snapping is to align content within the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) for optimal viewing, a scroll position cannot be considered a valid [snap position](#scroll-snap-position) if [snapping](#scroll-snap) to it would leave the contributing [snap area](#scroll-snap-area) entirely outside the [snapport](#scroll-snapport), even if it otherwise satisfies the required alignment of the <a id="ref-for-scroll-snap-area①②"></a>snap area.

<a id="ref-for-scroll-snap-area①③"></a>

<a id="ref-for-scroll-snapport⑧"></a>

<a id="ref-for-scroll-snap-position②⑧"></a>

<a id="ref-for-scroll-snap-area①⑤"></a>

<a id="ref-for-scroll-container③⑥"></a>

<a id="ref-for-scroll-snap①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40897215"></a> For example, a [snap area](#scroll-snap-area) is top-aligned to the [snapport](#scroll-snapport) if its top edge is coincident with the <a id="ref-for-scroll-snapport⑨"></a>snapport’s top edge; and this would be considered a valid [snap position](#scroll-snap-position) for block-axis start-aligned snapping of that <a id="ref-for-scroll-snap-area①④"></a>snap area <em>if at least part of the <a href="#scroll-snap-area">snap area</a> is on-screen</em>. If the entire <a id="ref-for-scroll-snap-area①⑥"></a>snap area is outside the <a id="ref-for-scroll-snapport①⓪"></a>snapport, however, then the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) cannot be considered to be [snapped](#scroll-snap) because the required alignment, though satisfied, would not be relevant to the viewer.
>
> ```text
> ╔════viewport════╗┈┈┈┈┈┈┈┈┌──────────────┐
> ║  ┌─────┐ ┌──┐  ║        │ top-snapping │
> ║  ├──┐  │ └──┘  ║        │   element    │
> ║  └──┴──┘       ║        │              │
> ╚════════════════╝        │              │
>                           └──────────────┘
> ```
>
> <a id="ref-for-scroll-snap①⑦"></a>
>
> Alignment of an off-screen element is not considered [snapping](#scroll-snap).

> <strong data-conversion-semantic="note">Note</strong>
>
> Why limit snapping to only when the element is visible?
>
> As the [WebKit implementers point out](https://www.webkit.org/blog/4017/scroll-snapping-with-css-snap-points/), extending a snap edge infinitely across the canvas only allows for snapping gridded layouts, and produces odd behavior for the user when off-screen elements do not align with on-screen elements. (If this requirement is onerous for implementers however, we can default to a gridded behavior and introduce a switch to get smarter behavior later.)

<a id="ref-for-propdef-scroll-snap-type⑧"></a>

<a id="ref-for-scroll-snap-position②⑨"></a>

<a id="ref-for-scroll-snap-area①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although [scroll-snap-type: both](#propdef-scroll-snap-type) evaluates [snap positions](#scroll-snap-position) independently in each axis, [choosing](#choosing) of a <a id="ref-for-scroll-snap-position③⓪"></a>snap position in one axis can be influenced by <a id="ref-for-scroll-snap-position③①"></a>snap positions in the other axis. For example, snapping in one axis may push off-screen the [snap area](#scroll-snap-area) that the other axis would otherwise align to, making its <a id="ref-for-scroll-snap-position③②"></a>snap position invalid and therefore unchooseable.

#### <a id="snap-overflow"></a>5.2.2.  Snapping Boxes that Overflow the Scrollport

<a id="ref-for-scroll-snap-area①⑧"></a>

<a id="ref-for-scroll-snapport①①"></a>

<a id="ref-for-scroll-snap-position③③"></a>

If the [snap area](#scroll-snap-area) is larger than the [snapport](#scroll-snapport) in a particular axis, then any scroll position in which the <a id="ref-for-scroll-snap-area①⑨"></a>snap area covers the <a id="ref-for-scroll-snapport①②"></a>snapport, and the distance between the geometrically previous and subsequent [snap positions](#scroll-snap-position) in that axis is larger than size of the <a id="ref-for-scroll-snapport①③"></a>snapport in that axis, is a valid <a id="ref-for-scroll-snap-position③④"></a>snap position in that axis. The UA may use the specified alignment as a more precise target for certain scroll operations (e.g. explicit paging).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f4980a0"></a> For example, take the first example in [§ 2 Motivating Examples](#examples), which had a photo as the area. The author wants mandatory snapping from item to item, but if the item happens to be larger than your viewport, you want to be able to scroll around the whole thing once you’re over it.
>
> <a id="ref-for-scroll-snap-area②⓪"></a>
>
> <a id="ref-for-scroll-snapport①④"></a>
>
> <a id="ref-for-scroll-snap-position③⑤"></a>
>
> Since the [snap area](#scroll-snap-area) is larger than the [snapport](#scroll-snapport), while the area fully fills the viewport, the container can be scrolled arbitrarily and will not try to snap back to its aligned position. However, if the container is scrolled such that the area no longer fully fills the viewport in an axis, the area resists outward scrolling until it is scrolled sufficiently to trigger snapping to a different [snap position](#scroll-snap-position).

<a id="ref-for-the-section-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2e65c5ac"></a> For another example, mandatory top-snapping on nested <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-section-element">section</a></code> elements can produce large snapping areas (from large top-level sections) potentially filled with smaller snapping areas (from the subsections). When the subsections are small enough, they snap normally; when they’re longer, the viewer can scroll arbitrarily within them, or within a large segment of the top-level section that has no subsections to snap to.
>
> ```text
> ┌─ top-level section ─┐ ━┓
> │                     │ 1┃
> │                     │  ┃
> │                     │ ━┩
> │                     │  ┆
> │                     │  ┆
> │┌─── sub-section ───┐│  ╯ ━┓
> │└───────────────────┘│    2┃
> │┌─── sub-section ───┐│ ━┓  ┃
> ││                   ││ 3┃ ━┛
> │└───────────────────┘│  ┃
> │┌─── sub-section ───┐│ ━┛ ━┓
> │└───────────────────┘│    4┃
> │┌─── sub-section ───┐│ ━┓  ┃
> ││                   ││ 5┃ ━┛
> ││                   ││  ┃
> ││                   ││ ━┩
> ││                   ││  ┆
> ││                   ││  ┆
> ││                   ││  ┆
> │└───────────────────┘│  ┆
> └─────────────────────┘  ╯
> ```
>
> In the figure above, the five numbered viewports represent the five snap positions associated with the top-level section and its four subsections. Because the first and last snap positions are part of ranges taller than the viewport, the viewer is allowed to scroll freely between the top and bottom of each range.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: If the author had instead set mandatory snap positions on the <em>headings</em> of each section (rather than the sections themselves), the contents of the first and fifth sections would be partially inaccessible to the user, as the heading snap area does not extend to cover the whole section. This is why it’s a bad idea to use mandatory snap positions on elements that might be widely spaced apart.

#### <a id="unreachable"></a>5.2.3.  Unreachable Snap Positions

<a id="ref-for-scroll-snap-position③⑥"></a>

<a id="ref-for-scroll-container③⑦"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

<a id="ref-for-scroll-snap-area②①"></a>

If a [snap position](#scroll-snap-position) is unreachable as specified, such that aligning to it would require scrolling the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s viewport past the edge of its [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region), the <em>used</em> <a id="ref-for-scroll-snap-position③⑦"></a>snap position for this [snap area](#scroll-snap-area) is the position resulting from scrolling <em>as much as possible</em> in each relevant axis toward the desired <a id="ref-for-scroll-snap-position③⑧"></a>snap position.

<a id="ref-for-propdef-scroll-snap-stop①"></a>

### <a id="scroll-snap-stop"></a>5.3. Scroll Snap Limits: the [scroll-snap-stop](#propdef-scroll-snap-stop) property

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-snap-stop"></a>scroll-snap-stop

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①⓪"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) always

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-scroll-container③⑧"></a>

<a id="ref-for-scroll-snap-position③⑨"></a>

<a id="ref-for-propdef-scroll-snap-stop②"></a>

When scrolling with an intended direction, the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) can “pass over” several possible [snap positions](#scroll-snap-position) (that would be valid to snap to, if the scrolling operation used the same direction but a lesser distance) before reaching the natural endpoint of the scroll operation and selecting its final scroll position. The [scroll-snap-stop](#propdef-scroll-snap-stop) property allows such a possible <a id="ref-for-scroll-snap-position④⓪"></a>snap position to “trap” the scrolling operation, forcing the <a id="ref-for-scroll-container③⑨"></a>scroll container to stop before the scrolling operation would naturally end.

Values are defined as follows:

<a id="valdef-scroll-snap-stop-normal"></a>normal  
<a id="ref-for-scroll-snap-position④①"></a>

<a id="ref-for-scroll-container④⓪"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) may pass over a [snap position](#scroll-snap-position) defined by this element during the execution of a scrolling operation.

<a id="valdef-scroll-snap-stop-always"></a>always  
<a id="ref-for-scroll-snap-position④②"></a>

<a id="ref-for-scroll-container④①"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) must not pass over a [snap position](#scroll-snap-position) defined by this element during the execution of a scrolling operation; it must instead snap to the first of this element’s <a id="ref-for-scroll-snap-position④③"></a>snap positions.

<a id="ref-for-intended-end-position"></a>

<a id="ref-for-scroll-snap-position④④"></a>

This property has no effect on scrolling operations with only an [intended end position](#intended-end-position), as they do not conceptually “pass over” any [snap positions](#scroll-snap-position).

## <a id="snap-concepts"></a>6. Snapping Mechanics

<a id="ref-for-scroll-snap-position④⑤"></a>

The precise model algorithm to select a [snap position](#scroll-snap-position) to snap to is intentionally left mostly undefined, so that user agents can take into account sophisticated models of user intention and interaction and adjust how they respond over time, to best serve the user.

This section defines some useful concepts to aid in discussing scroll-snapping mechanics, and provides some guidelines for what an effective scroll-snapping strategy might look like. User agents are encouraged to adapt this guidance and apply their own best judgement when defining their own snapping behavior. It also provides a small number of behavior requirements, to ensure a minimum reasonable behavior that authors can depend on when designing their interfaces with scroll-snapping in mind.

### <a id="scroll-types"></a>6.1. Types of Scrolling Methods

When a page is scrolled, the action is performed with an intended end position and/or an intended direction. Each combination of these two things defines a distinct category of scrolling, which can be treated slightly differently:

<a id="intended-end-position"></a>intended end position  
<a id="ref-for-intended-end-position①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3822aa28"></a> Common examples of scrolls with only an [intended end position](#intended-end-position) include:
> - a panning gesture, released without momentum
>
> - manipulating the scrollbar “thumb” explicitly
>
> - <a id="ref-for-dom-window-scrollto①"></a>
>
>   programmatically scrolling via APIs such as <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-scrollto">scrollTo()</a></code>
>
> - tabbing through the document’s focusable elements
>
> - navigating to an anchor within the page
>
> - homing operations such as the `Home`/`End` keys

<a id="intended-direction-and-end-position"></a>intended direction and end position  
<a id="ref-for-intended-direction-and-end-position"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f50b3650"></a> Common examples of scrolls with both an [intended direction and end position](#intended-direction-and-end-position) include:
> - a “fling” gesture, interpreted with momentum
>
> - <a id="ref-for-dom-window-scrollby"></a>
>
>   programmatically scrolling via APIs such as <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-scrollby">scrollBy()</a></code>
>
> - paging operations such as the `PgUp`/`PgDn` keys (or equivalent operations on the scrollbar)

The intended end point of the scroll prior to intervention from features such as snap points is its <a id="natural-end-point"></a>natural end-point.

<a id="intended-direction"></a>intended direction  
<a id="ref-for-intended-direction"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33cf1041"></a> Common examples of scrolls with only an [intended direction](#intended-direction) include:
> - pressing an arrow key on the keyboard (or equivalent operations on the scrollbar)
>
> - a swiping gesture interpreted as a fixed (rather than inertial) scroll

<a id="ref-for-axis-lock"></a>

Additionally, because page layouts usually align things vertically and/or horizontally, UAs sometimes <a id="axis-lock"></a>axis-lock a scroll when its direction is sufficiently vertical or horizontal. An [axis-locked](#axis-lock) scroll is bound to only scroll along that axis. This prevents less-precise input mechanisms from drifting in the non-primary axis.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification only applies to scrolling methods supported by the user agent; it does not require the user agent to support any particular input or scrolling method.

### <a id="choosing"></a>6.2. Choosing Snap Positions

<a id="ref-for-scroll-container④②"></a>

<a id="ref-for-scroll-snap-area②②"></a>

<a id="ref-for-scrollable-overflow-region②"></a>

<a id="ref-for-scroll-snap-position④⑥"></a>

A [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) can have many [snap areas](#scroll-snap-area) scattered throughout its [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region). A naïve algorithm for selecting a [snap position](#scroll-snap-position) can produce behavior that is unintuitive for users, so care is required when designing a selection algorithm. Here are a few pointers that can aid in the selection process:

- <a id="ref-for-scroll-snap-position④⑦"></a>

  <a id="ref-for-natural-end-point"></a>

  [Snap positions](#scroll-snap-position) should be chosen to minimize the distance between the end-point (or the [natural end-point](#natural-end-point)) and the final snapped scroll position, subject to the additional constraints listed in this section.

- <a id="ref-for-axis-lock①"></a>

  <a id="ref-for-scroll-snap-position④⑧"></a>

  If a scroll is [axis-locked](#axis-lock), any [snap positions](#scroll-snap-position) in the other axis should be ignored during the scroll. (However, <a id="ref-for-scroll-snap-position④⑨"></a>snap positions in the other axis can still effect the final scroll position.)

- <a id="ref-for-scroll-snap-position⑤⓪"></a>

  <a id="ref-for-scroll-snapport①⑤"></a>

  <a id="ref-for-scrollable-overflow-region③"></a>

  <a id="ref-for-intended-direction①"></a>

  <a id="ref-for-intended-end-position②"></a>

  In order to prevent a far-offscreen element from having difficult-to-understand effects on the scroll position, [snap positions](#scroll-snap-position) should be ignored if their elements are far outside of the “corridor” that the [snapport](#scroll-snapport) defines as it moves through the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region), or a hypothetical “corridor” in the direction of a scroll with only an [intended direction](#intended-direction), or the <a id="ref-for-scroll-snapport①⑥"></a>snapport after an scroll with only an [intended end position](#intended-end-position).

- <a id="ref-for-scroll-snap-position⑤①"></a>

  <a id="ref-for-valdef-scroll-snap-type-mandatory②"></a>

  User agents <em>must</em> ensure that a user can “escape” a [snap position](#scroll-snap-position), regardless of the scroll method. For example, if the snap type is [mandatory](#valdef-scroll-snap-type-mandatory) and the next <a id="ref-for-scroll-snap-position⑤②"></a>snap position is more than two screen-widths away, a naïve “always snap to nearest” selection algorithm might “trap” the user if their end position was only one screen-width away. Instead, a smarter algorithm that only returned to the starting <a id="ref-for-scroll-snap-position⑤③"></a>snap position if the end-point was a fairly small distance from it, and otherwise ignored the starting snap position, would give better behavior.

  <a id="ref-for-intended-direction②"></a>

  <a id="ref-for-scroll-snap-position⑤④"></a>

  (This implies that a scroll with only an [intended direction](#intended-direction) must always ignore the starting [snap positions](#scroll-snap-position).)

- <a id="ref-for-target-pseudo①"></a>

  <a id="ref-for-dom-element-scrollintoview①"></a>

  <a id="ref-for-scroll-snap-position⑤⑤"></a>

  <a id="ref-for-scroll-snap①⑧"></a>

  <a id="ref-for-scroll-container④③"></a>

  <a id="ref-for-scroll-snap-container⑧"></a>

  <a id="ref-for-propdef-scroll-snap-type⑨"></a>

  If a page is navigated to a fragment that defines a target element (e.g. one that would be matched by [:target](https://www.w3.org/TR/selectors-4/#target-pseudo), or the target of <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>), and that element defines some [snap positions](#scroll-snap-position), the user agent must [snap](#scroll-snap) to one of that element’s <a id="ref-for-scroll-snap-position⑤⑥"></a>snap positions if its nearest [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) is a [scroll snap container](#scroll-snap-container). The user agent <em>may</em> also do this even when the <a id="ref-for-scroll-container④④"></a>scroll container has [scroll-snap-type: none](#propdef-scroll-snap-type).

## <a id="longhands"></a>Appendix A: Longhands

The physical and logical longhands (and their shorthands) interact as defined in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1).

<a id="ref-for-propdef-scroll-padding⑦"></a>

### <a id="padding-longhands-physical"></a>Physical Longhands for [scroll-padding](#propdef-scroll-padding)

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-padding-top"></a>scroll-padding-top, <a id="propdef-scroll-padding-right"></a>scroll-padding-right, <a id="propdef-scroll-padding-bottom"></a>scroll-padding-bottom, <a id="propdef-scroll-padding-left"></a>scroll-padding-left

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-one①①"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container④⑤"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-valdef-scroll-padding-auto①"></a>

the keyword [auto](#valdef-scroll-padding-auto) or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-longhand"></a>

<a id="ref-for-propdef-scroll-padding⑧"></a>

<a id="ref-for-scroll-snapport①⑦"></a>

These [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-padding](#propdef-scroll-padding) specify the top, right, bottom, and left edges of the [snapport](#scroll-snapport), respectively. Negative values are invalid.

<a id="ref-for-propdef-scroll-padding⑨"></a>

### <a id="padding-longhands-logical"></a>Flow-relative Longhands for [scroll-padding](#propdef-scroll-padding)

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-padding-inline-start"></a>scroll-padding-inline-start, <a id="propdef-scroll-padding-block-start"></a>scroll-padding-block-start, <a id="propdef-scroll-padding-inline-end"></a>scroll-padding-inline-end, <a id="propdef-scroll-padding-block-end"></a>scroll-padding-block-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="ref-for-comb-one①②"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container④⑥"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="ref-for-valdef-scroll-padding-auto②"></a>

the keyword [auto](#valdef-scroll-padding-auto) or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-longhand①"></a>

<a id="ref-for-propdef-scroll-padding①⓪"></a>

<a id="ref-for-scroll-snapport①⑧"></a>

These [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-padding](#propdef-scroll-padding) specify the block-start, inline-start, block-end, and inline-end edges of the [snapport](#scroll-snapport), respectively. Negative values are invalid.

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-padding-block"></a>scroll-padding-block, <a id="propdef-scroll-padding-inline"></a>scroll-padding-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range③"></a>

<a id="ref-for-typedef-length-percentage⑦"></a>

<a id="ref-for-comb-one①③"></a>

\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-scroll-container④⑦"></a>

[scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-scroll-padding-block-start"></a>

<a id="ref-for-propdef-scroll-padding-block-end"></a>

<a id="ref-for-propdef-scroll-padding-inline-start"></a>

<a id="ref-for-propdef-scroll-padding-inline-end"></a>

<a id="ref-for-longhand②"></a>

<a id="ref-for-propdef-scroll-padding①①"></a>

<a id="ref-for-scroll-snapport①⑨"></a>

These [shorthands](https://www.w3.org/TR/css-cascade-5/#shorthand-property) of [scroll-padding-block-start](#propdef-scroll-padding-block-start) + [scroll-padding-block-end](#propdef-scroll-padding-block-end) and [scroll-padding-inline-start](#propdef-scroll-padding-inline-start) + [scroll-padding-inline-end](#propdef-scroll-padding-inline-end) are [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-padding](#propdef-scroll-padding), and specify the block-axis and inline-axis edges of the [snapport](#scroll-snapport), respectively.

If two values are specified, the first gives the start value and the second gives the end value.

<a id="ref-for-propdef-scroll-margin③"></a>

### <a id="margin-longhands-physical"></a>Physical Longhands for [scroll-margin](#propdef-scroll-margin)

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-margin-top"></a>scroll-margin-top, <a id="propdef-scroll-margin-right"></a>scroll-margin-right, <a id="propdef-scroll-margin-bottom"></a>scroll-margin-bottom, <a id="propdef-scroll-margin-left"></a>scroll-margin-left

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value①"></a>

[\<length\>](https://www.w3.org/TR/css-values-3/#length-value)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

absolute length

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-longhand③"></a>

<a id="ref-for-propdef-scroll-margin④"></a>

<a id="ref-for-scroll-snap-area②③"></a>

These [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-margin](#propdef-scroll-margin) specify the top, right, bottom, and left edges of the [scroll snap area](#scroll-snap-area), respectively.

<a id="ref-for-propdef-scroll-margin⑤"></a>

### <a id="margin-longhands-logical"></a>Flow-relative Longhands for [scroll-margin](#propdef-scroll-margin)

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-margin-block-start"></a>scroll-margin-block-start, <a id="propdef-scroll-margin-inline-start"></a>scroll-margin-inline-start, <a id="propdef-scroll-margin-block-end"></a>scroll-margin-block-end, <a id="propdef-scroll-margin-inline-end"></a>scroll-margin-inline-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value②"></a>

[\<length\>](https://www.w3.org/TR/css-values-3/#length-value)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

absolute length

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-longhand④"></a>

<a id="ref-for-propdef-scroll-margin⑥"></a>

<a id="ref-for-scroll-snap-area②④"></a>

These [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-margin](#propdef-scroll-margin) specify the block-start, inline-start, block-end, and inline-end edges of the [scroll snap area](#scroll-snap-area), respectively.

<strong>Table 11 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-scroll-margin-block"></a>scroll-margin-block, <a id="propdef-scroll-margin-inline"></a>scroll-margin-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range④"></a>

<a id="ref-for-length-value③"></a>

[\<length\>](https://www.w3.org/TR/css-values-3/#length-value)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-shorthand-property③"></a>

<a id="ref-for-propdef-scroll-margin-block-start"></a>

<a id="ref-for-propdef-scroll-margin-block-end"></a>

<a id="ref-for-propdef-scroll-margin-inline-start"></a>

<a id="ref-for-propdef-scroll-margin-inline-end"></a>

<a id="ref-for-longhand⑤"></a>

<a id="ref-for-propdef-scroll-margin⑦"></a>

<a id="ref-for-scroll-snap-area②⑤"></a>

These [shorthands](https://www.w3.org/TR/css-cascade-5/#shorthand-property) of [scroll-margin-block-start](#propdef-scroll-margin-block-start) + [scroll-margin-block-end](#propdef-scroll-margin-block-end) and [scroll-margin-inline-start](#propdef-scroll-margin-inline-start) + [scroll-margin-inline-end](#propdef-scroll-margin-inline-end) are [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) of [scroll-margin](#propdef-scroll-margin), and specify the block-axis and inline-axis edges of the [scroll snap area](#scroll-snap-area), respectively.

If two values are specified, the first gives the start value and the second gives the end value.

## <a id="priv-sec"></a>7. Privacy and Security Considerations

This specification does not expose any information whatsoever that is not already exposed to the DOM directly; it just makes scrolling slightly more functional. There are no new privacy or security considerations.

## <a id="acknowledgements"></a>8. Acknowledgements

Many thanks to David Baron, Simon Fraser, Håkon Wium Lie, Theresa O’Connor, François Remy, Majid Valpour, and most especially Robert O’Callahan for their proposals and recommendations, which have been incorporated into this document.

## <a id="changes"></a>9. Changes

### <a id="changes-20190319"></a>9.1. Changes Since 19 March 2019 CR

Changes since the [19 March 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-scroll-snap-1-20190319/) include:

- <a id="ref-for-propdef-scroll-snap-align③"></a>

  <a id="change-2019-clarify-writing-mode"></a> Specified which writing mode is used to resolve [scroll-snap-align](#propdef-scroll-snap-align). ([Issue 3815](https://github.com/w3c/csswg-drafts/issues/3815))

- <a id="change-2019-resnap-multiple"></a> Define requirements for re-snapping when multiple elements coincide. ([Issue 4651](https://github.com/w3c/csswg-drafts/issues/4651))

  > <a id="ref-for-scroll-container④⑧"></a>
  >
  > <a id="ref-for-scroll-snap①⑨"></a>
  >
  > <a id="ref-for-scroll-snap-position⑤⑦"></a>
  >
  > <a id="ref-for-scroll-snap②⓪"></a>
  >
  > <a id="ref-for-scroll-snap-position⑤⑧"></a>
  >
  > <a id="ref-for-scroll-container④⑨"></a>
  >
  > If the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) was [snapped](#scroll-snap) before the content change and that same [snap position](#scroll-snap-position) still exists (e.g. its associated element was not deleted), the scroll container must be re-snapped to that same snap position after the content change. <u>If multiple boxes were [snapped](#scroll-snap) before and their [snap positions](#scroll-snap-position) no longer coincide, then if one of them is focused or targeted, the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) must re-snap to that one and otherwise which one to re-snap to is UA-defined. (The UA may, for example, track which element is snapped as layout shifts align and de-align the <a id="ref-for-scroll-snap-position⑤⑨"></a>snap positions of other elements.)</u>

- <a id="change-2019-resnap-animation"></a> Require re-snapping to a new element to animate the same way as any other scroll-into-view operation. ([Issue 4609](https://github.com/w3c/csswg-drafts/issues/4609))

  > <a id="ref-for-propdef-scroll-behavior①"></a>
  >
  > <u>Scrolling required by a re-snap operation to a new or different box must behave and animate the same way as any other scroll-into-view operation, including honoring controls such as [scroll-behavior](https://www.w3.org/TR/cssom-view/#propdef-scroll-behavior). Scrolling behavior for re-snapping to the same box as before however, is UA-defined. The UA may, for example, when snapped to the start of a section, choose not to animation the scroll to the section’s new position as content is dynamically added earlier in the document in order to create the illusion of not scrolling.</u>

- <a id="ref-for-propdef-scroll-padding①②"></a>

  <a id="ref-for-propdef-scroll-snap-type①⓪"></a>

  <a id="change-2019-propagation"></a> Defined explicitly that [scroll-snap-type](#propdef-scroll-snap-type) and [scroll-padding](#propdef-scroll-padding) values are propagated from the root element to the document viewport as would be expected. ([Issue 3740](https://github.com/w3c/csswg-drafts/issues/3740))

  > <a id="ref-for-propdef-scroll-snap-type①①"></a>
  >
  > <a id="ref-for-propdef-overflow②"></a>
  >
  > <a id="ref-for-the-body-element②"></a>
  >
  > <u>UAs must apply the [scroll-snap-type](#propdef-scroll-snap-type) value set on the root element to the document viewport. Note that, unlike [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), <a id="ref-for-propdef-scroll-snap-type①②"></a>scroll-snap-type values are <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.</u>

  > <a id="ref-for-propdef-scroll-padding①③"></a>
  >
  > <a id="ref-for-propdef-overflow③"></a>
  >
  > <a id="ref-for-the-body-element③"></a>
  >
  > <u>UAs must apply the [scroll-padding](#propdef-scroll-padding) values set on the root element to the document viewport. (Note that, unlike [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), <a id="ref-for-propdef-scroll-padding①④"></a>scroll-padding values are <em>not</em> propagated from HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code>.)</u>

- <a id="ref-for-propdef-inset"></a>

  <a id="ref-for-propdef-scroll-padding①⑤"></a>

  <a id="change-2019-padding-viewport"></a> Clarified that while snap alignment is relative to the visual viewport, [scroll-padding](#propdef-scroll-padding) is resolved against the layout viewport, so that <a id="ref-for-propdef-scroll-padding①⑥"></a>scroll-padding and [inset](https://www.w3.org/TR/css-logical-1/#propdef-inset) are consistent on the root viewport. ([Issue 4393](https://github.com/w3c/csswg-drafts/issues/4393))

  > <a id="ref-for-scrollport⑥"></a>
  >
  > <a id="ref-for-inset-properties①"></a>
  >
  > <a id="ref-for-fixed-position①"></a>
  >
  > <a id="ref-for-optimal-viewing-region③"></a>
  >
  > Defines an inward offset from the corresponding edge of the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport). <u>When applied to the root viewport, the offset is calculated and applied relative to the layout viewport (rather than the visual viewport) the same way as the corresponding [inset properties](https://drafts.csswg.org/css-logical-1/#inset-properties) on [fixed-positioned boxes](https://www.w3.org/TR/css-position-3/#fixed-position); the [optimal viewing region](#optimal-viewing-region) is the remaining area that intersects with the visual viewport.</u>

- <a id="ref-for-propdef-scroll-padding-block"></a>

  <a id="ref-for-propdef-scroll-padding-inline"></a>

  <a id="change-2019-padding-applies-to"></a> Corrected the “Applies to” line for [scroll-padding-inline](#propdef-scroll-padding-inline) and [scroll-padding-block](#propdef-scroll-padding-block). ([Issue 5845](https://github.com/w3c/csswg-drafts/issues/5845))

A [Disposition of Comments](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2019) is available.

### <a id="changes-20190131"></a>9.2. Changes Since 31 January 2019 CR

Changes since the [31 January 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-scroll-snap-1-20190131/) include:

- <a id="ref-for-propdef-scroll-margin⑧"></a>

  <a id="ref-for-propdef-scroll-padding①⑦"></a>

  <a id="change-2019-clarify-nonsnapping"></a> Emphasized that [scroll-padding](#propdef-scroll-padding) and [scroll-margin](#propdef-scroll-margin) do apply even when scroll snapping is off. ([Issue 3721](https://github.com/w3c/csswg-drafts/issues/3721))

  > <a id="ref-for-propdef-scroll-padding①⑧"></a>
  >
  > <a id="ref-for-scroll-container⑤⓪"></a>
  >
  > <a id="ref-for-optimal-viewing-region④"></a>
  >
  > <a id="ref-for-propdef-scroll-margin⑨"></a>
  >
  > <u>Also, to offer better control over paging and scroll positioning even when snapping is off, this module defines the [scroll-padding](#propdef-scroll-padding) property for use on all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container), to adjust the <a id="ref-for-scroll-container⑤①"></a>scroll container’s [optimal viewing region](#optimal-viewing-region) for the purpose of paging and scroll-into-view operations; similarly the [scroll-margin](#propdef-scroll-margin) property can be used on any box to adjust its visual area for the purpose of scroll-into-view operations.</u>

  > <a id="ref-for-scroll-container⑤②"></a>
  >
  > <a id="ref-for-scroll-snap-container⑨"></a>
  >
  > <a id="ref-for-optimal-viewing-region⑤"></a>
  >
  > This property specifies <u>(for all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container), not just [scroll snap containers](#scroll-snap-container))</u> offsets that define the [optimal viewing region](#optimal-viewing-region) of a scrollport…

  > <a id="ref-for-target-pseudo②"></a>
  >
  > <a id="ref-for-dom-element-scrollintoview②"></a>
  >
  > <a id="ref-for-scroll-snap-area②⑥"></a>
  >
  > <a id="ref-for-scrollable-overflow-region④"></a>
  >
  > If a page is navigated to a fragment that defines a target element (one that would be matched by [:target](https://www.w3.org/TR/selectors-4/#target-pseudo), or the target of <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>), the UA should use the element’s [scroll snap area](#scroll-snap-area), rather than just its border box, to determine which area of the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) to bring into view <u>, <em>even when snapping is off
				or not applied on this element</em></u> .

### <a id="changes-20180814"></a>9.3. Changes Since 14 August 2018 CR

Changes since the [14 August 2018 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-scroll-snap-1-20180814/) include:

- <a id="ref-for-valdef-scroll-padding-auto③"></a>

  <a id="ref-for-propdef-scroll-padding①⑨"></a>

  <a id="change-2018-padding-initial"></a> Corrected [scroll-padding](#propdef-scroll-padding) longhands to list the new [auto](#valdef-scroll-padding-auto) keyword in their property definition tables. ([Issue 3189](https://github.com/w3c/csswg-drafts/issues/3189))

- <a id="change-2018-computed-animation"></a> Fixed up “Computed value” and “Animation type” lines in the property definition tables.

- <a id="ref-for-propdef-scroll-margin①⓪"></a>

  <a id="ref-for-percentage-value"></a>

  <a id="change-2018-margin-percentage"></a> Cleaned up stray [\<percentage\>](https://www.w3.org/TR/css-values-3/#percentage-value) values in [scroll-margin](#propdef-scroll-margin) property definition tables. ([3289](https://github.com/w3c/csswg-drafts/issues/3289))

A [Disposition of Comments is available](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2018).

### <a id="changes-20171214"></a>9.4. Changes Since 14 December 2017 CR

Changes since the [14 December 2017 Candidate Recommendation](https://www.w3.org/TR/2017/CR-css-scroll-snap-1-20171214/) include:

- <a id="ref-for-propdef-scroll-snap-align④"></a>

  <a id="change-2017-align-values-backwards"></a> Fixed [scroll-snap-align](#propdef-scroll-snap-align) shorthand to assign block-axis value first, inline-axis value second, accordingly to logical shorthand conventions. ([Issue 2232](https://github.com/w3c/csswg-drafts/issues/2232)

- <a id="ref-for-valdef-scroll-padding-auto④"></a>

  <a id="change-2017-scroll-padding-auto"></a> Added [auto](#valdef-scroll-padding-auto) keyword to 'scroll-padding as its initial value to account for UA heuristics. ([Issue 2728](https://github.com/w3c/csswg-drafts/issues/2728)

- <a id="ref-for-dom-window-scrollto③"></a>

  <a id="ref-for-scroll-snap-position⑥⓪"></a>

  <a id="ref-for-scroll-container⑤③"></a>

  <a id="ref-for-propdef-scroll-snap-type①④"></a>

  <a id="ref-for-dom-window-scrollto②"></a>

  <a id="ref-for-propdef-scroll-snap-type①③"></a>

  <a id="change-2017-snap-vs-api-scroll"></a> Clarified in the definition of [scroll-snap-type](#propdef-scroll-snap-type) that programmatic scrolls such as <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-scrollto">scrollTo()</a></code> are also subject to snapping. ([Issue 2593](https://github.com/w3c/csswg-drafts/issues/2593))

  > Using the [scroll-snap-type](#propdef-scroll-snap-type) property on the relevant [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the author can request a particular bias for the scrollport to land on a [snap position](#scroll-snap-position) after scrolling operations <u>(including programmatic scrolls such as the <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-scrollto">scrollTo()</a></code> method)</u> .

- <a id="change-2017-clarify-mandatory-visibility"></a> Adjusted wording in [§ 5.2.1 Scoping Valid Snap Positions to Visible Boxes](#snap-scope) to be clearer—compared to [old version](https://www.w3.org/TR/2017/CR-css-scroll-snap-1-20171214/#snap-scope). ([Issue 2526](https://github.com/w3c/csswg-drafts/issues/2526))

A [Disposition of Comments](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2017-12) is available.

### <a id="changes-20170824"></a>9.5. Changes Since 24 August 2017 CR

Changes since the [24 August 2017 Candidate Recommendation](https://www.w3.org/TR/2017/CR-css-scroll-snap-1-20170824/) include:

- <a id="ref-for-propdef-scroll-margin①①"></a>

  <a id="ref-for-dom-element-scrollintoview③"></a>

  <a id="ref-for-target-pseudo③"></a>

  <a id="change-2017-scroll-margin"></a> [:target](https://www.w3.org/TR/selectors-4/#target-pseudo)/<code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>/etc should take [scroll-margin](#propdef-scroll-margin) into account, regardless of whether snapping is turned on or not. ([Issue 1](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2016-08#issue-1)

  > <a id="ref-for-target-pseudo④"></a>
  >
  > <a id="ref-for-dom-element-scrollintoview④"></a>
  >
  > <a id="ref-for-scroll-snap-area②⑦"></a>
  >
  > <a id="ref-for-scrollable-overflow-region⑤"></a>
  >
  > <u>If a page is navigated to a fragment that defines a target element (one that would be matched by [:target](https://www.w3.org/TR/selectors-4/#target-pseudo), or the target of <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>), the UA should use the element’s [scroll snap area](#scroll-snap-area), rather than just its border box, to determine which area of the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) to bring into view.</u>

- <a id="ref-for-dom-element-scrollintoview⑤"></a>

  <a id="ref-for-target-pseudo⑤"></a>

  <a id="change-2017-target-snap-must"></a> [:target](https://www.w3.org/TR/selectors-4/#target-pseudo)/<code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>/etc must (rather than should) use snap positions if snapping is turned on. ([Issue 1](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2016-08#issue-1)

  > <a id="ref-for-target-pseudo⑥"></a>
  >
  > <a id="ref-for-dom-element-scrollintoview⑥"></a>
  >
  > <a id="ref-for-scroll-snap-position⑥①"></a>
  >
  > <a id="ref-for-scroll-snap②①"></a>
  >
  > <a id="ref-for-scroll-container⑤④"></a>
  >
  > <a id="ref-for-scroll-snap-container①⓪"></a>
  >
  > <a id="ref-for-scroll-container⑤⑤"></a>
  >
  > <a id="ref-for-propdef-scroll-snap-type①⑤"></a>
  >
  > If a page is navigated to a fragment that defines a target element (one that would be matched by [:target](https://www.w3.org/TR/selectors-4/#target-pseudo), or the target of <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>), and that element defines some [snap positions](#scroll-snap-position), the user agent ~~should~~ <u>must</u> [snap](#scroll-snap) to one of that element’s <a id="ref-for-scroll-snap-position⑥②"></a>snap positions <u>if its nearest [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) is a [scroll snap container](#scroll-snap-container)</u> . The user agent <em>may</em> also do this even when the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scroll-snap-type: none](#propdef-scroll-snap-type).

- <a id="ref-for-propdef-scroll-margin①②"></a>

  <a id="change-2017-rename-scroll-margin"></a> Renamed scroll-snap-margin to [scroll-margin](#propdef-scroll-margin) to reflect its more generic role in providing breathing space for scrolling to an element regardless of snapping behavior. ([Issue 4](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2016-08#issue-4))

A [Disposition of Comments](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2016-08) is available.

### <a id="changes-20161020"></a>9.6. Changes Since 20 October 2016 CR

Changes since the [20 October 2016 Candidate Recommendation](https://www.w3.org/TR/2016/CR-css-scroll-snap-1-20161020/) include:

- <a id="ref-for-propdef-scroll-padding②⓪"></a>

  Restricted [scroll-padding](#propdef-scroll-padding) to non-negative values. ([Issue 1084](https://github.com/w3c/csswg-drafts/issues/1084))

  > <a id="ref-for-propdef-padding①"></a>
  >
  > Values <u>must be non-negative and</u> are interpreted as for [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) …

- Added paging and homing operations to examples in [§ 6.1 Types of Scrolling Methods](#scroll-types). ([Issue 1605](https://github.com/w3c/csswg-drafts/issues/1605))

- Clarified that snapping in one axis may affect whether snapping to a particular snap area is possible in the other axis. ([Issue 950](https://github.com/w3c/csswg-drafts/issues/950))

  > <a id="ref-for-propdef-scroll-snap-type①⑥"></a>
  >
  > <a id="ref-for-scroll-snap-position⑥③"></a>
  >
  > <a id="ref-for-scroll-snap-area②⑧"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Although [scroll-snap-type: both](#propdef-scroll-snap-type) evaluates [snap positions](#scroll-snap-position) independently in each axis, [choosing](#choosing) of a <a id="ref-for-scroll-snap-position⑥④"></a>snap position in one axis may be influenced by <a id="ref-for-scroll-snap-position⑥⑤"></a>snap positions in the other axis. For example, snapping in one axis may push off-screen the [snap area](#scroll-snap-area) that the other axis would otherwise align to, making its <a id="ref-for-scroll-snap-position⑥⑥"></a>snap position invalid and therefore unchooseable.

- <a id="ref-for-propdef-scroll-margin①③"></a>

  <a id="ref-for-propdef-scroll-padding②①"></a>

  Clarified how the [scroll-padding](#propdef-scroll-padding) and [scroll-margin](#propdef-scroll-margin) shorthands assign values to their longhands. ([Issue 1050](https://github.com/w3c/csswg-drafts/issues/1050))

- Clarified that scroll snapping does not mandate any particular input method. ([Issue 1305](https://github.com/w3c/csswg-drafts/issues/1305))

  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > This specification only applies to scrolling methods supported by the user agent; it does not require the user agent to support any particular input or scrolling method.

- <a id="ref-for-propdef-scroll-snap-stop③"></a>

  Clarified the intended effects of [scroll-snap-stop](#propdef-scroll-snap-stop) on various scrolling operations. ([Issue 1552](https://github.com/w3c/csswg-drafts/issues/1552))

- <a id="ref-for-scroll-snap-container①①"></a>

  <a id="ref-for-scroll-snap-position⑥⑦"></a>

  <a id="ref-for-propdef-scroll-snap-stop④"></a>

  Clarified that [scroll-snap-stop](#propdef-scroll-snap-stop) is applied to the [snap positions](#scroll-snap-position) defined by the element, not applied to all <a id="ref-for-scroll-snap-position⑥⑧"></a>snap positions in the [scroll snap container](#scroll-snap-container).

- <a id="ref-for-propdef-scroll-snap-type①⑦"></a>

  Fixed some syntax errors in examples and added a new one to the [scroll-snap-type](#propdef-scroll-snap-type) section. ([Issue 827](https://github.com/w3c/csswg-drafts/issues/827))

A [Disposition of Comments](https://drafts.csswg.org/css-scroll-snap-1/issues-cr-2016) is available.

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

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

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

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

- [always](#valdef-scroll-snap-stop-always), in §5.3
- [auto](#valdef-scroll-padding-auto), in §4.2
- [axis-lock](#axis-lock), in §6.1
- [axis value](#axis-value), in §4.1.1
- [block](#valdef-scroll-snap-type-block), in §4.1.1
- [both](#valdef-scroll-snap-type-both), in §4.1.1
- [captures snap positions](#captures-snap-positions), in §4.1.2
- [center](#valdef-scroll-snap-align-center), in §5.2
- [end](#valdef-scroll-snap-align-end), in §5.2
- [inline](#valdef-scroll-snap-type-inline), in §4.1.1
- [intended direction](#intended-direction), in §6.1
- [intended direction and end position](#intended-direction-and-end-position), in §6.1
- [intended end position](#intended-end-position), in §6.1
- [\<length-percentage\>](#valdef-scroll-padding-length-percentage), in §4.2
- [mandatory](#valdef-scroll-snap-type-mandatory), in §4.1.2
- [natural end-point](#natural-end-point), in §6.1
- none
  - [value for scroll-snap-align](#valdef-scroll-snap-align-none), in §5.2
  - [value for scroll-snap-type](#valdef-scroll-snap-type-none), in §4.1.2
- [normal](#valdef-scroll-snap-stop-normal), in §5.3
- [optimal viewing region](#optimal-viewing-region), in §4.2
- [proximity](#valdef-scroll-snap-type-proximity), in §4.1.2
- [re-snap](#re-snap), in §4.1.2
- [scroll-margin](#propdef-scroll-margin), in §5.1
- [scroll-margin-block](#propdef-scroll-margin-block), in §Unnumbered section
- [scroll-margin-block-end](#propdef-scroll-margin-block-end), in §Unnumbered section
- [scroll-margin-block-start](#propdef-scroll-margin-block-start), in §Unnumbered section
- [scroll-margin-bottom](#propdef-scroll-margin-bottom), in §Unnumbered section
- [scroll-margin-inline](#propdef-scroll-margin-inline), in §Unnumbered section
- [scroll-margin-inline-end](#propdef-scroll-margin-inline-end), in §Unnumbered section
- [scroll-margin-inline-start](#propdef-scroll-margin-inline-start), in §Unnumbered section
- [scroll-margin-left](#propdef-scroll-margin-left), in §Unnumbered section
- [scroll-margin-right](#propdef-scroll-margin-right), in §Unnumbered section
- [scroll-margin-top](#propdef-scroll-margin-top), in §Unnumbered section
- [scroll-padding](#propdef-scroll-padding), in §4.2
- [scroll-padding-block](#propdef-scroll-padding-block), in §Unnumbered section
- [scroll-padding-block-end](#propdef-scroll-padding-block-end), in §Unnumbered section
- [scroll-padding-block-start](#propdef-scroll-padding-block-start), in §Unnumbered section
- [scroll-padding-bottom](#propdef-scroll-padding-bottom), in §Unnumbered section
- [scroll-padding-inline](#propdef-scroll-padding-inline), in §Unnumbered section
- [scroll-padding-inline-end](#propdef-scroll-padding-inline-end), in §Unnumbered section
- [scroll-padding-inline-start](#propdef-scroll-padding-inline-start), in §Unnumbered section
- [scroll-padding-left](#propdef-scroll-padding-left), in §Unnumbered section
- [scroll-padding-right](#propdef-scroll-padding-right), in §Unnumbered section
- [scroll-padding-top](#propdef-scroll-padding-top), in §Unnumbered section
- [scroll snap](#scroll-snap), in §3
- [scroll-snap-align](#propdef-scroll-snap-align), in §5.2
- [scroll snap area](#scroll-snap-area), in §5.1
- [scroll snap container](#scroll-snap-container), in §4.1.2
- [scroll snapport](#scroll-snapport), in §4.2
- [scroll snap position](#scroll-snap-position), in §3
- [scroll-snap-stop](#propdef-scroll-snap-stop), in §5.3
- [scroll-snap-type](#propdef-scroll-snap-type), in §4.1
- [snap](#scroll-snap), in §3
- [snap area](#scroll-snap-area), in §5.1
- [snap container](#scroll-snap-container), in §4.1.2
- [snapport](#scroll-snapport), in §4.2
- [snap position](#scroll-snap-position), in §3
- [start](#valdef-scroll-snap-align-start), in §5.2
- [strictness value](#strictness-value), in §4.1.2
- [x](#valdef-scroll-snap-type-x), in §4.1.1
- [y](#valdef-scroll-snap-type-y), in §4.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-align-3\] defines the following terms:
  - <a id="term-for-alignment-container"></a>alignment container
  - <a id="term-for-alignment-subject"></a>alignment subject
- \[css-box-4\] defines the following terms:
  - <a id="term-for-propdef-margin"></a>margin
  - <a id="term-for-propdef-padding"></a>padding
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-longhand"></a>longhand
  - <a id="term-for-shorthand-property"></a>shorthand
  - <a id="term-for-shorthand-property①"></a>shorthand property
- \[css-display-3\] defines the following terms:
  - <a id="term-for-containing-block-chain"></a>containing block chain
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="term-for-propdef-inset"></a>inset
  - <a id="term-for-inset-properties"></a>inset properties
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-scroll-container"></a>scroll container
  - <a id="term-for-scrollable-overflow-region"></a>scrollable overflow area
  - <a id="term-for-scrollport"></a>scrollport
- \[css-position-3\] defines the following terms:
  - <a id="term-for-fixed-position"></a>fixed-positioned box
- \[css-pseudo-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-first-letter"></a>::first-letter
  - <a id="term-for-selectordef-first-line"></a>::first-line
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
  - <a id="term-for-inline-axis"></a>inline axis
  - <a id="term-for-writing-mode"></a>writing mode
- \[cssom-view-1\] defines the following terms:
  - <a id="term-for-propdef-scroll-behavior"></a>scroll-behavior
  - <a id="term-for-dom-window-scrollby"></a>scrollBy()
  - <a id="term-for-dom-element-scrollintoview"></a>scrollIntoView()
  - <a id="term-for-dom-window-scrollto"></a>scrollTo()
- \[HTML\] defines the following terms:
  - <a id="term-for-the-body-element"></a>body
  - <a id="term-for-the-section-element"></a>section
- \[selectors-4\] defines the following terms:
  - <a id="term-for-target-pseudo"></a>:target

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 19 January 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; et al. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 19 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 11 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

## <a id="property-index"></a>Property Index

<strong>Table 12 — structured row/cell transcription</strong>

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

Anim­ation type

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin①④"></a>

[scroll-margin](#propdef-scroll-margin)

<strong>Column 2 (data cell):</strong>

\<length\>{1,4}

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

per side, an absolute length

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-block"></a>

[scroll-margin-block](#propdef-scroll-margin-block)

<strong>Column 2 (data cell):</strong>

\<length\>{1,2}

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-block-end①"></a>

[scroll-margin-block-end](#propdef-scroll-margin-block-end)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-block-start①"></a>

[scroll-margin-block-start](#propdef-scroll-margin-block-start)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-bottom"></a>

[scroll-margin-bottom](#propdef-scroll-margin-bottom)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-inline"></a>

[scroll-margin-inline](#propdef-scroll-margin-inline)

<strong>Column 2 (data cell):</strong>

\<length\>{1,2}

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-inline-end①"></a>

[scroll-margin-inline-end](#propdef-scroll-margin-inline-end)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-inline-start①"></a>

[scroll-margin-inline-start](#propdef-scroll-margin-inline-start)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-left"></a>

[scroll-margin-left](#propdef-scroll-margin-left)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 11</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-right"></a>

[scroll-margin-right](#propdef-scroll-margin-right)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 12</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-margin-top"></a>

[scroll-margin-top](#propdef-scroll-margin-top)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length

<strong>Row 13</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding②②"></a>

[scroll-padding](#propdef-scroll-padding)

<strong>Column 2 (data cell):</strong>

\[ auto \| \<length-percentage\> \]{1,4}

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the corresponding dimension of the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

per side, either the keyword auto or a computed \<length-percentage\> value

<strong>Row 14</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-block①"></a>

[scroll-padding-block](#propdef-scroll-padding-block)

<strong>Column 2 (data cell):</strong>

\[ auto \| \<length-percentage\> \]{1,2}

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 15</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-block-end①"></a>

[scroll-padding-block-end](#propdef-scroll-padding-block-end)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 16</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-block-start①"></a>

[scroll-padding-block-start](#propdef-scroll-padding-block-start)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 17</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-bottom"></a>

[scroll-padding-bottom](#propdef-scroll-padding-bottom)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 18</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-inline①"></a>

[scroll-padding-inline](#propdef-scroll-padding-inline)

<strong>Column 2 (data cell):</strong>

\[ auto \| \<length-percentage\> \]{1,2}

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 19</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-inline-end①"></a>

[scroll-padding-inline-end](#propdef-scroll-padding-inline-end)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 20</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-inline-start①"></a>

[scroll-padding-inline-start](#propdef-scroll-padding-inline-start)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 21</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-left"></a>

[scroll-padding-left](#propdef-scroll-padding-left)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 22</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-right"></a>

[scroll-padding-right](#propdef-scroll-padding-right)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 23</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-padding-top"></a>

[scroll-padding-top](#propdef-scroll-padding-top)

<strong>Column 2 (data cell):</strong>

auto \| \<length-percentage\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

scroll containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the scroll container’s scrollport

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or a computed \<length-percentage\> value

<strong>Row 24</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-snap-align⑤"></a>

[scroll-snap-align](#propdef-scroll-snap-align)

<strong>Column 2 (data cell):</strong>

\[ none \| start \| end \| center \]{1,2}

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

two keywords

<strong>Row 25</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-snap-stop⑤"></a>

[scroll-snap-stop](#propdef-scroll-snap-stop)

<strong>Column 2 (data cell):</strong>

normal \| always

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

<strong>Row 26</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-scroll-snap-type①⑧"></a>

[scroll-snap-type](#propdef-scroll-snap-type)

<strong>Column 2 (data cell):</strong>

none \| \[ x \| y \| block \| inline \| both \] \[ mandatory \| proximity \]?

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword(s)
