Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Transitions Module Level 1](https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Transitions Module Level 1

Source snapshot: https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/

Snapshot SHA-256: 969d35d13ee4e4248421ebf991156a1af6b577d17a1be2565ffeaa94a012ec80

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 7 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Transitions Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

CSS Transitions allows property changes in CSS values to occur smoothly over a specified duration.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-transitions” in the title, like this: “\[css-transitions\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-transitions%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<strong>This document</strong> is expected to be relatively close to last call. While some issues raised have yet to be addressed, new features are extremely unlikely to be considered for this level.

## <a id="introduction"></a>1. Introduction

<em>This section is not normative.</em>

This document introduces new CSS features to enable <em>implicit transitions</em>, which describe how CSS properties can be made to change smoothly from one value to another over a given duration.

Tests

crashes

- [delete-image-set.html](https://wpt.fyi/results/css/css-transitions/crashtests/delete-image-set.html) [(live test)](http://wpt.live/css/css-transitions/crashtests/delete-image-set.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/crashtests/delete-image-set.html)
- [size-container-transition-crash.html](https://wpt.fyi/results/css/css-transitions/crashtests/size-container-transition-crash.html) [(live test)](http://wpt.live/css/css-transitions/crashtests/size-container-transition-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/crashtests/size-container-transition-crash.html)
- [transition-during-style-attr-mutation.html](https://wpt.fyi/results/css/css-transitions/crashtests/transition-during-style-attr-mutation.html) [(live test)](http://wpt.live/css/css-transitions/crashtests/transition-during-style-attr-mutation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/crashtests/transition-during-style-attr-mutation.html)
- [transition-large-word-spacing-001.html](https://wpt.fyi/results/css/css-transitions/crashtests/transition-large-word-spacing-001.html) [(live test)](http://wpt.live/css/css-transitions/crashtests/transition-large-word-spacing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/crashtests/transition-large-word-spacing-001.html)

------------------------------------------------------------------------

### <a id="values"></a>1.1. Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="transitions"></a>2. <a id="transitions-"></a> Transitions

Normally when the value of a CSS property changes, the rendered result is instantly updated, with the affected elements immediately changing from the old property value to the new property value. This section describes a way to specify gradual transitions using new CSS properties. These properties are used to animate smoothly from the old state to the new state over time.

<a id="ref-for-propdef-left"></a>

<a id="ref-for-propdef-background-color"></a>

For example, suppose that transitions of one second have been defined on the [left](https://www.w3.org/TR/CSS2/visuren.html#propdef-left) and [background-color](https://www.w3.org/TR/CSS2/colors.html#propdef-background-color) properties. The following diagram illustrates the effect of updating those properties on an element, in this case moving it to the right and changing the background from red to blue. This assumes other transition parameters still have their default values.

![Example showing the initial, intermediate, and final states of a box whose color and position is interpolated](https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/images/transition-example.svg)

<a id="ref-for-propdef-left①"></a>

<a id="ref-for-propdef-background-color①"></a>

Transitions of [left](https://www.w3.org/TR/CSS2/visuren.html#propdef-left) and [background-color](https://www.w3.org/TR/CSS2/colors.html#propdef-background-color).

<a id="ref-for-computed-value"></a>

Transitions are a presentational effect. The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a property transitions over time from the old value to the new value. Therefore if a script queries the <a id="ref-for-computed-value①"></a>computed value of a property (or other data depending on it) as it is transitioning, it will see an intermediate value that represents the current animated value of the property.

The transition for a property is defined using a number of new properties. For example:

<a id="ref-for-propdef-opacity"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c14634d9"></a>
>
> Example(s):
>
> ```text
> div {
>   transition-property: opacity;
>   transition-duration: 2s;
> }
> ```
>
> The above example defines a transition on the [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) property that, when a new value is assigned to it, will cause a smooth change between the old value and the new value over a period of two seconds.

Each of the transition properties accepts a comma-separated list, allowing multiple transitions to be defined, each acting on a different property. In this case, the individual transitions take their parameters from the same index in all the lists. For example:

<a id="ref-for-propdef-opacity①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0825db11"></a>
>
> Example(s):
>
> ```text
> div {
>   transition-property: opacity, left;
>   transition-duration: 2s, 4s;
> }
> 
> ```
>
> This will cause the [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) property to transition over a period of two seconds and the left property to transition over a period of four seconds.

<a id="ref-for-propdef-transition-property"></a>

<a id="ref-for-propdef-background-image"></a>

<a id="ref-for-propdef-transition-property②"></a>

<a id="list-matching"></a> In the case where the lists of values in transition properties do not have the same length, the length of the [transition-property](#propdef-transition-property) list determines the number of items in each list examined when starting transitions. The lists are matched up from the first value: excess values at the end are not used. If one of the other properties doesn’t have enough comma-separated values to match the number of values of <a id="ref-for-propdef-transition-property①"></a>transition-property, the user agent must calculate its used value by repeating the list of values until there are enough. This truncation or repetition does not affect the computed value. <strong data-conversion-semantic="note">Note:</strong> Note: This is analogous to the behavior of the background-\* properties, with [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) analogous to [transition-property](#propdef-transition-property).

<a id="ref-for-propdef-opacity②"></a>

<a id="ref-for-propdef-left②"></a>

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-width"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d94cbd75"></a>
>
> Example(s):
>
> ```text
> div {
>   transition-property: opacity, left, top, width;
>   transition-duration: 2s, 1s;
> }
> ```
>
> The above example defines a transition on the [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) property of 2 seconds duration, a transition on the [left](https://www.w3.org/TR/CSS2/visuren.html#propdef-left) property of 1 second duration, a transition on the [top](https://www.w3.org/TR/CSS2/visuren.html#propdef-top) property of 2 seconds duration and a transition on the [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) property of 1 second duration.

While authors can use transitions to create dynamically changing content, dynamically changing content can lead to seizures in some users. For information on how to avoid content that can lead to seizures, see [Guideline 2.3: Seizures: Do not design content in a way that is known to cause seizures](https://www.w3.org/TR/WCAG20/#seizure) ([\[WCAG20\]](#biblio-wcag20)).

Tests

- [animate-with-color-mix.html](https://wpt.fyi/results/css/css-transitions/animations/animate-with-color-mix.html) [(live test)](http://wpt.live/css/css-transitions/animations/animate-with-color-mix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/animate-with-color-mix.html)
- [color-transition-premultiplied.html](https://wpt.fyi/results/css/css-transitions/animations/color-transition-premultiplied.html) [(live test)](http://wpt.live/css/css-transitions/animations/color-transition-premultiplied.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/color-transition-premultiplied.html)
- [text-shadow-composition.html](https://wpt.fyi/results/css/css-transitions/animations/text-shadow-composition.html) [(live test)](http://wpt.live/css/css-transitions/animations/text-shadow-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/text-shadow-composition.html)
- [text-shadow-interpolation.html](https://wpt.fyi/results/css/css-transitions/animations/text-shadow-interpolation.html) [(live test)](http://wpt.live/css/css-transitions/animations/text-shadow-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/text-shadow-interpolation.html)
- [vertical-align-composition.html](https://wpt.fyi/results/css/css-transitions/animations/vertical-align-composition.html) [(live test)](http://wpt.live/css/css-transitions/animations/vertical-align-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/vertical-align-composition.html)
- [vertical-align-interpolation.html](https://wpt.fyi/results/css/css-transitions/animations/vertical-align-interpolation.html) [(live test)](http://wpt.live/css/css-transitions/animations/vertical-align-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/vertical-align-interpolation.html)
- [z-index-interpolation.html](https://wpt.fyi/results/css/css-transitions/animations/z-index-interpolation.html) [(live test)](http://wpt.live/css/css-transitions/animations/z-index-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/z-index-interpolation.html)
- [idlharness.html](https://wpt.fyi/results/css/css-transitions/idlharness.html) [(live test)](http://wpt.live/css/css-transitions/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/idlharness.html)
- [inherit-background-color-transition.html](https://wpt.fyi/results/css/css-transitions/inherit-background-color-transition.html) [(live test)](http://wpt.live/css/css-transitions/inherit-background-color-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/inherit-background-color-transition.html)
- [inheritance.html](https://wpt.fyi/results/css/css-transitions/inheritance.html) [(live test)](http://wpt.live/css/css-transitions/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/inheritance.html)
- [properties-value-001.html](https://wpt.fyi/results/css/css-transitions/properties-value-001.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-001.html)
- [properties-value-002.html](https://wpt.fyi/results/css/css-transitions/properties-value-002.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-002.html)
- [properties-value-003.html](https://wpt.fyi/results/css/css-transitions/properties-value-003.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-003.html)
- [properties-value-implicit-001.html](https://wpt.fyi/results/css/css-transitions/properties-value-implicit-001.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-implicit-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-implicit-001.html)
- [retargetted-transition-with-box-sizing.html](https://wpt.fyi/results/css/css-transitions/retargetted-transition-with-box-sizing.html) [(live test)](http://wpt.live/css/css-transitions/retargetted-transition-with-box-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/retargetted-transition-with-box-sizing.html)
- [shadow-root-insertion.html](https://wpt.fyi/results/css/css-transitions/shadow-root-insertion.html) [(live test)](http://wpt.live/css/css-transitions/shadow-root-insertion.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/shadow-root-insertion.html)
- [transition-base-response-001.html](https://wpt.fyi/results/css/css-transitions/transition-base-response-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-base-response-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-base-response-001.html)
- [transition-base-response-002.html](https://wpt.fyi/results/css/css-transitions/transition-base-response-002.html) [(live test)](http://wpt.live/css/css-transitions/transition-base-response-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-base-response-002.html)
- [transition-base-response-003.html](https://wpt.fyi/results/css/css-transitions/transition-base-response-003.html) [(live test)](http://wpt.live/css/css-transitions/transition-base-response-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-base-response-003.html)
- [transition-in-iframe-001.html](https://wpt.fyi/results/css/css-transitions/transition-in-iframe-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-in-iframe-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-in-iframe-001.html)

<a id="ref-for-propdef-transition-property③"></a>

### <a id="transition-property-property"></a>2.1. <a id="the-transition-property-property-"></a>The [transition-property](#propdef-transition-property) Property

<a id="ref-for-propdef-transition-property④"></a>

The [transition-property](#propdef-transition-property) property specifies the name of the CSS property to which the transition is applied.

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-transition-property"></a>transition-property

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-single-transition-property"></a>

<a id="ref-for-comb-one"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<single-transition-property\>](#single-transition-property)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

all

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

<a id="ref-for-valdef-transition-property-none"></a>

the keyword [none](#valdef-transition-property-none) else a list of identifiers

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

Tests

- [move-after-transition.html](https://wpt.fyi/results/css/css-transitions/animations/move-after-transition.html) [(live test)](http://wpt.live/css/css-transitions/animations/move-after-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/move-after-transition.html)
- [transition-end-event-shorthands.html](https://wpt.fyi/results/css/css-transitions/animations/transition-end-event-shorthands.html) [(live test)](http://wpt.live/css/css-transitions/animations/transition-end-event-shorthands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/transition-end-event-shorthands.html)
- [transition-property-computed.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-property-computed.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-property-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-property-computed.html)
- [transition-property-invalid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-property-invalid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-property-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-property-invalid.html)
- [transition-property-valid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-property-valid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-property-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-property-valid.html)
- [pseudo-element-transform.html](https://wpt.fyi/results/css/css-transitions/pseudo-element-transform.html) [(live test)](http://wpt.live/css/css-transitions/pseudo-element-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/pseudo-element-transform.html)
- [pseudo-elements-001.html](https://wpt.fyi/results/css/css-transitions/pseudo-elements-001.html) [(live test)](http://wpt.live/css/css-transitions/pseudo-elements-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/pseudo-elements-001.html)
- [pseudo-elements-002.html](https://wpt.fyi/results/css/css-transitions/pseudo-elements-002.html) [(live test)](http://wpt.live/css/css-transitions/pseudo-elements-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/pseudo-elements-002.html)
- [transition-background-position-with-edge-offset.html](https://wpt.fyi/results/css/css-transitions/transition-background-position-with-edge-offset.html) [(live test)](http://wpt.live/css/css-transitions/transition-background-position-with-edge-offset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-background-position-with-edge-offset.html)
- [transition-property-001.html](https://wpt.fyi/results/css/css-transitions/transition-property-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-property-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-001.html)
- [transition-property-002.html](https://wpt.fyi/results/css/css-transitions/transition-property-002.html) [(live test)](http://wpt.live/css/css-transitions/transition-property-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-002.html)
- transition-property-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-003-manual.html)
- transition-property-004-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-004-manual.html)
- transition-property-005-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-005-manual.html)
- transition-property-006-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-006-manual.html)
- transition-property-007-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-007-manual.html)
- transition-property-008-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-008-manual.html)
- transition-property-009-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-009-manual.html)
- transition-property-010-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-010-manual.html)
- transition-property-011-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-011-manual.html)
- transition-property-012-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-012-manual.html)
- transition-property-013-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-013-manual.html)
- transition-property-014-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-014-manual.html)
- transition-property-015-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-015-manual.html)
- transition-property-016-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-016-manual.html)
- transition-property-017-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-017-manual.html)
- transition-property-018-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-018-manual.html)
- transition-property-019-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-019-manual.html)
- transition-property-020-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-020-manual.html)
- transition-property-021-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-021-manual.html)
- transition-property-022-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-022-manual.html)
- transition-property-023-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-023-manual.html)
- transition-property-024-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-024-manual.html)
- transition-property-025-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-025-manual.html)
- transition-property-026-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-026-manual.html)
- transition-property-027-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-027-manual.html)
- transition-property-028-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-028-manual.html)
- transition-property-029-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-029-manual.html)
- transition-property-030-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-030-manual.html)
- transition-property-031-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-031-manual.html)
- transition-property-032-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-032-manual.html)
- transition-property-033-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-033-manual.html)
- transition-property-034-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-034-manual.html)
- transition-property-035-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-035-manual.html)
- transition-property-036-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-036-manual.html)
- transition-property-037-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-037-manual.html)
- transition-property-038-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-038-manual.html)
- transition-property-039-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-039-manual.html)
- transition-property-040-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-040-manual.html)
- transition-property-041-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-041-manual.html)
- transition-property-042-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-042-manual.html)
- transition-property-043-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-043-manual.html)
- transition-property-044-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-044-manual.html)
- transition-property-045-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-property-045-manual.html)
- [transition-test.html](https://wpt.fyi/results/css/css-transitions/transition-test.html) [(live test)](http://wpt.live/css/css-transitions/transition-test.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-test.html)
- [transition-zero-duration-with-delay.html](https://wpt.fyi/results/css/css-transitions/transition-zero-duration-with-delay.html) [(live test)](http://wpt.live/css/css-transitions/transition-zero-duration-with-delay.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-zero-duration-with-delay.html)

<a id="ref-for-valdef-transition-property-all"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-identifier-value"></a>

<a id="single-transition-property"></a>\<single-transition-property\> = [all](#valdef-transition-property-all) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

A value of <a id="valdef-transition-property-none"></a>none means that no property will transition. Otherwise, a list of properties to be transitioned, or the keyword <a id="valdef-transition-property-all"></a>all which indicates that all properties are to be transitioned, is given.

<a id="ref-for-propdef-transition-duration"></a>

<a id="ref-for-propdef-transition-delay"></a>

<a id="ref-for-propdef-transition-timing-function"></a>

If one of the identifiers listed is not a recognized property name, the implementation must still start transitions on the animatable properties in the list using the duration, delay, and timing function at their respective indices in the lists for [transition-duration](#propdef-transition-duration), [transition-delay](#propdef-transition-delay), and [transition-timing-function](#propdef-transition-timing-function). In other words, unrecognized properties must be kept in the list to preserve the matching of indices.

<a id="ref-for-identifier-value①"></a>

<a id="ref-for-single-transition-property①"></a>

<a id="ref-for-valdef-transition-property-none①"></a>

<a id="ref-for-valdef-all-inherit"></a>

<a id="ref-for-valdef-all-initial"></a>

The [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) production in [\<single-transition-property\>](#single-transition-property) also excludes the keyword [none](#valdef-transition-property-none), in addition to the keywords always excluded from <a id="ref-for-identifier-value②"></a>\<custom-ident\>. This means that <a id="ref-for-valdef-transition-property-none②"></a>none, [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit), and [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) are not permitted as items within a list of more that one identifier; any list that uses them is syntactically invalid.

<a id="ref-for-valdef-transition-property-all①"></a>

For the keyword [all](#valdef-transition-property-all), or if one of the identifiers listed is a shorthand property, implementations must start transitions for all its longhand sub-properties (or, for <a id="ref-for-valdef-transition-property-all②"></a>all, all properties), using the duration, delay, and timing function at the index corresponding to the shorthand.

<a id="ref-for-propdef-transition-property⑤"></a>

<a id="ref-for-valdef-transition-property-all③"></a>

If a property is specified multiple times in the value of [transition-property](#propdef-transition-property) (either on its own, via a shorthand that contains it, or via the [all](#valdef-transition-property-all) value), then the transition that starts uses the duration, delay, and timing function at the index corresponding to the <em>last</em> item in the value of <a id="ref-for-propdef-transition-property⑥"></a>transition-property that calls for animating that property.

<a id="ref-for-valdef-transition-property-all④"></a>

<a id="ref-for-propdef-all"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [all](#valdef-transition-property-all) value and [all](https://www.w3.org/TR/css-cascade-5/#propdef-all) shorthand property work in similar ways, so the <a id="ref-for-valdef-transition-property-all⑤"></a>all value is just like a shorthand that covers all properties.

<a id="ref-for-propdef-transition-duration①"></a>

### <a id="transition-duration-property"></a>2.2. <a id="the-transition-duration-property-"></a>The [transition-duration](#propdef-transition-duration) Property

<a id="ref-for-propdef-transition-duration②"></a>

The [transition-duration](#propdef-transition-duration) property defines the length of time that a transition takes.

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-transition-duration"></a>transition-duration

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

Tests

- [infinite-duration-crash.html](https://wpt.fyi/results/css/css-transitions/infinite-duration-crash.html) [(live test)](http://wpt.live/css/css-transitions/infinite-duration-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/infinite-duration-crash.html)
- [transition-duration-computed.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-duration-computed.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-duration-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-duration-computed.html)
- [transition-duration-invalid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-duration-invalid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-duration-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-duration-invalid.html)
- [transition-duration-valid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-duration-valid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-duration-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-duration-valid.html)
- [transition-duration-001.html](https://wpt.fyi/results/css/css-transitions/transition-duration-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-duration-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-duration-001.html)
- transition-duration-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-duration-002-manual.html)
- transition-duration-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-duration-003-manual.html)
- transition-duration-004-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-duration-004-manual.html)
- [transition-duration-shorthand.html](https://wpt.fyi/results/css/css-transitions/transition-duration-shorthand.html) [(live test)](http://wpt.live/css/css-transitions/transition-duration-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-duration-shorthand.html)

<a id="ref-for-propdef-transition-duration③"></a>

This property specifies how long the transition from the old value to the new value should take. By default the value is 0s, meaning that the transition is immediate (i.e. there will be no animation). A negative value for [transition-duration](#propdef-transition-duration) renders the declaration invalid.

<a id="ref-for-propdef-transition-timing-function①"></a>

### <a id="transition-timing-function-property"></a>2.3. <a id="transition-timing-function_tag"></a>The [transition-timing-function](#propdef-transition-timing-function) Property

<a id="ref-for-propdef-transition-timing-function②"></a>

The [transition-timing-function](#propdef-transition-timing-function) property describes how the intermediate values used during a transition will be calculated. It allows for a transition to change speed over its duration. These effects are commonly called <em>easing</em> functions.

<a id="ref-for-input-progress-value"></a>

<a id="ref-for-output-progress-value"></a>

<a id="ref-for-interpolation"></a>

Timing functions are defined in the separate CSS Easing Functions module [\[css-easing-1\]](#biblio-css-easing-1). The [input progress value](https://www.w3.org/TR/css-easing-2/#input-progress-value) used is the percentage of the transition duration, and the [output progress value](https://www.w3.org/TR/css-easing-2/#output-progress-value) is used as the <var>p</var> value when [interpolating](https://www.w3.org/TR/css-values-4/#interpolation) the property value (see [§ 4 Application of transitions](#application)).

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-transition-timing-function"></a>transition-timing-function

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-typedef-easing-function"></a>

[\<easing-function\>](https://www.w3.org/TR/css-easing-2/#typedef-easing-function)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

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

not animatable

Tests

- [transition-timing-function.html](https://wpt.fyi/results/css/css-transitions/animations/transition-timing-function.html) [(live test)](http://wpt.live/css/css-transitions/animations/transition-timing-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/transition-timing-function.html)
- [transition-timing-function-computed.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-timing-function-computed.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-timing-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-timing-function-computed.html)
- [transition-timing-function-invalid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-timing-function-invalid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-timing-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-timing-function-invalid.html)
- [transition-timing-function-valid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-timing-function-valid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-timing-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-timing-function-valid.html)
- transition-timing-function-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-002-manual.html)
- transition-timing-function-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-003-manual.html)
- transition-timing-function-004-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-004-manual.html)
- transition-timing-function-005-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-005-manual.html)
- transition-timing-function-006-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-006-manual.html)
- transition-timing-function-010-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-timing-function-010-manual.html)

<a id="ref-for-propdef-transition-delay①"></a>

### <a id="transition-delay-property"></a>2.4. <a id="the-transition-delay-property-"></a>The [transition-delay](#propdef-transition-delay) Property

<a id="ref-for-propdef-transition-delay②"></a>

The [transition-delay](#propdef-transition-delay) property defines when the transition will start. It allows a transition to begin execution after some period of time from when it is applied. A <a id="ref-for-propdef-transition-delay③"></a>transition-delay value of 0s means the transition will execute as soon as the property is changed. Otherwise, the value specifies an offset from the moment the property is changed, and the transition will delay execution by that offset.

<a id="ref-for-propdef-transition-delay④"></a>

If the value for [transition-delay](#propdef-transition-delay) is a negative time offset then the transition will execute the moment the property is changed, but will appear to have begun execution at the specified offset. That is, the transition will appear to begin part-way through its play cycle. In the case where a transition has implied starting values and a negative <a id="ref-for-propdef-transition-delay⑤"></a>transition-delay, the starting values are taken from the moment the property is changed.

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-transition-delay"></a>transition-delay

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-time-value①"></a>

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

Tests

- [transition-delay-computed.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-delay-computed.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-delay-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-delay-computed.html)
- [transition-delay-invalid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-delay-invalid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-delay-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-delay-invalid.html)
- [transition-delay-valid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-delay-valid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-delay-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-delay-valid.html)
- transition-delay-000-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-delay-000-manual.html)
- [transition-delay-001.html](https://wpt.fyi/results/css/css-transitions/transition-delay-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-delay-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-delay-001.html)
- transition-delay-002-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-delay-002-manual.html)
- transition-delay-003-manual.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-delay-003-manual.html)

<a id="ref-for-propdef-transition"></a>

### <a id="transition-shorthand-property"></a>2.5. <a id="the-transition-shorthand-property-"></a>The [transition](#propdef-transition) Shorthand Property

<a id="ref-for-propdef-transition①"></a>

The [transition](#propdef-transition) shorthand property combines the four properties described above into a single property.

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-transition"></a>transition

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-comma④"></a>

<a id="ref-for-single-transition"></a>

[\<single-transition\>](#single-transition)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)

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

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [transition-computed.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-computed.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-computed.html)
- [transition-invalid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-invalid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-invalid.html)
- [transition-valid.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-valid.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-valid.html)
- [transition-shorthand.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-shorthand.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-shorthand.html)
- [transition-001.html](https://wpt.fyi/results/css/css-transitions/transition-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-001.html)

<a id="ref-for-valdef-transition-property-none③"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-single-transition-property②"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-time-value②"></a>

<a id="ref-for-typedef-easing-function①"></a>

<a id="single-transition"></a>\<single-transition\> = \[ [none](#valdef-transition-property-none) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<single-transition-property\>](#single-transition-property) \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<time\>](https://www.w3.org/TR/css-values-3/#time-value) <a id="ref-for-comb-any①"></a>\|\| [\<easing-function\>](https://www.w3.org/TR/css-easing-2/#typedef-easing-function) <a id="ref-for-comb-any②"></a>\|\| <a id="ref-for-time-value③"></a>\<time\>

Note that order is important within the items in this property: the first value that can be parsed as a time is assigned to the transition-duration, and the second value that can be parsed as a time is assigned to transition-delay.

<a id="ref-for-single-transition①"></a>

<a id="ref-for-valdef-transition-property-none④"></a>

<a id="ref-for-single-transition-property③"></a>

If there is more than one [\<single-transition\>](#single-transition) in the shorthand, and any of the transitions has [none](#valdef-transition-property-none) as the [\<single-transition-property\>](#single-transition-property), then the declaration is invalid.

## <a id="starting"></a>3. Starting of transitions

<a id="ref-for-dfn-complete"></a>

<a id="ref-for-transition-reversing-adjusted-start-value"></a>

<a id="ref-for-transition-reversing-shortening-factor"></a>

Implementations must maintain a set of <a id="running-transition"></a>running transitions, each of which applies to a specific element and non-shorthand property. Each of these transitions also has a <a id="transition-start-time"></a>start time, <a id="transition-end-time"></a>end time, <a id="transition-start-value"></a>start value, <a id="transition-end-value"></a>end value, <a id="transition-reversing-adjusted-start-value"></a>reversing-adjusted start value, and <a id="transition-reversing-shortening-factor"></a>reversing shortening factor. Transitions are added to this set as described in this section, and are removed from this set when they [complete](#dfn-complete) or when implementations are required to <a id="transition-cancel"></a>cancel them. <strong data-conversion-semantic="note">Note:</strong> For the rationale behind the [reversing-adjusted start value](#transition-reversing-adjusted-start-value) and [reversing shortening factor](#transition-reversing-shortening-factor), see [§ 3.1 Faster reversing of interrupted transitions](#reversing).

Tests

- [transitioncancel-003.html](https://wpt.fyi/results/css/css-transitions/transitioncancel-003.html) [(live test)](http://wpt.live/css/css-transitions/transitioncancel-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transitioncancel-003.html)

<a id="ref-for-running-transition"></a>

<a id="ref-for-running-transition①"></a>

<a id="ref-for-completed-transition"></a>

Implementations must also maintain a set of <a id="completed-transition"></a>completed transitions, each of which (like [running transitions](#running-transition)) applies to a specific element and non-shorthand property. <strong data-conversion-semantic="note">Note:</strong> This specification maintains the invariant that there is never both a [running transition](#running-transition) and a [completed transition](#completed-transition) for the same property and element.

<a id="ref-for-transition-cancel"></a>

<a id="ref-for-running-transition②"></a>

<a id="ref-for-completed-transition①"></a>

If an element is no longer in the document, implementations must [cancel](#transition-cancel) any [running transitions](#running-transition) on it and remove transitions on it from the [completed transitions](#completed-transition).

> <strong data-conversion-semantic="note">Note</strong>
>
> This set of completed transitions needs to be maintained in order to prevent transitions from repeating themselves in certain cases, i.e., to maintain the invariant that this specification tries to maintain that unrelated style changes do not trigger transitions.
>
> <a id="ref-for-propdef-transition②"></a>
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-4b88a0cc"></a> An example where maintaining the set of completed transitions is necessary would be a transition on an inherited property, where the parent specifies a transition of that property for a longer duration (say, [transition: 4s text-indent](#propdef-transition)) and a child element that inherits the parent’s value specifies a transition of the same property for a shorter duration (say, <a id="ref-for-propdef-transition③"></a>transition: 1s text-indent). Without the maintenance of this set of completed transitions, implementations could start additional transitions on the child after the initial 1 second transition on the child completes.

<a id="ref-for-computed-value②"></a>

<a id="ref-for-style-change-event"></a>

Various things can cause the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of properties on an element to change. These include insertion and removal of elements from the document tree (which both changes whether those elements have <a id="ref-for-computed-value③"></a>computed values and can change the styles of other elements through selector matching), changes to the document tree that cause changes to which selectors match elements, changes to style sheets or style attributes, and other things. This specification does not define when <a id="ref-for-computed-value④"></a>computed values are updated, beyond saying that implementations must not use, present, or display something resulting from the CSS cascading, value computation, and inheritance process [\[CSS3CASCADE\]](#biblio-css3cascade) without updating the <a id="ref-for-computed-value⑤"></a>computed value (which means merely that implementations cannot avoid meeting requirements of this specification by claiming not to have updated the <a id="ref-for-computed-value⑥"></a>computed value as part of handling a style change). However, when an implementation updates the <a id="ref-for-computed-value⑦"></a>computed value of a property on an element to reflect one of these changes, or computes the <a id="ref-for-computed-value⑧"></a>computed value of a property on an element newly added to the document, it must update the <a id="ref-for-computed-value⑨"></a>computed value for all properties and elements to reflect all of these changes at the same time (or at least it must be undetectable that it was done at a different time). This processing of a set of simultaneous style changes is called a <a id="style-change-event"></a>style change event. (Implementations typically have a [style change event](#style-change-event) to correspond with their desired screen refresh rate, and when up-to-date computed style or layout information is needed for a script API that depends on it.)

Tests

- [no-transition-from-ua-to-blocking-stylesheet.html](https://wpt.fyi/results/css/css-transitions/render-blocking/no-transition-from-ua-to-blocking-stylesheet.html) [(live test)](http://wpt.live/css/css-transitions/render-blocking/no-transition-from-ua-to-blocking-stylesheet.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/render-blocking/no-transition-from-ua-to-blocking-stylesheet.html)

<a id="ref-for-style-change-event①"></a>

Since this specification does not define when a [style change event](#style-change-event) occurs, and thus what changes to computed values are considered simultaneous, authors should be aware that changing any of the transition properties a small amount of time after making a change that might transition can result in behavior that varies between implementations, since the changes might be considered simultaneous in some implementations but not others.

<a id="ref-for-style-change-event②"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-before-change-style"></a>

<a id="ref-for-after-change-style"></a>

When a [style change event](#style-change-event) occurs, implementations must start transitions based on the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) that changed in that event. If an element is not in the document during that style change event or was not in the document during the previous style change event, then transitions are not started for that element in that style change event. Otherwise, define the <a id="before-change-style"></a>before-change style as the <a id="ref-for-computed-value①①"></a>computed values of all properties on the element as of the previous <a id="ref-for-style-change-event③"></a>style change event, except with any styles derived from declarative animations such as CSS Transitions, CSS Animations ([\[CSS3-ANIMATIONS\]](#biblio-css3-animations)), and SMIL Animations ([\[SMIL-ANIMATION\]](#biblio-smil-animation), [\[SVG11\]](#biblio-svg11)) updated to the current time. Likewise, define the <a id="after-change-style"></a>after-change style as the <a id="ref-for-computed-value①②"></a>computed values of all properties on the element based on the information known at the start of that <a id="ref-for-style-change-event④"></a>style change event, but using the computed values of the animation-\* properties from the [before-change style](#before-change-style), excluding any styles from CSS Transitions in the computation, and inheriting from the [after-change style](#after-change-style) of the parent. Note that this means the <a id="ref-for-after-change-style①"></a>after-change style does not differ from the <a id="ref-for-before-change-style①"></a>before-change style due to newly created or canceled CSS Animations.

Tests

- [after-change-style-inherited-try-fallback.html](https://wpt.fyi/results/css/css-transitions/after-change-style-inherited-try-fallback.html) [(live test)](http://wpt.live/css/css-transitions/after-change-style-inherited-try-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/after-change-style-inherited-try-fallback.html)
- [after-change-style-inherited.html](https://wpt.fyi/results/css/css-transitions/after-change-style-inherited.html) [(live test)](http://wpt.live/css/css-transitions/after-change-style-inherited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/after-change-style-inherited.html)
- [change-duration-during-transition.html](https://wpt.fyi/results/css/css-transitions/animations/change-duration-during-transition.html) [(live test)](http://wpt.live/css/css-transitions/animations/change-duration-during-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/change-duration-during-transition.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-after-change-style②"></a>
>
> Note that this definition of the [after-change style](#after-change-style) means that a single change can start a transition on the same property on both an ancestor element and its descendant element. This can happen when a property change is inherited from one element with transition-\* properties that say to animate the changing property to another element with transition-\* properties that also say to animate the changing property.
>
> When this happens, both transitions will run, and the transition on the descendant will override the transition on the ancestor because of the normal CSS cascading and inheritance rules ([\[CSS3CASCADE\]](#biblio-css3cascade)).
>
> If the transition on the descendant completes before the transition on the ancestor, the descendant will then resume inheriting the (still transitioning) value from its parent. This effect is likely not a desirable effect, but it is essentially doing what the author asked for.

<a id="ref-for-before-change-style②"></a>

<a id="ref-for-after-change-style③"></a>

<a id="ref-for-propdef-transition-property⑦"></a>

<a id="ref-for-propdef-transition-duration④"></a>

<a id="ref-for-propdef-transition-delay⑥"></a>

<a id="ref-for-propdef-transition-timing-function③"></a>

<a id="ref-for-matching-transition-duration"></a>

<a id="ref-for-matching-transition-delay"></a>

For each element with a [before-change style](#before-change-style) and an [after-change style](#after-change-style), and each property (other than shorthands), define the <a id="matching-transition-property-value"></a>matching transition-property value as the last value in the [transition-property](#propdef-transition-property) in the element’s <a id="ref-for-after-change-style④"></a>after-change style that matches the property, as described in [§ 2.1 The transition-property Property](#transition-property-property). If there is such a value, then corresponding to it, there is a <a id="matching-transition-duration"></a>matching transition duration, a <a id="matching-transition-delay"></a>matching transition delay, and a <a id="matching-transition-timing-function"></a>matching transition timing function in the values in the <a id="ref-for-after-change-style⑤"></a>after-change style of [transition-duration](#propdef-transition-duration), [transition-delay](#propdef-transition-delay), and [transition-timing-function](#propdef-transition-timing-function) (see [the rules on matching lists](#list-matching)). Define the <a id="transition-combined-duration"></a>combined duration of the transition as the sum of max([matching transition duration](#matching-transition-duration), 0s) and the [matching transition delay](#matching-transition-delay).

<a id="ref-for-before-change-style③"></a>

<a id="ref-for-after-change-style⑥"></a>

<a id="ref-for-animation-type"></a>

<a id="ref-for-not-animatable"></a>

<a id="ref-for-discrete"></a>

When comparing the [before-change style](#before-change-style) and [after-change style](#after-change-style) for a given property, the property values are <a id="transitionable"></a>transitionable if they have an [animation type](https://www.w3.org/TR/web-animations-1/#animation-type) that is <em>neither</em> [not animatable](https://www.w3.org/TR/web-animations-1/#not-animatable) <em>nor</em> [discrete](https://www.w3.org/TR/web-animations-1/#discrete).

<a id="ref-for-animation-type①"></a>

<a id="ref-for-discrete①"></a>

<a id="ref-for-propdef-box-shadow"></a>

<a id="ref-for-combining-shadow-lists"></a>

<a id="ref-for-shadow-inset"></a>

<a id="ref-for-transitionable"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even if a <em>property</em> is defined to have an [animation type](https://www.w3.org/TR/web-animations-1/#animation-type) that is <em>not</em> [discrete](https://www.w3.org/TR/web-animations-1/#discrete), for a particular pair of <em>property values</em> the <a id="ref-for-animation-type②"></a>animation type may be <a id="ref-for-discrete②"></a>discrete. For example, the <a id="ref-for-animation-type③"></a>animation type of the [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) property is [shadow list](https://www.w3.org/TR/web-animations-1/#combining-shadow-lists), which defines that when the [inset](https://www.w3.org/TR/css3-background/#shadow-inset) keyword is absent in one value but present in the other, <a id="ref-for-discrete③"></a>discrete animation is used. As a result 0px 0px black and inset 10px 10px black are <em>not</em> [transitionable](#transitionable).

For each element and property, the implementation must act as follows:

1.  <a id="ref-for-completed-transition④"></a>

    If all of the following are true:

    - <a id="ref-for-running-transition③"></a>

      the element does not have a [running transition](#running-transition) for the property,

    - <a id="ref-for-transitionable①"></a>

      <a id="ref-for-after-change-style⑦"></a>

      <a id="ref-for-before-change-style④"></a>

      the [before-change style](#before-change-style) is different from the [after-change style](#after-change-style) for that property, and the values for the property are [transitionable](#transitionable),

    - <a id="ref-for-after-change-style⑧"></a>

      <a id="ref-for-transition-end-value"></a>

      <a id="ref-for-completed-transition②"></a>

      the element does not have a [completed transition](#completed-transition) for the property or the [end value](#transition-end-value) of the <a id="ref-for-completed-transition③"></a>completed transition is different from the [after-change style](#after-change-style) for the property,

    - <a id="ref-for-matching-transition-property-value"></a>

      there is a [matching transition-property value](#matching-transition-property-value), and

    - <a id="ref-for-transition-combined-duration"></a>

      the [combined duration](#transition-combined-duration) is greater than 0s,

    then implementations must remove the [completed transition](#completed-transition) (if present) from the set of completed transitions and start a transition whose:

    - <a id="ref-for-matching-transition-delay①"></a>

      <a id="ref-for-style-change-event⑤"></a>

      <a id="ref-for-transition-start-time"></a>

      [start time](#transition-start-time) is the time of the [style change event](#style-change-event) plus the [matching transition delay](#matching-transition-delay),

    - <a id="ref-for-matching-transition-duration①"></a>

      <a id="ref-for-transition-start-time①"></a>

      <a id="ref-for-transition-end-time"></a>

      [end time](#transition-end-time) is the [start time](#transition-start-time) plus the [matching transition duration](#matching-transition-duration),

    - <a id="ref-for-before-change-style⑤"></a>

      <a id="ref-for-transition-start-value"></a>

      [start value](#transition-start-value) is the value of the transitioning property in the [before-change style](#before-change-style),

    - <a id="ref-for-after-change-style⑨"></a>

      <a id="ref-for-transition-end-value①"></a>

      [end value](#transition-end-value) is the value of the transitioning property in the [after-change style](#after-change-style),

    - <a id="ref-for-transition-start-value①"></a>

      <a id="ref-for-transition-reversing-adjusted-start-value①"></a>

      [reversing-adjusted start value](#transition-reversing-adjusted-start-value) is the same as the [start value](#transition-start-value), and

    - <a id="ref-for-transition-reversing-shortening-factor①"></a>

      [reversing shortening factor](#transition-reversing-shortening-factor) is 1.

2.  <a id="ref-for-after-change-style①⓪"></a>

    <a id="ref-for-transition-end-value②"></a>

    <a id="ref-for-completed-transition⑤"></a>

    Otherwise, if the element has a [completed transition](#completed-transition) for the property and the [end value](#transition-end-value) of the <a id="ref-for-completed-transition⑥"></a>completed transition is different from the [after-change style](#after-change-style) for the property, then implementations must remove the <a id="ref-for-completed-transition⑦"></a>completed transition from the set of <a id="ref-for-completed-transition⑧"></a>completed transitions.

3.  <a id="ref-for-transition-cancel①"></a>

    <a id="ref-for-matching-transition-property-value①"></a>

    <a id="ref-for-completed-transition⑨"></a>

    <a id="ref-for-running-transition④"></a>

    If the element has a [running transition](#running-transition) or [completed transition](#completed-transition) for the property, and there is <strong>not</strong> a [matching transition-property value](#matching-transition-property-value), then implementations must [cancel](#transition-cancel) the <a id="ref-for-running-transition⑤"></a>running transition or remove the <a id="ref-for-completed-transition①⓪"></a>completed transition from the set of <a id="ref-for-completed-transition①①"></a>completed transitions.

4.  <a id="ref-for-after-change-style①①"></a>

    <a id="ref-for-transition-end-value③"></a>

    <a id="ref-for-matching-transition-property-value②"></a>

    <a id="ref-for-running-transition⑥"></a>

    If the element has a [running transition](#running-transition) for the property, there is a [matching transition-property value](#matching-transition-property-value), and the [end value](#transition-end-value) of the <a id="ref-for-running-transition⑦"></a>running transition is <strong>not</strong> equal to the value of the property in the [after-change style](#after-change-style), then:

    1.  <a id="ref-for-transition-cancel②"></a>

        <a id="ref-for-transitionable②"></a>

        <a id="ref-for-after-change-style①②"></a>

        <a id="ref-for-running-transition⑧"></a>

        <a id="ref-for-current-value"></a>

        If the [current value](#current-value) of the property in the [running transition](#running-transition) is equal to the value of the property in the [after-change style](#after-change-style), or if these two values are not [transitionable](#transitionable), then implementations must [cancel](#transition-cancel) the <a id="ref-for-running-transition⑨"></a>running transition.

    2.  <a id="ref-for-transition-cancel③"></a>

        <a id="ref-for-after-change-style①③"></a>

        <a id="ref-for-transitionable③"></a>

        <a id="ref-for-running-transition①⓪"></a>

        <a id="ref-for-current-value①"></a>

        <a id="ref-for-transition-combined-duration①"></a>

        Otherwise, if the [combined duration](#transition-combined-duration) is less than or equal to 0s, or if the [current value](#current-value) of the property in the [running transition](#running-transition) is not [transitionable](#transitionable) with the value of the property in the [after-change style](#after-change-style), then implementations must [cancel](#transition-cancel) the <a id="ref-for-running-transition①①"></a>running transition.

    3.  <a id="ref-for-transition-cancel④"></a>

        <a id="ref-for-after-change-style①④"></a>

        <a id="ref-for-running-transition①②"></a>

        <a id="ref-for-transition-reversing-adjusted-start-value②"></a>

        Otherwise, if the [reversing-adjusted start value](#transition-reversing-adjusted-start-value) of the [running transition](#running-transition) is the same as the value of the property in the [after-change style](#after-change-style) <strong data-conversion-semantic="note">Note:</strong> (see the [section on reversing of transitions](#reversing) for why these case exists), implementations must [cancel](#transition-cancel) the <a id="ref-for-running-transition①③"></a>running transition and start a new transition whose:

        - <a id="ref-for-running-transition①④"></a>

          <a id="ref-for-transition-end-value④"></a>

          <a id="ref-for-transition-reversing-adjusted-start-value③"></a>

          [reversing-adjusted start value](#transition-reversing-adjusted-start-value) is the [end value](#transition-end-value) of the [running transition](#running-transition) <strong data-conversion-semantic="note">Note:</strong> (Note: This represents the logical start state of the transition, and allows some calculations to ignore that the transition started before that state was reached, which in turn allows repeated reversals of the same transition to work correctly),

        - <a id="ref-for-transition-end-value⑤"></a>

          <a id="ref-for-transition-reversing-adjusted-start-value④"></a>

          <a id="ref-for-transition-reversing-shortening-factor②"></a>

          [reversing shortening factor](#transition-reversing-shortening-factor) is the absolute value, clamped to the range \[0, 1\], of the sum of:

          1.  <a id="ref-for-transition-reversing-shortening-factor③"></a>

              <a id="ref-for-style-change-event⑥"></a>

              the output of the timing function of the old transition at the time of the [style change event](#style-change-event), times the [reversing shortening factor](#transition-reversing-shortening-factor) of the old transition

          2.  <a id="ref-for-transition-reversing-shortening-factor④"></a>

              1 minus the [reversing shortening factor](#transition-reversing-shortening-factor) of the old transition.

          <strong data-conversion-semantic="note">Note:</strong> Note: This represents the portion of the space between the [reversing-adjusted start value](#transition-reversing-adjusted-start-value) and the [end value](#transition-end-value) that the old transition has traversed (in amounts of the value, not time), except with the absolute value and clamping to handle timing functions that have y1 or y2 outside the range \[0, 1\].

        - <a id="ref-for-style-change-event⑦"></a>

          <a id="ref-for-transition-start-time②"></a>

          [start time](#transition-start-time) is the time of the [style change event](#style-change-event) plus:

          1.  <a id="ref-for-matching-transition-delay②"></a>

              if the [matching transition delay](#matching-transition-delay) is nonnegative, the <a id="ref-for-matching-transition-delay③"></a>matching transition delay, or

          2.  <a id="ref-for-transition-reversing-shortening-factor⑤"></a>

              <a id="ref-for-matching-transition-delay④"></a>

              if the [matching transition delay](#matching-transition-delay) is negative, the product of the new transition’s [reversing shortening factor](#transition-reversing-shortening-factor) and the <a id="ref-for-matching-transition-delay⑤"></a>matching transition delay,

        - <a id="ref-for-transition-reversing-shortening-factor⑥"></a>

          <a id="ref-for-matching-transition-duration②"></a>

          <a id="ref-for-transition-start-time③"></a>

          <a id="ref-for-transition-end-time①"></a>

          [end time](#transition-end-time) is the [start time](#transition-start-time) plus the product of the [matching transition duration](#matching-transition-duration) and the new transition’s [reversing shortening factor](#transition-reversing-shortening-factor),

        - <a id="ref-for-running-transition①⑤"></a>

          <a id="ref-for-current-value②"></a>

          <a id="ref-for-transition-start-value②"></a>

          [start value](#transition-start-value) is the [current value](#current-value) of the property in the [running transition](#running-transition),

        - <a id="ref-for-after-change-style①⑤"></a>

          <a id="ref-for-transition-end-value⑥"></a>

          [end value](#transition-end-value) is the value of the property in the [after-change style](#after-change-style),

    4.  <a id="ref-for-running-transition①⑥"></a>

        <a id="ref-for-transition-cancel⑤"></a>

        Otherwise, implementations must [cancel](#transition-cancel) the [running transition](#running-transition) and start a new transition whose:

        - <a id="ref-for-matching-transition-delay⑥"></a>

          <a id="ref-for-style-change-event⑧"></a>

          <a id="ref-for-transition-start-time④"></a>

          [start time](#transition-start-time) is the time of the [style change event](#style-change-event) plus the [matching transition delay](#matching-transition-delay),

        - <a id="ref-for-matching-transition-duration③"></a>

          <a id="ref-for-transition-start-time⑤"></a>

          <a id="ref-for-transition-end-time②"></a>

          [end time](#transition-end-time) is the [start time](#transition-start-time) plus the [matching transition duration](#matching-transition-duration),

        - <a id="ref-for-running-transition①⑦"></a>

          <a id="ref-for-current-value③"></a>

          <a id="ref-for-transition-start-value③"></a>

          [start value](#transition-start-value) is the [current value](#current-value) of the property in the [running transition](#running-transition),

        - <a id="ref-for-after-change-style①⑥"></a>

          <a id="ref-for-transition-end-value⑦"></a>

          [end value](#transition-end-value) is the value of the property in the [after-change style](#after-change-style),

        - <a id="ref-for-transition-start-value④"></a>

          <a id="ref-for-transition-reversing-adjusted-start-value⑤"></a>

          [reversing-adjusted start value](#transition-reversing-adjusted-start-value) is the same as the [start value](#transition-start-value), and

        - <a id="ref-for-transition-reversing-shortening-factor⑦"></a>

          [reversing shortening factor](#transition-reversing-shortening-factor) is 1.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-propdef-transition-property⑧"></a>
>
> <a id="ref-for-propdef-transition-duration⑤"></a>
>
> <a id="ref-for-propdef-transition-timing-function④"></a>
>
> <a id="ref-for-propdef-transition-delay⑦"></a>
>
> Note that the above rules mean that when the computed value of an animatable property changes, the transitions that start are based on the values of the [transition-property](#propdef-transition-property), [transition-duration](#propdef-transition-duration), [transition-timing-function](#propdef-transition-timing-function), and [transition-delay](#propdef-transition-delay) properties at the time the animatable property would first have its new computed value. This means that when one of these transition-\* properties changes at the same time as a property whose change might transition, it is the <em>new</em> values of the transition-\* properties that control the transition.
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="manual-reversing-example"></a>
> >
> > Example(s):
> >
> > <a id="ref-for-propdef-transition-duration⑥"></a>
> >
> > <a id="ref-for-propdef-transition-timing-function⑤"></a>
> >
> > <a id="ref-for-propdef-transition-delay⑧"></a>
> >
> > This provides a way for authors to specify different values of the transition-\* properties for the “forward” and “reverse” transitions, when the transitions are between two states (but see [below](#reversing) for special reversing behavior when an <em>incomplete</em> transition is interrupted). Authors can specify the value of [transition-duration](#propdef-transition-duration), [transition-timing-function](#propdef-transition-timing-function), or [transition-delay](#propdef-transition-delay) in the same rule where they specify the value that triggers the transition, or can change these properties at the same time as they change the property that triggers the transition. Since it’s the new values of these transition-\* properties that affect the transition, these values will be used for the transitions <em>to</em> the associated transitioning values. For example:
> >
> > ```text
> > li {
> >   transition: background-color linear 1s;
> >   background: blue;
> > }
> > li:hover {
> >   background-color: green;
> >   transition-duration: 2s; /* applies to the transition *to* the :hover state */
> > }
> > ```
> >
> > <a id="ref-for-propdef-transition-duration⑦"></a>
> >
> > <a id="ref-for-propdef-background-color②"></a>
> >
> > <a id="ref-for-valdef-color-green"></a>
> >
> > <a id="ref-for-valdef-color-blue"></a>
> >
> > When a list item with these style rules enters the :hover state, the computed [transition-duration](#propdef-transition-duration) at the time that [background-color](https://www.w3.org/TR/CSS2/colors.html#propdef-background-color) would have its new value ([green](https://www.w3.org/TR/css-color-4/#valdef-color-green)) is 2s, so the transition from [blue](https://www.w3.org/TR/css-color-4/#valdef-color-blue) to <a id="ref-for-valdef-color-green①"></a>green takes 2 seconds. However, when the list item leaves the :hover state, the transition from <a id="ref-for-valdef-color-green②"></a>green to <a id="ref-for-valdef-color-blue①"></a>blue takes 1 second.

<a id="ref-for-propdef-transition-timing-function⑥"></a>

<a id="ref-for-propdef-transition-duration⑧"></a>

<a id="ref-for-propdef-transition-delay⑨"></a>

<a id="ref-for-propdef-transition-property⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that once the transition of a property has started (including being in its delay phase), it continues running based on the original timing function, duration, and delay, even if the [transition-timing-function](#propdef-transition-timing-function), [transition-duration](#propdef-transition-duration), or [transition-delay](#propdef-transition-delay) property changes before the transition is complete. However, if the [transition-property](#propdef-transition-property) property changes such that the transition would not have started, the transition stops (and the property immediately changes to its final value).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that above rules mean that transitions do not start when the computed value of a property changes as a result of declarative animation (as opposed to scripted animation). This happens because the before-change style includes up-to-date style for declarative animations.

Tests

- [before-load-001.html](https://wpt.fyi/results/css/css-transitions/before-load-001.html) [(live test)](http://wpt.live/css/css-transitions/before-load-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/before-load-001.html)
- [changing-while-transition-001.html](https://wpt.fyi/results/css/css-transitions/changing-while-transition-001.html) [(live test)](http://wpt.live/css/css-transitions/changing-while-transition-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/changing-while-transition-001.html)
- [changing-while-transition-002.html](https://wpt.fyi/results/css/css-transitions/changing-while-transition-002.html) [(live test)](http://wpt.live/css/css-transitions/changing-while-transition-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/changing-while-transition-002.html)
- [changing-while-transition-003.html](https://wpt.fyi/results/css/css-transitions/changing-while-transition-003.html) [(live test)](http://wpt.live/css/css-transitions/changing-while-transition-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/changing-while-transition-003.html)
- [changing-while-transition-004.html](https://wpt.fyi/results/css/css-transitions/changing-while-transition-004.html) [(live test)](http://wpt.live/css/css-transitions/changing-while-transition-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/changing-while-transition-004.html)
- [currentcolor-animation-001.html](https://wpt.fyi/results/css/css-transitions/currentcolor-animation-001.html) [(live test)](http://wpt.live/css/css-transitions/currentcolor-animation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/currentcolor-animation-001.html)
- [disconnected-element-001.html](https://wpt.fyi/results/css/css-transitions/disconnected-element-001.html) [(live test)](http://wpt.live/css/css-transitions/disconnected-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/disconnected-element-001.html)
- [dynamic-root-element.html](https://wpt.fyi/results/css/css-transitions/dynamic-root-element.html) [(live test)](http://wpt.live/css/css-transitions/dynamic-root-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/dynamic-root-element.html)
- [historical.html](https://wpt.fyi/results/css/css-transitions/historical.html) [(live test)](http://wpt.live/css/css-transitions/historical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/historical.html)
- [inherit-height-transition.html](https://wpt.fyi/results/css/css-transitions/inherit-height-transition.html) [(live test)](http://wpt.live/css/css-transitions/inherit-height-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/inherit-height-transition.html)
- [non-rendered-element-001.html](https://wpt.fyi/results/css/css-transitions/non-rendered-element-001.html) [(live test)](http://wpt.live/css/css-transitions/non-rendered-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/non-rendered-element-001.html)
- [non-rendered-element-002.html](https://wpt.fyi/results/css/css-transitions/non-rendered-element-002.html) [(live test)](http://wpt.live/css/css-transitions/non-rendered-element-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/non-rendered-element-002.html)
- [properties-value-inherit-001.html](https://wpt.fyi/results/css/css-transitions/properties-value-inherit-001.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-inherit-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-inherit-001.html)
- [properties-value-inherit-002.html](https://wpt.fyi/results/css/css-transitions/properties-value-inherit-002.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-inherit-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-inherit-002.html)
- [properties-value-inherit-003.html](https://wpt.fyi/results/css/css-transitions/properties-value-inherit-003.html) [(live test)](http://wpt.live/css/css-transitions/properties-value-inherit-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/properties-value-inherit-003.html)
- [starting-of-transitions-001.html](https://wpt.fyi/results/css/css-transitions/starting-of-transitions-001.html) [(live test)](http://wpt.live/css/css-transitions/starting-of-transitions-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-of-transitions-001.html)
- [root-color-transition.html](https://wpt.fyi/results/css/css-transitions/root-color-transition.html) [(live test)](http://wpt.live/css/css-transitions/root-color-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/root-color-transition.html)
- [transition-after-animation-001.html](https://wpt.fyi/results/css/css-transitions/transition-after-animation-001.html) [(live test)](http://wpt.live/css/css-transitions/transition-after-animation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-after-animation-001.html)
- [transition-remove-and-change-immediate.html](https://wpt.fyi/results/css/css-transitions/transition-remove-and-change-immediate.html) [(live test)](http://wpt.live/css/css-transitions/transition-remove-and-change-immediate.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-remove-and-change-immediate.html)
- [transition-reparented.html](https://wpt.fyi/results/css/css-transitions/transition-reparented.html) [(live test)](http://wpt.live/css/css-transitions/transition-reparented.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-reparented.html)
- [transitions-retarget.html](https://wpt.fyi/results/css/css-transitions/transitions-retarget.html) [(live test)](http://wpt.live/css/css-transitions/transitions-retarget.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transitions-retarget.html)
- [zero-duration-multiple-transition.html](https://wpt.fyi/results/css/css-transitions/zero-duration-multiple-transition.html) [(live test)](http://wpt.live/css/css-transitions/zero-duration-multiple-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/zero-duration-multiple-transition.html)

### <a id="reversing"></a>3.1. Faster reversing of interrupted transitions

> <strong data-conversion-semantic="note">Note</strong>
>
> Many common transitions effects involve transitions between two states, such as the transition that occurs when the mouse pointer moves over a user interface element, and then later moves out of that element. With these effects, it is common for a running transition to be interrupted before it completes, and the property reset to the starting value of that transition. An example is a hover effect on an element, where a transition starts when the pointer enters the element, and then the pointer exits the element before the effect has completed. If the outgoing and incoming transitions are executed using their specified durations and timing functions, the resulting effect can be distractingly asymmetric because the second transition takes the full specified time to move a shortened distance. Instead, this specification makes second transition shorter.
>
> <a id="ref-for-transition-reversing-shortening-factor⑧"></a>
>
> <a id="ref-for-transition-reversing-adjusted-start-value⑥"></a>
>
> The mechanism the above rules use to cause this involves the [reversing shortening factor](#transition-reversing-shortening-factor) and the [reversing-adjusted start value](#transition-reversing-adjusted-start-value). In particular, the reversing behavior is present whenever the <a id="ref-for-transition-reversing-shortening-factor⑨"></a>reversing shortening factor is less than 1.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note that these rules do not fully address the problem for transition patterns that involve more than two states.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note that these rules lead to the entire timing function of the new transition being used, rather than jumping into the middle of a timing function, which can create a jarring effect.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > This was one of several possibilities that was considered by the working group. See the [reversing demo](https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/examples/transition-reversing-demo) demonstrating a number of them, leading to a working group resolution made on 2013-06-07 and edits made on 2013-11-11.

## <a id="application"></a>4. Application of transitions

<a id="ref-for-transition-end-time③"></a>

When a property on an element is undergoing a transition (that is, when or after the transition has started and before the [end time](#transition-end-time) of the transition) the transition adds a style called the <a id="current-value"></a>current value to the CSS cascade at the level defined for CSS Transitions in [\[CSS3CASCADE\]](#biblio-css3cascade).

Tests

- [transition-important.html](https://wpt.fyi/results/css/css-transitions/transition-important.html) [(live test)](http://wpt.live/css/css-transitions/transition-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-important.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that this means that computed values resulting from CSS transitions can inherit to descendants just like any other computed values. In the normal case, this means that a transition of an inherited property applies to descendant elements just as an author would expect.

Implementations must add this value to the cascade if and only if that property is not currently undergoing a CSS Animation ([\[CSS3-ANIMATIONS\]](#biblio-css3-animations)) on the same element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that this behavior of transitions not applying to the cascade when an animation on the same element and property is running does not affect whether the transition has started or ended. APIs that expose whether transitions are running (such as [transition events](#transition-events)) still report that a transition is running.

<a id="ref-for-transition-start-time⑥"></a>

<a id="ref-for-current-value④"></a>

<a id="ref-for-transition-start-value⑤"></a>

If the current time is at or before the [start time](#transition-start-time) of the transition (that is, during the delay phase of the transition), the [current value](#current-value) is a specified style that will compute to the [start value](#transition-start-value) of the transition.

<a id="ref-for-transition-start-time⑦"></a>

<a id="ref-for-current-value⑤"></a>

<a id="ref-for-interpolation①"></a>

If the current time is after the [start time](#transition-start-time) of the transition (that is, during the duration phase of the transition), the [current value](#current-value) is a specified style that will compute to the result of [interpolating](https://www.w3.org/TR/css-values-4/#interpolation) the property using the following values:

- <a id="ref-for-transition-start-value⑥"></a>

  <var>V<sub>a</sub></var>: [start value](#transition-start-value) of the transition

- <a id="ref-for-transition-end-value⑧"></a>

  <var>V<sub>b</sub></var>: [end value](#transition-end-value) of the transition

- <var>p</var>: the output of the [timing function](#transition-timing-function-property) for input (current time - start time) / (end time - start time)

<a id="ref-for-interpolation②"></a>

<a id="ref-for-animation-type④"></a>

The specific [interpolation](https://www.w3.org/TR/css-values-4/#interpolation) procedure to be used is defined by the property’s [animation type](https://www.w3.org/TR/web-animations-1/#animation-type).

## <a id="complete"></a>5. Completion of transitions

<a id="ref-for-running-transition①⑧"></a>

<a id="ref-for-style-change-event⑨"></a>

<a id="ref-for-transition-end-time④"></a>

<a id="ref-for-completed-transition①②"></a>

<a id="ref-for-running-transition②⓪"></a>

<a id="ref-for-completed-transition①③"></a>

[Running transitions](#running-transition) <a id="dfn-complete"></a>complete at a time that is equal to or after their end time, but prior to the first [style change event](#style-change-event) whose time is equal to or after their [end time](#transition-end-time). When a transition completes, implementations must move all transitions that complete at that time from the set of <a id="ref-for-running-transition①⑨"></a>running transitions to the set of [completed transitions](#completed-transition) and then fire the [events](#transition-events) for those completions. <strong data-conversion-semantic="note">Note:</strong> (Note that doing otherwise, that is, firing some of the events before doing all of the moving from [running transitions](#running-transition) to [completed transitions](#completed-transition), could allow a style change event to happen without the necessary transitions completing, since firing the event could cause a style change event, if an event handler requests up-to-date computed style or layout data.)

## <a id="transition-events"></a>6. <a id="transition-events-"></a>Transition Events

<a id="ref-for-concept-event-dispatch"></a>

The creation, beginning, completion, and cancellation of CSS transitions generate corresponding DOM Events. An event is [dispatched](https://html.spec.whatwg.org/multipage/infrastructure.html#concept-event-dispatch) to the element for each property that undergoes a transition on that element. This allows a content developer to perform actions that synchronize with changes to transitions.

Each event provides the name of the property the transition is associated with as well as the duration of the transition.

Tests

- [transition-events-with-document-change.html](https://wpt.fyi/results/css/css-transitions/transition-events-with-document-change.html) [(live test)](http://wpt.live/css/css-transitions/transition-events-with-document-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-events-with-document-change.html)

<a id="ref-for-transitionevent"></a>

### <a id="interface-transitionevent"></a>6.1. Interface <code><a href="#transitionevent">TransitionEvent</a></code>

<a id="ref-for-transitionevent①"></a>

The <code><a href="#transitionevent">TransitionEvent</a></code> interface provides specific contextual information associated with transitions.

#### <a id="interface-transitionevent-idl"></a>6.1.1. IDL Definition

<a id="ref-for-Exposed"></a>

<a id="transitionevent"></a>

<a id="ref-for-event"></a>

<a id="ref-for-dom-transitionevent-transitionevent"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-transitionevent-transitionevent-type-transitioneventinitdict-type"></a>

<a id="ref-for-dictdef-transitioneventinit"></a>

<a id="dom-transitionevent-transitionevent-type-transitioneventinitdict-transitioneventinitdict"></a>

<a id="ref-for-cssomstring①"></a>

<a id="ref-for-Events-TransitionEvent-propertyName"></a>

<a id="ref-for-idl-double"></a>

<a id="ref-for-Events-TransitionEvent-elapsedTime"></a>

<a id="ref-for-cssomstring②"></a>

<a id="ref-for-Events-TransitionEvent-pseudoElement"></a>

<a id="dictdef-transitioneventinit"></a>

<a id="ref-for-dictdef-eventinit"></a>

<a id="ref-for-cssomstring③"></a>

<a id="dom-transitioneventinit-propertyname"></a>

<a id="ref-for-idl-double①"></a>

<a id="dom-transitioneventinit-elapsedtime"></a>

<a id="ref-for-cssomstring④"></a>

<a id="dom-transitioneventinit-pseudoelement"></a>

```text
[Exposed=Window]
interface TransitionEvent : Event {
  constructor(CSSOMString type, optional TransitionEventInit transitionEventInitDict = {});
  readonly attribute CSSOMString propertyName;
  readonly attribute double elapsedTime;
  readonly attribute CSSOMString pseudoElement;
};

dictionary TransitionEventInit : EventInit {
  CSSOMString propertyName = "";
  double elapsedTime = 0.0;
  CSSOMString pseudoElement = "";
};
```
Tests

- [transitionevent-interface.html](https://wpt.fyi/results/css/css-transitions/transitionevent-interface.html) [(live test)](http://wpt.live/css/css-transitions/transitionevent-interface.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transitionevent-interface.html)

#### <a id="interface-transitionevent-attributes"></a>6.1.2. Attributes

<a id="ref-for-cssomstring⑤"></a>

<a id="Events-TransitionEvent-propertyName"></a>

<code><dfn><code>propertyName</code></dfn>, <span> of type <a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a>, readonly</span></code>

The name of the CSS property associated with the transition.

<a id="ref-for-propdef-transition-property①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is always the name of a longhand property. See [transition-property](#propdef-transition-property) for how specifying shorthand properties causes transitions on longhands.

<a id="ref-for-idl-double②"></a>

<a id="Events-TransitionEvent-elapsedTime"></a>

<code><dfn><code>elapsedTime</code></dfn>, <span> of type <a href="https://webidl.spec.whatwg.org/#idl-double">double</a>, readonly</span></code>

The amount of time the transition has been running, in seconds, when this event fired not including any time spent in the delay phase. The calculation for of this member is defined along with each event type.

<a id="ref-for-cssomstring⑥"></a>

<a id="Events-TransitionEvent-pseudoElement"></a>

<code><dfn><code>pseudoElement</code></dfn>, <span> of type <a href="https://www.w3.org/TR/cssom-1/#cssomstring">CSSOMString</a>, readonly</span></code>

The name (beginning with two colons) of the CSS pseudo-element on which the transition occurred (in which case the target of the event is that pseudo-element’s corresponding element), or the empty string if the transition occurred on an element (which means the target of the event is that element).

<a id="TransitionEvent-constructor"></a>

<a id="dom-transitionevent-transitionevent"></a>

<a id="ref-for-constructing-events"></a>

`TransitionEvent(type, transitionEventInitDict)` is an [event constructor](http://w3c.github.io/dom/#constructing-events).

### <a id="event-transitionevent"></a>6.2. Types of `TransitionEvent`

The different types of transition events that can occur are:

<a id="transitionrun"></a>`transitionrun`  
<a id="ref-for-running-transition②①"></a>

<a id="ref-for-transitionrun"></a>

The <code><a href="#transitionrun">transitionrun</a></code> event occurs when a transition is created (i.e., when it is added to the set of [running transitions](#running-transition)).

<a id="ref-for-propdef-transition-delay①⓪"></a>

<a id="ref-for-Events-TransitionEvent-elapsedTime①"></a>

<a id="ref-for-propdef-transition-duration⑨"></a>

<a id="ref-for-propdef-transition-delay①①"></a>

<a id="ref-for-propdef-transition-duration①⓪"></a>

A negative [transition-delay](#propdef-transition-delay) will cause the event to fire with an <code><a href="#Events-TransitionEvent-elapsedTime">elapsedTime</a></code> equal to the absolute value of the delay capped to the [transition-duration](#propdef-transition-duration) of the animation. That is, the elapsed time is equal to <code>min(max(-<a href="#propdef-transition-delay">transition-delay</a>, 0), <a href="#propdef-transition-duration">transition-duration</a>)</code>.

- Bubbles: Yes
- Cancelable: No
- Context Info: propertyName, elapsedTime, pseudoElement

<a id="transitionstart"></a>`transitionstart`  
<a id="ref-for-transitionstart"></a>

The <code><a href="#transitionstart">transitionstart</a></code> event occurs when a transition’s delay phase ends.

<a id="ref-for-Events-TransitionEvent-elapsedTime②"></a>

<a id="ref-for-transitionstart①"></a>

<a id="ref-for-transitionrun①"></a>

The value of <code><a href="#Events-TransitionEvent-elapsedTime">elapsedTime</a></code> for <code><a href="#transitionstart">transitionstart</a></code> events is the same as the value used for <code><a href="#transitionrun">transitionrun</a></code> events.

- Bubbles: Yes
- Cancelable: No
- Context Info: propertyName, elapsedTime, pseudoElement

<a id="transitionend"></a>`transitionend`  
<a id="ref-for-propdef-transition-property①①"></a>

<a id="ref-for-transitionend"></a>

The <code><a href="#transitionend">transitionend</a></code> event occurs at the completion of the transition. In the case where a transition is removed before completion, such as if the [transition-property](#propdef-transition-property) is removed, then the event will not fire.

<a id="ref-for-Events-TransitionEvent-elapsedTime③"></a>

<a id="ref-for-propdef-transition-duration①①"></a>

The value of <code><a href="#Events-TransitionEvent-elapsedTime">elapsedTime</a></code> for this event is equal to the value of [transition-duration](#propdef-transition-duration).

- Bubbles: Yes
- Cancelable: No
- Context Info: propertyName, elapsedTime, pseudoElement

Tests

- [events-001.html](https://wpt.fyi/results/css/css-transitions/events-001.html) [(live test)](http://wpt.live/css/css-transitions/events-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-001.html)
- [events-002.html](https://wpt.fyi/results/css/css-transitions/events-002.html) [(live test)](http://wpt.live/css/css-transitions/events-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-002.html)
- [events-003.html](https://wpt.fyi/results/css/css-transitions/events-003.html) [(live test)](http://wpt.live/css/css-transitions/events-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-003.html)
- [events-004.html](https://wpt.fyi/results/css/css-transitions/events-004.html) [(live test)](http://wpt.live/css/css-transitions/events-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-004.html)
- [events-005.html](https://wpt.fyi/results/css/css-transitions/events-005.html) [(live test)](http://wpt.live/css/css-transitions/events-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-005.html)
- [events-006.html](https://wpt.fyi/results/css/css-transitions/events-006.html) [(live test)](http://wpt.live/css/css-transitions/events-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-006.html)
- [events-007.html](https://wpt.fyi/results/css/css-transitions/events-007.html) [(live test)](http://wpt.live/css/css-transitions/events-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/events-007.html)

<a id="transitioncancel"></a>`transitioncancel`  
<a id="ref-for-transition-cancel⑥"></a>

<a id="ref-for-transitioncancel"></a>

The <code><a href="#transitioncancel">transitioncancel</a></code> event occurs when a transition is [canceled](#transition-cancel).

<a id="ref-for-Events-TransitionEvent-elapsedTime④"></a>

<a id="ref-for-transitioncancel①"></a>

<a id="ref-for-propdef-transition-delay①②"></a>

<a id="ref-for-dom-animationevent-elapsedtime"></a>

The <code><a href="#Events-TransitionEvent-elapsedTime">elapsedTime</a></code> for <code><a href="#transitioncancel">transitioncancel</a></code> events is the number of seconds from the end of the transition’s delay to the moment when the transition was canceled. If the transition had a negative [transition-delay](#propdef-transition-delay), the beginning of the transition is the moment equal to the absolute value of <a id="ref-for-propdef-transition-delay①③"></a>transition-delay seconds <em>prior</em> to when the transition was actually triggered. Alternatively, if the transition had a positive <a id="ref-for-propdef-transition-delay①④"></a>transition-delay and the event is fired before the transition’s delay has expired, the <code><a href="https://www.w3.org/TR/css-animations-1/#dom-animationevent-elapsedtime">elapsedTime</a></code> will be zero.

- Bubbles: Yes
- Cancelable: No
- Context Info: propertyName, elapsedTime, pseudoElement

Tests

- [transitioncancel-001.html](https://wpt.fyi/results/css/css-transitions/transitioncancel-001.html) [(live test)](http://wpt.live/css/css-transitions/transitioncancel-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transitioncancel-001.html)
- [transitioncancel-002.html](https://wpt.fyi/results/css/css-transitions/transitioncancel-002.html) [(live test)](http://wpt.live/css/css-transitions/transitioncancel-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transitioncancel-002.html)

### <a id="event-handlers-on-elements-document-objects-and-window-objects"></a>6.3. Event handlers on elements, `Document` objects, and `Window` objects

<a id="ref-for-event-handlers"></a>

<a id="ref-for-event-handler-event-type"></a>

<a id="ref-for-html-elements"></a>

<a id="ref-for-event-handler-content-attributes"></a>

<a id="ref-for-event-handler-idl-attributes"></a>

<a id="ref-for-document"></a>

<a id="ref-for-window"></a>

The following are the [event handlers](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers) (and their corresponding [event handler event types](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type)) that must be supported by all [HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements), as both [event handler content attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-content-attributes) and [event handler IDL attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes); and that must be supported by all <code><a href="https://html.spec.whatwg.org/#document">Document</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> objects, as <a id="ref-for-event-handler-idl-attributes①"></a>event handler IDL attributes:

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-event-handlers①"></a>

[Event handler](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers)

<strong>Column 2 (header cell):</strong>

<a id="ref-for-event-handler-event-type①"></a>

[Event handler event type](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type)

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-dom-globaleventhandlers-ontransitionrun"></a>

<code><a href="#dom-globaleventhandlers-ontransitionrun">ontransitionrun</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transitionrun②"></a>

[transitionrun](#transitionrun)

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-dom-globaleventhandlers-ontransitionstart"></a>

<code><a href="#dom-globaleventhandlers-ontransitionstart">ontransitionstart</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transitionstart②"></a>

[transitionstart](#transitionstart)

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-dom-globaleventhandlers-ontransitionend"></a>

<code><a href="#dom-globaleventhandlers-ontransitionend">ontransitionend</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transitionend①"></a>

[transitionend](#transitionend)

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-dom-globaleventhandlers-ontransitioncancel"></a>

<code><a href="#dom-globaleventhandlers-ontransitioncancel">ontransitioncancel</a></code>

<strong>Column 2 (data cell):</strong>

<a id="ref-for-transitioncancel②"></a>

[transitioncancel](#transitioncancel)

## <a id="interface-dom"></a>7. DOM Interfaces

<a id="ref-for-globaleventhandlers"></a>

<a id="ref-for-event-handler-idl-attributes②"></a>

This specification extends the <code><a href="https://html.spec.whatwg.org/multipage/webappapis.html#globaleventhandlers">GlobalEventHandlers</a></code> interface mixin from HTML to add [event handler IDL attributes](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes) for [transition events](#transition-events) as defined in [§ 6.3 Event handlers on elements, Document objects, and Window objects](#event-handlers-on-elements-document-objects-and-window-objects).

### <a id="interface-globaleventhandlers-idl"></a>7.1. IDL Definition

<a id="ref-for-globaleventhandlers①"></a>

<a id="ref-for-eventhandler"></a>

<a id="dom-globaleventhandlers-ontransitionrun"></a>

<a id="ref-for-eventhandler①"></a>

<a id="dom-globaleventhandlers-ontransitionstart"></a>

<a id="ref-for-eventhandler②"></a>

<a id="dom-globaleventhandlers-ontransitionend"></a>

<a id="ref-for-eventhandler③"></a>

<a id="dom-globaleventhandlers-ontransitioncancel"></a>

```text
partial interface mixin GlobalEventHandlers {
  attribute EventHandler ontransitionrun;
  attribute EventHandler ontransitionstart;
  attribute EventHandler ontransitionend;
  attribute EventHandler ontransitioncancel;
};
```
## <a id="security"></a>8. Security Considerations

<em>This section is not normative.</em>

The security implications of this specification are limited because it doesn’t allow Web content to do things that it could not do before. Rather, it allows things that could previously be done with script to be done declaratively, and it ways that implementations can optimize (for frame rate and CPU usage).

<a id="ref-for-propdef-transform"></a>

<a id="ref-for-propdef-opacity③"></a>

One of the major categories of optimizations that implementations can make is implementing animation of certain high-value properties (such as [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) and [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)) run on a browser’s compositor thread or process without updating style or layout on the main Web content thread unless up-to-date style data are needed. This optimization often requires allocations of graphics memory to display the contents of the element being animated. Implementations should take care to ensure that Web content cannot trigger unsafe out-of-memory handling by using large numbers of animations or animations on elements covering large areas (where large may be defined in terms of pre-transform or post-transform size).

## <a id="privacy"></a>9. Privacy Considerations

<em>This section is not normative.</em>

As for security, the privacy considerations of this specification are limited because it does not allow Web content to do things that it could not do before.

This specification may provide additional mechanisms that help to determine characteristics of the user’s hardware or software. However, ability to determine performance characteristics of the user’s hardware or software is common to many Web technologies, and this specification does not introduce new capabilities.

As described in [§ 10 Accessibility Considerations](#accessibility), implementations may provide mitigations to help users with disabilities. These mitigations are likely to be detectable by Web content, which means that users who would benefit from these mitigations may face a tradeoff between keeping their disability private from the Web content or benefiting from the mitigation.

## <a id="accessibility"></a>10. Accessibility Considerations

<em>This section is not normative.</em>

### <a id="accessibility-motion"></a>10.1. Motion

This specification provides declarative mechanisms for animations that previously needed to be done using script. Providing a declarative mechanism has multiple effects: it makes such animations easier to make and thus likely to be more common, but it also makes it easier for user agents to modify those animations if such modifications are needed to meet a user’s accessibility needs.

Thus, users who are sensitive to movement, or who require additional time to read or understand content, may benefit from user agent features that allow animations to be disabled or slowed down. (But see [§ 9 Privacy Considerations](#privacy) for information on the privacy implications of such mitigations.)

User agent implementors should be aware that Web content may depend on the firing of [transition events](#transition-events), so implementations of such mitigations may wish to fire transition events even if the transitions were not run as continuous animations. However, it is probably poor practice for Web content to depend on such events to function correctly.

### <a id="accessibility-cascade"></a>10.2. Cascade

<a id="ref-for-cascade"></a>

The CSS [cascade](https://www.w3.org/TR/css-cascade-6/#cascade) is a general mechanism in CSS that allows user needs to interact with author styles. This specification interacts with the cascade, but since it only allows animation between values that result from the existing cascade rules, it does not interfere with the user’s ability to force CSS properties to have particular values.

The cascade also allows users to disable transitions entirely by overriding the transition properties.

## <a id="changes"></a>11. Changes

### <a id="changes-2018"></a>11.1. Changes since Working Draft of 11 October 2018

The following are the substantive changes made since the [Working Draft dated 11 October 2018](https://www.w3.org/TR/2018/WD-css-transitions-1-20181011/):

- <a id="ref-for-globaleventhandlers②"></a>

  Clarified that <code><a href="https://html.spec.whatwg.org/multipage/webappapis.html#globaleventhandlers">GlobalEventHandlers</a></code> is a mixin.

- <a id="ref-for-typedef-easing-function②"></a>

  Replaced defunct `<timing-function>` by [\<easing-function\>](https://www.w3.org/TR/css-easing-2/#typedef-easing-function).

- <a id="ref-for-dom-transitionevent-transitionevent-type-transitioneventinitdict-transitioneventinitdict"></a>

  Added default value to <code><a href="#dom-transitionevent-transitionevent-type-transitioneventinitdict-transitioneventinitdict">transitionEventInitDict</a></code> dictionary.

- <a id="ref-for-single-transition-property④"></a>

  Removed trailing semicolon from [\<single-transition-property\>](#single-transition-property) syntax.

- <a id="ref-for-globaleventhandlers③"></a>

  Associated event definitions with their <code><a href="https://html.spec.whatwg.org/multipage/webappapis.html#globaleventhandlers">GlobalEventHandlers</a></code> container.

- Added range definition notations to property values.

- Added Web Platform Tests coverage.

- Minor editorial fixes and improvements.

For more details on these changes, see the version control [change log](https://github.com/w3c/csswg-drafts/commits/main/css-transitions-1/Overview.bs).

### <a id="changes-earlier"></a>11.2. Earlier changes

For changes in earlier working drafts:

1.  See the [changes section in the 11 October 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-transitions-1-20181011/#changes).

2.  See the [changes section in the 30 November 2017 Working Draft](https://www.w3.org/TR/2017/WD-css-transitions-1-20171130/#changes).

3.  See the [changes section in the 19 November 2013 Working Draft](https://www.w3.org/TR/2013/WD-css3-transitions-20131119/#changes).

4.  See the [the ChangeLog](https://www.w3.org/TR/2013/WD-css3-transitions-20130212/ChangeLog) for changes in previous working drafts.

5.  For more details on these changes, see the version control change logs, which are split in many parts because of file renaming:

    - [change log since 2017 October 12](https://github.com/w3c/csswg-drafts/commits/main/css-transitions-1/Overview.bs),

## <a id="acknowledgments"></a>12. Acknowledgments

Thanks especially to the feedback from Tab Atkins, Carine Bournez, Aryeh Gregor, Vincent Hardy, Anne van Kesteren, Cameron McCormack, Alex Mogilevsky, Jasper St. Pierre, Estelle Weyl, and all the rest of the [www-style](https://lists.w3.org/Archives/Public/www-style/) community.

Tests

Tests related to later levels of this specification

- [allow-discrete-auto-inset.html](https://wpt.fyi/results/css/css-transitions/allow-discrete-auto-inset.html) [(live test)](http://wpt.live/css/css-transitions/allow-discrete-auto-inset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/allow-discrete-auto-inset.html)
- [custom-property-and-allow-discrete.html](https://wpt.fyi/results/css/css-transitions/custom-property-and-allow-discrete.html) [(live test)](http://wpt.live/css/css-transitions/custom-property-and-allow-discrete.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/custom-property-and-allow-discrete.html)
- [display-none-no-animations.html](https://wpt.fyi/results/css/css-transitions/display-none-no-animations.html) [(live test)](http://wpt.live/css/css-transitions/display-none-no-animations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/display-none-no-animations.html)
- [idlharness-2.html](https://wpt.fyi/results/css/css-transitions/idlharness-2.html) [(live test)](http://wpt.live/css/css-transitions/idlharness-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/idlharness-2.html)
- [inert-while-transitioning-to-display-none.html](https://wpt.fyi/results/css/css-transitions/inert-while-transitioning-to-display-none.html) [(live test)](http://wpt.live/css/css-transitions/inert-while-transitioning-to-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/inert-while-transitioning-to-display-none.html)
- [starting-style-parsing.html](https://wpt.fyi/results/css/css-transitions/parsing/starting-style-parsing.html) [(live test)](http://wpt.live/css/css-transitions/parsing/starting-style-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/starting-style-parsing.html)
- [transition-behavior.html](https://wpt.fyi/results/css/css-transitions/parsing/transition-behavior.html) [(live test)](http://wpt.live/css/css-transitions/parsing/transition-behavior.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/parsing/transition-behavior.html)
- [starting-style-adjustment.html](https://wpt.fyi/results/css/css-transitions/starting-style-adjustment.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-adjustment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-adjustment.html)
- [starting-style-cascade.html](https://wpt.fyi/results/css/css-transitions/starting-style-cascade.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-cascade.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-cascade.html)
- [starting-style-first-letter-crash.html](https://wpt.fyi/results/css/css-transitions/starting-style-first-letter-crash.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-first-letter-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-first-letter-crash.html)
- [starting-style-name-defining-rules.html](https://wpt.fyi/results/css/css-transitions/starting-style-name-defining-rules.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-name-defining-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-name-defining-rules.html)
- [starting-style-rule-basic.html](https://wpt.fyi/results/css/css-transitions/starting-style-rule-basic.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-rule-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-rule-basic.html)
- [starting-style-rule-none.html](https://wpt.fyi/results/css/css-transitions/starting-style-rule-none.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-rule-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-rule-none.html)
- [starting-style-rule-pseudo-elements.html](https://wpt.fyi/results/css/css-transitions/starting-style-rule-pseudo-elements.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-rule-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-rule-pseudo-elements.html)
- [starting-style-size-container.html](https://wpt.fyi/results/css/css-transitions/starting-style-size-container.html) [(live test)](http://wpt.live/css/css-transitions/starting-style-size-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/starting-style-size-container.html)
- [transition-behavior.html](https://wpt.fyi/results/css/css-transitions/transition-behavior.html) [(live test)](http://wpt.live/css/css-transitions/transition-behavior.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-behavior.html)
- [transition-behavior-events.html](https://wpt.fyi/results/css/css-transitions/transition-behavior-events.html) [(live test)](http://wpt.live/css/css-transitions/transition-behavior-events.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/transition-behavior-events.html)

------------------------------------------------------------------------

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

- [after-change style](#after-change-style), in § 3
- [all](#valdef-transition-property-all), in § 2.1
- [before-change style](#before-change-style), in § 3
- [cancel](#transition-cancel), in § 3
- [combined duration](#transition-combined-duration), in § 3
- [complete](#dfn-complete), in § 5
- [completed transition](#completed-transition), in § 3
- [constructor(type)](#dom-transitionevent-transitionevent), in § 6.1.2
- [constructor(type, transitionEventInitDict)](#dom-transitionevent-transitionevent), in § 6.1.2
- [current value](#current-value), in § 4
- elapsedTime
  - [attribute for TransitionEvent](#Events-TransitionEvent-elapsedTime), in § 6.1.2
  - [dict-member for TransitionEventInit](#dom-transitioneventinit-elapsedtime), in § 6.1.1
- [end time](#transition-end-time), in § 3
- [end value](#transition-end-value), in § 3
- [matching transition delay](#matching-transition-delay), in § 3
- [matching transition duration](#matching-transition-duration), in § 3
- [matching transition-property value](#matching-transition-property-value), in § 3
- [matching transition timing function](#matching-transition-timing-function), in § 3
- [none](#valdef-transition-property-none), in § 2.1
- [ontransitioncancel](#dom-globaleventhandlers-ontransitioncancel), in § 7.1
- [ontransitionend](#dom-globaleventhandlers-ontransitionend), in § 7.1
- [ontransitionrun](#dom-globaleventhandlers-ontransitionrun), in § 7.1
- [ontransitionstart](#dom-globaleventhandlers-ontransitionstart), in § 7.1
- propertyName
  - [attribute for TransitionEvent](#Events-TransitionEvent-propertyName), in § 6.1.2
  - [dict-member for TransitionEventInit](#dom-transitioneventinit-propertyname), in § 6.1.1
- pseudoElement
  - [attribute for TransitionEvent](#Events-TransitionEvent-pseudoElement), in § 6.1.2
  - [dict-member for TransitionEventInit](#dom-transitioneventinit-pseudoelement), in § 6.1.1
- [reversing-adjusted start value](#transition-reversing-adjusted-start-value), in § 3
- [reversing shortening factor](#transition-reversing-shortening-factor), in § 3
- [running transition](#running-transition), in § 3
- [\<single-transition\>](#single-transition), in § 2.5
- [\<single-transition-property\>](#single-transition-property), in § 2.1
- [start time](#transition-start-time), in § 3
- [start value](#transition-start-value), in § 3
- [style change event](#style-change-event), in § 3
- [transition](#propdef-transition), in § 2.5
- [transitionable](#transitionable), in § 3
- [transitioncancel](#transitioncancel), in § 6.2
- [transition-delay](#propdef-transition-delay), in § 2.4
- [transition-duration](#propdef-transition-duration), in § 2.2
- [transitionend](#transitionend), in § 6.2
- [TransitionEvent](#transitionevent), in § 6.1.1
- [TransitionEventInit](#dictdef-transitioneventinit), in § 6.1.1
- [TransitionEvent(type)](#dom-transitionevent-transitionevent), in § 6.1.2
- [TransitionEvent(type, transitionEventInitDict)](#dom-transitionevent-transitionevent), in § 6.1.2
- [transition-property](#propdef-transition-property), in § 2.1
- [transitionrun](#transitionrun), in § 6.2
- [Transitions](#transitions), in § 1.1
- [transitionstart](#transitionstart), in § 6.2
- [transition-timing-function](#propdef-transition-timing-function), in § 2.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[\] defines the following terms:
  - <a id="74f00209"></a>event constructor
  - <a id="5074dc0c"></a>inset
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="5ced56d0"></a>background-image
  - <a id="c48eaa20"></a>box-shadow
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="d3b48763"></a>all
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="762bad34"></a>initial
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="aa433d97"></a>cascade
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="a976c737"></a>blue
  - <a id="45dae945"></a>green
  - <a id="3b7558dc"></a>opacity
- \[CSS-EASING-2\] defines the following terms:
  - <a id="eef9f659"></a>\<easing-function\>
  - <a id="1dd77088"></a>input progress value
  - <a id="035a0bad"></a>output progress value
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="e7c6bf78"></a>transform
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="bb5f9e40"></a>\<time\>
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="c2e11f7b"></a>interpolate
  - <a id="a0feb601"></a>interpolation
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS2\] defines the following terms:
  - <a id="69d1180f"></a>background-color
  - <a id="795e7148"></a>left
  - <a id="97fa58af"></a>top
  - <a id="49c5c029"></a>width
- \[CSS3-ANIMATIONS\] defines the following terms:
  - <a id="7ade1e36"></a>elapsedTime
- \[CSSOM-1\] defines the following terms:
  - <a id="9d357000"></a>CSSOMString
- \[DOM\] defines the following terms:
  - <a id="129bdae8"></a>Event
  - <a id="44a7708c"></a>EventInit
- \[HTML\] defines the following terms:
  - <a id="07cd5a09"></a>Document
  - <a id="f0951476"></a>EventHandler
  - <a id="762ce945"></a>GlobalEventHandlers
  - <a id="5d7209e9"></a>Window
  - <a id="f87fb171"></a>dispatch
  - <a id="b22ae7ee"></a>event handler content attributes
  - <a id="9d386f55"></a>event handler event type
  - <a id="d4689c67"></a>event handler IDL attributes
  - <a id="e9189aba"></a>event handlers
  - <a id="49a64d88"></a>HTML elements
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="d376a2a1"></a>animation type
  - <a id="19f221a3"></a>combining shadow lists
  - <a id="2cec4673"></a>discrete
  - <a id="ddbb25e9"></a>not animatable
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="8c800cdf"></a>double

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 6 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-easing-2"></a>\[CSS-EASING-2\]  
[CSS Easing Functions Level 2](https://www.w3.org/TR/css-easing-2/). 29 August 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-2&#x2F;](https://www.w3.org/TR/css-easing-2/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-animations"></a>\[CSS3-ANIMATIONS\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

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

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-smil-animation"></a>\[SMIL-ANIMATION\]  
Patrick Schmitz; Aaron Cohen. [SMIL Animation](https://www.w3.org/TR/smil-animation/). 4 September 2001. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;smil-animation&#x2F;](https://www.w3.org/TR/smil-animation/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-wcag20"></a>\[WCAG20\]  
Ben Caldwell; et al. [Web Content Accessibility Guidelines (WCAG) 2.0](https://www.w3.org/TR/WCAG20/). 11 December 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG20&#x2F;](https://www.w3.org/TR/WCAG20/)

## <a id="property-index"></a>Property Index

<strong>Table 7 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-transition④"></a>

[transition](#propdef-transition)

<strong>Column 2 (data cell):</strong>

\<single-transition\>#

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

<a id="ref-for-propdef-transition-delay①⑤"></a>

[transition-delay](#propdef-transition-delay)

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

<a id="ref-for-propdef-transition-duration①②"></a>

[transition-duration](#propdef-transition-duration)

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

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-transition-property①②"></a>

[transition-property](#propdef-transition-property)

<strong>Column 2 (data cell):</strong>

none \| \<single-transition-property\>#

<strong>Column 3 (data cell):</strong>

all

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

the keyword none else a list of identifiers

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-transition-timing-function⑦"></a>

[transition-timing-function](#propdef-transition-timing-function)

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

as specified

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface TransitionEvent : Event {
  constructor(CSSOMString type, optional TransitionEventInit transitionEventInitDict = {});
  readonly attribute CSSOMString propertyName;
  readonly attribute double elapsedTime;
  readonly attribute CSSOMString pseudoElement;
};

dictionary TransitionEventInit : EventInit {
  CSSOMString propertyName = "";
  double elapsedTime = 0.0;
  CSSOMString pseudoElement = "";
};

partial interface mixin GlobalEventHandlers {
  attribute EventHandler ontransitionrun;
  attribute EventHandler ontransitionstart;
  attribute EventHandler ontransitionend;
  attribute EventHandler ontransitioncancel;
};

```