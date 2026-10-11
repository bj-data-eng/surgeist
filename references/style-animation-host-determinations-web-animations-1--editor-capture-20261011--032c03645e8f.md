Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [Web Animations Module Level 1](https://drafts.csswg.org/web-animations-1/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `13e0307c3c72626740baca4c9d38e30848674a2c6904d5f513e19c70ec6bd58f`; 1286635 bytes. Bounded HTML SHA-256 `032c03645e8fa254814cb1944ec7522e65f7b6d0e90d460773683ea649dc6bef`; 126109 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `document-timelines`, `sorting-animation-events`, `the-animation-interface`, `changes-since-last-publication`, `model-liveness`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Web Animations Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/web-animations-1/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/web-animations-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2023/WD-web-animations-1-20230605/>

<https://www.w3.org/TR/2021/WD-web-animations-1-20220908/>

<https://www.w3.org/TR/2021/WD-web-animations-1-20210518/>

<https://www.w3.org/TR/2018/WD-web-animations-1-20181011/>

<https://www.w3.org/TR/2016/WD-web-animations-1-20160913/>

<https://www.w3.org/TR/2015/WD-web-animations-1-20150707/>

<https://www.w3.org/TR/2014/WD-web-animations-20140605/>

<https://www.w3.org/TR/2013/WD-web-animations-20130625/>

<strong>Version History:</strong>

<https://github.com/w3c/csswg-drafts/commits/main/web-animations-1>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/web-animations-1)

[Inline In Spec](https://drafts.csswg.org/web-animations-1/#issues-index)

<strong>Editors:</strong>

[Brian Birtles](mailto:brian@birchill.co.jp) (Invited Expert)

[Robert Flack](mailto:flackr@google.com) (Google)

[Stephen McGruer](mailto:smcgruer@google.com) (Google)

[Antoine Quint](mailto:graouts@apple.com) (Apple Inc)

<strong>Former Editors:</strong>

[Shane Stephens](mailto:shans@google.com) (Google)

[Douglas Stockwell](mailto:dstockwell@google.com) (Google)

[Alex Danilo](mailto:adanilo@google.com) (Google)

[Tab Atkins](mailto:jackalmage@gmail.com) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/web-animations-1/Overview.bs)

<strong>Participate:</strong>

[Fix the text through GitHub](https://github.com/w3c/csswg-drafts/tree/main/web-animations-1)

Join the "waapi" channel on the [Animation at Work](https://damp-lake-50659.herokuapp.com/) slack

IRC: [\#webanimations](ircs://irc.w3.org:6667/webanimations) on W3C’s IRC

<strong>Test Suite:</strong>

<https://wpt.fyi/results/web-animations/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “web-animations” in the title, like this: “\[web-animations\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bweb-animations%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

#### <a id="document-timelines"></a>4.3.1. Document timelines[](#document-timelines)

A <a id="document-timeline"></a><strong>document timeline</strong> is a type of <a id="ref-for-timeline⑦"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline) that is <a id="ref-for-timeline-associated-with-a-document"></a>[associated with a document](https://drafts.csswg.org/web-animations-1/#timeline-associated-with-a-document) and whose <a id="ref-for-timeline-current-time④"></a>[current time](https://drafts.csswg.org/web-animations-1/#timeline-current-time) is calculated as a fixed offset from the <var>now</var> timestamp provided each time the <a id="ref-for-update-animations-and-send-events"></a>[update animations and send events](https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events) procedure is run. This fixed offset is equal to the <a id="ref-for-timeline-current-time⑤"></a>current time of the <a id="ref-for-document-default-document-timeline"></a>[default document timeline](#document-default-document-timeline) when this timeline’s <a id="ref-for-timeline-current-time⑥"></a>current time was zero, and is thus referred to as the document timeline’s <a id="origin-time"></a><strong>origin time</strong>.

<a id="issue-9fa706cb"></a>

<strong>Issue:</strong>

[](#issue-9fa706cb) There must be a better term than "origin time"— it’s too similar to "time origin". [\[Issue \#2079\]](https://github.com/w3c/csswg-drafts/issues/2079)

Prior to establishing the <a id="ref-for-concept-settings-object-time-origin"></a>[time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin) for its associated document, a <a id="ref-for-document-timeline"></a>[document timeline](#document-timeline) is <a id="ref-for-inactive-timeline"></a>[inactive](https://drafts.csswg.org/web-animations-1/#inactive-timeline).

After a <a id="ref-for-document-timeline①"></a>[document timeline](#document-timeline) becomes <a id="ref-for-active-timeline"></a>[active](https://drafts.csswg.org/web-animations-1/#active-timeline), it is <a id="ref-for-monotonically-increasing-timeline"></a>[monotonically increasing](https://drafts.csswg.org/web-animations-1/#monotonically-increasing-timeline).

A <a id="ref-for-document-timeline②"></a>[document timeline](#document-timeline) that is associated with a <code><a id="ref-for-document"></a>[Document](https://html.spec.whatwg.org/#document)</code> that is not an <a id="ref-for-active-document"></a>[active document](https://html.spec.whatwg.org/multipage/browsers.html#active-document) is also considered to be <a id="ref-for-inactive-timeline①"></a>[inactive](https://drafts.csswg.org/web-animations-1/#inactive-timeline).

To <a id="ref-for-timeline-time-to-origin-relative-time"></a>[convert a timeline time, <var>timeline time</var>, to an origin-relative time](https://drafts.csswg.org/web-animations-1/#timeline-time-to-origin-relative-time) for a document timeline, <var>timeline</var>, return the sum of the <var>timeline time</var> and <var>timeline</var>’s <a id="ref-for-origin-time"></a>[origin time](#origin-time). If <var>timeline</var> is inactive, return an <a id="ref-for-unresolved③"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) <a id="ref-for-time-value①③"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value).

##### <a id="the-documents-default-timeline"></a>4.3.1.1. The default document timeline[](#the-documents-default-timeline)

Each <code><a id="ref-for-document①"></a>[Document](https://html.spec.whatwg.org/#document)</code> has a <a id="ref-for-document-timeline③"></a>[document timeline](#document-timeline) called the <a id="document-default-document-timeline"></a><strong>default document timeline</strong>. The <a id="ref-for-document-default-document-timeline①"></a>[default document timeline](#document-default-document-timeline) is unique to each document and persists for the lifetime of the document, including calls to <a id="ref-for-dom-document-open"></a>[document.open()](https://html.spec.whatwg.org/multipage/webappapis.html#dom-document-open) [\[HTML\]](https://drafts.csswg.org/web-animations-1/#biblio-html).

The <a id="ref-for-document-default-document-timeline②"></a>[default document timeline](#document-default-document-timeline) has an <a id="ref-for-origin-time①"></a>[origin time](#origin-time) of zero.

##### <a id="document-wallclock-time"></a>4.3.1.2. Relationship to wall-clock time[](#document-wallclock-time)

<em>This section is non-normative</em>

Since no scaling is applied to the <var>now</var> timestamp values provided to a <a id="ref-for-document-timeline④"></a>[document timeline](#document-timeline), the <a id="ref-for-time-value①④"></a>[time values](https://drafts.csswg.org/web-animations-1/#time-value) it produces will be proportional to wall-clock milliseconds.

Furthermore, since the <a id="ref-for-time-value①⑤"></a>[time values](https://drafts.csswg.org/web-animations-1/#time-value) of the <a id="ref-for-document-default-document-timeline③"></a>[default document timeline](#document-default-document-timeline) have a zero offset from the <a id="ref-for-concept-settings-object-time-origin①"></a>[time origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-time-origin), <code>document.timeline.currentTime</code> will roughly correspond to [<code>Performance.now()</code>](https://www.w3.org/TR/hr-time/#dom-performance-now) [\[HR-TIME\]](https://drafts.csswg.org/web-animations-1/#biblio-hr-time) with the exception that <code>document.timeline.currentTime</code> does not change in between calls to the <a id="ref-for-update-animations-and-send-events①"></a>[update animations and send events](https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events) procedure.

##### <a id="sorting-animation-events"></a>4.5.18.1. Sorting animation events[](#sorting-animation-events)

The following definitions are provided to assist with sorting queued events.

To <a id="animation-time-to-timeline-time"></a><strong>convert an animation time to timeline time</strong> a <a id="ref-for-time-value③⑥"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value), <var>time</var>, that is relative to the <a id="ref-for-animation-start-time④⑨"></a>[start time](https://drafts.csswg.org/web-animations-1/#animation-start-time) of an animation, <var>animation</var>, perform the following steps:

1.  If <var>time</var> is <a id="ref-for-unresolved⑧①"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved), return <var>time</var>.

2.  If <var>time</var> is infinity, return an <a id="ref-for-unresolved⑧②"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) <a id="ref-for-time-value③⑦"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value).

3.  If <var>animation</var>’s <a id="ref-for-playback-rate③⑤"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) is zero, return an <a id="ref-for-unresolved⑧③"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) <a id="ref-for-time-value③⑧"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value).

4.  If <var>animation</var>’s <a id="ref-for-animation-start-time⑤⓪"></a>[start time](https://drafts.csswg.org/web-animations-1/#animation-start-time) is <a id="ref-for-unresolved⑧④"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved), return an <a id="ref-for-unresolved⑧⑤"></a>unresolved <a id="ref-for-time-value③⑨"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value).

5.  Return the result of calculating <code><var>time</var> × (1 / <var>playback rate</var>) + <var>start time</var></code>, where <var>playback rate</var> and <var>start time</var> are the <a id="ref-for-playback-rate③⑥"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) and <a id="ref-for-animation-start-time⑤①"></a>[start time](https://drafts.csswg.org/web-animations-1/#animation-start-time) of <var>animation</var>, respectively.

To <a id="animation-time-to-origin-relative-time"></a><strong>convert a timeline time to an origin-relative time</strong> a <a id="ref-for-time-value④⓪"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value), <var>time</var>, that is expressed in the same scale as the <a id="ref-for-time-value④①"></a>time values of a <a id="ref-for-timeline④⑧"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline), <var>timeline</var>, perform the following steps:

1.  Let <var>timeline time</var> be the result of <a id="ref-for-animation-time-to-timeline-time"></a>[converting](#animation-time-to-timeline-time) <var>time</var> from an animation time to a timeline time.

2.  If <var>timeline time</var> is <a id="ref-for-unresolved⑧⑥"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved), return <var>time</var>.

3.  If <var>animation</var> is not associated with a <a id="ref-for-timeline④⑨"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline), return an <a id="ref-for-unresolved⑧⑦"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) time value.

4.  If <var>animation</var> is associated with an <a id="ref-for-inactive-timeline①①"></a>[inactive timeline](https://drafts.csswg.org/web-animations-1/#inactive-timeline), return an <a id="ref-for-unresolved⑧⑧"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) time value.

5.  If there is no procedure to <a id="ref-for-timeline-time-to-origin-relative-time②"></a>[convert a timeline time to an origin-relative time](https://drafts.csswg.org/web-animations-1/#timeline-time-to-origin-relative-time) for the timeline associated with <var>animation</var>, return an <a id="ref-for-unresolved⑧⑨"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) <a id="ref-for-time-value④②"></a>[time value](https://drafts.csswg.org/web-animations-1/#time-value).

6.  Return the result of <a id="ref-for-timeline-time-to-origin-relative-time③"></a>[converting](https://drafts.csswg.org/web-animations-1/#timeline-time-to-origin-relative-time) <var>timeline time</var> to an origin-relative time using the procedure defined for the <a id="ref-for-timeline⑤⓪"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline) associated with <var>animation</var>.

### <a id="the-animation-interface"></a>6.4. The <code><a id="ref-for-animation①"></a>[Animation](#animation)</code> interface[](#the-animation-interface)

<a id="ref-for-concept-animation⑥⑦"></a>[Animations](https://drafts.csswg.org/web-animations-1/#concept-animation) are represented in the Web Animations API by the <a id="animation"></a><strong><code>Animation</code></strong> interface.

<a id="ref-for-Exposed②"></a><a id="ref-for-animation②"></a><a id="ref-for-eventtarget"></a><a id="ref-for-dom-animation-animation"></a><a id="ref-for-animationeffect"></a><a id="ref-for-dom-animation-animation-effect-timeline-effect"></a><a id="ref-for-animationtimeline③"></a><a id="ref-for-dom-animation-animation-effect-timeline-timeline"></a><a id="ref-for-idl-DOMString"></a><a id="ref-for-dom-animation-id"></a><a id="ref-for-animationeffect①"></a><a id="ref-for-dom-animation-effect"></a><a id="ref-for-animationtimeline④"></a><a id="ref-for-dom-animation-timeline"></a><a id="ref-for-idl-double②"></a><a id="ref-for-dom-animation-starttime"></a><a id="ref-for-idl-double③"></a><a id="ref-for-dom-animation-currenttime"></a><a id="ref-for-idl-double④"></a><a id="ref-for-dom-animation-playbackrate"></a><a id="ref-for-enumdef-animationplaystate"></a><a id="ref-for-dom-animation-playstate"></a><a id="ref-for-enumdef-animationreplacestate"></a><a id="ref-for-dom-animation-replacestate"></a><a id="ref-for-idl-boolean"></a><a id="ref-for-dom-animation-pending"></a><a id="ref-for-idl-promise"></a><a id="ref-for-animation③"></a><a id="ref-for-dom-animation-ready"></a><a id="ref-for-idl-promise①"></a><a id="ref-for-animation④"></a><a id="ref-for-dom-animation-finished"></a><a id="ref-for-eventhandler"></a><a id="ref-for-dom-animation-onfinish"></a><a id="ref-for-eventhandler①"></a><a id="ref-for-dom-animation-oncancel"></a><a id="ref-for-eventhandler②"></a><a id="ref-for-dom-animation-onremove"></a><a id="ref-for-idl-undefined"></a><a id="ref-for-dom-animation-cancel"></a><a id="ref-for-idl-undefined①"></a><a id="ref-for-dom-animation-finish①"></a><a id="ref-for-idl-undefined②"></a><a id="ref-for-dom-animation-play"></a><a id="ref-for-idl-undefined③"></a><a id="ref-for-dom-animation-pause"></a><a id="ref-for-idl-undefined④"></a><a id="ref-for-dom-animation-updateplaybackrate"></a><a id="ref-for-idl-double⑤"></a><a id="ref-for-dom-animation-updateplaybackrate-playbackrate-playbackrate"></a><a id="ref-for-idl-undefined⑤"></a><a id="ref-for-dom-animation-reverse"></a><a id="ref-for-idl-undefined⑥"></a><a id="ref-for-dom-animation-persist"></a><a id="ref-for-cereactions"></a><a id="ref-for-idl-undefined⑦"></a><a id="ref-for-dom-animation-commitstyles①"></a>

``` text
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

<strong><a id="dom-animation-animation"></a><strong><code>Animation (<var>effect</var>, <var>timeline</var>)</code></strong></strong>

Creates a new <code><a id="ref-for-animation⑤"></a>[Animation](#animation)</code> object using the following procedure:

1.  Let <var>animation</var> be a new <code><a id="ref-for-animation⑥"></a>[Animation](#animation)</code> object.

2.  Run the procedure to <a id="ref-for-animation-set-the-timeline-of-an-animation"></a>[set the timeline of an animation](https://drafts.csswg.org/web-animations-1/#animation-set-the-timeline-of-an-animation) on <var>animation</var> passing <var>timeline</var> as the <var>new timeline</var>; or, if the <var>timeline</var> argument is missing, passing the <a id="ref-for-document-default-document-timeline⑥"></a>[default document timeline](#document-default-document-timeline) of the <code><a id="ref-for-document⑨"></a>[Document](https://html.spec.whatwg.org/#document)</code> <a id="ref-for-concept-document-window①"></a>[associated](https://html.spec.whatwg.org/multipage/browsers.html#concept-document-window) with the <code><a id="ref-for-window①"></a>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> that is the <a id="ref-for-current-global-object③"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object).

3.  Run the procedure to <a id="ref-for-animation-set-the-associated-effect-of-an-animation①"></a>[set the associated effect of an animation](https://drafts.csswg.org/web-animations-1/#animation-set-the-associated-effect-of-an-animation) on <var>animation</var> passing <var>source</var> as the <var>new effect</var>.

<strong><a id="dom-animation-animation-effect-timeline-effect"></a><strong><code>effect</code></strong></strong>

An optional value which, if not <code>null</code>, specifies the <a id="ref-for-animation-associated-effect③①"></a>[associated effect](https://drafts.csswg.org/web-animations-1/#animation-associated-effect) to assign to the newly created <a id="ref-for-concept-animation⑥⑧"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation).

<strong><a id="dom-animation-animation-effect-timeline-timeline"></a><strong><code>timeline</code></strong></strong>

An optional value which, if present, specifies the <a id="ref-for-timeline⑤⑥"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline) with which to associate the newly-created <a id="ref-for-concept-animation⑥⑨"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation). If missing, the <a id="ref-for-document-default-document-timeline⑦"></a>[default document timeline](#document-default-document-timeline) of the <code><a id="ref-for-document①⓪"></a>[Document](https://html.spec.whatwg.org/#document)</code> <a id="ref-for-concept-document-window②"></a>[associated](https://html.spec.whatwg.org/multipage/browsers.html#concept-document-window) with the <code><a id="ref-for-window②"></a>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> that is the <a id="ref-for-current-global-object④"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object) is used.

Tests

- [constructor.html](https://wpt.fyi/results/web-animations/interfaces/Animation/constructor.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/constructor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/constructor.html)

<strong><a id="dom-animation-id"></a><strong><code>id</code></strong>, of type <a id="ref-for-idl-DOMString①"></a>[DOMString](https://webidl.spec.whatwg.org/#idl-DOMString)</strong>

A string used to identify the animation.

Tests

- [id.html](https://wpt.fyi/results/web-animations/interfaces/Animation/id.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/id.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/id.html)

<strong><a id="dom-animation-effect"></a><strong><code>effect</code></strong>, of type <a id="ref-for-animationeffect②"></a>[AnimationEffect](https://drafts.csswg.org/web-animations-1/#animationeffect), nullable</strong>

The <a id="ref-for-animation-associated-effect③②"></a>[associated effect](https://drafts.csswg.org/web-animations-1/#animation-associated-effect) of this animation. Setting this attribute updates the object’s <a id="ref-for-animation-associated-effect③③"></a>associated effect using the procedure to <a id="ref-for-animation-set-the-associated-effect-of-an-animation②"></a>[set the associated effect of an animation](https://drafts.csswg.org/web-animations-1/#animation-set-the-associated-effect-of-an-animation).

Tests

- [effect.html](https://wpt.fyi/results/web-animations/interfaces/Animation/effect.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/effect.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/effect.html)

<strong><a id="dom-animation-timeline"></a><strong><code>timeline</code></strong>, of type <a id="ref-for-animationtimeline⑤"></a>[AnimationTimeline](https://drafts.csswg.org/web-animations-1/#animationtimeline), nullable</strong>

The <a id="ref-for-timeline⑤⑦"></a>[timeline](https://drafts.csswg.org/web-animations-1/#timeline) associated with this animation. Setting this attribute updates the object’s <a id="ref-for-timeline⑤⑧"></a>timeline using the procedure to <a id="ref-for-animation-set-the-timeline-of-an-animation①"></a>[set the timeline of an animation](https://drafts.csswg.org/web-animations-1/#animation-set-the-timeline-of-an-animation).

<strong><a id="dom-animation-starttime"></a><strong><code>startTime</code></strong>, of type <a id="ref-for-idl-double⑥"></a>[double](https://webidl.spec.whatwg.org/#idl-double), nullable</strong>

Returns the <a id="ref-for-animation-start-time⑤⑧"></a>[start time](https://drafts.csswg.org/web-animations-1/#animation-start-time) of this animation. Setting this attribute updates the <a id="ref-for-animation-start-time⑤⑨"></a>start time using the procedure to <a id="ref-for-set-the-start-time①"></a>[set the start time](https://drafts.csswg.org/web-animations-1/#set-the-start-time) of this object to the new value.

<strong><a id="dom-animation-currenttime"></a><strong><code>currentTime</code></strong>, of type <a id="ref-for-idl-double⑦"></a>[double](https://webidl.spec.whatwg.org/#idl-double), nullable</strong>

The <a id="ref-for-animation-current-time⑤⑦"></a>[current time](https://drafts.csswg.org/web-animations-1/#animation-current-time) of this animation. Setting this attribute follows the procedure to <a id="ref-for-animation-set-the-current-time③"></a>[set the current time](https://drafts.csswg.org/web-animations-1/#animation-set-the-current-time) of this object to the new value.

<strong><a id="dom-animation-playbackrate"></a><strong><code>playbackRate</code></strong>, of type <a id="ref-for-idl-double⑧"></a>[double](https://webidl.spec.whatwg.org/#idl-double)</strong>

The <a id="ref-for-playback-rate④①"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) of this animation. Setting this attribute follows the procedure to <a id="ref-for-set-the-playback-rate②"></a>[set the playback rate](https://drafts.csswg.org/web-animations-1/#set-the-playback-rate) of this object to the new value.

<strong>Note:</strong>

Setting this attribute performs a synchronous update to the <a id="ref-for-playback-rate④②"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) meaning that it does not make any attempt to synchronize with the playback state of animations running on a separate process or thread. As a result, setting the <code><a id="ref-for-dom-animation-playbackrate①"></a>[playbackRate](#dom-animation-playbackrate)</code> for an in-flight animation can cause it to jump.

To set the <a id="ref-for-playback-rate④③"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) for an in-flight animation such that it smoothly updates, use the asynchronous <code><a id="ref-for-dom-animation-updateplaybackrate①"></a>[updatePlaybackRate()](#dom-animation-updateplaybackrate)</code> method.

<strong><a id="dom-animation-playstate"></a><strong><code>playState</code></strong>, of type <a id="ref-for-enumdef-animationplaystate①"></a>[AnimationPlayState](#enumdef-animationplaystate), readonly</strong>

The <a id="ref-for-animation-play-state①①"></a>[play state](https://drafts.csswg.org/web-animations-1/#animation-play-state) of this animation.

<strong><a id="dom-animation-replacestate"></a><strong><code>replaceState</code></strong>, of type <a id="ref-for-enumdef-animationreplacestate①"></a>[AnimationReplaceState](#enumdef-animationreplacestate), readonly</strong>

The <a id="ref-for-replace-state⑦"></a>[replace state](https://drafts.csswg.org/web-animations-1/#replace-state) of this animation.

<strong><a id="dom-animation-pending"></a><strong><code>pending</code></strong>, of type <a id="ref-for-idl-boolean①"></a>[boolean](https://webidl.spec.whatwg.org/#idl-boolean), readonly</strong>

Returns true if this animation has a <a id="ref-for-pending-play-task①⑨"></a>[pending play task](https://drafts.csswg.org/web-animations-1/#pending-play-task) or a <a id="ref-for-pending-pause-task①⑨"></a>[pending pause task](https://drafts.csswg.org/web-animations-1/#pending-pause-task).

Tests

- [pending.html](https://wpt.fyi/results/web-animations/interfaces/Animation/pending.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/pending.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/pending.html)

<strong><a id="dom-animation-ready"></a><strong><code>ready</code></strong>, of type Promise\<<a id="ref-for-animation⑦"></a>[Animation](#animation)\>, readonly</strong>

Returns the <a id="ref-for-current-ready-promise①⑦"></a>[current ready promise](https://drafts.csswg.org/web-animations-1/#current-ready-promise) for this object.

Tests

- [ready.html](https://wpt.fyi/results/web-animations/interfaces/Animation/ready.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/ready.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/ready.html)

<strong><a id="dom-animation-finished"></a><strong><code>finished</code></strong>, of type Promise\<<a id="ref-for-animation⑧"></a>[Animation](#animation)\>, readonly</strong>

Returns the <a id="ref-for-current-finished-promise①②"></a>[current finished promise](https://drafts.csswg.org/web-animations-1/#current-finished-promise) for this object.

Tests

- [finished.html](https://wpt.fyi/results/web-animations/interfaces/Animation/finished.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/finished.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/finished.html)

<strong><a id="dom-animation-onfinish"></a><strong><code>onfinish</code></strong>, of type <a id="ref-for-eventhandler③"></a>[EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)</strong>

The event handler for the <a id="ref-for-finish-event②"></a>[finish event](https://drafts.csswg.org/web-animations-1/#finish-event).

Tests

- [onfinish.html](https://wpt.fyi/results/web-animations/interfaces/Animation/onfinish.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/onfinish.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/onfinish.html)

<strong><a id="dom-animation-oncancel"></a><strong><code>oncancel</code></strong>, of type <a id="ref-for-eventhandler④"></a>[EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)</strong>

The event handler for the <a id="ref-for-cancel-event③"></a>[cancel event](https://drafts.csswg.org/web-animations-1/#cancel-event).

Tests

- [oncancel.html](https://wpt.fyi/results/web-animations/interfaces/Animation/oncancel.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/oncancel.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/oncancel.html)

<strong><a id="dom-animation-onremove"></a><strong><code>onremove</code></strong>, of type <a id="ref-for-eventhandler⑤"></a>[EventHandler](https://html.spec.whatwg.org/multipage/webappapis.html#eventhandler)</strong>

The event handler for the <a id="ref-for-remove-event①"></a>[remove event](https://drafts.csswg.org/web-animations-1/#remove-event).

Tests

- [onremove.html](https://wpt.fyi/results/web-animations/interfaces/Animation/onremove.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/onremove.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/onremove.html)

<strong><a id="dom-animation-cancel"></a><strong><code>void cancel()</code></strong></strong>

Clears all effects caused by this animation and aborts its playback by running the <a id="ref-for-cancel-an-animation①"></a>[cancel an animation](https://drafts.csswg.org/web-animations-1/#cancel-an-animation) procedure for this object.

Tests

- [cancel.html](https://wpt.fyi/results/web-animations/interfaces/Animation/cancel.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/cancel.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/cancel.html)

<strong><a id="dom-animation-finish"></a><strong><code>void finish()</code></strong></strong>

<a id="ref-for-seek③"></a>[Seeks](https://drafts.csswg.org/web-animations-1/#seek) the animation to the <a id="ref-for-associated-effect-end①⑦"></a>[associated effect end](https://drafts.csswg.org/web-animations-1/#associated-effect-end) in the current direction by running the <a id="ref-for-finish-an-animation②"></a>[finish an animation](https://drafts.csswg.org/web-animations-1/#finish-an-animation) procedure for this object.

<strong><code><a id="ref-for-idl-DOMException⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-invalidstateerror④"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>"</strong>

Raised if this animation’s <a id="ref-for-effective-playback-rate①⓪"></a>[effective playback rate](https://drafts.csswg.org/web-animations-1/#effective-playback-rate) is zero, or if this animation’s <a id="ref-for-effective-playback-rate①①"></a>effective playback rate is \> zero and the <a id="ref-for-associated-effect-end①⑧"></a>[associated effect end](https://drafts.csswg.org/web-animations-1/#associated-effect-end) is infinity.

<strong><a id="dom-animation-play"></a><strong><code>void play()</code></strong></strong>

Begins or resumes playback of the animation by running the procedure to <a id="ref-for-play-an-animation④"></a>[play an animation](https://drafts.csswg.org/web-animations-1/#play-an-animation) passing true as the value of the <var>auto-rewind</var> flag.

<strong><code><a id="ref-for-idl-DOMException⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-invalidstateerror⑤"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>"</strong>

Raised if this animation’s <a id="ref-for-effective-playback-rate①②"></a>[effective playback rate](https://drafts.csswg.org/web-animations-1/#effective-playback-rate) is negative, the <a id="ref-for-associated-effect-end①⑨"></a>[associated effect end](https://drafts.csswg.org/web-animations-1/#associated-effect-end) is infinity, and its <a id="ref-for-animation-current-time⑤⑧"></a>[current time](https://drafts.csswg.org/web-animations-1/#animation-current-time) is unresolved or less than or equal to zero.

<strong><a id="dom-animation-pause"></a><strong><code>void pause()</code></strong></strong>

Suspends the playback of this animation by running the procedure to <a id="ref-for-pause-an-animation"></a>[pause an animation](https://drafts.csswg.org/web-animations-1/#pause-an-animation) for this object.

<strong><code><a id="ref-for-idl-DOMException⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-invalidstateerror⑥"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>"</strong>

Raised if this animation’s <a id="ref-for-animation-current-time⑤⑨"></a>[current time](https://drafts.csswg.org/web-animations-1/#animation-current-time) is unresolved, its <a id="ref-for-playback-rate④④"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) is negative, and the <a id="ref-for-associated-effect-end②⓪"></a>[associated effect end](https://drafts.csswg.org/web-animations-1/#associated-effect-end) is infinity.

Tests

- [pause.html](https://wpt.fyi/results/web-animations/interfaces/Animation/pause.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/pause.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/pause.html)

<strong><a id="dom-animation-updateplaybackrate"></a><strong><code>void updatePlaybackRate(<var>playbackRate</var>)</code></strong></strong>

Performs an asynchronous update of the <a id="ref-for-playback-rate④⑤"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) of this animation by performing the <a id="ref-for-seamlessly-update-the-playback-rate"></a>[seamlessly update the playback rate](https://drafts.csswg.org/web-animations-1/#seamlessly-update-the-playback-rate) procedure, passing <code><a id="ref-for-dom-animation-updateplaybackrate-playbackrate-playbackrate①"></a>[playbackRate](#dom-animation-updateplaybackrate-playbackrate-playbackrate)</code> as the <var>new playback rate</var>.

<strong><a id="dom-animation-updateplaybackrate-playbackrate-playbackrate"></a><strong><code>playbackRate</code></strong></strong>

A finite real number specifying the updated playback rate to use.

<strong><a id="dom-animation-reverse"></a><strong><code>void reverse()</code></strong></strong>

Inverts the <a id="ref-for-playback-rate④⑥"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate) of this animation and plays it using the <a id="ref-for-reverse-an-animation"></a>[reverse an animation](https://drafts.csswg.org/web-animations-1/#reverse-an-animation) procedure for this object. As with <code><a id="ref-for-dom-animation-play①"></a>[play()](#dom-animation-play)</code>, this method unpauses the animation and, if the animation has already finished playing in the reversed direction, <a id="ref-for-seek④"></a>[seeks](https://drafts.csswg.org/web-animations-1/#seek) to the start of the <a id="ref-for-animation-associated-effect③④"></a>[associated effect](https://drafts.csswg.org/web-animations-1/#animation-associated-effect).

<strong><code><a id="ref-for-idl-DOMException⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-invalidstateerror⑦"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>"</strong>

Raised if this animation has no associated <a id="ref-for-active-timeline⑥"></a>[active timeline](https://drafts.csswg.org/web-animations-1/#active-timeline), or if its reversed <a id="ref-for-effective-playback-rate①③"></a>[effective playback rate](https://drafts.csswg.org/web-animations-1/#effective-playback-rate) would be negative, the <a id="ref-for-associated-effect-end②①"></a>[associated effect end](https://drafts.csswg.org/web-animations-1/#associated-effect-end) is infinity, and its <a id="ref-for-animation-current-time⑥⓪"></a>[current time](https://drafts.csswg.org/web-animations-1/#animation-current-time) is unresolved or less than or equal to zero.

<strong><a id="dom-animation-persist"></a><strong><code>void persist()</code></strong></strong>

Sets this animation’s <a id="ref-for-replace-state⑧"></a>[replace state](https://drafts.csswg.org/web-animations-1/#replace-state) to <a id="ref-for-persisted-replace-state"></a>[persisted](https://drafts.csswg.org/web-animations-1/#persisted-replace-state).

<strong><a id="dom-animation-commitstyles"></a><strong><code>void commitStyles()</code></strong></strong>

Writes the current <a id="ref-for-effect-value②②"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) produced by this animation’s <a id="ref-for-animation-effect⑧⓪"></a>[animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) to their corresponding <a id="ref-for-keyframe-effect-effect-target①④"></a>[effect targets](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target)' inline style using the <a id="ref-for-commit-computed-styles"></a>[commit computed styles](#commit-computed-styles) procedure.

Unlike most other methods defined on this interface, calling this method <em>does</em> trigger a <a id="ref-for-style-change-event"></a>[style change event](https://drafts.csswg.org/css-transitions-1/#style-change-event) (see [§ 6.13 Model liveness](#model-liveness)).

<strong><code><a id="ref-for-idl-DOMException①⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-nomodificationallowederror"></a>[NoModificationAllowedError](https://webidl.spec.whatwg.org/#nomodificationallowederror)</code>"</strong>

Raised if any <a id="ref-for-keyframe-effect-effect-target①⑤"></a>[effect target](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) is not an element capable of having a <a id="ref-for-style-attribute"></a>[style attribute](https://drafts.csswg.org/css-style-attr/#style-attribute).

<strong><code><a id="ref-for-idl-DOMException①①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> of type "<code><a id="ref-for-invalidstateerror⑧"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>"</strong>

Raised if any <a id="ref-for-keyframe-effect-effect-target①⑥"></a>[effect target](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) is not <a id="ref-for-being-rendered"></a>[being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered).

Tests

- [commitStyles.html](https://wpt.fyi/results/web-animations/interfaces/Animation/commitStyles.html) [(live test)](http://wpt.live/web-animations/interfaces/Animation/commitStyles.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/interfaces/Animation/commitStyles.html)

<strong>Note:</strong>

In order to simplify the common case of persisting a completed animation, the procedure to <a id="ref-for-commit-computed-styles①"></a>[commit computed styles](#commit-computed-styles) uses endpoint-<em>inclusive</em> timing (see [§ 4.6.9.3 Interval timing](https://drafts.csswg.org/web-animations-1/#interval-timing)) when determining the phase of the animation (see [§ 4.6.6 Animation effect phases and states](https://drafts.csswg.org/web-animations-1/#animation-effect-phases-and-states)).

As a result, the following code will persist the <code>transform&#58; translateY(100px)</code> style in specified style despite not having a <a id="ref-for-fill-mode①②"></a>[fill mode](https://drafts.csswg.org/web-animations-1/#fill-mode) of <a id="ref-for-fill-mode-forwards④"></a>[forwards](https://drafts.csswg.org/web-animations-1/#fill-mode-forwards) or <a id="ref-for-fill-mode-both④"></a>[both](https://drafts.csswg.org/web-animations-1/#fill-mode-both).

``` text
elem.animate({ transform: 'translateY(100px)' }, 200).finished.then(() => {
  elem.commitStyles();
});
```

<strong>Note:</strong>

Since the procedure to <a id="ref-for-commit-computed-styles②"></a>[commit computed styles](#commit-computed-styles) includes the <a id="ref-for-effect-value②③"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) for the animation even if it is <a id="ref-for-removed-replace-state⑤"></a>[removed](https://drafts.csswg.org/web-animations-1/#removed-replace-state), this method is useful for retaining the effect of an animation after it has been replaced (see [§ 5.5.2 Removing replaced animations](https://drafts.csswg.org/web-animations-1/#removing-replaced-animations)) without retaining the actual animation.

Note that the values committed are the <em>computed</em> values produced by the <a id="ref-for-animation-effect⑧①"></a>[animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) at the time when this method is called. Since these values are computed values, they do not reflect to changes to context such as responding to changes to CSS variables or recalculating <a id="ref-for-em"></a>[em](https://drafts.csswg.org/css-values-4/#em) units based on changes to the computed font-size in the way the values produced by a live animation would.

In order to retain full fidelity of a filling animation’s result after it has been replaced (see [§ 5.5 Replacing animations](https://drafts.csswg.org/web-animations-1/#replacing-animations)), the <code><a id="ref-for-dom-animation-persist①"></a>[persist()](#dom-animation-persist)</code> method can be used, but note that doing so will mean the animation continues to consume resources.

To <a id="commit-computed-styles"></a><strong>commit computed styles</strong> for an <a id="ref-for-concept-animation⑦⓪"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation), <var>animation</var>:

1.  Let <var>targets</var> be the <a id="ref-for-ordered-set"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of all <a id="ref-for-keyframe-effect-effect-target①⑦"></a>[effect targets](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) for <a id="ref-for-animation-effect⑧②"></a>[animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) <a id="ref-for-animation-effect-associated-animation②⓪"></a>[associated](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) with <var>animation</var>.

2.  <a id="ref-for-list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>target</var> in <var>targets</var>:

    1.  If <var>target</var> is not an element capable of having a <a id="ref-for-style-attribute①"></a>[style attribute](https://drafts.csswg.org/css-style-attr/#style-attribute) [\[CSS-STYLE-ATTR\]](https://drafts.csswg.org/web-animations-1/#biblio-css-style-attr) (for example, it is a <a id="ref-for-pseudo-element⑤"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) or is an element in a document format for which style attributes are not defined) <a id="ref-for-dfn-throw⑤"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-nomodificationallowederror①"></a>[NoModificationAllowedError](https://webidl.spec.whatwg.org/#nomodificationallowederror)</code>" <code><a id="ref-for-idl-DOMException①②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> and abort these steps.

    2.  If, after applying any pending style changes, <var>target</var> is not <a id="ref-for-being-rendered①"></a>[being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered), <a id="ref-for-dfn-throw⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidstateerror⑨"></a>[InvalidStateError](https://webidl.spec.whatwg.org/#invalidstateerror)</code>" <code><a id="ref-for-idl-DOMException①③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> and abort these steps.

        <a id="issue-8f3b970c"></a>

        <strong>Issue:</strong>

        [](#issue-8f3b970c) The definition of <a id="ref-for-being-rendered②"></a>[being rendered](https://html.spec.whatwg.org/multipage/browsers.html#being-rendered) [\[HTML\]](https://drafts.csswg.org/web-animations-1/#biblio-html) with regards to <a id="ref-for-propdef-display"></a>[display: contents](https://drafts.csswg.org/css-display-4/#propdef-display) is still [under discussion](https://github.com/whatwg/html/issues/1837). For the purpose of this procedure, we assume that an element with <a id="ref-for-propdef-display①"></a>display: contents that otherwise would have associated layout boxes (i.e. it is <a id="ref-for-connected"></a>[connected](https://dom.spec.whatwg.org/#connected) and not part of a <a id="ref-for-propdef-display②"></a>display: none subtree) <em>is</em> being rendered.

    3.  Let <var>inline style</var> be the result of getting the <a id="ref-for-css-declaration-block"></a>[CSS declaration block](https://drafts.csswg.org/cssom-1/#css-declaration-block) corresponding to <var>target</var>’s <a id="ref-for-style-attribute②"></a>[style attribute](https://drafts.csswg.org/css-style-attr/#style-attribute). If <var>target</var> does not <a id="ref-for-concept-element-attribute-has"></a>[have](https://dom.spec.whatwg.org/#concept-element-attribute-has) a <a id="ref-for-style-attribute③"></a>style attribute, let <var>inline style</var> be a new empty <a id="ref-for-css-declaration-block①"></a>CSS declaration block with the <a id="ref-for-cssstyledeclaration-owner-node"></a>[owner node](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-owner-node) set to <var>target</var>.

    4.  Let <var>targeted properties</var> be the <a id="ref-for-ordered-set①"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of physical longhand properties that are a <a id="ref-for-target-property①⑥"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property) for at least one <a id="ref-for-animation-effect⑧③"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect) <a id="ref-for-animation-effect-associated-animation②①"></a>[associated](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) with <var>animation</var> whose <a id="ref-for-keyframe-effect-effect-target①⑧"></a>[effect target](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) is <var>target</var>.

    5.  For each property, <var>property</var>, in <var>targeted properties</var>:

        1.  Let <var>partialEffectStack</var> be a copy of the <a id="ref-for-effect-stack①④"></a>[effect stack](https://drafts.csswg.org/web-animations-1/#effect-stack) for <var>property</var> on <var>target</var>.

        2.  If <var>animation</var>’s <a id="ref-for-replace-state⑨"></a>[replace state](https://drafts.csswg.org/web-animations-1/#replace-state) is <a id="ref-for-removed-replace-state⑥"></a>[removed](https://drafts.csswg.org/web-animations-1/#removed-replace-state), add all <a id="ref-for-animation-effect⑧④"></a>[animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) <a id="ref-for-animation-effect-associated-animation②②"></a>[associated](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) with <var>animation</var> whose <a id="ref-for-keyframe-effect-effect-target①⑨"></a>[effect target](https://drafts.csswg.org/web-animations-1/#keyframe-effect-effect-target) is <var>target</var> and that include <var>property</var> as a <a id="ref-for-target-property①⑦"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property) to <var>partialEffectStack</var>.

        3.  Remove from <var>partialEffectStack</var> any <a id="ref-for-animation-effect⑧⑤"></a>[animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) whose <a id="ref-for-animation-effect-associated-animation②③"></a>[associated](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) <a id="ref-for-concept-animation⑦①"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation) has a higher <a id="ref-for-animation-composite-order②"></a>[composite order](https://drafts.csswg.org/web-animations-1/#animation-composite-order) than <var>animation</var>.

        4.  Let <var>effect value</var> be the result of calculating the result of <var>partialEffectStack</var> for <var>property</var> using <var>target</var>’s computed style (see [§ 5.4.3 Calculating the result of an effect stack](https://drafts.csswg.org/web-animations-1/#calculating-the-result-of-an-effect-stack)) and setting the <a id="ref-for-endpoint-inclusive-active-interval②"></a>[endpoint-inclusive active interval](https://drafts.csswg.org/web-animations-1/#endpoint-inclusive-active-interval) flag to true when calculating the animation effect phase (see [§ 4.6.6 Animation effect phases and states](https://drafts.csswg.org/web-animations-1/#animation-effect-phases-and-states)).

        5.  <a id="ref-for-set-a-css-declaration"></a>[Set a CSS declaration](https://drafts.csswg.org/cssom-1/#set-a-css-declaration) of <var>property</var> for <var>effect value</var> in <var>inline style</var>.

    6.  <a id="ref-for-update-style-attribute-for"></a>[Update style attribute for](https://drafts.csswg.org/cssom-1/#update-style-attribute-for) <var>inline style</var>.

#### <a id="the-animationplaystate-enumeration"></a>6.4.1. The <code><a id="ref-for-enumdef-animationplaystate②"></a>[AnimationPlayState](#enumdef-animationplaystate)</code> enumeration[](#the-animationplaystate-enumeration)

<a id="enumdef-animationplaystate"></a><a id="ref-for-dom-animationplaystate-idle"></a><a id="ref-for-dom-animationplaystate-running"></a><a id="ref-for-dom-animationplaystate-paused"></a><a id="ref-for-dom-animationplaystate-finished"></a>

``` text
enum AnimationPlayState { "idle", "running", "paused", "finished" };
```

<strong><a id="dom-animationplaystate-idle"></a><strong><code>idle</code></strong></strong>

Corresponds to the <a id="ref-for-play-state-idle⑧"></a>[idle play state](https://drafts.csswg.org/web-animations-1/#play-state-idle).

<strong><a id="dom-animationplaystate-running"></a><strong><code>running</code></strong></strong>

Corresponds to the <a id="ref-for-play-state-running⑤"></a>[running play state](https://drafts.csswg.org/web-animations-1/#play-state-running).

<strong><a id="dom-animationplaystate-paused"></a><strong><code>paused</code></strong></strong>

Corresponds to the <a id="ref-for-play-state-paused①⓪"></a>[paused play state](https://drafts.csswg.org/web-animations-1/#play-state-paused).

<strong><a id="dom-animationplaystate-finished"></a><strong><code>finished</code></strong></strong>

Corresponds to the <a id="ref-for-play-state-finished①⑥"></a>[finished play state](https://drafts.csswg.org/web-animations-1/#play-state-finished).

#### <a id="the-animationreplacestate-enumeration"></a>6.4.2. The <code><a id="ref-for-enumdef-animationreplacestate②"></a>[AnimationReplaceState](#enumdef-animationreplacestate)</code> enumeration[](#the-animationreplacestate-enumeration)

<a id="enumdef-animationreplacestate"></a><a id="dom-animationreplacestate-active"></a><a id="dom-animationreplacestate-removed"></a><a id="dom-animationreplacestate-persisted"></a>

``` text
enum AnimationReplaceState { "active", "removed", "persisted" };
```

<strong><a id="dom-animationreplacedstate-active"></a><strong><code>active</code></strong></strong>

Corresponds to the <a id="ref-for-active-replace-state②"></a>[active replace state](https://drafts.csswg.org/web-animations-1/#active-replace-state).

<strong><a id="dom-animationreplacedstate-removed"></a><strong><code>removed</code></strong></strong>

Corresponds to the <a id="ref-for-removed-replace-state⑦"></a>[removed replace state](https://drafts.csswg.org/web-animations-1/#removed-replace-state).

<strong><a id="dom-animationreplacedstate-persisted"></a><strong><code>persisted</code></strong></strong>

Corresponds to the <a id="ref-for-persisted-replace-state①"></a>[persisted replace state](https://drafts.csswg.org/web-animations-1/#persisted-replace-state).

## <a id="changes-since-last-publication"></a>10. Changes since last publication[](#changes-since-last-publication)

The following changes have been made since the [5 June 2023 Working Draft](https://www.w3.org/TR/2023/WD-web-animations-1-20230605/):

- Improved definitions and cross-linking for the Web Animations model, and reorganized [§ 4.6 Animation effects](https://drafts.csswg.org/web-animations-1/#animation-effects) to introduce concepts before the procedures in [§ 4.7 Calculating progress](https://drafts.csswg.org/web-animations-1/#core-animation-effect-calculations). ([PR 8965](https://github.com/w3c/csswg-drafts/pull/8965), [PR 8985](https://github.com/w3c/csswg-drafts/pull/8985))

- Renamed “timing function” to “<a id="ref-for-easing-function①⑧"></a>[easing function](https://drafts.csswg.org/css-easing-2/#easing-function)” throughout to match the terminology in [\[CSS-EASING-1\]](https://drafts.csswg.org/web-animations-1/#biblio-css-easing-1). ([PR 9218](https://github.com/w3c/csswg-drafts/pull/9218))

- Corrected the calculation of <a id="ref-for-active-time①⑦"></a>[active time](https://drafts.csswg.org/web-animations-1/#active-time) for an <a id="ref-for-animation-effect①①⑤"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect) in the <a id="ref-for-animation-effect-after-phase①①"></a>[after phase](https://drafts.csswg.org/web-animations-1/#animation-effect-after-phase) with a <a id="ref-for-fill-mode①⑨"></a>[fill mode](https://drafts.csswg.org/web-animations-1/#fill-mode) of <a id="ref-for-fill-mode-both⑥"></a>[both](https://drafts.csswg.org/web-animations-1/#fill-mode-both). ([Issue 11613](https://github.com/w3c/csswg-drafts/issues/11613))

- Simplified the implicit <a id="ref-for-keyframe⑦②"></a>[keyframes](https://drafts.csswg.org/web-animations-1/#keyframe) generated when calculating the <a id="ref-for-effect-value②④"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) of a <a id="ref-for-keyframe-effect④③"></a>[keyframe effect](https://drafts.csswg.org/web-animations-1/#keyframe-effect) to use the <a id="ref-for-underlying-value①⑥"></a>[underlying value](https://drafts.csswg.org/web-animations-1/#underlying-value) directly rather than adding a neutral value to it. ([Issue 2914](https://github.com/w3c/csswg-drafts/issues/2914))

- Clarified that the implicit <a id="ref-for-keyframe⑦③"></a>[keyframes](https://drafts.csswg.org/web-animations-1/#keyframe) generated when calculating the <a id="ref-for-effect-value②⑤"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) of a <a id="ref-for-keyframe-effect④④"></a>[keyframe effect](https://drafts.csswg.org/web-animations-1/#keyframe-effect) use the <a id="ref-for-linear-easing-function③"></a>[linear easing function](https://drafts.csswg.org/css-easing-2/#linear-easing-function). ([Issue 14492](https://github.com/w3c/csswg-drafts/issues/14492))

- Added annotations describing exceptions thrown by methods in the [§ 6 Programming interface](https://drafts.csswg.org/web-animations-1/#programming-interface). ([Issue 2050](https://github.com/w3c/csswg-drafts/issues/2050))

- Added a non-normative summary of how the <a id="ref-for-animation-start-time⑥②"></a>[start time](https://drafts.csswg.org/web-animations-1/#animation-start-time) and <a id="ref-for-animation-hold-time④⑦"></a>[hold time](https://drafts.csswg.org/web-animations-1/#animation-hold-time) of an <a id="ref-for-concept-animation⑧②"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation) determine its <a id="ref-for-animation-play-state①②"></a>[play state](https://drafts.csswg.org/web-animations-1/#animation-play-state). ([Issue 2047](https://github.com/w3c/csswg-drafts/issues/2047))

- Explicitly invoked the procedure to <a id="ref-for-update-an-animations-finished-state①④"></a>[update an animation’s finished state](https://drafts.csswg.org/web-animations-1/#update-an-animations-finished-state) when updating effect timing or setting the <a id="ref-for-playback-rate④⑦"></a>[playback rate](https://drafts.csswg.org/web-animations-1/#playback-rate), and clarified the flags used when updating the finished state. ([Issue 2555](https://github.com/w3c/csswg-drafts/issues/2555))

- Made <code><a id="ref-for-dom-animation-commitstyles②"></a>[commitStyles()](#dom-animation-commitstyles)</code> use endpoint-inclusive timing so it can commit the endpoint of a completed animation without requiring a forwards <a id="ref-for-fill-mode②⓪"></a>[fill mode](https://drafts.csswg.org/web-animations-1/#fill-mode). ([Issue 5394](https://github.com/w3c/csswg-drafts/issues/5394))

- Specified that updating <code><a id="ref-for-dom-effecttiming-duration⑨"></a>[duration](https://drafts.csswg.org/web-animations-1/#dom-effecttiming-duration)</code> with a string other than the lowercase <code>auto</code> throws a <code><a id="ref-for-exceptiondef-typeerror①⑥"></a>[TypeError](https://webidl.spec.whatwg.org/#exceptiondef-typeerror)</code>. ([Issue 14206](https://github.com/w3c/csswg-drafts/issues/14206))

- Clarified that array-form keyframes can contain <code>null</code> or <code>undefined</code> entries. ([PR 11594](https://github.com/w3c/csswg-drafts/pull/11594))

- Corrected the procedure to <a id="ref-for-process-a-keyframe-like-object②"></a>[process a keyframe-like object](https://drafts.csswg.org/web-animations-1/#process-a-keyframe-like-object) to return the dictionary defaults for <code>null</code> or <code>undefined</code> input before enumerating its properties. ([Issue 14113](https://github.com/w3c/csswg-drafts/issues/14113))

- Added the <code><a id="ref-for-dom-getanimationsoptions-pseudoelement③"></a>[pseudoElement](https://drafts.csswg.org/web-animations-1/#dom-getanimationsoptions-pseudoelement)</code> option to <code><a id="ref-for-dom-animatable-getanimations③"></a>[getAnimations()](https://drafts.csswg.org/web-animations-1/#dom-animatable-getanimations)</code> for requesting animations targeting a particular <a id="ref-for-pseudo-element⑨"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element). ([PR 11050](https://github.com/w3c/csswg-drafts/pull/11050))

- Removed the "Interaction with page display" section since it contradicted the relationship of time values to other parts of the platform and was not implemented.

The [changelog](https://github.com/w3c/csswg-drafts/commits/main/web-animations-1) provides a more detailed history.

### <a id="model-liveness"></a>6.13. Model liveness[](#model-liveness)

Changes made to any part of the model cause the entire timing model to be updated and any dependent style.

Unless otherwise stated, invoking the methods or constructors, or getting or setting the members of interfaces defined in the <a id="ref-for-web-animations-api①⓪"></a>[Web Animations API](https://drafts.csswg.org/web-animations-1/#web-animations-api) does <em>not</em> produce a <a id="ref-for-style-change-event③"></a>[style change event](https://drafts.csswg.org/css-transitions-1/#style-change-event).

Note: Other specifications that extend this specification are expected to refine the requirements on <a id="ref-for-style-change-event④"></a>[style change events](https://drafts.csswg.org/css-transitions-1/#style-change-event) by introducing circumstances where such events <em>are</em> triggered. For example, when the interfaces in this specification represent animations defined by CSS markup, many of their methods will need to trigger <a id="ref-for-style-change-event⑤"></a>style change events in order to reflect changes to the specified style.

<em>This section is non-normative</em>

Based on the above requirement and normative requirements elsewhere in this specification, the following invariants can be observed:

<strong>Changes made to the <a id="ref-for-web-animations-model①①"></a>[Web Animations model](https://drafts.csswg.org/web-animations-1/#web-animations-model) take effect immediately</strong>

For example, if the <code><a id="ref-for-keyframeeffect①⑥"></a>[KeyframeEffect](https://drafts.csswg.org/web-animations-1/#keyframeeffect)</code> associated with an <code><a id="ref-for-animation①⑥"></a>[Animation](#animation)</code> is <a id="ref-for-seek⑤"></a>[seeked](https://drafts.csswg.org/web-animations-1/#seek) (see [§ 4.5.4 Setting the current time of an animation](https://drafts.csswg.org/web-animations-1/#setting-the-current-time-of-an-animation)) via the <a id="ref-for-web-animations-api①①"></a>[Web Animations API](https://drafts.csswg.org/web-animations-1/#web-animations-api), the value returned when querying the animation’s <code>startTime</code> will reflect updated state of the model immediately.

<a id="example-f6015dd0"></a>

<strong>Example:</strong>

[](#example-f6015dd0)

``` text
// Initially animation.effect.getComputedTiming().localTime is 3000
animation.currentTime += 2000;
alert(animation.effect.getComputedTiming().localTime); // Displays "5000"
```

<strong>Querying the computed style of a property affected by animation returns the fully up-to-date state of the animation</strong>

For example, if the used style of an element is queried immediately after applying a new <code><a id="ref-for-animation①⑦"></a>[Animation](#animation)</code> to that element, the result of the new animation will be incorporated in the value returned.

<a id="example-ac9a1207"></a>

<strong>Example:</strong>

[](#example-ac9a1207)

``` text
// Set opacity to 0 immediately
elem.animate({ opacity: 0 }, { fill: 'forwards' });
alert(window.getComputedStyle(elem).opacity); // Displays "0"
```

<strong>Changes made within the same task are synchronized such that the whole set of changes is rendered together</strong>

As a result of changes to the model taking effect immediately combined with ECMAScript’s run-to-completion semantics, there should never be a situation where, for example, <em>only</em> the changes to specified style are rendered without applying animation.

<a id="example-4f302d5c"></a>

<strong>Example:</strong>

[](#example-4f302d5c)

``` text
// Fade the opacity with fallback for browsers that don't
// support Element.animate
elem.style.opacity = '0';
elem.animate([ { opacity: 1 }, { opacity: 0 } ], 500);
```

<strong>Note:</strong>

Note, however, that in the example above, a user agent may render a frame with <em>none</em> of the above changes applied. This might happen, for example, if rendering occurs in a separate process that is scheduled to run shortly after the above task completes but before the changes can be communicated to the process.

<strong>The value returned by the <code>currentTime</code> attribute of a <a id="ref-for-document-timeline⑥"></a>[document timeline](#document-timeline) will not change within a task</strong>

Due to the requirement on <a id="ref-for-timeline⑥①"></a>[timelines](https://drafts.csswg.org/web-animations-1/#timeline) to update their <a id="ref-for-timeline-current-time①④"></a>[current time](https://drafts.csswg.org/web-animations-1/#timeline-current-time) each time the <a id="ref-for-update-animations-and-send-events④"></a>[update animations and send events](https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events) procedure is run, querying the <code>currentTime</code> twice within a long block of code that is executed in the same script block will return the same value as shown in the following example.

<a id="example-16dafcc0"></a>

<strong>Example:</strong>

[](#example-16dafcc0)

``` text
var a = document.timeline.currentTime;
// ... many lines of code ...
var b = document.timeline.currentTime;
alert(b - a); // Displays 0
```

<strong>The time passed to a <code>requestAnimationFrame</code> callback will be equal to <code>document.timeline.currentTime</code></strong>

Since HTML’s <a id="ref-for-event-loop-processing-model"></a>[event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model) defines that the procedure to <a id="ref-for-update-animations-and-send-events⑤"></a>[update animations and send events](https://drafts.csswg.org/web-animations-1/#update-animations-and-send-events) is performed prior to <a id="ref-for-run-the-animation-frame-callbacks"></a>[running animation frame callbacks](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html.html#run-the-animation-frame-callbacks), and since the time passed to such callbacks is the same <var>now</var> timestamp is passed to both procedures, the <a id="ref-for-timeline-current-time①⑤"></a>[current time](https://drafts.csswg.org/web-animations-1/#timeline-current-time) of a the <a id="ref-for-document-default-document-timeline①⓪"></a>[default document timeline](#document-default-document-timeline) should match the time passed to <code>requestAnimationFrame</code>.

<a id="example-a0637e5c"></a>

<strong>Example:</strong>

[](#example-a0637e5c)

``` text
window.requestAnimationFrame(function(now) {
  // Displays 0
  alert(now - document.timeline.currentTime);
});
```

<strong>Calling methods from this programming interface will generally <em>not</em> cause transitions to be triggered</strong>

Consider the following example:

<a id="example-a8ba32df"></a>

<strong>Example:</strong>

[](#example-a8ba32df)

``` text
// Setup transition start point
div.style.opacity = '1';
getComputedStyle(div).opacity;

// Setup transition end point
div.style.transition = 'opacity 1s';
div.style.opacity = '0';

// Fire an animation
div.animate({ opacity: [0.5, 1] }, 500);

// Wait for the transition to end -- the following will never be called!
div.addEventListener('transitionend', () => {
  console.log('transitionend');
});
```

In this case, calling <code><a id="ref-for-dom-animatable-animate③"></a>[animate()](https://drafts.csswg.org/web-animations-1/#dom-animatable-animate)</code> will <em>not</em> trigger a <a id="ref-for-style-change-event⑥"></a>[style change event](https://drafts.csswg.org/css-transitions-1/#style-change-event). As a result, the pending style change will be processed at the same time as the style change resulting from the new animation. Since the animation style will override the <a id="ref-for-before-change-style"></a>[before-change style](https://drafts.csswg.org/css-transitions-1/#before-change-style) and the <a id="ref-for-after-change-style"></a>[after-change style](https://drafts.csswg.org/css-transitions-1/#after-change-style), no transition will be generated and the event handler for the <a id="ref-for-transitionend"></a>[transitionend](https://drafts.csswg.org/css-transitions/#transitionend) event will never be called.
