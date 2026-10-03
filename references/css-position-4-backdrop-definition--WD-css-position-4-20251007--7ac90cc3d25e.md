Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Positioned Layout Module Level 4](https://www.w3.org/TR/2025/WD-css-position-4-20251007/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Positioned Layout Module Level 4

Source snapshot: https://www.w3.org/TR/2025/WD-css-position-4-20251007/

Snapshot SHA-256: 7ac90cc3d25e38d054d28f5899774eb3eb7ce50b8a930fee6530681cab043412

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 2 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

<a id="ref-for-containing-block"></a>

<a id="ref-for-absolute-position①"></a>

<a id="ref-for-absolute-positioning-containing-block"></a>

<a id="ref-for-scroll-container"></a>

# <a id="title"></a>CSS Positioned Layout Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-x34"></a>

<a id="ref-for-sticky-position"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-fixed-position"></a>

This module contains defines coordinate-based positioning and offsetting schemes of [CSS](https://www.w3.org/TR/CSS/): [relative positioning](https://www.w3.org/TR/CSS2/visuren.html#x34), [sticky positioning](https://www.w3.org/TR/css-position-3/#sticky-position), [absolute positioning](https://www.w3.org/TR/css-position-3/#absolute-position), and [fixed positioning](https://www.w3.org/TR/css-position-3/#fixed-position).

It also defines the painting/rendering model of CSS.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-position” in the title, like this: “\[css-position\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-position%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

This is an early delta spec over [\[css-position-3\]](#biblio-css-position-3).

## <a id="scrollable-cb"></a>2. Scrollable Containing Block

When a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) establishes an [absolute positioning containing block](https://www.w3.org/TR/css-position-3/#absolute-positioning-containing-block) for an [absolutely positioned box](https://www.w3.org/TR/css-position-3/#absolute-position), one of three possible [containing blocks](https://www.w3.org/TR/css-display-4/#containing-block) are used:

<a id="fixed-containing-block"></a>fixed containing block  
<a id="ref-for-fixed-containing-block"></a>

<a id="ref-for-scroll-container①"></a>

<a id="ref-for-scrollport"></a>

<a id="ref-for-padding-box"></a>

The [fixed containing block](#fixed-containing-block) of a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) corresponds to the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport), i.e. the inner [padding box](https://www.w3.org/TR/css-box-4/#padding-box) edges of the <a id="ref-for-scroll-container②"></a>scroll container, but scrolling with the outer context, not the <a id="ref-for-scroll-container③"></a>scroll container’s contents.

<a id="ref-for-fixed-containing-block①"></a>

<a id="ref-for-fixed-positioning-containing-block"></a>

The [fixed containing block](#fixed-containing-block) established by the document is the [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block).

<a id="local-containing-block"></a>local containing block  
<a id="ref-for-local-containing-block"></a>

<a id="ref-for-scroll-container④"></a>

<a id="ref-for-padding-box①"></a>

<a id="ref-for-scrollable-overflow-region"></a>

The [local containing block](#local-containing-block) of a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) corresponds to the [padding box](https://www.w3.org/TR/css-box-4/#padding-box) edges of the <a id="ref-for-scroll-container⑤"></a>scroll container, but is affixed to the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region), and scrolls with the <a id="ref-for-scroll-container⑥"></a>scroll container’s contents.

<a id="ref-for-local-containing-block①"></a>

<a id="ref-for-initial-containing-block"></a>

The [local containing block](#local-containing-block) established by the document is the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block).

<a id="scrollable-containing-block"></a>scrollable containing block  
<a id="ref-for-scrollable-containing-block"></a>

<a id="ref-for-scroll-container⑦"></a>

<a id="ref-for-padding-edge"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

<a id="ref-for-padding"></a>

<a id="ref-for-local-containing-block②"></a>

The [scrollable containing block](#scrollable-containing-block) of a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) corresponds to the [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) of the <a id="ref-for-scroll-container⑧"></a>scroll container’s [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region), i.e. the outer edge of the [padding](https://www.w3.org/TR/css-box-4/#padding) surrounding its content when determining the extent of its <a id="ref-for-scrollable-overflow-region②"></a>scrollable overflow area. See [CSS Overflow 3 § 2.2 Scrollable Overflow](https://www.w3.org/TR/css-overflow-3/#scrollable). In all cases the <a id="ref-for-scrollable-containing-block①"></a>scrollable containing block at least encompasses the [local containing block](#local-containing-block).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This includes floats, but ignores absolutely positioned descendants, content overflowing descendant boxes, and the effects of relative positioning and transforms, which do otherwise extend the scrollable area to display them.

<a id="ref-for-scrollable-containing-block②"></a>

<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-root-element"></a>

<a id="ref-for-box"></a>

The [scrollable containing block](#scrollable-containing-block) established by the document is the union of the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) and the [margin box](https://www.w3.org/TR/css-box-4/#margin-box) of the [root element](https://www.w3.org/TR/css-display-4/#root-element)’s generated [box](https://www.w3.org/TR/css-display-4/#box).

<a id="ref-for-padding-box②"></a>

<a id="ref-for-scrollable-overflow"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While all of these are in some sense corresponding to the [padding box](https://www.w3.org/TR/css-box-4/#padding-box), they do not coincide for boxes with [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-806ae415"></a> Figure out the exact concepts needed for top layer. They probably want to do something a tiny bit different, specific to their layer.

<a id="ref-for-local-containing-block③"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-fixed-position①"></a>

<a id="ref-for-fixed-containing-block②"></a>

<a id="ref-for-propdef-position-area"></a>

<a id="ref-for-valdef-position-area-none"></a>

<a id="ref-for-scrollable-containing-block③"></a>

Unless otherwise specified, absolutely positioned boxes use the [local containing block](#local-containing-block). Certain CSS features can specify a different [containing block](https://www.w3.org/TR/css-display-4/#containing-block). For example, [fixed-positioned boxes](https://www.w3.org/TR/css-position-3/#fixed-position) typically use the document’s [fixed containing block](#fixed-containing-block), and [position-area](https://www.w3.org/TR/css-anchor-position-1/#propdef-position-area) values other than [none](https://www.w3.org/TR/css-anchor-position-1/#valdef-position-area-none) can opt absolutely positioned boxes into the [scrollable containing block](#scrollable-containing-block).

<a id="ref-for-fixed-containing-block③"></a>

<a id="ref-for-fixed-positioning-containing-block①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is not currently any way to refer to a [fixed containing block](#fixed-containing-block) other than the [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block), but one might be added in the future.

## <a id="top-layer"></a>3. Top Layer

<a id="ref-for-document"></a>

<a id="ref-for-ordered-set"></a>

<a id="ref-for-concept-element"></a>

<a id="ref-for-document-top-layer"></a>

<a id="ref-for-box①"></a>

<code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>s have a <a id="document-top-layer"></a>top layer, an [ordered set](https://infra.spec.whatwg.org/#ordered-set) containing [elements](https://dom.spec.whatwg.org/#concept-element) from the document. Elements in the [top layer](#document-top-layer) do not lay out normally based on their position in the document; instead they generate [boxes](https://www.w3.org/TR/css-display-4/#box) as if they were siblings of the root element. <a id="ref-for-document-top-layer①"></a>Top layer elements are rendered in the order they appear in the <a id="ref-for-document-top-layer②"></a>top layer; the last element in the <a id="ref-for-document-top-layer③"></a>top layer is rendered on top of everything else.

<a id="ref-for-document-top-layer④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This special rendering behavior ensures that elements in the [top layer](#document-top-layer) cannot be clipped by anything in the document, or obscured by anything except elements later in the <a id="ref-for-document-top-layer⑤"></a>top layer. This ensures that things like [popovers](https://html.spec.whatwg.org/multipage/popover.html#the-popover-attribute) can be displayed reliably, regardless of what their ancestor elements might be doing.

<a id="ref-for-document-top-layer⑥"></a>

The <a id="top-layer-root"></a>top layer root of an element is its nearest ancestor element that is in the [top layer](#document-top-layer), or none otherwise (in which case it’s painted as part of the document as normal).

<a id="ref-for-top-layer-root"></a>

<a id="ref-for-document-top-layer⑦"></a>

Two elements are <a id="in-the-same-top-layer"></a>in the same top layer if they have the same [top layer root](#top-layer-root) (including if both are none). An element A is <a id="in-a-higher-top-layer"></a>in a higher top layer than an element B if A has a <a id="ref-for-top-layer-root①"></a>top layer root, and either B has a <a id="ref-for-top-layer-root②"></a>top layer root earlier in the [top layer](#document-top-layer) than A’s, or B doesn’t have a <a id="ref-for-top-layer-root③"></a>top layer root at all.

<a id="ref-for-document-top-layer⑧"></a>

<a id="ref-for-user-agent"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [top layer](#document-top-layer) is managed entirely by the [user agent](https://infra.spec.whatwg.org/#user-agent); it cannot be directly manipulated by authors. This ensures that "nested" invocations of top-layer-using APIs, like a popup within a popup, will display correctly.

<a id="ref-for-document-top-layer⑨"></a>

<a id="ref-for-propdef-overlay"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [top layer](#document-top-layer) interacts with the [overlay](#propdef-overlay) property in a somewhat unusual way. See <a id="ref-for-propdef-overlay①"></a>overlay for details.

<a id="ref-for-document①"></a>

<a id="ref-for-ordered-set①"></a>

<code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>s also have a <a id="pending-top-layer-removals"></a>pending top layer removals [ordered set](https://infra.spec.whatwg.org/#ordered-set), containing elements that are pending removal. (See the algorithms, below, for details on how this is used.)

<a id="ref-for-document-top-layer①⓪"></a>

<a id="ref-for-pending-top-layer-removals"></a>

The [top layer](#document-top-layer) (and the [pending top layer removals](#pending-top-layer-removals)) should not be interacted with directly by specification algorithms. (Individual features using the <a id="ref-for-document-top-layer①①"></a>top layer might have ownership over various things in the top layer, like a popover inside of a popover, that need to be reordered or moved as a group.) Instead, specifications should use the following algorithms.

### <a id="top-styling"></a>3.1. Top Layer Styling

<a id="ref-for-render-in-the-top-layer"></a>

<a id="ref-for-selectordef-backdrop"></a>

Every element [rendered in the top layer](#render-in-the-top-layer), as well as its corresponding [::backdrop](#selectordef-backdrop) pseudo-element, are rendered with the following qualities:

- <a id="ref-for-x43"></a>

  It generates a new [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

- Its parent stacking context is the root stacking context.

- It is rendered as an atomic unit as if it were a sibling of the document’s root.

  <a id="ref-for-propdef-overflow"></a>

  <a id="ref-for-propdef-opacity"></a>

  <a id="ref-for-propdef-mask"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Ancestor elements with [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask), etc. cannot affect it.

- <a id="ref-for-propdef-position"></a>

  <a id="ref-for-valdef-position-fixed"></a>

  If its [position](https://www.w3.org/TR/css-position-3/#propdef-position) property computes to [fixed](https://www.w3.org/TR/css-position-3/#valdef-position-fixed), its containing block is the viewport; otherwise, it’s the initial containing block.

- <a id="ref-for-selectordef-backdrop①"></a>

  <a id="ref-for-concept-shadow-including-inclusive-ancestor"></a>

  <a id="ref-for-propdef-display"></a>

  If it is an element, it and its [::backdrop](#selectordef-backdrop) pseudo-element are not rendered if its [shadow-including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) has the [display: none](https://www.w3.org/TR/css-display-4/#propdef-display).

- <a id="ref-for-propdef-display①"></a>

  <a id="ref-for-valdef-display-contents"></a>

  <a id="ref-for-valdef-display-block"></a>

  If its specified [display](https://www.w3.org/TR/css-display-4/#propdef-display) property is [contents](https://www.w3.org/TR/css-display-4/#valdef-display-contents), it computes to [block](https://www.w3.org/TR/css-display-4/#valdef-display-block).

- <a id="ref-for-propdef-position①"></a>

  <a id="ref-for-valdef-position-absolute"></a>

  <a id="ref-for-valdef-position-fixed①"></a>

  If its specified [position](https://www.w3.org/TR/css-position-3/#propdef-position) property is not [absolute](https://www.w3.org/TR/css-position-3/#valdef-position-absolute) or [fixed](https://www.w3.org/TR/css-position-3/#valdef-position-fixed), it computes to <a id="ref-for-valdef-position-absolute①"></a>absolute.

- Unless overridden by another specification, its static position for left, right, and top is zero.

<a id="ref-for-selectordef-backdrop②"></a>

### <a id="backdrop"></a>3.2. The [::backdrop](#selectordef-backdrop) Pseudo-Element

<a id="ref-for-render-in-the-top-layer①"></a>

<a id="ref-for-originating-element"></a>

Each element [rendered in the top layer](#render-in-the-top-layer) has a <a id="selectordef-backdrop"></a>::backdrop pseudo-element, for which it is the [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

<a id="ref-for-propdef-content"></a>

<a id="ref-for-valdef-content-none"></a>

<a id="ref-for-selectordef-backdrop③"></a>

<a id="ref-for-document-top-layer①②"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-paint-a-document"></a>

When its computed [content](https://www.w3.org/TR/css-content-3/#propdef-content) value is not [none](https://www.w3.org/TR/css-content-3/#valdef-content-none), [::backdrop](#selectordef-backdrop) pseudo-elements generate boxes as if they were siblings of the root element. They’re automatically rendered as a separate item in the [top layer](#document-top-layer), below their [originating element](https://www.w3.org/TR/selectors-4/#originating-element). (See "[paint a document](#paint-a-document)" for details.)

<a id="ref-for-selectordef-backdrop④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [::backdrop](#selectordef-backdrop) pseudo-element can be used to create a backdrop that hides the underlying document for an element in a top layer (such as an element that is displayed fullscreen).

<a id="ref-for-selectordef-backdrop⑤"></a>

<a id="ref-for-fully-styleable"></a>

The [::backdrop](#selectordef-backdrop) pseudo-element is a [fully styleable pseudo-element](https://www.w3.org/TR/css-pseudo-4/#fully-styleable).

User agents should contain the following rules in a UA-level style sheet:

```text
::backdrop {
  position: fixed;
  inset: 0;
}
```
<a id="ref-for-selectordef-backdrop⑥"></a>

Other specifications can additional properties to the default [::backdrop](#selectordef-backdrop) rendering.

<a id="ref-for-selectordef-backdrop⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For example, a fullscreen element (see [\[FULLSCREEN\]](#biblio-fullscreen)) styles its [::backdrop](#selectordef-backdrop) as opaque black by default.

<a id="ref-for-selectordef-backdrop⑧"></a>

See [§ 3.1 Top Layer Styling](#top-styling) for additional details on how [::backdrop](#selectordef-backdrop) elements are rendered.

### <a id="top-manip"></a>3.3. Top Layer Manipulation

<a id="ref-for-list-contain"></a>

<a id="ref-for-concept-node-document"></a>

<a id="ref-for-document-top-layer①③"></a>

<a id="ref-for-pending-top-layer-removals①"></a>

An element <var>el</var> is <a id="in-the-top-layer"></a>in the top layer if <var>el</var> is [contained](https://infra.spec.whatwg.org/#list-contain) in its [node document’s](https://dom.spec.whatwg.org/#concept-node-document) [top layer](#document-top-layer) but <em>not</em> <a id="ref-for-list-contain①"></a>contained in its <a id="ref-for-concept-node-document①"></a>node document’s [pending top layer removals](#pending-top-layer-removals).

<a id="ref-for-render-in-the-top-layer②"></a>

<a id="ref-for-propdef-overlay②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specs should use this concept, rather than [rendered in the top layer](#render-in-the-top-layer), when they are manipulating the top layer itself. Using this concept avoids the behavior being different based on whether there’s an [overlay](#propdef-overlay) transition, or whether two operations happened <em>between</em> rendering updates or <em>across</em> them.

<a id="ref-for-list-contain②"></a>

<a id="ref-for-concept-node-document②"></a>

<a id="ref-for-document-top-layer①④"></a>

<a id="ref-for-propdef-overlay③"></a>

An element <var>el</var> is <a id="render-in-the-top-layer"></a>rendered in the top layer if <var>el</var> is [contained](https://infra.spec.whatwg.org/#list-contain) in its [node document’s](https://dom.spec.whatwg.org/#concept-node-document) [top layer](#document-top-layer), and <var>el</var> has [overlay: auto](#propdef-overlay).

<a id="ref-for-in-the-top-layer"></a>

<a id="ref-for-selectordef-backdrop⑨"></a>

<a id="ref-for-render-in-the-top-layer③"></a>

<a id="ref-for-pending-top-layer-removals②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specs should use this concept, rather than [in the top layer](#in-the-top-layer), when they are not manipulating the top layer itself, but rather responding to the rendering behavior of being "on top of everything". For example, the presence of a [::backdrop](#selectordef-backdrop) pseudo relies on the element being [rendered in the top layer](#render-in-the-top-layer); even if the element is [pending removal](#pending-top-layer-removals), it has a <a id="ref-for-selectordef-backdrop①⓪"></a>::backdrop as long as it’s being displayed on top of everything.

<a id="ref-for-element"></a>

To <a id="add-an-element-to-the-top-layer"></a>add an element to the top layer, given an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> <var>el</var>:

1.  <a id="ref-for-concept-node-document③"></a>

    Let <var>doc</var> be <var>el</var>’s [node document](https://dom.spec.whatwg.org/#concept-node-document).

2.  <a id="ref-for-list-contain③"></a>

    <a id="ref-for-document-top-layer①⑤"></a>

    If <var>el</var> is already [contained](https://infra.spec.whatwg.org/#list-contain) in <var>doc</var>’s [top layer](#document-top-layer):

    - <a id="ref-for-pending-top-layer-removals③"></a>

      Assert: <var>el</var> is also in <var>doc</var>’s [pending top layer removals](#pending-top-layer-removals). (Otherwise, this is a spec error.)

    - <a id="ref-for-list-remove"></a>

      <a id="ref-for-document-top-layer①⑥"></a>

      <a id="ref-for-pending-top-layer-removals④"></a>

      [Remove](https://infra.spec.whatwg.org/#list-remove) <var>el</var> from both <var>doc</var>’s [top layer](#document-top-layer) and [pending top layer removals](#pending-top-layer-removals).

3.  <a id="ref-for-set-append"></a>

    <a id="ref-for-document-top-layer①⑦"></a>

    [Append](https://infra.spec.whatwg.org/#set-append) <var>el</var> to <var>doc</var>’s [top layer](#document-top-layer).

4.  <a id="ref-for-origin"></a>

    <a id="ref-for-propdef-overlay④"></a>

    At the UA !important [cascade origin](https://www.w3.org/TR/css-cascade-6/#origin), add a rule targeting <var>el</var> containing an [overlay: auto](#propdef-overlay) declaration.

<a id="ref-for-element①"></a>

To <a id="request-an-element-to-be-removed-from-the-top-layer"></a>request an element to be removed from the top layer, given an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> <var>el</var>:

1.  <a id="ref-for-concept-node-document④"></a>

    Let <var>doc</var> be <var>el</var>’s [node document](https://dom.spec.whatwg.org/#concept-node-document).

2.  <a id="ref-for-list-contain④"></a>

    <a id="ref-for-document-top-layer①⑧"></a>

    <a id="ref-for-pending-top-layer-removals⑤"></a>

    If <var>el</var> is not [contained](https://infra.spec.whatwg.org/#list-contain) <var>doc</var>’s [top layer](#document-top-layer), or <var>el</var> <em>is</em> already <a id="ref-for-list-contain⑤"></a>contained in <var>doc</var>’s [pending top layer removals](#pending-top-layer-removals), return.

3.  <a id="ref-for-propdef-overlay⑤"></a>

    Remove the UA !important [overlay: auto](#propdef-overlay) rule targeting <var>el</var>.

4.  <a id="ref-for-set-append①"></a>

    <a id="ref-for-pending-top-layer-removals⑥"></a>

    [Append](https://infra.spec.whatwg.org/#set-append) <var>el</var> to <var>doc</var>’s [pending top layer removals](#pending-top-layer-removals).

<a id="ref-for-element②"></a>

To <a id="remove-an-element-from-the-top-layer-immediately"></a>remove an element from the top layer immediately, given an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> <var>el</var>:

1.  <a id="ref-for-concept-node-document⑤"></a>

    Let <var>doc</var> be <var>el</var>’s [node document](https://dom.spec.whatwg.org/#concept-node-document).

2.  <a id="ref-for-list-remove①"></a>

    <a id="ref-for-document-top-layer①⑨"></a>

    <a id="ref-for-pending-top-layer-removals⑦"></a>

    [Remove](https://infra.spec.whatwg.org/#list-remove) <var>el</var> from <var>doc</var>’s [top layer](#document-top-layer) and [pending top layer removals](#pending-top-layer-removals).

3.  <a id="ref-for-propdef-overlay⑥"></a>

    Remove the UA !important [overlay: auto](#propdef-overlay) rule targeting <var>el</var>, if it exists.

<a id="ref-for-propdef-overlay⑦"></a>

<a id="ref-for-request-an-element-to-be-removed-from-the-top-layer"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm is only intended to be used in special cases where removing something from the top layer immediately (bypassing things like an [overlay](#propdef-overlay) transition) is necessary, such as a modal dialog that is removed from the document. Most of the time, [requesting removal from the top layer](#request-an-element-to-be-removed-from-the-top-layer) is more appropriate.

<a id="ref-for-document②"></a>

To <a id="process-top-layer-removals"></a>process top layer removals, given a <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> <var>doc</var>:

1.  <a id="ref-for-pending-top-layer-removals⑧"></a>

    <a id="ref-for-propdef-overlay⑧"></a>

    <a id="ref-for-valdef-overlay-none"></a>

    <a id="ref-for-remove-an-element-from-the-top-layer-immediately"></a>

    For each element <var>el</var> in <var>doc</var>’s [pending top layer removals](#pending-top-layer-removals): if <var>el</var>'s computed value of [overlay](#propdef-overlay) is [none](#valdef-overlay-none), or <var>el</var> is <u>not rendered</u>, [remove from the top layer immediately](#remove-an-element-from-the-top-layer-immediately) <var>el</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is intended to be called during the "Update the Rendering" step of HTML’s rendering algorithm. It is not intended to be called by other algorithms.

<a id="ref-for-propdef-overlay⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [overlay](#propdef-overlay) check can be delayed arbitrarily long by author-level transitions; see [§ 3.4 Controlling the Top Layer: the overlay property](#overlay) for details.

<a id="ref-for-propdef-overlay①⓪"></a>

### <a id="overlay"></a>3.4. Controlling the Top Layer: the [overlay](#propdef-overlay) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-overlay"></a>overlay

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

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

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see prose

<a id="ref-for-in-the-top-layer①"></a>

<a id="ref-for-propdef-overlay①①"></a>

<a id="ref-for-render-in-the-top-layer④"></a>

When an element is [in the top layer](#in-the-top-layer), the [overlay](#propdef-overlay) property determines whether it is actually [rendered in the top layer](#render-in-the-top-layer) or not.

<a id="valdef-overlay-none"></a>none  
<a id="ref-for-render-in-the-top-layer⑤"></a>

The element isn’t [rendered in the top layer](#render-in-the-top-layer).

<a id="valdef-overlay-auto"></a>auto  
<a id="ref-for-render-in-the-top-layer⑥"></a>

<a id="ref-for-in-the-top-layer②"></a>

The element is [rendered in the top layer](#render-in-the-top-layer) if it is [in the top layer](#in-the-top-layer).

Rather than generating boxes as part of its normal position in the document, it generates boxes as a sibling of the root element, rendered "above" it.

<a id="ref-for-propdef-overlay①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="overlay-guidance"></a> Note: [overlay](#propdef-overlay) is a somewhat unusual property, as it is <em>only</em> set by the user agent, and can’t be set by authors <em>at all</em>.
>
> <a id="ref-for-propdef-overlay①③"></a>
>
> <a id="ref-for-propdef-transition"></a>
>
> However, authors <em>do</em> have the ability to affect <em>when</em> [overlay](#propdef-overlay) changes its value, by setting a [transition](https://www.w3.org/TR/css-transitions-1/#propdef-transition) on the property. This allows an author to align an animation with the transition, with the element moving in or out of the top layer only at the desired point in the animation. This allows, for example, an animation that causes an element to fade out of its normal position on the page, then fade in at its new top-layer position, or vice versa.

<a id="ref-for-valdef-overlay-auto"></a>

<a id="ref-for-interpolation"></a>

For animation, [auto](#valdef-overlay-auto) is [interpolated](https://www.w3.org/TR/css-values-4/#interpolation) as a discrete step where values of p such that `0 < p < 1` map to <a id="ref-for-valdef-overlay-auto①"></a>auto and other values of p map to the closer endpoint; if neither value is <a id="ref-for-valdef-overlay-auto②"></a>auto then discrete animation is used.

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-easing-function"></a>

<a id="ref-for-render-in-the-top-layer⑦"></a>

<a id="ref-for-valdef-step-easing-function-step-start"></a>

<a id="ref-for-valdef-step-easing-function-step-end"></a>

<a id="ref-for-funcdef-linear"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is similar to how [visibility](https://www.w3.org/TR/css-display-4/#propdef-visibility) animates. With most [easing functions](https://www.w3.org/TR/css-easing-2/#easing-function), this will keep the element [rendered in the top layer](#render-in-the-top-layer) for the entire duration of the transition, whether it’s entering or leaving the top layer. [step-start](https://www.w3.org/TR/css-easing-2/#valdef-step-easing-function-step-start)/[step-end](https://www.w3.org/TR/css-easing-2/#valdef-step-easing-function-step-end)/[linear()](https://www.w3.org/TR/css-easing-2/#funcdef-linear) can be used to control when the value flips more precisely.

User agents must have the following rule in their UA stylesheet:

```text
* { overlay: none !important; }
```
<a id="ref-for-propdef-overlay①④"></a>

<a id="ref-for-document-top-layer②⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that the [overlay](#propdef-overlay) property <em>cannot be set by authors or users</em>—​it is entirely controlled by the user agent (which sets elements to <a id="ref-for-propdef-overlay①⑤"></a>overlay: auto, via another UA-!important rule, when they’re in the [top layer](#document-top-layer)).

<a id="ref-for-transitions"></a>

<a id="ref-for-propdef-overlay①⑥"></a>

<a id="ref-for-propdef-transition①"></a>

<a id="ref-for-document-top-layer②①"></a>

User agents <em>may</em>, at their discretion, remove a running [transition](https://drafts.csswg.org/css-transitions-1/#transitions) on [overlay](#propdef-overlay). The conditions for this are intentionally undefined. <strong data-conversion-semantic="note">Note:</strong> (This is to prevent potential abuse scenarios where a [transition: overlay 1e9s;](https://www.w3.org/TR/css-transitions-1/#propdef-transition) or similar attempts to keep an element in the [top layer](#document-top-layer) permanently.)

## <a id="painting-order"></a>4. Painting Order and Stacking Contexts

<a id="ref-for-box-tree"></a>

This chapter describes the painting order of CSS’s [box tree](https://www.w3.org/TR/css-display-4/#box-tree).

<a id="ref-for-box-tree①"></a>

<a id="ref-for-concept-tree-order"></a>

<a id="ref-for-fragment"></a>

When traversing the [box tree](https://www.w3.org/TR/css-display-4/#box-tree), [tree order](https://dom.spec.whatwg.org/#concept-tree-order) is often used. For [fragments](https://www.w3.org/TR/css-break-4/#fragment), this refers to the logical order of the fragments, not the visual order. (This can be relevant, for example, when rending bidirectional text.)

Painting order is defined in terms of a "painter’s model", where elements are described as painting in a stack, with the bottom of the stack rendered "first", below items higher in the stack. The user is implied to exist above the top of the stack, looking down:

```text
             |     |         |
             |          |    |   ⇦ 🧑‍🎨
             |          |        user
z-index:  canvas  -1    0    1
```
The stacking context background and most negative positioned stacking contexts are at the bottom of the stack, while the most positive positioned stacking contexts are at the top of the stack.

The canvas is transparent if contained within another, and given a UA-defined color if it is not. It is infinite in extent and contains the root element. Initially, the viewport is anchored with its top left corner at the canvas origin.

To <a id="paint-a-document"></a>paint a document, given a document <var>doc</var> and an infinite canvas <var>canvas</var>:

1.  <a id="ref-for-paint-a-stacking-context"></a>

    [Paint a stacking context](#paint-a-stacking-context) given <var>doc</var>’s root element and <var>canvas</var>.

2.  <a id="ref-for-document-top-layer②②"></a>

    For each element <var>el</var> in <var>doc</var>’s [top layer](#document-top-layer):

    1.  <a id="ref-for-paint-a-stacking-context①"></a>

        <a id="ref-for-selectordef-backdrop①①"></a>

        [Paint a stacking context](#paint-a-stacking-context) given <var>el</var>'s [::backdrop](#selectordef-backdrop) pseudo-element and <var>canvas</var>.

    2.  <a id="ref-for-paint-a-stacking-context②"></a>

        <a id="ref-for-x43①"></a>

        <a id="ref-for-initial-containing-block②"></a>

        <a id="ref-for-containing-block②"></a>

        [Paint a stacking context](#paint-a-stacking-context) given <var>el</var> and <var>canvas</var>, treating <var>el</var> as a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43), with the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) as its [containing block](https://www.w3.org/TR/css-display-4/#containing-block).

<a id="ref-for-concept-element①"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-box②"></a>

To <a id="paint-a-stacking-context"></a>paint a stacking context given an [element](https://dom.spec.whatwg.org/#concept-element), [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element), or [box](https://www.w3.org/TR/css-display-4/#box) <var>root</var>, and an infinite canvas <var>canvas</var>:

1.  <a id="ref-for-concept-element②"></a>

    <a id="ref-for-paint-a-stacking-context③"></a>

    <a id="ref-for-principal-box"></a>

    If <var>root</var> is an [element](https://dom.spec.whatwg.org/#concept-element), [paint a stacking context](#paint-a-stacking-context) given <var>root</var>’s [principal box](https://www.w3.org/TR/css-display-4/#principal-box) and <var>canvas</var>, then return.

2.  <a id="ref-for-box③"></a>

    <a id="ref-for-x43②"></a>

    Assert: <var>root</var> is a [box](https://www.w3.org/TR/css-display-4/#box), and generates a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

3.  <a id="ref-for-root-element①"></a>

    <a id="ref-for-principal-box①"></a>

    If <var>root</var> is a [root element’s](https://www.w3.org/TR/css-display-4/#root-element) [principal box](https://www.w3.org/TR/css-display-4/#principal-box), paint <var>root</var>’s background over the entire <var>canvas</var>, with the origin of the background positioning area being the position on <var>canvas</var> that would be used if <var>root</var>’s background was being painted normally.

4.  <a id="ref-for-block-level-box"></a>

    <a id="ref-for-paint-a-blocks-decorations"></a>

    If <var>root</var> is a [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box), [paint a block’s decorations](#paint-a-blocks-decorations) given <var>root</var> and <var>canvas</var>.

5.  <a id="ref-for-propdef-z-index"></a>

    <a id="ref-for-concept-tree-order①"></a>

    <a id="ref-for-paint-a-stacking-context④"></a>

    For each of <var>root</var>’s positioned descendants with negative (non-zero) [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) values, sort those descendants by <a id="ref-for-propdef-z-index①"></a>z-index order (most negative first) then [tree order](https://dom.spec.whatwg.org/#concept-tree-order), and [paint a stacking context](#paint-a-stacking-context) given each descendant and <var>canvas</var>.

6.  <a id="ref-for-concept-tree-order②"></a>

    <a id="ref-for-paint-a-blocks-decorations①"></a>

    For each of <var>root</var>’s in-flow, non-positioned, block-level descendants, in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), [paint a block’s decorations](#paint-a-blocks-decorations) given the descendant and <var>canvas</var>.

7.  <a id="ref-for-paint-a-stacking-container"></a>

    For each of <var>root</var>’s non-positioned floating descendants, in tree order, [paint a stacking container](#paint-a-stacking-container) given the descendant and <var>canvas</var>.

8.  <a id="ref-for-inline-level"></a>

    If <var>root</var> is an [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) box

    <a id="ref-for-paint-a-box-in-a-line-box"></a>

    For each line box <var>root</var> is in, [paint a box in a line box](#paint-a-box-in-a-line-box) given <var>root</var>, the line box, and <var>canvas</var>.

    Otherwise

    <a id="ref-for-block-level"></a>

    <a id="ref-for-concept-tree-order③"></a>

    First for <var>root</var>, then for all its in-flow, non-positioned, [block-level](https://www.w3.org/TR/css-display-4/#block-level) descendant boxes, in [tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    1.  <a id="ref-for-replaced-element"></a>

        If the box is a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element), paint the replaced content into <var>canvas</var>, atomically.

    2.  <a id="ref-for-paint-a-box-in-a-line-box①"></a>

        Otherwise, for each line box of the box, [paint a box in a line box](#paint-a-box-in-a-line-box) given the box, the line box, and <var>canvas</var>.

    3.  <a id="ref-for-in-band-outline"></a>

        If the UA uses [in-band outlines](#in-band-outline), paint the outlines of the box into <var>canvas</var>.

9.  <a id="ref-for-propdef-z-index②"></a>

    <a id="ref-for-concept-tree-order④"></a>

    For each of <var>root</var>’s positioned descendants with [z-index: auto](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) or <a id="ref-for-propdef-z-index③"></a>z-index: 0, in [tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    <a id="ref-for-propdef-z-index④"></a>

    descendant has [z-index: auto](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index)

    <a id="ref-for-paint-a-stacking-container①"></a>

    [Paint a stacking container](#paint-a-stacking-container) given the descendant and <var>canvas</var>.

    <a id="ref-for-propdef-z-index⑤"></a>

    descendant has [z-index: 0](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index)

    <a id="ref-for-paint-a-stacking-context⑤"></a>

    [Paint a stacking context](#paint-a-stacking-context) given the descendant and <var>canvas</var>.

10. <a id="ref-for-propdef-z-index⑥"></a>

    <a id="ref-for-concept-tree-order⑤"></a>

    <a id="ref-for-paint-a-stacking-context⑥"></a>

    For each of <var>root</var>’s positioned descendants with positive (non-zero) [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) values, sort those descendants by <a id="ref-for-propdef-z-index⑦"></a>z-index order (smallest first) then [tree order](https://dom.spec.whatwg.org/#concept-tree-order), and [paint a stacking context](#paint-a-stacking-context) given each descendant and <var>canvas</var>.

11. <a id="ref-for-out-of-band-outline"></a>

    <a id="ref-for-in-band-outline①"></a>

    If the UA uses [out-of-band outlines](#out-of-band-outline), draw all of <var>root</var>’s outlines (those that it skipped drawing due to not using [in-band outlines](#in-band-outline) during the current invocation of this algorithm) into <var>canvas</var>.

To <a id="paint-a-blocks-decorations"></a>paint a block’s decorations given a block box <var>root</var> and a canvas <var>canvas</var>:

1.  <a id="ref-for-table-wrapper-box"></a>

    If <var>root</var> is not a [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box):

    1.  <a id="ref-for-root-element②"></a>

        <a id="ref-for-principal-box②"></a>

        Paint <var>root</var>’s background to <var>canvas</var> if it is not the [root element’s](https://www.w3.org/TR/css-display-4/#root-element) [principal box](https://www.w3.org/TR/css-display-4/#principal-box).

    2.  Paint <var>root</var>’s border to <var>canvas</var>.

2.  <a id="ref-for-table-wrapper-box①"></a>

    If <var>root</var> is a [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box):

    1.  <a id="ref-for-root-element③"></a>

        <a id="ref-for-principal-box③"></a>

        Paint <var>root</var>’s background to <var>canvas</var> if it is not the [root element’s](https://www.w3.org/TR/css-display-4/#root-element) [principal box](https://www.w3.org/TR/css-display-4/#principal-box).

    2.  <a id="ref-for-concept-tree-order⑥"></a>

        For each column group of <var>root</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), paint the column group’s background to <var>canvas</var>.

    3.  <a id="ref-for-concept-tree-order⑦"></a>

        For each column of <var>root</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), paint the column’s background to <var>canvas</var>.

    4.  <a id="ref-for-concept-tree-order⑧"></a>

        For each row group of <var>root</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), paint the row group’s background to <var>canvas</var>.

    5.  <a id="ref-for-concept-tree-order⑨"></a>

        For each row of <var>root</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), paint the row’s background to <var>canvas</var>.

    6.  <a id="ref-for-concept-tree-order①⓪"></a>

        For each cell of <var>root</var> in [tree order](https://dom.spec.whatwg.org/#concept-tree-order), paint the cell’s background to <var>canvas</var>.

    7.  <a id="ref-for-concept-tree-order①①"></a>

        Paint the borders of all of the table elements of <var>root</var>. If the borders are separated, do so in [tree order](https://dom.spec.whatwg.org/#concept-tree-order); if connected, do so as specified in [\[css-tables-3\]](#biblio-css-tables-3).

To <a id="paint-a-box-in-a-line-box"></a>paint a box in a line box, given a box <var>root</var>, a line box <var>line box</var>, and a canvas <var>canvas</var>:

1.  <a id="ref-for-fragment①"></a>

    Paint the backgrounds of <var>root</var>’s [fragments](https://www.w3.org/TR/css-break-4/#fragment) that are in <var>line box</var> into <var>canvas</var>.

2.  <a id="ref-for-fragment②"></a>

    Paint the borders of <var>root</var>’s [fragments](https://www.w3.org/TR/css-break-4/#fragment) that are in <var>line box</var> into <var>canvas</var>.

3.  <a id="ref-for-inline-box"></a>

    If <var>root</var> is an [inline box](https://www.w3.org/TR/css-display-4/#inline-box)

    <a id="ref-for-fragment③"></a>

    <a id="ref-for-css-text-sequence"></a>

    <a id="ref-for-concept-tree-order①②"></a>

    For all <var>root</var>’s in-flow, non-positioned, inline-level children that generate [fragments](https://www.w3.org/TR/css-break-4/#fragment) in <var>line box</var>, and all child [text sequences](https://www.w3.org/TR/css-display-4/#css-text-sequence) that generate <a id="ref-for-fragment④"></a>fragments in <var>line box</var>, in [tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    <a id="ref-for-css-text-sequence①"></a>

    If this child is a [text sequence](https://www.w3.org/TR/css-display-4/#css-text-sequence), then:

    1.  Paint any underlining affecting the text, in tree order of the elements applying the underlining (such that the deepest element’s underlining, if any, is painted topmost and the root element’s underlining, if any, is drawn bottommost) into <var>canvas</var>.

    2.  Paint any overlining affecting the text, in tree order of the elements applying the overlining (such that the deepest element’s overlining, if any, is painted topmost and the root element’s overlining, if any, is drawn bottommost) into <var>canvas</var>.

    3.  Paint the text into <var>canvas</var>.

    4.  Paint any line-through affecting the text, in tree order of the elements applying the line-through (such that the deepest element’s line-through, if any, is painted topmost and the root element’s line-through, if any, is drawn bottommost) into <var>canvas</var>.

    <a id="ref-for-box④"></a>

    If this child is a [box](https://www.w3.org/TR/css-display-4/#box):

    <a id="ref-for-paint-a-box-in-a-line-box②"></a>

    [Paint a box in a line box](#paint-a-box-in-a-line-box) given the child, <var>line box</var>, and <var>canvas</var>.

    <a id="ref-for-table-wrapper-box②"></a>

    <a id="ref-for-block-box"></a>

    <a id="ref-for-inline-level①"></a>

    If <var>root</var> is an [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) [block](https://www.w3.org/TR/css-display-4/#block-box) or [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box)

    <a id="ref-for-paint-a-stacking-container②"></a>

    [Paint a stacking container](#paint-a-stacking-container) given <var>root</var> and <var>canvas</var>.

    <a id="ref-for-inline-level②"></a>

    If <var>root</var> is an [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) replaced element

    Paint the replaced content into <var>canvas</var>, atomically.

4.  <a id="ref-for-in-band-outline②"></a>

    <a id="ref-for-fragment⑤"></a>

    If the UA uses [in-band outlines](#in-band-outline), paint the outlines of <var>root</var>’s [fragments](https://www.w3.org/TR/css-break-4/#fragment) that are in <var>line box</var> into <var>canvas</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even though the visual order of inline box fragments can be rearranged within a line due to [bidirectional reordering](https://www.w3.org/TR/css-writing-modes-3/#bidi-box-model), the fragments are nevertheless painted in tree order.

<a id="ref-for-box⑤"></a>

To <a id="paint-a-stacking-container"></a>paint a stacking container, given a [box](https://www.w3.org/TR/css-display-4/#box) <var>root</var> and a canvas <var>canvas</var>:

1.  <a id="ref-for-paint-a-stacking-context⑦"></a>

    [Paint a stacking context](#paint-a-stacking-context) given <var>root</var> and <var>canvas</var>, treating <var>root</var> as if it created a new stacking context, but omitting any positioned descendants or descendants that actually create a stacking context (letting the parent stacking context paint them, instead).

<a id="ref-for-propdef-outline"></a>

<a id="ref-for-out-of-band-outline①"></a>

UAs can draw outlines (from the [outline](https://www.w3.org/TR/css-ui-4/#propdef-outline) property) either <a id="in-band-outline"></a>in-band (painted along each element, and thus potentially obscured/overlapping by following content) or <a id="out-of-band-outline"></a>out-of-band (all outlines painted at the end of the stacking context, so nothing in the stacking context can obscure them). It is recommended that UAs use [out-of-band outlines](#out-of-band-outline), as making outlines easily visible is an important accessibility feature.

## <a id="changes"></a>5. Changes

Significant changes since the [8 July 2025 First Public Working Draft](https://www.w3.org/TR/2025/WD-css-position-4-20250708/):

- None (markup and linking fixes only).

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

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

- [add an element to the top layer](#add-an-element-to-the-top-layer), in § 3.3
- [add to the top layer](#add-an-element-to-the-top-layer), in § 3.3
- [auto](#valdef-overlay-auto), in § 3.4
- [::backdrop](#selectordef-backdrop), in § 3.2
- [fixed containing block](#fixed-containing-block), in § 2
- [in a higher top layer](#in-a-higher-top-layer), in § 3
- [in a lower top layer](#in-a-higher-top-layer), in § 3
- [in-band outline](#in-band-outline), in § 4
- [in the same top layer](#in-the-same-top-layer), in § 3
- [in the top layer](#in-the-top-layer), in § 3.3
- [local containing block](#local-containing-block), in § 2
- [none](#valdef-overlay-none), in § 3.4
- [out-of-band outline](#out-of-band-outline), in § 4
- [overlay](#propdef-overlay), in § 3.4
- [paint a block’s decorations](#paint-a-blocks-decorations), in § 4
- [paint a box in a line box](#paint-a-box-in-a-line-box), in § 4
- [paint a document](#paint-a-document), in § 4
- [paint a stacking container](#paint-a-stacking-container), in § 4
- [paint a stacking context](#paint-a-stacking-context), in § 4
- [pending top layer removals](#pending-top-layer-removals), in § 3
- [process top layer removals](#process-top-layer-removals), in § 3.3
- [remove an element from the top layer immediately](#remove-an-element-from-the-top-layer-immediately), in § 3.3
- [remove from the top layer immediately](#remove-an-element-from-the-top-layer-immediately), in § 3.3
- [render in the top layer](#render-in-the-top-layer), in § 3.3
- [request an element to be removed from the top layer](#request-an-element-to-be-removed-from-the-top-layer), in § 3.3
- [request removal from the top layer](#request-an-element-to-be-removed-from-the-top-layer), in § 3.3
- [scrollable containing block](#scrollable-containing-block), in § 2
- [top layer](#document-top-layer), in § 3
- [top layer root](#top-layer-root), in § 3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANCHOR-POSITION-1\] defines the following terms:
  - <a id="1a417197"></a>none
  - <a id="5013f3ce"></a>position-area
- \[CSS-BOX-4\] defines the following terms:
  - <a id="0778a939"></a>margin box
  - <a id="a2be8c84"></a>padding
  - <a id="15e1e804"></a>padding box
  - <a id="093a0ff1"></a>padding edge
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="64bdec0d"></a>fragment
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="9f07bd0d"></a>cascade origin
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="3b7558dc"></a>opacity
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="f3e8378c"></a>content
  - <a id="85ca4a34"></a>none
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="3df5b022"></a>block
  - <a id="45f9eae9"></a>block box
  - <a id="a015488b"></a>block-level
  - <a id="91b1f11d"></a>block-level box
  - <a id="95bf6f06"></a>box
  - <a id="2a8247fa"></a>box tree
  - <a id="0923db9e"></a>containing block
  - <a id="cc63eecd"></a>contents
  - <a id="e8c16097"></a>display
  - <a id="d1ebdd75"></a>initial containing block
  - <a id="f089a6e1"></a>inline box
  - <a id="6b9bba07"></a>inline-level
  - <a id="7a605ac8"></a>principal box
  - <a id="a9db5d6d"></a>replaced element
  - <a id="143ef105"></a>root element
  - <a id="e4657f7f"></a>text sequence
  - <a id="aceda213"></a>visibility
- \[CSS-EASING-2\] defines the following terms:
  - <a id="a2a5dbcf"></a>easing function
  - <a id="b50b6ff3"></a>linear()
  - <a id="879c6025"></a>step-end
  - <a id="fce1b4b3"></a>step-start
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="5b43edf4"></a>mask
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
  - <a id="e3488cd0"></a>scrollable overflow
  - <a id="3ed7991e"></a>scrollable overflow area
  - <a id="700ea31d"></a>scrollport
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="415bbc47"></a>absolute
  - <a id="4cf3c57e"></a>absolute position
  - <a id="8bf8e632"></a>absolute positioning containing block
  - <a id="dec20430"></a>absolutely positioned box
  - <a id="2c8d43ae"></a>fixed
  - <a id="0a872ef1"></a>fixed position
  - <a id="1febb260"></a>fixed positioning containing block
  - <a id="ebcabeea"></a>fixed-positioned box
  - <a id="b8c34db8"></a>position
  - <a id="0363c82a"></a>sticky position
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="5c9e608d"></a>fully styleable pseudo-elements
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="1b178ec1"></a>table wrapper box
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="d3706df0"></a>transition
  - <a id="9523276c"></a>transitions
- \[CSS-UI-4\] defines the following terms:
  - <a id="4e2aade2"></a>outline
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c2e11f7b"></a>interpolate
  - <a id="4eb9d37e"></a>\|
- \[CSS2\] defines the following terms:
  - <a id="f285fdb1"></a>relative positioning
  - <a id="b8c9eb8a"></a>stacking context
  - <a id="1848f1d3"></a>z-index
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document
  - <a id="296f3551"></a>Element
  - <a id="27d9b7ea"></a>element
  - <a id="5216e1a0"></a>node document
  - <a id="3d6a3d36"></a>shadow-including inclusive ancestor
  - <a id="25538777"></a>tree order
- \[INFRA\] defines the following terms:
  - <a id="a3b18719"></a>append
  - <a id="ae8def21"></a>contain
  - <a id="692595fe"></a>ordered set
  - <a id="99c988d6"></a>remove
  - <a id="6d19ac93"></a>user agent
- \[SELECTORS-4\] defines the following terms:
  - <a id="7b5d8638"></a>originating element
  - <a id="4d06fa38"></a>pseudo-element

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-anchor-position-1"></a>\[CSS-ANCHOR-POSITION-1\]  
Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick. [CSS Anchor Positioning](https://www.w3.org/TR/css-anchor-position-1/). 9 May 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-anchor-position-1&#x2F;](https://www.w3.org/TR/css-anchor-position-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 6 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 11 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
François Remy; Greg Whitworth; David Baron. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 27 July 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

### <a id="informative"></a>Informative References

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-easing-2"></a>\[CSS-EASING-2\]  
[CSS Easing Functions Level 2](https://www.w3.org/TR/css-easing-2/). 29 August 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-2&#x2F;](https://www.w3.org/TR/css-easing-2/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-fullscreen"></a>\[FULLSCREEN\]  
Philip Jägenstedt. [Fullscreen API Standard](https://fullscreen.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fullscreen&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fullscreen.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="property-index"></a>Property Index

<strong>Table 2 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-overlay①⑦"></a>

[overlay](#propdef-overlay)

<strong>Column 2 (data cell):</strong>

none \| auto

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

see prose

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Figure out the exact concepts needed for top layer. They probably want to do something a tiny bit different, specific to their layer. [↵](#issue-806ae415)
