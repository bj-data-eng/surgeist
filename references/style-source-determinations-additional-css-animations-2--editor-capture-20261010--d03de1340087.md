Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Animations Module Level 2](https://drafts.csswg.org/css-animations-2/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10 America/New_York (UTC-04:00). Capture time: `2026-10-11T02:56:27.815952+00:00` UTC; Source status: Editor’s Draft, 18 August 2026.

Full captured HTML SHA-256: `f99950324f4253347cd9481a211edd23ae7b7db90f866782d87f84098c347b6c` (326742 bytes). Conversion input SHA-256: `d03de13400872cebbcf97fd39c63ef14cab43289aca79df4ac18da104bdff67f` (63184 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `delta`, `animation-composite-order`, `keyframe-processing`, `animation-timing-function`, `animation-composition`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Animations Module Level 2

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 18 August 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-animations-2/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-animations-2/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2023/WD-css-animations-2-20230602/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-animations-2)

[Inline In Spec](https://drafts.csswg.org/css-animations-2/#issues-index)

<strong>Editors:</strong>

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Brian Birtles](mailto:brian@birchill.co.jp) (Invited Expert)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-animations-2/Overview.bs)

<strong>Issues List:</strong>

<div>https://github.com/w3c/csswg-drafts/labels/css-animations-2</div>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="delta"></a>1. Delta specification[](#delta)

This is a delta specification, meaning that it currently contains only the differences from CSS Animations Level 1 [\[CSS3-ANIMATIONS\]](https://drafts.csswg.org/css-animations-2/#biblio-css3-animations). Once the Level 1 specification is closer to complete, it will be merged with the additions here into a complete level 2 specification.

### <a id="animation-composite-order"></a>2.2. Animation composite order[](#animation-composite-order)

<a id="ref-for-concept-animation"></a>[Animations](https://drafts.csswg.org/web-animations-1/#concept-animation) generated from the markup defined in this specification have an <a id="ref-for-animation-class"></a>[animation class](https://drafts.csswg.org/web-animations-1/#animation-class) of ‘CSS Animation’.

CSS Animations <em>with</em> an <a id="ref-for-owning-element⑥"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element) have a <em>later</em> composite order than CSS Transitions but an <em>earlier</em> composite order than animations without a specific <a id="ref-for-animation-class①"></a>[animation class](https://drafts.csswg.org/web-animations-1/#animation-class).

Within the set of CSS Animations <em>with</em> an <a id="ref-for-owning-element⑦"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element), two animations <var>A</var> and <var>B</var> are sorted in composite order (first to last) as follows:

1.  If the <a id="ref-for-owning-element⑧"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element) of <var>A</var> and <var>B</var> differs, sort <var>A</var> and <var>B</var> by <a id="ref-for-concept-tree-order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) of their corresponding <a id="ref-for-owning-element⑨"></a>owning elements. With regard to pseudo-elements, the sort order is as follows:

    - element

    - ::marker

    - ::before

    - any other pseudo-elements not mentioned specifically in this list, sorted in ascending order by the Unicode codepoints that make up each selector

    - ::after

    - element children

2.  Otherwise, sort <var>A</var> and <var>B</var> based on their position in the computed value of the <a id="ref-for-propdef-animation-name⑤"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property of the (common) <a id="ref-for-owning-element①⓪"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element).

