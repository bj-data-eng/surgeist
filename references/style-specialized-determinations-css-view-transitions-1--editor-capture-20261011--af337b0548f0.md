Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS View Transitions Module Level 1](https://drafts.csswg.org/css-view-transitions-1/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `ce6513a48a9b35e17affe835cc778ae560567734cd0530dc5eecada35fd4e7c1`; 569419 bytes. Bounded HTML SHA-256 `af337b0548f015067ee4409e17356890b2641e058d039e2d10bc5453b68caf38`; 108542 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `captured-elements`, `setup-view-transition-algorithm`, `skip-the-view-transition-algorithm`, `handle-transition-frame-algorithm`, `clear-view-transition-algorithm`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS View Transitions Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 16 February 2025

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-view-transitions-1/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-view-transitions-1/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-view-transitions>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-view-transitions-1)

<strong>Editors:</strong>

[Tab Atkins-Bittner](http://xanthir.com/contact/) (Google)

Jake Archibald (Google)

Khushal Sagar (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-view-transitions-1/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-view-transitions” in the title, like this: “\[css-view-transitions\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-view-transitions%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

#### <a id="captured-elements"></a>7.1.3. <a id="ref-for-captured-element①"></a>[Captured elements](#captured-element)[](#captured-elements)

A <a id="captured-element"></a><strong>captured element</strong> is a <a id="ref-for-struct"></a>[struct](https://infra.spec.whatwg.org/#struct) with the following:

<strong><a id="captured-element-old-image"></a><strong>old image</strong></strong>

an 2D bitmap or null. Initially null.

<strong><a id="captured-element-old-width"></a><strong>old width</strong></strong>

<strong><a id="captured-element-old-height"></a><strong>old height</strong></strong>

an <code><a id="ref-for-idl-unrestricted-double"></a>[unrestricted double](https://webidl.spec.whatwg.org/#idl-unrestricted-double)</code>, initially zero.

<strong><a id="captured-element-old-transform"></a><strong>old transform</strong></strong>

a <a id="ref-for-typedef-transform-function"></a>[\<transform-function\>](https://drafts.csswg.org/css-transforms-2/#typedef-transform-function), initially the <a id="ref-for-identity-transform-function"></a>[identity transform function](https://drafts.csswg.org/css-transforms-1/#identity-transform-function).

<strong><a id="captured-element-old-writing-mode"></a><strong>old writing-mode</strong></strong>

Null or a <a id="ref-for-propdef-writing-mode"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode), initially null.

<strong><a id="captured-element-old-direction"></a><strong>old direction</strong></strong>

Null or a <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction), initially null.

<strong><a id="captured-element-old-text-orientation"></a><strong>old text-orientation</strong></strong>

Null or a <a id="ref-for-propdef-text-orientation"></a>[text-orientation](https://drafts.csswg.org/css-writing-modes-4/#propdef-text-orientation), initially null.

<strong><a id="captured-element-old-mix-blend-mode"></a><strong>old mix-blend-mode</strong></strong>

Null or a <a id="ref-for-propdef-mix-blend-mode①"></a>[mix-blend-mode](https://drafts.csswg.org/compositing-2/#propdef-mix-blend-mode), initially null.

<strong><a id="captured-element-old-backdrop-filter"></a><strong>old backdrop-filter</strong></strong>

Null or a <a id="ref-for-propdef-backdrop-filter"></a>[backdrop-filter](https://drafts.csswg.org/filter-effects-2/#propdef-backdrop-filter), initially null.

<strong><a id="captured-element-old-color-scheme"></a><strong>old color-scheme</strong></strong>

Null or a <a id="ref-for-propdef-color-scheme"></a>[color-scheme](https://drafts.csswg.org/css-color-adjust-1/#propdef-color-scheme), initially null.

<strong><a id="captured-element-new-element"></a><strong>new element</strong></strong>

an <a id="ref-for-element⑧"></a>[element](https://drafts.csswg.org/css2/#element) or null. Initially null.

In addition, a <a id="ref-for-captured-element②"></a>[captured element](#captured-element) has the following <a id="captured-element-style-definitions"></a><strong>style definitions</strong>:

<strong><a id="captured-element-group-keyframes"></a><strong>group keyframes</strong></strong>

A <code><a id="ref-for-csskeyframesrule"></a>[CSSKeyframesRule](https://drafts.csswg.org/css-animations-1/#csskeyframesrule)</code> or null. Initially null.

<strong><a id="captured-element-group-animation-name-rule"></a><strong>group animation name rule</strong></strong>

A <code><a id="ref-for-cssstylerule"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> or null. Initially null.

<strong><a id="captured-element-group-styles-rule"></a><strong>group styles rule</strong></strong>

A <code><a id="ref-for-cssstylerule①"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> or null. Initially null.

<strong><a id="captured-element-image-pair-isolation-rule"></a><strong>image pair isolation rule</strong></strong>

A <code><a id="ref-for-cssstylerule②"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> or null. Initially null.

<strong><a id="captured-element-image-animation-name-rule"></a><strong>image animation name rule</strong></strong>

A <code><a id="ref-for-cssstylerule③"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> or null. Initially null.

Note: These are used to update, and later remove styles from a <a id="ref-for-concept-document①"></a>[document](https://dom.spec.whatwg.org/#concept-document)’s <a id="ref-for-document-dynamic-view-transition-style-sheet④"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet).

### <a id="setup-view-transition-algorithm"></a>7.3. <a id="ref-for-setup-view-transition②"></a>[Setup view transition](#setup-view-transition)[](#setup-view-transition-algorithm)

To <a id="setup-view-transition"></a><strong>setup view transition</strong> for a <code><a id="ref-for-viewtransition①③"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>, perform the following steps:

Note: This algorithm captures the current state of the document, calls the transition’s <code><a id="ref-for-callbackdef-viewtransitionupdatecallback①⑤"></a>[ViewTransitionUpdateCallback](https://drafts.csswg.org/css-view-transitions-1/#callbackdef-viewtransitionupdatecallback)</code>, then captures the new state of the document.

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global①"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window①"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-flush-the-update-callback-queue"></a>[Flush the update callback queue](https://drafts.csswg.org/css-view-transitions-1/#flush-the-update-callback-queue).

    Note: this ensures that any changes to the DOM scheduled by other skipped transitions are done before the old state for this transition is captured.

3.  <a id="ref-for-capture-the-old-state"></a>[Capture the old state](#capture-the-old-state) for <var>transition</var>.

    If failure is returned, then <a id="ref-for-skip-the-view-transition④"></a>[skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a id="ref-for-invalidstateerror①"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm⑤"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

4.  Set <var>document</var>’s <a id="ref-for-document-rendering-suppression-for-view-transitions①"></a>[rendering suppression for view transitions](https://drafts.csswg.org/css-view-transitions-1/#document-rendering-suppression-for-view-transitions) to true.

5.  <a id="ref-for-queue-a-global-task"></a>[Queue a global task](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-global-task) on the <a id="ref-for-dom-manipulation-task-source"></a>[DOM manipulation task source](https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source), given <var>transition</var>’s <a id="ref-for-concept-relevant-global②"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global), to perform the following steps:

    Note: A task is queued here because the texture read back in <a id="ref-for-capture-the-image③"></a>[capturing the image](https://drafts.csswg.org/css-view-transitions-1/#capture-the-image) may be async, although the render steps in the HTML spec act as if it’s synchronous.

    1.  If <var>transition</var>’s <a id="ref-for-viewtransition-phase④"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) is "<code>done</code>", then abort these steps.

        Note: This happens if <var>transition</var> was <a id="ref-for-skip-the-view-transition⑤"></a>[skipped](#skip-the-view-transition) before this point.

    2.  <a id="ref-for-schedule-the-update-callback"></a>[schedule the update callback](https://drafts.csswg.org/css-view-transitions-1/#schedule-the-update-callback) for <var>transition</var>.

    3.  <a id="ref-for-flush-the-update-callback-queue①"></a>[Flush the update callback queue](https://drafts.csswg.org/css-view-transitions-1/#flush-the-update-callback-queue).

To <a id="activate-view-transition"></a><strong>activate view transition</strong> for a <code><a id="ref-for-viewtransition①④"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>, perform the following steps:

1.  If <var>transition</var>’s <a id="ref-for-viewtransition-phase⑤"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) is "<code>done</code>", then return.

    Note: This happens if <var>transition</var> was <a id="ref-for-skip-the-view-transition⑥"></a>[skipped](#skip-the-view-transition) before this point.

2.  Set <var>transition</var>’s <a id="ref-for-concept-relevant-global③"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window②"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window)’s <a id="ref-for-document-rendering-suppression-for-view-transitions②"></a>[rendering suppression for view transitions](https://drafts.csswg.org/css-view-transitions-1/#document-rendering-suppression-for-view-transitions) to false.

3.  If <var>transition</var>’s <a id="ref-for-viewtransition-initial-snapshot-containing-block-size"></a>[initial snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-initial-snapshot-containing-block-size) is not equal to the <a id="ref-for-snapshot-containing-block-size①"></a>[snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block-size), then <a id="ref-for-skip-the-view-transition⑦"></a>[skip](#skip-the-view-transition) <var>transition</var> with an "<code><a id="ref-for-invalidstateerror②"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm⑥"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

4.  <a id="ref-for-capture-the-new-state"></a>[Capture the new state](#capture-the-new-state) for <var>transition</var>.

    If failure is returned, then <a id="ref-for-skip-the-view-transition⑧"></a>[skip](#skip-the-view-transition) <var>transition</var> with an "<code><a id="ref-for-invalidstateerror③"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm⑦"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

5.  <a id="ref-for-list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>capturedElement</var> of <var>transition</var>’s <a id="ref-for-viewtransition-named-elements"></a>[named elements](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-named-elements)' <a id="ref-for-map-getting-the-values"></a>[values](https://infra.spec.whatwg.org/#map-getting-the-values):

    1.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element①"></a>[new element](#captured-element-new-element) is not null, then set <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element②"></a>new element’s <a id="ref-for-captured-in-a-view-transition③"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) to true.

6.  <a id="ref-for-setup-transition-pseudo-elements④"></a>[Setup transition pseudo-elements](#setup-transition-pseudo-elements) for <var>transition</var>.

7.  <a id="ref-for-update-pseudo-element-styles②"></a>[Update pseudo-element styles](https://drafts.csswg.org/css-view-transitions-1/#update-pseudo-element-styles) for <var>transition</var>.

    If failure is returned, then <a id="ref-for-skip-the-view-transition⑨"></a>[skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a id="ref-for-invalidstateerror④"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm⑧"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

    Note: The above steps will require running document lifecycle phases, to compute information calculated during style/layout.

8.  Set <var>transition</var>’s <a id="ref-for-viewtransition-phase⑥"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) to "<code>animating</code>".

9.  <a id="ref-for-resolve"></a>[Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s <a id="ref-for-viewtransition-ready-promise②"></a>[ready promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-ready-promise).

#### <a id="capture-old-state-algorithm"></a>7.3.1. <a id="ref-for-capture-the-old-state①"></a>[Capture the old state](#capture-the-old-state)[](#capture-old-state-algorithm)

To <a id="capture-the-old-state"></a><strong>capture the old state</strong> for <code><a id="ref-for-viewtransition①⑤"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>:

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global④"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window③"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  Let <var>namedElements</var> be <var>transition</var>’s <a id="ref-for-viewtransition-named-elements①"></a>[named elements](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-named-elements).

3.  Let <var>usedTransitionNames</var> be a new <a id="ref-for-ordered-set"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of strings.

4.  Let <var>captureElements</var> be a new <a id="ref-for-list①"></a>[list](https://infra.spec.whatwg.org/#list) of elements.

5.  If the <a id="ref-for-snapshot-containing-block-size②"></a>[snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block-size) exceeds an <a id="ref-for-implementation-defined"></a>[implementation-defined](https://infra.spec.whatwg.org/#implementation-defined) maximum, then return failure.

6.  Set <var>transition</var>’s <a id="ref-for-viewtransition-initial-snapshot-containing-block-size①"></a>[initial snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-initial-snapshot-containing-block-size) to the <a id="ref-for-snapshot-containing-block-size③"></a>[snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block-size).

7.  <a id="ref-for-list-iterate①"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of every <a id="ref-for-element⑨"></a>[element](https://drafts.csswg.org/css2/#element) that is <a id="ref-for-connected"></a>[connected](https://dom.spec.whatwg.org/#connected), and has a <a id="ref-for-concept-node-document①"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) equal to <var>document</var>, in [paint order](https://drafts.csswg.org/css2/#painting-order):

    We iterate in paint order to ensure that this order is cached in
    <strong>Note:</strong>

    <var>namedElements</var>. This defines the DOM order for ::view-transition-group <a id="ref-for-pseudo-element①⑨"></a>[pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element), such that the element at the bottom of the paint stack generates the first pseudo child of <a id="ref-for-selectordef-view-transition⑨"></a>[::view-transition](https://drafts.csswg.org/css-view-transitions-1/#selectordef-view-transition).
    1.  If any <a id="ref-for-flat-tree"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) ancestor of this <var>element</var> <a id="ref-for-skips-its-contents①"></a>[skips its contents](https://drafts.csswg.org/css-contain-2/#skips-its-contents), then <a id="ref-for-iteration-continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  If <var>element</var> has more than one <a id="ref-for-box-fragment"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment), then <a id="ref-for-iteration-continue①"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

        Note: We might want to enable transitions for fragmented elements in future versions. See [\#8900](https://github.com/w3c/csswg-drafts/issues/8900).

        Note: <a id="ref-for-box-fragment①"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment) here does not refer to fragmentation of [inline boxes](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) across [line boxes](https://www.w3.org/TR/CSS2/visuren.html#line-box). Such inlines can participate in a transition.

    3.  Let <var>transitionName</var> be the <var>element</var>’s <a id="ref-for-document-scoped-view-transition-name"></a>[document-scoped view transition name](https://drafts.csswg.org/css-view-transitions-1/#document-scoped-view-transition-name).

    4.  If <var>transitionName</var> is <a id="ref-for-valdef-view-transition-name-none②"></a>[none](https://drafts.csswg.org/css-view-transitions-1/#valdef-view-transition-name-none), or <var>element</var> is <a id="ref-for-element-not-rendered①"></a>[not rendered](https://drafts.csswg.org/css-images-4/#element-not-rendered), then <a id="ref-for-iteration-continue②"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    5.  If <var>usedTransitionNames</var> <a id="ref-for-list-contain"></a>[contains](https://infra.spec.whatwg.org/#list-contain) <var>transitionName</var>, then:

        1.  <a id="ref-for-list-iterate②"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> in <var>captureElements</var>:

            1.  Set <var>element</var>’s <a id="ref-for-captured-in-a-view-transition④"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) to false.

        2.  return failure.

    6.  <a id="ref-for-set-append"></a>[Append](https://infra.spec.whatwg.org/#set-append) <var>transitionName</var> to <var>usedTransitionNames</var>.

    7.  Set <var>element</var>’s <a id="ref-for-captured-in-a-view-transition⑤"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) to true.

    8.  <a id="ref-for-list-append"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>element</var> to <var>captureElements</var>.

    The algorithm continues in a separate loop to ensure that
    <strong>Note:</strong>

    <a id="ref-for-captured-in-a-view-transition⑥"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) is set on all elements participating in this capture before it is read by future steps in the algorithm.

8.  <a id="ref-for-list-iterate③"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> in <var>captureElements</var>:

    1.  Let <var>capture</var> be a new <a id="ref-for-captured-element③"></a>[captured element](#captured-element) struct.

    2.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-image①"></a>[old image](#captured-element-old-image) to the result of <a id="ref-for-capture-the-image④"></a>[capturing the image](https://drafts.csswg.org/css-view-transitions-1/#capture-the-image) of <var>element</var>.

    3.  Let <var>originalRect</var> be <a id="ref-for-snapshot-containing-block①⓪"></a>[snapshot containing block](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block) if <var>element</var> is the <a id="ref-for-document-element⑧"></a>[document element](https://dom.spec.whatwg.org/#document-element), otherwise, the <var>element</var>’s <a id="ref-for-border-box②"></a>[border box](https://drafts.csswg.org/css-box-4/#border-box).

    4.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-width"></a>[old width](#captured-element-old-width) to <var>originalRect</var>’s <code><a id="ref-for-dom-domrect-width"></a>[width](https://drafts.csswg.org/geometry-1/#dom-domrect-width)</code>.

    5.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-height"></a>[old height](#captured-element-old-height) to <var>originalRect</var>’s <code><a id="ref-for-dom-domrect-height"></a>[height](https://drafts.csswg.org/geometry-1/#dom-domrect-height)</code>.

    6.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-transform"></a>[old transform](#captured-element-old-transform) to a <a id="ref-for-typedef-transform-function①"></a>[\<transform-function\>](https://drafts.csswg.org/css-transforms-2/#typedef-transform-function) that would map <var>element</var>’s <a id="ref-for-border-box③"></a>[border box](https://drafts.csswg.org/css-box-4/#border-box) from the <a id="ref-for-snapshot-containing-block-origin"></a>[snapshot containing block origin](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block-origin) to its current visual position.

    7.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-writing-mode"></a>[old writing-mode](#captured-element-old-writing-mode) to the <a id="ref-for-computed-value②"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-writing-mode①"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode) on <var>element</var>.

    8.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-direction"></a>[old direction](#captured-element-old-direction) to the <a id="ref-for-computed-value③"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-direction①"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) on <var>element</var>.

    9.  Set <var>capture</var>’s <a id="ref-for-captured-element-old-text-orientation"></a>[old text-orientation](#captured-element-old-text-orientation) to the <a id="ref-for-computed-value④"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-text-orientation①"></a>[text-orientation](https://drafts.csswg.org/css-writing-modes-4/#propdef-text-orientation) on <var>element</var>.

    10. Set <var>capture</var>’s <a id="ref-for-captured-element-old-mix-blend-mode"></a>[old mix-blend-mode](#captured-element-old-mix-blend-mode) to the <a id="ref-for-computed-value⑤"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-mix-blend-mode②"></a>[mix-blend-mode](https://drafts.csswg.org/compositing-2/#propdef-mix-blend-mode) on <var>element</var>.

    11. Set <var>capture</var>’s <a id="ref-for-captured-element-old-backdrop-filter"></a>[old backdrop-filter](#captured-element-old-backdrop-filter) to the <a id="ref-for-computed-value⑥"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-backdrop-filter①"></a>[backdrop-filter](https://drafts.csswg.org/filter-effects-2/#propdef-backdrop-filter) on <var>element</var>.

    12. Set <var>capture</var>’s <a id="ref-for-captured-element-old-color-scheme"></a>[old color-scheme](#captured-element-old-color-scheme) to the <a id="ref-for-computed-value⑦"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-color-scheme①"></a>[color-scheme](https://drafts.csswg.org/css-color-adjust-1/#propdef-color-scheme) on <var>element</var>.

    13. Let <var>transitionName</var> be the <a id="ref-for-computed-value⑧"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-view-transition-name①②"></a>[view-transition-name](https://drafts.csswg.org/css-view-transitions-1/#propdef-view-transition-name) for <var>element</var>.

    14. Set <var>namedElements</var>\[<var>transitionName</var>\] to <var>capture</var>.

9.  <a id="ref-for-list-iterate④"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> in <var>captureElements</var>:

    1.  Set <var>element</var>’s <a id="ref-for-captured-in-a-view-transition⑦"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) to false.

#### <a id="capture-new-state-algorithm"></a>7.3.2. <a id="ref-for-capture-the-new-state①"></a>[Capture the new state](#capture-the-new-state)[](#capture-new-state-algorithm)

To <a id="capture-the-new-state"></a><strong>capture the new state</strong> for <code><a id="ref-for-viewtransition①⑥"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>:

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global⑤"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window④"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  Let <var>namedElements</var> be <var>transition</var>’s <a id="ref-for-viewtransition-named-elements②"></a>[named elements](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-named-elements).

3.  Let <var>usedTransitionNames</var> be a new <a id="ref-for-ordered-set①"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of strings.

4.  <a id="ref-for-list-iterate⑤"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of every <a id="ref-for-element①⓪"></a>[element](https://drafts.csswg.org/css2/#element) that is <a id="ref-for-connected①"></a>[connected](https://dom.spec.whatwg.org/#connected), and has a <a id="ref-for-concept-node-document②"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) equal to <var>document</var>, in [paint order](https://drafts.csswg.org/css2/#painting-order):

    1.  If any <a id="ref-for-flat-tree①"></a>[flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) ancestor of this <var>element</var> <a id="ref-for-skips-its-contents②"></a>[skips its contents](https://drafts.csswg.org/css-contain-2/#skips-its-contents), then <a id="ref-for-iteration-continue③"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  Let <var>transitionName</var> be <var>element</var>’s <a id="ref-for-document-scoped-view-transition-name①"></a>[document-scoped view transition name](https://drafts.csswg.org/css-view-transitions-1/#document-scoped-view-transition-name).

    3.  If <var>transitionName</var> is <a id="ref-for-valdef-view-transition-name-none③"></a>[none](https://drafts.csswg.org/css-view-transitions-1/#valdef-view-transition-name-none), or <var>element</var> is <a id="ref-for-element-not-rendered②"></a>[not rendered](https://drafts.csswg.org/css-images-4/#element-not-rendered), then <a id="ref-for-iteration-continue④"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  If <var>element</var> has more than one <a id="ref-for-box-fragment②"></a>[box fragment](https://drafts.csswg.org/css-break-4/#box-fragment), then <a id="ref-for-iteration-continue⑤"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    5.  If <var>usedTransitionNames</var> <a id="ref-for-list-contain①"></a>[contains](https://infra.spec.whatwg.org/#list-contain) <var>transitionName</var>, then return failure.

    6.  <a id="ref-for-set-append①"></a>[Append](https://infra.spec.whatwg.org/#set-append) <var>transitionName</var> to <var>usedTransitionNames</var>.

    7.  If <var>namedElements</var>\[<var>transitionName</var>\] does not <a id="ref-for-map-exists"></a>[exist](https://infra.spec.whatwg.org/#map-exists), then set <var>namedElements</var>\[<var>transitionName</var>\] to a new <a id="ref-for-captured-element④"></a>[captured element](#captured-element) struct.

        Note: We intentionally add this struct to the end of this ordered map. This implies than names which only exist in the new DOM (entry animations) will be painted on top of names only in the old DOM (exit animations) and names in both DOMs (paired animations). This might not be the right layering for all cases. See [issue 8941](https://github.com/w3c/csswg-drafts/issues/8941).

    8.  Set <var>namedElements</var>\[<var>transitionName</var>\]'s <a id="ref-for-captured-element-new-element③"></a>[new element](#captured-element-new-element) to <var>element</var>.

#### <a id="setup-transition-pseudo-elements-algorithm"></a>7.3.3. <a id="ref-for-setup-transition-pseudo-elements⑤"></a>[Setup transition pseudo-elements](#setup-transition-pseudo-elements)[](#setup-transition-pseudo-elements-algorithm)

To <a id="setup-transition-pseudo-elements"></a><strong>setup transition pseudo-elements</strong> for a <code><a id="ref-for-viewtransition①⑦"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>:

Note: This algorithm constructs the <a id="ref-for-pseudo-element-tree③"></a>[pseudo-element tree](https://drafts.csswg.org/css-view-transitions-1/#pseudo-element-tree) for the transition, and generates initial styles. The structure of the pseudo-tree is covered at a higher level in [§ 3.2 View Transition Pseudo-elements](https://drafts.csswg.org/css-view-transitions-1/#view-transition-pseudos).

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global⑥"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window⑤"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  Set <var>document</var>’s <a id="ref-for-document-show-view-transition-tree"></a>[show view transition tree](https://drafts.csswg.org/css-view-transitions-1/#document-show-view-transition-tree) to true.

3.  <a id="ref-for-map-iterate"></a>[For each](https://infra.spec.whatwg.org/#map-iterate) <var>transitionName</var> → <var>capturedElement</var> of <var>transition</var>’s <a id="ref-for-viewtransition-named-elements③"></a>[named elements](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-named-elements):

    1.  Let <var>group</var> be a new <a id="ref-for-selectordef-view-transition-group①⑦"></a>[::view-transition-group()](https://drafts.csswg.org/css-view-transitions-1/#selectordef-view-transition-group), with its <a id="ref-for-view-transition-name①②"></a>[view transition name](https://drafts.csswg.org/css-view-transitions-1/#view-transition-name) set to <var>transitionName</var>.

    2.  Append <var>group</var> to <var>transition</var>’s <a id="ref-for-viewtransition-transition-root-pseudo-element④"></a>[transition root pseudo-element](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-transition-root-pseudo-element).

    3.  Let <var>imagePair</var> be a new <a id="ref-for-selectordef-view-transition-image-pair⑦"></a>[::view-transition-image-pair()](https://drafts.csswg.org/css-view-transitions-1/#selectordef-view-transition-image-pair), with its <a id="ref-for-view-transition-name①③"></a>[view transition name](https://drafts.csswg.org/css-view-transitions-1/#view-transition-name) set to <var>transitionName</var>.

    4.  Append <var>imagePair</var> to <var>group</var>.

    5.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-old-image②"></a>[old image](#captured-element-old-image) is not null, then:

        1.  Let <var>old</var> be a new <a id="ref-for-selectordef-view-transition-old⑨"></a>[::view-transition-old()](https://drafts.csswg.org/css-view-transitions-1/#selectordef-view-transition-old), with its <a id="ref-for-view-transition-name①④"></a>[view transition name](https://drafts.csswg.org/css-view-transitions-1/#view-transition-name) set to <var>transitionName</var>, displaying <var>capturedElement</var>’s <a id="ref-for-captured-element-old-image③"></a>[old image](#captured-element-old-image) as its <a id="ref-for-replaced-element②"></a>[replaced](https://drafts.csswg.org/css-display-3/#replaced-element) content.

        2.  Append <var>old</var> to <var>imagePair</var>.

    6.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element④"></a>[new element](#captured-element-new-element) is not null, then:

        1.  Let <var>new</var> be a new <a id="ref-for-selectordef-view-transition-new⑨"></a>[::view-transition-new()](https://drafts.csswg.org/css-view-transitions-1/#selectordef-view-transition-new), with its <a id="ref-for-view-transition-name①⑤"></a>[view transition name](https://drafts.csswg.org/css-view-transitions-1/#view-transition-name) set to <var>transitionName</var>.

            Note: The styling of this pseudo is handled in <a id="ref-for-update-pseudo-element-styles③"></a>[update pseudo-element styles](https://drafts.csswg.org/css-view-transitions-1/#update-pseudo-element-styles).

        2.  Append <var>new</var> to <var>imagePair</var>.

    7.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-old-image④"></a>[old image](#captured-element-old-image) is null, then:

        1.  <a id="ref-for-assert"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element⑤"></a>[new element](#captured-element-new-element) is not null.

        2.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-image-animation-name-rule"></a>[image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a id="ref-for-cssstylerule④"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet⑤"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            :root::view-transition-new(transitionName) {
              animation-name: -ua-view-transition-fade-in;
            }
            ```

            Note: The above code example contains variables to be replaced.

    8.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element⑥"></a>[new element](#captured-element-new-element) is null, then:

        1.  <a id="ref-for-assert①"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>capturedElement</var>’s <a id="ref-for-captured-element-old-image⑤"></a>[old image](#captured-element-old-image) is not null.

        2.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-image-animation-name-rule①"></a>[image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a id="ref-for-cssstylerule⑤"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet⑥"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            :root::view-transition-old(transitionName) {
              animation-name: -ua-view-transition-fade-out;
            }
            ```

            Note: The above code example contains variables to be replaced.

    9.  If both of <var>capturedElement</var>’s <a id="ref-for-captured-element-old-image⑥"></a>[old image](#captured-element-old-image) and <a id="ref-for-captured-element-new-element⑦"></a>[new element](#captured-element-new-element) are not null, then:

        1.  Let <var>transform</var> be <var>capturedElement</var>’s <a id="ref-for-captured-element-old-transform①"></a>[old transform](#captured-element-old-transform).

        2.  Let <var>width</var> be <var>capturedElement</var>’s <a id="ref-for-captured-element-old-width①"></a>[old width](#captured-element-old-width).

        3.  Let <var>height</var> be <var>capturedElement</var>’s <a id="ref-for-captured-element-old-height①"></a>[old height](#captured-element-old-height).

        4.  Let <var>backdropFilter</var> be <var>capturedElement</var>’s <a id="ref-for-captured-element-old-backdrop-filter①"></a>[old backdrop-filter](#captured-element-old-backdrop-filter).

        5.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-group-keyframes"></a>[group keyframes](#captured-element-group-keyframes) to a new <code><a id="ref-for-csskeyframesrule①"></a>[CSSKeyframesRule](https://drafts.csswg.org/css-animations-1/#csskeyframesrule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet⑦"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            @keyframes -ua-view-transition-group-anim-transitionName {
              from {
                transform: transform;
                width: width;
                height: height;
                backdrop-filter: backdropFilter;
              }
            }
            ```

            Note: The above code example contains variables to be replaced.

        6.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-group-animation-name-rule"></a>[group animation name rule](#captured-element-group-animation-name-rule) to a new <code><a id="ref-for-cssstylerule⑥"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet⑧"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            :root::view-transition-group(transitionName) {
              animation-name: -ua-view-transition-group-anim-transitionName;
            }
            ```

            Note: The above code example contains variables to be replaced.

        7.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-image-pair-isolation-rule"></a>[image pair isolation rule](#captured-element-image-pair-isolation-rule) to a new <code><a id="ref-for-cssstylerule⑦"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet⑨"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            :root::view-transition-image-pair(transitionName) {
              isolation: isolate;
            }
            ```

            Note: The above code example contains variables to be replaced.

        8.  Set <var>capturedElement</var>’s <a id="ref-for-captured-element-image-animation-name-rule②"></a>[image animation name rule](#captured-element-image-animation-name-rule) to a new <code><a id="ref-for-cssstylerule⑧"></a>[CSSStyleRule](https://drafts.csswg.org/cssom-1/#cssstylerule)</code> representing the following CSS, and append it to <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet①⓪"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet):

            ``` text
            :root::view-transition-old(transitionName) {
              animation-name: -ua-view-transition-fade-out, -ua-mix-blend-mode-plus-lighter;
            }
            :root::view-transition-new(transitionName) {
              animation-name: -ua-view-transition-fade-in, -ua-mix-blend-mode-plus-lighter;
            }
            ```

            Note: The above code example contains variables to be replaced.

            Note: <a id="ref-for-propdef-mix-blend-mode③"></a>[mix-blend-mode: plus-lighter](https://drafts.csswg.org/compositing-2/#propdef-mix-blend-mode) ensures that the blending of identical pixels from the old and new images results in the same color value as those pixels, and achieves a “correct” cross-fade.

### <a id="skip-the-view-transition-algorithm"></a>7.5. <a id="ref-for-skip-the-view-transition①⑤"></a>[Skip the view transition](#skip-the-view-transition)[](#skip-the-view-transition-algorithm)

To <a id="skip-the-view-transition"></a><strong>skip the view transition</strong> for <code><a id="ref-for-viewtransition②①"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var> with reason <var>reason</var>:

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global⑨"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window⑥"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-assert③"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>transition</var>’s <a id="ref-for-viewtransition-phase①②"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) is not "<code>done</code>".

3.  If <var>transition</var>’s <a id="ref-for-viewtransition-phase①③"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) is before "<code>update-callback-called</code>", then <a id="ref-for-schedule-the-update-callback①"></a>[schedule the update callback](https://drafts.csswg.org/css-view-transitions-1/#schedule-the-update-callback) for <var>transition</var>.

4.  Set <a id="ref-for-document-rendering-suppression-for-view-transitions③"></a>[rendering suppression for view transitions](https://drafts.csswg.org/css-view-transitions-1/#document-rendering-suppression-for-view-transitions) to false.

5.  If <var>document</var>’s <a id="ref-for-document-active-view-transition①①"></a>[active view transition](https://drafts.csswg.org/css-view-transitions-1/#document-active-view-transition) is <var>transition</var>, <a id="ref-for-clear-view-transition①"></a>[Clear view transition](#clear-view-transition) <var>transition</var>.

6.  Set <var>transition</var>’s <a id="ref-for-viewtransition-phase①④"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) to "<code>done</code>".

7.  <a id="ref-for-reject①"></a>[Reject](https://webidl.spec.whatwg.org/#reject) <var>transition</var>’s <a id="ref-for-viewtransition-ready-promise④"></a>[ready promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-ready-promise) with <var>reason</var>.

    Note: The <a id="ref-for-viewtransition-ready-promise⑤"></a>[ready promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-ready-promise) may already be resolved at this point, if <code><a id="ref-for-dom-viewtransition-skiptransition⑤"></a>[skipTransition()](https://drafts.csswg.org/css-view-transitions-1/#dom-viewtransition-skiptransition)</code> is called after we start animating. In that case, this step is a no-op.

8.  <a id="ref-for-resolve②"></a>[Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s <a id="ref-for-viewtransition-finished-promise①"></a>[finished promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-finished-promise) with the result of <a id="ref-for-dfn-perform-steps-once-promise-is-settled①"></a>[reacting](https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled) to <var>transition</var>’s <a id="ref-for-viewtransition-update-callback-done-promise⑥"></a>[update callback done promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-update-callback-done-promise):

    - If the promise was fulfilled, then return undefined.

    Note: Since the rejection of <var>transition</var>’s <a id="ref-for-viewtransition-update-callback-done-promise⑦"></a>[update callback done promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-update-callback-done-promise) isn’t explicitly handled here, if <var>transition</var>’s <a id="ref-for-viewtransition-update-callback-done-promise⑧"></a>update callback done promise rejects, then <var>transition</var>’s <a id="ref-for-viewtransition-finished-promise②"></a>[finished promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-finished-promise) will reject with the same reason.

### <a id="handle-transition-frame-algorithm"></a>7.8. <a id="ref-for-handle-transition-frame①"></a>[Handle transition frame](#handle-transition-frame)[](#handle-transition-frame-algorithm)

To <a id="handle-transition-frame"></a><strong>handle transition frame</strong> given a <code><a id="ref-for-viewtransition②②"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>:

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global①①"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window⑦"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  Let <var>hasActiveAnimations</var> be a boolean, initially false.

3.  <a id="ref-for-list-iterate⑧"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>element</var> of <var>transition</var>’s <a id="ref-for-viewtransition-transition-root-pseudo-element⑤"></a>[transition root pseudo-element](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-transition-root-pseudo-element)’s <a id="ref-for-concept-tree-inclusive-descendant①"></a>[inclusive descendants](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant):

    1.  For each <var>animation</var> whose <a id="ref-for-timeline①"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline) is a <a id="ref-for-document-timeline"></a>[document timeline](https://drafts.csswg.org/web-animations-1/#document-timeline) associated with <var>document</var>, and contains at least one <a id="ref-for-animation-associated-effect"></a>[associated effect](https://drafts.csswg.org/web-animations-1/#animation-associated-effect) whose <a id="ref-for-keyframe-effect-effect-target"></a>[effect target](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) is <var>element</var>, set <var>hasActiveAnimations</var> to true if any of the following conditions are true:

        - <var>animation</var>’s <a id="ref-for-animation-play-state"></a>[play state](https://drafts.csswg.org/web-animations-1/#animation-play-state) is <a id="ref-for-play-state-paused"></a>[paused](https://drafts.csswg.org/web-animations-1/#play-state-paused) or <a id="ref-for-play-state-running"></a>[running](https://drafts.csswg.org/web-animations-1/#play-state-running).

        - <var>document</var>’s <a id="ref-for-pending-animation-event-queue"></a>[pending animation event queue](https://drafts.csswg.org/web-animations-1/#pending-animation-event-queue) has any events associated with <var>animation</var>.

4.  If <var>hasActiveAnimations</var> is false:

    1.  Set <var>transition</var>’s <a id="ref-for-viewtransition-phase①⑤"></a>[phase](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-phase) to "<code>done</code>".

    2.  <a id="ref-for-clear-view-transition②"></a>[Clear view transition](#clear-view-transition) <var>transition</var>.

    3.  <a id="ref-for-resolve③"></a>[Resolve](https://webidl.spec.whatwg.org/#resolve) <var>transition</var>’s <a id="ref-for-viewtransition-finished-promise③"></a>[finished promise](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-finished-promise).

    4.  Return.

5.  If <var>transition</var>’s <a id="ref-for-viewtransition-initial-snapshot-containing-block-size②"></a>[initial snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-initial-snapshot-containing-block-size) is not equal to the <a id="ref-for-snapshot-containing-block-size④"></a>[snapshot containing block size](https://drafts.csswg.org/css-view-transitions-1/#snapshot-containing-block-size), then <a id="ref-for-skip-the-view-transition①⑦"></a>[skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a id="ref-for-invalidstateerror⑥"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm①⓪"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

6.  <a id="ref-for-update-pseudo-element-styles④"></a>[Update pseudo-element styles](https://drafts.csswg.org/css-view-transitions-1/#update-pseudo-element-styles) for <var>transition</var>.

    If failure is returned, then <a id="ref-for-skip-the-view-transition①⑧"></a>[skip the view transition](#skip-the-view-transition) for <var>transition</var> with an "<code><a id="ref-for-invalidstateerror⑦"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException①⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> in <var>transition</var>’s <a id="ref-for-concept-relevant-realm①①"></a>[relevant Realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), and return.

    Note: The above implies that a change in incoming element’s size or position will cause a new keyframe to be generated. This can cause a visual jump. We could retarget smoothly but don’t have a use-case to justify the complexity. See [issue 7813](https://github.com/w3c/csswg-drafts/issues/7813) for details.

### <a id="clear-view-transition-algorithm"></a>7.10. <a id="ref-for-clear-view-transition③"></a>[Clear view transition](#clear-view-transition)[](#clear-view-transition-algorithm)

To <a id="clear-view-transition"></a><strong>clear view transition</strong> of a <code><a id="ref-for-viewtransition②④"></a>[ViewTransition](https://drafts.csswg.org/css-view-transitions-1/#viewtransition)</code> <var>transition</var>:

1.  Let <var>document</var> be <var>transition</var>’s <a id="ref-for-concept-relevant-global①③"></a>[relevant global object’s](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) <a id="ref-for-concept-document-window⑨"></a>[associated document](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-assert⑤"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>document</var>’s <a id="ref-for-document-active-view-transition①⑤"></a>[active view transition](https://drafts.csswg.org/css-view-transitions-1/#document-active-view-transition) is <var>transition</var>.

3.  <a id="ref-for-list-iterate⑨"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>capturedElement</var> of <var>transition</var>’s <a id="ref-for-viewtransition-named-elements⑤"></a>[named elements](https://drafts.csswg.org/css-view-transitions-1/#viewtransition-named-elements)' <a id="ref-for-map-getting-the-values①"></a>[values](https://infra.spec.whatwg.org/#map-getting-the-values):

    1.  If <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element②①"></a>[new element](#captured-element-new-element) is not null, then set <var>capturedElement</var>’s <a id="ref-for-captured-element-new-element②②"></a>new element’s <a id="ref-for-captured-in-a-view-transition①⓪"></a>[captured in a view transition](https://drafts.csswg.org/css-view-transitions-1/#captured-in-a-view-transition) to false.

    2.  <a id="ref-for-list-iterate①⓪"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>style</var> of <var>capturedElement</var>’s <a id="ref-for-captured-element-style-definitions"></a>[style definitions](#captured-element-style-definitions):

        1.  If <var>style</var> is not null, and <var>style</var> is in <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet①②"></a>[dynamic view transition style sheet](https://drafts.csswg.org/css-view-transitions-1/#document-dynamic-view-transition-style-sheet), then remove <var>style</var> from <var>document</var>’s <a id="ref-for-document-dynamic-view-transition-style-sheet①③"></a>dynamic view transition style sheet.

4.  Set <var>document</var>’s <a id="ref-for-document-show-view-transition-tree①"></a>[show view transition tree](https://drafts.csswg.org/css-view-transitions-1/#document-show-view-transition-tree) to false.

5.  Set <var>document</var>’s <a id="ref-for-document-active-view-transition①⑥"></a>[active view transition](https://drafts.csswg.org/css-view-transitions-1/#document-active-view-transition) to null.
