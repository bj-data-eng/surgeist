Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [Web Animations Module Level 1](https://drafts.csswg.org/web-animations-1/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10 America/New_York (UTC-04:00). Capture time: `2026-10-11T02:56:28.202202+00:00` UTC; Source status: Editor’s Draft, 8 October 2026.

Full captured HTML SHA-256: `13e0307c3c72626740baca4c9d38e30848674a2c6904d5f513e19c70ec6bd58f` (1286635 bytes). Conversion input SHA-256: `92e1d5e27678954ad98240e55991656f127e3b35411f7b3f339f2c2b1cd56579` (111246 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `animating-properties`, `keyframe-effects`, `combining-effects`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

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

### <a id="animating-properties"></a>5.2. Animating properties[](#animating-properties)

Unless otherwise specified, all CSS properties are <a id="concept-animatable"></a><strong>animatable</strong>. How property values combine is defined by the <a id="animation-type"></a><strong>Animation type</strong> line in each property’s property definition table:

<strong><a id="not-animatable"></a><strong>not animatable</strong></strong>

The property is not animatable. It is not processed when listed in an animation keyframe, and is not affected by transitions.

Note: Properties are typically excluded from animation because animating them would create excessive complications. For example, properties defining animation parameters are <a id="ref-for-not-animatable"></a>[not animatable](#not-animatable) since doing so would create complex recursive behavior.

Note: An <a id="ref-for-animation-effect⑥⑨"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect) that targets only properties that are <a id="ref-for-not-animatable①"></a>[not animatable](#not-animatable) will still exhibit the usual behavior for an <a id="ref-for-animation-effect⑦⓪"></a>animation effect such as firing events and delaying the fulfillment of the <a id="ref-for-concept-animation④⑧"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation)’s <a id="ref-for-current-finished-promise①①"></a>[current finished promise](https://drafts.csswg.org/web-animations-1/#current-finished-promise).

<strong><a id="discrete"></a><strong>discrete</strong></strong>

The property’s values cannot be meaningfully combined, thus it is <a id="ref-for-not-additive"></a>[not additive](https://drafts.csswg.org/css-values-4/#not-additive). <a id="ref-for-interpolation"></a>[Interpolation](https://drafts.csswg.org/css-values-4/#interpolation) swaps from <var>V<sub>a</sub></var> to <var>V<sub>b</sub></var> at 50% (<var>p=0.5</var>), i.e.

<span>Vresult= Vstart if p &lt;0.5 Vend if p &#8805;0.5</span>

<strong><a id="by-computed-value"></a><strong>by computed value</strong></strong>

Corresponding individual components of the <a id="ref-for-computed-value"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) are combined (interpolated, added, or accumulated) using the indicated procedure for that value type (see [CSS Values 4 § 3 Combining Values: Interpolation, Addition, and Accumulation](https://drafts.csswg.org/css-values-4/#combining-values)). If the number of components or the types of corresponding components do not match, or if any component value uses <a id="ref-for-discrete"></a>[discrete](#discrete) animation and the two corresponding values do not match, then the property values combine as <a id="ref-for-discrete①"></a>discrete.

<strong><a id="repeatable-list"></a><strong>repeatable list</strong></strong>

Same as <a id="ref-for-by-computed-value"></a>[by computed value](#by-computed-value) except that if the two lists have differing numbers of items, they are first repeated to the least common multiple number of items. Each item is then combined <a id="ref-for-by-computed-value①"></a>by computed value. If a pair of values cannot be combined or if any component value uses <a id="ref-for-discrete②"></a>[discrete](#discrete) animation, then the property values combine as <a id="ref-for-discrete③"></a>discrete.

Note: The repeatable list concept ensures that a list that is conceptually repeated to a certain length (as <a id="ref-for-propdef-background-origin"></a>[background-origin](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-origin) is repeated to the length of the <a id="ref-for-propdef-background-image"></a>[background-image](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-image) list) or repeated infinitely will smoothly transition between any values, and so that the computed value will properly represent the result (and potentially be inherited correctly).

<strong>(See prose)</strong>

Some properties have specific interpolation behavior not covered by the above cases; in this case the animation behavior will be specified explicitly for that property.

The <a id="ref-for-animation-type①"></a>[animation type](#animation-type) of properties that do not yet include an <a id="ref-for-animation-type②"></a>Animation type line in their property definition, is defined in [Appendix A: Animation types of existing properties](https://drafts.csswg.org/web-animations-1/#animation-types).

#### <a id="custom-properties"></a>5.2.1. Custom Properties[](#custom-properties)

For <a id="ref-for-custom-property"></a>[custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) registered using the <code><a id="ref-for-dom-css-registerproperty"></a>[registerProperty()](https://drafts.css-houdini.org/css-properties-values-api-1/#dom-css-registerproperty)</code> method for the <a id="ref-for-current-global-object①"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object), the <a id="ref-for-animation-type③"></a>[animation type](#animation-type) is <a id="ref-for-by-computed-value②"></a>[by computed value](#by-computed-value), derived from the type used in the property’s <a id="ref-for-syntax-definition"></a>[syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#syntax-definition). Where there is no <a id="ref-for-computed-value①"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) type that corresponds to the property’s specified syntax (e.g. when the syntax is the <a id="ref-for-universal-syntax-definition"></a>[universal syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#universal-syntax-definition)) or when the <a id="ref-for-custom-property①"></a>custom property is not registered, the <a id="ref-for-animation-type④"></a>animation type is <a id="ref-for-discrete④"></a>[discrete](#discrete).

### <a id="keyframe-effects"></a>5.3. Keyframe effects[](#keyframe-effects)

<a id="keyframe-effect"></a><strong>Keyframe effects</strong> are a kind of <a id="ref-for-animation-effect⑦①"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect) that uses the output of the timing model to update CSS properties of an element or <a id="ref-for-pseudo-element①"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) (such as <code>&#58;&#58;before</code> or <code>&#58;&#58;after</code> [\[select\]](https://drafts.csswg.org/web-animations-1/#biblio-select)) referred to as the <a id="keyframe-effect-effect-target"></a><strong>effect target</strong>.

The <a id="ref-for-keyframe-effect-effect-target②"></a>[effect target](#keyframe-effect-effect-target) is comprised of an <code><a id="ref-for-element"></a>[Element](https://dom.spec.whatwg.org/#element)</code> known as the <a id="effect-target-target-element"></a><strong>target element</strong> and a <a id="ref-for-pseudo-element②"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element) selector known as the <a id="effect-target-target-pseudo-selector"></a><strong>target pseudo-selector</strong>. If the <a id="ref-for-keyframe-effect-effect-target③"></a>effect target is an <code><a id="ref-for-element①"></a>[Element](https://dom.spec.whatwg.org/#element)</code>, the <a id="ref-for-effect-target-target-element"></a>[target element](#effect-target-target-element) is that element and the <a id="ref-for-effect-target-target-pseudo-selector"></a>[target pseudo-selector](#effect-target-target-pseudo-selector) is null. If the <a id="ref-for-keyframe-effect-effect-target④"></a>effect target is a <a id="ref-for-pseudo-element③"></a>pseudo-element, the <a id="ref-for-effect-target-target-element①"></a>target element is its <a id="ref-for-originating-element"></a>[originating element](https://drafts.csswg.org/selectors-4/#originating-element) and the <a id="ref-for-effect-target-target-pseudo-selector①"></a>target pseudo-selector is one that specifies that particular <a id="ref-for-pseudo-element④"></a>pseudo-element.

Note that not all <a id="ref-for-keyframe-effect-effect-target⑤"></a>[effect targets](#keyframe-effect-effect-target) specified in this manner (such as <a id="ref-for-selectordef-part"></a>[::part()](https://drafts.csswg.org/css-shadow-1/#selectordef-part) pseudo-elements and unsupported pseudo-elements) have computed property values defined.

#### <a id="keyframes-section"></a>5.3.1. Keyframes[](#keyframes-section)

The <a id="ref-for-effect-value"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) for a <a id="ref-for-keyframe-effect③"></a>[keyframe effect](#keyframe-effect) are calculated by interpolating between a series of property values positioned at fractional offsets. Each set of property values indexed by an offset is called a <a id="keyframe"></a><strong>keyframe</strong>.

The <a id="keyframe-offset"></a><strong>offset of a keyframe</strong> is a value in the range \[0, 1\] or the special value null. The list of <a id="ref-for-keyframe"></a>[keyframes](#keyframe) for a <a id="ref-for-keyframe-effect④"></a>[keyframe effect](#keyframe-effect) must be <a id="loosely-sorted-by-offset"></a><strong>loosely sorted by offset</strong>, which means that for each <a id="ref-for-keyframe①"></a>keyframe in the list that has a <a id="ref-for-keyframe-offset"></a>[keyframe offset](#keyframe-offset) that is not null, the offset must be greater than or equal to the offset of the previous <a id="ref-for-keyframe②"></a>keyframe in the list with a <a id="ref-for-keyframe-offset①"></a>keyframe offset that is not null, if any.

The behavior when <a id="ref-for-keyframe③"></a>[keyframes](#keyframe) overlap or have unsupported values is defined in [§ 5.3.4 The effect value of a keyframe effect](#the-effect-value-of-a-keyframe-animation-effect).

Each keyframe also has a <a id="keyframe-specific-easing-function"></a><strong>keyframe-specific easing function</strong> associated with it, which is an <a id="ref-for-easing-function⑤"></a>[easing function](https://drafts.csswg.org/css-easing-2/#easing-function) that is applied to the period of time between the keyframe on which it is specified and the <em>next</em> keyframe in the list.

Note: The <a id="ref-for-easing-function⑥"></a>[easing function](https://drafts.csswg.org/css-easing-2/#easing-function) specified on the last keyframe in the list is never applied.

Each <a id="ref-for-keyframe④"></a>[keyframe](#keyframe) may have a <a id="keyframe-specific-composite-operation"></a><strong>keyframe-specific composite operation</strong> that, if set, is applied to all values specified in that <a id="ref-for-keyframe⑤"></a>keyframe. The possible operations and their meanings are identical to those defined for the <a id="ref-for-composite-operation"></a>[composite operation](#composite-operation) associated with the <a id="ref-for-keyframe-effect⑤"></a>[keyframe effect](#keyframe-effect) as a whole in [§ 5.4.4 Effect composition](#effect-composition). If the <a id="ref-for-keyframe-specific-composite-operation"></a>[keyframe-specific composite operation](#keyframe-specific-composite-operation) for a <a id="ref-for-keyframe⑥"></a>keyframe is not set, the <a id="ref-for-composite-operation①"></a>composite operation specified for the <a id="ref-for-keyframe-effect⑥"></a>keyframe effect as a whole is used for values specified in that keyframe.

Tests

- [applying-interpolated-transform.html](https://wpt.fyi/results/web-animations/animation-model/combining-effects/applying-interpolated-transform.html) [(live test)](http://wpt.live/web-animations/animation-model/combining-effects/applying-interpolated-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/combining-effects/applying-interpolated-transform.html)

#### <a id="computing-property-values"></a>5.3.2. Computing property values[](#computing-property-values)

To <a id="compute-a-property-value"></a><strong>compute a property value</strong> given a property <var>property</var>, a value <var>value</var>, and an <code><a id="ref-for-element②"></a>[Element](https://dom.spec.whatwg.org/#element)</code> <var>element</var>: resolve <var>value</var> according to the “Computed Value” line of the <var>property</var>’s definition table, using the <a id="ref-for-computed-value②"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) of <var>element</var> as the context for resolving dependencies, and return the result.

Note: The <a id="ref-for-computed-value③"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) on <var>element</var> are not affected by this algorithm.

This algorithm implies that property values specified in keyframes can establish order dependencies. When <a id="ref-for-compute-a-property-value"></a>[computing a property value](#compute-a-property-value), the <a id="ref-for-computed-value④"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) of dependencies held by <var>value</var> must be calculated <em>first</em>.

<a id="example-62f40e1a"></a>

<strong>Example:</strong>

[](#example-62f40e1a)

``` text
var animation = elem.animate([{ fontSize: '10px', width: '10em' },
                              { fontSize: '20px', width: '20em' }], 1000);
animation.currentTime = 500;
console.log(getComputedStyle(elem).fontSize); // Should be 15px
console.log(getComputedStyle(elem).width); // Should be 225px
```

In this example, in order to <a id="ref-for-compute-a-property-value①"></a>[compute a property value](#compute-a-property-value) for <code>10em</code>, we need to know the <a id="ref-for-computed-value⑤"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of <a id="ref-for-propdef-font-size"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) on the <a id="ref-for-effect-target-target-element②"></a>[target element](#effect-target-target-element), which in turn is determined by the <a id="ref-for-effect-value①"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) for <a id="ref-for-propdef-font-size①"></a>font-size, which in turn depends on computing property values for <a id="ref-for-propdef-font-size②"></a>font-size. Hence, computing property values are subject to ordering constraints.

#### <a id="calculating-computed-keyframes"></a>5.3.3. Calculating computed keyframes[](#calculating-computed-keyframes)

Before calculating the <a id="ref-for-effect-value②"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) of a <a id="ref-for-keyframe-effect⑦"></a>[keyframe effect](#keyframe-effect), the property values on its <a id="ref-for-keyframe⑦"></a>[keyframes](#keyframe) are [computed](#computing-property-values), and the offset to use for any keyframes with a null <a id="ref-for-keyframe-offset②"></a>[keyframe offset](#keyframe-offset) is computed. The result of resolving these values is a set of <a id="computed-keyframes"></a><strong>computed keyframes</strong>.

The calculated <a id="ref-for-keyframe-offset③"></a>[keyframe offsets](#keyframe-offset) of a set of <a id="ref-for-keyframe⑧"></a>[keyframe](#keyframe) that includes suitable values for each null <a id="ref-for-keyframe-offset④"></a>keyframe offset are referred to as the <a id="computed-keyframe-offsets"></a><strong>computed keyframe offsets</strong>.

To produce <a id="ref-for-computed-keyframe-offsets"></a>[computed keyframe offsets](#computed-keyframe-offsets), we define a procedure to <a id="compute-missing-keyframe-offsets"></a><strong>compute missing keyframe offsets</strong> that takes a sequence of <a id="ref-for-keyframe⑨"></a>[keyframes](#keyframe), <var>keyframes</var>, and has the following steps:

1.  For each <a id="ref-for-keyframe①⓪"></a>[keyframe](#keyframe) in <var>keyframes</var>, let the <a id="ref-for-computed-keyframe-offsets①"></a>[computed keyframe offset](#computed-keyframe-offsets) of the <a id="ref-for-keyframe①①"></a>keyframe be equal to its <a id="ref-for-keyframe-offset⑤"></a>[keyframe offset](#keyframe-offset) value.

2.  If <var>keyframes</var> contains more than one <a id="ref-for-keyframe①②"></a>[keyframe](#keyframe) and the <a id="ref-for-computed-keyframe-offsets②"></a>[computed keyframe offset](#computed-keyframe-offsets) of the first <a id="ref-for-keyframe①③"></a>keyframe in <var>keyframes</var> is null, set the <a id="ref-for-computed-keyframe-offsets③"></a>computed keyframe offset of the first <a id="ref-for-keyframe①④"></a>keyframe to 0.

3.  If the <a id="ref-for-computed-keyframe-offsets④"></a>[computed keyframe offset](#computed-keyframe-offsets) of the last <a id="ref-for-keyframe①⑤"></a>[keyframe](#keyframe) in <var>keyframes</var> is null, set its <a id="ref-for-computed-keyframe-offsets⑤"></a>computed keyframe offset to 1.

4.  For each pair of <a id="ref-for-keyframe①⑥"></a>[keyframes](#keyframe) <var>A</var> and <var>B</var> where:

    - <var>A</var> appears before <var>B</var> in <var>keyframes</var>, and

    - <var>A</var> and <var>B</var> have a <a id="ref-for-computed-keyframe-offsets⑥"></a>[computed keyframe offset](#computed-keyframe-offsets) that is not null, and

    - all <a id="ref-for-keyframe①⑦"></a>[keyframes](#keyframe) between <var>A</var> and <var>B</var> have a null <a id="ref-for-computed-keyframe-offsets⑦"></a>[computed keyframe offset](#computed-keyframe-offsets),

    calculate the <a id="ref-for-computed-keyframe-offsets⑧"></a>[computed keyframe offset](#computed-keyframe-offsets) of each <a id="ref-for-keyframe①⑧"></a>[keyframe](#keyframe) between <var>A</var> and <var>B</var> as follows:

    1.  Let <a id="offsetk"></a><strong>offset<sub><var>k</var></sub></strong> be the <a id="ref-for-computed-keyframe-offsets⑨"></a>[computed keyframe offset](#computed-keyframe-offsets) of a <a id="ref-for-keyframe①⑨"></a>[keyframe](#keyframe) <var>k</var>.

    2.  Let <var>n</var> be the number of keyframes <em>between</em> and including <var>A</var> and <var>B</var> minus 1.

    3.  Let <var>index</var> refer to the position of <var>keyframe</var> in the sequence of keyframes between <var>A</var> and <var>B</var> such that the first keyframe after <var>A</var> has an <var>index</var> of 1.

    4.  Set the <a id="ref-for-computed-keyframe-offsets①⓪"></a>[computed keyframe offset](#computed-keyframe-offsets) of <var>keyframe</var> to <a id="ref-for-offsetk"></a>[offset](#offsetk)<sub><var>A</var></sub> + (<a id="ref-for-offsetk①"></a>offset<sub><var>B</var></sub> − <a id="ref-for-offsetk②"></a>offset<sub><var>A</var></sub>) × <var>index</var> / <var>n</var>.

Tests

- [background-shorthand.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/background-shorthand.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/background-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/background-shorthand.html)
- [computed-keyframes-shorthands.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/computed-keyframes-shorthands.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/computed-keyframes-shorthands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/computed-keyframes-shorthands.html)
- [effect-value-context-filling.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-context-filling.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-context-filling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-context-filling.html)
- [effect-value-context.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-context.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-context.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-context.html)

<a id="ref-for-computed-keyframes"></a>[Computed keyframes](#computed-keyframes) are produced using the following procedure. Note that this procedure is only performed on a <a id="ref-for-keyframe-effect⑧"></a>[keyframe effect](#keyframe-effect) having an <a id="ref-for-keyframe-effect-effect-target⑥"></a>[effect target](#keyframe-effect-effect-target) for which computed property values can be calculated.

1.  Let <var>computed keyframes</var> be an empty list of <a id="ref-for-keyframe②⓪"></a>[keyframes](#keyframe).

2.  For each <var>keyframe</var> in the list of <a id="ref-for-keyframe②①"></a>[keyframes](#keyframe) specified on this <a id="ref-for-keyframe-effect⑨"></a>[keyframe effect](#keyframe-effect), perform the following steps:

    1.  Add a new empty <a id="ref-for-keyframe②②"></a>[keyframe](#keyframe), <var>computed keyframe</var>, to <var>computed keyframes</var>.

    2.  For each property specified in <var>keyframe</var>:

        - <a id="ref-for-compute-a-property-value②"></a>[Compute a property value](#compute-a-property-value) using the value specified on <var>keyframe</var> as the value, and the <a id="ref-for-effect-target-target-element③"></a>[target element](#effect-target-target-element) as the element; then add the property and resulting value to <var>computed keyframe</var>.

        - For shorthand properties, add the equivalent longhand properties.

        - For logical properties [\[CSS-LOGICAL-1\]](https://drafts.csswg.org/web-animations-1/#biblio-css-logical-1), add the <a id="ref-for-logical-to-physical"></a>[equivalent physical properties](https://drafts.csswg.org/css-writing-modes-4/#logical-to-physical) [\[CSS-WRITING-MODES-4\]](https://drafts.csswg.org/web-animations-1/#biblio-css-writing-modes-4) based on the computed value of <a id="ref-for-propdef-writing-mode"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode) and/or <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) for the <a id="ref-for-keyframe-effect-effect-target⑦"></a>[effect target](#keyframe-effect-effect-target).

        For example, if <var>keyframe</var> has a value of 12pt for the <a id="ref-for-propdef-border-width"></a>[border-width](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-width) <a id="ref-for-shorthand-property"></a>[shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property), the user agent will <a id="ref-for-compute-a-property-value③"></a>[compute a property value](#compute-a-property-value) of 16px for each of the <a id="ref-for-longhand"></a>[longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand): <a id="ref-for-propdef-border-bottom-width"></a>[border-bottom-width](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-bottom-width), <a id="ref-for-propdef-border-left-width"></a>[border-left-width](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-left-width), <a id="ref-for-propdef-border-right-width"></a>[border-right-width](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-right-width), and <a id="ref-for-propdef-border-top-width"></a>[border-top-width](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-top-width). As a result, <var>computed keyframe</var> would <em>not</em> have a value for the <a id="ref-for-propdef-border-width①"></a>border-width property, but would instead include each of the longhand properties, each with the value 16px.

        If conflicts arise when expanding shorthand properties or replacing logical properties with physical properties, apply the following rules in order until the conflict is resolved:

        1.  Longhand properties override shorthand properties (e.g. <a id="ref-for-propdef-border-top-color"></a>[border-top-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-top-color) overrides <a id="ref-for-propdef-border-top"></a>[border-top](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-top)).

        2.  Shorthand properties with fewer longhand components override those with more longhand components (e.g. <a id="ref-for-propdef-border-top①"></a>[border-top](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-top) overrides <a id="ref-for-propdef-border-color"></a>[border-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-border-color)).

        3.  Physical properties override logical properties.

        4.  For shorthand properties with an equal number of longhand components, properties whose IDL name (see the <a id="ref-for-css-property-to-idl-attribute"></a>[CSS property to IDL attribute](https://drafts.csswg.org/cssom/#css-property-to-idl-attribute) algorithm [\[CSSOM\]](https://drafts.csswg.org/web-animations-1/#biblio-cssom)) appears earlier when sorted in ascending order by the Unicode codepoints that make up each IDL name, override those who appear later.

3.  Apply the procedure to <a id="ref-for-compute-missing-keyframe-offsets"></a>[compute missing keyframe offsets](#compute-missing-keyframe-offsets) to <var>computed keyframes</var>.

4.  Return <var>computed keyframes</var>.

#### <a id="the-effect-value-of-a-keyframe-animation-effect"></a>5.3.4. The effect value of a keyframe effect[](#the-effect-value-of-a-keyframe-animation-effect)

The <a id="ref-for-effect-value③"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) of a single property referenced by a <a id="ref-for-keyframe-effect①⓪"></a>[keyframe effect](#keyframe-effect) as one of its <a id="ref-for-target-property④"></a>[target properties](https://drafts.csswg.org/web-animations-1/#target-property), for a given <var>iteration progress</var>, <var>iteration index</var> and <var>underlying value</var> is calculated as follows.

1.  If <var>iteration progress</var> is <a id="ref-for-unresolved①①①"></a>[unresolved](https://drafts.csswg.org/web-animations-1/#unresolved) abort this procedure.

2.  Let <var>target property</var> be the <a id="ref-for-longhand①"></a>[longhand property](https://drafts.csswg.org/css-cascade-5/#longhand) for which the <a id="ref-for-effect-value④"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) is to be calculated.

3.  If <a id="ref-for-animation-type⑤"></a>[animation type](#animation-type) of the <var>target property</var> is <a id="ref-for-not-animatable②"></a>[not animatable](#not-animatable) abort this procedure since the effect cannot be applied.

4.  If the <a id="ref-for-keyframe-effect①①"></a>[keyframe effect](#keyframe-effect) does not have an <a id="ref-for-keyframe-effect-effect-target⑧"></a>[effect target](#keyframe-effect-effect-target), or if the <a id="ref-for-keyframe-effect-effect-target⑨"></a>effect target cannot have computed property values calculated, abort this procedure.

5.  Let <var>property-specific keyframes</var> be the result of getting the set of <a id="ref-for-computed-keyframes①"></a>[computed keyframes](#computed-keyframes) for this <a id="ref-for-keyframe-effect①②"></a>[keyframe effect](#keyframe-effect).

6.  Remove any <a id="ref-for-keyframe②③"></a>[keyframes](#keyframe) from <var>property-specific keyframes</var> that do not have a property value for <var>target property</var>.

7.  If <var>property-specific keyframes</var> is empty, return <var>underlying value</var>.

8.  If there is no <a id="ref-for-keyframe②④"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> with a <a id="ref-for-computed-keyframe-offsets①①"></a>[computed keyframe offset](#computed-keyframe-offsets) of 0, create a new <a id="ref-for-keyframe②⑤"></a>keyframe with a <a id="ref-for-computed-keyframe-offsets①②"></a>computed keyframe offset of 0, a property value set to <var>underlying value</var>, a <a id="ref-for-keyframe-specific-composite-operation①"></a>[keyframe-specific composite operation](#keyframe-specific-composite-operation) of <a id="ref-for-composite-operation-replace"></a>[replace](#composite-operation-replace), and a <a id="ref-for-keyframe-specific-easing-function①"></a>[keyframe-specific easing function](#keyframe-specific-easing-function) of the <a id="ref-for-linear-easing-function①"></a>[linear easing function](https://drafts.csswg.org/css-easing-2/#linear-easing-function), and prepend it to the beginning of <var>property-specific keyframes</var>.

9.  Similarly, if there is no <a id="ref-for-keyframe②⑥"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> with a <a id="ref-for-computed-keyframe-offsets①③"></a>[computed keyframe offset](#computed-keyframe-offsets) of 1, create a new <a id="ref-for-keyframe②⑦"></a>keyframe with a <a id="ref-for-computed-keyframe-offsets①④"></a>computed keyframe offset of 1, a property value set to <var>underlying value</var>, a <a id="ref-for-keyframe-specific-composite-operation②"></a>[keyframe-specific composite operation](#keyframe-specific-composite-operation) of <a id="ref-for-composite-operation-replace①"></a>[replace](#composite-operation-replace), and a <a id="ref-for-keyframe-specific-easing-function②"></a>[keyframe-specific easing function](#keyframe-specific-easing-function) of the <a id="ref-for-linear-easing-function②"></a>[linear easing function](https://drafts.csswg.org/css-easing-2/#linear-easing-function), and append it to the end of <var>property-specific keyframes</var>.

<strong>Note:</strong>

Other specifications may override the implicit keyframe generation behavior in the preceding two steps. For example, the [named timeline range keyframe selectors](https://drafts.csswg.org/scroll-animations-1/#named-range-keyframes) defined by Scroll Animations can cause these implicit keyframes not to be generated.

A future level of this specification should incorporate the overridden behavior directly into this algorithm.

1.  Let <var>interval endpoints</var> be an empty sequence of keyframes.

2.  Populate <var>interval endpoints</var> by following the steps from the first matching condition below:

    <strong>If <var>iteration progress</var> \< 0 and there is more than one <a id="ref-for-keyframe②⑧"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> with a <a id="ref-for-computed-keyframe-offsets①⑤"></a>[computed keyframe offset](#computed-keyframe-offsets) of 0,</strong>

    Add the first <a id="ref-for-keyframe②⑨"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> to <var>interval endpoints</var>.

    <strong>If <var>iteration progress</var> ≥ 1 and there is more than one <a id="ref-for-keyframe③⓪"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> with a <a id="ref-for-computed-keyframe-offsets①⑥"></a>[computed keyframe offset](#computed-keyframe-offsets) of 1,</strong>

    Add the last <a id="ref-for-keyframe③①"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> to <var>interval endpoints</var>.

    <strong>Otherwise,</strong>

    1.  Append to <var>interval endpoints</var> the last <a id="ref-for-keyframe③②"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> whose <a id="ref-for-computed-keyframe-offsets①⑦"></a>[computed keyframe offset](#computed-keyframe-offsets) is less than or equal to <var>iteration progress</var> and less than 1. If there is no such <a id="ref-for-keyframe③③"></a>keyframe (because, for example, the <a id="ref-for-iteration-progress①⑤"></a>[iteration progress](https://drafts.csswg.org/web-animations-1/#iteration-progress) is negative), add the last <a id="ref-for-keyframe③④"></a>keyframe whose <a id="ref-for-computed-keyframe-offsets①⑧"></a>computed keyframe offset is 0.

    2.  Append to <var>interval endpoints</var> the next <a id="ref-for-keyframe③⑤"></a>[keyframe](#keyframe) in <var>property-specific keyframes</var> after the one added in the previous step.

3.  For each <var>keyframe</var> in <var>interval endpoints</var>:

    1.  If <var>keyframe</var> has a <a id="ref-for-composite-operation②"></a>[composite operation](#composite-operation) that is <em>not</em> <a id="ref-for-composite-operation-replace②"></a>[replace](#composite-operation-replace), or <var>keyframe</var> has no <a id="ref-for-composite-operation③"></a>composite operation and the <a id="ref-for-composite-operation④"></a>composite operation of this <a id="ref-for-keyframe-effect①③"></a>[keyframe effect](#keyframe-effect) is <em>not</em> <a id="ref-for-composite-operation-replace③"></a>replace, then perform the following steps:

        1.  Let <var>composite operation to use</var> be the <a id="ref-for-composite-operation⑤"></a>[composite operation](#composite-operation) of <var>keyframe</var>, or if it has none, the <a id="ref-for-composite-operation⑥"></a>composite operation of this <a id="ref-for-keyframe-effect①④"></a>[keyframe effect](#keyframe-effect).

        2.  Let <var>value to combine</var> be the property value of <var>target property</var> specified on <var>keyframe</var>.

        3.  Replace the property value of <var>target property</var> on <var>keyframe</var> with the result of combining <var>underlying value</var> (<var>V</var><sub>a</sub>) and <var>value to combine</var> (<var>V</var><sub>b</sub>) using the procedure for the <var>composite operation to use</var> corresponding to the <var>target property</var>’s <a id="ref-for-animation-type⑥"></a>[animation type](#animation-type).

4.  If there is only one keyframe in <var>interval endpoints</var> return the property value of <var>target property</var> on that keyframe.

5.  Let <var>start offset</var> be the <a id="ref-for-computed-keyframe-offsets①⑨"></a>[computed keyframe offset](#computed-keyframe-offsets) of the first keyframe in <var>interval endpoints</var>.

6.  Let <var>end offset</var> be the <a id="ref-for-computed-keyframe-offsets②⓪"></a>[computed keyframe offset](#computed-keyframe-offsets) of last keyframe in <var>interval endpoints</var>.

7.  Let <var>interval distance</var> be the result of evaluating <code>(<var>iteration progress</var> − <var>start offset</var>) / (<var>end offset</var> − <var>start offset</var>)</code>.

8.  Let <var>transformed distance</var> be the result of evaluating the <a id="ref-for-easing-function⑦"></a>[easing function](https://drafts.csswg.org/css-easing-2/#easing-function) associated with the first keyframe in <var>interval endpoints</var> passing <var>interval distance</var> as the input progress.

9.  Return the result of applying the <a id="ref-for-interpolation①"></a>[interpolation procedure](https://drafts.csswg.org/css-values-4/#interpolation) defined by the <a id="ref-for-animation-type⑦"></a>[animation type](#animation-type) of the <var>target property</var> to the values of the <var>target property</var> specified on the two keyframes in <var>interval endpoints</var>, taking the first such value as <var>V</var><sub>start</sub> and the second as <var>V</var><sub>end</sub> and using <var>transformed distance</var> as the interpolation parameter <var>p</var>.

Tests

- [effect-value-interval-distance.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-interval-distance.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-interval-distance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-interval-distance.html)
- [effect-value-iteration-composite-operation.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-iteration-composite-operation.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-iteration-composite-operation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-iteration-composite-operation.html)
- [effect-value-opacity-replaced-effect-in-shadow-root.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect-in-shadow-root.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect-in-shadow-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect-in-shadow-root.html)
- [effect-value-opacity-replaced-effect.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-opacity-replaced-effect.html)
- [effect-value-overlapping-keyframes.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-overlapping-keyframes.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-overlapping-keyframes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-overlapping-keyframes.html)
- [effect-value-replaced-animations.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-replaced-animations.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-replaced-animations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-replaced-animations.html)
- [effect-value-transformed-distance.html](https://wpt.fyi/results/web-animations/animation-model/keyframe-effects/effect-value-transformed-distance.html) [(live test)](http://wpt.live/web-animations/animation-model/keyframe-effects/effect-value-transformed-distance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/keyframe-effects/effect-value-transformed-distance.html)

<strong>Note:</strong>

Note that this procedure assumes the following about the list of <a id="ref-for-keyframe③⑥"></a>[keyframes](#keyframe) specified on the effect:

- Each <a id="ref-for-keyframe③⑦"></a>[keyframe](#keyframe) has a specified <a id="ref-for-computed-keyframe-offsets②①"></a>[computed keyframe offset](#computed-keyframe-offsets) in the range \[0, 1\].

- The list of <a id="ref-for-keyframe③⑧"></a>[keyframes](#keyframe) is sorted in ascending order by <a id="ref-for-computed-keyframe-offsets②②"></a>[computed keyframe offset](#computed-keyframe-offsets).

- For a given property, there is at most one specified property value on each keyframe.

It is the responsibility of the user of the model (for example, a declarative markup or programming interface) to ensure these conditions are met.

For example, for the [programming interface](https://drafts.csswg.org/web-animations-1/#programming-interface) defined by this specification, these conditions are met by the procedure to produce the <a id="ref-for-computed-keyframes②"></a>[computed keyframes](#computed-keyframes) that become the input to this procedure.

Note: this procedure permits overlapping <a id="ref-for-keyframe③⑨"></a>[keyframes](#keyframe). The behavior is that at the point of overlap the output value jumps to the value of the last defined <a id="ref-for-keyframe④⓪"></a>keyframe at that offset. For overlapping keyframes at 0 or 1, the output value for <a id="ref-for-iteration-progress①⑥"></a>[iteration progress](https://drafts.csswg.org/web-animations-1/#iteration-progress) values less than 0 or greater than or equal to 1 is the value of the first <a id="ref-for-keyframe④①"></a>keyframe or the last <a id="ref-for-keyframe④②"></a>keyframe in <var>keyframes</var> respectively.

Note that

<strong>Note:</strong>

<a id="ref-for-computed-keyframes③"></a>[computed keyframes](#computed-keyframes) are “live”: user-agents behave as if they are recreated every time the <a id="ref-for-effect-value⑤"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) is calculated.

For example, if there is an ongoing transition on the <a id="ref-for-propdef-font-size③"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) property from <code>10px</code> to <code>20px</code>, a property value specified as <code>1em</code> in a <a id="ref-for-keyframe④③"></a>[keyframe](#keyframe) would during [keyframe computation](#calculating-computed-keyframes) resolve against the <a id="ref-for-computed-value⑥"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) in the range \[<code>10px</code>, <code>20px</code>\] produced by the transition on <a id="ref-for-propdef-font-size④"></a>font-size.

### <a id="combining-effects"></a>5.4. Combining effects[](#combining-effects)

<em>This section is non-normative</em>

After calculating the <a id="ref-for-effect-value⑥"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) for a <a id="ref-for-keyframe-effect①⑤"></a>[keyframe effect](#keyframe-effect), they are applied to the <a id="ref-for-animation-effect⑦②"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect)’s <a id="ref-for-target-property⑤"></a>[target properties](https://drafts.csswg.org/web-animations-1/#target-property).

Since it is possible for multiple <a id="ref-for-in-effect③"></a>[in effect](https://drafts.csswg.org/web-animations-1/#in-effect) <a id="ref-for-keyframe-effect①⑥"></a>[keyframe effects](#keyframe-effect) to target the same property it is often necessary to combine the results of several <a id="ref-for-keyframe-effect①⑦"></a>keyframe effects together. This process is called <a id="composite"></a><strong>compositing</strong> and is based on establishing an <a id="ref-for-effect-stack①"></a>[effect stack](#effect-stack) for each property targeted by an <a id="ref-for-in-effect④"></a>in effect <a id="ref-for-animation-effect⑦③"></a>[animation effect](https://drafts.csswg.org/web-animations-1/#animation-effect).

After <a id="ref-for-composite"></a>[compositing](#composite) the results of <a id="ref-for-keyframe-effect①⑧"></a>[keyframe effects](#keyframe-effect) together, the composited result is combined with other values specified for the <a id="ref-for-target-property⑥"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property).

The arrangement is illustrated below:

<figure>
<img src="https://drafts.csswg.org/web-animations-1/images/animation-cascade.svg" alt="Overview of the application of effect values to their target properties" />
<figcaption>Overview of the application of <a id="ref-for-effect-value⑦"></a><a href="https://drafts.csswg.org/web-animations-1/#effect-value">effect values</a> to their <a id="ref-for-target-property⑦"></a><a href="https://drafts.csswg.org/web-animations-1/#target-property">target properties</a>.<br />
The results of <a id="ref-for-keyframe-effect①⑨"></a><a href="#keyframe-effect">keyframe effects</a> targeting the same property are composited together using an <a id="ref-for-effect-stack②"></a><a href="#effect-stack">effect stack</a>.<br />
The result of this composition is then inserted into the CSS cascade at an appropriate point.</figcaption>
</figure>

For the first part of this operation—​combining <a id="ref-for-effect-value⑧"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) that target the same <a id="ref-for-target-property⑧"></a>[property](https://drafts.csswg.org/web-animations-1/#target-property)—​it is necessary to determine both <em>how</em> <a id="ref-for-keyframe-effect②⓪"></a>[keyframe effects](#keyframe-effect) are combined with one another, as well as the <em>order</em> in which they are applied, that is, their relative <em>composite order</em>.

The matter of <em>how</em> <a id="ref-for-effect-value⑨"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) are combined is governed by the <a id="ref-for-composite-operation⑦"></a>[composite operation](#composite-operation) of the corresponding <a id="ref-for-keyframe-effect②①"></a>[keyframe effects](#keyframe-effect).

The relative <em>composite order</em> of <a id="ref-for-effect-value①⓪"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) is determined by an <a id="ref-for-effect-stack③"></a>[effect stack](#effect-stack) established for each animated property.

#### <a id="animation-classes"></a>5.4.1. Animation classes[](#animation-classes)

This specification provides a common animation model intended to be used by other specifications that define markup or programming interfaces on top of this model. The particular markup or programming interface that generated an <a id="ref-for-concept-animation④⑨"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation) defines its <a id="animation-class"></a><strong>animation class</strong>.

Further specifications may define specialized behavior for composite ordering between different classes of animations or within a particular class.

<em>This section is non-normative</em>

For example, animations whose <a id="ref-for-animation-class①"></a>[class](#animation-class) is “CSS animation” are defined as having a <em>higher</em> composite order than animations whose class is “CSS transition” but <em>lower</em> than other animations without a specific class.

Within the set of “CSS animation” objects, specialized composite ordering is defined based on the <a id="ref-for-propdef-animation-name"></a>[animation-name](https://drafts.csswg.org/css-animations-1/#propdef-animation-name) property amongst other factors.

#### <a id="the-effect-stack"></a>5.4.2. The effect stack[](#the-effect-stack)

An <a id="effect-stack"></a><strong>effect stack</strong> is associated with each property <a id="ref-for-target-property⑨"></a>[targeted](https://drafts.csswg.org/web-animations-1/#target-property) by one or more <a id="ref-for-keyframe-effect②②"></a>[keyframe effects](#keyframe-effect). The <a id="ref-for-effect-stack④"></a>[effect stack](#effect-stack) establishes the relative composite order of <a id="ref-for-keyframe-effect②③"></a>keyframe effects.

The relative <a id="animation-composite-order"></a><strong>composite order</strong> of any two <a id="ref-for-keyframe-effect②④"></a>[keyframe effects](#keyframe-effect) <var>A</var> and <var>B</var> within an <a id="ref-for-effect-stack⑤"></a>[effect stack](#effect-stack) is established by comparing their properties as follows:

1.  Sort <var>A</var> and <var>B</var> by applying the following conditions in turn until the order is resolved,

    1.  If <var>A</var> and <var>B</var>’s <a id="ref-for-animation-effect-associated-animation①③"></a>[associated animations](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) differ by <a id="ref-for-animation-class②"></a>[class](#animation-class), sort by any inter-class composite order as defined for the corresponding classes.

    2.  If <var>A</var> and <var>B</var> are still not sorted, sort by any <a id="ref-for-animation-class③"></a>[class](#animation-class)-specific composite order defined by the common class of <var>A</var> and <var>B</var>’s <a id="ref-for-animation-effect-associated-animation①④"></a>[associated animations](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation).

    3.  If <var>A</var> and <var>B</var> are still not sorted, sort by the position of their <a id="ref-for-animation-effect-associated-animation①⑤"></a>[associated animations](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) in the <a id="ref-for-global-animation-list"></a>[global animation list](https://drafts.csswg.org/web-animations-1/#global-animation-list).

<a id="ref-for-animation-effect⑦④"></a>[Animation effects](https://drafts.csswg.org/web-animations-1/#animation-effect) that sort earlier have <em>lower</em> composite order.

#### <a id="calculating-the-result-of-an-effect-stack"></a>5.4.3. Calculating the result of an effect stack[](#calculating-the-result-of-an-effect-stack)

In order to calculate the final value of an <a id="ref-for-effect-stack⑥"></a>[effect stack](#effect-stack), the <a id="ref-for-effect-value①①"></a>[effect values](https://drafts.csswg.org/web-animations-1/#effect-value) of each <a id="ref-for-keyframe-effect②⑤"></a>[keyframe effect](#keyframe-effect) in the stack are combined in composite order.

Each step in the process of evaluating an <a id="ref-for-effect-stack⑦"></a>[effect stack](#effect-stack) takes an <a id="underlying-value"></a><strong>underlying value</strong> as input.

For each <a id="ref-for-keyframe-effect②⑥"></a>[keyframe effect](#keyframe-effect) in the stack, the appropriate <a id="ref-for-effect-value①②"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) from the <a id="ref-for-keyframe-effect②⑦"></a>keyframe effect is combined with the <a id="ref-for-underlying-value①"></a>[underlying value](#underlying-value) to produce a new value. This resulting value becomes the <a id="ref-for-underlying-value②"></a>underlying value for combining the next <a id="ref-for-keyframe-effect②⑧"></a>keyframe effect in the stack.

The final value of an <a id="ref-for-effect-stack⑧"></a>[effect stack](#effect-stack), called the <a id="composited-value"></a><strong>composited value</strong>, is simply the result of combining the <a id="ref-for-effect-value①③"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) of the final (highest composite order) <a id="ref-for-keyframe-effect②⑨"></a>[keyframe effect](#keyframe-effect) in the stack with the <a id="ref-for-underlying-value③"></a>[underlying value](#underlying-value) at that point.

#### <a id="effect-composition"></a>5.4.4. Effect composition[](#effect-composition)

The specific operation used to combine an <a id="ref-for-effect-value①④"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) with an<a id="ref-for-underlying-value④"></a>[underlying value](#underlying-value) is determined by the <a id="composite-operation"></a><strong>composite operation</strong> of the <a id="ref-for-keyframe-effect③⓪"></a>[keyframe effect](#keyframe-effect) that produced the <a id="ref-for-effect-value①⑤"></a>effect value.

This specification defines three <a id="ref-for-composite-operation⑧"></a>[composite operations](#composite-operation) as follows:

<strong><a id="composite-operation-replace"></a><strong>replace</strong></strong>

The result of compositing the <a id="ref-for-effect-value①⑥"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) with the <a id="ref-for-underlying-value⑤"></a>[underlying value](#underlying-value) is simply the <a id="ref-for-effect-value①⑦"></a>effect value.

<strong><a id="composite-operation-add"></a><strong>add</strong></strong>

The <a id="ref-for-effect-value①⑧"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) is <a id="ref-for-addition"></a>[added](https://drafts.csswg.org/css-values-4/#addition) to the <a id="ref-for-underlying-value⑥"></a>[underlying value](#underlying-value). For <a id="ref-for-animation-type⑧"></a>[animation types](#animation-type) where the <a id="ref-for-addition①"></a>addition operation is defined such that it is not commutative, the order of the operands is <code><a id="ref-for-underlying-value⑦"></a>[underlying value](#underlying-value) + <a id="ref-for-effect-value①⑨"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value)</code>.

<strong><a id="composite-operation-accumulate"></a><strong>accumulate</strong></strong>

The <a id="ref-for-effect-value②⓪"></a>[effect value](https://drafts.csswg.org/web-animations-1/#effect-value) is <a id="ref-for-accumulation"></a>[accumulated](https://drafts.csswg.org/css-values-4/#accumulation) onto the <a id="ref-for-underlying-value⑧"></a>[underlying value](#underlying-value). For <a id="ref-for-animation-type⑨"></a>[animation types](#animation-type) where the <a id="ref-for-accumulation①"></a>accumulation operation is defined such that it is not commutative, the order of the operands is <a id="ref-for-underlying-value⑨"></a>underlying value followed by <a id="ref-for-effect-value②①"></a>effect value.

Tests

- [effect-composition.html](https://wpt.fyi/results/web-animations/animation-model/combining-effects/effect-composition.html) [(live test)](http://wpt.live/web-animations/animation-model/combining-effects/effect-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/combining-effects/effect-composition.html)

#### <a id="applying-the-composited-result"></a>5.4.5. Applying the composited result[](#applying-the-composited-result)

Applying a <a id="ref-for-composited-value"></a>[composited value](#composited-value) to a <a id="ref-for-target-property①⓪"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property) is achieved by adding a specified value to the CSS cascade.

The level of the cascade to which this specified value is added depends on the <a id="ref-for-animation-class④"></a>[class](#animation-class) of the <a id="ref-for-concept-animation⑤⓪"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation) <a id="ref-for-animation-effect-associated-animation①⑥"></a>[associated with](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) the effect with the highest composite order in the <a id="ref-for-effect-stack⑨"></a>[effect stack](#effect-stack) for a given property. By default, the specified value is added to the <a id="ref-for-cascade-origin-animation"></a>[Animation Origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-animation) of the cascade. [\[CSS-CASCADE-3\]](https://drafts.csswg.org/web-animations-1/#biblio-css-cascade-3)

Tests

- [applying-the-composited-result.html](https://wpt.fyi/results/web-animations/animation-model/combining-effects/applying-the-composited-result.html) [(live test)](http://wpt.live/web-animations/animation-model/combining-effects/applying-the-composited-result.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/web-animations/animation-model/combining-effects/applying-the-composited-result.html)

<a id="example-b6e41efa"></a>

<strong>Example:</strong>

[](#example-b6e41efa) For example, if the effect with the highest composite order is associated with a “CSS transition”-class animation, the <a id="ref-for-composited-value①"></a>[composited value](#composited-value) will be added to the <a id="ref-for-cascade-origin-transition"></a>[Transition Origin](https://drafts.csswg.org/css-cascade-5/#cascade-origin-transition) of the cascade.

The <a id="ref-for-composited-value②"></a>[composited value](#composited-value) calculated for a CSS <a id="ref-for-target-property①①"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property) is applied using the following process:

1.  Calculate the <var>base value</var> of the property as the value generated for that property by finding the <a id="ref-for-computed-value⑦"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) for that property in the absence of animations.

2.  Establish the <a id="ref-for-effect-stack①⓪"></a>[effect stack](#effect-stack) for the property (see [§ 5.4.2 The effect stack](#the-effect-stack)).

3.  Calculate the <a id="ref-for-composited-value③"></a>[composited value](#composited-value) of the <a id="ref-for-effect-stack①①"></a>[effect stack](#effect-stack) passing in the <var>base value</var> of the property as the initial <a id="ref-for-underlying-value①⓪"></a>[underlying value](#underlying-value) (see [§ 5.4.3 Calculating the result of an effect stack](#calculating-the-result-of-an-effect-stack)).

4.  Insert the <a id="ref-for-composited-value④"></a>[composited value](#composited-value) into the CSS cascade at the level defined for the <a id="ref-for-animation-class⑤"></a>[class](#animation-class) of the <a id="ref-for-concept-animation⑤①"></a>[animation](https://drafts.csswg.org/web-animations-1/#concept-animation) <a id="ref-for-animation-effect-associated-animation①⑦"></a>[associated with](https://drafts.csswg.org/web-animations-1/#animation-effect-associated-animation) the effect at the top of the <a id="ref-for-effect-stack①②"></a>[effect stack](#effect-stack) established for the <a id="ref-for-target-property①②"></a>[target property](https://drafts.csswg.org/web-animations-1/#target-property).
