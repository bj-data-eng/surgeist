Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Paged media](https://www.w3.org/TR/2011/REC-CSS2-20110607/page.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Paged media

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/page.html

Snapshot SHA-256: 984664616bba69b3ef8a2fbbfb8da02fc67f250349fab495d39d5d1ddbbe7a2f

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="the-page"></a>

# 13 Paged media

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="page-intro"></a>

## 13.1 Introduction to paged media

Paged media (e.g., paper, transparencies, pages that are displayed on computer screens, etc.) differ from [continuous media](css2--media.html--3a324a170379.md#continuous-media-group) in that the content of the document is split into one or more discrete pages. To handle pages, CSS 2.1 describes how page margins are set on [page boxes](#page-box), and how [page breaks](#page-breaks) are declared.

<a id="x0"></a>

The user agent is responsible for transferring the page boxes of a document onto the real sheets where the document will ultimately be rendered (paper, transparency, screen, etc.). There is often a 1-to-1 relationship between a page box and a sheet, but this is not always the case. Transfer possibilities include:

- Transferring one page box to one sheet (e.g., single-sided printing).
- Transferring two page boxes to both sides of the same sheet (e.g., double-sided printing).
- Transferring N (small) page boxes to one sheet (called "n-up").
- Transferring one (large) page box to N x M sheets (called "tiling").
- Creating signatures. A signature is a group of pages printed on a sheet, which, when folded and trimmed like a book, appear in their proper sequence.
- Printing one document to several output trays.
- Outputting to a file.

<a id="page-box"></a>

## 13.2 Page boxes: the @page rule

<a id="x1"></a>

The page box is a rectangular region that contains two areas:

- <a id="page-area"></a>

  The page area. The page area includes the boxes laid out on that page. The edges of the first page area establish the rectangle that is the initial [containing block](css2--visudet.html--12e8bc0e6b7c.md#containing-block-details) of the document. The canvas background is painted within and covers the page area.

- The margin area, which surrounds the page area. The page margin area is transparent.

The size of a page box cannot be specified in CSS 2.1.

<a id="x3"></a>

<a id="page-context"></a>

Authors can specify the margins of a page box inside an @page rule. An @page rule consists of the keyword "@page", followed by an optional page selector, followed by a block containing declarations and at-rules. Comments and white space are allowed, but optional, between the @page token and the page selector and between the page selector and the block. The declarations in an @page rule are said to be in the page context.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS level 2 has no at-rules that may appear inside @page, but such at-rules are expected to be defined in level 3.

<a id="x5"></a>

The page selector specifies for which pages the declarations apply. In CSS 2.1, page selectors may designate the first page, all left pages, or all right pages

The rules for handling malformed declarations, malformed statements, and invalid at-rules inside @page are as defined in [section 4.2,](css2--syndata.html--02e71c159e14.md#parsing-errors) with the following addition: when the UA expects the start of a declaration or at-rule (i.e., an IDENT token or an ATKEYWORD token) but finds an unexpected token instead, that token is considered to be the first token of a malformed declaration. I.e., the rule for malformed declarations, rather than malformed statements is used to determine which tokens to ignore in that case.

<a id="page-margins"></a>

### 13.2.1 Page margins

In CSS 2.1, only the [margin properties](css2--box.html--8875bbcdefe6.md#margin-properties) (['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right), ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom), ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left), and ['margin'](css2--box.html--8875bbcdefe6.md#propdef-margin)) apply within the [page context](#page-context). The following diagram shows the relationships between the sheet, page box, and page margins:

<a id="img-page-info"></a>

![Illustration of sheet, page box, margin, and page area.](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/page-info.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/page-info-desc.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here is a simple example which sets all page margins on all pages:
>
> ```text
> 
> @page {
>   margin: 3cm;
> }
> ```
The [page context](#page-context) has no notion of fonts, so 'em' and 'ex' units are not allowed. Percentage values on the margin properties are relative to the dimensions of the [page box](#page-box); for left and right margins, they refer to the width of the page box while for top and bottom margins, they refer to the height of the page box. All other units associated with the respective CSS 2.1 properties are allowed.

Due to negative margin values (either on the page box or on elements) or [absolute positioning](css2--visuren.html--3f334c530cf4.md#absolute-positioning) content may end up outside the page box, but this content may be "cut" — by the user agent, the printer, or ultimately, the paper cutter.

<a id="page-selectors"></a>

### 13.2.2 Page selectors: selecting left, right, and first pages

When printing double-sided documents, the [page boxes](#page-box) on left and right pages may be different. This can be expressed through two CSS pseudo-classes that may be used in page selectors.

<a id="x6"></a>

<a id="x8"></a>

All pages are automatically classified by user agents into either the :left or :right pseudo-class. Whether the first page of a document is :left or :right depends on the major writing direction of the root element. For example, the first page of a document with a left-to-right major writing direction would be a :right page, and the first page of a document with a right-to-left major writing direction would be a :left page. To explicitly force a document to begin printing on a left or right page, authors can [insert a page break](#page-break-props) before the first generated box.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> @page :left {
>   margin-left: 4cm;
>   margin-right: 3cm;
> }
> 
> @page :right {
>   margin-left: 3cm;
>   margin-right: 4cm;
> }
> ```
If different declarations have been given for left and right pages, the user agent must honor these declarations even if the user agent does not transfer the page boxes to left and right sheets (e.g., a printer that only prints single-sided).

<a id="x10"></a>

Authors may also specify style for the first page of a document with the :first pseudo-class:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> @page { margin: 2cm } /* All margins set to 2cm */
> 
> @page :first {
>   margin-top: 10cm    /* Top margin on first page 10cm */
> }
> ```
Properties specified in a :left or :right @page rule override those specified in an @page rule that has no pseudo-class specified. Properties specified in a :first @page rule override those specified in :left or :right @page rules.

If a [forced break](#forced) occurs before the first generated box, it is undefined in CSS 2.1 whether ':first' applies to the blank page before the break or to the page after it.

Margin declarations on left, right, and first pages may result in different [page area](#page-area) widths. To simplify implementations, user agents may use a single page area width on left, right, and first pages. In this case, the page area width of the first page should be used.

<a id="outside-page-box"></a>

### 13.2.3 Content outside the page box

When formatting content in the page model, some content may end up outside the current page box. For example, an element whose ['white-space'](css2--text.html--467a8857ae69.md#propdef-white-space) property has the value 'pre' may generate a box that is wider than the page box. As another example, when boxes are positioned absolutely or relatively, they may end up in "inconvenient" locations. For example, images may be placed on the edge of the page box or 100,000 meters below the page box.

The exact formatting of such elements lies outside the scope of this specification. However, we recommend that authors and user agents observe the following general principles concerning content outside the page box:

- Content should be allowed slightly beyond the page box to allow pages to "bleed".
- User agents should avoid generating a large number of empty page boxes to honor the positioning of elements (e.g., you do not want to print 100 blank pages).
- Authors should not position elements in inconvenient locations just to avoid rendering them.
- User agents may handle boxes positioned outside the page box in several ways, including discarding them or creating page boxes for them at the end of the document.

<a id="page-breaks"></a>

## 13.3 Page breaks

This section describes page breaks in CSS 2.1. Five properties indicate where the user agent may or should break pages, and on what page (left or right) the subsequent content should resume. Each page break ends layout in the current [page box](#page-box) and causes remaining pieces of the [document tree](css2--conform.html--de58593b67d7.md#doctree) to be laid out in a new page box.

<a id="page-break-props"></a>

### 13.3.1 Page break properties: ['page-break-before'](css2--page.html--984664616bba.md#propdef-page-break-before), ['page-break-after'](css2--page.html--984664616bba.md#propdef-page-break-after), ['page-break-inside'](css2--page.html--984664616bba.md#propdef-page-break-inside)

<a id="propdef-page-break-before"></a>

<strong>'page-break-before'</strong>

|                       |                                                                                                                                                                          |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | auto \| always \| avoid \| left \| right \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit)                                       |
| <em>Initial:</em>   | auto                                                                                                                                                                     |
| <em>Applies to:</em>   | block-level elements (but see text)                                                                                                                                      |
| <em>Inherited:</em>   | no                                                                                                                                                                       |
| <em>Percentages:</em>   | N/A                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [paged](css2--media.html--3a324a170379.md#paged-media-group) |
| <em>Computed value:</em>   | as specified                                                                                                                                                             |

<a id="propdef-page-break-after"></a>

<strong>'page-break-after'</strong>

|                       |                                                                                                                                                                          |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | auto \| always \| avoid \| left \| right \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit)                                       |
| <em>Initial:</em>   | auto                                                                                                                                                                     |
| <em>Applies to:</em>   | block-level elements (but see text)                                                                                                                                      |
| <em>Inherited:</em>   | no                                                                                                                                                                       |
| <em>Percentages:</em>   | N/A                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [paged](css2--media.html--3a324a170379.md#paged-media-group) |
| <em>Computed value:</em>   | as specified                                                                                                                                                             |

<a id="propdef-page-break-inside"></a>

<strong>'page-break-inside'</strong>

|                       |                                                                                                                                                                          |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | avoid \| auto \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit)                                                                  |
| <em>Initial:</em>   | auto                                                                                                                                                                     |
| <em>Applies to:</em>   | block-level elements (but see text)                                                                                                                                      |
| <em>Inherited:</em>   | no                                                                                                                                                                       |
| <em>Percentages:</em>   | N/A                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [paged](css2--media.html--3a324a170379.md#paged-media-group) |
| <em>Computed value:</em>   | as specified                                                                                                                                                             |

Values for these properties have the following meanings:

<strong>auto</strong>  
Neither force nor forbid a page break before (after, inside) the generated box.

<strong>always</strong>  
Always force a page break before (after) the generated box.

<strong>avoid</strong>  
Avoid a page break before (after, inside) the generated box.

<strong>left</strong>  
Force one or two page breaks before (after) the generated box so that the next page is formatted as a left page.

<strong>right</strong>  
Force one or two page breaks before (after) the generated box so that the next page is formatted as a right page.

A conforming user agent may interpret the values 'left' and 'right' as 'always'.

A potential page break location is typically under the influence of the parent element's ['page-break-inside'](css2--page.html--984664616bba.md#propdef-page-break-inside) property, the ['page-break-after'](css2--page.html--984664616bba.md#propdef-page-break-after) property of the preceding element, and the ['page-break-before'](css2--page.html--984664616bba.md#propdef-page-break-before) property of the following element. When these properties have values other than 'auto', the values 'always', 'left', and 'right' take precedence over 'avoid'.

User Agents must apply these properties to block-level elements in the normal flow of the root element. User agents may also apply these properties to other elements, e.g., 'table-row' elements.

When a page break splits a box, the box's margins, borders, and padding have no visual effect where the split occurs.

<a id="break-inside"></a>

### 13.3.2 Breaks inside elements: ['orphans'](css2--page.html--984664616bba.md#propdef-orphans), ['widows'](css2--page.html--984664616bba.md#propdef-widows)

<a id="propdef-orphans"></a>

<strong>'orphans'</strong>

|                       |                                                                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<integer\>](css2--syndata.html--02e71c159e14.md#value-def-integer) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 2                                                                                                                                                                                    |
| <em>Applies to:</em>   | block container elements                                                                                                                                                             |
| <em>Inherited:</em>   | yes                                                                                                                                                                                  |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                  |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [paged](css2--media.html--3a324a170379.md#paged-media-group)             |
| <em>Computed value:</em>   | as specified                                                                                                                                                                         |

<a id="propdef-widows"></a>

<strong>'widows'</strong>

|                       |                                                                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<integer\>](css2--syndata.html--02e71c159e14.md#value-def-integer) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 2                                                                                                                                                                                    |
| <em>Applies to:</em>   | block container elements                                                                                                                                                             |
| <em>Inherited:</em>   | yes                                                                                                                                                                                  |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                  |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [paged](css2--media.html--3a324a170379.md#paged-media-group)             |
| <em>Computed value:</em>   | as specified                                                                                                                                                                         |

The ['orphans'](css2--page.html--984664616bba.md#propdef-orphans) property specifies the minimum number of lines in a block container that must be left at the bottom of a page. The ['widows'](css2--page.html--984664616bba.md#propdef-widows) property specifies the minimum number of lines in a block container that must be left at the top of a page. Examples of how they are used to control page breaks are given below.

Only positive values are allowed.

For information about paragraph formatting, please consult the section on [line boxes](css2--visuren.html--3f334c530cf4.md#line-box).

<a id="allowed-page-breaks"></a>

### 13.3.3 Allowed page breaks

In the normal flow, page breaks can occur at the following places:

1.  In the vertical margin between block-level boxes. When an unforced page break occurs here, the [used values](css2--cascade.html--c7aff33e6f0d.md#used-value) of the relevant ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top) and ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) properties are set to '0'. When a forced page break occurs here, the used value of the relevant ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) property is set to '0'; the relevant ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top) used value may either be set to '0' or retained.
2.  Between [line boxes](css2--visuren.html--3f334c530cf4.md#line-box) inside a [block container](css2--visuren.html--3f334c530cf4.md#block-boxes) box.
3.  Between the content edge of a block container box and the outer edges of its child content (margin edges of block-level children or line box edges for inline-level children) if there is a (non-zero) gap between them.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that CSS3 will specify that the relevant 'margin-top' applies (i.e., is not set to '0') after a forced page break.

These breaks are subject to the following rules:

- <strong>Rule A:</strong> Breaking at (1) is allowed only if the ['page-break-after'](css2--page.html--984664616bba.md#propdef-page-break-after) and ['page-break-before'](css2--page.html--984664616bba.md#propdef-page-break-before) properties of all the elements generating boxes that meet at this margin allow it, which is when at least one of them has the value 'always', 'left', or 'right', or when all of them are 'auto'.
- <strong>Rule B:</strong> However, if all of them are 'auto' and a common ancestor of all the elements has a ['page-break-inside'](css2--page.html--984664616bba.md#propdef-page-break-inside) value of 'avoid', then breaking here is not allowed.
- <strong>Rule C:</strong> Breaking at (2) is allowed only if the number of [line boxes](css2--visuren.html--3f334c530cf4.md#line-box) between the break and the start of the enclosing block box is the value of ['orphans'](css2--page.html--984664616bba.md#propdef-orphans) or more, and the number of line boxes between the break and the end of the box is the value of ['widows'](css2--page.html--984664616bba.md#propdef-widows) or more.
- <strong>Rule D:</strong> In addition, breaking at (2) or (3) is allowed only if the ['page-break-inside'](css2--page.html--984664616bba.md#propdef-page-break-inside) property of the element and all its ancestors is 'auto'.

If the above does not provide enough break points to keep content from overflowing the page boxes, then rules A, B and D are dropped in order to find additional breakpoints.

If that still does not lead to sufficient break points, rule C is dropped as well, to find still more break points.

<a id="forced"></a>

### 13.3.4 Forced page breaks

A page break <em>must</em> occur at (1) if, among the ['page-break-after'](css2--page.html--984664616bba.md#propdef-page-break-after) and ['page-break-before'](css2--page.html--984664616bba.md#propdef-page-break-before) properties of all the elements generating boxes that meet at this margin, there is at least one with the value 'always', 'left', or 'right'.

<a id="best-page-breaks"></a>

### 13.3.5 "Best" page breaks

CSS 2.1 does <em>not</em> define which of a set of allowed page breaks must be used; CSS 2.1 does not forbid a user agent from breaking at every possible break point, or not to break at all. But CSS 2.1 does recommend that user agents observe the following heuristics (while recognizing that they are sometimes contradictory):

- Break as few times as possible.
- Make all pages that do not end with a forced break appear to have about the same height.
- Avoid breaking inside a replaced element.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Suppose, for example, that the style sheet contains 'orphans: 4', 'widows: 2', and there are 20 lines ([line boxes](css2--visuren.html--3f334c530cf4.md#line-box)) available at the bottom of the current page:
>
> - If a paragraph at the end of the current page contains 20 lines or fewer, it should be placed on the current page.
> - If the paragraph contains 21 or 22 lines, the second part of the paragraph must not violate the ['widows'](css2--page.html--984664616bba.md#propdef-widows) constraint, and so the second part must contain exactly two lines
> - If the paragraph contains 23 lines or more, the first part should contain 20 lines and the second part the remaining lines.
>
> Now suppose that ['orphans'](css2--page.html--984664616bba.md#propdef-orphans) is '10', ['widows'](css2--page.html--984664616bba.md#propdef-widows) is '20', and there are 8 lines available at the bottom of the current page:
>
> - If a paragraph at the end of the current page contains 8 lines or fewer, it should be placed on the current page.
> - If the paragraph contains 9 lines or more, it cannot be split (that would violate the orphan constraint), so it should move as a block to the next page.

<a id="page-cascade"></a>

## 13.4 Cascading in the page context

Declarations in the [page context](#page-context) obey the [cascade](css2--cascade.html--c7aff33e6f0d.md) just like normal CSS declarations.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Consider the following example:
>
> ```text
> 
> @page {
>   margin-left: 3cm;
> }
> 
> @page :left {
>   margin-left: 4cm;
> }
> ```
>
> Due to the [higher specificity](css2--cascade.html--c7aff33e6f0d.md#cascading-order) of the pseudo-class selector, the left margin on left pages will be '4cm' and all other pages (i.e., the right pages) will have a left margin of '3cm'.