When determining the composite order in order to sort [animation events](https://drafts.csswg.org/css-animations-2/#events) where either or both of the events is an <code><a id="ref-for-eventdef-globaleventhandlers-animationcancel"></a>[animationcancel](https://drafts.csswg.org/css-animations-1/#eventdef-globaleventhandlers-animationcancel)</code> event, treat the CSS Animation(s) for which the <code><a id="ref-for-eventdef-globaleventhandlers-animationcancel①"></a>[animationcancel](https://drafts.csswg.org/css-animations-1/#eventdef-globaleventhandlers-animationcancel)</code> event was generated as having an <a id="ref-for-owning-element①①"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element) corresponding to the owning element in use at the moment when the CSS Animation was cancelled. Furthermore, use the position of the animation in the <a id="ref-for-propdef-animation-name⑥"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property in effect at the time when the CSS Animation was cancelled sorting such that positions of cancelled animations sort before positions of animations that have not been cancelled.

The composite order of CSS Animations <em>without</em> an <a id="ref-for-owning-element①②"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element) is based on their position in the <a id="ref-for-global-animation-list"></a>[global animation list](https://drafts.csswg.org/web-animations-1/#global-animation-list).

<a id="issue-f7fefefa"></a>

<strong>Issue:</strong>

[](#issue-f7fefefa) This differs from the behavior defined for transitions. We should probably sort transitions first, then animation, then use the global animation list. The reason being that when developer tools etc. hang on to orphaned animations and transitions in order to replay them, they should maintain roughly the same composite order.

CSS Animations generated using the markup defined in this specification are <em>not</em> added to the <a id="ref-for-global-animation-list①"></a>[global animation list](https://drafts.csswg.org/web-animations-1/#global-animation-list) when they are created. Instead, these animations are appended to the <a id="ref-for-global-animation-list②"></a>global animation list at the first moment when they transition out of the <a id="ref-for-play-state-idle"></a>[idle](https://drafts.csswg.org/web-animations-1/#play-state-idle) play state after being disassociated from their <a id="ref-for-owning-element①③"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element). CSS Animations that have been disassociated from their <a id="ref-for-owning-element①④"></a>owning element but are still <a id="ref-for-play-state-idle①"></a>idle do not have a defined composite order.

<strong>Note:</strong>

Note, this behavior relies on the fact that disassociating an animation from its <a id="ref-for-owning-element①⑤"></a>[owning element](https://drafts.csswg.org/css-animations-2/#owning-element) always causes it to enter (or remain) in the <a id="ref-for-play-state-idle②"></a>[idle](https://drafts.csswg.org/web-animations-1/#play-state-idle) play state.

### <a id="keyframe-processing"></a>3.2.  Processing Keyframes[](#keyframe-processing)

For each animation effect defined by the <var>N</var>th item in the <a id="ref-for-coordinated-value-list"></a>[coordinated value list](https://drafts.csswg.org/css-values-4/#coordinated-value-list) of the animation-\* properties on target (pseudo-)element <var>element</var>, its associated <a id="ref-for-keyframe"></a>[keyframes](https://drafts.csswg.org/web-animations-1/#keyframe) are generated as follows:

1.  <strong>Set Defaults:</strong>

    - Let <var>default timing function</var> be the corresponding <a id="ref-for-computed-value"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-animation-timing-function①"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) on <var>element</var>.

    - Let <var>default composite</var> be the corresponding <a id="ref-for-computed-value①"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-animation-composition"></a>[animation-composition](#propdef-animation-composition) on <var>element</var>.

    - Let <var>keyframes</var> be an empty sequence of <a id="ref-for-keyframe①"></a>[keyframe](https://drafts.csswg.org/web-animations-1/#keyframe) objects, each possessing a <var>keyframe offset</var>, <var>keyframe timing function</var>, <var>keyframe composite</var>, and <var>keyframe values</var>.

    - Let <var>animated properties</var> be an empty set of CSS <a id="ref-for-css-declaration-property-name"></a>[property names](https://drafts.csswg.org/cssom-1/#css-declaration-property-name).

2.  <strong>Collect Declared Keyframes:</strong>

    1.  Find the last <a id="ref-for-at-ruledef-keyframes⑤"></a>[&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) at-rule in document order with <a id="ref-for-typedef-keyframes-name"></a>[\<keyframes-name\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframes-name) matching the corresponding <a id="ref-for-propdef-animation-name⑦"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) value <var>name</var>.

        If there is no <a id="ref-for-at-ruledef-keyframes⑥"></a>[&#64;keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) at-rule with <a id="ref-for-typedef-keyframes-name①"></a>[\<keyframes-name\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframes-name) matching <var>name</var> (or if <var>name</var> is <a id="ref-for-valdef-animation-name-none"></a>[none](https://drafts.csswg.org/css-animations-1/#valdef-animation-name-none)), abort this procedure. In this case no animation is generated, and any existing animation matching <var>name</var> is canceled.

    2.  Group together all <a id="ref-for-typedef-keyframe-block"></a>[\<keyframe-block\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframe-block) declarations that share the same <a id="ref-for-specified-value"></a>[specified](https://drafts.csswg.org/css-cascade-5/#specified-value) <a id="ref-for-typedef-keyframe-selector"></a>[\<keyframe-selector\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframe-selector) (treating from as 0% and to as 100%), last declared <a id="ref-for-propdef-animation-timing-function②"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) <a id="ref-for-computed-value②"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) (defaulting to <var>default timing function</var> if there is no such declaration), and last declared <a id="ref-for-propdef-animation-composition①"></a>[animation-composition](#propdef-animation-composition) <a id="ref-for-computed-value③"></a>computed value (defaulting to <var>default composite</var> if there is no such declaration).

    3.  For each such group of matching <a id="ref-for-typedef-keyframe-block①"></a>[\<keyframe-block\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframe-block) declarations, ordered by their earliest <a id="ref-for-typedef-keyframe-block②"></a>\<keyframe-block\> in the sorted order:

        1.  <a id="ref-for-cascade"></a>[Cascade](https://drafts.csswg.org/css-cascade-6/#cascade) together all of its <a id="ref-for-x14"></a>[declaration blocks](https://www.w3.org/TR/CSS2/syndata.html#x14) such that for each CSS property (except those that are “not animatable”, which must be ignored) the last declaration among all its <a id="ref-for-typedef-keyframe-block③"></a>[\<keyframe-block\>](https://drafts.csswg.org/css-animations-1/#typedef-keyframe-block) declarations takes precedence. [\[CSS-CASCADE-4\]](https://drafts.csswg.org/css-animations-2/#biblio-css-cascade-4)

            Note: The <a id="ref-for-cascade①"></a>[cascade](https://drafts.csswg.org/css-cascade-6/#cascade) will expand <a id="ref-for-shorthand-property"></a>[shorthand properties](https://drafts.csswg.org/css-cascade-5/#shorthand-property) into their <a id="ref-for-longhand"></a>[sub-properties](https://drafts.csswg.org/css-cascade-5/#longhand) and map together corresponding property pairs in each <a id="ref-for-logical-property-group"></a>[logical property group](https://drafts.csswg.org/css-logical-1/#logical-property-group) according to the <var>element</var>’s <a id="ref-for-computed-value④"></a>[computed](https://drafts.csswg.org/css-cascade-5/#computed-value) <a id="ref-for-writing-mode"></a>[writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode).

        2.  Append to <var>keyframes</var> a new empty <a id="ref-for-keyframe②"></a>[keyframe](https://drafts.csswg.org/web-animations-1/#keyframe) with the group’s <var>keyframe offset</var>, <var>keyframe timing function</var>, and <var>keyframe composite</var>. Give its <var>keyframe values</var> the set of <a id="ref-for-declared-value"></a>[declared values](https://drafts.csswg.org/css-cascade-5/#declared-value) resulting from this cascade.

        3.  Add each <a id="ref-for-css-declaration-property-name①"></a>[property name](https://drafts.csswg.org/cssom-1/#css-declaration-property-name) that was added to its <code><a id="ref-for-dom-csskeyframesrule-cssrules"></a>[cssRules](https://drafts.csswg.org/css-animations-1/#dom-csskeyframesrule-cssrules)</code> to <var>animated properties</var>.

3.  <strong>Generate Initial and Final Frames:</strong>

    1.  Find or create the <var>initial keyframe</var>, a <a id="ref-for-keyframe③"></a>[keyframe](https://drafts.csswg.org/web-animations-1/#keyframe) with a <var>keyframe offset</var> of 0%, <var>default timing function</var> as its <var>keyframe timing function</var>, and <var>default composite</var> as its <var>keyframe composite</var>.

    2.  For any property in <var>animated properties</var> that is not otherwise present in a keyframe with an offset of 0% or one that would be positioned earlier in the <a id="ref-for-used-keyframe-order"></a>[used keyframe order](#used-keyframe-order), add the <a id="ref-for-computed-value⑤"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of that property on <var>element</var> to <var>initial keyframe</var>’s <var>keyframe values</var>.

    3.  If <var>initial keyframe</var>’s <var>keyframe values</var> is not empty, prepend <var>initial keyframe</var> to <var>keyframes</var>.

    4.  Repeat for <var>final keyframe</var>, using an offset of 100%, considering keyframes positioned <em>later</em> in the <a id="ref-for-used-keyframe-order①"></a>[used keyframe order](#used-keyframe-order), and <em>appending</em> to <var>keyframes</var>.

4.  <strong>Sort Frames:</strong>

    - The <a id="specified-keyframe-order"></a><strong>specified order</strong> of <var>keyframes</var> is the order resulting from the steps above, i.e. document order with duplicate keyframes collapsed to the earliest position.

    - The <a id="computed-keyframe-order"></a><strong>computed order</strong> of <var>keyframes</var>—​which is the order returned by <code><a id="ref-for-dom-keyframeeffect-getkeyframes"></a>[getKeyframes()](https://drafts.csswg.org/web-animations-1/#dom-keyframeeffect-getkeyframes)</code>—​is found by shifting any keyframes whose offset was specified as a <a id="ref-for-percentage-value"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), from keyword, or to keyword to the front of the list (after the generated <var>initial keyframe</var>, if any), and performing a stable sort on these keyframes by their <var>keyframe offset</var>s.

    - The <a id="used-keyframe-order"></a><strong>used order</strong> of <var>keyframes</var>—​which is the order used to interpolate and compute the actual animation frames—​is found by attaching the <var>keyframes</var> onto the animation effect’s timeline assuming an <a id="ref-for-iteration-count"></a>[iteration count](https://drafts.csswg.org/web-animations-1/#iteration-count) of 1 and ordering them from earliest to latest, breaking ties by using the <a id="ref-for-computed-keyframe-order"></a>[computed keyframe order](#computed-keyframe-order).

    <a id="issue-67663de2"></a>

    <strong>Issue:</strong>

    [](#issue-67663de2) Any specific requirements on sorting computed keyframes introduced by this spec should be integrated into [Web Animations § 5.3.3 Calculating computed keyframes](https://drafts.csswg.org/web-animations-1/#calculating-computed-keyframes). Any specific requirements on used keyframes introduced by this spec should be integrated into [Web Animations § 5.3.4 The effect value of a keyframe effect](https://drafts.csswg.org/web-animations-1/#the-effect-value-of-a-keyframe-animation-effect). The above description of the distinction between these sets of keyframes should be moved to an informative note.

    Note: Although the <a id="ref-for-computed-keyframe-order①"></a>[computed keyframe order](#computed-keyframe-order) sorts keyframes with <a id="ref-for-percentage-value①"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) offsets, it maintains keyframes specified with a <a id="ref-for-typedef-timeline-range-name"></a>[\<timeline-range-name\>](https://drafts.csswg.org/scroll-animations-1/#typedef-timeline-range-name) in their <a id="ref-for-specified-keyframe-order"></a>[specified keyframe order](#specified-keyframe-order)—​after any <a id="ref-for-percentage-value②"></a>\<percentage\> keyframes (other than a generated <var>final keyframe</var>), even if these come later in the <a id="ref-for-used-keyframe-order②"></a>[used keyframe order](#used-keyframe-order).

### <a id="animation-timing-function"></a>4.2. The <a id="ref-for-propdef-animation-timing-function③"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) property[](#animation-timing-function)

The <a id="ref-for-propdef-animation-timing-function④"></a>[animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function) is used to determine the <a id="ref-for-easing-function"></a>[timing function](https://drafts.csswg.org/css-easing-2/#easing-function) applied to each <a id="ref-for-keyframe④"></a>[keyframe](https://drafts.csswg.org/web-animations-1/#keyframe) as defined in [§ 3 Assembling Keyframes](https://drafts.csswg.org/css-animations-2/#keyframes).

### <a id="animation-composition"></a>4.10. The <a id="ref-for-propdef-animation-composition②"></a>[animation-composition](#propdef-animation-composition) property[](#animation-composition)

The <a id="ref-for-propdef-animation-composition③"></a>[animation-composition](#propdef-animation-composition) property defines the <a id="ref-for-composite-operation"></a>[composite operation](https://drafts.csswg.org/web-animations-1/#composite-operation) used when multiple animations affect the same property simultaneously.

| Field                                                                                    | Definition                                                                                                                                                                                                                |
|------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-animation-composition"></a><strong>animation-composition</strong>                                                                                                                                          |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-typedef-single-animation-composition"></a>[\<single-animation-composition\>](#typedef-single-animation-composition)<a id="ref-for-mult-comma④"></a>[\#](https://drafts.csswg.org/css-values-4/#mult-comma) |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | replace                                                                                                                                                                                                                   |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                       |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                        |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                                                                                                       |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | list, each item a keyword as specified                                                                                                                                                                                    |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                               |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | not animatable                                                                                                                                                                                                            |

<a id="typedef-single-animation-composition"></a><strong>\<single-animation-composition\></strong> = replace <a id="ref-for-comb-one①"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) add <a id="ref-for-comb-one②"></a>\| accumulate

The values of <a id="ref-for-propdef-animation-composition④"></a>[animation-composition](#propdef-animation-composition) have the meaning defined for the corresponding values of the <a id="ref-for-composite-operation①"></a>[composite operation](https://drafts.csswg.org/web-animations-1/#composite-operation) defined in Web Animations [\[WEB-ANIMATIONS\]](https://drafts.csswg.org/css-animations-2/#biblio-web-animations).

When specified in a keyframe, <a id="ref-for-propdef-animation-composition⑤"></a>[animation-composition](#propdef-animation-composition) defines the composite operation to use for each property specified in that keyframe until the next keyframe specifying each property.

<a id="example-05aa5a15"></a>

<strong>Example:</strong>

[](#example-05aa5a15) For example, the following stylesheet defines two different animations targeting the <a id="ref-for-propdef-scale"></a>[scale](https://drafts.csswg.org/css-transforms-2/#propdef-scale) property.

``` text
@keyframes heartbeat {
  from {
    scale: 1;
    animation-timing-function: ease-out;
  }
  30% {
    scale: 1.3;
  }
}
.heartbeat {
  animation: heartbeat 0.3s 2s infinite;
}

@keyframes throb {
  50% {
    scale: 1.8;
  }
}
.icon:mouseover {
  animation: throb 0.4s add;
}
```

If these two animations are applied to the same element, normally only one animation would apply, but by specifying add as the <a id="ref-for-propdef-animation-composition⑥"></a>[animation-composition](#propdef-animation-composition) on the second animation, the result of the two animations will be combined.

Since CSS Transitions [\[CSS3-TRANSITIONS\]](https://drafts.csswg.org/css-animations-2/#biblio-css3-transitions) have a lower composite order, it is possible to use <a id="ref-for-propdef-animation-composition⑦"></a>[animation-composition](#propdef-animation-composition) to combine CSS Animations with underlying transitions as in the following example.

``` text
.icon {
  filter: blur(20px);
  transition: filter 0.5s;
}
.icon:hover {
  filter: blur(0px);
  animation: brightness-pulse 3s infinite add;
}

@keyframes brightness-pulse {
  0% {
    scale: 1.1;
    filter: brightness(130%);
  }
  10% {
    scale: 1;
    filter: brightness(100%);
  }
}
```

<a id="issue-69776a9f"></a>

<strong>Issue:</strong>

[](#issue-69776a9f) Create pictures of these examples and verify they make sense.
