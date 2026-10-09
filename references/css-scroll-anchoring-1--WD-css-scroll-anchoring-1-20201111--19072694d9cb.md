Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Scroll Anchoring Module Level 1](https://www.w3.org/TR/2020/WD-css-scroll-anchoring-1-20201111/).

Original copyright notice: Copyright © 2020 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Scroll Anchoring Module Level 1

Source snapshot: https://www.w3.org/TR/2020/WD-css-scroll-anchoring-1-20201111/

Snapshot SHA-256: 19072694d9cb63f46fcb122d020452401218f4c5e5c9ef576f56bc3ff205455b

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 2 source tables are presented as readable Markdown tables or explicit labeled layouts: 2 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Scroll Anchoring Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2020 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-scrolling-box"></a>

Changes in DOM elements above the visible region of a [scrolling box](https://drafts.csswg.org/cssom-view-1/#scrolling-box) can result in the page moving while the user is in the middle of consuming the content.

This spec proposes a mechanism to mitigate this jarring user experience by keeping track of the position of an anchor node and adjusting the scroll offset accordingly.

This spec also proposes an API for web developers to opt-out of this behavior.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	Other documents may supersede this document.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Working Draft</strong>. Publication as a Working Draft does not imply endorsement by the W3C Membership.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-scroll-anchoring” in the title, like this: “\[css-scroll-anchoring\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/public-css-archive/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-scroll-anchoring%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20170801/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20170801/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20170801/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

Today, users of the web are often distracted by content moving around due to changes that occur outside the viewport. Examples include script inserting an iframe containing an ad, or non-sized images loading on a slow network.

Historically the browser’s default behavior has been to preserve the absolute scroll position when such changes occur. This means that to avoid shifting content, the webpage can attempt to reserve space on the page for anything that will load later. In practice, few websites do this consistently.

Scroll anchoring aims to minimize surprising content shifts. It does this by adjusting the scroll position to compensate for the changes outside the viewport.

The [explainer document](https://github.com/WICG/ScrollAnchoring/blob/master/explainer.md) gives an informal overview of scroll anchoring.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="description"></a>2.  Description

Scroll anchoring attempts to keep the user’s view of the document stable across layout changes. It works by selecting a DOM node (the <a id="scroll-anchoring-anchor-node"></a>anchor node) whose movement is used to determine adjustments to the scroll position.

However, if the scroll container is currently snapped to an element, (see [\[CSS-SCROLL-SNAP-1\]](#biblio-css-scroll-snap-1)) scroll anchoring is limited to adjustments that would be allowed by re-snapping.

### <a id="anchor-node-selection"></a>2.1.  Anchor Node Selection

<a id="ref-for-scrolling-box①"></a>

<a id="ref-for-scroll-anchoring-anchor-node"></a>

<a id="ref-for-optimal-viewing-region"></a>

Each [scrolling box](https://drafts.csswg.org/cssom-view-1/#scrolling-box) aims to select an [anchor node](#scroll-anchoring-anchor-node) that is deep in the DOM and either should be prioritized as an important DOM node or is close to the block start edge of its [optimal viewing region](https://www.w3.org/TR/css-scroll-snap-1/#optimal-viewing-region).

<a id="ref-for-propdef-scroll-padding"></a>

<a id="ref-for-content-area"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the user agent does not support the [scroll-padding](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding) property, the optimal viewing region of the scrolling box is equivalent to its [content area](https://www.w3.org/TR/css-box-4/#content-area).

<a id="ref-for-box"></a>

<a id="ref-for-atomic-inline"></a>

<a id="ref-for-concept-tree-descendant"></a>

<a id="ref-for-scrolling-box②"></a>

An anchor node can be any [box](https://www.w3.org/TR/css-display-3/#box) except one for a non-[atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline). The anchor node is always a [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) of the [scrolling box](https://drafts.csswg.org/cssom-view-1/#scrolling-box). In some cases, a scrolling box may not select any anchor node.

An element <var>C</var> is a <a id="anchor-viable-candidate"></a>viable candidate for becoming a scroll anchor for a scrolling box <var>S</var> if it meets all of the following criteria:

- <a id="ref-for-atomic-inline①"></a>

  <var>C</var> is an element that is not a non-[atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline).

- <a id="ref-for-partially-visible"></a>

  <a id="ref-for-fully-visible"></a>

  <var>C</var> is [partially visible](#partially-visible) or [fully visible](#fully-visible) in <var>S</var>

- <var>C</var> is a descendant of <var>S</var>

- <a id="ref-for-excluded-subtree"></a>

  <var>C</var> is not in an [excluded subtree](#excluded-subtree)

- <a id="ref-for-excluded-subtree①"></a>

  None of the ancestors of <var>C</var> up to <var>S</var> are in an [excluded subtree](#excluded-subtree)

Some elements are considered to be <a id="anchor-priority-candidates"></a>priority candidates for anchor selection:

1.  <a id="ref-for-dom-anchor"></a>

    <a id="ref-for-focused-area-of-the-document"></a>

    The [DOM anchor](https://html.spec.whatwg.org/multipage/interaction.html#dom-anchor) of the [focused area of the document](https://html.spec.whatwg.org/multipage/interaction.html#focused-area-of-the-document).

2.  An element containing the current active selected match of the find-in-page user-agent algorithm. If the match spans multiple elements, then consider only the first such element.

<a id="ref-for-anchor-priority-candidates"></a>

<a id="ref-for-atomic-inline②"></a>

Note that if the [priority candidate](#anchor-priority-candidates) is a non-[atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline) element, then instead consider its nearest ancestor element that is not a non-atomic inline element as the priority candidate.

The <a id="anchoring-algorithm"></a>anchor node selection algorithm for a scrolling box <var>S</var> is as follows:

1.  <a id="ref-for-propdef-overflow-anchor"></a>

    <a id="ref-for-valdef-overflow-anchor-none"></a>

    If <var>S</var> is associated with an element whose computed value of the [overflow-anchor](#propdef-overflow-anchor) property is [none](#valdef-overflow-anchor-none), then do not select an anchor node for <var>S</var>.

2.  <a id="ref-for-anchor-priority-candidates①"></a>

    <a id="ref-for-anchor-viable-candidate"></a>

    Otherwise, for each [priority candidate](#anchor-priority-candidates) <var>PC</var> in order specified, check if <var>PC</var> is a [viable candidate](#anchor-viable-candidate) in <var>S</var>. If so, select it as an anchor node and terminate.

3.  <a id="ref-for-candidate-examination"></a>

    Otherwise, for each DOM child <var>N</var> of the element or document associated with <var>S</var>, perform the [candidate examination algorithm](#candidate-examination) for <var>N</var> in <var>S</var>, and terminate if it selects an anchor node.

The <a id="candidate-examination"></a>candidate examination algorithm for a candidate DOM node <var>N</var> in a scrolling box <var>S</var> is as follows:

1.  <a id="ref-for-excluded-subtree②"></a>

    <a id="ref-for-fully-clipped"></a>

    If <var>N</var> is an [excluded subtree](#excluded-subtree), or if <var>N</var> is [fully clipped](#fully-clipped) in <var>S</var>, then do nothing (<var>N</var> and its descendants are skipped).

2.  <a id="ref-for-fully-visible①"></a>

    If <var>N</var> is [fully visible](#fully-visible) in <var>S</var>, select <var>N</var> as the anchor node.

3.  <a id="ref-for-partially-visible①"></a>

    If N is [partially visible](#partially-visible):

    1.  <a id="ref-for-candidate-examination①"></a>

        For each DOM child <var>C</var> of <var>N</var>, perform the [candidate examination algorithm](#candidate-examination) for <var>C</var> in <var>S</var>, and terminate if it selects an anchor node.

    2.  <a id="ref-for-containing-block"></a>

        <a id="ref-for-candidate-examination②"></a>

        For each absolutely-positioned element <var>A</var> whose [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is <var>N</var>, but whose DOM parent is not <var>N</var>, perform the [candidate examination algorithm](#candidate-examination) for <var>A</var> in <var>S</var>, and terminate if it selects an anchor node.

    3.  Select <var>N</var> as the anchor node. (If this step is reached, no suitable anchor node was found among <var>N</var>’s descendants.)

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Deeper nodes are preferred to minimize the possibility of content changing inside the anchor node but outside the viewport, which would cause visible content to shift without triggering any scroll anchoring adjustment.

Conceptually, a new anchor node is computed for every scrolling box whenever the scroll position of any scrolling box changes. (As a performance optimization, the implementation may wait until the anchor node is needed before computing it.)

A DOM node <var>N</var> is an <a id="excluded-subtree"></a>excluded subtree if it is an element and any of the following conditions holds:

- <a id="ref-for-propdef-display"></a>

  <a id="ref-for-valdef-display-none"></a>

  <var>N</var>’s computed value of the [display](https://www.w3.org/TR/css-display-3/#propdef-display) property is [none](https://www.w3.org/TR/css-display-3/#valdef-display-none).

- <a id="ref-for-propdef-position"></a>

  <a id="ref-for-valdef-position-fixed"></a>

  <var>N</var>’s computed value of the [position](https://www.w3.org/TR/css-position-3/#propdef-position) property is [fixed](https://www.w3.org/TR/css-position-3/#valdef-position-fixed).

- <a id="ref-for-propdef-position①"></a>

  <a id="ref-for-valdef-position-absolute"></a>

  <a id="ref-for-containing-block①"></a>

  <var>N</var>’s computed value of the [position](https://www.w3.org/TR/css-position-3/#propdef-position) property is [absolute](https://www.w3.org/TR/css-position-3/#valdef-position-absolute) and <var>N</var>’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is an ancestor of the scrolling box.

- <a id="ref-for-propdef-overflow-anchor①"></a>

  <a id="ref-for-valdef-overflow-anchor-none①"></a>

  <var>N</var>’s computed value of the [overflow-anchor](#propdef-overflow-anchor) property is [none](#valdef-overflow-anchor-none).

<a id="ref-for-scroll-anchoring-bounding-rect"></a>

<a id="ref-for-optimal-viewing-region①"></a>

A DOM node <var>N</var> is <a id="fully-visible"></a>fully visible in a scrolling box <var>S</var> if <var>N</var>’s [scroll anchoring bounding rect](#scroll-anchoring-bounding-rect) is entirely within the [optimal viewing region](https://www.w3.org/TR/css-scroll-snap-1/#optimal-viewing-region) of <var>S</var>.

<a id="ref-for-scroll-anchoring-bounding-rect①"></a>

<a id="ref-for-optimal-viewing-region②"></a>

A DOM node <var>N</var> is <a id="fully-clipped"></a>fully clipped in a scrolling box <var>S</var> if <var>N</var>’s [scroll anchoring bounding rect](#scroll-anchoring-bounding-rect) is entirely outside the [optimal viewing region](https://www.w3.org/TR/css-scroll-snap-1/#optimal-viewing-region) of <var>S</var>.

<a id="ref-for-fully-visible②"></a>

<a id="ref-for-fully-clipped①"></a>

A DOM node <var>N</var> is <a id="partially-visible"></a>partially visible in a scrolling box <var>S</var> if <var>N</var> is neither [fully visible](#fully-visible) in <var>S</var> nor [fully clipped](#fully-clipped) in <var>S</var>.

<a id="ref-for-scrollable-overflow-rectangle"></a>

The <a id="scroll-anchoring-bounding-rect"></a>scroll anchoring bounding rect of a DOM node <var>N</var> is <var>N</var>’s [scrollable overflow rectangle](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-rectangle).

### <a id="scroll-adjustment"></a>2.2.  Scroll Adjustment

<a id="ref-for-scroll-anchoring-bounding-rect②"></a>

<a id="ref-for-block-flow-direction"></a>

If an anchor node was selected, then when the anchor node moves, the browser computes the previous offset `y0`, and the current offset `y1`, of the block start edge of the anchor node’s [scroll anchoring bounding rect](#scroll-anchoring-bounding-rect), relative to the block start edge of the scrolling content in the [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) of the scroller.

<a id="ref-for-suppression-window"></a>

It then queues an adjustment to the scroll position of `y1 - y0`, in the block flow direction, to be performed at the end of the [suppression window](#suppression-window).

<a id="ref-for-eventdef-document-scroll"></a>

The scroll adjustment is a type of \[\[cssom-view-1#scrolling-events#scrolling\]\] as defined by [\[CSSOM-VIEW\]](#biblio-cssom-view), and generates [scroll events](https://www.w3.org/TR/cssom-view/#eventdef-document-scroll) in the manner described there.

#### <a id="suppression-windows"></a>2.2.1.  Suppression Window

Every movement of an anchor node occurs within a window of time called the <a id="suppression-window"></a>suppression window, defined as follows:

- The suppression window begins at the start of the current iteration of the [HTML Processing Model](https://html.spec.whatwg.org/multipage/webappapis.html#processing-model-8) event loop, or at the end of the most recently completed suppression window, whichever is more recent.

- <a id="ref-for-dom-element-getboundingclientrect"></a>

  The suppression window ends at the end of the current iteration of the [HTML Processing Model](https://html.spec.whatwg.org/multipage/webappapis.html#processing-model-8) event loop, or immediately before the next operation whose result or side effects would differ as a result of a change in the scroll position (for example, an invocation of <code><a href="https://www.w3.org/TR/cssom-view/#dom-element-getboundingclientrect">getBoundingClientRect()</a></code>), whichever comes sooner.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The suppression window boundaries should be incorporated into the HTML standard once the scroll anchoring API is stabilized.

More than one anchor node movement may occur within the same suppression window.

<a id="ref-for-suppression-trigger"></a>

At the end of a suppression window, the user agent performs all scroll adjustments that were queued during the window and not suppressed by any [suppression trigger](#suppression-trigger) during the window.

#### <a id="suppression-triggers"></a>2.2.2.  Suppression Triggers

A <a id="suppression-trigger"></a>suppression trigger is an operation that suppresses the scroll anchoring adjustment for an anchor node movement, if it occurs within the suppression window for that movement. These triggers are:

- Any change to the computed value of any of the following properties, on any element in the path from the anchor node to the scrollable element (or document), inclusive of both:

  - <a id="ref-for-propdef-top"></a>

    <a id="ref-for-propdef-left"></a>

    <a id="ref-for-propdef-right"></a>

    <a id="ref-for-propdef-bottom"></a>

    [top](https://www.w3.org/TR/css-position-3/#propdef-top), [left](https://www.w3.org/TR/css-position-3/#propdef-left), [right](https://www.w3.org/TR/css-position-3/#propdef-right), or [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)

  - <a id="ref-for-propdef-margin"></a>

    [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) or its longhands

  - <a id="ref-for-propdef-padding"></a>

    [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) or its longhands

  - <a id="ref-for-propdef-width"></a>

    <a id="ref-for-propdef-height"></a>

    <a id="ref-for-propdef-min-width"></a>

    <a id="ref-for-propdef-max-width"></a>

    <a id="ref-for-propdef-min-height"></a>

    <a id="ref-for-propdef-max-height"></a>

    [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), [max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width), [min-height](https://www.w3.org/TR/css-sizing-3/#propdef-min-height), or [max-height](https://www.w3.org/TR/css-sizing-3/#propdef-max-height)

  - <a id="ref-for-propdef-position②"></a>

    [position](https://www.w3.org/TR/css-position-3/#propdef-position)

  - <a id="ref-for-propdef-transform"></a>

    [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform)

- <a id="ref-for-propdef-position③"></a>

  Any change to the computed value of the [position](https://www.w3.org/TR/css-position-3/#propdef-position) property on any element within the scrollable element (or document), such that the element becomes or stops being absolutely positioned. Note that this trigger applies regardless of whether the modified element is on the path from the anchor node to the scrollable element.

- The scroll offset of the scrollable element being zero.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Suppression triggers exist for compatibility with existing web content that has negative interactions with scroll anchoring due to shifting content in scroll event handlers.

## <a id="exclusion-api"></a>3.  Exclusion API

<a id="ref-for-propdef-overflow-anchor②"></a>

Scroll anchoring aims to be the default mode of behavior when launched, so that users benefit from it even on legacy content. [overflow-anchor](#propdef-overflow-anchor) can disable scroll anchoring in part or all of a webpage (opt out), or exclude portions of the DOM from the anchor node selection algorithm.

| Field               | Definition                                                                      |
|---------------------|---------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow-anchor"></a>overflow-anchor                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                            |
| <strong>Applies to:&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                               |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                        |

Values are defined as follows:

<a id="valdef-overflow-anchor-auto"></a>auto  
<a id="ref-for-anchoring-algorithm"></a>

Declares that the element is potentially eligible to participate in the [anchor node selection algorithm](#anchoring-algorithm) for any scrolling box created by the element or an ancestor.

<a id="valdef-overflow-anchor-none"></a>none  
<a id="ref-for-anchoring-algorithm①"></a>

Declares that the element and its descendants (that aren’t nested inside of another scrolling element) are <em>not</em> eligible to participate in the [anchor node selection algorithm](#anchoring-algorithm) for any scrolling box created by the element or an ancestor.

<a id="ref-for-propdef-overflow-anchor③"></a>

<a id="ref-for-scroll-container"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is not possible to turn scroll anchoring "back on" for descendants of a [overflow-anchor: none](#propdef-overflow-anchor) element. However, descendant [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) automatically "turn it back on" (for their own scrolling box) unless they explicitly have <a id="ref-for-propdef-overflow-anchor④"></a>overflow-anchor: none set on them as well.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The `overflow-anchor` property was also proposed (with different values) for [CSS Sticky Scrollbars](http://tabatkins.github.io/specs/css-sticky-scrollbars/), which has now been [superseded](https://tabatkins.github.io/specs/css-sticky-scrollbars/#intro).

## <a id="priv-sec"></a>4.  Privacy and Security Considerations

This specification, as it only adjusts how we compute scroll positions, introduces no new privacy or security considerations.

## <a id="changes"></a> Changes

### <a id="changes-20200211"></a> Changes Since the Feb 11 2020 Working Draft

- <a id="ref-for-anchor-viable-candidate①"></a>

  <a id="ref-for-anchor-priority-candidates②"></a>

  Added definitions of [viable candidate](#anchor-viable-candidate) and [priority candidate](#anchor-priority-candidates).

- Clarified interaction between scroll anchoring and scroll snapping.

## <a id="conformance"></a> Conformance

### <a id="conventions"></a> Document conventions

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

### <a id="conformance-classes"></a> Conformance classes

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

### <a id="partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [anchor node](#scroll-anchoring-anchor-node), in §2
- [anchor node selection algorithm](#anchoring-algorithm), in §2.1
- [auto](#valdef-overflow-anchor-auto), in §3
- [candidate examination algorithm](#candidate-examination), in §2.1
- [excluded subtree](#excluded-subtree), in §2.1
- [fully clipped](#fully-clipped), in §2.1
- [fully visible](#fully-visible), in §2.1
- [none](#valdef-overflow-anchor-none), in §3
- [overflow-anchor](#propdef-overflow-anchor), in §3
- [partially visible](#partially-visible), in §2.1
- [priority candidates](#anchor-priority-candidates), in §2.1
- [scroll anchoring bounding rect](#scroll-anchoring-bounding-rect), in §2.1
- [suppression trigger](#suppression-trigger), in §2.2.2
- [suppression window](#suppression-window), in §2.2.1
- [viable candidate](#anchor-viable-candidate), in §2.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-box-4\] defines the following terms:
  - <a id="term-for-content-area"></a>content area
  - <a id="term-for-propdef-margin"></a>margin
  - <a id="term-for-propdef-padding"></a>padding
- \[css-display-3\] defines the following terms:
  - <a id="term-for-atomic-inline"></a>atomic inline
  - <a id="term-for-box"></a>box
  - <a id="term-for-containing-block"></a>containing block
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-valdef-display-none"></a>none
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-scroll-container"></a>scroll container
  - <a id="term-for-scrollable-overflow-rectangle"></a>scrollable overflow rectangle
- \[css-position-3\] defines the following terms:
  - <a id="term-for-valdef-position-absolute"></a>absolute
  - <a id="term-for-propdef-bottom"></a>bottom
  - <a id="term-for-valdef-position-fixed"></a>fixed
  - <a id="term-for-propdef-left"></a>left
  - <a id="term-for-propdef-position"></a>position
  - <a id="term-for-propdef-right"></a>right
  - <a id="term-for-propdef-top"></a>top
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="term-for-optimal-viewing-region"></a>optimal viewing region
  - <a id="term-for-propdef-scroll-padding"></a>scroll-padding
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-max-height"></a>max-height
  - <a id="term-for-propdef-max-width"></a>max-width
  - <a id="term-for-propdef-min-height"></a>min-height
  - <a id="term-for-propdef-min-width"></a>min-width
  - <a id="term-for-propdef-width"></a>width
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-propdef-transform"></a>transform
- \[css-values-4\] defines the following terms:
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-flow-direction"></a>block flow direction
- \[CSSOM-VIEW\] defines the following terms:
  - <a id="term-for-dom-element-getboundingclientrect"></a>getBoundingClientRect()
  - <a id="term-for-eventdef-document-scroll"></a>scroll
  - <a id="term-for-scrolling-box"></a>scrolling box
- \[DOM\] defines the following terms:
  - <a id="term-for-concept-tree-descendant"></a>descendant
- \[HTML\] defines the following terms:
  - <a id="term-for-dom-anchor"></a>dom anchor
  - <a id="term-for-focused-area-of-the-document"></a>focused area of the document

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 19 May 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; et al. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 19 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-scroll-snap-1"></a>\[CSS-SCROLL-SNAP-1\]  
Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 19 March 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-1&#x2F;](https://www.w3.org/TR/css-scroll-snap-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 23 October 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 31 January 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-view"></a>\[CSSOM-VIEW\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

## <a id="property-index"></a>Property Index

| Name                | Value        | Initial | Applies to   | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value    |
|---------------------|--------------|---------|--------------|------|-------|----------------|-----------------|-------------------|
| <strong><span><a id="ref-for-propdef-overflow-anchor⑤"></a></span><a href="#propdef-overflow-anchor">overflow-anchor</a>&#xA;      </strong> | auto \| none | auto    | all elements | no   | n/a   | discrete       | per grammar     | specified keyword |

