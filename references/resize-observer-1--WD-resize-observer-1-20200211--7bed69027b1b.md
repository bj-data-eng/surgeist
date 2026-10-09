Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Resize Observer](https://www.w3.org/TR/2020/WD-resize-observer-1-20200211/).

Original copyright notice: Copyright © 2020 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Resize Observer

Source snapshot: https://www.w3.org/TR/2020/WD-resize-observer-1-20200211/

Snapshot SHA-256: 7bed69027b1b3bb7d406c73ebee4f34deb693885c77f8db38259438492240528

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>Resize Observer

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2020 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This specification describes an API for observing changes to Element’s size.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of
   its publication. Other documents may supersede this document. A list of
   current W3C publications and the latest revision of this technical report
   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports
   index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document is a <b>First Public Working Draft</b>.

Publication as a First Public Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “resize-observer” in the title, preferably like this: “\[resize-observer\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 March 2019 W3C Process Document](https://www.w3.org/2019/Process-20190301/).

## <a id="intro"></a>1. Introduction

<em>This section is non-normative.</em>

<a id="ref-for-element"></a>

<a id="ref-for-element①"></a>

Responsive Web Components need to respond to <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s size changes. An example is an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> that displays a map:

- <a id="ref-for-element②"></a>

  it displays a map by tiling its content box with <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> tiles.

- when resized, it must redo the tiling.

<a id="ref-for-viewport"></a>

<a id="ref-for-eventdef-window-resize"></a>

Responsive Web Applications can already respond to [viewport](https://www.w3.org/TR/css3-positioning/#viewport) size changes. This is done with CSS media queries, or window.<code><a href="https://www.w3.org/TR/cssom-view/#eventdef-window-resize">resize</a></code> event.

<a id="ref-for-element③"></a>

<a id="ref-for-eventdef-window-resize①"></a>

The ResizeObserver API is an interface for observing changes to Element’s size. It is an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s counterpart to window.<code><a href="https://www.w3.org/TR/cssom-view/#eventdef-window-resize">resize</a></code> event.

<a id="ref-for-element④"></a>

ResizeObserver’s notifications can be used to respond to changes in <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s size. Some interesting facts about these observations:

- Observation will fire when watched Element is inserted/removed from DOM.

- <a id="ref-for-propdef-display"></a>

  Observation will fire when watched Element [display](https://www.w3.org/TR/css-display-3/#propdef-display) gets set to none.

- Observations do not fire for non-replaced inline Elements.

- Observations will not be triggered by CSS transforms.

- Observation will fire when observation starts if Element is [being rendered](https://html.spec.whatwg.org/#being-rendered), and Element’s size is not 0,0.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ecce5bcd"></a>
>
> ```text
> <canvas id="elipse" style="display:block"></canvas>
> <div id="menu" style="display:block;width:100px">
>     <img src="hamburger.jpg" style="width:24px;height:24px">
>     <p class="title">menu title</p>
> </div>
> ```
>
> ```text
> // In response to resize, elipse paints an elipse inside a canvas
> document.querySelector('#elipse').handleResize = entry => {
>     entry.target.width = entry.borderBoxSize[0].inlineSize;
>     entry.target.height = entry.borderBoxSize[0].blockSize;
>     let rx = Math.floor(entry.target.width / 2);
>     let ry = Math.floor(entry.target.height / 2);
>     let ctx = entry.target.getContext('2d');
>     ctx.beginPath();
>     ctx.ellipse(rx, ry, rx, ry, 0, 0, 2 * Math.PI);
>     ctx.stroke();
> }
> // In response to resize, change title visibility depending on width
> document.querySelector('#menu').handleResize = entry => {
>     let title = entry.target.querySelector(".title")
>     if (entry.borderBoxSize[0].inlineSize < 40)
>         title.style.display = "none";
>     else
>         title.style.display = "inline-block";
> }
> 
> var ro = new ResizeObserver( entries => {
>   for (let entry of entries) {
>     let cs = window.getComputedStyle(entry.target);
>     console.log('watching element:', entry.target);
>     console.log(entry.contentRect.top,' is ', cs.paddingTop);
>     console.log(entry.contentRect.left,' is ', cs.paddingLeft);
>     console.log(entry.borderBoxSize[0].inlineSize,' is ', cs.width);
>     console.log(entry.borderBoxSize[0].blockSize,' is ', cs.height);
>     if (entry.target.handleResize)
>         entry.target.handleResize(entry);
>   }
> });
> ro.observe(document.querySelector('#elipse'));
> ro.observe(document.querySelector('#menu'));
> ```
## <a id="api"></a>2. Resize Observer API

### <a id="resize-observer-interface"></a>2.1. ResizeObserver interface

<a id="ref-for-element⑤"></a>

The ResizeObserver interface is used to observe changes to <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s size.

<a id="ref-for-mutationobserver"></a>

<a id="ref-for-intersection-observer-interface"></a>

It is modeled after <code><a href="https://dom.spec.whatwg.org/#mutationobserver">MutationObserver</a></code> and <code><a href="https://www.w3.org/TR/intersection-observer/#intersection-observer-interface">IntersectionObserver</a></code>.

<a id="enumdef-resizeobserverboxoptions"></a>

<a id="dom-resizeobserverboxoptions-border-box"></a>

<a id="dom-resizeobserverboxoptions-content-box"></a>

<a id="dom-resizeobserverboxoptions-device-pixel-content-box"></a>

```text
enum ResizeObserverBoxOptions {
    "border-box", "content-box", "device-pixel-content-box"
};
```
ResizeObserver can observe different kinds of CSS sizes:

- <a id="ref-for-dom-resizeobserverboxoptions-border-box"></a>

  <a id="ref-for-box-border-area"></a>

  <code><a href="#dom-resizeobserverboxoptions-border-box">border-box</a></code> : size of [box border area](https://www.w3.org/TR/CSS21/box.html#box-border-area) as defined in CSS2.

- <a id="ref-for-dom-resizeobserverboxoptions-content-box"></a>

  <a id="ref-for-content-area"></a>

  <code><a href="#dom-resizeobserverboxoptions-content-box">content-box</a></code> : size of [content area](https://drafts.csswg.org/css-box-3/#content-area) as defined in CSS2.

- <a id="ref-for-dom-resizeobserverboxoptions-device-pixel-content-box"></a>

  <a id="ref-for-content-area①"></a>

  <code><a href="#dom-resizeobserverboxoptions-device-pixel-content-box">device-pixel-content-box</a></code> : size of [content area](https://drafts.csswg.org/css-box-3/#content-area) as defined in CSS2, in device pixels, before applying any CSS transforms on the element or its ancestors. This size must contain integer values.

<a id="ref-for-dom-resizeobserverboxoptions-device-pixel-content-box①"></a>

<a id="ref-for-dom-window-devicepixelratio"></a>

<a id="ref-for-dom-resizeobserverboxoptions-content-box①"></a>

<a id="ref-for-dom-resizeobserverboxoptions-content-box②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The <code><a href="#dom-resizeobserverboxoptions-device-pixel-content-box">device-pixel-content-box</a></code> can be approximated by multiplying [devicePixelRatio](https://www.w3.org/TR/cssom-view-1/#dom-window-devicepixelratio) by the <code><a href="#dom-resizeobserverboxoptions-content-box">content-box</a></code> size. However, due to browser-specific subpixel snapping behavior, authors cannot determine the correct way to round this scaled <code><a href="#dom-resizeobserverboxoptions-content-box">content-box</a></code> size. How a UA computes the device pixel box for an element is implementation-dependent. One possible implementation could be to multiply the box size and position by the device pixel ratio, then round both the resulting floating-point size and position of the box to integer values, in a way that maximizes the quality of the rendered output.

Note that this size can be affected by position changes to the target, and thus is typically more expensive to observe than the other sizes.

<a id="dictdef-resizeobserveroptions"></a>

<a id="ref-for-enumdef-resizeobserverboxoptions"></a>

<a id="dom-resizeobserveroptions-box"></a>

```text
dictionary ResizeObserverOptions {
    ResizeObserverBoxOptions box = "content-box";
};
```
This section is non-normative. An author may desire to observe more than one CSS box. In this case, author will need to use multiple ResizeObservers.

```text
// Observe the content-box
ro.observe(document.querySelector('#menu'), { box: 'content-box' });

// Observe just the border box. Replaces previous observation.
ro1.observe(document.querySelector('#menu'), { box: 'border-box' });
```
> <strong data-conversion-semantic="note">Note</strong>
>
> This does not have any impact on which box dimensions are returned to the defined callback when the event is fired, it solely defines which box the author wishes to observe layout changes on.

<a id="ref-for-Exposed"></a>

<a id="ref-for-dom-resizeobserver-resizeobserver"></a>

<a id="ref-for-callbackdef-resizeobservercallback"></a>

<a id="dom-resizeobserver-resizeobserver-callback-callback"></a>

<a id="resizeobserver"></a>

<a id="ref-for-dom-resizeobserver-observe"></a>

<a id="ref-for-element⑥"></a>

<a id="dom-resizeobserver-observe-target-options-target"></a>

<a id="ref-for-dictdef-resizeobserveroptions"></a>

<a id="dom-resizeobserver-observe-target-options-options"></a>

<a id="ref-for-dom-resizeobserver-unobserve"></a>

<a id="ref-for-element⑦"></a>

<a id="dom-resizeobserver-unobserve-target-target"></a>

<a id="ref-for-dom-resizeobserver-disconnect"></a>

```text
[Exposed=(Window),
 Constructor(ResizeObserverCallback callback)]
interface ResizeObserver {
    void observe(Element target, optional ResizeObserverOptions options);
    void unobserve(Element target);
    void disconnect();
};
```
<a id="dom-resizeobserver-resizeobserver"></a>`new ResizeObserver(callback)`  
1.  <a id="ref-for-resizeobserver"></a>

    Let <var>this</var> be a new <code><a href="#resizeobserver">ResizeObserver</a></code> object.

2.  Set <var>this</var>.<var>callback</var> internal slot to callback.

3.  Set <var>this</var>.<var>observationTargets</var> internal slot to an empty list.

4.  <a id="ref-for-document"></a>

    Add <var>this</var> to <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>.<var>resizeObservers</var> slot.

<a id="dom-resizeobserver-observe"></a>`observe(target, options)`  
Adds target to the list of observed elements.

1.  <a id="ref-for-dom-resizeobserver-observationtargets"></a>

    If <var>target</var> is in <code><a href="#dom-resizeobserver-observationtargets">observationTargets</a></code> slot, call unobserve(<var>target</var>).

2.  <a id="ref-for-resizeobservation"></a>

    Let <var>resizeObservation</var> be new <code><a href="#resizeobservation">ResizeObservation</a></code>(<var>target</var>, <var>options</var>).

3.  Add the <var>resizeObservation</var> to the <var>observationTargets</var> slot.

<a id="dom-resizeobserver-unobserve"></a>`unobserve(target)`  
Removes <var>target</var> from the list of observed elements.

1.  <a id="ref-for-resizeobservation①"></a>

    <a id="ref-for-dom-resizeobserver-observationtargets①"></a>

    Let <var>observation</var> be <code><a href="#resizeobservation">ResizeObservation</a></code> in <code><a href="#dom-resizeobserver-observationtargets">observationTargets</a></code> whose target slot is <var>target</var>.

2.  If <var>observation</var> is not found, return.

3.  <a id="ref-for-dom-resizeobserver-observationtargets②"></a>

    Remove <var>observation</var> from <code><a href="#dom-resizeobserver-observationtargets">observationTargets</a></code>.

<a id="dom-resizeobserver-disconnect"></a>`disconnect()`  
1.  <a id="ref-for-dom-resizeobserver-observationtargets③"></a>

    Clear the <code><a href="#dom-resizeobserver-observationtargets">observationTargets</a></code> list.

2.  <a id="ref-for-dom-resizeobserver-activetargets"></a>

    Clear the <code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code> list.

### <a id="resize-observer-callback"></a>2.2. ResizeObserverCallback

<a id="callbackdef-resizeobservercallback"></a>

<a id="ref-for-resizeobserverentry"></a>

<a id="dom-resizeobservercallback-entries"></a>

<a id="ref-for-resizeobserver①"></a>

<a id="dom-resizeobservercallback-observer"></a>

```text
callback ResizeObserverCallback = void (sequence<ResizeObserverEntry> entries, ResizeObserver observer);
```
<a id="ref-for-resizeobserver②"></a>

<a id="ref-for-broadcast-active-observations"></a>

This callback delivers <code><a href="#resizeobserver">ResizeObserver</a></code>'s notifications. It is invoked by a [broadcast active observations](#broadcast-active-observations) algorithm.

### <a id="resize-observer-entry-interface"></a>2.3. ResizeObserverEntry

<a id="ref-for-Exposed①"></a>

<a id="resizeobserverentry"></a>

<a id="ref-for-element⑧"></a>

<a id="ref-for-dom-resizeobserverentry-target"></a>

<a id="ref-for-domrectreadonly"></a>

<a id="ref-for-dom-resizeobserverentry-contentrect"></a>

<a id="ref-for-resizeobserversize"></a>

<a id="ref-for-dom-resizeobserverentry-borderboxsize"></a>

<a id="ref-for-resizeobserversize①"></a>

<a id="ref-for-dom-resizeobserverentry-contentboxsize"></a>

<a id="ref-for-resizeobserversize②"></a>

<a id="ref-for-dom-resizeobserverentry-devicepixelcontentboxsize"></a>

```text
[Exposed=Window]
interface ResizeObserverEntry {
    readonly attribute Element target;
    readonly attribute DOMRectReadOnly contentRect;
    readonly attribute sequence<ResizeObserverSize> borderBoxSize;
    readonly attribute sequence<ResizeObserverSize> contentBoxSize;
    readonly attribute sequence<ResizeObserverSize> devicePixelContentBoxSize;
};
```
> <strong data-conversion-semantic="note">Note</strong>
>
> contentRect is from the incubation phase of ResizeObserver and is only included for current web compat reasons. It may be deprecated in future levels.

<a id="ref-for-element⑨"></a>

<a id="dom-resizeobserverentry-target"></a>`target`, of type [Element](https://dom.spec.whatwg.org/#element), readonly

<a id="ref-for-element①⓪"></a>

The <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> whose size has changed.

<a id="ref-for-domrectreadonly①"></a>

<a id="dom-resizeobserverentry-contentrect"></a>`contentRect`, of type [DOMRectReadOnly](https://www.w3.org/TR/geometry-1/#domrectreadonly), readonly

<a id="ref-for-element①①"></a>

<a id="ref-for-content-rect"></a>

<a id="ref-for-callbackdef-resizeobservercallback①"></a>

<code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s [content rect](#content-rect) when <code><a href="#callbackdef-resizeobservercallback">ResizeObserverCallback</a></code> is invoked.

<a id="ref-for-resizeobserversize③"></a>

<a id="dom-resizeobserverentry-borderboxsize"></a>`borderBoxSize`, of type sequence\<[ResizeObserverSize](#resizeobserversize)\>, readonly

<a id="ref-for-element①②"></a>

<a id="ref-for-border-box"></a>

<a id="ref-for-callbackdef-resizeobservercallback②"></a>

A sequence containing the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s [border box](https://www.w3.org/TR/css-box-3/#border-box) size when <code><a href="#callbackdef-resizeobservercallback">ResizeObserverCallback</a></code> is invoked.

<a id="ref-for-resizeobserversize④"></a>

<a id="dom-resizeobserverentry-contentboxsize"></a>`contentBoxSize`, of type sequence\<[ResizeObserverSize](#resizeobserversize)\>, readonly

<a id="ref-for-element①③"></a>

<a id="ref-for-content-rect①"></a>

<a id="ref-for-callbackdef-resizeobservercallback③"></a>

A sequence containing the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s [content rect](#content-rect) size when <code><a href="#callbackdef-resizeobservercallback">ResizeObserverCallback</a></code> is invoked.

<a id="ref-for-resizeobserversize⑤"></a>

<a id="dom-resizeobserverentry-devicepixelcontentboxsize"></a>`devicePixelContentBoxSize`, of type sequence\<[ResizeObserverSize](#resizeobserversize)\>, readonly

<a id="ref-for-element①④"></a>

<a id="ref-for-content-rect②"></a>

<a id="ref-for-callbackdef-resizeobservercallback④"></a>

A sequence containing the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s [content rect](#content-rect) size in integral device pixels when <code><a href="#callbackdef-resizeobservercallback">ResizeObserverCallback</a></code> is invoked.

<a id="ref-for-content-rect③"></a>

<a id="ref-for-border-box①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The box size properties are exposed as sequences in order to support elements that have multiple fragments, which occur in [multi-column](https://www.w3.org/TR/css3-multicol/) scenarios. However the current definitions of [content rect](#content-rect) and [border box](https://www.w3.org/TR/css-box-3/#border-box) do not mention how those boxes are affected by multi-column layout. In this spec, there will only be a single ResizeObserverSize returned in the sequences, which will correspond to the dimensions of the first column. A future version of this spec will extend the returned sequences to contain the per-fragment size information.

<a id="resizeobserversize"></a>

<a id="ref-for-idl-unrestricted-double"></a>

<a id="dom-resizeobserversize-inlinesize"></a>

<a id="ref-for-idl-unrestricted-double①"></a>

<a id="dom-resizeobserversize-blocksize"></a>

```text
interface ResizeObserverSize {
    readonly attribute unrestricted double inlineSize;
    readonly attribute unrestricted double blockSize;
};
```
## <a id="processing-model"></a>3. Processing Model

### <a id="resize-observation-interface"></a>3.1. ResizeObservation example struct

<a id="ref-for-element①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> This section is non-normative. ResizeObservation is an example struct that can be used in implementation of Resize Observer. It is being included here in order to help provide clarity during the processing model. It effectively holds observation information for a single <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>. This interface is not visible to Javascript.

<a id="dom-resizeobservation-resizeobservation"></a>

<a id="ref-for-element①⑥"></a>

<a id="dom-resizeobservation-resizeobservation-target-target"></a>

<a id="resizeobservation"></a>

<a id="ref-for-element①⑦"></a>

<a id="ref-for-dom-resizeobservation-target"></a>

<a id="ref-for-enumdef-resizeobserverboxoptions①"></a>

<a id="ref-for-dom-resizeobservation-observedbox"></a>

<a id="ref-for-resizeobserversize⑥"></a>

<a id="ref-for-dom-resizeobservation-lastreportedsizes"></a>

```text
[Constructor(Element target)
]
interface ResizeObservation {
    readonly attribute Element target;
    readonly attribute ResizeObserverBoxOptions observedBox;
    readonly attribute sequence<ResizeObserverSize> lastReportedSizes;
};
```
<a id="ref-for-element①⑧"></a>

<a id="dom-resizeobservation-target"></a>`target`, of type [Element](https://dom.spec.whatwg.org/#element), readonly

<a id="ref-for-element①⑨"></a>

The observed <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>.

<a id="ref-for-enumdef-resizeobserverboxoptions②"></a>

<a id="dom-resizeobservation-observedbox"></a>`observedBox`, of type [ResizeObserverBoxOptions](#enumdef-resizeobserverboxoptions), readonly

Which box is being observed.

<a id="ref-for-resizeobserversize⑦"></a>

<a id="dom-resizeobservation-lastreportedsizes"></a>`lastReportedSizes`, of type sequence\<[ResizeObserverSize](#resizeobserversize)\>, readonly

Ordered sequence of last reported sizes.

<a id="dom-resizeobservation-resizeobservation-target-options"></a>`new ResizeObservation(target, observedBox)`  
1.  <a id="ref-for-resizeobservation②"></a>

    Let <var>this</var> be a new <code><a href="#resizeobservation">ResizeObservation</a></code> object

2.  <a id="ref-for-dom-resizeobservation-target①"></a>

    Set <var>this</var> internal <code><a href="#dom-resizeobservation-target">target</a></code> slot to <var>target</var>

3.  <a id="ref-for-dom-resizeobservation-observedbox①"></a>

    Set <var>this</var> internal <code><a href="#dom-resizeobservation-observedbox">observedBox</a></code> slot to <var>observedBox</var>

4.  <a id="ref-for-dom-resizeobservation-lastreportedsizes①"></a>

    Set <var>this</var> internal <code><a href="#dom-resizeobservation-lastreportedsizes">lastReportedSizes</a></code> slot to \[(0,0)\]

<a id="dom-resizeobservation-isactive"></a>`isActive()`  
1.  <a id="ref-for-calculate-box-size①"></a>

    Set <var>currentSize</var> by [calculate box size](#calculate-box-size%E2%91%A0) given <var>target</var> and <var>observedBox</var>.

2.  <a id="ref-for-dom-resizeobservation-lastreportedsizes②"></a>

    Return true if <var>currentSize</var> is not equal to the first entry in this.<code><a href="#dom-resizeobservation-lastreportedsizes">lastReportedSizes</a></code>.

3.  Return false.

### <a id="internal-slot-definitions"></a>3.2. Internal Slot Definitions

#### <a id="document-slots"></a>3.2.1. Document

<a id="ref-for-concept-document"></a>

<a id="ref-for-resizeobserver③"></a>

[Document](https://dom.spec.whatwg.org/#concept-document) has a <a id="dom-document-resizeobservers"></a>`resizeObservers` slot that is a list of <code><a href="#resizeobserver">ResizeObserver</a></code>s in this document. It is initialized to empty.

#### <a id="resize-observer-slots"></a>3.2.2. ResizeObserver

<a id="ref-for-resizeobserver④"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> has a <a id="dom-resizeobserver-callback"></a>`callback` slot, initialized by constructor.

<a id="ref-for-resizeobserver⑤"></a>

<a id="ref-for-resizeobservation③"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> has an <a id="dom-resizeobserver-observationtargets"></a>`observationTargets` slot, which is a list of <code><a href="#resizeobservation">ResizeObservation</a></code>s. It represents all Elements being observed.

<a id="ref-for-resizeobserver⑥"></a>

<a id="ref-for-resizeobservation④"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> has a <a id="dom-resizeobserver-activetargets"></a>`activeTargets` slot, which is a list of <code><a href="#resizeobservation">ResizeObservation</a></code>s. It represents all Elements whose size has changed since last observation broadcast that are eligible for broadcast.

<a id="ref-for-resizeobserver⑦"></a>

<a id="ref-for-resizeobservation⑤"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> has a <a id="dom-resizeobserver-skippedtargets"></a>`skippedTargets` slot, which is a list of <code><a href="#resizeobservation">ResizeObservation</a></code>s. It represents all Elements whose size has changed since last observation broadcast that are <strong>not</strong> eligible for broadcast

### <a id="css-definitions"></a>3.3. CSS Definitions

#### <a id="content-rect-h"></a>3.3.1. content rect

DOM <a id="content-rect"></a>content rect is a rect whose:

- <a id="ref-for-content-width"></a>

  width is [content width](https://www.w3.org/TR/CSS2/box.html#content-width)

- <a id="ref-for-content-height"></a>

  height is [content height](https://www.w3.org/TR/CSS2/box.html#content-height)

- <a id="ref-for-padding-physical"></a>

  top is [padding top](https://drafts.csswg.org/css-box-3/#padding-physical)

- <a id="ref-for-padding-physical①"></a>

  left is [padding left](https://drafts.csswg.org/css-box-3/#padding-physical)

<a id="ref-for-content-width①"></a>

<a id="ref-for-element②⓪"></a>

[content width](https://www.w3.org/TR/CSS2/box.html#content-width) spec does not mention how [multi-column](https://www.w3.org/TR/css3-multicol/) layout affects content box. In this spec, content width of an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> inside multi-column is the result of `getComputedStyle(element).width`. This currently evaluates to width of the first column.

Having content rect position be padding-top/left is useful for absolute positioning of target’s children. Absolute position coordinate space origin is topLeft of the padding rect.

Watching content rect means that:

- observation will fire when watched Element is inserted/removed from DOM.

- observation will fire when watched Element display gets set to none.

- non-replaced inline Elements will always have an empty content rect.

- observations will not be triggered by CSS transforms.

<a id="ref-for-BoundingBoxes"></a>

<a id="ref-for-InterfaceSVGGraphicsElement"></a>

Web content can also contain SVG elements. SVG Elements define [bounding box](https://www.w3.org/TR/SVG2/coords.html#BoundingBoxes) instead of a content box. Content rect for [SVGGraphicsElement](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGGraphicsElement)s is a rect whose:

- <a id="ref-for-BoundingBoxes①"></a>

  width is [bounding box](https://www.w3.org/TR/SVG2/coords.html#BoundingBoxes) width

- <a id="ref-for-BoundingBoxes②"></a>

  height is [bounding box](https://www.w3.org/TR/SVG2/coords.html#BoundingBoxes) height

- top and left are 0

### <a id="algorithms"></a>3.4. Algorithms

#### <a id="gather-active-observations-h"></a>3.4.1. Gather active observations at depth

It computes all active observations for a <var>document</var>. To <a id="gather-active-observations-at-depth"></a>gather active observations at depth, run these steps:

1.  Let <var>depth</var> be the depth passed in.

2.  <a id="ref-for-dom-document-resizeobservers"></a>

    For each <var>observer</var> in <code><a href="#dom-document-resizeobservers">resizeObservers</a></code> run these steps:

    1.  <a id="ref-for-dom-resizeobserver-activetargets①"></a>

        <a id="ref-for-dom-resizeobserver-skippedtargets"></a>

        Clear <var>observer</var>’s <code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code>, and <code><a href="#dom-resizeobserver-skippedtargets">skippedTargets</a></code>.

    2.  <a id="ref-for-dom-resizeobserver-observationtargets④"></a>

        For each <var>observation</var> in <var>observer</var>.<code><a href="#dom-resizeobserver-observationtargets">observationTargets</a></code> run this step:

        1.  <a id="ref-for-dom-resizeobservation-isactive"></a>

            If <var>observation</var>.<code><a href="#dom-resizeobservation-isactive">isActive()</a></code> is true

            1.  <a id="ref-for-calculate-depth-for-node"></a>

                <a id="ref-for-dom-resizeobservation-target②"></a>

                Let <var>targetDepth</var> be result of [calculate depth for node](#calculate-depth-for-node) for <var>observation</var>.<code><a href="#dom-resizeobservation-target">target</a></code>.

            2.  <a id="ref-for-dom-resizeobserver-activetargets②"></a>

                If <var>targetDepth</var> is greater than <var>depth</var> then add <var>observation</var> to <code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code>.

            3.  <a id="ref-for-dom-resizeobserver-skippedtargets①"></a>

                Else add <var>observation</var> to <code><a href="#dom-resizeobserver-skippedtargets">skippedTargets</a></code>.

#### <a id="has-active-observations-h"></a>3.4.2. Has active observations

<a id="ref-for-document①"></a>

To determine if <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> <a id="has-active-observations"></a>has active observations run these steps:

1.  <a id="ref-for-dom-document-resizeobservers①"></a>

    For each <var>observer</var> in <code><a href="#dom-document-resizeobservers">resizeObservers</a></code> run this step:

    1.  <a id="ref-for-dom-resizeobserver-activetargets③"></a>

        If <var>observer</var>.<code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code> is not empty, return true.

2.  return false.

#### <a id="has-skipped-observations-h"></a>3.4.3. Has skipped observations

<a id="ref-for-document②"></a>

To determine if <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> <a id="has-skipped-observations"></a>has skipped observations run these steps:

1.  <a id="ref-for-dom-document-resizeobservers②"></a>

    For each <var>observer</var> in <code><a href="#dom-document-resizeobservers">resizeObservers</a></code> run this step:

    1.  <a id="ref-for-dom-resizeobserver-skippedtargets②"></a>

        If <var>observer</var>.<code><a href="#dom-resizeobserver-skippedtargets">skippedTargets</a></code> is not empty, return true.

2.  return false.

#### <a id="create-and-populate-resizeobserverentry-h"></a>3.4.4.  Create and populate a ResizeObserverEntry 

To <a id="create-and-populate-a-resizeobserverentry"></a>create and populate a ResizeObserverEntry for a given <var>target</var>, run these steps:

1.  <a id="ref-for-resizeobserverentry①"></a>

    Let <var>this</var> be a new <code><a href="#resizeobserverentry">ResizeObserverEntry</a></code>.

2.  <a id="ref-for-dom-resizeobserverentry-target①"></a>

    Set <var>this</var>.<code><a href="#dom-resizeobserverentry-target">target</a></code> slot to <var>target</var>.

3.  <a id="ref-for-dom-resizeobserverentry-borderboxsize①"></a>

    Set <var>this</var>.<code><a href="#dom-resizeobserverentry-borderboxsize">borderBoxSize</a></code> slot to result of [computing size given <var>target</var> and observedBox of "border-box"](#calculate-box-size).

4.  <a id="ref-for-dom-resizeobserverentry-contentboxsize①"></a>

    Set <var>this</var>.<code><a href="#dom-resizeobserverentry-contentboxsize">contentBoxSize</a></code> slot to result of [computing size given <var>target</var> and observedBox of "content-box"](#calculate-box-size).

5.  <a id="ref-for-dom-resizeobserverentry-devicepixelcontentboxsize①"></a>

    Set <var>this</var>.<code><a href="#dom-resizeobserverentry-devicepixelcontentboxsize">devicePixelContentBoxSize</a></code> slot to result of [computing size given <var>target</var> and observedBox of "device-pixel-content-box"](#calculate-box-size).

6.  <a id="ref-for-dom-resizeobserverentry-contentrect①"></a>

    <a id="ref-for-dom-resizeobserverentry-contentboxsize②"></a>

    Set <var>this</var>.<code><a href="#dom-resizeobserverentry-contentrect">contentRect</a></code> to logical <var>this</var>.<code><a href="#dom-resizeobserverentry-contentboxsize">contentBoxSize</a></code> given <var>target</var> and observedBox of "content-box".

7.  If <var>target</var> is not an SVG element do these steps:

    1.  <a id="ref-for-padding-physical②"></a>

        Set <var>this</var>.<var>contentRect</var>.top to <var>target</var>.[padding top](https://drafts.csswg.org/css-box-3/#padding-physical).

    2.  <a id="ref-for-padding-physical③"></a>

        Set <var>this</var>.<var>contentRect</var>.left to <var>target</var>.[padding left](https://drafts.csswg.org/css-box-3/#padding-physical).

8.  If <var>target</var> is an SVG element do these steps:

    1.  Set <var>this</var>.<var>contentRect</var>.top and <var>this</var>.contentRect.left to 0.

#### <a id="broadcast-resize-notifications-h"></a>3.4.5. Broadcast active observations

<a id="broadcast-active-observations"></a>broadcast active observations delivers all active observations in a document, and returns the depth of the shallowest broadcast target depth.

To broadcast active observations for a <var>document</var>, run these steps:

1.  Let <var>shallowestTargetDepth</var> be ∞

2.  <a id="ref-for-dom-document-resizeobservers③"></a>

    For each <var>observer</var> in <var>document</var>.<code><a href="#dom-document-resizeobservers">resizeObservers</a></code> run these steps:

    1.  <a id="ref-for-dom-resizeobserver-activetargets④"></a>

        If <var>observer</var>.<code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code> slot is empty, continue.

    2.  <a id="ref-for-resizeobserverentry②"></a>

        Let <var>entries</var> be an empty list of <code><a href="#resizeobserverentry">ResizeObserverEntry</a></code>ies.

    3.  <a id="ref-for-dom-resizeobserver-activetargets⑤"></a>

        For each <var>observation</var> in <code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code> perform these steps:

        1.  <a id="ref-for-create-and-populate-a-resizeobserverentry"></a>

            <a id="ref-for-dom-resizeobservation-target③"></a>

            Let <var>entry</var> be the result of running [create and populate a ResizeObserverEntry](#create-and-populate-a-resizeobserverentry) given <var>observation</var>.<code><a href="#dom-resizeobservation-target">target</a></code>.

        2.  Add <var>entry</var> to <var>entries</var>.

        3.  <a id="ref-for-dom-resizeobservation-lastreportedsizes③"></a>

            Set <var>observation</var>.<code><a href="#dom-resizeobservation-lastreportedsizes">lastReportedSizes</a></code> to matching <var>entry</var> sizes.

            1.  <a id="ref-for-dom-resizeobserverentry-borderboxsize②"></a>

                <a id="ref-for-dom-resizeobservation-observedbox②"></a>

                Matching sizes are <var>entry</var>.<code><a href="#dom-resizeobserverentry-borderboxsize">borderBoxSize</a></code> if <var>observation</var>.<code><a href="#dom-resizeobservation-observedbox">observedBox</a></code> is "border-box"

            2.  <a id="ref-for-dom-resizeobserverentry-contentboxsize③"></a>

                <a id="ref-for-dom-resizeobservation-observedbox③"></a>

                Matching sizes are <var>entry</var>.<code><a href="#dom-resizeobserverentry-contentboxsize">contentBoxSize</a></code> if <var>observation</var>.<code><a href="#dom-resizeobservation-observedbox">observedBox</a></code> is "content-box"

            3.  <a id="ref-for-dom-resizeobserverentry-devicepixelcontentboxsize②"></a>

                <a id="ref-for-dom-resizeobservation-observedbox④"></a>

                Matching sizes are <var>entry</var>.<code><a href="#dom-resizeobserverentry-devicepixelcontentboxsize">devicePixelContentBoxSize</a></code> if <var>observation</var>.<code><a href="#dom-resizeobservation-observedbox">observedBox</a></code> is "device-pixel-content-box"

        4.  <a id="ref-for-calculate-depth-for-node①"></a>

            <a id="ref-for-dom-resizeobservation-target④"></a>

            Set <var>targetDepth</var> to the result of [calculate depth for node](#calculate-depth-for-node) for <var>observation</var>.<code><a href="#dom-resizeobservation-target">target</a></code>.

        5.  Set <var>shallowestTargetDepth</var> to <var>targetDepth</var> if <var>targetDepth</var> \< <var>shallowestTargetDepth</var>

    4.  <a id="ref-for-dom-resizeobserver-callback"></a>

        Invoke <var>observer</var>.<code><a href="#dom-resizeobserver-callback">callback</a></code> with <var>entries</var>.

    5.  <a id="ref-for-dom-resizeobserver-activetargets⑥"></a>

        Clear <var>observer</var>.<code><a href="#dom-resizeobserver-activetargets">activeTargets</a></code>.

3.  Return <var>shallowestTargetDepth</var>.

#### <a id="deliver-resize-error"></a>3.4.6. Deliver Resize Loop Error

To <a id="deliver-resize-loop-error-notification"></a>deliver resize loop error notification run these steps:

1.  <a id="ref-for-errorevent"></a>

    Create a new <code><a href="https://html.spec.whatwg.org/multipage/webappapis.html#errorevent">ErrorEvent</a></code>.

2.  Initialize <var>event</var>’s message slot to "ResizeObserver loop completed with undelivered notifications.".

3.  Report the exception <var>event</var>.

#### <a id="calculate-depth-for-node-h"></a>3.4.7. Calculate depth for node

To <a id="calculate-depth-for-node"></a>calculate depth for node, given a <var>node</var>, run these steps:

1.  Let <var>p</var> be the parent-traversal path from <var>node</var> to a root Element of this element’s flattened DOM tree.

2.  Return number of nodes in <var>p</var>.

#### <a id="calculate-box-size"></a>3.4.8. Calculate box size, given target and observed box

<a id="ref-for-element②①"></a>

<a id="ref-for-enumdef-resizeobserverboxoptions③"></a>

This algorithm computes <var>target</var> <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>'s observed box size. Type of box is described by <code><a href="#enumdef-resizeobserverboxoptions">ResizeObserverBoxOptions</a></code>. SVG Elements are an exception. SVG size is always its bounding box size, because SVG elements do not use standard CSS box model.

To <a id="calculate-box-size①"></a>calculate box size, given <var>target</var> and <var>observedBox</var>, run these steps:

1.  <a id="ref-for-InterfaceSVGGraphicsElement①"></a>

    If <var>target</var> is an <code><a href="https://www.w3.org/TR/svg2/types.html#InterfaceSVGGraphicsElement">SVGGraphicsElement</a></code>

    1.  <a id="ref-for-BoundingBoxes③"></a>

        Set <var>computedSize</var>.inlineSize to <var>target</var>’s [bounding box](https://www.w3.org/TR/SVG2/coords.html#BoundingBoxes) inline length.

    2.  <a id="ref-for-BoundingBoxes④"></a>

        Set <var>computedSize</var>.blockSize to <var>target</var>’s [bounding box](https://www.w3.org/TR/SVG2/coords.html#BoundingBoxes) block length.

2.  <a id="ref-for-InterfaceSVGGraphicsElement②"></a>

    If <var>target</var> is not an <code><a href="https://www.w3.org/TR/svg2/types.html#InterfaceSVGGraphicsElement">SVGGraphicsElement</a></code>

    1.  If <var>observedBox</var> is "border-box"

        1.  <a id="ref-for-border-area"></a>

            Set <var>computedSize</var>.inlineSize to target’s [border area](https://www.w3.org/TR/css-box-3/#border-area) inline length.

        2.  <a id="ref-for-border-area①"></a>

            Set <var>computedSize</var>.blockSize to target’s [border area](https://www.w3.org/TR/css-box-3/#border-area) block length.

    2.  If <var>observedBox</var> is "content-box"

        1.  <a id="ref-for-content-area②"></a>

            Set <var>computedSize</var>.inlineSize to target’s [content area](https://drafts.csswg.org/css-box-3/#content-area) inline length.

        2.  <a id="ref-for-content-area③"></a>

            Set <var>computedSize</var>.blockSize to target’s [content area](https://drafts.csswg.org/css-box-3/#content-area) block length.

    3.  If <var>observedBox</var> is "device-pixel-content-box"

        1.  <a id="ref-for-content-area④"></a>

            Set <var>computedSize</var>.inlineSize to target’s [content area](https://drafts.csswg.org/css-box-3/#content-area) inline length, in integral device pixels.

        2.  <a id="ref-for-content-area⑤"></a>

            Set <var>computedSize</var>.blockSize to target’s [content area](https://drafts.csswg.org/css-box-3/#content-area) block length, in integral device pixels.

    4.  return <var>computedSize</var>.

### <a id="lifetime"></a>3.5. ResizeObserver Lifetime

<a id="ref-for-resizeobserver⑧"></a>

A <code><a href="#resizeobserver">ResizeObserver</a></code> will remain alive until both of these conditions are met:

- there are no scripting references to the observer.

- the observer is not observing any targets.

### <a id="integrations"></a>3.6. External Spec Integrations

#### <a id="html-event-loop"></a>3.6.1.  HTML Processing Model: Event Loop

<a id="ref-for-resizeobserver⑨"></a>

<a id="ref-for-processing-model-8"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> processing happens inside the step 7.12 of the [HTML Processing Model](https://html.spec.whatwg.org/multipage/webappapis.html#processing-model-8) event loop.

Step 12 is currently underspecified as:

“For each fully active Document in docs, update the rendering or user interface of that Document and its browsing context to reflect the current state.”.

Existing step 12 can be fully specified as:

For each fully active Document in docs, run the following steps for that Document and its browsing contents:

1.  Recalc styles

2.  Update layout

3.  Paint

<a id="ref-for-resizeobserver①⓪"></a>

<code><a href="#resizeobserver">ResizeObserver</a></code> extends step 12 with resize notifications. It tries to deliver all pending notifications by looping until no pending notifications are available. This can cause an infinite loop.

Infinite loop is prevented by shrinking the set of nodes that can notify at every iteration. In each iteration, only nodes deeper than the shallowest node in previous iteration can notify.

An error is generated if notification loop completes, and there are undelivered notifications. Elements with undelivered notifications will be considered for delivery in the next loop.

<a id="ref-for-resizeobserver①①"></a>

Step 12 with <code><a href="#resizeobserver">ResizeObserver</a></code> notifications is:

For each fully active Document in docs, run the following steps for that Document and its browsing context:

1.  Recalc styles

2.  Update layout

3.  Set <var>depth</var> to 0

4.  <a id="ref-for-gather-active-observations-at-depth"></a>

    <a id="ref-for-document③"></a>

    [Gather active observations at depth](#gather-active-observations-at-depth) <var>depth</var> for <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>

5.  <a id="ref-for-has-active-observations"></a>

    Repeat while document [has active observations](#has-active-observations)

    2.  <a id="ref-for-broadcast-active-observations①"></a>

        Set <var>depth</var> to [broadcast active observations](#broadcast-active-observations).

    3.  Recalc styles

    4.  Update layout

    5.  <a id="ref-for-gather-active-observations-at-depth①"></a>

        <a id="ref-for-document④"></a>

        [Gather active observations at depth](#gather-active-observations-at-depth) <var>depth</var> for <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>

6.  <a id="ref-for-document⑤"></a>

    <a id="ref-for-has-skipped-observations"></a>

    <a id="ref-for-deliver-resize-loop-error-notification"></a>

    If <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> [has skipped observations](#has-skipped-observations) then [deliver resize loop error notification](#deliver-resize-loop-error-notification)

7.  <a id="ref-for-document⑥"></a>

    Update the rendering or user interface of <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> and its browsing context to reflect the current state.

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

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid
		(and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)
		any at-rules, properties, property values, keywords, and other syntactic constructs
		for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [activeTargets](#dom-resizeobserver-activetargets), in §3.2.2
- [blockSize](#dom-resizeobserversize-blocksize), in §2.3
- ["border-box"](#dom-resizeobserverboxoptions-border-box), in §2.1
- [borderBoxSize](#dom-resizeobserverentry-borderboxsize), in §2.3
- [box](#dom-resizeobserveroptions-box), in §2.1
- [broadcast active observations](#broadcast-active-observations), in §3.4.5
- [calculate box size](#calculate-box-size%E2%91%A0), in §3.4.8
- [calculate depth for node](#calculate-depth-for-node), in §3.4.7
- [callback](#dom-resizeobserver-callback), in §3.2.2
- ["content-box"](#dom-resizeobserverboxoptions-content-box), in §2.1
- [contentBoxSize](#dom-resizeobserverentry-contentboxsize), in §2.3
- [content rect](#content-rect), in §3.3.1
- [contentRect](#dom-resizeobserverentry-contentrect), in §2.3
- [create and populate a ResizeObserverEntry](#create-and-populate-a-resizeobserverentry), in §3.4.4
- [deliver resize loop error notification](#deliver-resize-loop-error-notification), in §3.4.6
- ["device-pixel-content-box"](#dom-resizeobserverboxoptions-device-pixel-content-box), in §2.1
- [devicePixelContentBoxSize](#dom-resizeobserverentry-devicepixelcontentboxsize), in §2.3
- [disconnect()](#dom-resizeobserver-disconnect), in §2.1
- [gather active observations at depth](#gather-active-observations-at-depth), in §3.4.1
- [has active observations](#has-active-observations), in §3.4.2
- [has skipped observations](#has-skipped-observations), in §3.4.3
- [inlineSize](#dom-resizeobserversize-inlinesize), in §2.3
- [isActive()](#dom-resizeobservation-isactive), in §3.1
- [lastReportedSizes](#dom-resizeobservation-lastreportedsizes), in §3.1
- [observationTargets](#dom-resizeobserver-observationtargets), in §3.2.2
- [observedBox](#dom-resizeobservation-observedbox), in §3.1
- [observe(target)](#dom-resizeobserver-observe), in §2.1
- [observe(target, options)](#dom-resizeobserver-observe), in §2.1
- [ResizeObservation](#resizeobservation), in §3.1
- [ResizeObservation(target)](#dom-resizeobservation-resizeobservation), in §3.1
- [ResizeObservation(target, options)](#dom-resizeobservation-resizeobservation-target-options), in §3.1
- [ResizeObserver](#resizeobserver), in §2.1
- [ResizeObserverBoxOptions](#enumdef-resizeobserverboxoptions), in §2.1
- [ResizeObserver(callback)](#dom-resizeobserver-resizeobserver), in §2.1
- [ResizeObserverCallback](#callbackdef-resizeobservercallback), in §2.2
- [ResizeObserverEntry](#resizeobserverentry), in §2.3
- [ResizeObserverOptions](#dictdef-resizeobserveroptions), in §2.1
- [resizeObservers](#dom-document-resizeobservers), in §3.2.1
- [ResizeObserverSize](#resizeobserversize), in §2.3
- [skippedTargets](#dom-resizeobserver-skippedtargets), in §3.2.2
- target
  - [attribute for ResizeObservation](#dom-resizeobservation-target), in §3.1
  - [attribute for ResizeObserverEntry](#dom-resizeobserverentry-target), in §2.3
- [unobserve(target)](#dom-resizeobserver-unobserve), in §2.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-box-3\] defines the following terms:
  - <a id="term-for-border-area"></a>border area
  - <a id="term-for-border-box"></a>border box
- \[cssom-view-1\] defines the following terms:
  - <a id="term-for-eventdef-window-resize"></a>resize
- \[DOM\] defines the following terms:
  - <a id="term-for-document"></a>Document
  - <a id="term-for-element"></a>Element
  - <a id="term-for-mutationobserver"></a>MutationObserver
  - <a id="term-for-concept-document"></a>document
- \[geometry-1\] defines the following terms:
  - <a id="term-for-domrectreadonly"></a>DOMRectReadOnly
- \[HTML\] defines the following terms:
  - <a id="term-for-errorevent"></a>ErrorEvent
- \[SVG2\] defines the following terms:
  - <a id="term-for-InterfaceSVGGraphicsElement"></a>SVGGraphicsElement
- \[WebIDL\] defines the following terms:
  - <a id="term-for-Exposed"></a>Exposed
  - <a id="term-for-idl-unrestricted-double"></a>unrestricted double

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-geometry-1"></a>\[GEOMETRY-1\]  
Simon Pieters; Chris Harrelson. [Geometry Interfaces Module Level 1](https://www.w3.org/TR/geometry-1/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;geometry-1&#x2F;](https://www.w3.org/TR/geometry-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-webidl"></a>\[WebIDL\]  
Boris Zbarsky. [Web IDL](https://heycam.github.io/webidl/). 15 December 2016. ED. URL: [https&#x3A;&#x2F;&#x2F;heycam&#x2E;github&#x2E;io&#x2F;webidl&#x2F;](https://heycam.github.io/webidl/)

## <a id="idl-index"></a>IDL Index

<a id="ref-for-enumdef-resizeobserverboxoptions④"></a>

<a id="ref-for-Exposed②"></a>

<a id="ref-for-dom-resizeobserver-resizeobserver①"></a>

<a id="ref-for-callbackdef-resizeobservercallback⑤"></a>

<a id="ref-for-dom-resizeobserver-observe①"></a>

<a id="ref-for-element⑥①"></a>

<a id="ref-for-dictdef-resizeobserveroptions①"></a>

<a id="ref-for-dom-resizeobserver-unobserve①"></a>

<a id="ref-for-element⑦①"></a>

<a id="ref-for-dom-resizeobserver-disconnect①"></a>

<a id="ref-for-resizeobserverentry③"></a>

<a id="ref-for-resizeobserver①②"></a>

<a id="ref-for-Exposed①①"></a>

<a id="ref-for-element⑧①"></a>

<a id="ref-for-dom-resizeobserverentry-target②"></a>

<a id="ref-for-domrectreadonly②"></a>

<a id="ref-for-dom-resizeobserverentry-contentrect②"></a>

<a id="ref-for-resizeobserversize⑧"></a>

<a id="ref-for-dom-resizeobserverentry-borderboxsize③"></a>

<a id="ref-for-resizeobserversize①①"></a>

<a id="ref-for-dom-resizeobserverentry-contentboxsize④"></a>

<a id="ref-for-resizeobserversize②①"></a>

<a id="ref-for-dom-resizeobserverentry-devicepixelcontentboxsize③"></a>

<a id="ref-for-idl-unrestricted-double②"></a>

<a id="ref-for-idl-unrestricted-double①①"></a>

<a id="ref-for-element①⑥①"></a>

<a id="ref-for-element①⑦①"></a>

<a id="ref-for-dom-resizeobservation-target⑤"></a>

<a id="ref-for-enumdef-resizeobserverboxoptions①①"></a>

<a id="ref-for-dom-resizeobservation-observedbox⑤"></a>

<a id="ref-for-resizeobserversize⑥①"></a>

<a id="ref-for-dom-resizeobservation-lastreportedsizes④"></a>

```text
enum ResizeObserverBoxOptions {
    "border-box", "content-box", "device-pixel-content-box"
};

dictionary ResizeObserverOptions {
    ResizeObserverBoxOptions box = "content-box";
};

[Exposed=(Window),
 Constructor(ResizeObserverCallback callback)]
interface ResizeObserver {
    void observe(Element target, optional ResizeObserverOptions options);
    void unobserve(Element target);
    void disconnect();
};

callback ResizeObserverCallback = void (sequence<ResizeObserverEntry> entries, ResizeObserver observer);

[Exposed=Window]
interface ResizeObserverEntry {
    readonly attribute Element target;
    readonly attribute DOMRectReadOnly contentRect;
    readonly attribute sequence<ResizeObserverSize> borderBoxSize;
    readonly attribute sequence<ResizeObserverSize> contentBoxSize;
    readonly attribute sequence<ResizeObserverSize> devicePixelContentBoxSize;
};

interface ResizeObserverSize {
    readonly attribute unrestricted double inlineSize;
    readonly attribute unrestricted double blockSize;
};

[Constructor(Element target)
]
interface ResizeObservation {
    readonly attribute Element target;
    readonly attribute ResizeObserverBoxOptions observedBox;
    readonly attribute sequence<ResizeObserverSize> lastReportedSizes;
};

```