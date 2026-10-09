Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Fullscreen](https://www.w3.org/TR/2012/WD-fullscreen-20120703/).

Original copyright notice: Copyright © 2012 W3C ® ( MIT , ERCIM , Keio ), All Rights Reserved. W3C liability , trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Fullscreen

Source snapshot: https://www.w3.org/TR/2012/WD-fullscreen-20120703/

Snapshot SHA-256: c83bc890be04c170d09ba9fb23b022e3c43cd2da55a0e2f4a42669859fa968b4

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

# Fullscreen

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2012 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.org/), [Keio](http://www.keio.ac.jp/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

Fullscreen defines the fullscreen API for the web platform.

## <a id="status-of-this-document"></a><a id="sotd"></a>Status of this Document

<i>This section describes the status of this document at the time of its&#xA;publication. Other documents may supersede this document. A list of current W3C&#xA;publications and the latest revision of this technical report can be found in&#xA;the <a href="https://www.w3.org/TR/">W3C technical reports index</a> at&#xA;http&#58;//www&#46;w3&#46;org/TR/.</i>

This is the 03 July 2012 W3C First Public Working Draft of Fullscreen.

This document was jointly produced by the [Web Applications Working Group](https://www.w3.org/2008/webapps/) and the [CSS Working Group](https://www.w3.org/Style/CSS/members). The Web Applications Working Group is part of the [Rich Web Clients Activity](https://www.w3.org/2006/rwc/Activity) and the CSS Working Group is part of the [Style Activity](https://www.w3.org/Style/). Both of these Working Groups are part of the the W3C [Interaction Domain](https://www.w3.org/Interaction/).

Comments related to the API part of this document should be sent to the [public-webapps](mailto:public-webapps@w3.org?subject=%5Bfullscreen-api%5D%20) mail list ([archived](http://lists.w3.org/Archives/Public/public-webapps/)) with a subject header of `[fullscreen]` and comments related to the rendering part of this document should be sent to the [www-style](mailto:www-style@w3.org?subject=%5Bfullscreen-css%5D%20) mail list ([archived](http://lists.w3.org/Archives/Public/www-style/)) with a subject header of `[fullscreen]`. Alternatively, [file a bug](https://www.w3.org/Bugs/Public/enter_bug.cgi?product=WebAppsWG&component=Fullscreen).

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

​​​​​This specification was produced by the [WebApps Working Group](https://www.w3.org/2008/webapps/) (part of the [Graphics Activity](https://www.w3.org/Graphics/)) and the [CSS Working Group](https://www.w3.org/Style/CSS/members) (part of the [Style Activity](https://www.w3.org/Style/)).

This document was produced by groups operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures (WebApps)](https://www.w3.org/2004/01/pp-impl/42538/status) and a [public list of any patent disclosures (CSS)](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of each group; these pages also include instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

## <a id="table-of-contents"></a>Table of Contents

## <a id="conformance"></a>1 Conformance

All diagrams, examples, and notes in this specification are non-normative, as are all sections explicitly marked non-normative. Everything else in this specification is normative.

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in the normative parts of this specification are to be interpreted as described in RFC2119. For readability, these words do not appear in all uppercase letters in this specification. [\[RFC2119\]](#refsRFC2119)

## <a id="terminology"></a>2 Terminology

Most terminology used in this specification is from CSS, DOM, and HTML. [\[CSS\]](#refsCSS) [\[DOM\]](#refsDOM) [\[HTML5\]](#refsHTML5)

The term <a id="context-object"></a>context object means the object on which the method or attribute being discussed was called. When the [context object](#context-object) is unambiguous, the term can be omitted.

A [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context) <var title="">A</var> is called a <a id="descendant-browsing-context"></a>descendant browsing context of a [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context) <var title="">B</var> if and only if <var title="">B</var> is an [ancestor browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#ancestor-browsing-context) of <var title="">A</var>.

## <a id="model"></a>3 Model

The [task source](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#task-source) used by this specification is the [user interaction task source](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#user-interaction-task-source).

All [documents](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) have an associated <a id="fullscreen-enabled-flag"></a>fullscreen enabled flag and <a id="fullscreen-element-stack"></a>fullscreen element stack. Unless otherwise stated the [fullscreen enabled flag](#fullscreen-enabled-flag) is unset and the [fullscreen element stack](#fullscreen-element-stack) is empty.

> <strong data-conversion-semantic="note">Note</strong>
>
> HTML defines under what conditions the [fullscreen enabled flag](#fullscreen-enabled-flag) is set. [\[HTML\]](#refsHTML)

To <a id="concept-fullscreen-add"></a>add an <var title="">element</var> on a <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack), add it on top of <var title="">document</var>'s [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context)'s [top layer](#top-layer), and then add it on top of <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack).

To <a id="concept-fullscreen-remove"></a>remove an <var title="">element</var> from a <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack), remove it from <var title="">document</var>'s [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context)'s [top layer](#top-layer), and then remove it from <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack).

To <a id="concept-fullscreen-empty"></a>empty a <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack), remove all the [elements](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) in it from <var title="">document</var>'s [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context)'s [top layer](#top-layer), and then empty <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack).

------------------------------------------------------------------------

To <a id="fully-exit-fullscreen"></a>fully exit fullscreen act as if the [`exitFullscreen()`](#dom-document-exitfullscreen) method was invoked on the [top-level browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#top-level-browsing-context)'s [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) and subsequently [empty](#concept-fullscreen-empty) that [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document)'s [fullscreen element stack](#fullscreen-element-stack).

> <strong data-conversion-semantic="note">Note</strong>
>
> Since a `fullscreenchange` event will be dispatched regardless, emptying the [fullscreen element stack](#fullscreen-element-stack) per above is fine.

If an [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) at the top of a [fullscreen element stack](#fullscreen-element-stack) is removed from a [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document), [fully exit fullscreen](#fully-exit-fullscreen).

## <a id="api"></a>4 API

```text
partial interface Element {
  void requestFullscreen();
};

partial interface Document {
  readonly attribute boolean fullscreenEnabled;
  readonly attribute Element? fullscreenElement;

  void exitFullscreen();
};
```
<code><var title="">element</var>&#x20;.&#x20;<a href="#dom-element-requestfullscreen" title="dom-Element-requestFullscreen">requestFullscreen</a>()</code>  
Displays <var title="">element</var> fullscreen.

<code><var title="">document</var>&#x20;.&#x20;<a href="#dom-document-fullscreenenabled" title="dom-Document-fullscreenEnabled">fullscreenEnabled</a></code>  
Returns true if <var title="">document</var> has the ability to display [elements](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) fullscreen, or false otherwise.

<code><var title="">document</var>&#x20;.&#x20;<a href="#dom-document-fullscreenelement" title="dom-Document-fullscreenElement">fullscreenElement</a></code>  
Returns the [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) that is displayed fullscreen, or null if there is no such [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element).

<code><var title="">document</var>&#x20;.&#x20;<a href="#dom-document-exitfullscreen" title="dom-Document-exitFullscreen">exitFullscreen</a>()</code>  
Stops any [elements](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) within <var title="">document</var> from being displayed fullscreen.

The <a id="dom-element-requestfullscreen"></a>`requestFullscreen()` method must run these steps:

1.  If any of the following conditions are true, [queue a task](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#queue-a-task) to [fire an event](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-event-fire) named `fullscreenerror` with its `bubbles` attribute set to true on the [context object](#context-object)'s [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document), and then terminate these steps:

    - The [context object](#context-object) is not [in a document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#in-a-document).

    - The [context object](#context-object)'s [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document), or an [ancestor browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#ancestor-browsing-context)'s [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) does not have the [fullscreen enabled flag](#fullscreen-enabled-flag) set.

    - The [context object](#context-object)'s [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document) [fullscreen element stack](#fullscreen-element-stack) is not empty and its top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) is not an [ancestor](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-tree-ancestor) of the [context object](#context-object).

    - A [descendant browsing context](#descendant-browsing-context)'s [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) has a non-empty [fullscreen element stack](#fullscreen-element-stack).

    - This algorithm is not [allowed to show a pop-up](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#allowed-to-show-a-pop-up).

    - There is a previously-established user preference, security risk, or platform limitation.

2.  Return, and run the remaining steps asynchronously.

3.  Optionally, perform some animation.

4.  Let <var title="">doc</var> be <var title="">element</var>'s [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document).

5.  Let <var title="">docs</var> be all <var title="">doc</var>'s [ancestor browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#ancestor-browsing-context)'s [documents](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) (if any) and <var title="">doc</var>, in order of furthest away from <var title="">doc</var> to <var title="">doc</var>.

6.  For each <var title="">document</var> in <var title="">docs</var>, run these substeps:

    1.  Let <var title="">following document</var> be the [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) after <var title="">document</var> in <var title="">docs</var>, or null if there is no such document.

    2.  If <var title="">following document</var> is null, [queue a task](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#queue-a-task) to [add](#concept-fullscreen-add) [context object](#context-object) on <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack) and [fire an event](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-event-fire) named `fullscreenchange` with its `bubbles` attribute set to true on the <var title="">document</var>.

    3.  Otherwise, if <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack) is either empty or its top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) is not <var title="">following document</var>'s [browsing context container](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context-container), [queue a task](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#queue-a-task) to [add](#concept-fullscreen-add) <var title="">following document</var>'s [browsing context container](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context-container) on <var title="">document</var>'s [fullscreen element stack](#fullscreen-element-stack) and [fire an event](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-event-fire) named `fullscreenchange` with its `bubbles` attribute set to true on <var title="">document</var>.

    4.  Otherwise, do nothing for this <var title="">document</var>. It stays the same.

7.  Optionally, display a message indicating how the user can exit displaying the [context object](#context-object) fullscreen.

The <a id="dom-document-fullscreenenabled"></a>`fullscreenEnabled` attribute must return true if the [context object](#context-object) and all [ancestor browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#ancestor-browsing-context)'s [documents](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) have their [fullscreen enabled flag](#fullscreen-enabled-flag) set, or false otherwise.

The <a id="dom-document-fullscreenelement"></a>`fullscreenElement` attribute must return the top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) of the [context object](#context-object)'s [fullscreen element stack](#fullscreen-element-stack), or null otherwise.

The <a id="dom-document-exitfullscreen"></a>`exitFullscreen()` method must run these steps:

1.  Let <var title="">doc</var> be the [context object](#context-object).

2.  If <var title="">doc</var>'s [fullscreen element stack](#fullscreen-element-stack) is empty, terminate these steps.

3.  Return, and run the remaining steps asynchronously.

4.  Optionally, perform some animation.

5.  Let <var title="">descendants</var> be all the <var title="">doc</var>'s [descendant browsing context](#descendant-browsing-context)'s [documents](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) with a non-empty [fullscreen element stack](#fullscreen-element-stack) (if any), ordered so that the child of the <var title="">doc</var> is last and the [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document) furthest away from the <var title="">doc</var> is first.

6.  For each <var title="">descendant</var> in <var title="">descendants</var>, [queue a task](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#queue-a-task) to [empty](#concept-fullscreen-empty) <var title="">descendant</var>'s [fullscreen element stack](#fullscreen-element-stack) and [fire an event](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-event-fire) named `fullscreenchange` with its `bubbles` attribute set to true on <var title="">descendant</var>.

7.  While <var title="">doc</var> is not null, run these substeps:

    1.  [Remove](#concept-fullscreen-remove) the top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) of <var title="">doc</var>'s [fullscreen element stack](#fullscreen-element-stack).

        If <var title="">doc</var>'s [fullscreen element stack](#fullscreen-element-stack) is non-empty and the [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) now at the top is either not [in a document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#in-a-document) or its [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document) is not <var title="">doc</var>, run these substeps again.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > This is needed because a non-top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) in the [fullscreen element stack](#fullscreen-element-stack) could have been removed from <var title="">doc</var>.

    2.  [Queue a task](https://www.w3.org/TR/2012/WD-html5-20120329/webappapis.html#queue-a-task) to [fire an event](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-event-fire) named `fullscreenchange` with its `bubbles` attribute set to true on <var title="">doc</var>.

    3.  If <var title="">doc</var>'s [fullscreen element stack](#fullscreen-element-stack) is empty and <var title="">doc</var>'s [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context) has a [browsing context container](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context-container), set <var title="">doc</var> to that [browsing context container](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context-container)'s [node document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-node-document).

    4.  Otherwise, set <var title="">doc</var> to null.

## <a id="ui"></a>5 UI

User agents are encouraged to implement native media fullscreen controls in terms of [`requestFullscreen()`](#dom-element-requestfullscreen) and [`exitFullscreen()`](#dom-document-exitfullscreen).

If the end user instructs the user agent to end a fullscreen session initiated via [`requestFullscreen()`](#dom-element-requestfullscreen), [fully exit fullscreen](#fully-exit-fullscreen).

## <a id="rendering"></a>6 Rendering

This section is to be interpreted equivalently to the Rendering section of HTML. [\[HTML\]](#refsHTML)

### <a id="new-stacking-layer"></a>6.1 New stacking layer

This specification introduces a new stacking layer to the [Elaborate description of Stacking Contexts](https://www.w3.org/TR/CSS21/zindex.html) of CSS 2.1. It is called the <a id="top-layer"></a>top layer, comes after step 10 in the painting order, and is therefore rendered closest to the user within a viewport. Each [browsing context](https://www.w3.org/TR/2012/WD-html5-20120329/browsers.html#browsing-context) has one associated viewport and therefore also one [top layer](#top-layer). [\[CSS\]](#refsCSS)

> <strong data-conversion-semantic="note">Note</strong>
>
> The terminology used in this and following subsection attempts to match CSS 2.1 Appendix E.

The [top layer](#top-layer) consists of a stack of elements, rendered in the order they have been added to the stack. The last element added to the stack is rendered closest to the user.

> <strong data-conversion-semantic="note">Note</strong>
>
> The `z-index` property has no effect in the [top layer](#top-layer).

An element in this stack has the following attributes:

- It generates a new stacking context.

- Its parent stacking context is the root stacking context.

- Its containing block is the initial containing block.

- It is rendered as an atomic unit as if it were a sibling of the root element.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Ancestor elements with overflow, opacity, masks, etc. cannot affect it.

- It is not rendered if it, or an ancestor, has the `display` property set to `none`.

- If its specified `position` property is `static`, it computes to `absolute`.

- Its outline, if any, is to be rendered before step 10 in the painting order.

- Unless overridden by another specification, its static position for `left`, `right`, and `top` is zero.

### <a id="::backdrop-pseudo-element"></a>6.2 `::backdrop` pseudo-element

Each element in the [top layer](#top-layer)'s stack has a <a id="css-pe-backdrop"></a>`::backdrop` pseudo-element. This pseudo-element is a box rendered immediately below the element (and above the element below the element in the stack, if any), within the same [top layer](#top-layer).

> <strong data-conversion-semantic="note">Note</strong>
>
> The `::backdrop` pseudo-element can be used to create a backdrop that hides the underlying document for an element in the [top layer](#top-layer)'s stack. E.g. for the element that is displayed fullscreen as described by this specification.

It does not inherit from any element and is not inherited from. No restrictions are made on what properties apply to this pseudo-element either.

### <a id=":fullscreen-pseudo-class"></a>6.3 `:fullscreen` pseudo-class

The <a id="css-pc-fullscreen"></a>`:fullscreen` pseudo-class must match the top [element](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-element) of the [document](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html#concept-document)'s [fullscreen element stack](#fullscreen-element-stack) (if any).

### <a id="user-agent-level-style-sheet-defaults"></a>6.4 User-agent level style sheet defaults

```text
@namespace "http://www.w3.org/1999/xhtml";

*|*:fullscreen {
  position:fixed;
  top:0; right:0; bottom:0; left:0;
  margin:0;
  box-sizing:border-box;
  width:100%;
  height:100%;
  object-fit:contain;
}

iframe:fullscreen {
  border:none;
}

*|*:fullscreen::backdrop {
  position:fixed;
  top:0; right:0; bottom:0; left:0;
  background:black;
}
```
## <a id="security-and-privacy-considerations"></a>7 Security and Privacy Considerations

User agents should ensure, e.g. by means of an overlay, that the end user is aware something is displayed fullscreen. User agents should provide a means of exiting fullscreen that always works and advertise this to the user. This is to prevent a site from spoofing the end user by recreating the user agent or even operating system environment when fullscreen. See also the definition of [`requestFullscreen()`](#dom-element-requestfullscreen).

To prevent embedded content from going fullscreen only embedded content specifically allowed via the `allowfullscreen` attribute of the HTML `iframe` element will be able to go fullscreen. This prevents untrusted content from going fullscreen.

## <a id="references"></a>References

<a id="anolis-references"></a>

<a id="refsCSS"></a>\[CSS\]  
[CSS](https://www.w3.org/TR/CSS2/), Bert Bos, Tantek Çelik, Ian Hickson et al.. W3C.

<a id="refsDOM"></a>\[DOM\]  
[DOM4](http://dvcs.w3.org/hg/domcore/raw-file/tip/Overview.html), Anne van Kesteren, Aryeh Gregor and Ms2ger. W3C.

<a id="refsHTML"></a>\[HTML\]  
[HTML](https://www.whatwg.org/C), Ian Hickson. WHATWG.

<a id="refsHTML5"></a>\[HTML5\]  
[HTML5](http://dev.w3.org/html5/spec/), Ian Hickson. W3C.

<a id="refsRFC2119"></a>\[RFC2119\]  
[Key words for use in RFCs to Indicate Requirement Levels](http://tools.ietf.org/html/rfc2119), Scott Bradner. IETF.

## <a id="acknowledgments"></a>Acknowledgments

Many thanks to Robert O'Callahan for designing the initial model and being awesome.

Thanks to Chris Pearce, Darin Fisher, Edward O'Connor, <i title="">fantasai</i>, Glenn Maynard, Ian Hickson, João Eiras, Josh Soref, Øyvind Stenhaug, Rafał Chłodnicki, Rune Lillesveen, Sigbjørn Vik, Simon Pieters, and Tab Atkins for also being awesome.
