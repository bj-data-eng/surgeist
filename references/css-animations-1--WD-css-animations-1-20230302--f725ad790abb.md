Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Animations Level 1](https://www.w3.org/TR/2023/WD-css-animations-1-20230302/).

Original copyright notice: Copyright © 2023 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Animations Level 1

Source snapshot: https://www.w3.org/TR/2023/WD-css-animations-1-20230302/

Snapshot SHA-256: f725ad790abbf87f7f53b00f383b8c206a95decb27cd5c257a9725646abb4d98

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 11 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Animations Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2023 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes a way for authors to animate the values of CSS properties over time, using keyframes. The behavior of these keyframe animations can be controlled by specifying their duration, number of repeats, and repeating behavior.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-animations” in the title, like this: “\[css-animations\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-animations%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative</em>

CSS Transitions [\[CSS3-TRANSITIONS\]](#biblio-css3-transitions) provide a way to interpolate CSS property values when they change as a result of underlying property changes. This provides an easy way to do simple animation, but the start and end states of the animation are controlled by the existing property values, and transitions provide little control to the author on how the animation progresses.

This proposal introduces defined animations, in which the author can specify the changes in CSS properties over time as a set of keyframes. Animations are similar to transitions in that they change the presentational value of CSS properties over time. The principal difference is that while transitions trigger implicitly when property values change, animations are explicitly executed when the animation properties are applied. Because of this, animations require explicit values for the properties being animated. These values are specified using animation keyframes, described below.

Many aspects of the animation can be controlled, including how many times the animation iterates, whether or not it alternates between the begin and end values, and whether or not the animation should be running or paused. An animation can also delay its start time.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="animations"></a>2.  Animations

CSS Animations affect computed property values. This effect happens by adding a specified value to the CSS cascade ([\[CSS3CASCADE\]](#biblio-css3cascade)) (at the level for CSS Animations) that will produce the correct computed value for the current state of the animation. As defined in \[CSS3CASCADE\], animations override all normal rules, but are overridden by !important rules.

<a id="ref-for-propdef-animation-name"></a>

If at some point in time there are multiple animations specifying behavior for the same property, the animation which occurs last in the value of [animation-name](#propdef-animation-name) will override the other animations at that point.

<a id="ref-for-propdef-animation-name①"></a>

<a id="ref-for-propdef-animation-fill-mode"></a>

An animation does not affect the computed value before the application of the animation (that is, when the [animation-name](#propdef-animation-name) property is set on an element) or after it is removed. Furthermore, typically an animation does not affect the computed value before the animation delay has expired or after the end of the animation, but may do so depending on the [animation-fill-mode](#propdef-animation-fill-mode) property.

While running, the animation computes the value of those properties it animates. Other values may take precedence over the animated value according to the CSS cascade ([\[CSS3CASCADE\]](#biblio-css3cascade)).

<a id="ref-for-propdef-animation-fill-mode①"></a>

<a id="ref-for-valdef-animation-fill-mode-forwards"></a>

<a id="ref-for-valdef-animation-fill-mode-both"></a>

<a id="ref-for-propdef-will-change"></a>

While an animation is applied but has not finished, or has finished but has an [animation-fill-mode](#propdef-animation-fill-mode) of [forwards](#valdef-animation-fill-mode-forwards) or [both](#valdef-animation-fill-mode-both), the user agent must act as if the [will-change](https://www.w3.org/TR/css-will-change-1/#propdef-will-change) property ([\[css-will-change-1\]](#biblio-css-will-change-1)) on the element additionally includes all the properties animated by the animation.

The start time of an animation is the time at which the style applying the animation and the corresponding @keyframes rule are both resolved. If an animation is specified for an element but the corresponding @keyframes rule does not yet exist, the animation cannot start; the animation will start from the beginning as soon as a matching @keyframes rule can be resolved. An animation specified by dynamically modifying the element’s style will start when this style is resolved; that may be immediately in the case of a pseudo style rule such as hover, or may be when the scripting engine returns control to the browser (in the case of style applied by script). Note that dynamically updating keyframe style rules does not start or re-start an animation.

<a id="ref-for-propdef-animation-name②"></a>

<a id="ref-for-propdef-animation-delay"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationend"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationstart"></a>

An animation applies to an element if its name appears as one of the identifiers in the computed value of the [animation-name](#propdef-animation-name) property and the animation uses a valid @keyframes rule. Once an animation has started it continues until it ends or the <a id="ref-for-propdef-animation-name③"></a>animation-name is removed. Changes to the values of animation properties while the animation is running apply as if the animation had those values from when it began. For example, shortening the [animation-delay](#propdef-animation-delay) may cause the animation to jump forwards or even finish immediately and dispatch an <code><a href="#eventdef-globaleventhandlers-animationend">animationend</a></code> event. Conversely, extending the <a id="ref-for-propdef-animation-delay①"></a>animation-delay may cause an animation to re-start and dispatch an <code><a href="#eventdef-globaleventhandlers-animationstart">animationstart</a></code> event.

<a id="ref-for-propdef-animation-name④"></a>

The same @keyframes rule name may be repeated within an [animation-name](#propdef-animation-name). Changes to the <a id="ref-for-propdef-animation-name⑤"></a>animation-name update existing animations by iterating over the new list of animations from last to first, and, for each animation, finding the <em>last</em> matching animation in the list of existing animations. If a match is found, the existing animation is updated using the animation properties corresponding to its position in the new list of animations, whilst maintaining its current playback time as described above. The matching animation is removed from the existing list of animations such that it will not match twice. If a match is not found, a new animation is created. As a result, updating <a id="ref-for-propdef-animation-name⑥"></a>animation-name from ‘a’ to ‘a, a’ will cause the existing animation for ‘a’ to become the <em>second</em> animation in the list and a new animation will be created for the first item in the list.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4e34d7ba"></a>
>
> ```text
> div {
>   animation-name: diagonal-slide;
>   animation-duration: 5s;
>   animation-iteration-count: 10;
> }
> 
> @keyframes diagonal-slide {
> 
>   from {
>     left: 0;
>     top: 0;
>   }
> 
>   to {
>     left: 100px;
>     top: 100px;
>   }
> 
> }
> ```
>
> This will produce an animation that moves an element from (0, 0) to (100px, 100px) over five seconds and repeats itself nine times (for a total of ten iterations).

<a id="ref-for-propdef-display"></a>

<a id="ref-for-valdef-display-none"></a>

<a id="ref-for-propdef-animation-name⑦"></a>

Setting the [display](https://drafts.csswg.org/css2/#propdef-display) property to [none](https://www.w3.org/TR/css-display-3/#valdef-display-none) will terminate any running animation applied to the element and its descendants. If an element has a <a id="ref-for-propdef-display①"></a>display of <a id="ref-for-valdef-display-none①"></a>none, updating <a id="ref-for-propdef-display②"></a>display to a value other than <a id="ref-for-valdef-display-none②"></a>none will start all animations applied to the element by the [animation-name](#propdef-animation-name) property, as well as all animations applied to descendants with <a id="ref-for-propdef-display③"></a>display other than <a id="ref-for-valdef-display-none③"></a>none.

While authors can use animations to create dynamically changing content, dynamically changing content can lead to seizures in some users. For information on how to avoid content that can lead to seizures, see Guideline 2.3: Seizures: Do not design content in a way that is known to cause seizures ([\[WCAG20\]](#biblio-wcag20)).

Implementations may ignore animations when the rendering medium is not interactive e.g. when printed. A future version of this specification may define how to render animations for these media.

## <a id="keyframes"></a>3.  Keyframes

Keyframes are used to specify the values for the animating properties at various points during the animation. The keyframes specify the behavior of one cycle of the animation; the animation may iterate zero or more times.

Keyframes are specified using the <a id="at-ruledef-keyframes"></a>@keyframes at-rule, defined as follows:

<a id="ref-for-typedef-keyframes-name"></a>

<a id="ref-for-typedef-rule-list"></a>

<a id="typedef-keyframes-name"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-string-value"></a>

<a id="typedef-keyframe-block"></a>

<a id="ref-for-typedef-keyframe-selector"></a>

<a id="ref-for-typedef-declaration-list"></a>

<a id="typedef-keyframe-selector"></a>

<a id="ref-for-percentage-value"></a>

```text
@keyframes = @keyframes <keyframes-name> { <rule-list> }

<keyframes-name> = <custom-ident> | <string>

<keyframe-block> = <keyframe-selector># { <declaration-list> }

<keyframe-selector> = from | to | <percentage [0,100]>
```
<a id="ref-for-typedef-rule-list①"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-typedef-keyframe-block"></a>

The [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list) inside of [@keyframes](#at-ruledef-keyframes) can only contain [\<keyframe-block\>](#typedef-keyframe-block) rules.

<a id="ref-for-typedef-declaration-list①"></a>

<a id="ref-for-typedef-keyframe-block①"></a>

<a id="ref-for-propdef-animation-timing-function"></a>

The [\<declaration-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-list) inside of [\<keyframe-block\>](#typedef-keyframe-block) accepts any CSS property except those defined in this specification, but <em>does</em> accept the [animation-timing-function](#propdef-animation-timing-function) property and interprets it specially. None of the properties interact with the cascade (so using !important on them is invalid and will cause the property to be ignored).

<a id="ref-for-at-ruledef-keyframes①"></a>

<a id="ref-for-identifier-value①"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-valdef-animation-name-none"></a>

A [@keyframes](#at-ruledef-keyframes) block has a name given by the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) or [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) in its prelude. The two syntaxes are equivalent in functionality; the name is the value of the ident or string. As normal for <a id="ref-for-identifier-value②"></a>\<custom-ident\>s and <a id="ref-for-string-value②"></a>\<string\>s, the names are fully case-sensitive; two names are equal only if they are codepoint-by-codepoint equal. The <a id="ref-for-identifier-value③"></a>\<custom-ident\> additionally excludes the [none](#valdef-animation-name-none) keyword.

<a id="ref-for-at-ruledef-keyframes②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d0936079"></a> For example, the following two [@keyframes](#at-ruledef-keyframes) rules have the same name, so the first will be ignored:
>
> ```css
> @keyframes foo { /* ... */ }
> @keyframes "foo" { /* ... */ }
> ```
>
> <a id="ref-for-at-ruledef-keyframes③"></a>
>
> On the other hand, the following [@keyframes](#at-ruledef-keyframes) rule’s name is <em>different</em> from the previous two rules:
>
> ```css
> @keyframes FOO { /* ... */ }
> ```
>
> <a id="ref-for-at-ruledef-keyframes④"></a>
>
> <a id="ref-for-identifier-value④"></a>
>
> The following [@keyframes](#at-ruledef-keyframes) rules are invalid because they use disallowed [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) values:
>
> ```css
> @keyframes initial { /* ... */ }
> @keyframes None { /* ... */ }
> ```
>
> <a id="ref-for-string-value③"></a>
>
> However, those names <em>can</em> be specified with a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value), so the following are both <em>valid</em>:
>
> ```css
> @keyframes "initial" { /* ... */ }
> @keyframes "None" { /* ... */ }
> ```
<a id="ref-for-typedef-keyframe-selector①"></a>

<a id="ref-for-typedef-keyframe-block②"></a>

<a id="ref-for-valdef-shape-to"></a>

The [\<keyframe-selector\>](#typedef-keyframe-selector) for a [\<keyframe-block\>](#typedef-keyframe-block) consists of a comma-separated list of percentage values or the keywords from or [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to). The selector is used to specify the percentage along the duration of the animation that the keyframe represents. The keyframe itself is specified by the block of property values declared on the selector. The keyword from is equivalent to the value 0%. The keyword <a id="ref-for-valdef-shape-to①"></a>to is equivalent to the value 100%. Values less than 0% or higher than 100% are invalid and cause their <a id="ref-for-typedef-keyframe-block③"></a>\<keyframe-block\> to be ignored.

<strong data-conversion-semantic="note">Note:</strong> Note that the percentage unit specifier must be used on percentage values. Therefore, 0 is an invalid keyframe selector.

<a id="ref-for-valdef-shape-to②"></a>

If a 0% or from keyframe is not specified, then the user agent constructs a 0% keyframe using the computed values of the properties being animated. If a 100% or [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) keyframe is not specified, then the user agent constructs a 100% keyframe using the computed values of the properties being animated.

<a id="ref-for-typedef-keyframe-block④"></a>

<a id="ref-for-propdef-animation-timing-function①"></a>

The [\<keyframe-block\>](#typedef-keyframe-block) contains properties and values. The properties defined by this specification are ignored in these rules, with the exception of [animation-timing-function](#propdef-animation-timing-function), the behavior of which is described below. In addition, properties qualified with !important are invalid and ignored.

<a id="ref-for-at-ruledef-keyframes⑤"></a>

If multiple [@keyframes](#at-ruledef-keyframes) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b784185b"></a>
>
> ```text
> div {
>   animation-name: slide-right;
>   animation-duration: 2s;
> }
> 
> @keyframes slide-right {
> 
>   from {
>     margin-left: 0px;
>   }
> 
>   50% {
>     margin-left: 110px;
>     opacity: 1;
>   }
> 
>   50% {
>     opacity: 0.9;
>   }
> 
>   to {
>     margin-left: 200px;
>   }
> 
> }
> ```
>
> The two 50% rules from above can also be combined into an equivalent single rule as illustrated below:
>
> ```text
> @keyframes slide-right {
> 
>   from {
>     margin-left: 0px;
>   }
> 
>   50% {
>     margin-left: 110px;
>     opacity: 0.9;
>   }
> 
>   to {
>     margin-left: 200px;
>   }
> 
> }
> ```
<a id="ref-for-at-ruledef-keyframes⑥"></a>

To determine the set of keyframes, all of the values in the selectors are sorted in increasing order by time. The rules within the [@keyframes](#at-ruledef-keyframes) rule then cascade; the properties of a keyframe may thus derive from more than one <a id="ref-for-at-ruledef-keyframes⑦"></a>@keyframes rule with the same selector value.

If a property is not specified for a keyframe, or is specified but invalid, the animation of that property proceeds as if that keyframe did not exist. Conceptually, it is as if a set of keyframes is constructed for each property that is present in any of the keyframes, and an animation is run independently for each property.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-10d90bd9"></a>
>
> ```text
> @keyframes wobble {
>   0% {
>     left: 100px;
>   }
> 
>   40% {
>     left: 150px;
>   }
> 
>   60% {
>     left: 75px;
>   }
> 
>   100% {
>     left: 100px;
>   }
> }
> ```
>
> <a id="ref-for-propdef-left"></a>
>
> Four keyframes are specified for the animation named "wobble". In the first keyframe, shown at the beginning of the animation cycle, the value of the [left](https://www.w3.org/TR/css-position-3/#propdef-left) property being animated is 100px. By 40% of the animation duration, <a id="ref-for-propdef-left①"></a>left has animated to 150px. At 60% of the animation duration, <a id="ref-for-propdef-left②"></a>left has animated back to 75px. At the end of the animation cycle, the value of <a id="ref-for-propdef-left③"></a>left has returned to 100px. The diagram below shows the state of the animation if it were given a duration of 10s.
>
> ![](https://www.w3.org/TR/2023/WD-css-animations-1-20230302/images/animation1.png)
>
> Animation states specified by keyframes

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-73aacf21"></a> This specification needs to define how the value is determined from the keyframes, like the section on [Application of transitions](https://drafts.csswg.org/css-transitions/#application) does for CSS Transitions.

### <a id="timing-functions"></a>3.1.  Timing functions for keyframes

A keyframe style rule may also declare the timing function that is to be used as the animation moves to the next keyframe.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a9d2ebec"></a>
>
> ```text
> @keyframes bounce {
> 
>   from {
>     top: 100px;
>     animation-timing-function: ease-out;
>   }
> 
>   25% {
>     top: 50px;
>     animation-timing-function: ease-in;
>   }
> 
>   50% {
>     top: 100px;
>     animation-timing-function: ease-out;
>   }
> 
>   75% {
>     top: 75px;
>     animation-timing-function: ease-in;
>   }
> 
>   to {
>     top: 100px;
>   }
> 
> }
> ```
>
> Five keyframes are specified for the animation named "bounce". Between the first and second keyframe (i.e., between 0% and 25%) an ease-out timing function is used. Between the second and third keyframe (i.e., between 25% and 50%) an ease-in timing function is used. And so on. The effect will appear as an element that moves up the page 50px, slowing down as it reaches its highest point then speeding up as it falls back to 100px. The second half of the animation behaves in a similar manner, but only moves the element 25px up the page.

<a id="ref-for-valdef-shape-to③"></a>

A timing function specified on the [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) or 100% keyframe is ignored.

<a id="ref-for-propdef-animation-timing-function②"></a>

See the [animation-timing-function](#propdef-animation-timing-function) property for more information.

<a id="ref-for-propdef-animation-name⑧"></a>

### <a id="animation-name"></a>3.2.  The [animation-name](#propdef-animation-name) property

<a id="ref-for-propdef-animation-name⑨"></a>

The [animation-name](#propdef-animation-name) property defines a list of animations that apply. Each name is used to select the keyframe at-rule that provides the property values for the animation. If the name does not match any keyframe at-rule, there are no properties to be animated and the animation will not execute. Furthermore, if the animation name is `none` then there will be no animation. This can be used to override any animations coming from the cascade. If multiple animations are attempting to modify the same property, then the animation closest to the end of the list of names wins.

<a id="ref-for-propdef-animation-name①⓪"></a>

Each animation listed by name should have a corresponding value for the other animation properties listed below. If the lists of values for the other animation properties do not have the same length, the length of the [animation-name](#propdef-animation-name) list determines the number of items in each list examined when starting animations. The lists are matched up from the first value: excess values at the end are not used. If one of the other properties doesn’t have enough comma-separated values to match the number of values of <a id="ref-for-propdef-animation-name①①"></a>animation-name, the UA must calculate its used value by repeating the list of values until there are enough. This truncation or repetition does not affect the computed value.

<a id="ref-for-propdef-background-image"></a>

<a id="ref-for-propdef-animation-name①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is analogous to the behavior of the background-\* properties, with [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) analogous to [animation-name](#propdef-animation-name).

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-name"></a>animation-name

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-typedef-keyframes-name①"></a>

<a id="ref-for-comb-one"></a>

\[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<keyframes-name\>](#typedef-keyframes-name) \][\#](https://www.w3.org/TR/css-values-4/#mult-comma)

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-animation-name-none①"></a>

<a id="ref-for-css-css-identifier"></a>

list, each item either a case-sensitive [css identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier) or the keyword [none](#valdef-animation-name-none)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-animation-name①③"></a>

The values of [animation-name](#propdef-animation-name) have the following meanings:

<a id="valdef-animation-name-none"></a>none

No keyframes are specified at all, so there will be no animation. Any other animations properties specified for this animation have no effect.

<a id="ref-for-typedef-keyframes-name②"></a>

<a id="valdef-animation-name-keyframes-name"></a>[\<keyframes-name\>](#typedef-keyframes-name)

<a id="ref-for-at-ruledef-keyframes⑧"></a>

<a id="ref-for-typedef-keyframes-name③"></a>

The animation will use the keyframes with the name specified by the [\<keyframes-name\>](#typedef-keyframes-name), if they exist. If no [@keyframes](#at-ruledef-keyframes) rule with that name exists, there is no animation.

<a id="ref-for-propdef-animation-duration"></a>

### <a id="animation-duration"></a>3.3.  The [animation-duration](#propdef-animation-duration) property

<a id="ref-for-propdef-animation-duration①"></a>

The [animation-duration](#propdef-animation-duration) property defines duration of a single animation cycle.

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-duration"></a>animation-duration

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma①"></a>

<a id="ref-for-time-value"></a>

[\<time \[0s,∞\]\>](https://www.w3.org/TR/css-values-3/#time-value)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0s

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

list, each item a duration

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-time-value①"></a>

<a id="valdef-animation-duration-time-0s"></a>[\<time \[0s,∞\]\>](https://www.w3.org/TR/css-values-3/#time-value)

<a id="ref-for-time-value②"></a>

The [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) specifies the length of time that an animation takes to complete one cycle. A negative <a id="ref-for-time-value③"></a>\<time\> is invalid.

<a id="ref-for-time-value④"></a>

<a id="ref-for-propdef-animation-fill-mode②"></a>

<a id="ref-for-valdef-animation-fill-mode-backwards"></a>

<a id="ref-for-valdef-animation-fill-mode-both①"></a>

<a id="ref-for-propdef-animation-direction"></a>

<a id="ref-for-propdef-animation-delay②"></a>

<a id="ref-for-valdef-animation-fill-mode-forwards①"></a>

<a id="ref-for-valdef-animation-fill-mode-none"></a>

If the [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) is 0s, like the initial value, the keyframes of the animation have no effect, but the animation itself still occurs instantaneously. Specifically, start and end events are fired; if [animation-fill-mode](#propdef-animation-fill-mode) is set to [backwards](#valdef-animation-fill-mode-backwards) or [both](#valdef-animation-fill-mode-both), the first frame of the animation, as defined by [animation-direction](#propdef-animation-direction), will be displayed during the [animation-delay](#propdef-animation-delay). After the <a id="ref-for-propdef-animation-delay③"></a>animation-delay the last frame of the animation, as defined by <a id="ref-for-propdef-animation-direction①"></a>animation-direction, will be displayed if <a id="ref-for-propdef-animation-fill-mode③"></a>animation-fill-mode is set to [forwards](#valdef-animation-fill-mode-forwards) or <a id="ref-for-valdef-animation-fill-mode-both②"></a>both. If <a id="ref-for-propdef-animation-fill-mode④"></a>animation-fill-mode is set to [none](#valdef-animation-fill-mode-none) the animation will have no visible effect.

<a id="ref-for-propdef-animation-timing-function③"></a>

### <a id="animation-timing-function"></a>3.4.  The [animation-timing-function](#propdef-animation-timing-function) property

<a id="ref-for-propdef-animation-timing-function④"></a>

The [animation-timing-function](#propdef-animation-timing-function) property describes how the animation will progress between each pair of keyframes. Timing functions are defined in the separate CSS Easing Functions module [\[css-easing-1\]](#biblio-css-easing-1).

<a id="ref-for-input-progress-value"></a>

<a id="ref-for-propdef-animation-direction②"></a>

The [input progress value](https://www.w3.org/TR/css-easing-1/#input-progress-value) used is the percentage of the time elapsed between the current keyframe and the next keyframe <em>after</em> incorporating the effect of the [animation-direction](#propdef-animation-direction) property.

<a id="ref-for-propdef-animation-delay④"></a>

<a id="ref-for-propdef-animation-timing-function⑤"></a>

During the [animation-delay](#propdef-animation-delay), the [animation-timing-function](#propdef-animation-timing-function) is not applied.

<a id="ref-for-step-easing-function"></a>

<a id="ref-for-step-position"></a>

<a id="ref-for-valdef-steps-start"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition is necessary because otherwise a [step easing function](https://www.w3.org/TR/css-easing-1/#step-easing-function) with a [step position](https://www.w3.org/TR/css-easing-1/#step-position) of [start](https://www.w3.org/TR/css-easing-1/#valdef-steps-start) would produce a backwards fill equal to the top of the first step in the function.

<a id="ref-for-output-progress-value"></a>

The [output progress value](https://www.w3.org/TR/css-easing-1/#output-progress-value) is used as the <var>p</var> value when interpolating the property values between the current and next keyframe.

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-timing-function"></a>animation-timing-function

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-typedef-easing-function"></a>

[\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

ease

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-easing-function①"></a>

list, each item a computed [\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-animation-timing-function⑥"></a>

When specified in a keyframe, [animation-timing-function](#propdef-animation-timing-function) defines the progression of the animation between the current keyframe and the next keyframe for the animating property in sorted keyframe selector order (which may be an implicit 100% keyframe).

<a id="ref-for-propdef-animation-iteration-count"></a>

### <a id="animation-iteration-count"></a>3.5.  The [animation-iteration-count](#propdef-animation-iteration-count) property

<a id="ref-for-propdef-animation-iteration-count①"></a>

<a id="ref-for-propdef-animation-direction③"></a>

<a id="ref-for-valdef-animation-direction-alternate"></a>

The [animation-iteration-count](#propdef-animation-iteration-count) property specifies the number of times an animation cycle is played. The initial value is 1, meaning the animation will play from beginning to end once. This property is often used in conjunction with an [animation-direction](#propdef-animation-direction) value of [alternate](#valdef-animation-direction-alternate), which will cause the animation to play in reverse on alternate cycles.

The time window during which the animation is active (`duration` x `iteration-count`) is known as the <a id="active-duration"></a>active duration.

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-iteration-count"></a>animation-iteration-count

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-typedef-single-animation-iteration-count"></a>

[\<single-animation-iteration-count\>](#typedef-single-animation-iteration-count)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

1

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-animation-iteration-count-infinite"></a>

list, each item either a number or the keyword [infinite](#valdef-animation-iteration-count-infinite)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-comb-one①"></a>

<a id="ref-for-number-value"></a>

<a id="typedef-single-animation-iteration-count"></a>\<single-animation-iteration-count\> = infinite [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="valdef-animation-iteration-count-infinite"></a>infinite

The animation will repeat forever.

<a id="ref-for-number-value①"></a>

<a id="valdef-animation-iteration-count-number-0"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

The animation will repeat the specified number of times. If the number is not an integer, the animation will end partway through its last cycle. Negative numbers are invalid.

<a id="ref-for-propdef-animation-duration②"></a>

A value of 0 is valid and, similar to an [animation-duration](#propdef-animation-duration) of 0s, causes the animation to occur instantaneously.

<a id="ref-for-propdef-animation-iteration-count②"></a>

<a id="ref-for-valdef-animation-iteration-count-infinite①"></a>

If the animation has a duration of 0s, it will occur instantaneously for any valid value of [animation-iteration-count](#propdef-animation-iteration-count), including [infinite](#valdef-animation-iteration-count-infinite).

<a id="ref-for-propdef-animation-direction④"></a>

### <a id="animation-direction"></a>3.6.  The [animation-direction](#propdef-animation-direction) property

<a id="ref-for-propdef-animation-direction⑤"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-out"></a>

The [animation-direction](#propdef-animation-direction) property defines whether or not the animation should play in reverse on some or all cycles. When an animation is played in reverse the timing functions are also reversed. For example, when played in reverse an [ease-in](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-easing-function-ease-in) animation would appear to be an [ease-out](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-easing-function-ease-out) animation.

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-direction"></a>animation-direction

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma④"></a>

<a id="ref-for-typedef-single-animation-direction"></a>

[\<single-animation-direction\>](#typedef-single-animation-direction)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-comb-one②"></a>

<a id="typedef-single-animation-direction"></a>\<single-animation-direction\> = normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) reverse <a id="ref-for-comb-one③"></a>\| alternate <a id="ref-for-comb-one④"></a>\| alternate-reverse

<a id="valdef-animation-direction-normal"></a>normal  
All iterations of the animation are played as specified.

<a id="valdef-animation-direction-reverse"></a>reverse  
All iterations of the animation are played in the reverse direction from the way they were specified.

<a id="valdef-animation-direction-alternate"></a>alternate  
The animation cycle iterations that are odd counts are played in the normal direction, and the animation cycle iterations that are even counts are played in a reverse direction.

<a id="valdef-animation-direction-alternate-reverse"></a>alternate-reverse  
The animation cycle iterations that are odd counts are played in the reverse direction, and the animation cycle iterations that are even counts are played in a normal direction.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the purpose of determining whether an iteration is even or odd, iterations start counting from 1.

<a id="ref-for-propdef-animation-play-state"></a>

### <a id="animation-play-state"></a>3.7.  The [animation-play-state](#propdef-animation-play-state) property

<a id="ref-for-propdef-animation-play-state①"></a>

The [animation-play-state](#propdef-animation-play-state) property defines whether the animation is running or paused.

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-play-state"></a>animation-play-state

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma⑤"></a>

<a id="ref-for-typedef-single-animation-play-state"></a>

[\<single-animation-play-state\>](#typedef-single-animation-play-state)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

running

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-comb-one⑤"></a>

<a id="typedef-single-animation-play-state"></a>\<single-animation-play-state\> = running [\|](https://www.w3.org/TR/css-values-4/#comb-one) paused

<a id="valdef-animation-play-state-running"></a>running  
<a id="ref-for-valdef-animation-play-state-running"></a>

While this property is set to [running](#valdef-animation-play-state-running), the animation proceeds as normal.

<a id="valdef-animation-play-state-paused"></a>paused  
<a id="ref-for-valdef-animation-play-state-running①"></a>

<a id="ref-for-valdef-animation-play-state-paused"></a>

While this property is set to [paused](#valdef-animation-play-state-paused), the animation is paused. The animation continues to apply to the element with the progress it had made before being paused. When unpaused (set back to [running](#valdef-animation-play-state-running)), it restarts from where it left off, as if the "clock" that controls the animation had stopped and started again.

<a id="ref-for-valdef-animation-play-state-paused①"></a>

<a id="ref-for-propdef-animation-play-state②"></a>

<a id="ref-for-valdef-animation-play-state-running②"></a>

If the property is set to [paused](#valdef-animation-play-state-paused) during the delay phase of the animation, the delay clock is also paused and resumes as soon as [animation-play-state](#propdef-animation-play-state) is set back to [running](#valdef-animation-play-state-running).

<a id="ref-for-propdef-animation-delay⑤"></a>

### <a id="animation-delay"></a>3.8.  The [animation-delay](#propdef-animation-delay) property

<a id="ref-for-propdef-animation-delay⑥"></a>

The [animation-delay](#propdef-animation-delay) property defines when the animation will start. It allows an animation to begin execution some time after it is applied, or to appear to have begun execution some time <em>before</em> it is applied.

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-delay"></a>animation-delay

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma⑥"></a>

<a id="ref-for-time-value⑤"></a>

[\<time\>](https://www.w3.org/TR/css-values-3/#time-value)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0s

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

list, each item a duration

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-time-value⑥"></a>

<a id="valdef-animation-delay-time"></a>[\<time\>](https://www.w3.org/TR/css-values-3/#time-value)

<a id="ref-for-time-value⑦"></a>

The [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) defines how long of a delay there is between the start of the animation (when the animation is applied to the element via these properties) and when it begins executing. A delay of 0s (the initial value) means that the animation will execute as soon as it is applied.

A negative delay is <strong>valid</strong>. Similar to a delay of 0s, it means that the animation executes immediately, but is automatically progressed by the absolute value of the delay, as if the animation had started the specified time in the past, and so it appears to start partway through its [active duration](#animation-iteration-count). If an animation’s keyframes have an implied starting value, the values are taken from the time the animation starts, not some time in the past.

<a id="ref-for-propdef-animation-fill-mode⑤"></a>

### <a id="animation-fill-mode"></a>3.9.  The [animation-fill-mode](#propdef-animation-fill-mode) property

<a id="ref-for-propdef-animation-fill-mode⑥"></a>

<a id="ref-for-propdef-animation-name①④"></a>

<a id="ref-for-propdef-animation-delay⑦"></a>

<a id="ref-for-propdef-animation-duration③"></a>

<a id="ref-for-propdef-animation-iteration-count③"></a>

The [animation-fill-mode](#propdef-animation-fill-mode) property defines what values are applied by the animation outside the time it is executing. By default, an animation will not affect property values between the time it is applied (the [animation-name](#propdef-animation-name) property is set on an element) and the time it begins execution (which is determined by the [animation-delay](#propdef-animation-delay) property). Also, by default an animation does not affect property values after the animation ends (determined by the [animation-duration](#propdef-animation-duration) and [animation-iteration-count](#propdef-animation-iteration-count) properties). The <a id="ref-for-propdef-animation-fill-mode⑦"></a>animation-fill-mode property can override this behavior. Dynamic updates to the property will be reflected by property values as needed, whether during the animation delay or after the animation ends.

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation-fill-mode"></a>animation-fill-mode

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma⑦"></a>

<a id="ref-for-typedef-single-animation-fill-mode"></a>

[\<single-animation-fill-mode\>](#typedef-single-animation-fill-mode)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-comb-one⑥"></a>

<a id="typedef-single-animation-fill-mode"></a>\<single-animation-fill-mode\> = none [\|](https://www.w3.org/TR/css-values-4/#comb-one) forwards <a id="ref-for-comb-one⑦"></a>\| backwards <a id="ref-for-comb-one⑧"></a>\| both

<a id="valdef-animation-fill-mode-none"></a>none  
The animation has no effect when it is applied but not executing.

<a id="valdef-animation-fill-mode-forwards"></a>forwards  
<a id="ref-for-valdef-animation-fill-mode-backwards①"></a>

<a id="ref-for-propdef-animation-fill-mode⑧"></a>

<a id="ref-for-propdef-animation-iteration-count④"></a>

After the animation ends (as determined by its [animation-iteration-count](#propdef-animation-iteration-count)), the animation will apply the property values for the time the animation ended. When <a id="ref-for-propdef-animation-iteration-count⑤"></a>animation-iteration-count is an integer greater than zero, the values applied will be those for the end of the last completed iteration of the animation (rather than the values for the start of the iteration that would be next). When <a id="ref-for-propdef-animation-iteration-count⑥"></a>animation-iteration-count is zero, the values applied will be those that would start the first iteration (just as when [animation-fill-mode](#propdef-animation-fill-mode) is [backwards](#valdef-animation-fill-mode-backwards)).

<a id="valdef-animation-fill-mode-backwards"></a>backwards  
<a id="ref-for-valdef-animation-direction-alternate-reverse"></a>

<a id="ref-for-valdef-animation-direction-reverse"></a>

<a id="ref-for-valdef-shape-to④"></a>

<a id="ref-for-valdef-animation-direction-alternate①"></a>

<a id="ref-for-valdef-animation-direction-normal"></a>

<a id="ref-for-propdef-animation-direction⑥"></a>

<a id="ref-for-propdef-animation-delay⑧"></a>

During the period defined by [animation-delay](#propdef-animation-delay), the animation will apply the property values defined in the keyframe that will start the first iteration of the animation. These are either the values of the from keyframe (when [animation-direction](#propdef-animation-direction) is [normal](#valdef-animation-direction-normal) or [alternate](#valdef-animation-direction-alternate)) or those of the [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) keyframe (when <a id="ref-for-propdef-animation-direction⑦"></a>animation-direction is [reverse](#valdef-animation-direction-reverse) or [alternate-reverse](#valdef-animation-direction-alternate-reverse)).

<a id="valdef-animation-fill-mode-both"></a>both  
<a id="ref-for-valdef-animation-fill-mode-backwards②"></a>

<a id="ref-for-valdef-animation-fill-mode-forwards②"></a>

The effects of both [forwards](#valdef-animation-fill-mode-forwards) and [backwards](#valdef-animation-fill-mode-backwards) fill apply.

<a id="ref-for-propdef-animation"></a>

### <a id="animation"></a>3.10.  The [animation](#propdef-animation) shorthand property

<a id="ref-for-propdef-animation①"></a>

<a id="ref-for-propdef-animation-name①⑤"></a>

The [animation](#propdef-animation) shorthand property is a comma-separated list of animation definitions. Each item in the list gives one item of the value for all of the subproperties of the shorthand, which are known as the animation properties. (See the definition of [animation-name](#propdef-animation-name) for what happens when these properties have lists of different lengths, a problem that cannot occur when they are defined using only the <a id="ref-for-propdef-animation②"></a>animation shorthand.)

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-animation"></a>animation

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma⑧"></a>

<a id="ref-for-typedef-single-animation"></a>

[\<single-animation\>](#typedef-single-animation)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

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

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-time-value⑧"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-typedef-easing-function②"></a>

<a id="ref-for-typedef-single-animation-iteration-count①"></a>

<a id="ref-for-typedef-single-animation-direction①"></a>

<a id="ref-for-typedef-single-animation-fill-mode①"></a>

<a id="ref-for-typedef-single-animation-play-state①"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-keyframes-name④"></a>

<a id="typedef-single-animation"></a>\<single-animation\> = [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function) <a id="ref-for-comb-any①"></a>\|\| <a id="ref-for-time-value⑨"></a>\<time\> <a id="ref-for-comb-any②"></a>\|\| [\<single-animation-iteration-count\>](#typedef-single-animation-iteration-count) <a id="ref-for-comb-any③"></a>\|\| [\<single-animation-direction\>](#typedef-single-animation-direction) <a id="ref-for-comb-any④"></a>\|\| [\<single-animation-fill-mode\>](#typedef-single-animation-fill-mode) <a id="ref-for-comb-any⑤"></a>\|\| [\<single-animation-play-state\>](#typedef-single-animation-play-state) <a id="ref-for-comb-any⑥"></a>\|\| \[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<keyframes-name\>](#typedef-keyframes-name) \]

<a id="ref-for-typedef-single-animation①"></a>

<a id="ref-for-time-value①⓪"></a>

<a id="ref-for-propdef-animation-duration④"></a>

<a id="ref-for-propdef-animation-delay⑨"></a>

Note that order is important within each animation definition: the first value in each [\<single-animation\>](#typedef-single-animation) that can be parsed as a [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) is assigned to the [animation-duration](#propdef-animation-duration), and the second value in each <a id="ref-for-typedef-single-animation②"></a>\<single-animation\> that can be parsed as a <a id="ref-for-time-value①①"></a>\<time\> is assigned to [animation-delay](#propdef-animation-delay).

<a id="ref-for-typedef-keyframes-name⑤"></a>

<a id="ref-for-propdef-animation-name①⑥"></a>

Note that order is also important within each animation definition for distinguishing [\<keyframes-name\>](#typedef-keyframes-name) values from other keywords. When parsing, keywords that are valid for properties other than [animation-name](#propdef-animation-name) whose values were not found earlier in the shorthand must be accepted for those properties rather than for <a id="ref-for-propdef-animation-name①⑦"></a>animation-name. Furthermore, when serializing, default values of other properties must be output in at least the cases necessary to distinguish an <a id="ref-for-propdef-animation-name①⑧"></a>animation-name that could be a value of another property, and may be output in additional cases.

<a id="ref-for-propdef-animation③"></a>

<a id="ref-for-propdef-animation-fill-mode⑨"></a>

<a id="ref-for-valdef-animation-fill-mode-none①"></a>

<a id="ref-for-propdef-animation-name①⑨"></a>

<a id="ref-for-valdef-animation-fill-mode-backwards③"></a>

<a id="ref-for-valdef-animation-name-none②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f4782312"></a> For example, a value parsed from [animation: 3s none backwards](#propdef-animation) (where [animation-fill-mode](#propdef-animation-fill-mode) is [none](#valdef-animation-fill-mode-none) and [animation-name](#propdef-animation-name) is backwards) must not be serialized as <a id="ref-for-propdef-animation④"></a>animation: 3s backwards (where <a id="ref-for-propdef-animation-fill-mode①⓪"></a>animation-fill-mode is [backwards](#valdef-animation-fill-mode-backwards) and <a id="ref-for-propdef-animation-name②⓪"></a>animation-name is [none](#valdef-animation-name-none)).

## <a id="events"></a>4.  Animation Events

<a id="ref-for-propdef-animation-name②①"></a>

Several animation-related events are available through the DOM Event system. The start and end of an animation, and the end of each iteration of an animation, all generate DOM events. An element can have multiple properties being animated simultaneously. This can occur either with a single [animation-name](#propdef-animation-name) value with keyframes containing multiple properties, or with multiple <a id="ref-for-propdef-animation-name②②"></a>animation-name values. For the purposes of events, each <a id="ref-for-propdef-animation-name②③"></a>animation-name specifies a single animation. Therefore an event will be generated for each <a id="ref-for-propdef-animation-name②④"></a>animation-name value and not necessarily for each property being animated.

Any animation for which a valid keyframe rule is defined will run and generate events; this includes animations with empty keyframe rules.

<a id="ref-for-valdef-animation-play-state-paused②"></a>

The time the animation has been running is sent with each event generated. This allows the event handler to determine the current iteration of a looping animation or the current position of an alternating animation. This time does not include any time the animation was in the [paused](#valdef-animation-play-state-paused) play state.

### <a id="interface-animationevent"></a>4.1.  The `AnimationEvent` Interface

The `AnimationEvent` interface provides specific contextual information associated with Animation events.

#### <a id="interface-animationevent-idl"></a>4.1.1.  IDL Definition

<a id="ref-for-Exposed"></a>

<a id="animationevent"></a>

<a id="ref-for-event"></a>

<a id="ref-for-dom-animationevent-animationevent"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-animationevent-animationevent-type-animationeventinitdict-type"></a>

<a id="ref-for-dictdef-animationeventinit"></a>

<a id="dom-animationevent-animationevent-type-animationeventinitdict-animationeventinitdict"></a>

<a id="ref-for-cssomstring①"></a>

<a id="ref-for-dom-animationevent-animationname"></a>

<a id="ref-for-idl-double"></a>

<a id="ref-for-dom-animationevent-elapsedtime"></a>

<a id="ref-for-cssomstring②"></a>

<a id="ref-for-dom-animationevent-pseudoelement"></a>

<a id="dictdef-animationeventinit"></a>

<a id="ref-for-dictdef-eventinit"></a>

<a id="ref-for-cssomstring③"></a>

<a id="dom-animationeventinit-animationname"></a>

<a id="ref-for-idl-double①"></a>

<a id="dom-animationeventinit-elapsedtime"></a>

<a id="ref-for-cssomstring④"></a>

<a id="dom-animationeventinit-pseudoelement"></a>

```text
[Exposed=Window]
interface AnimationEvent : Event {
  constructor(CSSOMString type, optional AnimationEventInit animationEventInitDict = {});
  readonly attribute CSSOMString animationName;
  readonly attribute double elapsedTime;
  readonly attribute CSSOMString pseudoElement;
};
dictionary AnimationEventInit : EventInit {
  CSSOMString animationName = "";
  double elapsedTime = 0.0;
  CSSOMString pseudoElement = "";
};
```
#### <a id="interface-animationevent-attributes"></a>4.1.2.  Attributes

<a id="ref-for-cssomstring⑤"></a>

<a id="dom-animationevent-animationname"></a>`animationName`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

<a id="ref-for-propdef-animation-name②⑤"></a>

The value of the [animation-name](#propdef-animation-name) property of the animation that fired the event.

<a id="ref-for-idl-double②"></a>

<a id="dom-animationevent-elapsedtime"></a>`elapsedTime`, of type [double](https://webidl.spec.whatwg.org/#idl-double), readonly

The amount of time the animation has been running, in seconds, when this event fired, excluding any time the animation was paused. The precise calculation for of this member is defined along with each event type.

<a id="ref-for-cssomstring⑥"></a>

<a id="dom-animationevent-pseudoelement"></a>`pseudoElement`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

The name (beginning with two colons) of the CSS pseudo-element on which the animation runs (in which case the target of the event is that pseudo-element’s corresponding element), or the empty string if the animation runs on an element (which means the target of the event is that element).

<a id="ref-for-constructing-events"></a>

<a id="dom-animationevent-animationevent"></a>`AnimationEvent(type, animationEventInitDict)` is an [event constructor](https://dom.spec.whatwg.org/#constructing-events).

### <a id="event-animationevent"></a>4.2.  Types of `AnimationEvent`

The different types of animation events that can occur are:

<a id="eventdef-globaleventhandlers-animationstart"></a>`animationstart`  
<a id="ref-for-propdef-animation-delay①⓪"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationstart①"></a>

The <code><a href="#eventdef-globaleventhandlers-animationstart">animationstart</a></code> event occurs at the start of the animation. If there is an [animation-delay](#propdef-animation-delay) then this event will fire once the delay period has expired.

<a id="ref-for-dom-animationevent-elapsedtime①"></a>

<a id="ref-for-active-duration"></a>

<a id="ref-for-propdef-animation-delay①①"></a>

<a id="ref-for-active-duration①"></a>

<a id="ref-for-propdef-animation-play-state③"></a>

<a id="ref-for-valdef-animation-play-state-running③"></a>

<a id="ref-for-valdef-animation-play-state-paused③"></a>

A negative delay will cause the event to fire with an <code><a href="#dom-animationevent-elapsedtime">elapsedTime</a></code> equal to the absolute value of the delay capped to the [active duration](#active-duration) of the animation, that is, <code>min(max(-<a href="#propdef-animation-delay">animation-delay</a>, 0), <a href="#active-duration">active duration</a>)</code>; in this case the event will fire whether [animation-play-state](#propdef-animation-play-state) is set to [running](#valdef-animation-play-state-running) or [paused](#valdef-animation-play-state-paused).

- Bubbles: Yes
- Cancelable: No
- Context Info: animationName, elapsedTime, pseudoElement

<a id="eventdef-globaleventhandlers-animationend"></a>`animationend`  
<a id="ref-for-active-duration②"></a>

<a id="ref-for-dom-animationevent-elapsedtime②"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationend①"></a>

The <code><a href="#eventdef-globaleventhandlers-animationend">animationend</a></code> event occurs when the animation finishes. In this case the value of the <code><a href="#dom-animationevent-elapsedtime">elapsedTime</a></code> member of the event is equal to the [active duration](#active-duration).

- Bubbles: Yes
- Cancelable: No
- Context Info: animationName, elapsedTime, pseudoElement

<a id="eventdef-globaleventhandlers-animationiteration"></a>`animationiteration`  
<a id="ref-for-eventdef-globaleventhandlers-animationiteration"></a>

The <code><a href="#eventdef-globaleventhandlers-animationiteration">animationiteration</a></code> event occurs at the end of each iteration of an animation, except when an animationend event would fire at the same time. This means that this event does not occur for animations with an iteration count of one or less.

<a id="ref-for-dom-animationevent-elapsedtime③"></a>

<a id="ref-for-propdef-animation-duration⑤"></a>

<a id="ref-for-propdef-animation-delay①②"></a>

The <code><a href="#dom-animationevent-elapsedtime">elapsedTime</a></code> member in this case is equal to the product of the <var>current iteration</var> and [animation-duration](#propdef-animation-duration) where the <var>current iteration</var> is the zero-based index of the new iteration. For example, assuming no negative [animation-delay](#propdef-animation-delay), after one iteration completes the <var>current iteration</var> would be one.

- Bubbles: Yes
- Cancelable: No
- Context Info: animationName, elapsedTime, pseudoElement

<a id="eventdef-globaleventhandlers-animationcancel"></a>`animationcancel`  
<a id="ref-for-propdef-animation-name②⑥"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationend②"></a>

<a id="ref-for-eventdef-globaleventhandlers-animationcancel"></a>

The <code><a href="#eventdef-globaleventhandlers-animationcancel">animationcancel</a></code> event occurs when the animation stops running in a way that does not fire an <code><a href="#eventdef-globaleventhandlers-animationend">animationend</a></code> event, such as a change in the [animation-name](#propdef-animation-name) that removes the animation, or the animating element or one of its ancestors becoming display:none.

<a id="ref-for-dom-animationevent-elapsedtime④"></a>

<a id="ref-for-propdef-animation-delay①③"></a>

<a id="ref-for-dom-animationevent-elapsedtime⑤"></a>

The <code><a href="#dom-animationevent-elapsedtime">elapsedTime</a></code> member for this event indicates the number of seconds that had elapsed since the beginning of the animation at the moment when the animation was canceled. This excludes any time where the animation was paused. If the animation had a negative [animation-delay](#propdef-animation-delay), the beginning of the animation is the moment equal to the absolute value of <a id="ref-for-propdef-animation-delay①④"></a>animation-delay seconds <em>prior</em> to when the animation was actually triggered. Alternatively, if the animation had a positive <a id="ref-for-propdef-animation-delay①⑤"></a>animation-delay and the event is fired before the animation’s delay has expired, the <code><a href="#dom-animationevent-elapsedtime">elapsedTime</a></code> will be zero.

- Bubbles: Yes
- Cancelable: No
- Context Info: animationName, elapsedTime, pseudoElement

### <a id="event-handlers-on-elements-document-objects-and-window-objects"></a>4.3. Event handlers on elements, `Document` objects, and `Window` objects

<a id="ref-for-event-handlers"></a>

<a id="ref-for-event-handler-event-type"></a>

<a id="ref-for-html-elements"></a>

<a id="ref-for-event-handler-content-attributes"></a>

<a id="ref-for-event-handler-idl-attributes"></a>

<a id="ref-for-document"></a>

<a id="ref-for-window"></a>

The following are the [event handlers](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers) (and their corresponding [event handler event types](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type)) that must be supported by all [HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements), as both [event handler content attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-content-attributes) and [event handler IDL attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes); and that must be supported by all <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> objects, as <a id="ref-for-event-handler-idl-attributes①"></a>event handler IDL attributes:

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-event-handlers①"></a>

[Event handler](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers)

<strong>Column 2 (header cell):</strong>

<a id="ref-for-event-handler-event-type①"></a>

[Event handler event type](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type)

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="dom-document-onanimationstart"></a>`onanimationstart`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-globaleventhandlers-animationstart②"></a>

<code><a href="#eventdef-globaleventhandlers-animationstart">animationstart</a></code>

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="dom-document-onanimationiteration"></a>`onanimationiteration`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-globaleventhandlers-animationiteration①"></a>

<code><a href="#eventdef-globaleventhandlers-animationiteration">animationiteration</a></code>

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="dom-document-onanimationend"></a>`onanimationend`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-globaleventhandlers-animationend③"></a>

<code><a href="#eventdef-globaleventhandlers-animationend">animationend</a></code>

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="dom-document-onanimationcancel"></a>`onanimationcancel`

<strong>Column 2 (data cell):</strong>

<a id="ref-for-eventdef-globaleventhandlers-animationcancel①"></a>

<code><a href="#eventdef-globaleventhandlers-animationcancel">animationcancel</a></code>

## <a id="interface-dom"></a>5.  DOM Interfaces

CSS animations are exposed to the CSSOM through a pair of new interfaces describing the keyframes.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the interfaces defined below reflect the interoperable API available as of this level of the specification. Future levels may deprecate parts of this API and extend others.

### <a id="interface-cssrule"></a>5.1.  The `CSSRule` Interface

<a id="ref-for-cssrule"></a>

The following two rule types are added to the <code><a href="https://www.w3.org/TR/cssom-1/#cssrule">CSSRule</a></code> interface. They provide identification for the new keyframe and keyframes rules.

#### <a id="interface-cssrule-idl"></a>5.1.1.  IDL Definition

<a id="ref-for-cssrule①"></a>

<a id="ref-for-idl-unsigned-short"></a>

<a id="dom-cssrule-keyframes_rule"></a>

<a id="ref-for-idl-unsigned-short①"></a>

<a id="dom-cssrule-keyframe_rule"></a>

```text
partial interface CSSRule {
    const unsigned short KEYFRAMES_RULE = 7;
    const unsigned short KEYFRAME_RULE = 8;
};
```
### <a id="interface-csskeyframerule"></a>5.2.  The `CSSKeyframeRule` Interface

<a id="ref-for-csskeyframerule"></a>

The <code><a href="#csskeyframerule">CSSKeyframeRule</a></code> interface represents the style rule for a single key.

#### <a id="interface-csskeyframerule-idl"></a>5.2.1.  IDL Definition

<a id="ref-for-Exposed①"></a>

<a id="csskeyframerule"></a>

<a id="ref-for-cssrule②"></a>

<a id="ref-for-cssomstring⑦"></a>

<a id="ref-for-dom-csskeyframerule-keytext"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-PutForwards"></a>

<a id="ref-for-dom-cssstyledeclaration-csstext"></a>

<a id="ref-for-cssstyledeclaration"></a>

<a id="ref-for-dom-csskeyframerule-style"></a>

```text
[Exposed=Window]
interface CSSKeyframeRule : CSSRule {
  attribute CSSOMString keyText;
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleDeclaration style;
};
```
#### <a id="interface-csskeyframerule-attributes"></a>5.2.2.  Attributes

<a id="ref-for-cssomstring⑧"></a>

<a id="dom-csskeyframerule-keytext"></a>`keyText`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-valdef-shape-to⑤"></a>

This attribute represents the keyframe selector as a comma-separated list of percentage values. The from and [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) keywords map to 0% and 100%, respectively.

<a id="ref-for-dom-csskeyframerule-keytext①"></a>

<a id="ref-for-syntaxerror"></a>

If [keyText](#dom-csskeyframerule-keytext) is updated with an invalid keyframe selector, a [SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror) exception must be thrown and the value of <a id="ref-for-dom-csskeyframerule-keytext②"></a>keyText must remain unchanged.

<a id="ref-for-cssstyledeclaration①"></a>

<a id="dom-csskeyframerule-style"></a>`style`, of type [CSSStyleDeclaration](https://www.w3.org/TR/cssom-1/#cssstyledeclaration), readonly

<a id="ref-for-cssstyledeclaration②"></a>

Must return a <code><a href="https://www.w3.org/TR/cssom-1/#cssstyledeclaration">CSSStyleDeclaration</a></code> object for the keyframe rule, with the following properties:

readonly flag

Unset.

<a id="ref-for-cssstyledeclaration-declarations"></a>

[declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations)

<a id="ref-for-concept-declarations-specified-order"></a>

The declared declarations in the rule, in [specified order](https://www.w3.org/TR/cssom-1/#concept-declarations-specified-order).

<a id="ref-for-cssstyledeclaration-parent-css-rule"></a>

[parent CSS rule](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-parent-css-rule)

<a id="ref-for-csskeyframerule①"></a>

The context object (i.e. this <code><a href="#csskeyframerule">CSSKeyframeRule</a></code>).

<a id="ref-for-cssstyledeclaration-owner-node"></a>

[owner node](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-owner-node)

Null.

### <a id="interface-csskeyframesrule"></a>5.3.  The `CSSKeyframesRule` Interface

<a id="ref-for-csskeyframesrule"></a>

The <code><a href="#csskeyframesrule">CSSKeyframesRule</a></code> interface represents a complete set of keyframes for a single animation.

#### <a id="interface-csskeyframesrule-idl"></a>5.3.1.  IDL Definition

<a id="ref-for-Exposed②"></a>

<a id="csskeyframesrule"></a>

<a id="ref-for-cssrule③"></a>

<a id="ref-for-cssomstring⑨"></a>

<a id="ref-for-dom-csskeyframesrule-name"></a>

<a id="ref-for-cssrulelist"></a>

<a id="ref-for-dom-csskeyframesrule-cssrules"></a>

<a id="ref-for-idl-unsigned-long"></a>

<a id="ref-for-dom-csskeyframesrule-length"></a>

<a id="ref-for-csskeyframerule②"></a>

<a id="ref-for-idl-unsigned-long①"></a>

<a id="ref-for-dom-csskeyframesrule-__getter__-index-index"></a>

<a id="ref-for-idl-undefined"></a>

<a id="ref-for-dom-csskeyframesrule-appendrule"></a>

<a id="ref-for-cssomstring①⓪"></a>

<a id="ref-for-dom-csskeyframesrule-appendrule-rule-rule"></a>

<a id="ref-for-idl-undefined①"></a>

<a id="ref-for-dom-csskeyframesrule-deleterule"></a>

<a id="ref-for-cssomstring①①"></a>

<a id="ref-for-dom-csskeyframesrule-deleterule-select-select"></a>

<a id="ref-for-csskeyframerule③"></a>

<a id="ref-for-dom-csskeyframesrule-findrule"></a>

<a id="ref-for-cssomstring①②"></a>

<a id="ref-for-dom-csskeyframesrule-findrule-select-select"></a>

```text
[Exposed=Window]
interface CSSKeyframesRule : CSSRule {
           attribute CSSOMString name;
  readonly attribute CSSRuleList cssRules;
  readonly attribute unsigned long length;

  getter CSSKeyframeRule (unsigned long index);
  undefined        appendRule(CSSOMString rule);
  undefined        deleteRule(CSSOMString select);
  CSSKeyframeRule? findRule(CSSOMString select);
};
```
#### <a id="interface-csskeyframesrule-attributes"></a>5.3.2.  Attributes

<a id="ref-for-cssomstring①③"></a>

<a id="dom-csskeyframesrule-name"></a>`name`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring)

<a id="ref-for-propdef-animation-name②⑦"></a>

This attribute is the name of the keyframes, used by the [animation-name](#propdef-animation-name) property.

<a id="ref-for-cssrulelist①"></a>

<a id="dom-csskeyframesrule-cssrules"></a>`cssRules`, of type [CSSRuleList](https://www.w3.org/TR/cssom-1/#cssrulelist), readonly

This attribute gives access to the keyframes in the list.

<a id="ref-for-idl-unsigned-long②"></a>

<a id="dom-csskeyframesrule-length"></a>`length`, of type [unsigned long](https://webidl.spec.whatwg.org/#idl-unsigned-long), readonly

This attribute is the number of keyframes in the list.

#### <a id="interface-csskeyframesrule-indexed-property-getter"></a>5.3.3.  The indexed property getter

<a id="ref-for-csskeyframerule④"></a>

The <a id="dom-csskeyframesrule-__getter__"></a>

```text
indexed property
	getter
```
returns the <code><a href="#csskeyframerule">CSSKeyframeRule</a></code> from the list of keyframes at the indicated position.

Parameters:

<a id="ref-for-idl-unsigned-long③"></a>

<a id="dom-csskeyframesrule-__getter__-index-index"></a>`index` of type <code><a href="https://webidl.spec.whatwg.org/#idl-unsigned-long">unsigned long</a></code>

The zero-based index of the rule to return.

Return Value:

<a id="ref-for-csskeyframerule⑤"></a>

<code><a href="#csskeyframerule">CSSKeyframeRule</a></code>

<a id="ref-for-idl-undefined②"></a>

The found rule or <code><a href="https://webidl.spec.whatwg.org/#idl-undefined">undefined</a></code> if there is no rule at the specific index.

No Exceptions

#### <a id="interface-csskeyframesrule-appendrule"></a>5.3.4.  The `appendRule` method

<a id="ref-for-csskeyframerule⑥"></a>

The <a id="dom-csskeyframesrule-appendrule"></a>`appendRule` method appends the passed <code><a href="#csskeyframerule">CSSKeyframeRule</a></code> at the end of the keyframes rule.

Parameters:

<a id="ref-for-cssomstring①④"></a>

<a id="dom-csskeyframesrule-appendrule-rule-rule"></a>`rule` of type <code><a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a></code>

<a id="ref-for-at-ruledef-keyframes⑨"></a>

The rule to be appended, expressed in the same syntax as one entry in the [@keyframes](#at-ruledef-keyframes) rule. A valid rule is always appended e.g. even if its key(s) already exists.

No Return Value

No Exceptions

#### <a id="interface-csskeyframesrule-deleterule"></a>5.3.5.  The `deleteRule` method

<a id="ref-for-csskeyframerule⑦"></a>

The <a id="dom-csskeyframesrule-deleterule"></a>`deleteRule` method deletes the last declared <code><a href="#csskeyframerule">CSSKeyframeRule</a></code> matching the specified keyframe selector. If no matching rule exists, the method does nothing.

Parameters:

<a id="ref-for-cssomstring①⑤"></a>

<a id="dom-csskeyframesrule-deleterule-select-select"></a>`select` of type <code><a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a></code>

<a id="ref-for-valdef-shape-to⑥"></a>

The keyframe selector of the rule to be deleted: a comma-separated list of percentage values between 0% and 100% or the keywords from or [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) which resolve to 0% and 100%, respectively.

The number and order of the values in the specified keyframe selector must match those of the targeted keyframe rule(s). The match is not sensitive to white space around the values in the list.

No Return Value

No Exceptions

#### <a id="interface-csskeyframesrule-findrule"></a>5.3.6.  The `findRule` method

<a id="ref-for-csskeyframerule⑧"></a>

The <a id="dom-csskeyframesrule-findrule"></a>`findRule` returns the last declared <code><a href="#csskeyframerule">CSSKeyframeRule</a></code> matching the specified keyframe selector. If no matching rule exists, the method does nothing.

Parameters:

<a id="ref-for-cssomstring①⑥"></a>

<a id="dom-csskeyframesrule-findrule-select-select"></a>`select` of type <code><a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a></code>

<a id="ref-for-valdef-shape-to⑦"></a>

The keyframe selector of the rule to be found: a comma-separated list of percentage values between 0% and 100% or the keywords from or [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) which resolve to 0% and 100%, respectively.

The number and order of the values in the specified keyframe selector must match those of the targeted keyframe rule(s). The match is not sensitive to white space around the values in the list.

Return Value:

<a id="ref-for-csskeyframerule⑨"></a>

<code><a href="#csskeyframerule">CSSKeyframeRule</a></code>

The found rule.

No Exceptions

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d1f87811"></a> For example, given the following animation:
>
> ```text
> @keyframes colorful-diagonal-slide {
> 
>   from {
>     left: 0;
>     top: 0;
>   }
> 
>   10% {
>     background-color: blue;
>   }
> 
>   10% {
>     background-color: green;
>   }
> 
>   25%, 75% {
>     background-color: red;
>   }
> 
>   100% {
>     left: 100px;
>     top: 100px;
>   }
> 
> }
> ```
>
> Assuming the variable `anim` holds a reference to a CSSKeyframesRule object for this animation, then:
>
> ```text
> anim.deleteRule('10%');
> var tenPercent = anim.findRule('10%');
> ```
>
> will start by deleting the last 10% rule i.e. the green background color rule; then find the remaining blue background rule and return it into `tenPercent`.
>
> The following:
>
> ```text
> var red = anim.findRule('75%');
> ```
>
> will set `red` to `null`. The full selector for the red background color rule must be used instead:
>
> ```text
> var red = anim.findRule('25%,75%');
> ```
>
> <a id="ref-for-valdef-shape-to⑧"></a>
>
> Since from maps to 0% and [to](https://drafts.csswg.org/css-shapes-2/#valdef-shape-to) maps to 100%, we can find these rules using either value:
>
> ```text
> var from = anim.findRule('0%'); // Returns from { left: 0; top: 0; } rule
> var to = anim.findRule('to');   // Returns 100% { left: 100px; top: 100px; } rule
> ```
### <a id="interface-globaleventhandlers"></a>5.4.  Extensions to the `GlobalEventHandlers` Interface Mixin

<a id="ref-for-globaleventhandlers"></a>

<a id="ref-for-event-handler-idl-attributes②"></a>

This specification extends the <code><a href="https://html.spec.whatwg.org/multipage/webappapis.html#globaleventhandlers">GlobalEventHandlers</a></code> interface mixin from HTML to add [event handler IDL attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes) for [animation events](#events) as defined in [§ 4.3 Event handlers on elements, Document objects, and Window objects](#event-handlers-on-elements-document-objects-and-window-objects).

#### <a id="interface-globaleventhandlers-idl"></a>5.4.1.  IDL Definition

<a id="ref-for-globaleventhandlers①"></a>

<a id="ref-for-eventhandler"></a>

<a id="dom-globaleventhandlers-onanimationstart"></a>

<a id="ref-for-eventhandler①"></a>

<a id="dom-globaleventhandlers-onanimationiteration"></a>

<a id="ref-for-eventhandler②"></a>

<a id="dom-globaleventhandlers-onanimationend"></a>

<a id="ref-for-eventhandler③"></a>

<a id="dom-globaleventhandlers-onanimationcancel"></a>

```text
partial interface mixin GlobalEventHandlers {
  attribute EventHandler onanimationstart;
  attribute EventHandler onanimationiteration;
  attribute EventHandler onanimationend;
  attribute EventHandler onanimationcancel;
};
```
## <a id="priv"></a>6.  Privacy Considerations

No privacy concerns have been reported on this specification.

## <a id="sec"></a>7.  Security Considerations

No security concerns have been reported on this specification.

## <a id="changes"></a>8. Changes

### <a id="changes-20181011"></a>8.1. Changes since the [Working Draft of 11 October 2018](https://www.w3.org/TR/2018/WD-css-animations-1-20181011/)

The following substantive changes were made:

- Defined indexed property getter for CSSKeyframesRule
- Added constructor type on AnimationEvent’s definition
- Added required unit for dimension in range notation
- Applied range definition notation to descriptor and rule’s prelude values
- Applied range definition notation to property values
- Associated event definitions with their EventHandler container
- Better markup for productions
- Corrected typo (rule to be found, not rule to be deleted)
- Made value definition reference consistent with other CSS specifications
- IDL aligned with Web IDL specification
- Added default dictionary value to constructor
- Rewrote confusing example ([\#4118](https://github.com/w3c/csswg-drafts/issues/4118))
- Clarified handling of zero-duration animations
- Use "not animatable" rather than "none"
- Timing functions now called easing functions
- Changed GlobalEventHandlers to be a mixin

## <a id="acknowledgements"></a>9.  Acknowledgements

Thanks especially to the feedback from Tab Atkins, Brian Birtles, Shane Stephens, Carine Bournez, Christian Budde, Anne van Kesteren, Øyvind Stenhaug, Estelle Weyl, and all the rest of the www-style community.

## <a id="other-open-issues"></a>10. Other open issues

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c01277bc"></a>Need to [specify how keyframes interact](https://lists.w3.org/Archives/Public/www-style/2015Jul/0391.html).

## <a id="wg-resolutions-pending"></a>11.  Working Group Resolutions that are pending editing

<em>This section is informative and temporary.</em>

The editors are currently behind on editing this spec. The following working group resolutions still need to be edited in:

- 2014-09-09 minutes (Antibes f2f)
  - Issue(7335): Detail how/when keyframe values are computed; using [G.beta in dbaron’s mail](https://lists.w3.org/Archives/Public/www-style/2014Aug/0132.html)
  - ~~Agreed that both transitions and animations animate all properties. css-transitions to define animation of non-interoperable/discrete values. They take their starting values below 50% timing function progress, and end values above~~
  - ~~Dynamic changes to animation properties/keyframes. Tab to propose resolution. ([Bug 14713](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14713))~~
  - ~~Negative animation-delay values apply against the active duration of the animation i.e. (animation-duration\*animation-iteration-count). The delay can thus swallow iterations for which no iteration event will be fired. The start/end events are still fired. Even when delay == (-1\*active_duration)~~
  - ~~Fire animation start/end events when animation-duration is zero, with 0 elapsedTime~~
  - ~~If animation-iteration-count is infinite and duration is 0, treat the iteration-count as if it was finite and run a 0s second (option A in [Brian’s mail](https://lists.w3.org/Archives/Public/www-style/2014Sep/0056.html))~~
  - ~~If an animation with a negative animation delay is initially paused, the start event still fires~~
- 2012-10-29 minutes
  - ~~Change the animation properties to be dynamically changeable~~
  - ~~@keyframes can be dynamically changed~~
  - ~~When you encounter duplicate animations names, last one wins.~~
  - ~~Make \*animations\* transition \*all\* properties. Unless otherwise specified, discrete properties take their starting values below 50% timing function progress, and end values above 50% timing function progress.~~
- 2012-12-12 minutes and intermediate comments ~~and 2012-12-19 minutes~~
  - ~~Animations only run if they contain at least one valid keyframe rule ([Bug](https://www.w3.org/Bugs/Public/show_bug.cgi?id=15251))~~
  - ~~When an element changes from display:none to display: non-none, animations start immediately ([Bug](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14785))~~
  - ~~An initially-paused animation is still started (fires start events etc.) ([Bug](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14774))~~
  - ~~Animations can be paused during their delay phase, which freezes the remaining delay to be applied after it unpauses ([Bug](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14774))~~
  - ~~animation-play-state has the same list behavior as the other animation properties, matching the length of animation-name ([Bug](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14786))~~
- 2013-02-20 minutes
  - ~~Øyvind’s clarification accepted~~
  - ~~keyframe rules cascade~~
  - ~~mark pseudoElement at-risk~~
- 2013-05-30 minutes
  - ~~expectations on animations in non-interactive media~~
- 2014-01-27 minutes
  - ~~remove text about waiting for document load~~

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

- [active duration](#active-duration), in § 3.5
- [alternate](#valdef-animation-direction-alternate), in § 3.6
- [alternate-reverse](#valdef-animation-direction-alternate-reverse), in § 3.6
- [animation](#propdef-animation), in § 3.10
- [animationcancel](#eventdef-globaleventhandlers-animationcancel), in § 4.2
- [animation-delay](#propdef-animation-delay), in § 3.8
- [animation-direction](#propdef-animation-direction), in § 3.6
- [animation-duration](#propdef-animation-duration), in § 3.3
- [animationend](#eventdef-globaleventhandlers-animationend), in § 4.2
- [AnimationEvent](#animationevent), in § 4.1.1
- [AnimationEventInit](#dictdef-animationeventinit), in § 4.1.1
- [AnimationEvent(type)](#dom-animationevent-animationevent), in § 4.1.2
- [AnimationEvent(type, animationEventInitDict)](#dom-animationevent-animationevent), in § 4.1.2
- [animation-fill-mode](#propdef-animation-fill-mode), in § 3.9
- [animationiteration](#eventdef-globaleventhandlers-animationiteration), in § 4.2
- [animation-iteration-count](#propdef-animation-iteration-count), in § 3.5
- [animation-name](#propdef-animation-name), in § 3.2
- animationName
  - [attribute for AnimationEvent](#dom-animationevent-animationname), in § 4.1.2
  - [dict-member for AnimationEventInit](#dom-animationeventinit-animationname), in § 4.1.1
- [animation-play-state](#propdef-animation-play-state), in § 3.7
- [animationstart](#eventdef-globaleventhandlers-animationstart), in § 4.2
- [animation-timing-function](#propdef-animation-timing-function), in § 3.4
- [appendRule(rule)](#dom-csskeyframesrule-appendrule), in § 5.3.4
- [backwards](#valdef-animation-fill-mode-backwards), in § 3.9
- [both](#valdef-animation-fill-mode-both), in § 3.9
- [constructor(type)](#dom-animationevent-animationevent), in § 4.1.2
- [constructor(type, animationEventInitDict)](#dom-animationevent-animationevent), in § 4.1.2
- [CSSKeyframeRule](#csskeyframerule), in § 5.2.1
- [CSSKeyframesRule](#csskeyframesrule), in § 5.3.1
- [cssRules](#dom-csskeyframesrule-cssrules), in § 5.3.2
- [deleteRule(select)](#dom-csskeyframesrule-deleterule), in § 5.3.5
- elapsedTime
  - [attribute for AnimationEvent](#dom-animationevent-elapsedtime), in § 4.1.2
  - [dict-member for AnimationEventInit](#dom-animationeventinit-elapsedtime), in § 4.1.1
- [findRule(select)](#dom-csskeyframesrule-findrule), in § 5.3.6
- [forwards](#valdef-animation-fill-mode-forwards), in § 3.9
- [\_\_getter\_\_(index)](#dom-csskeyframesrule-__getter__), in § 5.3.3
- [infinite](#valdef-animation-iteration-count-infinite), in § 3.5
- [\<keyframe-block\>](#typedef-keyframe-block), in § 3
- [KEYFRAME_RULE](#dom-cssrule-keyframe_rule), in § 5.1.1
- [@keyframes](#at-ruledef-keyframes), in § 3
- [\<keyframe-selector\>](#typedef-keyframe-selector), in § 3
- \<keyframes-name\>
  - [(type)](#typedef-keyframes-name), in § 3
  - [value for animation-name](#valdef-animation-name-keyframes-name), in § 3.2
- [KEYFRAMES_RULE](#dom-cssrule-keyframes_rule), in § 5.1.1
- [keyText](#dom-csskeyframerule-keytext), in § 5.2.2
- [length](#dom-csskeyframesrule-length), in § 5.3.2
- [name](#dom-csskeyframesrule-name), in § 5.3.2
- none
  - [value for animation-fill-mode](#valdef-animation-fill-mode-none), in § 3.9
  - [value for animation-name](#valdef-animation-name-none), in § 3.2
- [normal](#valdef-animation-direction-normal), in § 3.6
- [\<number \[0,∞\]\>](#valdef-animation-iteration-count-number-0), in § 3.5
- onanimationcancel
  - [attribute for Document, Window](#dom-document-onanimationcancel), in § 4.3
  - [attribute for GlobalEventHandlers](#dom-globaleventhandlers-onanimationcancel), in § 5.4.1
- onanimationend
  - [attribute for Document, Window](#dom-document-onanimationend), in § 4.3
  - [attribute for GlobalEventHandlers](#dom-globaleventhandlers-onanimationend), in § 5.4.1
- onanimationiteration
  - [attribute for Document, Window](#dom-document-onanimationiteration), in § 4.3
  - [attribute for GlobalEventHandlers](#dom-globaleventhandlers-onanimationiteration), in § 5.4.1
- onanimationstart
  - [attribute for Document, Window](#dom-document-onanimationstart), in § 4.3
  - [attribute for GlobalEventHandlers](#dom-globaleventhandlers-onanimationstart), in § 5.4.1
- [paused](#valdef-animation-play-state-paused), in § 3.7
- pseudoElement
  - [attribute for AnimationEvent](#dom-animationevent-pseudoelement), in § 4.1.2
  - [dict-member for AnimationEventInit](#dom-animationeventinit-pseudoelement), in § 4.1.1
- [reverse](#valdef-animation-direction-reverse), in § 3.6
- [running](#valdef-animation-play-state-running), in § 3.7
- [\<single-animation\>](#typedef-single-animation), in § 3.10
- [\<single-animation-direction\>](#typedef-single-animation-direction), in § 3.6
- [\<single-animation-fill-mode\>](#typedef-single-animation-fill-mode), in § 3.9
- [\<single-animation-iteration-count\>](#typedef-single-animation-iteration-count), in § 3.5
- [\<single-animation-play-state\>](#typedef-single-animation-play-state), in § 3.7
- [style](#dom-csskeyframerule-style), in § 5.2.2
- [\<time\>](#valdef-animation-delay-time), in § 3.8
- [\<time \[0s,∞\]\>](#valdef-animation-duration-time-0s), in § 3.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[\] defines the following terms:
  - <a id="61bb7a5b9421836575c9d121d3e9e6b6"></a>event constructor
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="6f4bad3ae2bdf5420e49c663f6bbb320"></a>background-image
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="4838ae505a5acb011e1fe852daa781db"></a>none
- \[CSS-EASING-1\] defines the following terms:
  - <a id="ee6fd6276e163b081bce62e112ac728f"></a>\<easing-function\>
  - <a id="ea4b508302a3ac14144f3bdb0707a879"></a>ease-in
  - <a id="d481ea65ec51309b9e09703341f6a3cd"></a>ease-out
  - <a id="b6709c5d9590020da895139c3710f75c"></a>input progress value
  - <a id="a2cb467b447e045ec0c4d43c3ba1191a"></a>output progress value
  - <a id="3664095edfb1e49f89502483c0358cfc"></a>start
  - <a id="65e652b05e077a9bb1465b5f732f0a85"></a>step easing function
  - <a id="dca3f97e7febeaa5f99a1f33e9d6c462"></a>step position
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="51d732fc13e8056bb31023dbe0bb875e"></a>left
- \[CSS-SHAPES-2\] defines the following terms:
  - <a id="4182fe9596ded044e6d866ad0d0ec393"></a>to
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="41fc454547a6d3114f8215630f9e193d"></a>\<declaration-list\>
  - <a id="6f85cb0a8ba4f0187e7e4bae7523df81"></a>\<rule-list\>
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="b6621fae79cb57af396aa49f7e7db347"></a>\<time\>
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="0b5bb323a01d33c3301ca5e25beb9662"></a>\#
  - <a id="b0e6219d0e78585a78e5a52d1b3c955f"></a>\<custom-ident\>
  - <a id="85d5ec3b90b3bb54a006449512e66508"></a>\<number\>
  - <a id="d38ad2dc5194c097d2b903765aca9070"></a>\<percentage\>
  - <a id="19ccb6e7dc9e8cb62749742d224f6b3e"></a>\<string\>
  - <a id="9b0f481cd87f347029bea93451ee1f59"></a>css identifier
  - <a id="f014258978db0b5cb28d306e0a9068b1"></a>css-wide keywords
  - <a id="63f2f251f3aa0bf36028f290b3f6aba0"></a>\|
  - <a id="2421c65c76a70f9151ba02cfcf7cc94f"></a>\|\|
- \[CSS-WILL-CHANGE-1\] defines the following terms:
  - <a id="f92ba3babed5c7ce1fb44d90cb7186a0"></a>will-change
- \[CSS22\] defines the following terms:
  - <a id="e28bb5a9e849b0f1e5b2ae3660fa3877"></a>display
- \[CSSOM-1\] defines the following terms:
  - <a id="04e9b32172136734191119e0f5813089"></a>CSSOMString
  - <a id="0ece9ffd885320a6170fc48bc8950860"></a>CSSRule
  - <a id="9316b7d65b3a1e735cd036f01323632e"></a>CSSRuleList
  - <a id="0c3c45982bf4380904c7feec4a801558"></a>CSSStyleDeclaration
  - <a id="38e799448029d969a3254b7b96f37325"></a>cssText
  - <a id="76fc356132329f01f7550f1023e6517a"></a>declarations
  - <a id="34289defb505583ed0b980230b5462b4"></a>owner node
  - <a id="c9fc7022d24dca777f4025f155d9c62d"></a>parent css rule
  - <a id="eb9f04b66c61821424180491f6d3d814"></a>specified order
- \[DOM\] defines the following terms:
  - <a id="07886e33695d0e7b473ef18f656ad7ac"></a>Document
  - <a id="316772b0029bcc028521d2db08fd3717"></a>Event
  - <a id="a41247dbf908b8c39ef1dea1be6618d3"></a>EventInit
- \[HTML\] defines the following terms:
  - <a id="716703f1068cc638b4235f83154eff6c"></a>EventHandler
  - <a id="fc4257a537585da23125bc5f94aace80"></a>GlobalEventHandlers
  - <a id="d1f0a86fd49be388d8b0159e241a1172"></a>Window
  - <a id="ebf12c6eb41baf0a7aad11d90d47c9b1"></a>event handler content attributes
  - <a id="63ae16e3c4cc120a678b317e8329b704"></a>event handler event type
  - <a id="e3136d72f4de2e72313bad6865c2c425"></a>event handler idl attributes
  - <a id="d1998b785302094e32cb2d2b77722407"></a>event handlers
  - <a id="46f51e654a5b9ed42223d88e4d334fe1"></a>html elements
- \[WEBIDL\] defines the following terms:
  - <a id="dde049eb112f043fc45407ec0d4cb457"></a>Exposed
  - <a id="6db05de9e5a0ef0d34be1ac8d17623d0"></a>PutForwards
  - <a id="f572762cd4679aeacd36722d99d2b741"></a>SameObject
  - <a id="fb01fee229e88e7f1925373838fe6d85"></a>SyntaxError
  - <a id="d3fd0dc5b12c87e613395fec1824ac77"></a>double
  - <a id="cee863f1f2ec91066cfb7f5d309d59dc"></a>undefined
  - <a id="03fa8b99cc63b35d2e7dcadb686676f1"></a>unsigned long
  - <a id="85ad66f191d72a7d6508e0bab70044dc"></a>unsigned short

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-shapes-2"></a>\[CSS-SHAPES-2\]  
CSS Shapes Module Level 2 URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shapes-2&#x2F;](https://drafts.csswg.org/css-shapes-2/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 1 December 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 19 October 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-will-change-1"></a>\[CSS-WILL-CHANGE-1\]  
Tab Atkins Jr.. [CSS Will Change Module Level 1](https://www.w3.org/TR/css-will-change-1/). 5 May 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-will-change-1&#x2F;](https://www.w3.org/TR/css-will-change-1/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-css3cascade"></a>\[CSS3CASCADE\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-wcag20"></a>\[WCAG20\]  
Ben Caldwell; et al. [Web Content Accessibility Guidelines (WCAG) 2.0](https://www.w3.org/TR/WCAG20/). 11 December 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG20&#x2F;](https://www.w3.org/TR/WCAG20/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 14 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css3-transitions"></a>\[CSS3-TRANSITIONS\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

## <a id="property-index"></a>Property Index

<strong>Table 11 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-animation⑤"></a>

[animation](#propdef-animation)

<strong>Column 2 (data cell):</strong>

\<single-animation\>#

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-delay①⑥"></a>

[animation-delay](#propdef-animation-delay)

<strong>Column 2 (data cell):</strong>

\<time\>#

<strong>Column 3 (data cell):</strong>

0s

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a duration

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-direction⑧"></a>

[animation-direction](#propdef-animation-direction)

<strong>Column 2 (data cell):</strong>

\<single-animation-direction\>#

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-duration⑥"></a>

[animation-duration](#propdef-animation-duration)

<strong>Column 2 (data cell):</strong>

\<time \[0s,∞\]\>#

<strong>Column 3 (data cell):</strong>

0s

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a duration

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-fill-mode①①"></a>

[animation-fill-mode](#propdef-animation-fill-mode)

<strong>Column 2 (data cell):</strong>

\<single-animation-fill-mode\>#

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-iteration-count⑦"></a>

[animation-iteration-count](#propdef-animation-iteration-count)

<strong>Column 2 (data cell):</strong>

\<single-animation-iteration-count\>#

<strong>Column 3 (data cell):</strong>

1

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item either a number or the keyword infinite

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-name②⑧"></a>

[animation-name](#propdef-animation-name)

<strong>Column 2 (data cell):</strong>

\[ none \| \<keyframes-name\> \]#

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item either a case-sensitive css identifier or the keyword none

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-play-state④"></a>

[animation-play-state](#propdef-animation-play-state)

<strong>Column 2 (data cell):</strong>

\<single-animation-play-state\>#

<strong>Column 3 (data cell):</strong>

running

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a keyword as specified

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-animation-timing-function⑦"></a>

[animation-timing-function](#propdef-animation-timing-function)

<strong>Column 2 (data cell):</strong>

\<easing-function\>#

<strong>Column 3 (data cell):</strong>

ease

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

list, each item a computed \<easing-function\>

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface AnimationEvent : Event {
  constructor(CSSOMString type, optional AnimationEventInit animationEventInitDict = {});
  readonly attribute CSSOMString animationName;
  readonly attribute double elapsedTime;
  readonly attribute CSSOMString pseudoElement;
};
dictionary AnimationEventInit : EventInit {
  CSSOMString animationName = "";
  double elapsedTime = 0.0;
  CSSOMString pseudoElement = "";
};

partial interface CSSRule {
    const unsigned short KEYFRAMES_RULE = 7;
    const unsigned short KEYFRAME_RULE = 8;
};

[Exposed=Window]
interface CSSKeyframeRule : CSSRule {
  attribute CSSOMString keyText;
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleDeclaration style;
};

[Exposed=Window]
interface CSSKeyframesRule : CSSRule {
           attribute CSSOMString name;
  readonly attribute CSSRuleList cssRules;
  readonly attribute unsigned long length;

  getter CSSKeyframeRule (unsigned long index);
  undefined        appendRule(CSSOMString rule);
  undefined        deleteRule(CSSOMString select);
  CSSKeyframeRule? findRule(CSSOMString select);
};

partial interface mixin GlobalEventHandlers {
  attribute EventHandler onanimationstart;
  attribute EventHandler onanimationiteration;
  attribute EventHandler onanimationend;
  attribute EventHandler onanimationcancel;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This specification needs to define how the value is determined from the keyframes, like the section on [Application of transitions](https://drafts.csswg.org/css-transitions/#application) does for CSS Transitions. [↵](#issue-73aacf21)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Need to [specify how keyframes interact](https://lists.w3.org/Archives/Public/www-style/2015Jul/0391.html). [↵](#issue-c01277bc)
