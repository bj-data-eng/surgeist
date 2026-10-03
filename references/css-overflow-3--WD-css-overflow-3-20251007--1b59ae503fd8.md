Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Overflow Module Level 3](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Overflow Module Level 3

Source snapshot: https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/

Snapshot SHA-256: 1b59ae503fd88cdaeb07ae2a1c6b5ff344baead236ce4e1b55425f880eed85fb

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 10 source tables are presented as readable Markdown tables or explicit labeled layouts: 7 ordinary table conversions, 3 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Live HTML/CSS demonstrations are represented by static source code and text, not equivalent browser appearance. Incidental whitespace in sample-display elements may collapse as in HTML; exact source markup is retained, and true preformatted/code blocks stay literal.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Overflow Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-propdef-text-overflow"></a>

<a id="ref-for-propdef-overflow-clip-margin"></a>

This module contains the features of CSS relating to scrollable overflow handling in visual media. This level is focused on completing a precise specification for the existing overflow features, including the [overflow](#propdef-overflow) property and its longhands; and the [text-overflow](#propdef-text-overflow) property. A few additional features introduced in support of [\[CSS-CONTAIN-1\]](#biblio-css-contain-1) and [\[CSS-CONTAIN-2\]](#biblio-css-contain-2) are also defined: <a id="ref-for-propdef-overflow①"></a>overflow: clip and the [overflow-clip-margin](#propdef-overflow-clip-margin) property.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-overflow” in the title, like this: “\[css-overflow\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-overflow%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<a id="ref-for-propdef-overflow②"></a>

<a id="ref-for-propdef-overflow-clip-margin①"></a>

<a id="ref-for-propdef-text-overflow①"></a>

<a id="ref-for-propdef-line-clamp"></a>

The description of [overflow](#propdef-overflow) and its longhands is considered significantly more complete and correct than previous working drafts or than [\[CSS2\]](#biblio-css2), but a few questions and issues remain open. <a id="ref-for-propdef-overflow③"></a>overflow: clip and [overflow-clip-margin](#propdef-overflow-clip-margin) are rather new, and lack implementation experience. [text-overflow](#propdef-text-overflow) is stable, is unchanged form its earlier definition in [\[CSS-UI-3\]](#biblio-css-ui-3). While not yet fully validated by implementation experience, the design of [line-clamp](https://www.w3.org/TR/css-overflow-4/#propdef-line-clamp) and its longhands is considered roughly complete. Earlier versions of this specification included experimental new ideas for handling overflow by creating new boxes using fragmentation. These ideas are not abandoned; they are merely [deferred until Level 4](https://drafts.csswg.org/css-overflow-4/). Work will resume on fragmented overflow once this level stabilizes completed.

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-max-lines"></a>

  the [max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines) property

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

In CSS Level 1 [\[CSS1\]](#biblio-css1), placing more content than would fit inside an element with a specified size was generally an authoring error. Doing so caused the content to extend outside the bounds of the element, which would likely cause that content to overlap with other elements.

<a id="ref-for-propdef-overflow④"></a>

CSS Level 2 [\[CSS2\]](#biblio-css2) introduced the [overflow](#propdef-overflow) property, which allows authors to have overflow be handled by scrolling, which means it is no longer an authoring error. It also allows authors to specify that overflow is handled by clipping, which makes sense when the author’s intent is that the content not be shown.

<a id="ref-for-propdef-overflow-x"></a>

<a id="ref-for-propdef-overflow-y"></a>

<a id="ref-for-valdef-overflow-clip"></a>

This specification introduces the long-standing de-facto [overflow-x](#propdef-overflow-x) and [overflow-y](#propdef-overflow-y) properties, adds a [clip](#valdef-overflow-clip) value, and defines overflow handling more fully.

<a id="ref-for-propdef-max-lines①"></a>

\[Something something [max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines).\]

<a id="ref-for-propdef-text-overflow②"></a>

<a id="ref-for-propdef-block-ellipsis"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification also reproduces the definition of the [text-overflow](#propdef-text-overflow) property previously defined in [\[CSS-UI-3\]](#biblio-css-ui-3), with no addition or modification, in order to present <a id="ref-for-propdef-text-overflow③"></a>text-overflow and [block-ellipsis](https://www.w3.org/TR/css-overflow-4/#propdef-block-ellipsis) together.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="placement"></a>1.2.  Module Interactions

This module replaces (supersedes) and extends features defined in [\[CSS2\]](#biblio-css2) section [11.1 Overflow and clipping](https://www.w3.org/TR/CSS2/visufx.html#overflow-clipping) and [\[CSS-UI-3\]](#biblio-css-ui-3) section [5.2. Overflow Ellipsis: the text-overflow property](https://www.w3.org/TR/css-ui-3/#text-overflow).

## <a id="overflow-concepts"></a>2.  Overflow Concepts and Terminology

<a id="ref-for-containing-block-chain"></a>

CSS uses the term <a id="overflow"></a>overflow to describe the contents of a box that extend outside one of that box’s edges (i.e., its <i>content edge</i>, <i>padding edge</i>, <i>border edge</i>, or <i>margin edge</i>). The term might be interpreted as elements or features that cause this overflow, the non-rectangular region occupied by these features, or, more commonly, as the minimal rectangle that bounds that region. A box’s overflow is computed based on the layout and styling of the box itself and of all descendants whose [containing block chain](https://www.w3.org/TR/css-display-4/#containing-block-chain) includes the box.

<a id="ref-for-overflow"></a>

<a id="ref-for-propdef-transform-style"></a>

In most cases, [overflow](#overflow) can be computed for any box from the bounds and properties of that box itself, plus the <a id="ref-for-overflow①"></a>overflow of each of its children. However, this is not always the case; for example, when [transform-style: preserve-3d](https://www.w3.org/TR/css-transforms-2/#propdef-transform-style) [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms) is used on some of the children, any of their descendants with <a id="ref-for-propdef-transform-style①"></a>transform-style: preserve-3d must also be examined.

There are two different types of overflow, which are used for different purposes by the UA:

- <a id="ref-for-ink-overflow"></a>

  [ink overflow](#ink-overflow)

- <a id="ref-for-scrollable-overflow"></a>

  [scrollable overflow](#scrollable-overflow)

### <a id="ink"></a>2.1. Ink Overflow

<a id="ref-for-scrollable-overflow-region"></a>

The <a id="ink-overflow"></a>ink overflow of a box is the part of that box and its contents that creates a visual effect outside of the box’s border box. Ink overflow is the overflow of painting effects defined to not affect layout or otherwise extend the [scrollable overflow area](#scrollable-overflow-region), such as [box shadows](https://www.w3.org/TR/css-backgrounds/#box-shadow), [border images](), [text decoration](https://www.w3.org/TR/css-text-decor-3/), overhanging glyphs (with negative side bearings, or with ascenders/descenders extending outside the em box), [outlines](https://www.w3.org/TR/css-ui-3/#outline-props), etc.

<a id="ref-for-propdef-text-shadow"></a>

<a id="ref-for-propdef-box-shadow"></a>

<a id="ref-for-ink-overflow①"></a>

Since some effects in CSS (for example, the blurs in [text-shadow](https://www.w3.org/TR/css-text-decor-4/#propdef-text-shadow) [\[CSS-TEXT-3\]](#biblio-css-text-3) and [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) [\[CSS-BACKGROUNDS-3\]](#biblio-css-backgrounds-3), which are theoretically infinite) do not define what visual extent they cover, the extent of the [ink overflow](#ink-overflow) is undefined.

<a id="ref-for-ink-overflow②"></a>

<a id="ref-for-ink-overflow-region"></a>

<a id="ref-for-ink-overflow-rectangle"></a>

The <a id="ink-overflow-region"></a>ink overflow area is the non-rectangular area occupied by the [ink overflow](#ink-overflow) of a box and its contents, and the <a id="ink-overflow-rectangle"></a>ink overflow rectangle is the minimal rectangle whose axes are aligned to the box’s axes and that contains the [ink overflow area](#ink-overflow-region). Note that the [ink overflow rectangle](#ink-overflow-rectangle) is a rectangle in the box’s coordinate system, but might be non-rectangular in other coordinate systems due to transforms. [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms)

<a id="ref-for-replaced-element"></a>

<a id="ref-for-ink-overflow③"></a>

<a id="ref-for-scrollable-overflow①"></a>

Any overflow of [replaced](https://www.w3.org/TR/css-display-4/#replaced-element) content is always [ink overflow](#ink-overflow) (as opposed to [scrollable overflow](#scrollable-overflow)).

### <a id="scrollable"></a>2.2.  Scrollable Overflow

The <a id="scrollable-overflow"></a>scrollable overflow of a box is the set of things extending outside of that box’s padding edge for which a scrolling mechanism needs to be provided.

<a id="ref-for-scrollable-overflow②"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

The <a id="scrollable-overflow-region"></a>scrollable overflow area is the non-rectangular region occupied by the [scrollable overflow](#scrollable-overflow), and the <a id="scrollable-overflow-rectangle"></a>scrollable overflow rectangle is the minimal rectangle whose axes are aligned to the box’s axes and that contains the [scrollable overflow area](#scrollable-overflow-region).

<a id="ref-for-scrollable-overflow-region②"></a>

The [scrollable overflow area](#scrollable-overflow-region) is the union of:

- <a id="ref-for-padding-box"></a>

  <a id="ref-for-scroll-container"></a>

  The [scroll container](#scroll-container)’s own [padding box](https://www.w3.org/TR/css-box-4/#padding-box).

- <a id="ref-for-scroll-container①"></a>

  <a id="ref-for-line-box"></a>

  All [line boxes](https://www.w3.org/TR/css-inline-3/#line-box) directly contained by the [scroll container](#scroll-container).

- <a id="ref-for-3d-rendering-context"></a>

  <a id="ref-for-unreachable-scrollable-overflow-region"></a>

  The border boxes of all boxes for which it is the containing block and whose border boxes are positioned not wholly in the [unreachable scrollable overflow region](#unreachable-scrollable-overflow-region), accounting for transforms by projecting each box onto the plane of the element that establishes its [3D rendering context](https://www.w3.org/TR/css-transforms-2/#3d-rendering-context). [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms)

  > <strong data-conversion-semantic="issue">Issue</strong>
  >
  > <a id="issue-df7ef6c3"></a> Is this description of handling transforms sufficiently accurate?

  <a id="ref-for-scrollable-overflow-region③"></a>

  Border boxes with zero area do not affect the [scrollable overflow area](#scrollable-overflow-region).

- <a id="ref-for-flex-item"></a>

  <a id="ref-for-grid-item"></a>

  The margin areas of [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) and [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item) boxes for which the box establishes a containing block.

  The UA may <em>additionally</em> include the margin areas of other boxes for which the box establishes a containing block; however, the conditions under which such margin areas are included is undefined in this level. <strong data-conversion-semantic="issue">Issue:</strong> <a id="issue-b9c7269c"></a>This needs further testing and investigation; is therefore deferred in this draft.

- <a id="ref-for-propdef-contain"></a>

  <a id="ref-for-propdef-clip"></a>

  <a id="ref-for-scrollable-overflow③"></a>

  <a id="ref-for-propdef-overflow⑤"></a>

  <a id="ref-for-scrollable-overflow-region④"></a>

  The [scrollable overflow areas](#scrollable-overflow-region) of all of the above boxes (including zero-area boxes and accounting for transforms as described above), provided they themselves have [overflow: visible](#propdef-overflow) (i.e. do not themselves trap the overflow) and that [scrollable overflow](#scrollable-overflow) is not already clipped (e.g. by the [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip) property or the [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain) property).

  <a id="ref-for-scrollable-overflow-region⑤"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The mask-\* properties [\[CSS-MASKING-1\]](#biblio-css-masking-1) do not affect the [scrollable overflow area](#scrollable-overflow-region).

  <a id="ref-for-scrollable-overflow④"></a>

  > <strong data-conversion-semantic="issue">Issue</strong>
  >
  > <a id="issue-b127a295"></a> Need to evaluate what should/should not clip [scrollable overflow](#scrollable-overflow). [\[Issue \#8607\]](https://github.com/w3c/csswg-drafts/issues/8607)

- <a id="ref-for-propdef-place-content"></a>

  <a id="ref-for-scrollable-overflow-rectangle"></a>

  Additional padding added to the [scrollable overflow rectangle](#scrollable-overflow-rectangle) as necessary to enable scroll positions that satisfy the requirements of both [place-content: start](https://www.w3.org/TR/css-align-3/#propdef-place-content) and <a id="ref-for-propdef-place-content①"></a>place-content: end alignment.

  <a id="ref-for-scrollable-overflow-rectangle①"></a>

  <a id="ref-for-scroll-container②"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This padding represents, within the [scrollable overflow rectangle](#scrollable-overflow-rectangle), the box’s own padding so that when its content is scrolled to its end, there is padding between the edge of its in-flow (or floated) content and the border edge of the box. It typically ends up being exactly the same size as the box’s own padding, except in a few cases—​such as when an out-of-flow positioned element, or the visible overflow of a descendent, has already increased the size of the <a id="ref-for-scrollable-overflow-rectangle②"></a>scrollable overflow rectangle outside the conceptual “content edge” of the [scroll container](#scroll-container)’s content.

  ![](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/images/scroll-align-padding.jpg)
  Issue: Replace this image with a proper SVG.

<a id="ref-for-unreachable-scrollable-overflow-region①"></a>

Additionally, due to Web-compatibility constraints (caused by authors exploiting legacy bugs to surreptitiously hide content from visual readers but not search engines and/or speech output), UAs must clip any content in the [unreachable scrollable overflow region](#unreachable-scrollable-overflow-region).

<a id="ref-for-content-distribution-properties"></a>

<a id="ref-for-scroll-container③"></a>

<a id="ref-for-alignment-subject"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [content-distribution properties](https://www.w3.org/TR/css-align-3/#content-distribution-properties) can [alter the unreachable scrollable overflow region](https://www.w3.org/TR/css-align-3/#overflow-scroll-position) to ensure that a [scroll container](#scroll-container)’s [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) is reachable after alignment. [\[CSS-ALIGN-3\]](#biblio-css-align-3)

<a id="ref-for-scrollable-overflow-rectangle③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [scrollable overflow rectangle](#scrollable-overflow-rectangle) is always a rectangle in the box’s own coordinate system, but might be non-rectangular in other coordinate systems due to transforms [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms). This means scrollbars can sometimes appear when not actually necessary.

### <a id="scrolling"></a>2.3.  Scrolling Overflow

<a id="ref-for-overflow②"></a>

<a id="ref-for-scrollable-overflow-region⑥"></a>

<a id="ref-for-scroll-container④"></a>

<a id="ref-for-scrollport"></a>

A box’s [overflow](#overflow) can be visible or clipped. CSS also allows a box to be a <a id="scroll-container"></a>scroll container that allows clipped parts of its [scrollable overflow area](#scrollable-overflow-region) to be scrolled into view. The visual “viewport” of a [scroll container](#scroll-container) (through which the <a id="ref-for-scrollable-overflow-region⑦"></a>scrollable overflow area can be viewed) coincides with its padding box, and is called the <a id="scrollport"></a>scrollport. A box’s <a id="nearest-scrollport"></a>nearest scrollport is the [scrollport](#scrollport) of its nearest <a id="ref-for-scroll-container⑤"></a>scroll container ancestor.

<a id="ref-for-dom-element-scrollintoview"></a>

<a id="ref-for-dom-window-focus"></a>

<a id="ref-for-scrollable-overflow-rectangle④"></a>

<a id="ref-for-scrollport①"></a>

<a id="ref-for-initial-scroll-position"></a>

<a id="ref-for-scroll-container⑥"></a>

<a id="ref-for-writing-mode"></a>

<a id="ref-for-scroll-origin-position"></a>

<a id="ref-for-propdef-scroll-initial-target"></a>

Scrolling operations can be initiated by the user (for example, by manipulating a scrollbar, swiping a touchscreen, or using keyboard controls) or by script (for example, by the <code><a href="https://www.w3.org/TR/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/interaction.html#dom-window-focus">focus()</a></code> APIs). The initial position of the [scrollable overflow rectangle](#scrollable-overflow-rectangle) within the [scrollport](#scrollport) before any scrolling operations take effect is the <a id="initial-scroll-position"></a>initial scroll position. The [initial scroll position](#initial-scroll-position) is typically dependent on the [scroll container](#scroll-container)’s [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode), and, unless otherwise specified, coincides with its [scroll origin position](#scroll-origin-position). For example, [scroll-initial-target](https://drafts.csswg.org/css-scroll-snap-2/#propdef-scroll-initial-target) property can be used to change the <a id="ref-for-initial-scroll-position①"></a>initial scroll position. [\[CSS-SCROLL-SNAP-2\]](#biblio-css-scroll-snap-2)

<a id="ref-for-scrollable-overflow-rectangle⑤"></a>

<a id="ref-for-scrollport②"></a>

<a id="ref-for-scroll-origin"></a>

A <a id="scroll-position"></a>scroll position is a particular alignment of the [scrollable overflow rectangle](#scrollable-overflow-rectangle) within its [scrollport](#scrollport). It is associated with a <a id="scroll-offset"></a>scroll offset which is its distance from the [scroll origin](#scroll-origin).

<a id="ref-for-scrollable-overflow-rectangle⑥"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-flex-container"></a>

<a id="ref-for-main-start"></a>

<a id="ref-for-cross-start"></a>

<a id="ref-for-scroll-origin①"></a>

<a id="ref-for-scroll-container⑦"></a>

<a id="ref-for-scrollport③"></a>

<a id="ref-for-scroll-position"></a>

<a id="ref-for-initial-scroll-position②"></a>

The <a id="scroll-origin"></a>scroll origin is the anchor coordinate of the [scrollable overflow rectangle](#scrollable-overflow-rectangle), from which the <a id="ref-for-scrollable-overflow-rectangle⑦"></a>scrollable overflow rectangle expands. Unless otherwise specified, it is the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) corner of the <a id="ref-for-scrollable-overflow-rectangle⑧"></a>scrollable overflow rectangle. (For example, in a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container) it is the [main-start](https://www.w3.org/TR/css-flexbox-1/#main-start) [cross-start](https://www.w3.org/TR/css-flexbox-1/#cross-start) corner.) Unless otherwise adjusted (e.g. [by content alignment](https://www.w3.org/TR/css-align-3/#overflow-scroll-position) [\[css-align-3\]](#biblio-css-align-3)), the area beyond the [scroll origin](#scroll-origin) in either axis is considered the <a id="unreachable-scrollable-overflow-region"></a>unreachable scrollable overflow region: content rendered here is not accessible to the reader, see [§ 2.2 Scrollable Overflow](#scrollable). A [scroll container](#scroll-container) is said to be scrolled to its <a id="ref-for-scroll-origin②"></a>scroll origin when its <a id="ref-for-scroll-origin③"></a>scroll origin coincides with the corresponding corner of its [scrollport](#scrollport). This [scroll position](#scroll-position), the <a id="scroll-origin-position"></a>scroll origin position, usually, but not always, coincides with the [initial scroll position](#initial-scroll-position).

<a id="ref-for-scroll-snap"></a>

<a id="ref-for-initial-scroll-position③"></a>

<a id="ref-for-scroll-origin-position①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f3202600"></a> For example, [scroll snapping](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap) [\[CSS-SCROLL-SNAP-1\]](#biblio-css-scroll-snap-1) can change the [initial scroll position](#initial-scroll-position) away from the [scroll origin position](#scroll-origin-position).

<a id="ref-for-baseline-alignment"></a>

<a id="ref-for-initial-scroll-position④"></a>

<a id="ref-for-scroll-origin-position②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ba58968a"></a> Check whether things like [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) depend on the [initial scroll position](#initial-scroll-position) or the [scroll origin position](#scroll-origin-position).

<a id="ref-for-scroll-offset"></a>

<a id="ref-for-scroll-origin④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-223d3747"></a> This doesn’t define a coordinate system for [scroll offsets](#scroll-offset). Whether they increase downward/rightward, block/inline-axis endward, or away from the [scroll origin](#scroll-origin) is not defined. Should each API define its coordinate model?

<a id="ref-for-canvas"></a>

<a id="ref-for-principal-writing-mode"></a>

<a id="ref-for-scroll-origin⑤"></a>

<a id="ref-for-initial-scroll-position⑤"></a>

The root viewport, which scrolls the page [canvas](https://www.w3.org/TR/CSS2/intro.html#canvas), uses the [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode) for determining its [scroll origin](#scroll-origin) and [initial scroll position](#initial-scroll-position).

<a id="ref-for-scroll-container⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In the case where a [scroll container](#scroll-container) (or one of its ancestors) is the target of a graphical transform, the UA might need to take this transform into account when mapping user inputs to scrolling operations. For instance, on a touch screen where the user scrolls by directly dragging the content, the transform would be expected to be taken into account to match the direction of scrolling to the gesture. On the other hand, other user inputs (such as the Page Down key, or a 1D scroll wheel) might be more naturally interpreted ignoring the transform. Choosing the appropriate behavior for each scrolling mechanism is the responsibility of the UA.

## <a id="overflow-properties"></a>3.  Scrolling and Clipping Overflow

<a id="ref-for-propdef-overflow-x①"></a>

<a id="ref-for-propdef-overflow-y①"></a>

<a id="ref-for-propdef-overflow⑥"></a>

### <a id="overflow-control"></a>3.1.  Managing Overflow: the [overflow-x](#propdef-overflow-x), [overflow-y](#propdef-overflow-y), and [overflow](#propdef-overflow) properties

<a id="ref-for-overflow③"></a>

<a id="ref-for-scroll-container⑨"></a>

These properties specify whether a box’s [overflow](#overflow) is clipped, and if so, whether it is a [scroll container](#scroll-container).

| Field               | Definition                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow-x"></a>overflow-x, <a id="propdef-overflow-y"></a>overflow-y, <a id="propdef-overflow-block"></a>overflow-block, <a id="propdef-overflow-inline"></a>overflow-inline                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>visible [\|](https://www.w3.org/TR/css-values-4/#comb-one) hidden <a id="ref-for-comb-one①"></a>\| clip <a id="ref-for-comb-one②"></a>\| scroll <a id="ref-for-comb-one③"></a>\| auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | <a id="ref-for-valdef-overflow-visible"></a>[visible](#valdef-overflow-visible)                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers [\[CSS2\]](#biblio-css2), flex containers [\[CSS3-FLEXBOX\]](#biblio-css3-flexbox), grid containers [\[CSS3-GRID-LAYOUT\]](#biblio-css3-grid-layout)   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | usually specified value, but see text                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-overflow⑦"></a>[overflow](#propdef-overflow)                                                                                                                        |

<a id="ref-for-propdef-overflow-x②"></a>

<a id="ref-for-overflow④"></a>

<a id="ref-for-propdef-overflow-y②"></a>

The [overflow-x](#propdef-overflow-x) property specifies the handling of [overflow](#overflow) in the horizontal axis (i.e., overflow from the left and right sides of the box), and the [overflow-y](#propdef-overflow-y) property specifies the handling of <a id="ref-for-overflow⑤"></a>overflow in the vertical axis (i.e., overflow from the top and bottom sides of the box).

<a id="ref-for-propdef-overflow-block"></a>

<a id="ref-for-propdef-overflow-inline"></a>

<a id="ref-for-overflow⑥"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-inline-axis"></a>

The [overflow-block](#propdef-overflow-block) and [overflow-inline](#propdef-overflow-inline) properties likewise specify the handling of [overflow](#overflow) in the [block](https://www.w3.org/TR/css-writing-modes-4/#block-axis) and [inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) axis, respectively

<a id="ref-for-logical-property-group"></a>

<a id="ref-for-propdef-overflow⑧"></a>

<a id="ref-for-shorthand-property"></a>

These four properties form a [logical property group](https://www.w3.org/TR/css-logical-1/#logical-property-group) together with the [overflow](#propdef-overflow) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property), and interact as defined in [CSS Logical Properties 1 § 4 Flow-Relative Box Model Properties](https://www.w3.org/TR/css-logical-1/#box).

| Field               | Definition                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow"></a>overflow                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-propdef-overflow-block①"></a>[\<'overflow-block'\>](#propdef-overflow-block)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | visible                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers [\[CSS2\]](#biblio-css2), flex containers [\[CSS3-FLEXBOX\]](#biblio-css3-flexbox), and grid containers [\[CSS3-GRID-LAYOUT\]](#biblio-css3-grid-layout) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                               |

<a id="ref-for-propdef-overflow⑨"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-overflow-x③"></a>

<a id="ref-for-propdef-overflow-y③"></a>

The [overflow](#propdef-overflow) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets the specified values of [overflow-x](#propdef-overflow-x) and [overflow-y](#propdef-overflow-y) in that order. If the second value is omitted, it is copied from the first.

Values have the following meanings:

<a id="valdef-overflow-visible"></a>visible  
<a id="ref-for-scroll-container①⓪"></a>

There is no special handling of overflow, that is, the box’s content is rendered outside the box if positioned there. The box is not a [scroll container](#scroll-container).

<a id="valdef-overflow-hidden"></a>hidden  
<a id="ref-for-scroll-container①①"></a>

<a id="ref-for-padding-box①"></a>

This value indicates that the box’s content is clipped to its [padding box](https://www.w3.org/TR/css-box-4/#padding-box) and that the UA must not provide any scrolling user interface to view the content outside the clipping region, nor allow scrolling by direct intervention of the user, such as dragging on a touch screen or using the scrolling wheel on a mouse. However, the content must still be scrollable programmatically, for example using the mechanisms defined in [\[CSSOM-VIEW\]](#biblio-cssom-view), and the box is therefore still a [scroll container](#scroll-container).

<a id="valdef-overflow-clip"></a>clip  
<a id="ref-for-scroll-container①②"></a>

<a id="ref-for-propdef-overflow①⓪"></a>

<a id="ref-for-overflow-clip-edge"></a>

This value indicates that the box’s content is clipped to its [overflow clip edge](#overflow-clip-edge) and that no scrolling user interface should be provided by the UA to view the content outside the clipping region. In addition, unlike [overflow: hidden](#propdef-overflow) which still allows programmatic scrolling, <a id="ref-for-propdef-overflow①①"></a>overflow: clip forbids scrolling entirely, through any mechanism, and therefore the box is not a [scroll container](#scroll-container).

<a id="ref-for-valdef-overflow-hidden"></a>

Unlike [hidden](#valdef-overflow-hidden), this value <strong>does not</strong> cause the element to establish a new formatting context.

<a id="ref-for-propdef-display"></a>

<a id="ref-for-propdef-overflow①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors who also want the box to establish a formatting context may use [display: flow-root](https://www.w3.org/TR/css-display-3/#propdef-display) together with [overflow: clip](#propdef-overflow).

<a id="valdef-overflow-scroll"></a>scroll  
<a id="ref-for-scroll-container①③"></a>

<a id="ref-for-padding-box②"></a>

This value indicates that the content is clipped to the [padding box](https://www.w3.org/TR/css-box-4/#padding-box), but can be scrolled into view (and therefore the box is a [scroll container](#scroll-container)). Furthermore, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism should be displayed whether or not any of its content is clipped. This avoids any problem with scrollbars appearing and disappearing in a dynamic environment. When the target medium is print, overflowing content may be printed; it is not defined where it may be printed.

<a id="valdef-overflow-auto"></a>auto  
<a id="ref-for-valdef-overflow-hidden①"></a>

<a id="ref-for-scrollable-overflow⑤"></a>

<a id="ref-for-valdef-overflow-scroll"></a>

Like [scroll](#valdef-overflow-scroll) when the box has [scrollable overflow](#scrollable-overflow); like [hidden](#valdef-overflow-hidden) otherwise. Thus, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism will only be displayed if there is overflow.

<a id="ref-for-valdef-overflow-scroll①"></a>

<a id="ref-for-valdef-overflow-auto"></a>

<a id="ref-for-valdef-overflow-hidden②"></a>

<a id="ref-for-propdef-overflow①③"></a>

<a id="ref-for-valdef-overflow-visible①"></a>

<a id="ref-for-valdef-overflow-clip①"></a>

The [scroll](#valdef-overflow-scroll), [auto](#valdef-overflow-auto), and [hidden](#valdef-overflow-hidden) values are known as the <a id="scrollable-overflow-value"></a>scrollable values of [overflow](#propdef-overflow). The [visible](#valdef-overflow-visible) and [clip](#valdef-overflow-clip) values are known as the <a id="non-scrollable-overflow-value"></a>non-scrollable values.

<a id="ref-for-valdef-overflow-visible②"></a>

<a id="ref-for-valdef-overflow-clip②"></a>

<a id="ref-for-propdef-overflow①④"></a>

<a id="ref-for-valdef-overflow-auto①"></a>

<a id="ref-for-valdef-overflow-hidden③"></a>

<a id="ref-for-propdef-overflow-x④"></a>

<a id="ref-for-propdef-overflow-y④"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-used-value"></a>

The [visible](#valdef-overflow-visible)/[clip](#valdef-overflow-clip) values of [overflow](#propdef-overflow) compute to [auto](#valdef-overflow-auto)/[hidden](#valdef-overflow-hidden) (respectively) if one of [overflow-x](#propdef-overflow-x) or [overflow-y](#propdef-overflow-y) is neither <a id="ref-for-valdef-overflow-visible③"></a>visible nor <a id="ref-for-valdef-overflow-clip③"></a>clip. On [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element), a [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) <a id="ref-for-valdef-overflow-hidden④"></a>hidden value further resolves to a [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of <a id="ref-for-valdef-overflow-clip④"></a>clip.

<a id="ref-for-propdef-overflow①⑤"></a>

<a id="ref-for-block-box"></a>

<a id="ref-for-valdef-overflow-visible④"></a>

<a id="ref-for-valdef-overflow-clip⑤"></a>

<a id="ref-for-establish-an-independent-formatting-context"></a>

If the computed value of [overflow](#propdef-overflow) on a [block box](https://www.w3.org/TR/css-display-4/#block-box) is neither [visible](#valdef-overflow-visible) nor [clip](#valdef-overflow-clip) nor a combination thereof, it [establishes an independent formatting context](https://www.w3.org/TR/css-display-4/#establish-an-independent-formatting-context) for its contents.

<a id="ref-for-css-legacy-value-alias"></a>

<a id="ref-for-valdef-overflow-auto②"></a>

User agents must also support the <a id="valdef-overflow-overlay"></a>overlay keyword as a [legacy value alias](https://www.w3.org/TR/css-cascade-5/#css-legacy-value-alias) of [auto](#valdef-overflow-auto).

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-overflow①⑥"></a>

#### <a id="scroll-visibility"></a>3.1.1.  Interaction of [visibility](https://www.w3.org/TR/css-display-4/#propdef-visibility) and [overflow](#propdef-overflow)

<a id="ref-for-propdef-visibility①"></a>

<a id="ref-for-valdef-visibility-hidden"></a>

<a id="ref-for-valdef-visibility-collapse"></a>

<a id="ref-for-propdef-overflow①⑦"></a>

<a id="ref-for-valdef-overflow-scroll②"></a>

<a id="ref-for-valdef-overflow-auto③"></a>

If the computed value of the [visibility](https://www.w3.org/TR/css-display-4/#propdef-visibility) property is [hidden](https://www.w3.org/TR/css-display-4/#valdef-visibility-hidden) (or [collapse](https://www.w3.org/TR/css-display-4/#valdef-visibility-collapse) when it has the same effect as <a id="ref-for-valdef-visibility-hidden①"></a>hidden), and [overflow](#propdef-overflow) is either [scroll](#valdef-overflow-scroll) or [auto](#valdef-overflow-auto), then:

- <a id="ref-for-propdef-visibility②"></a>

  The user agent must not make any scrolling mechanism visible. To the extent that the scrolling mechanism that would normally be visible in the absence of [visibility: hidden](https://www.w3.org/TR/css-display-4/#propdef-visibility) affects layout, it continues to do so, but is not painted.

- <a id="ref-for-propdef-overflow①⑧"></a>

  As would be the case with [overflow: hidden](#propdef-overflow), scrolling directly triggered by user interactions is disabled, but programmatic scrolling continues to take effect.

- <a id="ref-for-propdef-visibility③"></a>

  <a id="ref-for-scroll-container①④"></a>

  The lack of interactive direct scrolling is enforced even if the user interacts (e.g. with a mouse scrolling wheel) with a descendent of the [visibility: hidden](https://www.w3.org/TR/css-display-4/#propdef-visibility) [scroll container](#scroll-container) that is itself set to <a id="ref-for-propdef-visibility④"></a>visibility: visible.

<a id="ref-for-propdef-border-radius"></a>

<a id="ref-for-propdef-overflow①⑨"></a>

#### <a id="corner-clipping"></a>3.1.2.  Interaction of [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) and [overflow](#propdef-overflow)

<a id="ref-for-propdef-overflow②⓪"></a>

As mentioned in [CSS Backgrounds 3 § 4.3 Corner Clipping](https://www.w3.org/TR/css-backgrounds-3/#corner-clipping), the clipping region established by [overflow](#propdef-overflow) can be rounded:

- <a id="ref-for-propdef-overflow-x⑤"></a>

  <a id="ref-for-propdef-overflow-y⑤"></a>

  <a id="ref-for-valdef-overflow-hidden⑤"></a>

  <a id="ref-for-valdef-overflow-scroll③"></a>

  <a id="ref-for-valdef-overflow-auto④"></a>

  <a id="ref-for-padding-edge"></a>

  When [overflow-x](#propdef-overflow-x) and [overflow-y](#propdef-overflow-y) compute to [hidden](#valdef-overflow-hidden), [scroll](#valdef-overflow-scroll), or [auto](#valdef-overflow-auto), the clipping region is rounded based on the border radius, adjusted to the [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge), as described in [CSS Backgrounds 3 § 4.2 Corner Shaping](https://www.w3.org/TR/css-backgrounds-3/#corner-shaping).

- <a id="ref-for-propdef-overflow-x⑥"></a>

  <a id="ref-for-propdef-overflow-y⑥"></a>

  <a id="ref-for-valdef-overflow-clip⑥"></a>

  When both [overflow-x](#propdef-overflow-x) and [overflow-y](#propdef-overflow-y) compute to [clip](#valdef-overflow-clip), the clipping region is rounded as described in [§ 3.2 Expanding Clipping Bounds: the overflow-clip-margin property](#overflow-clip-margin).

- <a id="ref-for-propdef-overflow-x⑦"></a>

  <a id="ref-for-propdef-overflow-y⑦"></a>

  <a id="ref-for-valdef-overflow-clip⑦"></a>

  <a id="ref-for-valdef-overflow-visible⑤"></a>

  However, when one of [overflow-x](#propdef-overflow-x) or [overflow-y](#propdef-overflow-y) computes to [clip](#valdef-overflow-clip) and the other computes to [visible](#valdef-overflow-visible), the clipping region is not rounded.

#### <a id="static-media"></a>3.1.3.  Overflow in Print and Other Static Media

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Since scrolling is not possible in static media (such as print) authors should be careful to make content accessible in such media, for example by using @media print, (update: none) { … } to adjust layout such that all relevant content is simultaneously visible.

<a id="ref-for-scroll-container①⑤"></a>

<a id="ref-for-propdef-overflow②①"></a>

<a id="ref-for-valdef-overflow-auto⑤"></a>

<a id="ref-for-valdef-overflow-scroll④"></a>

<a id="ref-for-valdef-overflow-hidden⑥"></a>

On [scroll containers](#scroll-container) in non-interactive media with an [overflow](#propdef-overflow) value of [auto](#valdef-overflow-auto) or [scroll](#valdef-overflow-scroll) (but not [hidden](#valdef-overflow-hidden)) UAs may display an indication of any scrollable overflow, such as by displaying scrollbars or an ellipsis.

<a id="ref-for-paged-media"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Not all [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media) is non-interactive: for example, e-book readers paginate content, but are interactive.

<a id="ref-for-propdef-overflow-clip-margin②"></a>

### <a id="overflow-clip-margin"></a>3.2.  Expanding Clipping Bounds: the [overflow-clip-margin](#propdef-overflow-clip-margin) property

| Field               | Definition                                                                                                                                                                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow-clip-margin"></a>overflow-clip-margin                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-visual-box"></a>[\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0px                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-overflow②②"></a>boxes to which [overflow](#propdef-overflow) applies                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-visual-box①"></a><a id="ref-for-length-value①"></a>the computed [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) and a [\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) keyword                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-typedef-visual-box②"></a>per computed value if the [\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) values match; otherwise discrete                                                                                                        |

<a id="ref-for-propdef-overflow②③"></a>

<a id="ref-for-overflow-clip-edge①"></a>

This property defines the <a id="overflow-clip-edge"></a>overflow clip edge of the box, i.e. precisely <em>how far</em> outside its bounds the box’s content is allowed to paint before being clipped by effects (such as [overflow: clip](#propdef-overflow), above) that are defined to clip to the box’s [overflow clip edge](#overflow-clip-edge).

Values are defined as follows:

<a id="ref-for-typedef-visual-box③"></a>

<a id="valdef-overflow-clip-margin-visual-box"></a>[\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box)

<a id="ref-for-overflow-clip-edge②"></a>

Specifies the box edge to use as the [overflow clip edge](#overflow-clip-edge) origin, i.e. when the specified offset is zero.

If omitted, defaults to padding-box.

<a id="ref-for-length-value②"></a>

<a id="valdef-overflow-clip-margin-length-0"></a>[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-overflow-clip-edge③"></a>

The specified offset dictates how much the [overflow clip edge](#overflow-clip-edge) is expanded from the specified box edge Negative values are invalid. Defaults to zero if omitted.

<a id="ref-for-overflow-clip-edge④"></a>

<a id="ref-for-box-shadow-outer-box-shadow"></a>

<a id="ref-for-border-edge"></a>

The [overflow clip edge](#overflow-clip-edge) is shaped in the corners exactly the same way as an [outer box-shadow](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-outer-box-shadow) with a spread radius of the same cumulative offset from the box’s [border edge](https://www.w3.org/TR/css-box-4/#border-edge). See [CSS Backgrounds 3 § 4.2 Corner Shaping](https://www.w3.org/TR/css-backgrounds-3/#corner-shaping) and [CSS Backgrounds 3 § 6.1.1 Shadow Shape, Spread, and Knockout](https://www.w3.org/TR/css-backgrounds-3/#shadow-shape), noting in particular the formula for outsets beyond the <a id="ref-for-border-edge①"></a>border edge.

<a id="ref-for-propdef-overflow②④"></a>

<a id="ref-for-overflow-clip-edge⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property has no effect on boxes with [overflow: hidden](#propdef-overflow) or <a id="ref-for-propdef-overflow②⑤"></a>overflow: scroll, which are not defined to use the [overflow clip edge](#overflow-clip-edge).

### <a id="overflow-propagation"></a>3.3.  Overflow Viewport Propagation

<a id="ref-for-x1"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-display-none"></a>

<a id="ref-for-the-html-element"></a>

<a id="ref-for-propdef-overflow②⑥"></a>

<a id="ref-for-valdef-overflow-visible⑥"></a>

<a id="ref-for-the-body-element"></a>

UAs must apply the overflow-\* values set on the root element to the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1) when the root element’s [display](https://www.w3.org/TR/css-display-3/#propdef-display) value is not [none](https://www.w3.org/TR/css-display-4/#valdef-display-none). However, when the root element is an [\[HTML\]](#biblio-html) <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> element (including [XML syntax for HTML](https://html.spec.whatwg.org/multipage/introduction.html#html-vs-xhtml)) whose [overflow](#propdef-overflow) value is [visible](#valdef-overflow-visible) (in both axes), and that element has as a child a <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element whose <a id="ref-for-propdef-display②"></a>display value is also not <a id="ref-for-valdef-display-none①"></a>none, user agents must instead apply the overflow-\* values of the first such child element to the viewport. The element from which the value is propagated must then have a used <a id="ref-for-propdef-overflow②⑦"></a>overflow value of <a id="ref-for-valdef-overflow-visible⑦"></a>visible.

<a id="ref-for-containment"></a>

<a id="ref-for-the-html-element①"></a>

<a id="ref-for-the-body-element①"></a>

<a id="ref-for-the-body-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Using [containment](https://www.w3.org/TR/css-contain-2/#containment) on the HTML <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> elements disables this special handling of the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element. See the [CSS Containment 1 § 2 Strong Containment: the contain property](https://www.w3.org/TR/css-contain-1/#contain-property) for details.

<a id="ref-for-propdef-overflow②⑧"></a>

<a id="ref-for-initial-containing-block"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [overflow: hidden](#propdef-overflow) on the root element might not clip everything outside the [Initial Containing Block](https://www.w3.org/TR/css-display-4/#initial-containing-block) if the ICB is smaller than the viewport, which can happen on mobile.

<a id="ref-for-valdef-overflow-visible⑧"></a>

<a id="ref-for-valdef-overflow-auto⑥"></a>

<a id="ref-for-valdef-overflow-clip⑧"></a>

<a id="ref-for-valdef-overflow-hidden⑦"></a>

If [visible](#valdef-overflow-visible) is applied to the viewport, it must be interpreted as [auto](#valdef-overflow-auto). If [clip](#valdef-overflow-clip) is applied to the viewport, it must be interpreted as [hidden](#valdef-overflow-hidden).

<a id="ref-for-propdef-scroll-behavior"></a>

### <a id="smooth-scrolling"></a>3.4.  Smooth Scrolling: the [scroll-behavior](#propdef-scroll-behavior) Property

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-scroll-behavior"></a>scroll-behavior                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) smooth |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-scroll-container①⑥"></a>[scroll containers](#scroll-container)                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                    |

<a id="ref-for-propdef-scroll-behavior①"></a>

<a id="ref-for-scroll-container①⑦"></a>

<a id="ref-for-x1①"></a>

The [scroll-behavior](#propdef-scroll-behavior) property specifies the scrolling behavior for a [scroll container](#scroll-container), when scrolling happens due to navigation, scrolling APIs [\[CSSOM-VIEW\]](#biblio-cssom-view), or scroll snapping operations not initiated by the user [\[CSS-SCROLL-SNAP-1\]](#biblio-css-scroll-snap-1). Any other scrolls, e.g. those that are performed by the user, are not affected by this property. When this property is specified on the root element, it applies to the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1) instead.

<a id="ref-for-propdef-scroll-behavior②"></a>

<a id="ref-for-the-body-element③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [scroll-behavior](#propdef-scroll-behavior) property of the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element is <em>not</em> propagated to the viewport.

<a id="valdef-scroll-behavior-auto"></a>auto  
<a id="ref-for-concept-instant-scroll"></a>

<a id="ref-for-scroll-container①⑧"></a>

The [scroll container](#scroll-container) is scrolled in an [instant](https://www.w3.org/TR/cssom-view-1/#concept-instant-scroll) fashion.

<a id="valdef-scroll-behavior-smooth"></a>smooth  
<a id="ref-for-concept-smooth-scroll"></a>

<a id="ref-for-scroll-container①⑨"></a>

The [scroll container](#scroll-container) is scrolled in a [smooth](https://www.w3.org/TR/cssom-view-1/#concept-smooth-scroll) fashion using a user-agent-defined timing function over a user-agent-defined period of time. User agents should follow platform conventions, if any.

User agents may ignore this property.

## <a id="scrollbar-layout"></a>4.  Scrollbars and Layout

### <a id="scrollbar-sizing"></a>4.1.  Scrollbar Contributions to Sizing

<a id="ref-for-background-positioning-area"></a>

<a id="ref-for-background-painting-area"></a>

<a id="ref-for-padding-box③"></a>

When reserving space for a scrollbar placed at the edge of an element’s box, the reserved space is inserted between the inner border edge and the outer padding edge. For the purpose of the [background positioning area](https://www.w3.org/TR/css-backgrounds-3/#background-positioning-area) and [background painting area](https://www.w3.org/TR/css-backgrounds-3/#background-painting-area) however, this reserved space is considered to be part of the [padding box](https://www.w3.org/TR/css-box-4/#padding-box).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eb6b07cc"></a> In the following document fragment, both an absolutely-positioned element and a background image are positioned to the top right of the box.
>
> ```text
> <style>
>   article {
>     background: top right no-repeat url(circle.png);
>     position: relative;
>     overflow: auto; }
>   aside { position: absolute; top: 0; right: 0; }
> </style>
> <article>
>   <aside>×</aside>
> </article>
> ```
>
> <a id="ref-for-background-positioning-area①"></a>
>
> If no scrollbars are present on `<article>`, they will both coincide in the top right padding edge corner. However, if scrollbars are present then `<aside>` will be completely visible, on the right padding-box edge adjacent to the scrollbars; whereas the background image will be tucked underneath the scrollbars, in the top right corner of the scrollbar-extended [background positioning area](https://www.w3.org/TR/css-backgrounds-3/#background-positioning-area).

<a id="ref-for-content-area"></a>

When the box is intrinsically sized, this reserved space is added to the size of its contents. It is otherwise subtracted from space allotted to the [content area](https://www.w3.org/TR/css-box-4/#content-area). To the extent that the presence of scrollbars can affect sizing, UAs must start with the assumption that no scrollbars are needed, and recalculate sizes if it turns out they are.

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-max-height"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4d3316b1"></a> In the following document fragment, the outer `<article>` has [height: auto](https://www.w3.org/TR/css-sizing-3/#propdef-height), but [max-height: 5em](https://www.w3.org/TR/css-sizing-3/#propdef-max-height). The inner `<section>` has large margins and would normally just fit:
>
> ```text
> ...
>     article { overflow: auto; max-height: 5em;  width: max-content; }
>     section { margin: 2em; line-height: 1 }
> ...
> <article>
>   <section>
>     This section has big margins.
>   </section>
> </article>
> ```
>
> If we assumed that `<article>` needed scrollbars, then the height of `<section>`, including the single line of text and twice 2em of margins, adds up to 5em plus a scrollbar. Since that is greater than 5em, the maximum allowed height, it seems we made the right assumption and d1 indeed needs scrollbars.
>
> However, we should have started by assuming that no scrollbars are needed. In that case the content height of `<article>` is exactly the maximum height of 5em, proving that the assumption was correct and `<article>` indeed should not have scrollbars.

<a id="ref-for-propdef-scrollbar-gutter"></a>

### <a id="scrollbar-gutter-property"></a>4.2.  Reserving space for the scrollbar: the [scrollbar-gutter](#propdef-scrollbar-gutter) property

The space between the inner border edge and the outer padding edge which user agents may reserve to display the scrollbar is called the <a id="scrollbar-gutter"></a>scrollbar gutter.

<a id="ref-for-propdef-scrollbar-gutter①"></a>

<a id="ref-for-scrollbar-gutter"></a>

<a id="ref-for-propdef-overflow②⑨"></a>

The [scrollbar-gutter](#propdef-scrollbar-gutter) property gives control to the author over the presence of [scrollbar gutters](#scrollbar-gutter) separately from the ability to control the presence of scrollbars provided by the [overflow](#propdef-overflow) property.

| Field               | Definition                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-scrollbar-gutter"></a>scrollbar-gutter                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-comb-all"></a><a id="ref-for-comb-one⑤"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) stable [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) both-edges[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-scroll-container②⓪"></a>[scroll containers](#scroll-container)                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                           |

<a id="ref-for-scrollbar-gutter①"></a>

<a id="ref-for-inline-start①"></a>

<a id="ref-for-inline-end"></a>

This property affects the presence of [scrollbar gutters](#scrollbar-gutter) placed at the [inline start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge or [inline end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge of the box.

<a id="ref-for-scrollbar-gutter②"></a>

<a id="ref-for-block-start①"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-inline-start②"></a>

<a id="ref-for-inline-end①"></a>

<a id="ref-for-propdef-scrollbar-gutter②"></a>

<a id="ref-for-valdef-scrollbar-gutter-auto"></a>

The presence of a [scrollbar gutter](#scrollbar-gutter) at the [block start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge and [block end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge of the box cannot be controlled in this level, and is determined the same way as the presence of <a id="ref-for-scrollbar-gutter③"></a>scrollbar gutters placed at the [inline start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge or [inline end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge of the box when [scrollbar-gutter](#propdef-scrollbar-gutter) is [auto](#valdef-scrollbar-gutter-auto).

<a id="ref-for-scrollbar-gutter④"></a>

Scrollbars which by default are placed over the content box and do not cause [scrollbar gutters](#scrollbar-gutter) to be created are called <a id="overlay-scrollbars"></a>overlay scrollbars. Such scrollbars are usually partially transparent, revealing the content behind them if any. Their appearance and size may vary based on whether and how the user is interacting with them.

<a id="ref-for-scrollbar-gutter⑤"></a>

Scrollbars which are always placed in a [scrollbar gutter](#scrollbar-gutter), consuming space when present, are called <a id="classic-scrollbars"></a>classic scrollbars. Such scrollbars are usually opaque.

<a id="ref-for-classic-scrollbars"></a>

<a id="ref-for-overlay-scrollbars"></a>

Whether [classic scrollbars](#classic-scrollbars) or [overlay scrollbars](#overlay-scrollbars) are used, the appearance and size of the scrollbar, and whether scrollbars appear on the start or end edge of the box, is UA defined.

<a id="ref-for-bidirectionality"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Which side a scrollbar appears on may depend on operating system conventions, [bidirectionality](https://www.w3.org/TR/css-writing-modes-4/#bidirectionality), or other ergonomic considerations.

<a id="ref-for-classic-scrollbars①"></a>

<a id="ref-for-scrollbar-gutter⑥"></a>

<a id="ref-for-overlay-scrollbars①"></a>

In the case of [classic scrollbars](#classic-scrollbars), the width of the [scrollbar gutter](#scrollbar-gutter), if present (see below), is the same as the width of the scrollbar. In the case of [overlay scrollbars](#overlay-scrollbars), no <a id="ref-for-scrollbar-gutter⑦"></a>scrollbar gutter is present.

<a id="ref-for-scrollbar-gutter⑧"></a>

<a id="ref-for-overlay-scrollbars②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There are known use cases that could be addressed by enabling [scrollbar gutters](#scrollbar-gutter) for [overlay scrollbars](#overlay-scrollbars), but no satisfactory design has been agreed to so far. This could be addressed by future extensions of this property. See [CSS Overflow 4 § B Possible extensions for scrollbar-gutter](https://www.w3.org/TR/css-overflow-4/#sbg-ext).

The values of this property have the following meaning:

<a id="valdef-scrollbar-gutter-auto"></a>auto  
<a id="ref-for-overlay-scrollbars③"></a>

<a id="ref-for-valdef-overflow-auto⑦"></a>

<a id="ref-for-valdef-overflow-scroll⑤"></a>

<a id="ref-for-propdef-overflow③⓪"></a>

<a id="ref-for-scrollbar-gutter⑨"></a>

<a id="ref-for-classic-scrollbars②"></a>

[Classic scrollbars](#classic-scrollbars) consume space by creating a [scrollbar gutter](#scrollbar-gutter) when [overflow](#propdef-overflow) is [scroll](#valdef-overflow-scroll), or when <a id="ref-for-propdef-overflow③①"></a>overflow is [auto](#valdef-overflow-auto) and the box is overflowing. [Overlay scrollbars](#overlay-scrollbars) do not consume space.

<a id="valdef-scrollbar-gutter-stable"></a>stable  
<a id="ref-for-overlay-scrollbars④"></a>

<a id="ref-for-valdef-overflow-auto⑧"></a>

<a id="ref-for-valdef-overflow-scroll⑥"></a>

<a id="ref-for-valdef-overflow-hidden⑧"></a>

<a id="ref-for-propdef-overflow③②"></a>

<a id="ref-for-classic-scrollbars③"></a>

<a id="ref-for-scrollbar-gutter①⓪"></a>

The [scrollbar gutter](#scrollbar-gutter) is present for [classic scrollbars](#classic-scrollbars) when [overflow](#propdef-overflow) is [hidden](#valdef-overflow-hidden), [scroll](#valdef-overflow-scroll), or [auto](#valdef-overflow-auto), regardless of whether the box is actually overflowing. [Overlay scrollbars](#overlay-scrollbars) do not consume space.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not change whether the scrollbar itself is visible, only the presence of a gutter is affected.

<a id="valdef-scrollbar-gutter-both-edges"></a>both-edges  
<a id="ref-for-scrollbar-gutter①①"></a>

If a [scrollbar gutter](#scrollbar-gutter) would be present on one of the inline start edge or the inline end edge of the box, another <a id="ref-for-scrollbar-gutter①②"></a>scrollbar gutter must be present on the opposite edge as well.

<a id="ref-for-scrollbar-gutter①③"></a>

When the [scrollbar gutter](#scrollbar-gutter) is present but the scrollbar is not, or the scrollbar is transparent or otherwise does not fully obscure the <a id="ref-for-scrollbar-gutter①④"></a>scrollbar gutter, the background of the <a id="ref-for-scrollbar-gutter①⑤"></a>scrollbar gutter must be painted as an extension of the padding.

<a id="ref-for-propdef-overflow③③"></a>

<a id="ref-for-propdef-scrollbar-gutter③"></a>

<a id="ref-for-the-body-element④"></a>

As for the [overflow](#propdef-overflow) property, when [scrollbar-gutter](#propdef-scrollbar-gutter) is set on the root element, the user agent must apply it to the viewport instead, and the used value on the root element itself is <a id="ref-for-propdef-scrollbar-gutter④"></a>scrollbar-gutter: auto. However, unlike the <a id="ref-for-propdef-overflow③④"></a>overflow property, the user agent must not propagate <a id="ref-for-propdef-scrollbar-gutter⑤"></a>scrollbar-gutter from the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element.

<a id="ref-for-propdef-overflow③⑤"></a>

<a id="ref-for-propdef-scrollbar-gutter⑥"></a>

<a id="ref-for-classic-scrollbars④"></a>

<a id="ref-for-scrollbar-gutter①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The following table summarizes the interaction of [overflow](#propdef-overflow) and [scrollbar-gutter](#propdef-scrollbar-gutter) showing in which case space is reserved for a [classic scrollbar](#classic-scrollbars)’s [scrollbar gutter](#scrollbar-gutter).
>
> <a id="ref-for-classic-scrollbars⑤"></a>
>
> <a id="ref-for-scrollbar-gutter①⑦"></a>
>
> **Table 6**
>
> Should a [classic scrollbar](#classic-scrollbars)’s [scrollbar gutter](#scrollbar-gutter) be present?
>
> Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.
>
> | <a id="ref-for-propdef-overflow③⑥"></a> [overflow](#propdef-overflow) | <a id="ref-for-propdef-scrollbar-gutter⑦"></a> [scrollbar-gutter](#propdef-scrollbar-gutter) | Overflowing | Not overflowing |
> | --- | --- | --- | --- |
> | <a id="ref-for-valdef-overflow-scroll⑦"></a> [scroll](#valdef-overflow-scroll) | <a id="ref-for-valdef-scrollbar-gutter-auto①"></a> [auto](#valdef-scrollbar-gutter-auto) | yes | yes |
> | [scroll](#valdef-overflow-scroll) | <a id="ref-for-valdef-scrollbar-gutter-stable"></a> [stable](#valdef-scrollbar-gutter-stable) | yes | yes |
> | <a id="ref-for-valdef-overflow-auto⑨"></a> [auto](#valdef-overflow-auto) | <a id="ref-for-valdef-scrollbar-gutter-auto②"></a> [auto](#valdef-scrollbar-gutter-auto) | yes |  |
> | [auto](#valdef-overflow-auto) | <a id="ref-for-valdef-scrollbar-gutter-stable①"></a> [stable](#valdef-scrollbar-gutter-stable) | yes | yes |
> | <a id="ref-for-valdef-overflow-hidden⑨"></a> [hidden](#valdef-overflow-hidden) | <a id="ref-for-valdef-scrollbar-gutter-auto③"></a> [auto](#valdef-scrollbar-gutter-auto) |  |  |
> | [hidden](#valdef-overflow-hidden) | <a id="ref-for-valdef-scrollbar-gutter-stable②"></a> [stable](#valdef-scrollbar-gutter-stable) | yes | yes |
> | <a id="ref-for-valdef-overflow-clip⑨"></a> <a id="ref-for-valdef-overflow-visible⑨"></a> [visible](#valdef-overflow-visible), [clip](#valdef-overflow-clip) | <a id="ref-for-valdef-scrollbar-gutter-auto④"></a> [auto](#valdef-scrollbar-gutter-auto) |  |  |
> | [visible](#valdef-overflow-visible), [clip](#valdef-overflow-clip) | <a id="ref-for-valdef-scrollbar-gutter-stable③"></a> [stable](#valdef-scrollbar-gutter-stable) |  |  |

## <a id="auto-ellipsis"></a>5.  Automatic Ellipses

<a id="ref-for-propdef-text-overflow④"></a>

### <a id="text-overflow"></a>5.1.  Overflow Ellipsis: the [text-overflow](#propdef-text-overflow) property

| Field               | Definition                                                                          |
|---------------------|-------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-overflow"></a>text-overflow                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑥"></a>clip [\|](https://www.w3.org/TR/css-values-4/#comb-one) ellipsis |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | clip                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                            |

<a id="ref-for-end"></a>

<a id="ref-for-propdef-overflow③⑦"></a>

<a id="ref-for-valdef-overflow-visible①⓪"></a>

This property specifies rendering when inline content overflows its [end](https://www.w3.org/TR/css-writing-modes-4/#end) line box edge in the inline progression direction of its block container element ("the block") that has [overflow](#propdef-overflow) other than [visible](#valdef-overflow-visible).

Text can overflow for example when it is prevented from wrapping (e.g. due to `white-space: nowrap` or a single word is too long to fit). Values have the following meanings:

<a id="overflow-clip"></a>clip  
Clip inline content that overflows its block container element. Characters may be only partially rendered.

<a id="overflow-ellipsis"></a>ellipsis  
Render an ellipsis character (U+2026) to represent clipped inline content. Implementations may substitute a more language, script, or writing-mode appropriate ellipsis character, or three dots "..." if the ellipsis character is unavailable.

The term "character" is used in this property definition for better readability and means "grapheme cluster" [\[UAX29\]](#biblio-uax29) for implementation purposes.

<a id="ref-for-end①"></a>

For the ellipsis value implementations must hide characters and [atomic inline-level elements](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) at the [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge of the line as necessary to fit the ellipsis, and place the ellipsis immediately adjacent to the <a id="ref-for-end②"></a>end edge of the remaining inline content. The first character or [atomic inline-level element](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) on a line must be clipped rather than ellipsed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7153ff61"></a>
>
> #### <a id="bidi-ellipsis"></a>Bidi ellipsis examples
>
> These examples demonstrate which characters get hidden to make room for the ellipsis in a bidi situation: those visually at the end edge of the line.
>
> Sample CSS:
>
> ```text
> div {
>   font-family: monospace;
>   white-space: pre;
>   overflow: hidden;
>   width: 9ch;
>   text-overflow: ellipsis;
> }
> ```
>
> Sample HTML fragments, renderings, and your browser:
>
>
> These are static transcriptions of the source’s live browser demonstration. The “browser” text below is the source content, not a measured rendering. Original demonstration HTML/CSS is included so clipping, direction, and line-breaking are not lost. See the [source demonstration](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#example-7153ff61).
>
> **Demonstration stylesheet from the source**
>
> ```css
> .awesome-table td { padding: 5px; }
> .awesome-table {
> 	color: #000;
> 	background: #fff;
> 	margin: auto;
> }
> ```
>
> **Example 1**
>
> **HTML**
>
> `<div>שלום 123456</div>`
>
> **Reference rendering**
>
> 123456 ם…
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family:monospace">123456 ם…</div>
> ```
>
> **Your Browser**
>
> שלום 123456
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family: monospace; white-space: pre; overflow: hidden; width: 9ch; text-overflow: ellipsis">שלום 123456</div>
> ```
>
> **Example 2**
>
> **HTML**
>
> `<div dir=rtl>שלום 123456</div>`
>
> **Reference rendering**
>
> …456 שלום
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family:monospace">…456 שלום</div>
> ```
>
> **Your Browser**
>
> שלום 123456
>
> **Original browser-demonstration HTML**
>
> ```html
> <div dir="rtl" style="font-family: monospace; white-space: pre; overflow: hidden; width: 9ch; text-overflow: ellipsis">שלום 123456</div>
> ```
>

#### <a id="ellipsing-details"></a> ellipsing details

- <a id="ref-for-propdef-text-overflow⑤"></a>

  Ellipsing only affects rendering and must not affect layout nor dispatching of pointer events: The UA should dispatch any pointer event on the ellipsis to the elided element, as if [text-overflow](#propdef-text-overflow) had been none.

- The ellipsis is styled and baseline-aligned according to the block.

- Ellipsing occurs after relative positioning and other graphical transformations.

- If there is insufficient space for the ellipsis, then clip the rendering of the ellipsis itself (on the same side that neutral characters on the line would have otherwise been clipped with the text-overflow:clip value).

#### <a id="ellipsis-interaction"></a> user interaction with ellipsis

- <a id="ref-for-propdef-text-overflow⑥"></a>

  When the user is interacting with content (e.g. editing, selecting, scrolling), the user agent may treat [text-overflow: ellipsis](#propdef-text-overflow) as <a id="ref-for-propdef-text-overflow⑦"></a>text-overflow: clip.

- Selecting the ellipsis should select the ellipsed text. If all of the ellipsed text is selected, UAs should show selection of the ellipsis. Behavior of partially-selected ellipsed text is up to the UA.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cb85f151"></a>
>
> Example(s):
>
> #### <a id="text-overflow-examples"></a>text-overflow examples
>
> These examples demonstrate setting the text-overflow of a block container element that has text which overflows its dimensions:
>
> sample CSS for a div:
>
> ```text
> div {
>   font-family:Helvetica,sans-serif; line-height:1.1;
>   width:3.1em; padding:.2em; border:solid .1em black; margin:1em 0;
> }
> ```
>
> sample HTML fragments, renderings, and your browser:
>
>
> These are static transcriptions of the source’s live browser demonstration. The “browser” text below is the source content, not a measured rendering. Original demonstration HTML/CSS is included so clipping, direction, and line-breaking are not lost. See the [source demonstration](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#example-cb85f151).
>
> **Demonstration stylesheet from the source**
>
> ```css
> .awesome-table td { padding: 5px; }
> .awesome-table {
> 	color: #000;
> 	background: #fff;
> 	margin: auto;
> }
> ```
>
> **Example 1**
>
> **HTML**
>
> ```text
> <div>
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> First, a box with text drawing outside of it.
>
> ![First, a box with text drawing outside of it.](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/images/cssisawesome.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em; font-family:Helvetica,sans-serif; line-height:1.1;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 2**
>
> **HTML**
>
> ```text
> <div style="text-overflow:clip; overflow:hidden">
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> Second, a similar box with the text clipped outside the box.
>
> ![Second, a similar box with the text clipped outside the box.](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/images/cssisaweso.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em; font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:clip;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 3**
>
> **HTML**
>
> ```text
> <div style="text-overflow:ellipsis; overflow:hidden">
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> Third, a similar box with an ellipsis representing the clipped text.
>
> ![Third, a similar box with an ellipsis representing the clipped text.](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/images/cssisaw.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em;  font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:ellipsis;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 4**
>
> **HTML**
>
> ```text
> <div style="text-overflow:ellipsis; overflow:hidden">
> NESTED
>  <p>PARAGRAPH</p>
> WON’T ELLIPSE.
> </div>
> ```
>
> **sample rendering**
>
> Fourth, a box with a nested paragraph demonstrating anonymous block boxes equivalency and non-inheritance into a nested element.
>
> ![Fourth, a box with a nested paragraph demonstrating anonymous block boxes equivalency and non-inheritance into a nested element.](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/images/nes.png)
>
> **your browser**
>
> NESTED
>
> PARAGRAPH
>
> WON’T ELLIPSE.
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em;  font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:ellipsis;">
>          NESTED
> 						
>          <p>PARAGRAPH</p>
>          
> 						WON’T ELLIPSE.
>         </div>
> ```
>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-physical-left"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the side of the line that the ellipsis is placed depends on the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) of the block. E.g. an overflow hidden right-to-left (`direction: rtl`) block clips inline content on the [left](https://www.w3.org/TR/css-writing-modes-3/#physical-left) side, thus would place a text-overflow ellipsis on the <a id="ref-for-physical-left①"></a>left to represent that clipped content.

#### <a id="ellipsis-scrolling"></a> ellipsis interaction with scrolling interfaces

This section applies to elements with text-overflow other than text-overflow:clip (non-clip text-overflow) and overflow:scroll.

When an element with non-clip text-overflow has overflow of scroll in the inline progression dimension of the text, and the browser provides a mechanism for scrolling (e.g. a scrollbar on the element, or a touch interface to swipe-scroll, etc.), there are additional implementation details that provide a better user experience:

When an element is scrolled (e.g. by the user, DOM manipulation), more of the element’s content is shown. The value of text-overflow should not affect whether more of the element’s content is shown or not. If a non-clip text-overflow is set, then as more content is scrolled into view, implementations should show whatever additional content fits, only truncating content which would otherwise be clipped (or is necessary to make room for the ellipsis/string), until the element is scrolled far enough to display the edge of the content at which point that content should be displayed rather than an ellipsis/string.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-45d2531f"></a>
>
> This example uses text-overflow on an element with overflow scroll to demonstrate the above described behavior.
>
> sample CSS:
>
> ```text
> div.crawlbar {
>   text-overflow: ellipsis;
>   height: 2em;
>   overflow: scroll;
>   white-space: nowrap;
>   width: 15em;
>   border:1em solid black;
> }
> ```
>
> sample HTML fragment:
>
> ```text
> <div class="crawlbar">
> CSS is awesome, especially when you can scroll
> to see extra text instead of just
> having it overlap other text by default.
> </div>
> ```
>
> demonstration of sample CSS and HTML:
>
> CSS is awesome, especially when you can scroll to see extra text instead of just having it overlap other text by default.

While the content is being scrolled, implementations may adjust their rendering of ellipses (e.g. align to the box edge rather than line edge).

## <a id="priv"></a>Appendix A. Privacy Considerations

This specification introduces no new privacy concerns.

## <a id="sec"></a>Appendix B. Security Considerations

This specification introduces no new security concerns.

## <a id="changes"></a>Appendix C. Changes

This appendix is <em>informative</em>.

Significant changes since the [29 March 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-overflow-3-20230329/):

- <a id="ref-for-propdef-overflow③⑧"></a>

  <a id="ref-for-replaced-element②"></a>

  Define that [overflow: hidden](#propdef-overflow) is treated as <a id="ref-for-propdef-overflow③⑨"></a>overflow: clip on [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element). ([Issue 7714](https://github.com/w3c/csswg-drafts/issues/7714))

- <a id="ref-for-content-distribute"></a>

  <a id="ref-for-scroll-container②①"></a>

  <a id="ref-for-scrollable-overflow-region⑧"></a>

  Redefine the interaction of overflowing [content distribution](https://www.w3.org/TR/css-align-3/#content-distribute) and [scroll containers](#scroll-container) to not impact layout, but to only affect the extent of the [scrollable overflow area](#scrollable-overflow-region). ([Issue 4957](https://github.com/w3c/csswg-drafts/issues/4957))

- Defined some more terminology to help other specs connect to concepts in this one.

See also [Previous Changes](https://www.w3.org/TR/2023/WD-css-overflow-3-20230329/#changes).

## <a id="acknowledgments"></a> Acknowledgments

Thanks especially to the feedback from Rossen Atanassov, Tab Atkins, Bert Bos, Tantek Çelik, John Daggett, Daniel Glazman, Vincent Hardy, Ian Kilpatrick, Håkon Wium Lie, Peter Linss, Robert O’Callahan, Florian Rivoal, Alan Stearns, Steve Zilles, and all the rest of the [www-style](https://lists.w3.org/Archives/Public/www-style/) community.

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

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong>
        UAs MUST provide an accessible alternative.
    </strong>

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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- auto
  - [value for overflow, overflow-x, overflow-y](#valdef-overflow-auto), in § 3.1
  - [value for scroll-behavior](#valdef-scroll-behavior-auto), in § 3.4
  - [value for scrollbar-gutter](#valdef-scrollbar-gutter-auto), in § 4.2
- [both-edges](#valdef-scrollbar-gutter-both-edges), in § 4.2
- [classic scrollbars](#classic-scrollbars), in § 4.2
- clip
  - [value for overflow, overflow-x, overflow-y](#valdef-overflow-clip), in § 3.1
  - [value for text-overflow](#overflow-clip), in § 5.1
- [ellipsis](#overflow-ellipsis), in § 5.1
- [hidden](#valdef-overflow-hidden), in § 3.1
- [initial scroll position](#initial-scroll-position), in § 2.3
- [ink overflow](#ink-overflow), in § 2.1
- [ink overflow area](#ink-overflow-region), in § 2.1
- [ink overflow rectangle](#ink-overflow-rectangle), in § 2.1
- [ink overflow region](#ink-overflow-region), in § 2.1
- [\<length \[0,∞\]\>](#valdef-overflow-clip-margin-length-0), in § 3.2
- [nearest scrollport](#nearest-scrollport), in § 2.3
- [non-scrollable overflow value](#non-scrollable-overflow-value), in § 3.1
- overflow
  - [(property)](#propdef-overflow), in § 3.1
  - [definition of](#overflow), in § 2
- [overflow-block](#propdef-overflow-block), in § 3.1
- [overflow clip edge](#overflow-clip-edge), in § 3.2
- [overflow-clip-margin](#propdef-overflow-clip-margin), in § 3.2
- [overflow-inline](#propdef-overflow-inline), in § 3.1
- [overflow-x](#propdef-overflow-x), in § 3.1
- [overflow-y](#propdef-overflow-y), in § 3.1
- [overlay](#valdef-overflow-overlay), in § 3.1
- [overlay scrollbars](#overlay-scrollbars), in § 4.2
- [scroll](#valdef-overflow-scroll), in § 3.1
- [scrollable overflow](#scrollable-overflow), in § 2.2
- [scrollable overflow area](#scrollable-overflow-region), in § 2.2
- [scrollable overflow rectangle](#scrollable-overflow-rectangle), in § 2.2
- [scrollable overflow region](#scrollable-overflow-region), in § 2.2
- [scrollable overflow value](#scrollable-overflow-value), in § 3.1
- [scrollbar gutter](#scrollbar-gutter), in § 4.2
- [scrollbar-gutter](#propdef-scrollbar-gutter), in § 4.2
- [scroll-behavior](#propdef-scroll-behavior), in § 3.4
- [scroll container](#scroll-container), in § 2.3
- [scroll offset](#scroll-offset), in § 2.3
- [scroll origin](#scroll-origin), in § 2.3
- [scroll origin position](#scroll-origin-position), in § 2.3
- [scrollport](#scrollport), in § 2.3
- [scroll position](#scroll-position), in § 2.3
- [smooth](#valdef-scroll-behavior-smooth), in § 3.4
- [stable](#valdef-scrollbar-gutter-stable), in § 4.2
- [text-overflow](#propdef-text-overflow), in § 5.1
- [unreachable scrollable overflow region](#unreachable-scrollable-overflow-region), in § 2.3
- [visible](#valdef-overflow-visible), in § 3.1
- [\<visual-box\>](#valdef-overflow-clip-margin-visual-box), in § 3.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="dc2ecc7a"></a>alignment subject
  - <a id="4d2cf2cf"></a>baseline alignment
  - <a id="46bad8dc"></a>content distribution
  - <a id="c4db3a90"></a>content-distribution properties
  - <a id="0ef4bcd3"></a>place-content
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="4433b30f"></a>background painting area
  - <a id="23ef8473"></a>background positioning area
  - <a id="3a4a9318"></a>border-radius
  - <a id="c48eaa20"></a>box-shadow
  - <a id="46036d71"></a>outer box-shadow
- \[CSS-BOX-4\] defines the following terms:
  - <a id="c87746d2"></a>\<visual-box\>
  - <a id="3e6781f5"></a>border edge
  - <a id="df86efcb"></a>content area
  - <a id="15e1e804"></a>padding box
  - <a id="093a0ff1"></a>padding edge
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="7a6ac42f"></a>legacy value alias
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="5dfeee7f"></a>contain
  - <a id="4f31b139"></a>containment
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="2ccfe434"></a>display
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="45f9eae9"></a>block box
  - <a id="031a61f7"></a>collapse
  - <a id="b28a080f"></a>containing block chain
  - <a id="abe7937d"></a>establishes an independent formatting context
  - <a id="d4c109b6"></a>hidden
  - <a id="d1ebdd75"></a>initial containing block
  - <a id="e1f4f7f3"></a>none
  - <a id="380d5174"></a>replaced
  - <a id="a9db5d6d"></a>replaced element
  - <a id="aceda213"></a>visibility
- \[CSS-GRID-2\] defines the following terms:
  - <a id="ba30fc9a"></a>grid item
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="a9330658"></a>line box
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="b174cda6"></a>logical property group
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="e97d95b6"></a>clip
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="50eadabc"></a>block-ellipsis
  - <a id="8c4b9d8f"></a>line-clamp
  - <a id="072f08da"></a>max-lines
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="f6ae3416"></a>scroll snap
- \[CSS-SCROLL-SNAP-2\] defines the following terms:
  - <a id="ec547e39"></a>scroll-initial-target
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="2d68423f"></a>max-height
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="a7f17cc4"></a>text-shadow
- \[CSS-TRANSFORMS-2\] defines the following terms:
  - <a id="5e090b52"></a>3D rendering context
  - <a id="98511d95"></a>transform-style
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="3bafef5e"></a>{A,B}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="8a5584f2"></a>left
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="e02e3409"></a>bidirectionality
  - <a id="b8dade0f"></a>block axis
  - <a id="7922a8cf"></a>block end
  - <a id="c447ee9e"></a>block start
  - <a id="1118d052"></a>block-start
  - <a id="e112902f"></a>end
  - <a id="a6eb24bb"></a>inline axis
  - <a id="905ff85d"></a>inline end
  - <a id="e31b81f6"></a>inline start
  - <a id="0da67e16"></a>inline-start
  - <a id="953ffdad"></a>principal writing mode
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="0a714736"></a>canvas
  - <a id="e12287dd"></a>viewport
- \[CSS3-FLEXBOX\] defines the following terms:
  - <a id="a2a5c5e6"></a>cross-start
  - <a id="cc7f0a64"></a>flex container
  - <a id="9f6d5ab0"></a>flex item
  - <a id="abee8195"></a>main-start
- \[CSSOM-VIEW\] defines the following terms:
  - <a id="69994004"></a>instant scroll
  - <a id="cc250dc6"></a>scrollIntoView()
  - <a id="b7acabc4"></a>smooth scroll
- \[HTML\] defines the following terms:
  - <a id="2f0492ac"></a>body
  - <a id="240893a8"></a>focus()
  - <a id="c3dd181e"></a>html
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="23af89d0"></a>paged media

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 11 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-scroll-snap-1"></a>\[CSS-SCROLL-SNAP-1\]  
Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 11 March 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-1&#x2F;](https://www.w3.org/TR/css-scroll-snap-1/)

<a id="biblio-css-scroll-snap-2"></a>\[CSS-SCROLL-SNAP-2\]  
Elika Etemad; Tab Atkins Jr.; Adam Argyle. [CSS Scroll Snap Module Level 2](https://www.w3.org/TR/css-scroll-snap-2/). 23 July 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-2&#x2F;](https://www.w3.org/TR/css-scroll-snap-2/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://www.w3.org/TR/css-transforms-2/). 9 November 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-2&#x2F;](https://www.w3.org/TR/css-transforms-2/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-flexbox"></a>\[CSS3-FLEXBOX\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css3-grid-layout"></a>\[CSS3-GRID-LAYOUT\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css3-transforms"></a>\[CSS3-TRANSFORMS\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-cssom-view"></a>\[CSSOM-VIEW\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-uax29"></a>\[UAX29\]  
Josh Hadley. [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/tr29-47.html). 17 August 2025. Unicode Standard Annex \#29. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;tr29-47&#x2E;html](https://www.unicode.org/reports/tr29/tr29-47.html)

### <a id="informative"></a>Informative References

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 25 June 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 30 September 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css1"></a>\[CSS1\]  
Håkon Wium Lie; Bert Bos. [Cascading Style Sheets, level 1](https://www.w3.org/TR/CSS1/). 13 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS1&#x2F;](https://www.w3.org/TR/CSS1/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                     | Initial | Applies to                                                                                            | Inh. | %ages | Anim­ation type                                                            | Canonical order | Com­puted value                                       | Logical property group |
|---------------------|-----------------------------------------------------------|---------|-------------------------------------------------------------------------------------------------------|------|-------|---------------------------------------------------------------------------|-----------------|------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-overflow④⓪"></a></span><a href="#propdef-overflow">overflow</a>&#xA;      </strong> | \<'overflow-block'\>{1,2}                                 | visible | block containers \[CSS2\], flex containers \[CSS3-FLEXBOX\], and grid containers \[CSS3-GRID-LAYOUT\] | no   | N/A   | discrete                                                                  | per grammar     | see individual properties                            |                        |
| <strong><span><a id="ref-for-propdef-overflow-block②"></a></span><a href="#propdef-overflow-block">overflow-block</a>&#xA;      </strong> | visible \| hidden \| clip \| scroll \| auto               | visible | block containers \[CSS2\], flex containers \[CSS3-FLEXBOX\], grid containers \[CSS3-GRID-LAYOUT\]     | no   | N/A   | discrete                                                                  | per grammar     | usually specified value, but see text                | overflow               |
| <strong><span><a id="ref-for-propdef-overflow-clip-margin③"></a></span><a href="#propdef-overflow-clip-margin">overflow-clip-margin</a>&#xA;      </strong> | \<visual-box\> \|\| \<length \[0,∞\]\>                    | 0px     | boxes to which overflow applies                                                                       | no   | n/a   | per computed value if the \<visual-box\> values match; otherwise discrete | per grammar     | the computed \<length\> and a \<visual-box\> keyword |                        |
| <strong><span><a id="ref-for-propdef-overflow-inline①"></a></span><a href="#propdef-overflow-inline">overflow-inline</a>&#xA;      </strong> | visible \| hidden \| clip \| scroll \| auto               | visible | block containers \[CSS2\], flex containers \[CSS3-FLEXBOX\], grid containers \[CSS3-GRID-LAYOUT\]     | no   | N/A   | discrete                                                                  | per grammar     | usually specified value, but see text                | overflow               |
| <strong><span><a id="ref-for-propdef-overflow-x⑧"></a></span><a href="#propdef-overflow-x">overflow-x</a>&#xA;      </strong> | visible \| hidden \| clip \| scroll \| auto               | visible | block containers \[CSS2\], flex containers \[CSS3-FLEXBOX\], grid containers \[CSS3-GRID-LAYOUT\]     | no   | N/A   | discrete                                                                  | per grammar     | usually specified value, but see text                | overflow               |
| <strong><span><a id="ref-for-propdef-overflow-y⑧"></a></span><a href="#propdef-overflow-y">overflow-y</a>&#xA;      </strong> | visible \| hidden \| clip \| scroll \| auto               | visible | block containers \[CSS2\], flex containers \[CSS3-FLEXBOX\], grid containers \[CSS3-GRID-LAYOUT\]     | no   | N/A   | discrete                                                                  | per grammar     | usually specified value, but see text                | overflow               |
| <strong><span><a id="ref-for-propdef-scroll-behavior③"></a></span><a href="#propdef-scroll-behavior">scroll-behavior</a>&#xA;      </strong> | auto \| smooth                                            | auto    | scroll containers                                                                                     | no   | n/a   | not animatable                                                            | per grammar     | specified value                                      |                        |
| <strong><span><a id="ref-for-propdef-scrollbar-gutter⑧"></a></span><a href="#propdef-scrollbar-gutter">scrollbar-gutter</a>&#xA;      </strong> | auto \| stable &#x26;&#x26; both-edges? | auto    | scroll containers                                                                                     | no   | n/a   | discrete                                                                  | per grammar     | specified keyword(s)                                 |                        |
| <strong><span><a id="ref-for-propdef-text-overflow⑧"></a></span><a href="#propdef-text-overflow">text-overflow</a>&#xA;      </strong> | clip \| ellipsis                                          | clip    | block containers                                                                                      | no   | N/A   | discrete                                                                  | per grammar     | specified keyword                                    |                        |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is this description of handling transforms sufficiently accurate? [↵](#issue-df7ef6c3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This needs further testing and investigation; is therefore deferred in this draft. [↵](#issue-b9c7269c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Need to evaluate what should/should not clip [scrollable overflow](#scrollable-overflow). [\[Issue \#8607\]](https://github.com/w3c/csswg-drafts/issues/8607) [↵](#issue-b127a295)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Check whether things like [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) depend on the [initial scroll position](#initial-scroll-position) or the [scroll origin position](#scroll-origin-position). [↵](#issue-ba58968a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This doesn’t define a coordinate system for [scroll offsets](#scroll-offset). Whether they increase downward/rightward, block/inline-axis endward, or away from the [scroll origin](#scroll-origin) is not defined. Should each API define its coordinate model? [↵](#issue-223d3747)
