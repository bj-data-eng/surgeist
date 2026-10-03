Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS View Transitions Module Level 1](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS View Transitions Module Level 1

Source snapshot: https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/

Snapshot SHA-256: 7f8e537956b6c0fb0259d0e9231395ea985b35ee8f690e5d9e05e226110fc044

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 2 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS View Transitions Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module defines the View Transition API, along with associated properties and pseudo-elements, which allows developers to create animated visual transitions representing changes in the document state.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-view-transitions” in the title, like this: “\[css-view-transitions\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-view-transitions%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

<em>This section is non-normative.</em>

<a id="ref-for-concept-document"></a>

This specification introduces a DOM API and associated CSS features that allow developers to create animated visual transitions, called <a id="view-transitions"></a>view transitions between different states of a [document](https://dom.spec.whatwg.org/#concept-document).

### <a id="separating-transitions"></a>1.1. Separating Visual Transitions from DOM Updates

Traditionally, creating a visual transition between two document states required a period where both states were present in the DOM at the same time. In fact, it usually involved creating a specific DOM structure that could represent both states. For example, if one element was “moving” between containers, that element often needed to exist outside of either container for the period of the transition, to avoid clipping from either container or their ancestor elements.

This extra in-between state often resulted in UX and accessibility issues, as the structure of the DOM was compromised for a purely-visual effect.

<a id="ref-for-view-transitions"></a>

<a id="ref-for-x22"></a>

[View Transitions](#view-transitions) avoid this troublesome in-between state by allowing the DOM to switch between states instantaneously, then performing a customizable visual transition between the two states in another layer, using a static visual capture of the old state, and a live capture of the new state. These captures are represented as a tree of [pseudo-elements](https://www.w3.org/TR/CSS21/selector.html#x22) (detailed in [§ 3.2 View Transition Pseudo-elements](#view-transition-pseudos)), where the old visual state co-exists with the new state, allowing effects such as cross-fading while animating from the old to new size and position.

### <a id="customizing"></a>1.2. View Transition Customization

<a id="ref-for-dom-document-startviewtransition"></a>

<a id="ref-for-view-transitions①"></a>

<a id="ref-for-propdef-view-transition-name"></a>

By default, <code>document.<code><a href="#dom-document-startviewtransition">startViewTransition()</a></code></code> creates a [view transition](#view-transitions) consisting of a page-wide cross-fade between the two DOM states. Developers can also choose which elements are captured independently using the [view-transition-name](#propdef-view-transition-name) CSS property, allowing these to be animated independently of the rest of the page. Since the transitional state (where both old and new visual captures exist) is represented as pseudo-elements, developers can customize each transition using familiar features such as [CSS Animations](https://www.w3.org/TR/css-animations/) and [Web Animations](https://www.w3.org/TR/web-animations/).

### <a id="lifecycle"></a>1.3. View Transition Lifecycle

<a id="ref-for-view-transitions②"></a>

A successful [view transition](#view-transitions) goes through the following phases:

1.  <a id="ref-for-dom-document-startviewtransition①"></a>

    <a id="ref-for-callbackdef-updatecallback"></a>

    <a id="ref-for-viewtransition"></a>

    Developer calls <code>document.<code><a href="#dom-document-startviewtransition">startViewTransition</a></code>(<code><a href="#callbackdef-updatecallback">updateCallback</a></code>)</code>, which returns a <code><a href="#viewtransition">ViewTransition</a></code>, <var>viewTransition</var>.

2.  Current state captured as the “old” state.

3.  Rendering paused.

4.  <a id="ref-for-callbackdef-updatecallback①"></a>

    Developer’s <code><a href="#callbackdef-updatecallback">updateCallback</a></code> function, if provided, is called, which updates the document state.

5.  <a id="ref-for-dom-viewtransition-updatecallbackdone"></a>

    <code><var>viewTransition</var>.<code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code></code> fulfills.

6.  Current state captured as the “new” state.

7.  Transition pseudo-elements created. See [§ 3.2 View Transition Pseudo-elements](#view-transition-pseudos) for an overview of this structure.

8.  Rendering unpaused, revealing the transition pseudo-elements.

9.  <a id="ref-for-dom-viewtransition-ready"></a>

    <code><var>viewTransition</var>.<code><a href="#dom-viewtransition-ready">ready</a></code></code> fulfills.

10. Pseudo-elements animate until finished.

11. Transition pseudo-elements removed.

12. <a id="ref-for-dom-viewtransition-finished"></a>

    <code><var>viewTransition</var>.<code><a href="#dom-viewtransition-finished">finished</a></code></code> fulfills.

<a id="phases-diagram"></a>

[Embedded iframe resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phases/phases.html)

Previous Next

### <a id="transitions-as-enhancements"></a>1.4. Transitions as an enhancement

<a id="ref-for-callbackdef-updatecallback②"></a>

A key part of the View Transition API design is that an animated transition is a visual <em>enhancement</em> to an underlying document state change. That means a failure to create a visual transition, which can happen due to misconfiguration or device constraints, will not prevent the developer’s <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code> being called, even if it’s known in advance that the transition animations cannot happen.

<a id="ref-for-dom-viewtransition-skiptransition"></a>

<a id="ref-for-view-transition-tree"></a>

<a id="ref-for-callbackdef-updatecallback③"></a>

For example, if the developer calls <code><a href="#dom-viewtransition-skiptransition">skipTransition()</a></code> at the start of the [view transition lifecycle](#lifecycle), the steps relating to the animated transition, such as creating the [view transition tree](#view-transition-tree), will not happen. However, the <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code> will still be called. It’s only the visual transition that’s skipped, not the underlying state change.

<a id="ref-for-navigateevent"></a>

<a id="ref-for-ref-for-dom-navigateevent-signal①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the DOM change should also be skipped, then that needs to be handled by another feature. <code><code><a href="https://wicg.github.io/navigation-api/#navigateevent">navigateEvent</a></code>.<code><a href="https://wicg.github.io/navigation-api/#ref-for-dom-navigateevent-signal%E2%91%A0">signal</a></code></code> is an example of a feature developers could use to handle this.

<a id="ref-for-callbackdef-updatecallback④"></a>

Although the View Transition API allows DOM changes to be asynchronous via the <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code>, the API is not responsible for queuing or otherwise scheduling DOM changes beyond any scheduling needed for the transition itself. Some asynchronous DOM changes can happen concurrently (e.g if they’re happening within independent components), whereas others need to queue, or abort an earlier change. This is best left to a feature or framework that has a more holistic view of the application.

### <a id="rendering-model"></a>1.5. Rendering Model

<a id="ref-for-propdef-filter"></a>

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-propdef-clip-path"></a>

<a id="ref-for-capture-the-image"></a>

View Transition works by replicating an element’s rendered state using UA generated pseudo-elements. Aspects of the element’s rendering which apply to the element itself or its descendants, for example visual effects like [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter) or [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) and clipping from [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) or [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path), are applied when generating its image in [Capture the image](#capture-the-image).

<a id="ref-for-propdef-mix-blend-mode"></a>

<a id="ref-for-selectordef-view-transition-group"></a>

However, properties like [mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode) which define how the element draws when it is embedded can’t be applied to its image. Such properties are applied to the element’s corresponding [::view-transition-group()](#selectordef-view-transition-group) pseudo-element, which is meant to generate a box equivalent to the element.

<a id="ref-for-selectordef-view-transition-group①"></a>

If the [::view-transition-group()](#selectordef-view-transition-group) has a corresponding element in the "new" states, the browser keeps the properties copied over to the <a id="ref-for-selectordef-view-transition-group②"></a>::view-transition-group() in sync with the DOM element in the "new" state. If the <a id="ref-for-selectordef-view-transition-group③"></a>::view-transition-group() has a corresponding both in the "old" and "new" state, and the property being copied is interpolatable, the browser also sets up a default animation to animate the property smoothly.

### <a id="examples"></a>1.6. Examples

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6d7fc030"></a> Taking a page that already updates its content using a pattern like this:
>
> ```js
> function spaNavigate(data) {
>   updateTheDOMSomehow(data);
> }
> ```
>
> <a id="ref-for-view-transitions③"></a>
>
> A [view transition](#view-transitions) could be added like this:
>
> ```js
> function spaNavigate(data) {
>   // Fallback for browsers that don't support this API:
>   if (!document.startViewTransition) {
>     updateTheDOMSomehow(data);
>     return;
>   }
> 
>   // With a transition:
>   document.startViewTransition(() => updateTheDOMSomehow(data));
> }
> ```
>
> This results in the default transition of a quick cross-fade:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/default.mp4)
>
> The cross-fade is achieved using CSS animations on a [tree of pseudo-elements](#view-transition-pseudos), so customizations can be made using CSS. For example:
>
> ```css
> ::view-transition-old(root),
> ::view-transition-new(root) {
>   animation-duration: 5s;
> }
> ```
>
> This results in a slower transition:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/slow.mp4)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5766389"></a> Building on the previous example, motion can be added:
>
> ```css
> @keyframes fade-in {
>   from { opacity: 0; }
> }
> 
> @keyframes fade-out {
>   to { opacity: 0; }
> }
> 
> @keyframes slide-from-right {
>   from { transform: translateX(30px); }
> }
> 
> @keyframes slide-to-left {
>   to { transform: translateX(-30px); }
> }
> 
> ::view-transition-old(root) {
>   animation: 90ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
>     300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-left;
> }
> 
> ::view-transition-new(root) {
>   animation: 210ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in,
>     300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
> }
> ```
>
> Here’s the result:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/slide.mp4)

<a id="ref-for-selectordef-view-transition-group④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4377639b"></a> Building on the previous example, the header and text within the header can be given their own [::view-transition-group()](#selectordef-view-transition-group)s for the transition:
>
> ```css
> .main-header {
>   view-transition-name: main-header;
> }
> 
> .main-header-text {
>   view-transition-name: main-header-text;
>   /* Give the element a consistent size, assuming identical text: */
>   width: fit-content;
> }
> ```
>
> By default, these groups will transition size and position from their “old” to “new” state, while their visual states cross-fade:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/header.mp4)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6b9396d2"></a> Building on the previous example, let’s say some pages have a sidebar:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/bad-sidebar.mp4)
>
> In this case, things would look better if the sidebar was static if it was in both the “old” and “new” states. Otherwise, it should animate in or out.
>
> <a id="ref-for-only-child-pseudo"></a>
>
> The [:only-child](https://www.w3.org/TR/selectors-4/#only-child-pseudo) pseudo-class can be used to create animations specifically for these states:
>
> ```css
> .sidebar {
>   view-transition-name: sidebar;
> }
> 
> @keyframes slide-to-right {
>   to { transform: translateX(30px); }
> }
> 
> /* Entry transition */
> ::view-transition-new(sidebar):only-child {
>   animation: 300ms cubic-bezier(0, 0, 0.2, 1) both fade-in,
>     300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
> }
> 
> /* Exit transition */
> ::view-transition-old(sidebar):only-child {
>   animation: 150ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
>     300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-right;
> }
> ```
>
> For cases where the sidebar has both an “old” and “new” state, the default animation is correct.
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/good-sidebar.mp4)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3e1ac75e"></a> Not building from previous examples this time, let’s say we wanted to create a circular reveal from the user’s cursor. This can’t be done with CSS alone.
>
> Firstly, in the CSS, allow the “old” and “new” states to layer on top of one another without the default blending, and prevent the default cross-fade animation:
>
> ```css
> ::view-transition-image-pair(root) {
>   isolation: auto;
> }
> 
> ::view-transition-old(root),
> ::view-transition-new(root) {
>   animation: none;
>   mix-blend-mode: normal;
> }
> ```
>
> Then, the JavaScript:
>
> ```js
> // Store the last click event
> let lastClick;
> addEventListener('click', event => (lastClick = event));
> 
> function spaNavigate(data) {
>   // Fallback for browsers that don't support this API:
>   if (!document.startViewTransition) {
>     updateTheDOMSomehow(data);
>     return;
>   }
> 
>   // Get the click position, or fallback to the middle of the screen
>   const x = lastClick?.clientX ?? innerWidth / 2;
>   const y = lastClick?.clientY ?? innerHeight / 2;
>   // Get the distance to the furthest corner
>   const endRadius = Math.hypot(
>     Math.max(x, innerWidth - x),
>     Math.max(y, innerHeight - y)
>   );
> 
>   // Create a transition:
>   const transition = document.startViewTransition(() => {
>     updateTheDOMSomehow(data);
>   });
> 
>   // Wait for the pseudo-elements to be created:
>   transition.ready.then(() => {
>     // Animate the root's new view
>     document.documentElement.animate(
>       {
>         clipPath: [
>           \`circle(0 at ${x}px ${y}px)\`,
>           \`circle(${endRadius}px at ${x}px ${y}px)\`,
>         ],
>       },
>       {
>         duration: 500,
>         easing: 'ease-in',
>         // Specify which pseudo-element to animate
>         pseudoElement: '::view-transition-new(root)',
>       }
>     );
>   });
> }
> ```
>
> And here’s the result:
>
> [Embedded video resource](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/videos/circle.mp4)

## <a id="css-properties"></a>2. CSS properties

<a id="ref-for-propdef-view-transition-name①"></a>

### <a id="view-transition-name-prop"></a>2.1. Tagging Individually Transitioning Subtrees: the [view-transition-name](#propdef-view-transition-name) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-view-transition-name"></a>view-transition-name

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-comb-one"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

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

discrete

<a id="ref-for-propdef-view-transition-name②"></a>

<a id="ref-for-discrete"></a>

<a id="ref-for-timeline"></a>

<a id="ref-for-scroll-driven-animations"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: though [view-transition-name](#propdef-view-transition-name) is [discretely animatable](https://www.w3.org/TR/web-animations-1/#discrete), animating it doesn’t affect the running view transition. Rather, it’s a way to set its value in a way that can change over time or based on a [timeline](https://www.w3.org/TR/web-animations-1/#timeline). An example for using this would be to change the <a id="ref-for-propdef-view-transition-name③"></a>view-transition-name based on [scroll-driven animations](https://www.w3.org/TR/scroll-animations-1/#scroll-driven-animations).

<a id="ref-for-propdef-view-transition-name④"></a>

<a id="ref-for-captured-in-a-view-transition"></a>

<a id="ref-for-view-transition-tree①"></a>

The [view-transition-name](#propdef-view-transition-name) property “tags” an element for [capture in a view transition](#captured-in-a-view-transition), tracking it independently in the [view transition tree](#view-transition-tree) under the specified <a id="view-transition-name"></a>view transition name. An element so captured is animated independently of the rest of the page.

<a id="valdef-view-transition-name-none"></a>none

<a id="ref-for-element"></a>

The [element](https://drafts.csswg.org/css2/#element) will not participate independently in a view transition.

<a id="ref-for-identifier-value①"></a>

<a id="valdef-view-transition-name-custom-ident"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

<a id="ref-for-element①"></a>

<a id="ref-for-view-transition-name"></a>

The [element](https://drafts.csswg.org/css2/#element) participates independently in a view transition—​as either an old or new <a id="ref-for-element②"></a>element—​with the specified [view transition name](#view-transition-name).

<a id="ref-for-identifier-value②"></a>

The values none and auto are excluded from [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) here.

<a id="ref-for-view-transition-name①"></a>

<a id="ref-for-view-transitions④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If this name is not unique (i.e. if two elements simultaneously specify the same [view transition name](#view-transition-name)) then the [view transition](#view-transitions) will abort.

<a id="ref-for-view-transition-name②"></a>

<a id="ref-for-view-transition-tree②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the purposes of this API, if one element has [view transition name](#view-transition-name) foo in the old state, and another element has <a id="ref-for-view-transition-name③"></a>view transition name foo in the new state, they are treated as representing different visual state of the same element, and will be paired in the [view transition tree](#view-transition-tree). This may be confusing, since the elements themselves are not necessarily referring to the same object, but it is a useful model to consider them to be visual states of the same conceptual page entity.

<a id="ref-for-principal-box"></a>

<a id="ref-for-fragment"></a>

<a id="ref-for-skips-its-contents"></a>

<a id="ref-for-element-not-rendered"></a>

If the element’s [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is [fragmented](https://www.w3.org/TR/css-break-4/#fragment), [skipped](https://www.w3.org/TR/css-contain-2/#skips-its-contents), or [not rendered](https://www.w3.org/TR/css-images-4/#element-not-rendered), this property has no effect. See [§ 7 Algorithms](#algorithms) for exact details.

#### <a id="named-and-transitioning"></a>2.1.1. Rendering Consolidation

<a id="ref-for-element③"></a>

<a id="ref-for-captured-in-a-view-transition①"></a>

<a id="ref-for-view-transitions⑤"></a>

<a id="ref-for-propdef-view-transition-name⑤"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-valdef-view-transition-name-none"></a>

[Elements](https://drafts.csswg.org/css2/#element) [captured in a view transition](#captured-in-a-view-transition) during a [view transition](#view-transitions) or whose [view-transition-name](#propdef-view-transition-name) [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is not [none](#valdef-view-transition-name-none) (at any time):

- <a id="ref-for-x43"></a>

  Form a [stacking context](https://www.w3.org/TR/CSS21/visuren.html#x43).

- Are [flattened in 3D transforms](https://www.w3.org/TR/css-transforms-2/#grouping-property-values).

- <a id="ref-for-backdrop-root"></a>

  Form a [backdrop root](https://drafts.fxtf.org/filter-effects-2/#backdrop-root).

## <a id="pseudo"></a>3. Pseudo-elements

### <a id="pseudo-root"></a>3.1. Pseudo-element Trees

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is a general definition for trees of pseudo-elements. If other features need this behavior, these definitions will be moved to [\[css-pseudo-4\]](#biblio-css-pseudo-4).

<a id="ref-for-tree-abiding"></a>

<a id="ref-for-concept-tree-root"></a>

<a id="ref-for-concept-tree"></a>

A <a id="pseudo-element-root"></a>pseudo-element root is a type of [tree-abiding pseudo-element](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) that is the [root](https://dom.spec.whatwg.org/#concept-tree-root) in a [tree](https://dom.spec.whatwg.org/#concept-tree) of <a id="ref-for-tree-abiding①"></a>tree-abiding pseudo-elements, known as the <a id="pseudo-element-tree"></a>pseudo-element tree.

<a id="ref-for-pseudo-element-tree"></a>

<a id="ref-for-concept-tree-descendant"></a>

<a id="ref-for-tree-abiding②"></a>

The [pseudo-element tree](#pseudo-element-tree) defines the document order of its [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) [tree-abiding pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#tree-abiding).

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-concept-tree-participate"></a>

<a id="ref-for-pseudo-element-tree①"></a>

<a id="ref-for-originating-pseudo-element"></a>

<a id="ref-for-concept-tree-parent"></a>

When a [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) [participates](https://dom.spec.whatwg.org/#concept-tree-participate) in a [pseudo-element tree](#pseudo-element-tree), its [originating pseudo-element](https://www.w3.org/TR/selectors-4/#originating-pseudo-element) is its [parent](https://dom.spec.whatwg.org/#concept-tree-parent).

<a id="ref-for-concept-tree-descendant①"></a>

<a id="ref-for-pseudo-element-root"></a>

<a id="ref-for-concept-tree-sibling"></a>

<a id="ref-for-only-child-pseudo①"></a>

If a [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>pseudo</var> of a [pseudo-element root](#pseudo-element-root) has no other [siblings](https://dom.spec.whatwg.org/#concept-tree-sibling), then [:only-child](https://www.w3.org/TR/selectors-4/#only-child-pseudo) matches that <var>pseudo</var>.

<a id="ref-for-concept-tree-child"></a>

<a id="ref-for-concept-tree-sibling①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that `::view-transition-new(ident):only-child` will only select `::view-transition-new(ident)` if the parent `::view-transition-image-pair(ident)` contains a single [child](https://dom.spec.whatwg.org/#concept-tree-child). As in, there is no [sibling](https://dom.spec.whatwg.org/#concept-tree-sibling) `::view-transition-old(ident)`.

### <a id="view-transition-pseudos"></a>3.2. View Transition Pseudo-elements

<a id="ref-for-view-transitions⑥"></a>

<a id="ref-for-pseudo-element-tree②"></a>

<a id="ref-for-setup-transition-pseudo-elements"></a>

<a id="ref-for-selectordef-view-transition"></a>

<a id="ref-for-originating-element"></a>

<a id="ref-for-root-element"></a>

<a id="ref-for-view-transition-pseudo-elements"></a>

<a id="ref-for-ultimate-originating-element"></a>

<a id="ref-for-document-element"></a>

The visualization of a [view transition](#view-transitions) is represented as a [pseudo-element tree](#pseudo-element-tree) called the <a id="view-transition-tree"></a>view transition tree composed of the <a id="view-transition-pseudo-elements"></a>view transition pseudo-elements defined below. This tree is built during the [setup transition pseudo-elements](#setup-transition-pseudo-elements) step, and is rooted under a [::view-transition](#selectordef-view-transition) pseudo-element [originating](https://www.w3.org/TR/selectors-4/#originating-element) from the [root element](https://www.w3.org/TR/css-display-3/#root-element). All of the [view transition pseudo-elements](#view-transition-pseudo-elements) are selected from their [ultimate originating element](https://www.w3.org/TR/selectors-4/#ultimate-originating-element), the [document element](https://dom.spec.whatwg.org/#document-element).

<a id="ref-for-view-transition-tree③"></a>

The [view transition tree](#view-transition-tree) is not exposed to the accessibility tree.

<a id="ref-for-selectordef-view-transition-group⑤"></a>

<a id="ref-for-selectordef-view-transition①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f80a7b18"></a> For example, the [::view-transition-group()](#selectordef-view-transition-group) pseudo-element is attached to the root element selector directly, as in :root::view-transition-group(); it is not attached to its parent, the [::view-transition](#selectordef-view-transition) pseudo-element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Once the user-agent has captured both the “old” and “new” states of the document, it creates a structure of pseudo-elements like the following:
>
> ```text
> ::view-transition
> ├─ ::view-transition-group(name)
> │  └─ ::view-transition-image-pair(name)
> │     ├─ ::view-transition-old(name)
> │     └─ ::view-transition-new(name)
> └─ …other groups…
> ```
>
> <a id="ref-for-propdef-view-transition-name⑥"></a>
>
> <a id="ref-for-selectordef-view-transition-group⑥"></a>
>
> Each element with a [view-transition-name](#propdef-view-transition-name) is captured separately, and a [::view-transition-group()](#selectordef-view-transition-group) is created for each unique <a id="ref-for-propdef-view-transition-name⑦"></a>view-transition-name.
>
> <a id="ref-for-document-element①"></a>
>
> <a id="ref-for-propdef-view-transition-name⑧"></a>
>
> For convenience, the [document element](https://dom.spec.whatwg.org/#document-element) is given the [view-transition-name](#propdef-view-transition-name) "root" in the [user-agent style sheet](#ua-styles).
>
> <a id="ref-for-selectordef-view-transition-old"></a>
>
> <a id="ref-for-selectordef-view-transition-new"></a>
>
> Either [::view-transition-old()](#selectordef-view-transition-old) or [::view-transition-new()](#selectordef-view-transition-new) are absent in cases where the capture does not have an “old” or “new” state.
>
> Each of the pseudo-elements generated can be targeted by CSS in order to customize its appearance, behavior and/or add animations. This enables full customization of the transition.

#### <a id="named-view-transition-pseudo"></a>3.2.1. Named View Transition Pseudo-elements

<a id="ref-for-view-transition-pseudo-elements①"></a>

<a id="ref-for-functional-pseudo-element"></a>

<a id="ref-for-tree-abiding③"></a>

<a id="ref-for-view-transition-name④"></a>

<a id="ref-for-typedef-pt-name-selector"></a>

Several of the [view transition pseudo-elements](#view-transition-pseudo-elements) are <a id="named-view-transition-pseudo-elements"></a>named view transition pseudo-elements, which are [functional](https://drafts.csswg.org/selectors-4/#functional-pseudo-element) [tree-abiding](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) <a id="ref-for-view-transition-pseudo-elements②"></a>view transition pseudo-elements associated with a [view transition name](#view-transition-name). These pseudo-elements take a [\<pt-name-selector\>](#typedef-pt-name-selector) as their argument, and their syntax follows the pattern:

<a id="ref-for-typedef-pt-name-selector①"></a>

```text
::view-transition-pseudo(<pt-name-selector>)
```
<a id="ref-for-typedef-pt-name-selector②"></a>

<a id="ref-for-view-transition-name⑤"></a>

where [\<pt-name-selector\>](#typedef-pt-name-selector) selects a [view transition name](#view-transition-name), and has the following syntax definition:

<a id="typedef-pt-name-selector"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-identifier-value③"></a>

```text
<pt-name-selector> = '*' | <custom-ident>
```
<a id="ref-for-named-view-transition-pseudo-elements"></a>

<a id="ref-for-selector"></a>

<a id="ref-for-pseudo-element①"></a>

<a id="ref-for-typedef-pt-name-selector③"></a>

<a id="ref-for-view-transition-name⑥"></a>

<a id="ref-for-x"></a>

<a id="ref-for-identifier-value④"></a>

A [named view transition pseudo-element](#named-view-transition-pseudo-elements) [selector](https://www.w3.org/TR/selectors-4/#selector) only matches a corresponding [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) if its [\<pt-name-selector\>](#typedef-pt-name-selector) matches that <a id="ref-for-pseudo-element②"></a>pseudo-element’s [view transition name](#view-transition-name), i.e. if it is either [\*](https://www.w3.org/TR/selectors-3/#x) or a matching [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value).

<a id="ref-for-view-transition-name⑦"></a>

<a id="ref-for-view-transition-pseudo-elements③"></a>

<a id="ref-for-propdef-view-transition-name⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [view transition name](#view-transition-name) of a [view transition pseudo-element](#view-transition-pseudo-elements) is set to the [view-transition-name](#propdef-view-transition-name) that triggered its creation.

<a id="ref-for-named-view-transition-pseudo-elements①"></a>

<a id="ref-for-selector①"></a>

<a id="ref-for-identifier-value⑤"></a>

<a id="ref-for-type-selector"></a>

<a id="ref-for-x①"></a>

The specificity of a [named view transition pseudo-element](#named-view-transition-pseudo-elements) [selector](https://www.w3.org/TR/selectors-4/#selector) with a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) argument is equivalent to a [type selector](https://www.w3.org/TR/selectors-4/#type-selector). The specificity of a <a id="ref-for-named-view-transition-pseudo-elements②"></a>named view transition pseudo-element <a id="ref-for-selector②"></a>selector with a [\*](https://www.w3.org/TR/selectors-3/#x) argument is zero.

<a id="ref-for-selectordef-view-transition②"></a>

#### <a id="view-transition-pseudo"></a>3.2.2. View Transition Tree Root: the [::view-transition](#selectordef-view-transition) pseudo-element

<a id="ref-for-pseudo-element③"></a>

<a id="ref-for-tree-abiding④"></a>

<a id="ref-for-pseudo-element-root①"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-document-element②"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-snapshot-containing-block"></a>

The <a id="selectordef-view-transition"></a>::view-transition [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) is a [tree-abiding pseudo-element](https://www.w3.org/TR/css-pseudo-4/#tree-abiding) that is also a [pseudo-element root](#pseudo-element-root). Its [originating element](https://www.w3.org/TR/selectors-4/#originating-element) is the document’s [document element](https://dom.spec.whatwg.org/#document-element), and its [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is the [snapshot containing block](#snapshot-containing-block).

<a id="ref-for-concept-tree-parent①"></a>

<a id="ref-for-selectordef-view-transition-group⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This element serves as the [parent](https://dom.spec.whatwg.org/#concept-tree-parent) of all [::view-transition-group()](#selectordef-view-transition-group) pseudo-elements.

<a id="ref-for-selectordef-view-transition-group⑧"></a>

#### <a id="::view-transition-group"></a>3.2.3. View Transition Named Subtree Root: the [::view-transition-group()](#selectordef-view-transition-group) pseudo-element

<a id="ref-for-pseudo-element④"></a>

<a id="ref-for-named-view-transition-pseudo-elements③"></a>

<a id="ref-for-view-transitions⑦"></a>

<a id="ref-for-selectordef-view-transition-group⑨"></a>

<a id="ref-for-view-transition-name⑧"></a>

<a id="ref-for-concept-tree-child①"></a>

<a id="ref-for-selectordef-view-transition③"></a>

<a id="ref-for-selectordef-view-transition-image-pair"></a>

The <a id="selectordef-view-transition-group"></a>::view-transition-group() [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) is a [named view transition pseudo-element](#named-view-transition-pseudo-elements) that represents a matching named [view transition](#view-transitions) capture. A [::view-transition-group()](#selectordef-view-transition-group) <a id="ref-for-pseudo-element⑤"></a>pseudo-element is generated for each [view transition name](#view-transition-name) as a [child](https://dom.spec.whatwg.org/#concept-tree-child) of the [::view-transition](#selectordef-view-transition) <a id="ref-for-pseudo-element⑥"></a>pseudo-element, and contains a corresponding [::view-transition-image-pair()](#selectordef-view-transition-image-pair).

> <strong data-conversion-semantic="note">Note</strong>
>
> This element initially mirrors the size and position of the “old” element, or the “new” element if there isn’t an “old” element.
>
> <a id="ref-for-document-dynamic-view-transition-style-sheet"></a>
>
> <a id="ref-for-propdef-width"></a>
>
> <a id="ref-for-propdef-height"></a>
>
> <a id="ref-for-border-box"></a>
>
> If there’s both an “old” and “new” state, styles in the [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet) animate this pseudo-element’s [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) from the size of the old element’s [border box](https://www.w3.org/TR/css-box-4/#border-box) to that of the new element’s <a id="ref-for-border-box①"></a>border box.
>
> <a id="ref-for-propdef-transform"></a>
>
> Also the element’s [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) is animated from the old element’s screen space transform to the new element’s screen space transform.
>
> This style is generated dynamically since the values of animated properties are determined at the time that the transition begins.

<a id="ref-for-selectordef-view-transition-image-pair①"></a>

#### <a id="::view-transition-image-pair"></a>3.2.4. View Transition Image Pair Isolation: the [::view-transition-image-pair()](#selectordef-view-transition-image-pair) pseudo-element

<a id="ref-for-pseudo-element⑦"></a>

<a id="ref-for-named-view-transition-pseudo-elements④"></a>

<a id="ref-for-view-transitions⑧"></a>

<a id="ref-for-concept-tree-child②"></a>

<a id="ref-for-selectordef-view-transition-group①⓪"></a>

<a id="ref-for-selectordef-view-transition-old①"></a>

<a id="ref-for-selectordef-view-transition-new①"></a>

The <a id="selectordef-view-transition-image-pair"></a>::view-transition-image-pair() [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) is a [named view transition pseudo-element](#named-view-transition-pseudo-elements) that represents a pair of corresponding old/new [view transition](#view-transitions) captures. This pseudo-element is a [child](https://dom.spec.whatwg.org/#concept-tree-child) of the corresponding [::view-transition-group()](#selectordef-view-transition-group) pseudo-element and contains a corresponding [::view-transition-old()](#selectordef-view-transition-old) pseudo-element and/or a corresponding [::view-transition-new()](#selectordef-view-transition-new) pseudo-element (in that order).

<a id="ref-for-propdef-isolation"></a>

<a id="ref-for-concept-tree-child③"></a>

<a id="ref-for-selectordef-view-transition-group①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> This element exists to provide [isolation: isolate](https://www.w3.org/TR/compositing-1/#propdef-isolation) for its children, and is always present as a [child](https://dom.spec.whatwg.org/#concept-tree-child) of each [::view-transition-group()](#selectordef-view-transition-group). This isolation allows the image pair to be blended with non-normal blend modes without affecting other visual outputs.

<a id="ref-for-selectordef-view-transition-old②"></a>

#### <a id="::view-transition-old"></a>3.2.5. View Transition Old State Image: the [::view-transition-old()](#selectordef-view-transition-old) pseudo-element

<a id="ref-for-pseudo-element⑧"></a>

<a id="ref-for-named-view-transition-pseudo-elements⑤"></a>

<a id="ref-for-replaced-element"></a>

<a id="ref-for-selectordef-view-transition-old③"></a>

<a id="ref-for-concept-tree-child④"></a>

<a id="ref-for-selectordef-view-transition-image-pair②"></a>

The <a id="selectordef-view-transition-old"></a>::view-transition-old() [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) is an empty [named view transition pseudo-element](#named-view-transition-pseudo-elements) that represents a visual snapshot of the “old” state as a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element); it is omitted if there’s no “old” state to represent. Each [::view-transition-old()](#selectordef-view-transition-old) pseudo-element is a [child](https://dom.spec.whatwg.org/#concept-tree-child) of the corresponding [::view-transition-image-pair()](#selectordef-view-transition-image-pair) pseudo-element.

<a id="ref-for-only-child-pseudo②"></a>

<a id="ref-for-selectordef-view-transition-image-pair③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> [:only-child](https://www.w3.org/TR/selectors-4/#only-child-pseudo) can be used to match cases where this element is the only element in the [::view-transition-image-pair()](#selectordef-view-transition-image-pair).
>
> The appearance of this element can be manipulated with `object-*` properties in the same way that other replaced elements can be.

<a id="ref-for-natural-dimensions"></a>

<a id="ref-for-capture-the-image①"></a>

<a id="ref-for-setup-transition-pseudo-elements①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The content and [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions) of the image are captured in [capture the image](#capture-the-image), and set in [setup transition pseudo-elements](#setup-transition-pseudo-elements).

<a id="ref-for-document-dynamic-view-transition-style-sheet①"></a>

<a id="ref-for-setup-transition-pseudo-elements②"></a>

<a id="ref-for-update-pseudo-element-styles"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Additional styles in the [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet) added to animate these pseudo-elements are detailed in [setup transition pseudo-elements](#setup-transition-pseudo-elements) and [update pseudo-element styles](#update-pseudo-element-styles).

<a id="ref-for-selectordef-view-transition-new②"></a>

#### <a id="::view-transition-new"></a>3.2.6. View Transition New State Image: the [::view-transition-new()](#selectordef-view-transition-new) pseudo-element

<a id="ref-for-pseudo-element⑨"></a>

<a id="ref-for-selectordef-view-transition-old④"></a>

<a id="ref-for-named-view-transition-pseudo-elements⑥"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-selectordef-view-transition-new③"></a>

<a id="ref-for-concept-tree-child⑤"></a>

<a id="ref-for-selectordef-view-transition-image-pair④"></a>

The <a id="selectordef-view-transition-new"></a>::view-transition-new() [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) (like the analogous [::view-transition-old()](#selectordef-view-transition-old) pseudo-element) is an empty [named view transition pseudo-element](#named-view-transition-pseudo-elements) that represents a visual snapshot of the “new” state as a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element); it is omitted if there’s no “new” state to represent. Each [::view-transition-new()](#selectordef-view-transition-new) pseudo-element is a [child](https://dom.spec.whatwg.org/#concept-tree-child) of the corresponding [::view-transition-image-pair()](#selectordef-view-transition-image-pair) pseudo-element.

<a id="ref-for-natural-dimensions①"></a>

<a id="ref-for-capture-the-image②"></a>

<a id="ref-for-setup-transition-pseudo-elements③"></a>

<a id="ref-for-update-pseudo-element-styles①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The content and [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions) of the image are captured in [capture the image](#capture-the-image), then set and updated in [setup transition pseudo-elements](#setup-transition-pseudo-elements) and [update pseudo-element styles](#update-pseudo-element-styles).

## <a id="view-transition-rendering"></a>4. View Transition Layout

<a id="ref-for-view-transition-pseudo-elements④"></a>

<a id="ref-for-snapshot-containing-block①"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-view-transition-layer"></a>

The [view transition pseudo-elements](#view-transition-pseudo-elements) are styled, laid out, and rendered like normal elements, except that they originate in the [snapshot containing block](#snapshot-containing-block) rather than the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block) and are painted in the [view transition layer](#view-transition-layer) above the rest of the document.

### <a id="snapshot-containing-block-concept"></a>4.1. The Snapshot Containing Block

<a id="ref-for-interactive-widget"></a>

<a id="ref-for-document-element③"></a>

<a id="ref-for-captured-element-old-image"></a>

<a id="ref-for-captured-element-new-element"></a>

The <a id="snapshot-containing-block"></a>snapshot containing block is a rectangle that covers all areas of the window that could potentially display page content (and is therefore consistent regardless of root scrollbars or [interactive widgets](https://www.w3.org/TR/css-viewport-1/#interactive-widget)). This makes it likely to be consistent for the [document element](https://dom.spec.whatwg.org/#document-element)'s [old image](#captured-element-old-image) and [new element](#captured-element-new-element).

<a id="ref-for-snapshot-containing-block②"></a>

<a id="ref-for-initial-containing-block①"></a>

For iframes, the [snapshot containing block](#snapshot-containing-block) corresponds to its [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block).

![A diagram of a phone screen, including a top status bar, a browser URL bar, web-content area with a floating scrollbar, a virtual keyboard, and a bottom bar with an OS back button](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser.svg) ![The previous diagram, but highlights the area that's the 'snapshot containing block', which includes everything except the top status bar and the bottom bar with the OS back button](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser-snapshot-root.svg)

<a id="ref-for-snapshot-containing-block③"></a>

An example of the [snapshot containing block](#snapshot-containing-block) on a mobile OS. The snapshot includes the URL bar, as this can be scrolled away. The keyboard is included as this appears and disappears. The top and bottom bars are part of the OS rather than the browser, so they’re not included in the snapshot containing block.

![A diagram of a desktop browser window, including a tab bar, a URL bar, and a web-content area featuring both horizontal and vertical scrollbars](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/desktop-browser.svg) ![The previous diagram, but highlights the area that's the 'snapshot containing block', which includes the web content area and the scrollbars](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/desktop-browser-snapshot-root.svg)

<a id="ref-for-snapshot-containing-block④"></a>

An example of the [snapshot containing block](#snapshot-containing-block) on a desktop OS. This includes the scrollbars, but does not include the URL bar, as web content never appears in that area.

<a id="ref-for-snapshot-containing-block⑤"></a>

The <a id="snapshot-containing-block-origin"></a>snapshot containing block origin refers to the top-left corner of the [snapshot containing block](#snapshot-containing-block).

<a id="ref-for-snapshot-containing-block⑥"></a>

<a id="ref-for-tuple"></a>

The <a id="snapshot-containing-block-size"></a>snapshot containing block size refers to the width and height of the [snapshot containing block](#snapshot-containing-block) as a [tuple](https://infra.spec.whatwg.org/#tuple) of two numbers.

<a id="ref-for-snapshot-containing-block⑦"></a>

<a id="ref-for-absolute-positioning-containing-block"></a>

<a id="ref-for-fixed-positioning-containing-block"></a>

<a id="ref-for-selectordef-view-transition④"></a>

The [snapshot containing block](#snapshot-containing-block) is considered to be an [absolute positioning containing block](https://www.w3.org/TR/css-position-3/#absolute-positioning-containing-block) and a [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block) for [::view-transition](#selectordef-view-transition) and its descendants.

### <a id="view-transition-stacking-layer"></a>4.2. View Transition Painting Order

<a id="ref-for-view-transition-layer①"></a>

This specification introduces a new stacking layer, the [view transition layer](#view-transition-layer), to the end of the painting order established in [CSS2§E Elaborate Description of Stacking Contexts](https://www.w3.org/TR/CSS22/zindex.html). [\[CSS2\]](#biblio-css2)

<a id="ref-for-selectordef-view-transition⑤"></a>

<a id="ref-for-document-top-layer"></a>

<a id="ref-for-selectordef-view-transition-old⑤"></a>

<a id="ref-for-selectordef-view-transition-new④"></a>

The [::view-transition](#selectordef-view-transition) pseudo-element generates a new stacking context, called the <a id="view-transition-layer"></a>view transition layer, which paints after all other content of the document (including any content rendered in the [top layer](https://drafts.csswg.org/css-position-4/#document-top-layer)), after any filters and effects that are applied to such content. (It is not subject to such filters or effects, except insofar as they affect the rendered contents of the [::view-transition-old()](#selectordef-view-transition-old) and [::view-transition-new()](#selectordef-view-transition-new) pseudo-elements.)

<a id="ref-for-view-transition-layer②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The intent of the feature is to be able to capture the contents of the page, which includes the top layer elements. In order to accomplish that, the [view transition layer](#view-transition-layer) cannot be a part of the captured stacking contexts, since that results in a circular dependency. Therefore, the <a id="ref-for-view-transition-layer③"></a>view transition layer is a sibling of all other content.

<a id="ref-for-document"></a>

<a id="ref-for-document-active-view-transition"></a>

<a id="ref-for-viewtransition-phase"></a>

<a id="ref-for-document①"></a>

<a id="ref-for-captured-in-a-view-transition②"></a>

<a id="ref-for-element-contents"></a>

<a id="ref-for-viewtransition-transition-root-pseudo-element"></a>

<a id="ref-for-concept-tree-inclusive-descendant"></a>

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-pointer-events"></a>

When a <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>'s [active view transition](#document-active-view-transition)'s [phase](#viewtransition-phase) is "`animating`", the boxes generated by any element in that <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> with [captured in a view transition](#captured-in-a-view-transition) and its [element contents](https://drafts.csswg.org/css-contain-2/#element-contents), except [transition root pseudo-element](#viewtransition-transition-root-pseudo-element)'s [inclusive descendants](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant), are not painted (as if they had [visibility: hidden](https://www.w3.org/TR/css-display-3/#propdef-visibility)) and do not respond to hit-testing (as if they had [pointer-events: none](https://drafts.csswg.org/css-ui-4/#propdef-pointer-events)).

<a id="ref-for-selectordef-view-transition-new⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Elements participating in a transition need to skip painting in their DOM location because their image is painted in the corresponding [::view-transition-new()](#selectordef-view-transition-new) pseudo-element instead. Similarly, hit-testing is skipped because the element’s DOM location does not correspond to where its contents are rendered. However, there is no change in how these elements are accessed by assistive technologies or the accessibility tree.

## <a id="ua-styles"></a>5. User Agent Stylesheet

<a id="ref-for-cascade-origin-ua"></a>

The <a id="global-view-transition-user-agent-style-sheet"></a>global view transition user agent style sheet is a [user-agent origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua) style sheet containing the following rules:

```css
:root {
  view-transition-name: root;
}

:root::view-transition {
  position: fixed;
  inset: 0;
}

:root::view-transition-group(*) {
  position: absolute;
  top: 0;
  left: 0;

  animation-duration: 0.25s;
  animation-fill-mode: both;
}

:root::view-transition-image-pair(*) {
  position: absolute;
  inset: 0;

  animation-duration: inherit;
  animation-fill-mode: inherit;
  animation-delay: inherit;
}

:root::view-transition-old(*),
:root::view-transition-new(*) {
  position: absolute;
  inset-block-start: 0;
  inline-size: 100%;
  block-size: auto;

  animation-duration: inherit;
  animation-fill-mode: inherit;
  animation-delay: inherit;
}

/* Default cross-fade transition */
@keyframes -ua-view-transition-fade-out {
  to { opacity: 0; }
}
@keyframes -ua-view-transition-fade-in {
  from { opacity: 0; }
}

/* Keyframes for blending when there are 2 images */
@keyframes -ua-mix-blend-mode-plus-lighter {
  from { mix-blend-mode: plus-lighter }
  to { mix-blend-mode: plus-lighter }
}
```
> <strong data-conversion-semantic="note">Note</strong>
>
> Explanatory Summary
>
> This UA style sheet does several things:
>
> - <a id="ref-for-selectordef-view-transition⑥"></a>
>
>   <a id="ref-for-snapshot-containing-block⑧"></a>
>
>   Lay out [::view-transition](#selectordef-view-transition) to cover the entire [snapshot containing block](#snapshot-containing-block) so that each :view-transition-group() child can lay out relative to it.
>
> - <a id="ref-for-root-element①"></a>
>
>   <a id="ref-for-view-transition-name⑨"></a>
>
>   Give the [root element](https://www.w3.org/TR/css-display-3/#root-element) a default [view transition name](#view-transition-name), to allow it to be independently selected.
>
> - <a id="ref-for-selectordef-view-transition-image-pair⑤"></a>
>
>   <a id="ref-for-selectordef-view-transition-old⑥"></a>
>
>   <a id="ref-for-selectordef-view-transition-new⑥"></a>
>
>   <a id="ref-for-selectordef-view-transition-group①②"></a>
>
>   Reduce layout interference from the [::view-transition-image-pair()](#selectordef-view-transition-image-pair) pseudo-element so that authors can essentially treat [::view-transition-old()](#selectordef-view-transition-old) and [::view-transition-new()](#selectordef-view-transition-new) as direct children of [::view-transition-group()](#selectordef-view-transition-group) for most purposes.
>
> - <a id="ref-for-selectordef-view-transition-group①③"></a>
>
>   Inherit animation timing through the tree so that by default, the animation timing set on a [::view-transition-group()](#selectordef-view-transition-group) will dictate the animation timing of all its descendants.
>
> - <a id="ref-for-selectordef-view-transition-old⑦"></a>
>
>   <a id="ref-for-selectordef-view-transition-new⑦"></a>
>
>   <a id="ref-for-selectordef-view-transition-group①④"></a>
>
>   <a id="ref-for-document-dynamic-view-transition-style-sheet②"></a>
>
>   Style the element captures [::view-transition-old()](#selectordef-view-transition-old) and [::view-transition-new()](#selectordef-view-transition-new) to match the size and position set on [::view-transition-group()](#selectordef-view-transition-group) (insofar as possible without breaking their aspect ratios) as it interpolates between them. Since the sizing of these elements depends on the mapping between logical and physical coordinates, [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet) copies relevant styles from the DOM elements.
>
> - <a id="ref-for-selectordef-view-transition-group①⑤"></a>
>
>   Set up a default quarter-second cross-fade animation for each [::view-transition-group()](#selectordef-view-transition-group).

<a id="ref-for-cascade-origin-ua①"></a>

<a id="ref-for-view-transitions⑨"></a>

<a id="ref-for-document-dynamic-view-transition-style-sheet③"></a>

Additional styles are dynamically added to the [user-agent origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua) during a [view transition](#view-transitions) through the [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet).

## <a id="api"></a>6. API

<a id="ref-for-document②"></a>

### <a id="additions-to-document-api"></a>6.1. Additions to <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>

<a id="ref-for-document③"></a>

<a id="ref-for-viewtransition①"></a>

<a id="ref-for-dom-document-startviewtransition②"></a>

<a id="ref-for-callbackdef-updatecallback⑤"></a>

<a id="dom-document-startviewtransition-updatecallback-updatecallback"></a>

<a id="callbackdef-updatecallback"></a>

<a id="ref-for-idl-promise"></a>

<a id="ref-for-idl-any"></a>

```text
partial interface Document {
  ViewTransition startViewTransition(optional UpdateCallback updateCallback);
};

callback UpdateCallback = Promise<any> ();
```
<a id="ref-for-callbackdef-updatecallback⑥"></a>

<a id="ref-for-dom-document-startviewtransition③"></a>

<a id="ref-for-document④"></a>

<a id="ref-for-viewtransition②"></a>

<code><code><a href="#viewtransition">viewTransition</a></code> = <code><a href="https://dom.spec.whatwg.org/#document">document</a></code>.<code><a href="#dom-document-startviewtransition">startViewTransition</a></code>(<code><a href="#callbackdef-updatecallback">updateCallback</a></code>)</code>

<a id="ref-for-view-transitions①⓪"></a>

<a id="ref-for-document⑤"></a>

<a id="ref-for-document-active-view-transition①"></a>

Starts a new [view transition](#view-transitions) (canceling the <code><a href="https://dom.spec.whatwg.org/#document">document</a></code>’s existing [active view transition](#document-active-view-transition), if any).

<a id="ref-for-callbackdef-updatecallback⑦"></a>

<a id="ref-for-callbackdef-updatecallback⑧"></a>

<code><a href="#callbackdef-updatecallback">updateCallback</a></code>, if provided, is called asynchronously, once the current state of the document is captured. Then, when the promise returned by <code><a href="#callbackdef-updatecallback">updateCallback</a></code> fulfills, the new state of the document is captured and the transition is initiated.

<a id="ref-for-callbackdef-updatecallback⑨"></a>

Note that <code><a href="#callbackdef-updatecallback">updateCallback</a></code>, if provided, is <em>always</em> called, even if the transition cannot happen (e.g. due to duplicate `view-transition-name` values). The transition is an enhancement around the state change, so a failure to create a transition never prevents the state change. See [§ 1.4 Transitions as an enhancement](#transitions-as-enhancements) for more details on this principle.

<a id="ref-for-callbackdef-updatecallback①⓪"></a>

If the promise returned by <code><a href="#callbackdef-updatecallback">updateCallback</a></code> rejects, the transition is skipped.

<a id="ref-for-dom-document-startviewtransition④"></a>

#### <a id="ViewTransition-prepare"></a>6.1.1. <code><a href="#dom-document-startviewtransition">startViewTransition()</a></code> Method Steps

<a id="ref-for-method-steps"></a>

The [method steps](https://webidl.spec.whatwg.org/#method-steps) for <a id="dom-document-startviewtransition"></a><code>startViewTransition(<var>updateCallback</var>)</code> are as follows:

1.  <a id="ref-for-viewtransition③"></a>

    <a id="ref-for-this"></a>

    <a id="ref-for-concept-relevant-realm"></a>

    Let <var>transition</var> be a new <code><a href="#viewtransition">ViewTransition</a></code> object in [this’s](https://webidl.spec.whatwg.org/#this) [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

2.  <a id="ref-for-viewtransition-update-callback"></a>

    If <var>updateCallback</var> is provided, set <var>transition</var>’s [update callback](#viewtransition-update-callback) to <var>updateCallback</var>.

3.  <a id="ref-for-this①"></a>

    <a id="ref-for-concept-relevant-global"></a>

    <a id="ref-for-concept-document-window"></a>

    Let <var>document</var> be [this’s](https://webidl.spec.whatwg.org/#this) [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

4.  <a id="ref-for-visibility-state"></a>

    <a id="ref-for-skip-the-view-transition"></a>

    <a id="ref-for-invalidstateerror"></a>

    <a id="ref-for-idl-DOMException"></a>

    If <var>document</var>’s [visibility state](https://html.spec.whatwg.org/multipage/interaction.html#visibility-state) is "`hidden`", then [skip](#skip-the-view-transition) <var>transition</var> with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>, and return <var>transition</var>.

5.  <a id="ref-for-document-active-view-transition②"></a>

    <a id="ref-for-skip-the-view-transition①"></a>

    <a id="ref-for-aborterror"></a>

    <a id="ref-for-idl-DOMException①"></a>

    <a id="ref-for-this②"></a>

    <a id="ref-for-concept-relevant-realm①"></a>

    If <var>document</var>’s [active view transition](#document-active-view-transition) is not null, then [skip that view transition](#skip-the-view-transition) with an "<code><a href="https://webidl.spec.whatwg.org/#aborterror">AbortError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> in [this’s](https://webidl.spec.whatwg.org/#this) [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

    <a id="ref-for-viewtransition-update-callback①"></a>

    <a id="ref-for-document-active-view-transition③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This can result in two asynchronous [update callbacks](#viewtransition-update-callback) running concurrently (and therefore possibly out of sequence): one for the <var>document</var>’s current [active view transition](#document-active-view-transition), and another for this <var>transition</var>. As per the [design of this feature](#transitions-as-enhancements), it’s assumed that the developer is using another feature or framework to correctly schedule these DOM changes.

6.  <a id="ref-for-document-active-view-transition④"></a>

    Set <var>document</var>’s [active view transition](#document-active-view-transition) to <var>transition</var>.

    <a id="ref-for-view-transitions①①"></a>

    <a id="ref-for-setup-view-transition"></a>

    <a id="ref-for-perform-pending-transition-operations"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The [view transition](#view-transitions) process continues in [setup view transition](#setup-view-transition), via [perform pending transition operations](#perform-pending-transition-operations).

7.  Return <var>transition</var>.

<a id="ref-for-viewtransition④"></a>

### <a id="the-domtransition-interface"></a>6.2. The <code><a href="#viewtransition">ViewTransition</a></code> interface

<a id="ref-for-Exposed"></a>

<a id="viewtransition"></a>

<a id="ref-for-idl-promise①"></a>

<a id="ref-for-idl-undefined"></a>

<a id="dom-viewtransition-updatecallbackdone"></a>

<a id="ref-for-idl-promise②"></a>

<a id="ref-for-idl-undefined①"></a>

<a id="dom-viewtransition-ready"></a>

<a id="ref-for-idl-promise③"></a>

<a id="ref-for-idl-undefined②"></a>

<a id="dom-viewtransition-finished"></a>

<a id="ref-for-idl-undefined③"></a>

<a id="ref-for-dom-viewtransition-skiptransition①"></a>

```text
[Exposed=Window]
interface ViewTransition {
  readonly attribute Promise<undefined> updateCallbackDone;
  readonly attribute Promise<undefined> ready;
  readonly attribute Promise<undefined> finished;
  undefined skipTransition();
};
```
<a id="ref-for-viewtransition⑤"></a>

<a id="ref-for-view-transitions①②"></a>

The <code><a href="#viewtransition">ViewTransition</a></code> interface represents and controls a single same-document [view transition](#view-transitions), i.e. a transition where the starting and ending document are the same, possibly with changes to the document’s DOM structure.

<a id="ref-for-dom-viewtransition-updatecallbackdone①"></a>

<a id="ref-for-viewtransition⑥"></a>

<code><code><a href="#viewtransition">viewTransition</a></code>.<code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code></code>

<a id="ref-for-callbackdef-updatecallback①①"></a>

A promise that fulfills when the promise returned by <code><a href="#callbackdef-updatecallback">updateCallback</a></code> fulfills, or rejects when it rejects.

<a id="ref-for-dom-viewtransition-updatecallbackdone②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The View Transition API wraps a DOM change and creates a visual transition. However, sometimes you don’t care about the success/failure of the transition animation, you just want to know if and when the DOM change happens. <code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code> is for that use-case.)

<a id="ref-for-dom-viewtransition-ready①"></a>

<a id="ref-for-viewtransition⑦"></a>

<code><code><a href="#viewtransition">viewTransition</a></code>.<code><a href="#dom-viewtransition-ready">ready</a></code></code>

A promise that fulfills once the pseudo-elements for the transition are created, and the animation is about to start.

<a id="ref-for-dom-viewtransition-updatecallbackdone③"></a>

It rejects if the transition cannot begin. This can be due to misconfiguration, such as duplicate 'view-transition-name’s, or if <code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code> returns a rejected promise.

<a id="ref-for-dom-viewtransition-ready②"></a>

<a id="ref-for-view-transition-pseudo-elements⑤"></a>

The point that <code><a href="#dom-viewtransition-ready">ready</a></code> fulfills is the ideal opportunity to animate the [view transition pseudo-elements](#view-transition-pseudo-elements) with the [Web Animation API](https://www.w3.org/TR/web-animations-1/#extensions-to-the-element-interface).

<a id="ref-for-dom-viewtransition-finished①"></a>

<a id="ref-for-viewtransition⑧"></a>

<code><code><a href="#viewtransition">viewTransition</a></code>.<code><a href="#dom-viewtransition-finished">finished</a></code></code>

A promise that fulfills once the end state is fully visible and interactive to the user.

<a id="ref-for-callbackdef-updatecallback①②"></a>

It only rejects if <code><a href="#callbackdef-updatecallback">updateCallback</a></code> returns a rejected promise, as this indicates the end state wasn’t created.

<a id="ref-for-dom-viewtransition-skiptransition②"></a>

<a id="ref-for-dom-viewtransition-finished②"></a>

Otherwise, if a transition fails to begin, or is skipped (by <code><a href="#dom-viewtransition-skiptransition">skipTransition()</a></code>), the end state is still reached, so <code><a href="#dom-viewtransition-finished">finished</a></code> fulfills.

<a id="ref-for-dom-viewtransition-skiptransition③"></a>

<a id="ref-for-viewtransition⑨"></a>

<code><code><a href="#viewtransition">viewTransition</a></code>.<code><a href="#dom-viewtransition-skiptransition">skipTransition</a></code>()</code>

Immediately finish the transition, or prevent it starting.

<a id="ref-for-callbackdef-updatecallback①③"></a>

This never prevents <code><a href="#callbackdef-updatecallback">updateCallback</a></code> being called, as the DOM change is independent of the transition. See [§ 1.4 Transitions as an enhancement](#transitions-as-enhancements) for more details on this principle.

<a id="ref-for-dom-viewtransition-ready③"></a>

<a id="ref-for-dom-viewtransition-ready④"></a>

If this is called before <code><a href="#dom-viewtransition-ready">ready</a></code> resolves, <code><a href="#dom-viewtransition-ready">ready</a></code> will reject.

<a id="ref-for-dom-viewtransition-finished③"></a>

<a id="ref-for-dom-viewtransition-updatecallbackdone④"></a>

If <code><a href="#dom-viewtransition-finished">finished</a></code> hasn’t resolved, it will fulfill or reject along with <code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code>.

<a id="ref-for-viewtransition①⓪"></a>

A <code><a href="#viewtransition">ViewTransition</a></code> has the following:

<a id="viewtransition-named-elements"></a>named elements  
<a id="ref-for-ordered-map"></a>

<a id="ref-for-view-transition-name①⓪"></a>

<a id="ref-for-captured-element"></a>

<a id="ref-for-viewtransition①①"></a>

<a id="ref-for-clear-view-transition"></a>

a [map](https://infra.spec.whatwg.org/#ordered-map), whose keys are [view transition names](#view-transition-name) and whose values are [captured elements](#captured-element). Initially a new <a id="ref-for-ordered-map①"></a>map. Note: Since this is associated to the <code><a href="#viewtransition">ViewTransition</a></code>, it will be cleaned up when [Clear view transition](#clear-view-transition) is called.

<a id="viewtransition-phase"></a>phase  
One of the following ordered phases, initially "`pending-capture`":

1.  "`pending-capture`".

2.  "`update-callback-called`".

3.  "`animating`".

4.  "`done`".

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the most part, a developer using this API does not need to worry about the different phases, since they progress automatically. It is, however, important to understand what steps happen in each of the phases: when the snapshots are captured, when pseudo-element DOM is created, etc. The description of the phases below tries to be as precise as possible, with an intent to provide an unambiguous set of steps for implementors to follow in order to produce a spec-compliant implementation.

<a id="viewtransition-update-callback"></a>update callback  
<a id="ref-for-callbackdef-updatecallback①④"></a>

an <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code> or null. Initially null.

<a id="viewtransition-ready-promise"></a>ready promise  
<a id="ref-for-idl-promise④"></a>

<a id="ref-for-a-new-promise"></a>

<a id="ref-for-this③"></a>

<a id="ref-for-concept-relevant-realm②"></a>

a <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code>. Initially [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in [this’s](https://webidl.spec.whatwg.org/#this) [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

<a id="viewtransition-update-callback-done-promise"></a>update callback done promise  
<a id="ref-for-idl-promise⑤"></a>

<a id="ref-for-a-new-promise①"></a>

<a id="ref-for-this④"></a>

<a id="ref-for-concept-relevant-realm③"></a>

a <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code>. Initially [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in [this’s](https://webidl.spec.whatwg.org/#this) [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

<a id="ref-for-viewtransition-ready-promise"></a>

<a id="ref-for-viewtransition-update-callback-done-promise"></a>

<a id="ref-for-event-unhandledrejection"></a>

<a id="ref-for-mark-a-promise-as-handled"></a>

<a id="ref-for-dom-viewtransition-updatecallbackdone⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [ready promise](#viewtransition-ready-promise) and [update callback done promise](#viewtransition-update-callback-done-promise) are immediately created, so rejections will cause <code><a href="https://html.spec.whatwg.org/multipage/indices.html#event-unhandledrejection">unhandledrejection</a></code>s unless they’re [handled](https://webidl.spec.whatwg.org/#mark-a-promise-as-handled), even if the getters such as <code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code> are not accessed.

<a id="viewtransition-finished-promise"></a>finished promise  
<a id="ref-for-idl-promise⑥"></a>

<a id="ref-for-a-new-promise②"></a>

<a id="ref-for-this⑤"></a>

<a id="ref-for-concept-relevant-realm④"></a>

<a id="ref-for-mark-a-promise-as-handled①"></a>

a <code><a href="https://webidl.spec.whatwg.org/#idl-promise">Promise</a></code>. Initially [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in [this’s](https://webidl.spec.whatwg.org/#this) [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), [marked as handled](https://webidl.spec.whatwg.org/#mark-a-promise-as-handled).

<a id="ref-for-mark-a-promise-as-handled②"></a>

<a id="ref-for-event-unhandledrejection①"></a>

<a id="ref-for-viewtransition-update-callback-done-promise①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is [marked as handled](https://webidl.spec.whatwg.org/#mark-a-promise-as-handled) to prevent duplicate <code><a href="https://html.spec.whatwg.org/multipage/indices.html#event-unhandledrejection">unhandledrejection</a></code>s, as this promise only ever rejects along with the [update callback done promise](#viewtransition-update-callback-done-promise).

<a id="viewtransition-transition-root-pseudo-element"></a>transition root pseudo-element  
<a id="ref-for-selectordef-view-transition⑦"></a>

a [::view-transition](#selectordef-view-transition). Initially a new <a id="ref-for-selectordef-view-transition⑧"></a>::view-transition.

<a id="viewtransition-initial-snapshot-containing-block-size"></a>initial snapshot containing block size  
<a id="ref-for-tuple①"></a>

a [tuple](https://infra.spec.whatwg.org/#tuple) of two numbers (width and height), or null. Initially null.

<a id="ref-for-snapshot-containing-block-size"></a>

<a id="ref-for-skip-the-view-transition②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is used to detect changes in the [snapshot containing block size](#snapshot-containing-block-size), which causes the transition to [skip](#skip-the-view-transition). [Discussion of this behavior](https://github.com/w3c/csswg-drafts/issues/8045).

<a id="ref-for-dom-viewtransition-finished④"></a>

<a id="ref-for-getter-steps"></a>

<a id="ref-for-this⑥"></a>

<a id="ref-for-viewtransition-finished-promise"></a>

The <code><a href="#dom-viewtransition-finished">finished</a></code> [getter steps](https://webidl.spec.whatwg.org/#getter-steps) are to return [this’s](https://webidl.spec.whatwg.org/#this) [finished promise](#viewtransition-finished-promise).

<a id="ref-for-dom-viewtransition-ready⑤"></a>

<a id="ref-for-getter-steps①"></a>

<a id="ref-for-this⑦"></a>

<a id="ref-for-viewtransition-ready-promise①"></a>

The <code><a href="#dom-viewtransition-ready">ready</a></code> [getter steps](https://webidl.spec.whatwg.org/#getter-steps) are to return [this’s](https://webidl.spec.whatwg.org/#this) [ready promise](#viewtransition-ready-promise).

<a id="ref-for-dom-viewtransition-updatecallbackdone⑥"></a>

<a id="ref-for-getter-steps②"></a>

<a id="ref-for-this⑧"></a>

<a id="ref-for-viewtransition-update-callback-done-promise②"></a>

The <code><a href="#dom-viewtransition-updatecallbackdone">updateCallbackDone</a></code> [getter steps](https://webidl.spec.whatwg.org/#getter-steps) are to return [this’s](https://webidl.spec.whatwg.org/#this) [update callback done promise](#viewtransition-update-callback-done-promise).

<a id="ref-for-dom-viewtransition-skiptransition④"></a>

#### <a id="ViewTransition-skipTransition"></a>6.2.1. <code><a href="#dom-viewtransition-skiptransition">skipTransition()</a></code> Method Steps

<a id="ref-for-method-steps①"></a>

The [method steps](https://webidl.spec.whatwg.org/#method-steps) for <a id="dom-viewtransition-skiptransition"></a>`skipTransition()` are:

1.  <a id="ref-for-this⑨"></a>

    <a id="ref-for-viewtransition-phase①"></a>

    <a id="ref-for-skip-the-view-transition③"></a>

    <a id="ref-for-aborterror①"></a>

    <a id="ref-for-idl-DOMException②"></a>

    If [this](https://webidl.spec.whatwg.org/#this)'s [phase](#viewtransition-phase) is not "`done`", then [skip the view transition](#skip-the-view-transition) for <a id="ref-for-this①⓪"></a>this with an "<code><a href="https://webidl.spec.whatwg.org/#aborterror">AbortError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

## <a id="algorithms"></a>7. Algorithms

### <a id="concepts"></a>7.1. Data Structures

<a id="ref-for-document⑥"></a>

#### <a id="additions-to-document"></a>7.1.1. Additions to <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>

<a id="ref-for-document⑦"></a>

A <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> additionally has:

<a id="document-active-view-transition"></a>active view transition  
<a id="ref-for-viewtransition①②"></a>

a <code><a href="#viewtransition">ViewTransition</a></code> or null. Initially null.

<a id="document-rendering-suppression-for-view-transitions"></a>rendering suppression for view transitions  
a boolean. Initially false.

<a id="ref-for-document⑧"></a>

<a id="ref-for-document-rendering-suppression-for-view-transitions"></a>

<a id="ref-for-document-element④"></a>

<a id="ref-for-element④"></a>

While a <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>’s [rendering suppression for view transitions](#document-rendering-suppression-for-view-transitions) is true, all pointer hit testing must target its [document element](https://dom.spec.whatwg.org/#document-element), ignoring all other [elements](https://drafts.csswg.org/css2/#element).

<a id="ref-for-dfn-pointer-capture"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not affect pointers that are [captured](https://www.w3.org/TR/pointerevents3/#dfn-pointer-capture).

<a id="document-dynamic-view-transition-style-sheet"></a>dynamic view transition style sheet  
<a id="ref-for-style-sheet"></a>

<a id="ref-for-cascade-origin-ua②"></a>

<a id="ref-for-global-view-transition-user-agent-style-sheet"></a>

a [style sheet](https://www.w3.org/TR/css-2022/#style-sheet). Initially a new <a id="ref-for-style-sheet①"></a>style sheet in the [user-agent origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua), ordered after the [global view transition user agent style sheet](#global-view-transition-user-agent-style-sheet).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is used to hold dynamic styles relating to transitions.

<a id="document-show-view-transition-tree"></a>show view transition tree  
A boolean. Initially false.

<a id="ref-for-this①①"></a>

<a id="ref-for-document-active-view-transition⑤"></a>

<a id="ref-for-viewtransition-transition-root-pseudo-element①"></a>

<a id="ref-for-document-element⑤"></a>

<a id="ref-for-originating-element②"></a>

When this is true, [this](https://webidl.spec.whatwg.org/#this)'s [active view transition](#document-active-view-transition)'s [transition root pseudo-element](#viewtransition-transition-root-pseudo-element) renders as a child of <a id="ref-for-this①②"></a>this's [document element](https://dom.spec.whatwg.org/#document-element), with <a id="ref-for-this①③"></a>this's <a id="ref-for-document-element⑥"></a>document element is its [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

<a id="ref-for-viewtransition-transition-root-pseudo-element②"></a>

<a id="ref-for-document-element⑦"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-snapshot-containing-block⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The position of the [transition root pseudo-element](#viewtransition-transition-root-pseudo-element) within the [document element](https://dom.spec.whatwg.org/#document-element) does not matter, as the <a id="ref-for-viewtransition-transition-root-pseudo-element③"></a>transition root pseudo-element's [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is the [snapshot containing block](#snapshot-containing-block).

#### <a id="elements-concept"></a>7.1.2. Additions to Elements

<a id="ref-for-element⑤"></a>

[Elements](https://drafts.csswg.org/css2/#element) have a <a id="captured-in-a-view-transition"></a>captured in a view transition boolean, initially false.

<a id="ref-for-element⑥"></a>

<a id="ref-for-x22①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This spec uses CSS’s definition of [element](https://drafts.csswg.org/css2/#element), which includes [pseudo-elements](https://www.w3.org/TR/CSS21/selector.html#x22).

<a id="ref-for-captured-element①"></a>

#### <a id="captured-elements"></a>7.1.3. [Captured elements](#captured-element)

<a id="ref-for-struct"></a>

A <a id="captured-element"></a>captured element is a [struct](https://infra.spec.whatwg.org/#struct) with the following:

<a id="captured-element-old-image"></a>old image  
an 2D bitmap or null. Initially null.

<a id="captured-element-old-width"></a>old width  
<a id="captured-element-old-height"></a>old height  
<a id="ref-for-idl-unrestricted-double"></a>

an <code><a href="https://webidl.spec.whatwg.org/#idl-unrestricted-double">unrestricted double</a></code>, initially zero.

<a id="captured-element-old-transform"></a>old transform  
<a id="ref-for-typedef-transform-function"></a>

<a id="ref-for-identity-transform-function"></a>

a [\<transform-function\>](https://www.w3.org/TR/css-transforms-2/#typedef-transform-function), initially the [identity transform function](https://www.w3.org/TR/css-transforms-1/#identity-transform-function).

<a id="captured-element-old-writing-mode"></a>old writing-mode  
<a id="ref-for-propdef-writing-mode"></a>

Null or a [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), initially null.

<a id="captured-element-old-direction"></a>old direction  
<a id="ref-for-propdef-direction"></a>

Null or a [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), initially null.

<a id="captured-element-old-text-orientation"></a>old text-orientation  
<a id="ref-for-propdef-text-orientation"></a>

Null or a [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation), initially null.

<a id="captured-element-old-mix-blend-mode"></a>old mix-blend-mode  
<a id="ref-for-propdef-mix-blend-mode①"></a>

Null or a [mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode), initially null.

<a id="captured-element-old-backdrop-filter"></a>old backdrop-filter  
<a id="ref-for-propdef-backdrop-filter"></a>

Null or a [backdrop-filter](https://drafts.fxtf.org/filter-effects-2/#propdef-backdrop-filter), initially null.

<a id="captured-element-old-color-scheme"></a>old color-scheme  
<a id="ref-for-propdef-color-scheme"></a>

Null or a [color-scheme](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-scheme), initially null.

<a id="captured-element-new-element"></a>new element  
<a id="ref-for-element⑦"></a>

an [element](https://drafts.csswg.org/css2/#element) or null. Initially null.

<a id="ref-for-captured-element②"></a>

In addition, a [captured element](#captured-element) has the following <a id="captured-element-style-definitions"></a>style definitions:

<a id="captured-element-group-keyframes"></a>group keyframes  
<a id="ref-for-csskeyframesrule"></a>

A <code><a href="https://www.w3.org/TR/css-animations-1/#csskeyframesrule">CSSKeyframesRule</a></code> or null. Initially null.

<a id="captured-element-group-animation-name-rule"></a>group animation name rule  
<a id="ref-for-cssstylerule"></a>

A <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or null. Initially null.

<a id="captured-element-group-styles-rule"></a>group styles rule  
<a id="ref-for-cssstylerule①"></a>

A <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or null. Initially null.

<a id="captured-element-image-pair-isolation-rule"></a>image pair isolation rule  
<a id="ref-for-cssstylerule②"></a>

A <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or null. Initially null.

<a id="captured-element-image-animation-name-rule"></a>image animation name rule  
<a id="ref-for-cssstylerule③"></a>

A <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> or null. Initially null.

<a id="ref-for-concept-document①"></a>

<a id="ref-for-document-dynamic-view-transition-style-sheet④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These are used to update, and later remove styles from a [document](https://dom.spec.whatwg.org/#concept-document)'s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet).

<a id="ref-for-perform-pending-transition-operations①"></a>

### <a id="perform-pending-transition-operations-algorithm"></a>7.2. [Perform pending transition operations](#perform-pending-transition-operations)

> <strong data-conversion-semantic="note">Note</strong>
>
> This algorithm is invoked as a part of [update the rendering loop](https://html.spec.whatwg.org/#event-loop-processing-model:perform-pending-transition-operations) in the html spec.

<a id="ref-for-document⑨"></a>

To <a id="perform-pending-transition-operations"></a>perform pending transition operations given a <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> <var>document</var>, perform the following steps:

1.  <a id="ref-for-document-active-view-transition⑥"></a>

    If <var>document</var>’s [active view transition](#document-active-view-transition) is not null, then:

    1.  <a id="ref-for-document-active-view-transition⑦"></a>

        <a id="ref-for-viewtransition-phase②"></a>

        <a id="ref-for-setup-view-transition①"></a>

        If <var>document</var>’s [active view transition](#document-active-view-transition)'s [phase](#viewtransition-phase) is "`pending-capture`", then [setup view transition](#setup-view-transition) for <var>document</var>’s <a id="ref-for-document-active-view-transition⑧"></a>active view transition.

    2.  <a id="ref-for-document-active-view-transition⑨"></a>

        <a id="ref-for-viewtransition-phase③"></a>

        <a id="ref-for-handle-transition-frame"></a>

        Otherwise, if <var>document</var>’s [active view transition](#document-active-view-transition)'s [phase](#viewtransition-phase) is "`animating`", then [handle transition frame](#handle-transition-frame) for <var>document</var>’s <a id="ref-for-document-active-view-transition①⓪"></a>active view transition.

<a id="ref-for-setup-view-transition②"></a>

### <a id="setup-view-transition-algorithm"></a>7.3. [Setup view transition](#setup-view-transition)

<a id="ref-for-viewtransition①③"></a>

To <a id="setup-view-transition"></a>setup view transition for a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>, perform the following steps:

<a id="ref-for-callbackdef-updatecallback①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm captures the current state of the document, calls the transition’s <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code>, then captures the new state of the document.

1.  <a id="ref-for-concept-relevant-global①"></a>

    <a id="ref-for-concept-document-window①"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-capture-the-old-state"></a>

    [Capture the old state](#capture-the-old-state) for <var>transition</var>.

    <a id="ref-for-skip-the-view-transition④"></a>

    <a id="ref-for-invalidstateerror①"></a>

    <a id="ref-for-idl-DOMException③"></a>

    <a id="ref-for-concept-relevant-realm⑤"></a>

    If failure is returned, then [skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> in <var>transition</var>’s [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

3.  <a id="ref-for-document-rendering-suppression-for-view-transitions①"></a>

    Set <var>document</var>’s [rendering suppression for view transitions](#document-rendering-suppression-for-view-transitions) to true.

4.  <a id="ref-for-queue-a-global-task"></a>

    <a id="ref-for-dom-manipulation-task-source"></a>

    <a id="ref-for-concept-relevant-global②"></a>

    [Queue a global task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-global-task) on the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source), given <var>transition</var>’s [relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global), to perform the following steps:

    <a id="ref-for-capture-the-image③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: A task is queued here because the texture read back in [capturing the image](#capture-the-image) may be async, although the render steps in the HTML spec act as if it’s synchronous.

    1.  <a id="ref-for-viewtransition-phase④"></a>

        If <var>transition</var>’s [phase](#viewtransition-phase) is "`done`", then abort these steps.

        <a id="ref-for-skip-the-view-transition⑤"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This happens if <var>transition</var> was [skipped](#skip-the-view-transition) before this point.

    2.  <a id="ref-for-call-the-update-callback"></a>

        [call the update callback](#call-the-update-callback).

<a id="ref-for-viewtransition①④"></a>

To <a id="activate-view-transition"></a>activate view transition for a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>, perform the following steps:

1.  <a id="ref-for-viewtransition-phase⑤"></a>

    If <var>transition</var>’s [phase](#viewtransition-phase) is "`done`", then return.

    <a id="ref-for-skip-the-view-transition⑥"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This happens if <var>transition</var> was [skipped](#skip-the-view-transition) before this point.

2.  <a id="ref-for-document-rendering-suppression-for-view-transitions②"></a>

    Set [rendering suppression for view transitions](#document-rendering-suppression-for-view-transitions) to false.

3.  <a id="ref-for-viewtransition-initial-snapshot-containing-block-size"></a>

    <a id="ref-for-snapshot-containing-block-size①"></a>

    <a id="ref-for-skip-the-view-transition⑦"></a>

    If <var>transition</var>’s [initial snapshot containing block size](#viewtransition-initial-snapshot-containing-block-size) is not equal to the [snapshot containing block size](#snapshot-containing-block-size), then [skip the view transition](#skip-the-view-transition) for <var>transition</var>, and return.

4.  <a id="ref-for-capture-the-new-state"></a>

    [Capture the new state](#capture-the-new-state) for <var>transition</var>.

    <a id="ref-for-skip-the-view-transition⑧"></a>

    <a id="ref-for-invalidstateerror②"></a>

    <a id="ref-for-idl-DOMException④"></a>

    <a id="ref-for-concept-relevant-realm⑥"></a>

    If failure is returned, then [skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> in <var>transition</var>’s [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

5.  <a id="ref-for-resolve"></a>

    <a id="ref-for-viewtransition-update-callback-done-promise③"></a>

    [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise) with undefined.

6.  <a id="ref-for-list-iterate"></a>

    <a id="ref-for-viewtransition-named-elements"></a>

    <a id="ref-for-map-getting-the-values"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>capturedElement</var> of <var>transition</var>’s [named elements](#viewtransition-named-elements)' [values](https://infra.spec.whatwg.org/#map-getting-the-values):

    1.  <a id="ref-for-captured-element-new-element①"></a>

        <a id="ref-for-captured-in-a-view-transition③"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is not null, then set <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element②"></a>new element's [captured in a view transition](#captured-in-a-view-transition) to true.

7.  <a id="ref-for-setup-transition-pseudo-elements④"></a>

    [Setup transition pseudo-elements](#setup-transition-pseudo-elements) for <var>transition</var>.

8.  <a id="ref-for-update-pseudo-element-styles②"></a>

    [Update pseudo-element styles](#update-pseudo-element-styles) for <var>transition</var>.

    <a id="ref-for-skip-the-view-transition⑨"></a>

    <a id="ref-for-invalidstateerror③"></a>

    <a id="ref-for-idl-DOMException⑤"></a>

    <a id="ref-for-concept-relevant-realm⑦"></a>

    If failure is returned, then [skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> in <var>transition</var>’s [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The above steps will require running document lifecycle phases, to compute information calculated during style/layout.

9.  <a id="ref-for-viewtransition-phase⑥"></a>

    Set <var>transition</var>’s [phase](#viewtransition-phase) to "`animating`".

10. <a id="ref-for-resolve①"></a>

    <a id="ref-for-viewtransition-ready-promise②"></a>

    [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s [ready promise](#viewtransition-ready-promise).

<a id="ref-for-capture-the-old-state①"></a>

#### <a id="capture-old-state-algorithm"></a>7.3.1. [Capture the old state](#capture-the-old-state)

<a id="ref-for-viewtransition①⑤"></a>

To <a id="capture-the-old-state"></a>capture the old state for <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

1.  <a id="ref-for-concept-relevant-global③"></a>

    <a id="ref-for-concept-document-window②"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-viewtransition-named-elements①"></a>

    Let <var>namedElements</var> be <var>transition</var>’s [named elements](#viewtransition-named-elements).

3.  <a id="ref-for-ordered-set"></a>

    Let <var>usedTransitionNames</var> be a new [set](https://infra.spec.whatwg.org/#ordered-set) of strings.

4.  <a id="ref-for-list"></a>

    Let <var>captureElements</var> be a new [list](https://infra.spec.whatwg.org/#list) of elements.

5.  <a id="ref-for-concept-relevant-global④"></a>

    <a id="ref-for-concept-document-window③"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

6.  <a id="ref-for-snapshot-containing-block-size②"></a>

    <a id="ref-for-implementation-defined"></a>

    If the [snapshot containing block size](#snapshot-containing-block-size) exceeds an [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined) maximum, then return failure.

7.  <a id="ref-for-viewtransition-initial-snapshot-containing-block-size①"></a>

    <a id="ref-for-snapshot-containing-block-size③"></a>

    Set <var>transition</var>’s [initial snapshot containing block size](#viewtransition-initial-snapshot-containing-block-size) to the [snapshot containing block size](#snapshot-containing-block-size).

8.  <a id="ref-for-list-iterate①"></a>

    <a id="ref-for-element⑧"></a>

    <a id="ref-for-connected"></a>

    <a id="ref-for-concept-node-document"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of every [element](https://drafts.csswg.org/css2/#element) that is [connected](https://dom.spec.whatwg.org/#connected), and has a [node document](https://dom.spec.whatwg.org/#concept-node-document) equal to to <var>document</var>, in [paint order](https://drafts.csswg.org/css2/#painting-order):

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > We iterate in paint order to ensure that this order is cached in <var>namedElements</var>. This defines the DOM order for ::view-transition-group pseudo-elements, such that the element at the bottom of the paint stack generates the first pseudo child of ::view-transition.

    1.  <a id="ref-for-flat-tree"></a>

        <a id="ref-for-skips-its-contents①"></a>

        <a id="ref-for-iteration-continue"></a>

        If any [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) ancestor of this <var>element</var> [skips its contents](https://www.w3.org/TR/css-contain-2/#skips-its-contents), then [continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  <a id="ref-for-box-fragment"></a>

        <a id="ref-for-iteration-continue①"></a>

        If <var>element</var> has more than one [box fragment](https://www.w3.org/TR/css-break-4/#box-fragment), then [continue](https://infra.spec.whatwg.org/#iteration-continue).

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: We might want to enable transitions for fragmented elements in future versions. See [\#8900](https://github.com/w3c/csswg-drafts/issues/8900).

        <a id="ref-for-box-fragment①"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: [box fragment](https://www.w3.org/TR/css-break-4/#box-fragment) here does not refer to fragmentation of [inline boxes](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) across [line boxes](https://www.w3.org/TR/CSS2/visuren.html#line-box). Such inlines can participate in a transition.

    3.  <a id="ref-for-computed-value①"></a>

        <a id="ref-for-propdef-view-transition-name①⓪"></a>

        Let <var>transitionName</var> be the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [view-transition-name](#propdef-view-transition-name) for <var>element</var>.

    4.  <a id="ref-for-valdef-view-transition-name-none①"></a>

        <a id="ref-for-element-not-rendered①"></a>

        <a id="ref-for-iteration-continue②"></a>

        If <var>transitionName</var> is [none](#valdef-view-transition-name-none), or <var>element</var> is [not rendered](https://www.w3.org/TR/css-images-4/#element-not-rendered), then [continue](https://infra.spec.whatwg.org/#iteration-continue).

    5.  <a id="ref-for-list-contain"></a>

        If <var>usedTransitionNames</var> [contains](https://infra.spec.whatwg.org/#list-contain) <var>transitionName</var>, then return failure.

    6.  <a id="ref-for-set-append"></a>

        [Append](https://infra.spec.whatwg.org/#set-append) <var>transitionName</var> to <var>usedTransitionNames</var>.

    7.  <a id="ref-for-captured-in-a-view-transition④"></a>

        Set <var>element</var>’s [captured in a view transition](#captured-in-a-view-transition) to true.

    8.  <a id="ref-for-list-append"></a>

        [Append](https://infra.spec.whatwg.org/#list-append) <var>element</var> to <var>captureElements</var>.

    <a id="ref-for-captured-in-a-view-transition⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > The algorithm continues in a separate loop to ensure that [captured in a view transition](#captured-in-a-view-transition) is set on all elements participating in this capture before it is read by future steps in the algorithm.

9.  <a id="ref-for-list-iterate②"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> in <var>captureElements</var>:

    1.  <a id="ref-for-captured-element③"></a>

        Let <var>capture</var> be a new [captured element](#captured-element) struct.

    2.  <a id="ref-for-captured-element-old-image①"></a>

        <a id="ref-for-capture-the-image④"></a>

        Set <var>capture</var>’s [old image](#captured-element-old-image) to the result of [capturing the image](#capture-the-image) of <var>element</var>.

    3.  <a id="ref-for-snapshot-containing-block①⓪"></a>

        <a id="ref-for-document-element⑧"></a>

        <a id="ref-for-border-box②"></a>

        Let <var>originalRect</var> be [snapshot containing block](#snapshot-containing-block) if <var>element</var> is the [document element](https://dom.spec.whatwg.org/#document-element), otherwise, the element\|'s [border box](https://www.w3.org/TR/css-box-4/#border-box).

    4.  <a id="ref-for-captured-element-old-width"></a>

        <a id="ref-for-dom-domrect-width"></a>

        Set <var>capture</var>’s [old width](#captured-element-old-width) to <var>originalRect</var>’s <code><a href="https://www.w3.org/TR/geometry-1/#dom-domrect-width">width</a></code>.

    5.  <a id="ref-for-captured-element-old-height"></a>

        <a id="ref-for-dom-domrect-height"></a>

        Set <var>capture</var>’s [old height](#captured-element-old-height) to <var>originalRect</var>’s <code><a href="https://www.w3.org/TR/geometry-1/#dom-domrect-height">height</a></code>.

    6.  <a id="ref-for-captured-element-old-transform"></a>

        <a id="ref-for-typedef-transform-function①"></a>

        <a id="ref-for-border-box③"></a>

        <a id="ref-for-snapshot-containing-block-origin"></a>

        Set <var>capture</var>’s [old transform](#captured-element-old-transform) to a [\<transform-function\>](https://www.w3.org/TR/css-transforms-2/#typedef-transform-function) that would map <var>element</var>’s [border box](https://www.w3.org/TR/css-box-4/#border-box) from the [snapshot containing block origin](#snapshot-containing-block-origin) to its current visual position.

    7.  <a id="ref-for-captured-element-old-writing-mode"></a>

        <a id="ref-for-computed-value②"></a>

        <a id="ref-for-propdef-writing-mode①"></a>

        Set <var>capture</var>’s [old writing-mode](#captured-element-old-writing-mode) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) on <var>element</var>.

    8.  <a id="ref-for-captured-element-old-direction"></a>

        <a id="ref-for-computed-value③"></a>

        <a id="ref-for-propdef-direction①"></a>

        Set <var>capture</var>’s [old direction](#captured-element-old-direction) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) on <var>element</var>.

    9.  <a id="ref-for-captured-element-old-text-orientation"></a>

        <a id="ref-for-computed-value④"></a>

        <a id="ref-for-propdef-text-orientation①"></a>

        Set <var>capture</var>’s [old text-orientation](#captured-element-old-text-orientation) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) on <var>element</var>.

    10. <a id="ref-for-captured-element-old-mix-blend-mode"></a>

        <a id="ref-for-computed-value⑤"></a>

        <a id="ref-for-propdef-mix-blend-mode②"></a>

        Set <var>capture</var>’s [old mix-blend-mode](#captured-element-old-mix-blend-mode) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode) on <var>element</var>.

    11. <a id="ref-for-captured-element-old-backdrop-filter"></a>

        <a id="ref-for-computed-value⑥"></a>

        <a id="ref-for-propdef-backdrop-filter①"></a>

        Set <var>capture</var>’s [old backdrop-filter](#captured-element-old-backdrop-filter) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [backdrop-filter](https://drafts.fxtf.org/filter-effects-2/#propdef-backdrop-filter) on <var>element</var>.

    12. <a id="ref-for-captured-element-old-color-scheme"></a>

        <a id="ref-for-computed-value⑦"></a>

        <a id="ref-for-propdef-color-scheme①"></a>

        Set <var>capture</var>’s [old color-scheme](#captured-element-old-color-scheme) to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [color-scheme](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-scheme) on <var>element</var>.

    13. <a id="ref-for-computed-value⑧"></a>

        <a id="ref-for-propdef-view-transition-name①①"></a>

        Let <var>transitionName</var> be the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [view-transition-name](#propdef-view-transition-name) for <var>element</var>.

    14. Set <var>namedElements</var>\[<var>transitionName</var>\] to <var>capture</var>.

10. <a id="ref-for-list-iterate③"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> in <var>captureElements</var>:

    1.  <a id="ref-for-captured-in-a-view-transition⑥"></a>

        Set <var>element</var>’s [captured in a view transition](#captured-in-a-view-transition) to false.

<a id="ref-for-capture-the-new-state①"></a>

#### <a id="capture-new-state-algorithm"></a>7.3.2. [Capture the new state](#capture-the-new-state)

<a id="ref-for-viewtransition①⑥"></a>

To <a id="capture-the-new-state"></a>capture the new state for <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

1.  <a id="ref-for-concept-relevant-global⑤"></a>

    <a id="ref-for-concept-document-window④"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-viewtransition-named-elements②"></a>

    Let <var>namedElements</var> be <var>transition</var>’s [named elements](#viewtransition-named-elements).

3.  <a id="ref-for-ordered-set①"></a>

    Let <var>usedTransitionNames</var> be a new [set](https://infra.spec.whatwg.org/#ordered-set) of strings.

4.  <a id="ref-for-list-iterate④"></a>

    <a id="ref-for-element⑨"></a>

    <a id="ref-for-connected①"></a>

    <a id="ref-for-concept-node-document①"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of every [element](https://drafts.csswg.org/css2/#element) that is [connected](https://dom.spec.whatwg.org/#connected), and has a [node document](https://dom.spec.whatwg.org/#concept-node-document) equal to to <var>document</var>, in [paint order](https://drafts.csswg.org/css2/#painting-order):

    1.  <a id="ref-for-flat-tree①"></a>

        <a id="ref-for-skips-its-contents②"></a>

        <a id="ref-for-iteration-continue③"></a>

        If any [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) ancestor of this <var>element</var> [skips its contents](https://www.w3.org/TR/css-contain-2/#skips-its-contents), then [continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  <a id="ref-for-computed-value⑨"></a>

        <a id="ref-for-propdef-view-transition-name①②"></a>

        Let <var>transitionName</var> be the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [view-transition-name](#propdef-view-transition-name) for <var>element</var>.

    3.  <a id="ref-for-valdef-view-transition-name-none②"></a>

        <a id="ref-for-element-not-rendered②"></a>

        <a id="ref-for-iteration-continue④"></a>

        If <var>transitionName</var> is [none](#valdef-view-transition-name-none), or <var>element</var> is [not rendered](https://www.w3.org/TR/css-images-4/#element-not-rendered), then [continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  <a id="ref-for-list-contain①"></a>

        If <var>usedTransitionNames</var> [contains](https://infra.spec.whatwg.org/#list-contain) <var>transitionName</var>, then return failure.

    5.  <a id="ref-for-set-append①"></a>

        [Append](https://infra.spec.whatwg.org/#set-append) <var>transitionName</var> to <var>usedTransitionNames</var>.

    6.  <a id="ref-for-map-exists"></a>

        <a id="ref-for-captured-element④"></a>

        If <var>namedElements</var>\[<var>transitionName</var>\] does not [exist](https://infra.spec.whatwg.org/#map-exists), then set <var>namedElements</var>\[<var>transitionName</var>\] to a new [captured element](#captured-element) struct.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: We intentionally add this struct to the end of this ordered map. This implies than names which only exist in the new DOM (entry animations) will be painted on top of names only in the old DOM (exit animations) and names in both DOMs (paired animations). This might not be the right layering for all cases. See [issue 8941](https://github.com/w3c/csswg-drafts/issues/8941).

    7.  <a id="ref-for-captured-element-new-element③"></a>

        Set <var>namedElements</var>\[<var>transitionName</var>\]'s [new element](#captured-element-new-element) to <var>element</var>.

<a id="ref-for-setup-transition-pseudo-elements⑤"></a>

#### <a id="setup-transition-pseudo-elements-algorithm"></a>7.3.3. [Setup transition pseudo-elements](#setup-transition-pseudo-elements)

<a id="ref-for-viewtransition①⑦"></a>

To <a id="setup-transition-pseudo-elements"></a>setup transition pseudo-elements for a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm constructs the pseudo-element tree for the transition, and generates initial styles. The structure of the pseudo-tree is covered at a higher level in [§ 3.2 View Transition Pseudo-elements](#view-transition-pseudos).

1.  <a id="ref-for-this①④"></a>

    <a id="ref-for-concept-relevant-global⑥"></a>

    <a id="ref-for-concept-document-window⑤"></a>

    Let <var>document</var> be [this’s](https://webidl.spec.whatwg.org/#this) [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-document-show-view-transition-tree"></a>

    Set <var>document</var>’s [show view transition tree](#document-show-view-transition-tree) to true.

3.  <a id="ref-for-map-iterate"></a>

    <a id="ref-for-viewtransition-named-elements③"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>transitionName</var> → <var>capturedElement</var> of <var>transition</var>’s [named elements](#viewtransition-named-elements):

    1.  <a id="ref-for-selectordef-view-transition-group①⑥"></a>

        <a id="ref-for-view-transition-name①①"></a>

        Let <var>group</var> be a new [::view-transition-group()](#selectordef-view-transition-group), with its [view transition name](#view-transition-name) set to <var>transitionName</var>.

    2.  <a id="ref-for-viewtransition-transition-root-pseudo-element④"></a>

        Append <var>group</var> to <var>transition</var>’s [transition root pseudo-element](#viewtransition-transition-root-pseudo-element).

    3.  <a id="ref-for-selectordef-view-transition-image-pair⑥"></a>

        <a id="ref-for-view-transition-name①②"></a>

        Let <var>imagePair</var> be a new [::view-transition-image-pair()](#selectordef-view-transition-image-pair), with its [view transition name](#view-transition-name) set to <var>transitionName</var>.

    4.  Append <var>imagePair</var> to <var>group</var>.

    5.  <a id="ref-for-captured-element-old-image②"></a>

        If <var>capturedElement</var>’s [old image](#captured-element-old-image) is not null, then:

        1.  <a id="ref-for-selectordef-view-transition-old⑧"></a>

            <a id="ref-for-view-transition-name①③"></a>

            <a id="ref-for-captured-element-old-image③"></a>

            <a id="ref-for-replaced-element②"></a>

            Let <var>old</var> be a new [::view-transition-old()](#selectordef-view-transition-old), with its [view transition name](#view-transition-name) set to <var>transitionName</var>, displaying <var>capturedElement</var>’s [old image](#captured-element-old-image) as its [replaced](https://www.w3.org/TR/css-display-3/#replaced-element) content.

        2.  Append <var>old</var> to <var>imagePair</var>.

    6.  <a id="ref-for-captured-element-new-element④"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is not null, then:

        1.  <a id="ref-for-selectordef-view-transition-new⑧"></a>

            <a id="ref-for-view-transition-name①④"></a>

            Let <var>new</var> be a new [::view-transition-new()](#selectordef-view-transition-new), with its [view transition name](#view-transition-name) set to <var>transitionName</var>.

            <a id="ref-for-update-pseudo-element-styles③"></a>

            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The styling of this pseudo is handled in [update pseudo-element styles](#update-pseudo-element-styles).

        2.  Append <var>new</var> to <var>imagePair</var>.

    7.  <a id="ref-for-captured-element-old-image④"></a>

        If <var>capturedElement</var>’s [old image](#captured-element-old-image) is null, then:

        1.  <a id="ref-for-assert"></a>

            <a id="ref-for-captured-element-new-element⑤"></a>

            [Assert](https://infra.spec.whatwg.org/#assert): <var>capturedElement</var>’s [new element](#captured-element-new-element) is not null.

        2.  <a id="ref-for-captured-element-image-animation-name-rule"></a>

            <a id="ref-for-cssstylerule④"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet⑤"></a>

            Set <var>capturedElement</var>’s [image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            :root::view-transition-new(transitionName) {
              animation-name: -ua-view-transition-fade-in;
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

    8.  <a id="ref-for-captured-element-new-element⑥"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is null, then:

        1.  <a id="ref-for-assert①"></a>

            <a id="ref-for-captured-element-old-image⑤"></a>

            [Assert](https://infra.spec.whatwg.org/#assert): <var>capturedElement</var>’s [old image](#captured-element-old-image) is not null.

        2.  <a id="ref-for-captured-element-image-animation-name-rule①"></a>

            <a id="ref-for-cssstylerule⑤"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet⑥"></a>

            Set <var>capturedElement</var>’s [image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            :root::view-transition-old(transitionName) {
              animation-name: -ua-view-transition-fade-out;
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

    9.  <a id="ref-for-captured-element-old-image⑥"></a>

        <a id="ref-for-captured-element-new-element⑦"></a>

        If both of <var>capturedElement</var>’s [old image](#captured-element-old-image) and [new element](#captured-element-new-element) are not null, then:

        1.  <a id="ref-for-captured-element-old-transform①"></a>

            Let <var>transform</var> be <var>capturedElement</var>’s [old transform](#captured-element-old-transform).

        2.  <a id="ref-for-captured-element-old-width①"></a>

            Let <var>width</var> be <var>capturedElement</var>’s [old width](#captured-element-old-width).

        3.  <a id="ref-for-captured-element-old-height①"></a>

            Let <var>height</var> be <var>capturedElement</var>’s [old height](#captured-element-old-height).

        4.  <a id="ref-for-captured-element-old-backdrop-filter①"></a>

            Let <var>backdropFilter</var> be <var>capturedElement</var>’s [old backdrop-filter](#captured-element-old-backdrop-filter).

        5.  <a id="ref-for-captured-element-group-keyframes"></a>

            <a id="ref-for-csskeyframesrule①"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet⑦"></a>

            Set <var>capturedElement</var>’s [group keyframes](#captured-element-group-keyframes) to a new <code><a href="https://www.w3.org/TR/css-animations-1/#csskeyframesrule">CSSKeyframesRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            @keyframes -ua-view-transition-group-anim-transitionName {
              from {
                transform: transform;
                width: width;
                height: height;
                backdrop-filter: backdropFilter;
              }
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

        6.  <a id="ref-for-captured-element-group-animation-name-rule"></a>

            <a id="ref-for-cssstylerule⑥"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet⑧"></a>

            Set <var>capturedElement</var>’s [group animation name rule](#captured-element-group-animation-name-rule) to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            :root::view-transition-group(transitionName) {
              animation-name: -ua-view-transition-group-anim-transitionName;
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

        7.  <a id="ref-for-captured-element-image-pair-isolation-rule"></a>

            <a id="ref-for-cssstylerule⑦"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet⑨"></a>

            Set <var>capturedElement</var>’s [image pair isolation rule](#captured-element-image-pair-isolation-rule) to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            :root::view-transition-image-pair(transitionName) {
              isolation: isolate;
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

        8.  <a id="ref-for-captured-element-image-animation-name-rule②"></a>

            <a id="ref-for-cssstylerule⑧"></a>

            <a id="ref-for-document-dynamic-view-transition-style-sheet①⓪"></a>

            Set <var>capturedElement</var>’s [image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet):

            ```text
            :root::view-transition-old(transitionName) {
              animation-name: -ua-view-transition-fade-out, -ua-mix-blend-mode-plus-lighter;
            }
            :root::view-transition-new(transitionName) {
              animation-name: -ua-view-transition-fade-in, -ua-mix-blend-mode-plus-lighter;
            }
            ```
            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: The above code example contains variables to be replaced.

            <a id="ref-for-propdef-mix-blend-mode③"></a>

            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: [mix-blend-mode: plus-lighter](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode) ensures that the blending of identical pixels from the old and new images results in the same color value as those pixels, and achieves a “correct” cross-fade.

<a id="ref-for-call-the-update-callback①"></a>

### <a id="call-dom-update-callback-algorithm"></a>7.4. [Call the update callback](#call-the-update-callback)

<a id="ref-for-viewtransition①⑧"></a>

To <a id="call-the-update-callback"></a>call the update callback of a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

<a id="ref-for-viewtransition①⑨"></a>

<a id="ref-for-skip-the-view-transition①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is guaranteed to happen for every <code><a href="#viewtransition">ViewTransition</a></code>, even if the transition is [skipped](#skip-the-view-transition). The reasons for this are discussed in [§ 1.4 Transitions as an enhancement](#transitions-as-enhancements).

1.  <a id="ref-for-assert②"></a>

    <a id="ref-for-viewtransition-phase⑦"></a>

    [Assert](https://infra.spec.whatwg.org/#assert): <var>transition</var>’s [phase](#viewtransition-phase) is "`done`", or before "`update-callback-called`".

2.  Let <var>callbackPromise</var> be null.

3.  <a id="ref-for-viewtransition-update-callback②"></a>

    <a id="ref-for-a-promise-resolved-with"></a>

    <a id="ref-for-concept-relevant-realm⑧"></a>

    If <var>transition</var>’s [update callback](#viewtransition-update-callback) is null, then set <var>callbackPromise</var> to [a promise resolved with](https://webidl.spec.whatwg.org/#a-promise-resolved-with) undefined, in <var>transition</var>’s [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

4.  <a id="ref-for-invoke-a-callback-function"></a>

    <a id="ref-for-viewtransition-update-callback③"></a>

    Otherwise, set <var>callbackPromise</var> to the result of [invoking](https://webidl.spec.whatwg.org/#invoke-a-callback-function) <var>transition</var>’s [update callback](#viewtransition-update-callback).

5.  <a id="ref-for-viewtransition-phase⑧"></a>

    If <var>transition</var>’s [phase](#viewtransition-phase) is not "`done`", then set <var>transition</var>’s <a id="ref-for-viewtransition-phase⑨"></a>phase to "`update-callback-called`".

6.  Let <var>fulfillSteps</var> be to following steps:

    1.  <a id="ref-for-activate-view-transition"></a>

        [Activate](#activate-view-transition) <var>transition</var>.

    2.  <a id="ref-for-resolve②"></a>

        <a id="ref-for-viewtransition-update-callback-done-promise④"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise) with undefined.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This would be a no-op if the previous step already resolved the promise.

7.  Let <var>rejectSteps</var> be the following steps given <var>reason</var>:

    1.  <a id="ref-for-reject"></a>

        <a id="ref-for-viewtransition-update-callback-done-promise⑤"></a>

        [Reject](https://webidl.spec.whatwg.org/#reject) <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise) with <var>reason</var>.

    2.  <a id="ref-for-viewtransition-phase①⓪"></a>

        If <var>transition</var>’s [phase](#viewtransition-phase) is "`done`", then return.

        <a id="ref-for-skip-the-view-transition①①"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This happens if <var>transition</var> was [skipped](#skip-the-view-transition) before this point.

    3.  <a id="ref-for-mark-a-promise-as-handled③"></a>

        <a id="ref-for-viewtransition-ready-promise③"></a>

        [Mark as handled](https://webidl.spec.whatwg.org/#mark-a-promise-as-handled) <var>transition</var>’s [ready promise](#viewtransition-ready-promise).

        <a id="ref-for-viewtransition-update-callback-done-promise⑥"></a>

        <a id="ref-for-event-unhandledrejection②"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise) will provide the <code><a href="https://html.spec.whatwg.org/multipage/indices.html#event-unhandledrejection">unhandledrejection</a></code>. This step avoids a duplicate.

    4.  <a id="ref-for-skip-the-view-transition①②"></a>

        [Skip the view transition](#skip-the-view-transition) <var>transition</var> with <var>reason</var>.

8.  <a id="ref-for-dfn-perform-steps-once-promise-is-settled"></a>

    [React](https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled) to <var>callbackPromise</var> with <var>fulfillSteps</var> and <var>rejectSteps</var>.

9.  <a id="ref-for-in-parallel"></a>

    To skip a transition after a timeout, the user agent may perform the following steps [in parallel](https://html.spec.whatwg.org/multipage/infrastructure.html#in-parallel):

    1.  <a id="ref-for-dfn-duration"></a>

        Wait for an implementation-defined [duration](https://www.w3.org/TR/hr-time-3/#dfn-duration).

    2.  <a id="ref-for-queue-a-global-task①"></a>

        <a id="ref-for-dom-manipulation-task-source①"></a>

        <a id="ref-for-concept-relevant-global⑦"></a>

        [Queue a global task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-global-task) on the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source), given <var>transition</var>’s [relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global), to perform the following steps:

        1.  <a id="ref-for-viewtransition-phase①①"></a>

            If <var>transition</var>’s [phase](#viewtransition-phase) is "`done`", then return.

            <a id="ref-for-skip-the-view-transition①③"></a>

            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: This happens if <var>transition</var> was [skipped](#skip-the-view-transition) before this point.

        2.  <a id="ref-for-skip-the-view-transition①④"></a>

            <a id="ref-for-timeouterror"></a>

            <a id="ref-for-idl-DOMException⑥"></a>

            [Skip](#skip-the-view-transition) <var>transition</var> with a "<code><a href="https://webidl.spec.whatwg.org/#timeouterror">TimeoutError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

<a id="ref-for-skip-the-view-transition①⑤"></a>

### <a id="skip-the-view-transition-algorithm"></a>7.5. [Skip the view transition](#skip-the-view-transition)

<a id="ref-for-viewtransition②⓪"></a>

To <a id="skip-the-view-transition"></a>skip the view transition for <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var> with reason <var>reason</var>:

1.  <a id="ref-for-concept-relevant-global⑧"></a>

    <a id="ref-for-concept-document-window⑥"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-assert③"></a>

    <a id="ref-for-viewtransition-phase①②"></a>

    [Assert](https://infra.spec.whatwg.org/#assert): <var>transition</var>’s [phase](#viewtransition-phase) is not "`done`".

3.  <a id="ref-for-viewtransition-phase①③"></a>

    <a id="ref-for-queue-a-global-task②"></a>

    <a id="ref-for-dom-manipulation-task-source②"></a>

    <a id="ref-for-concept-relevant-global⑨"></a>

    <a id="ref-for-call-the-update-callback②"></a>

    If <var>transition</var>’s [phase](#viewtransition-phase) is before "`update-callback-called`", then [queue a global task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-global-task) on the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source), given <var>transition</var>’s [relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global), to [call the update callback](#call-the-update-callback) of <var>transition</var>.

4.  <a id="ref-for-document-rendering-suppression-for-view-transitions③"></a>

    Set [rendering suppression for view transitions](#document-rendering-suppression-for-view-transitions) to false.

5.  <a id="ref-for-document-active-view-transition①①"></a>

    <a id="ref-for-clear-view-transition①"></a>

    If <var>document</var>’s [active view transition](#document-active-view-transition) is <var>transition</var>, [Clear view transition](#clear-view-transition) <var>transition</var>.

6.  <a id="ref-for-viewtransition-phase①④"></a>

    Set <var>transition</var>’s [phase](#viewtransition-phase) to "`done`".

7.  <a id="ref-for-reject①"></a>

    <a id="ref-for-viewtransition-ready-promise④"></a>

    [Reject](https://webidl.spec.whatwg.org/#reject) <var>transition</var>’s [ready promise](#viewtransition-ready-promise) with <var>reason</var>.

    <a id="ref-for-viewtransition-ready-promise⑤"></a>

    <a id="ref-for-dom-viewtransition-skiptransition⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The [ready promise](#viewtransition-ready-promise) may already be resolved at this point, if <code><a href="#dom-viewtransition-skiptransition">skipTransition()</a></code> is called after we start animating. In that case, this step is a no-op.

8.  <a id="ref-for-resolve③"></a>

    <a id="ref-for-viewtransition-finished-promise①"></a>

    <a id="ref-for-dfn-perform-steps-once-promise-is-settled①"></a>

    <a id="ref-for-viewtransition-update-callback-done-promise⑦"></a>

    [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s [finished promise](#viewtransition-finished-promise) with the result of [reacting](https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled) to <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise):

    - If the promise was fulfilled, then return undefined.

    <a id="ref-for-viewtransition-update-callback-done-promise⑧"></a>

    <a id="ref-for-viewtransition-finished-promise②"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Since the rejection of <var>transition</var>’s [update callback done promise](#viewtransition-update-callback-done-promise) isn’t explicitly handled here, if <var>transition</var>’s <a id="ref-for-viewtransition-update-callback-done-promise⑨"></a>update callback done promise rejects, then <var>transition</var>’s [finished promise](#viewtransition-finished-promise) will reject with the same reason.

### <a id="page-visibility-change-steps"></a>7.6. View transition page-visibility change steps

<a id="ref-for-document①⓪"></a>

The <a id="view-transition-page-visibility-change-steps"></a>view transition page-visibility change steps given <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> <var>document</var> are:

1.  <a id="ref-for-visibility-state①"></a>

    If <var>document</var>’s [visibility state](https://html.spec.whatwg.org/multipage/interaction.html#visibility-state) is "`hidden`", then:

    1.  <a id="ref-for-document-active-view-transition①②"></a>

        <a id="ref-for-skip-the-view-transition①⑥"></a>

        <a id="ref-for-invalidstateerror④"></a>

        <a id="ref-for-idl-DOMException⑦"></a>

        If <var>document</var>’s [active view transition](#document-active-view-transition) is not null, then [skip](#skip-the-view-transition) <var>document</var>’s <a id="ref-for-document-active-view-transition①③"></a>active view transition with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

2.  <a id="ref-for-assert④"></a>

    <a id="ref-for-document-active-view-transition①④"></a>

    Otherwise, [assert](https://infra.spec.whatwg.org/#assert): [active view transition](#document-active-view-transition) is null.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this is called from the HTML spec.

<a id="ref-for-capture-the-image⑤"></a>

### <a id="capture-the-image-algorithm"></a>7.7. [Capture the image](#capture-the-image)

<a id="ref-for-element①⓪"></a>

To <a id="capture-the-image"></a>capture the image given an [element](https://drafts.csswg.org/css2/#element) <var>element</var>, perform the following steps. They return an image.

1.  <a id="ref-for-document-element⑨"></a>

    If <var>element</var> is the [document element](https://dom.spec.whatwg.org/#document-element), then:

    1.  <a id="ref-for-canvas-background"></a>

        <a id="ref-for-document-top-layer①"></a>

        <a id="ref-for-snapshot-containing-block①①"></a>

        <a id="ref-for-capture-rendering-characteristics"></a>

        Render the region of document (including its [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background) and any [top layer](https://drafts.csswg.org/css-position-4/#document-top-layer) content) that intersects the [snapshot containing block](#snapshot-containing-block), on a transparent canvas the size of the <a id="ref-for-snapshot-containing-block①②"></a>snapshot containing block, following the [capture rendering characteristics](#capture-rendering-characteristics), and these additional characteristics:

        - <a id="ref-for-scrolling-box"></a>

          <a id="ref-for-layout-viewport"></a>

          <a id="ref-for-intersectionobserver"></a>

          Areas outside <var>element</var>’s [scrolling box](https://drafts.csswg.org/cssom-view-1/#scrolling-box) should be rendered as if they were scrolled to, without moving or resizing the [layout viewport](https://drafts.csswg.org/cssom-view-1/#layout-viewport). This must not trigger events related to scrolling or resizing, such as <code><a href="https://www.w3.org/TR/intersection-observer/#intersectionobserver">IntersectionObserver</a></code>s.

          ![A phone browser window, showing a URL bar, a fixed-position element directly beneath it, and some page content beneath that. A scroll bar indicates the page has been scrolled significantly.](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser-with-url.svg) ![The captured snapshot. It shows that content beneath the URL bar was included in the capture.](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser-without-url.svg)
          An example of what the user sees compared to the captured snapshot. This example assumes the root is the only element with a transition name.

        - <a id="ref-for-canvas-background①"></a>

          Areas that cannot be scrolled to (i.e. they are out of scrolling bounds), should render the [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background).

          ![A phone browser window, showing a URL bar, and some content beneath. A scroll bar indicates the page is scrolled to the top.](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser-scrolled-to-top-with-url.svg) ![The captured snapshot. It shows the area underneath the URL bar as the same color as the rest of the document.](https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/diagrams/phone-browser-scrolled-to-top-without-url.svg)
          An example of what the user sees compared to the captured snapshot. This example assumes the root is the only element with a transition name.

    2.  <a id="ref-for-snapshot-containing-block①③"></a>

        Return this canvas as an image. The natural size of the image is equal to the [snapshot containing block](#snapshot-containing-block).

2.  Otherwise:

    1.  <a id="ref-for-concept-tree-descendant②"></a>

        <a id="ref-for-concept-node-document②"></a>

        <a id="ref-for-capture-rendering-characteristics①"></a>

        Render <var>element</var> and its [descendants](https://dom.spec.whatwg.org/#concept-tree-descendant), at the same size it appears in its [node document](https://dom.spec.whatwg.org/#concept-node-document), over an infinite transparent canvas, following the [capture rendering characteristics](#capture-rendering-characteristics).

    2.  <a id="ref-for-ink-overflow-rectangle"></a>

        <a id="ref-for-natural-dimensions②"></a>

        <a id="ref-for-principal-box①"></a>

        <a id="ref-for-border-box④"></a>

        <a id="ref-for-ink-overflow"></a>

        Return the portion of this canvas that includes <var>element</var>’s [ink overflow rectangle](https://www.w3.org/TR/css-overflow-3/#ink-overflow-rectangle) as an image. The [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions) of this image must be those of its [principal](https://www.w3.org/TR/css-display-3/#principal-box) [border box](https://www.w3.org/TR/css-box-4/#border-box), and its origin must correspond to that <a id="ref-for-border-box⑤"></a>border box's origin, such that the image represents the contents of this <a id="ref-for-border-box⑥"></a>border box and any captured [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) is represented outside these bounds.

        <a id="ref-for-replaced-element③"></a>

        <a id="ref-for-natural-size"></a>

        <a id="ref-for-principal-box②"></a>

        <a id="ref-for-ink-overflow①"></a>

        <a id="ref-for-content-box"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: When this image is rendered as a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) at its [natural size](https://www.w3.org/TR/css-images-3/#natural-size), it will display with the size and contents of element’s [principal box](https://www.w3.org/TR/css-display-3/#principal-box), with any captured [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) overflowing its [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-capture-rendering-characteristics②"></a>

#### <a id="capture-rendering-characteristics-algorithm"></a>7.7.1. [Capture rendering characteristics](#capture-rendering-characteristics)

The <a id="capture-rendering-characteristics"></a>capture rendering characteristics are as follows:

- If the referenced element has a transform applied to it (or its ancestors), then the transform is ignored.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This transform is applied to the snapshot using the `transform` property of the associated ::view-transition-group pseudo-element.

- <a id="ref-for-propdef-opacity①"></a>

  <a id="ref-for-propdef-filter①"></a>

  Effects applied on the element and its descendants, such as [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) and [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter), are applied to the capture. Effects applied to the element from its ancestors are ignored.

- <a id="ref-for-ink-overflow-rectangle①"></a>

  <a id="ref-for-implementation-defined①"></a>

  <a id="ref-for-snapshot-containing-block①④"></a>

  <a id="ref-for-ink-overflow-region"></a>

  Implementations may clip the rendered contents if the [ink overflow rectangle](https://www.w3.org/TR/css-overflow-3/#ink-overflow-rectangle) exceeds some [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined) maximum. However, the captured image should include, at the very least, the contents of <var>element</var> that intersect with the [snapshot containing block](#snapshot-containing-block). Implementations may adjust the rasterization quality to account for elements with a large [ink overflow area](https://www.w3.org/TR/css-overflow-3/#ink-overflow-region) that are transformed into view.

- <a id="ref-for-list-iterate⑤"></a>

  <a id="ref-for-concept-shadow-including-descendant"></a>

  <a id="ref-for-element①①"></a>

  <a id="ref-for-pseudo-element①⓪"></a>

  <a id="ref-for-captured-in-a-view-transition⑦"></a>

  [For each](https://infra.spec.whatwg.org/#list-iterate) <var>descendant</var> of [shadow-including descendant](https://dom.spec.whatwg.org/#concept-shadow-including-descendant) <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> and [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) of <var>element</var>, if <var>descendant</var> is [captured in a view transition](#captured-in-a-view-transition), then skip painting <var>descendant</var>.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This is necessary since the descendant will generate its own snapshot which will be displayed and animated independently.

<a id="ref-for-handle-transition-frame①"></a>

### <a id="handle-transition-frame-algorithm"></a>7.8. [Handle transition frame](#handle-transition-frame)

<a id="ref-for-viewtransition②①"></a>

To <a id="handle-transition-frame"></a>handle transition frame given a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

1.  <a id="ref-for-concept-relevant-global①⓪"></a>

    <a id="ref-for-concept-document-window⑦"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  Let <var>hasActiveAnimations</var> be a boolean, initially false.

3.  <a id="ref-for-list-iterate⑥"></a>

    <a id="ref-for-viewtransition-transition-root-pseudo-element⑤"></a>

    <a id="ref-for-concept-tree-inclusive-descendant①"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of <var>transition</var>’s [transition root pseudo-element](#viewtransition-transition-root-pseudo-element)'s [inclusive descendants](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant):

    1.  <a id="ref-for-timeline①"></a>

        <a id="ref-for-document-timeline"></a>

        <a id="ref-for-animation-associated-effect"></a>

        <a id="ref-for-keyframe-effect-effect-target"></a>

        For each <var>animation</var> whose [timeline](https://www.w3.org/TR/web-animations-1/#timeline) is a [document timeline](https://www.w3.org/TR/web-animations-1/#document-timeline) associated with <var>document</var>, and contains at least one [associated effect](https://www.w3.org/TR/web-animations-1/#animation-associated-effect) whose [effect target](https://www.w3.org/TR/web-animations-1/#keyframe-effect-effect-target) is <var>element</var>, set <var>hasActiveAnimations</var> to true if any of the following conditions is true:

        - <a id="ref-for-animation-play-state"></a>

          <a id="ref-for-play-state-paused"></a>

          <a id="ref-for-play-state-running"></a>

          <var>animation</var>’s [play state](https://www.w3.org/TR/web-animations-1/#animation-play-state) is [paused](https://www.w3.org/TR/web-animations-1/#play-state-paused) or [running](https://www.w3.org/TR/web-animations-1/#play-state-running).

        - <a id="ref-for-pending-animation-event-queue"></a>

          <var>document</var>’s [pending animation event queue](https://www.w3.org/TR/web-animations-1/#pending-animation-event-queue) has any events associated with <var>animation</var>.

4.  If <var>hasActiveAnimations</var> is false:

    1.  <a id="ref-for-viewtransition-phase①⑤"></a>

        Set <var>transition</var>’s [phase](#viewtransition-phase) to "`done`".

    2.  <a id="ref-for-clear-view-transition②"></a>

        [Clear view transition](#clear-view-transition) <var>transition</var>.

    3.  <a id="ref-for-resolve④"></a>

        <a id="ref-for-viewtransition-finished-promise③"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s [finished promise](#viewtransition-finished-promise).

    4.  Return.

5.  <a id="ref-for-viewtransition-initial-snapshot-containing-block-size②"></a>

    <a id="ref-for-snapshot-containing-block-size④"></a>

    <a id="ref-for-skip-the-view-transition①⑦"></a>

    If <var>transition</var>’s [initial snapshot containing block size](#viewtransition-initial-snapshot-containing-block-size) is not equal to the [snapshot containing block size](#snapshot-containing-block-size), then [skip the view transition](#skip-the-view-transition) for <var>transition</var>, and return.

6.  <a id="ref-for-update-pseudo-element-styles④"></a>

    [Update pseudo-element styles](#update-pseudo-element-styles) for <var>transition</var>.

    <a id="ref-for-skip-the-view-transition①⑧"></a>

    <a id="ref-for-invalidstateerror⑤"></a>

    <a id="ref-for-idl-DOMException⑧"></a>

    <a id="ref-for-concept-relevant-realm⑨"></a>

    If failure is returned, then [skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> in <var>transition</var>’s [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The above implies that a change in incoming element’s size or position will cause a new keyframe to be generated. This can cause a visual jump. We could retarget smoothly but don’t have a use-case to justify the complexity. See [issue 7813](https://github.com/w3c/csswg-drafts/issues/7813) for details.

<a id="ref-for-update-pseudo-element-styles⑤"></a>

### <a id="style-transition-pseudo-elements-algorithm"></a>7.9. [Update pseudo-element styles](#update-pseudo-element-styles)

<a id="ref-for-viewtransition②②"></a>

To <a id="update-pseudo-element-styles"></a>update pseudo-element styles for a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

1.  <a id="ref-for-map-iterate①"></a>

    <a id="ref-for-viewtransition-named-elements④"></a>

    [For each](https://infra.spec.whatwg.org/#map-iterate) <var>transitionName</var> → <var>capturedElement</var> of <var>transition</var>’s [named elements](#viewtransition-named-elements):

    1.  Let <var>width</var>, <var>height</var>, <var>transform</var>, <var>writingMode</var>, <var>direction</var>, <var>textOrientation</var>, <var>mixBlendMode</var>, <var>backdropFilter</var> and <var>colorScheme</var> be null.

    2.  <a id="ref-for-captured-element-new-element⑧"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is null, then:

        1.  <a id="ref-for-captured-element-old-width②"></a>

            Set <var>width</var> to <var>capturedElement</var>’s [old width](#captured-element-old-width).

        2.  <a id="ref-for-captured-element-old-height②"></a>

            Set <var>height</var> to <var>capturedElement</var>’s [old height](#captured-element-old-height).

        3.  <a id="ref-for-captured-element-old-transform②"></a>

            Set <var>transform</var> to <var>capturedElement</var>’s [old transform](#captured-element-old-transform).

        4.  <a id="ref-for-captured-element-old-writing-mode①"></a>

            Set <var>writingMode</var> to <var>capturedElement</var>’s [old writing-mode](#captured-element-old-writing-mode).

        5.  <a id="ref-for-captured-element-old-direction①"></a>

            Set <var>direction</var> to <var>capturedElement</var>’s [old direction](#captured-element-old-direction).

        6.  <a id="ref-for-captured-element-old-text-orientation①"></a>

            Set <var>textOrientation</var> to <var>capturedElement</var>’s [old text-orientation](#captured-element-old-text-orientation).

        7.  <a id="ref-for-captured-element-old-mix-blend-mode①"></a>

            Set <var>mixBlendMode</var> to <var>capturedElement</var>’s [old mix-blend-mode](#captured-element-old-mix-blend-mode).

        8.  <a id="ref-for-captured-element-old-backdrop-filter②"></a>

            Set <var>backdropFilter</var> to <var>capturedElement</var>’s [old backdrop-filter](#captured-element-old-backdrop-filter).

        9.  <a id="ref-for-captured-element-old-color-scheme①"></a>

            Set <var>colorScheme</var> to <var>capturedElement</var>’s [old color-scheme](#captured-element-old-color-scheme).

    3.  Otherwise:

        1.  Return failure if any of the following conditions is true:

            - <a id="ref-for-captured-element-new-element⑨"></a>

              <a id="ref-for-flat-tree②"></a>

              <a id="ref-for-skips-its-contents③"></a>

              <var>capturedElement</var>’s [new element](#captured-element-new-element) has a [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) ancestor that [skips its contents](https://www.w3.org/TR/css-contain-2/#skips-its-contents).

            - <a id="ref-for-captured-element-new-element①⓪"></a>

              <a id="ref-for-element-not-rendered③"></a>

              <var>capturedElement</var>’s [new element](#captured-element-new-element) is [not rendered](https://www.w3.org/TR/css-images-4/#element-not-rendered).

            - <a id="ref-for-box-fragment②"></a>

              <var>capturedElement</var> has more than one [box fragment](https://www.w3.org/TR/css-break-4/#box-fragment).

            <a id="ref-for-captured-element-new-element①①"></a>

            <a id="ref-for-captured-in-a-view-transition⑧"></a>

            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Note: Other rendering constraints are enforced via <var>capturedElement</var>’s [new element](#captured-element-new-element) being [captured in a view transition](#captured-in-a-view-transition).

        2.  <a id="ref-for-captured-element-new-element①②"></a>

            <a id="ref-for-border-box⑦"></a>

            Set <var>width</var> to the current width of <var>capturedElement</var>’s [new element](#captured-element-new-element)'s [border box](https://www.w3.org/TR/css-box-4/#border-box).

        3.  <a id="ref-for-captured-element-new-element①③"></a>

            <a id="ref-for-border-box⑧"></a>

            Set <var>height</var> to the current height of <var>capturedElement</var>’s [new element](#captured-element-new-element)'s [border box](https://www.w3.org/TR/css-box-4/#border-box).

        4.  <a id="ref-for-captured-element-new-element①④"></a>

            <a id="ref-for-border-box⑨"></a>

            <a id="ref-for-snapshot-containing-block-origin①"></a>

            Set <var>transform</var> to a transform that would map <var>capturedElement</var>’s [new element](#captured-element-new-element)'s [border box](https://www.w3.org/TR/css-box-4/#border-box) from the [snapshot containing block origin](#snapshot-containing-block-origin) to its current visual position.

        5.  <a id="ref-for-computed-value①⓪"></a>

            <a id="ref-for-propdef-writing-mode②"></a>

            <a id="ref-for-captured-element-new-element①⑤"></a>

            Set <var>writingMode</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

        6.  <a id="ref-for-computed-value①①"></a>

            <a id="ref-for-propdef-direction②"></a>

            <a id="ref-for-captured-element-new-element①⑥"></a>

            Set <var>direction</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

        7.  <a id="ref-for-computed-value①②"></a>

            <a id="ref-for-propdef-text-orientation②"></a>

            <a id="ref-for-captured-element-new-element①⑦"></a>

            Set <var>textOrientation</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

        8.  <a id="ref-for-computed-value①③"></a>

            <a id="ref-for-propdef-mix-blend-mode④"></a>

            <a id="ref-for-captured-element-new-element①⑧"></a>

            Set <var>mixBlendMode</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

        9.  <a id="ref-for-computed-value①④"></a>

            <a id="ref-for-propdef-backdrop-filter②"></a>

            <a id="ref-for-captured-element-new-element①⑨"></a>

            Set <var>backdropFilter</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [backdrop-filter](https://drafts.fxtf.org/filter-effects-2/#propdef-backdrop-filter) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

        10. <a id="ref-for-computed-value①⑤"></a>

            <a id="ref-for-propdef-color-scheme②"></a>

            <a id="ref-for-captured-element-new-element②⓪"></a>

            Set <var>colorScheme</var> to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [color-scheme](https://www.w3.org/TR/css-color-adjust-1/#propdef-color-scheme) on <var>capturedElement</var>’s [new element](#captured-element-new-element).

    4.  <a id="ref-for-captured-element-group-styles-rule"></a>

        <a id="ref-for-cssstylerule⑨"></a>

        <a id="ref-for-concept-relevant-global①①"></a>

        <a id="ref-for-concept-document-window⑧"></a>

        <a id="ref-for-document-dynamic-view-transition-style-sheet①①"></a>

        If <var>capturedElement</var>’s [group styles rule](#captured-element-group-styles-rule) is null, then set <var>capturedElement</var>’s <a id="ref-for-captured-element-group-styles-rule①"></a>group styles rule to a new <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> representing the following CSS, and append it to <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window)'s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet).

        <a id="ref-for-captured-element-group-styles-rule②"></a>

        Otherwise, update <var>capturedElement</var>’s [group styles rule](#captured-element-group-styles-rule) to match the following CSS:

        ```text
        :root::view-transition-group(transitionName) {
          width: width;
          height: height;
          transform: transform;
          writing-mode: writingMode;
          direction: direction;
          text-orientation: textOrientation;
          mix-blend-mode: mixBlendMode;
          backdrop-filter: backdropFilter;
          color-scheme: colorScheme;
        }
        ```
        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: The above code example contains variables to be replaced.

    5.  <a id="ref-for-captured-element-new-element②①"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is not null, then:

        1.  <a id="ref-for-selectordef-view-transition-new⑨"></a>

            <a id="ref-for-view-transition-name①⑤"></a>

            Let <var>new</var> be the [::view-transition-new()](#selectordef-view-transition-new) with the [view transition name](#view-transition-name) <var>transitionName</var>.

        2.  <a id="ref-for-replaced-element④"></a>

            <a id="ref-for-capture-the-image⑥"></a>

            <a id="ref-for-captured-element-new-element②②"></a>

            Set <var>new</var>’s [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) content to the result of [capturing the image](#capture-the-image) of <var>capturedElement</var>’s [new element](#captured-element-new-element).

<a id="ref-for-cascade-origin-ua③"></a>

This algorithm must be executed to update styles in [user-agent origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua) if its effects can be observed by a web API.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An example of such a web API is `window.getComputedStyle(document.documentElement, "::view-transition")`.

<a id="ref-for-clear-view-transition③"></a>

### <a id="clear-view-transition-algorithm"></a>7.10. [Clear view transition](#clear-view-transition)

<a id="ref-for-viewtransition②③"></a>

To <a id="clear-view-transition"></a>clear view transition of a <code><a href="#viewtransition">ViewTransition</a></code> <var>transition</var>:

1.  <a id="ref-for-concept-relevant-global①②"></a>

    <a id="ref-for-concept-document-window⑨"></a>

    Let <var>document</var> be <var>transition</var>’s [relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) [associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-assert⑤"></a>

    <a id="ref-for-document-active-view-transition①⑤"></a>

    [Assert](https://infra.spec.whatwg.org/#assert): <var>document</var>’s [active view transition](#document-active-view-transition) is <var>transition</var>.

3.  <a id="ref-for-list-iterate⑦"></a>

    <a id="ref-for-viewtransition-named-elements⑤"></a>

    <a id="ref-for-map-getting-the-values①"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>capturedElement</var> of <var>transition</var>’s [named elements](#viewtransition-named-elements)' [values](https://infra.spec.whatwg.org/#map-getting-the-values):

    1.  <a id="ref-for-captured-element-new-element②③"></a>

        <a id="ref-for-captured-in-a-view-transition⑨"></a>

        If <var>capturedElement</var>’s [new element](#captured-element-new-element) is not null, then set <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element②④"></a>new element's [captured in a view transition](#captured-in-a-view-transition) to false.

    2.  <a id="ref-for-list-iterate⑧"></a>

        <a id="ref-for-captured-element-style-definitions"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) <var>style</var> of <var>capturedElement</var>’s [style definitions](#captured-element-style-definitions):

        1.  <a id="ref-for-document-dynamic-view-transition-style-sheet①②"></a>

            If <var>style</var> is not null, and <var>style</var> is in <var>document</var>’s [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet), then remove <var>style</var> from <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet①③"></a>dynamic view transition style sheet.

4.  <a id="ref-for-document-show-view-transition-tree①"></a>

    Set <var>document</var>’s [show view transition tree](#document-show-view-transition-tree) to false.

5.  <a id="ref-for-document-active-view-transition①⑥"></a>

    Set <var>document</var>’s [active view transition](#document-active-view-transition) to null.

## <a id="priv"></a>Privacy Considerations

This specification introduces no new privacy considerations.

## <a id="sec"></a>Security Considerations

<a id="ref-for-capture-the-image⑦"></a>

The images generated using [capture the image](#capture-the-image) algorithm could contain cross-origin data (if the Document is embedding cross-origin resources) or sensitive information like visited links. The implementations must ensure this data can not be accessed by the Document. This should be feasible since access to this data should already be prevented in the default rendering of the Document.

## <a id="changes"></a>Appendix A. Changes

This appendix is <em>informative</em>.

### <a id="changes-since-2022-05-30"></a> Changes from [2022-05-30 Working Draft](https://www.w3.org/TR/2023/WD-css-view-transitions-1-20230530/) 

- Use a keyframe to add plus-lighter blending during cross-fade. See [issue 8924](https://github.com/w3c/csswg-drafts/issues/8924).

- Add mix-blend-mode to list of properties copied over to the ::view-transition-group. See [issue 8962](https://github.com/w3c/csswg-drafts/issues/8962).

- Add text-orientation to list of properties copied over to the ::view-transition-group. See [issue 8230](https://github.com/w3c/csswg-drafts/issues/8230).

- <a id="ref-for-captured-in-a-view-transition①⓪"></a>

  Refactor the old capture algorithm to properly set [captured in a view transition](#captured-in-a-view-transition) before reading the value.

- <a id="ref-for-dom-document-startviewtransition⑤"></a>

  Make the <code><a href="#dom-document-startviewtransition">startViewTransition()</a></code> parameter non-nullable. See [issue 9460](https://github.com/w3c/csswg-drafts/issues/9460).

- <a id="ref-for-view-transitions①③"></a>

  Elements participating in a [view transition](#view-transitions) are exposed to accessibility tree. See [issue 9365](https://github.com/w3c/csswg-drafts/issues/9365).

- <a id="ref-for-view-transition-tree④"></a>

  The [view transition tree](#view-transition-tree) is not exposed to accessibility tree. See [issue 9365](https://github.com/w3c/csswg-drafts/issues/9365).

- Animate back-drop filter similar to transform/size. See [issue 9358](https://github.com/w3c/csswg-drafts/issues/9358).

- <a id="ref-for-selectordef-view-transition-group①⑦"></a>

  Copy `color-scheme` from DOM element to [::view-transition-group()](#selectordef-view-transition-group). See [issue 9276](https://github.com/w3c/csswg-drafts/issues/9276).

- <a id="ref-for-document①①"></a>

  Expose auto-skip view transition for a <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>, to allow having outbound cross-document transitions preceed programmatic view transiitons. see [issue 9512](https://github.com/w3c/csswg-drafts/issues/9512).

- <a id="ref-for-propdef-view-transition-name①③"></a>

  Add a note about why [view-transition-name](#propdef-view-transition-name) should be animatable.

- `view-transition-name: auto` should be an invalid value. See [issue 9639](https://github.com/w3c/csswg-drafts/issues/9639).

- Add note to explain paint order for entry animations. See [issue 9672](https://github.com/w3c/csswg-drafts/issues/9672).

- Add note to explain how the named elements are cleaned up. See [issue 9669](https://github.com/w3c/csswg-drafts/issues/9669).

- Refactor algorithm to clarify timing, especially of \`updateCallbackDone. See [issue 9762](https://github.com/w3c/csswg-drafts/issues/9762).

- Add animation-delay inherit to UA stylesheet rules for (::view-transition) -image-pair, -old, and -new. See [issue 9817](https://github.com/w3c/csswg-drafts/issues/9817).

- Auto-skip animation when document is hidden. See [issue 9543](https://github.com/w3c/csswg-drafts/issues/9543).

- Remove references to cross-document view-transitions, to keep the L1 spec clean. See [Issue 9886](https://github.com/w3c/csswg-drafts/issues/9886).

- Export an algorithm to skip the active transition when the page is hidden. See [issue 9543](https://github.com/w3c/csswg-drafts/issues/9543).

### <a id="changes-since-2022-05-25"></a> Changes from [2022-05-25 Working Draft](https://www.w3.org/TR/2023/WD-css-view-transitions-1-20230525/) 

- Fix typo in ::view-transition-new user agent style sheet. See [PR](https://github.com/w3c/csswg-drafts/pull/8879).

### <a id="changes-since-2022-11-24"></a> Changes from [2022-11-24 Working Draft](https://www.w3.org/TR/2022/WD-css-view-transitions-1-20221124/) 

- Pointer events resolve to the documentElement when rendering is suppressed. See [issue 7797](https://github.com/w3c/csswg-drafts/issues/7797).

- Add rendering constraints to elements participating in a transition. See [issue 8139](https://github.com/w3c/csswg-drafts/issues/8139) and [issue 7882](https://github.com/w3c/csswg-drafts/issues/7882).

- Remove html specifics from UA stylesheet to support ViewTransitions on SVG Documents.

- <a id="ref-for-callbackdef-updatecallback①⑥"></a>

  Rename updateDOMCallback to <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code>. See [issue 8144](https://github.com/w3c/csswg-drafts/issues/8144).

- <a id="ref-for-snapshot-containing-block①⑤"></a>

  Rename snapshot viewport to [snapshot containing block](#snapshot-containing-block).

- Skip the transition if viewport size changes. See [issue 8045](https://github.com/w3c/csswg-drafts/issues/8045).

- Add support for :only-child. See [issue 8057](https://github.com/w3c/csswg-drafts/issues/8057).

- <a id="ref-for-pseudo-element-root②"></a>

  Add concept of a tree of pseudo-elements under [pseudo-element root](#pseudo-element-root). See [issue 8113](https://github.com/w3c/csswg-drafts/issues/8113).

- <a id="ref-for-callbackdef-updatecallback①⑦"></a>

  When skipping a transition, the <code><a href="#callbackdef-updatecallback">UpdateCallback</a></code> is called in own task rather than synchronously. See [issue 7904](https://github.com/w3c/csswg-drafts/issues/7904)

- When capturing images, at least the in-viewport part of the image should be captured, downscale if needed. See [issue 8561](https://github.com/w3c/csswg-drafts/issues/8561).

- <a id="ref-for-ink-overflow②"></a>

  <a id="ref-for-natural-size①"></a>

  Applying the [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) to the captured image is implementation defined, and doesn’t affect the image’s [natural size](https://www.w3.org/TR/css-images-3/#natural-size). See [issue 8597](https://github.com/w3c/csswg-drafts/issues/8597).

- Fragmented elements don’t participate in view transitions. See [issue 8339](https://github.com/w3c/csswg-drafts/issues/8339).

- <a id="ref-for-absolute-positioning-containing-block①"></a>

  <a id="ref-for-fixed-positioning-containing-block①"></a>

  Rename "snapshot root" to "snapshot containing block", and make it an [absolute positioning containing block](https://www.w3.org/TR/css-position-3/#absolute-positioning-containing-block) and a [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block) for its descendants. See [issue 8505](https://github.com/w3c/csswg-drafts/issues/8505).

### <a id="changes-since-2022-10-25"></a> Changes from [2022-10-25 Working Draft (FPWD)](https://www.w3.org/TR/2022/WD-css-view-transitions-1-20221025/) 

- <a id="ref-for-document-dynamic-view-transition-style-sheet①④"></a>

  Add [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet) concept for dynamically generated UA styles scoped to the current Document.

- Add snapshot viewport concept. See [issue 7859](https://github.com/w3c/csswg-drafts/issues/7859).

- Clarify timing for resolving/rejecting promises when skipping the transition. See [issue 7956](https://github.com/w3c/csswg-drafts/issues/7956).

- Elements under a content-visibility:auto element that skips its contents are ignored. See [issue 7874](https://github.com/w3c/csswg-drafts/issues/7874).

- UA styles on the pseudo-DOM stay in sync with author DOM for any developer observable API. See [issue 7812](https://github.com/w3c/csswg-drafts/issues/7812).

- Suppress rendering during updateCallback. See [issue 7784](https://github.com/w3c/csswg-drafts/issues/7784).

- Changes in size/position of elements in the new Document generate new UA animation keyframes. See [issue 7813](https://github.com/w3c/csswg-drafts/issues/7813).

- Scope keyframes to user agent stylesheets using -ua- prefix. See [issue 7560](https://github.com/w3c/csswg-drafts/issues/7560).

- Update pseudo element names to view-transition\*. See [issue 7960](https://github.com/w3c/csswg-drafts/issues/7960).

- Update selector syntax for pseudo-elements. See [issue 7788](https://github.com/w3c/csswg-drafts/issues/7788).

- Add sections for security/privacy considerations.

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

- [activate view transition](#activate-view-transition), in § 7.3
- [active view transition](#document-active-view-transition), in § 7.1.1
- [call the update callback](#call-the-update-callback), in § 7.4
- [captured element](#captured-element), in § 7.1.3
- [captured in a view transition](#captured-in-a-view-transition), in § 7.1.2
- [capture rendering characteristics](#capture-rendering-characteristics), in § 7.7.1
- [capture the image](#capture-the-image), in § 7.7
- [capture the new state](#capture-the-new-state), in § 7.3.2
- [capture the old state](#capture-the-old-state), in § 7.3.1
- [capturing the image](#capture-the-image), in § 7.7
- [clear view transition](#clear-view-transition), in § 7.10
- [\<custom-ident\>](#valdef-view-transition-name-custom-ident), in § 2.1
- [dynamic view transition style sheet](#document-dynamic-view-transition-style-sheet), in § 7.1.1
- [finished](#dom-viewtransition-finished), in § 6.2
- [finished promise](#viewtransition-finished-promise), in § 6.2
- [global view transition user agent style sheet](#global-view-transition-user-agent-style-sheet), in § 5
- [group animation name rule](#captured-element-group-animation-name-rule), in § 7.1.3
- [group keyframes](#captured-element-group-keyframes), in § 7.1.3
- [group styles rule](#captured-element-group-styles-rule), in § 7.1.3
- [handle transition frame](#handle-transition-frame), in § 7.8
- [image animation name rule](#captured-element-image-animation-name-rule), in § 7.1.3
- [image pair isolation rule](#captured-element-image-pair-isolation-rule), in § 7.1.3
- [initial snapshot containing block size](#viewtransition-initial-snapshot-containing-block-size), in § 6.2
- [named elements](#viewtransition-named-elements), in § 6.2
- [named view transition pseudo-elements](#named-view-transition-pseudo-elements), in § 3.2.1
- [new element](#captured-element-new-element), in § 7.1.3
- [none](#valdef-view-transition-name-none), in § 2.1
- [old backdrop-filter](#captured-element-old-backdrop-filter), in § 7.1.3
- [old color-scheme](#captured-element-old-color-scheme), in § 7.1.3
- [old direction](#captured-element-old-direction), in § 7.1.3
- [old height](#captured-element-old-height), in § 7.1.3
- [old image](#captured-element-old-image), in § 7.1.3
- [old mix-blend-mode](#captured-element-old-mix-blend-mode), in § 7.1.3
- [old text-orientation](#captured-element-old-text-orientation), in § 7.1.3
- [old transform](#captured-element-old-transform), in § 7.1.3
- [old width](#captured-element-old-width), in § 7.1.3
- [old writing-mode](#captured-element-old-writing-mode), in § 7.1.3
- [perform pending transition operations](#perform-pending-transition-operations), in § 7.2
- [phase](#viewtransition-phase), in § 6.2
- [pseudo-element root](#pseudo-element-root), in § 3.1
- [pseudo-element tree](#pseudo-element-tree), in § 3.1
- [\<pt-name-selector\>](#typedef-pt-name-selector), in § 3.2.1
- [ready](#dom-viewtransition-ready), in § 6.2
- [ready promise](#viewtransition-ready-promise), in § 6.2
- [rendering suppression for view transitions](#document-rendering-suppression-for-view-transitions), in § 7.1.1
- [setup transition pseudo-elements](#setup-transition-pseudo-elements), in § 7.3.3
- [setup view transition](#setup-view-transition), in § 7.3
- [show view transition tree](#document-show-view-transition-tree), in § 7.1.1
- [skip the view transition](#skip-the-view-transition), in § 7.5
- [skipTransition()](#dom-viewtransition-skiptransition), in § 6.2.1
- [snapshot containing block](#snapshot-containing-block), in § 4.1
- [snapshot containing block origin](#snapshot-containing-block-origin), in § 4.1
- [snapshot containing block size](#snapshot-containing-block-size), in § 4.1
- [startViewTransition()](#dom-document-startviewtransition), in § 6.1.1
- [startViewTransition(updateCallback)](#dom-document-startviewtransition), in § 6.1.1
- [style definitions](#captured-element-style-definitions), in § 7.1.3
- [transition root pseudo-element](#viewtransition-transition-root-pseudo-element), in § 6.2
- [update callback](#viewtransition-update-callback), in § 6.2
- [UpdateCallback](#callbackdef-updatecallback), in § 6.1
- [updateCallbackDone](#dom-viewtransition-updatecallbackdone), in § 6.2
- [update callback done promise](#viewtransition-update-callback-done-promise), in § 6.2
- [update pseudo-element styles](#update-pseudo-element-styles), in § 7.9
- [::view-transition](#selectordef-view-transition), in § 3.2.2
- [ViewTransition](#viewtransition), in § 6.2
- [::view-transition-group()](#selectordef-view-transition-group), in § 3.2.3
- [::view-transition-image-pair()](#selectordef-view-transition-image-pair), in § 3.2.4
- [view transition layer](#view-transition-layer), in § 4.2
- [view transition name](#view-transition-name), in § 2.1
- [view-transition-name](#propdef-view-transition-name), in § 2.1
- [::view-transition-new()](#selectordef-view-transition-new), in § 3.2.6
- [::view-transition-old()](#selectordef-view-transition-old), in § 3.2.5
- [view transition page-visibility change steps](#view-transition-page-visibility-change-steps), in § 7.6
- [view transition pseudo-elements](#view-transition-pseudo-elements), in § 3.2
- [view transitions](#view-transitions), in § 1
- [view transition tree](#view-transition-tree), in § 3.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[\] defines the following terms:
  - <a id="2bc52ca2"></a>NavigateEvent
  - <a id="a3331757"></a>signal
- \[COMPOSITING-1\] defines the following terms:
  - <a id="ecfe8e64"></a>isolation
  - <a id="3249d67d"></a>mix-blend-mode
- \[CSS-2022\] defines the following terms:
  - <a id="226d9efe"></a>style sheet
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="256cde36"></a>CSSKeyframesRule
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="fd42d148"></a>canvas background
- \[CSS-BOX-4\] defines the following terms:
  - <a id="85c399c0"></a>border box
  - <a id="f72f5cb4"></a>content box
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="64bdec0d"></a>fragment
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="a9479ee7"></a>user-agent origin
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="3b7558dc"></a>opacity
- \[CSS-COLOR-ADJUST-1\] defines the following terms:
  - <a id="b75080dc"></a>color-scheme
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="644cdfa3"></a>element contents
  - <a id="0830f093"></a>skips its contents
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="6b4fc208"></a>containing block
  - <a id="e26aa9bf"></a>initial containing block
  - <a id="93f98063"></a>principal box
  - <a id="299e10e4"></a>replaced element
  - <a id="8b4f8a45"></a>root element
  - <a id="d3ff9a69"></a>visibility
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="487e1aa9"></a>natural dimension
  - <a id="c0cc78c8"></a>natural size
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="1389fc39"></a>element-not-rendered
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="e2b06daa"></a>clip-path
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="00d2e365"></a>ink overflow
  - <a id="1c933e34"></a>ink overflow area
  - <a id="0938e0be"></a>ink overflow rectangle
  - <a id="add377f4"></a>overflow
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="8bf8e632"></a>absolute positioning containing block
  - <a id="1febb260"></a>fixed positioning containing block
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="8f59bb58"></a>top layer
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="b5ecbd4e"></a>tree-abiding pseudo-element
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="22109b0e"></a>flat tree
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="49731d1d"></a>width
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="4525418a"></a>identity transform function
  - <a id="e7c6bf78"></a>transform
- \[CSS-TRANSFORMS-2\] defines the following terms:
  - <a id="c43222a2"></a>\<transform-function\>
- \[CSS-UI-4\] defines the following terms:
  - <a id="73bc6606"></a>pointer-events
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="4eb9d37e"></a>\|
- \[CSS-VIEWPORT-1\] defines the following terms:
  - <a id="d37bcaf0"></a>interactive-widget
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="8664e85f"></a>text-orientation
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="2161cf2b"></a>pseudo-elements
  - <a id="a50c2771"></a>stacking context
- \[CSS22\] defines the following terms:
  - <a id="e8b75507"></a>element
- \[CSSOM-1\] defines the following terms:
  - <a id="d293a05b"></a>CSSStyleRule
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="fafa15bc"></a>layout viewport
  - <a id="4186822a"></a>scrolling box
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document
  - <a id="296f3551"></a>Element
  - <a id="84f47c16"></a>child
  - <a id="9638f92b"></a>connected
  - <a id="da8b8e0e"></a>descendant
  - <a id="a973e0fe"></a>document
  - <a id="2f0ba72c"></a>document element
  - <a id="f9d909f7"></a>inclusive descendant
  - <a id="5216e1a0"></a>node document
  - <a id="d729a9ff"></a>parent
  - <a id="58df5166"></a>participate
  - <a id="f7960529"></a>root
  - <a id="fd32e3c9"></a>shadow-including descendant
  - <a id="ad10ee29"></a>sibling
  - <a id="e6cc3311"></a>tree
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="e9f6aadb"></a>filter
- \[FILTER-EFFECTS-2\] defines the following terms:
  - <a id="31c41a75"></a>backdrop root
  - <a id="54fa0afb"></a>backdrop-filter
- \[GEOMETRY-1\] defines the following terms:
  - <a id="b904c22d"></a>height
  - <a id="c7cb8568"></a>width
- \[HR-TIME-3\] defines the following terms:
  - <a id="8297dfb0"></a>duration
- \[HTML\] defines the following terms:
  - <a id="3349d69f"></a>associated document
  - <a id="e10e7eb7"></a>dom manipulation task source
  - <a id="a72449dd"></a>in parallel
  - <a id="48438eb3"></a>queue a global task
  - <a id="e99bd18e"></a>relevant global object
  - <a id="5991ccfb"></a>relevant realm
  - <a id="f1041b37"></a>unhandledrejection
  - <a id="b4c365f8"></a>visibility state
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append (for list)
  - <a id="a3b18719"></a>append (for set)
  - <a id="77b4c09a"></a>assert
  - <a id="ae8def21"></a>contain
  - <a id="f937b7b6"></a>continue
  - <a id="1243a891"></a>exist
  - <a id="16d07e10"></a>for each (for list)
  - <a id="45209803"></a>for each (for map)
  - <a id="860300d4"></a>implementation-defined
  - <a id="649608b9"></a>list
  - <a id="3fca5a9e"></a>map
  - <a id="15e48c39"></a>set
  - <a id="984221ca"></a>struct
  - <a id="0e8de730"></a>tuple
  - <a id="12d6b9a8"></a>values
- \[INTERSECTION-OBSERVER\] defines the following terms:
  - <a id="8e7046bc"></a>IntersectionObserver
- \[POINTEREVENTS3\] defines the following terms:
  - <a id="95302000"></a>pointer capture
- \[SCROLL-ANIMATIONS-1\] defines the following terms:
  - <a id="6fac9c76"></a>scroll-driven animations
- \[SELECTORS-3\] defines the following terms:
  - <a id="dfd67b05"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="32fb2b6a"></a>:only-child
  - <a id="2e965368"></a>functional pseudo-element
  - <a id="7b5d8638"></a>originating element
  - <a id="00ffc4c8"></a>originating pseudo-element
  - <a id="4d06fa38"></a>pseudo-element
  - <a id="388bc3fc"></a>selector
  - <a id="15e81c46"></a>type selector
  - <a id="fc8ac26a"></a>ultimate originating element
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="f3133dfc"></a>associated effect
  - <a id="2cec4673"></a>discrete
  - <a id="fe9b8cc4"></a>document timeline
  - <a id="8a965e7d"></a>effect target
  - <a id="59df29fe"></a>paused
  - <a id="ac0fba29"></a>pending animation event queue
  - <a id="bb293700"></a>play state
  - <a id="960dded0"></a>running
  - <a id="4c9d6912"></a>timeline
- \[WEBIDL\] defines the following terms:
  - <a id="d25dfb2c"></a>AbortError
  - <a id="dca2de17"></a>DOMException
  - <a id="889e932f"></a>Exposed
  - <a id="797018a7"></a>InvalidStateError
  - <a id="bdbd19d1"></a>Promise
  - <a id="f0ba495b"></a>TimeoutError
  - <a id="dacde8b5"></a>a new promise
  - <a id="8f5c2179"></a>a promise resolved with
  - <a id="6c6b1005"></a>any
  - <a id="833154f2"></a>getter steps
  - <a id="10ce5f6f"></a>invoke
  - <a id="1171257a"></a>mark as handled
  - <a id="3024958b"></a>method steps
  - <a id="750a2f08"></a>react
  - <a id="116848ac"></a>reacting
  - <a id="b262501e"></a>reject
  - <a id="3b90bdcd"></a>resolve
  - <a id="4013a022"></a>this
  - <a id="5f90bbfb"></a>undefined
  - <a id="11e0b87f"></a>unrestricted double

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-compositing-1"></a>\[COMPOSITING-1\]  
Chris Harrelson. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 21 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-2022"></a>\[CSS-2022\]  
Tab Atkins Jr.; Elika Etemad; Florian Rivoal. [CSS Snapshot 2022](https://www.w3.org/TR/css-2022/). 22 November 2022. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2022&#x2F;](https://www.w3.org/TR/css-2022/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 13 February 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-adjust-1"></a>\[CSS-COLOR-ADJUST-1\]  
Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/css-color-adjust-1/). 14 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-adjust-1&#x2F;](https://www.w3.org/TR/css-color-adjust-1/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 3 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-position-4"></a>\[CSS-POSITION-4\]  
[CSS Positioned Layout Module Level 4](https://drafts.csswg.org/css-position-4/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-position-4&#x2F;](https://drafts.csswg.org/css-position-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://www.w3.org/TR/css-transforms-2/). 9 November 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-2&#x2F;](https://www.w3.org/TR/css-transforms-2/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-viewport-1"></a>\[CSS-VIEWPORT-1\]  
Florian Rivoal; Emilio Cobos Álvarez. [CSS Viewport Module Level 1](https://www.w3.org/TR/css-viewport-1/). 25 January 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-viewport-1&#x2F;](https://www.w3.org/TR/css-viewport-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-filter-effects-2"></a>\[FILTER-EFFECTS-2\]  
[Filter Effects Module Level 2](https://drafts.fxtf.org/filter-effects-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;fxtf&#x2E;org&#x2F;filter-effects-2&#x2F;](https://drafts.fxtf.org/filter-effects-2/)

<a id="biblio-geometry-1"></a>\[GEOMETRY-1\]  
Simon Pieters; Chris Harrelson. [Geometry Interfaces Module Level 1](https://www.w3.org/TR/geometry-1/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;geometry-1&#x2F;](https://www.w3.org/TR/geometry-1/)

<a id="biblio-hr-time-3"></a>\[HR-TIME-3\]  
Yoav Weiss. [High Resolution Time](https://www.w3.org/TR/hr-time-3/). 19 July 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;hr-time-3&#x2F;](https://www.w3.org/TR/hr-time-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-intersection-observer"></a>\[INTERSECTION-OBSERVER\]  
Stefan Zager; Emilio Cobos Álvarez; Traian Captan. [Intersection Observer](https://www.w3.org/TR/intersection-observer/). 18 October 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;intersection-observer&#x2F;](https://www.w3.org/TR/intersection-observer/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-pointerevents3"></a>\[POINTEREVENTS3\]  
Patrick Lauke; Robert Flack. [Pointer Events](https://www.w3.org/TR/pointerevents3/). 18 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;pointerevents3&#x2F;](https://www.w3.org/TR/pointerevents3/)

<a id="biblio-scroll-animations-1"></a>\[SCROLL-ANIMATIONS-1\]  
Brian Birtles; et al. [Scroll-driven Animations](https://www.w3.org/TR/scroll-animations-1/). 6 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;scroll-animations-1&#x2F;](https://www.w3.org/TR/scroll-animations-1/)

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

<a id="ref-for-propdef-view-transition-name①④"></a>

[view-transition-name](#propdef-view-transition-name)

<strong>Column 2 (data cell):</strong>

none \| \<custom-ident\>

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

as specified

## <a id="idl-index"></a>IDL Index

```text
partial interface Document {
  ViewTransition startViewTransition(optional UpdateCallback updateCallback);
};

callback UpdateCallback = Promise<any> ();

[Exposed=Window]
interface ViewTransition {
  readonly attribute Promise<undefined> updateCallbackDone;
  readonly attribute Promise<undefined> ready;
  readonly attribute Promise<undefined> finished;
  undefined skipTransition();
};

```