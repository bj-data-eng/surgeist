Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Web Animations](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/).

Original copyright notice: Copyright © 2023 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Web Animations

Source snapshot: https://www.w3.org/TR/2023/WD-web-animations-1-20230605/

Snapshot SHA-256: e814a799ef2442a6c2f5bb783307080054441215ae75921349c018aaef3a2e55

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 1 MathML expressions are represented as portable fenced TeX. Independent round-trip checks cover mathematical tokens, matrix shape/order, scripts, fractions and root structure; exact source MathML is retained in verification metadata. Visual equivalence is not certified.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="ref-for-iteration-count⑨"></a>

<a id="ref-for-iteration-duration⑨"></a>

# <a id="title"></a>Web Animations

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2023 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This specification defines a model for synchronization and timing of changes to the presentation of a Web page. This specification also defines an application programming interface for interacting with this model and it is expected that further specifications will define declarative means for exposing these features.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “web-animations” in the title, like this: “\[web-animations\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bweb-animations%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="introduction"></a>1. Introduction

<em>This section is non-normative</em>

Web Animations defines a model for supporting animation and synchronization on the Web platform. It is intended that other specifications will build on this model and expose its features through declarative means. In addition, this specification also defines a programming interface to the model that may be implemented by user agents that provide support for scripting.

### <a id="use-cases"></a>1.1. Use cases

The Web Animations model is intended to provide the features necessary for expressing CSS Transitions [\[CSS-TRANSITIONS-1\]](#biblio-css-transitions-1), CSS Animations [\[CSS-ANIMATIONS-1\]](#biblio-css-animations-1), and SVG [\[SVG11\]](#biblio-svg11). As such, the use cases of Web Animations model is the union of use cases for those three specifications.

The use cases for the programming interface include the following:

Inspecting running animations  
Often Web applications must wait for certain animated effects to complete before updating some state. The programming interface in this specification allows such applications to wait for all currently running animation to complete, regardless of whether they are defined by CSS Transitions, CSS Animations, SVG animations, or created directly using the programming interface.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e8eccc9e"></a>
> ```javascript
> // Wait until all animations have finished before removing the element
> Promise.all(
>   elem.getAnimations().map(animation => animation.finished)
> ).then(() => elem.remove());
> ```
Alternatively, applications may wish to query the playback state of animations without waiting.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9a95d87d"></a>
> ```javascript
> const isAnimating = elem.getAnimations().some(
>   animation => animation.playState === 'running'
> );
> ```
Controlling running animations  
It is sometimes useful to perform playback control on animations so that they can respond to external inputs. For example, it may be necessary to pause all existing animations before displaying a modal dialog so that they do not distract the user’s attention.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e7bbf635"></a>
> ```javascript
> // Pause all existing animations in the document
> for (const animation of document.getAnimations()) {
>   animation.pause()
> }
> ```
Creating animations from script  
While it is possible to use ECMAScript to perform animation using `requestAnimationFrame` [\[HTML\]](#biblio-html), such animations behave differently to declarative animation in terms of how they are represented in the CSS cascade and the performance optimizations that are possible such as performing the animation on a separate thread. Using the Web Animations programming interface, it is possible to create animations from script that have the same behavior and performance characteristics as declarative animations.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-824e8c83"></a>
> ```javascript
> // Fade out quickly
> elem.animate({ transform: 'scale(0)', opacity: 0 }, 300);
> ```
Animation debugging  
In a complex application, it may be difficult to determine how an element arrived in its present state. The Web Animations programming interface may be used to inspect running animations to answer questions such as, "Why is the opacity of this element changing?"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6c6854a7"></a>
> ```javascript
> // Print the id of any opacity animations on elem
> for (const animation of elem.getAnimations()) {
>   if (
>     animation.effect instanceof KeyframeEffect &&
>     animation.effect
>       .getKeyframes()
>       .some(frame => frame.hasOwnProperty('opacity'))
>   ) {
>     console.log(animation.id);
>   }
> }
> ```
Likewise, in order to fine tune animations, it is often necessary to reduce their playback rate and replay them.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-06eda695"></a>
> ```javascript
> // Slow down and replay any transform animations
> const transformAnimations = elem.getAnimations().filter(
>   animation =>
>     animation.effect instanceof KeyframeEffect &&
>     animation.effect.getKeyframes().some(
>       frame => frame.hasOwnProperty('transform')
>     )
> );
> 
> for (const animation of transformAnimations) {
>   animation.currentTime = 0;
>   animation.updatePlaybackRate(0.5);
> }
> ```
Testing animations  
In order to test applications that make use of animations it is often impractical to wait for such animations to run to completion. Rather, it is desirable to seek the animations to specific times.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d0c0c637"></a>
>
> ```javascript
> // Seek to the half-way point of an animation and check that the opacity is 50%
> for (const animation of elem.getAnimations()) {
>   const { delay, activeDuration } = animation.effect.getComputedTiming();
>   animation.currentTime = delay + activeDuration / 2;
> }
> assert.strictEqual(getComputedStyle(elem).opacity, '0.5');
> 
> // Check that the loading screen is hidden after the animations finish
> for (const animation of elem.getAnimations()) {
>   animation.finish();
> }
> // Wait one frame so that event handlers have a chance to run
> requestAnimationFrame(() => {
>   assert.strictEqual(
>     getComputedStyle(document.querySelector('#loading')).display, 'none');
> });
> ```
### <a id="relationship-to-other-specifications"></a>1.2. Relationship to other specifications

CSS Transitions [\[CSS-TRANSITIONS-1\]](#biblio-css-transitions-1), CSS Animations [\[CSS-ANIMATIONS-1\]](#biblio-css-animations-1), and SVG [\[SVG11\]](#biblio-svg11) all provide mechanisms that generate animated content on a Web page. Although the three specifications provide many similar features, they are described in different terms. This specification proposes an abstract animation model that encompasses the common features of all three specifications. This model is backwards-compatible with the current behavior of these specifications such that they can be defined in terms of this model without any observable change.

The animation features in SVG 1.1 are defined in terms of SMIL Animation [\[SMIL-ANIMATION\]](#biblio-smil-animation). It is intended that by defining SVG’s animation features in terms of the Web Animations model, the dependency between SVG and SMIL Animation can be removed.

<a id="ref-for-animation-frames"></a>

As with [animation frame callbacks](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html.html#animation-frames) (commonly referred to as "requestAnimationFrame") [\[HTML\]](#biblio-html), the programming interface component of this specification allows animations to be created from script. The animations created using the interface defined in this specification, however, once created, are executed entirely by the user agent meaning they share the same performance characteristics as animations defined by markup. Using this interface it is possible to create animations from script in a simpler and more performant manner.

<a id="ref-for-animation-frames①"></a>

The time values used within the programming interface correspond with those used in [animation frame callbacks](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html.html#animation-frames) [\[HTML\]](#biblio-html) and their execution order is defined such that the two interfaces can be used simultaneously without conflict.

The programming interface component of this specification makes some additions to interfaces defined in HTML [\[HTML\]](#biblio-html).

### <a id="overview-of-this-specification"></a>1.3. Overview of this specification

This specification begins by defining an abstract model for animation. This is followed by a programming interface defined in terms of the abstract model. The programming interface is defined in terms of the abstract model and is only relevant to user agents that provide scripting support.

## <a id="spec-conventions"></a>2. Specification conventions

<a id="ref-for-concept-animation"></a>

<a id="ref-for-animation-effect"></a>

<a id="ref-for-playback-rate"></a>

<a id="ref-for-iteration-duration"></a>

<a id="ref-for-set-the-playback-rate"></a>

<a id="ref-for-set-the-start-time"></a>

This specification begins by describing abstract concepts such as [animations](#concept-animation) and [animation effects](#animation-effect) and properties that belong to them such as their [playback rate](#playback-rate) or [iteration duration](#iteration-duration). In addition to these properties, there are often specific procedures for updating these properties such as the procedure to [set the playback rate](#set-the-playback-rate) or the procedure to [set the start time](#set-the-start-time) of an animation.

<a id="ref-for-animation-start-time"></a>

<a id="ref-for-unresolved"></a>

Where this specification does not specifically link to a procedure, text that requires the user agent to update a property such as, "make <var>animation</var>’s [start time](#animation-start-time) [unresolved](#unresolved)", should be understood to refer to updating the property directly <em>without</em> invoking any related procedure.

Further documentation conventions that are not specific to this specification are described in [Document conventions](#w3c-conventions).

## <a id="web-animations-model-overview"></a>3. Web Animations model overview

<em>This section is non-normative</em>

At a glance, the Web Animations model consists of two largely independent pieces, a <em>timing model</em> and an <em>animation&#xA;model</em>. The role of these pieces is as follows:

Timing model  
Takes a moment in time and converts it to a proportional distance within a single iteration of an animation called the <em>iteration&#xA;progress</em>. The <em>iteration index</em> is also recorded since some animations vary each time they repeat.

Animation model  
Takes the <em>iteration progress</em> values and <em>iteration indices</em> produced by the timing model and converts them into a series of values to apply to the target properties.

Graphically, this flow can be represented as follows:

![Overview of the operation of the Web Animations model.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/timing-and-animation-models.svg)

Overview of the operation of the Web Animations model.  
The current time is input to the timing model which produces an iteration progress value and an iteration index.  
These parameters are used as input to the animation model which produces the values to apply.

For example, consider an animation that:

- starts after 3 seconds

- runs twice,

- takes 2 seconds every time, and

- changes the width of a rectangle from 50 pixels to 100 pixels.

The first three points apply to the timing model. At a time of 6 seconds, it will calculate that the animation should be half-way through its second iteration and produces the result 0.5. The animation model then uses that information to calculate a width.

This specification begins with the timing model and then proceeds to the animation model.

## <a id="timing-model"></a>4. Timing model

This section describes and defines the behavior of the Web Animations timing model.

### <a id="timing-model-overview"></a>4.1. Timing model overview

<em>This section is non-normative</em>

Two features characterize the Web Animations timing model: it is <em>stateless</em> and it is <em>hierarchical</em>.

#### <a id="stateless"></a>4.1.1. Stateless

The Web Animations timing model operates by taking an input time and producing an output iteration progress. Since the output is based solely on the input time and is independent of previous inputs, the model may be described as stateless. This gives the model the following properties:

Frame-rate independent  
Since the output is independent of previous inputs, the rate at which the model is updated will not affect its progress. Provided the input times are proportional to the progress of real-world time, animations will progress at an identical rate regardless of the capabilities of the device running them.

Direction-agnostic  
Since the sequence of inputs is insignificant, the model is directionless. This means that the model can be updated to an arbitrary moment without requiring any specialized handling.

Constant-time seeking  
Since each input is independent of the previous input, the processing required to perform a seek operation, even far into the future, is at least potentially constant.

There are a few exceptions to the stateless behavior of the timing model.

Firstly, a number of methods defined in the [programming interface](#programming-interface) to the model provide play control such as pausing an animation. These methods are defined in terms of the time at which they are called and are therefore stative. These methods are provided primarily for convenience and are not part of the core timing model but are layered on top.

<a id="ref-for-animation-associated-effect"></a>

Similarly, the [finishing behavior](#reaching-the-end) of animations means that dynamic changes to the end time of the media ([associated effect](#animation-associated-effect)) of an animation may produce a different result depending on when the change occurs. This behavior is somewhat unfortunate but has been deemed intuitive and consistent with HTML. As a result, the model can only truly be described as stateless <em>in the absence of dynamic changes to its timing properties</em>.

Finally, each time the model is updated, it can be considered to establish a temporary state. While this temporary state affects the values returned from the [programming interface](#programming-interface), it has no influence on the subsequent updates and hence does not conflict with the stateless qualities described above.

#### <a id="hierarchical"></a>4.1.2. Hierarchical

The other characteristic feature of the timing model is that time is inherited. Time begins at a timeline and cascades down a number of steps to each animation effect. At each step, time may be shifted backwards and forwards, scaled, reversed, paused, and repeated.

![A hierarchy of timing nodes](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/time-hierarchy.svg)

A hierarchy of timing nodes. Each node in the tree derives its time from its parent node.

In this level of the specification the hierarchy is shallow. A subsequent level of this specification will introduce the concept of group effects which allows for deeper timing hierarchies.

### <a id="time-value-section"></a>4.2. Time values

<a id="ref-for-time-value"></a>

Timing is based on a hierarchy of time relationships between timing nodes. Parent nodes provide timing information to their child nodes in the form of [time values](#time-value).

<a id="ref-for-time-value①"></a>

A <a id="time-value"></a>time value is a real number which nominally represents a number of milliseconds from some moment. The connection between [time values](#time-value) and wall-clock milliseconds may be obscured by any number of transformations applied to the value as it passes through the time hierarchy.

> <strong data-conversion-semantic="note">Note</strong>
>
> In the future there may be timelines that are based on scroll position or UI gestures in which case the connection between time values and milliseconds will be weakened even further.

<a id="ref-for-time-value②"></a>

A [time value](#time-value) may also be <a id="unresolved"></a>unresolved if, for example, a timing node is not in a state to produce a <a id="ref-for-time-value③"></a>time value.

### <a id="timelines"></a>4.3. Timelines

<a id="ref-for-time-value④"></a>

A <a id="timeline"></a>timeline provides a source of [time values](#time-value) for the purpose of synchronization.

<a id="ref-for-timeline"></a>

<a id="ref-for-time-value⑤"></a>

At any given moment, a [timeline](#timeline) has a single current [time value](#time-value) known simply as the timeline’s <a id="timeline-current-time"></a>current time.

<a id="ref-for-timeline①"></a>

<a id="ref-for-time-value⑥"></a>

<a id="ref-for-unresolved①"></a>

A [timeline](#timeline) may not always be able to return a meaningful [time value](#time-value), but only an [unresolved](#unresolved) time value. For example, it may be defined relative to a moment that has yet to occur, such as the firing of a document’s load event. A <a id="ref-for-timeline②"></a>timeline is considered to be <a id="inactive-timeline"></a>inactive when its <a id="ref-for-time-value⑦"></a>time value is <a id="ref-for-unresolved②"></a>unresolved.

<a id="ref-for-timeline③"></a>

<a id="ref-for-timeline-current-time"></a>

A [timeline](#timeline) is <a id="monotonically-increasing-timeline"></a>monotonically increasing if its reported [current time](#timeline-current-time) is always greater than or equal than its previously reported <a id="ref-for-timeline-current-time①"></a>current time.

<a id="ref-for-timeline④"></a>

<a id="ref-for-time-value⑧"></a>

Specific types of [timelines](#timeline) may define a procedure to <a id="timeline-time-to-origin-relative-time"></a>convert a timeline time to an origin-relative time for [time value](#time-value) <var>time</var>, so that the <a id="ref-for-time-value⑨"></a>time values produced by wallclock-based timelines can be compared.

<a id="ref-for-timeline⑤"></a>

A [timeline](#timeline) may be <a id="timeline-associated-with-a-document"></a>associated with a document.

<a id="ref-for-document"></a>

When asked to <a id="update-animations-and-send-events"></a>update animations and send events for a <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> <var>doc</var> at timestamp <var>now</var>, run these steps:

1.  <a id="ref-for-timeline-current-time②"></a>

    <a id="ref-for-timeline-associated-with-a-document"></a>

    Update the [current time](#timeline-current-time) of all timelines [associated with <var>doc</var>](#timeline-associated-with-a-document) passing <var>now</var> as the timestamp.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > <a id="ref-for-timeline-current-time③"></a>
    >
    > <a id="ref-for-timeline⑥"></a>
    >
    > Due to the hierarchical nature of the timing model, updating the [current time](#timeline-current-time) of a [timeline](#timeline) also involves:
    >
    > - <a id="ref-for-animation-current-time"></a>
    >
    >   <a id="ref-for-concept-animation①"></a>
    >
    >   <a id="ref-for-associated-with-a-timeline"></a>
    >
    >   Updating the [current time](#animation-current-time) of any [animations](#concept-animation) [associated with](#associated-with-a-timeline) the timeline.
    >
    > - <a id="ref-for-update-an-animations-finished-state"></a>
    >
    >   <a id="ref-for-animation-current-time①"></a>
    >
    >   Running the [update an animation’s finished state](#update-an-animations-finished-state) procedure for any animations whose [current time](#animation-current-time) has been updated.
    >
    > - <a id="ref-for-animation-events"></a>
    >
    >   Queueing [animation events](#animation-events) for any such animations.

2.  <a id="ref-for-remove-replaced-animations"></a>

    [Remove replaced animations](#remove-replaced-animations) for <var>doc</var>.

3.  <a id="ref-for-perform-a-microtask-checkpoint"></a>

    [Perform a microtask checkpoint](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is to ensure that any microtasks queued up as a result of resolving or rejecting Promise objects as part of updating timelines in the previous step, run their callbacks prior to dispatching animation events.

4.  <a id="ref-for-pending-animation-event-queue"></a>

    Let <var>events to dispatch</var> be a copy of <var>doc</var>’s [pending animation event queue](#pending-animation-event-queue).

5.  <a id="ref-for-pending-animation-event-queue①"></a>

    Clear <var>doc</var>’s [pending animation event queue](#pending-animation-event-queue).

6.  <a id="ref-for-animation-events①"></a>

    Perform a stable sort of the [animation events](#animation-events) in <var>events to dispatch</var> as follows:

    1.  <a id="ref-for-scheduled-event-time"></a>

        <a id="ref-for-unresolved③"></a>

        Sort the events by their [scheduled event time](#scheduled-event-time) such that events that were scheduled to occur earlier, sort before events scheduled to occur later and events whose scheduled event time is [unresolved](#unresolved) sort before events with a <a id="ref-for-unresolved④"></a>resolved scheduled event time.

    2.  <a id="ref-for-scheduled-event-time①"></a>

        <a id="ref-for-animation-composite-order"></a>

        Within events with equal [scheduled event times](#scheduled-event-time), sort by their [composite order](#animation-composite-order).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The purpose of sorting events is to ensure that, as best possible, even on devices with differing capabilities and hence different frame rates, events are dispatched in a consistent order.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The requirement for the sort to be a stable sort is because sometimes events may be queued with the same scheduled event time. For example, a CSS animation with a duration of zero, may dispatch both an `animationstart` and an `animationend` event and the order of these events should be preserved.

7.  <a id="ref-for-concept-event-dispatch"></a>

    [Dispatch](https://dom.spec.whatwg.org/#concept-event-dispatch) each of the events in <var>events to dispatch</var> at their corresponding target using the order established in the previous step.

<a id="ref-for-concept-animation②"></a>

<a id="ref-for-animation-effect①"></a>

<a id="ref-for-animation-frame"></a>

It is often convenient to describe each time this procedure is invoked as establishing a new <a id="animation-frame"></a>animation frame. Changes to the timing properties of [animations](#concept-animation) or [animation effects](#animation-effect), or the addition and removal of the objects may cause the output of the timing or animation model to change, but these operations in themselves do not create a new [animation frame](#animation-frame), rather they merely update the current <a id="ref-for-animation-frame①"></a>animation frame.

#### <a id="document-timelines"></a>4.3.1. Document timelines

<a id="ref-for-timeline⑦"></a>

<a id="ref-for-timeline-associated-with-a-document①"></a>

<a id="ref-for-timeline-current-time④"></a>

<a id="ref-for-update-animations-and-send-events"></a>

A <a id="document-timeline"></a>document timeline is a type of [timeline](#timeline) that is [associated with a document](#timeline-associated-with-a-document) and whose [current time](#timeline-current-time) is calculated as a fixed offset from the <var>now</var> timestamp provided each time the [update animations and send events](#update-animations-and-send-events) procedure is run. This fixed offset is referred to as the document timeline’s <a id="origin-time"></a>origin time.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9fa706cb"></a> There must be a better term than "origin time"— it’s too similar to "time origin". [\[Issue \#2079\]](https://github.com/w3c/csswg-drafts/issues/2079)

<a id="ref-for-concept-settings-object-time-origin"></a>

<a id="ref-for-document-timeline"></a>

<a id="ref-for-inactive-timeline"></a>

Prior to establishing the [time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin) for its associated document, a [document timeline](#document-timeline) is [inactive](#inactive-timeline).

<a id="ref-for-document-timeline①"></a>

<a id="ref-for-inactive-timeline①"></a>

<a id="ref-for-monotonically-increasing-timeline"></a>

After a [document timeline](#document-timeline) becomes [active](#inactive-timeline), it is [monotonically increasing](#monotonically-increasing-timeline).

<a id="ref-for-document-timeline②"></a>

<a id="ref-for-document①"></a>

<a id="ref-for-active-document"></a>

<a id="ref-for-inactive-timeline②"></a>

A [document timeline](#document-timeline) that is associated with a <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> which is not an [active document](https://html.spec.whatwg.org/multipage/browsers.html#active-document) is also considered to be [inactive](#inactive-timeline).

<a id="ref-for-timeline-time-to-origin-relative-time"></a>

<a id="ref-for-origin-time"></a>

<a id="ref-for-unresolved⑤"></a>

<a id="ref-for-time-value①⓪"></a>

To [convert a timeline time, <var>timeline time</var>, to an origin-relative time](#timeline-time-to-origin-relative-time) for a document timeline, <var>timeline</var>, return the sum of the <var>timeline time</var> and <var>timeline</var>’s [origin time](#origin-time). If <var>timeline</var> is inactive, return an [unresolved](#unresolved) [time value](#time-value).

#### <a id="the-documents-default-timeline"></a>4.3.2. The default document timeline

<a id="ref-for-document②"></a>

<a id="ref-for-document-timeline③"></a>

<a id="ref-for-document-default-document-timeline"></a>

<a id="ref-for-dom-document-open"></a>

Each <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> has a [document timeline](#document-timeline) called the <a id="document-default-document-timeline"></a>default document timeline. The [default document timeline](#document-default-document-timeline) is unique to each document and persists for the lifetime of the document including calls to [document.open()](https://html.spec.whatwg.org/multipage/webappapis.html#dom-document-open) [\[HTML\]](#biblio-html).

<a id="ref-for-document-default-document-timeline①"></a>

<a id="ref-for-origin-time①"></a>

The [default document timeline](#document-default-document-timeline) has an [origin time](#origin-time) of zero.

<em>This section is non-normative</em>

<a id="ref-for-document-timeline④"></a>

<a id="ref-for-time-value①①"></a>

Since no scaling is applied to the <var>now</var> timestamp values provided to [document timelines](#document-timeline), the [time values](#time-value) it produces will be proportional to wall-clock milliseconds.

<a id="ref-for-time-value①②"></a>

<a id="ref-for-document-default-document-timeline②"></a>

<a id="ref-for-concept-settings-object-time-origin①"></a>

<a id="ref-for-update-animations-and-send-events①"></a>

Furthermore, since the [time values](#time-value) of the [default document timeline](#document-default-document-timeline) have a zero offset from the [time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin), `document.timeline.currentTime` will roughly correspond to [`Performance.now()`](https://www.w3.org/TR/hr-time/#dom-performance-now) [\[HR-TIME\]](#biblio-hr-time) with the exception that `document.timeline.currentTime` does not change in between calls to the [update animations and send events](#update-animations-and-send-events) procedure.

### <a id="animations"></a>4.4. Animations

<em>This section is non-normative</em>

<a id="ref-for-timeline⑧"></a>

<a id="ref-for-animation-effect②"></a>

The children of a [timeline](#timeline) are called <em>animations</em>. An animation takes an [animation effect](#animation-effect) which is a static description of some timed behavior and binds it to a <a id="ref-for-timeline⑨"></a>timeline so that it runs. An animation also allows run-time control of the connection between the <a id="ref-for-animation-effect③"></a>animation effect and its <a id="ref-for-timeline①⓪"></a>timeline by providing pausing, seeking, and speed control. The relationship between an animation and an <a id="ref-for-animation-effect④"></a>animation effect is analogous to that of a DVD player and a DVD.

<a id="ref-for-animation-effect⑤"></a>

<a id="ref-for-timeline①①"></a>

<a id="ref-for-concept-animation③"></a>

<a id="ref-for-animation-associated-effect①"></a>

An <a id="concept-animation"></a>animation connects a single [animation effect](#animation-effect), called its <a id="animation-associated-effect"></a>associated effect, to a [timeline](#timeline) and provides playback control. Both of these associations are optional and configurable such that an [animation](#concept-animation) may have no [associated effect](#animation-associated-effect) or <a id="ref-for-timeline①②"></a>timeline at a given moment.

<a id="ref-for-concept-animation④"></a>

<a id="ref-for-document③"></a>

<a id="ref-for-timeline①③"></a>

<a id="ref-for-timeline-associated-with-a-document②"></a>

<a id="ref-for-animation-document-for-timing"></a>

An [animation’s](#concept-animation) <a id="animation-document-for-timing"></a>document for timing is the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> with which its [timeline](#timeline) is [associated](#timeline-associated-with-a-document). If an animation is not associated with a timeline, or its timeline is not associated with a document, then it has no [document for timing](#animation-document-for-timing).

<a id="ref-for-concept-animation⑤"></a>

<a id="ref-for-time-value①③"></a>

<a id="ref-for-timeline①④"></a>

<a id="ref-for-animation-associated-effect②"></a>

<a id="ref-for-unresolved⑥"></a>

An [animation](#concept-animation)’s <a id="animation-start-time"></a>start time is the [time value](#time-value) of its [timeline](#timeline) when its [associated effect](#animation-associated-effect) is scheduled to begin playback. An animation’s start time is initially [unresolved](#unresolved).

<a id="ref-for-concept-animation⑥"></a>

<a id="ref-for-time-value①④"></a>

<a id="ref-for-animation-current-time②"></a>

<a id="ref-for-animation-hold-time"></a>

<a id="ref-for-unresolved⑦"></a>

An [animation](#concept-animation) also maintains a <a id="animation-hold-time"></a>hold time [time value](#time-value) which is used to fix the animation’s output <a id="ref-for-time-value①⑤"></a>time value, called its [current time](#animation-current-time), in circumstances such as pausing. The [hold time](#animation-hold-time) is initially [unresolved](#unresolved).

<a id="ref-for-concept-animation⑦"></a>

<a id="ref-for-animation-class"></a>

In order to establish the relative ordering of conflicting [animations](#concept-animation), animations are appended to a <a id="global-animation-list"></a>global animation list in the order in which they are created. Certain [classes of animations](#animation-class), however, may provide alternative means of ordering animations (see [§ 5.4.1 Animation classes](#animation-classes)).

#### <a id="setting-the-timeline"></a>4.4.1. Setting the timeline of an animation

The procedure to <a id="animation-set-the-timeline-of-an-animation"></a>set the timeline of an animation, <var>animation</var>, to <var>new timeline</var> which may be null, is as follows:

1.  <a id="ref-for-timeline①⑤"></a>

    Let <var>old timeline</var> be the current [timeline](#timeline) of <var>animation</var>, if any.

2.  If <var>new timeline</var> is the same object as <var>old timeline</var>, abort this procedure.

3.  <a id="ref-for-timeline①⑥"></a>

    Let the [timeline](#timeline) of <var>animation</var> be <var>new timeline</var>.

4.  <a id="ref-for-animation-start-time①"></a>

    <a id="ref-for-unresolved⑧"></a>

    <a id="ref-for-animation-hold-time①"></a>

    If the [start time](#animation-start-time) of <var>animation</var> is [resolved](#unresolved), make <var>animation</var>’s [hold time](#animation-hold-time) <a id="ref-for-unresolved⑨"></a>unresolved.

    <a id="ref-for-play-state-finished"></a>

    <a id="ref-for-animation-current-time③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This step ensures that the [finished play state](#play-state-finished) of <var>animation</var> is not "sticky" but is re-evaluated based on its updated [current time](#animation-current-time).

5.  <a id="ref-for-update-an-animations-finished-state①"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

#### <a id="setting-the-associated-effect"></a>4.4.2. Setting the associated effect of an animation<a id="setting-the-target-effect"></a>

The procedure to <a id="animation-set-the-associated-effect-of-an-animation"></a>set the associated effect of an animation, <var>animation</var>, to <var>new effect</var> which may be null, is as follows:

1.  <a id="ref-for-animation-associated-effect③"></a>

    Let <var>old effect</var> be the current [associated effect](#animation-associated-effect) of <var>animation</var>, if any.

2.  If <var>new effect</var> is the same object as <var>old effect</var>, abort this procedure.

3.  <a id="ref-for-pending-pause-task"></a>

    <a id="ref-for-ready"></a>

    If <var>animation</var> has a [pending pause task](#pending-pause-task), reschedule that task to run as soon as <var>animation</var> is [ready](#ready).

4.  <a id="ref-for-pending-play-task"></a>

    <a id="ref-for-ready①"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task), reschedule that task to run as soon as <var>animation</var> is [ready](#ready) to play <var>new&#xA;effect</var>.

5.  <a id="ref-for-animation-associated-effect④"></a>

    <a id="ref-for-concept-animation⑧"></a>

    <a id="ref-for-animation-set-the-associated-effect-of-an-animation"></a>

    If <var>new effect</var> is not `null` and if <var>new effect</var> is the [associated effect](#animation-associated-effect) of another [animation](#concept-animation), <var>previous animation</var>, run the procedure to [set the associated effect of an animation](#animation-set-the-associated-effect-of-an-animation) (this procedure) on <var>previous&#xA;animation</var> passing null as <var>new effect</var>.

6.  <a id="ref-for-animation-associated-effect⑤"></a>

    Let the [associated effect](#animation-associated-effect) of <var>animation</var> be <var>new&#xA;effect</var>.

7.  <a id="ref-for-update-an-animations-finished-state②"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

#### <a id="the-current-time-of-an-animation"></a>4.4.3. The current time of an animation

<a id="ref-for-concept-animation⑨"></a>

<a id="ref-for-time-value①⑥"></a>

<a id="ref-for-animation-associated-effect⑥"></a>

[Animations](#concept-animation) provide a [time value](#time-value) to their [associated effect](#animation-associated-effect) called the animation’s <a id="animation-current-time"></a>current time.

<a id="ref-for-animation-current-time④"></a>

The [current time](#animation-current-time) is calculated from the first matching condition from below:

<a id="ref-for-unresolved①⓪"></a>

<a id="ref-for-animation-hold-time②"></a>

If the animation’s [hold time](#animation-hold-time) is [resolved](#unresolved),

<a id="ref-for-animation-current-time⑤"></a>

<a id="ref-for-animation-hold-time③"></a>

The [current time](#animation-current-time) is the animation’s [hold time](#animation-hold-time).

If <em>any</em> of the following are true:

1.  <a id="ref-for-timeline①⑦"></a>

    the animation has no associated [timeline](#timeline), or

2.  <a id="ref-for-timeline①⑧"></a>

    <a id="ref-for-inactive-timeline③"></a>

    the associated [timeline](#timeline) is [inactive](#inactive-timeline), or

3.  <a id="ref-for-animation-start-time②"></a>

    <a id="ref-for-unresolved①①"></a>

    the animation’s [start time](#animation-start-time) is [unresolved](#unresolved).

<a id="ref-for-animation-current-time⑥"></a>

<a id="ref-for-unresolved①②"></a>

The [current time](#animation-current-time) is an [unresolved](#unresolved) time value.

Otherwise,

<a id="ref-for-playback-rate①"></a>

<a id="ref-for-animation-start-time③"></a>

<a id="ref-for-animation-current-time⑦"></a>

> <code><a href="#animation-current-time">current time</a> =&#xA;    (<var>timeline time</var> - <a href="#animation-start-time">start time</a>)&#xA;    × <a href="#playback-rate">playback rate</a></code>

<a id="ref-for-time-value①⑦"></a>

<a id="ref-for-timeline①⑨"></a>

<a id="ref-for-playback-rate②"></a>

Where <var>timeline time</var> is the current [time value](#time-value) of the associated [timeline](#timeline). The [playback rate](#playback-rate) value is defined in [§ 4.4.15 Speed control](#speed-control).

#### <a id="setting-the-current-time-of-an-animation"></a>4.4.4. Setting the current time of an animation

<a id="ref-for-animation-current-time⑧"></a>

The [current time](#animation-current-time) of an animation can be set to a new value to <em>seek</em> the animation. The procedure for setting the current time is defined in two parts.

The procedure to <a id="animation-silently-set-the-current-time"></a>silently set the current time of an animation, <var>animation</var>, to <var>seek time</var> is as follows:

1.  <a id="ref-for-unresolved①③"></a>

    If <var>seek time</var> is an [unresolved](#unresolved) time value, then perform the following steps.

    1.  <a id="ref-for-animation-current-time⑨"></a>

        <a id="ref-for-unresolved①④"></a>

        <a id="ref-for-dfn-throw"></a>

        If the [current time](#animation-current-time) is [resolved](#unresolved), then [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError.

    2.  Abort these steps.

2.  <a id="ref-for-animation-hold-time④"></a>

    <a id="ref-for-animation-start-time④"></a>

    Update either <var>animation</var>’s [hold time](#animation-hold-time) or [start time](#animation-start-time) as follows:

    If <em>any</em> of the following conditions are true:

    - <a id="ref-for-animation-hold-time⑤"></a>

      <a id="ref-for-unresolved①⑤"></a>

      <var>animation</var>’s [hold time](#animation-hold-time) is [resolved](#unresolved), or

    - <a id="ref-for-animation-start-time⑤"></a>

      <a id="ref-for-unresolved①⑥"></a>

      <var>animation</var>’s [start time](#animation-start-time) is [unresolved](#unresolved), or

    - <a id="ref-for-timeline②⓪"></a>

      <a id="ref-for-inactive-timeline④"></a>

      <var>animation</var> has no associated [timeline](#timeline) or the associated <a id="ref-for-timeline②①"></a>timeline is [inactive](#inactive-timeline), or

    - <a id="ref-for-playback-rate③"></a>

      <var>animation</var>’s [playback rate](#playback-rate) is 0,

    <a id="ref-for-animation-hold-time⑥"></a>

    Set <var>animation</var>’s [hold time](#animation-hold-time) to <var>seek time</var>.

    Otherwise,
    <a id="ref-for-animation-start-time⑥"></a>

    <a id="ref-for-playback-rate④"></a>

    <a id="ref-for-time-value①⑧"></a>

    <a id="ref-for-timeline②②"></a>

    Set <var>animation</var>’s [start time](#animation-start-time) to the result of evaluating <code><var>timeline time</var> - (<var>seek time</var> / <a href="#playback-rate">playback rate</a>)</code> where <var>timeline time</var> is the current [time value](#time-value) of [timeline](#timeline) associated with <var>animation</var>.

3.  <a id="ref-for-timeline②③"></a>

    <a id="ref-for-inactive-timeline⑤"></a>

    <a id="ref-for-animation-start-time⑦"></a>

    <a id="ref-for-unresolved①⑦"></a>

    If <var>animation</var> has no associated [timeline](#timeline) or the associated <a id="ref-for-timeline②④"></a>timeline is [inactive](#inactive-timeline), make <var>animation</var>’s [start time](#animation-start-time) [unresolved](#unresolved).

    <a id="ref-for-animation-start-time⑧"></a>

    <a id="ref-for-animation-current-time①⓪"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > This preserves the invariant that when we don’t have an active timeline it is only possible to set <em>either</em> the [start time](#animation-start-time) <em>or</em> the animation’s [current time](#animation-current-time).

4.  <a id="ref-for-previous-current-time"></a>

    <a id="ref-for-unresolved①⑧"></a>

    Make <var>animation</var>’s [previous current time](#previous-current-time) [unresolved](#unresolved).

The procedure to <a id="animation-set-the-current-time"></a>set the current time of an animation, <var>animation</var>, to <var>seek time</var> is as follows:

1.  <a id="ref-for-animation-silently-set-the-current-time"></a>

    Run the steps to [silently set the current time](#animation-silently-set-the-current-time) of <var>animation</var> to <var>seek time</var>.

2.  <a id="ref-for-pending-pause-task①"></a>

    If <var>animation</var> has a [pending pause task](#pending-pause-task), synchronously complete the pause operation by performing the following steps:

    1.  <a id="ref-for-animation-hold-time⑦"></a>

        Set <var>animation</var>’s [hold time](#animation-hold-time) to <var>seek time</var>.

    2.  <a id="ref-for-apply-any-pending-playback-rate"></a>

        [Apply any pending playback rate](#apply-any-pending-playback-rate) to <var>animation</var>.

    3.  <a id="ref-for-animation-start-time⑨"></a>

        <a id="ref-for-unresolved①⑨"></a>

        Make <var>animation</var>’s [start time](#animation-start-time) [unresolved](#unresolved).

    4.  <a id="ref-for-pending-pause-task②"></a>

        Cancel the [pending pause task](#pending-pause-task).

    5.  <a id="ref-for-resolve"></a>

        <a id="ref-for-current-ready-promise"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>animation</var>’s [current ready promise](#current-ready-promise) with <var>animation</var>.

3.  <a id="ref-for-update-an-animations-finished-state③"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to true, and the <var>synchronously notify</var> flag set to false.

#### <a id="setting-the-start-time-of-an-animation"></a>4.4.5. Setting the start time of an animation

<a id="ref-for-concept-animation①⓪"></a>

The procedure to <a id="set-the-start-time"></a>set the start time of [animation](#concept-animation), <var>animation</var>, to <var>new start time</var>, is as follows:

1.  <a id="ref-for-time-value①⑨"></a>

    <a id="ref-for-timeline②⑤"></a>

    <a id="ref-for-inactive-timeline⑥"></a>

    <a id="ref-for-unresolved②⓪"></a>

    Let <var>timeline time</var> be the current [time value](#time-value) of the [timeline](#timeline) that <var>animation</var> is associated with. If there is no <a id="ref-for-timeline②⑥"></a>timeline associated with <var>animation</var> or the associated timeline is [inactive](#inactive-timeline), let the <var>timeline time</var> be [unresolved](#unresolved).

2.  <a id="ref-for-unresolved②①"></a>

    <a id="ref-for-animation-hold-time⑧"></a>

    If <var>timeline time</var> is [unresolved](#unresolved) and <var>new start&#xA;time</var> is <a id="ref-for-unresolved②②"></a>resolved, make <var>animation</var>’s [hold time](#animation-hold-time) <a id="ref-for-unresolved②③"></a>unresolved.

    <a id="ref-for-animation-start-time①⓪"></a>

    <a id="ref-for-animation-current-time①①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > This preserves the invariant that when we don’t have an active timeline it is only possible to set <em>either</em> the [start time](#animation-start-time) <em>or</em> the animation’s [current time](#animation-current-time).

3.  <a id="ref-for-animation-current-time①②"></a>

    Let <var>previous current time</var> be <var>animation</var>’s [current time](#animation-current-time).

    <a id="ref-for-animation-current-time①③"></a>

    <a id="ref-for-unresolved②④"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is the [current time](#animation-current-time) after applying the changes from the previous step which may cause the current time to become [unresolved](#unresolved).

4.  <a id="ref-for-apply-any-pending-playback-rate①"></a>

    [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

5.  <a id="ref-for-animation-start-time①①"></a>

    Set <var>animation</var>’s [start time](#animation-start-time) to <var>new start time</var>.

6.  <a id="ref-for-animation-hold-time⑨"></a>

    Update <var>animation</var>’s [hold time](#animation-hold-time) based on the first matching condition from the following,

    <a id="ref-for-unresolved②⑤"></a>

    If <var>new start time</var> is [resolved](#unresolved),
    <a id="ref-for-playback-rate⑤"></a>

    <a id="ref-for-animation-hold-time①⓪"></a>

    <a id="ref-for-unresolved②⑥"></a>

    If <var>animation</var>’s [playback rate](#playback-rate) is not zero, make <var>animation</var>’s [hold time](#animation-hold-time) [unresolved](#unresolved).

    <a id="ref-for-unresolved②⑦"></a>

    Otherwise (<var>new start time</var> is [unresolved](#unresolved)),
    <a id="ref-for-animation-hold-time①①"></a>

    <a id="ref-for-unresolved②⑧"></a>

    Set <var>animation</var>’s [hold time](#animation-hold-time) to <var>previous current&#xA;time</var> even if <var>previous current time</var> is [unresolved](#unresolved).

7.  <a id="ref-for-pending-play-task①"></a>

    <a id="ref-for-pending-pause-task③"></a>

    <a id="ref-for-resolve①"></a>

    <a id="ref-for-current-ready-promise①"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task), cancel that task and [resolve](https://webidl.spec.whatwg.org/#resolve) <var>animation</var>’s [current ready promise](#current-ready-promise) with <var>animation</var>.

8.  <a id="ref-for-update-an-animations-finished-state④"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to true, and the <var>synchronously notify</var> flag set to false.

#### <a id="waiting-for-the-associated-effect"></a>4.4.6. Waiting for the associated effect<a id="waiting-for-the-target-effect"></a>

<em>This section is non-normative</em>

<a id="ref-for-concept-animation①①"></a>

Some operations performed by an [animation](#concept-animation) may not occur instantaneously. For example, some user agents may delegate the playback of an animation to a separate process or to specialized graphics hardware each of which may incur some setup overhead.

If such an animation is timed from the moment when the animation was triggered there may be a significant jump between the first and second frames of the animation corresponding to the setup time involved.

<a id="ref-for-unresolved②⑨"></a>

<a id="ref-for-animation-start-time①②"></a>

<a id="ref-for-concept-animation①②"></a>

<a id="ref-for-ready②"></a>

<a id="ref-for-time-value②⓪"></a>

To avoid this problem, Web Animations typically begins timing animations from the moment when the first frame of the animation is complete. This is represented by an [unresolved](#unresolved) [start time](#animation-start-time) on the [animation](#concept-animation) which becomes resolved when the animation is [ready](#ready). Content may opt out of this behavior by setting the <a id="ref-for-animation-start-time①③"></a>start time to a <a id="ref-for-unresolved③⓪"></a>resolved [time value](#time-value).

An animation is <a id="ready"></a>ready at the first moment where <em>both</em> of the following conditions are true:

- <a id="ref-for-animation-associated-effect⑦"></a>

  <a id="ref-for-keyframe-effect"></a>

  the user agent has completed any setup required to begin the playback of the animation’s [associated effect](#animation-associated-effect) including rendering the first frame of any [keyframe effect](#keyframe-effect).

- <a id="ref-for-timeline②⑦"></a>

  <a id="ref-for-inactive-timeline⑦"></a>

  the animation is associated with a [timeline](#timeline) that is not [inactive](#inactive-timeline).

#### <a id="the-current-ready-promise"></a>4.4.7. The current ready promise

<a id="ref-for-concept-animation①③"></a>

<a id="ref-for-current-ready-promise②"></a>

<a id="ref-for-sec-promise-objects"></a>

<a id="ref-for-a-promise-resolved-with"></a>

<a id="ref-for-concept-relevant-realm"></a>

Each [animation](#concept-animation) has a <a id="current-ready-promise"></a>current ready promise. The [current ready promise](#current-ready-promise) is initially a resolved [Promise](http://www.ecma-international.org/ecma-262/6.0/#sec-promise-objects) created using the procedure to [create a new resolved Promise](https://webidl.spec.whatwg.org/#a-promise-resolved-with) with the animation itself as its value and created in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of the animation.

<a id="ref-for-sec-promise-objects①"></a>

<a id="ref-for-pending-play-task②"></a>

<a id="ref-for-pending-pause-task④"></a>

The object is replaced with a new [Promise object](http://www.ecma-international.org/ecma-262/6.0/#sec-promise-objects) every time the animation queues a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task) when it previously did not have a pending task, or when the animation is canceled (see [§ 4.4.14 Canceling an animation](#canceling-an-animation-section)).

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-sec-promise-objects②"></a>
>
> Note that since the same object is used for both pending play and pending pause requests, authors are advised to check the state of the animation when the [Promise object](http://www.ecma-international.org/ecma-262/6.0/#sec-promise-objects) is resolved.
>
> <a id="ref-for-play-state-running"></a>
>
> <a id="ref-for-current-ready-promise③"></a>
>
> <a id="ref-for-pending-play-task③"></a>
>
> For example, in the following code fragment, the state of the animation will be [running](#play-state-running) when the [current ready promise](#current-ready-promise) is resolved. This is because the `play` operation occurs while a [pending play task](#pending-play-task) is still queued and hence the <a id="ref-for-current-ready-promise④"></a>current ready promise is re-used.
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-eb69e98f"></a>
> >
> > ```javascript
> > animation.pause();
> > animation.ready.then(function() {
> >   // Displays 'running'
> >   alert(animation.playState);
> > });
> > animation.play();
> > ```
#### <a id="playing-an-animation-section"></a>4.4.8. Playing an animation

The procedure to <a id="play-an-animation"></a>play an animation, <var>animation</var>, given a flag <var>auto-rewind</var>, is as follows:

1.  <a id="ref-for-pending-pause-task⑤"></a>

    Let <var>aborted pause</var> be a boolean flag that is true if <var>animation</var> has a [pending pause task](#pending-pause-task), and false otherwise.

2.  Let <var>has pending ready promise</var> be a boolean flag that is initially false.

3.  <a id="ref-for-time-value②①"></a>

    <a id="ref-for-unresolved③①"></a>

    Let <var>seek time</var> be a [time value](#time-value) that is initially [unresolved](#unresolved).

4.  If the <var>auto-rewind</var> flag is true, perform the steps corresponding to the <em>first</em> matching condition from the following, if any:

    <a id="ref-for-effective-playback-rate"></a>

    <a id="ref-for-animation-current-time①④"></a>

    If <var>animation</var>’s [effective playback rate](#effective-playback-rate) ≥ 0, and <var>animation</var>’s [current time](#animation-current-time) is <em>either</em>:

    - <a id="ref-for-unresolved③②"></a>

      [unresolved](#unresolved), or

    - less than zero, or

    - <a id="ref-for-associated-effect-end"></a>

      greater than or equal to [associated effect end](#associated-effect-end),

    Set <var>seek time</var> to zero.

    <a id="ref-for-effective-playback-rate①"></a>

    <a id="ref-for-animation-current-time①⑤"></a>

    If <var>animation</var>’s [effective playback rate](#effective-playback-rate) \< 0, and <var>animation</var>’s [current time](#animation-current-time) is <em>either</em>:

    - <a id="ref-for-unresolved③③"></a>

      [unresolved](#unresolved), or

    - less than or equal to zero, or

    - <a id="ref-for-associated-effect-end①"></a>

      greater than [associated effect end](#associated-effect-end),

    <a id="ref-for-associated-effect-end②"></a>

    If [associated effect end](#associated-effect-end) is positive infinity,
    <a id="ref-for-dfn-throw①"></a>

    <a id="ref-for-invalidstateerror"></a>

    <a id="ref-for-idl-DOMException"></a>

    [throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

    Otherwise,
    <a id="ref-for-associated-effect-end③"></a>

    Set <var>seek time</var> to <var>animation</var>’s [associated effect end](#associated-effect-end).

5.  If the following three conditions are <em>all</em> satisfied:

    - <a id="ref-for-unresolved③④"></a>

      <var>seek time</var> is [unresolved](#unresolved), and

    - <a id="ref-for-animation-start-time①④"></a>

      <a id="ref-for-unresolved③⑤"></a>

      <var>animation</var>’s [start time](#animation-start-time) is [unresolved](#unresolved), and

    - <a id="ref-for-animation-current-time①⑥"></a>

      <a id="ref-for-unresolved③⑥"></a>

      <var>animation</var>’s [current time](#animation-current-time) is [unresolved](#unresolved),

    set <var>seek time</var> to zero.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The above step ensures that this procedure will play an idle animation regardless of the setting of the <var>auto-rewind</var> flag.

6.  <a id="ref-for-timeline②⑧"></a>

    <a id="ref-for-monotonically-increasing-timeline①"></a>

    Let <var>has finite timeline</var> be true if <var>animation</var> has an associated [timeline](#timeline) that is not [monotonically increasing](#monotonically-increasing-timeline).

7.  <a id="ref-for-unresolved③⑦"></a>

    If <var>seek time</var> is [resolved](#unresolved),

    If <var>has finite timeline</var> is true,  
    1.  <a id="ref-for-animation-start-time①⑤"></a>

        Set <var>animation</var>’s [start time](#animation-start-time) to <var>seek time</var>.

    2.  <a id="ref-for-animation-hold-time①②"></a>

        <a id="ref-for-unresolved③⑧"></a>

        Let <var>animation</var>’s [hold time](#animation-hold-time) be [unresolved](#unresolved).

    3.  <a id="ref-for-apply-any-pending-playback-rate②"></a>

        [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

    Otherwise,  
    <a id="ref-for-animation-hold-time①③"></a>

    Set <var>animation</var>’s [hold time](#animation-hold-time) to <var>seek time</var>.

8.  <a id="ref-for-animation-hold-time①④"></a>

    <a id="ref-for-unresolved③⑨"></a>

    <a id="ref-for-animation-start-time①⑥"></a>

    If <var>animation</var>’s [hold time](#animation-hold-time) is [resolved](#unresolved), let its [start time](#animation-start-time) be <a id="ref-for-unresolved④⓪"></a>unresolved.

9.  <a id="ref-for-pending-play-task④"></a>

    <a id="ref-for-pending-pause-task⑥"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task),

    1.  Cancel that task.

    2.  Set <var>has pending ready promise</var> to true.

10. If the following four conditions are <em>all</em> satisfied:

    - <a id="ref-for-animation-hold-time①⑤"></a>

      <a id="ref-for-unresolved④①"></a>

      <var>animation</var>’s [hold time](#animation-hold-time) is [unresolved](#unresolved), and

    - <a id="ref-for-unresolved④②"></a>

      <var>seek time</var> is [unresolved](#unresolved), and

    - <var>aborted pause</var> is false, and

    - <a id="ref-for-pending-playback-rate"></a>

      <var>animation</var> does <em>not</em> have a [pending playback rate](#pending-playback-rate),

    abort this procedure.

11. <a id="ref-for-current-ready-promise⑤"></a>

    <a id="ref-for-a-new-promise"></a>

    <a id="ref-for-concept-relevant-realm①"></a>

    If <var>has pending ready promise</var> is false, let <var>animation</var>’s [current ready promise](#current-ready-promise) be [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>animation</var>.

12. <a id="ref-for-ready③"></a>

    Schedule a task to run as soon as <var>animation</var> is [ready](#ready). The task shall perform the following steps:

    1.  <a id="ref-for-animation-start-time①⑦"></a>

        <a id="ref-for-animation-hold-time①⑥"></a>

        <a id="ref-for-unresolved④③"></a>

        Assert that at least one of <var>animation</var>’s [start time](#animation-start-time) or [hold time](#animation-hold-time) is [resolved](#unresolved).

    2.  <a id="ref-for-time-value②②"></a>

        <a id="ref-for-timeline②⑨"></a>

        <a id="ref-for-ready④"></a>

        Let <var>ready time</var> be the [time value](#time-value) of the [timeline](#timeline) associated with <var>animation</var> at the moment when <var>animation</var> became [ready](#ready).

    3.  Perform the steps corresponding to the first matching condition below, if any:

        <a id="ref-for-unresolved④④"></a>

        <a id="ref-for-animation-hold-time①⑦"></a>

        If <var>animation</var>’s [hold time](#animation-hold-time) is [resolved](#unresolved),
        1.  <a id="ref-for-apply-any-pending-playback-rate③"></a>

            [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

        2.  <a id="ref-for-animation-hold-time①⑧"></a>

            <a id="ref-for-playback-rate⑥"></a>

            <a id="ref-for-playback-rate⑦"></a>

            Let <var>new start time</var> be the result of evaluating <code><var>ready time</var> - <a href="#animation-hold-time">hold time</a> / <a href="#playback-rate">playback rate</a></code> for <var>animation</var>. If the [playback rate](#playback-rate) is zero, let <var>new start time</var> be simply <var>ready time</var>.

        3.  <a id="ref-for-animation-start-time①⑧"></a>

            Set the [start time](#animation-start-time) of <var>animation</var> to <var>new start time</var>.

        4.  <a id="ref-for-playback-rate⑧"></a>

            <a id="ref-for-animation-hold-time①⑨"></a>

            <a id="ref-for-unresolved④⑤"></a>

            If <var>animation</var>’s [playback rate](#playback-rate) is not 0, make <var>animation</var>’s [hold time](#animation-hold-time) [unresolved](#unresolved).

        <a id="ref-for-pending-playback-rate①"></a>

        <a id="ref-for-animation-start-time①⑨"></a>

        If <var>animation</var>’s [start time](#animation-start-time) is resolved and <var>animation</var> has a [pending playback rate](#pending-playback-rate),
        1.  <a id="ref-for-animation-start-time②⓪"></a>

            <a id="ref-for-playback-rate⑨"></a>

            Let <var>current time to match</var> be the result of evaluating <code>(<var>ready time</var> - <a href="#animation-start-time">start time</a>) × <a href="#playback-rate">playback rate</a></code> for <var>animation</var>.

        2.  <a id="ref-for-apply-any-pending-playback-rate④"></a>

            [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

        3.  <a id="ref-for-playback-rate①⓪"></a>

            <a id="ref-for-animation-hold-time②⓪"></a>

            If <var>animation</var>’s [playback rate](#playback-rate) is zero, let <var>animation</var>’s [hold time](#animation-hold-time) be <var>current time to match</var>.

        4.  <a id="ref-for-playback-rate①①"></a>

            <a id="ref-for-playback-rate①②"></a>

            Let <var>new start time</var> be the result of evaluating <code><var>ready time</var> - <var>current time to match</var> / <a href="#playback-rate">playback rate</a></code> for <var>animation</var>. If the [playback rate](#playback-rate) is zero, let <var>new start time</var> be simply <var>ready time</var>.

        5.  <a id="ref-for-animation-start-time②①"></a>

            Set the [start time](#animation-start-time) of <var>animation</var> to <var>new start time</var>.

    4.  <a id="ref-for-resolve②"></a>

        <a id="ref-for-current-ready-promise⑥"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>animation</var>’s [current ready promise](#current-ready-promise) with <var>animation</var>.

    5.  <a id="ref-for-update-an-animations-finished-state⑤"></a>

        Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

        <a id="ref-for-animation-associated-effect⑧"></a>

        <a id="ref-for-current-ready-promise⑦"></a>

        <a id="ref-for-current-finished-promise"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note that the order of the above two steps is important since it means that an animation with zero-length [associated effect](#animation-associated-effect) will resolve its [current ready promise](#current-ready-promise) before its [current finished promise](#current-finished-promise).

    <a id="ref-for-pending-play-task⑤"></a>

    So long as the above task is scheduled but has yet to run, <var>animation</var> is described as having a <a id="pending-play-task"></a>pending play task. While the task is running, however, <var>animation</var> does <em>not</em> have a [pending play task](#pending-play-task).

    <a id="ref-for-ready⑤"></a>

    <a id="ref-for-perform-a-microtask-checkpoint①"></a>

    If a user agent determines that <var>animation</var> is immediately [ready](#ready), it may schedule the above task as a microtask such that it runs at the next [microtask checkpoint](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint), but it must <em>not</em> perform the task synchronously.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > <a id="ref-for-pending-play-task⑥"></a>
    >
    > The above requirement to run the [pending play task](#pending-play-task) asynchronously ensures that code such as the following behaves consistently between implementations:
    >
    > > <strong data-conversion-semantic="example">Example</strong>
    > >
    > > <a id="example-b6f05729"></a>
    > > ```javascript
    > > animation.play();
    > > animation.ready.then(
    > >   () => { console.log('Playback commenced'); },
    > >   () => { console.log('Playback was canceled'); }
    > > );
    > > // Suppose some condition requires playback to be canceled...
    > > animation.cancel();
    > > // "Playback was canceled" will be printed to the console.
    > > ```
    >
    > <a id="ref-for-pending-play-task⑦"></a>
    >
    > <a id="ref-for-current-ready-promise⑧"></a>
    >
    > In the above code, were the [pending play task](#pending-play-task) run synchronously, the [current ready promise](#current-ready-promise) would not be rejected.

13. <a id="ref-for-update-an-animations-finished-state⑥"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

#### <a id="pausing-an-animation-section"></a>4.4.9. Pausing an animation

<a id="ref-for-concept-animation①④"></a>

<a id="ref-for-unresolved④⑥"></a>

<a id="ref-for-animation-start-time②②"></a>

<a id="ref-for-animation-current-time①⑦"></a>

Whenever an [animation](#concept-animation) has an [unresolved](#unresolved) [start time](#animation-start-time), its [current time](#animation-current-time) will be suspended.

<a id="ref-for-play-an-animation"></a>

<a id="ref-for-animation-current-time①⑧"></a>

As with [playing an animation](#play-an-animation), pausing may not happen instantaneously (see [§ 4.4.6 Waiting for the associated effect](#waiting-for-the-associated-effect)). For example, if animation is performed by a separate process, it may be necessary to synchronize the [current time](#animation-current-time) to ensure that it reflects the state drawn by the animation process.

The procedure to <a id="pause-an-animation"></a>pause an animation, <var>animation</var>, is as follows:

1.  <a id="ref-for-pending-pause-task⑦"></a>

    If <var>animation</var> has a [pending pause task](#pending-pause-task), abort these steps.

2.  <a id="ref-for-animation-play-state"></a>

    <a id="ref-for-play-state-paused"></a>

    If the [play state](#animation-play-state) of <var>animation</var> is [paused](#play-state-paused), abort these steps.

3.  <a id="ref-for-time-value②③"></a>

    <a id="ref-for-unresolved④⑦"></a>

    Let <var>seek time</var> be a [time value](#time-value) that is initially [unresolved](#unresolved).

4.  <a id="ref-for-timeline③⓪"></a>

    <a id="ref-for-monotonically-increasing-timeline②"></a>

    Let <var>has finite timeline</var> be true if <var>animation</var> has an associated [timeline](#timeline) that is not [monotonically increasing](#monotonically-increasing-timeline).

5.  <a id="ref-for-animation-current-time①⑨"></a>

    <a id="ref-for-unresolved④⑧"></a>

    If the <var>animation</var>’s [current time](#animation-current-time) is [unresolved](#unresolved), perform the steps according to the first matching condition from below:

    <a id="ref-for-playback-rate①③"></a>

    If <var>animation</var>’s [playback rate](#playback-rate) is ≥ 0,
    Set <var>seek time</var> to zero.

    Otherwise,
    <a id="ref-for-associated-effect-end④"></a>

    If [associated effect end](#associated-effect-end) for <var>animation</var> is positive infinity,
    <a id="ref-for-dfn-throw②"></a>

    <a id="ref-for-invalidstateerror①"></a>

    <a id="ref-for-idl-DOMException①"></a>

    [throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

    Otherwise,
    <a id="ref-for-associated-effect-end⑤"></a>

    Set <var>seek time</var> to <var>animation</var>’s [associated effect end](#associated-effect-end).

6.  <a id="ref-for-unresolved④⑨"></a>

    If <var>seek time</var> is [resolved](#unresolved),

    If <var>has finite timeline</var> is true,  
    <a id="ref-for-animation-start-time②③"></a>

    Set <var>animation</var>’s [start time](#animation-start-time) to <var>seek time</var>.

    Otherwise,  
    <a id="ref-for-animation-hold-time②①"></a>

    Set <var>animation</var>’s [hold time](#animation-hold-time) to <var>seek time</var>.

7.  Let <var>has pending ready promise</var> be a boolean flag that is initially false.

8.  <a id="ref-for-pending-play-task⑧"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task), cancel that task and let <var>has pending ready promise</var> be true.

9.  <a id="ref-for-current-ready-promise⑨"></a>

    <a id="ref-for-a-new-promise①"></a>

    <a id="ref-for-concept-relevant-realm②"></a>

    If <var>has pending ready promise</var> is false, set <var>animation</var>’s [current ready promise](#current-ready-promise) to [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>animation</var>.

10. Schedule a task to be executed at the first possible moment where <em>both</em> of the following conditions are true:

    - <a id="ref-for-animation-associated-effect⑨"></a>

      the user agent has performed any processing necessary to suspend the playback of <var>animation</var>’s [associated effect](#animation-associated-effect), if any.

    - <a id="ref-for-timeline③①"></a>

      <a id="ref-for-inactive-timeline⑧"></a>

      the animation is associated with a [timeline](#timeline) that is not [inactive](#inactive-timeline).

    The task shall perform the following steps:

    1.  <a id="ref-for-animation-associated-effect①⓪"></a>

        Let <var>ready time</var> be the time value of the timeline associated with <var>animation</var> at the moment when the user agent completed processing necessary to suspend playback of <var>animation</var>’s [associated effect](#animation-associated-effect).

    2.  <a id="ref-for-animation-start-time②④"></a>

        <a id="ref-for-unresolved⑤⓪"></a>

        <a id="ref-for-animation-hold-time②②"></a>

        <a id="ref-for-animation-start-time②⑤"></a>

        <a id="ref-for-playback-rate①④"></a>

        If <var>animation</var>’s [start time](#animation-start-time) is [resolved](#unresolved) and its [hold time](#animation-hold-time) is <em>not</em> resolved, let <var>animation</var>’s <a id="ref-for-animation-hold-time②③"></a>hold time be the result of evaluating <code>(<var>ready time</var> - <a href="#animation-start-time">start time</a>) × <a href="#playback-rate">playback rate</a></code>.

        <a id="ref-for-animation-hold-time②④"></a>

        <a id="ref-for-play-state-finished①"></a>

        <a id="ref-for-pending-play-task⑨"></a>

        <a id="ref-for-play-state-paused①"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: The [hold time](#animation-hold-time) might be already set if the animation is [finished](#play-state-finished), or if the animation has a [pending play task](#pending-play-task). In either case we want to preserve the <a id="ref-for-animation-hold-time②⑤"></a>hold time as we enter the [paused](#play-state-paused) state.

    3.  <a id="ref-for-apply-any-pending-playback-rate⑤"></a>

        [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

    4.  <a id="ref-for-animation-start-time②⑥"></a>

        Make <var>animation</var>’s [start time](#animation-start-time) unresolved.

    5.  <a id="ref-for-resolve③"></a>

        <a id="ref-for-current-ready-promise①⓪"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>animation</var>’s [current ready promise](#current-ready-promise) with <var>animation</var>.

    6.  <a id="ref-for-update-an-animations-finished-state⑦"></a>

        Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

    <a id="ref-for-pending-pause-task⑧"></a>

    So long as the above task is scheduled but has yet to run, <var>animation</var> is described as having a <a id="pending-pause-task"></a>pending pause task. While the task is running, however, <var>animation</var> does <em>not</em> have a [pending pause task](#pending-pause-task).

    <a id="ref-for-pending-play-task①⓪"></a>

    <a id="ref-for-pending-pause-task⑨"></a>

    <a id="ref-for-perform-a-microtask-checkpoint②"></a>

    As with the [pending play task](#pending-play-task), the user agent must run the [pending pause task](#pending-pause-task) asynchronously, although that may be as soon as the next [microtask checkpoint](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint).

11. <a id="ref-for-update-an-animations-finished-state⑧"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

#### <a id="reaching-the-end"></a>4.4.10. Reaching the end

<em>This section is non-normative</em>

<a id="ref-for-media-element"></a>

<a id="ref-for-animation-current-time②⓪"></a>

<a id="ref-for-end-time"></a>

<a id="ref-for-animation-associated-effect①①"></a>

DVD players or cassette players typically continue playing until they reach the end of their media at which point they stop. If such players are able to play in reverse, they typically stop playing when they reach the beginning of their media. In order to emulate this behavior and to provide consistency with HTML’s [media elements](https://html.spec.whatwg.org/multipage/embedded-content.html#media-element) [\[HTML\]](#biblio-html), the [current time](#animation-current-time) of Web Animations' animations do not play forwards beyond the [end time](#end-time) of their [associated effect](#animation-associated-effect) or play backwards past time zero.

An animation that has reached the natural boundary of its playback range is said to have <em>finished</em>.

Graphically, the effect of limiting the current time is shown below.

![The effect of limiting the current time of an animation.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/limiting.svg)

<a id="ref-for-animation-current-time②①"></a>

<a id="ref-for-concept-animation①⑤"></a>

<a id="ref-for-animation-associated-effect①②"></a>

<a id="ref-for-playback-rate①⑤"></a>

The effect of limiting the [current time](#animation-current-time) of an [animation](#concept-animation) with a start time of 1s, an [associated effect](#animation-associated-effect) of length 3s, and a positive [playback rate](#playback-rate). After the current time of the animation reaches the end of the associated effect, it is capped at 3s.

<a id="ref-for-animation-current-time②②"></a>

<a id="ref-for-concept-animation①⑥"></a>

<a id="ref-for-animation-associated-effect①③"></a>

It is possible, however, to <em>seek</em> the [current time](#animation-current-time) of an [animation](#concept-animation) to a time past the end of the [associated effect](#animation-associated-effect). When doing so, the current time will not progress but the animation will act as if it had been paused at the seeked time.

<a id="ref-for-animation-current-time②③"></a>

<a id="ref-for-animation-associated-effect①④"></a>

<a id="ref-for-end-time①"></a>

This allows, for example, seeking the [current time](#animation-current-time) of an animation with <em>no</em> [associated effect](#animation-associated-effect) to 5s. If <a id="ref-for-animation-associated-effect①⑤"></a>associated effect with an [end time](#end-time) later than 5s is later associated with the animation, playback will begin from the 5s mark.

<a id="ref-for-animation-associated-effect①⑥"></a>

Similar behavior to the above scenario may arise when the length of an animation’s [associated effect](#animation-associated-effect) changes.

<a id="ref-for-playback-rate①⑥"></a>

<a id="ref-for-animation-current-time②④"></a>

Similarly, when the [playback rate](#playback-rate) is negative, the [current time](#animation-current-time) does not progress past time zero.

#### <a id="the-current-finished-promise"></a>4.4.11. The current finished promise

<a id="ref-for-current-finished-promise①"></a>

<a id="ref-for-sec-promise-objects③"></a>

Each animation has a <a id="current-finished-promise"></a>current finished promise. The [current finished promise](#current-finished-promise) is initially a pending [Promise](http://www.ecma-international.org/ecma-262/6.0/#sec-promise-objects) object.

<a id="ref-for-sec-promise-objects④"></a>

<a id="ref-for-play-state-finished②"></a>

The object is replaced with a new [promise](http://www.ecma-international.org/ecma-262/6.0/#sec-promise-objects) every time the animation leaves the [finished play state](#play-state-finished).

#### <a id="updating-the-finished-state"></a>4.4.12. Updating the finished state

<a id="ref-for-playback-rate①⑦"></a>

<a id="ref-for-animation-current-time②⑤"></a>

<a id="ref-for-associated-effect-end⑥"></a>

For an animation with a positive [playback rate](#playback-rate), the [current time](#animation-current-time) continues to increase until it reaches the [associated effect end](#associated-effect-end).

<a id="ref-for-end-time②"></a>

<a id="ref-for-animation-associated-effect①⑦"></a>

<a id="ref-for-associated-effect-end⑦"></a>

The <a id="associated-effect-end"></a>associated effect end of an animation is equal to the [end time](#end-time) of the animation’s [associated effect](#animation-associated-effect). If the animation has no <a id="ref-for-animation-associated-effect①⑧"></a>associated effect, the [associated effect end](#associated-effect-end) is zero.

<a id="ref-for-playback-rate①⑧"></a>

<a id="ref-for-animation-current-time②⑥"></a>

For an animation with a negative [playback rate](#playback-rate), the [current time](#animation-current-time) continues to decrease until it reaches zero.

<a id="ref-for-unresolved⑤①"></a>

<a id="ref-for-animation-start-time②⑦"></a>

<a id="ref-for-play-state-finished③"></a>

A running animation that has reached this boundary (or overshot it) and has a [resolved](#unresolved) [start time](#animation-start-time) is said to be [finished](#play-state-finished).

<a id="ref-for-update-an-animations-finished-state⑨"></a>

<a id="ref-for-update-animations-and-send-events②"></a>

The crossing of this boundary is checked on each modification to the animation object using the procedure to [update an animation’s finished state](#update-an-animations-finished-state) defined below. This procedure is also run as part of the [update animations and send events](#update-animations-and-send-events) procedure. In both cases the <var>did seek</var> flag, defined below, is set to false.

<a id="ref-for-time-value②④"></a>

<a id="ref-for-unresolved⑤②"></a>

For each animation, the user agent maintains a <a id="previous-current-time"></a>previous current time [time value](#time-value) that is originally [unresolved](#unresolved).

<a id="ref-for-animation-current-time②⑦"></a>

<a id="ref-for-concept-animation①⑦"></a>

<a id="ref-for-animation-set-the-current-time"></a>

Whilst during normal playback the [current time](#animation-current-time) of an [animation](#concept-animation) is limited to the boundaries described above, it is possible to seek the current time of an animation to times outside those boundaries using the procedure to [set the current time](#animation-set-the-current-time) of an animation.

<a id="ref-for-animation-set-the-current-time①"></a>

The procedure to <a id="update-an-animations-finished-state"></a>update an animation’s finished state for <var>animation</var>, given a flag <var>did seek</var> (to indicate if the update is being performed after [setting the current time](#animation-set-the-current-time)), and a flag <var>synchronously notify</var> (to indicate the update was called in a context where we expect finished event queueing and finished promise resolution to happen immediately, if at all) is as follows:

1.  <a id="ref-for-animation-current-time②⑧"></a>

    <a id="ref-for-unresolved⑤③"></a>

    <a id="ref-for-animation-hold-time②⑥"></a>

    Let the <var>unconstrained current time</var> be the result of calculating the [current time](#animation-current-time) substituting an [unresolved](#unresolved) time value for the [hold time](#animation-hold-time) if <var>did seek</var> is false. If <var>did seek</var> is true, the <var>unconstrained current time</var> is equal to the <a id="ref-for-animation-current-time②⑨"></a>current time.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is required to accommodate timelines that may change direction. Without this definition, a once-finished animation would remain finished even when its timeline progresses in the opposite direction.

2.  If <em>all three</em> of the following conditions are true,

    - <a id="ref-for-unresolved⑤④"></a>

      the <var>unconstrained current time</var> is [resolved](#unresolved), <em>and</em>

    - <a id="ref-for-animation-start-time②⑧"></a>

      <a id="ref-for-unresolved⑤⑤"></a>

      <var>animation</var>’s [start time](#animation-start-time) is [resolved](#unresolved), <em>and</em>

    - <a id="ref-for-pending-play-task①①"></a>

      <a id="ref-for-pending-pause-task①⓪"></a>

      <var>animation</var> does <em>not</em> have a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task),

    <a id="ref-for-animation-hold-time②⑦"></a>

    then update <var>animation</var>’s [hold time](#animation-hold-time) based on the first matching condition for <var>animation</var> from below, if any:

    <a id="ref-for-associated-effect-end⑧"></a>

    <a id="ref-for-playback-rate①⑨"></a>

    If [playback rate](#playback-rate) \> 0 and <var>unconstrained current time</var> is greater than or equal to [associated effect end](#associated-effect-end),
    <a id="ref-for-animation-hold-time②⑧"></a>

    If <var>did seek</var> is true, let the [hold time](#animation-hold-time) be the value of <var>unconstrained current time</var>.

    <a id="ref-for-animation-hold-time②⑨"></a>

    <a id="ref-for-previous-current-time①"></a>

    <a id="ref-for-associated-effect-end⑨"></a>

    <a id="ref-for-unresolved⑤⑥"></a>

    If <var>did seek</var> is false, let the [hold time](#animation-hold-time) be the maximum value of [previous current time](#previous-current-time) and [associated effect end](#associated-effect-end). If the <a id="ref-for-previous-current-time②"></a>previous current time is [unresolved](#unresolved), let the <a id="ref-for-animation-hold-time③⓪"></a>hold time be <a id="ref-for-associated-effect-end①⓪"></a>associated effect end.

    <a id="ref-for-playback-rate②⓪"></a>

    If [playback rate](#playback-rate) \< 0 and <var>unconstrained current time</var> is less than or equal to 0,
    <a id="ref-for-animation-hold-time③①"></a>

    If <var>did seek</var> is true, let the [hold time](#animation-hold-time) be the value of <var>unconstrained current time</var>.

    <a id="ref-for-animation-hold-time③②"></a>

    <a id="ref-for-previous-current-time③"></a>

    <a id="ref-for-unresolved⑤⑦"></a>

    If <var>did seek</var> is false, let the [hold time](#animation-hold-time) be the minimum value of [previous current time](#previous-current-time) and zero. If the <a id="ref-for-previous-current-time④"></a>previous current time is [unresolved](#unresolved), let the <a id="ref-for-animation-hold-time③③"></a>hold time be zero.

    <a id="ref-for-inactive-timeline⑨"></a>

    <a id="ref-for-playback-rate②①"></a>

    If [playback rate](#playback-rate) ≠ 0, and <var>animation</var> is associated with an [active timeline](#inactive-timeline),
    Perform the following steps:

    1.  <a id="ref-for-animation-hold-time③④"></a>

        <a id="ref-for-unresolved⑤⑧"></a>

        <a id="ref-for-animation-start-time②⑨"></a>

        <a id="ref-for-animation-hold-time③⑤"></a>

        <a id="ref-for-playback-rate②②"></a>

        <a id="ref-for-time-value②⑤"></a>

        <a id="ref-for-timeline③②"></a>

        If <var>did seek</var> is true and the [hold time](#animation-hold-time) is [resolved](#unresolved), let <var>animation</var>’s [start time](#animation-start-time) be equal to the result of evaluating <code><var>timeline time</var> - (<a href="#animation-hold-time">hold time</a> / <a href="#playback-rate">playback rate</a>)</code> where <var>timeline time</var> is the current [time value](#time-value) of [timeline](#timeline) associated with <var>animation</var>.

    2.  <a id="ref-for-animation-hold-time③⑥"></a>

        <a id="ref-for-unresolved⑤⑨"></a>

        Let the [hold time](#animation-hold-time) be [unresolved](#unresolved).

3.  <a id="ref-for-previous-current-time⑤"></a>

    <a id="ref-for-animation-current-time③⓪"></a>

    Set the [previous current time](#previous-current-time) of <var>animation</var> be the result of calculating its [current time](#animation-current-time).

4.  <a id="ref-for-animation-play-state①"></a>

    <a id="ref-for-play-state-finished④"></a>

    Let <var>current finished state</var> be true if the [play state](#animation-play-state) of <var>animation</var> is [finished](#play-state-finished). Otherwise, let it be false.

5.  <a id="ref-for-current-finished-promise②"></a>

    If <var>current finished state</var> is true and the [current finished promise](#current-finished-promise) is not yet resolved, perform the following steps:

    1.  Let <a id="finish-notification-steps"></a>finish notification steps refer to the following procedure:

        1.  <a id="ref-for-animation-play-state②"></a>

            <a id="ref-for-play-state-finished⑤"></a>

            If <var>animation</var>’s [play state](#animation-play-state) is not equal to [finished](#play-state-finished), abort these steps.

        2.  <a id="ref-for-resolve④"></a>

            <a id="ref-for-current-finished-promise③"></a>

            [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>animation</var>’s [current finished promise](#current-finished-promise) object with <var>animation</var>.

        3.  <a id="ref-for-concept-event-create"></a>

            <a id="ref-for-animationplaybackevent"></a>

            [Create](https://dom.spec.whatwg.org/#concept-event-create) an <code><a href="#animationplaybackevent">AnimationPlaybackEvent</a></code>, <var>finishEvent</var>.

        4.  <a id="ref-for-dom-event-type"></a>

            <a id="ref-for-finish-event"></a>

            Set <var>finishEvent</var>’s <code><a href="https://dom.spec.whatwg.org/#dom-event-type">type</a></code> attribute to [finish](#finish-event).

        5.  <a id="ref-for-dom-animationplaybackevent-currenttime"></a>

            <a id="ref-for-animation-current-time③①"></a>

            Set <var>finishEvent</var>’s <code><a href="#dom-animationplaybackevent-currenttime">currentTime</a></code> attribute to the [current time](#animation-current-time) of <var>animation</var>.

        6.  <a id="ref-for-dom-animationplaybackevent-timelinetime"></a>

            <a id="ref-for-timeline-current-time⑤"></a>

            <a id="ref-for-timeline③③"></a>

            <a id="ref-for-inactive-timeline①⓪"></a>

            <a id="ref-for-dom-animationplaybackevent-timelinetime①"></a>

            Set <var>finishEvent</var>’s <code><a href="#dom-animationplaybackevent-timelinetime">timelineTime</a></code> attribute to the [current time](#timeline-current-time) of the [timeline](#timeline) with which <var>animation</var> is associated. If <var>animation</var> is not associated with a timeline, or the timeline is [inactive](#inactive-timeline), let <code><a href="#dom-animationplaybackevent-timelinetime">timelineTime</a></code> be `null`.

        7.  <a id="ref-for-animation-document-for-timing①"></a>

            <a id="ref-for-pending-animation-event-queue②"></a>

            <a id="ref-for-scheduled-event-time②"></a>

            <a id="ref-for-animation-time-to-origin-relative-time"></a>

            <a id="ref-for-associated-effect-end①①"></a>

            If <var>animation</var> has a [document for timing](#animation-document-for-timing), then append <var>finishEvent</var> to its <a id="ref-for-animation-document-for-timing②"></a>document for timing's [pending animation event queue](#pending-animation-event-queue) along with its target, <var>animation</var>. For the [scheduled event time](#scheduled-event-time), use the result of [converting](#animation-time-to-origin-relative-time) <var>animation</var>’s [associated effect end](#associated-effect-end) to an origin-relative time.

            <a id="ref-for-queue-a-task"></a>

            <a id="ref-for-concept-event-dispatch①"></a>

            <a id="ref-for-dom-manipulation-task-source"></a>

            Otherwise, [queue a task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-task) to [dispatch](https://dom.spec.whatwg.org/#concept-event-dispatch) <var>finishEvent</var> at <var>animation</var>. The task source for this task is the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source).

    2.  <a id="ref-for-finish-notification-steps"></a>

        If <var>synchronously notify</var> is true, cancel any queued microtask to run the [finish notification steps](#finish-notification-steps) for this <var>animation</var>, and run the <a id="ref-for-finish-notification-steps①"></a>finish notification steps immediately.

        <a id="ref-for-queue-a-microtask"></a>

        <a id="ref-for-finish-notification-steps②"></a>

        Otherwise, if <var>synchronously notify</var> is false, [queue a microtask](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-microtask) to run [finish notification steps](#finish-notification-steps) for <var>animation</var> unless there is already a microtask queued to run those steps for <var>animation</var>.

6.  <a id="ref-for-current-finished-promise④"></a>

    <a id="ref-for-a-new-promise②"></a>

    <a id="ref-for-concept-relevant-realm③"></a>

    If <var>current finished state</var> is false and <var>animation</var>’s [current finished promise](#current-finished-promise) is already resolved, set <var>animation</var>’s <a id="ref-for-current-finished-promise⑤"></a>current finished promise to [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>animation</var>.

<a id="ref-for-play-state-finished⑥"></a>

Typically, notification about the finished state of an animation is performed asynchronously. This allows for the animation to temporarily enter the [finished play state](#play-state-finished) without triggering events to be fired or promises to be resolved.

<a id="ref-for-finish-event①"></a>

<a id="ref-for-current-finished-promise⑥"></a>

For example, in the following code fragment, `animation` temporarily enters the finished state. If notification of the finished state occurred synchronously this code would cause the [finish event](#finish-event) to be queued and the [current finished promise](#current-finished-promise) to be resolved. However, if we reverse the order of the two statements such that the `iterations` is updated first, this would not happen. To avoid this surprising behavior, notification about the finished state of an animation is typically performed asynchronously.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63369f6e"></a>
>
> ```javascript
> var animation = elem.animate({ left: '100px' }, 2000);
> animation.playbackRate = 2;
> animation.currentTime = 1000; // animation is now finished
> animation.effect.updateTiming({ iterations: 2 }); // animation is no longer finished
> ```
<a id="ref-for-finish-an-animation"></a>

<a id="ref-for-dom-animation-finish"></a>

The one exception to this asynchronous behavior is when the [finish an animation](#finish-an-animation) procedure is performed (typically by calling the <code><a href="#dom-animation-finish">finish()</a></code> method). In this case the author’s intention to finish the animation is clear so the notification about the finished state of the animation occurs synchronously as demonstrated below.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3a2cd29f"></a>
>
> ```javascript
> var animation = elem.animate({ left: '100px' }, 1000);
> animation.finish(); // finish event is queued immediately and finished promise
>                     // is resolved despite the fact that the following statement
>                     // causes the animation to leave the finished state
> animation.currentTime = 0;
> ```
<a id="ref-for-finish-an-animation①"></a>

<a id="ref-for-cancel-an-animation"></a>

<a id="ref-for-cancel-event"></a>

<a id="ref-for-current-finished-promise⑦"></a>

<a id="ref-for-current-ready-promise①①"></a>

Note that like the procedure to [finish an animation](#finish-an-animation), the procedure to [cancel an animation](#cancel-an-animation) similarly queues the [cancel event](#cancel-event) and rejects the [current finished promise](#current-finished-promise) and [current ready promise](#current-ready-promise) in a <em>synchronous</em> manner.

#### <a id="finishing-an-animation-section"></a>4.4.13. Finishing an animation

An animation can be advanced to the natural end of its current playback direction by using the procedure to <a id="finish-an-animation"></a>finish an animation for <var>animation</var> defined below:

1.  <a id="ref-for-effective-playback-rate②"></a>

    <a id="ref-for-associated-effect-end①②"></a>

    <a id="ref-for-dfn-throw③"></a>

    <a id="ref-for-invalidstateerror②"></a>

    <a id="ref-for-idl-DOMException②"></a>

    If <var>animation</var>’s [effective playback rate](#effective-playback-rate) is zero, or if <var>animation</var>’s <a id="ref-for-effective-playback-rate③"></a>effective playback rate \> 0 and [associated effect end](#associated-effect-end) is infinity, [throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

2.  <a id="ref-for-apply-any-pending-playback-rate⑥"></a>

    [Apply any pending playback rate](#apply-any-pending-playback-rate) to <var>animation</var>.

3.  Set <var>limit</var> as follows:

    <a id="ref-for-playback-rate②③"></a>

    If [playback rate](#playback-rate) \> 0,
    <a id="ref-for-associated-effect-end①③"></a>

    Let <var>limit</var> be [associated effect end](#associated-effect-end).

    Otherwise,
    Let <var>limit</var> be zero.

4.  <a id="ref-for-animation-silently-set-the-current-time①"></a>

    [Silently set the current time](#animation-silently-set-the-current-time) to <var>limit</var>.

5.  <a id="ref-for-animation-start-time③⓪"></a>

    <a id="ref-for-unresolved⑥⓪"></a>

    <a id="ref-for-inactive-timeline①①"></a>

    <a id="ref-for-timeline③④"></a>

    <a id="ref-for-playback-rate②④"></a>

    <a id="ref-for-time-value②⑥"></a>

    If <var>animation</var>’s [start time](#animation-start-time) is [unresolved](#unresolved) and <var>animation</var> has an associated [active](#inactive-timeline) [timeline](#timeline), let the <a id="ref-for-animation-start-time③①"></a>start time be the result of evaluating <code><var>timeline time</var> -&#xA;(<var>limit</var> / <a href="#playback-rate">playback rate</a>)</code> where <var>timeline time</var> is the current [time value](#time-value) of the associated <a id="ref-for-timeline③⑤"></a>timeline.

6.  <a id="ref-for-pending-pause-task①①"></a>

    <a id="ref-for-animation-start-time③②"></a>

    <a id="ref-for-unresolved⑥①"></a>

    If there is a [pending pause task](#pending-pause-task) and [start time](#animation-start-time) is [resolved](#unresolved),

    1.  <a id="ref-for-animation-hold-time③⑦"></a>

        <a id="ref-for-unresolved⑥②"></a>

        Let the [hold time](#animation-hold-time) be [unresolved](#unresolved).

        <a id="ref-for-animation-hold-time③⑧"></a>

        <a id="ref-for-play-state-idle"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Typically the [hold time](#animation-hold-time) will already be unresolved except in the case when the animation was previously [idle](#play-state-idle).

    2.  <a id="ref-for-pending-pause-task①②"></a>

        Cancel the [pending pause task](#pending-pause-task).

    3.  <a id="ref-for-resolve⑤"></a>

        <a id="ref-for-current-ready-promise①②"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) the [current ready promise](#current-ready-promise) of <var>animation</var> with <var>animation</var>.

7.  <a id="ref-for-pending-play-task①②"></a>

    <a id="ref-for-animation-start-time③③"></a>

    <a id="ref-for-unresolved⑥③"></a>

    <a id="ref-for-resolve⑥"></a>

    <a id="ref-for-current-ready-promise①③"></a>

    If there is a [pending play task](#pending-play-task) and [start time](#animation-start-time) is [resolved](#unresolved), cancel that task and [resolve](https://webidl.spec.whatwg.org/#resolve) the [current ready promise](#current-ready-promise) of <var>animation</var> with <var>animation</var>.

8.  <a id="ref-for-update-an-animations-finished-state①⓪"></a>

    Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to true, and the <var>synchronously notify</var> flag set to true.

#### <a id="canceling-an-animation-section"></a>4.4.14. Canceling an animation

<a id="ref-for-animation-current-time③②"></a>

<a id="ref-for-unresolved⑥④"></a>

<a id="ref-for-animation-associated-effect①⑨"></a>

An animation can be canceled which causes the [current time](#animation-current-time) to become [unresolved](#unresolved) hence removing any effects caused by the [associated effect](#animation-associated-effect).

The procedure to <a id="cancel-an-animation"></a>cancel an animation for <var>animation</var> is as follows:

1.  <a id="ref-for-animation-play-state③"></a>

    <a id="ref-for-play-state-idle①"></a>

    If <var>animation</var>’s [play state](#animation-play-state) is <em>not</em> [idle](#play-state-idle), perform the following steps:

    1.  <a id="ref-for-animation-reset-an-animations-pending-tasks"></a>

        Run the procedure to [reset an animation’s pending tasks](#animation-reset-an-animations-pending-tasks) on <var>animation</var>.

    2.  <a id="ref-for-reject"></a>

        <a id="ref-for-current-finished-promise⑧"></a>

        [Reject](https://webidl.spec.whatwg.org/#reject) the [current finished promise](#current-finished-promise) with a DOMException named "AbortError".

    3.  <a id="ref-for-current-finished-promise⑨"></a>

        Set the \[\[PromiseIsHandled\]\] internal slot of the [current finished promise](#current-finished-promise) to true.

    4.  <a id="ref-for-current-finished-promise①⓪"></a>

        <a id="ref-for-a-new-promise③"></a>

        <a id="ref-for-concept-relevant-realm④"></a>

        Let [current finished promise](#current-finished-promise) be [a new promise](https://webidl.spec.whatwg.org/#a-new-promise) in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>animation</var>.

    5.  <a id="ref-for-concept-event-create①"></a>

        <a id="ref-for-animationplaybackevent①"></a>

        [Create](https://dom.spec.whatwg.org/#concept-event-create) an <code><a href="#animationplaybackevent">AnimationPlaybackEvent</a></code>, <var>cancelEvent</var>.

    6.  <a id="ref-for-dom-event-type①"></a>

        <a id="ref-for-cancel-event①"></a>

        Set <var>cancelEvent</var>’s <code><a href="https://dom.spec.whatwg.org/#dom-event-type">type</a></code> attribute to [cancel](#cancel-event).

    7.  <a id="ref-for-dom-animationplaybackevent-currenttime①"></a>

        Set <var>cancelEvent</var>’s <code><a href="#dom-animationplaybackevent-currenttime">currentTime</a></code> to `null`.

    8.  <a id="ref-for-timeline-current-time⑥"></a>

        <a id="ref-for-timeline③⑥"></a>

        <a id="ref-for-inactive-timeline①②"></a>

        <a id="ref-for-unresolved⑥⑤"></a>

        <a id="ref-for-time-value②⑦"></a>

        Let <var>timeline time</var> be the [current time](#timeline-current-time) of the [timeline](#timeline) with which <var>animation</var> is associated. If <var>animation</var> is not associated with an [active timeline](#inactive-timeline), let <var>timeline time</var> be n [unresolved](#unresolved) [time value](#time-value).

    9.  <a id="ref-for-dom-animationplaybackevent-timelinetime②"></a>

        <a id="ref-for-unresolved⑥⑥"></a>

        Set <var>cancelEvent</var>’s <code><a href="#dom-animationplaybackevent-timelinetime">timelineTime</a></code> to <var>timeline time</var>. If <var>timeline time</var> is [unresolved](#unresolved), set it to `null`.

    10. <a id="ref-for-animation-document-for-timing③"></a>

        <a id="ref-for-pending-animation-event-queue③"></a>

        <a id="ref-for-inactive-timeline①③"></a>

        <a id="ref-for-timeline-time-to-origin-relative-time①"></a>

        <a id="ref-for-scheduled-event-time③"></a>

        <a id="ref-for-unresolved⑥⑦"></a>

        <a id="ref-for-time-value②⑧"></a>

        If <var>animation</var> has a [document for timing](#animation-document-for-timing), then append <var>cancelEvent</var> to its <a id="ref-for-animation-document-for-timing④"></a>document for timing's [pending animation event queue](#pending-animation-event-queue) along with its target, <var>animation</var>. If <var>animation</var> is associated with an [active timeline](#inactive-timeline) that defines a procedure to [convert timeline times to origin-relative time](#timeline-time-to-origin-relative-time), let the [scheduled event time](#scheduled-event-time) be the result of applying that procedure to <var>timeline time</var>. Otherwise, the <a id="ref-for-scheduled-event-time④"></a>scheduled event time is an [unresolved](#unresolved) [time value](#time-value).

        <a id="ref-for-queue-a-task①"></a>

        <a id="ref-for-concept-event-dispatch②"></a>

        <a id="ref-for-dom-manipulation-task-source①"></a>

        Otherwise, [queue a task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-task) to [dispatch](https://dom.spec.whatwg.org/#concept-event-dispatch) <var>cancelEvent</var> at <var>animation</var>. The task source for this task is the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source).

2.  <a id="ref-for-animation-hold-time③⑨"></a>

    <a id="ref-for-unresolved⑥⑧"></a>

    Make <var>animation</var>’s [hold time](#animation-hold-time) [unresolved](#unresolved).

3.  <a id="ref-for-animation-start-time③④"></a>

    <a id="ref-for-unresolved⑥⑨"></a>

    Make <var>animation</var>’s [start time](#animation-start-time) [unresolved](#unresolved).

The procedure to <a id="animation-reset-an-animations-pending-tasks"></a>reset an animation’s pending tasks for <var>animation</var> is as follows:

1.  <a id="ref-for-pending-play-task①③"></a>

    <a id="ref-for-pending-pause-task①③"></a>

    If <var>animation</var> does not have a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task), abort this procedure.

2.  <a id="ref-for-pending-play-task①④"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task), cancel that task.

3.  <a id="ref-for-pending-pause-task①④"></a>

    If <var>animation</var> has a [pending pause task](#pending-pause-task), cancel that task.

4.  <a id="ref-for-apply-any-pending-playback-rate⑦"></a>

    [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

5.  <a id="ref-for-reject①"></a>

    <a id="ref-for-current-ready-promise①④"></a>

    [Reject](https://webidl.spec.whatwg.org/#reject) <var>animation</var>’s [current ready promise](#current-ready-promise) with a DOMException named "AbortError".

6.  <a id="ref-for-current-ready-promise①⑤"></a>

    Set the \[\[PromiseIsHandled\]\] internal slot of <var>animation</var>’s [current ready promise](#current-ready-promise) to true.

7.  <a id="ref-for-current-ready-promise①⑥"></a>

    <a id="ref-for-a-promise-resolved-with①"></a>

    <a id="ref-for-concept-relevant-realm⑤"></a>

    Let <var>animation</var>’s [current ready promise](#current-ready-promise) be the result of [creating a new resolved Promise object](https://webidl.spec.whatwg.org/#a-promise-resolved-with) with value <var>animation</var> in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>animation</var>.

#### <a id="speed-control"></a>4.4.15. Speed control

<a id="ref-for-animation-current-time③③"></a>

<a id="ref-for-timeline③⑦"></a>

<a id="ref-for-time-value②⑨"></a>

The rate of play of an animation can be controlled by setting its <em>playback rate</em>. For example, setting a playback rate of 2 will cause the animation’s [current time](#animation-current-time) to increase at twice the rate of its [timeline](#timeline). Similarly, a playback rate of -1 will cause the animation’s <a id="ref-for-animation-current-time③④"></a>current time to decrease at the same rate as the [time values](#time-value) from its <a id="ref-for-timeline③⑧"></a>timeline increase.

<a id="ref-for-concept-animation①⑧"></a>

<a id="ref-for-timeline③⑨"></a>

<a id="ref-for-time-value③⓪"></a>

<a id="ref-for-animation-current-time③⑤"></a>

<a id="ref-for-playback-rate②⑤"></a>

[Animations](#concept-animation) have a <a id="playback-rate"></a>playback rate that provides a scaling factor from the rate of change of the associated [timeline](#timeline)’s [time values](#time-value) to the animation’s [current time](#animation-current-time). The [playback rate](#playback-rate) is initially 1.

<a id="ref-for-playback-rate②⑥"></a>

<a id="ref-for-animation-play-state④"></a>

<a id="ref-for-play-state-paused②"></a>

Setting an animation’s [playback rate](#playback-rate) to zero effectively pauses the animation (however, the [play state](#animation-play-state) does not necessarily become [paused](#play-state-paused)).

##### <a id="setting-the-playback-rate-of-an-animation"></a>4.4.15.1. Setting the playback rate of an animation

<a id="ref-for-concept-animation①⑨"></a>

The procedure to <a id="set-the-playback-rate"></a>set the playback rate of an [animation](#concept-animation), <var>animation</var> to <var>new playback rate</var> is as follows:

1.  <a id="ref-for-pending-playback-rate②"></a>

    Clear any [pending playback rate](#pending-playback-rate) on <var>animation</var>.

2.  <a id="ref-for-animation-current-time③⑥"></a>

    <a id="ref-for-playback-rate②⑦"></a>

    Let <var>previous time</var> be the value of the [current time](#animation-current-time) of <var>animation</var> before changing the [playback rate](#playback-rate).

3.  <a id="ref-for-effective-playback-rate④"></a>

    Let <var>previous playback rate</var> be the current [effective playback rate](#effective-playback-rate) of <var>animation</var>.

4.  <a id="ref-for-playback-rate②⑧"></a>

    Set the [playback rate](#playback-rate) to <var>new playback rate</var>.

5.  Perform the steps corresponding to the <em>first</em> matching condition from the following, if any:

    <a id="ref-for-unresolved⑦⓪"></a>

    <a id="ref-for-timeline④⓪"></a>

    <a id="ref-for-monotonically-increasing-timeline③"></a>

    If <var>animation</var> is associated with a [monotonically increasing](#monotonically-increasing-timeline) [timeline](#timeline) and the <var>previous time</var> is [resolved](#unresolved),
    <a id="ref-for-animation-set-the-current-time②"></a>

    [set the current time](#animation-set-the-current-time) of <var>animation</var> to <var>previous time</var>.

    <a id="ref-for-timeline④①"></a>

    <a id="ref-for-monotonically-increasing-timeline④"></a>

    <a id="ref-for-animation-start-time③⑤"></a>

    <a id="ref-for-unresolved⑦①"></a>

    <a id="ref-for-associated-effect-end①④"></a>

    If <var>animation</var> is associated with a non-null [timeline](#timeline) that is not [monotonically increasing](#monotonically-increasing-timeline), the [start time](#animation-start-time) of <var>animation</var> is [resolved](#unresolved), [associated effect end](#associated-effect-end) is not infinity, and <em>either</em>:

    - the <var>previous playback rate</var> \< 0 and the <var>new playback rate</var> ≥ 0, or

    - the <var>previous playback rate</var> ≥ 0 and the <var>new playback rate</var> \< 0,

    <a id="ref-for-animation-start-time③⑥"></a>

    <a id="ref-for-associated-effect-end①⑤"></a>

    <a id="ref-for-animation-start-time③⑦"></a>

    Set <var>animation</var>’s [start time](#animation-start-time) to the result of evaluating <code><a href="#associated-effect-end">associated effect end</a> - <a href="#animation-start-time">start time</a></code> for <var>animation</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This effectively flips the animation start/end times on non-monotonic timelines preserving the relative offset of the start time from the other direction.

##### <a id="seamlessly-updating-the-playback-rate-of-an-animation"></a>4.4.15.2. Seamlessly updating the playback rate of an animation

<a id="ref-for-set-the-playback-rate①"></a>

For an in-flight animation that is running on another process or thread, the procedure to [set the playback rate](#set-the-playback-rate) may cause the animation to jump if the process or thread running the animation is not currently synchronized with the process or thread performing the update.

<a id="ref-for-playback-rate②⑨"></a>

<a id="ref-for-concept-animation②⓪"></a>

In order to produce seamless changes to the [playback rate](#playback-rate) of an [animation](#concept-animation), animation’s may have a <a id="pending-playback-rate"></a>pending playback rate that defines a playback rate to be applied after any necessary synchronization has taken place (for the case of animations running in a different thread or process).

<a id="ref-for-pending-playback-rate③"></a>

<a id="ref-for-concept-animation②①"></a>

Initially the [pending playback rate](#pending-playback-rate) of an [animation](#concept-animation) is unset.

<a id="ref-for-pending-playback-rate④"></a>

<a id="ref-for-playback-rate③⓪"></a>

The <a id="effective-playback-rate"></a>effective playback rate of an <var>animation</var> is its [pending playback rate](#pending-playback-rate), if set, otherwise it is the animation’s [playback rate](#playback-rate).

<a id="ref-for-concept-animation②②"></a>

When an [animation](#concept-animation), <var>animation</var>, is to <a id="apply-any-pending-playback-rate"></a>apply any pending playback rate the following steps are performed:

1.  <a id="ref-for-pending-playback-rate⑤"></a>

    If <var>animation</var> does not have a [pending playback rate](#pending-playback-rate), abort these steps.

2.  <a id="ref-for-playback-rate③①"></a>

    <a id="ref-for-pending-playback-rate⑥"></a>

    Set <var>animation</var>’s [playback rate](#playback-rate) to its [pending playback rate](#pending-playback-rate).

3.  <a id="ref-for-pending-playback-rate⑦"></a>

    Clear <var>animation</var>’s [pending playback rate](#pending-playback-rate).

<a id="ref-for-concept-animation②③"></a>

<a id="ref-for-animation-current-time③⑦"></a>

The procedure to <a id="seamlessly-update-the-playback-rate"></a>seamlessly update the playback rate an [animation](#concept-animation), <var>animation</var>, to <var>new playback rate</var> preserving its [current time](#animation-current-time) is as follows:

1.  <a id="ref-for-animation-play-state⑤"></a>

    Let <var>previous play state</var> be <var>animation</var>’s [play state](#animation-play-state).

    <a id="ref-for-effective-playback-rate⑤"></a>

    <a id="ref-for-pending-playback-rate⑧"></a>

    <a id="ref-for-play-state-finished⑦"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: It is necessary to record the play state before updating <var>animation</var>’s [effective playback rate](#effective-playback-rate) since, in the following logic, we want to immediately apply the [pending playback rate](#pending-playback-rate) of <var>animation</var> if it is <em>currently</em> [finished](#play-state-finished) regardless of whether or not it will still be finished after we apply the <a id="ref-for-pending-playback-rate⑨"></a>pending playback rate.

2.  <a id="ref-for-pending-playback-rate①⓪"></a>

    Let <var>animation</var>’s [pending playback rate](#pending-playback-rate) be <var>new playback rate</var>.

3.  Perform the steps corresponding to the first matching condition from below:

    <a id="ref-for-pending-pause-task①⑤"></a>

    <a id="ref-for-pending-play-task①⑤"></a>

    If <var>animation</var> has a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task),
    Abort these steps.

    <a id="ref-for-pending-playback-rate①①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The different types of pending tasks will apply the [pending playback rate](#pending-playback-rate) when they run so there is no further action required in this case.

    <a id="ref-for-unresolved⑦②"></a>

    <a id="ref-for-animation-current-time③⑧"></a>

    <a id="ref-for-play-state-paused③"></a>

    <a id="ref-for-play-state-idle②"></a>

    If <var>previous play state</var> is [idle](#play-state-idle) or [paused](#play-state-paused), or <var>animation</var>’s [current time](#animation-current-time) is [unresolved](#unresolved),
    <a id="ref-for-apply-any-pending-playback-rate⑧"></a>

    [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

    <a id="ref-for-play-state-running①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: the second condition above is required so that if we have a [running](#play-state-running) animation with an unresolved current time and no pending play task, we do not attempt to play it below.

    <a id="ref-for-play-state-finished⑧"></a>

    If <var>previous play state</var> is [finished](#play-state-finished),
    1.  <a id="ref-for-animation-current-time③⑨"></a>

        <a id="ref-for-unresolved⑦③"></a>

        <a id="ref-for-animation-hold-time④⓪"></a>

        Let the <var>unconstrained current time</var> be the result of calculating the [current time](#animation-current-time) of <var>animation</var> substituting an [unresolved](#unresolved) time value for the [hold time](#animation-hold-time).

    2.  <a id="ref-for-pending-playback-rate①②"></a>

        <a id="ref-for-animation-start-time③⑧"></a>

        Let <var>animation</var>’s [start time](#animation-start-time) be the result of evaluating the following expression:

        > <code><var>timeline time</var> - (<var>unconstrained current time</var> / <a href="#pending-playback-rate">pending playback rate</a>)</code>

        <a id="ref-for-time-value③①"></a>

        <a id="ref-for-timeline④②"></a>

        Where <var>timeline time</var> is the current [time value](#time-value) of the [timeline](#timeline) associated with <var>animation</var>.

        <a id="ref-for-pending-playback-rate①③"></a>

        <a id="ref-for-animation-start-time③⑨"></a>

        If [pending playback rate](#pending-playback-rate) is zero, let <var>animation</var>’s [start time](#animation-start-time) be <var>timeline time</var>.

    3.  <a id="ref-for-apply-any-pending-playback-rate⑨"></a>

        [Apply any pending playback rate](#apply-any-pending-playback-rate) on <var>animation</var>.

    4.  <a id="ref-for-update-an-animations-finished-state①①"></a>

        Run the procedure to [update an animation’s finished state](#update-an-animations-finished-state) for <var>animation</var> with the <var>did seek</var> flag set to false, and the <var>synchronously notify</var> flag set to false.

    Otherwise,
    <a id="ref-for-play-an-animation①"></a>

    Run the procedure to [play an animation](#play-an-animation) for <var>animation</var> with the <var>auto-rewind</var> flag set to false.

#### <a id="reversing-an-animation-section"></a>4.4.16. Reversing an animation

<a id="ref-for-concept-animation②④"></a>

The procedure to <a id="reverse-an-animation"></a>reverse an animation of [animation](#concept-animation) <var>animation</var> is as follows:

1.  <a id="ref-for-timeline④③"></a>

    <a id="ref-for-inactive-timeline①④"></a>

    <a id="ref-for-dfn-throw④"></a>

    <a id="ref-for-invalidstateerror③"></a>

    <a id="ref-for-idl-DOMException③"></a>

    If there is no [timeline](#timeline) associated with <var>animation</var>, or the associated <a id="ref-for-timeline④④"></a>timeline is [inactive](#inactive-timeline) [throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

2.  <a id="ref-for-pending-playback-rate①④"></a>

    Let <var>original pending playback rate</var> be <var>animation</var>’s [pending playback rate](#pending-playback-rate).

3.  <a id="ref-for-pending-playback-rate①⑤"></a>

    <a id="ref-for-effective-playback-rate⑥"></a>

    <a id="ref-for-effective-playback-rate⑦"></a>

    Let <var>animation</var>’s [pending playback rate](#pending-playback-rate) be the additive inverse of its [effective playback rate](#effective-playback-rate) (i.e. <code>-<a href="#effective-playback-rate">effective playback rate</a></code>).

4.  <a id="ref-for-play-an-animation②"></a>

    Run the steps to [play an animation](#play-an-animation) for <var>animation</var> with the <var>auto-rewind</var> flag set to true.

    <a id="ref-for-play-an-animation③"></a>

    <a id="ref-for-pending-playback-rate①⑥"></a>

    If the steps to [play an animation](#play-an-animation) throw an exception, set <var>animation</var>’s [pending playback rate](#pending-playback-rate) to <var>original pending playback rate</var> and propagate the exception.

#### <a id="play-states"></a>4.4.17. Play states

<a id="ref-for-concept-animation②⑤"></a>

An [animation](#concept-animation) may be described as being in one of the following <a id="animation-play-state"></a>play states for each of which, a non-normative description is also provided:

<a id="ref-for-play-state-idle③"></a>

[idle](#play-state-idle)

<a id="ref-for-animation-current-time④⓪"></a>

<a id="ref-for-unresolved⑦④"></a>

<a id="ref-for-animation-start-time④⓪"></a>

The [current time](#animation-current-time) of the animation is [unresolved](#unresolved) and the [start time](#animation-start-time) of the animation is <a id="ref-for-unresolved⑦⑤"></a>unresolved and there are no pending tasks. In this state the animation has no effect.

<a id="ref-for-play-state-running②"></a>

[running](#play-state-running)

<a id="ref-for-animation-current-time④①"></a>

<a id="ref-for-animation-frame②"></a>

<a id="ref-for-playback-rate③②"></a>

<a id="ref-for-timeline④⑤"></a>

<a id="ref-for-inactive-timeline①⑤"></a>

<a id="ref-for-monotonically-increasing-timeline⑤"></a>

The animation has a resolved [current time](#animation-current-time) that changes on each [animation frame](#animation-frame) (provided the [playback rate](#playback-rate) is not zero and the [timeline](#timeline) is [active](#inactive-timeline) and [monotonically increasing](#monotonically-increasing-timeline)).

<a id="ref-for-play-state-paused④"></a>

[paused](#play-state-paused)

<a id="ref-for-animation-current-time④②"></a>

The animation has been suspended and the [current time](#animation-current-time) is no longer changing.

<a id="ref-for-play-state-finished⑨"></a>

[finished](#play-state-finished)

<a id="ref-for-animation-current-time④③"></a>

The animation has reached the natural boundary of its playback range and the [current time](#animation-current-time) is no longer updating.

<a id="ref-for-animation-play-state⑥"></a>

<a id="ref-for-concept-animation②⑥"></a>

The [play state](#animation-play-state) of [animation](#concept-animation), <var>animation</var>, at a given moment is the state corresponding to the <em>first</em> matching condition from the following:

<em>All</em> of the following conditions are true:

- <a id="ref-for-animation-current-time④④"></a>

  <a id="ref-for-unresolved⑦⑥"></a>

  The [current time](#animation-current-time) of <var>animation</var> is [unresolved](#unresolved), <em>and</em>

- <a id="ref-for-animation-start-time④①"></a>

  <a id="ref-for-unresolved⑦⑦"></a>

  the [start time](#animation-start-time) of <var>animation</var> is [unresolved](#unresolved), <em>and</em>

- <a id="ref-for-pending-play-task①⑥"></a>

  <a id="ref-for-pending-pause-task①⑥"></a>

  <var>animation</var> does <em>not</em> have <em>either</em> a [pending play task](#pending-play-task) <em>or</em> a [pending pause task](#pending-pause-task),

→ <a id="play-state-idle"></a>idle

<em>Either</em> of the following conditions are true:

- <a id="ref-for-pending-pause-task①⑦"></a>

  <var>animation</var> has a [pending pause task](#pending-pause-task), <em>or</em>

- <a id="ref-for-animation-start-time④②"></a>

  <a id="ref-for-unresolved⑦⑧"></a>

  <a id="ref-for-pending-play-task①⑦"></a>

  <em>both</em> the [start time](#animation-start-time) of <var>animation</var> is [unresolved](#unresolved) <em>and</em> it does <em>not</em> have a [pending play task](#pending-play-task),

→ <a id="play-state-paused"></a>paused

<a id="ref-for-animation-current-time④⑤"></a>

<a id="ref-for-unresolved⑦⑨"></a>

For <var>animation</var>, [current time](#animation-current-time) is [resolved](#unresolved) and <em>either</em> of the following conditions are true:

- <a id="ref-for-effective-playback-rate⑧"></a>

  <a id="ref-for-animation-current-time④⑥"></a>

  <a id="ref-for-associated-effect-end①⑥"></a>

  <var>animation</var>’s [effective playback rate](#effective-playback-rate) \> 0 and [current time](#animation-current-time) ≥ [associated effect end](#associated-effect-end); <em>or</em>

- <a id="ref-for-effective-playback-rate⑨"></a>

  <a id="ref-for-animation-current-time④⑦"></a>

  <var>animation</var>’s [effective playback rate](#effective-playback-rate) \< 0 and [current time](#animation-current-time) ≤ 0,

→ <a id="play-state-finished"></a>finished

Otherwise,

→ <a id="play-state-running"></a>running

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-play-state-paused⑤"></a>
>
> <a id="ref-for-play-state-finished①⓪"></a>
>
> Note that the [paused play state](#play-state-paused) effectively "wins" over the [finished play state](#play-state-finished).
>
> <a id="ref-for-play-state-paused⑥"></a>
>
> <a id="ref-for-play-state-finished①①"></a>
>
> <a id="ref-for-animation-start-time④③"></a>
>
> However, an animation that is paused outside of its natural playback range can be converted from a [paused](#play-state-paused) animation into a [finished](#play-state-finished) animation without restarting by setting the [start time](#animation-start-time) such as below:
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-e0f669c8"></a>
> >
> > ```javascript
> > animation.effect.updateTiming({ duration: 5000 });
> > animation.currentTime = 4000;
> > animation.pause();
> > animation.ready.then(function() {
> >   animation.effect.updateTiming({ duration: 3000 });
> >   alert(animation.playState); // Displays 'paused'
> >   animation.startTime =
> >     document.timeline.currentTime - animation.currentTime * animation.playbackRate;
> >   alert(animation.playState); // Displays 'finished'
> > });
> > ```
#### <a id="animation-events-section"></a>4.4.18. Animation events

<a id="ref-for-animation-playback-events"></a>

<a id="ref-for-transition-events"></a>

<a id="ref-for-events"></a>

<a id="ref-for-animation-events②"></a>

<a id="animation-events"></a>Animation events include the [animation playback events](#animation-playback-events) defined in this specification as well as the [events from CSS transitions](https://drafts.csswg.org/css-transitions/#transition-events) [\[CSS-TRANSITIONS-1\]](#biblio-css-transitions-1) and [events from CSS animations](https://drafts.csswg.org/css-animations/#events) [\[CSS-ANIMATIONS-1\]](#biblio-css-animations-1). Future specifications may extend this set with further types of [animation events](#animation-events).

<a id="ref-for-document④"></a>

<a id="ref-for-animation-events③"></a>

<a id="ref-for-scheduled-event-time⑤"></a>

<a id="ref-for-time-value③②"></a>

<a id="ref-for-concept-settings-object-time-origin②"></a>

<a id="ref-for-update-animations-and-send-events③"></a>

<a id="ref-for-unresolved⑧⓪"></a>

<a id="ref-for-concept-animation②⑦"></a>

<a id="ref-for-timeline④⑥"></a>

<a id="ref-for-inactive-timeline①⑥"></a>

Each <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> maintains a <a id="pending-animation-event-queue"></a>pending animation event queue that stores [animation events](#animation-events) along with their corresponding event targets and <a id="scheduled-event-time"></a>scheduled event time. The [scheduled event time](#scheduled-event-time) is a [time value](#time-value) relative to the [time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin) representing when the event would ideally have been dispatched were animations updated at an infinitely high frequency. It is used by the procedure to [update animations and send events](#update-animations-and-send-events) to sort queued <a id="ref-for-animation-events④"></a>animation events chronologically. Note that this value may be [unresolved](#unresolved) if, for example, the [animation](#concept-animation)'s [timeline](#timeline) produces values that are unrelated to the <a id="ref-for-concept-settings-object-time-origin③"></a>time origin (e.g. a timeline that tracks scroll-position) or if the <a id="ref-for-timeline④⑦"></a>timeline is [inactive](#inactive-timeline).

##### <a id="sorting-animation-events"></a>4.4.18.1. Sorting animation events

The following definitions are provided to assist with sorting queued events.

<a id="ref-for-time-value③③"></a>

<a id="ref-for-animation-start-time④④"></a>

To <a id="animation-time-to-timeline-time"></a>convert an animation time to timeline time a [time value](#time-value), <var>time</var>, that is relative to the [start time](#animation-start-time) of an animation, <var>animation</var>, perform the following steps:

1.  <a id="ref-for-unresolved⑧①"></a>

    If <var>time</var> is [unresolved](#unresolved), return <var>time</var>.

2.  <a id="ref-for-unresolved⑧②"></a>

    <a id="ref-for-time-value③④"></a>

    If <var>time</var> is infinity, return an [unresolved](#unresolved) [time value](#time-value).

3.  <a id="ref-for-playback-rate③③"></a>

    <a id="ref-for-unresolved⑧③"></a>

    <a id="ref-for-time-value③⑤"></a>

    If <var>animation</var>’s [playback rate](#playback-rate) is zero, return an [unresolved](#unresolved) [time value](#time-value).

4.  <a id="ref-for-animation-start-time④⑤"></a>

    <a id="ref-for-unresolved⑧④"></a>

    <a id="ref-for-time-value③⑥"></a>

    If <var>animation</var>’s [start time](#animation-start-time) is [unresolved](#unresolved), return an <a id="ref-for-unresolved⑧⑤"></a>unresolved [time value](#time-value).

5.  <a id="ref-for-playback-rate③④"></a>

    <a id="ref-for-animation-start-time④⑥"></a>

    Return the result of calculating: <code><var>time</var> × (1 / <var>playback rate</var>) + <var>start time</var></code> (where <var>playback rate</var> and <var>start time</var> are the [playback rate](#playback-rate) and [start time](#animation-start-time) of <var>animation</var>, respectively).

<a id="ref-for-time-value③⑦"></a>

<a id="ref-for-timeline④⑧"></a>

To <a id="animation-time-to-origin-relative-time"></a>convert a timeline time to an origin-relative time a [time value](#time-value), <var>time</var>, that is expressed in the same scale as the <a id="ref-for-time-value③⑧"></a>time values of a [timeline](#timeline), <var>timeline</var>, perform the following steps:

1.  <a id="ref-for-animation-time-to-timeline-time"></a>

    Let <var>timeline time</var> be the result of [converting](#animation-time-to-timeline-time) <var>time</var> from an animation time to a timeline time.

2.  <a id="ref-for-unresolved⑧⑥"></a>

    If <var>timeline time</var> is [unresolved](#unresolved), return <var>time</var>.

3.  <a id="ref-for-timeline④⑨"></a>

    <a id="ref-for-unresolved⑧⑦"></a>

    If <var>animation</var> is not associated with a [timeline](#timeline), return an [unresolved](#unresolved) time value.

4.  <a id="ref-for-inactive-timeline①⑦"></a>

    <a id="ref-for-unresolved⑧⑧"></a>

    If <var>animation</var> is associated with an [inactive timeline](#inactive-timeline), return an [unresolved](#unresolved) time value.

5.  <a id="ref-for-timeline-time-to-origin-relative-time②"></a>

    <a id="ref-for-unresolved⑧⑨"></a>

    <a id="ref-for-time-value③⑨"></a>

    If there is no procedure to [convert a timeline time to an origin-relative time](#timeline-time-to-origin-relative-time) for the timeline associated with <var>animation</var>, return an [unresolved](#unresolved) [time value](#time-value).

6.  <a id="ref-for-timeline-time-to-origin-relative-time③"></a>

    <a id="ref-for-timeline⑤⓪"></a>

    Return the result of [converting](#timeline-time-to-origin-relative-time) <var>timeline time</var> to an origin-relative time using the procedure defined for the [timeline](#timeline) associated with <var>animation</var>.

##### <a id="animation-playback-events-section"></a>4.4.18.2. Animation playback events

<a id="ref-for-concept-animation②⑧"></a>

As [animations](#concept-animation) play, they report changes to their status through <a id="animation-playback-events"></a>animation playback events.

<a id="ref-for-animation-playback-events①"></a>

<a id="ref-for-animation-associated-effect②⓪"></a>

<a id="ref-for-concept-animation②⑨"></a>

[Animation playback events](#animation-playback-events) are a property of the timing model. As a result they are dispatched even when the [associated effect](#animation-associated-effect) of the [animation](#concept-animation) is absent or has no observable result.

##### <a id="animation-playback-event-types"></a>4.4.18.3. Types of animation playback events

<a id="finish-event"></a>finish  
<a id="ref-for-play-state-finished①②"></a>

Queued whenever an animation enters the [finished play state](#play-state-finished).

<a id="cancel-event"></a>cancel  
<a id="ref-for-play-state-idle④"></a>

<a id="ref-for-concept-animation③⓪"></a>

<a id="ref-for-cancel-event②"></a>

Queued whenever an animation enters the [idle play state](#play-state-idle) from another state. Creating a new [animation](#concept-animation) that is initially idle does <em>not</em> generate a new [cancel event](#cancel-event).

<a id="remove-event"></a>remove  
Queued whenever an animation is automatically removed. See [§ 5.5 Replacing animations](#replacing-animations).

### <a id="animation-effects"></a>4.5. Animation effects

An <a id="animation-effect"></a>animation effect is an abstract term referring to an item in the timing hierarchy.

#### <a id="animation-effects-and-animations"></a>4.5.1. Relationship between animation effects and animations

<a id="ref-for-animation-associated-effect②①"></a>

<a id="ref-for-concept-animation③①"></a>

<a id="ref-for-animation-effect⑥"></a>

<a id="ref-for-associated-with-an-animation"></a>

The [associated effect](#animation-associated-effect) of an [animation](#concept-animation), if set, is a type of [animation effect](#animation-effect). The <a id="ref-for-animation-associated-effect②②"></a>associated effect of an <a id="ref-for-concept-animation③②"></a>animation is said to be <a id="associated-with-an-animation"></a>associated with that animation. At a given moment, an <a id="ref-for-animation-effect⑦"></a>animation effect can be [associated](#associated-with-an-animation) with at most one <a id="ref-for-concept-animation③③"></a>animation.

<a id="ref-for-animation-effect⑧"></a>

<a id="ref-for-associated-with-an-animation①"></a>

An [animation effect](#animation-effect), <var>effect</var>, is <a id="associated-with-a-timeline"></a>associated with a timeline, <var>timeline</var>, if <var>effect</var> is [associated with an animation](#associated-with-an-animation) which, in turn, is associated with <var>timeline</var>.

#### <a id="types-of-animation-effects"></a>4.5.2. Types of animation effects

<a id="ref-for-animation-effect⑨"></a>

<a id="ref-for-keyframe-effect①"></a>

This specification defines a single type of [animation effect](#animation-effect): [keyframe effects](#keyframe-effect). Subsequent levels of this specification will define further types of <a id="ref-for-animation-effect①⓪"></a>animation effects.

<a id="ref-for-animation-effect①①"></a>

All types of [animation effects](#animation-effect) define a number of common properties which are described in the following sections.

#### <a id="the-active-interval"></a>4.5.3. The active interval

<a id="ref-for-animation-effect①②"></a>

<a id="ref-for-active-interval"></a>

The period that an [animation effect](#animation-effect) is scheduled to run is called its [active interval](#active-interval). Each <a id="ref-for-animation-effect①③"></a>animation effect has only one such interval.

<a id="ref-for-active-interval①"></a>

<a id="ref-for-animation-start-time④⑦"></a>

<a id="ref-for-concept-animation③④"></a>

<a id="ref-for-animation-effect①④"></a>

<a id="ref-for-start-delay"></a>

The lower bound of the [active interval](#active-interval) typically corresponds to the [start time](#animation-start-time) of the [animation](#concept-animation) associated with this [animation effect](#animation-effect) but may be shifted by a [start delay](#start-delay) on the <a id="ref-for-animation-effect①⑤"></a>animation effect.

<a id="ref-for-active-duration"></a>

The upper bound of the interval is determined by the [active duration](#active-duration).

<a id="ref-for-animation-start-time④⑧"></a>

<a id="ref-for-start-delay①"></a>

<a id="ref-for-active-duration①"></a>

The relationship between the [start time](#animation-start-time), [start delay](#start-delay), and [active duration](#active-duration) is illustrated below.

![Examples of the effect of the start delay on the endpoints of the active interval](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/active-interval-examples.svg)

<a id="ref-for-start-delay②"></a>

<a id="ref-for-active-interval②"></a>

<a id="ref-for-animation-start-time④⑨"></a>

Examples of the effect of the [start delay](#start-delay) on the endpoints of the [active interval](#active-interval).  
(a) An animation effect with no delay; the [start time](#animation-start-time) and beginning of the <a id="ref-for-active-interval③"></a>active interval are coincident.  
(b) An animation effect with a positive delay; the beginning of the <a id="ref-for-active-interval④"></a>active interval is deferred by the delay.  
(c) An animation effect with a negative delay; the beginning of the <a id="ref-for-active-interval⑤"></a>active interval is brought forward by the delay.

<a id="ref-for-end-delay"></a>

An [end delay](#end-delay) may also be specified but is primarily only of use when sequencing animations.

<a id="ref-for-animation-effect①⑥"></a>

<a id="ref-for-fill-mode"></a>

<a id="ref-for-active-interval⑥"></a>

[Animation effects](#animation-effect) define an <a id="active-interval"></a>active interval which is the period of time during which the effect is scheduled to produce its effect with the exception of [fill modes](#fill-mode) which apply outside the [active interval](#active-interval).

<a id="ref-for-active-interval⑦"></a>

<a id="ref-for-start-delay③"></a>

The lower bound of the [active interval](#active-interval) is defined by the [start delay](#start-delay).

<a id="ref-for-animation-effect①⑦"></a>

<a id="ref-for-animation-start-time⑤⓪"></a>

<a id="ref-for-concept-animation③⑤"></a>

The <a id="start-delay"></a>start delay of an [animation effect](#animation-effect) is a signed offset from the [start time](#animation-start-time) of the [animation](#concept-animation) with which the animation effect is associated.

<a id="ref-for-active-interval⑧"></a>

<a id="ref-for-active-duration②"></a>

The length of the [active interval](#active-interval) is called the [active duration](#active-duration), the calculation of which is defined in [§ 4.8.2 Calculating the active duration](#calculating-the-active-duration).

<a id="ref-for-start-delay④"></a>

<a id="ref-for-animation-effect①⑧"></a>

<a id="ref-for-end-time③"></a>

Similar to the [start delay](#start-delay), an [animation effect](#animation-effect) also has an <a id="end-delay"></a>end delay which is primarily of use when sequencing animations based on the [end time](#end-time) of another <a id="ref-for-animation-effect①⑨"></a>animation effect. Although this is typically only useful in combination with sequence effects which are introduced in a subsequent level of this specification, it is included here for the purpose of representing the [`min`](https://www.w3.org/TR/SVG/animate.html#MinAttribute) attribute in SVG ([\[SVG11\]](#biblio-svg11), Chapter 19).

<a id="ref-for-animation-effect②⓪"></a>

<a id="ref-for-start-delay⑤"></a>

<a id="ref-for-active-duration③"></a>

<a id="ref-for-end-delay①"></a>

The <a id="end-time"></a>end time of an [animation effect](#animation-effect) is the result of evaluating <code>max(<a href="#start-delay">start delay</a> + <a href="#active-duration">active&#xA;duration</a> + <a href="#end-delay">end delay</a>, 0)</code>.

#### <a id="local-time-section"></a>4.5.4. Local time

<a id="ref-for-animation-effect②①"></a>

The <a id="local-time"></a>local time of an [animation effect](#animation-effect) at a given moment is based on the first matching condition from the following:

<a id="ref-for-associated-with-an-animation②"></a>

<a id="ref-for-animation-effect②②"></a>

If the [animation effect](#animation-effect) is [associated with an animation](#associated-with-an-animation),

<a id="ref-for-animation-current-time④⑧"></a>

<a id="ref-for-concept-animation③⑥"></a>

the local time is the [current time](#animation-current-time) of the [animation](#concept-animation).

Otherwise,

<a id="ref-for-unresolved⑨⓪"></a>

the local time is [unresolved](#unresolved).

#### <a id="animation-effect-phases-and-states"></a>4.5.5. Animation effect phases and states

<em>This section is non-normative</em>

<a id="ref-for-animation-effect②③"></a>

<a id="ref-for-unresolved⑨①"></a>

<a id="ref-for-local-time"></a>

At a given moment, an [animation effect](#animation-effect) may be in one of three possible <em>phases</em>. If an <a id="ref-for-animation-effect②④"></a>animation effect has an [unresolved](#unresolved) [local time](#local-time) it will not be in any phase.

The different phases are illustrated below.

![An example of the different phases and states used to describe an animation effect.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/animation-effect-phases-and-states.svg)

<a id="ref-for-animation-effect②⑤"></a>

An example of the different phases and states used to describe an [animation effect](#animation-effect).

The phases are as follows:

<a id="ref-for-animation-effect-before-phase"></a>

[before phase](#animation-effect-before-phase)

<a id="ref-for-animation-effect②⑥"></a>

<a id="ref-for-local-time①"></a>

<a id="ref-for-active-interval⑨"></a>

<a id="ref-for-end-time④"></a>

<a id="ref-for-start-delay⑥"></a>

The [animation effect](#animation-effect)’s [local time](#local-time) falls before the effect’s [active interval](#active-interval) and [end time](#end-time), <em>or</em> occurs during the range when a negative [start delay](#start-delay) is in effect.

<a id="ref-for-animation-effect-active-phase"></a>

[active phase](#animation-effect-active-phase)

<a id="ref-for-animation-effect②⑦"></a>

<a id="ref-for-local-time②"></a>

<a id="ref-for-active-interval①⓪"></a>

<a id="ref-for-start-delay⑦"></a>

<a id="ref-for-end-delay②"></a>

The [animation effect](#animation-effect)’s [local time](#local-time) falls inside the effect’s [active interval](#active-interval) and outside the range of any negative [start delay](#start-delay) or negative [end delay](#end-delay).

<a id="ref-for-animation-effect-after-phase"></a>

[after phase](#animation-effect-after-phase)

<a id="ref-for-animation-effect②⑧"></a>

<a id="ref-for-local-time③"></a>

<a id="ref-for-active-interval①①"></a>

<a id="ref-for-end-time⑤"></a>

<a id="ref-for-end-delay③"></a>

<a id="ref-for-start-delay⑧"></a>

The [animation effect](#animation-effect)’s [local time](#local-time) falls after the effect’s [active interval](#active-interval) or after the [end time](#end-time) if that comes first (due to a negative [end delay](#end-delay)), but <em>not</em> during the range when a negative [start delay](#start-delay) is in effect.

<a id="ref-for-animation-effect②⑨"></a>

<a id="ref-for-animation-frame③"></a>

In addition to these phases, an [animation effect](#animation-effect) may also be described as being in one of several overlapping <em>states</em>. These states are only established for the duration of a single [animation frame](#animation-frame) and are primarily a convenience for describing stative parts of the model.

These states and their usage within the model are summarized as follows:

<a id="ref-for-in-play"></a>

[in play](#in-play)

<a id="ref-for-animation-effect③⓪"></a>

<a id="ref-for-active-time"></a>

Corresponds to an [animation effect](#animation-effect) whose [active time](#active-time) is changing on each frame.

<a id="ref-for-current"></a>

[current](#current)

<a id="ref-for-animation-effect③①"></a>

<a id="ref-for-in-play①"></a>

<a id="ref-for-concept-animation③⑦"></a>

<a id="ref-for-playback-rate③⑤"></a>

Corresponds to an [animation effect](#animation-effect) that is either [in play](#in-play) or may become <a id="ref-for-in-play②"></a>in play in the future based on its [animation](#concept-animation)'s current [playback rate](#playback-rate).

<a id="ref-for-in-effect"></a>

[in effect](#in-effect)

<a id="ref-for-animation-effect③②"></a>

<a id="ref-for-active-time①"></a>

<a id="ref-for-animation-effect-active-phase①"></a>

<a id="ref-for-fill-mode①"></a>

<a id="ref-for-in-effect①"></a>

Corresponds to an [animation effect](#animation-effect) that has a resolved [active time](#active-time). This occurs when either the <a id="ref-for-animation-effect③③"></a>animation effect is in its [active phase](#animation-effect-active-phase) or outside the <a id="ref-for-animation-effect-active-phase②"></a>active phase but at a time where the effect’s [fill mode](#fill-mode) (see [§ 4.6 Fill behavior](#fill-behavior)) causes its <a id="ref-for-active-time②"></a>active time to be resolved. Only [in effect](#in-effect) <a id="ref-for-animation-effect③④"></a>animation effects apply a result to their target.

The normative definition of each of these states follows.

<a id="ref-for-animation-effect③⑤"></a>

Determining the phase of an [animation effect](#animation-effect) requires the following definitions:

<a id="animation-direction"></a>animation direction  
<a id="ref-for-associated-with-an-animation③"></a>

<a id="ref-for-concept-animation③⑧"></a>

<a id="ref-for-playback-rate③⑥"></a>

<a id="ref-for-animation-direction"></a>

"backwards" if the effect is [associated with an animation](#associated-with-an-animation) <em>and</em> the associated [animation](#concept-animation)’s [playback rate](#playback-rate) is less than zero; in all other cases, the [animation direction](#animation-direction) is "forwards".

<a id="before-active-boundary-time"></a>before-active boundary time  
<a id="ref-for-start-delay⑨"></a>

<a id="ref-for-end-time⑥"></a>

<code>max(min(<a href="#start-delay">start delay</a>, <a href="#end-time">end time</a>), 0)</code>

<a id="active-after-boundary-time"></a>active-after boundary time  
<a id="ref-for-start-delay①⓪"></a>

<a id="ref-for-active-duration④"></a>

<a id="ref-for-end-time⑦"></a>

<code>max(min(<a href="#start-delay">start delay</a> + <a href="#active-duration">active duration</a>, <a href="#end-time">end time</a>), 0)</code>

<a id="ref-for-animation-effect③⑥"></a>

<a id="ref-for-local-time④"></a>

<a id="ref-for-unresolved⑨②"></a>

An [animation effect](#animation-effect) is in the <a id="animation-effect-before-phase"></a>before phase if the animation effect’s [local time](#local-time) is not [unresolved](#unresolved) and <em>either</em> of the following conditions are met:

1.  <a id="ref-for-local-time⑤"></a>

    <a id="ref-for-before-active-boundary-time"></a>

    the [local time](#local-time) is less than the [before-active boundary time](#before-active-boundary-time), <em>or</em>

2.  <a id="ref-for-animation-direction①"></a>

    <a id="ref-for-local-time⑥"></a>

    <a id="ref-for-before-active-boundary-time①"></a>

    the [animation direction](#animation-direction) is "backwards" and the [local time](#local-time) is equal to the [before-active boundary time](#before-active-boundary-time).

<a id="ref-for-animation-effect③⑦"></a>

<a id="ref-for-local-time⑦"></a>

<a id="ref-for-unresolved⑨③"></a>

An [animation effect](#animation-effect) is in the <a id="animation-effect-after-phase"></a>after phase if the animation effect’s [local time](#local-time) is not [unresolved](#unresolved) and <em>either</em> of the following conditions are met:

1.  <a id="ref-for-local-time⑧"></a>

    <a id="ref-for-active-after-boundary-time"></a>

    the [local time](#local-time) is greater than the [active-after boundary time](#active-after-boundary-time), <em>or</em>

2.  <a id="ref-for-animation-direction②"></a>

    <a id="ref-for-local-time⑨"></a>

    <a id="ref-for-active-after-boundary-time①"></a>

    the [animation direction](#animation-direction) is "forwards" and the [local time](#local-time) is equal to the [active-after boundary time](#active-after-boundary-time).

<a id="ref-for-animation-effect③⑧"></a>

<a id="ref-for-local-time①⓪"></a>

<a id="ref-for-unresolved⑨④"></a>

<a id="ref-for-animation-effect-before-phase①"></a>

<a id="ref-for-animation-effect-after-phase①"></a>

An [animation effect](#animation-effect) is in the <a id="animation-effect-active-phase"></a>active phase if the animation effect’s [local time](#local-time) is not [unresolved](#unresolved) and it is not in either the [before phase](#animation-effect-before-phase) nor the [after phase](#animation-effect-after-phase).

Furthermore, it is often convenient to refer to the case when an animation effect is in none of the above phases as being in the <a id="animation-effect-idle-phase"></a>idle phase.

<a id="ref-for-animation-effect③⑨"></a>

An [animation effect](#animation-effect) is <a id="in-play"></a>in play if <em>all</em> of the following conditions are met:

1.  <a id="ref-for-animation-effect④⓪"></a>

    <a id="ref-for-animation-effect-active-phase③"></a>

    the [animation effect](#animation-effect) is in the [active phase](#animation-effect-active-phase), and

2.  <a id="ref-for-animation-effect④①"></a>

    <a id="ref-for-associated-with-an-animation④"></a>

    <a id="ref-for-play-state-finished①③"></a>

    the [animation effect](#animation-effect) is [associated with an animation](#associated-with-an-animation) that is not [finished](#play-state-finished).

<a id="ref-for-animation-effect④②"></a>

An [animation effect](#animation-effect) is <a id="current"></a>current if <em>any</em> of the following conditions are true:

- <a id="ref-for-animation-effect④③"></a>

  <a id="ref-for-in-play③"></a>

  the [animation effect](#animation-effect) is [in play](#in-play), or

- <a id="ref-for-animation-effect④④"></a>

  <a id="ref-for-associated-with-an-animation⑤"></a>

  <a id="ref-for-playback-rate③⑦"></a>

  <a id="ref-for-animation-effect-before-phase②"></a>

  the [animation effect](#animation-effect) is [associated with an animation](#associated-with-an-animation) with a [playback rate](#playback-rate) \> 0 and the <a id="ref-for-animation-effect④⑤"></a>animation effect is in the [before phase](#animation-effect-before-phase), or

- <a id="ref-for-animation-effect④⑥"></a>

  <a id="ref-for-associated-with-an-animation⑥"></a>

  <a id="ref-for-playback-rate③⑧"></a>

  <a id="ref-for-animation-effect-after-phase②"></a>

  the [animation effect](#animation-effect) is [associated with an animation](#associated-with-an-animation) with a [playback rate](#playback-rate) \< 0 and the <a id="ref-for-animation-effect④⑦"></a>animation effect is in the [after phase](#animation-effect-after-phase), or

- <a id="ref-for-animation-effect④⑧"></a>

  <a id="ref-for-associated-with-an-animation⑦"></a>

  <a id="ref-for-play-state-idle⑤"></a>

  <a id="ref-for-animation-play-state⑦"></a>

  <a id="ref-for-timeline⑤①"></a>

  <a id="ref-for-monotonically-increasing-timeline⑥"></a>

  the [animation effect](#animation-effect) is [associated with an animation](#associated-with-an-animation) not in the [idle](#play-state-idle) [play state](#animation-play-state) with a non-null associated [timeline](#timeline) that is not [monotonically increasing](#monotonically-increasing-timeline).

<a id="ref-for-active-time③"></a>

<a id="ref-for-unresolved⑨⑤"></a>

An animation effect is <a id="in-effect"></a>in effect if its [active time](#active-time), as calculated according to the procedure in [§ 4.8.3.1 Calculating the active time](#calculating-the-active-time), is <em>not</em> [unresolved](#unresolved).

#### <a id="relevant-animations-section"></a>4.5.6. Relevant animations

<a id="ref-for-concept-animation③⑨"></a>

<a id="ref-for-animation-relevant"></a>

<a id="ref-for-animation-effect④⑨"></a>

<a id="ref-for-animation-associated-effect②③"></a>

We may define an [animation](#concept-animation) as being [relevant](#animation-relevant) based on the [animation effect](#animation-effect) [associated](#animation-associated-effect) with it.

<a id="ref-for-concept-animation④⓪"></a>

An [animation](#concept-animation) is <a id="animation-relevant"></a>relevant if:

- <a id="ref-for-animation-associated-effect②④"></a>

  <a id="ref-for-current①"></a>

  <a id="ref-for-in-effect②"></a>

  Its [associated effect](#animation-associated-effect) is [current](#current) <em>or</em> [in effect](#in-effect), <em>and</em>

- <a id="ref-for-replace-state"></a>

  <a id="ref-for-removed-replace-state"></a>

  Its [replace state](#replace-state) is <em>not</em> [removed](#removed-replace-state).

<a id="ref-for-concept-animation④①"></a>

<a id="ref-for-animation-effect⑤⓪"></a>

<a id="ref-for-keyframe-effect-effect-target"></a>

The <a id="relevant-animations"></a>relevant animations for an element or pseudo-element, <var>target</var>, is the set of all [animations](#concept-animation) that contain at least one [animation effect](#animation-effect) whose [effect target](#keyframe-effect-effect-target) is <var>target</var>.

<a id="ref-for-concept-document"></a>

<a id="ref-for-concept-shadow-root"></a>

<a id="ref-for-concept-animation④②"></a>

<a id="ref-for-animation-effect⑤①"></a>

<a id="ref-for-keyframe-effect-effect-target①"></a>

<a id="ref-for-concept-tree-inclusive-descendant"></a>

<a id="ref-for-concept-tree-descendant"></a>

<a id="ref-for-pseudo-element"></a>

The <a id="relevant-animations-for-a-subtree"></a>relevant animations for a subtree of an element, pseudo-element, [document](https://dom.spec.whatwg.org/#concept-document), or [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)—<var>target</var>—is the set of all [animations](#concept-animation) that contain at least one [animation effect](#animation-effect) whose [effect target](#keyframe-effect-effect-target) is an [inclusive descendant](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant) (or [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) if <var>target</var> is a <a id="ref-for-concept-document①"></a>document or <a id="ref-for-concept-shadow-root①"></a>shadow root) of <var>target</var> or a [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) of such a descendant.

### <a id="fill-behavior"></a>4.6. Fill behavior

<a id="ref-for-animation-effect⑤②"></a>

<a id="ref-for-in-play④"></a>

The effect of an [animation effect](#animation-effect) when it is not [in play](#in-play) is determined by its <a id="fill-mode"></a>fill mode.

<a id="ref-for-fill-mode②"></a>

The possible [fill modes](#fill-mode) are:

- none,

- forwards,

- backwards, and

- both.

<a id="ref-for-active-time④"></a>

The normative definition of these modes is incorporated in the calculation of the [active time](#active-time) in [§ 4.8.3.1 Calculating the active time](#calculating-the-active-time).

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> <a id="ref-for-fill-mode③"></a>
>
> <a id="ref-for-propdef-animation-fill-mode"></a>
>
> Authors are discouraged from using [fill modes](#fill-mode) to produce animations whose effect is applied indefinitely. <a id="ref-for-fill-mode④"></a>Fill modes were introduced in order to represent the [animation-fill-mode](https://www.w3.org/TR/css-animations-1/#propdef-animation-fill-mode) property defined by CSS animations [\[CSS-ANIMATIONS-1\]](#biblio-css-animations-1). However, they produce situations where animation state would be accumulated indefinitely necessitating the automatic removal of animations defined in [§ 5.5 Replacing animations](#replacing-animations). Furthermore, indefinitely filling animations can cause changes to specified style to be ineffective long after all animations have completed since the animation style takes precedence in the CSS cascade [\[css-cascade-3\]](#biblio-css-cascade-3).
>
> Where possible, authors should prefer to set the final state of the animation directly in specified style. This can be achieved by waiting for the animation to finish and then updating the style as illustrated below:
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-3cbeb1f1"></a>
> >
> > ```javascript
> > // In the first frame after the following animation finishes, the callback for
> > // the `finished` promise will run BEFORE style is updated and hence will NOT
> > // flicker.
> > elem.animate({ transform: 'translateY(100px)' }, 200).finished.then(() => {
> >   elem.style.transform = 'translateY(100px)';
> > });
> > ```
>
> Alternatively, the author may set the specified style at the start of the animation and then animate <em>from</em> the original value as illustrated below:
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-fb96320c"></a>
> >
> > ```javascript
> > elem.style.transform = 'translateY(100px)';
> > elem.animate({ transform: 'none', offset: 0 }, 200);
> > ```
>
> Complex effects involving layering many animations on top of one another may require temporary use of forwards fill modes to capture the final value of an animation before canceling it. For example:
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-8cd36716"></a>
> >
> > ```javascript
> > elem.addEventListener('click', async evt => {
> >   const animation = elem.animate(
> >     { transform: `translate(${evt.clientX}px, ${evt.clientY}px)` },
> >     { duration: 800, fill: 'forwards' }
> >   );
> >   await animation.finished;
> >   // commitStyles will record the style up to and including `animation` and
> >   // update elem’s specified style with the result.
> >   animation.commitStyles();
> >   animation.cancel();
> > });
> > ```
#### <a id="fill-modes"></a>4.6.1. Fill modes

<em>This section is non-normative</em>

<a id="ref-for-fill-mode⑤"></a>

The effect of each [fill mode](#fill-mode) is as follows:

none  
<a id="ref-for-in-play⑤"></a>

The animation effect has no effect when it is not [in play](#in-play).

forwards  
<a id="ref-for-animation-effect-after-phase③"></a>

<a id="ref-for-iteration-progress"></a>

<a id="ref-for-in-play⑥"></a>

When the animation effect is in the [after phase](#animation-effect-after-phase), the animation effect will produce the same [iteration progress](#iteration-progress) value as the last moment it is scheduled to be [in play](#in-play).

<a id="ref-for-in-play⑦"></a>

For all other times that the animation effect is not [in play](#in-play), it will have no effect.

backwards  
<a id="ref-for-animation-effect-before-phase③"></a>

<a id="ref-for-iteration-progress①"></a>

<a id="ref-for-in-play⑧"></a>

When the animation effect is in the [before phase](#animation-effect-before-phase), the animation effect will produce the same [iteration progress](#iteration-progress) value as the earliest moment that it is scheduled to be [in play](#in-play).

<a id="ref-for-in-play⑨"></a>

For all other times that the animation effect is not [in play](#in-play), it will have no effect.

both  
<a id="ref-for-animation-effect-before-phase④"></a>

When the animation effect is in its [before phase](#animation-effect-before-phase), backwards fill behavior is used.

<a id="ref-for-animation-effect-after-phase④"></a>

When the animation effect is in its [after phase](#animation-effect-after-phase), forwards fill behavior is used.

Some examples of the these fill modes are illustrated below.

![Examples of various fill modes and the states produced.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/animation-state-and-fill-behavior.svg)

<a id="ref-for-iteration-progress②"></a>

Examples of various fill modes and the states produced.  
(a) fill mode "none". The animation effect has no effect outside its active phase.  
(b) fill mode "forwards". After the active phase has finished, the [iteration progress](#iteration-progress) value continues to maintain a fill value.  
(c) fill mode "backwards". The animation effect produces a fill value until the start of the active phase.  
(d) fill mode "both". Both before and after the active phase the animation effect produces a fill value.

<a id="ref-for-active-interval①②"></a>

<a id="ref-for-active-time⑤"></a>

<a id="ref-for-unresolved⑨⑥"></a>

<a id="ref-for-animation-effect-active-phase④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: setting a fill mode has no bearing on the endpoints of the [active interval](#active-interval) or the boundaries between [phases](#animation-effect-phases-and-states). However, the fill mode <em>does</em> have an effect on various other properties of the timing model since the [active time](#active-time) of an animation effect is only defined (that is, not [unresolved](#unresolved)) inside the [active phase](#animation-effect-active-phase) <em>or</em> when a fill is applied.

### <a id="repeating"></a>4.7. Repeating

#### <a id="iteration-intervals"></a>4.7.1. Iteration intervals

<a id="ref-for-active-interval①③"></a>

It is possible to specify that an animation effect should repeat a fixed number of times or indefinitely. This repetition occurs <em>within</em> the [active interval](#active-interval). The span of time during which a single repetition takes place is called an <a id="iteration-interval"></a>iteration interval.

<a id="ref-for-active-interval①④"></a>

<a id="ref-for-iteration-interval"></a>

<a id="ref-for-current-iteration"></a>

Unlike the [active interval](#active-interval), an animation effect can have multiple [iteration intervals](#iteration-interval) although typically only the interval corresponding to the [current iteration](#current-iteration) is of interest.

<a id="ref-for-iteration-duration①"></a>

The length of a single iteration is called the <a id="iteration-duration"></a>iteration duration. The initial [iteration duration](#iteration-duration) of an animation effect is zero.

<em>This section is non-normative</em>

<a id="ref-for-iteration-duration②"></a>

<a id="ref-for-active-duration⑤"></a>

Comparing the [iteration duration](#iteration-duration) and the [active duration](#active-duration) we have:

Iteration duration  
The time taken for a single iteration of the animation effect to complete.

Active duration  
<a id="ref-for-iteration-duration③"></a>

The time taken for the entire animation effect to complete, including repetitions. This may be longer or shorter than the [iteration duration](#iteration-duration).

<a id="ref-for-iteration-duration④"></a>

<a id="ref-for-active-duration⑥"></a>

The relationship between the [iteration duration](#iteration-duration) and [active duration](#active-duration) is illustrated below.

![Comparison of the iteration duration and active time.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/iteration-intervals.svg)

<a id="ref-for-iteration-duration⑤"></a>

<a id="ref-for-active-duration⑦"></a>

<a id="ref-for-iteration-count"></a>

A comparison of the [iteration duration](#iteration-duration) and [active duration](#active-duration) of an animation effect with an [iteration count](#iteration-count) of 2.5. Note that the <a id="ref-for-iteration-duration⑥"></a>iteration duration for the final iteration does not change, it is simply cut-off by the <a id="ref-for-active-duration⑧"></a>active duration.

#### <a id="controlling-iteration"></a>4.7.2. Controlling iteration

<a id="ref-for-animation-effect⑤③"></a>

<a id="ref-for-iteration-count①"></a>

The number of times an [animation effect](#animation-effect) repeats is called its <a id="iteration-count"></a>iteration count. The [iteration count](#iteration-count) is a real number greater than or equal to zero. The <a id="ref-for-iteration-count②"></a>iteration count may also be positive infinity to represent that the <a id="ref-for-animation-effect⑤④"></a>animation effect repeats indefinitely.

<a id="ref-for-iteration-count③"></a>

<a id="ref-for-animation-effect⑤⑤"></a>

<a id="ref-for-iteration-start"></a>

In addition to the [iteration count](#iteration-count), [animation effects](#animation-effect) also have an <a id="iteration-start"></a>iteration start property which specifies an offset into the series of iterations at which the <a id="ref-for-animation-effect⑤⑥"></a>animation effect should begin. The [iteration start](#iteration-start) is a finite real number greater than or equal to zero.

The behavior of these parameters is defined in the calculations in [§ 4.8 Core animation effect calculations](#core-animation-effect-calculations).

<em>This section is non-normative</em>

<a id="ref-for-iteration-count④"></a>

<a id="ref-for-iteration-start①"></a>

The effect of the [iteration count](#iteration-count) and [iteration start](#iteration-start) parameters is illustrated below.

![The effect of the iteration count and iteration start parameters](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/iteration-count-and-start.svg)

<a id="ref-for-iteration-count⑤"></a>

<a id="ref-for-iteration-start②"></a>

<a id="ref-for-iteration-interval①"></a>

<a id="ref-for-animation-effect⑤⑦"></a>

The effect of the [iteration count](#iteration-count) and [iteration start](#iteration-start) parameters.  
In the first case the <a id="ref-for-iteration-count⑥"></a>iteration count is 2.5 resulting in the third iteration being cut-off half way through its [iteration interval](#iteration-interval).  
The second case is the same but with an <a id="ref-for-iteration-start③"></a>iteration start of 0.5. This causes the [animation effect](#animation-effect) to begin half way through the first iteration.

<a id="ref-for-iteration-count⑦"></a>

<a id="ref-for-iteration-start④"></a>

<a id="ref-for-active-duration⑨"></a>

Unlike the [iteration count](#iteration-count) parameter, the [iteration start](#iteration-start) parameter does not effect the length of the [active duration](#active-duration).

<a id="ref-for-iteration-start⑤"></a>

<a id="ref-for-animation-effect⑤⑧"></a>

<a id="ref-for-iteration-composite-operation"></a>

<a id="ref-for-iteration-composite-operation-accumulate"></a>

Note that values of [iteration start](#iteration-start) greater than or equal to one are generally not useful unless used in combination with an [animation effect](#animation-effect) that has an [iteration composite operation](https://drafts.csswg.org/web-animations-2/#iteration-composite-operation) of [accumulate](https://drafts.csswg.org/web-animations-2/#iteration-composite-operation-accumulate).

#### <a id="iteration-time-space"></a>4.7.3. Iteration time space

<em>This section is non-normative</em>

In Web Animations all times are relative to some point of reference. These different points of reference produce different <em>time&#xA;spaces</em>.

This can be compared to coordinate spaces as used in computer graphics. The zero time of a time space is analogous to the origin of a coordinate space.

We can describe animations that repeat as establishing a new time space each time the animation repeats: the <em>iteration time space</em>.

<em>Iteration time space</em> is a time space whose zero time is the beginning of an animation effect’s current iteration.

<a id="ref-for-active-time⑥"></a>

Within the Web Animations model we also refer to [active time](#active-time) which is a time relative to the beginning of the active interval. This time space, however, is internal to the model and not exposed in the programming interface or in markup.

These time spaces are illustrated below.

![A comparison of local time, active time, and iteration time.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/time-spaces.svg)

A comparison of local time, active time, and iteration time for an animation with a iteration duration of 1s and an iteration count of 2.5.

<a id="ref-for-active-time⑦"></a>

<a id="ref-for-iteration-progress③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While the time spaces themselves are not bounded, Web Animations defines [active time](#active-time) and the [iteration progress](#iteration-progress) such that they are clamped to a set range as shown in the diagram. For example, whilst a time of -1 second is a valid time in <em>active time space</em>, the procedure for calculating the <a id="ref-for-active-time⑧"></a>active time defined in [§ 4.8.3.1 Calculating the active time](#calculating-the-active-time) will never return a negative value.

<a id="ref-for-time-value④⓪"></a>

<a id="ref-for-document-default-document-timeline③"></a>

<a id="ref-for-document⑤"></a>

<a id="ref-for-current-global-object"></a>

In addition to these time spaces we can also refer to the <em>document time space</em> which is time space of the [time values](#time-value) of the [default document timeline](#document-default-document-timeline) of the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> of the [current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).

#### <a id="interval-timing"></a>4.7.4. Interval timing

<em>This section is non-normative</em>

When an animation effect repeats we must define the behavior at the iteration boundaries. For this, and indeed for all interval timing, Web Animations uses an endpoint-exclusive timing model. This means that whilst the begin time of an interval is included in the interval, the end time is not. In interval notation this can written `[begin, end)`. This model provides sensible behavior when intervals are repeated and sequenced since there is no overlap between the intervals.

<a id="ref-for-animation-associated-effect②⑤"></a>

<a id="ref-for-in-play①⓪"></a>

In the examples below, for the repeated effect, at local time 1s, the iteration time is 0. For the sequenced animations, at timeline time 1s, only animation B’s [associated effect](#animation-associated-effect) will be [in play](#in-play); there is no overlap.

![Illustration of end-point exclusive timing.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/endpoint-exclusive-timing.svg)

Illustration of end-point exclusive timing. For both repeated and sequenced animation effects there is no overlap at the boundaries between intervals.

An exception to this behavior is that when performing a [fill](#fill-behavior), if the fill begins at an interval endpoint, the endpoint is used. This behavior falls out of the algorithm given in [§ 4.8.3.3 Calculating the simple iteration progress](#calculating-the-simple-iteration-progress) and is illustrated below.

![Effect of iterations and fill on iteration time.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/endpoint-exclusive-timing-and-fill.svg)

<a id="ref-for-iteration-progress④"></a>

After one iteration, the [iteration progress](#iteration-progress) is 0, but after two iterations (and there onwards), the <a id="ref-for-iteration-progress⑤"></a>iteration progress is 1 due to the special behavior defined when an animation effect fills.

### <a id="core-animation-effect-calculations"></a>4.8. Core animation effect calculations

#### <a id="animation-effect-calculations-overview"></a>4.8.1. Overview

<em>This section is non-normative</em>

<a id="ref-for-local-time①①"></a>

<a id="ref-for-iteration-progress⑥"></a>

At the core of the Web Animations timing model is the process that takes a [local time](#local-time) value and converts it to an [iteration progress](#iteration-progress).

<a id="ref-for-active-interval①⑤"></a>

<a id="ref-for-active-duration①⓪"></a>

The first step in this process is to calculate the bounds of the [active interval](#active-interval) which is determined by the [active duration](#active-duration).

This process is illustrated below.

![Calculation of the active duration.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/active-duration-calculation.svg)

<a id="ref-for-active-duration①①"></a>

<a id="ref-for-iteration-duration⑦"></a>

<a id="ref-for-iteration-count⑧"></a>

Calculation of the [active duration](#active-duration) is based on multiplying the [iteration duration](#iteration-duration) by the [iteration count](#iteration-count).

<a id="ref-for-active-duration①②"></a>

The process for calculating the [active duration](#active-duration) is normatively defined in [§ 4.8.2 Calculating the active duration](#calculating-the-active-duration).

<a id="ref-for-active-duration①③"></a>

<a id="ref-for-animation-effect⑤⑨"></a>

<a id="ref-for-local-time①②"></a>

<a id="ref-for-transformed-progress"></a>

<a id="ref-for-iteration-progress⑦"></a>

Having established the [active duration](#active-duration), the process for transforming an [animation effect](#animation-effect)’s [local time](#local-time) into its [transformed progress](#transformed-progress) ([iteration progress](#iteration-progress)) is illustrated below.

![An overview of timing model calculations.](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/time-calculations.svg)

<a id="ref-for-local-time①③"></a>

<a id="ref-for-concept-animation④③"></a>

<a id="ref-for-active-time⑨"></a>

<a id="ref-for-start-delay①①"></a>

<a id="ref-for-iteration-duration⑧"></a>

<a id="ref-for-iteration-start⑥"></a>

<a id="ref-for-overall-progress"></a>

<a id="ref-for-simple-iteration-progress"></a>

<a id="ref-for-directed-progress"></a>

<a id="ref-for-playback-direction"></a>

<a id="ref-for-transformed-progress①"></a>

An overview of timing model calculations.  
(1) The [local time](#local-time) is determined from the associated [animation](#concept-animation).  
(2) The <a id="ref-for-local-time①④"></a>local time is converted into an [active time](#active-time) by incorporating the [start delay](#start-delay).  
(3) The <a id="ref-for-active-time①⓪"></a>active time is divided by the [iteration duration](#iteration-duration) incorporating also the [iteration start](#iteration-start) property to produce the [overall progress](#overall-progress).  
(4) The <a id="ref-for-overall-progress①"></a>overall progress time is then converted to an offset within a single iteration: the [simple iteration progress](#simple-iteration-progress).  
(5) The <a id="ref-for-simple-iteration-progress①"></a>simple iteration progress is converted into a [directed progress](#directed-progress) by incorporating the [playback direction](#playback-direction).  
(6) Finally, a timing function is applied to the <a id="ref-for-directed-progress①"></a>directed progress to produce the [transformed progress](#transformed-progress).

<a id="ref-for-local-time①⑤"></a>

The first step, calculating the [local time](#local-time) is described in [§ 4.5.4 Local time](#local-time-section). Steps 2 to 4 in the diagram are described in the following sections. Steps 5 and 6 are described in [§ 4.9.1 Calculating the directed progress](#calculating-the-directed-progress) and [§ 4.10.1 Calculating the transformed progress](#calculating-the-transformed-progress) respectively.

#### <a id="calculating-the-active-duration"></a>4.8.2. Calculating the active duration

<a id="ref-for-active-duration①④"></a>

The [active duration](#active-duration) is calculated as follows:

> <a id="active-duration"></a>active duration = <code><a href="#iteration-duration">iteration duration</a> × <a href="#iteration-count">iteration count</a></code>
>
> <a id="ref-for-iteration-duration①⓪"></a>
>
> <a id="ref-for-iteration-count①⓪"></a>
>
> <a id="ref-for-active-duration①⑤"></a>
>
> If either the [iteration duration](#iteration-duration) or [iteration count](#iteration-count) are zero, the [active duration](#active-duration) is zero.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > This clarification is needed since the result of infinity multiplied by zero is undefined according to IEEE 754-2008.

#### <a id="transforming-the-local-time"></a>4.8.3. Transforming the local time

##### <a id="calculating-the-active-time"></a>4.8.3.1. Calculating the active time

<a id="ref-for-local-time①⑥"></a>

<a id="ref-for-start-delay①②"></a>

<a id="ref-for-animation-effect⑥⓪"></a>

<a id="ref-for-fill-mode⑥"></a>

The <a id="active-time"></a>active time is based on the [local time](#local-time) and [start delay](#start-delay). However, it is only defined when the [animation effect](#animation-effect) should produce an output and hence depends on its [fill mode](#fill-mode) and phase as follows,

<a id="ref-for-animation-effect-before-phase⑤"></a>

If the animation effect is in the [before phase](#animation-effect-before-phase),

The result depends on the first matching condition from the following,

<a id="ref-for-fill-mode⑦"></a>

If the [fill mode](#fill-mode) is backwards or both,

<a id="ref-for-local-time①⑦"></a>

<a id="ref-for-start-delay①③"></a>

Return the result of evaluating <code>max(<a href="#local-time">local time</a> - <a href="#start-delay">start delay</a>, 0)</code>.

Otherwise,

<a id="ref-for-unresolved⑨⑦"></a>

<a id="ref-for-time-value④①"></a>

Return an [unresolved](#unresolved) [time value](#time-value).

<a id="ref-for-animation-effect-active-phase⑤"></a>

If the animation effect is in the [active phase](#animation-effect-active-phase),

<a id="ref-for-local-time①⑧"></a>

<a id="ref-for-start-delay①④"></a>

Return the result of evaluating <code><a href="#local-time">local time</a> - <a href="#start-delay">start delay</a></code>.

<a id="ref-for-animation-effect-after-phase⑤"></a>

If the animation effect is in the [after phase](#animation-effect-after-phase),

The result depends on the first matching condition from the following,

<a id="ref-for-fill-mode⑧"></a>

If the [fill mode](#fill-mode) is forwards or both,

<a id="ref-for-local-time①⑨"></a>

<a id="ref-for-start-delay①⑤"></a>

<a id="ref-for-active-duration①⑥"></a>

Return the result of evaluating <code>max(min(<a href="#local-time">local time</a> - <a href="#start-delay">start delay</a>, <a href="#active-duration">active duration</a>),&#xA;          0)</code>.

Otherwise,

<a id="ref-for-unresolved⑨⑧"></a>

<a id="ref-for-time-value④②"></a>

Return an [unresolved](#unresolved) [time value](#time-value).

<a id="ref-for-unresolved⑨⑨"></a>

<a id="ref-for-local-time②⓪"></a>

Otherwise (the [local time](#local-time) is [unresolved](#unresolved)),

<a id="ref-for-unresolved①⓪⓪"></a>

<a id="ref-for-time-value④③"></a>

Return an [unresolved](#unresolved) [time value](#time-value).

##### <a id="calculating-the-overall-progress"></a>4.8.3.2. Calculating the overall progress

The <a id="overall-progress"></a>overall progress describes the number of iterations that have completed (including partial iterations) and is defined as follows:

1.  <a id="ref-for-active-time①①"></a>

    <a id="ref-for-unresolved①⓪①"></a>

    If the [active time](#active-time) is [unresolved](#unresolved), return <a id="ref-for-unresolved①⓪②"></a>unresolved.

2.  Calculate an initial value for <var>overall progress</var> based on the first matching condition from below,

    <a id="ref-for-iteration-duration①①"></a>

    If the [iteration duration](#iteration-duration) is zero,
    <a id="ref-for-animation-effect-before-phase⑥"></a>

    <a id="ref-for-iteration-count①①"></a>

    If the animation effect is in the [before phase](#animation-effect-before-phase), let <var>overall progress</var> be zero, otherwise, let it be equal to the [iteration count](#iteration-count).

    Otherwise,
    <a id="ref-for-active-time①②"></a>

    <a id="ref-for-iteration-duration①②"></a>

    Let <var>overall progress</var> be the result of calculating <code><a href="#active-time">active time</a> / <a href="#iteration-duration">iteration duration</a></code>.

3.  <a id="ref-for-iteration-start⑦"></a>

    Return the result of calculating <code><var>overall&#xA;progress</var> + <a href="#iteration-start">iteration start</a></code>.

##### <a id="calculating-the-simple-iteration-progress"></a>4.8.3.3. Calculating the simple iteration progress

<a id="ref-for-playback-direction①"></a>

<a id="ref-for-easing-function"></a>

The <a id="simple-iteration-progress"></a>simple iteration progress is a fraction of the progress through the current iteration that ignores transformations to the time introduced by the [playback direction](#playback-direction) or [timing functions](https://www.w3.org/TR/css-easing-1/#easing-function) applied to the effect, and is calculated as follows:

1.  <a id="ref-for-overall-progress②"></a>

    <a id="ref-for-unresolved①⓪③"></a>

    If the [overall progress](#overall-progress) is [unresolved](#unresolved), return <a id="ref-for-unresolved①⓪④"></a>unresolved.

2.  <a id="ref-for-overall-progress③"></a>

    <a id="ref-for-iteration-start⑧"></a>

    <a id="ref-for-overall-progress④"></a>

    If [overall progress](#overall-progress) is infinity, let the <var>simple iteration&#xA;progress</var> be <code><a href="#iteration-start">iteration start</a> % 1.0</code>, otherwise, let the <var>simple iteration progress</var> be <code><a href="#overall-progress">overall progress</a> % 1.0</code>.

3.  If <em>all</em> of the following conditions are true,

    - the <var>simple iteration progress</var> calculated above is zero, <em>and</em>

    - <a id="ref-for-animation-effect-active-phase⑥"></a>

      <a id="ref-for-animation-effect-after-phase⑥"></a>

      the animation effect is in the [active phase](#animation-effect-active-phase) <em>or</em> the [after phase](#animation-effect-after-phase), <em>and</em>

    - <a id="ref-for-active-time①③"></a>

      <a id="ref-for-active-duration①⑦"></a>

      the [active time](#active-time) is equal to the [active duration](#active-duration), <em>and</em>

    - <a id="ref-for-iteration-count①②"></a>

      the [iteration count](#iteration-count) is <em>not</em> equal to zero.

    let the <var>simple iteration progress</var> be 1.0.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > The above step implements the behavior that when an animation’s active interval ends precisely at the end of an iteration, it fills by holding the endpoint of the final iteration rather than the start of the next iteration.
    >
    > <a id="ref-for-iteration-count①③"></a>
    >
    > The final condition prevents this from applying when we never played any iterations of the animation to begin with because the [iteration count](#iteration-count) was zero.

4.  Return <var>simple iteration progress</var>.

#### <a id="calculating-the-current-iteration"></a>4.8.4. Calculating the current iteration

The <a id="current-iteration"></a>current iteration can be calculated using the following steps:

1.  <a id="ref-for-active-time①④"></a>

    <a id="ref-for-unresolved①⓪⑤"></a>

    If the [active time](#active-time) is [unresolved](#unresolved), return <a id="ref-for-unresolved①⓪⑥"></a>unresolved.

2.  <a id="ref-for-animation-effect-after-phase⑦"></a>

    <a id="ref-for-iteration-count①④"></a>

    If the animation effect is in the [after phase](#animation-effect-after-phase) <em>and</em> the [iteration count](#iteration-count) is infinity, return infinity.

3.  <a id="ref-for-simple-iteration-progress②"></a>

    <a id="ref-for-overall-progress⑤"></a>

    If the [simple iteration progress](#simple-iteration-progress) is 1.0, return <code>floor(<a href="#overall-progress">overall progress</a>) - 1</code>.

4.  <a id="ref-for-overall-progress⑥"></a>

    Otherwise, return <code>floor(<a href="#overall-progress">overall progress</a>)</code>.

### <a id="direction-control"></a>4.9. Direction control

<a id="ref-for-animation-effect⑥①"></a>

[Animation effects](#animation-effect) may also be configured to run iterations in alternative directions using direction control. For this purpose, <a id="ref-for-animation-effect⑥②"></a>animation effects have a <a id="playback-direction"></a>playback direction parameter which takes one of the following values:

- normal,

- reverse,

- alternate, or

- alternate-reverse.

<a id="ref-for-directed-progress②"></a>

The semantics of these values are incorporated into the calculation of the [directed progress](#directed-progress) which follows.

<em>This section is non-normative</em>

A non-normative definition of these values is as follows:

normal  
All iterations are played as specified.

reverse  
All iterations are played in the reverse direction from the way they are specified.

alternate  
Even iterations are played as specified, odd iterations are played in the reverse direction from the way they are specified.

alternate-reverse  
Even iterations are played in the reverse direction from the way they are specified, odd iterations are played as specified.

#### <a id="calculating-the-directed-progress"></a>4.9.1. Calculating the directed progress

<a id="ref-for-simple-iteration-progress③"></a>

The <a id="directed-progress"></a>directed progress is calculated from the [simple iteration progress](#simple-iteration-progress) using the following steps:

1.  <a id="ref-for-simple-iteration-progress④"></a>

    <a id="ref-for-unresolved①⓪⑦"></a>

    If the [simple iteration progress](#simple-iteration-progress) is [unresolved](#unresolved), return <a id="ref-for-unresolved①⓪⑧"></a>unresolved.

2.  Calculate the <var>current direction</var> using the first matching condition from the following list:

    <a id="ref-for-playback-direction②"></a>

    If [playback direction](#playback-direction) is `normal`,
    Let the <var>current direction</var> be forwards.

    <a id="ref-for-playback-direction③"></a>

    If [playback direction](#playback-direction) is `reverse`,
    Let the <var>current direction</var> be reverse.

    Otherwise,
    1.  <a id="ref-for-current-iteration①"></a>

        Let <var>d</var> be the [current iteration](#current-iteration).

    2.  <a id="ref-for-playback-direction④"></a>

        If [playback direction](#playback-direction) is `alternate-reverse` increment <var>d</var> by 1.

    3.  If <code><var>d</var> % 2 == 0</code>, let the <var>current direction</var> be forwards, otherwise let the <var>current direction</var> be reverse. If <var>d</var> is infinity, let the <var>current direction</var> be forwards.

3.  <a id="ref-for-simple-iteration-progress⑤"></a>

    If the <var>current direction</var> is forwards then return the [simple iteration progress](#simple-iteration-progress).

    <a id="ref-for-simple-iteration-progress⑥"></a>

    Otherwise, return <code>1.0 - <a href="#simple-iteration-progress">simple iteration progress</a></code>.

### <a id="time-transformations"></a>4.10. Time transformations

<a id="ref-for-animation-effect⑥③"></a>

<a id="ref-for-easing-function①"></a>

It is often desirable to control the rate at which an [animation effect](#animation-effect) progresses. For example, easing the rate of animation can create a sense of momentum and produce a more natural effect. The CSS Easing Functions Module [\[CSS-EASING-1\]](#biblio-css-easing-1) defines [timing functions](https://www.w3.org/TR/css-easing-1/#easing-function) for this purpose.

<a id="ref-for-animation-effect⑥④"></a>

<a id="ref-for-easing-function②"></a>

<a id="ref-for-linear-easing-function"></a>

[Animation effects](#animation-effect) have one [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) associated with them. The default <a id="ref-for-easing-function③"></a>timing function is the [linear timing function](https://www.w3.org/TR/css-easing-1/#linear-easing-function).

#### <a id="calculating-the-transformed-progress"></a>4.10.1. Calculating the transformed progress

<a id="ref-for-directed-progress③"></a>

The <a id="transformed-progress"></a>transformed progress is calculated from the [directed progress](#directed-progress) using the following steps:

1.  <a id="ref-for-directed-progress④"></a>

    <a id="ref-for-unresolved①⓪⑨"></a>

    If the [directed progress](#directed-progress) is [unresolved](#unresolved), return <a id="ref-for-unresolved①①⓪"></a>unresolved.

2.  Calculate the value of the <var>before flag</var> as follows:

    1.  Determine the <var>current direction</var> using the procedure defined in [§ 4.9.1 Calculating the directed progress](#calculating-the-directed-progress).

    2.  If the <var>current direction</var> is forwards, let <var>going forwards</var> be true, otherwise it is false.

    3.  <a id="ref-for-animation-effect-before-phase⑦"></a>

        <a id="ref-for-animation-effect-after-phase⑧"></a>

        The <var>before flag</var> is set if the animation effect is in the [before phase](#animation-effect-before-phase) and <var>going forwards</var> is true; or if the animation effect is in the [after phase](#animation-effect-after-phase) and <var>going&#xA;forwards</var> is false.

3.  <a id="ref-for-animation-effect⑥⑤"></a>

    <a id="ref-for-easing-function④"></a>

    <a id="ref-for-directed-progress⑤"></a>

    <a id="ref-for-input-progress-value"></a>

    <a id="ref-for-before-flag"></a>

    Return the result of evaluating the [animation effect](#animation-effect)’s [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) passing [directed progress](#directed-progress) as the [input progress value](https://www.w3.org/TR/css-easing-1/#input-progress-value) and <var>before flag</var> as the [before flag](https://www.w3.org/TR/css-easing-1/#before-flag).

### <a id="the-iteration-progress"></a>4.11. The iteration progress

<a id="ref-for-animation-effect⑥⑥"></a>

<a id="ref-for-transformed-progress②"></a>

The <a id="iteration-progress"></a>iteration progress of an [animation effect](#animation-effect) is simply its [transformed progress](#transformed-progress).

## <a id="animation-model"></a>5. Animation model

<em>This section is non-normative</em>

<a id="ref-for-animation-effect⑥⑦"></a>

<a id="ref-for-iteration-progress⑧"></a>

<a id="ref-for-current-iteration②"></a>

For some kinds of [animation effects](#animation-effect), the Web Animations <em>animation&#xA;model</em> takes the [iteration progress](#iteration-progress) and [current iteration](#current-iteration) values produced by the <em>timing model</em> and uses them to calculate a corresponding output.

<a id="ref-for-effect-stack"></a>

The output of each such animation effect is then combined with that of others using an [effect stack](#effect-stack) before being applied to the target properties (see [§ 5.4 Combining effects](#combining-effects)).

### <a id="introduction-to-the-animation-model"></a>5.1. Introduction

<a id="ref-for-animation-effect⑥⑧"></a>

An [animation effect](#animation-effect) has zero or more associated properties that it affects in response to changes to its timing output. These properties are referred to as the effect’s <a id="target-property"></a>target properties.

<a id="ref-for-iteration-progress⑨"></a>

<a id="ref-for-current-iteration③"></a>

<a id="ref-for-underlying-value"></a>

<a id="ref-for-animation-effect⑥⑨"></a>

<a id="ref-for-concept-animatable"></a>

<a id="ref-for-target-property"></a>

<a id="ref-for-animation-type"></a>

Given an [iteration progress](#iteration-progress), a [current iteration](#current-iteration), and an [underlying value](#underlying-value), an [animation effect](#animation-effect) produces an <a id="effect-value"></a>effect value for each [animatable](#concept-animatable) [target property](#target-property) by applying the procedures from the [animation type](#animation-type) appropriate to the property.

### <a id="animating-properties"></a>5.2. Animating properties

Unless otherwise specified, all CSS properties are <a id="concept-animatable"></a>animatable. How property values combine is defined by the <a id="animation-type"></a>Animation type line in each property’s property definition table:

<a id="not-animatable"></a>not animatable  
The property is not animatable. It is not processed when listed in an animation keyframe, and is not affected by transitions.

<a id="ref-for-not-animatable"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Properties are typically excluded from animation because animating them would create excessive complications. For example, properties defining animation parameters are [not animatable](#not-animatable) since doing so would create complex recursive behavior.

<a id="ref-for-animation-effect⑦⓪"></a>

<a id="ref-for-not-animatable①"></a>

<a id="ref-for-concept-animation④④"></a>

<a id="ref-for-current-finished-promise①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An [animation effect](#animation-effect) that targets only properties that are [not animatable](#not-animatable) will still exhibit the usual behavior for an <a id="ref-for-animation-effect⑦①"></a>animation effect such as firing events and delaying the fulfillment of the [animation](#concept-animation)’s [current finished promise](#current-finished-promise).

<a id="discrete"></a>discrete  
<a id="ref-for-interpolation"></a>

<a id="ref-for-not-additive"></a>

The property’s values cannot be meaningfully combined, thus it is [not additive](https://www.w3.org/TR/css-values-4/#not-additive) and [interpolation](https://www.w3.org/TR/css-values-4/#interpolation) swaps from <var>V<sub>a</sub></var> to <var>V<sub>b</sub></var> at 50% (<var>p=0.5</var>), i.e.

<strong>Mathematical expression 1</strong>

TeX transcription (renderer-independent source notation):

``` language-tex
V_{result} = \begin{cases}
V_{start} & {\text{if~}p < 0.5} \\
V_{end} & {\text{if~}p \geq 0.5}
\end{cases}
```

Source mathematical tokens: V result = V start if p \< 0.5 V end if p ≥ 0.5

<a id="by-computed-value"></a>by computed value  
<a id="ref-for-discrete"></a>

<a id="ref-for-computed-value"></a>

Corresponding individual components of the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) are combined (interpolated, added, or accumulated) using the indicated procedure for that value type (see [CSS Values 4 § 3 Combining Values: Interpolation, Addition, and Accumulation](https://www.w3.org/TR/css-values-4/#combining-values)). If the number of components or the types of corresponding components do not match, or if any component value uses [discrete](#discrete) animation and the two corresponding values do not match, then the property values combine as <a id="ref-for-discrete①"></a>discrete.

<a id="repeatable-list"></a>repeatable list  
<a id="ref-for-discrete②"></a>

<a id="ref-for-by-computed-value"></a>

Same as [by computed value](#by-computed-value) except that if the two lists have differing numbers of items, they are first repeated to the least common multiple number of items. Each item is then combined <a id="ref-for-by-computed-value①"></a>by computed value. If a pair of values cannot be combined or if any component value uses [discrete](#discrete) animation, then the property values combine as <a id="ref-for-discrete③"></a>discrete.

<a id="ref-for-propdef-background-origin"></a>

<a id="ref-for-propdef-background-image"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The repeatable list concept ensures that a list that is conceptually repeated to a certain length (as [background-origin](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-origin) is repeated to the length of the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) list) or repeated infinitely will smoothly transition between any values, and so that the computed value will properly represent the result (and potentially be inherited correctly).

(See prose)  
Some properties have specific interpolation behavior not covered by the above cases; in this case the animation behavior will be specified explicitly for that property.

<a id="ref-for-animation-type①"></a>

The [animation type](#animation-type) of properties that do not yet include an <a id="ref-for-animation-type②"></a>Animation type line in their property definition, is defined in [Appendix A: Animation types of existing properties](#animation-types).

#### <a id="custom-properties"></a>5.2.1. Custom Properties

<a id="ref-for-custom-property"></a>

<a id="ref-for-dom-css-registerproperty"></a>

<a id="ref-for-current-global-object①"></a>

<a id="ref-for-animation-type③"></a>

<a id="ref-for-by-computed-value②"></a>

<a id="ref-for-syntax-definition"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-universal-syntax-definition"></a>

<a id="ref-for-discrete④"></a>

For [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) registered using the <code><a href="https://www.w3.org/TR/css-properties-values-api-1/#dom-css-registerproperty">registerProperty()</a></code> method for the [current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object), the [animation type](#animation-type) is [by computed value](#by-computed-value), derived from the type used in the property’s [syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#syntax-definition). Where there is no [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) type that corresponds to the property’s specified syntax (e.g. when the syntax is the [universal syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#universal-syntax-definition)) or when the <a id="ref-for-custom-property①"></a>custom property is not registered, the <a id="ref-for-animation-type④"></a>animation type is [discrete](#discrete).

### <a id="keyframe-effects"></a>5.3. Keyframe effects

<a id="ref-for-animation-effect⑦②"></a>

<a id="ref-for-pseudo-element①"></a>

<a id="keyframe-effect"></a>Keyframe effects are a kind of [animation effect](#animation-effect) that use the output of the timing model to update CSS properties of an element or [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) (such as `::before` or `::after` [\[select\]](#biblio-select)) referred to as the <a id="keyframe-effect-effect-target"></a>effect target.

<a id="ref-for-keyframe-effect-effect-target②"></a>

<a id="ref-for-element"></a>

<a id="ref-for-pseudo-element②"></a>

<a id="ref-for-element①"></a>

<a id="ref-for-effect-target-target-element"></a>

<a id="ref-for-effect-target-target-pseudo-selector"></a>

<a id="ref-for-originating-element"></a>

The [effect target](#keyframe-effect-effect-target) is comprised of an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> known as the <a id="effect-target-target-element"></a>target element and a [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) selector known as the <a id="effect-target-target-pseudo-selector"></a>target pseudo-selector. If the <a id="ref-for-keyframe-effect-effect-target③"></a>effect target is an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>, the [target element](#effect-target-target-element) is that element and the [target pseudo-selector](#effect-target-target-pseudo-selector) is `null`. If the <a id="ref-for-keyframe-effect-effect-target④"></a>effect target is a <a id="ref-for-pseudo-element③"></a>pseudo-element, the <a id="ref-for-effect-target-target-element①"></a>target element is its [originating element](https://www.w3.org/TR/selectors-4/#originating-element) and the <a id="ref-for-effect-target-target-pseudo-selector①"></a>target pseudo-selector is as required to specify that particular <a id="ref-for-pseudo-element④"></a>pseudo-element.

<a id="ref-for-keyframe-effect-effect-target⑤"></a>

<a id="ref-for-selectordef-part"></a>

Note that not all [effect targets](#keyframe-effect-effect-target) specified in this manner (such as [::part()](https://www.w3.org/TR/css-shadow-parts-1/#selectordef-part) pseudo-elements and unsupported pseudo-elements) have computed property values defined.

#### <a id="keyframes-section"></a>5.3.1. Keyframes

<a id="ref-for-effect-value"></a>

<a id="ref-for-keyframe-effect②"></a>

The [effect values](#effect-value) for a [keyframe effect](#keyframe-effect) are calculated by interpolating between a series of property values positioned at fractional offsets. Each set of property values indexed by an offset is called a <a id="keyframe"></a>keyframe.

<a id="ref-for-keyframe"></a>

<a id="ref-for-keyframe-effect③"></a>

<a id="ref-for-keyframe-offset"></a>

The <a id="keyframe-offset"></a>offset of a keyframe is a value in the range \[0, 1\] or the special value null. The list of [keyframes](#keyframe) for a [keyframe effect](#keyframe-effect) must be <a id="loosely-sorted-by-offset"></a>loosely sorted by offset which means that for each <a id="ref-for-keyframe①"></a>keyframe in the list that has a [keyframe offset](#keyframe-offset) that is not null, the offset is greater than or equal to the offset of the previous <a id="ref-for-keyframe②"></a>keyframe in the list with a <a id="ref-for-keyframe-offset①"></a>keyframe offset that is not null, if any.

<a id="ref-for-keyframe③"></a>

The behavior when [keyframes](#keyframe) overlap or have unsupported values is defined in [§ 5.3.4 The effect value of a keyframe effect](#the-effect-value-of-a-keyframe-animation-effect).

<a id="ref-for-easing-function⑤"></a>

Each keyframe also has a [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) associated with it that is applied to the period of time between the keyframe on which it is specified and the <em>next</em> keyframe in the list. The <a id="ref-for-easing-function⑥"></a>timing function specified on the last keyframe in the list is never applied.

<a id="ref-for-keyframe④"></a>

<a id="ref-for-composite-operation"></a>

<a id="ref-for-keyframe-effect④"></a>

<a id="ref-for-keyframe-specific-composite-operation"></a>

Each [keyframe](#keyframe) may have a <a id="keyframe-specific-composite-operation"></a>keyframe-specific composite operation that, if set, is applied to all values specified in that <a id="ref-for-keyframe⑤"></a>keyframe. The possible operations and their meanings are identical to those defined for the [composite operation](#composite-operation) associated with the [keyframe effect](#keyframe-effect) as a whole in [§ 5.4.4 Effect composition](#effect-composition). If the [keyframe-specific composite operation](#keyframe-specific-composite-operation) for a <a id="ref-for-keyframe⑥"></a>keyframe is not set, the <a id="ref-for-composite-operation①"></a>composite operation specified for the <a id="ref-for-keyframe-effect⑤"></a>keyframe effect as a whole is used for values specified in that keyframe.

#### <a id="computing-property-values"></a>5.3.2. Computing property values

<a id="ref-for-element②"></a>

<a id="ref-for-computed-value②"></a>

To <a id="compute-a-property-value"></a>compute a property value given a property <var>property</var>, a value <var>value</var>, and an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> <var>element</var>: resolve <var>value</var> according to the "Computed Value" line of the <var>property</var>’s definition table, using the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of <var>element</var> as the context for resolving dependencies, and return the result.

<a id="ref-for-computed-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) on <var>element</var> are not affected by this algorithm.

<a id="ref-for-compute-a-property-value"></a>

<a id="ref-for-computed-value④"></a>

This algorithm implies that property values specified in keyframes can establish order dependencies. When [computing a property value](#compute-a-property-value), the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of dependencies held by <var>value</var> must be calculated <em>first</em>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6f975f2f"></a>
>
> ```javascript
> var animation = elem.animate([{ fontSize: '10px', width: '10em' },
>                               { fontSize: '20px', width: '20em' }], 1000);
> animation.currentTime = 500;
> console.log(getComputedStyle(elem).fontSize); // Should be 15px
> console.log(getComputedStyle(elem).width); // Should be 225px
> ```
>
> <a id="ref-for-compute-a-property-value①"></a>
>
> <a id="ref-for-computed-value⑤"></a>
>
> <a id="ref-for-propdef-font-size"></a>
>
> <a id="ref-for-effect-target-target-element②"></a>
>
> <a id="ref-for-effect-value①"></a>
>
> In this example, in order to [compute a property value](#compute-a-property-value) for `10em`, we must know the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the [target element](#effect-target-target-element), which in turn is determined by the [effect value](#effect-value) for <a id="ref-for-propdef-font-size①"></a>font-size, which in turn requires that we compute property values for <a id="ref-for-propdef-font-size②"></a>font-size. Hence, computing property values are subject to ordering constraints.

#### <a id="calculating-computed-keyframes"></a>5.3.3. Calculating computed keyframes

<a id="ref-for-effect-value②"></a>

<a id="ref-for-keyframe-effect⑥"></a>

<a id="ref-for-keyframe⑦"></a>

<a id="ref-for-keyframe-offset②"></a>

Before calculating the [effect value](#effect-value) of a [keyframe effect](#keyframe-effect), the property values on its [keyframes](#keyframe) are [computed](#computing-property-values), and the offset to use for any keyframes with a null [keyframe offset](#keyframe-offset) is computed. The result of resolving these values is a set of <a id="computed-keyframes"></a>computed keyframes.

<a id="ref-for-keyframe-offset③"></a>

<a id="ref-for-keyframe⑧"></a>

The calculated [keyframe offsets](#keyframe-offset) of a set of [keyframe](#keyframe) that includes suitable values for each null <a id="ref-for-keyframe-offset④"></a>keyframe offset are referred to as the <a id="computed-keyframe-offset"></a>computed keyframe offsets.

<a id="ref-for-computed-keyframe-offset"></a>

<a id="ref-for-keyframe⑨"></a>

To produce [computed keyframe offsets](#computed-keyframe-offset), we define a procedure to <a id="compute-missing-keyframe-offsets"></a>compute missing keyframe offsets that takes a sequence of [keyframes](#keyframe), <var>keyframes</var>, and has the following steps:

1.  <a id="ref-for-keyframe①⓪"></a>

    <a id="ref-for-computed-keyframe-offset①"></a>

    <a id="ref-for-keyframe-offset⑤"></a>

    For each [keyframe](#keyframe), in <var>keyframes</var>, let the [computed keyframe offset](#computed-keyframe-offset) of the <a id="ref-for-keyframe①①"></a>keyframe be equal to its [keyframe offset](#keyframe-offset) value.

2.  <a id="ref-for-keyframe①②"></a>

    <a id="ref-for-computed-keyframe-offset②"></a>

    If <var>keyframes</var> contains more than one [keyframe](#keyframe) and the [computed keyframe offset](#computed-keyframe-offset) of the first <a id="ref-for-keyframe①③"></a>keyframe in <var>keyframes</var> is null, set the <a id="ref-for-computed-keyframe-offset③"></a>computed keyframe offset of the first <a id="ref-for-keyframe①④"></a>keyframe to 0.

3.  <a id="ref-for-computed-keyframe-offset④"></a>

    <a id="ref-for-keyframe①⑤"></a>

    If the [computed keyframe offset](#computed-keyframe-offset) of the last [keyframe](#keyframe) in <var>keyframes</var> is null, set its <a id="ref-for-computed-keyframe-offset⑤"></a>computed keyframe offset to 1.

4.  <a id="ref-for-keyframe①⑥"></a>

    For each pair of [keyframes](#keyframe) <var>A</var> and <var>B</var> where:

    - <var>A</var> appears before <var>B</var> in <var>keyframes</var>, and

    - <a id="ref-for-computed-keyframe-offset⑥"></a>

      <var>A</var> and <var>B</var> have a [computed keyframe offset](#computed-keyframe-offset) that is not null, and

    - <a id="ref-for-keyframe①⑦"></a>

      <a id="ref-for-computed-keyframe-offset⑦"></a>

      all [keyframes](#keyframe) between <var>A</var> and <var>B</var> have a null [computed keyframe offset](#computed-keyframe-offset),

    <a id="ref-for-computed-keyframe-offset⑧"></a>

    <a id="ref-for-keyframe①⑧"></a>

    calculate the [computed keyframe offset](#computed-keyframe-offset) of each [keyframe](#keyframe) between <var>A</var> and <var>B</var> as follows:

    1.  <a id="ref-for-computed-keyframe-offset⑨"></a>

        <a id="ref-for-keyframe①⑨"></a>

        Let <a id="offsetk"></a>offset<sub><var>k</var></sub> be the [computed keyframe offset](#computed-keyframe-offset) of a [keyframe](#keyframe) <var>k</var>.

    2.  Let <var>n</var> be the number of keyframes <em>between</em> and including <var>A</var> and <var>B</var> minus 1.

    3.  Let <var>index</var> refer to the position of <var>keyframe</var> in the sequence of keyframes between <var>A</var> and <var>B</var> such that the first keyframe after <var>A</var> has an <var>index</var> of 1.

    4.  <a id="ref-for-computed-keyframe-offset①⓪"></a>

        <a id="ref-for-offsetk"></a>

        Set the [computed keyframe offset](#computed-keyframe-offset) of <var>keyframe</var> to [offset](#offsetk)<sub><var>A</var></sub> + (<a id="ref-for-offsetk①"></a>offset<sub><var>B</var></sub> − <a id="ref-for-offsetk②"></a>offset<sub><var>A</var></sub>) × <var>index</var> / <var>n</var>.

<a id="ref-for-computed-keyframes"></a>

<a id="ref-for-keyframe-effect⑦"></a>

<a id="ref-for-keyframe-effect-effect-target⑥"></a>

[Computed keyframes](#computed-keyframes) are produced using the following procedure. Note that this procedure is only performed on a [keyframe effect](#keyframe-effect) having an [effect target](#keyframe-effect-effect-target) for which computed property values can be calculated.

1.  <a id="ref-for-keyframe②⓪"></a>

    Let <var>computed keyframes</var> be an empty list of [keyframes](#keyframe).

2.  <a id="ref-for-keyframe②①"></a>

    <a id="ref-for-keyframe-effect⑧"></a>

    For each <var>keyframe</var> in the list of [keyframes](#keyframe) specified on this [keyframe effect](#keyframe-effect), perform the following steps:

    1.  <a id="ref-for-keyframe②②"></a>

        Add a new empty [keyframe](#keyframe), <var>computed keyframe</var>, to <var>computed keyframes</var>.

    2.  For each property specified in <var>keyframe</var>:

        - <a id="ref-for-compute-a-property-value②"></a>

          <a id="ref-for-effect-target-target-element③"></a>

          [Compute a property value](#compute-a-property-value) using the value specified on <var>keyframe</var> as the value, and the [target element](#effect-target-target-element) as the element; then add the property and resulting value to <var>computed keyframe</var>.

        - For shorthand properties, add the equivalent longhand properties.

        - <a id="ref-for-logical-to-physical"></a>

          <a id="ref-for-propdef-writing-mode"></a>

          <a id="ref-for-propdef-direction"></a>

          <a id="ref-for-keyframe-effect-effect-target⑦"></a>

          For logical properties [\[CSS-LOGICAL-1\]](#biblio-css-logical-1), add the [equivalent physical properties](https://drafts.csswg.org/css-writing-modes-4/#logical-to-physical) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4) based on the computed value of [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) and/or [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) for the [effect target](#keyframe-effect-effect-target).

        <a id="ref-for-propdef-border-width"></a>

        <a id="ref-for-compute-a-property-value③"></a>

        <a id="ref-for-propdef-border-bottom-width"></a>

        <a id="ref-for-propdef-border-left-width"></a>

        <a id="ref-for-propdef-border-right-width"></a>

        <a id="ref-for-propdef-border-top-width"></a>

        For example, if <var>keyframe</var> has a value of "12pt" for the [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width) property, the user agent may [compute a property value](#compute-a-property-value) of "16px" for each of the longhand properties: [border-bottom-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width), [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width), [border-right-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width), and [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width). As a result, <var>computed keyframe</var> would <em>not</em> have a value for the <a id="ref-for-propdef-border-width①"></a>border-width property, but would instead include each of the longhand properties, and each with the value "16px".

        If conflicts arise when expanding shorthand properties or replacing logical properties with physical properties, apply the following rules in order until the conflict is resolved:

        1.  <a id="ref-for-propdef-border-top-color"></a>

            <a id="ref-for-propdef-border-top"></a>

            Longhand properties override shorthand properties (e.g. [border-top-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color) overrides [border-top](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top)).

        2.  <a id="ref-for-propdef-border-top①"></a>

            <a id="ref-for-propdef-border-color"></a>

            Shorthand properties with fewer longhand components override those with more longhand components (e.g. [border-top](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top) overrides [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color)).

        3.  Physical properties override logical properties.

        4.  <a id="ref-for-css-property-to-idl-attribute"></a>

            For shorthand properties with an equal number of longhand components, properties whose IDL name (see the [CSS property to IDL attribute](https://drafts.csswg.org/cssom/#css-property-to-idl-attribute) algorithm [\[CSSOM\]](#biblio-cssom)) appears earlier when sorted in ascending order by the Unicode codepoints that make up each IDL name, override those who appear later.

3.  <a id="ref-for-compute-missing-keyframe-offsets"></a>

    Apply the procedure to [compute missing keyframe offsets](#compute-missing-keyframe-offsets) to <var>computed keyframes</var>.

4.  Return <var>computed keyframes</var>.

#### <a id="the-effect-value-of-a-keyframe-animation-effect"></a>5.3.4. The effect value of a keyframe effect

<a id="ref-for-effect-value③"></a>

<a id="ref-for-keyframe-effect⑨"></a>

<a id="ref-for-target-property①"></a>

The [effect value](#effect-value) of a single property referenced by a [keyframe effect](#keyframe-effect) as one of its [target properties](#target-property), for a given <var>iteration progress</var>, <var>current iteration</var> and <var>underlying value</var> is calculated as follows.

1.  <a id="ref-for-unresolved①①①"></a>

    If <var>iteration progress</var> is [unresolved](#unresolved) abort this procedure.

2.  <a id="ref-for-effect-value④"></a>

    Let <var>target property</var> be the longhand property for which the [effect value](#effect-value) is to be calculated.

3.  <a id="ref-for-animation-type⑤"></a>

    <a id="ref-for-not-animatable②"></a>

    If [animation type](#animation-type) of the <var>target property</var> is [not animatable](#not-animatable) abort this procedure since the effect cannot be applied.

4.  <a id="ref-for-keyframe-effect①⓪"></a>

    <a id="ref-for-keyframe-effect-effect-target⑧"></a>

    If the [keyframe effect](#keyframe-effect) does not have an [effect target](#keyframe-effect-effect-target), or if the <a id="ref-for-keyframe-effect-effect-target⑨"></a>effect target cannot have computed property values calculated, abort this procedure.

5.  <a id="ref-for-underlying-value①"></a>

    <a id="ref-for-composite-operation-add"></a>

    <a id="ref-for-composite-operation②"></a>

    Define the <a id="neutral-value-for-composition"></a>neutral value for composition as a value which, when combined with an [underlying value](#underlying-value) using the [add](#composite-operation-add) [composite operation](#composite-operation), produces the <a id="ref-for-underlying-value②"></a>underlying value.

6.  <a id="ref-for-computed-keyframes①"></a>

    <a id="ref-for-keyframe-effect①①"></a>

    Let <var>property-specific keyframes</var> be the result of getting the set of [computed keyframes](#computed-keyframes) for this [keyframe effect](#keyframe-effect).

7.  <a id="ref-for-keyframe②③"></a>

    Remove any [keyframes](#keyframe) from <var>property-specific&#xA;keyframes</var> that do not have a property value for <var>target property</var>.

8.  If <var>property-specific keyframes</var> is empty, return <var>underlying value</var>.

9.  <a id="ref-for-keyframe②④"></a>

    <a id="ref-for-computed-keyframe-offset①①"></a>

    <a id="ref-for-neutral-value-for-composition"></a>

    <a id="ref-for-composite-operation③"></a>

    <a id="ref-for-composite-operation-add①"></a>

    If there is no [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> with a [computed keyframe offset](#computed-keyframe-offset) of 0, create a new <a id="ref-for-keyframe②⑤"></a>keyframe with a <a id="ref-for-computed-keyframe-offset①②"></a>computed keyframe offset of 0, a property value set to the [neutral value for composition](#neutral-value-for-composition), and a [composite operation](#composite-operation) of [add](#composite-operation-add), and prepend it to the beginning of <var>property-specific keyframes</var>.

10. <a id="ref-for-keyframe②⑥"></a>

    <a id="ref-for-computed-keyframe-offset①③"></a>

    <a id="ref-for-neutral-value-for-composition①"></a>

    <a id="ref-for-composite-operation④"></a>

    <a id="ref-for-composite-operation-add②"></a>

    Similarly, if there is no [keyframe](#keyframe) in <var>property-specific keyframes</var> with a [computed keyframe offset](#computed-keyframe-offset) of 1, create a new <a id="ref-for-keyframe②⑦"></a>keyframe with a <a id="ref-for-computed-keyframe-offset①④"></a>computed keyframe offset of 1, a property value set to the [neutral value for composition](#neutral-value-for-composition), and a [composite operation](#composite-operation) of [add](#composite-operation-add), and append it to the end of <var>property-specific keyframes</var>.

11. Let <var>interval endpoints</var> be an empty sequence of keyframes.

12. Populate <var>interval endpoints</var> by following the steps from the first matching condition from below:

    <a id="ref-for-computed-keyframe-offset①⑤"></a>

    <a id="ref-for-keyframe②⑧"></a>

    If <var>iteration progress</var> \< 0 and there is more than one [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> with a [computed keyframe offset](#computed-keyframe-offset) of 0,
    <a id="ref-for-keyframe②⑨"></a>

    Add the first [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> to <var>interval endpoints</var>.

    <a id="ref-for-computed-keyframe-offset①⑥"></a>

    <a id="ref-for-keyframe③⓪"></a>

    If <var>iteration progress</var> ≥ 1 and there is more than one [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> with a [computed keyframe offset](#computed-keyframe-offset) of 1,
    <a id="ref-for-keyframe③①"></a>

    Add the last [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> to <var>interval endpoints</var>.

    Otherwise,
    1.  <a id="ref-for-keyframe③②"></a>

        <a id="ref-for-computed-keyframe-offset①⑦"></a>

        <a id="ref-for-iteration-progress①⓪"></a>

        Append to <var>interval endpoints</var> the last [keyframe](#keyframe) in <var>property-specific&#xA;keyframes</var> whose [computed keyframe offset](#computed-keyframe-offset) is less than or equal to <var>iteration progress</var> and less than 1. If there is no such <a id="ref-for-keyframe③③"></a>keyframe (because, for example, the [iteration progress](#iteration-progress) is negative), add the last <a id="ref-for-keyframe③④"></a>keyframe whose <a id="ref-for-computed-keyframe-offset①⑧"></a>computed keyframe offset is 0.

    2.  <a id="ref-for-keyframe③⑤"></a>

        Append to <var>interval endpoints</var> the next [keyframe](#keyframe) in <var>property-specific keyframes</var> after the one added in the previous step.

13. For each <var>keyframe</var> in <var>interval endpoints</var>:

    1.  <a id="ref-for-composite-operation⑤"></a>

        <a id="ref-for-composite-operation-replace"></a>

        <a id="ref-for-keyframe-effect①②"></a>

        If <var>keyframe</var> has a [composite operation](#composite-operation) that is <em>not</em> [replace](#composite-operation-replace), or <var>keyframe</var> has no <a id="ref-for-composite-operation⑥"></a>composite operation and the <a id="ref-for-composite-operation⑦"></a>composite operation of this [keyframe effect](#keyframe-effect) is <em>not</em> <a id="ref-for-composite-operation-replace①"></a>replace, then perform the following steps:

        1.  <a id="ref-for-composite-operation⑧"></a>

            <a id="ref-for-keyframe-effect①③"></a>

            Let <var>composite operation to use</var> be the [composite operation](#composite-operation) of <var>keyframe</var>, or if it has none, the <a id="ref-for-composite-operation⑨"></a>composite operation of this [keyframe effect](#keyframe-effect).

        2.  Let <var>value to combine</var> be the property value of <var>target property</var> specified on <var>keyframe</var>.

        3.  <a id="ref-for-animation-type⑥"></a>

            Replace the property value of <var>target property</var> on <var>keyframe</var> with the result of combining <var>underlying value</var> (<var>V</var><sub>a</sub>) and <var>value to combine</var> (<var>V</var><sub>b</sub>) using the procedure for the <var>composite operation to use</var> corresponding to the <var>target property</var>’s [animation type](#animation-type).

14. If there is only one keyframe in <var>interval endpoints</var> return the property value of <var>target property</var> on that keyframe.

15. <a id="ref-for-computed-keyframe-offset①⑨"></a>

    Let <var>start offset</var> be the [computed keyframe offset](#computed-keyframe-offset) of the first keyframe in <var>interval endpoints</var>.

16. <a id="ref-for-computed-keyframe-offset②⓪"></a>

    Let <var>end offset</var> be the [computed keyframe offset](#computed-keyframe-offset) of last keyframe in <var>interval endpoints</var>.

17. Let <var>interval distance</var> be the result of evaluating <code>(<var>iteration progress</var> - <var>start offset</var>) /&#xA;(<var>end offset</var> - <var>start offset</var>)</code>.

18. <a id="ref-for-easing-function⑦"></a>

    Let <var>transformed distance</var> be the result of evaluating the [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) associated with the first keyframe in <var>interval endpoints</var> passing <var>interval distance</var> as the input progress.

19. <a id="ref-for-interpolation①"></a>

    <a id="ref-for-animation-type⑦"></a>

    Return the result of applying the [interpolation procedure](https://www.w3.org/TR/css-values-4/#interpolation) defined by the [animation type](#animation-type) of the <var>target property</var>, to the values of the <var>target property</var> specified on the two keyframes in <var>interval endpoints</var> taking the first such value as <var>V</var><sub>start</sub> and the second as <var>V</var><sub>end</sub> and using <var>transformed&#xA;distance</var> as the interpolation parameter <var>p</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-keyframe③⑥"></a>
>
> Note that this procedure assumes the following about the list of [keyframes](#keyframe) specified on the effect:
>
> - <a id="ref-for-keyframe③⑦"></a>
>
>   <a id="ref-for-computed-keyframe-offset②①"></a>
>
>   Each [keyframe](#keyframe) has a specified [computed keyframe offset](#computed-keyframe-offset) in the range \[0, 1\].
>
> - <a id="ref-for-keyframe③⑧"></a>
>
>   <a id="ref-for-computed-keyframe-offset②②"></a>
>
>   The list of [keyframes](#keyframe) is sorted in ascending order by [computed keyframe offset](#computed-keyframe-offset).
>
> - For a given property, there is at most one specified property value on each keyframe.
>
> It is the responsibility of the user of the model (for example, a declarative markup or programming interface) to ensure these conditions are met.
>
> <a id="ref-for-computed-keyframes②"></a>
>
> For example, for the [programming interface](#programming-interface) defined by this specification, these conditions are met by the procedure to produce the [computed keyframes](#computed-keyframes) that become the input to this procedure.

<a id="ref-for-keyframe③⑨"></a>

<a id="ref-for-iteration-progress①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this procedure permits overlapping [keyframes](#keyframe). The behavior is that at the point of overlap the output value jumps to the value of the last defined <a id="ref-for-keyframe④⓪"></a>keyframe at that offset. For overlapping keyframes at 0 or 1, the output value for [iteration progress](#iteration-progress) values less than 0 or greater than or equal to 1 is the value of the first <a id="ref-for-keyframe④①"></a>keyframe or the last <a id="ref-for-keyframe④②"></a>keyframe in <var>keyframes</var> respectively.

<a id="ref-for-computed-keyframes③"></a>

<a id="ref-for-effect-value⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [computed keyframes](#computed-keyframes) are "live": user-agents must behave as if they are recreated every time the [effect value](#effect-value) is calculated.
>
> <a id="ref-for-propdef-font-size③"></a>
>
> <a id="ref-for-keyframe④③"></a>
>
> <a id="ref-for-computed-value⑥"></a>
>
> For example, if there is an ongoing transition on the [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) property from `10px` to `20px`, a property value specified as `1em` in a [keyframe](#keyframe) would during [keyframe computation](#calculating-computed-keyframes) resolve against the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) in the range \[`10px`, `20px`\] produced by the transition on <a id="ref-for-propdef-font-size④"></a>font-size.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9e46aa87"></a> In the presence of certain timing functions, the input iteration progress to an animation effect is not limited to the range \[0, 1\]. Currently, however, keyframe offsets <em>are</em> limited to the range \[0, 1\] and property values are simply extrapolated for input iteration progress values outside this range.
>
> We have considered removing this restriction since some cases exist where it is useful to be able to specify non-linear changes in property values at iteration progress values outside the range \[0, 1\]. One example is an animation that interpolates from green to yellow but has an overshoot timing function that makes it temporarily interpolate "beyond" yellow to red before settling back to yellow.
>
> While this effect could be achieved by modification of the keyframes and timing function, this approach seems to break the model’s separation of timing concerns from animation effects.
>
> It is not clear how this effect should be achieved but we note that allowing keyframe offsets outside \[0, 1\] may make the currently specified behavior where keyframes at offset 0 and 1 are synthesized as necessary, inconsistent.
>
> See [section 4 (Keyframe offsets outside \[0, 1\]) of minuted discussion from Tokyo 2013 F2F](https://lists.w3.org/Archives/Public/public-fx/2013AprJun/0184.html).
>
> [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;2081&#x3E;](https://github.com/w3c/csswg-drafts/issues/2081)

### <a id="combining-effects"></a>5.4. Combining effects

<em>This section is non-normative</em>

<a id="ref-for-effect-value⑥"></a>

<a id="ref-for-keyframe-effect①④"></a>

<a id="ref-for-animation-effect⑦③"></a>

<a id="ref-for-target-property②"></a>

After calculating the [effect values](#effect-value) for a [keyframe effect](#keyframe-effect), they are applied to the [animation effect](#animation-effect)’s [target properties](#target-property).

<a id="ref-for-in-effect③"></a>

<a id="ref-for-keyframe-effect①⑤"></a>

<a id="ref-for-effect-stack①"></a>

<a id="ref-for-animation-effect⑦④"></a>

Since it is possible for multiple [in effect](#in-effect) [keyframe effects](#keyframe-effect) to target the same property it is often necessary to combine the results of several <a id="ref-for-keyframe-effect①⑥"></a>keyframe effects together. This process is called <a id="composite"></a>compositing and is based on establishing an [effect stack](#effect-stack) for each property targeted by an <a id="ref-for-in-effect④"></a>in effect [animation effect](#animation-effect).

<a id="ref-for-composite"></a>

<a id="ref-for-keyframe-effect①⑦"></a>

<a id="ref-for-target-property③"></a>

After [compositing](#composite) the results of [keyframe effects](#keyframe-effect) together, the composited result is combined with other values specified for the [target property](#target-property).

The arrangement is illustrated below:

![Overview of the application of effect values to their target properties](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/images/animation-cascade.svg)

<a id="ref-for-effect-value⑦"></a>

<a id="ref-for-target-property④"></a>

<a id="ref-for-keyframe-effect①⑧"></a>

<a id="ref-for-effect-stack②"></a>

Overview of the application of [effect values](#effect-value) to their [target properties](#target-property).  
The results of [keyframe effects](#keyframe-effect) targeting the same property are composited together using an [effect stack](#effect-stack).  
The result of this composition is then inserted into the CSS cascade at an appropriate point.

<a id="ref-for-effect-value⑧"></a>

<a id="ref-for-target-property⑤"></a>

<a id="ref-for-keyframe-effect①⑨"></a>

For the first part of this operation—combining [effect values](#effect-value) that target the same [property](#target-property)— it is necessary to determine both <em>how</em> [keyframe effects](#keyframe-effect) are combined with one another, as well as the <em>order</em> in which they are applied, that is, their relative <em>composite order</em>.

<a id="ref-for-effect-value⑨"></a>

<a id="ref-for-composite-operation①⓪"></a>

<a id="ref-for-keyframe-effect②⓪"></a>

The matter of <em>how</em> [effect values](#effect-value) are combined is governed by the [composite operation](#composite-operation) of the corresponding [keyframe effects](#keyframe-effect).

<a id="ref-for-effect-value①⓪"></a>

<a id="ref-for-effect-stack③"></a>

The relative <em>composite order</em> of [effect values](#effect-value) is determined by an [effect stack](#effect-stack) established for each animated property.

#### <a id="animation-classes"></a>5.4.1. Animation classes

<a id="ref-for-concept-animation④⑤"></a>

This specification provides a common animation model intended to be used by other specifications that define markup or programming interfaces on top of this model. The particular markup or programming interface that generated an [animation](#concept-animation) defines its <a id="animation-class"></a>animation class.

Further specifications may define specialized behavior for composite ordering between different classes of animations or within a particular class.

<em>This section is non-normative</em>

<a id="ref-for-animation-class①"></a>

For example, animations whose [class](#animation-class) is "CSS animation" are defined as having a <em>higher</em> composite order than animations whose class is "CSS transition" but <em>lower</em> than other animations without a specific class.

<a id="ref-for-propdef-animation-name"></a>

Within the set of "CSS animation" objects, specialized composite ordering is defined based on the [animation-name](https://www.w3.org/TR/css-animations-1/#propdef-animation-name) property amongst other factors.

#### <a id="the-effect-stack"></a>5.4.2. The effect stack

<a id="ref-for-target-property⑥"></a>

<a id="ref-for-keyframe-effect②①"></a>

<a id="ref-for-effect-stack④"></a>

An <a id="effect-stack"></a>effect stack is associated with each property [targeted](#target-property) by one or more [keyframe effects](#keyframe-effect). The [effect stack](#effect-stack) establishes the relative composite order of <a id="ref-for-keyframe-effect②②"></a>keyframe effects.

<a id="ref-for-keyframe-effect②③"></a>

<a id="ref-for-effect-stack⑤"></a>

The relative <a id="animation-composite-order"></a>composite order of any two [keyframe effects](#keyframe-effect), <var>A</var> and <var>B</var>, within an [effect stack](#effect-stack) is established by comparing their properties as follows:

1.  <a id="ref-for-concept-animation④⑥"></a>

    <a id="ref-for-associated-with-an-animation⑧"></a>

    <a id="ref-for-animation-effect⑦⑤"></a>

    Let the <a id="animation-effect-associated-animation"></a>associated animation of an animation effect be the [animation](#concept-animation) [associated](#associated-with-an-animation) with the [animation effect](#animation-effect).

2.  Sort <var>A</var> and <var>B</var> by applying the following conditions in turn until the order is resolved,

    1.  <a id="ref-for-animation-effect-associated-animation"></a>

        <a id="ref-for-animation-class②"></a>

        If <var>A</var> and <var>B</var>’s [associated animations](#animation-effect-associated-animation) differ by [class](#animation-class), sort by any inter-class composite order defined for the corresponding classes.

    2.  <a id="ref-for-animation-class③"></a>

        <a id="ref-for-animation-effect-associated-animation①"></a>

        If <var>A</var> and <var>B</var> are still not sorted, sort by any [class](#animation-class)-specific composite order defined by the common class of <var>A</var> and <var>B</var>’s [associated animations](#animation-effect-associated-animation).

    3.  <a id="ref-for-animation-effect-associated-animation②"></a>

        <a id="ref-for-global-animation-list"></a>

        If <var>A</var> and <var>B</var> are still not sorted, sort by the position of their [associated animations](#animation-effect-associated-animation) in the [global animation list](#global-animation-list).

<a id="ref-for-animation-effect⑦⑥"></a>

[Animation effects](#animation-effect) that sort earlier have <em>lower</em> composite order.

#### <a id="calculating-the-result-of-an-effect-stack"></a>5.4.3. Calculating the result of an effect stack

<a id="ref-for-effect-stack⑥"></a>

<a id="ref-for-effect-value①①"></a>

<a id="ref-for-keyframe-effect②④"></a>

In order to calculate the final value of an [effect stack](#effect-stack), the [effect values](#effect-value) of each [keyframe effect](#keyframe-effect) in the stack are combined in composite order.

<a id="ref-for-effect-stack⑦"></a>

Each step in the process of evaluating an [effect stack](#effect-stack) takes an <a id="underlying-value"></a>underlying value as input.

<a id="ref-for-keyframe-effect②⑤"></a>

<a id="ref-for-effect-value①②"></a>

<a id="ref-for-underlying-value③"></a>

For each [keyframe effect](#keyframe-effect) in the stack, the appropriate [effect value](#effect-value) from the <a id="ref-for-keyframe-effect②⑥"></a>keyframe effect is combined with the [underlying value](#underlying-value) to produce a new value. This resulting value becomes the <a id="ref-for-underlying-value④"></a>underlying value for combining the next <a id="ref-for-keyframe-effect②⑦"></a>keyframe effect in the stack.

<a id="ref-for-effect-stack⑧"></a>

<a id="ref-for-effect-value①③"></a>

<a id="ref-for-keyframe-effect②⑧"></a>

<a id="ref-for-underlying-value⑤"></a>

The final value of an [effect stack](#effect-stack), called the <a id="composited-value"></a>composited value, is simply the result of combining the [effect value](#effect-value) of the final (highest composite order) [keyframe effect](#keyframe-effect) in the stack with the [underlying value](#underlying-value) at that point.

#### <a id="effect-composition"></a>5.4.4. Effect composition

<a id="ref-for-effect-value①④"></a>

<a id="ref-for-underlying-value⑥"></a>

<a id="ref-for-keyframe-effect②⑨"></a>

The specific operation used to combine an [effect value](#effect-value) with an [underlying value](#underlying-value) is determined by the <a id="composite-operation"></a>composite operation of the [keyframe effect](#keyframe-effect) that produced the <a id="ref-for-effect-value①⑤"></a>effect value.

<a id="ref-for-composite-operation①①"></a>

This specification defines three [composite operations](#composite-operation) as follows:

<a id="composite-operation-replace"></a>replace  
<a id="ref-for-effect-value①⑥"></a>

<a id="ref-for-underlying-value⑦"></a>

The result of compositing the [effect value](#effect-value) with the [underlying value](#underlying-value) is simply the <a id="ref-for-effect-value①⑦"></a>effect value.

<a id="composite-operation-add"></a>add  
<a id="ref-for-effect-value①⑧"></a>

<a id="ref-for-addition"></a>

<a id="ref-for-underlying-value⑧"></a>

<a id="ref-for-animation-type⑧"></a>

<a id="ref-for-underlying-value⑨"></a>

<a id="ref-for-effect-value①⑨"></a>

The [effect value](#effect-value) is [added](https://www.w3.org/TR/css-values-4/#addition) to the [underlying value](#underlying-value). For [animation types](#animation-type) where the <a id="ref-for-addition①"></a>addition operation is defined such that it is not commutative, the order of the operands is <code><a href="#underlying-value">underlying value</a> + <a href="#effect-value">effect value</a></code>.

<a id="composite-operation-accumulate"></a>accumulate  
<a id="ref-for-effect-value②⓪"></a>

<a id="ref-for-accumulation"></a>

<a id="ref-for-underlying-value①⓪"></a>

<a id="ref-for-animation-type⑨"></a>

The [effect value](#effect-value) is [accumulated](https://www.w3.org/TR/css-values-4/#accumulation) onto the [underlying value](#underlying-value). For [animation types](#animation-type) where the <a id="ref-for-accumulation①"></a>accumulation operation is defined such that it is not commutative, the order of the operands is <a id="ref-for-underlying-value①①"></a>underlying value followed by <a id="ref-for-effect-value②①"></a>effect value.

#### <a id="applying-the-composited-result"></a>5.4.5. Applying the composited result

<a id="ref-for-composited-value"></a>

<a id="ref-for-target-property⑦"></a>

Applying a [composited value](#composited-value) to a [target property](#target-property) is achieved by adding a specified value to the CSS cascade.

<a id="ref-for-animation-class④"></a>

<a id="ref-for-concept-animation④⑦"></a>

<a id="ref-for-animation-effect-associated-animation③"></a>

<a id="ref-for-effect-stack⑨"></a>

The level of the cascade to which this specified value is added depends on the [class](#animation-class) of the [animation](#concept-animation) [associated with](#animation-effect-associated-animation) the effect with the highest composite order in the [effect stack](#effect-stack) for a given property. By default, the specified value is added to the "Animation declarations" level of the cascade ([\[css-cascade-3\]](#biblio-css-cascade-3)).

<em>This section is non-normative</em>

<a id="ref-for-composited-value①"></a>

For example, if the effect with the highest composite order is associated with a "CSS transition"-class animation, the [composited value](#composited-value) will be added to "Transition declarations" level of the cascade.

<a id="ref-for-composited-value②"></a>

<a id="ref-for-target-property⑧"></a>

The [composited value](#composited-value) calculated for a CSS [target property](#target-property) is applied using the following process.

1.  Calculate the <var>base value</var> of the property as the value generated for that property by computing the computed value for that property in the absence of animations.

2.  <a id="ref-for-effect-stack①⓪"></a>

    Establish the [effect stack](#effect-stack) for the property (see [§ 5.4.2 The effect stack](#the-effect-stack)).

3.  <a id="ref-for-composited-value③"></a>

    <a id="ref-for-effect-stack①①"></a>

    <a id="ref-for-underlying-value①②"></a>

    Calculate the [composited value](#composited-value) of the [effect stack](#effect-stack) passing in the <var>base value</var> of the property as the initial [underlying value](#underlying-value) (see [§ 5.4.3 Calculating the result of an effect stack](#calculating-the-result-of-an-effect-stack)).

4.  <a id="ref-for-composited-value④"></a>

    <a id="ref-for-animation-class⑤"></a>

    <a id="ref-for-concept-animation④⑧"></a>

    <a id="ref-for-animation-effect-associated-animation④"></a>

    <a id="ref-for-effect-stack①②"></a>

    Insert the [composited value](#composited-value) into the CSS cascade at the level defined for the [class](#animation-class) of the [animation](#concept-animation) [associated with](#animation-effect-associated-animation) the effect at the top of the [effect stack](#effect-stack) established for the target property.

### <a id="replacing-animations"></a>5.5. Replacing animations

<em>This section is non-normative</em>

Using the programming interface defined in this specification, it is possible to repeatedly trigger new animations that contribute to an element’s animated style indefinitely.

For example, consider the following code:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-86ce4327"></a>
>
> ```javascript
> elem.addEventListener('mousemove', evt => {
>   circle.animate(
>     { transform: `translate(${evt.clientX}px, ${evt.clientY}px)` },
>     { duration: 500, fill: 'forwards' }
>   );
> });
> ```
This will generate a new forwards-filling animation each time the mouse is moved, quickly producing hundreds, even thousands of forwards-filling animations.

If the user agent is required to retain <em>all</em> such animations, the list of animations would grow in an unbounded fashion, producing a memory leak.

This section defines a mechanism that causes overridden animations to be automatically removed unless the author explicitly requests they be retained.

#### <a id="animation-replace-state"></a>5.5.1. Replace state

<a id="ref-for-concept-animation④⑨"></a>

An [animation](#concept-animation) maintains a <a id="replace-state"></a>replace state that may be one of the following values:

- <a id="active-replace-state"></a>active

- <a id="removed-replace-state"></a>removed

- <a id="persisted-replace-state"></a>persisted

<a id="ref-for-concept-animation⑤⓪"></a>

<a id="ref-for-replace-state①"></a>

<a id="ref-for-active-replace-state"></a>

The initial value of an [animation](#concept-animation)'s [replace state](#replace-state) is [active](#active-replace-state).

<a id="ref-for-animation-effect⑦⑦"></a>

<a id="ref-for-concept-animation⑤①"></a>

<a id="ref-for-replace-state②"></a>

<a id="ref-for-removed-replace-state①"></a>

<a id="ref-for-effect-stack①③"></a>

<a id="ref-for-target-property⑨"></a>

The [animation effects](#animation-effect) of an [animation](#concept-animation) whose [replace state](#replace-state) is [removed](#removed-replace-state) are not included in the [effect stacks](#effect-stack) of their [target properties](#target-property).

#### <a id="removing-replaced-animations"></a>5.5.2. Removing replaced animations

<a id="ref-for-concept-animation⑤②"></a>

An [animation](#concept-animation) is <a id="replaceable-animation"></a>replaceable if <em>all</em> of the following conditions are true:

- <a id="ref-for-concept-animation⑤③"></a>

  <a id="ref-for-owning-element"></a>

  <a id="ref-for-owning-element①"></a>

  The existence of the [animation](#concept-animation) is <em>not</em> prescribed by markup. That is, it is <em>not</em> a CSS animation with an [owning element](https://drafts.csswg.org/css-animations-2/#owning-element), nor a CSS transition with an [owning element](https://drafts.csswg.org/css-transitions-2/#owning-element).

- <a id="ref-for-concept-animation⑤④"></a>

  <a id="ref-for-animation-play-state⑧"></a>

  <a id="ref-for-play-state-finished①④"></a>

  The [animation](#concept-animation)'s [play state](#animation-play-state) is [finished](#play-state-finished).

- <a id="ref-for-concept-animation⑤⑤"></a>

  <a id="ref-for-replace-state③"></a>

  <a id="ref-for-removed-replace-state②"></a>

  The [animation](#concept-animation)'s [replace state](#replace-state) is <em>not</em> [removed](#removed-replace-state).

- <a id="ref-for-concept-animation⑤⑥"></a>

  <a id="ref-for-monotonically-increasing-timeline⑦"></a>

  <a id="ref-for-timeline⑤②"></a>

  The [animation](#concept-animation) is associated with a [monotonically increasing](#monotonically-increasing-timeline) [timeline](#timeline).

- <a id="ref-for-concept-animation⑤⑦"></a>

  <a id="ref-for-animation-associated-effect②⑥"></a>

  The [animation](#concept-animation) has an [associated effect](#animation-associated-effect).

- <a id="ref-for-concept-animation⑤⑧"></a>

  <a id="ref-for-animation-associated-effect②⑦"></a>

  <a id="ref-for-in-effect⑤"></a>

  The [animation](#concept-animation)'s [associated effect](#animation-associated-effect) is [in effect](#in-effect).

- <a id="ref-for-concept-animation⑤⑨"></a>

  <a id="ref-for-animation-associated-effect②⑧"></a>

  <a id="ref-for-keyframe-effect-effect-target①⓪"></a>

  The [animation](#concept-animation)'s [associated effect](#animation-associated-effect) has an [effect target](#keyframe-effect-effect-target).

<a id="ref-for-document⑥"></a>

<a id="ref-for-concept-animation⑥⓪"></a>

When asked to <a id="remove-replaced-animations"></a>remove replaced animations for a <code><a href="https://html.spec.whatwg.org/#document">Document</a></code>, <var>doc</var>, then for every [animation](#concept-animation), <var>animation</var>, that:

- <a id="ref-for-associated-with-an-animation⑨"></a>

  <a id="ref-for-animation-effect⑦⑧"></a>

  <a id="ref-for-keyframe-effect-effect-target①①"></a>

  <a id="ref-for-concept-tree-descendant①"></a>

  has an [associated](#associated-with-an-animation) [animation effect](#animation-effect) whose [effect target](#keyframe-effect-effect-target) is a [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) of <var>doc</var>, and

- <a id="ref-for-replaceable-animation"></a>

  is [replaceable](#replaceable-animation), and

- <a id="ref-for-replace-state④"></a>

  <a id="ref-for-active-replace-state①"></a>

  has a [replace state](#replace-state) of [active](#active-replace-state), and

- <a id="ref-for-target-property①⓪"></a>

  <a id="ref-for-animation-effect⑦⑨"></a>

  <a id="ref-for-associated-with-an-animation①⓪"></a>

  <a id="ref-for-replaceable-animation①"></a>

  <a id="ref-for-concept-animation⑥①"></a>

  <a id="ref-for-animation-composite-order①"></a>

  for which there exists for each [target property](#target-property) of every [animation effect](#animation-effect) [associated](#associated-with-an-animation) with <var>animation</var>, an <a id="ref-for-animation-effect⑧⓪"></a>animation effect associated with a [replaceable](#replaceable-animation) [animation](#concept-animation) with a higher [composite order](#animation-composite-order) than <var>animation</var> that includes the same <a id="ref-for-target-property①①"></a>target property

perform the following steps:

1.  <a id="ref-for-replace-state⑤"></a>

    <a id="ref-for-removed-replace-state③"></a>

    Set <var>animation</var>’s [replace state](#replace-state) to [removed](#removed-replace-state).

2.  <a id="ref-for-concept-event-create②"></a>

    <a id="ref-for-animationplaybackevent②"></a>

    [Create](https://dom.spec.whatwg.org/#concept-event-create) an <code><a href="#animationplaybackevent">AnimationPlaybackEvent</a></code>, <var>removeEvent</var>.

3.  <a id="ref-for-dom-event-type②"></a>

    <a id="ref-for-remove-event"></a>

    Set <var>removeEvent</var>’s <code><a href="https://dom.spec.whatwg.org/#dom-event-type">type</a></code> attribute to [remove](#remove-event).

4.  <a id="ref-for-dom-animationplaybackevent-currenttime②"></a>

    <a id="ref-for-animation-current-time④⑨"></a>

    Set <var>removeEvent</var>’s <code><a href="#dom-animationplaybackevent-currenttime">currentTime</a></code> attribute to the [current time](#animation-current-time) of <var>animation</var>.

5.  <a id="ref-for-dom-animationplaybackevent-timelinetime③"></a>

    <a id="ref-for-timeline-current-time⑦"></a>

    <a id="ref-for-timeline⑤③"></a>

    Set <var>removeEvent</var>’s <code><a href="#dom-animationplaybackevent-timelinetime">timelineTime</a></code> attribute to the [current time](#timeline-current-time) of the [timeline](#timeline) with which <var>animation</var> is associated.

6.  <a id="ref-for-animation-document-for-timing⑤"></a>

    <a id="ref-for-pending-animation-event-queue④"></a>

    <a id="ref-for-scheduled-event-time⑥"></a>

    <a id="ref-for-timeline-time-to-origin-relative-time④"></a>

    <a id="ref-for-timeline-current-time⑧"></a>

    <a id="ref-for-timeline⑤④"></a>

    If <var>animation</var> has a [document for timing](#animation-document-for-timing), then append <var>removeEvent</var> to its <a id="ref-for-animation-document-for-timing⑥"></a>document for timing's [pending animation event queue](#pending-animation-event-queue) along with its target, <var>animation</var>. For the [scheduled event time](#scheduled-event-time), use the result of applying the procedure to convert [timeline time to origin-relative time](#timeline-time-to-origin-relative-time) to the [current time](#timeline-current-time) of the [timeline](#timeline) with which <var>animation</var> is associated.

    <a id="ref-for-queue-a-task②"></a>

    <a id="ref-for-concept-event-dispatch③"></a>

    <a id="ref-for-dom-manipulation-task-source②"></a>

    Otherwise, [queue a task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-task) to [dispatch](https://dom.spec.whatwg.org/#concept-event-dispatch) <var>removeEvent</var> at <var>animation</var>. The task source for this task is the [DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source).

### <a id="side-effects-section"></a>5.6. Side effects of animation

<a id="ref-for-animation-effect⑧①"></a>

<a id="ref-for-current②"></a>

<a id="ref-for-in-effect⑥"></a>

<a id="ref-for-concept-animation⑥②"></a>

<a id="ref-for-replace-state⑥"></a>

<a id="ref-for-removed-replace-state④"></a>

<a id="ref-for-propdef-will-change"></a>

<a id="ref-for-keyframe-effect-effect-target①②"></a>

For every property targeted by at least one [animation effect](#animation-effect) that is [current](#current) or [in effect](#in-effect), and which is associated with an [animation](#concept-animation) whose [replace state](#replace-state) is <em>not</em> [removed](#removed-replace-state), the user agent must act as if the [will-change](https://www.w3.org/TR/css-will-change-1/#propdef-will-change) property ([\[css-will-change-1\]](#biblio-css-will-change-1)) on the [effect target](#keyframe-effect-effect-target) includes the property.

<em>This section is non-normative</em>

<a id="ref-for-propdef-transform"></a>

<a id="ref-for-x43"></a>

<a id="ref-for-keyframe-effect-effect-target①③"></a>

<a id="ref-for-concept-animation⑥③"></a>

<a id="ref-for-animation-effect-before-phase⑧"></a>

<a id="ref-for-animation-effect-active-phase⑦"></a>

<a id="ref-for-fill-mode⑨"></a>

<a id="ref-for-animation-effect-after-phase⑨"></a>

As a result of the above requirement, if an animation targets, for example, the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property of an element, a [stacking context](https://www.w3.org/TR/CSS21/visuren.html#x43) will be created for the [effect target](#keyframe-effect-effect-target) so long as the [animation](#concept-animation) is in the [before phase](#animation-effect-before-phase), the [active phase](#animation-effect-active-phase) or, if it has a [fill mode](#fill-mode) of "forwards" or "both", the [after phase](#animation-effect-after-phase).

## <a id="programming-interface"></a>6. Programming interface

<em>This section is non-normative</em>

In addition to the abstract model described above, Web Animations also defines a programming interface to the model. This interface can be used to inspect and extend animations produced by declarative means or for directly producing animations when a procedural approach is more suitable.

### <a id="time-values-in-the-programming-interface"></a>6.1. Time values in the programming interface

<a id="ref-for-time-value④④"></a>

<a id="ref-for-unresolved①①②"></a>

[Time values](#time-value) are represented in the programming interface with the type `double`. [Unresolved](#unresolved) time values are represented by the value `null`.

### <a id="the-animationtimeline-interface"></a>6.2. The `AnimationTimeline` interface

<a id="ref-for-timeline⑤⑤"></a>

<a id="ref-for-animationtimeline"></a>

[Timelines](#timeline) are represented in the Web Animations API by the <code><a href="#animationtimeline">AnimationTimeline</a></code> interface.

<a id="ref-for-Exposed"></a>

<a id="animationtimeline"></a>

<a id="ref-for-idl-double"></a>

<a id="ref-for-dom-animationtimeline-currenttime"></a>

```text
[Exposed=Window]
interface AnimationTimeline {
    readonly attribute double? currentTime;
};
```
<a id="ref-for-idl-double①"></a>

<a id="dom-animationtimeline-currenttime"></a>`currentTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), readonly, nullable

<a id="ref-for-timeline-current-time⑨"></a>

<a id="ref-for-inactive-timeline①⑧"></a>

Returns the [current time](#timeline-current-time) for this timeline or `null` if this timeline is [inactive](#inactive-timeline).

### <a id="the-documenttimeline-interface"></a>6.3. The `DocumentTimeline` interface

<a id="ref-for-document-timeline⑤"></a>

<a id="ref-for-document-default-document-timeline④"></a>

<a id="ref-for-documenttimeline"></a>

[Document timelines](#document-timeline), including the [default document timeline](#document-default-document-timeline), are represented in the Web Animations API by the <code><a href="#documenttimeline">DocumentTimeline</a></code> interface.

<a id="dictdef-documenttimelineoptions"></a>

<a id="ref-for-dom-domhighrestimestamp"></a>

<a id="ref-for-dom-documenttimelineoptions-origintime"></a>

<a id="ref-for-Exposed①"></a>

<a id="documenttimeline"></a>

<a id="ref-for-animationtimeline①"></a>

<a id="ref-for-dom-documenttimeline-documenttimeline"></a>

<a id="ref-for-dictdef-documenttimelineoptions"></a>

<a id="ref-for-dom-documenttimeline-documenttimeline-options-options"></a>

```text
dictionary DocumentTimelineOptions {
  DOMHighResTimeStamp originTime = 0;
};

[Exposed=Window]
interface DocumentTimeline : AnimationTimeline {
  constructor(optional DocumentTimelineOptions options = {});
};
```
<a id="ref-for-dom-domhighrestimestamp①"></a>

<a id="dom-documenttimelineoptions-origintime"></a>`originTime`, of type [DOMHighResTimeStamp](https://www.w3.org/TR/hr-time-3/#dom-domhighrestimestamp), defaulting to `0`

<a id="ref-for-origin-time②"></a>

<a id="ref-for-concept-settings-object-time-origin④"></a>

The [origin time](#origin-time) for the timeline specified as a real number of milliseconds relative to the [time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin).

<a id="dom-documenttimeline-documenttimeline"></a>`DocumentTimeline (options)`  
<a id="ref-for-documenttimeline①"></a>

<a id="ref-for-document⑦"></a>

<a id="ref-for-document⑧"></a>

<a id="ref-for-concept-document-window"></a>

<a id="ref-for-window"></a>

<a id="ref-for-current-global-object②"></a>

Creates a new <code><a href="#documenttimeline">DocumentTimeline</a></code>. The <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> with which the timeline is associated is the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> [associated](https://html.spec.whatwg.org/multipage/browsers.html#concept-document-window) with the <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> that is the [current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).

<a id="dom-documenttimeline-documenttimeline-options-options"></a>`options`  
<a id="ref-for-dom-documenttimelineoptions-origintime①"></a>

Configuration parameters for the newly-created timeline. This specification defines only the <code><a href="#dom-documenttimelineoptions-origintime">originTime</a></code> member but other specifications may extend this set.

### <a id="the-animation-interface"></a>6.4. The `Animation` interface

<a id="ref-for-concept-animation⑥④"></a>

<a id="ref-for-animation"></a>

[Animations](#concept-animation) are represented in the Web Animations API by the <code><a href="#animation">Animation</a></code> interface.

<a id="ref-for-Exposed②"></a>

<a id="animation"></a>

<a id="ref-for-eventtarget"></a>

<a id="ref-for-dom-animation-animation"></a>

<a id="ref-for-animationeffect"></a>

<a id="ref-for-dom-animation-animation-effect-timeline-effect"></a>

<a id="ref-for-animationtimeline②"></a>

<a id="ref-for-dom-animation-animation-effect-timeline-timeline"></a>

<a id="ref-for-idl-DOMString"></a>

<a id="ref-for-dom-animation-id"></a>

<a id="ref-for-animationeffect①"></a>

<a id="ref-for-dom-animation-effect"></a>

<a id="ref-for-animationtimeline③"></a>

<a id="ref-for-dom-animation-timeline"></a>

<a id="ref-for-idl-double②"></a>

<a id="ref-for-dom-animation-starttime"></a>

<a id="ref-for-idl-double③"></a>

<a id="ref-for-dom-animation-currenttime"></a>

<a id="ref-for-idl-double④"></a>

<a id="ref-for-dom-animation-playbackrate"></a>

<a id="ref-for-enumdef-animationplaystate"></a>

<a id="ref-for-dom-animation-playstate"></a>

<a id="ref-for-enumdef-animationreplacestate"></a>

<a id="ref-for-dom-animation-replacestate"></a>

<a id="ref-for-idl-boolean"></a>

<a id="ref-for-dom-animation-pending"></a>

<a id="ref-for-idl-promise"></a>

<a id="ref-for-animation①"></a>

<a id="ref-for-dom-animation-ready"></a>

<a id="ref-for-idl-promise①"></a>

<a id="ref-for-animation②"></a>

<a id="ref-for-dom-animation-finished"></a>

<a id="ref-for-eventhandler"></a>

<a id="ref-for-dom-animation-onfinish"></a>

<a id="ref-for-eventhandler①"></a>

<a id="ref-for-dom-animation-oncancel"></a>

<a id="ref-for-eventhandler②"></a>

<a id="ref-for-dom-animation-onremove"></a>

<a id="ref-for-idl-undefined"></a>

<a id="ref-for-dom-animation-cancel"></a>

<a id="ref-for-idl-undefined①"></a>

<a id="ref-for-dom-animation-finish①"></a>

<a id="ref-for-idl-undefined②"></a>

<a id="ref-for-dom-animation-play"></a>

<a id="ref-for-idl-undefined③"></a>

<a id="ref-for-dom-animation-pause"></a>

<a id="ref-for-idl-undefined④"></a>

<a id="ref-for-dom-animation-updateplaybackrate"></a>

<a id="ref-for-idl-double⑤"></a>

<a id="ref-for-dom-animation-updateplaybackrate-playbackrate-playbackrate"></a>

<a id="ref-for-idl-undefined⑤"></a>

<a id="ref-for-dom-animation-reverse"></a>

<a id="ref-for-idl-undefined⑥"></a>

<a id="ref-for-dom-animation-persist"></a>

<a id="ref-for-cereactions"></a>

<a id="ref-for-idl-undefined⑦"></a>

<a id="ref-for-dom-animation-commitstyles"></a>

```text
[Exposed=Window]
interface Animation : EventTarget {
    constructor(optional AnimationEffect? effect = null,
                optional AnimationTimeline? timeline);
             attribute DOMString                id;
             attribute AnimationEffect?         effect;
             attribute AnimationTimeline?       timeline;
             attribute double?                  startTime;
             attribute double?                  currentTime;
             attribute double                   playbackRate;
    readonly attribute AnimationPlayState       playState;
    readonly attribute AnimationReplaceState    replaceState;
    readonly attribute boolean                  pending;
    readonly attribute Promise<Animation>       ready;
    readonly attribute Promise<Animation>       finished;
             attribute EventHandler             onfinish;
             attribute EventHandler             oncancel;
             attribute EventHandler             onremove;
    undefined cancel();
    undefined finish();
    undefined play();
    undefined pause();
    undefined updatePlaybackRate(double playbackRate);
    undefined reverse();
    undefined persist();
    [CEReactions]
    undefined commitStyles();
};
```
<a id="dom-animation-animation"></a>`Animation (effect, timeline)`  
<a id="ref-for-animation③"></a>

Creates a new <code><a href="#animation">Animation</a></code> object using the following procedure.

1.  <a id="ref-for-animation④"></a>

    Let <var>animation</var> be a new <code><a href="#animation">Animation</a></code> object.

2.  <a id="ref-for-animation-set-the-timeline-of-an-animation"></a>

    <a id="ref-for-document-default-document-timeline⑤"></a>

    <a id="ref-for-document⑨"></a>

    <a id="ref-for-concept-document-window①"></a>

    <a id="ref-for-window①"></a>

    <a id="ref-for-current-global-object③"></a>

    Run the procedure to [set the timeline of an animation](#animation-set-the-timeline-of-an-animation) on <var>animation</var> passing <var>timeline</var> as the <var>new&#xA;timeline</var> or, if a <var>timeline</var> argument is missing, passing the [default document timeline](#document-default-document-timeline) of the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> [associated](https://html.spec.whatwg.org/multipage/browsers.html#concept-document-window) with the <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> that is the [current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).

3.  <a id="ref-for-animation-set-the-associated-effect-of-an-animation①"></a>

    Run the procedure to [set the associated effect of an animation](#animation-set-the-associated-effect-of-an-animation) on <var>animation</var> passing <var>source</var> as the <var>new&#xA;effect</var>.

<a id="dom-animation-animation-effect-timeline-effect"></a>`effect`  
<a id="ref-for-animation-associated-effect②⑨"></a>

<a id="ref-for-concept-animation⑥⑤"></a>

An optional value which, if not null, specifies the [associated effect](#animation-associated-effect) to assign to the newly created [animation](#concept-animation).

<a id="dom-animation-animation-effect-timeline-timeline"></a>`timeline`  
<a id="ref-for-timeline⑤⑥"></a>

<a id="ref-for-concept-animation⑥⑥"></a>

<a id="ref-for-document-default-document-timeline⑥"></a>

<a id="ref-for-document①⓪"></a>

<a id="ref-for-concept-document-window②"></a>

<a id="ref-for-window②"></a>

<a id="ref-for-current-global-object④"></a>

An optional value which, if present, specifies the [timeline](#timeline) with which to associate the newly-created [animation](#concept-animation). If missing, the [default document timeline](#document-default-document-timeline) of the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> [associated](https://html.spec.whatwg.org/multipage/browsers.html#concept-document-window) with the <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> that is the [current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object) is used.

<a id="ref-for-idl-DOMString①"></a>

<a id="dom-animation-id"></a>`id`, of type [DOMString](https://webidl.spec.whatwg.org/#idl-DOMString)

A string used to identify the animation.

<a id="ref-for-animationeffect②"></a>

<a id="dom-animation-effect"></a>`effect`, of type [AnimationEffect](#animationeffect), nullable

<a id="ref-for-animation-associated-effect③⓪"></a>

<a id="ref-for-animation-set-the-associated-effect-of-an-animation②"></a>

The [associated effect](#animation-associated-effect) of this animation. Setting this attribute updates the object’s <a id="ref-for-animation-associated-effect③①"></a>associated effect using the procedure to [set the associated effect of an animation](#animation-set-the-associated-effect-of-an-animation).

<a id="ref-for-animationtimeline④"></a>

<a id="dom-animation-timeline"></a>`timeline`, of type [AnimationTimeline](#animationtimeline), nullable

<a id="ref-for-timeline⑤⑦"></a>

<a id="ref-for-animation-set-the-timeline-of-an-animation①"></a>

The [timeline](#timeline) associated with this animation. Setting this attribute updates the object’s <a id="ref-for-timeline⑤⑧"></a>timeline using the procedure to [set the timeline of an animation](#animation-set-the-timeline-of-an-animation).

<a id="ref-for-idl-double⑥"></a>

<a id="dom-animation-starttime"></a>`startTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable

<a id="ref-for-animation-start-time⑤①"></a>

<a id="ref-for-set-the-start-time①"></a>

Returns the [start time](#animation-start-time) of this animation. Setting this attribute updates the <a id="ref-for-animation-start-time⑤②"></a>start time using the procedure to [set the start time](#set-the-start-time) of this object to the new value.

<a id="ref-for-idl-double⑦"></a>

<a id="dom-animation-currenttime"></a>`currentTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable

<a id="ref-for-animation-current-time⑤⓪"></a>

<a id="ref-for-animation-set-the-current-time③"></a>

The [current time](#animation-current-time) of this animation. Setting this attribute follows the procedure to [set the current time](#animation-set-the-current-time) of this object to the new value.

<a id="ref-for-idl-double⑧"></a>

<a id="dom-animation-playbackrate"></a>`playbackRate`, of type [double](https://webidl.spec.whatwg.org/#idl-double)

<a id="ref-for-playback-rate③⑨"></a>

<a id="ref-for-set-the-playback-rate②"></a>

The [playback rate](#playback-rate) of this animation. Setting this attribute follows the procedure to [set the playback rate](#set-the-playback-rate) of this object to the new value.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-playback-rate④⓪"></a>
>
> <a id="ref-for-dom-animation-playbackrate①"></a>
>
> Setting this attribute performs a synchronous update to the [playback rate](#playback-rate) meaning that it does not make any attempt to synchronize with the playback state of animations running on a separate process or thread. As a result, setting the <code><a href="#dom-animation-playbackrate">playbackRate</a></code> for an in-flight animation may cause it to jump.
>
> <a id="ref-for-playback-rate④①"></a>
>
> <a id="ref-for-dom-animation-updateplaybackrate①"></a>
>
> To set the [playback rate](#playback-rate) for an in-flight animation such that it smoothly updates, use the asynchronous <code><a href="#dom-animation-updateplaybackrate">updatePlaybackRate()</a></code> method.

<a id="ref-for-enumdef-animationplaystate①"></a>

<a id="dom-animation-playstate"></a>`playState`, of type [AnimationPlayState](#enumdef-animationplaystate), readonly

<a id="ref-for-animation-play-state⑨"></a>

The [play state](#animation-play-state) of this animation.

<a id="ref-for-enumdef-animationreplacestate①"></a>

<a id="dom-animation-replacestate"></a>`replaceState`, of type [AnimationReplaceState](#enumdef-animationreplacestate), readonly

<a id="ref-for-replace-state⑦"></a>

The [replace state](#replace-state) of this animation.

<a id="ref-for-idl-boolean①"></a>

<a id="dom-animation-pending"></a>`pending`, of type [boolean](https://webidl.spec.whatwg.org/#idl-boolean), readonly

<a id="ref-for-pending-play-task①⑧"></a>

<a id="ref-for-pending-pause-task①⑧"></a>

Returns true if this animation has a [pending play task](#pending-play-task) or a [pending pause task](#pending-pause-task).

<a id="ref-for-animation⑤"></a>

<a id="dom-animation-ready"></a>`ready`, of type Promise\<[Animation](#animation)\>, readonly

<a id="ref-for-current-ready-promise①⑦"></a>

Returns the [current ready promise](#current-ready-promise) for this object.

<a id="ref-for-animation⑥"></a>

<a id="dom-animation-finished"></a>`finished`, of type Promise\<[Animation](#animation)\>, readonly

<a id="ref-for-current-finished-promise①②"></a>

Returns the [current finished promise](#current-finished-promise) for this object.

<a id="ref-for-eventhandler③"></a>

<a id="dom-animation-onfinish"></a>`onfinish`, of type [EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)

<a id="ref-for-finish-event②"></a>

The event handler for the [finish event](#finish-event).

<a id="ref-for-eventhandler④"></a>

<a id="dom-animation-oncancel"></a>`oncancel`, of type [EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)

<a id="ref-for-cancel-event③"></a>

The event handler for the [cancel event](#cancel-event).

<a id="ref-for-eventhandler⑤"></a>

<a id="dom-animation-onremove"></a>`onremove`, of type [EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)

<a id="ref-for-remove-event①"></a>

The event handler for the [remove event](#remove-event).

<a id="dom-animation-cancel"></a>`void cancel()`  
<a id="ref-for-cancel-an-animation①"></a>

Clears all effects caused by this animation and aborts its playback by running the [cancel an animation](#cancel-an-animation) procedure for this object.

<a id="dom-animation-finish"></a>`void finish()`  
<a id="ref-for-associated-effect-end①⑦"></a>

<a id="ref-for-finish-an-animation②"></a>

Seeks the animation to the [associated effect end](#associated-effect-end) in the current direction by running the [finish an animation](#finish-an-animation) procedure for this object.

<a id="ref-for-invalidstateerror④"></a>

DOMException of type <code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>

<a id="ref-for-playback-rate④②"></a>

<a id="ref-for-associated-effect-end①⑧"></a>

Raised if this animation’s [playback rate](#playback-rate) is zero, or if this animation’s <a id="ref-for-playback-rate④③"></a>playback rate is \> zero and the [associated effect end](#associated-effect-end) is infinity.

<a id="dom-animation-play"></a>`void play()`  
<a id="ref-for-play-an-animation④"></a>

Begins or resumes playback of the animation by running the procedure to [play an animation](#play-an-animation) passing true as the value of the <var>auto-rewind</var> flag.

<a id="dom-animation-pause"></a>`void pause()`  
<a id="ref-for-pause-an-animation"></a>

Suspends the playback of this animation by running the procedure to [pause an animation](#pause-an-animation) for this object.

<a id="dom-animation-updateplaybackrate"></a>`void updatePlaybackRate(playbackRate)`  
<a id="ref-for-playback-rate④④"></a>

<a id="ref-for-seamlessly-update-the-playback-rate"></a>

<a id="ref-for-dom-animation-updateplaybackrate-playbackrate-playbackrate①"></a>

Performs an asynchronous update of the [playback rate](#playback-rate) of this animation by performing the [seamlessly update the playback rate](#seamlessly-update-the-playback-rate) procedure, passing <code><a href="#dom-animation-updateplaybackrate-playbackrate-playbackrate">playbackRate</a></code> as the <var>new&#xA;playback rate</var>.

<a id="dom-animation-updateplaybackrate-playbackrate-playbackrate"></a>`playbackRate`  
A finite real number specifying the updated playback rate to use.

<a id="dom-animation-reverse"></a>`void reverse()`  
<a id="ref-for-playback-rate④⑤"></a>

<a id="ref-for-reverse-an-animation"></a>

<a id="ref-for-dom-animation-play①"></a>

<a id="ref-for-animation-associated-effect③②"></a>

Inverts the [playback rate](#playback-rate) of this animation and plays it using the [reverse an animation](#reverse-an-animation) procedure for this object. As with [play()](#dom-animation-play), this method unpauses the animation and, if the animation has already finished playing in the reversed direction, seeks to the start of the [associated effect](#animation-associated-effect).

<a id="dom-animation-persist"></a>`void persist()`  
<a id="ref-for-replace-state⑧"></a>

<a id="ref-for-persisted-replace-state"></a>

Sets this animation’s [replace state](#replace-state) to [persisted](#persisted-replace-state).

<a id="dom-animation-commitstyles"></a>`void commitStyles()`  
<a id="ref-for-effect-value②②"></a>

<a id="ref-for-animation-effect⑧②"></a>

<a id="ref-for-keyframe-effect-effect-target①④"></a>

<a id="ref-for-commit-computed-styles"></a>

Writes the current [effect values](#effect-value) produced by this animation’s [animation effects](#animation-effect) to their corresponding [effect targets](#keyframe-effect-effect-target)' inline style using the [commit computed styles](#commit-computed-styles) procedure.

<a id="ref-for-style-change-event"></a>

Unlike most other methods defined on this interface, calling this method <em>does</em> trigger a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event) (see [§ 6.13 Model liveness](#model-liveness)).

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-commit-computed-styles①"></a>
>
> <a id="ref-for-effect-value②③"></a>
>
> <a id="ref-for-removed-replace-state⑤"></a>
>
> Since the procedure to [commit computed styles](#commit-computed-styles) includes the [effect values](#effect-value) for the animation even if it is [removed](#removed-replace-state), this method is useful for retaining the effect of an animation after it has been replaced (see [§ 5.5.2 Removing replaced animations](#removing-replaced-animations)) without retaining the actual animation.
>
> <a id="ref-for-animation-effect⑧③"></a>
>
> Note that the values committed are the <em>computed</em> values produced by the [animation effects](#animation-effect) at the time when this method is called. Since these values are computed values, they do not reflect to changes to context such as responding to changes to CSS variables or recalculating em units based on changes to the computed font-size in the way the values produced by a live animation would.
>
> <a id="ref-for-dom-animation-persist①"></a>
>
> In order to retain full fidelity of a filling animation’s result after it has been replaced (see [§ 5.5 Replacing animations](#replacing-animations)), the <code><a href="#dom-animation-persist">persist()</a></code> method may be used, but note that doing so will mean the animation continues to consume resources.

<a id="ref-for-concept-animation⑥⑦"></a>

To <a id="commit-computed-styles"></a>commit computed styles for an [animation](#concept-animation), <var>animation</var>:

1.  <a id="ref-for-ordered-set"></a>

    <a id="ref-for-keyframe-effect-effect-target①⑤"></a>

    <a id="ref-for-animation-effect⑧④"></a>

    <a id="ref-for-associated-with-an-animation①①"></a>

    Let <var>targets</var> be the [set](https://infra.spec.whatwg.org/#ordered-set) of all [effect targets](#keyframe-effect-effect-target) for [animation effects](#animation-effect) [associated](#associated-with-an-animation) with <var>animation</var>.

2.  <a id="ref-for-list-iterate"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>target</var> in <var>targets</var>:

    1.  <a id="ref-for-style-attribute"></a>

        <a id="ref-for-dfn-throw⑤"></a>

        <a id="ref-for-nomodificationallowederror"></a>

        <a id="ref-for-idl-DOMException④"></a>

        If <var>target</var> is not an element capable of having a [style attribute](https://drafts.csswg.org/css-style-attr/#style-attribute) [\[CSS-STYLE-ATTR\]](#biblio-css-style-attr) (for example, it is a pseudo-element or is an element in a document format for which style attributes are not defined) [throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a href="https://webidl.spec.whatwg.org/#nomodificationallowederror">NoModificationAllowedError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

    2.  <a id="ref-for-being-rendered"></a>

        <a id="ref-for-dfn-throw⑥"></a>

        <a id="ref-for-invalidstateerror⑤"></a>

        <a id="ref-for-idl-DOMException⑤"></a>

        If, after applying any pending style changes, <var>target</var> is not [being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered), [throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a href="https://webidl.spec.whatwg.org/#invalidstateerror">InvalidStateError</a></code>" <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> and abort these steps.

        > <strong data-conversion-semantic="issue">Issue</strong>
        >
        > <a id="issue-8f3b970c"></a>
        > <a id="ref-for-being-rendered①"></a>
        >
        > <a id="ref-for-propdef-display"></a>
        >
        > <a id="ref-for-connected"></a>
        >
        > The definition of [being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered) [\[HTML\]](#biblio-html) with regards to [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) is still [under discussion](https://github.com/whatwg/html/issues/1837). For the purpose of this procedure, we assume that an element with <a id="ref-for-propdef-display①"></a>display: contents that otherwise would have associated layout boxes (i.e. it is [connected](https://dom.spec.whatwg.org/#connected) and not part of a <a id="ref-for-propdef-display②"></a>display: none subtree) <em>is</em> being rendered.

    3.  <a id="ref-for-css-declaration-block"></a>

        <a id="ref-for-style-attribute①"></a>

        <a id="ref-for-concept-element-attribute-has"></a>

        <a id="ref-for-cssstyledeclaration-owner-node"></a>

        Let <var>inline style</var> be the result of getting the [CSS declaration block](https://www.w3.org/TR/cssom-1/#css-declaration-block) corresponding to <var>target</var>’s [style attribute](https://drafts.csswg.org/css-style-attr/#style-attribute). If <var>target</var> does not [have](https://dom.spec.whatwg.org/#concept-element-attribute-has) a <a id="ref-for-style-attribute②"></a>style attribute, let <var>inline style</var> be a new empty <a id="ref-for-css-declaration-block①"></a>CSS declaration block with the [owner node](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-owner-node) set to <var>target</var>.

    4.  <a id="ref-for-ordered-set①"></a>

        <a id="ref-for-target-property①②"></a>

        <a id="ref-for-animation-effect⑧⑤"></a>

        <a id="ref-for-associated-with-an-animation①②"></a>

        <a id="ref-for-keyframe-effect-effect-target①⑥"></a>

        Let <var>targeted properties</var> be the [set](https://infra.spec.whatwg.org/#ordered-set) of physical longhand properties that are a [target property](#target-property) for at least one [animation effect](#animation-effect) [associated](#associated-with-an-animation) with <var>animation</var> whose [effect target](#keyframe-effect-effect-target) is <var>target</var>.

    5.  For each property, <var>property</var>, in <var>targeted properties</var>:

        1.  <a id="ref-for-effect-stack①④"></a>

            Let <var>partialEffectStack</var> be a copy of the [effect stack](#effect-stack) for <var>property</var> on <var>target</var>.

        2.  <a id="ref-for-replace-state⑨"></a>

            <a id="ref-for-removed-replace-state⑥"></a>

            <a id="ref-for-animation-effect⑧⑥"></a>

            <a id="ref-for-associated-with-an-animation①③"></a>

            <a id="ref-for-keyframe-effect-effect-target①⑦"></a>

            <a id="ref-for-target-property①③"></a>

            If <var>animation</var>’s [replace state](#replace-state) is [removed](#removed-replace-state), add all [animation effects](#animation-effect) [associated](#associated-with-an-animation) with <var>animation</var> whose [effect target](#keyframe-effect-effect-target) is <var>target</var> and which include <var>property</var> as a [target property](#target-property) to <var>partialEffectStack</var>.

        3.  <a id="ref-for-animation-effect⑧⑦"></a>

            <a id="ref-for-associated-with-an-animation①④"></a>

            <a id="ref-for-concept-animation⑥⑧"></a>

            <a id="ref-for-animation-composite-order②"></a>

            Remove from <var>partialEffectStack</var> any [animation effects](#animation-effect) whose [associated](#associated-with-an-animation) [animation](#concept-animation) has a higher [composite order](#animation-composite-order) than <var>animation</var>.

        4.  Let <var>effect value</var> be the result of calculating the result of <var>partialEffectStack</var> for <var>property</var> using <var>target</var>’s computed style (see [§ 5.4.3 Calculating the result of an effect stack](#calculating-the-result-of-an-effect-stack)).

        5.  <a id="ref-for-set-a-css-declaration"></a>

            [Set a CSS declaration](https://www.w3.org/TR/cssom-1/#set-a-css-declaration) <var>property</var> for <var>effect value</var> in <var>inline style</var>.

    6.  <a id="ref-for-update-style-attribute-for"></a>

        [Update style attribute for](https://www.w3.org/TR/cssom-1/#update-style-attribute-for) <var>inline style</var>.

#### <a id="the-animationplaystate-enumeration"></a>6.4.1. The `AnimationPlayState` enumeration

<a id="enumdef-animationplaystate"></a>

<a id="dom-animationplaystate-idle"></a>

<a id="dom-animationplaystate-running"></a>

<a id="dom-animationplaystate-paused"></a>

<a id="dom-animationplaystate-finished"></a>

```text
enum AnimationPlayState { "idle", "running", "paused", "finished" };
```
`idle`  
<a id="ref-for-play-state-idle⑥"></a>

Corresponds to the [idle play state](#play-state-idle).

`running`  
<a id="ref-for-play-state-running③"></a>

Corresponds to the [running play state](#play-state-running).

`paused`  
<a id="ref-for-play-state-paused⑦"></a>

Corresponds to the [paused play state](#play-state-paused).

`finished`  
<a id="ref-for-play-state-finished①⑤"></a>

Corresponds to the [finished play state](#play-state-finished).

#### <a id="the-animationreplacestate-enumeration"></a>6.4.2. The `AnimationReplaceState` enumeration

<a id="enumdef-animationreplacestate"></a>

<a id="dom-animationreplacestate-active"></a>

<a id="dom-animationreplacestate-removed"></a>

<a id="dom-animationreplacestate-persisted"></a>

```text
enum AnimationReplaceState { "active", "removed", "persisted" };
```
`active`  
<a id="ref-for-active-replace-state②"></a>

Corresponds to the [active replace state](#active-replace-state).

`removed`  
<a id="ref-for-removed-replace-state⑦"></a>

Corresponds to the [removed replace state](#removed-replace-state).

`persisted`  
<a id="ref-for-persisted-replace-state①"></a>

Corresponds to the [persisted replace state](#persisted-replace-state).

### <a id="the-animationeffect-interface"></a>6.5. The `AnimationEffect` interface

<a id="ref-for-animation-effect⑧⑧"></a>

<a id="ref-for-animationeffect③"></a>

[Animation effects](#animation-effect) are represented in the Web Animations API by the abstract <code><a href="#animationeffect">AnimationEffect</a></code> interface.

<a id="ref-for-Exposed③"></a>

<a id="animationeffect"></a>

<a id="ref-for-dictdef-effecttiming"></a>

<a id="ref-for-dom-animationeffect-gettiming"></a>

<a id="ref-for-dictdef-computedeffecttiming"></a>

<a id="ref-for-dom-animationeffect-getcomputedtiming"></a>

<a id="ref-for-idl-undefined⑧"></a>

<a id="ref-for-dom-animationeffect-updatetiming"></a>

<a id="ref-for-dictdef-optionaleffecttiming"></a>

<a id="ref-for-dom-animationeffect-updatetiming-timing-timing"></a>

```text
[Exposed=Window]
interface AnimationEffect {
    EffectTiming         getTiming();
    ComputedEffectTiming getComputedTiming();
    undefined            updateTiming(optional OptionalEffectTiming timing = {});
};
```
> <strong data-conversion-semantic="note">Note</strong>
>
> In future, we may expose
>
> ```text
> any onupdate (double? progress,
> double currentIteration, Animatable? target, any
> underlyingValue)
> ```
>
> so that the animation effects can be driven apart from the timing model.

<a id="dom-animationeffect-gettiming"></a>`getTiming()`  
<a id="ref-for-animation-effect⑧⑨"></a>

Returns the specified timing properties for this [animation effect](#animation-effect).

<a id="ref-for-dictdef-effecttiming①"></a>

<a id="ref-for-dictdef-effecttiming②"></a>

For the correspondence between the members of the returned <code><a href="#dictdef-effecttiming">EffectTiming</a></code> object and properties of the [timing model](#timing-model), see the the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> interface.

<a id="dom-animationeffect-getcomputedtiming"></a>`getComputedTiming()`  
<a id="ref-for-animation-effect⑨⓪"></a>

Returns the calculated timing properties for this [animation effect](#animation-effect).

<a id="ref-for-dom-animationeffect-gettiming①"></a>

<a id="ref-for-dom-animationeffect-getcomputedtiming①"></a>

Although some of the attributes of the object returned by <code><a href="#dom-animationeffect-gettiming">getTiming()</a></code> and <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code> are common, their values may differ in the following ways:

- <a id="ref-for-dom-effecttiming-duration"></a>

  <a id="ref-for-dom-animationeffect-gettiming②"></a>

  <a id="ref-for-dom-animationeffect-getcomputedtiming②"></a>

  <a id="ref-for-iteration-duration①③"></a>

  <a id="ref-for-dom-effecttiming-duration①"></a>

  <a id="ref-for-dictdef-effecttiming③"></a>

  <code><a href="#dom-effecttiming-duration">duration</a></code> – while <code><a href="#dom-animationeffect-gettiming">getTiming()</a></code> may return the string `auto`, <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code> must return a number corresponding to the calculated value of the [iteration duration](#iteration-duration) as defined in the description of the <code><a href="#dom-effecttiming-duration">duration</a></code> member of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> interface.

  In this level of the specification, that simply means that an `auto` value is replaced by zero.

- <a id="ref-for-dom-effecttiming-fill"></a>

  <a id="ref-for-dom-animationeffect-gettiming③"></a>

  <a id="ref-for-dom-animationeffect-getcomputedtiming③"></a>

  <a id="ref-for-enumdef-fillmode"></a>

  <a id="ref-for-dom-effecttiming-fill①"></a>

  <a id="ref-for-dictdef-effecttiming④"></a>

  <code><a href="#dom-effecttiming-fill">fill</a></code> – likewise, while <code><a href="#dom-animationeffect-gettiming">getTiming()</a></code> may return the string `auto`, <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code> must return the specific [FillMode](#enumdef-fillmode) used for timing calculations as defined in the description of the <code><a href="#dom-effecttiming-fill">fill</a></code> member of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> interface.

  <a id="ref-for-enumdef-fillmode①"></a>

  In this level of the specification, that simply means that an `auto` value is replaced by the `none` [FillMode](#enumdef-fillmode).

<a id="ref-for-dom-animationeffect-getcomputedtiming④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is likely that other timing members may be extended in future to include `auto`-like values. When performing timing calculations, authors are encouraged to use <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code> where possible to avoid incompatibility should the range or type of allowed specified values be changed.

<a id="ref-for-dom-animationeffect-gettiming④"></a>

<a id="ref-for-dom-animationeffect-getcomputedtiming⑤"></a>

<a id="ref-for-dictdef-computedeffecttiming①"></a>

In addition to possible differences in the values returned, compared to <code><a href="#dom-animationeffect-gettiming">getTiming()</a></code>, <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code> returns additional timing information as defined by the <code><a href="#dictdef-computedeffecttiming">ComputedEffectTiming</a></code> dictionary.

<a id="dom-animationeffect-updatetiming"></a>`updateTiming(timing)`  
<a id="ref-for-animation-effect⑨①"></a>

<a id="ref-for-update-the-timing-properties-of-an-animation-effect"></a>

<a id="ref-for-dom-animationeffect-updatetiming-timing-timing①"></a>

Updates the specified timing properties of this [animation effect](#animation-effect) by performing the procedure to [update the timing properties of an animation effect](#update-the-timing-properties-of-an-animation-effect) passing the <code><a href="#dom-animationeffect-updatetiming-timing-timing">timing</a></code> parameter as <var>input</var>.

<a id="ref-for-dictdef-optionaleffecttiming①"></a>

<a id="dom-animationeffect-updatetiming-timing-timing"></a><code>optional <code><a href="#dictdef-optionaleffecttiming">OptionalEffectTiming</a></code> timing</code>

<a id="ref-for-map-exists"></a>

<a id="ref-for-dom-animationeffect-updatetiming-timing-timing②"></a>

The timing properties to update. The timing properties corresponding to any members that do not [exist](https://infra.spec.whatwg.org/#map-exists) on <code><a href="#dom-animationeffect-updatetiming-timing-timing">timing</a></code> will <em>not</em> be modified.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-178c6cfa"></a> The `remove()` method can be used to remove an effect from either its parent group or animation. Should we keep it in level 1 and define it simply as removing the animation effect from its animation? [\[Issue \#2082\]](https://github.com/w3c/csswg-drafts/issues/2082)

#### <a id="the-effecttiming-dictionaries"></a>6.5.1. The `EffectTiming` and `OptionalEffectTiming` dictionaries

<a id="ref-for-dictdef-effecttiming⑤"></a>

<a id="ref-for-animationeffect④"></a>

The <code><a href="#dictdef-effecttiming">EffectTiming</a></code> dictionary represents the timing properties of an <code><a href="#animationeffect">AnimationEffect</a></code>.

<a id="ref-for-dictdef-optionaleffecttiming②"></a>

<a id="ref-for-dictdef-effecttiming⑥"></a>

<a id="ref-for-map-exists①"></a>

<a id="ref-for-dom-animationeffect-updatetiming①"></a>

<a id="ref-for-animationeffect⑤"></a>

<a id="ref-for-animation-effect⑨②"></a>

The <code><a href="#dictdef-optionaleffecttiming">OptionalEffectTiming</a></code> dictionary is a variant of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> dictionary that allows some members to not [exist](https://infra.spec.whatwg.org/#map-exists). This is used by the <code><a href="#dom-animationeffect-updatetiming">updateTiming()</a></code> method of the <code><a href="#animationeffect">AnimationEffect</a></code> interface to perform a delta update to the timing properties of an [animation effect](#animation-effect).

<a id="dictdef-effecttiming"></a>

<a id="ref-for-idl-double⑨"></a>

<a id="ref-for-dom-effecttiming-delay"></a>

<a id="ref-for-idl-double①⓪"></a>

<a id="ref-for-dom-effecttiming-enddelay"></a>

<a id="ref-for-enumdef-fillmode②"></a>

<a id="ref-for-dom-effecttiming-fill②"></a>

<a id="ref-for-idl-double①①"></a>

<a id="ref-for-dom-effecttiming-iterationstart"></a>

<a id="ref-for-idl-unrestricted-double"></a>

<a id="ref-for-dom-effecttiming-iterations"></a>

<a id="ref-for-idl-unrestricted-double①"></a>

<a id="ref-for-idl-DOMString②"></a>

<a id="ref-for-dom-effecttiming-duration②"></a>

<a id="ref-for-enumdef-playbackdirection"></a>

<a id="ref-for-dom-effecttiming-direction"></a>

<a id="ref-for-idl-DOMString③"></a>

<a id="ref-for-dom-effecttiming-easing"></a>

<a id="dictdef-optionaleffecttiming"></a>

<a id="ref-for-idl-double①②"></a>

<a id="ref-for-dom-optionaleffecttiming-delay"></a>

<a id="ref-for-idl-double①③"></a>

<a id="ref-for-dom-optionaleffecttiming-enddelay"></a>

<a id="ref-for-enumdef-fillmode③"></a>

<a id="ref-for-dom-optionaleffecttiming-fill"></a>

<a id="ref-for-idl-double①④"></a>

<a id="ref-for-dom-optionaleffecttiming-iterationstart"></a>

<a id="ref-for-idl-unrestricted-double②"></a>

<a id="ref-for-dom-optionaleffecttiming-iterations"></a>

<a id="ref-for-idl-unrestricted-double③"></a>

<a id="ref-for-idl-DOMString④"></a>

<a id="ref-for-dom-optionaleffecttiming-duration"></a>

<a id="ref-for-enumdef-playbackdirection①"></a>

<a id="ref-for-dom-optionaleffecttiming-direction"></a>

<a id="ref-for-idl-DOMString⑤"></a>

<a id="ref-for-dom-optionaleffecttiming-easing"></a>

```text
dictionary EffectTiming {
    double                             delay = 0;
    double                             endDelay = 0;
    FillMode                           fill = "auto";
    double                             iterationStart = 0.0;
    unrestricted double                iterations = 1.0;
    (unrestricted double or DOMString) duration = "auto";
    PlaybackDirection                  direction = "normal";
    DOMString                          easing = "linear";
};

dictionary OptionalEffectTiming {
    double                             delay;
    double                             endDelay;
    FillMode                           fill;
    double                             iterationStart;
    unrestricted double                iterations;
    (unrestricted double or DOMString) duration;
    PlaybackDirection                  direction;
    DOMString                          easing;
};
```
<a id="ref-for-idl-double①⑤"></a>

<a id="dom-effecttiming-delay"></a>`delay`, of type [double](https://webidl.spec.whatwg.org/#idl-double), defaulting to `0`<a id="dom-optionaleffecttiming-delay"></a>``

<a id="ref-for-start-delay①⑥"></a>

<a id="ref-for-animation-start-time⑤③"></a>

<a id="ref-for-concept-animation⑥⑨"></a>

<a id="ref-for-active-interval①⑥"></a>

The [start delay](#start-delay) which represents the number of milliseconds from the [start time](#animation-start-time) of the associated [animation](#concept-animation) to the start of the [active interval](#active-interval).

<a id="ref-for-idl-double①⑥"></a>

<a id="dom-effecttiming-enddelay"></a>`endDelay`, of type [double](https://webidl.spec.whatwg.org/#idl-double), defaulting to `0`<a id="dom-optionaleffecttiming-enddelay"></a>``

<a id="ref-for-end-delay④"></a>

<a id="ref-for-animation-effect⑨③"></a>

<a id="ref-for-active-interval①⑦"></a>

<a id="ref-for-end-time⑧"></a>

The [end delay](#end-delay) which represents the number of milliseconds from the end of an [animation effect](#animation-effect)’s [active interval](#active-interval) until its [end time](#end-time).

<a id="ref-for-enumdef-fillmode④"></a>

<a id="dom-effecttiming-fill"></a>`fill`, of type [FillMode](#enumdef-fillmode), defaulting to `"auto"`<a id="dom-optionaleffecttiming-fill"></a>``

<a id="ref-for-fill-mode①⓪"></a>

<a id="ref-for-animation-effect⑨④"></a>

<a id="ref-for-active-interval①⑧"></a>

The [fill mode](#fill-mode) which defines the behavior of the [animation effect](#animation-effect) outside its [active interval](#active-interval).

<a id="ref-for-fill-mode①①"></a>

When performing timing calculations the special string value `auto` is expanded to one of the [fill modes](#fill-mode) recognized by the timing model as follows,

<a id="ref-for-keyframe-effect③⓪"></a>

<a id="ref-for-animation-effect⑨⑤"></a>

If the [animation effect](#animation-effect) to which the fill mode is being is applied is a [keyframe effect](#keyframe-effect),

<a id="ref-for-fill-mode①②"></a>

Use none as the [fill mode](#fill-mode).

Otherwise,

<a id="ref-for-fill-mode①③"></a>

Use both as the [fill mode](#fill-mode).

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> As described in [§ 4.6 Fill behavior](#fill-behavior), authors are discouraged from using indefinitely filling animations.

<a id="ref-for-idl-double①⑦"></a>

<a id="dom-effecttiming-iterationstart"></a>`iterationStart`, of type [double](https://webidl.spec.whatwg.org/#idl-double), defaulting to `0.0`<a id="dom-optionaleffecttiming-iterationstart"></a>``

<a id="ref-for-animation-effect⑨⑥"></a>

<a id="ref-for-iteration-start⑨"></a>

The [animation effect](#animation-effect)’s [iteration start](#iteration-start) property which is a finite real number greater than or equal to zero representing the iteration index at which the animation effect begins and its progress through that iteration.

For example, a value of 0.5 indicates that the animation effect begins half way through its first iteration. A value of 1.2 indicates the animation effect begins 20% of the way through its second iteration.

<a id="ref-for-dom-effecttiming-iterations①"></a>

<a id="ref-for-dom-effecttiming-iterationstart①"></a>

<a id="ref-for-dom-effecttiming-iterationstart②"></a>

<a id="ref-for-dom-effecttiming-iterations②"></a>

<a id="ref-for-iteration-interval②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the value of <code><a href="#dom-effecttiming-iterations">iterations</a></code> is effectively <em>added</em> to the <code><a href="#dom-effecttiming-iterationstart">iterationStart</a></code> such that an animation effect with an <code><a href="#dom-effecttiming-iterationstart">iterationStart</a></code> of "0.5" and <code><a href="#dom-effecttiming-iterations">iterations</a></code> of "2" will still repeat twice however it will begin and end half-way through its [iteration interval](#iteration-interval).
>
> <a id="ref-for-dom-effecttiming-iterationstart③"></a>
>
> <a id="ref-for-iteration-composite-operation①"></a>
>
> <a id="ref-for-current-iteration④"></a>
>
> <code><a href="#dom-effecttiming-iterationstart">iterationStart</a></code> values greater than or equal to one are typically only useful in combination with an animation effect that has an [iteration composite operation](https://drafts.csswg.org/web-animations-2/#iteration-composite-operation) of accumulate or when the [current iteration](#current-iteration) index is otherwise significant.

<a id="ref-for-idl-unrestricted-double④"></a>

<a id="dom-effecttiming-iterations"></a>`iterations`, of type [unrestricted double](https://webidl.spec.whatwg.org/#idl-unrestricted-double), defaulting to `1.0`<a id="dom-optionaleffecttiming-iterations"></a>``

<a id="ref-for-animation-effect⑨⑦"></a>

<a id="ref-for-iteration-count①⑤"></a>

The [animation effect](#animation-effect)’s [iteration count](#iteration-count) property which is a real number greater than or equal to zero (including positive infinity) representing the number of times to the animation effect repeats.

<a id="ref-for-animation-effect⑨⑧"></a>

This may be set to `+Infinity` to cause the [animation effect](#animation-effect) to repeat forever (unless the duration of the effect is zero, in which case it will finish immediately).

<a id="dom-effecttiming-duration"></a>`duration`, of type `(unrestricted double or DOMString)`, defaulting to `"auto"`<a id="dom-optionaleffecttiming-duration"></a>``

<a id="ref-for-iteration-duration①④"></a>

<a id="ref-for-animation-effect⑨⑨"></a>

The [iteration duration](#iteration-duration) which is a real number greater than or equal to zero (including positive infinity) representing the time taken to complete a single iteration of the [animation effect](#animation-effect).

<a id="ref-for-dom-effecttiming-duration③"></a>

<a id="ref-for-dom-animationeffect-getcomputedtiming⑥"></a>

<a id="ref-for-dom-effecttiming-duration④"></a>

<a id="ref-for-dom-animationeffect-gettiming⑤"></a>

In this level of this specification, the string value `auto` is treated as the value zero for the purpose of timing model calculations and for the result of the <code><a href="#dom-effecttiming-duration">duration</a></code> member returned from <code><a href="#dom-animationeffect-getcomputedtiming">getComputedTiming()</a></code>. If the author specifies the `auto` value user agents must, however, return `auto` for the <code><a href="#dom-effecttiming-duration">duration</a></code> member returned from <code><a href="#dom-animationeffect-gettiming">getTiming()</a></code>.

This is a forwards-compatibility measure since a future level of this specification is expected to introduce group effects where the `auto` value expands to include the duration of the child effects.

<a id="ref-for-enumdef-playbackdirection②"></a>

<a id="dom-effecttiming-direction"></a>`direction`, of type [PlaybackDirection](#enumdef-playbackdirection), defaulting to `"normal"`<a id="dom-optionaleffecttiming-direction"></a>``

<a id="ref-for-playback-direction⑤"></a>

<a id="ref-for-animation-effect①⓪⓪"></a>

The [playback direction](#playback-direction) of the [animation effect](#animation-effect) which defines whether playback proceeds forwards, backwards, or alternates on each iteration.

<a id="ref-for-idl-DOMString⑥"></a>

<a id="dom-effecttiming-easing"></a>`easing`, of type [DOMString](https://webidl.spec.whatwg.org/#idl-DOMString), defaulting to `"linear"`<a id="dom-optionaleffecttiming-easing"></a>``

<a id="ref-for-easing-function⑧"></a>

The [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) used to scale the time to produce easing effects.

<a id="ref-for-typedef-easing-function"></a>

The syntax of the string is defined by the [\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function) production [\[CSS-EASING-1\]](#biblio-css-easing-1).

#### <a id="the-fillmode-enumeration"></a>6.5.2. The `FillMode` enumeration

<a id="enumdef-fillmode"></a>

<a id="dom-fillmode-none"></a>

<a id="dom-fillmode-forwards"></a>

<a id="dom-fillmode-backwards"></a>

<a id="dom-fillmode-both"></a>

<a id="dom-fillmode-auto"></a>

```text
enum FillMode { "none", "forwards", "backwards", "both", "auto" };
```
`none`  
No fill.

`forwards`  
Fill forwards.

`backwards`  
Fill backwards.

`both`  
Fill backwards and forwards.

`auto`  
<a id="ref-for-animation-effect①⓪①"></a>

No fill. In a subsequent level of this specification, this may produce different behavior for other types of [animation effects](#animation-effect).

#### <a id="the-playbackdirection-enumeration"></a>6.5.3. The `PlaybackDirection` enumeration

<a id="enumdef-playbackdirection"></a>

<a id="dom-playbackdirection-normal"></a>

<a id="dom-playbackdirection-reverse"></a>

<a id="dom-playbackdirection-alternate"></a>

<a id="dom-playbackdirection-alternate-reverse"></a>

```text
enum PlaybackDirection { "normal", "reverse", "alternate", "alternate-reverse" };
```
`normal`  
All iterations are played as specified.

`reverse`  
All iterations are played in the reverse direction from the order they are specified.

`alternate`  
Even iterations are played as specified, odd iterations are played in the reverse direction from the order they are specified.

`alternate-reverse`  
Even iterations are played in the reverse direction from the order they are specified, odd iterations are played as specified.

#### <a id="updating-animationeffect-timing"></a>6.5.4. Updating the timing of an `AnimationEffect`

<a id="ref-for-dictdef-effecttiming⑦"></a>

<a id="ref-for-dictdef-optionaleffecttiming③"></a>

To <a id="update-the-timing-properties-of-an-animation-effect"></a>update the timing properties of an animation effect, <var>effect</var>, from an <code><a href="#dictdef-effecttiming">EffectTiming</a></code> or <code><a href="#dictdef-optionaleffecttiming">OptionalEffectTiming</a></code> object, <var>input</var>, perform the following steps:

1.  <a id="ref-for-dom-effecttiming-iterationstart④"></a>

    <a id="ref-for-map-exists②"></a>

    <a id="ref-for-dfn-throw⑦"></a>

    If the <code><a href="#dom-effecttiming-iterationstart">iterationStart</a></code> member of <var>input</var> [exists](https://infra.spec.whatwg.org/#map-exists) and is less than zero, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

    <a id="ref-for-EnforceRange"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The reason for using a TypeError rather than a RangeError is to mirror the behavior of WebIDL’s [\[EnforceRange\]](https://webidl.spec.whatwg.org/#EnforceRange) annotation should that annotation be able to be used with floating-point values in the future.

2.  <a id="ref-for-dom-effecttiming-iterations③"></a>

    <a id="ref-for-map-exists③"></a>

    <a id="ref-for-dfn-throw⑧"></a>

    If the <code><a href="#dom-effecttiming-iterations">iterations</a></code> member of <var>input</var> [exists](https://infra.spec.whatwg.org/#map-exists), and is less than zero or is the value `NaN`, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

3.  <a id="ref-for-dom-effecttiming-duration⑤"></a>

    <a id="ref-for-map-exists④"></a>

    <a id="ref-for-dfn-throw⑨"></a>

    If the <code><a href="#dom-effecttiming-duration">duration</a></code> member of <var>input</var> [exists](https://infra.spec.whatwg.org/#map-exists), and is less than zero or is the value `NaN`, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

4.  <a id="ref-for-dom-effecttiming-easing①"></a>

    <a id="ref-for-map-exists⑤"></a>

    <a id="ref-for-typedef-easing-function①"></a>

    <a id="ref-for-dfn-throw①⓪"></a>

    If the <code><a href="#dom-effecttiming-easing">easing</a></code> member of <var>input</var> [exists](https://infra.spec.whatwg.org/#map-exists) but cannot be parsed using the [\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function) production [\[CSS-EASING-1\]](#biblio-css-easing-1), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

5.  <a id="ref-for-map-exists⑥"></a>

    Assign each member that [exists](https://infra.spec.whatwg.org/#map-exists) in <var>input</var> to the corresponding timing property of <var>effect</var> as follows:

    - <a id="ref-for-dom-effecttiming-delay①"></a>

      <a id="ref-for-start-delay①⑦"></a>

      <code><a href="#dom-effecttiming-delay">delay</a></code> → [start delay](#start-delay)

    - <a id="ref-for-dom-effecttiming-enddelay①"></a>

      <a id="ref-for-end-delay⑤"></a>

      <code><a href="#dom-effecttiming-enddelay">endDelay</a></code> → [end delay](#end-delay)

    - <a id="ref-for-dom-effecttiming-fill③"></a>

      <a id="ref-for-fill-mode①④"></a>

      <code><a href="#dom-effecttiming-fill">fill</a></code> → [fill mode](#fill-mode)

    - <a id="ref-for-dom-effecttiming-iterationstart⑤"></a>

      <a id="ref-for-iteration-start①⓪"></a>

      <code><a href="#dom-effecttiming-iterationstart">iterationStart</a></code> → [iteration start](#iteration-start)

    - <a id="ref-for-dom-effecttiming-iterations④"></a>

      <a id="ref-for-iteration-count①⑥"></a>

      <code><a href="#dom-effecttiming-iterations">iterations</a></code> → [iteration count](#iteration-count)

    - <a id="ref-for-dom-effecttiming-duration⑥"></a>

      <a id="ref-for-iteration-duration①⑤"></a>

      <code><a href="#dom-effecttiming-duration">duration</a></code> → [iteration duration](#iteration-duration)

    - <a id="ref-for-dom-effecttiming-direction①"></a>

      <a id="ref-for-playback-direction⑥"></a>

      <code><a href="#dom-effecttiming-direction">direction</a></code> → [playback direction](#playback-direction)

    - <a id="ref-for-dom-effecttiming-easing②"></a>

      <a id="ref-for-easing-function⑨"></a>

      <code><a href="#dom-effecttiming-easing">easing</a></code> → [timing function](https://www.w3.org/TR/css-easing-1/#easing-function)

#### <a id="the-computedeffecttiming-dictionary"></a>6.5.5. The `ComputedEffectTiming` dictionary

<a id="ref-for-dictdef-computedeffecttiming②"></a>

Timing properties calculated by the timing model are exposed using <code><a href="#dictdef-computedeffecttiming">ComputedEffectTiming</a></code> dictionary objects.

<a id="dictdef-computedeffecttiming"></a>

<a id="ref-for-dictdef-effecttiming⑧"></a>

<a id="ref-for-idl-unrestricted-double⑤"></a>

<a id="ref-for-dom-computedeffecttiming-endtime"></a>

<a id="ref-for-idl-unrestricted-double⑥"></a>

<a id="ref-for-dom-computedeffecttiming-activeduration"></a>

<a id="ref-for-idl-double①⑧"></a>

<a id="ref-for-dom-computedeffecttiming-localtime"></a>

<a id="ref-for-idl-double①⑨"></a>

<a id="ref-for-dom-computedeffecttiming-progress"></a>

<a id="ref-for-idl-unrestricted-double⑦"></a>

<a id="ref-for-dom-computedeffecttiming-currentiteration"></a>

```text
dictionary ComputedEffectTiming : EffectTiming {
    unrestricted double  endTime;
    unrestricted double  activeDuration;
    double?              localTime;
    double?              progress;
    unrestricted double? currentIteration;
};
```
<a id="ref-for-idl-unrestricted-double⑧"></a>

<a id="dom-computedeffecttiming-endtime"></a>`endTime`, of type [unrestricted double](https://webidl.spec.whatwg.org/#idl-unrestricted-double)

<a id="ref-for-end-time⑨"></a>

<a id="ref-for-animation-effect①⓪②"></a>

<a id="ref-for-local-time②①"></a>

<a id="ref-for-concept-animation⑦⓪"></a>

<a id="ref-for-animation-start-time⑤④"></a>

<a id="ref-for-associated-with-an-animation①⑤"></a>

<a id="ref-for-end-delay⑥"></a>

The [end time](#end-time) of the [animation effect](#animation-effect) expressed in milliseconds since zero [local time](#local-time) (that is, since the associated [animation](#concept-animation)’s [start time](#animation-start-time) if this <a id="ref-for-animation-effect①⓪③"></a>animation effect is [associated with an animation](#associated-with-an-animation)). This corresponds to the end of the <a id="ref-for-animation-effect①⓪④"></a>animation effect’s active interval plus any [end delay](#end-delay).

<a id="ref-for-idl-unrestricted-double⑨"></a>

<a id="dom-computedeffecttiming-activeduration"></a>`activeDuration`, of type [unrestricted double](https://webidl.spec.whatwg.org/#idl-unrestricted-double)

<a id="ref-for-active-duration①⑧"></a>

<a id="ref-for-animation-effect①⓪⑤"></a>

The [active duration](#active-duration) of this [animation effect](#animation-effect).

<a id="ref-for-idl-double②⓪"></a>

<a id="dom-computedeffecttiming-localtime"></a>`localTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable

<a id="ref-for-local-time②②"></a>

<a id="ref-for-animation-effect①⓪⑥"></a>

The [local time](#local-time) of this [animation effect](#animation-effect).

<a id="ref-for-animation-effect①⓪⑦"></a>

<a id="ref-for-associated-with-an-animation①⑥"></a>

This will be `null` if this [animation effect](#animation-effect) is not [associated with an animation](#associated-with-an-animation).

<a id="ref-for-idl-double②①"></a>

<a id="dom-computedeffecttiming-progress"></a>`progress`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable

<a id="ref-for-iteration-progress①②"></a>

<a id="ref-for-animation-effect①⓪⑧"></a>

The current [iteration progress](#iteration-progress) of this [animation effect](#animation-effect).

<a id="ref-for-idl-unrestricted-double①⓪"></a>

<a id="dom-computedeffecttiming-currentiteration"></a>`currentIteration`, of type [unrestricted double](https://webidl.spec.whatwg.org/#idl-unrestricted-double), nullable

<a id="ref-for-current-iteration⑤"></a>

The [current iteration](#current-iteration) index beginning with zero for the first iteration.

In most cases this will be a (positive) integer. However, for a zero-duration animation that repeats infinite times, the value will be positive Infinity.

<a id="ref-for-unresolved①①③"></a>

<a id="ref-for-current-iteration⑥"></a>

As with [unresolved](#unresolved) times, an unresolved [current iteration](#current-iteration) is represented by a null value.

### <a id="the-keyframeeffect-interface"></a>6.6. The `KeyframeEffect` interface

<a id="ref-for-keyframe-effect③①"></a>

<a id="ref-for-keyframeeffect"></a>

[Keyframe effects](#keyframe-effect) are represented by the <code><a href="#keyframeeffect">KeyframeEffect</a></code> interface.

<a id="ref-for-Exposed④"></a>

<a id="keyframeeffect"></a>

<a id="ref-for-animationeffect⑥"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect"></a>

<a id="ref-for-element③"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-target-keyframes-options-target"></a>

<a id="ref-for-idl-object"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-target-keyframes-options-keyframes"></a>

<a id="ref-for-idl-unrestricted-double①①"></a>

<a id="ref-for-dictdef-keyframeeffectoptions"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-target-keyframes-options-options"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-source"></a>

<a id="ref-for-keyframeeffect①"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-source-source"></a>

<a id="ref-for-element④"></a>

<a id="ref-for-dom-keyframeeffect-target"></a>

<a id="ref-for-cssomstring"></a>

<a id="ref-for-dom-keyframeeffect-pseudoelement"></a>

<a id="ref-for-enumdef-compositeoperation"></a>

<a id="ref-for-dom-keyframeeffect-composite"></a>

<a id="ref-for-idl-sequence"></a>

<a id="ref-for-idl-object①"></a>

<a id="ref-for-dom-keyframeeffect-getkeyframes"></a>

<a id="ref-for-idl-undefined⑨"></a>

<a id="ref-for-dom-keyframeeffect-setkeyframes"></a>

<a id="ref-for-idl-object②"></a>

<a id="dom-keyframeeffect-setkeyframes-keyframes-keyframes"></a>

```text
[Exposed=Window]
interface KeyframeEffect : AnimationEffect {
    constructor(Element? target,
                object? keyframes,
                optional (unrestricted double or KeyframeEffectOptions) options = {});
    constructor(KeyframeEffect source);
    attribute Element?           target;
    attribute CSSOMString?       pseudoElement;
    attribute CompositeOperation composite;
    sequence<object> getKeyframes();
    undefined        setKeyframes(object? keyframes);
};
```
<a id="dom-keyframeeffect-keyframeeffect"></a>`  KeyframeEffect (target, keyframes, options) `  
<a id="ref-for-keyframeeffect②"></a>

Creates a new <code><a href="#keyframeeffect">KeyframeEffect</a></code> object using the following procedure:

1.  <a id="ref-for-keyframeeffect③"></a>

    Create a new <code><a href="#keyframeeffect">KeyframeEffect</a></code> object, <var>effect</var>.

2.  <a id="ref-for-effect-target-target-element④"></a>

    Set the [target element](#effect-target-target-element) of <var>effect</var> to <var>target</var>.

3.  <a id="ref-for-effect-target-target-pseudo-selector②"></a>

    Set the [target pseudo-selector](#effect-target-target-pseudo-selector) to the result corresponding to the first matching condition from below.

    <a id="ref-for-dom-keyframeeffectoptions-pseudoelement"></a>

    <a id="ref-for-dictdef-keyframeeffectoptions①"></a>

    If <var>options</var> is a <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> object with a <code><a href="#dom-keyframeeffectoptions-pseudoelement">pseudoElement</a></code> property,

    <a id="ref-for-effect-target-target-pseudo-selector③"></a>

    <a id="ref-for-dom-keyframeeffectoptions-pseudoelement①"></a>

    Set the [target pseudo-selector](#effect-target-target-pseudo-selector) to the value of the <code><a href="#dom-keyframeeffectoptions-pseudoelement">pseudoElement</a></code> property.

    <a id="ref-for-dom-keyframeeffect-pseudoelement①"></a>

    When assigning this property, the error-handling defined for the <code><a href="#dom-keyframeeffect-pseudoelement">pseudoElement</a></code> setter on the interface is applied. If the setter requires an exception to be thrown, this procedure must throw the same exception and abort all further steps.

    Otherwise,

    <a id="ref-for-effect-target-target-pseudo-selector④"></a>

    Set the [target pseudo-selector](#effect-target-target-pseudo-selector) to `null`.

4.  Let <var>timing input</var> be the result corresponding to the first matching condition from below.

    <a id="ref-for-dictdef-keyframeeffectoptions②"></a>

    If <var>options</var> is a <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> object,

    Let <var>timing input</var> be <var>options</var>.

    Otherwise (if <var>options</var> is a `double`),

    <a id="ref-for-dictdef-effecttiming⑨"></a>

    <a id="ref-for-dom-effecttiming-duration⑦"></a>

    Let <var>timing input</var> be a new <code><a href="#dictdef-effecttiming">EffectTiming</a></code> object with all members set to their default values and <code><a href="#dom-effecttiming-duration">duration</a></code> set to <var>options</var>.

5.  <a id="ref-for-update-the-timing-properties-of-an-animation-effect①"></a>

    Call the procedure to [update the timing properties of an animation effect](#update-the-timing-properties-of-an-animation-effect) of <var>effect</var> from <var>timing input</var>.

    If that procedure causes an exception to be thrown, propagate the exception and abort this procedure.

6.  <a id="ref-for-dictdef-keyframeeffectoptions③"></a>

    <a id="ref-for-dom-keyframeeffect-composite①"></a>

    If <var>options</var> is a <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> object, assign the <code><a href="#dom-keyframeeffect-composite">composite</a></code> property of <var>effect</var> to the corresponding value from <var>options</var>.

    <a id="ref-for-keyframeeffect④"></a>

    <a id="ref-for-dfn-throw①①"></a>

    When assigning this property, the error-handling defined for the corresponding setter on the <code><a href="#keyframeeffect">KeyframeEffect</a></code> interface is applied. If the setter requires an exception to be thrown for the value specified by <var>options</var>, this procedure must [throw](https://webidl.spec.whatwg.org/#dfn-throw) the same exception and abort all further steps.

7.  <a id="ref-for-keyframe④④"></a>

    <a id="ref-for-dom-keyframeeffect-setkeyframes①"></a>

    Initialize the set of [keyframes](#keyframe) by performing the procedure defined for <code><a href="#dom-keyframeeffect-setkeyframes">setKeyframes()</a></code> passing <var>keyframes</var> as the input.

<a id="ref-for-element⑤"></a>

<a id="dom-keyframeeffect-keyframeeffect-target-keyframes-options-target"></a><code><code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>? target</code>

<a id="ref-for-effect-target-target-element⑤"></a>

The [target element](#effect-target-target-element). This may be `null` for animations that do not target a specific element.

<a id="dom-keyframeeffect-keyframeeffect-target-keyframes-options-keyframes"></a>`object? keyframes`

<a id="ref-for-keyframe④⑤"></a>

The set of [keyframes](#keyframe) to use. The format and processing of this argument is defined in [§ 6.6.3 Processing a keyframes argument](#processing-a-keyframes-argument).

<a id="ref-for-dictdef-keyframeeffectoptions④"></a>

<a id="dom-keyframeeffect-keyframeeffect-target-keyframes-options-options"></a><code>optional <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> options</code>

<a id="ref-for-iteration-duration①⑥"></a>

Either a number specifying the [iteration duration](#iteration-duration) of the effect, or a collection of properties specifying the timing and behavior of the effect.

Examples of the usage of this constructor are given in [§ 6.6.1 Creating a new KeyframeEffect object](#creating-a-new-keyframeeffect-object).

<a id="dom-keyframeeffect-keyframeeffect-source"></a>`KeyframeEffect (source)`  
<a id="ref-for-keyframeeffect⑤"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect-source-source①"></a>

Creates a new <code><a href="#keyframeeffect">KeyframeEffect</a></code> object with the same properties as <code><a href="#dom-keyframeeffect-keyframeeffect-source-source">source</a></code> using the following procedure:

1.  <a id="ref-for-keyframeeffect⑥"></a>

    Create a new <code><a href="#keyframeeffect">KeyframeEffect</a></code> object, <var>effect</var>.

2.  Set the following properties of <var>effect</var> using the corresponding values of <var>source</var>:

    - <a id="ref-for-keyframe-effect-effect-target①⑧"></a>

      [effect target](#keyframe-effect-effect-target),

    - <a id="ref-for-keyframe④⑥"></a>

      [keyframes](#keyframe),

    - <a id="ref-for-composite-operation①②"></a>

      [composite operation](#composite-operation), and

    - all specified timing properties:

      - <a id="ref-for-start-delay①⑧"></a>

        [start delay](#start-delay),

      - <a id="ref-for-end-delay⑦"></a>

        [end delay](#end-delay),

      - <a id="ref-for-fill-mode①⑤"></a>

        [fill mode](#fill-mode),

      - <a id="ref-for-iteration-start①①"></a>

        [iteration start](#iteration-start),

      - <a id="ref-for-iteration-count①⑦"></a>

        [iteration count](#iteration-count),

      - <a id="ref-for-iteration-duration①⑦"></a>

        [iteration duration](#iteration-duration),

      - <a id="ref-for-playback-direction⑦"></a>

        [playback direction](#playback-direction), and

      - <a id="ref-for-easing-function①⓪"></a>

        [timing function](https://www.w3.org/TR/css-easing-1/#easing-function).

    <a id="ref-for-dom-keyframeeffect-keyframeeffect①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Unlike the <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect(target, keyframes, options)</a></code> constructor, we do not need to re-throw exceptions since the timing properties specified on <var>source</var> can be assumed to be valid.

<a id="ref-for-keyframeeffect⑦"></a>

<a id="dom-keyframeeffect-keyframeeffect-source-source"></a><code><code><a href="#keyframeeffect">KeyframeEffect</a></code> source</code>

<a id="ref-for-keyframe-effect③②"></a>

The [keyframe effect](#keyframe-effect) from which to copy the properties that define the new <a id="ref-for-keyframe-effect③③"></a>keyframe effect.

<a id="ref-for-element⑥"></a>

<a id="dom-keyframeeffect-target"></a>`target`, of type [Element](https://dom.spec.whatwg.org/#element), nullable

<a id="ref-for-effect-target-target-element⑥"></a>

<a id="ref-for-keyframe-effect-effect-target①⑨"></a>

<a id="ref-for-element⑦"></a>

<a id="ref-for-originating-element①"></a>

The [target element](#effect-target-target-element) being animated by this object (either the [effect target](#keyframe-effect-effect-target) if it is an <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> or its [originating element](https://www.w3.org/TR/selectors-4/#originating-element) if it is a pseudo-element). This may be `null` for animations that do not target a specific element such as an animation that produces a sound using an audio API.

<a id="ref-for-cssomstring①"></a>

<a id="dom-keyframeeffect-pseudoelement"></a>`pseudoElement`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), nullable

<a id="ref-for-effect-target-target-pseudo-selector⑤"></a>

<a id="ref-for-keyframe-effect-effect-target②⓪"></a>

The [target pseudo-selector](#effect-target-target-pseudo-selector). `null` if this effect has no [effect target](#keyframe-effect-effect-target) or if the <a id="ref-for-keyframe-effect-effect-target②①"></a>effect target is an element (i.e. not a pseudo-element). When the <a id="ref-for-keyframe-effect-effect-target②②"></a>effect target is a pseudo-element, this specifies the pseudo-element selector (e.g. `::before`).

<a id="ref-for-effect-target-target-pseudo-selector⑥"></a>

<a id="ref-for-animation-effect①⓪⑨"></a>

On setting, sets the [target pseudo-selector](#effect-target-target-pseudo-selector) of the [animation effect](#animation-effect) to the provided value after applying the following exceptions:

- <a id="ref-for-typedef-pseudo-element-selector"></a>

  <a id="ref-for-dfn-throw①②"></a>

  <a id="ref-for-idl-DOMException⑥"></a>

  <a id="ref-for-syntaxerror"></a>

  <a id="ref-for-effect-target-target-pseudo-selector⑦"></a>

  <a id="ref-for-animation-effect①①⓪"></a>

  If the provided value is not `null` and is an invalid [\<pseudo-element-selector\>](https://www.w3.org/TR/selectors-4/#typedef-pseudo-element-selector), the user agent must [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code> with error name <code><a href="https://webidl.spec.whatwg.org/#syntaxerror">SyntaxError</a></code> and leave the [target pseudo-selector](#effect-target-target-pseudo-selector) of this [animation effect](#animation-effect) unchanged.

  <a id="ref-for-invalid-selector"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note, that invalid in this context follows the definition of an [invalid selector](https://www.w3.org/TR/selectors-4/#invalid-selector) defined in [\[SELECTORS-4\]](#biblio-selectors-4) such that syntactically invalid pseudo-elements as well as pseudo-elements for which the user agent has no usable level of support are both deemed invalid.

- <a id="ref-for-effect-target-target-pseudo-selector⑧"></a>

  If one of the legacy Selectors Level 2 single-colon selectors (':before', ':after', ':first-letter', or ':first-line') is specified, the [target pseudo-selector](#effect-target-target-pseudo-selector) must be set to the equivalent two-colon selector (e.g. '::before').

<a id="ref-for-enumdef-compositeoperation①"></a>

<a id="dom-keyframeeffect-composite"></a>`composite`, of type [CompositeOperation](#enumdef-compositeoperation)

<a id="ref-for-composite-operation①③"></a>

<a id="ref-for-keyframe-effect③④"></a>

<a id="ref-for-effect-stack①⑤"></a>

<a id="ref-for-compositeoperation"></a>

The [composite operation](#composite-operation) used to composite this [keyframe effect](#keyframe-effect) with the [effect stack](#effect-stack), as specified by one of the [CompositeOperation](#compositeoperation) enumeration values.

<a id="ref-for-composite-operation①④"></a>

<a id="ref-for-animation-effect①①①"></a>

On setting, sets the [composite operation](#composite-operation) property of this [animation effect](#animation-effect) to the provided value.

<a id="dom-keyframeeffect-getkeyframes"></a>`  sequence<object> getKeyframes() `  
<a id="ref-for-computed-keyframe-offset②③"></a>

Returns the keyframes that make up this effect along with their [computed keyframe offsets](#computed-keyframe-offset).

<em>This section is non-normative</em>

The result of this method is a sequence of objects of the following format:

```idl
dictionary ComputedKeyframe {
    // ... property-value pairs ...
    // i.e. DOMString propertyName
    double?                  offset = null;
    double                   computedOffset;
    DOMString                easing = "linear";
    CompositeOperationOrAuto composite = "auto";
};
```
The meaning and values of each member is as follows:

`offset`  
<a id="ref-for-keyframe-offset⑥"></a>

<a id="ref-for-keyframe④⑦"></a>

The [keyframe offset](#keyframe-offset) of the [keyframe](#keyframe) specified as a number between 0.0 and 1.0 inclusive, or `null`.

<a id="ref-for-keyframe④⑧"></a>

This will be `null` if the [keyframe](#keyframe) is to be automatically spaced between adjacent keyframes.

`computedOffset`  
<a id="ref-for-computed-keyframe-offset②④"></a>

<a id="ref-for-keyframe④⑨"></a>

<a id="ref-for-compute-missing-keyframe-offsets①"></a>

The [computed keyframe offset](#computed-keyframe-offset) for this [keyframe](#keyframe) calculated as part of running the [compute missing keyframe offsets](#compute-missing-keyframe-offsets) procedure.

Unlike the `offset` member, the `computedOffset` is never null.

`easing`  
<a id="ref-for-easing-function①①"></a>

The [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) used to transform the progress of time from this keyframe until the next keyframe in the series.

`composite`  
<a id="ref-for-keyframe-specific-composite-operation①"></a>

<a id="ref-for-underlying-value①③"></a>

The [keyframe-specific composite operation](#keyframe-specific-composite-operation) used to combine the values specified in this keyframe with the [underlying value](#underlying-value).

<a id="ref-for-dom-compositeoperationorauto-auto"></a>

<a id="ref-for-composite-operation①⑤"></a>

<a id="ref-for-keyframe-effect③⑤"></a>

This member will be <code><a href="#dom-compositeoperationorauto-auto">auto</a></code> if the [composite operation](#composite-operation) specified on the [keyframe effect](#keyframe-effect) is being used.

<a id="ref-for-keyframe⑤⓪"></a>

Since [keyframes](#keyframe) are represented by a partially open-ended dictionary type that is not currently able to be expressed with WebIDL, the procedure used to prepare the result of this method is defined in prose below:

1.  Let <var>result</var> be an empty sequence of objects.

2.  Let <var>keyframes</var> be one of the following:

    1.  <a id="ref-for-keyframe-effect③⑥"></a>

        <a id="ref-for-cssanimation"></a>

        <a id="ref-for-keyframe⑤①"></a>

        <a id="ref-for-dom-keyframeeffect-setkeyframes②"></a>

        <a id="ref-for-computed-keyframes④"></a>

        If this [keyframe effect](#keyframe-effect) is associated with a <code><a href="https://www.w3.org/TR/css-animations-2/#cssanimation">CSSAnimation</a></code>, and its [keyframes](#keyframe) have not been replaced by a successful call to <code><a href="#dom-keyframeeffect-setkeyframes">setKeyframes()</a></code>; the [computed keyframes](#computed-keyframes) for this <a id="ref-for-keyframe-effect③⑦"></a>keyframe effect.

    2.  <a id="ref-for-compute-missing-keyframe-offsets②"></a>

        <a id="ref-for-keyframe⑤②"></a>

        <a id="ref-for-keyframe-effect③⑧"></a>

        Otherwise, the result of applying the procedure [compute missing keyframe offsets](#compute-missing-keyframe-offsets) to the [keyframes](#keyframe) for this [keyframe effect](#keyframe-effect).

    <a id="ref-for-computed-keyframes⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: We return [computed keyframes](#computed-keyframes) for CSS Animations because not all keyframes specified in CSS can be represented by a dictionary.

3.  For each <var>keyframe</var> in <var>keyframes</var> perform the following steps:

    1.  Initialize a dictionary object, <var>output keyframe</var>, using the following definition:

        <a id="dictdef-basecomputedkeyframe"></a>

        <a id="ref-for-idl-double②②"></a>

        <a id="dom-basecomputedkeyframe-offset"></a>

        <a id="ref-for-idl-double②③"></a>

        <a id="dom-basecomputedkeyframe-computedoffset"></a>

        <a id="ref-for-idl-DOMString⑦"></a>

        <a id="dom-basecomputedkeyframe-easing"></a>

        <a id="ref-for-enumdef-compositeoperationorauto"></a>

        <a id="dom-basecomputedkeyframe-composite"></a>

        ```text
        dictionary BaseComputedKeyframe {
             double?                  offset = null;
             double                   computedOffset;
             DOMString                easing = "linear";
             CompositeOperationOrAuto composite = "auto";
        };
        ```
    2.  <a id="ref-for-dom-basecomputedkeyframe-offset"></a>

        <a id="ref-for-dom-basecomputedkeyframe-computedoffset"></a>

        <a id="ref-for-dom-basecomputedkeyframe-easing"></a>

        <a id="ref-for-dom-basecomputedkeyframe-composite"></a>

        <a id="ref-for-keyframe-offset⑦"></a>

        <a id="ref-for-computed-keyframe-offset②⑤"></a>

        <a id="ref-for-easing-function①②"></a>

        <a id="ref-for-keyframe-specific-composite-operation②"></a>

        Set the <code><a href="#dom-basecomputedkeyframe-offset">offset</a></code>, <code><a href="#dom-basecomputedkeyframe-computedoffset">computedOffset</a></code>, <code><a href="#dom-basecomputedkeyframe-easing">easing</a></code>, and <code><a href="#dom-basecomputedkeyframe-composite">composite</a></code> members of <var>output keyframe</var> to the respective [keyframe offset](#keyframe-offset), [computed keyframe offset](#computed-keyframe-offset), keyframe-specific [timing function](https://www.w3.org/TR/css-easing-1/#easing-function), and [keyframe-specific composite operation](#keyframe-specific-composite-operation) values of <var>keyframe</var>.

    3.  For each animation property-value pair in <var>keyframe</var>, <var>declaration</var>, perform the following steps:

        1.  <a id="ref-for-animation-property-name-to-idl-attribute-name"></a>

            Let <var>property name</var> be the result of applying the [animation property name to IDL attribute name](#animation-property-name-to-idl-attribute-name) algorithm to the property name of <var>declaration</var>.

        2.  <a id="ref-for-serialize-a-css-value"></a>

            Let <var>IDL value</var> be the result of serializing the property value of <var>declaration</var> by passing <var>declaration</var> to the algorithm to [serialize a CSS value](https://drafts.csswg.org/cssom/#serialize-a-css-value) [\[CSSOM\]](#biblio-cssom).

        3.  <a id="ref-for-DOMString-to-es"></a>

            Let <var>value</var> be the result of [converting](https://webidl.spec.whatwg.org/#DOMString-to-es) <var>IDL value</var> to an ECMAScript String value.

        4.  <a id="ref-for-sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc"></a>

            Call the [\[\[DefineOwnProperty\]\]](http://www.ecma-international.org/ecma-262/6.0/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc) internal method on <var>output keyframe</var> with property name <var>property&#xA;name</var>, Property Descriptor { \[\[Writable\]\]: true, \[\[Enumerable\]\]: true, \[\[Configurable\]\]: true, \[\[Value\]\]: <var>value</var> } and Boolean flag false.

    4.  Append <var>output keyframe</var> to <var>result</var>.

4.  Return <var>result</var>.

<a id="dom-keyframeeffect-setkeyframes"></a>`  void setKeyframes(object? keyframes) `  
<a id="ref-for-keyframe⑤③"></a>

Replaces the set of [keyframes](#keyframe) that make up this effect.

<a id="dom-keyframeeffect-setkeyframes-keyframes"></a>`object? keyframes`  
A series of keyframes whose format and processing is defined by [§ 6.6.3 Processing a keyframes argument](#processing-a-keyframes-argument).

<a id="ref-for-keyframe⑤④"></a>

<a id="ref-for-process-a-keyframes-argument"></a>

This effect’s set of [keyframes](#keyframe) is replaced with the result of performing the procedure to [process a keyframes argument](#process-a-keyframes-argument). If that procedure throws an exception, this effect’s <a id="ref-for-keyframe⑤⑤"></a>keyframes are not modified.

#### <a id="creating-a-new-keyframeeffect-object"></a>6.6.1. Creating a new `KeyframeEffect` object

<em>This section is non-normative</em>

<a id="ref-for-dom-keyframeeffect-keyframeeffect②"></a>

<a id="ref-for-keyframeeffect⑧"></a>

The <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect</a></code> constructor offers a number of approaches to creating new <code><a href="#keyframeeffect">KeyframeEffect</a></code> objects.

<a id="ref-for-keyframeeffect⑨"></a>

At its simplest, a <code><a href="#keyframeeffect">KeyframeEffect</a></code> object that changes the "left" property of `elem` to 100px over three seconds can be constructed as follows:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7a28f33c"></a>
>
> ```javascript
> var effect = new KeyframeEffect(elem, { left: '100px' }, 3000);
> ```
The second parameter, representing the list of keyframes, may specify multiple properties. (See [§ 6.6.3 Processing a keyframes argument](#processing-a-keyframes-argument).)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d6624b71"></a>
>
> ```javascript
> // Specify multiple properties at once
> var effectA = new KeyframeEffect(elem, { left: '100px', top: '300px' }, 3000);
> 
> // Specify multiple keyframes
> var effectB = new KeyframeEffect(elem, [ { left: '100px' }, { left: '300px' } ], 3000);
> ```
<a id="ref-for-iteration-duration①⑧"></a>

<a id="ref-for-start-delay①⑨"></a>

<a id="ref-for-dictdef-effecttiming①⓪"></a>

The third parameter, representing the animation’s timing, may simply be a number representing the [iteration duration](#iteration-duration) in milliseconds as above, or, to specify further timing properties such as the [start delay](#start-delay), an <code><a href="#dictdef-effecttiming">EffectTiming</a></code> object can be used, as follows:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4d5977de"></a>
>
> ```javascript
> var effect =
>   new KeyframeEffect(elem, { left: '100px' }, { duration: 3000, delay: 2000 });
> ```
If the duration is not specified, a value of zero is used. It is possible to create an animation that simply sets a property without any interpolation as follows:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d3a5d53d"></a>
>
> ```javascript
> var effect =
>   new KeyframeEffect(elem, { visibility: 'hidden' }, { fill: 'forwards' });
> ```
As described in [§ 4.6 Fill behavior](#fill-behavior) however, using indefinitely filling animations in this way is discouraged.

<a id="ref-for-keyframeeffect①⓪"></a>

<a id="ref-for-animation⑦"></a>

Having created a <code><a href="#keyframeeffect">KeyframeEffect</a></code>, it can be played by adding it to an <code><a href="#animation">Animation</a></code> and then playing that animation. For simple effects, however, the [`Element.animate`](#extensions-to-the-element-interface) shortcut is more convenient since it performs these steps automatically. For example,

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a7ce65dd"></a>
>
> ```javascript
> elem.animate({ left: '100px' }, 3000);
> ```
#### <a id="property-name-conversion"></a>6.6.2. Property names and IDL names

The <a id="animation-property-name-to-idl-attribute-name"></a>animation property name to IDL attribute name algorithm for <var>property</var> is as follows:

1.  <a id="ref-for-typedef-custom-property-name"></a>

    If <var>property</var> follows the [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name) production, return <var>property</var>.

2.  <a id="ref-for-propdef-float"></a>

    If <var>property</var> refers to the CSS [float](https://drafts.csswg.org/css2/#propdef-float) property, return the string "cssFloat".

3.  <a id="ref-for-propdef-offset"></a>

    If <var>property</var> refers to the CSS [offset](https://www.w3.org/TR/motion-1/#propdef-offset) property, return the string "cssOffset".

4.  <a id="ref-for-css-property-to-idl-attribute①"></a>

    Otherwise, return the result of applying the [CSS property to IDL attribute](https://drafts.csswg.org/cssom/#css-property-to-idl-attribute) algorithm [\[CSSOM\]](#biblio-cssom) to <var>property</var>.

The <a id="idl-attribute-name-to-animation-property-name"></a>IDL attribute name to animation property name algorithm for <var>attribute</var> is as follows:

1.  <a id="ref-for-typedef-custom-property-name①"></a>

    If <var>attribute</var> conforms to the [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name) production, return <var>attribute</var>.

2.  <a id="ref-for-propdef-float①"></a>

    If <var>attribute</var> is the string "cssFloat", then return an animation property representing the CSS [float](https://drafts.csswg.org/css2/#propdef-float) property.

3.  <a id="ref-for-propdef-offset①"></a>

    If <var>attribute</var> is the string "cssOffset", then return an animation property representing the CSS [offset](https://www.w3.org/TR/motion-1/#propdef-offset) property.

4.  <a id="ref-for-idl-attribute-to-css-property"></a>

    Otherwise, return the result of applying the [IDL attribute to CSS property](https://drafts.csswg.org/cssom/#idl-attribute-to-css-property) algorithm [\[CSSOM\]](#biblio-cssom) to <var>attribute</var>.

#### <a id="processing-a-keyframes-argument"></a>6.6.3. Processing a `keyframes` argument

<em>This section is non-normative</em>

The following methods all accept a set of keyframes as an argument:

- <a id="ref-for-dom-keyframeeffect-keyframeeffect③"></a>

  the <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect(target, keyframes, options)</a></code> constructor,

- <a id="ref-for-dom-keyframeeffect-setkeyframes③"></a>

  <a id="ref-for-keyframeeffect①①"></a>

  the <code><a href="#dom-keyframeeffect-setkeyframes">setKeyframes()</a></code> method on the <code><a href="#keyframeeffect">KeyframeEffect</a></code> interface,

- <a id="ref-for-dom-animatable-animate"></a>

  <a id="ref-for-animatable"></a>

  the <code><a href="#dom-animatable-animate">animate()</a></code> method of the <code><a href="#animatable">Animatable</a></code> interface mixin.

This argument may be specified in the one of two forms as illustrated below.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-23ca44fc"></a>
>
> ```javascript
> // The following two expressions produce the same result:
> elem.animate([ { color: 'blue' },
>                { color: 'green' },
>                { color: 'red' },
>                { color: 'yellow' } ], 2000);
> elem.animate({ color: [ 'blue', 'green', 'red', 'yellow' ] }, 2000);
> 
> // Likewise, for a multi-property animation, the following two
> // expressions are equivalent:
> elem.animate([ { color: 'blue', left: '0px' },
>                { color: 'green', left: '-20px' },
>                { color: 'red', left: '100px' },
>                { color: 'yellow', left: '50px'} ], 2000);
> elem.animate({ color: [ 'blue', 'green', 'red', 'yellow' ],
>                left: [ '0px', '-20px', '100px', '50px' ] }, 2000);
> 
> // Incidentally, the following three expressions are all equivalent:
> elem.animate([ { color: 'red' } ], 1000);
> elem.animate({ color: [ 'red' ] }, 1000);
> elem.animate({ color: 'red' }, 1000);
> ```
The first form (the array-form) consists of an array of keyframes where each keyframe may specify at most one value per animation property. The second form (the object-form) consists of an object where each animation property may specify a single animation value or an array of animation values.

<a id="ref-for-dom-keyframeeffect-getkeyframes①"></a>

The first array-form is the canonical form and is the form returned by the <code><a href="#dom-keyframeeffect-getkeyframes">getKeyframes()</a></code> method.

<a id="ref-for-keyframe-offset⑧"></a>

[Keyframe offsets](#keyframe-offset) can be specified using either form as illustrated below:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-34879cb3"></a>
>
> ```javascript
> // The keyframes without offsets will automatically have offsets computed
> // as 0 for the first keyframe, 0.65 for the middle keyframe, and 1 for the
> // final keyframe.
> elem.animate([ { color: 'blue' },
>                { color: 'green', offset: 0.5 },
>                { color: 'red' },
>                { color: 'yellow', offset: 0.8 },
>                { color: 'pink' } ], 2000);
> 
> // The following produces the same result. Note that it is not necessary to
> // specify the last value: it will automatically be treated as 'null' and then
> // the automatic assignment will apply as with the previous case.
> elem.animate({ color: [ 'blue', 'green', 'red', 'yellow', 'pink' ],
>                offset: [ null, 0.5, null, 0.8 ] }, 2000);
> ```
<a id="ref-for-easing-function①③"></a>

<a id="ref-for-keyframe-specific-composite-operation③"></a>

<a id="ref-for-keyframe⑤⑥"></a>

Likewise [timing functions](https://www.w3.org/TR/css-easing-1/#easing-function) and [keyframe-specific composite operations](#keyframe-specific-composite-operation) may be specified in either form. The array-form allows specifying different values for each [keyframe](#keyframe) whilst for the object-form, the list of values will be repeated as needed until each keyframe has been assigned a value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-52bc65a2"></a>
>
> ```javascript
> // Since timing functions apply _between_ keyframes, even if we specify a
> // a timing function on the last keyframe it will be ignored.
> elem.animate([ { color: 'blue', easing: 'ease-in' },
>                { color: 'green', easing: 'ease-out' },
>                { color: 'yellow' } ], 2000);
> 
> // The following produces the same result.
> elem.animate({ color: [ 'blue', 'green', 'yellow' ],
>                easing: [ 'ease-in', 'ease-out' ] }, 2000);
> 
> // The repeating behavior makes assigning the same value to all keyframes
> // simple:
> elem.animate({ color: [ 'blue', 'green', 'yellow' ],
>                easing: 'ease-in-out' }, 2000);
> ```
<a id="ref-for-easing-function①④"></a>

<a id="ref-for-iteration-duration①⑨"></a>

<a id="ref-for-keyframe-effect③⑨"></a>

<a id="ref-for-dictdef-keyframeeffectoptions⑤"></a>

<a id="ref-for-dictdef-keyframeanimationoptions"></a>

<a id="ref-for-dom-animatable-animate①"></a>

<a id="ref-for-animatable①"></a>

Note that the `easing` property in either form sets the <em>keyframe-specific timing function</em>. This is independent from the [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) that applies to the entire [iteration duration](#iteration-duration) of the [keyframe effect](#keyframe-effect) as specified using a <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> object (or <code><a href="#dictdef-keyframeanimationoptions">KeyframeAnimationOptions</a></code> object when using the <code><a href="#dom-animatable-animate">animate()</a></code> method of the <code><a href="#animatable">Animatable</a></code> interface mixin).

In the following example, the two statements produce different results.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-148601d7"></a>
>
> ```javascript
> // Here, 'ease-in-out' is applied between each color value.
> elem.animate({ color: [ 'blue', 'green', 'yellow' ],
>                easing: 'ease-in-out' }, 2000);
> 
> // However, in this case, 'ease-in-out' is applied across the whole span
> // of the animation, that is from 'blue' to 'yellow'.
> elem.animate({ color: [ 'blue', 'green', 'yellow' ] },
>              { duration: 2000, easing: 'ease-in-out' });
> ```
The type of the `keyframes` argument cannot be expressed in WebIDL since it relies on a partially-open dictionary type.

Conceptually, the type of this argument is equivalent to the following WebIDL-like definition:

```idl
dictionary Keyframe {
    // ... property-value pairs ...
    // i.e. DOMString propertyName
    double?                   offset = null;
    DOMString                 easing = "linear";
    CompositeOperationOrAuto  composite = "auto";
};

dictionary PropertyIndexedKeyframes {
    // ... property-value and property-valuelist pairs ...
    // i.e. (DOMString or sequence&lt;DOMString&gt;) propertyName
    (double? or sequence<double?>)                         offset = [];
    (DOMString or sequence<DOMString>)                     easing = [];
    (CompositeOperationOrAuto or sequence<CompositeOperationOrAuto>) composite = [];
};

typedef (sequence<Keyframe> or PropertyIndexedKeyframes) KeyframeArgument;
```
The meaning and allowed values of each argument is as follows:

offset  
<a id="ref-for-keyframe-offset⑨"></a>

<a id="ref-for-keyframe⑤⑦"></a>

The [keyframe offset](#keyframe-offset) of the [keyframe](#keyframe) specified as a number between 0.0 and 1.0 inclusive or `null`.

<a id="ref-for-keyframe⑤⑧"></a>

A `null` value indicates that the [keyframe](#keyframe) should be automatically spaced between adjacent keyframes.

Specifying an offset outside the range \[0.0, 1.0\] will cause a TypeError to be thrown.

Keyframes that specify an offset must be provided in increasing order of offset. Adjacent and equal offsets, however, are permitted.

easing  
<a id="ref-for-easing-function①⑤"></a>

The [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) used to transform the progress of time from this keyframe until the next keyframe in the series.

<a id="ref-for-dom-effecttiming-easing③"></a>

<a id="ref-for-dictdef-effecttiming①①"></a>

The syntax and error-handling associated with parsing this string is identical to that defined for the <code><a href="#dom-effecttiming-easing">easing</a></code> attribute of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> interface.

composite  
<a id="ref-for-keyframe-specific-composite-operation④"></a>

<a id="ref-for-underlying-value①④"></a>

The [keyframe-specific composite operation](#keyframe-specific-composite-operation) used to combine the values specified in this keyframe with the [underlying value](#underlying-value).

<a id="ref-for-dom-compositeoperationorauto-auto①"></a>

<a id="ref-for-composite-operation①⑥"></a>

<a id="ref-for-keyframe-effect④⓪"></a>

If <code><a href="#dom-compositeoperationorauto-auto">auto</a></code>, the [composite operation](#composite-operation) specified on the [keyframe effect](#keyframe-effect) will be used.

Since this type cannot be expressed in WebIDL, its processing is defined in prose following.

<a id="ref-for-process-a-keyframes-argument①"></a>

For each method that takes a `keyframes` argument, the procedure to [process a keyframes argument](#process-a-keyframes-argument) is run on the input and the result of that procedure is retained.

First we define two supporting definitions.

<a id="ref-for-sec-completion-record-specification-type"></a>

The instruction, <a id="check-the-completion-record"></a>check the completion record of <var>result</var>, where <var>result</var> is a [completion record](http://www.ecma-international.org/ecma-262/6.0/#sec-completion-record-specification-type) from calling an ECMAScript operation, is equivalent to the following steps:

1.  <a id="ref-for-sec-completion-record-specification-type①"></a>

    <a id="ref-for-dfn-throw①③"></a>

    If <var>result</var> is an [abrupt completion](http://www.ecma-international.org/ecma-262/6.0/#sec-completion-record-specification-type), [throw](https://webidl.spec.whatwg.org/#dfn-throw) the exception contained in the \[\[value\]\] field of <var>result</var> and abort the procedure.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-47a68024"></a> What should we do if the \[\[type\]\] is break, continue, or return? Can it be?

2.  Replace <var>result</var> with the value contained in the \[\[value\]\] field of <var>result</var>.

The procedure to <a id="process-a-keyframe-like-object"></a>process a keyframe-like object, takes two arguments:

- an ECMAScript object, <var>keyframe input</var>, and

- an <var>allow lists</var> boolean flag

and returns a map from either property names to DOMString values if <var>allow&#xA;lists</var> is false, or from property names to sequences of DOMString values otherwise, using the following procedure:

1.  <a id="ref-for-es-to-dictionary"></a>

    Run the procedure to [convert an ECMAScript value to a dictionary type](https://webidl.spec.whatwg.org/#es-to-dictionary) [\[WEBIDL\]](#biblio-webidl) with <var>keyframe input</var> as the ECMAScript value, and the dictionary type depending on the value of the <var>allow lists</var> flag as follows:

    If <var>allow lists</var> is true,  
    Use the following dictionary type:

    <a id="dictdef-basepropertyindexedkeyframe"></a>

    <a id="ref-for-idl-double②④"></a>

    <a id="ref-for-idl-sequence①"></a>

    <a id="ref-for-idl-double②⑤"></a>

    <a id="dom-basepropertyindexedkeyframe-offset"></a>

    <a id="ref-for-idl-DOMString⑧"></a>

    <a id="ref-for-idl-sequence②"></a>

    <a id="ref-for-idl-DOMString⑨"></a>

    <a id="dom-basepropertyindexedkeyframe-easing"></a>

    <a id="ref-for-enumdef-compositeoperationorauto①"></a>

    <a id="ref-for-idl-sequence③"></a>

    <a id="ref-for-enumdef-compositeoperationorauto②"></a>

    <a id="dom-basepropertyindexedkeyframe-composite"></a>

    ```text
    dictionary BasePropertyIndexedKeyframe {
        (double? or sequence<double?>)                         offset = [];
        (DOMString or sequence<DOMString>)                     easing = [];
        (CompositeOperationOrAuto or sequence<CompositeOperationOrAuto>) composite = [];
    };
    ```
    Otherwise,  
    Use the following dictionary type,

    <a id="dictdef-basekeyframe"></a>

    <a id="ref-for-idl-double②⑥"></a>

    <a id="dom-basekeyframe-offset"></a>

    <a id="ref-for-idl-DOMString①⓪"></a>

    <a id="dom-basekeyframe-easing"></a>

    <a id="ref-for-enumdef-compositeoperationorauto③"></a>

    <a id="dom-basekeyframe-composite"></a>

    ```text
    dictionary BaseKeyframe {
        double?                  offset = null;
        DOMString                easing = "linear";
        CompositeOperationOrAuto composite = "auto";
    };
    ```
    Store the result of this procedure as <var>keyframe output</var>.

2.  Build up a list of <var>animatable properties</var> as follows:

    1.  Let <var>animatable properties</var> be a list of property names (including shorthand properties that have longhand sub-properties that are animatable) that can be animated by the implementation.

    2.  <a id="ref-for-animation-property-name-to-idl-attribute-name①"></a>

        Convert each property name in <var>animatable properties</var> to the equivalent IDL attribute by applying the [animation property name to IDL attribute name](#animation-property-name-to-idl-attribute-name) algorithm.

3.  <a id="ref-for-sec-enumerableownnames"></a>

    Let <var>input properties</var> be the result of calling the [EnumerableOwnNames](http://www.ecma-international.org/ecma-262/6.0/#sec-enumerableownnames) operation with <var>keyframe input</var> as the object.

4.  <a id="ref-for-typedef-custom-property-name②"></a>

    Make up a new list <var>animation properties</var> that consists of all of the properties that are in <em>both</em> <var>input&#xA;properties</var> and <var>animatable properties</var>, <em>or</em> which are in <var>input properties</var> and conform to the [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name) production.

5.  Sort <var>animation properties</var> in ascending order by the Unicode codepoints that define each property name.

6.  For each <var>property name</var> in <var>animation properties</var>,

    1.  <a id="ref-for-sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver"></a>

        Let <var>raw value</var> be the result of calling the [\[\[Get\]\]](http://www.ecma-international.org/ecma-262/6.0/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver) internal method on <var>keyframe input</var>, with <var>property&#xA;name</var> as the property key and <var>keyframe input</var> as the receiver.

    2.  <a id="ref-for-check-the-completion-record"></a>

        [Check the completion record](#check-the-completion-record) of <var>raw value</var>.

    3.  Convert <var>raw value</var> to a DOMString or sequence of DOMStrings <var>property values</var> as follows:

        If <var>allow lists</var> is true,  
        <a id="ref-for-dfn-convert-ecmascript-to-idl-value"></a>

        Let <var>property values</var> be the result of converting <var>raw&#xA;value</var> to IDL type

        ```text
        (DOMString or
        sequence<DOMString>)
        ```
        using the [procedures defined for converting an ECMAScript value to an IDL value](https://webidl.spec.whatwg.org/#dfn-convert-ecmascript-to-idl-value) [\[WEBIDL\]](#biblio-webidl).

        If <var>property values</var> is a single DOMString, replace <var>property values</var> with a sequence of DOMStrings with the original value of <var>property values</var> as the only element.

        Otherwise,  
        <a id="ref-for-es-to-DOMString"></a>

        Let <var>property values</var> be the result of converting <var>raw&#xA;value</var> to a DOMString using the [procedure for converting an ECMAScript value to a DOMString](https://webidl.spec.whatwg.org/#es-to-DOMString) [\[WEBIDL\]](#biblio-webidl).

    4.  <a id="ref-for-idl-attribute-name-to-animation-property-name"></a>

        Calculate the <var>normalized property name</var> as the result of applying the [IDL attribute name to animation property name](#idl-attribute-name-to-animation-property-name) algorithm to <var>property name</var>.

    5.  Add a property to <var>keyframe output</var> with <var>normalized&#xA;property name</var> as the property name, and <var>property values</var> as the property value.

7.  Return <var>keyframe output</var>.

The procedure to <a id="process-a-keyframes-argument"></a>process a keyframes argument takes a nullable ECMAScript object, <var>object</var>, as input and returns a sequence of keyframes using the following procedure:

1.  If <var>object</var> is null, return an empty sequence of keyframes.

2.  <a id="ref-for-keyframe⑤⑨"></a>

    Let <var>processed keyframes</var> be an empty sequence of [keyframes](#keyframe).

3.  <a id="ref-for-sec-getmethod"></a>

    <a id="ref-for-sec-well-known-symbols"></a>

    Let <var>method</var> be the result of [GetMethod](http://www.ecma-international.org/ecma-262/6.0/#sec-getmethod)(<var>object</var>, [@@iterator](http://www.ecma-international.org/ecma-262/6.0/#sec-well-known-symbols)).

4.  <a id="ref-for-check-the-completion-record①"></a>

    [Check the completion record](#check-the-completion-record) of <var>method</var>.

5.  Perform the steps corresponding to the first matching condition from below,

    If <var>method</var> is not undefined,  
    1.  <a id="ref-for-sec-getiterator"></a>

        Let <var>iter</var> be [GetIterator](http://www.ecma-international.org/ecma-262/6.0/#sec-getiterator)(<var>object</var>, <var>method</var>).

    2.  <a id="ref-for-check-the-completion-record②"></a>

        [Check the completion record](#check-the-completion-record) of <var>iter</var>.

    3.  Repeat:

        1.  <a id="ref-for-sec-iteratorstep"></a>

            Let <var>next</var> be [IteratorStep](http://www.ecma-international.org/ecma-262/6.0/#sec-iteratorstep)(<var>iter</var>).

        2.  <a id="ref-for-check-the-completion-record③"></a>

            [Check the completion record](#check-the-completion-record) of <var>next</var>.

        3.  If <var>next</var> is false abort this loop.

        4.  <a id="ref-for-sec-iteratorvalue"></a>

            Let <var>nextItem</var> be [IteratorValue](http://www.ecma-international.org/ecma-262/6.0/#sec-iteratorvalue)(next).

        5.  <a id="ref-for-check-the-completion-record④"></a>

            [Check the completion record](#check-the-completion-record) of <var>nextItem</var>.

        6.  <a id="ref-for-sec-ecmascript-data-types-and-values"></a>

            If [Type](http://www.ecma-international.org/ecma-262/6.0/#sec-ecmascript-data-types-and-values)(<var>nextItem</var>) is not Undefined, Null or Object, then throw a TypeError and abort these steps.

        7.  <a id="ref-for-process-a-keyframe-like-object"></a>

            Append to <var>processed keyframes</var> the result of running the procedure to [process a keyframe-like object](#process-a-keyframe-like-object) passing <var>nextItem</var> as the <var>keyframe input</var> and with the <var>allow lists</var> flag set to false.

    Otherwise,  
    1.  <a id="ref-for-process-a-keyframe-like-object①"></a>

        Let <var>property-indexed keyframe</var> be the result of running the procedure to [process a keyframe-like object](#process-a-keyframe-like-object) passing <var>object</var> as the <var>keyframe input</var> and with the <var>allow lists</var> flag set to true.

    2.  For each member, <var>m</var>, in <var>property-indexed&#xA;keyframe</var>, perform the following steps:

        1.  Let <var>property name</var> be the key for <var>m</var>.

        2.  If <var>property name</var> is "composite", or "easing", or "offset", skip the remaining steps in this loop and continue from the next member in <var>property-indexed keyframe</var> after <var>m</var>.

        3.  Let <var>property values</var> be the value for <var>m</var>.

        4.  <a id="ref-for-keyframe⑥⓪"></a>

            Let <var>property keyframes</var> be an empty sequence of [keyframes](#keyframe).

        5.  For each value, <var>v</var>, in <var>property values</var> perform the following steps:

            1.  <a id="ref-for-keyframe⑥①"></a>

                <a id="ref-for-keyframe-offset①⓪"></a>

                Let <var>k</var> be a new [keyframe](#keyframe) with a null [keyframe offset](#keyframe-offset).

            2.  Add the property-value pair, <var>property name</var> → <var>v</var>, to <var>k</var>.

            3.  Append <var>k</var> to <var>property keyframes</var>.

        6.  <a id="ref-for-compute-missing-keyframe-offsets③"></a>

            Apply the procedure to [compute missing keyframe offsets](#compute-missing-keyframe-offsets) to <var>property keyframes</var>.

        7.  <a id="ref-for-keyframe⑥②"></a>

            Add [keyframes](#keyframe) in <var>property keyframes</var> to <var>processed keyframes</var>.

    3.  <a id="ref-for-computed-keyframe-offset②⑥"></a>

        <a id="ref-for-keyframe⑥③"></a>

        Sort <var>processed keyframes</var> by the [computed keyframe offset](#computed-keyframe-offset) of each [keyframe](#keyframe) in increasing order.

    4.  <a id="ref-for-keyframe⑥④"></a>

        <a id="ref-for-computed-keyframe-offset②⑦"></a>

        Merge adjacent [keyframes](#keyframe) in <var>processed keyframes</var> when they have equal [computed keyframe offsets](#computed-keyframe-offset).

    5.  <a id="ref-for-dfn-nullable-type"></a>

        Let <var>offsets</var> be a sequence of [nullable](https://webidl.spec.whatwg.org/#dfn-nullable-type) `double` values assigned based on the type of the "offset" member of the <var>property-indexed&#xA;keyframe</var> as follows:

        `sequence<double?>`,  
        The value of "offset" as-is.

        `double?`,  
        A sequence of length one with the value of "offset" as its single item, i.e. « `offset` »,

    6.  <a id="ref-for-keyframe-offset①①"></a>

        <a id="ref-for-keyframe⑥⑤"></a>

        Assign each value in <var>offsets</var> to the [keyframe offset](#keyframe-offset) of the [keyframe](#keyframe) with corresponding position in <var>processed keyframes</var> until the end of either sequence is reached.

    7.  Let <var>easings</var> be a sequence of `DOMString` values assigned based on the type of the "easing" member of the <var>property-indexed keyframe</var> as follows:

        `sequence<DOMString>`,  
        The value of "easing" as-is.

        `DOMString`,  
        A sequence of length one with the value of "easing" as its single item, i.e. « `easing` »,

    8.  If <var>easings</var> is an empty sequence, let it be a sequence of length one containing the single value "linear", i.e. « "linear" ».

    9.  If <var>easings</var> has fewer items than <var>processed&#xA;keyframes</var>, repeat the elements in <var>easings</var> successively starting from the beginning of the list until <var>easings</var> has as many items as <var>processed&#xA;keyframes</var>.

        > <strong data-conversion-semantic="example">Example</strong>
        >
        > <a id="example-649bc6e3"></a> For example, if <var>processed keyframes</var> has five items, and <var>easings</var> is the sequence « "ease-in", "ease-out" », <var>easings</var> would be repeated to become « "ease-in", "ease-out", "ease-in", "ease-out", "ease-in" ».

    10. If <var>easings</var> has more items than <var>processed&#xA;keyframes</var>, store the excess items as <var>unused&#xA;easings</var>.

    11. <a id="ref-for-keyframe⑥⑥"></a>

        Assign each value in <var>easings</var> to a property named "easing" on the [keyframe](#keyframe) with the corresponding position in <var>processed keyframes</var> until the end of <var>processed keyframes</var> is reached.

    12. If the "composite" member of the <var>property-indexed&#xA;keyframe</var> is <em>not</em> an empty sequence:

        1.  <a id="ref-for-enumdef-compositeoperationorauto④"></a>

            <a id="ref-for-enumdef-compositeoperationorauto⑤"></a>

            Let <var>composite modes</var> be a sequence of <code><a href="#enumdef-compositeoperationorauto">CompositeOperationOrAuto</a></code> values assigned from the "composite" member of <var>property-indexed&#xA;keyframe</var>. If that member is a single <code><a href="#enumdef-compositeoperationorauto">CompositeOperationOrAuto</a></code> value operation, let <var>composite modes</var> be a sequence of length one, with the value of the "composite" as its single item.

        2.  As with <var>easings</var>, if <var>composite modes</var> has fewer items than <var>processed keyframes</var>, repeat the elements in <var>composite modes</var> successively starting from the beginning of the list until <var>composite modes</var> has as many items as <var>processed keyframes</var>.

        3.  <a id="ref-for-dom-compositeoperationorauto-auto②"></a>

            <a id="ref-for-keyframe-specific-composite-operation⑤"></a>

            <a id="ref-for-keyframe⑥⑦"></a>

            Assign each value in <var>composite modes</var> that is not <code><a href="#dom-compositeoperationorauto-auto">auto</a></code> to the [keyframe-specific composite operation](#keyframe-specific-composite-operation) on the [keyframe](#keyframe) with the corresponding position in <var>processed keyframes</var> until the end of <var>processed&#xA;keyframes</var> is reached.

6.  <a id="ref-for-loosely-sorted-by-offset"></a>

    <a id="ref-for-dfn-throw①④"></a>

    If <var>processed keyframes</var> is not [loosely sorted by offset](#loosely-sorted-by-offset), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort these steps.

7.  <a id="ref-for-keyframe⑥⑧"></a>

    <a id="ref-for-keyframe-offset①②"></a>

    <a id="ref-for-dfn-throw①⑤"></a>

    If there exist any [keyframe](#keyframe) in <var>processed keyframes</var> whose [keyframe offset](#keyframe-offset) is non-null and less than zero or greater than one, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort these steps.

8.  For each <var>frame</var> in <var>processed keyframes</var>, perform the following steps:

    1.  For each property-value pair in <var>frame</var>, parse the property value using the syntax specified for that property.

        If the property value is invalid according to the syntax for the property, discard the property-value pair. User agents that provide support for diagnosing errors in content SHOULD produce an appropriate warning highlighting the invalid property value.

    2.  <a id="ref-for-easing-function①⑥"></a>

        <a id="ref-for-dom-effecttiming-easing④"></a>

        <a id="ref-for-dictdef-effecttiming①②"></a>

        Let the [timing function](https://www.w3.org/TR/css-easing-1/#easing-function) of <var>frame</var> be the result of parsing the "easing" property on <var>frame</var> using the CSS syntax defined for the <code><a href="#dom-effecttiming-easing">easing</a></code> member of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> dictionary.

        <a id="ref-for-dfn-throw①⑥"></a>

        If parsing the "easing" property fails, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: Using the CSS parser in both of the above steps implies that CSS comments and escaping are allowed but are not retained when the value is successfully parsed.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: In the case where the "easing" property fails to parse, it is important that the TypeError is thrown <em>after</em> all reading the properties from <var>object</var> since failing to do so would be observable and will not match the behavior if partially open-ended dictionaries are later supported in WebIDL.

9.  <a id="ref-for-dom-effecttiming-easing⑤"></a>

    <a id="ref-for-dictdef-effecttiming①③"></a>

    <a id="ref-for-dfn-throw①⑦"></a>

    Parse each of the values in <var>unused easings</var> using the CSS syntax defined for <code><a href="#dom-effecttiming-easing">easing</a></code> member of the <code><a href="#dictdef-effecttiming">EffectTiming</a></code> interface, and if any of the values fail to parse, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a TypeError and abort this procedure.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > This final step is required in order to provide consistent behavior such that a TypeError is thrown in all of the following cases:
    >
    > ```javascript
    > elem.animate({ easing: 'invalid' });
    > elem.animate({ easing: ['invalid'] });
    > elem.animate([{ easing: 'invalid' }]);
    > ```
#### <a id="the-keyframeeffectoptions-dictionary"></a>6.6.4. The `KeyframeEffectOptions` dictionary

<a id="ref-for-dom-keyframeeffect-keyframeeffect④"></a>

<a id="ref-for-dictdef-keyframeeffectoptions⑥"></a>

Additional parameters may be passed to the <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect(target, keyframes,&#xA;options)</a></code> constructor by providing a <code><a href="#dictdef-keyframeeffectoptions">KeyframeEffectOptions</a></code> object.

<a id="dictdef-keyframeeffectoptions"></a>

<a id="ref-for-dictdef-effecttiming①④"></a>

<a id="ref-for-enumdef-compositeoperation②"></a>

<a id="ref-for-dom-keyframeeffectoptions-composite"></a>

<a id="ref-for-cssomstring②"></a>

<a id="ref-for-dom-keyframeeffectoptions-pseudoelement②"></a>

```text
dictionary KeyframeEffectOptions : EffectTiming {
    CompositeOperation composite = "replace";
    CSSOMString?       pseudoElement = null;
};
```
<a id="ref-for-enumdef-compositeoperation③"></a>

<a id="dom-keyframeeffectoptions-composite"></a>`composite`, of type [CompositeOperation](#enumdef-compositeoperation), defaulting to `"replace"`

<a id="ref-for-composite-operation①⑦"></a>

<a id="ref-for-effect-stack①⑥"></a>

<a id="ref-for-compositeoperation①"></a>

<a id="ref-for-keyframe⑥⑨"></a>

<a id="ref-for-dom-compositeoperationorauto-auto③"></a>

<a id="ref-for-keyframe-specific-composite-operation⑥"></a>

The [composite operation](#composite-operation) used to composite this animation with the [effect stack](#effect-stack), as specified by one of the [CompositeOperation](#compositeoperation) enumeration values. This is used for all [keyframes](#keyframe) that specify a <code><a href="#dom-compositeoperationorauto-auto">auto</a></code> [keyframe-specific composite operation](#keyframe-specific-composite-operation).

<a id="ref-for-cssomstring③"></a>

<a id="dom-keyframeeffectoptions-pseudoelement"></a>`pseudoElement`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), nullable, defaulting to `null`

<a id="ref-for-pseudo-element⑤"></a>

<a id="ref-for-keyframe-effect-effect-target②③"></a>

<a id="ref-for-effect-target-target-element⑦"></a>

The [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) selector (which must be valid or `null`) used to specify the [effect target](#keyframe-effect-effect-target) given the [target element](#effect-target-target-element).

### <a id="the-compositeoperation-enumeration"></a>6.7. The `CompositeOperation` and `CompositeOperationOrAuto` enumerations

<a id="ref-for-keyframe-effect④①"></a>

The possible values of an [keyframe effect](#keyframe-effect)’s composition behavior are represented by the <a id="compositeoperation"></a>CompositeOperation enumeration.

<a id="enumdef-compositeoperation"></a>

<a id="ref-for-dom-compositeoperation-replace"></a>

<a id="ref-for-dom-compositeoperation-add"></a>

<a id="ref-for-dom-compositeoperation-accumulate"></a>

```text
enum CompositeOperation { "replace", "add", "accumulate" };
```
<a id="dom-compositeoperation-replace"></a>`replace`  
<a id="ref-for-composite-operation-replace②"></a>

<a id="ref-for-composite-operation①⑧"></a>

<a id="ref-for-animation-effect①①②"></a>

<a id="ref-for-underlying-value①⑤"></a>

Corresponds to the [replace](#composite-operation-replace) [composite operation](#composite-operation) value such that the [animation effect](#animation-effect) overrides the [underlying value](#underlying-value) it is combined with.

<a id="dom-compositeoperation-add"></a>`add`  
<a id="ref-for-composite-operation-add③"></a>

<a id="ref-for-composite-operation①⑨"></a>

<a id="ref-for-animation-effect①①③"></a>

<a id="ref-for-addition②"></a>

<a id="ref-for-underlying-value①⑥"></a>

Corresponds to the [add](#composite-operation-add) [composite operation](#composite-operation) value such that the [animation effect](#animation-effect) is [added](https://www.w3.org/TR/css-values-4/#addition) to the [underlying value](#underlying-value) with which it is combined.

<a id="dom-compositeoperation-accumulate"></a>`accumulate`  
<a id="ref-for-composite-operation-accumulate"></a>

<a id="ref-for-composite-operation②⓪"></a>

<a id="ref-for-animation-effect①①④"></a>

<a id="ref-for-accumulation②"></a>

<a id="ref-for-underlying-value①⑦"></a>

Corresponds to the [accumulate](#composite-operation-accumulate) [composite operation](#composite-operation) value such that the [animation effect](#animation-effect) is [accumulated](https://www.w3.org/TR/css-values-4/#accumulation) on to the [underlying value](#underlying-value).

<a id="ref-for-keyframe⑦⓪"></a>

<a id="ref-for-enumdef-compositeoperation④"></a>

<a id="ref-for-dom-compositeoperationorauto-auto④"></a>

The possible values of a [keyframe](#keyframe)'s composition behavior share the same values as the <code><a href="#enumdef-compositeoperation">CompositeOperation</a></code> enumeration along with the additional <code><a href="#dom-compositeoperationorauto-auto">auto</a></code> value.

<a id="enumdef-compositeoperationorauto"></a>

<a id="ref-for-dom-compositeoperation-replace①"></a>

<a id="ref-for-dom-compositeoperation-add①"></a>

<a id="ref-for-dom-compositeoperation-accumulate①"></a>

<a id="ref-for-dom-compositeoperationorauto-auto⑤"></a>

```text
enum CompositeOperationOrAuto { "replace", "add", "accumulate", "auto" };
```
<a id="dom-compositeoperationorauto-auto"></a>`auto`  
<a id="ref-for-composite-operation②①"></a>

<a id="ref-for-keyframe-effect④②"></a>

Indicates that the [composite operation](#composite-operation) of the associated [keyframe effect](#keyframe-effect) should be used.

### <a id="the-animatable-interface-mixin"></a>6.8. The `Animatable` interface mixin

<a id="ref-for-keyframeeffect①②"></a>

<a id="ref-for-animatable②"></a>

Objects that may be the target of an <code><a href="#keyframeeffect">KeyframeEffect</a></code> object implement the <code><a href="#animatable">Animatable</a></code> interface mixin.

<a id="animatable"></a>

<a id="ref-for-animation⑧"></a>

<a id="ref-for-dom-animatable-animate②"></a>

<a id="ref-for-idl-object③"></a>

<a id="ref-for-dom-animatable-animate-keyframes-options-keyframes"></a>

<a id="ref-for-idl-unrestricted-double①②"></a>

<a id="ref-for-dictdef-keyframeanimationoptions①"></a>

<a id="ref-for-dom-animatable-animate-keyframes-options-options"></a>

<a id="ref-for-idl-sequence④"></a>

<a id="ref-for-animation⑨"></a>

<a id="ref-for-dom-animatable-getanimations"></a>

<a id="ref-for-dictdef-getanimationsoptions"></a>

<a id="ref-for-dom-animatable-getanimations-options-options"></a>

<a id="dictdef-keyframeanimationoptions"></a>

<a id="ref-for-dictdef-keyframeeffectoptions⑦"></a>

<a id="ref-for-idl-DOMString①①"></a>

<a id="ref-for-dom-keyframeanimationoptions-id"></a>

<a id="ref-for-animationtimeline⑤"></a>

<a id="ref-for-dom-keyframeanimationoptions-timeline"></a>

<a id="dictdef-getanimationsoptions"></a>

<a id="ref-for-idl-boolean②"></a>

<a id="ref-for-dom-getanimationsoptions-subtree"></a>

```text
interface mixin Animatable {
    Animation           animate(object? keyframes,
                                optional (unrestricted double or KeyframeAnimationOptions) options = {});
    sequence<Animation> getAnimations(optional GetAnimationsOptions options = {});
};

dictionary KeyframeAnimationOptions : KeyframeEffectOptions {
    DOMString id = "";
    AnimationTimeline? timeline;
};

dictionary GetAnimationsOptions {
    boolean subtree = false;
};
```
<a id="dom-animatable-animate"></a>`Animation animate(keyframes, options)`  
Performs the following steps:

1.  Let <var>target</var> be the object on which this method was called.

2.  <a id="ref-for-keyframeeffect①③"></a>

    <a id="ref-for-concept-relevant-realm⑥"></a>

    <a id="ref-for-dom-keyframeeffect-keyframeeffect⑤"></a>

    Construct a new <code><a href="#keyframeeffect">KeyframeEffect</a></code> object, <var>effect</var>, in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>target</var> by using the same procedure as the <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect(target, keyframes,&#xA;options)</a></code> constructor, passing <var>target</var> as the <var>target</var> argument, and the <var>keyframes</var> and <var>options</var> arguments as supplied.

    If the above procedure causes an exception to be thrown, propagate the exception and abort this procedure.

3.  <a id="ref-for-dictdef-keyframeanimationoptions②"></a>

    <a id="ref-for-document-default-document-timeline⑦"></a>

    <a id="ref-for-concept-node-document"></a>

    If <var>options</var> is a <code><a href="#dictdef-keyframeanimationoptions">KeyframeAnimationOptions</a></code> object, let <var>timeline</var> be the `timeline` member of <var>options</var> or, if `timeline` member of <var>options</var> is missing, be the [default document timeline](#document-default-document-timeline) of the [node document](https://dom.spec.whatwg.org/#concept-node-document) of the element on which this method was called.

4.  <a id="ref-for-animation①⓪"></a>

    <a id="ref-for-concept-relevant-realm⑦"></a>

    <a id="ref-for-dom-animation-animation①"></a>

    Construct a new <code><a href="#animation">Animation</a></code> object, <var>animation</var>, in the [relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm) of <var>target</var> by using the same procedure as the <code><a href="#dom-animation-animation">Animation()</a></code> constructor, passing <var>effect</var> and <var>timeline</var> as arguments of the same name.

5.  <a id="ref-for-dictdef-keyframeanimationoptions③"></a>

    <a id="ref-for-dom-animation-id①"></a>

    If <var>options</var> is a <code><a href="#dictdef-keyframeanimationoptions">KeyframeAnimationOptions</a></code> object, assign the value of the `id` member of <var>options</var> to <var>animation</var>’s <code><a href="#dom-animation-id">id</a></code> attribute.

6.  <a id="ref-for-play-an-animation⑤"></a>

    Run the procedure to [play an animation](#play-an-animation) for <var>animation</var> with the <var>auto-rewind</var> flag set to true.

7.  Return <var>animation</var>.

<em>This section is non-normative</em>

The following code fragment:

```javascript
var animation = elem.animate({ opacity: 0 }, 2000);
```
is roughly equivalent to:

```javascript
var effect = new KeyframeEffect(elem, { opacity: 0 }, 2000);
var animation = new Animation(effect, elem.ownerDocument.timeline);
animation.play();
```
<a id="dom-animatable-animate-keyframes-options-keyframes"></a>`keyframes`  
<a id="ref-for-keyframe⑦①"></a>

<a id="ref-for-dom-keyframeeffect-keyframeeffect⑥"></a>

The [keyframes](#keyframe) to use. This value is passed to the <code><a href="#dom-keyframeeffect-keyframeeffect">KeyframeEffect(target, keyframes,&#xA;options)</a></code> constructor as the <var>keyframes</var> parameter and has the same interpretation as defined for that constructor.

<a id="dom-animatable-animate-keyframes-options-options"></a>`options`  
<a id="ref-for-keyframeeffect①④"></a>

<a id="ref-for-animation①①"></a>

The timing and animation options for the created <code><a href="#keyframeeffect">KeyframeEffect</a></code> and <code><a href="#animation">Animation</a></code>.

<a id="dom-animatable-getanimations"></a>`  sequence<Animation> getAnimations(options) `  
<a id="ref-for-relevant-animations"></a>

<a id="ref-for-dom-animatable-getanimations-options-options①"></a>

<a id="ref-for-dom-getanimationsoptions-subtree①"></a>

<a id="ref-for-relevant-animations-for-a-subtree"></a>

Returns the set of [relevant animations](#relevant-animations) for this object, or, if an <code><a href="#dom-animatable-getanimations-options-options">options</a></code> parameter is passed with <code><a href="#dom-getanimationsoptions-subtree">subtree</a></code> set to true, returns the set of [relevant animations for a subtree](#relevant-animations-for-a-subtree) for this object.

<a id="ref-for-concept-animation⑦①"></a>

The returned list is sorted using the composite order described for the associated [animations](#concept-animation) of effects in [§ 5.4.2 The effect stack](#the-effect-stack).

<a id="ref-for-style-change-event①"></a>

<a id="ref-for-effect-target-target-element⑧"></a>

Calling this method triggers a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event) for the [target element](#effect-target-target-element). As a result, the returned list reflects the state <em>after</em> applying any pending style changes to animation such as changes to animation-related style properties that have yet to be processed.

<a id="dom-animatable-getanimations-options-options"></a>`options`  
<a id="ref-for-dom-animatable-getanimations①"></a>

Parameters governing the set of animations returned by <code><a href="#dom-animatable-getanimations">getAnimations()</a></code>.

<a id="ref-for-idl-DOMString①②"></a>

<a id="dom-keyframeanimationoptions-id"></a>`id`, of type [DOMString](https://webidl.spec.whatwg.org/#idl-DOMString), defaulting to `""`

<a id="ref-for-animation①②"></a>

<a id="ref-for-dom-animation-id②"></a>

The string to assign to the generated <code><a href="#animation">Animation</a></code>'s <code><a href="#dom-animation-id">id</a></code> attribute.

<a id="ref-for-animationtimeline⑥"></a>

<a id="dom-keyframeanimationoptions-timeline"></a>`timeline`, of type [AnimationTimeline](#animationtimeline), nullable

<a id="ref-for-timeline⑤⑨"></a>

<a id="ref-for-concept-animation⑦②"></a>

An optional value which, if present, specifies the [timeline](#timeline) with which to associate the newly-created [animation](#concept-animation).

<a id="ref-for-idl-boolean③"></a>

<a id="dom-getanimationsoptions-subtree"></a>`subtree`, of type [boolean](https://webidl.spec.whatwg.org/#idl-boolean), defaulting to `false`

<a id="ref-for-concept-animation⑦③"></a>

<a id="ref-for-animation-effect①①⑤"></a>

<a id="ref-for-effect-target-target-element⑨"></a>

<a id="ref-for-concept-tree-descendant②"></a>

<a id="ref-for-dom-animatable-getanimations②"></a>

If true, indicates that [animations](#concept-animation) associated with an [animation effect](#animation-effect) whose [target element](#effect-target-target-element) is a [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) of the object on which <code><a href="#dom-animatable-getanimations">getAnimations()</a></code> is called should also be included in the result.

### <a id="extensions-to-the-document-interface"></a>6.9. Extensions to the `Document` interface

<a id="ref-for-document①①"></a>

The following extensions are made to the <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> interface defined in [\[DOM\]](#biblio-dom).

<a id="ref-for-document①②"></a>

<a id="ref-for-documenttimeline②"></a>

<a id="ref-for-dom-document-timeline"></a>

```text
partial interface Document {
    readonly attribute DocumentTimeline timeline;
};
```
<a id="ref-for-documenttimeline③"></a>

<a id="dom-document-timeline"></a>`timeline`, of type [DocumentTimeline](#documenttimeline), readonly

<a id="ref-for-documenttimeline④"></a>

<a id="ref-for-document-default-document-timeline⑧"></a>

The <code><a href="#documenttimeline">DocumentTimeline</a></code> object representing the [default document timeline](#document-default-document-timeline).

### <a id="extensions-to-the-documentorshadowroot-interface-mixin"></a>6.10. Extensions to the `DocumentOrShadowRoot` interface mixin

<a id="ref-for-documentorshadowroot"></a>

The following extensions are made to the <code><a href="https://dom.spec.whatwg.org/#documentorshadowroot">DocumentOrShadowRoot</a></code> interface mixin defined in [\[DOM\]](#biblio-dom).

<a id="ref-for-documentorshadowroot①"></a>

<a id="ref-for-idl-sequence⑤"></a>

<a id="ref-for-animation①③"></a>

<a id="ref-for-dom-documentorshadowroot-getanimations"></a>

```text
partial interface mixin DocumentOrShadowRoot {
    sequence<Animation> getAnimations();
};
```
<a id="dom-documentorshadowroot-getanimations"></a>`  sequence<Animation> getAnimations() `  
<a id="ref-for-relevant-animations-for-a-subtree①"></a>

<a id="ref-for-concept-document②"></a>

<a id="ref-for-concept-shadow-root②"></a>

Returns the set of [relevant animations for a subtree](#relevant-animations-for-a-subtree) for the [document](https://dom.spec.whatwg.org/#concept-document) or [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) on which this method is called.

<a id="ref-for-concept-animation⑦④"></a>

The returned list is sorted using the composite order described for the associated [animations](#concept-animation) of effects in [§ 5.4.2 The effect stack](#the-effect-stack).

<a id="ref-for-style-change-event②"></a>

Calling this method triggers a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event) for the document. As a result, the returned list reflects the state <em>after</em> applying any pending style changes to animation such as changes to animation-related style properties that have yet to be processed.

### <a id="extensions-to-the-element-interface"></a>6.11. Extensions to the `Element` interface

<a id="ref-for-element⑧"></a>

Since DOM Elements may be the target of an animation, the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> interface [\[DOM\]](#biblio-dom) is extended as follows:

<a id="ref-for-element⑨"></a>

<a id="ref-for-animatable③"></a>

```text
Element includes Animatable;
```
This allows the following kind of usage.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3abce906"></a>
>
> ```javascript
> elem.animate({ color: 'red' }, 2000);
> ```
### <a id="the-animationplaybackevent-interface"></a>6.12. The `AnimationPlaybackEvent` interface

<a id="ref-for-animation-playback-events②"></a>

<a id="ref-for-animationplaybackevent③"></a>

[Animation playback events](#animation-playback-events) are represented using the <code><a href="#animationplaybackevent">AnimationPlaybackEvent</a></code> interface.

<a id="ref-for-Exposed⑤"></a>

<a id="animationplaybackevent"></a>

<a id="ref-for-event"></a>

<a id="ref-for-dom-animationplaybackevent-animationplaybackevent"></a>

<a id="ref-for-idl-DOMString①③"></a>

<a id="dom-animationplaybackevent-animationplaybackevent-type-eventinitdict-type"></a>

<a id="ref-for-dictdef-animationplaybackeventinit"></a>

<a id="dom-animationplaybackevent-animationplaybackevent-type-eventinitdict-eventinitdict"></a>

<a id="ref-for-idl-double②⑦"></a>

<a id="ref-for-dom-animationplaybackevent-currenttime③"></a>

<a id="ref-for-idl-double②⑧"></a>

<a id="ref-for-dom-animationplaybackevent-timelinetime④"></a>

<a id="dictdef-animationplaybackeventinit"></a>

<a id="ref-for-dictdef-eventinit"></a>

<a id="ref-for-idl-double②⑨"></a>

<a id="ref-for-dom-animationplaybackeventinit-currenttime"></a>

<a id="ref-for-idl-double③⓪"></a>

<a id="ref-for-dom-animationplaybackeventinit-timelinetime"></a>

```text
[Exposed=Window]
interface AnimationPlaybackEvent : Event {
    constructor(DOMString type, optional AnimationPlaybackEventInit eventInitDict = {});
    readonly attribute double? currentTime;
    readonly attribute double? timelineTime;
};
dictionary AnimationPlaybackEventInit : EventInit {
    double? currentTime = null;
    double? timelineTime = null;
};
```
<a id="dom-animationplaybackevent-animationplaybackevent"></a>`  AnimationPlaybackEvent(type, eventInitDict) `  
<a id="ref-for-animationplaybackevent④"></a>

<a id="ref-for-constructing-events"></a>

Constructs a new <code><a href="#animationplaybackevent">AnimationPlaybackEvent</a></code> object using the procedure defined for [constructing events](https://dom.spec.whatwg.org/#constructing-events) [\[DOM\]](#biblio-dom).

<a id="ref-for-idl-double③①"></a>

<a id="dom-animationplaybackevent-currenttime"></a>`currentTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), readonly, nullable

<a id="ref-for-animation-current-time⑤①"></a>

<a id="ref-for-concept-animation⑦⑤"></a>

<a id="ref-for-play-state-idle⑦"></a>

The [current time](#animation-current-time) of the [animation](#concept-animation) that generated the event at the moment the event as queued. This will be `null` if the <a id="ref-for-concept-animation⑦⑥"></a>animation was [idle](#play-state-idle) at the time the event was generated.

<a id="ref-for-idl-double③②"></a>

<a id="dom-animationplaybackevent-timelinetime"></a>`timelineTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), readonly, nullable

<a id="ref-for-time-value④⑤"></a>

<a id="ref-for-timeline⑥⓪"></a>

<a id="ref-for-concept-animation⑦⑦"></a>

<a id="ref-for-inactive-timeline①⑨"></a>

The [time value](#time-value) of the [timeline](#timeline) with which the [animation](#concept-animation) that generated the event is associated at the moment the event was queued. This will be `null` if the <a id="ref-for-concept-animation⑦⑧"></a>animation was not associated with an [active timeline](#inactive-timeline) at the time the event was queued.

<a id="ref-for-idl-double③③"></a>

<a id="dom-animationplaybackeventinit-currenttime"></a>`currentTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable, defaulting to `null`

<a id="ref-for-dom-animationplaybackevent-currenttime④"></a>

See the description of the <code><a href="#dom-animationplaybackevent-currenttime">currentTime</a></code> attribute.

<a id="ref-for-idl-double③④"></a>

<a id="dom-animationplaybackeventinit-timelinetime"></a>`timelineTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), nullable, defaulting to `null`

<a id="ref-for-dom-animationplaybackevent-timelinetime⑤"></a>

See the description of the <code><a href="#dom-animationplaybackevent-timelinetime">timelineTime</a></code> attribute.

### <a id="model-liveness"></a>6.13. Model liveness

Changes made to any part of the model, cause the entire timing model to be updated and any dependent style.

<a id="ref-for-style-change-event③"></a>

Unless otherwise stated, invoking the methods or constructors, or getting or setting the members of interfaces defined in the programming interface section of this specification does <em>not</em> produce a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event).

<a id="ref-for-style-change-event④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Other specifications that extend this specification are expected to refine the requirements on [style change events](https://www.w3.org/TR/css-transitions-1/#style-change-event) by introducing circumstances where such events <em>are</em> triggered. For example, when the interfaces in this specification represent animations defined by CSS markup, many of their methods will need to trigger <a id="ref-for-style-change-event⑤"></a>style change events in order to reflect changes to the specified style.

<em>This section is non-normative</em>

Based on the above requirement and normative requirements elsewhere in this specification, the following invariants can be observed:

Changes made to the Web Animations model take effect immediately

<a id="ref-for-keyframeeffect①⑤"></a>

<a id="ref-for-animation①④"></a>

For example, if the <code><a href="#keyframeeffect">KeyframeEffect</a></code> associated with an <code><a href="#animation">Animation</a></code> is seeked (see [§ 4.4.4 Setting the current time of an animation](#setting-the-current-time-of-an-animation)) via the programming interface, the value returned when querying the animation’s `startTime` will reflect updated state of the model immediately.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f6015dd0"></a>
>
> ```javascript
> // Initially animation.effect.getComputedTiming().localTime is 3000
> animation.currentTime += 2000;
> alert(animation.effect.getComputedTiming().localTime); // Displays "5000"
> ```
Querying the computed style of a property affected by animation returns the fully up-to-date state of the animation

<a id="ref-for-animation①⑤"></a>

For example, if the used style of an element is queried immediately after applying a new <code><a href="#animation">Animation</a></code> to that element, the result of the new animation will be incorporated in the value returned.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ac9a1207"></a>
>
> ```javascript
> // Set opacity to 0 immediately
> elem.animate({ opacity: 0 }, { fill: 'forwards' });
> alert(window.getComputedStyle(elem).opacity); // Displays "0"
> ```
Changes made within the same task are synchronized such that the whole set of changes is rendered together

As a result of changes to the model taking effect immediately combined with ECMAScript’s run-to-completion semantics, there should never be a situation where, for example, <em>only</em> the changes to specified style are rendered without applying animation.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a559e918"></a>
>
> ```javascript
> // Fade the opacity with fallback for browsers that don’t
> // support Element.animate
> elem.style.opacity = '0';
> elem.animate([ { opacity: 1 }, { opacity: 0 } ], 500);
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note, however, that in the example above, a user agent may render a frame with <em>none</em> of the above changes applied. This might happen, for example, if rendering occurs in a separate process that is scheduled to run shortly after the above task completes but before the changes can be communicated to the process.

<a id="ref-for-document-timeline⑥"></a>

The value returned by the `currentTime` attribute of a [document timeline](#document-timeline) will not change within a task

<a id="ref-for-timeline⑥①"></a>

<a id="ref-for-timeline-current-time①⓪"></a>

<a id="ref-for-update-animations-and-send-events④"></a>

Due to the requirement on [timelines](#timeline) to update their [current time](#timeline-current-time) each time the [update animations and send events](#update-animations-and-send-events) procedure is run, querying the `currentTime` twice within a long block of code that is executed in the same script block will return the same value as shown in the following example.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-16dafcc0"></a>
>
> ```javascript
> var a = document.timeline.currentTime;
> // ... many lines of code ...
> var b = document.timeline.currentTime;
> alert(b - a); // Displays 0
> ```
The time passed to a `requestAnimationFrame` callback will be equal to `document.timeline.currentTime`

<a id="ref-for-event-loop-processing-model"></a>

<a id="ref-for-update-animations-and-send-events⑤"></a>

<a id="ref-for-run-the-animation-frame-callbacks"></a>

<a id="ref-for-timeline-current-time①①"></a>

<a id="ref-for-document-default-document-timeline⑨"></a>

Since HTML’s [event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model) defines that the procedure to [update animations and send events](#update-animations-and-send-events) is performed prior to [running animation frame callbacks](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html.html#run-the-animation-frame-callbacks), and since the time passed to such callbacks is the same <var>now</var> timestamp is passed to both procedures, the [current time](#timeline-current-time) of a the [default document timeline](#document-default-document-timeline) should match the time passed to `requestAnimationFrame`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a0637e5c"></a>
>
> ```javascript
> window.requestAnimationFrame(function(now) {
>   // Displays 0
>   alert(now - document.timeline.currentTime);
> });
> ```
Calling methods from this programming interface will generally <em>not</em> cause transitions to be triggered

Consider the following example:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a8ba32df"></a>
>
> ```javascript
> // Setup transition start point
> div.style.opacity = '1';
> getComputedStyle(div).opacity;
> 
> // Setup transition end point
> div.style.transition = 'opacity 1s';
> div.style.opacity = '0';
> 
> // Fire an animation
> div.animate({ opacity: [0.5, 1] }, 500);
> 
> // Wait for the transition to end -- the following will never be called!
> div.addEventListener('transitionend', () => {
>   console.log('transitionend');
> });
> ```
<a id="ref-for-dom-animatable-animate③"></a>

<a id="ref-for-style-change-event⑥"></a>

<a id="ref-for-before-change-style"></a>

<a id="ref-for-after-change-style"></a>

<a id="ref-for-transitionend"></a>

In this case, calling <code><a href="#dom-animatable-animate">animate()</a></code> will <em>not</em> trigger a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event). As a result, the pending style change will be processed at the same time as the style change resulting from the new animation. Since the animation style will override the [before-change style](https://www.w3.org/TR/css-transitions-1/#before-change-style) and the [after-change style](https://www.w3.org/TR/css-transitions-1/#after-change-style), no transition will be generated and the event handler for the [transitionend](https://drafts.csswg.org/css-transitions/#transitionend) event will never be called.

## <a id="integration-with-media-fragments"></a>7. Integration with Media Fragments

<a id="ref-for-mime-registration"></a>

The Media Fragments specification [\[MEDIA-FRAGS\]](#biblio-media-frags) defines a means for addressing a temporal range of a media resource. The application of media fragments depends on the MIME type of the resource on which they are specified. For resources with the [SVG MIME type](https://svgwg.org/svg2-draft/mimereg.html#mime-registration) [\[SVG11\]](#biblio-svg11), the application of temporal parameters is defined in the [Animation Elements](https://svgwg.org/specs/animation-elements/) specification.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: media fragments are defined to operate on resources based on their MIME type. As a result, temporal addressing may not be supported in all situations where Web Animations content is used.

## <a id="interaction-with-page-display"></a>8. Interaction with page display

<a id="ref-for-an-entry-with-persisted-user-state"></a>

<a id="ref-for-session-history-entry"></a>

HTML permits user agents to store [user-agent defined state](https://html.spec.whatwg.org/multipage/browsers.html#an-entry-with-persisted-user-state) along with a [session history entry](https://html.spec.whatwg.org/multipage/browsers.html#session-history-entry) so that as a user navigates between pages, the previous state of the page can be restored including state such as scroll position [\[HTML\]](#biblio-html).

<a id="ref-for-media-element①"></a>

<a id="ref-for-time-value④⑥"></a>

<a id="ref-for-timeline⑥②"></a>

User agents that pause and resume [media elements](https://html.spec.whatwg.org/multipage/embedded-content.html#media-element) when the referencing document is unloaded and traversed, are encouraged to apply consistent handling to documents containing Web Animations content. If provided, this behavior SHOULD be achieved by adjusting the [time values](#time-value) of any [timelines](#timeline) that track wallclock time.

<a id="ref-for-time-value④⑦"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a128cfff"></a> Is this at odds with those [time values](#time-value) being relative to `navigationStart` and with `requestAnimationFrame` using the same time as `document.timeline.currentTime`? [\[Issue \#2083\]](https://github.com/w3c/csswg-drafts/issues/2083)

## <a id="implementation-requirements"></a>9. Implementation requirements

### <a id="precision-of-time-values"></a>9.1. Precision of time values

<a id="ref-for-time-value④⑧"></a>

The internal representation of time values is implementation dependent however, it is RECOMMENDED that user agents be able to represent input time values with microsecond precision so that a [time value](#time-value) (which nominally represents milliseconds) of 0.001 is distinguishable from 0.0.

### <a id="conformance-criteria"></a>9.2. Conformance criteria

This specification defines an abstract model for animation and, as such, for user agents that do not support scripting, there are no conformance criteria since there is no testable surface area.

User agents that do not support scripting, however, may implement additional technologies defined in terms of this specification in which case the definitions provided in this specification will form part of the conformance criteria of the additional technology.

A <a id="conforming-scripted-web-animations-user-agent"></a>conforming scripted Web Animations user agent is a user agent that implements the API defined in [§ 6 Programming interface](#programming-interface).

## <a id="acknowledgements"></a>10. Acknowledgements

Thank you to Steve Block, Michael Giuffrida, Ryan Seys, and Eric Willigers for their contributions to this specification.

Thank you also to Michiel "Pomax" Kamermans for help with the equations for a proposed smooth timing function although this feature has been deferred to a subsequent specification.

Our deep gratitude goes out to [Southern Star Animation](http://www.endemolshine.com.au) for their kind generosity and patience in introducing the editors to the processes and techniques used producing broadcast animations.

## <a id="changes-since-last-publication"></a>11. Changes since last publication

The following changes have been made since the [8 September 2022 Working Draft](https://www.w3.org/TR/2022/WD-web-animations-1-20220908/):

- <a id="ref-for-set-the-playback-rate③"></a>

  Updated the procedure to [set the playback rate](#set-the-playback-rate) to main the start time and swap start time position when playback rate is flipped on non-monotonic timelines.

- <a id="ref-for-relevant-animations①"></a>

  Factored out the definition of [relevant animations](#relevant-animations) for use by other specs.

- <a id="ref-for-current③"></a>

  Updated the definition of [current](#current) animation effects such that they are considered current if they are non-idle and associated with non-increasing timelines.

- Fixed a code example in the [§ 4.6 Fill behavior](#fill-behavior) section.

The [changelog](https://github.com/w3c/csswg-drafts/commits/main/web-animations-1) provides a more detailed history.

## <a id="animation-types"></a>Appendix A: Animation types of existing properties

<a id="ref-for-animation-type①⓪"></a>

<a id="ref-for-by-computed-value③"></a>

Typically the [animation type](#animation-type) of a property is included along with its definition. However, for some properties defined in older or very mature specifications the <a id="ref-for-animation-type①①"></a>animation type information is not included. All such properties are assumed to have an <a id="ref-for-animation-type①②"></a>animation type of [by computed value](#by-computed-value) unless they are one of the exceptions listed below.

<a id="ref-for-propdef-font-weight"></a>

### <a id="animating-font-weight"></a>Animation of [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight)

<a id="ref-for-propdef-font-weight①"></a>

[font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight) property values prior to level 4 are [combined](https://www.w3.org/TR/css-values-4/#combining-values) as follows:

- <a id="ref-for-interpolation②"></a>

  <a id="ref-for-number-value"></a>

  [Interpolated](https://www.w3.org/TR/css-values-4/#interpolation) via discrete steps (multiples of 100). The interpolation happens in real number space as for [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s and is converted to an integer by rounding to the nearest multiple of 100, with values halfway between multiples of 100 rounded towards positive infinity.

- <a id="ref-for-addition③"></a>

  <a id="ref-for-propdef-font-weight②"></a>

  [Addition](https://www.w3.org/TR/css-values-4/#addition) of [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight) values is defined as <var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var>

<a id="ref-for-propdef-font-weight③"></a>

<a id="ref-for-animation-type①③"></a>

<a id="ref-for-by-computed-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition is obsoleted by [\[CSS-FONTS-4\]](#biblio-css-fonts-4) where the requirement that a [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight) value be a multiple of 100 is dropped. At that point the [animation type](#animation-type) for <a id="ref-for-propdef-font-weight④"></a>font-weight is simply [by computed value](#by-computed-value).

<a id="ref-for-propdef-visibility"></a>

### <a id="animating-visibility"></a>Animation of [visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility)

<a id="ref-for-propdef-visibility①"></a>

<a id="ref-for-valdef-visibility-visible"></a>

<a id="ref-for-interpolation③"></a>

<a id="ref-for-discrete⑤"></a>

For the [visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility) property, [visible](https://www.w3.org/TR/css-display-3/#valdef-visibility-visible) is [interpolated](https://www.w3.org/TR/css-values-4/#interpolation) as a discrete step where values of <var>p</var> between 0 and 1 map to <a id="ref-for-valdef-visibility-visible①"></a>visible and other values of <var>p</var> map to the closer endpoint; if neither value is <a id="ref-for-valdef-visibility-visible②"></a>visible then [discrete](#discrete) animation is used.

<a id="ref-for-propdef-box-shadow"></a>

<a id="ref-for-propdef-text-shadow"></a>

### <a id="animating-shadow-lists"></a>Animation of [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) and [text-shadow](https://www.w3.org/TR/css-text-decor-4/#propdef-text-shadow)

<a id="ref-for-propdef-box-shadow①"></a>

<a id="ref-for-propdef-text-shadow①"></a>

Animation the [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) or [text-shadow](https://www.w3.org/TR/css-text-decor-4/#propdef-text-shadow) property follows the procedures for [combining](https://www.w3.org/TR/css-values-4/#combining-values) <a id="combining-shadow-lists"></a>shadow lists as follows:

<a id="ref-for-by-computed-value⑤"></a>

<a id="ref-for-discrete⑥"></a>

<a id="ref-for-valdef-color-transparent"></a>

Each shadow in the list (treating none as a 0-length list) is interpolated component-wise as with [by computed value](#by-computed-value) behavior. However, if both input shadows are inset or both input shadows are not inset, then the interpolated shadow must match the input shadows in that regard. If any pair of input shadows has one inset and the other not inset, the entire shadow-list uses [discrete](#discrete) animation. If the lists of shadows have different lengths, then the shorter list is padded at the end with shadows whose color is [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent), all lengths are 0, and whose inset (or not) matches the longer list.

<a id="ref-for-addition④"></a>

<a id="ref-for-combining-shadow-lists"></a>

<a id="ref-for-list"></a>

<a id="ref-for-list-extend"></a>

[Addition](https://www.w3.org/TR/css-values-4/#addition) of two [shadow lists](#combining-shadow-lists) <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> is defined as [list](https://infra.spec.whatwg.org/#list) concatenation such that <var>V<sub>result</sub></var> is equal to <var>V<sub>a</sub></var> [extended](https://infra.spec.whatwg.org/#list-extend) with <var>V<sub>b</sub></var>.

<a id="ref-for-accumulation③"></a>

<a id="ref-for-combining-shadow-lists①"></a>

<a id="ref-for-discrete⑦"></a>

[Accumulation](https://www.w3.org/TR/css-values-4/#accumulation) of [shadow lists](#combining-shadow-lists) follows the matching rules for interpolation above, performing addition on each component according to its type, or falling back to [discrete](#discrete) animation if the inset values do not match.

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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- ["accumulate"](#dom-compositeoperation-accumulate), in § 6.7
- [accumulate](#dom-compositeoperation-accumulate), in § 6.7
- ["active"](#dom-animationreplacestate-active), in § 6.4.2
- [active-after boundary time](#active-after-boundary-time), in § 4.5.5
- [active duration](#active-duration), in § 4.8.2
- [activeDuration](#dom-computedeffecttiming-activeduration), in § 6.5.5
- [active interval](#active-interval), in § 4.5.3
- [active phase](#animation-effect-active-phase), in § 4.5.5
- [active replace state](#active-replace-state), in § 5.5.1
- [active time](#active-time), in § 4.8.3.1
- ["add"](#dom-compositeoperation-add), in § 6.7
- [add](#dom-compositeoperation-add), in § 6.7
- [after phase](#animation-effect-after-phase), in § 4.5.5
- ["alternate"](#dom-playbackdirection-alternate), in § 6.5.3
- ["alternate-reverse"](#dom-playbackdirection-alternate-reverse), in § 6.5.3
- [Animatable](#animatable), in § 6.8
- [animatable](#concept-animatable), in § 5.2
- [animate(keyframes)](#dom-animatable-animate), in § 6.8
- [animate(keyframes, options)](#dom-animatable-animate), in § 6.8
- [Animation](#animation), in § 6.4
- [animation](#concept-animation), in § 4.4
- [Animation()](#dom-animation-animation), in § 6.4
- [animation class](#animation-class), in § 5.4.1
- [animation composite order](#animation-composite-order), in § 5.4.2
- [animation direction](#animation-direction), in § 4.5.5
- [animation effect](#animation-effect), in § 4.5
- [Animation(effect)](#dom-animation-animation), in § 6.4
- [AnimationEffect](#animationeffect), in § 6.5
- [Animation(effect, timeline)](#dom-animation-animation), in § 6.4
- [Animation events](#animation-events), in § 4.4.18
- [animation frame](#animation-frame), in § 4.3
- [AnimationPlaybackEvent](#animationplaybackevent), in § 6.12
- [AnimationPlaybackEventInit](#dictdef-animationplaybackeventinit), in § 6.12
- [animation playback events](#animation-playback-events), in § 4.4.18.2
- [AnimationPlaybackEvent(type)](#dom-animationplaybackevent-animationplaybackevent), in § 6.12
- [AnimationPlaybackEvent(type, eventInitDict)](#dom-animationplaybackevent-animationplaybackevent), in § 6.12
- [AnimationPlayState](#enumdef-animationplaystate), in § 6.4.1
- [animation property name to IDL attribute name](#animation-property-name-to-idl-attribute-name), in § 6.6.2
- [AnimationReplaceState](#enumdef-animationreplacestate), in § 6.4.2
- [AnimationTimeline](#animationtimeline), in § 6.2
- [animation time to origin-relative time](#animation-time-to-origin-relative-time), in § 4.4.18.1
- [animation time to timeline time](#animation-time-to-timeline-time), in § 4.4.18.1
- [Animation type](#animation-type), in § 5.2
- [apply any pending playback rate](#apply-any-pending-playback-rate), in § 4.4.15.2
- [associated animation](#animation-effect-associated-animation), in § 5.4.2
- [associated effect](#animation-associated-effect), in § 4.4
- [associated effect end](#associated-effect-end), in § 4.4.12
- [associated with an animation](#associated-with-an-animation), in § 4.5.1
- [associated with a timeline](#associated-with-a-timeline), in § 4.5.1
- "auto"
  - [enum-value for CompositeOperationOrAuto](#dom-compositeoperationorauto-auto), in § 6.7
  - [enum-value for FillMode](#dom-fillmode-auto), in § 6.5.2
- [auto](#dom-compositeoperationorauto-auto), in § 6.7
- ["backwards"](#dom-fillmode-backwards), in § 6.5.2
- [BaseComputedKeyframe](#dictdef-basecomputedkeyframe), in § 6.6
- [BaseKeyframe](#dictdef-basekeyframe), in § 6.6.3
- [BasePropertyIndexedKeyframe](#dictdef-basepropertyindexedkeyframe), in § 6.6.3
- [before-active boundary time](#before-active-boundary-time), in § 4.5.5
- [before phase](#animation-effect-before-phase), in § 4.5.5
- ["both"](#dom-fillmode-both), in § 6.5.2
- [by computed value](#by-computed-value), in § 5.2
- [cancel()](#dom-animation-cancel), in § 6.4
- [cancel an animation](#cancel-an-animation), in § 4.4.14
- [cancel event](#cancel-event), in § 4.4.18.3
- [check the completion record](#check-the-completion-record), in § 6.6.3
- [combining shadow lists](#combining-shadow-lists), in § Unnumbered section
- [commit computed styles](#commit-computed-styles), in § 6.4
- [commitStyles()](#dom-animation-commitstyles), in § 6.4
- composite
  - [attribute for KeyframeEffect](#dom-keyframeeffect-composite), in § 6.6
  - [definition of](#composite), in § 5.4
  - [dict-member for BaseComputedKeyframe](#dom-basecomputedkeyframe-composite), in § 6.6
  - [dict-member for BaseKeyframe](#dom-basekeyframe-composite), in § 6.6.3
  - [dict-member for BasePropertyIndexedKeyframe](#dom-basepropertyindexedkeyframe-composite), in § 6.6.3
  - [dict-member for KeyframeEffectOptions](#dom-keyframeeffectoptions-composite), in § 6.6.4
- [composited value](#composited-value), in § 5.4.3
- [composite operation](#composite-operation), in § 5.4.4
- CompositeOperation
  - [(enum)](#enumdef-compositeoperation), in § 6.7
  - [definition of](#compositeoperation), in § 6.7
- [composite operation accumulate](#composite-operation-accumulate), in § 5.4.4
- [composite operation add](#composite-operation-add), in § 5.4.4
- [CompositeOperationOrAuto](#enumdef-compositeoperationorauto), in § 6.7
- [composite operation replace](#composite-operation-replace), in § 5.4.4
- [composite order](#animation-composite-order), in § 5.4.2
- [composition](#composite), in § 5.4
- [compute a property value](#compute-a-property-value), in § 5.3.2
- [ComputedEffectTiming](#dictdef-computedeffecttiming), in § 6.5.5
- [computed keyframe offset](#computed-keyframe-offset), in § 5.3.3
- [computed keyframes](#computed-keyframes), in § 5.3.3
- [computedOffset](#dom-basecomputedkeyframe-computedoffset), in § 6.6
- [compute missing keyframe offsets](#compute-missing-keyframe-offsets), in § 5.3.3
- [conforming scripted Web Animations user agent](#conforming-scripted-web-animations-user-agent), in § 9.2
- constructor()
  - [constructor for Animation](#dom-animation-animation), in § 6.4
  - [constructor for DocumentTimeline](#dom-documenttimeline-documenttimeline), in § 6.3
- [constructor(effect)](#dom-animation-animation), in § 6.4
- [constructor(effect, timeline)](#dom-animation-animation), in § 6.4
- [constructor(options)](#dom-documenttimeline-documenttimeline), in § 6.3
- [constructor(source)](#dom-keyframeeffect-keyframeeffect-source), in § 6.6
- [constructor(target, keyframes)](#dom-keyframeeffect-keyframeeffect), in § 6.6
- [constructor(target, keyframes, options)](#dom-keyframeeffect-keyframeeffect), in § 6.6
- [constructor(type)](#dom-animationplaybackevent-animationplaybackevent), in § 6.12
- [constructor(type, eventInitDict)](#dom-animationplaybackevent-animationplaybackevent), in § 6.12
- [current](#current), in § 4.5.5
- [current finished promise](#current-finished-promise), in § 4.4.11
- [current iteration](#current-iteration), in § 4.8.4
- [currentIteration](#dom-computedeffecttiming-currentiteration), in § 6.5.5
- [current ready promise](#current-ready-promise), in § 4.4.7
- current time
  - [dfn for animation](#animation-current-time), in § 4.4.3
  - [dfn for timeline](#timeline-current-time), in § 4.3
- currentTime
  - [attribute for Animation](#dom-animation-currenttime), in § 6.4
  - [attribute for AnimationPlaybackEvent](#dom-animationplaybackevent-currenttime), in § 6.12
  - [attribute for AnimationTimeline](#dom-animationtimeline-currenttime), in § 6.2
  - [dict-member for AnimationPlaybackEventInit](#dom-animationplaybackeventinit-currenttime), in § 6.12
- [default document timeline](#document-default-document-timeline), in § 4.3.2
- delay
  - [dict-member for EffectTiming](#dom-effecttiming-delay), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-delay), in § 6.5.1
- [directed progress](#directed-progress), in § 4.9.1
- direction
  - [dict-member for EffectTiming](#dom-effecttiming-direction), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-direction), in § 6.5.1
- [discrete](#discrete), in § 5.2
- [document for timing](#animation-document-for-timing), in § 4.4
- [document timeline](#document-timeline), in § 4.3.1
- [DocumentTimeline](#documenttimeline), in § 6.3
- [DocumentTimeline()](#dom-documenttimeline-documenttimeline), in § 6.3
- [DocumentTimeline(options)](#dom-documenttimeline-documenttimeline), in § 6.3
- [DocumentTimelineOptions](#dictdef-documenttimelineoptions), in § 6.3
- duration
  - [dict-member for EffectTiming](#dom-effecttiming-duration), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-duration), in § 6.5.1
- easing
  - [dict-member for BaseComputedKeyframe](#dom-basecomputedkeyframe-easing), in § 6.6
  - [dict-member for BaseKeyframe](#dom-basekeyframe-easing), in § 6.6.3
  - [dict-member for BasePropertyIndexedKeyframe](#dom-basepropertyindexedkeyframe-easing), in § 6.6.3
  - [dict-member for EffectTiming](#dom-effecttiming-easing), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-easing), in § 6.5.1
- [effect](#dom-animation-effect), in § 6.4
- [effective playback rate](#effective-playback-rate), in § 4.4.15.2
- [effect stack](#effect-stack), in § 5.4.2
- [effect target](#keyframe-effect-effect-target), in § 5.3
- [EffectTiming](#dictdef-effecttiming), in § 6.5.1
- [effect value](#effect-value), in § 5.1
- [end delay](#end-delay), in § 4.5.3
- endDelay
  - [dict-member for EffectTiming](#dom-effecttiming-enddelay), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-enddelay), in § 6.5.1
- [end time](#end-time), in § 4.5.3
- [endTime](#dom-computedeffecttiming-endtime), in § 6.5.5
- fill
  - [dict-member for EffectTiming](#dom-effecttiming-fill), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-fill), in § 6.5.1
- [fill mode](#fill-mode), in § 4.6
- [FillMode](#enumdef-fillmode), in § 6.5.2
- [finish()](#dom-animation-finish), in § 6.4
- [finish an animation](#finish-an-animation), in § 4.4.13
- ["finished"](#dom-animationplaystate-finished), in § 6.4.1
- finished
  - [attribute for Animation](#dom-animation-finished), in § 6.4
  - [dfn for play state](#play-state-finished), in § 4.4.17
- [finished play state](#play-state-finished), in § 4.4.17
- [finish event](#finish-event), in § 4.4.18.3
- [finish notification steps](#finish-notification-steps), in § 4.4.12
- ["forwards"](#dom-fillmode-forwards), in § 6.5.2
- getAnimations()
  - [method for Animatable](#dom-animatable-getanimations), in § 6.8
  - [method for DocumentOrShadowRoot](#dom-documentorshadowroot-getanimations), in § 6.10
- [getAnimations(options)](#dom-animatable-getanimations), in § 6.8
- [GetAnimationsOptions](#dictdef-getanimationsoptions), in § 6.8
- [getComputedTiming()](#dom-animationeffect-getcomputedtiming), in § 6.5
- [getKeyframes()](#dom-keyframeeffect-getkeyframes), in § 6.6
- [getTiming()](#dom-animationeffect-gettiming), in § 6.5
- [global animation list](#global-animation-list), in § 4.4
- [hold time](#animation-hold-time), in § 4.4
- id
  - [attribute for Animation](#dom-animation-id), in § 6.4
  - [dict-member for KeyframeAnimationOptions](#dom-keyframeanimationoptions-id), in § 6.8
- [IDL attribute name to animation property name](#idl-attribute-name-to-animation-property-name), in § 6.6.2
- ["idle"](#dom-animationplaystate-idle), in § 6.4.1
- [idle](#play-state-idle), in § 4.4.17
- [idle phase](#animation-effect-idle-phase), in § 4.5.5
- [idle play state](#play-state-idle), in § 4.4.17
- [inactive timeline](#inactive-timeline), in § 4.3
- [in effect](#in-effect), in § 4.5.5
- [in play](#in-play), in § 4.5.5
- [iteration count](#iteration-count), in § 4.7.2
- [iteration duration](#iteration-duration), in § 4.7.1
- [iteration interval](#iteration-interval), in § 4.7.1
- [iteration progress](#iteration-progress), in § 4.11
- iterations
  - [dict-member for EffectTiming](#dom-effecttiming-iterations), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-iterations), in § 6.5.1
- [iteration start](#iteration-start), in § 4.7.2
- iterationStart
  - [dict-member for EffectTiming](#dom-effecttiming-iterationstart), in § 6.5.1
  - [dict-member for OptionalEffectTiming](#dom-optionaleffecttiming-iterationstart), in § 6.5.1
- [keyframe](#keyframe), in § 5.3.1
- [KeyframeAnimationOptions](#dictdef-keyframeanimationoptions), in § 6.8
- [keyframe effect](#keyframe-effect), in § 5.3
- [KeyframeEffect](#keyframeeffect), in § 6.6
- [KeyframeEffectOptions](#dictdef-keyframeeffectoptions), in § 6.6.4
- [KeyframeEffect(source)](#dom-keyframeeffect-keyframeeffect-source), in § 6.6
- [KeyframeEffect(target, keyframes)](#dom-keyframeeffect-keyframeeffect), in § 6.6
- [KeyframeEffect(target, keyframes, options)](#dom-keyframeeffect-keyframeeffect), in § 6.6
- [keyframe offset](#keyframe-offset), in § 5.3.1
- [keyframe-specific composite operation](#keyframe-specific-composite-operation), in § 5.3.1
- [local time](#local-time), in § 4.5.4
- [localTime](#dom-computedeffecttiming-localtime), in § 6.5.5
- [loosely sorted by offset](#loosely-sorted-by-offset), in § 5.3.1
- [monotonically increasing](#monotonically-increasing-timeline), in § 4.3
- [monotonically increasing timeline](#monotonically-increasing-timeline), in § 4.3
- [neutral value for composition](#neutral-value-for-composition), in § 5.3.4
- ["none"](#dom-fillmode-none), in § 6.5.2
- ["normal"](#dom-playbackdirection-normal), in § 6.5.3
- [not animatable](#not-animatable), in § 5.2
- offset
  - [dict-member for BaseComputedKeyframe](#dom-basecomputedkeyframe-offset), in § 6.6
  - [dict-member for BaseKeyframe](#dom-basekeyframe-offset), in § 6.6.3
  - [dict-member for BasePropertyIndexedKeyframe](#dom-basepropertyindexedkeyframe-offset), in § 6.6.3
- [offsetk](#offsetk), in § 5.3.3
- [oncancel](#dom-animation-oncancel), in § 6.4
- [onfinish](#dom-animation-onfinish), in § 6.4
- [onremove](#dom-animation-onremove), in § 6.4
- [OptionalEffectTiming](#dictdef-optionaleffecttiming), in § 6.5.1
- [origin time](#origin-time), in § 4.3.1
- [originTime](#dom-documenttimelineoptions-origintime), in § 6.3
- [overall progress](#overall-progress), in § 4.8.3.2
- [pause()](#dom-animation-pause), in § 6.4
- [pause an animation](#pause-an-animation), in § 4.4.9
- ["paused"](#dom-animationplaystate-paused), in § 6.4.1
- [paused](#play-state-paused), in § 4.4.17
- [paused play state](#play-state-paused), in § 4.4.17
- [pending](#dom-animation-pending), in § 6.4
- [pending animation event queue](#pending-animation-event-queue), in § 4.4.18
- [pending pause task](#pending-pause-task), in § 4.4.9
- [pending playback rate](#pending-playback-rate), in § 4.4.15.2
- [pending play task](#pending-play-task), in § 4.4.8
- [persist()](#dom-animation-persist), in § 6.4
- ["persisted"](#dom-animationreplacestate-persisted), in § 6.4.2
- [persisted replace state](#persisted-replace-state), in § 5.5.1
- [play()](#dom-animation-play), in § 6.4
- [play an animation](#play-an-animation), in § 4.4.8
- [playback direction](#playback-direction), in § 4.9
- [PlaybackDirection](#enumdef-playbackdirection), in § 6.5.3
- [playback rate](#playback-rate), in § 4.4.15
- [playbackRate](#dom-animation-playbackrate), in § 6.4
- [play state](#animation-play-state), in § 4.4.17
- [playState](#dom-animation-playstate), in § 6.4
- [previous current time](#previous-current-time), in § 4.4.12
- [process a keyframe-like object](#process-a-keyframe-like-object), in § 6.6.3
- [process a keyframes argument](#process-a-keyframes-argument), in § 6.6.3
- [progress](#dom-computedeffecttiming-progress), in § 6.5.5
- pseudoElement
  - [attribute for KeyframeEffect](#dom-keyframeeffect-pseudoelement), in § 6.6
  - [dict-member for KeyframeEffectOptions](#dom-keyframeeffectoptions-pseudoelement), in § 6.6.4
- ready
  - [attribute for Animation](#dom-animation-ready), in § 6.4
  - [definition of](#ready), in § 4.4.6
- [relevant](#animation-relevant), in § 4.5.6
- [relevant animations](#relevant-animations), in § 4.5.6
- [relevant animations for a subtree](#relevant-animations-for-a-subtree), in § 4.5.6
- ["removed"](#dom-animationreplacestate-removed), in § 6.4.2
- [removed replace state](#removed-replace-state), in § 5.5.1
- [remove event](#remove-event), in § 4.4.18.3
- [remove replaced animations](#remove-replaced-animations), in § 5.5.2
- [repeatable list](#repeatable-list), in § 5.2
- ["replace"](#dom-compositeoperation-replace), in § 6.7
- [replace](#dom-compositeoperation-replace), in § 6.7
- [replaceable](#replaceable-animation), in § 5.5.2
- [replaceable animation](#replaceable-animation), in § 5.5.2
- [replace state](#replace-state), in § 5.5.1
- [replaceState](#dom-animation-replacestate), in § 6.4
- [reset an animation’s pending tasks](#animation-reset-an-animations-pending-tasks), in § 4.4.14
- ["reverse"](#dom-playbackdirection-reverse), in § 6.5.3
- [reverse()](#dom-animation-reverse), in § 6.4
- [reverse an animation](#reverse-an-animation), in § 4.4.16
- ["running"](#dom-animationplaystate-running), in § 6.4.1
- [running](#play-state-running), in § 4.4.17
- [running play state](#play-state-running), in § 4.4.17
- [scheduled event time](#scheduled-event-time), in § 4.4.18
- [seamlessly update the playback rate](#seamlessly-update-the-playback-rate), in § 4.4.15.2
- [setKeyframes(keyframes)](#dom-keyframeeffect-setkeyframes), in § 6.6
- [set the associated effect of an animation](#animation-set-the-associated-effect-of-an-animation), in § 4.4.2
- [set the current time](#animation-set-the-current-time), in § 4.4.4
- [set the playback rate](#set-the-playback-rate), in § 4.4.15.1
- [set the start time](#set-the-start-time), in § 4.4.5
- [set the timeline of an animation](#animation-set-the-timeline-of-an-animation), in § 4.4.1
- [shadow lists](#combining-shadow-lists), in § Unnumbered section
- [silently set the current time](#animation-silently-set-the-current-time), in § 4.4.4
- [simple iteration progress](#simple-iteration-progress), in § 4.8.3.3
- [start delay](#start-delay), in § 4.5.3
- [start time](#animation-start-time), in § 4.4
- [startTime](#dom-animation-starttime), in § 6.4
- [subtree](#dom-getanimationsoptions-subtree), in § 6.8
- [target](#dom-keyframeeffect-target), in § 6.6
- [target element](#effect-target-target-element), in § 5.3
- [target property](#target-property), in § 5.1
- [target pseudo-selector](#effect-target-target-pseudo-selector), in § 5.3
- timeline
  - [attribute for Animation](#dom-animation-timeline), in § 6.4
  - [attribute for Document](#dom-document-timeline), in § 6.9
  - [definition of](#timeline), in § 4.3
  - [dict-member for KeyframeAnimationOptions](#dom-keyframeanimationoptions-timeline), in § 6.8
- [timeline associated with a document](#timeline-associated-with-a-document), in § 4.3
- timelineTime
  - [attribute for AnimationPlaybackEvent](#dom-animationplaybackevent-timelinetime), in § 6.12
  - [dict-member for AnimationPlaybackEventInit](#dom-animationplaybackeventinit-timelinetime), in § 6.12
- [timeline time to origin-relative time](#timeline-time-to-origin-relative-time), in § 4.3
- [time value](#time-value), in § 4.2
- [transformed progress](#transformed-progress), in § 4.10.1
- [underlying value](#underlying-value), in § 5.4.3
- [unresolved](#unresolved), in § 4.2
- [update an animation’s finished state](#update-an-animations-finished-state), in § 4.4.12
- [update animations and send events](#update-animations-and-send-events), in § 4.3
- [updatePlaybackRate(playbackRate)](#dom-animation-updateplaybackrate), in § 6.4
- [update the timing properties of an animation effect](#update-the-timing-properties-of-an-animation-effect), in § 6.5.4
- [updateTiming()](#dom-animationeffect-updatetiming), in § 6.5
- [updateTiming(timing)](#dom-animationeffect-updatetiming), in § 6.5

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="term-for-propdef-animation-fill-mode"></a>animation-fill-mode
  - <a id="term-for-propdef-animation-name"></a>animation-name
  - <a id="term-for-events"></a>events from css animations
- \[CSS-ANIMATIONS-2\] defines the following terms:
  - <a id="term-for-cssanimation"></a>CSSAnimation
  - <a id="term-for-owning-element"></a>owning element (animation)
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="term-for-propdef-background-image"></a>background-image
  - <a id="term-for-propdef-background-origin"></a>background-origin
  - <a id="term-for-propdef-border-bottom-width"></a>border-bottom-width
  - <a id="term-for-propdef-border-color"></a>border-color
  - <a id="term-for-propdef-border-left-width"></a>border-left-width
  - <a id="term-for-propdef-border-right-width"></a>border-right-width
  - <a id="term-for-propdef-border-top"></a>border-top
  - <a id="term-for-propdef-border-top-color"></a>border-top-color
  - <a id="term-for-propdef-border-top-width"></a>border-top-width
  - <a id="term-for-propdef-border-width"></a>border-width
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="term-for-valdef-color-transparent"></a>transparent
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-propdef-visibility"></a>visibility
  - <a id="term-for-valdef-visibility-visible"></a>visible
- \[CSS-EASING-1\] defines the following terms:
  - <a id="term-for-typedef-easing-function"></a>\<easing-function\>
  - <a id="term-for-before-flag"></a>before flag
  - <a id="term-for-input-progress-value"></a>input progress value
  - <a id="term-for-linear-easing-function"></a>linear timing function
  - <a id="term-for-easing-function"></a>timing function
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="term-for-propdef-font-size"></a>font-size
  - <a id="term-for-propdef-font-weight"></a>font-weight
- \[CSS-PROPERTIES-VALUES-API-1\] defines the following terms:
  - <a id="term-for-dom-css-registerproperty"></a>registerProperty(definition)
  - <a id="term-for-syntax-definition"></a>syntax definition
  - <a id="term-for-universal-syntax-definition"></a>universal syntax definition
- \[CSS-SHADOW-PARTS-1\] defines the following terms:
  - <a id="term-for-selectordef-part"></a>::part()
- \[CSS-STYLE-ATTR\] defines the following terms:
  - <a id="term-for-style-attribute"></a>style attribute
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="term-for-propdef-text-shadow"></a>text-shadow
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="term-for-propdef-transform"></a>transform
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="term-for-after-change-style"></a>after-change style
  - <a id="term-for-before-change-style"></a>before-change style
  - <a id="term-for-transition-events"></a>events from css transitions
  - <a id="term-for-style-change-event"></a>style change event
  - <a id="term-for-transitionend"></a>transitionend
- \[CSS-TRANSITIONS-2\] defines the following terms:
  - <a id="term-for-owning-element①"></a>owning element (transition)
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-interpolation"></a>interpolate
  - <a id="term-for-interpolation①"></a>interpolation
  - <a id="term-for-not-additive"></a>not additive
  - <a id="term-for-accumulation"></a>value accumulation
  - <a id="term-for-addition"></a>value addition
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="term-for-typedef-custom-property-name"></a>\<custom-property-name\>
  - <a id="term-for-custom-property"></a>custom property
- \[CSS-WILL-CHANGE-1\] defines the following terms:
  - <a id="term-for-propdef-will-change"></a>will-change
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="term-for-logical-to-physical"></a>equivalent physical property
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="term-for-x43"></a>stacking context
- \[CSS22\] defines the following terms:
  - <a id="term-for-propdef-float"></a>float
- \[CSSOM\] defines the following terms:
  - <a id="term-for-cssomstring"></a>CSSOMString
  - <a id="term-for-css-declaration-block"></a>css declaration block
  - <a id="term-for-css-property-to-idl-attribute"></a>css property to idl attribute
  - <a id="term-for-idl-attribute-to-css-property"></a>idl attribute to css property
  - <a id="term-for-cssstyledeclaration-owner-node"></a>owner node
  - <a id="term-for-serialize-a-css-value"></a>serialize a css value
  - <a id="term-for-set-a-css-declaration"></a>set a css declaration
  - <a id="term-for-update-style-attribute-for"></a>update style attribute for
- \[DOM\] defines the following terms:
  - <a id="term-for-documentorshadowroot"></a>DocumentOrShadowRoot
  - <a id="term-for-element"></a>Element
  - <a id="term-for-event"></a>Event
  - <a id="term-for-dictdef-eventinit"></a>EventInit
  - <a id="term-for-eventtarget"></a>EventTarget
  - <a id="term-for-connected"></a>connected
  - <a id="term-for-constructing-events"></a>constructing events
  - <a id="term-for-concept-event-create"></a>create an event
  - <a id="term-for-concept-tree-descendant"></a>descendant
  - <a id="term-for-concept-event-dispatch"></a>dispatch
  - <a id="term-for-concept-document"></a>document
  - <a id="term-for-concept-element-attribute-has"></a>has an attribute
  - <a id="term-for-concept-tree-inclusive-descendant"></a>inclusive descendant
  - <a id="term-for-concept-node-document"></a>node document
  - <a id="term-for-concept-shadow-root"></a>shadow root
  - <a id="term-for-dom-event-type"></a>type
- \[ECMASCRIPT\] defines the following terms:
  - <a id="term-for-sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc"></a>\[\[defineownproperty\]\]
  - <a id="term-for-sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver"></a>\[\[get\]\]
  - <a id="term-for-sec-completion-record-specification-type"></a>completion record specification type
  - <a id="term-for-sec-enumerableownnames"></a>enumerableownnames
  - <a id="term-for-sec-getiterator"></a>getiterator
  - <a id="term-for-sec-getmethod"></a>getmethod
  - <a id="term-for-sec-iteratorstep"></a>iteratorstep
  - <a id="term-for-sec-iteratorvalue"></a>iteratorvalue
  - <a id="term-for-sec-promise-objects"></a>promise
  - <a id="term-for-sec-promise-objects①"></a>promise object
  - <a id="term-for-sec-ecmascript-data-types-and-values"></a>type
  - <a id="term-for-sec-well-known-symbols"></a>well known symbols
- \[HR-TIME\] defines the following terms:
  - <a id="term-for-dom-domhighrestimestamp"></a>DOMHighResTimeStamp
- \[HTML\] defines the following terms:
  - <a id="term-for-cereactions"></a>CEReactions
  - <a id="term-for-document"></a>Document
  - <a id="term-for-eventhandler"></a>EventHandler
  - <a id="term-for-window"></a>Window
  - <a id="term-for-active-document"></a>active document
  - <a id="term-for-an-entry-with-persisted-user-state"></a>an entry with persisted user state
  - <a id="term-for-animation-frames"></a>animation frame callbacks
  - <a id="term-for-being-rendered"></a>being rendered
  - <a id="term-for-current-global-object"></a>current global object
  - <a id="term-for-concept-document-window"></a>document associated with a window
  - <a id="term-for-dom-document-open"></a>document.open()
  - <a id="term-for-dom-manipulation-task-source"></a>dom manipulation task source
  - <a id="term-for-event-loop-processing-model"></a>event loop processing model
  - <a id="term-for-media-element"></a>media element
  - <a id="term-for-perform-a-microtask-checkpoint"></a>perform a microtask checkpoint
  - <a id="term-for-queue-a-microtask"></a>queue a microtask
  - <a id="term-for-queue-a-task"></a>queue a task
  - <a id="term-for-concept-relevant-realm"></a>relevant realm
  - <a id="term-for-run-the-animation-frame-callbacks"></a>run the animation frame callbacks
  - <a id="term-for-session-history-entry"></a>session history entry
  - <a id="term-for-concept-settings-object-time-origin"></a>time origin
- \[INFRA\] defines the following terms:
  - <a id="term-for-map-exists"></a>exist
  - <a id="term-for-list-extend"></a>extend
  - <a id="term-for-list-iterate"></a>iterate
  - <a id="term-for-list"></a>list
  - <a id="term-for-ordered-set"></a>ordered set
- \[MOTION-1\] defines the following terms:
  - <a id="term-for-propdef-offset"></a>offset
- \[SELECTORS-4\] defines the following terms:
  - <a id="term-for-typedef-pseudo-element-selector"></a>\<pseudo-element-selector\>
  - <a id="term-for-invalid-selector"></a>invalid selector
  - <a id="term-for-originating-element"></a>originating element
  - <a id="term-for-pseudo-element"></a>pseudo-element
- \[SVG2\] defines the following terms:
  - <a id="term-for-mime-registration"></a>svg mime type
- \[WEB-ANIMATIONS-2\] defines the following terms:
  - <a id="term-for-iteration-composite-operation"></a>iteration composite operation
  - <a id="term-for-iteration-composite-operation-accumulate"></a>iteration composite operation accumulate
- \[WEBIDL\] defines the following terms:
  - <a id="term-for-idl-DOMException"></a>DOMException
  - <a id="term-for-idl-DOMString"></a>DOMString
  - <a id="term-for-Exposed"></a>Exposed
  - <a id="term-for-invalidstateerror"></a>InvalidStateError
  - <a id="term-for-nomodificationallowederror"></a>NoModificationAllowedError
  - <a id="term-for-idl-promise"></a>Promise
  - <a id="term-for-syntaxerror"></a>SyntaxError
  - <a id="term-for-EnforceRange"></a>\[enforcerange\]
  - <a id="term-for-a-new-promise"></a>a new promise
  - <a id="term-for-idl-boolean"></a>boolean
  - <a id="term-for-dfn-convert-ecmascript-to-idl-value"></a>convert ecmascript to idl value
  - <a id="term-for-a-promise-resolved-with"></a>create a new resolved promise
  - <a id="term-for-DOMString-to-es"></a>domstring to es
  - <a id="term-for-idl-double"></a>double
  - <a id="term-for-es-to-dictionary"></a>es to dictionary
  - <a id="term-for-es-to-DOMString"></a>es to domstring
  - <a id="term-for-dfn-nullable-type"></a>nullable
  - <a id="term-for-idl-object"></a>object
  - <a id="term-for-reject"></a>reject a promise
  - <a id="term-for-resolve"></a>resolve a promise
  - <a id="term-for-idl-sequence"></a>sequence
  - <a id="term-for-dfn-throw"></a>throw
  - <a id="term-for-idl-undefined"></a>undefined
  - <a id="term-for-idl-unrestricted-double"></a>unrestricted double

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-animations-2"></a>\[CSS-ANIMATIONS-2\]  
David Baron; Brian Birtles. [CSS Animations Level 2](https://www.w3.org/TR/css-animations-2/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-2&#x2F;](https://www.w3.org/TR/css-animations-2/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 14 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-3"></a>\[CSS-CASCADE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-properties-values-api-1"></a>\[CSS-PROPERTIES-VALUES-API-1\]  
Tab Atkins Jr.; et al. [CSS Properties and Values API Level 1](https://www.w3.org/TR/css-properties-values-api-1/). 13 October 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-properties-values-api-1&#x2F;](https://www.w3.org/TR/css-properties-values-api-1/)

<a id="biblio-css-shadow-parts-1"></a>\[CSS-SHADOW-PARTS-1\]  
Tab Atkins Jr.; Fergal Daly. [CSS Shadow Parts](https://www.w3.org/TR/css-shadow-parts-1/). 15 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shadow-parts-1&#x2F;](https://www.w3.org/TR/css-shadow-parts-1/)

<a id="biblio-css-style-attr"></a>\[CSS-STYLE-ATTR\]  
Tantek Çelik; Elika Etemad. [CSS Style Attributes](https://www.w3.org/TR/css-style-attr/). 7 November 2013. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-style-attr&#x2F;](https://www.w3.org/TR/css-style-attr/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-transitions-2"></a>\[CSS-TRANSITIONS-2\]  
CSS Transitions Level 2 URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-transitions-2&#x2F;](https://drafts.csswg.org/css-transitions-2/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 6 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-will-change-1"></a>\[CSS-WILL-CHANGE-1\]  
Tab Atkins Jr.. [CSS Will Change Module Level 1](https://www.w3.org/TR/css-will-change-1/). 5 May 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-will-change-1&#x2F;](https://www.w3.org/TR/css-will-change-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-ecmascript"></a>\[ECMASCRIPT\]  
[ECMAScript Language Specification](https://tc39.es/ecma262/multipage/). URL: [https&#x3A;&#x2F;&#x2F;tc39&#x2E;es&#x2F;ecma262&#x2F;multipage&#x2F;](https://tc39.es/ecma262/multipage/)

<a id="biblio-hr-time"></a>\[HR-TIME\]  
Yoav Weiss. [High Resolution Time](https://www.w3.org/TR/hr-time-3/). 25 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;hr-time-3&#x2F;](https://www.w3.org/TR/hr-time-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-media-frags"></a>\[MEDIA-FRAGS\]  
Raphaël Troncy; et al. [Media Fragments URI 1.0 (basic)](https://www.w3.org/TR/media-frags/). 25 September 2012. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;media-frags&#x2F;](https://www.w3.org/TR/media-frags/)

<a id="biblio-motion-1"></a>\[MOTION-1\]  
Dirk Schulze; et al. [Motion Path Module Level 1](https://www.w3.org/TR/motion-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;motion-1&#x2F;](https://www.w3.org/TR/motion-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-web-animations-2"></a>\[WEB-ANIMATIONS-2\]  
Brian Birtles; Robert Flack. [Web Animations Level 2](https://www.w3.org/TR/web-animations-2/). 21 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-2&#x2F;](https://www.w3.org/TR/web-animations-2/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-smil-animation"></a>\[SMIL-ANIMATION\]  
Patrick Schmitz; Aaron Cohen. [SMIL Animation](https://www.w3.org/TR/smil-animation/). 4 September 2001. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;smil-animation&#x2F;](https://www.w3.org/TR/smil-animation/)

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface AnimationTimeline {
    readonly attribute double? currentTime;
};

dictionary DocumentTimelineOptions {
  DOMHighResTimeStamp originTime = 0;
};

[Exposed=Window]
interface DocumentTimeline : AnimationTimeline {
  constructor(optional DocumentTimelineOptions options = {});
};

[Exposed=Window]
interface Animation : EventTarget {
    constructor(optional AnimationEffect? effect = null,
                optional AnimationTimeline? timeline);
             attribute DOMString                id;
             attribute AnimationEffect?         effect;
             attribute AnimationTimeline?       timeline;
             attribute double?                  startTime;
             attribute double?                  currentTime;
             attribute double                   playbackRate;
    readonly attribute AnimationPlayState       playState;
    readonly attribute AnimationReplaceState    replaceState;
    readonly attribute boolean                  pending;
    readonly attribute Promise<Animation>       ready;
    readonly attribute Promise<Animation>       finished;
             attribute EventHandler             onfinish;
             attribute EventHandler             oncancel;
             attribute EventHandler             onremove;
    undefined cancel();
    undefined finish();
    undefined play();
    undefined pause();
    undefined updatePlaybackRate(double playbackRate);
    undefined reverse();
    undefined persist();
    [CEReactions]
    undefined commitStyles();
};

enum AnimationPlayState { "idle", "running", "paused", "finished" };

enum AnimationReplaceState { "active", "removed", "persisted" };

[Exposed=Window]
interface AnimationEffect {
    EffectTiming         getTiming();
    ComputedEffectTiming getComputedTiming();
    undefined            updateTiming(optional OptionalEffectTiming timing = {});
};

dictionary EffectTiming {
    double                             delay = 0;
    double                             endDelay = 0;
    FillMode                           fill = "auto";
    double                             iterationStart = 0.0;
    unrestricted double                iterations = 1.0;
    (unrestricted double or DOMString) duration = "auto";
    PlaybackDirection                  direction = "normal";
    DOMString                          easing = "linear";
};

dictionary OptionalEffectTiming {
    double                             delay;
    double                             endDelay;
    FillMode                           fill;
    double                             iterationStart;
    unrestricted double                iterations;
    (unrestricted double or DOMString) duration;
    PlaybackDirection                  direction;
    DOMString                          easing;
};

enum FillMode { "none", "forwards", "backwards", "both", "auto" };

enum PlaybackDirection { "normal", "reverse", "alternate", "alternate-reverse" };

dictionary ComputedEffectTiming : EffectTiming {
    unrestricted double  endTime;
    unrestricted double  activeDuration;
    double?              localTime;
    double?              progress;
    unrestricted double? currentIteration;
};

[Exposed=Window]
interface KeyframeEffect : AnimationEffect {
    constructor(Element? target,
                object? keyframes,
                optional (unrestricted double or KeyframeEffectOptions) options = {});
    constructor(KeyframeEffect source);
    attribute Element?           target;
    attribute CSSOMString?       pseudoElement;
    attribute CompositeOperation composite;
    sequence<object> getKeyframes();
    undefined        setKeyframes(object? keyframes);
};

dictionary BaseComputedKeyframe {
     double?                  offset = null;
     double                   computedOffset;
     DOMString                easing = "linear";
     CompositeOperationOrAuto composite = "auto";
};

dictionary BasePropertyIndexedKeyframe {
    (double? or sequence<double?>)                         offset = [];
    (DOMString or sequence<DOMString>)                     easing = [];
    (CompositeOperationOrAuto or sequence<CompositeOperationOrAuto>) composite = [];
};

dictionary BaseKeyframe {
    double?                  offset = null;
    DOMString                easing = "linear";
    CompositeOperationOrAuto composite = "auto";
};

dictionary KeyframeEffectOptions : EffectTiming {
    CompositeOperation composite = "replace";
    CSSOMString?       pseudoElement = null;
};

enum CompositeOperation { "replace", "add", "accumulate" };

enum CompositeOperationOrAuto { "replace", "add", "accumulate", "auto" };

interface mixin Animatable {
    Animation           animate(object? keyframes,
                                optional (unrestricted double or KeyframeAnimationOptions) options = {});
    sequence<Animation> getAnimations(optional GetAnimationsOptions options = {});
};

dictionary KeyframeAnimationOptions : KeyframeEffectOptions {
    DOMString id = "";
    AnimationTimeline? timeline;
};

dictionary GetAnimationsOptions {
    boolean subtree = false;
};

partial interface Document {
    readonly attribute DocumentTimeline timeline;
};

partial interface mixin DocumentOrShadowRoot {
    sequence<Animation> getAnimations();
};

Element includes Animatable;

[Exposed=Window]
interface AnimationPlaybackEvent : Event {
    constructor(DOMString type, optional AnimationPlaybackEventInit eventInitDict = {});
    readonly attribute double? currentTime;
    readonly attribute double? timelineTime;
};
dictionary AnimationPlaybackEventInit : EventInit {
    double? currentTime = null;
    double? timelineTime = null;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> There must be a better term than "origin time"— it’s too similar to "time origin". [\[Issue \#2079\]](https://github.com/w3c/csswg-drafts/issues/2079) [↵](#issue-9fa706cb)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> In the presence of certain timing functions, the input iteration progress to an animation effect is not limited to the range \[0, 1\]. Currently, however, keyframe offsets <em>are</em> limited to the range \[0, 1\] and property values are simply extrapolated for input iteration progress values outside this range.
>
> We have considered removing this restriction since some cases exist where it is useful to be able to specify non-linear changes in property values at iteration progress values outside the range \[0, 1\]. One example is an animation that interpolates from green to yellow but has an overshoot timing function that makes it temporarily interpolate "beyond" yellow to red before settling back to yellow.
>
> While this effect could be achieved by modification of the keyframes and timing function, this approach seems to break the model’s separation of timing concerns from animation effects.
>
> It is not clear how this effect should be achieved but we note that allowing keyframe offsets outside \[0, 1\] may make the currently specified behavior where keyframes at offset 0 and 1 are synthesized as necessary, inconsistent.
>
> See [section 4 (Keyframe offsets outside \[0, 1\]) of minuted discussion from Tokyo 2013 F2F](https://lists.w3.org/Archives/Public/public-fx/2013AprJun/0184.html).
>
> [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;2081&#x3E;](https://github.com/w3c/csswg-drafts/issues/2081)
>
> [↵](#issue-9e46aa87)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The definition of [being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered) [\[HTML\]](#biblio-html) with regards to [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) is still [under discussion](https://github.com/whatwg/html/issues/1837). For the purpose of this procedure, we assume that an element with display: contents that otherwise would have associated layout boxes (i.e. it is [connected](https://dom.spec.whatwg.org/#connected) and not part of a display: none subtree) <em>is</em> being rendered.
>
> [↵](#issue-8f3b970c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The `remove()` method can be used to remove an effect from either its parent group or animation. Should we keep it in level 1 and define it simply as removing the animation effect from its animation? [\[Issue \#2082\]](https://github.com/w3c/csswg-drafts/issues/2082) [↵](#issue-178c6cfa)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What should we do if the \[\[type\]\] is break, continue, or return? Can it be? [↵](#issue-47a68024)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is this at odds with those [time values](#time-value) being relative to `navigationStart` and with `requestAnimationFrame` using the same time as `document.timeline.currentTime`? [\[Issue \#2083\]](https://github.com/w3c/csswg-drafts/issues/2083) [↵](#issue-a128cfff)
