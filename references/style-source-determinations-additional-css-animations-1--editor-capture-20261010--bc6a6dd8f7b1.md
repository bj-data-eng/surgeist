Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Animations Module Level 1](https://drafts.csswg.org/css-animations-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10 America/New_York (UTC-04:00). Capture time: `2026-10-11T02:56:27.498003+00:00` UTC; Source status: Editor’s Draft, 13 September 2026.

Full captured HTML SHA-256: `a6ef2cb48b0d502cbf3f7ad7636e48279b330b10f4451700e25741f326cb5be0` (376248 bytes). Conversion input SHA-256: `bc6a6dd8f7b11eb55b39a75487307c34acadeec2efff683f7ce6b564e4706531` (65342 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `animations`, `keyframes`, `other-open-issues`, `wg-resolutions-pending`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Animations Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 13 September 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-animations/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-animations-1/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2023/WD-css-animations-1-20230302/>

<https://www.w3.org/TR/2018/WD-css-animations-1-20181011/>

<https://www.w3.org/TR/2017/WD-css-animations-1-20171130/>

<https://www.w3.org/TR/2013/WD-css3-animations-20130219/>

<https://www.w3.org/TR/2012/WD-css3-animations-20120403/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-animations-1)

[Inline In Spec](https://drafts.csswg.org/css-animations-1/#issues-index)

<strong>Editors:</strong>

[Dean Jackson](mailto:dino@apple.com) (Apple Inc.)

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

[Brian Birtles](mailto:brian@birchill.co.jp) (Invited Expert)

<strong>Former Editors:</strong>

David Hyatt (Apple Inc.)

Chris Marrin (Apple Inc.)

[Sylvain Galineau](mailto:galineau@adobe.com) (Adobe)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-animations-1/Overview.bs)

<strong>Issues List:</strong>

<div>https://github.com/w3c/csswg-drafts/labels/css-animations-1</div>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="animations"></a>2.  CSS Animations Model[](#animations)

CSS Animations affect computed property values. This effect happens by adding a specified value to the CSS cascade ([\[CSS3CASCADE\]](https://drafts.csswg.org/css-animations-1/#biblio-css3cascade)) (at the level for CSS Animations) that will produce the correct computed value for the current state of the animation. As defined in [\[CSS3CASCADE\]](https://drafts.csswg.org/css-animations-1/#biblio-css3cascade), animations override all normal rules, but are overridden by !important rules.

If at some point in time there are multiple animations specifying behavior for the same property, the animation which occurs last in the value of <a id="ref-for-propdef-animation-name"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) will override the other animations at that point.

An animation does not affect the computed value before the application of the animation (that is, when the <a id="ref-for-propdef-animation-name①"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property is set on an element) or after it is removed. Furthermore, typically an animation does not affect the computed value before the animation delay has expired or after the end of the animation, but may do so depending on the <a id="ref-for-propdef-animation-fill-mode"></a>[animation-fill-mode](https://drafts.csswg.org/css-animations-1/#propdef-animation-fill-mode) property.

While running, the animation computes the value of those properties it animates. Other values may take precedence over the animated value according to the CSS cascade ([\[CSS3CASCADE\]](https://drafts.csswg.org/css-animations-1/#biblio-css3cascade)).

While an animation is applied but has not finished, or has finished but has an <a id="ref-for-propdef-animation-fill-mode①"></a>[animation-fill-mode](https://drafts.csswg.org/css-animations-1/#propdef-animation-fill-mode) of <a id="ref-for-valdef-animation-fill-mode-forwards"></a>[forwards](https://drafts.csswg.org/css-animations-1/#valdef-animation-fill-mode-forwards) or <a id="ref-for-valdef-animation-fill-mode-both"></a>[both](https://drafts.csswg.org/css-animations-1/#valdef-animation-fill-mode-both), the user agent must act as if the <a id="ref-for-propdef-will-change"></a>[will-change](https://drafts.csswg.org/css-will-change-1/#propdef-will-change) property ([\[css-will-change-1\]](https://drafts.csswg.org/css-animations-1/#biblio-css-will-change-1)) on the element additionally includes all the properties animated by the animation.

The start time of an animation is the time at which the style applying the animation and the corresponding &#64;keyframes rule are both resolved. If an animation is specified for an element but the corresponding &#64;keyframes rule does not yet exist, the animation cannot start; the animation will start from the beginning as soon as a matching &#64;keyframes rule can be resolved. An animation specified by dynamically modifying the element’s style will start when this style is resolved; that may be immediately in the case of a pseudo style rule such as hover, or may be when the scripting engine returns control to the browser (in the case of style applied by script). Note that dynamically updating keyframe style rules does not start or re-start an animation.

An animation applies to an element if its name appears as one of the identifiers in the computed value of the <a id="ref-for-propdef-animation-name②"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property and the animation uses a valid &#64;keyframes rule. Once an animation has started it continues until it ends or the <a id="ref-for-propdef-animation-name③"></a>animation-name is removed. Changes to the values of animation properties while the animation is running apply as if the animation had those values from when it began. For example, shortening the <a id="ref-for-propdef-animation-delay"></a>[animation-delay](https://drafts.csswg.org/css-animations-1/#propdef-animation-delay) may cause the animation to jump forwards or even finish immediately and dispatch an <code><a id="ref-for-eventdef-globaleventhandlers-animationend"></a>[animationend](https://drafts.csswg.org/css-animations-1/#eventdef-globaleventhandlers-animationend)</code> event. Conversely, extending the <a id="ref-for-propdef-animation-delay①"></a>animation-delay may cause an animation to re-start and dispatch an <code><a id="ref-for-eventdef-globaleventhandlers-animationstart"></a>[animationstart](https://drafts.csswg.org/css-animations-1/#eventdef-globaleventhandlers-animationstart)</code> event.

The same &#64;keyframes rule name may be repeated within an <a id="ref-for-propdef-animation-name④"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name). Changes to the <a id="ref-for-propdef-animation-name⑤"></a>animation-name update existing animations by iterating over the new list of animations from last to first, and, for each animation, finding the <em>last</em> matching animation in the list of existing animations. If a match is found, the existing animation is updated using the animation properties corresponding to its position in the new list of animations, whilst maintaining its current playback time as described above. The matching animation is removed from the existing list of animations such that it will not match twice. If a match is not found, a new animation is created. As a result, updating <a id="ref-for-propdef-animation-name⑥"></a>animation-name from ‘a’ to ‘a, a’ will cause the existing animation for ‘a’ to become the <em>second</em> animation in the list and a new animation will be created for the first item in the list.

<a id="example-4e34d7ba"></a>

<strong>Example:</strong>

[](#example-4e34d7ba)

``` text
div {
  animation-name: diagonal-slide;
  animation-duration: 5s;
  animation-iteration-count: 10;
}

@keyframes diagonal-slide {

  from {
    left: 0;
    top: 0;
  }

  to {
    left: 100px;
    top: 100px;
  }

}
```

This will produce an animation that moves an element from (0, 0) to (100px, 100px) over five seconds and repeats itself nine times (for a total of ten iterations).

Setting the <a id="ref-for-propdef-display"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display) property to <a id="ref-for-valdef-display-none"></a>[none](https://drafts.csswg.org/css-display-4/#valdef-display-none) will terminate any running animation applied to the element and its descendants. If an element has a <a id="ref-for-propdef-display①"></a>display of <a id="ref-for-valdef-display-none①"></a>none, updating <a id="ref-for-propdef-display②"></a>display to a value other than <a id="ref-for-valdef-display-none②"></a>none will start all animations applied to the element by the <a id="ref-for-propdef-animation-name⑦"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property, as well as all animations applied to descendants with <a id="ref-for-propdef-display③"></a>display other than <a id="ref-for-valdef-display-none③"></a>none.

While authors can use animations to create dynamically changing content, dynamically changing content can lead to seizures in some users. For information on how to avoid content that can lead to seizures, see Guideline 2.3: Seizures: Do not design content in a way that is known to cause seizures ([\[WCAG20\]](https://drafts.csswg.org/css-animations-1/#biblio-wcag20)).

Implementations may ignore animations when the rendering medium is not interactive e.g. when printed. A future version of this specification may define how to render animations for these media.

## <a id="keyframes"></a>3.  Declaring Keyframes[](#keyframes)

Keyframes are used to specify the values for the animating properties at various points during the animation. The keyframes specify the behavior of one cycle of the animation; the animation may iterate zero or more times.

Keyframes are specified using the <a id="at-ruledef-keyframes"></a><strong>&#64;keyframes</strong> at-rule, defined as follows:

<a id="ref-for-typedef-keyframes-name"></a><a id="ref-for-typedef-qualified-rule-list"></a><a id="typedef-keyframes-name"></a><a id="ref-for-identifier-value"></a><a id="ref-for-string-value"></a><a id="typedef-keyframe-block"></a><a id="ref-for-typedef-keyframe-selector"></a><a id="ref-for-typedef-declaration-list"></a><a id="typedef-keyframe-selector"></a><a id="ref-for-percentage-value"></a>

``` text
@keyframes = @keyframes <keyframes-name> { <qualified-rule-list> }

<keyframes-name> = <custom-ident> | <string>

<keyframe-block> = <keyframe-selector># { <declaration-list> }

<keyframe-selector> = from | to | <percentage [0,100]>
```

The <a id="ref-for-typedef-rule-list"></a>[\<rule-list\>](https://drafts.csswg.org/css-syntax-3/#typedef-rule-list) inside of <a id="ref-for-at-ruledef-keyframes"></a>[&#64;keyframes](#at-ruledef-keyframes) can only contain <a id="ref-for-typedef-keyframe-block"></a>[\<keyframe-block\>](#typedef-keyframe-block) rules.

The <a id="ref-for-typedef-declaration-list①"></a>[\<declaration-list\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-list) inside of <a id="ref-for-typedef-keyframe-block①"></a>[\<keyframe-block\>](#typedef-keyframe-block) accepts any CSS property except those defined in this specification, but <em>does</em> accept the <a id="ref-for-propdef-animation-timing-function"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) property and interprets it specially. None of the properties interact with the cascade (so using !important on them is invalid and will cause the property to be ignored).

A <a id="ref-for-at-ruledef-keyframes①"></a>[&#64;keyframes](#at-ruledef-keyframes) block has a name given by the <a id="ref-for-identifier-value①"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) or <a id="ref-for-string-value①"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value) in its prelude. The two syntaxes are equivalent in functionality; the name is the value of the ident or string. As normal for <a id="ref-for-identifier-value②"></a>\<custom-ident\>s and <a id="ref-for-string-value②"></a>\<string\>s, the names are fully <a id="ref-for-dfn-case-sensitive"></a>[case-sensitive](https://w3c.github.io/i18n-glossary/#dfn-case-sensitive); two names are equal only if they are codepoint-by-codepoint equal.

The <a id="ref-for-identifier-value③"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) additionally excludes the <a id="ref-for-valdef-animation-name-none"></a>[none](https://drafts.csswg.org/css-animations-1/#valdef-animation-name-none) keyword. The <a id="ref-for-string-value③"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value) additionally excludes the empty string (but allows the string "none" and other excluded keywords). When serialized, the value is serialized as an <a id="ref-for-typedef-ident"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) unless it’s a disallowed keyword, in which case it’s serialized as a <a id="ref-for-string-value④"></a>\<string\>.

<a id="example-d0936079"></a>

<strong>Example:</strong>

[](#example-d0936079) For example, the following two <a id="ref-for-at-ruledef-keyframes②"></a>[&#64;keyframes](#at-ruledef-keyframes) rules have the same name, so the first will be ignored:

``` text
@keyframes foo { /* ... */ }
@keyframes "foo" { /* ... */ }
```

On the other hand, the following <a id="ref-for-at-ruledef-keyframes③"></a>[&#64;keyframes](#at-ruledef-keyframes) rule’s name is <em>different</em> from the previous two rules:

``` text
@keyframes FOO { /* ... */ }
```

The following <a id="ref-for-at-ruledef-keyframes④"></a>[&#64;keyframes](#at-ruledef-keyframes) rules are invalid because they use disallowed <a id="ref-for-identifier-value④"></a>[\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) values:

``` text
@keyframes initial { /* ... */ }
@keyframes None { /* ... */ }
```

However, those names <em>can</em> be specified with a <a id="ref-for-string-value⑤"></a>[\<string\>](https://drafts.csswg.org/css-values-4/#string-value), so the following are both <em>valid</em>:

``` text
@keyframes "initial" { /* ... */ }
@keyframes "None" { /* ... */ }
```

The <a id="ref-for-typedef-keyframe-selector①"></a>[\<keyframe-selector\>](#typedef-keyframe-selector) for a <a id="ref-for-typedef-keyframe-block②"></a>[\<keyframe-block\>](#typedef-keyframe-block) consists of a comma-separated list of percentage values or the keywords from or <a id="ref-for-valdef-shape-to"></a>[to](https://drafts.csswg.org/css-shapes-1/#valdef-shape-to). The selector is used to specify the percentage along the duration of the animation that the keyframe represents. The keyframe itself is specified by the block of property values declared on the selector. The keyword from is equivalent to the value 0%. The keyword <a id="ref-for-valdef-shape-to①"></a>to is equivalent to the value 100%. Values less than 0% or higher than 100% are invalid and cause their <a id="ref-for-typedef-keyframe-block③"></a>\<keyframe-block\> to be ignored.

Note: Note that the percentage unit specifier must be used on percentage values. Therefore, 0 is an invalid keyframe selector.

If a 0% or from keyframe is not specified, then the user agent constructs a 0% keyframe using the computed values of the properties being animated. If a 100% or <a id="ref-for-valdef-shape-to②"></a>[to](https://drafts.csswg.org/css-shapes-1/#valdef-shape-to) keyframe is not specified, then the user agent constructs a 100% keyframe using the computed values of the properties being animated.

The <a id="ref-for-typedef-keyframe-block④"></a>[\<keyframe-block\>](#typedef-keyframe-block) contains properties and values. The properties defined by this specification are ignored in these rules, with the exception of <a id="ref-for-propdef-animation-timing-function①"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function), the behavior of which is described below. In addition, properties qualified with !important are invalid and ignored.

If multiple <a id="ref-for-at-ruledef-keyframes⑤"></a>[&#64;keyframes](#at-ruledef-keyframes) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

<a id="example-b784185b"></a>

<strong>Example:</strong>

[](#example-b784185b)

``` text
div {
  animation-name: slide-right;
  animation-duration: 2s;
}

@keyframes slide-right {

  from {
    margin-left: 0px;
  }

  50% {
    margin-left: 110px;
    opacity: 1;
  }

  50% {
    opacity: 0.9;
  }

  to {
    margin-left: 200px;
  }

}
```

The two 50% rules from above can also be combined into an equivalent single rule as illustrated below:

``` text
@keyframes slide-right {

  from {
    margin-left: 0px;
  }

  50% {
    margin-left: 110px;
    opacity: 0.9;
  }

  to {
    margin-left: 200px;
  }

}
```

To determine the set of keyframes, all of the values in the selectors are sorted in increasing order by time. The rules within the <a id="ref-for-at-ruledef-keyframes⑥"></a>[&#64;keyframes](#at-ruledef-keyframes) rule then cascade; the properties of a keyframe may thus derive from more than one <a id="ref-for-at-ruledef-keyframes⑦"></a>&#64;keyframes rule with the same selector value.

If a property is not specified for a keyframe, or is specified but invalid, the animation of that property proceeds as if that keyframe did not exist. Conceptually, it is as if a set of keyframes is constructed for each property that is present in any of the keyframes, and an animation is run independently for each property.

<a id="example-10d90bd9"></a>

<strong>Example:</strong>

[](#example-10d90bd9)

``` text
@keyframes wobble {
  0% {
    left: 100px;
  }

  40% {
    left: 150px;
  }

  60% {
    left: 75px;
  }

  100% {
    left: 100px;
  }
}
```

Four keyframes are specified for the animation named "wobble". In the first keyframe, shown at the beginning of the animation cycle, the value of the <a id="ref-for-propdef-left"></a>[left](https://drafts.csswg.org/css-position-3/#propdef-left) property being animated is 100px. By 40% of the animation duration, <a id="ref-for-propdef-left①"></a>left has animated to 150px. At 60% of the animation duration, <a id="ref-for-propdef-left②"></a>left has animated back to 75px. At the end of the animation cycle, the value of <a id="ref-for-propdef-left③"></a>left has returned to 100px. The diagram below shows the state of the animation if it were given a duration of 10s.

<figure>
<img src="https://drafts.csswg.org/css-animations-1/images/animation1.png" />
<figcaption>Animation states specified by keyframes</figcaption>
</figure>

<a id="issue-73aacf21"></a>

<strong>Issue:</strong>

[](#issue-73aacf21) This specification needs to define how the value is determined from the keyframes, like the section on [Application of transitions](https://drafts.csswg.org/css-transitions/#application) does for CSS Transitions.

### <a id="timing-functions"></a>3.1.  Timing functions for keyframes[](#timing-functions)

A keyframe style rule may also declare the timing function that is to be used as the animation moves to the next keyframe.

<a id="example-a9d2ebec"></a>

<strong>Example:</strong>

[](#example-a9d2ebec)

``` text
@keyframes bounce {

  from {
    top: 100px;
    animation-timing-function: ease-out;
  }

  25% {
    top: 50px;
    animation-timing-function: ease-in;
  }

  50% {
    top: 100px;
    animation-timing-function: ease-out;
  }

  75% {
    top: 75px;
    animation-timing-function: ease-in;
  }

  to {
    top: 100px;
  }

}
```

Five keyframes are specified for the animation named "bounce". Between the first and second keyframe (i.e., between 0% and 25%) an ease-out timing function is used. Between the second and third keyframe (i.e., between 25% and 50%) an ease-in timing function is used. And so on. The effect will appear as an element that moves up the page 50px, slowing down as it reaches its highest point then speeding up as it falls back to 100px. The second half of the animation behaves in a similar manner, but only moves the element 25px up the page.

A timing function specified on the <a id="ref-for-valdef-shape-to③"></a>[to](https://drafts.csswg.org/css-shapes-1/#valdef-shape-to) or 100% keyframe is ignored.

See the <a id="ref-for-propdef-animation-timing-function②"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) property for more information.

## <a id="other-open-issues"></a>11. Other open issues[](#other-open-issues)

<a id="issue-c01277bc"></a>

<strong>Issue:</strong>

[](#issue-c01277bc)Need to [specify how keyframes interact](https://lists.w3.org/Archives/Public/www-style/2015Jul/0391.html).

## <a id="wg-resolutions-pending"></a>12.  Working Group Resolutions that are pending editing[](#wg-resolutions-pending)

<em>This section is informative and temporary.</em>

The editors are currently behind on editing this spec. The following working group resolutions still need to be edited in:

- 2014-09-09 minutes (Antibes f2f)
  - Issue(7335): Detail how/when keyframe values are computed; using [G.beta in dbaron’s mail](http://lists.w3.org/Archives/Public/www-style/2014Aug/0132.html)
  - ~~Agreed that both transitions and animations animate all properties. css-transitions to define animation of non-interoperable/discrete values. They take their starting values below 50% timing function progress, and end values above~~
  - ~~Dynamic changes to animation properties/keyframes. Tab to propose resolution. ([Bug 14713](https://www.w3.org/Bugs/Public/show_bug.cgi?id=14713))~~
  - ~~Negative animation-delay values apply against the active duration of the animation i.e. (animation-duration\*animation-iteration-count). The delay can thus swallow iterations for which no iteration event will be fired. The start/end events are still fired. Even when delay == (-1\*active_duration)~~
  - ~~Fire animation start/end events when animation-duration is zero, with 0 elapsedTime~~
  - ~~If animation-iteration-count is infinite and duration is 0, treat the iteration-count as if it was finite and run a 0s second (option A in [Brian’s mail](http://lists.w3.org/Archives/Public/www-style/2014Sep/0056.html))~~
  - ~~If an animation with a negative animation delay is initially paused, the start event still fires~~
- 2012-10-29 minutes
  - ~~Change the animation properties to be dynamically changeable~~
  - ~~&#64;keyframes can be dynamically changed~~
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
