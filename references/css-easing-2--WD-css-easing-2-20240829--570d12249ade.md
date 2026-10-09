Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Easing Functions Level 2](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium . W3C ® liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Easing Functions Level 2

Source snapshot: https://www.w3.org/TR/2024/WD-css-easing-2-20240829/

Snapshot SHA-256: 570d12249adedc9d0be3d027f23a43de05dae173fa652b3cff0e180dbe92c836

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Easing Functions Level 2

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes a way for authors to define a transformation that controls the rate of change of some value. Applied to animations, such transformations can be used to produce animations that mimic physical phenomena such as momentum or to cause the animation to move in discrete steps producing robot-like movement. Level 2 adds more sophisticated functions for custom easing curves.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>First Public Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a First Public Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-easing” in the title, like this: “\[css-easing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-easing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="introduction"></a>1. Introduction

<em>This section is not normative.</em>

It is often desirable to control the rate at which some value changes. For example, gradually increasing the speed at which an element moves can give the element a sense of weight as it appears to gather momentum. This can be used to produce intuitive user interface elements or convincing cartoon props that behave like their physical counterparts. Alternatively, it is sometimes desirable for animation to move forwards in distinct steps such as a segmented wheel that rotates such that the segments always appear in the same position.

Similarly, controlling the rate of change of gradient interpolation can be used to produce different visual effects such as suggesting a concave or convex surface, or producing a striped effect.

<a id="ref-for-easing-function"></a>

[Easing functions](#easing-function) provide a means to transform such values by taking an input progress value and producing a corresponding transformed output progress value.

![Example of an easing function that produces an ease-in effect.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/easing-function-example.svg)

Example of an easing function that produces an ease-in effect.  
Given an input progress of 0.7, the easing function scales the value to produce an output progress of 0.52.  
Applying this easing function to an animation would cause it to progress more slowly at first but then gradually progress more quickly.

### <a id="values"></a>1.1.  Value Definitions

This specification uses the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

## <a id="easing-functions"></a>2. Easing functions<a id="timing-functions"></a>

<a id="ref-for-input-progress-value"></a>

<a id="ref-for-output-progress-value"></a>

An <a id="easing-function"></a>easing function takes an [input progress value](#input-progress-value) and produces an [output progress value](#output-progress-value).

<a id="ref-for-easing-function①"></a>

<a id="ref-for-output-progress-value①"></a>

An [easing function](#easing-function) must be a pure function meaning that for a given set of inputs, it always produces the same [output progress value](#output-progress-value).

<a id="ref-for-input-progress-value①"></a>

<a id="ref-for-easing-function②"></a>

The <a id="input-progress-value"></a>input progress value is a real number in the range \[-∞, ∞\]. Typically, the [input progress value](#input-progress-value) is in the range \[0, 1\] but this may not be the case when [easing functions](#easing-function) are chained together.

> <strong data-conversion-semantic="note">Note</strong>
>
> An example of when easing functions are chained together occurs in Web Animations [\[WEB-ANIMATIONS\]](#biblio-web-animations) where the output of the easing function specified on an animation effect may become the input to an easing function specified on one of the keyframes of a keyframe effect. In this scenario, the input to the easing function on the keyframe effect may be outside the range \[0, 1\].

The <a id="output-progress-value"></a>output progress value is a real number in the range \[-∞, ∞\].

<a id="ref-for-before-flag"></a>

Some types of easing functions also take an additional boolean [before flag](#before-flag) input which is defined subsequently.

This specification defines four types of easing functions whose definitions follow.

<a id="ref-for-easing-function③"></a>

The syntax for specifying an [easing function](#easing-function) is as follows:

<a id="ref-for-valdef-easing-function-linear"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-linear-easing-function"></a>

<a id="ref-for-typedef-cubic-bezier-easing-function"></a>

<a id="ref-for-typedef-step-easing-function"></a>

<a id="typedef-easing-function"></a>\<easing-function\> = [linear](#valdef-easing-function-linear) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<linear-easing-function\>](#typedef-linear-easing-function) <a id="ref-for-comb-one①"></a>\| [\<cubic-bezier-easing-function\>](#typedef-cubic-bezier-easing-function) <a id="ref-for-comb-one②"></a>\| [\<step-easing-function\>](#typedef-step-easing-function)

Tests

- [timing-functions-syntax-computed.html](https://wpt.fyi/results/css/css-easing/timing-functions-syntax-computed.html) [(live test)](http://wpt.live/css/css-easing/timing-functions-syntax-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/timing-functions-syntax-computed.html)
- [timing-functions-syntax-invalid.html](https://wpt.fyi/results/css/css-easing/timing-functions-syntax-invalid.html) [(live test)](http://wpt.live/css/css-easing/timing-functions-syntax-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/timing-functions-syntax-invalid.html)
- [timing-functions-syntax-valid.html](https://wpt.fyi/results/css/css-easing/timing-functions-syntax-valid.html) [(live test)](http://wpt.live/css/css-easing/timing-functions-syntax-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/timing-functions-syntax-valid.html)

<a id="ref-for-funcdef-linear"></a>

### <a id="the-linear-easing-function"></a>2.1. The linear easing function: [linear()](#funcdef-linear)

<a id="ref-for-easing-function④"></a>

<a id="ref-for-linear-easing-function-points"></a>

A <a id="linear-easing-function"></a>linear easing function is an [easing function](#easing-function) that interpolates linearly between its [points](#linear-easing-function-points).

<a id="ref-for-linear-easing-function"></a>

<a id="ref-for-list"></a>

<a id="ref-for-linear-easing-point"></a>

A [linear easing function](#linear-easing-function) has <a id="linear-easing-function-points"></a>points, a [list](https://infra.spec.whatwg.org/#list) of [linear easing points](#linear-easing-point). Initially a new empty <a id="ref-for-list①"></a>list.

<a id="ref-for-struct"></a>

A <a id="linear-easing-point"></a>linear easing point is a [struct](https://infra.spec.whatwg.org/#struct) that has:

<a id="linear-easing-point-input"></a>input  
A number or null

<a id="ref-for-create-a-linear-easing-function"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is only null during [create a linear easing function](#create-a-linear-easing-function).

<a id="linear-easing-point-output"></a>output  
A number

#### <a id="linear-easing-function-syntax"></a>2.1.1. Syntax

<a id="ref-for-linear-easing-function①"></a>

A [linear easing function](#linear-easing-function) has the following syntax:

<a id="typedef-linear-easing-function"></a>

<a id="funcdef-linear"></a>

<a id="ref-for-typedef-linear-stop-list"></a>

<a id="typedef-linear-stop-list"></a>

<a id="ref-for-typedef-linear-stop"></a>

<a id="ref-for-mult-comma"></a>

<a id="typedef-linear-stop"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-typedef-linear-stop-length"></a>

<a id="ref-for-mult-opt"></a>

<a id="typedef-linear-stop-length"></a>

<a id="ref-for-percentage-value"></a>

<a id="ref-for-mult-num-range"></a>

```text
<linear-easing-function> = linear(<linear-stop-list>)
<linear-stop-list> = [ <linear-stop> ]#
<linear-stop> = <number> && <linear-stop-length>?
<linear-stop-length> = <percentage>{1,2}
```
Tests

- [linear-timing-functions-syntax.html](https://wpt.fyi/results/css/css-easing/linear-timing-functions-syntax.html) [(live test)](http://wpt.live/css/css-easing/linear-timing-functions-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/linear-timing-functions-syntax.html)

<a id="ref-for-funcdef-linear①"></a>

<a id="ref-for-linear-easing-function②"></a>

<a id="ref-for-create-a-linear-easing-function①"></a>

<a id="ref-for-typedef-linear-stop-list①"></a>

<a id="ref-for-list②"></a>

<a id="ref-for-typedef-linear-stop①"></a>

[linear()](#funcdef-linear) is parsed into a [linear easing function](#linear-easing-function) by calling [create a linear easing function](#create-a-linear-easing-function), passing in its [\<linear-stop-list\>](#typedef-linear-stop-list) as a [list](https://infra.spec.whatwg.org/#list) of [\<linear-stop\>](#typedef-linear-stop)s.

#### <a id="linear-easing-function-parsing"></a>2.1.2. Parsing

<a id="ref-for-list③"></a>

<a id="ref-for-typedef-linear-stop②"></a>

<a id="ref-for-linear-easing-function③"></a>

To <a id="create-a-linear-easing-function"></a>create a linear easing function given a [list](https://infra.spec.whatwg.org/#list) of [\<linear-stop\>](#typedef-linear-stop)s <var>stopList</var>, perform the following. It returns a [linear easing function](#linear-easing-function) or <i>failure</i>.

1.  <a id="ref-for-linear-easing-function④"></a>

    Let <var>function</var> be a new [linear easing function](#linear-easing-function).

2.  Let <var>largestInput</var> be negative infinity.

3.  <a id="ref-for-list-item"></a>

    If there are less than two [items](https://infra.spec.whatwg.org/#list-item) in <var>stopList</var>, then return <i>failure</i>.

4.  <a id="ref-for-list-iterate"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>stop</var> in <var>stopList</var>:

    1.  <a id="ref-for-linear-easing-point①"></a>

        <a id="ref-for-linear-easing-point-output"></a>

        <a id="ref-for-number-value①"></a>

        Let <var>point</var> be a new [linear easing point](#linear-easing-point) with its [output](#linear-easing-point-output) set to <var>stop</var>’s [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) as a number.

    2.  <a id="ref-for-list-append"></a>

        <a id="ref-for-linear-easing-function-points①"></a>

        [Append](https://infra.spec.whatwg.org/#list-append) <var>point</var> to <var>function</var>’s [points](#linear-easing-function-points).

    3.  <a id="ref-for-typedef-linear-stop-length①"></a>

        If <var>stop</var> has a [\<linear-stop-length\>](#typedef-linear-stop-length), then:

        1.  <a id="ref-for-linear-easing-point-input"></a>

            <a id="ref-for-typedef-linear-stop-length②"></a>

            <a id="ref-for-percentage-value①"></a>

            Set <var>point</var>’s [input](#linear-easing-point-input) to whichever is greater: <var>stop</var>’s [\<linear-stop-length\>](#typedef-linear-stop-length)’s first [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) as a number, or <var>largestInput</var>.

        2.  <a id="ref-for-linear-easing-point-input①"></a>

            Set <var>largestInput</var> to <var>point</var>’s [input](#linear-easing-point-input).

        3.  <a id="ref-for-typedef-linear-stop-length③"></a>

            <a id="ref-for-percentage-value②"></a>

            If <var>stop</var>’s [\<linear-stop-length\>](#typedef-linear-stop-length) has a second [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), then:

            1.  <a id="ref-for-linear-easing-point②"></a>

                <a id="ref-for-linear-easing-point-output①"></a>

                <a id="ref-for-number-value②"></a>

                Let <var>extraPoint</var> be a new [linear easing point](#linear-easing-point) with its [output](#linear-easing-point-output) set to <var>stop</var>’s [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) as a number.

            2.  <a id="ref-for-list-append①"></a>

                <a id="ref-for-linear-easing-function-points②"></a>

                [Append](https://infra.spec.whatwg.org/#list-append) <var>extraPoint</var> to <var>function</var>’s [points](#linear-easing-function-points).

            3.  <a id="ref-for-linear-easing-point-input②"></a>

                <a id="ref-for-typedef-linear-stop-length④"></a>

                <a id="ref-for-percentage-value③"></a>

                Set <var>extraPoint</var>’s [input](#linear-easing-point-input) to whichever is greater: <var>stop</var>’s [\<linear-stop-length\>](#typedef-linear-stop-length)’s second [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) as a number, or <var>largestInput</var>.

            4.  <a id="ref-for-linear-easing-point-input③"></a>

                Set <var>largestInput</var> to <var>extraPoint</var>’s [input](#linear-easing-point-input).

    4.  <a id="ref-for-list-item①"></a>

        Otherwise, if <var>stop</var> is the first [item](https://infra.spec.whatwg.org/#list-item) in <var>stopList</var>, then:

        1.  <a id="ref-for-linear-easing-point-input④"></a>

            Set <var>point</var>’s [input](#linear-easing-point-input) to 0.

        2.  Set <var>largestInput</var> to 0.

    5.  <a id="ref-for-list-item②"></a>

        <a id="ref-for-linear-easing-point-input⑤"></a>

        Otherwise, if <var>stop</var> is the last [item](https://infra.spec.whatwg.org/#list-item) in <var>stopList</var>, then set <var>point</var>’s [input](#linear-easing-point-input) to whichever is greater: 1 or <var>largestInput</var>.

5.  <a id="ref-for-list-item③"></a>

    <a id="ref-for-linear-easing-function-points③"></a>

    <a id="ref-for-linear-easing-point-input⑥"></a>

    For runs of [items](https://infra.spec.whatwg.org/#list-item) in <var>function</var>’s [points](#linear-easing-function-points) that have a null [input](#linear-easing-point-input), assign a number to the <a id="ref-for-linear-easing-point-input⑦"></a>input by linearly interpolating between the closest previous and next <a id="ref-for-linear-easing-function-points④"></a>points that have a non-null <a id="ref-for-linear-easing-point-input⑧"></a>input.

6.  Return <var>function</var>.

#### <a id="linear-easing-function-serializing"></a>2.1.3. Serializing

<a id="ref-for-funcdef-linear②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The serialization of [linear()](#funcdef-linear) includes input values for each point, and input values are never less than the input of the previous point.
>
> For example:
>
> - linear(0, 0.25, 1) serializes as linear(0 0%, 0.25 50%, 1 100%)
>
> - linear(0 20%, 0.5 10%, 1) serializes as linear(0 20%, 0.5 20%, 1 100%)
>
> - linear(0, 0.25 25% 75%, 1) serializes as linear(0 0%, 0.25 25%, 0.25 75%, 1 100%)

<a id="ref-for-linear-easing-function⑤"></a>

<a id="ref-for-string"></a>

To get a [linear easing function](#linear-easing-function)'s (<var>linearEasingFunction</var>) <a id="linear-easing-function-serialized-computed-value"></a>serialized computed value, perform the following. It returns a [string](https://infra.spec.whatwg.org/#string).

1.  Let <var>output</var> be "`linear(`".

2.  <a id="ref-for-list-iterate①"></a>

    <a id="ref-for-linear-easing-function-points⑤"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>point</var> in <var>linearEasingFunction</var>’s [points](#linear-easing-function-points):

    1.  <a id="ref-for-list-item④"></a>

        <a id="ref-for-linear-easing-function-points⑥"></a>

        If <var>point</var> is not the first [item](https://infra.spec.whatwg.org/#list-item) of <var>linearEasingFunction</var>’s [points](#linear-easing-function-points), append "` ,  `" to <var>output</var>.

    2.  <a id="ref-for-linear-easing-point-output②"></a>

        <a id="ref-for-number-value③"></a>

        Append the computed value of <var>point</var>’s [output](#linear-easing-point-output), as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), to <var>output</var>.

    3.  Append "`   `" to <var>output</var>.

    4.  <a id="ref-for-linear-easing-point-input⑨"></a>

        <a id="ref-for-percentage-value④"></a>

        Append the computed value of <var>point</var>’s [input](#linear-easing-point-input), as a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), to <var>output</var>.

3.  Append "`)`" to <var>output</var>.

4.  Return <var>output</var>.

#### <a id="linear-easing-function-output"></a>2.1.4. Output of a linear easing function

<a id="ref-for-linear-easing-function⑥"></a>

<a id="ref-for-input-progress-value②"></a>

<a id="ref-for-output-progress-value②"></a>

To <a id="calculate-linear-easing-output-progress"></a>calculate linear easing output progress for a given [linear easing function](#linear-easing-function) <var>linearEasingFunction</var>, and an [input progress value](#input-progress-value) <var>inputProgress</var>, perform the following. It returns an [output progress value](#output-progress-value).

1.  <a id="ref-for-linear-easing-function-points⑦"></a>

    Let <var>points</var> be <var>linearEasingFunction</var>’s [points](#linear-easing-function-points).

2.  <a id="ref-for-list-item⑤"></a>

    <a id="ref-for-linear-easing-point-input①⓪"></a>

    Let <var>pointAIndex</var> be index of the last [item](https://infra.spec.whatwg.org/#list-item) in <var>points</var> with an [input](#linear-easing-point-input) less than or equal to <var>inputProgress</var>, or 0 if there is no match.

3.  <a id="ref-for-list-size"></a>

    If <var>pointAIndex</var> is equal to <var>points</var> [size](https://infra.spec.whatwg.org/#list-size) minus 1, decrement <var>pointAIndex</var> by 1.

    <a id="ref-for-linear-easing-point③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This ensures we have a "next" [point](#linear-easing-point) to compare to.

4.  Let <var>pointA</var> be <var>points</var>\[pointAIndex\].

5.  Let <var>pointB</var> be <var>points</var>\[pointAIndex + 1\].

6.  <a id="ref-for-linear-easing-point-input①①"></a>

    <a id="ref-for-linear-easing-point-output③"></a>

    If <var>pointA</var>’s [input](#linear-easing-point-input) is equal to <var>pointB</var>’s <a id="ref-for-linear-easing-point-input①②"></a>input, return <var>pointB</var>’s [output](#linear-easing-point-output).

7.  <a id="ref-for-linear-easing-point-input①③"></a>

    Let <var>progressFromPointA</var> be <var>inputProgress</var> minus <var>pointA</var>’s [input](#linear-easing-point-input).

8.  <a id="ref-for-linear-easing-point-input①④"></a>

    Let <var>pointInputRange</var> be <var>pointB</var>’s [input](#linear-easing-point-input) minus <var>pointA</var>’s <a id="ref-for-linear-easing-point-input①⑤"></a>input.

9.  Let <var>progressBetweenPoints</var> be <var>progressFromPointA</var> divided by <var>pointInputRange</var>.

10. <a id="ref-for-linear-easing-point-output④"></a>

    Let <var>pointOutputRange</var> be <var>pointB</var>’s [output](#linear-easing-point-output) minus <var>pointA</var>’s <a id="ref-for-linear-easing-point-output⑤"></a>output.

11. Let <var>outputFromLastPoint</var> be <var>progressBetweenPoints</var> multiplied by <var>pointOutputRange</var>.

12. <a id="ref-for-linear-easing-point-output⑥"></a>

    Return <var>pointA</var>’s [output](#linear-easing-point-output) plus <var>outputFromLastPoint</var>.

Tests

- [linear-timing-functions-output.html](https://wpt.fyi/results/css/css-easing/linear-timing-functions-output.html) [(live test)](http://wpt.live/css/css-easing/linear-timing-functions-output.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/linear-timing-functions-output.html)

#### <a id="linear-easing-function-examples"></a>2.1.5. Examples

<a id="ref-for-funcdef-linear③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a1f90788"></a> [linear()](#funcdef-linear) allows the definition of easing functions that interpolate linearly between a set of points.
>
> For example, linear(0, 0.25, 1) produces an easing function that moves linearly from 0, to 0.25, then to 1:
>
> ![linear(0, 0.25, 1) plotted on a graph](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/simple-linear-example.svg)

<a id="ref-for-percentage-value⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-84c8e469"></a> By default, values are spread evenly between entries that don’t have an explicit "input". Input values can be provided using a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value).
>
> For example, linear(0, 0.25 75%, 1) produces the following easing function, which spends 75% of the time transitioning from 0 to .25, then the last 25% transitioning from .25 to 1:
>
> ![linear(0, 0.25 75%, 1) plotted on a graph. The graph has three points. The first is at 0,0. The second is at 0.75,0.25. The third is at 1,1.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/linear-with-input-example.svg)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c0c15aff"></a> If two input values are provided for a single output, it results in two points with the same output.
>
> For example, linear(0, 0.25 25% 75%, 1) is equivalent to linear(0, 0.25 25%, 0.25 75%, 1), producing the following easing function:
>
> ![linear(0, 0.25 75%, 1) plotted on a graph. The graph has four points. The first is at 0,0. The second is at 0.25,0.25. The third is at 0.75,0.25. The forth is at 1,1.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/linear-with-double-input-example.svg)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-67848c25"></a> If the input is outside the range provided, the trajectory of the nearest two points is continued.
>
> For example, here are the implicit values from the previous function:
>
> ![linear(0, 0.25 75%, 1) plotted on a graph. The graph has four points. The first is at 0,0. The second is at 0.25,0.25. The third is at 0.75,0.25. The forth is at 1,1. The ends of the graph are extended at the angle of the nearest two lines.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/linear-with-double-input-example-continued.svg)

<a id="ref-for-funcdef-linear④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-59080c25"></a> A typical use of [linear()](#funcdef-linear) is to provide many points to create the illusion of a curve.
>
> <a id="ref-for-funcdef-linear⑤"></a>
>
> For example, here’s how [linear()](#funcdef-linear) could be used to create a reusable "bounce" easing function:
>
> ```css
> :root {
>   --bounce: linear(
>     /* Start to 1st bounce */
>     0, 0.063, 0.25, 0.563, 1 36.4%,
>     /* 1st to 2nd bounce */
>     0.812, 0.75, 0.813, 1 72.7%,
>     /* 2nd to 3rd bounce */
>     0.953, 0.938, 0.953, 1 90.9%,
>     /* 3rd bounce to end */
>     0.984, 1 100% 100%
>   );
> }
> 
> .example {
>   animation-timing-function: var(--bounce);
> }
> ```
>
> The definition ends `1 100% 100%` to create two final points, so inputs greater than 1 always output 1.
>
> ![The graph of a rough bounce easing.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/linear-bounce-example.svg)
>
> More points could be used to create a smoother result, which may be needed for slower animations.

<a id="ref-for-valdef-easing-function-linear①"></a>

### <a id="the-linear-easing-keyword"></a>2.2. The linear easing keyword: [linear](#valdef-easing-function-linear)<a id="linear-timing-function-section"></a>

<a id="ref-for-linear-easing-function⑦"></a>

<a id="ref-for-output-progress-value③"></a>

<a id="ref-for-input-progress-value③"></a>

The <a id="valdef-easing-function-linear"></a>linear keyword produces an identity [linear easing function](#linear-easing-function) whose [output progress value](#output-progress-value) is equal to the [input progress value](#input-progress-value) for all inputs.

This gives the same result as linear(0, 1).

<a id="ref-for-linear-easing-function⑧"></a>

<a id="ref-for-valdef-easing-function-linear②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although this produces a [linear easing function](#linear-easing-function), uses of the keyword [linear](#valdef-easing-function-linear) always serialize as-is, to <a id="ref-for-valdef-easing-function-linear③"></a>linear. Whereas the function equivalent linear(0, 1) will serialize to linear(0 0%, 1 100%). These rules are in [Serialization](#serialization).

<a id="ref-for-valdef-cubic-bezier-easing-function-ease"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-out"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in-out"></a>

<a id="ref-for-funcdef-cubic-bezier-easing-function-cubic-bezier"></a>

### <a id="cubic-bezier-easing-functions"></a>2.3. Cubic Bézier easing functions: [ease](#valdef-cubic-bezier-easing-function-ease), [ease-in](#valdef-cubic-bezier-easing-function-ease-in), [ease-out](#valdef-cubic-bezier-easing-function-ease-out), [ease-in-out](#valdef-cubic-bezier-easing-function-ease-in-out), [cubic-bezier()](#funcdef-cubic-bezier-easing-function-cubic-bezier)<a id="cubic-bezier-timing-functions"></a>

<a id="ref-for-easing-function⑤"></a>

A <a id="cubic-bzier-easing-function"></a>cubic Bézier easing function is a type of [easing function](#easing-function) defined by four real numbers that specify the two control points, <var>P1</var> and <var>P2</var>, of a cubic Bézier curve whose end points <var>P0</var> and <var>P3</var> are fixed at (0, 0) and (1, 1) respectively. The <var>x</var> coordinates of <var>P1</var> and <var>P2</var> are restricted to the range \[0, 1\].

![A cubic Bezier curve used as an easing function.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/cubic-bezier-easing-curve.svg)

A cubic Bézier curve used as an easing function.  
The shape of the curve is determined by the location of the control points <var>P1</var> and <var>P2</var>.  
Input progress values serve as <var>x</var> values of the curve, whilst the <var>y</var> values are the output progress values.

<a id="ref-for-cubic-bzier-easing-function"></a>

A [cubic Bézier easing function](#cubic-bzier-easing-function) has the following syntax (using notation from [\[CSS-VALUES-3\]](#biblio-css-values-3)):

<a id="ref-for-valdef-cubic-bezier-easing-function-ease①"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in①"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-out①"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in-out①"></a>

<a id="ref-for-funcdef-cubic-bezier-easing-function-cubic-bezier①"></a>

<a id="ref-for-number-value④"></a>

<a id="ref-for-comb-comma"></a>

<a id="typedef-cubic-bezier-easing-function"></a>\<cubic-bezier-easing-function\> = [ease](#valdef-cubic-bezier-easing-function-ease) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [ease-in](#valdef-cubic-bezier-easing-function-ease-in) <a id="ref-for-comb-one④"></a>\| [ease-out](#valdef-cubic-bezier-easing-function-ease-out) <a id="ref-for-comb-one⑤"></a>\| [ease-in-out](#valdef-cubic-bezier-easing-function-ease-in-out) <a id="ref-for-comb-one⑥"></a>\| [cubic-bezier](#funcdef-cubic-bezier-easing-function-cubic-bezier)([\<number \[0,1\]\>](https://www.w3.org/TR/css-values-4/#number-value)[,](https://www.w3.org/TR/css-values-4/#comb-comma) <a id="ref-for-number-value⑤"></a>\<number\><a id="ref-for-comb-comma①"></a>, <a id="ref-for-number-value⑥"></a>\<number \[0,1\]\><a id="ref-for-comb-comma②"></a>, <a id="ref-for-number-value⑦"></a>\<number\>)

The meaning of each value is as follows:

<a id="valdef-cubic-bezier-easing-function-ease"></a>ease

Equivalent to cubic-bezier(0.25, 0.1, 0.25, 1).

<a id="valdef-cubic-bezier-easing-function-ease-in"></a>ease-in

Equivalent to cubic-bezier(0.42, 0, 1, 1).

<a id="valdef-cubic-bezier-easing-function-ease-out"></a>ease-out

Equivalent to cubic-bezier(0, 0, 0.58, 1).

<a id="valdef-cubic-bezier-easing-function-ease-in-out"></a>ease-in-out

Equivalent to cubic-bezier(0.42, 0, 0.58, 1).

<a id="ref-for-number-value⑧"></a>

<a id="funcdef-cubic-bezier-easing-function-cubic-bezier"></a>cubic-bezier([\<number \[0,1\]\>](https://www.w3.org/TR/css-values-4/#number-value), <a id="ref-for-number-value⑨"></a>\<number\>, <a id="ref-for-number-value①⓪"></a>\<number \[0,1\]\>, <a id="ref-for-number-value①①"></a>\<number\>)

<a id="ref-for-cubic-bzier-easing-function①"></a>

Specifies a [cubic Bézier easing function](#cubic-bzier-easing-function). The four numbers specify points <var>P1</var> and <var>P2</var> of the curve as (<var>x1</var>, <var>y1</var>, <var>x2</var>, <var>y2</var>). Both <var>x</var> values must be in the range \[0, 1\] or the definition is invalid.

The keyword values listed above are illustrated below.

![The easing functions produced by keyword values.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/curve-keywords.svg)

The easing functions produced by each of the cubic Bézier easing function keyword values.

#### <a id="cubic-bezier-algo"></a>2.3.1. Output of a cubic bézier easing function

<a id="ref-for-output-progress-value④"></a>

<a id="ref-for-input-progress-value④"></a>

The mapping from input progress to output progress is performed by determining the corresponding <var>y</var> value ([output progress value](#output-progress-value)) for a given <var>x</var> value ([input progress value](#input-progress-value)). The evaluation of this curve is covered in many sources such as [\[FUND-COMP-GRAPHICS\]](#biblio-fund-comp-graphics).

<a id="ref-for-input-progress-value⑤"></a>

For [input progress values](#input-progress-value) outside the range \[0, 1\], the curve is extended infinitely using tangent of the curve at the closest endpoint as follows:

- <a id="ref-for-input-progress-value⑥"></a>

  For [input progress values](#input-progress-value) less than zero,

  1.  If the <var>x</var> value of P1 is greater than zero, use a straight line that passes through P1 and P0 as the tangent.

  2.  Otherwise, if the <var>x</var> value of P2 is greater than zero, use a straight line that passes through P2 and P0 as the tangent.

  3.  <a id="ref-for-output-progress-value⑤"></a>

      <a id="ref-for-input-progress-value⑦"></a>

      Otherwise, let the [output progress value](#output-progress-value) be zero for all [input progress values](#input-progress-value) in the range \[-∞, 0).

- <a id="ref-for-input-progress-value⑧"></a>

  For [input progress values](#input-progress-value) greater than one,

  1.  If the <var>x</var> value of P2 is less than one, use a straight line that passes through P2 and P3 as the tangent.

  2.  Otherwise, if the <var>x</var> value of P1 is less than one, use a straight line that passes through P1 and P3 as the tangent.

  3.  <a id="ref-for-output-progress-value⑥"></a>

      <a id="ref-for-input-progress-value⑨"></a>

      Otherwise, let the [output progress value](#output-progress-value) be one for all [input progress values](#input-progress-value) in the range (1, ∞\].

Tests

- [cubic-bezier-timing-functions-output.html](https://wpt.fyi/results/css/css-easing/cubic-bezier-timing-functions-output.html) [(live test)](http://wpt.live/css/css-easing/cubic-bezier-timing-functions-output.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/cubic-bezier-timing-functions-output.html)

<a id="ref-for-valdef-step-easing-function-step-start"></a>

<a id="ref-for-valdef-step-easing-function-step-end"></a>

<a id="ref-for-funcdef-step-easing-function-steps"></a>

### <a id="step-easing-functions"></a>2.4. Step easing functions: [step-start](#valdef-step-easing-function-step-start), [step-end](#valdef-step-easing-function-step-end), [steps()](#funcdef-step-easing-function-steps)<a id="step-timing-functions"></a>

<a id="ref-for-easing-function⑥"></a>

A <a id="step-easing-function"></a>step easing function is a type of [easing function](#easing-function) that divides the input time into a specified number of intervals that are equal in length. It is defined by a number of <a id="steps"></a>steps, and a <a id="step-position"></a>step position. It has following syntax:

<a id="ref-for-valdef-step-easing-function-step-start①"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-valdef-step-easing-function-step-end①"></a>

<a id="ref-for-funcdef-step-easing-function-steps①"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-step-position"></a>

<a id="ref-for-mult-opt①"></a>

<a id="typedef-step-easing-function"></a>\<step-easing-function\> = [step-start](#valdef-step-easing-function-step-start) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [step-end](#valdef-step-easing-function-step-end) <a id="ref-for-comb-one⑧"></a>\| [steps](#funcdef-step-easing-function-steps)([\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) [,](https://www.w3.org/TR/css-values-4/#comb-comma) [\<step-position\>](#typedef-step-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt))

<a id="ref-for-valdef-steps-jump-start"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-valdef-steps-jump-end"></a>

<a id="ref-for-valdef-steps-jump-none"></a>

<a id="ref-for-valdef-steps-jump-both"></a>

<a id="ref-for-valdef-steps-start"></a>

<a id="ref-for-valdef-steps-end"></a>

<a id="typedef-step-position"></a>\<step-position\> = [jump-start](#valdef-steps-jump-start) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [jump-end](#valdef-steps-jump-end) <a id="ref-for-comb-one①⓪"></a>\| [jump-none](#valdef-steps-jump-none) <a id="ref-for-comb-one①①"></a>\| [jump-both](#valdef-steps-jump-both) <a id="ref-for-comb-one①②"></a>\| [start](#valdef-steps-start) <a id="ref-for-comb-one①③"></a>\| [end](#valdef-steps-end)

Tests

- [step-timing-functions-syntax.html](https://wpt.fyi/results/css/css-easing/step-timing-functions-syntax.html) [(live test)](http://wpt.live/css/css-easing/step-timing-functions-syntax.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/step-timing-functions-syntax.html)

The meaning of each value is as follows:

<a id="valdef-step-easing-function-step-start"></a>step-start  
Computes to steps(1, start)

<a id="valdef-step-easing-function-step-end"></a>step-end  
Computes to steps(1, end)

![Example step easing keywords.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/step-easing-keyword-examples.svg)

Example step easing function keyword values.

<a id="funcdef-step-easing-function-steps"></a>steps(\<integer\>, \<step-position\>?)  
<a id="ref-for-valdef-steps-jump-none①"></a>

The first parameter specifies the number of intervals in the function. It must be a positive integer greater than 0 unless the second parameter is [jump-none](#valdef-steps-jump-none) in which case it must be a positive integer greater than 1.

<a id="ref-for-step-position"></a>

The second parameter, which is optional, specifies the [step position](#step-position) using one of the following values:

<a id="valdef-steps-jump-start"></a>jump-start  
<a id="ref-for-input-progress-value①⓪"></a>

The first rise occurs at [input progress value](#input-progress-value) of 0.

<a id="valdef-steps-jump-end"></a>jump-end  
<a id="ref-for-input-progress-value①①"></a>

The last rise occurs at [input progress value](#input-progress-value) of 1.

<a id="valdef-steps-jump-none"></a>jump-none  
All rises occur within the range (0, 1).

<a id="valdef-steps-jump-both"></a>jump-both  
<a id="ref-for-input-progress-value①②"></a>

The first rise occurs at [input progress value](#input-progress-value) of 0 and the last rise occurs at <a id="ref-for-input-progress-value①③"></a>input progress value of 1.

<a id="valdef-steps-start"></a>start  
<a id="ref-for-valdef-steps-jump-start①"></a>

Behaves as [jump-start](#valdef-steps-jump-start).

<a id="valdef-steps-end"></a>end  
<a id="ref-for-valdef-steps-jump-end①"></a>

Behaves as [jump-end](#valdef-steps-jump-end).

<a id="ref-for-valdef-steps-end①"></a>

If the second parameter is omitted, the value [end](#valdef-steps-end) is assumed.

These values are illustrated below:

![Example step easing functions.](https://www.w3.org/TR/2024/WD-css-easing-2-20240829/images/step-easing-func-examples.svg)

Example step easing functions.

#### <a id="step-easing-algo"></a>2.4.1. Output of a step easing function<a id="step-timing-function-algo"></a>

<a id="ref-for-step-easing-function"></a>

At the exact point where a step occurs, the result of the function is conceptually the top of the step. However, an additional <a id="before-flag"></a>before flag passed as input to the [step easing function](#step-easing-function), if true, will cause the result of the function to correspond to the bottom of the step at the step point.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7b6395b6"></a>
>
> <a id="ref-for-before-flag①"></a>
>
> <a id="ref-for-step-easing-function①"></a>
>
> <a id="ref-for-step-position①"></a>
>
> <a id="ref-for-valdef-steps-start①"></a>
>
> As an example of how the [before flag](#before-flag) affects the behavior of this function, consider an animation with a [step easing function](#step-easing-function) whose [step position](#step-position) is [start](#valdef-steps-start) and which has a positive delay and backwards fill.
>
> For example, using CSS animation:
>
> ```css
> animation: moveRight 5s 1s steps(5, start);
> ```
>
> <a id="ref-for-input-progress-value①④"></a>
>
> <a id="ref-for-before-flag②"></a>
>
> <a id="ref-for-output-progress-value⑦"></a>
>
> During the delay phase, the [input progress value](#input-progress-value) will be zero but if the [before flag](#before-flag) is set to indicate that the animation has yet to reach its animation interval, the easing function will produce zero as its [output progress value](#output-progress-value), i.e. the bottom of the first step.
>
> <a id="ref-for-input-progress-value①⑤"></a>
>
> <a id="ref-for-before-flag③"></a>
>
> At the exact moment when the animation interval begins, the [input progress value](#input-progress-value) will still be zero, but the [before flag](#before-flag) will not be set and hence the result of the easing function will correspond to the top of the first step.

<a id="ref-for-output-progress-value⑧"></a>

<a id="ref-for-step-position②"></a>

<a id="ref-for-valdef-steps-start②"></a>

<a id="ref-for-valdef-steps-jump-start②"></a>

<a id="ref-for-valdef-steps-end②"></a>

<a id="ref-for-valdef-steps-jump-end②"></a>

For the purposes of calculating the [output progress value](#output-progress-value), the [step position](#step-position) [start](#valdef-steps-start) is considered equivalent to [jump-start](#valdef-steps-jump-start). Likewise [end](#valdef-steps-end) is considered equivalent to [jump-end](#valdef-steps-jump-end). As a result, the following algorithm does not make explicit reference to <a id="ref-for-valdef-steps-start③"></a>start or <a id="ref-for-valdef-steps-end③"></a>end.

<a id="ref-for-valdef-steps-jump-start③"></a>

<a id="ref-for-valdef-steps-start④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents must still differentiate between [jump-start](#valdef-steps-jump-start) and [start](#valdef-steps-start) for the purpose of serialization (see [§ 2.5 Serialization](#serialization)).

<a id="ref-for-output-progress-value⑨"></a>

<a id="ref-for-input-progress-value①⑥"></a>

<a id="ref-for-before-flag④"></a>

The [output progress value](#output-progress-value) is calculated from the [input progress value](#input-progress-value) and [before flag](#before-flag) as follows:

1.  <a id="ref-for-input-progress-value①⑦"></a>

    <a id="ref-for-steps"></a>

    Calculate the <var>current step</var> as <code>floor(<a href="#input-progress-value">input&#x20;progress&#x20;value</a>&#x20;×&#x20;<a href="#steps">steps</a>)</code>.

2.  <a id="ref-for-step-position③"></a>

    If the [step position](#step-position) property is one of:

    - <a id="ref-for-valdef-steps-jump-start④"></a>

      [jump-start](#valdef-steps-jump-start),

    - <a id="ref-for-valdef-steps-jump-both①"></a>

      [jump-both](#valdef-steps-jump-both),

    increment <var>current step</var> by one.

3.  If <em>both</em> of the following conditions are true:

    - <a id="ref-for-before-flag⑤"></a>

      the [before flag](#before-flag) is set, <em>and</em>

    - <a id="ref-for-input-progress-value①⑧"></a>

      <a id="ref-for-steps①"></a>

      [input progress value](#input-progress-value) × [steps](#steps) mod 1 equals zero (that is, if <a id="ref-for-input-progress-value①⑨"></a>input progress value × <a id="ref-for-steps②"></a>steps is integral), then

    decrement <var>current step</var> by one.

4.  <a id="ref-for-input-progress-value②⓪"></a>

    If [input progress value](#input-progress-value) ≥ 0 and <var>current step</var> \< 0, let <var>current step</var> be zero.

5.  <a id="ref-for-step-position④"></a>

    Calculate <var>jumps</var> based on the [step position](#step-position) as follows:

    <a id="ref-for-valdef-steps-jump-end③"></a>

    <a id="ref-for-valdef-steps-jump-start⑤"></a>

    [jump-start](#valdef-steps-jump-start) or [jump-end](#valdef-steps-jump-end)

    <a id="ref-for-steps③"></a>

    [steps](#steps)

    <a id="ref-for-valdef-steps-jump-none②"></a>

    [jump-none](#valdef-steps-jump-none)

    <a id="ref-for-steps④"></a>

    [steps](#steps) - 1

    <a id="ref-for-valdef-steps-jump-both②"></a>

    [jump-both](#valdef-steps-jump-both)

    <a id="ref-for-steps⑤"></a>

    [steps](#steps) + 1

6.  <a id="ref-for-input-progress-value②①"></a>

    If [input progress value](#input-progress-value) ≤ 1 and <var>current step</var> \> <var>jumps</var>, let <var>current step</var> be <var>jumps</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > <a id="ref-for-input-progress-value②②"></a>
    >
    > <a id="ref-for-output-progress-value①⓪"></a>
    >
    > Steps 4 and 6 in this procedure ensure that given an [input progress value](#input-progress-value) in the range \[0, 1\], a step easing function does not produce an [output progress value](#output-progress-value) outside that range.
    >
    > <a id="ref-for-step-position⑤"></a>
    >
    > <a id="ref-for-valdef-steps-jump-start⑥"></a>
    >
    > <a id="ref-for-input-progress-value②③"></a>
    >
    > <a id="ref-for-output-progress-value①①"></a>
    >
    > For example, although mathematically we might expect that a step easing function with a [step position](#step-position) of [jump-start](#valdef-steps-jump-start) would step up (i.e. beyond 1) when the [input progress value](#input-progress-value) is 1, intuitively, when we apply such an easing function to a forwards-filling animation, we expect it to produce an [output progress value](#output-progress-value) of 1 as the animation fills forwards.
    >
    > <a id="ref-for-step-position⑥"></a>
    >
    > <a id="ref-for-valdef-steps-jump-end④"></a>
    >
    > A similar situation arises for a step easing function with a [step position](#step-position) of [jump-end](#valdef-steps-jump-end) when applied to an animation during its delay phase.

7.  <a id="ref-for-output-progress-value①②"></a>

    The [output progress value](#output-progress-value) is <code><var>current&#x20;step</var>&#x20;/&#x20;<var>jumps</var></code>.

Tests

- [step-timing-functions-output.html](https://wpt.fyi/results/css/css-easing/step-timing-functions-output.html) [(live test)](http://wpt.live/css/css-easing/step-timing-functions-output.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/step-timing-functions-output.html)

### <a id="serialization"></a>2.5. Serialization<a id="serializing-a-timing-function"></a>

Easing functions are serialized using the common serialization patterns defined in [\[CSSOM\]](#biblio-cssom) with the following additional requirements:

- <a id="ref-for-valdef-cubic-bezier-easing-function-ease②"></a>

  <a id="ref-for-valdef-easing-function-linear④"></a>

  <a id="ref-for-valdef-cubic-bezier-easing-function-ease-in②"></a>

  <a id="ref-for-valdef-cubic-bezier-easing-function-ease-out②"></a>

  <a id="ref-for-valdef-cubic-bezier-easing-function-ease-in-out②"></a>

  <a id="ref-for-funcdef-cubic-bezier-easing-function-cubic-bezier②"></a>

  <a id="ref-for-funcdef-linear⑥"></a>

  The keyword values [ease](#valdef-cubic-bezier-easing-function-ease), [linear](#valdef-easing-function-linear), [ease-in](#valdef-cubic-bezier-easing-function-ease-in), [ease-out](#valdef-cubic-bezier-easing-function-ease-out), and [ease-in-out](#valdef-cubic-bezier-easing-function-ease-in-out) are serialized as-is, that is, they are <em>not</em> converted to the equivalent [cubic-bezier()](#funcdef-cubic-bezier-easing-function-cubic-bezier) or [linear()](#funcdef-linear) function before serializing.

- <a id="ref-for-funcdef-step-easing-function-steps②"></a>

  <a id="ref-for-valdef-step-easing-function-step-start②"></a>

  <a id="ref-for-valdef-step-easing-function-step-end②"></a>

  Step easing functions, whether they are specified using the [steps()](#funcdef-step-easing-function-steps) function or either of the [step-start](#valdef-step-easing-function-step-start) or [step-end](#valdef-step-easing-function-step-end) keywords, are serialized as follows:

  1.  <a id="ref-for-step-position⑦"></a>

      <a id="ref-for-valdef-steps-jump-end⑤"></a>

      <a id="ref-for-valdef-steps-end④"></a>

      <a id="ref-for-funcdef-step-easing-function-steps③"></a>

      If the [step position](#step-position) is [jump-end](#valdef-steps-jump-end) or [end](#valdef-steps-end), serialize as [steps(\<integer\>)](#funcdef-step-easing-function-steps).

  2.  <a id="ref-for-funcdef-step-easing-function-steps④"></a>

      Otherwise, serialize as [steps(\<integer\>, \<step-position\>)](#funcdef-step-easing-function-steps).

- <a id="ref-for-linear-easing-function⑨"></a>

  <a id="ref-for-funcdef-linear⑦"></a>

  <a id="ref-for-linear-easing-function-serialized-computed-value"></a>

  A [linear easing function](#linear-easing-function) created via [linear()](#funcdef-linear) is serialized by getting its [serialized computed value](#linear-easing-function-serialized-computed-value).

Tests

- [timing-functions-syntax-computed.html](https://wpt.fyi/results/css/css-easing/timing-functions-syntax-computed.html) [(live test)](http://wpt.live/css/css-easing/timing-functions-syntax-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-easing/timing-functions-syntax-computed.html)

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

This specification does not directly introduce any new capabilities to the Web platform but rather provides common definitions that may be referenced by other specifications.

## <a id="security"></a>Security Considerations

<a id="ref-for-input-progress-value②④"></a>

<a id="ref-for-output-progress-value①③"></a>

Specifications referencing the features defined in this specification should consider that while easing functions most commonly take an [input progress value](#input-progress-value) in the range \[0,1\] and produce an [output progress value](#output-progress-value) in the range \[0, 1\], this is not always the case. Applications of easing functions should define the behavior for inputs and outputs outside this range to ensure they do not introduce new security considerations.

## <a id="changes"></a>3. Changes

### <a id="changes-L1"></a>3.1.  Additions Since Level 1

- <a id="ref-for-funcdef-linear⑧"></a>

  Added [linear()](#funcdef-linear) function.

## <a id="acknowledgements"></a>4. Acknowledgements

This specification is based on the [CSS Transitions](https://www.w3.org/TR/css3-transitions/) specification edited by L. David Baron, Dean Jackson, David Hyatt, and Chris Marrin. The editors would also like to thank Douglas Stockwell, Steve Block, Tab Atkins, Rachel Nabors, Martin Pitt, and the [Animation at Work](https://damp-lake-50659.herokuapp.com/) slack community for their feedback and contributions.

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

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

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

- [before flag](#before-flag), in § 2.4.1
- [calculate linear easing output progress](#calculate-linear-easing-output-progress), in § 2.1.4
- [create a linear easing function](#create-a-linear-easing-function), in § 2.1.2
- [cubic-bezier()](#funcdef-cubic-bezier-easing-function-cubic-bezier), in § 2.3
- [\<cubic-bezier-easing-function\>](#typedef-cubic-bezier-easing-function), in § 2.3
- [cubic Bézier easing function](#cubic-bzier-easing-function), in § 2.3
- [ease](#valdef-cubic-bezier-easing-function-ease), in § 2.3
- [ease-in](#valdef-cubic-bezier-easing-function-ease-in), in § 2.3
- [ease-in-out](#valdef-cubic-bezier-easing-function-ease-in-out), in § 2.3
- [ease-out](#valdef-cubic-bezier-easing-function-ease-out), in § 2.3
- [\<easing-function\>](#typedef-easing-function), in § 2
- [easing function](#easing-function), in § 2
- [end](#valdef-steps-end), in § 2.4
- [input](#linear-easing-point-input), in § 2.1
- [input progress value](#input-progress-value), in § 2
- [jump-both](#valdef-steps-jump-both), in § 2.4
- [jump-end](#valdef-steps-jump-end), in § 2.4
- [jump-none](#valdef-steps-jump-none), in § 2.4
- [jump-start](#valdef-steps-jump-start), in § 2.4
- [linear](#valdef-easing-function-linear), in § 2.2
- [linear()](#funcdef-linear), in § 2.1.1
- [\<linear-easing-function\>](#typedef-linear-easing-function), in § 2.1.1
- [linear easing function](#linear-easing-function), in § 2.1
- [linear easing point](#linear-easing-point), in § 2.1
- [\<linear-stop\>](#typedef-linear-stop), in § 2.1.1
- [\<linear-stop-length\>](#typedef-linear-stop-length), in § 2.1.1
- [\<linear-stop-list\>](#typedef-linear-stop-list), in § 2.1.1
- [output](#linear-easing-point-output), in § 2.1
- [output progress value](#output-progress-value), in § 2
- [points](#linear-easing-function-points), in § 2.1
- [serialized computed value](#linear-easing-function-serialized-computed-value), in § 2.1.3
- [start](#valdef-steps-start), in § 2.4
- [\<step-easing-function\>](#typedef-step-easing-function), in § 2.4
- [step easing function](#step-easing-function), in § 2.4
- [step-end](#valdef-step-easing-function-step-end), in § 2.4
- [\<step-position\>](#typedef-step-position), in § 2.4
- [step position](#step-position), in § 2.4
- [steps](#steps), in § 2.4
- [steps()](#funcdef-step-easing-function-steps), in § 2.4
- [step-start](#valdef-step-easing-function-step-start), in § 2.4
- [timing function](#easing-function), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="8cd4f032"></a>,
  - <a id="d73c993d"></a>\<integer\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="d4441b24"></a>?
  - <a id="3bafef5e"></a>{a,b}
  - <a id="4eb9d37e"></a>\|
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append
  - <a id="16d07e10"></a>for each
  - <a id="5afbefcd"></a>item
  - <a id="649608b9"></a>list
  - <a id="0204d188"></a>size
  - <a id="0698d556"></a>string
  - <a id="984221ca"></a>struct

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-fund-comp-graphics"></a>\[FUND-COMP-GRAPHICS\]  
Peter Shirley; Michael Ashikhmin; Steve Marschner. Fundamentals of Computer Graphics. 2009.

<a id="biblio-web-animations"></a>\[WEB-ANIMATIONS\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)
