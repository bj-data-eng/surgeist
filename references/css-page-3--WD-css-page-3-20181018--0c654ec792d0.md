Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Paged Media Module Level 3](https://www.w3.org/TR/2018/WD-css-page-3-20181018/).

Original copyright notice: Copyright © 2018 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Paged Media Module Level 3

Source snapshot: https://www.w3.org/TR/2018/WD-css-page-3-20181018/

Snapshot SHA-256: 0c654ec792d0d0d451d7bd420903c6aefe68febb7f3bfc2db58de5490740471d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 3 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Paged Media Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2018 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module specifies how pages are generated and laid out to hold fragmented content in a paged presentation. It adds functionality for controlling page margins, page size and orientation, and headers and footers, and extends generated content to enable page numbering and running headers / footers. The process of paginating a flow into such generated pages is covered in [\[CSS3-BREAK\]](#biblio-css3-break).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of&#xA;   its publication. Other documents may supersede this document. A list of&#xA;   current W3C publications and the latest revision of this technical report&#xA;   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports&#xA;   index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “css-page” in the title, preferably like this: “\[css-page\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) (part of the [Style Activity](https://www.w3.org/Style/)).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 February 2018 W3C Process Document](https://www.w3.org/2018/Process-20180201/).

## <a id="intro"></a>1.  Introduction

Paged media (e.g., paper, transparencies, photo album pages, pages displayed on computer screens as printed output simulations) differ from [continuous media](https://www.w3.org/TR/CSS2/media.html#continuous-media-group) in that the content of the document is split into one or more discrete static display surfaces. To handle pages, CSS3 Paged Media describes how:

- [page breaks](#page-breaks) are created and avoided;
- the page properties such as size, orientation, margins, border, and padding are specified;
- headers and footers are established within the page margins;
- content such as page counters are placed in the headers and footers; and
- orphans and widows can be controlled.

This module defines a [page model](#page-model) that specifies how a document is formatted within a rectangular area, called the [page box](#page-box-page-rule), that has finite width and height.

Although CSS3 does not specify how user agents transfer page boxes to sheets, it does include certain mechanisms for telling user agents about the intended page sheet [size and orientation](#page-size). In the general case, CSS3 assumes that one page box will be transferred to one surface of similar size.

All properties defined in this specification also accept the [inherit](https://www.w3.org/TR/CSS21/cascade.html#value-def-inherit) keyword as their value, but for readability it has not been listed explicitly.

## <a id="page-terms"></a>2.  Page Terminology

The following terminology and accompanying diagrams help to describe the page model:

<a id="page-sheet"></a>Page sheet  
![The corner of a page sheet with the non-printable area at the edge and printable area inside it](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/PageSheet.png) The page sheet is one surface of the physical medium. The illustration to the right shows a representation of the upper-left corner of a page sheet.

<a id="printable-area"></a>Printable and non-printable areas  
The non-printable area is the area of a page sheet that a physical device such as a printer is not capable of marking reliably, usually due to the printer’s paper handling mechanism. This value is printer dependent and is usually a small region along each edge of the page sheet. The printable area is the area of page sheet that a printer <em>is</em> capable of marking reliably. The size of the printable area is the size of the page sheet reduced by the size of the non-printable area. A user agent may not know the dimensions of this area for a particular printing device; but when its dimensions are known, user agents may adjust the formatting of the document so that content falls within the printable area. How this adjustment is accomplished is device dependent within the constraints expressed in the sections [§7.4 Rendering page boxes that do not fit a page sheet](#renderingpages) and [§3.2 Content outside the page box](#content-outside-box).

<a id="page-orientation"></a>Page Orientation  
<a id="ref-for-valdef-page-size-landscape"></a>

<a id="ref-for-valdef-page-size-portrait"></a>

<a id="ref-for-page-box"></a>

The page orientation is defined by comparing the length of the edges of a [page box](#page-box). The page box is a rectangle with two perpendicular edges called the long edge and the short edge. The length of the long edge is always greater than or equal to the length of the short edge. When the page box is square, the two edges are of the same length and either can be used as the long edge with the other being the short edge. This specification defines page orientations of [portrait](#valdef-page-size-portrait) and [landscape](#valdef-page-size-landscape).

<a id="portrait"></a>Portrait Orientation  
A portrait page’s height is greater than or equal to its width. Horizontal elements are parallel to the short edge and vertical elements to the long edge.

<a id="landscape"></a>Landscape Orientation  
A landscape page’s width is greater than or equal to its height. Horizontal elements are parallel to the long edge and vertical elements to the short edge.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that CSS3 makes no distinction between landscape and reverse-landscape orientations. However, future versions of CSS may do so. UAs should consider, when formatting for duplexed printing, the binding edge, page progression, and ease of reading when choosing between landscape and reverse-landscape renderings.

<a id="duplex-printing"></a>Duplex Printing  
Duplex printing prints one page box per side of a page sheet and uses both sides of the page sheet. This module provides no ability to specify whether a document is duplex printed, but the concept of left and right pages is based on the assumption that the document is duplex printed, regardless of whether or not it actually is.

<a id="binding-edge"></a>Binding Edge  
The binding edge is the edge of the page box that is toward the binding if the material is bound. The binding edge often has a larger margin than the opposite edge to provide for the space used by the binding. The binding edge can be any of the four edges. However, page sheets are customarily bound so that the binding edge of page boxes with portrait orientation is vertical. This module provides no method to specify the binding edge. In duplex printing, the binding edge is on opposite sides of the page box for the left and right pages.

<a id="facing-pages"></a>Facing Pages  
Facing pages are two sequential pages such that when the document is duplex printed they are on separate sheets of paper. Typically, the earlier page will be the back side of one sheet and the later page will be the front side of another. They are usually laid out so that the binding edges of facing pages are vertical and adjacent when the pages are placed in their normal reading orientation.

<a id="left-page"></a>Left Page  
<a id="ref-for-page-selector"></a>

<a id="ref-for-valdef-page-left"></a>

A page that would be on the left if it is part of a pair of facing pages as typically laid out. Page layouts for documents using a left-to-right page progression have the earlier of the facing pages on the left. Rules specific to the left page can be specified using the [:left](#valdef-page-left) [page selector](#page-selector).

<a id="right-page"></a>Right Page  
<a id="ref-for-page-selector①"></a>

<a id="ref-for-valdef-page-right"></a>

A page that would be on the right if it is part of a pair of facing pages as typically laid out. Page layouts for documents using a right-to-left page progression have the earlier of the facing pages on the right. Rules specific to the right page can be specified using the [:right](#valdef-page-right) [page selector](#page-selector).

## <a id="page-model"></a>3. The Page Model

In the paged media formatting model, the document is transferred into one or more page boxes. The <a id="page-box"></a>page box is a specialized CSS box that maps to a rectangular print media surface, such as a page of paper. It is roughly analogous to the [viewport](https://www.w3.org/TR/CSS21/visuren.html#viewport). ![](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/PageBox.png)

As with other CSS [boxes](https://www.w3.org/TR/CSS21/box.html), a page box consists of margin, border, padding, and content areas. The content and margin areas of a page box have special functions:

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-16b9253a"></a> In CSS 2.1, both the page box and page area are simple rectangles. Neither is a CSS box with margins, borders, and padding. This CSS box should be distinct from the page box and page area, which would be its margin area and content area, respectively. Naming ideas?

- <a id="ref-for-containing-block"></a>

  The content area of a page box is called the <a id="page-area"></a>page area. The content of the document is flowed into one or more page boxes. The page area acts as a container for all the boxes generated by the root element and its descendants that are laid out within a given page box. The edges of the page area on the first page establish the rectangle that is the initial [containing block](#containing-block) of the document.

- The margin area of a page box is divided into 16 <a id="page-margin-boxes"></a>page-margin boxes. Each page-margin box has its own margin, border, padding and content areas. Page-margin boxes are typically used to display running headers and footers.

<a id="ref-for-page-box①"></a>

<a id="ref-for-page-context"></a>

<a id="ref-for-at-ruledef-page"></a>

<a id="ref-for-page-margin-boxes"></a>

<a id="ref-for-margin-context"></a>

The properties of a [page box](#page-box) are determined by properties declared within the [page context](#page-context), which is the [declaration block](https://www.w3.org/TR/CSS21/syndata.html#x14) of the [@page](#at-ruledef-page) rule. Similarly the properties of a [page-margin box](#page-margin-boxes) are determined by properties declared within its [margin context](#margin-context). Declarations in the page context can affect the page box and/or inherit to the page-margin boxes, but they do not apply to or inherit into the document’s root element or other content.

<a id="ref-for-containing-block①"></a>

<a id="ref-for-descdef-page-size"></a>

<a id="ref-for-page-context①"></a>

The [containing block](#containing-block) of the page box is specified using the [size](#descdef-page-size) property in the [page context](#page-context). The width and horizontal margins of the page box are then calculated exactly as for a [non-replaced block element in normal flow](https://www.w3.org/TR/CSS21/visudet.html#blockwidth). [\[CSS21\]](#biblio-css21) The height and vertical margins of the page box are calculated analogously (instead of using the block height formulas). In both cases if the values are over-constrained, instead of ignoring any margins, the containing block is resized to coincide with the margin edges of the page box.

### <a id="painting"></a>3.1.  Page Backgrounds and Painting Order

When drawing a page of content, the page layers are painted in the following painting order (bottommost first):

1.  page background
2.  document canvas
3.  page borders
4.  document contents
5.  page-margin boxes

<a id="ref-for-background-painting-area"></a>

<a id="ref-for-bleed-area"></a>

<a id="ref-for-propdef-background-clip"></a>

<a id="ref-for-propdef-background-origin"></a>

<a id="ref-for-propdef-background-attachment"></a>

<a id="ref-for-valdef-background-attachment-fixed"></a>

<a id="ref-for-background-positioning-area"></a>

In the page model, the page background behaves similar to the root background: its [background painting area](https://www.w3.org/TR/css3-background/#background-painting-area) is the [bleed area](#bleed-area), which covers the entire page box, including its margins (regardless of [background-clip](https://www.w3.org/TR/css3-background/#propdef-background-clip)). Page backgrounds are anchored within the page box’s padding area by default (and honor [background-origin](https://www.w3.org/TR/css3-background/#propdef-background-origin) if the UA supports [\[CSS3BG\]](#biblio-css3bg)). However if [background-attachment](https://www.w3.org/TR/css3-background/#propdef-background-attachment) is [fixed](https://www.w3.org/TR/css3-background/#valdef-background-attachment-fixed) then the image is positioned relative to the page box including its margins (i.e. the [background positioning area](https://www.w3.org/TR/css3-background/#background-positioning-area) is the page’s margin box).

<a id="ref-for-background-painting-area①"></a>

<a id="ref-for-propdef-background-clip①"></a>

The document canvas background is drawn as the page box’s background: by default its [background painting area](https://www.w3.org/TR/css3-background/#background-painting-area) covers the page box’s border box, and for UAs that support [\[CSS3BG\]](#biblio-css3bg), follows the [background-clip](https://www.w3.org/TR/css3-background/#propdef-background-clip) value specified on the root element. It remains, however, positioned with respect to the root element or page area as usual.

<a id="ref-for-stacking-context"></a>

With respect to the page-margin boxes, the document canvas, page borders, and all of the document contents are treated as a single element with a `z-index` value of 0 that establishes a [stacking context](https://www.w3.org/TR/css3-positioning/#stacking-context) [\[CSS21\]](#biblio-css21): the page-margin boxes never interleave with parts of the document content or between the content and the canvas. They may only paint in front of the document content or behind the document canvas. The page background is always painted underneath everything else.

<a id="ref-for-propdef-z-index"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-propdef-z-index①"></a>

<a id="ref-for-propdef-position①"></a>

The [z-index](https://www.w3.org/TR/css3-positioning/#propdef-z-index) property applies to page-margin boxes. Since the [position](https://www.w3.org/TR/css3-positioning/#propdef-position) property does not apply to page-margin boxes, [z-index](https://www.w3.org/TR/css3-positioning/#propdef-z-index) always affects page-margin boxes as if they were positioned elements regardless of the [position](https://www.w3.org/TR/css3-positioning/#propdef-position) property’s value. Each page-margin boxes always establishes a stacking context.

The default painting order, or [CSS2.1 Appendix E](https://www.w3.org/TR/CSS21/zindex.html) "tree order", of page-margin boxes with respect to each other is as follows:

1.  <a id="ref-for-at-ruledef-top-left-corner"></a>

    [@top-left-corner](#at-ruledef-top-left-corner)

2.  <a id="ref-for-at-ruledef-top-left"></a>

    [@top-left](#at-ruledef-top-left)

3.  <a id="ref-for-at-ruledef-top-center"></a>

    [@top-center](#at-ruledef-top-center)

4.  <a id="ref-for-at-ruledef-top-right"></a>

    [@top-right](#at-ruledef-top-right)

5.  <a id="ref-for-at-ruledef-top-right-corner"></a>

    [@top-right-corner](#at-ruledef-top-right-corner)

6.  <a id="ref-for-at-ruledef-right-top"></a>

    [@right-top](#at-ruledef-right-top)

7.  <a id="ref-for-at-ruledef-right-middle"></a>

    [@right-middle](#at-ruledef-right-middle)

8.  <a id="ref-for-at-ruledef-right-bottom"></a>

    [@right-bottom](#at-ruledef-right-bottom)

9.  <a id="ref-for-at-ruledef-bottom-right-corner"></a>

    [@bottom-right-corner](#at-ruledef-bottom-right-corner)

10. <a id="ref-for-at-ruledef-bottom-right"></a>

    [@bottom-right](#at-ruledef-bottom-right)

11. <a id="ref-for-at-ruledef-bottom-center"></a>

    [@bottom-center](#at-ruledef-bottom-center)

12. <a id="ref-for-at-ruledef-bottom-left"></a>

    [@bottom-left](#at-ruledef-bottom-left)

13. <a id="ref-for-at-ruledef-bottom-left-corner"></a>

    [@bottom-left-corner](#at-ruledef-bottom-left-corner)

14. <a id="ref-for-at-ruledef-left-bottom"></a>

    [@left-bottom](#at-ruledef-left-bottom)

15. <a id="ref-for-at-ruledef-left-middle"></a>

    [@left-middle](#at-ruledef-left-middle)

16. <a id="ref-for-at-ruledef-left-top"></a>

    [@left-top](#at-ruledef-left-top)

<a id="ref-for-at-ruledef-top-left-corner①"></a>

<a id="ref-for-propdef-z-index②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Start with [@top-left-corner](#at-ruledef-top-left-corner), then go clockwise. This order is arbitrary but can be overridden with [z-index](https://www.w3.org/TR/css3-positioning/#propdef-z-index). It only has a visible effect when page-margin boxes overlap, which should not happen in most cases.

### <a id="content-outside-box"></a>3.2. Content outside the page box

<a id="ref-for-propdef-white-space"></a>

<a id="ref-for-valdef-white-space-pre"></a>

When formatting content in the page model, some content may end up outside the page box. For example, an element whose [white-space](https://www.w3.org/TR/css-text-3/#propdef-white-space) property has the value [pre](https://www.w3.org/TR/css-text-3/#valdef-white-space-pre) can generate a box that is wider than the page box. As another example, when boxes are positioned absolutely or relatively, they may end up in "inconvenient" locations. For example, images may be placed on the edge of the page box or 100,000 meters below the page box.

A specification for the exact formatting of such elements lies outside the scope of this document. However, it is recommended that authors and user agents observe the following general principles concerning content outside the page box:

- Content should be allowed slightly beyond the page box to allow pages to "bleed".

- User agents <em>SHOULD</em> avoid generating a large number of content-empty pages to honor the positioning of elements (e.g., printing 100 blank pages is probably neither the author’s nor the user’s intent). A <a id="content-empty"></a>Content-empty page is a page box whose page area contains no printable content other than backgrounds and/or borders. A page box whose page area contains generated content, or content whose visibility is hidden, or invisible content such as a zero-width space is not a content-empty page. On the other hand, a page containing only a background and/or borders and/or page-margin box content <em>is</em> a content-empty page.

  <a id="ref-for-propdef-page-break-before"></a>

  <a id="ref-for-propdef-break-before"></a>

  <a id="ref-for-propdef-page-break-after"></a>

  <a id="ref-for-propdef-break-after"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note, however, that generating a small number of empty page boxes is sometimes necessary to honor the forced-break values for [page-break-before](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-before)/[break-before](https://www.w3.org/TR/css3-break/#propdef-break-before) and [page-break-after](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-after)/[break-after](https://www.w3.org/TR/css3-break/#propdef-break-after). [\[CSS21\]](#biblio-css21) [\[CSS3-BREAK\]](#biblio-css3-break)

- Authors <em>SHOULD NOT</em> position elements in inconvenient locations just to avoid rendering them. Instead:
  - <a id="ref-for-valdef-display-none"></a>

    <a id="ref-for-propdef-display"></a>

    To suppress box generation entirely, set the [display](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) property to [none](https://www.w3.org/TR/css-display-3/#valdef-display-none).

  - <a id="ref-for-propdef-visibility"></a>

    To make a box invisible, set the [visibility](https://www.w3.org/TR/CSS21/visufx.html#propdef-visibility) property.

- This specification does not define how boxes positioned outside the page box are handled. Possibilities include discarding them or creating page boxes for them at the end of the document.

### <a id="progression"></a>3.3. Page Progression

CSS distinguishes between left pages and right pages on all documents, whether they are printed duplex or not. Each left page is followed by a right page and vice versa. Left and right pages can be styled differently with the [`:left` and `:right` pseudo-classes](#left-right-first).

Whether the first page of a document is a left page or a right page depends on the page progression of the document. The <a id="page-progression"></a>page progression is the direction in which the printed pages of a document would be sequenced when laid out side-to-side. For example, English and horizontally-set Japanese typically progress from left to right, whereas Arabic and vertically-set Japanese pages typically progress from right to left.

The page progression direction is determined as follows:

- <a id="ref-for-inline-base-direction"></a>

  <a id="ref-for-page-progression"></a>

  If text is laid out in horizontal lines, the [page progression](#page-progression) is the same as the [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction).

- <a id="ref-for-block-flow-direction"></a>

  <a id="ref-for-page-progression①"></a>

  If text is laid out in vertical lines, the [page progression](#page-progression) is the same as the [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction).

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-page-progression②"></a>

<a id="ref-for-principal-writing-mode"></a>

If the UA supports the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) and [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) properties from the CSS 3 Writing Modes Module [\[CSS3-WRITING-MODES\]](#biblio-css3-writing-modes), it must [determine](https://www.w3.org/TR/css3-writing-modes/#page-direction) the [page progression](#page-progression) is determed by the [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode).

<a id="ref-for-page-progression③"></a>

<a id="ref-for-propdef-break-before①"></a>

In documents with a left-to-right [page progression](#page-progression) the first page of the document is a right page, and vice versa. To explicitly force a document to begin printing on a left or right page, authors can specify a [break-before](https://www.w3.org/TR/css3-break/#propdef-break-before) value that that propagates a page break to the root. [\[CSS3-BREAK\]](#biblio-css3-break) The UA must suppress the first (empty) page(s) in this case (and the `:first` pseudo-class matches the first printed page).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0b89bad0"></a>
>
> ```text
> html { break-before: always }
> ```
>
> For an HTML document with a left-to-right page progression, the above style rule will cause the first page of the document to print on a ':left' page
>
> ```text
> html { break-before: left }
> ```
>
> For an HTML document, the above style rule will cause the first page of the document to print on a ':left' page, regardless of the page progression.

## <a id="page-selector-and-context"></a>4.  Page Selectors and the Page Context

### <a id="at-page-rule"></a>4.1. <a id="page-box-page-rule"></a> The @page Rule

<a id="ref-for-at-ruledef-page①"></a>

<a id="ref-for-at-ruledef-page②"></a>

<a id="ref-for-at-ruledef-page③"></a>

<a id="ref-for-page-selector②"></a>

<a id="ref-for-at-ruledef-page④"></a>

<a id="ref-for-margin-at-rule"></a>

<a id="ref-for-at-ruledef-page⑤"></a>

Authors can specify various aspects of a page box, such as its dimensions, orientation, and margins, within an <a id="at-ruledef-page"></a>@page rule. [@page](#at-ruledef-page) rules are allowed wherever [rule-sets](https://www.w3.org/TR/CSS21/syndata.html#rule-sets) are allowed. An [@page](#at-ruledef-page) rule consists of the keyword [@page](#at-ruledef-page), an optional comma-separated list of [page selectors](#page-selector) and a block of declarations (said to be in the <a id="page-context"></a>page context). An [@page](#at-ruledef-page) rule can also contain other at-rules, interleaved between declarations. The current level of this specification only allows [margin at-rules](#margin-at-rule) inside [@page](#at-ruledef-page).

<a id="ref-for-at-ruledef-page⑥"></a>

<a id="ref-for-at-ruledef-page⑦"></a>

<a id="ref-for-match"></a>

[@page](#at-ruledef-page) rules without a selector list apply to every page. Other [@page](#at-ruledef-page) rules apply to pages that [match](#match) at least one of their selectors. Properties declared within the page context apply to the page box.

If an error is encountered during the processing of a declaration block within a page or a margin context, the [Rules for handling parsing errors](https://www.w3.org/TR/CSS21/syndata.html#parsing-errors) apply; that is, valid declarations within the block are applied.

### <a id="page-selectors"></a>4.2. Page selectors

<a id="ref-for-page-type-selector"></a>

<a id="ref-for-page-pseudo-class"></a>

<a id="ref-for-page-pseudo-class①"></a>

<a id="ref-for-typedef-page-selector"></a>

A <a id="page-selector"></a>page selector is made of either a [page type selector](#page-type-selector) or a [page pseudo-class](#page-pseudo-class), followed by zero or more additional [page pseudo-classes](#page-pseudo-class). No whitespace is allowed between components of a selector. The [\<page-selector\>](#typedef-page-selector) grammar and examples can be found below.

<a id="ref-for-page-selector③"></a>

A [page selector](#page-selector) is said to <a id="match"></a>match a given page if and only if all of its components match the page.

<a id="ref-for-match①"></a>

<a id="ref-for-propdef-page"></a>

A <a id="page-type-selector"></a>page type selector is a case-sensitive [CSS identifier](https://www.w3.org/TR/CSS21/syndata.html#value-def-identifier) [\[CSS21\]](#biblio-css21). It [matches](#match) pages of the [named page type](#using-named-pages) generated by the [page](#propdef-page) property. <a id="page-selector-syntax-restrict"></a> A page type name of auto ([ASCII case-insensitive](https://www.w3.org/TR/CSS21/syndata.html#characters)) does not make the rule invalid, but must never match.

<a id="ref-for-page-pseudo-class②"></a>

A <a id="page-pseudo-class"></a>page pseudo-class is [ASCII case-insensitive](https://www.w3.org/TR/CSS21/syndata.html#characters) and has the same syntax as [pseudo-classes](https://www.w3.org/TR/selectors/#pseudo-classes) in regular Selectors. [\[SELECT\]](#biblio-select) The various [page pseudo-classes](#page-pseudo-class) are defined below.

<a id="ref-for-valdef-page-left①"></a>

<a id="ref-for-valdef-page-right①"></a>

#### <a id="spread-pseudos"></a>4.2.1. <a id="left-right-first"></a> Spread pseudo-classes: [:left](#valdef-page-left), [:right](#valdef-page-right)

<a id="ref-for-valdef-page-left②"></a>

<a id="ref-for-valdef-page-right②"></a>

When printing double-sided documents, left and right pages are often formatted differently. This can be expressed by using the [:left](#valdef-page-left) and [:right](#valdef-page-right) page pseudo-classes.

<a id="ref-for-match②"></a>

<a id="ref-for-left-page"></a>

<a id="ref-for-right-page"></a>

All pages are automatically classified by user agents as either left pages or right pages, based on [page progression](#progression). The <a id="valdef-page-left"></a>:left and <a id="valdef-page-right"></a>:right pseudo-classes only [match](#match) [left](#left-page) or [right pages](#right-page), respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d233352a"></a> The following example creates left and right binding edges using these pseudo-classes:
>
> ```text
> @page :left {
> margin-left: 3cm;
> margin-right: 4cm;
> }
> 
> @page :right {
> margin-left: 4cm;
> margin-right: 3cm;
> }
> ```
If different declarations have been given for left and right pages, the user agent must honor these declarations even if the user agent does not transfer the page boxes to left and right sheets (i.e., a printer that only prints on one side of the medium must nevertheless produce correctly formatted output).

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-valdef-page-left③"></a>
>
> <a id="ref-for-valdef-page-right③"></a>
>
> <em><strong>Note.</strong> Adding declarations to the <a href="#valdef-page-left">:left</a> or <a href="#valdef-page-right">:right</a> pseudo-class&#xA;&#x9;does not necessarily influence whether the document&#xA;&#x9;comes out of the printer double- or single-sided&#xA;&#x9;(which is outside the scope of this specification).</em>

<a id="ref-for-valdef-page-first"></a>

#### <a id="first-pseudo"></a>4.2.2.  First-page pseudo-class: [:first](#valdef-page-first)

<a id="ref-for-match③"></a>

The <a id="valdef-page-first"></a>:first pseudo-class [matches](#match) the first printed page of a document.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0c64c208"></a>
>
> ```text
> @page { margin: 2cm } /* All margins set to 2cm */
> 
> @page :first {
> margin-top: 10cm /* Top margin on first page 10cm */
> }
> ```
<a id="ref-for-valdef-page-blank"></a>

#### <a id="blank-pseudo"></a>4.2.3.  Blank-page pseudo-class: [:blank](#valdef-page-blank)

<a id="ref-for-match④"></a>

<a id="ref-for-content-empty"></a>

The <a id="valdef-page-blank"></a>:blank pseudo-class [matches](#match) [content-empty pages](#content-empty) that appear as a result of [forced page breaks](https://www.w3.org/TR/css3-break/#forced-breaks).

<a id="ref-for-valdef-break-before-left"></a>

<a id="ref-for-valdef-break-before-right"></a>

<a id="ref-for-valdef-break-before-recto"></a>

<a id="ref-for-valdef-break-before-verso"></a>

<a id="ref-for-propdef-break-before②"></a>

<a id="ref-for-propdef-break-after①"></a>

<a id="ref-for-valdef-page-blank①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Only the [left](https://www.w3.org/TR/css3-break/#valdef-break-before-left), [right](https://www.w3.org/TR/css3-break/#valdef-break-before-right), [recto](https://www.w3.org/TR/css3-break/#valdef-break-before-recto) and [verso](https://www.w3.org/TR/css3-break/#valdef-break-before-verso) values of the [break-before](https://www.w3.org/TR/css3-break/#propdef-break-before) and [break-after](https://www.w3.org/TR/css3-break/#propdef-break-after) properties can generate pages that match [:blank](#valdef-page-blank).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63622ae9"></a> In this example, forced page break may occur before `h1` elements.
>
> ```text
> h1 { break-before: left }
> 
> @page :blank {
>   @top-center { content: "This page is intentionally left blank" }
> }
> ```
<a id="ref-for-valdef-page-blank②"></a>

A page matched by [:blank](#valdef-page-blank) can also be matched by other page pseudo-classes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f6c58f27"></a>
>
> If headers have been specified on all right pages, a blank right page will be matched by both `:blank` and `:right`. Therefore, margin boxes set on right pages will have to be removed unless they are wanted on blank pages. Here is an example where the top center header is removed from blank pages, while the page number remains:
>
> ```text
> h1 { break-before: left }
> 
> @page :blank {
>   @top-center { content: none }
> }
> 
> @page :right {
>   @top-center { content: "Preliminary edition" }
>   @bottom-center { content: counter(page) }
> }
> ```
>
> <a id="ref-for-specificity"></a>
>
> Due to the higher [specificity](#specificity) of `:blank` over `:right`, the top center header is removed even if `content: none` comes before
>
> ```text
> content: "Preliminary
> edition"
> ```
>
> .

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> Future versions of CSS may include other page&#xA;pseudo-classes.</em>

### <a id="syntax-page-selector"></a>4.3. @page rule grammar

<a id="ref-for-at-ruledef-page⑧"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

[@page](#at-ruledef-page) rules are [parsed](https://drafts.csswg.org/css-syntax-3/#parse-grammar) according to the following grammar, plus the additional rules noted below:

<a id="ref-for-typedef-page-selector-list"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-declaration-list"></a>

<a id="typedef-page-selector-list"></a>

<a id="ref-for-typedef-page-selector-list①"></a>

<a id="ref-for-typedef-page-selector①"></a>

<a id="ref-for-mult-comma"></a>

<a id="typedef-page-selector"></a>

<a id="ref-for-typedef-page-selector②"></a>

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-pseudo-page"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-mult-req"></a>

<a id="typedef-pseudo-page"></a>

<a id="ref-for-typedef-pseudo-page①"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="at-ruledef-top-left-corner"></a>

<a id="ref-for-typedef-declaration-list①"></a>

<a id="at-ruledef-top-left"></a>

<a id="ref-for-typedef-declaration-list②"></a>

<a id="at-ruledef-top-center"></a>

<a id="ref-for-typedef-declaration-list③"></a>

<a id="at-ruledef-top-right"></a>

<a id="ref-for-typedef-declaration-list④"></a>

<a id="at-ruledef-top-right-corner"></a>

<a id="ref-for-typedef-declaration-list⑤"></a>

<a id="at-ruledef-bottom-left-corner"></a>

<a id="ref-for-typedef-declaration-list⑥"></a>

<a id="at-ruledef-bottom-left"></a>

<a id="ref-for-typedef-declaration-list⑦"></a>

<a id="at-ruledef-bottom-center"></a>

<a id="ref-for-typedef-declaration-list⑧"></a>

<a id="at-ruledef-bottom-right"></a>

<a id="ref-for-typedef-declaration-list⑨"></a>

<a id="at-ruledef-bottom-right-corner"></a>

<a id="ref-for-typedef-declaration-list①⓪"></a>

<a id="at-ruledef-left-top"></a>

<a id="ref-for-typedef-declaration-list①①"></a>

<a id="at-ruledef-left-middle"></a>

<a id="ref-for-typedef-declaration-list①②"></a>

<a id="at-ruledef-left-bottom"></a>

<a id="ref-for-typedef-declaration-list①③"></a>

<a id="at-ruledef-right-top"></a>

<a id="ref-for-typedef-declaration-list①④"></a>

<a id="at-ruledef-right-middle"></a>

<a id="ref-for-typedef-declaration-list①⑤"></a>

<a id="at-ruledef-right-bottom"></a>

<a id="ref-for-typedef-declaration-list①⑥"></a>

```text
@page = @page <page-selector-list>? { <declaration-list> }
<page-selector-list> = <page-selector>#
<page-selector> = [ <ident-token>? <pseudo-page>* ]!
<pseudo-page> = ':' [ left | right | first | blank ]

/* Margin rules */
@top-left-corner = @top-left-corner { <declaration-list> };
@top-left = @top-left { <declaration-list> };
@top-center = @top-center { <declaration-list> };
@top-right = @top-right { <declaration-list> };
@top-right-corner = @top-right-corner { <declaration-list> };
@bottom-left-corner = @bottom-left-corner { <declaration-list> };
@bottom-left = @bottom-left { <declaration-list> };
@bottom-center = @bottom-center { <declaration-list> };
@bottom-right = @bottom-right { <declaration-list> };
@bottom-right-corner = @bottom-right-corner { <declaration-list> };
@left-top = @left-top { <declaration-list> };
@left-middle = @left-middle { <declaration-list> };
@left-bottom = @left-bottom { <declaration-list> };
@right-top = @right-top { <declaration-list> };
@right-middle = @right-middle { <declaration-list> };
@right-bottom = @right-bottom { <declaration-list> };
```
In addition, the following rules apply:

- <a id="ref-for-typedef-page-selector③"></a>

  <a id="ref-for-typedef-pseudo-page②"></a>

  <a id="ref-for-typedef-compound-selector"></a>

  No whitespace is allowed between the productions in [\<page-selector\>](#typedef-page-selector) or [\<pseudo-page\>](#typedef-pseudo-page) (similar to the rule for [\<compound-selector\>](https://www.w3.org/TR/selectors4/#typedef-compound-selector)).

- <a id="ref-for-at-ruledef-page⑨"></a>

  <a id="ref-for-page-property"></a>

  <a id="ref-for-margin-at-rule①"></a>

  The [@page](#at-ruledef-page) rule can only contain [page properties](#page-property) and [margin at-rules](#margin-at-rule).

- <a id="ref-for-margin-at-rule②"></a>

  <a id="ref-for-page-margin-property"></a>

  The [margin at-rules](#margin-at-rule) can only contain [page-margin properties](#page-margin-property).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-691ff5b9"></a> The following are examples of page selectors (declaration block intentionally left blank)
>
> ```text
> @page { ... }
> @page :left { ... }
> @page :right { ... }
> @page LandscapeTable { ... }
> @page CompanyLetterHead:first { ... } /*  identifier and pseudo page. */
> @page:first { ... }
> @page toc, index { ... }
> @page :blank:first { ... }
> ```
>
> The following are examples of page-margin boxes where the declaration blocks are intentionally left blank.
>
> ```text
> @page {
>  @top-left { ... /* document name */ }
>  @bottom-center { ... /* page number */}
> }
> @page :left { @left-middle { ... /* page number in left margin */ }}
> @page :right{ @right-middle { ... /* page number in right margins of right pages */}}
> 
> @page :left { @bottom-left-corner { ... /* left page numbers */ }}
> @page :right { @bottom-right-corner { ... /* right page numbers */ }}
> @page :first { @bottom-left-corner { ... /* empty footer on 1st page */ }
> @bottom-right-corner { ... /* empty footer */ } }
> ```
### <a id="cascading-and-page-context"></a>4.4. Cascading in the page context

Declarations in page and margin contexts [cascade](https://www.w3.org/TR/CSS21/cascade.html) just like declarations in style rule for elements.

The <a id="specificity"></a>specificity of page a selector is computed in a manner analogous to the computations defined in the [Selectors](https://www.w3.org/TR/CSS21/selector.html) module:

- Count the number of page type names (= <var>f</var>)
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Given the syntax of page selectors, <var>f</var> can only ever be 0 or 1.
- Count the number of ':first' or ':blank' pseudo-classes (= <var>g</var>)
- Count the number of ':left' or ':right' pseudo-classes (= <var>h</var>)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Repeated occurrences of the same pseudo-classes are allowed and do increase specificity.

Due to storage limitations, implementations may have limitations on the size of <var>f</var>, <var>g</var>, or <var>h</var>. If so, values higher than the limit must be clamped to that limit, and not overflow.

Specificities are compared by comparing the three components in order (<var>f</var>, <var>g</var>, <var>h</var>): the specificity with a larger <var>f</var> value is more specific; if the two <var>f</var> values are tied, then the two <var>g</var> values are compared, etc. If all the values are tied, the two specificities are equal.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f7460463"></a>
>
> Some page specificity calculation examples follow:
>
> ```text
> @page { } /* specificity = (0,0,0) */
> @page :left { } /* specificity = (0,0,1) */
> @page :first { } /* specificity = (0,1,0) */
> @page :blank:left { } /* specificity = (0,1,1) */
> @page artsy { } /* specificity = (1,0,0) */
> @page artsy:left { } /* specificity = (1,0,1) */
> @page artsy:first { } /* specificity = (1,1,0) */
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-66f92469"></a>
>
> Consider the following usage example:
>
> ```text
> @page :left {
>   margin-left: 4cm;
> }
> 
> @page {
>   margin-left: 3cm;
> }
> ```
>
> Due to the higher specificity of the pseudo-class selector, the left margin on left pages will be 4cm and all other pages (the right-facing pages) will have a left margin of 3cm.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8b37b195"></a> In this example, the higher specificity of the green rules wins over the red rule. Therefore the first page will have blue text in the top-left page-margin box and green text in the top-right page-margin box, while subsequent pages will have red text in the page-margin boxes.
>
> ```text
> @page :first {
>   color: green;
> 
>   @top-left {
>     content: "foo";
>     color: blue;
>   }
>   @top-right {
>     content: "bar";
>   }
> }
> 
> @page { color: red;
>   @top-center {
>     content: "Page " counter(page);
>   }
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ba05142d"></a>
>
> Page contexts cascade, so the following stylesheet would style pages with 25 millimeter margins and 14 point type in the page-margin boxes:
>
> ```text
> @page { margin: 25mm;}
> @page { font-size: 14pt;}
> ```
## <a id="margin-boxes"></a>5. Page-Margin Boxes

Page-margin boxes are boxes within the page margin that, like pseudo-elements, can contain generated content.

Page-margin boxes can be used to create page headers and footers, which are portions of the page set aside for supplementary information such as the page number or document title.

<a id="ref-for-binding-edge"></a>

<a id="ref-for-at-ruledef-top-left-corner②"></a>

<a id="ref-for-at-ruledef-top-left①"></a>

<a id="ref-for-at-ruledef-top-center①"></a>

<a id="ref-for-at-ruledef-top-right①"></a>

<a id="ref-for-at-ruledef-top-right-corner①"></a>

<a id="ref-for-at-ruledef-right-top①"></a>

<a id="ref-for-at-ruledef-right-middle①"></a>

<a id="ref-for-at-ruledef-right-bottom①"></a>

<a id="ref-for-right-page①"></a>

<a id="ref-for-at-ruledef-left-top①"></a>

<a id="ref-for-at-ruledef-left-middle①"></a>

<a id="ref-for-at-ruledef-left-bottom①"></a>

<a id="ref-for-left-page①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e18d595c"></a> Typically, a <a id="page-header"></a>page header is located at the top of the page in documents with a predominately horizontal writing direction and on the side opposite the [binding edge](#binding-edge) for documents with a predominately vertical writing direction. One possible design of page headers for horizontally written documents uses the [@top-left-corner](#at-ruledef-top-left-corner), [@top-left](#at-ruledef-top-left), [@top-center](#at-ruledef-top-center), [@top-right](#at-ruledef-top-right) and [@top-right-corner](#at-ruledef-top-right-corner) page-margin boxes. Another design, for vertically written documents, could use the [@right-top](#at-ruledef-right-top), [@right-middle](#at-ruledef-right-middle), and [@right-bottom](#at-ruledef-right-bottom) page-margin boxes for [right facing pages](#right-page) and [@left-top](#at-ruledef-left-top), [@left-middle](#at-ruledef-left-middle), and [@left-bottom](#at-ruledef-left-bottom) for [left facing pages](#left-page).
>
> <a id="ref-for-at-ruledef-bottom-left-corner①"></a>
>
> <a id="ref-for-at-ruledef-bottom-left①"></a>
>
> <a id="ref-for-at-ruledef-bottom-center①"></a>
>
> <a id="ref-for-at-ruledef-bottom-right①"></a>
>
> <a id="ref-for-at-ruledef-bottom-right-corner①"></a>
>
> The <a id="page-footer"></a>page footer is typically at the opposite end of the page from the page header. For example, the design of a horizontally written document with a page header at the top of the page could use the [@bottom-left-corner](#at-ruledef-bottom-left-corner), [@bottom-left](#at-ruledef-bottom-left), [@bottom-center](#at-ruledef-bottom-center), [@bottom-right](#at-ruledef-bottom-right) and [@bottom-right-corner](#at-ruledef-bottom-right-corner) page-margin boxes as the page footer. The design of a vertically written document could use the page-margin boxes of the binding edge of the page for the page footer.

Page-margin boxes are positioned with respect to the page area and are independent of page orientation, for example the top page-margin boxes are above the page area in both portrait and landscape orientation. The various page-margin boxes are defined and illustrated in the diagram below:

<a id="margin-box-def"></a>

<a id="top-margin-boxes-def"></a>

<a id="top-left-box-def"></a>

<a id="top-center-box-def"></a>

<a id="top-right-box-def"></a>

<a id="left-margin-boxes-def"></a>

<a id="left-middle-box-def"></a>

<a id="left-bottom-box-def"></a>

<a id="right-margin-boxes-def"></a>

<a id="right-middle-box-def"></a>

<a id="right-bottom-box-def"></a>

<a id="bottom-margin-boxes-def"></a>

<a id="bottom-left-box-def"></a>

<a id="bottom-center-box-def"></a>

<a id="bottom-right-box-def"></a>

<a id="bottom-right-corner-box-def"></a>

**Table 1**

Table 1 Page-Margin Box Definitions

Representation note: each page-margin box keeps its description and placement. Shared placement images are repeated for every box covered by the source rowspan.

**Box**

**Description**

**Placement**

<a id="top-left-corner-box-def"></a>**top-left-corner**

a fixed-size box defined by the intersection of the top and left margins of the page box

**Placement**

![the top left corner box with margin, border, and padding, nested within intersection of the page’s top and left margins](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/TopLeftCornerBox.png)

**top-left**

a variable-width box filling the top page margin between the top-left-corner and top-center page-margin boxes

**Placement**

![the top left box with margin, border, and padding, nested in the page’s top margin next to the top left corner box](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/TopLeftMarginBox.png)

**top-center**

a variable-width box centered horizontally between the page’s left and right border edges and filling the page top margin between the top-left and top-right page-margin boxes

**Placement**

![the top center box with margin, border, and padding, centered within the page’s top margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/TopCenterMarginBox.png)

**top-right**

a variable-width box filling the top page margin between the top-center and top-right-corner page-margin boxes

**Placement**

![the top right box with margin, border, and padding, nested within the page’s top margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/TopRightMarginBox.png)

<a id="top-right-corner-box-def"></a>**top-right-corner**

a fixed-size box defined by the intersection of the top and right margins of the page box

**Placement**

![the top right corner box with margin, border, and padding, nested within the intersection of the page’s top and right margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/TopRightCornerMarginBox.png)

<a id="left-top-box-def"></a>**left-top**

a variable-height box filling the left page margin between the top-left-corner and left-middle page-margin boxes

**Placement**

![left-top, left-middle, and left-bottom page-margin boxes in the page box’s left margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/LeftMarginBoxes.png)

**left-middle**

a variable-height box centered vertically between the page’s top and bottom border edges and filling the left page margin between the left-top and left-bottom page-margin boxes

**Placement**

![left-top, left-middle, and left-bottom page-margin boxes in the page box’s left margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/LeftMarginBoxes.png)

**left-bottom**

a variable-height box filling the left page margin between the left-middle and bottom-left-corner page-margin boxes

**Placement**

![left-top, left-middle, and left-bottom page-margin boxes in the page box’s left margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/LeftMarginBoxes.png)

<a id="right-top-box-def"></a>**right-top**

a variable-height box filling the right page margin between the top-right-corner and right-middle page-margin boxes

**Placement**

![right-top, right-middle, and right-bottom page-margin boxes in the page box’s right margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/RightMarginBoxes.png)

**right-middle**

a variable-height box centered vertically between the page’s top and bottom border edges and filling the right page margin between the right-top and right-bottom page-margin boxes

**Placement**

![right-top, right-middle, and right-bottom page-margin boxes in the page box’s right margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/RightMarginBoxes.png)

**right-bottom**

a variable-height box filling the right page margin between the right-middle and bottom-right-corner page-margin boxes

**Placement**

![right-top, right-middle, and right-bottom page-margin boxes in the page box’s right margin](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/RightMarginBoxes.png)

<a id="bottom-left-corner-box-def"></a>**bottom-left-corner**

a fixed-size box defined by the intersection of the bottom and left margins of the page box

**Placement**

![bottom left corner box with margin, border, and padding, nested within the page margin at the intersection of the left and bottom page margins](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/BottomLeftCornerBox.png)

**bottom-left**

a variable-width box filling the bottom page margin between the bottom-left-corner and bottom-center page-margin boxes

**Placement**

![bottom left page-margin box with margin, border, and padding, nested within the page’s bottom margin next to the bottom-left-corner box](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/BottomLeftMarginBox.png)

**bottom-center**

a variable-width box centered horizontally between the page’s left and right border edges and filling the bottom page margin between the bottom-left and bottom-right page-margin boxes

**Placement**

![bottom center box with margin, border, and padding, nested within the page’s bottom margin and centered on the page](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/BottomCenterMarginBox.png)

**bottom-right**

a variable-width box filling the bottom page margin between the bottom-center and bottom-right-corner page-margin boxes

**Placement**

![bottom right page-margin box with margin, border, and padding, nested within the page’s bottom margin and next to the bottom-right-corner box](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/BottomRightMarginBox.png)

**bottom-right-corner**

a fixed-size box defined by the intersection of the bottom and right margins of the page box

**Placement**

![bottom right corner box with margin, border, and padding, nested within the page margin at the intersection of the right and bottom page margins](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/BottomRightCornerBox.png)

### <a id="margin-at-rules"></a>5.1. At-rules for page-margin boxes

<a id="ref-for-margin-at-rule③"></a>

<a id="ref-for-page-context④"></a>

Page-margin boxes are created by [margin at-rules](#margin-at-rule) inside the [page context](#page-context). Authors should put these rules after any declarations in the page context as legacy clients may not handle declarations after margin at-rules correctly.

<a id="ref-for-css-at-rule"></a>

<a id="ref-for-at-ruledef-top-left②"></a>

A <a id="margin-at-rule"></a>margin at-rule is an [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) that identifies the page-margin box (e.g. [@top-left](#at-ruledef-top-left)) and a block of descriptors (said to be in the <a id="margin-context"></a>margin context).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-26a4432a"></a>
>
> The following style sheet establishes a page header containing the title ("Hamlet") on the left side and the page number, preceded by "Page ", on the right side:
>
> ```text
> @page {
>   size: 8.5in 11in;
>   margin: 10%;
> 
>   @top-left {
>     content: "Hamlet";
>   }
>   @top-right {
>     content: "Page " counter(page);
>   }
> }
> ```
### <a id="populating-margin-boxes"></a>5.2. Populating page-margin boxes

<a id="ref-for-propdef-content"></a>

<a id="ref-for-valdef-content-none"></a>

<a id="ref-for-propdef-content①"></a>

<a id="ref-for-valdef-content-none①"></a>

<a id="ref-for-propdef-display①"></a>

As with the :before and :after pseudo-elements, a specified [content: normal](https://www.w3.org/TR/css-content-3/#propdef-content) on a page-margin box computes to [none](https://www.w3.org/TR/css-content-3/#content-property). A page-margin box is <a id="generated"></a>generated if and only if the computed value of its [content](https://www.w3.org/TR/css-content-3/#propdef-content) property is not [none](https://www.w3.org/TR/css-content-3/#content-property). Otherwise, no box is generated, as for elements with [display: none](https://www.w3.org/TR/CSS21/visuren.html#propdef-display).

<a id="ref-for-propdef-display②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [display](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) property does not apply to page-margin boxes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ab340c9c"></a> The following style sheet creates a green box in each corner of the page except the bottom-left corner.
>
> ```text
> @page {
>   @top-left-corner { content: " "; border: solid green; }
>   @top-right-corner { content: url(foo.png); border: solid green; }
>   @bottom-right-corner { content: counter(page); border: solid green; }
>   @bottom-left-corner { content: normal; border: solid green; }
> }
> ```
### <a id="margin-dimension"></a>5.3. Computing Page-margin Box Dimensions

The width and height of each page-margin box is determined by the rules below. These rules define the equivalent of CSS2.1 Sections 10.3 and 10.6 for page-margin boxes.

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-propdef-min-width①"></a>

<a id="ref-for-propdef-min-height②"></a>

<a id="ref-for-propdef-min-width②"></a>

The rules for applying [min-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-height), [max-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-height), [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width), and [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width) [\[CSS21\]](#biblio-css21) do apply to page-margin boxes and may imply a recalculation of the width, height, and/or margins if the dimensions resulting from the specified [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) or [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) violate their constraints. If the UA does not support the [min-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-height) or [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) properties then it must behave as if [min-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-height) and [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) were always zero.

#### <a id="margin-box-terms"></a>5.3.1.  Page-Margin Box Layout Terminology<a id="max-margin-dimension"></a>

In addition to the box model definitions in CSS2.1 [\[CSS21\]](#biblio-css21), and the sizing terms in CSS Intrinsic Sizing [\[CSS3-SIZING\]](#biblio-css3-sizing), the following terms are defined for use in the subsequent page-margin box calculations:

<a id="available-width"></a>available width  
<a id="ref-for-page-box③"></a>

<a id="ref-for-page-box②"></a>

The sum of the page’s left border width, left padding, [page area](#page-box) width, right padding, and right border width. In other words, it is the distance between the [page box](#page-box)’s left right border edges. This quantity is used when calculating dimensions of the top and bottom page-margin boxes.

<a id="available-height"></a>available height  
<a id="ref-for-page-box⑤"></a>

<a id="ref-for-page-box④"></a>

The sum of the page’s top border width, top padding, [page area](#page-box) height, bottom padding, and bottom border width. In other words, it is the distance between the [page box](#page-box)’s top bottom border edges. This quantity is used when calculating dimensions of the left and right page-margin boxes.

<a id="outer-width"></a>outer width  
<a id="ref-for-outer-edge"></a>

The width of the [outer edge](https://www.w3.org/TR/css-box-3/#outer-edge), as defined in [\[CSS-BOX-3\]](#biblio-css-box-3).

<a id="min-content-width"></a>min-content width  
<a id="ref-for-min-content-inline-size"></a>

<a id="ref-for-min-content-block-size"></a>

Whichever of [min-content block size](https://www.w3.org/TR/css-sizing-3/#min-content-block-size) or [min-content inline size](https://www.w3.org/TR/css-sizing-3/#min-content-inline-size) is the physical width.

<a id="max-content-width"></a>max-content width  
<a id="ref-for-max-content-inline-size"></a>

<a id="ref-for-max-content-block-size"></a>

Whichever of [max-content block size](https://www.w3.org/TR/css-sizing-3/#max-content-block-size) or [max-content inline size](https://www.w3.org/TR/css-sizing-3/#max-content-inline-size) is the physical width.

<a id="outer-min-width"></a>outer min width  
<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-min-content-width"></a>

<a id="ref-for-outer-width"></a>

Like the [outer width](#outer-width), except the [min-content width](#min-content-width) is used when the computed [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

<a id="outer-max-width"></a>outer max width  
<a id="ref-for-valdef-width-auto①"></a>

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-max-content-width"></a>

<a id="ref-for-outer-width①"></a>

Like the [outer width](#outer-width), except the [max-content width](#max-content-width) is used when the computed [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

The <a id="containing-block"></a>containing block for a page-margin box depends on its location:

For a corner page-margin box, it is the rectangle defined by the intersection of the two page margins meeting at that corner.

<a id="ref-for-containing-block②"></a>

<a id="ref-for-available-width"></a>

<a id="ref-for-available-height"></a>

For all other page-margin boxes, the [containing block](#containing-block) is the rectangle formed by the encapsulating page margin minus the containing blocks of the adjacent corners' page-margin boxes. This means that the size of this containing block is given in one dimension by the used page margin and in the other dimension by the [available width](#available-width) (for top and bottom page-margin boxes) or [available height](#available-height) (for left and right page-margin boxes).

#### <a id="variable-sizing"></a>5.3.2.  Page-Margin Box Variable Dimension Computation Rules

<a id="ref-for-at-ruledef-top-left③"></a>

<a id="ref-for-at-ruledef-top-center②"></a>

<a id="ref-for-at-ruledef-top-right②"></a>

The following rules apply to [@top-left](#at-ruledef-top-left), [@top-center](#at-ruledef-top-center) and [@top-right](#at-ruledef-top-right) page-margin boxes, which are referred to as A, B, and C, respectively, in this section.

##### <a id="variable-auto-margins"></a>5.3.2.1. Margins

<a id="ref-for-propdef-margin-left"></a>

<a id="ref-for-propdef-margin-right"></a>

If the [margin-left](https://www.w3.org/TR/css-box-3/#propdef-margin-left) or [margin-right](https://www.w3.org/TR/css-box-3/#propdef-margin-right) property of any of the three boxes computes to auto, the used value is zero.

<a id="ref-for-valdef-width-auto②"></a>

##### <a id="variable-auto-sizing"></a>5.3.2.2. Resolving [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) widths

<a id="ref-for-generated"></a>

<a id="ref-for-propdef-width③"></a>

<a id="ref-for-outer-width②"></a>

The following algorithm determines the used width of each box. For this purpose, boxes that are not [generated](#generated) are assumed to have a [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) and an [outer width](#outer-width) of zero.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The high-level goals are (in order of priority) to center the middle box (B) if it is generated, to minimize overflow and overlap, and to distribute space proportionally to the amount of content.

<a id="ref-for-generated①"></a>

<a id="ref-for-available-width①"></a>

If the middle box (B) is not [generated](#generated), distribute the [available width](#available-width) to A and C as follows:

- <a id="ref-for-available-width②"></a>

  <a id="ref-for-outer-width③"></a>

  If only one box has 'width: auto', its used width is resolved so that the sum of the [outer width](#outer-width)s equals [available width](#available-width).

- <a id="ref-for-flex-factor⑥"></a>

  <a id="flex-fit"></a> If A and C both have 'width: auto', distribute the space to each box as follows:

  1.  <a id="ref-for-flex-factor①"></a>

      <a id="ref-for-flex-factor"></a>

      <a id="ref-for-flex-space"></a>

      <a id="ref-for-max-content-width③"></a>

      <a id="ref-for-max-content-width②"></a>

      <a id="ref-for-available-width③"></a>

      <a id="ref-for-max-content-width①"></a>

      If the sum of the outer [max-content widths](#max-content-width) is less than the [available width](#available-width), call that difference the <a id="flex-space"></a>flex space. Calculate each box’s <a id="flex-factor"></a>flex factor as proportional to its outer [max-content width](#max-content-width), and set its used outer width to: <code><a href="#max-content-width">max-content&#x20;width</a>&#x20;+&#x20;<a href="#flex-space">flex&#x20;space</a>&#x20;×&#x20;<a href="#flex-factor">flex&#x20;factor</a>&#x20;÷&#x20;∑<a href="#flex-factor">flex&#x20;factors</a></code>

  2.  <a id="ref-for-flex-factor④"></a>

      <a id="ref-for-flex-factor③"></a>

      <a id="ref-for-flex-space②"></a>

      <a id="ref-for-min-content-width③"></a>

      <a id="ref-for-min-content-width②"></a>

      <a id="ref-for-max-content-width④"></a>

      <a id="ref-for-flex-factor②"></a>

      <a id="ref-for-flex-space①"></a>

      <a id="ref-for-available-width④"></a>

      <a id="ref-for-min-content-width①"></a>

      Otherwise if the sum of the outer [min-content widths](#min-content-width) is less than the [available width](#available-width), call that difference the [flex space](#flex-space) calculate each box’s [flex factor](#flex-factor) as proportional to its [max-content width](#max-content-width) minus [min-content width](#min-content-width), and set its used outer width to: <code><a href="#min-content-width">min-content&#x20;width</a>&#x20;+&#x20;<a href="#flex-space">flex&#x20;space</a>&#x20;×&#x20;<a href="#flex-factor">flex&#x20;factor</a>&#x20;÷&#x20;∑<a href="#flex-factor">flex&#x20;factors</a></code>

  3.  <a id="ref-for-min-content-width④"></a>

      <a id="ref-for-flex-factor⑤"></a>

      Otherwise, calculate its outer size as in the previous case, but set each box’s [flex factor](#flex-factor) as proportional to its outer [min-content width](#min-content-width).

  In each case, both [flex factors](#flex-factor) are assumed to be 1 if their sum is equal to zero.

<a id="ref-for-generated②"></a>

<a id="ref-for-valdef-width-auto③"></a>

If the middle box (B) is [generated](#generated), determine the [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) widths of A, B, and C as follows:

1.  <a id="ref-for-valdef-width-auto④"></a>

    First, resolve any [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) width of the middle box (B): Assume there are two boxes, B and AC, where each of AC’s dimensions is double the maximum of A and C. (This preserves B’s centering.) Distribute the space to these two boxes (B and the imaginary AC) as described for A and C [above](#flex-fit).

2.  <a id="ref-for-outer-width④"></a>

    <a id="ref-for-available-width⑤"></a>

    <a id="ref-for-valdef-width-auto⑤"></a>

    Then, resolve any [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) widths of the side boxes (A and C) by setting that box’s outer width to <code>(<a href="#available-width">available&#x20;width</a>&#x20;−&#x20;used&#x20;<a href="#outer-width">outer&#x20;width</a>s&#x20;of&#x20;B)&#x20;÷&#x20;2</code>

<a id="ref-for-propdef-min-width③"></a>

<a id="ref-for-propdef-max-width①"></a>

##### <a id="variable-minmax"></a>5.3.2.3. Handling [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) and [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width)

<a id="ref-for-propdef-min-width④"></a>

<a id="ref-for-propdef-max-width②"></a>

The [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) and [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width) properties [\[CSS21\]](#biblio-css21) apply to page-margin boxes in the variable dimension like on normal elements, except that the three boxes on the same side are considered together.

More precisely:

1.  <a id="ref-for-propdef-max-width③"></a>

    <a id="ref-for-propdef-min-width⑤"></a>

    The tentative used widths are calculated (without [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) and [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width)) following the rules under [§5.3.2.2 Resolving auto widths](#variable-auto-sizing).

2.  <a id="ref-for-propdef-width④"></a>

    <a id="ref-for-propdef-max-width⑤"></a>

    <a id="ref-for-propdef-max-width④"></a>

    If the tentative used width of any of the three boxes is greater than [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width), the rules above are applied again, but this time using the computed value of [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width) as the computed value for [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width).

3.  <a id="ref-for-propdef-width⑤"></a>

    <a id="ref-for-propdef-min-width⑦"></a>

    <a id="ref-for-propdef-min-width⑥"></a>

    If the resulting width of any of the three boxes is smaller than [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width), the rules above are applied again, but this time using the value of [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width) as the computed value for [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width).

##### <a id="variable-position"></a>5.3.2.4. Positioning

Once the dimensions of the boxes are determined, they are positioned as follows:

- The left outer edge of A is flush with the left edge of the containing block
- The outer area of B is centered in the containing block.
- The right outer edge of C is flush with the right edge of the containing block.

##### <a id="variable-mapping"></a>5.3.2.5. Boxes on other sides

<a id="ref-for-at-ruledef-bottom-left②"></a>

<a id="ref-for-at-ruledef-bottom-center②"></a>

<a id="ref-for-at-ruledef-bottom-right②"></a>

<a id="ref-for-at-ruledef-top-left④"></a>

<a id="ref-for-at-ruledef-top-center③"></a>

<a id="ref-for-at-ruledef-top-right③"></a>

The used values for [@bottom-left](#at-ruledef-bottom-left), [@bottom-center](#at-ruledef-bottom-center) and [@bottom-right](#at-ruledef-bottom-right) page-margin boxes are established by the same rules as for [@top-left](#at-ruledef-top-left), [@top-center](#at-ruledef-top-center), and [@top-right](#at-ruledef-top-right), respectively.

<a id="ref-for-at-ruledef-left-top②"></a>

<a id="ref-for-at-ruledef-left-middle②"></a>

<a id="ref-for-at-ruledef-left-bottom②"></a>

The used values for [@left-top](#at-ruledef-left-top), [@left-middle](#at-ruledef-left-middle) and [@left-bottom](#at-ruledef-left-bottom) boxes are established by the same rules, with "width" replaced by "height", "left" by "top", "right" by "bottom" and "center" by "middle".

<a id="ref-for-at-ruledef-right-top②"></a>

<a id="ref-for-at-ruledef-right-middle②"></a>

<a id="ref-for-at-ruledef-right-bottom②"></a>

<a id="ref-for-at-ruledef-left-top③"></a>

<a id="ref-for-at-ruledef-left-middle③"></a>

<a id="ref-for-at-ruledef-left-bottom③"></a>

The used values for [@right-top](#at-ruledef-right-top), [@right-middle](#at-ruledef-right-middle) and [@right-bottom](#at-ruledef-right-bottom) page-margin boxes are established by the same rules as for [@left-top](#at-ruledef-left-top), [@left-middle](#at-ruledef-left-middle) and [@left-bottom](#at-ruledef-left-bottom), respectively.

#### <a id="fixed-sizing"></a>5.3.3. Page-Margin Box Fixed Dimension Computation Rules

<a id="ref-for-at-ruledef-top-left-corner③"></a>

<a id="ref-for-at-ruledef-top-left⑤"></a>

<a id="ref-for-at-ruledef-top-center④"></a>

<a id="ref-for-at-ruledef-top-right④"></a>

<a id="ref-for-at-ruledef-top-right-corner②"></a>

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-propdef-margin-top"></a>

<a id="ref-for-propdef-margin-bottom"></a>

The rules below are used to calculate the used values of each [@top-left-corner](#at-ruledef-top-left-corner), [@top-left](#at-ruledef-top-left), [@top-center](#at-ruledef-top-center), [@top-right](#at-ruledef-top-right), and [@top-right-corner](#at-ruledef-top-right-corner) page-margin box’s [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height), [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top), and [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) properties:

1.  The following constraint must hold among the used values of the margin box’s properties:

    <a id="ref-for-propdef-margin-top①"></a>

    <a id="ref-for-propdef-border-top-width"></a>

    <a id="ref-for-propdef-padding-top"></a>

    <a id="ref-for-propdef-height②"></a>

    <a id="ref-for-propdef-padding-bottom"></a>

    <a id="ref-for-propdef-border-bottom-width"></a>

    <a id="ref-for-propdef-margin-bottom①"></a>

    [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) + [border-top-width](https://www.w3.org/TR/css3-background/#propdef-border-top-width) + [padding-top](https://www.w3.org/TR/css-box-3/#propdef-padding-top) + [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) + [padding-bottom](https://www.w3.org/TR/css-box-3/#propdef-padding-bottom) + [border-bottom-width](https://www.w3.org/TR/css3-background/#propdef-border-bottom-width) + [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) = top page margin

2.  <a id="ref-for-propdef-margin-bottom③"></a>

    <a id="ref-for-propdef-margin-top③"></a>

    <a id="ref-for-propdef-margin-bottom②"></a>

    <a id="ref-for-propdef-margin-top②"></a>

    <a id="ref-for-propdef-border-bottom-width①"></a>

    <a id="ref-for-propdef-padding-bottom①"></a>

    <a id="ref-for-valdef-width-auto⑥"></a>

    <a id="ref-for-propdef-height③"></a>

    <a id="ref-for-propdef-padding-top①"></a>

    <a id="ref-for-propdef-border-top-width①"></a>

    If [border-top-width](https://www.w3.org/TR/css3-background/#propdef-border-top-width) + [padding-top](https://www.w3.org/TR/css-box-3/#propdef-padding-top) + [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) (if it is not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)) + [padding-bottom](https://www.w3.org/TR/css-box-3/#propdef-padding-bottom) + [border-bottom-width](https://www.w3.org/TR/css3-background/#propdef-border-bottom-width), plus [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) and/or [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) if not auto, is larger than the height of the top page margin, then any auto values for [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) or [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) are, for the following rules, treated as zero.

3.  <a id="ref-for-propdef-margin-top⑤"></a>

    <a id="ref-for-valdef-width-auto⑦"></a>

    <a id="ref-for-propdef-margin-bottom④"></a>

    <a id="ref-for-propdef-margin-top④"></a>

    <a id="ref-for-propdef-height④"></a>

    If at this point all of [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height), [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top), and [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) have a computed value other than [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), the values are said to be "over-constrained". In this case, the specified value of [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) is treated as auto.

4.  If there is now exactly one value specified as auto, its used value follows from the equality.

5.  <a id="ref-for-propdef-height⑥"></a>

    <a id="ref-for-valdef-width-auto⑧"></a>

    <a id="ref-for-propdef-height⑤"></a>

    If [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) is set to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), any other auto values become 0 and [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) follows from the resulting equality

6.  <a id="ref-for-propdef-margin-bottom⑤"></a>

    <a id="ref-for-propdef-margin-top⑥"></a>

    If both [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) are auto, their used values are equal. This vertically centers the page-margin box content within the top page margin.

<a id="ref-for-propdef-margin-bottom⑥"></a>

<a id="ref-for-propdef-margin-top⑦"></a>

The same rules apply to the bottom page-margin boxes (bottom-left-corner, bottom-left, bottom-center, bottom-right, and bottom-right-corner), except that in the overconstrained case, the [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) is ignored rather than the [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top).

<a id="ref-for-propdef-width⑥"></a>

<a id="ref-for-propdef-margin-left①"></a>

<a id="ref-for-propdef-margin-right①"></a>

Analogous rules govern the properties for the left and right page-margin boxes with respect to [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) (top-left-corner, left-top, left-middle, left-bottom, and bottom-left-corner; top-right-corner, right-top, right-middle, right-bottom, bottom-right-corner), with "top" replaced by "left", "bottom" replaced by "right", and "height" replaced by "width". In the overconstrained case for left (right) page-margin boxes, the specified value of [margin-left](https://www.w3.org/TR/css-box-3/#propdef-margin-left) ([margin-right](https://www.w3.org/TR/css-box-3/#propdef-margin-right)) is ignored.

### <a id="margin-box-ex"></a>5.4.  Page-margin box examples

The following is a collection of examples of page-margin box usage.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4c59b2b2"></a> Here is an example of a page with only a top-left header:
>
> ```text
> @page {
>   @top-left { content: "Header in Left Cell (top-left)" }
> }
> ```
>
> Because there are no contents defined for the top-center or the top-right page-margin boxes, the extent of the top-left page-margin box is allowed to cross the center of the page box.
>
> ![Header Example 1](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-1.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-de87cb02"></a> The following is an example of a page with a centered header:
>
> ```text
> @page {
>   @top-center { content: "Header in Center Cell (top-center)" }
> }
> ```
>
> ![Header Example 2](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-2.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7c6898f8"></a> The following is an example of a page with a single header in the top-right page-margin box:
>
> ```text
> @page {
>   @top-right { content: "Header in Right Cell (top-right)" }
> }
> ```
>
> Because the content of the center cell is empty, the extent of the top-right page-margin box is allowed to cross the center of the page box.
>
> ![Header Example 3](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-3.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d8635b6e"></a> The following is an example of a page with a top-center and a top-left header:
>
> ```text
> @page {
>   @top-left { content: "Left Cell (top-left)" }
>   @top-center { content: "Header in Center Cell (top-center)" }
> }
> ```
>
> ![Header Example 4](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-4.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b1235340"></a> The following is an example of a page with a top-center and a top-right header:
>
> ```text
> @page {
>   @top-center { content: "Header in Center Cell (top-center)" }
>   @top-right { content: "Right Cell (top-right)" }
> }
> ```
>
> margin: 10%;
>
> ![Header Example 5](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-5.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-03fa12f3"></a> The following is an example of a page with top-left and top-right headers:
>
> ```text
> @page {
>   @top-left { content: "Header in top-left with approx. twice as many words as right cell." }
>   @top-right { content: "Right cell (top-right)" }
> }
> ```
>
> Because there are no center cell contents, the extent of the top-left is allowed to cross the center of the page box.
>
> ![Header Example 6](https://www.w3.org/TR/2018/WD-css-page-3-20181018/images/header-ex-6.png)

## <a id="page-properties"></a>6.  Page Properties

<a id="ref-for-page-context②"></a>

<a id="ref-for-descdef-page-size①"></a>

[Appendix A](#properties-list) defines the normative list of CSS 2.1 [\[CSS21\]](#biblio-css21) [properties that apply to page boxes](#page-property-list). If a conforming user agent supports any of these properties on block boxes, then it must also support that property in the [page context](#page-context). This specification additionally defines the [size](#descdef-page-size) property that only applies in the page context.

<a id="ref-for-valdef-all-inherit"></a>

Properties that apply to the page-margin boxes can also be set within the page context: if inheritable or explicitly inherited (with the [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit) keyword in the margin context), they will inherit to the page-margin boxes.

<a id="ref-for-margin-context①"></a>

The same appendix defines the normative list of CSS 2.1 [\[CSS21\]](#biblio-css21) [properties that apply to page-margin boxes](#page-property-list). If a conforming user agent supports any of these properties on block boxes, then it must also support that property in the [margin context](#margin-context).

Other properties defined by [\[CSS21\]](#biblio-css21) do not apply in these contexts. Behavior for properties not included in CSS 2.1 is undefined.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The intent of leaving other properties undefined is to allow the gradual addition of appropriate CSS3 properties as they emerge, without having to update this specification with each addition.

[As with elements in the document](https://www.w3.org/TR/CSS21/cascade.html#value-stages), both the page context and the margin context have a computed value for every property, even if that property does not apply to the page or page-margin box.

The normal rules for CSS properties apply with the following exceptions:

- page-margin boxes inherit from the page context. The page context inherits from the root element. However, since the previous revision of CSS Paged Media Level 3 did not specify this point, an implementation that sets inherited properties in the page context to their initial values (as for the root element) is also conformant to CSS Paged Media Level 3. Note that this exception will be removed in Level 4.

- <a id="ref-for-propdef-font-size③"></a>

  <a id="ref-for-ex①"></a>

  <a id="ref-for-em①"></a>

  <a id="ref-for-propdef-font-size②"></a>

  <a id="ref-for-propdef-font-size①"></a>

  <a id="ref-for-propdef-font-size"></a>

  <a id="ref-for-ex"></a>

  <a id="ref-for-em"></a>

  Values in units of [em](https://www.w3.org/TR/css-values-4/#em) and [ex](https://www.w3.org/TR/css-values-4/#ex) are interpreted relative to the font associated with their context. When used on the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) property in the margin context, they are relative to the font of the page context. When used on the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) property in the page context, they are relative to the [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) of the root element. However, since a previous revision of CSS Paged Media Level 3 was ambiguous on this point, an implementation that treats [em](https://www.w3.org/TR/css-values-4/#em) and [ex](https://www.w3.org/TR/css-values-4/#ex) on [font-size](https://www.w3.org/TR/css-fonts-3/#propdef-font-size) as relative to the initial value is also conformant to CSS Paged Media Level 3. Note that this exception will be removed in Level 4.

- Percentage values on the margin and padding properties are relative to the dimensions of the containing block. For right and left values, percentages are relative to the width of the containing block; for top and bottom values, percentages are relative to the height of the containing block.

- <a id="ref-for-propdef-height⑦"></a>

  <a id="ref-for-propdef-width⑦"></a>

  The used values of [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width) and [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height) have special computation rules for page boxes and page-margin boxes; see [Page Size](#page-size) and [Computing Page-Margin Box Dimensions](#margin-dimension).

- The page background is positioned and painted [as described above](#painting).

- The rules for counter scoping are modified [as described below](#page-based-counters).

- <a id="ref-for-valdef-content-none②"></a>

  <a id="ref-for-propdef-content②"></a>

  <a id="ref-for-valdef-content-normal"></a>

  As on the '::before' and '::after' pseudo-elements, the [normal](https://www.w3.org/TR/css-content-3/#content-property) value of the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property computes to [none](https://www.w3.org/TR/css-content-3/#content-property) on page-margin boxes.

- <a id="ref-for-propdef-vertical-align"></a>

  On page-margin boxes, the [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property behaves [as specified for table cells](https://www.w3.org/TR/CSS21/tables.html#height-layout). It <em>always</em> performs alignment in the vertical dimension, regardless of writing mode.

User agents should establish a default page margin via the user agent stylesheet that includes any non-printable area. Authors should assume that the default page area will not include unprintable regions.

### <a id="page-based-counters"></a>6.1.  Page-based counters

<a id="ref-for-at-ruledef-page①⓪"></a>

Counters can be defined and controlled within an [@page](#at-ruledef-page) rule, and used as content in page-margin boxes. This is useful for maintaining a page count.

<a id="ref-for-propdef-counter-increment"></a>

A [counter-increment](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-increment) within either a page or margin context causes the counter to increment with the generation of each page box.

If a counter is reset or incremented within the page context, it is in scope for all page-margin boxes and obscures all counters of the same name within the document.

If a counter is reset or incremented within a margin context, it is in scope for that page-margin box and obscures any counters of the same name in both the page context and the document.

If a counter that has not been reset or incremented within the margin context or the page context is used by counter() or counters() in the margin context, then the resultant value is exactly as if the page-margin box were an element within the document at the start of the page, inside the deepest element in the normal flow that spans the page break. Use of the counter in this way does not affect the calculation of the counter’s value.

<a id="ref-for-propdef-counter-increment①"></a>

<a id="ref-for-page-context③"></a>

<a id="ref-for-propdef-counter-increment②"></a>

<a id="ref-for-propdef-counter-reset"></a>

A counter named page is automatically created and incremented by 1 on every page of the document, unless the [counter-increment](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-increment) property in the [page context](#page-context) explicitly specifies a different increment for the page counter. The implied page counter is a real counter, and can be directly affected using the [counter-increment](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-increment) and [counter-reset](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-reset) properties when named explicitly in those properties. It can also be used in the 'counter()' and 'counters()' function forms.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-13051aa4"></a> The following rules result in the placement of the current page number in the middle of the outside margin of each page.
>
> ```text
> @page {
>   margin: 10%;
> 
>   @top-center {
>     font-family: sans-serif;
>     font-weight: bold;
>     font-size: 2em;
>     content: counter(page);
>   }
> }
> ```
>
> Adding the following rule will make all pages even-numbered.
>
> ```text
> @page {
>   counter-increment: page 2;
> }
> ```
<a id="ref-for-propdef-counter-reset①"></a>

<a id="ref-for-propdef-counter-increment③"></a>

Additionally, a counter named pages is automatically created by the UA. Its value is always the total number of pages in the document. (In continuous media this is always 1.) The value of pages cannot be manipulated: while [counter-reset](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-reset) and [counter-increment](https://www.w3.org/TR/CSS21/generate.html#propdef-counter-increment) statements that set it are valid, they have no effect.

In all other respects, page-associated counters behave as described in [\[CSS21\]](#biblio-css21), [Nested Counters and Scope](https://www.w3.org/TR/CSS21/generate.html#scope) and [Counters](https://www.w3.org/TR/CSS21/syndata.html#counter).

### <a id="margin-text-alignment"></a>6.2.  Page-margin boxes and default values

Properties used within page or margin contexts take their initial values from their respective property definitions; however, user agents must behave as though the values in the following table were established by rules in the UA default style sheet.

<a id="margin-values"></a>



| Page-margin box                                     | <a id="ref-for-propdef-text-align"></a>[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) | <a id="ref-for-propdef-vertical-align①"></a>[vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) |
|-----------------------------------------------------|---------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| [top-left-corner](#top-left-corner-box-def)         | right                                                                                 | middle                                                                                          |
| [top-left](#top-left-box-def)                       | left                                                                                  | middle                                                                                          |
| [top-center](#top-center-box-def)                   | center                                                                                | middle                                                                                          |
| [top-right](#top-right-box-def)                     | right                                                                                 | middle                                                                                          |
| [top-right-corner](#top-right-corner-box-def)       | left                                                                                  | middle                                                                                          |
| [left-top](#left-top-box-def)                       | center                                                                                | top                                                                                             |
| [left-middle](#left-middle-box-def)                 | center                                                                                | middle                                                                                          |
| [left-bottom](#left-bottom-box-def)                 | center                                                                                | bottom                                                                                          |
| [right-top](#right-top-box-def)                     | center                                                                                | top                                                                                             |
| [right-middle](#right-middle-box-def)               | center                                                                                | middle                                                                                          |
| [right-bottom](#right-bottom-box-def)               | center                                                                                | bottom                                                                                          |
| [bottom-left-corner](#bottom-left-corner-box-def)   | right                                                                                 | middle                                                                                          |
| [bottom-left](#bottom-left-box-def)                 | left                                                                                  | middle                                                                                          |
| [bottom-center](#bottom-center-box-def)             | center                                                                                | middle                                                                                          |
| [bottom-right](#bottom-right-box-def)               | right                                                                                 | middle                                                                                          |
| [bottom-right-corner](#bottom-right-corner-box-def) | left                                                                                  | middle                                                                                          |

Table 2. Default values for Page-Margin Boxes



> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6e92ac02"></a> This example style sheet could be used to create a centered header with the current chapter name:
>
> ```text
> body {counter-reset: chapter;}
> div.chapter {counter-increment: chapter;}
> @page {
>   margin: 10%;
>   @top-center { content: "Chapter" counter(chapter) }
> }
> ```
## <a id="page-size"></a>7.  Page Size

People around the world use many different paper sizes. It is a goal of this specification that web content should be adaptable to a range of different sizes without having to write a specific style sheet for each paper size.

<a id="ref-for-descdef-page-size②"></a>

However, in some situations it is important that a certain page size achieves a certain style. One way to achieve this goal is to utilize the [size](#descdef-page-size) property, which indicates that the document should preferentially be displayed on a surface of a certain size; another method is to use Media Queries [\[MEDIAQ\]](#biblio-mediaq) which allow different style sheets to be applied to different page sizes.

<a id="ref-for-descdef-page-size③"></a>

### <a id="page-size-prop"></a>7.1.  Page size: the [size](#descdef-page-size) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-page-size"></a>size                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-page①①"></a>[@page](#at-ruledef-page)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one⑤"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-page-size-page-size"></a><a id="ref-for-comb-one④"></a><a id="ref-for-comb-one③"></a><a id="ref-for-mult-num-range"></a><a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css3-values/#length-value)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<page-size\>](#typedef-page-size-page-size) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ portrait [\|](https://www.w3.org/TR/css-values-4/#comb-one) landscape \] \] |
| <strong>Initial:&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>Computed value:&#xA;      </strong> | <a id="ref-for-length-value①"></a>specified value, with [\<length\>](https://www.w3.org/TR/css3-values/#length-value)s made absolute.                                                                                                                                                                                                                                                                                                                                                                                                                                        |



<a id="ref-for-page-box⑥"></a>

<a id="ref-for-page-sheet"></a>

<a id="ref-for-descdef-page-size④"></a>

This property specifies the target size and orientation of the [page box](#page-box)’s containing block. In the general case, where one page box is rendered onto one [page sheet](#page-sheet), the [size](#descdef-page-size) property also indicates the size of the destination page sheet.

The size of a page box can either be "absolute" (fixed size) or "scalable" (i.e., fitting available sheet sizes). The first three values in the table below can be used to create scalable page boxes. Other values define a fixed-size page box, and thereby indicate the preferred output media size. When possible, output should be rendered on the media size indicated. If the specified size is not available, the rules for [transposing a page box to a different size](#renderingpages) apply.

<a id="ref-for-descdef-page-size⑤"></a>

<a id="ref-for-descdef-page-size⑥"></a>

<a id="ref-for-at-ruledef-page①②"></a>

If a [size](#descdef-page-size) property declaration is qualified by a width, height, device-width, device-height, aspect-ratio, device-aspect-ratio or orientation media query [\[MEDIAQ\]](#biblio-mediaq) (or other conditional on the size of the paper), then the declaration must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore). Media queries do not honor [size](#descdef-page-size): they assume the paper size that would be chosen if no [@page](#at-ruledef-page) rules were specified.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e030ddc1"></a> It would be useful if media queries could respond at least to sizes specified on an unqualified @page.
>
> <a id="ref-for-at-ruledef-viewport"></a>
>
> <a id="ref-for-at-ruledef-page①③"></a>
>
> Another option could be to do like [@viewport](https://www.w3.org/TR/css-device-adapt/#at-ruledef-viewport) rules [\[CSS-DEVICE-ADAPT\]](#biblio-css-device-adapt): First apply [@page](#at-ruledef-page) rules (matching which selectors?), using the UA’s default page size for Media Queries and [viewport-percentage lengths](https://www.w3.org/TR/css3-values/#viewport-relative-lengths) [\[CSS3VAL\]](#biblio-css3val). The resulting page size is the "base page size". The entire set of stylesheets is applied again, this time using the "base page size" for Media Queries and viewport-percentage lengths.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8dd5da73"></a> In the following example
>
> ```text
> @page {
>   size: 4in 6in;
> }
> 
> @media (max-width: 6in) {
>   @page {
>     size: letter;
>   }
> }
> ```
>
> The second `size` declaration is ignored, i.e. the specified value of the `size` property is `4in 6in`.

<a id="valdef-page-size-auto"></a>auto

The page box will be set to a size and orientation chosen by the UA. In the usual case, the page box size and orientation is chosen to match the target media sheet.

<a id="valdef-page-size-landscape"></a>landscape

<a id="ref-for-typedef-page-size-page-size①"></a>

Specifies that the page’s content be printed in landscape orientation. The longer sides of the page box are horizontal. If a [\<page-size\>](#typedef-page-size-page-size) is not specified, the size of the page sheet is chosen by the UA.

<a id="valdef-page-size-portrait"></a>portrait

<a id="ref-for-typedef-page-size-page-size②"></a>

Specifies that the page’s content be printed in portrait orientation. The shorter sides of the page box are horizontal. If a [\<page-size\>](#typedef-page-size-page-size) is not specified, the size of the page sheet is chosen by the UA.

<a id="ref-for-length-value②"></a>

<a id="valdef-page-size-length"></a>[\<length\>](https://www.w3.org/TR/css3-values/#length-value)

<a id="ref-for-ex②"></a>

<a id="ref-for-em②"></a>

The page box will be set to the given absolute dimension(s). If only one length value is specified, it sets both the width and height of the page box (i.e., the box is a square). If two length values are specified, the first establishes the page box width, and the second the page box height. Values in units of [em](https://www.w3.org/TR/css-values-4/#em) and [ex](https://www.w3.org/TR/css-values-4/#ex) refer to the page context’s font. Negative lengths are illegal.

<a id="ref-for-typedef-page-size-page-size③"></a>

<a id="typedef-page-size-page-size"></a>[\<page-size\>](#typedef-page-size-page-size)

<a id="ref-for-descdef-page-size⑦"></a>

A page size can be specified using one of the following media names. This is the equivalent of specifying [size](#descdef-page-size) using length values. The definition of the the media names comes from Media Standardized Names [\[PWGMSN\]](#biblio-pwgmsn).

<a id="valdef-page-size-a5"></a>A5  
Equivalent to the size of ISO A5 media: 148mm wide and 210 mm high.

<a id="valdef-page-size-a4"></a>A4  
Equivalent to the size of ISO A4 media: 210 mm wide and 297 mm high.

<a id="valdef-page-size-a3"></a>A3  
Equivalent to the size of ISO A3 media: 297mm wide and 420mm high.

<a id="valdef-page-size-b5"></a>B5  
Equivalent to the size of ISO B5 media: 176mm wide by 250mm high.

<a id="valdef-page-size-b4"></a>B4  
Equivalent to the size of ISO B4 media: 250mm wide by 353mm high.

<a id="valdef-page-size-jis-b5"></a>JIS-B5  
Equivalent to the size of JIS B5 media: 182mm wide by 257mm high.

<a id="valdef-page-size-jis-b4"></a>JIS-B4  
Equivalent to the size of JIS B4 media: 257mm wide by 364mm high.

<a id="valdef-page-size-letter"></a>letter  
Equivalent to the size of North American letter media: 8.5 inches wide and 11 inches high

<a id="valdef-page-size-legal"></a>legal  
Equivalent to the size of North American legal: 8.5 inches wide by 14 inches high.

<a id="valdef-page-size-ledger"></a>ledger  
Equivalent to the size of North American ledger: 11 inches wide by 17 inches high.

<a id="ref-for-typedef-page-size-page-size④"></a>

<a id="ref-for-valdef-page-size-landscape①"></a>

<a id="ref-for-valdef-page-size-portrait①"></a>

The [\<page-size\>](#typedef-page-size-page-size) names can be used in conjunction with [landscape](#valdef-page-size-landscape) or [portrait](#valdef-page-size-portrait) to indicate both size and orientation.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3998207e"></a>
>
> ```text
> @page {
> size: A4 landscape;
> }
> ```
>
> The above example sets the width of the page box to be 297mm and the height to be 210mm. The page box in this example should be rendered on a page sheet size of 210 mm by 297 mm.

<a id="ref-for-propdef-margin"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e0739c4c"></a> In the following example, the outer edges of the page box will align with the page. The percentage value on the [margin](https://www.w3.org/TR/CSS21/box.html#propdef-margin) property is relative to the page size so if the page sheet dimensions are 210mm x 297mm (i.e., A4), the margins are 21mm and 29.7mm. Assuming there are no page borders or padding set in the UA default style sheet, the resulting page area is 189mm by 367.3mm (210mm-21mm by 297mm-29.7mm).
>
> ```text
> @page {
> size: auto;/* auto is the initial value */
> margin: 10%;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bd008567"></a>
>
> ```text
> @page {
> size: 8.5in 11in;/* width height */
> }
> ```
>
> <a id="ref-for-valdef-page-size-portrait②"></a>
>
> The above example sets the width of the page box to be 8.5 inches and the height to be 11 inches. This indicates that the page sheet size should be 8.5"x11" and the orientation [portrait](#valdef-page-size-portrait).

#### <a id="page-size-media-query"></a>7.1.1.  Media Queries

<em>This section is informative.</em>

By using Media Queries [\[MEDIAQ\]](#biblio-mediaq), one style sheet can express different stylistic preferences for different page sizes. Consider this example:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-961a4690"></a>
>
> ```text
> /* style sheet for "A4" printing */
> @media print and (width: 21cm) and (height: 29.7cm) {
>   @page {
>      margin: 3cm;
>   }
>  }
> 
> /* style sheet for "letter" printing */
> @media print and (width: 8.5in) and (height: 11in) {
>   @page {
>   margin: 1in;
>   }
> }
> ```
In the example above, "A4" sheets are given a "3cm" page margin, and "letter" sheets are given a "1in" page margin.

<a id="ref-for-descdef-page-marks"></a>

### <a id="marks"></a>7.2.  Crop and Registration Marks: the [marks](#descdef-page-marks) property



| Field               | Definition                                                                                                                                                          |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-page-marks"></a>marks                                                                                                                                            |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-page①④"></a>[@page](#at-ruledef-page)                                                                                                                        |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-any①"></a><a id="ref-for-comb-one⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ crop [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) cross \] |
| <strong>Initial:&#xA;      </strong> | none                                                                                                                                                                |
| <strong>Computed value:&#xA;      </strong> | as specified                                                                                                                                                        |



This property adds crop and/or registration marks to the document. These are printed outside the page box to facilitate the trimming and alignment of sheets of paper. Values have the following meanings:

<a id="valdef-page-marks-none"></a>none  
<a id="ref-for-bleed-area①"></a>

Specifies that neither crop marks nor registration marks should be printed: the area outside the [bleed area](#bleed-area) will be completely blank.

<a id="valdef-page-marks-crop"></a>crop  
Specifies that crop marks should be printed. These are typically short lines outside the page box that are effectively extensions of the page box’s four edges, thereby indicating the precise location of those edges without placing any ink near or within the page box itself.

<a id="valdef-page-marks-cross"></a>cross  
Specifies that registration marks should be printed. These are typically cross-shaped marks outside each edge of the page box used to align sheets of paper during the printing process.

Note that crop marks and registration marks are only visible if the page box is smaller than the printable area.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dde4e44f"></a> To set crop and cross marks on a document, this code can be used:
>
> ```text
> @page { marks: crop cross }
> ```
<a id="ref-for-descdef-page-bleed"></a>

### <a id="bleed"></a>7.3.  Bleed Area: the [bleed](#descdef-page-bleed) property



| Field               | Definition                                                                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-page-bleed"></a>bleed                                                                                                                                    |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-page①⑤"></a>[@page](#at-ruledef-page)                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value③"></a><a id="ref-for-comb-one⑦"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css3-values/#length-value) |
| <strong>Initial:&#xA;      </strong> | auto                                                                                                                                                        |
| <strong>Computed value:&#xA;      </strong> | as specified                                                                                                                                                |



This property specifies the extent of the <a id="bleed-area"></a>bleed area outside the page box; in other words the extent beyond the page box at which the page rendering is clipped. Values have the following meanings:

<a id="valdef-page-bleed-auto"></a>auto

<a id="ref-for-valdef-page-marks-crop"></a>

<a id="ref-for-descdef-page-marks①"></a>

Computes to 6pt if [marks](#descdef-page-marks) has [crop](#valdef-page-marks-crop) and to zero otherwise.

<a id="ref-for-length-value④"></a>

<a id="valdef-page-bleed-length"></a>[\<length\>](https://www.w3.org/TR/css3-values/#length-value)

<a id="ref-for-page-box⑦"></a>

<a id="ref-for-bleed-area②"></a>

Specifies by how far outward, in each direction, the [bleed area](#bleed-area) extends past the [page box](#page-box). Values may be negative, but there may be implementation-specific limits.

### <a id="renderingpages"></a>7.4.  Rendering page boxes that do not fit a page sheet

If a page box does not match the target page sheet dimensions, the user agent should do one of the following (in order of preference):

1.  Render the page box at the indicated size on a larger page sheet.
2.  Rotate the page box 90° if this will make the page box fit the page sheet.
3.  Scale the page box to fit the page sheet. (The aspect ratio of the page box should be preserved.)
4.  Graphically "slice" the page box onto multiple page sheets.
5.  Clip overflowed content (least preferred).

The user agent may wish to consult the user before performing these operations.

### <a id="positioning-page-box"></a>7.5.  Positioning the page box on the sheet

When the page box is smaller than the page sheet, the user agent should either:

- center the page box on the sheet since this will align double-sided pages and avoid accidental loss of information that is printed near the edge of the sheet; or

- <a id="ref-for-propdef-writing-mode①"></a>

  <a id="ref-for-propdef-direction①"></a>

  position the page box in the upper left corner of the page sheet (or another corner, based on the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) and [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) properties of the page box) as this may minimize media consumption.

The user agent may wish to consult the user in this regard.

## <a id="page-breaks"></a>8.  <a id="pg-br-before-after"></a> <a id="page-break-before"></a> <a id="page-break-after"></a> <a id="page-break-inside"></a> <a id="breaks-inside"></a> <a id="orphans"></a> <a id="widows"></a> <a id="allowed-pg-brk"></a> <a id="brk-btw-blocks"></a> <a id="brk-btw-lines"></a> <a id="brk-end-block"></a> <a id="forced-pg-brk"></a> <a id="best-pg-brk"></a> Page Breaks 

<a id="ref-for-page-box⑧"></a>

The CSS Fragmentation Module [\[CSS3-BREAK\]](#biblio-css3-break) module defines how and where CSS boxes can be <em>fragmented</em>, including across page breaks. It defines a few properties that indicate where the user agent may or must break pages, and on what page (left or right) the subsequent content resumes. Each page break ends layout in the current [page box](#page-box) and causes remaining pieces of the document tree to be laid out in a new page box.

<a id="ref-for-propdef-page①"></a>

### <a id="using-named-pages"></a>8.1.  Using named pages: [page](#propdef-page)



| Field               | Definition                                                                                                                                                             |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-page"></a>page                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-identifier-value"></a><a id="ref-for-comb-one⑧"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                   |
| <strong>Applies to:&#xA;      </strong> | boxes that create [class A](https://www.w3.org/TR/css3-break/#btw-blocks) break points                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no (but see prose)                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                        |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations-1/#animating-properties">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                               |



<a id="ref-for-propdef-page②"></a>

<a id="ref-for-forced-break"></a>

The [page](#propdef-page) property is used to specify a particular type of page (called a <a id="named-page"></a>named page) on which an element must be displayed. If necessary, a [forced page break](https://www.w3.org/TR/css3-break/#forced-break) is introduced and a new page generated of the specified type.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This page can be styled by using the same type name in a [page selector](#page-selectors).

Page type names are case-sensitive identifiers. However the auto value, being a CSS keyword, is [ASCII case-insensitive](https://www.w3.org/TR/CSS21/syndata.html#characters).

<a id="ref-for-propdef-page③"></a>

<a id="ref-for-propdef-page④"></a>

The [page](#propdef-page) property does not inherit. However, if the [page](#propdef-page) value on an element is auto, then its used value is the value specified on its nearest ancestor with a non-auto value. When specified on the root element, the used value for auto is the empty string.

<a id="ref-for-propdef-page⑤"></a>

<a id="ref-for-propdef-page⑥"></a>

Because a previous version of this specification indicated that the [page](#propdef-page) property is inherited, an implementation that inherits the [page](#propdef-page) property and treats auto as always naming the empty string remains conformant to CSS Paged Media Level 3. Note that this exception will be removed in Level 4. Therefore authors should not explicitly specify the auto value on a descendant of an element with a non-auto value, as the resulting behavior will be unpredictable.

<a id="ref-for-propdef-page⑦"></a>

The [page](#propdef-page) property works as follows:

1.  First, any auto values are resolved against non-auto ancestors (as specified above).

2.  <a id="ref-for-propdef-page①⓪"></a>

    <a id="ref-for-end-page-value"></a>

    <a id="ref-for-start-page-value"></a>

    <a id="ref-for-propdef-page⑨"></a>

    <a id="ref-for-propdef-page⑧"></a>

    Next, a <a id="start-page-value"></a>start [page](#propdef-page) value and <a id="end-page-value"></a>end [page](#propdef-page) value is determined for each box as the value (if any) propagated from its first or last child box (respectively), else the used value on the box itself. A child propagates its own [start](#start-page-value) or [end page value](#end-page-value) if and only if the [page](#propdef-page) property applies to it.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: A first or last child <em>box</em> is not always generated by a first or last child <em>element</em>. For example, an element could only have a previous sibling with 'display: none' which does not generate any box.

3.  <a id="ref-for-start-page-value①"></a>

    The first printed page’s type is the [start page value](#start-page-value) of the root.

4.  <a id="ref-for-start-page-value②"></a>

    <a id="ref-for-end-page-value①"></a>

    If for any two boxes meeting at a [class A](https://www.w3.org/TR/css3-break/#btw-blocks) break point, the [end page value](#end-page-value) of the box preceding the break and [start page value](#start-page-value) of the box succeeding the break do not match, then a page break is forced between the two boxes, and content after the break resumes on a page box of the named type.

<a id="ref-for-propdef-page①①"></a>

<a id="ref-for-propdef-page①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Essentially, the two [page](#propdef-page) values compared are those from the deepest boxes meeting at the [class A break point](https://www.w3.org/TR/css3-break/#btw-blocks), ignoring any subtrees rooted by boxes to which the [page](#propdef-page) property does not apply.

See [\[CSS3-BREAK\]](#biblio-css3-break) for additional details on page breaks.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8e43e846"></a> In this example, the two tables are rendered on landscape pages (indeed, on the same page, if they fit). The page type "narrow" is used for the \<p\> after the second table, as the page properties for the table element are no longer in effect:
>
> ```text
> @page narrow { size: 9cm 18cm }
> @page rotated { size: landscape }
> div { page: narrow }
> table { page: rotated }
> ```
>
> with this document:
>
> ```text
> <div>
> <table>...</table>
> <table>...</table>
> <p>This text is rendered on a 'narrow' page</p>
> </div>
> ```
<a id="ref-for-propdef-page①③"></a>

<a id="ref-for-at-ruledef-page①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1652bb67"></a> In Japanese documents, sometimes different parts of a single document will have different <em lang="ja-Latn">kihon hanmen</em>. [\[JLREQ\]](#biblio-jlreq) The [page](#propdef-page) property, together with [@page](#at-ruledef-page) rules specifying different page widths, can accomodate this type of layout:
>
> ```text
> <!DOCTYPE html>
> <html lang="ja">
>   <style>
>     html   {
>       writing-mode: vertical-rl;
>       line-height: 1.6;
>     }
>     .main  {
>       page: main;
>       columns: 2;
>       column-gap: 1rem;
>     }
>     .index {
>       page: index;
>       columns: 3;
>       column-gap: 1rem;
>     }
>     @page {
>       margin: auto;  /* center kihon hanmen on page */
>       width:  40rem; /* 1.6 × 25 lines        */
>     }
>     @page main  { height: 61rem; } /* 2 × 30 chars + 1 × gap */
>     @page index { height: 62rem; } /* 3 × 20 chars + 2 × gap */
>   </style>
>   <section class="main"> ... </section>
>   <section class="index"> ... </section>
> </html>
> ```
## <a id="image-properties"></a>9.  Image Properties

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This section has been moved to [\[CSS3-IMAGES\]](#biblio-css3-images).

## <a id="properties-list"></a> Appendix A: Applicable CSS2.1 Properties

### <a id="page-property-list"></a> CSS 2.1 Properties that apply within the page context

This list defines the <a id="page-property"></a>page properties. They are further described in [§6 Page Properties](#page-properties).

**Table 7**

Representation note: each source property group retains the members covered by its rowspan. Source full-width standalone properties remain separate list items.

**[bidi properties](https://www.w3.org/TR/CSS21/visuren.html#direction)**

- direction

**[background properties](https://www.w3.org/TR/CSS21/colors.html#background-properties)**

- background-color
- background-image
- background-repeat
- background-attachment
- background-position
- background

**[border properties](https://www.w3.org/TR/CSS21/box.html#border-properties)**

- border-top-width
- border-right-width
- border-bottom-width
- border-left-width
- border-width
- border-top-color
- border-right-color
- border-bottom-color
- border-left-color
- border-color
- border-top-style
- border-right-style
- border-bottom-style
- border-left-style
- border-short-style
- border-top
- border-right
- border-bottom
- border-left
- border

**[counter properties](https://www.w3.org/TR/CSS21/generate.html#counters)**

- counter-reset
- counter-increment

- [color](https://www.w3.org/TR/CSS21/colors.html#propdef-color)

**[font properties](https://www.w3.org/TR/CSS21/fonts.html)**

- font-family
- font-size
- font-style
- font-variant
- font-weight
- font

**[height](https://www.w3.org/TR/CSS21/visudet.html#the-height-property) [properties](https://www.w3.org/TR/CSS21/visudet.html#min-max-heights)**

- height
- min-height
- max-height

- [line-height](https://www.w3.org/TR/CSS21/visudet.html#line-height)

**[margin properties](https://www.w3.org/TR/CSS21/box.html#margin-properties)**

- margin-top
- margin-right
- margin-bottom
- margin-left
- margin

**[outline properties](https://www.w3.org/TR/CSS21/ui.html#dynamic-outlines)**

- outline-width
- outline-style
- outline-color
- outline

**[padding properties](https://www.w3.org/TR/CSS21/box.html#padding-properties)**

- padding-top
- padding-right
- padding-bottom
- padding-left
- padding

- [quotes](https://www.w3.org/TR/CSS21/generate.html#quotes-specify)

**[text properties](https://www.w3.org/TR/CSS21/text.html)**

- letter-spacing
- text-align
- text-decoration
- text-indent
- text-transform
- white-space
- word-spacing

- [visibility](https://www.w3.org/TR/CSS21/visufx.html#visibility)

**[width](https://www.w3.org/TR/CSS21/visudet.html#the-width-property) [properties](https://www.w3.org/TR/CSS21/visudet.html#min-max-widths)**

- width
- min-width
- max-width

### <a id="margin-property-list"></a> CSS 2.1 properties that apply within the margin contexts

This list defines the <a id="page-margin-property"></a>page-margin properties. They are further described in [§6 Page Properties](#page-properties).

**Table 8**

Representation note: each source property group retains the members covered by its rowspan. Source full-width standalone properties remain separate list items.

**[bidi properties](https://www.w3.org/TR/CSS21/visuren.html#direction)**

- direction
- unicode-bidi

**[background properties](https://www.w3.org/TR/CSS21/colors.html#background-properties)**

- background-color
- background-image
- background-repeat
- background-attachment
- background-position
- background

**[border properties](https://www.w3.org/TR/CSS21/box.html#border-properties)**

- border-top-width
- border-right-width
- border-bottom-width
- border-left-width
- border-width
- border-top-color
- border-right-color
- border-bottom-color
- border-left-color
- border-color
- border-top-style
- border-right-style
- border-bottom-style
- border-left-style
- border-short-style
- border-top
- border-right
- border-bottom
- border-left
- border

**[counter properties](https://www.w3.org/TR/CSS21/generate.html#counters)**

- counter-reset
- counter-increment

- content

- [color](https://www.w3.org/TR/CSS21/colors.html#propdef-color)

**[font properties](https://www.w3.org/TR/CSS21/fonts.html)**

- font-family
- font-size
- font-style
- font-variant
- font-weight
- font

**[height](https://www.w3.org/TR/CSS21/visudet.html#the-height-property) [properties](https://www.w3.org/TR/CSS21/visudet.html#min-max-heights)**

- height
- min-height
- max-height

- [line-height](https://www.w3.org/TR/CSS21/visudet.html#line-height)

**[margin properties](https://www.w3.org/TR/CSS21/box.html#margin-properties)**

- margin-top
- margin-right
- margin-bottom
- margin-left
- margin

**[outline properties](https://www.w3.org/TR/CSS21/ui.html#dynamic-outlines)**

- outline-width
- outline-style
- outline-color
- outline

- [overflow](https://www.w3.org/TR/CSS21/visufx.html#overflow)

**[padding properties](https://www.w3.org/TR/CSS21/box.html#padding-properties)**

- padding-top
- padding-right
- padding-bottom
- padding-left
- padding

- [quotes](https://www.w3.org/TR/CSS21/generate.html#quotes-specify)

**[text properties](https://www.w3.org/TR/CSS21/text.html)**

- letter-spacing
- text-align
- text-decoration
- text-indent
- text-transform
- white-space
- word-spacing

- [vertical-align](https://www.w3.org/TR/CSS21/visudet.html#line-height)

- [visibility](https://www.w3.org/TR/CSS21/visufx.html#visibility)

**[width](https://www.w3.org/TR/CSS21/visudet.html#the-width-property) [properties](https://www.w3.org/TR/CSS21/visudet.html#min-max-widths)**

- width
- min-width
- max-width

- [z-index](https://www.w3.org/TR/CSS21/visuren.html#z-index)

## <a id="transfer-possibilities"></a> Appendix B: Transfer Possibilities

Often, but not always, the page box has a one-to-one correspondence to the physical surface onto which the document is ultimately rendered. The CSS3 page model specifies formatting within the page box, but it is the user agent’s responsibility to transfer the page box to the sheet. Some user agent transfer possibilities that are not addressed by CSS3 include:

<a id="complex-usecases"></a>

- Transferring one page box to one sheet (e.g. single-sided printing);
- Transferring two page boxes to the front and back surfaces of the same sheet (e.g. double-sided printing);
- Transferring N (small) page boxes to one sheet (called "N-up");
- Transferring one (large) page box to N x M sheets (called "tiling");
- Creating signatures. A <a id="signature"></a>signature is a group of pages printed on a sheet, which, when folded and trimmed like a book, appear in their proper sequence;
- Printing one document to multiple output trays;
- Generating files containing print instructions.

## <a id="priv-sec"></a> Privacy and Security Considerations

This specification introduces no new privacy or security considerations.

## <a id="changes"></a> Changes

Changes since the [14 March 2013 Working Draft](https://www.w3.org/TR/2013/WD-css3-page-20130314/) are:

- <a id="ref-for-descdef-page-bleed①"></a>

  <a id="ref-for-descdef-page-marks②"></a>

  Imported the [marks](#descdef-page-marks) and [bleed](#descdef-page-bleed) properties from [\[CSS3GCPM\]](#biblio-css3gcpm).

- <a id="ref-for-valdef-page-marks-none"></a>

  <a id="ref-for-descdef-page-marks③"></a>

  <a id="ref-for-descdef-page-bleed②"></a>

  <a id="ref-for-valdef-page-bleed-auto"></a>

  Added [auto](#valdef-page-bleed-auto) as the initial value of [bleed](#descdef-page-bleed) and allowed it to apply even when [marks](#descdef-page-marks) is [none](#valdef-page-marks-none).

- Added JIS-B4 and JIS-B5.

## <a id="acknowledgements"></a>Acknowledgements

The CSS Working Group would like to give very special thanks to this module’s former editors: Robert Stevahn (Hewlett-Packard), Håkon Wium Lie (Opera Software), Jim Bigelow (Hewlett-Packard), Jacob Refstrup (Hewlett-Packard), and Melinda Grant (Hewlett-Packard).

We would also like to acknowledge our expert contributors Michael Day (YesLogic), Shinyu Murakami (Antenna House), Peter Linss (Hewlett-Packard), and the other members of the CSS Working Group and www-style community who have provided review and comment on CSS Paged Media Level 3.

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

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [A3](#valdef-page-size-a3), in §7.1
- [A4](#valdef-page-size-a4), in §7.1
- [A5](#valdef-page-size-a5), in §7.1
- auto
  - [value for @page/bleed](#valdef-page-bleed-auto), in §7.3
  - [value for @page/size](#valdef-page-size-auto), in §7.1
- [available height](#available-height), in §5.3.1
- [available width](#available-width), in §5.3.1
- [B4](#valdef-page-size-b4), in §7.1
- [B5](#valdef-page-size-b5), in §7.1
- [Binding Edge](#binding-edge), in §2
- [:blank](#valdef-page-blank), in §4.2.3
- [bleed](#descdef-page-bleed), in §7.3
- [bleed area](#bleed-area), in §7.3
- [@bottom-center](#at-ruledef-bottom-center), in §4.3
- [@bottom-left](#at-ruledef-bottom-left), in §4.3
- [@bottom-left-corner](#at-ruledef-bottom-left-corner), in §4.3
- [@bottom-right](#at-ruledef-bottom-right), in §4.3
- [@bottom-right-corner](#at-ruledef-bottom-right-corner), in §4.3
- [containing block](#containing-block), in §5.3.1
- [Content-empty page](#content-empty), in §3.2
- [crop](#valdef-page-marks-crop), in §7.2
- [cross](#valdef-page-marks-cross), in §7.2
- [Duplex Printing](#duplex-printing), in §2
- [end page value](#end-page-value), in §8.1
- [Facing Pages](#facing-pages), in §2
- [:first](#valdef-page-first), in §4.2.2
- [flex factor](#flex-factor), in §5.3.2.2
- [flex space](#flex-space), in §5.3.2.2
- [generated](#generated), in §5.2
- [JIS-B4](#valdef-page-size-jis-b4), in §7.1
- [JIS-B5](#valdef-page-size-jis-b5), in §7.1
- landscape
  - [definition of](#landscape), in §2
  - [value for @page/size](#valdef-page-size-landscape), in §7.1
- [landscape orientation](#landscape), in §2
- [ledger](#valdef-page-size-ledger), in §7.1
- [:left](#valdef-page-left), in §4.2.1
- [@left-bottom](#at-ruledef-left-bottom), in §4.3
- [@left-middle](#at-ruledef-left-middle), in §4.3
- [Left Page](#left-page), in §2
- [@left-top](#at-ruledef-left-top), in §4.3
- [legal](#valdef-page-size-legal), in §7.1
- \<length\>
  - [value for @page/bleed](#valdef-page-bleed-length), in §7.3
  - [value for @page/size](#valdef-page-size-length), in §7.1
- [letter](#valdef-page-size-letter), in §7.1
- [margin at-rule](#margin-at-rule), in §5.1
- [margin context](#margin-context), in §5.1
- [marks](#descdef-page-marks), in §7.2
- [match](#match), in §4.2
- [max-content width](#max-content-width), in §5.3.1
- [min-content width](#min-content-width), in §5.3.1
- [named page](#named-page), in §8.1
- [none](#valdef-page-marks-none), in §7.2
- [outer max width](#outer-max-width), in §5.3.1
- [outer min width](#outer-min-width), in §5.3.1
- [outer width](#outer-width), in §5.3.1
- [page](#propdef-page), in §8.1
- [@page](#at-ruledef-page), in §4.1
- [page area](#page-area), in §3
- [page box](#page-box), in §3
- [page context](#page-context), in §4.1
- [page footer](#page-footer), in §5
- [page header](#page-header), in §5
- [page-margin boxes](#page-margin-boxes), in §3
- [page-margin property](#page-margin-property), in §Unnumbered section
- [Page Orientation](#page-orientation), in §2
- [page progression](#page-progression), in §3.3
- [page property](#page-property), in §Unnumbered section
- [page pseudo-class](#page-pseudo-class), in §4.2
- [\<page-selector\>](#typedef-page-selector), in §4.3
- [page selector](#page-selector), in §4.2
- [\<page-selector-list\>](#typedef-page-selector-list), in §4.3
- [Page sheet](#page-sheet), in §2
- [\<page-size\>](#typedef-page-size-page-size), in §7.1
- [page type selector](#page-type-selector), in §4.2
- portrait
  - [definition of](#portrait), in §2
  - [value for @page/size](#valdef-page-size-portrait), in §7.1
- [portrait orientation](#portrait), in §2
- [printable area](#printable-area), in §2
- [\<pseudo-page\>](#typedef-pseudo-page), in §4.3
- [:right](#valdef-page-right), in §4.2.1
- [@right-bottom](#at-ruledef-right-bottom), in §4.3
- [@right-middle](#at-ruledef-right-middle), in §4.3
- [Right Page](#right-page), in §2
- [@right-top](#at-ruledef-right-top), in §4.3
- [signature](#signature), in §Unnumbered section
- [size](#descdef-page-size), in §7.1
- [specificity](#specificity), in §4.4
- [start page value](#start-page-value), in §8.1
- [@top-center](#at-ruledef-top-center), in §4.3
- [@top-left](#at-ruledef-top-left), in §4.3
- [@top-left-corner](#at-ruledef-top-left-corner), in §4.3
- [@top-right](#at-ruledef-top-right), in §4.3
- [@top-right-corner](#at-ruledef-top-right-corner), in §4.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BOX-3\] defines the following terms:
  - <a id="term-for-propdef-margin-bottom"></a>margin-bottom
  - <a id="term-for-propdef-margin-left"></a>margin-left
  - <a id="term-for-propdef-margin-right"></a>margin-right
  - <a id="term-for-propdef-margin-top"></a>margin-top
  - <a id="term-for-outer-edge"></a>outer edge
  - <a id="term-for-propdef-padding-bottom"></a>padding-bottom
  - <a id="term-for-propdef-padding-top"></a>padding-top
- \[css-cascade-4\] defines the following terms:
  - <a id="term-for-valdef-all-inherit"></a>inherit
- \[css-content-3\] defines the following terms:
  - <a id="term-for-propdef-content"></a>content
  - <a id="term-for-valdef-content-none"></a>none
  - <a id="term-for-valdef-content-normal"></a>normal
- \[CSS-DEVICE-ADAPT\] defines the following terms:
  - <a id="term-for-at-ruledef-viewport"></a>@viewport
- \[css-display-3\] defines the following terms:
  - <a id="term-for-valdef-display-none"></a>none
- \[css-fonts-3\] defines the following terms:
  - <a id="term-for-propdef-font-size"></a>font-size
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-propdef-vertical-align"></a>vertical-align
- \[css-position-3\] defines the following terms:
  - <a id="term-for-propdef-position"></a>position
  - <a id="term-for-stacking-context"></a>stacking context
  - <a id="term-for-propdef-z-index"></a>z-index
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-typedef-declaration-list"></a>\<declaration-list\>
  - <a id="term-for-typedef-ident-token"></a>\<ident-token\>
  - <a id="term-for-css-at-rule"></a>at-rule
  - <a id="term-for-css-parse-something-according-to-a-css-grammar"></a>parse
- \[css-text-3\] defines the following terms:
  - <a id="term-for-valdef-white-space-pre"></a>pre
  - <a id="term-for-propdef-text-align"></a>text-align
  - <a id="term-for-propdef-white-space"></a>white-space
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-req"></a>!
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-mult-zero-plus"></a>\*
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-em"></a>em
  - <a id="term-for-ex"></a>ex
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-flow-direction"></a>block flow direction
  - <a id="term-for-inline-base-direction"></a>inline base direction
  - <a id="term-for-principal-writing-mode"></a>principal writing mode
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="term-for-propdef-counter-increment"></a>counter-increment
  - <a id="term-for-propdef-counter-reset"></a>counter-reset
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-margin"></a>margin
  - <a id="term-for-propdef-max-height"></a>max-height
  - <a id="term-for-propdef-max-width"></a>max-width
  - <a id="term-for-propdef-min-height"></a>min-height
  - <a id="term-for-propdef-min-width"></a>min-width
  - <a id="term-for-propdef-page-break-after"></a>page-break-after
  - <a id="term-for-propdef-page-break-before"></a>page-break-before
  - <a id="term-for-propdef-visibility"></a>visibility
  - <a id="term-for-propdef-width"></a>width
- \[CSS3-BREAK\] defines the following terms:
  - <a id="term-for-propdef-break-after"></a>break-after
  - <a id="term-for-propdef-break-before"></a>break-before
  - <a id="term-for-forced-break"></a>forced break
  - <a id="term-for-valdef-break-before-left"></a>left
  - <a id="term-for-valdef-break-before-recto"></a>recto
  - <a id="term-for-valdef-break-before-right"></a>right
  - <a id="term-for-valdef-break-before-verso"></a>verso
- \[CSS3-SIZING\] defines the following terms:
  - <a id="term-for-valdef-width-auto"></a>auto
  - <a id="term-for-max-content-block-size"></a>max-content block size
  - <a id="term-for-max-content-inline-size"></a>max-content inline size
  - <a id="term-for-min-content-block-size"></a>min-content block size
  - <a id="term-for-min-content-inline-size"></a>min-content inline size
- \[CSS3-WRITING-MODES\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
- \[CSS3BG\] defines the following terms:
  - <a id="term-for-background-painting-area"></a>background painting area
  - <a id="term-for-background-positioning-area"></a>background positioning area
  - <a id="term-for-propdef-background-attachment"></a>background-attachment
  - <a id="term-for-propdef-background-clip"></a>background-clip
  - <a id="term-for-propdef-background-origin"></a>background-origin
  - <a id="term-for-propdef-border-bottom-width"></a>border-bottom-width
  - <a id="term-for-propdef-border-top-width"></a>border-top-width
  - <a id="term-for-valdef-background-attachment-fixed"></a>fixed
- \[CSS3VAL\] defines the following terms:
  - <a id="term-for-length-value"></a>\<length\>
- \[selectors-4\] defines the following terms:
  - <a id="term-for-typedef-compound-selector"></a>\<compound-selector\>

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 9 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 June 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-device-adapt"></a>\[CSS-DEVICE-ADAPT\]  
Rune Lillesveen; Florian Rivoal; Matt Rakow. [CSS Device Adaptation Module Level 1](https://www.w3.org/TR/css-device-adapt-1/). 29 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-device-adapt-1&#x2F;](https://www.w3.org/TR/css-device-adapt-1/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 8 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Rossen Atanassov; Arron Eicholz. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 17 May 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 20 February 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 20 September 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 10 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-break"></a>\[CSS3-BREAK\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 9 February 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css3-sizing"></a>\[CSS3-SIZING\]  
Tab Atkins Jr.; Elika Etemad. [CSS Intrinsic &#x26; Extrinsic Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 4 March 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css3-writing-modes"></a>\[CSS3-WRITING-MODES\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 17 October 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3val"></a>\[CSS3VAL\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 14 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 5 September 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-pwgmsn"></a>\[PWGMSN\]  
Ron Bergman; Tom Hastings. [Media Standardized Names.](ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf) 26 February 2002. IEEE ISTO Printer Working Group 5101.1-2002. URL: <ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf>

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 11 September 2018. PR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 2 February 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

### <a id="informative"></a>Informative References

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Elika Etemad; Tab Atkins Jr.. [CSS Image Values and Replaced Content Module Level 3](https://www.w3.org/TR/css3-images/). 17 April 2012. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-images&#x2F;](https://www.w3.org/TR/css3-images/)

<a id="biblio-css3gcpm"></a>\[CSS3GCPM\]  
Dave Cramer. [CSS Generated Content for Paged Media Module](https://www.w3.org/TR/css-gcpm-3/). 13 May 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-gcpm-3&#x2F;](https://www.w3.org/TR/css-gcpm-3/)

<a id="biblio-jlreq"></a>\[JLREQ\]  
Yasuhiro Anan; et al. [Requirements for Japanese Text Layout](https://www.w3.org/TR/jlreq/). 3 April 2012. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;jlreq&#x2F;](https://www.w3.org/TR/jlreq/)

## <a id="property-index"></a>Property Index



| Name                | Value                    | Initial | Applies to                             | Inh.               | %ages | Anim­ation type | Canonical order | Com­puted value  |
|---------------------|--------------------------|---------|----------------------------------------|--------------------|-------|----------------|-----------------|-----------------|
| <strong><span><a id="ref-for-propdef-page①④"></a></span><a href="#propdef-page">page</a>&#xA;      </strong> | auto \| \<custom-ident\> | auto    | boxes that create class A break points | no (but see prose) | n/a   | discrete       | per grammar     | specified value |



<a id="ref-for-at-ruledef-page①⑦"></a>

### <a id="page-descriptor-table"></a>[@page](#at-ruledef-page) Descriptors



| Name                | Value                                                                           | Initial | Computed value                                   |
|---------------------|---------------------------------------------------------------------------------|---------|--------------------------------------------------|
| <strong><span><a id="ref-for-descdef-page-bleed③"></a></span><a href="#descdef-page-bleed">bleed</a>&#xA;      </strong> | auto \| \<length\>                                                              | auto    | as specified                                     |
| <strong><span><a id="ref-for-descdef-page-marks④"></a></span><a href="#descdef-page-marks">marks</a>&#xA;      </strong> | none \| \[ crop \|\| cross \]                                                   | none    | as specified                                     |
| <strong><span><a id="ref-for-descdef-page-size⑧"></a></span><a href="#descdef-page-size">size</a>&#xA;      </strong> | \<length\>{1,2} \| auto \| \[ \<page-size\> \|\| \[ portrait \| landscape \] \] | auto    | specified value, with \<length\>s made absolute. |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> In CSS 2.1, both the page box and page area are simple rectangles. Neither is a CSS box with margins, borders, and padding. This CSS box should be distinct from the page box and page area, which would be its margin area and content area, respectively. Naming ideas? [↵](#issue-16b9253a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> It would be useful if media queries could respond at least to sizes specified on an unqualified @page.
>
> Another option could be to do like [@viewport](https://www.w3.org/TR/css-device-adapt/#at-ruledef-viewport) rules [\[CSS-DEVICE-ADAPT\]](#biblio-css-device-adapt): First apply [@page](#at-ruledef-page) rules (matching which selectors?), using the UA’s default page size for Media Queries and [viewport-percentage lengths](https://www.w3.org/TR/css3-values/#viewport-relative-lengths) [\[CSS3VAL\]](#biblio-css3val). The resulting page size is the "base page size". The entire set of stylesheets is applied again, this time using the "base page size" for Media Queries and viewport-percentage lengths.
>
> [↵](#issue-e030ddc1)
