Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Speech Module Level 1](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/).

Original copyright notice: Copyright © 2023 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Speech Module Level 1

Source snapshot: https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/

Snapshot SHA-256: bb7b59c925640f089f19196a9d72f0c381bdcb10cec8ec411ba99cb9bc9b5c9a

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 17 source tables are presented as readable Markdown tables or explicit labeled layouts: 17 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Speech Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2023 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

The Speech module defines aural CSS properties that enable authors to declaratively control the rendering of documents via speech synthesis, and using optional audio cues. Note that this standard was developed in cooperation with the [Voice Browser Activity](https://www.w3.org/Voice/).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-speech” in the title, like this: “\[css-speech\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-speech%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-voice-stress"></a>

  <a id="ref-for-propdef-voice-range"></a>

  <a id="ref-for-propdef-voice-pitch"></a>

  <a id="ref-for-propdef-voice-duration"></a>

  <a id="ref-for-propdef-voice-balance"></a>

  [voice-balance](#propdef-voice-balance), [voice-duration](#propdef-voice-duration), [voice-pitch](#propdef-voice-pitch), [voice-range](#propdef-voice-range), and [voice-stress](#propdef-voice-stress)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction, design goals

<em>This section is non-normative.</em>

The aural presentation of information is commonly used by people who are blind, visually-impaired, or otherwise print-disabled. For instance, “screen readers” allow users to interact with visual interfaces that would otherwise be inaccessible to them. There are also circumstances in which <em>listening</em> to content (as opposed to <em>reading</em>) is preferred, or sometimes even required, irrespective of a person’s physical ability to access information. For instance: playing an e-book whilst driving a vehicle, learning how to manipulate industrial and medical devices, interacting with home entertainment systems, teaching young children how to read.

The CSS properties defined in this Speech module enable authors to declaratively control the presentation of a document in the aural dimension. The aural rendering of a document combines speech synthesis (also known as “TTS”, the acronym for “Text to Speech”) and auditory icons (which are referred-to as “audio cues” in this specification). The CSS Speech properties provide the ability to control speech pitch and rate, sound levels, TTS voices, etc. These stylesheet properties can be used together with visual properties (mixed media), or as a complete aural alternative to a visual presentation.

## <a id="background"></a>2.  Background information, CSS 2.1

<em>This section is non-normative.</em>

<a id="ref-for-valdef-media-aural"></a>

<a id="ref-for-valdef-media-speech"></a>

The CSS Speech module is a re-work of the informative [CSS2.1 Aural appendix](https://www.w3.org/TR/CSS2/aural.html), within which the [aural](https://www.w3.org/TR/mediaqueries-5/#valdef-media-aural) media type was described, but also deprecated (in favor of the [speech](https://drafts.csswg.org/css2/#valdef-media-speech) media type, which has now also been deprecated). Although the [\[CSS2\]](#biblio-css2) specification reserved the <a id="ref-for-valdef-media-speech①"></a>speech media type, it didn’t actually define the corresponding properties. The Speech module describes the CSS properties that apply to speech output, and defines a new “box” model specifically for the aural dimension.

<a id="ref-for-valdef-media-all"></a>

<a id="ref-for-valdef-media-screen"></a>

Content creators can include CSS properties for user agents with text to speech synthesis capabilities for any media type - though generally, they will only make sense for [all](https://drafts.csswg.org/css2/#valdef-media-all) and [screen](https://drafts.csswg.org/css2/#valdef-media-screen). These styles are simply ignored by user agents that do not support the Speech module.

## <a id="ssml-rel"></a>3.  Relationship with SSML

<em>This section is non-normative.</em>

Some of the features in this specification are conceptually similar to functionality described in the Speech Synthesis Markup Language (SSML) Version 1.1 [\[SSML\]](#biblio-ssml). However, the specificities of the CSS model mean that compatibility with SSML in terms of syntax and/or semantics is only partially achievable. The definition of each property in the Speech module includes informative statements, wherever necessary, to clarify their relationship with similar functionality from SSML.

### <a id="values"></a>3.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="example"></a>4.  Example

<a id="ref-for-the-span-element"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ee54fc10"></a> This example shows how authors can tell the speech synthesizer to speak HTML headings with a voice called "paul", using "moderate" emphasis (which is more than normal) and how to insert an audio cue (pre-recorded audio clip located at the given URL) before the start of TTS rendering for each heading. In a stereo-capable sound system, paragraphs marked with the CSS class `heidi` are rendered on the left audio channel (and with a female voice, etc.), whilst the class `peter` corresponds to the right channel (and to a male voice, etc.). The volume level of text spans marked with the class `special` is lower than normal, and a prosodic boundary is created by introducing a strong pause after it is spoken (note how the <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-span-element">span</a></code> inherits the voice-family from its parent paragraph).
>
> ```text
> h1, h2, h3, h4, h5, h6 {
>   voice-family: paul;
>   voice-stress: moderate;
>   cue-before: url(../audio/ping.wav);
>   voice-volume: medium 6dB;
> }
> p.heidi {
>   voice-family: female;
>   voice-balance: left;
>   voice-pitch: high;
>   voice-volume: -6dB;
> }
> p.peter {
>   voice-family: male;
>   voice-balance: right;
>   voice-rate: fast;
> }
> span.special {
>   voice-volume: soft;
>   pause-after: strong;
> }
> 
> ...
> 
> <h1>I am Paul, and I speak headings.</h1>
> <p class="heidi">Hello, I am Heidi.</p>
> <p class="peter">
>   <span class="special">Can you hear me ?</span>
>   I am Peter.
> </p>
> ```
## <a id="aural-model"></a>5.  The aural formatting model

<a id="ref-for-propdef-rest"></a>

<a id="ref-for-propdef-cue"></a>

<a id="ref-for-propdef-pause"></a>

<a id="ref-for-propdef-padding"></a>

<a id="ref-for-propdef-border"></a>

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-selectordef-before"></a>

<a id="ref-for-selectordef-after"></a>

The CSS formatting model for aural media is based on a sequence of sounds and silences that occur within a nested context similar to the [visual box model](https://www.w3.org/TR/css-box-3/#box-model), which we name the <a id="aural-box-model"></a>aural “box” model. The aural “canvas” consists of a two-channel (stereo) space and of a temporal dimension, within which synthetic speech and audio cues coexist. The selected element is surrounded by [rest](#propdef-rest), [cue](#propdef-cue) and [pause](#propdef-pause) properties (from the innermost to the outermost position). These can be seen as aural equivalents to [padding](https://www.w3.org/TR/css-box-4/#propdef-padding), [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) and [margin](https://www.w3.org/TR/css-box-4/#propdef-margin), respectively. When used, the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements [\[CSS2\]](#biblio-css2) get inserted between the element’s contents and the <a id="ref-for-propdef-rest①"></a>rest.

The following diagram illustrates the equivalence between properties of the visual and aural box models, applied to the selected \<element\>:

<a id="aural-box"></a>

![The aural 'box' model, illustrated by a diagram: the selected element is positioned in the center, on its left side are (from innermost to outermost) rest-before, cue-before, pause-before, on its right side are (from innermost to outermost) rest-after, cue-after, pause-after, where rest is conceptually similar to padding, cue is similar to border, pause is similar to margin.](https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/images/aural-box.png "The aural 'box' model, illustrated by a diagram: the selected element is positioned in the center, on its left side are (from innermost to outermost) rest-before, cue-before, pause-before, on its right side are (from innermost to outermost) rest-after, cue-after, pause-after, where rest is conceptually similar to padding, cue is similar to border, pause is similar to margin.")

## <a id="mixing-props"></a>6.  Mixing properties

<a id="ref-for-propdef-voice-volume"></a>

### <a id="mixing-props-voice-volume"></a>6.1.  The [voice-volume](#propdef-voice-volume) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-volume"></a>voice-volume                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-voice-volume-decibel"></a><a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>silent [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[\[x-soft <a id="ref-for-comb-one①"></a>\| soft <a id="ref-for-comb-one②"></a>\| medium <a id="ref-for-comb-one③"></a>\| loud <a id="ref-for-comb-one④"></a>\| x-loud\] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<decibel\>](#typedef-voice-volume-decibel)\] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-voice-volume-silent"></a>[silent](#valdef-voice-volume-silent), or a keyword value and optionally also a decibel offset (if not zero)                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                         |

<a id="ref-for-propdef-voice-volume①"></a>

<a id="ref-for-aural-box-model"></a>

The [voice-volume](#propdef-voice-volume) property allows authors to control the amplitude of the audio waveform generated by the speech synthesizer, and is also used to adjust the relative volume level of [audio cues](#cue-props) within the [aural box model](#aural-box-model) of the selected element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`volume` attribute of the `prosody` element](https://www.w3.org/TR/speech-synthesis11/#edef_prosody) from the SSML markup language [\[SSML\]](#biblio-ssml), there are notable discrepancies. For example, CSS Speech volume keywords and decibels units are not mutually-exclusive, due to how values are inherited and combined for selected elements.

<a id="valdef-voice-volume-silent"></a>silent

Specifies that no sound is generated (the text is read "silently").

<a id="ref-for-propdef-voice-volume②"></a>

<a id="ref-for-valdef-voice-volume-silent①"></a>

<a id="ref-for-propdef-speak"></a>

<a id="ref-for-aural-box-model①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This has the same effect as using negative infinity decibels. Also note that there is a difference between an element whose [voice-volume](#propdef-voice-volume) property has a value of [silent](#valdef-voice-volume-silent), and an element whose [speak](#propdef-speak) property has the value none. With the former, the selected element takes up the same time as if it was spoken, including any pause before and after the element, but no sound is generated (and descendants within the [aural box model](#aural-box-model) of the selected element can override the <a id="ref-for-propdef-voice-volume③"></a>voice-volume value, and may therefore generate audio output). With the latter, the selected element is not rendered in the aural dimension and no time is allocated for playback (descendants within the <a id="ref-for-aural-box-model②"></a>aural box model of the selected element can override the <a id="ref-for-propdef-speak①"></a>speak value, and may therefore generate audio output).

<a id="valdef-voice-volume-x-soft"></a>x-soft, <a id="valdef-voice-volume-soft"></a>soft, <a id="valdef-voice-volume-medium"></a>medium, <a id="valdef-voice-volume-loud"></a>loud, <a id="valdef-voice-volume-x-loud"></a>x-loud

<a id="ref-for-valdef-voice-volume-loud"></a>

<a id="ref-for-valdef-voice-volume-soft"></a>

<a id="ref-for-valdef-voice-volume-medium"></a>

<a id="ref-for-valdef-voice-volume-x-loud"></a>

<a id="ref-for-valdef-voice-volume-x-soft"></a>

This sequence of keywords corresponds to monotonically non-decreasing volume levels, mapped to implementation-dependent values that meet the listener’s requirements with regards to perceived loudness. These audio levels are typically provided via a preference mechanism that allow users to calibrate sound options according to their auditory environment. The keyword [x-soft](#valdef-voice-volume-x-soft) maps to the user’s <em>minimum audible</em> volume level, [x-loud](#valdef-voice-volume-x-loud) maps to the user’s <em>maximum tolerable</em> volume level, [medium](#valdef-voice-volume-medium) maps to the user’s <em>preferred</em> volume level, [soft](#valdef-voice-volume-soft) and [loud](#valdef-voice-volume-loud) map to intermediary values.

<a id="ref-for-typedef-voice-volume-decibel①"></a>

<a id="valdef-voice-volume-decibel"></a>[\<decibel\>](#typedef-voice-volume-decibel)

<a id="ref-for-typedef-voice-volume-decibel②"></a>

<a id="ref-for-propdef-voice-volume④"></a>

<a id="ref-for-valdef-voice-volume-silent②"></a>

This represents a change (positive or negative) relative to the given keyword value (see enumeration above), or to the default value for the root element, or otherwise to the inherited volume level (which may itself be a combination of a keyword value and decibel offset, in which case the decibel values are combined additively). When the inherited volume level is [silent](#valdef-voice-volume-silent), this [voice-volume](#propdef-voice-volume) resolves to <a id="ref-for-valdef-voice-volume-silent③"></a>silent too, regardless of the specified [\<decibel\>](#typedef-voice-volume-decibel) value.

<a id="ref-for-typedef-voice-volume-decibel③"></a>

<a id="ref-for-dimension"></a>

The <a id="typedef-voice-volume-decibel"></a>[\<decibel\>](#typedef-voice-volume-decibel) type denotes a [dimension](https://www.w3.org/TR/css-values-4/#dimension) with a "dB" (decibel unit) unit identifier. Decibels represent the ratio of the squares of the new signal amplitude <var>a1</var> and the current amplitude <var>a0</var>, as per the following logarithmic equation: volume(dB) = 20 × log10(<var>a1</var> / <var>a0</var>).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: -6.0dB is approximately half the amplitude of the audio signal, and +6.0dB is approximately twice the amplitude.

<a id="ref-for-valdef-voice-volume-x-soft①"></a>

<a id="ref-for-valdef-voice-volume-x-loud①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Perceived loudness depends on various factors, such as the listening environment, user preferences or physical abilities. The effective volume variation between [x-soft](#valdef-voice-volume-x-soft) and [x-loud](#valdef-voice-volume-x-loud) represents the dynamic range (in terms of loudness) of the audio output. Typically, this range would be compressed in a noisy context, i.e. the perceived loudness corresponding to <a id="ref-for-valdef-voice-volume-x-soft②"></a>x-soft would effectively be closer to <a id="ref-for-valdef-voice-volume-x-loud②"></a>x-loud than it would be in a quiet environment. There may also be situations where both <a id="ref-for-valdef-voice-volume-x-soft③"></a>x-soft and <a id="ref-for-valdef-voice-volume-x-loud③"></a>x-loud would map to low volume levels, such as in listening environments requiring discretion (e.g. library, night-reading).

<a id="ref-for-propdef-voice-balance①"></a>

### <a id="mixing-props-voice-balance"></a>6.2.  The [voice-balance](#propdef-voice-balance) property

| Field               | Definition                                                                                                                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-balance"></a>voice-balance                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑤"></a><a id="ref-for-number-value"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) left <a id="ref-for-comb-one⑥"></a>\| center <a id="ref-for-comb-one⑦"></a>\| right <a id="ref-for-comb-one⑧"></a>\| leftwards <a id="ref-for-comb-one⑨"></a>\| rightwards |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | center                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified value resolved to a \<number\> between -100 and 100 (inclusive)                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                            |

<a id="ref-for-propdef-voice-balance②"></a>

The [voice-balance](#propdef-voice-balance) property controls the spatial distribution of audio output across a lateral sound stage: one extremity is on the left, the other extremity is on the right hand side, relative to the listener’s position. Authors can specify intermediary steps between left hand right extremities, to represent the audio separation along the resulting left-right axis.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The functionality provided by this property has no match in the SSML markup language [\[SSML\]](#biblio-ssml).

<a id="ref-for-number-value①"></a>

<a id="valdef-voice-balance-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="ref-for-number"></a>

A [number](https://www.w3.org/TR/css-values-4/#number) between -100 and 100 (inclusive). Values smaller than -100 are clamped to -100. Values greater than 100 are clamped to 100. The value -100 represents the left side, and the value 100 represents the right side. The value 0 represents the center point whereby there is no discernible audio separation between left and right sides. (In a stereo sound system, this corresponds to equal distribution of audio signals between left and right speakers).

<a id="valdef-voice-balance-left"></a>left

Same as -100.

<a id="valdef-voice-balance-center"></a>center

Same as 0.

<a id="valdef-voice-balance-right"></a>right

Same as 100.

<a id="valdef-voice-balance-leftwards"></a>leftwards

<a id="ref-for-propdef-voice-balance③"></a>

Moves the sound to the left by subtracting 20 from the inherited [voice-balance](#propdef-voice-balance) value (and by clamping the resulting number to -100).

<a id="valdef-voice-balance-rightwards"></a>rightwards

<a id="ref-for-propdef-voice-balance④"></a>

Moves the sound to the right, by adding 20 to the inherited [voice-balance](#propdef-voice-balance) value (and by clamping the resulting number to 100).

User agents can be connected to different kinds of sound systems, featuring varying audio mixing capabilities. The expected behavior for mono, stereo, and surround sound systems is defined as follows:

- <a id="ref-for-propdef-voice-balance⑤"></a>

  When user agents produce audio via a mono-aural sound system (i.e. single-speaker setup), the [voice-balance](#propdef-voice-balance) property has no effect.

- <a id="ref-for-propdef-voice-balance⑥"></a>

  When user agents produce audio through a stereo sound system (e.g. two speakers, or a pair of headphones), the left-right distribution of audio signals can precisely match the authored values for the [voice-balance](#propdef-voice-balance) property.

- <a id="ref-for-propdef-voice-balance⑦"></a>

  <a id="ref-for-valdef-voice-balance-center"></a>

  When user agents are capable of mixing audio signals through more than 2 channels (e.g. 5-speakers surround sound system, including a dedicated center channel), the physical distribution of audio signals resulting from the application of the [voice-balance](#propdef-voice-balance) property should be performed so that the listener perceives sound as if it was coming from a basic stereo layout. For example, the center channel as well as the left/right speakers may be used all together in order to emulate the behavior of the [center](#valdef-voice-balance-center) value.

<a id="ref-for-propdef-voice-balance⑧"></a>

Future revisions of the CSS Speech module may include support for three-dimensional audio, which would effectively enable authors to specify “azimuth” and “elevation” values. In the future, content authored using the current specification may therefore be consumed by user agents which are compliant with the version of CSS Speech that supports three-dimensional audio. In order to prepare for this possibility, the values enabled by the current [voice-balance](#propdef-voice-balance) property are designed to remain compatible with “azimuth” angles. More precisely, the mapping between the current left-right audio axis (lateral sound stage) and the envisioned 360 degrees plane around the listener’s position is defined as follows:

- <a id="ref-for-valdef-voice-balance-center①"></a>

  The value 0 maps to zero degrees ([center](#valdef-voice-balance-center)). This is in "front" of the listener, not from "behind".

- <a id="ref-for-propdef-left"></a>

  The value -100 maps to -40 degrees ([left](https://www.w3.org/TR/css-position-3/#propdef-left)). Negative angles are in the counter-clockwise direction (assuming the audio stage is seen from the top).

- <a id="ref-for-propdef-right"></a>

  The value 100 maps to 40 degrees ([right](https://www.w3.org/TR/css-position-3/#propdef-right)). Positive angles are in the clockwise direction (assuming the audio stage is seen from the top).

- Intermediary values on the scale from 100 to 100 map to the angles between -40 and 40 degrees in a numerically linearly-proportional manner. For example, -50 maps to -20 degrees.

<a id="ref-for-propdef-voice-balance⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Sound systems can be configured by users in such a way that it would interfere with the left-right audio distribution specified by document authors. Typically, the various “surround” modes available in modern sound systems (including systems based on basic stereo speakers) tend to greatly alter the perceived spatial arrangement of audio signals. The illusion of a three-dimensional sound stage is often achieved using a combination of phase shifting, digital delay, volume control (channel mixing), and other techniques. Some users may even configure their system to “downgrade” any rendered sound to a single mono channel, in which case the effect of the [voice-balance](#propdef-voice-balance) property would obviously not be perceivable at all. The rendering fidelity of authored content is therefore dependent on such user customizations, and the <a id="ref-for-propdef-voice-balance①⓪"></a>voice-balance property merely specifies the desired end-result.

<a id="ref-for-propdef-voice-balance①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Many speech synthesizers only generate mono sound, and therefore do not intrinsically support the [voice-balance](#propdef-voice-balance) property. The sound distribution along the left-right axis consequently occurs at post-synthesis stage (when the speech-enabled user agent mixes the various audio sources authored within the document)

## <a id="speaking-props"></a>7.  Speaking properties

<a id="ref-for-propdef-speak②"></a>

### <a id="speaking-props-speak"></a>7.1.  The [speak](#propdef-speak) property

| Field               | Definition                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-speak"></a>speak                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) never <a id="ref-for-comb-one①①"></a>\| always |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                   |

<a id="ref-for-propdef-speak③"></a>

The [speak](#propdef-speak) property determines whether or not to render text aurally.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The functionality provided by this property has no match in the SSML markup language [\[SSML\]](#biblio-ssml).

<a id="valdef-speak-auto"></a>auto  
<a id="ref-for-valdef-visibility-visible"></a>

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-valdef-speak-always"></a>

<a id="ref-for-valdef-speak-auto"></a>

<a id="ref-for-valdef-display-none"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-valdef-speak-never"></a>

Resolves to a computed value of [never](#valdef-speak-never) when [display](https://www.w3.org/TR/css-display-3/#propdef-display) is [none](https://www.w3.org/TR/css-display-3/#valdef-display-none), otherwise resolves to a computed value of [auto](#valdef-speak-auto). The used value of a computed <a id="ref-for-valdef-speak-auto①"></a>auto is equivalent to [always](#valdef-speak-always) if [visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility) is [visible](https://www.w3.org/TR/css-display-3/#valdef-visibility-visible) and to <a id="ref-for-valdef-speak-never①"></a>never otherwise.

<a id="ref-for-valdef-display-none①"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-speak-auto②"></a>

<a id="ref-for-propdef-speak④"></a>

<a id="ref-for-valdef-speak-never②"></a>

<a id="ref-for-valdef-speak-always①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [none](https://www.w3.org/TR/css-display-3/#valdef-display-none) value of the [display](https://www.w3.org/TR/css-display-3/#propdef-display) property cannot be overridden by descendants of the selected element, but the [auto](#valdef-speak-auto) value of [speak](#propdef-speak) can, however, be overridden using either of [never](#valdef-speak-never) or [always](#valdef-speak-always).

<a id="valdef-speak-never"></a>never  
This value causes an element (including pauses, cues, rests and actual content) to not be rendered (i.e., the element has no effect in the aural dimension).

<a id="ref-for-propdef-display②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Any of the descendants of the affected element are allowed to override this value, so descendants can actually take part in the aural rendering despite using [display: none](https://www.w3.org/TR/css-display-3/#propdef-display) at this level. However, the pauses, cues, and rests of the ancestor element remain “deactivated” in the aural dimension, and therefore do not contribute to the [collapsing of pauses](#collapsed-pauses) or additive behavior of adjoining rests.

<a id="valdef-speak-always"></a>always  
<a id="ref-for-propdef-speak⑤"></a>

<a id="ref-for-propdef-display③"></a>

The element is rendered aurally (regardless of its [display](https://www.w3.org/TR/css-display-3/#propdef-display) value, or the <a id="ref-for-propdef-display④"></a>display or [speak](#propdef-speak) values of its ancestors).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Using this value can result in the element being rendered in the aural dimension even though it would not be rendered on the visual canvas.

<a id="ref-for-propdef-speak-as"></a>

### <a id="speaking-props-speak-as"></a>7.2.  The [speak-as](#propdef-speak-as) property

| Field               | Definition                                                                                                                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-speak-as"></a>speak-as                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any①"></a><a id="ref-for-comb-one①②"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) spell-out [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) digits <a id="ref-for-comb-any②"></a>\|\| \[ literal-punctuation <a id="ref-for-comb-one①③"></a>\| no-punctuation \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                  |

<a id="ref-for-propdef-speak-as①"></a>

The [speak-as](#propdef-speak-as) property determines in what manner text gets rendered aurally, based upon a predefined list of possibilities.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The functionality provided by this property is conceptually similar to the [`say-as` element](https://www.w3.org/TR/speech-synthesis11/#edef_say-as) from the SSML markup language [\[SSML\]](#biblio-ssml) (whose possible values are described in the [\[SSML-SAYAS\]](#biblio-ssml-sayas) W3C Note). Although the design goals are similar, the CSS model is limited to a basic set of pronunciation rules.

<a id="valdef-speak-as-normal"></a>normal  
Uses language-dependent pronunciation rules for rendering the element’s content. For example, punctuation is not spoken as-is, but instead rendered naturally as appropriate pauses.

<a id="valdef-speak-as-spell-out"></a>spell-out  
Spells the text one letter at a time (useful for acronyms and abbreviations). In languages where accented characters are rare, it is permitted to drop accents in favor of alternative unaccented spellings. As an example, in English, the word “rôle” can also be written as “role”. A conforming implementation would thus be able to spell-out “rôle” as “R O L E”.

<a id="valdef-speak-as-digits"></a>digits  
Speak numbers one digit at a time, for instance, “twelve” would be spoken as “one two”, and “31” as “three one”.

<a id="ref-for-propdef-speak-as②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Speech synthesizers are knowledgeable about what a <em>number</em> is. The [speak-as](#propdef-speak-as) property enables some level of control on how user agents render numbers, and may be implemented as a preprocessing step before passing the text to the actual speech synthesizer.

<a id="valdef-speak-as-literal-punctuation"></a>literal-punctuation  
Punctuation such as semicolons, braces, and so on is named aloud (i.e. spoken literally) rather than rendered naturally as appropriate pauses.

<a id="valdef-speak-as-no-punctuation"></a>no-punctuation  
Punctuation is not rendered: neither spoken nor rendered as pauses.

## <a id="pause-props"></a>8.  Pause properties 

<a id="ref-for-propdef-pause-before"></a>

<a id="ref-for-propdef-pause-after"></a>

### <a id="pause-props-pause-before-after"></a>8.1.  The [pause-before](#propdef-pause-before) and [pause-after](#propdef-pause-after) properties

| Field               | Definition                                                                                                                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-pause-before"></a>pause-before, <a id="propdef-pause-after"></a>pause-after                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①④"></a>\<time\> [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one①⑤"></a>\| x-weak <a id="ref-for-comb-one①⑥"></a>\| weak <a id="ref-for-comb-one①⑦"></a>\| medium <a id="ref-for-comb-one①⑧"></a>\| strong <a id="ref-for-comb-one①⑨"></a>\| x-strong |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                          |

<a id="ref-for-propdef-pause-before①"></a>

<a id="ref-for-propdef-pause-after①"></a>

<a id="ref-for-propdef-cue-before"></a>

<a id="ref-for-propdef-cue-after"></a>

<a id="ref-for-aural-box-model③"></a>

The [pause-before](#propdef-pause-before) and [pause-after](#propdef-pause-after) properties specify a prosodic boundary (silence with a specific duration) that occurs before (or after) the speech synthesis rendition of the element, or if any [cue-before](#propdef-cue-before) (or [cue-after](#propdef-cue-after)) is specified, before (or after) the cue within the [aural box model](#aural-box-model).

<a id="ref-for-propdef-pause①"></a>

<a id="ref-for-aural-box-model④"></a>

Note Although the functionality provided by this property is similar to the [`break` element](https://www.w3.org/TR/speech-synthesis11/#edef_break) from the SSML markup language [\[SSML\]](#biblio-ssml), the application of [pause](#propdef-pause) prosodic boundaries within the [aural box model](#aural-box-model) of CSS Speech requires special considerations (e.g. ["collapsed" pauses](#collapsed-pauses)).

<a id="ref-for-time-value"></a>

<a id="valdef-pause-before-time"></a>[\<time\>](https://www.w3.org/TR/css-values-4/#time-value)

Expresses the pause in absolute time units (seconds and milliseconds, e.g. "+3s", "250ms"). Only non-negative values are allowed.

<a id="valdef-pause-before-none"></a>none

Equivalent to 0ms (no prosodic break is produced by the speech processor).

<a id="valdef-pause-before-x-weak"></a>x-weak, <a id="valdef-pause-before-weak"></a>weak, <a id="valdef-pause-before-medium"></a>medium, <a id="valdef-pause-before-strong"></a>strong, and <a id="valdef-pause-before-x-strong"></a>x-strong

Expresses the pause by the strength of the prosodic break in speech output. The exact time is implementation-dependent. The values indicate monotonically non-decreasing (conceptually increasing) break strength between elements.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Stronger content boundaries are typically accompanied by pauses. For example, the breaks between paragraphs are typically much more substantial than the breaks between words within a sentence.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1bceb2dc"></a> This example illustrates how the default strengths of prosodic breaks for specific elements (which are defined by the user agent stylesheet) can be overridden by authored styles.
>
> ```text
> p { pause: none } /* pause-before: none; pause-after: none */
> ```
<a id="ref-for-propdef-pause②"></a>

### <a id="pause-props-pause"></a>8.2.  The [pause](#propdef-pause) shorthand property

| Field               | Definition                                                                                                                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-pause"></a>pause                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-propdef-pause-after②"></a><a id="ref-for-propdef-pause-before②"></a>[\<'pause-before'\>](#propdef-pause-before) [\<'pause-after'\>](#propdef-pause-after)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                     |

<a id="ref-for-propdef-pause③"></a>

<a id="ref-for-propdef-pause-before③"></a>

<a id="ref-for-propdef-pause-after③"></a>

The [pause](#propdef-pause) property is a shorthand property for [pause-before](#propdef-pause-before) and [pause-after](#propdef-pause-after). If two values are given, the first value is <a id="ref-for-propdef-pause-before④"></a>pause-before and the second is <a id="ref-for-propdef-pause-after④"></a>pause-after. If only one value is given, it applies to both properties.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-377a7cf6"></a>
>
> Examples of property values:
>
> ```text
> h1 { pause: 20ms; } /* pause-before: 20ms; pause-after: 20ms */
> h2 { pause: 30ms 40ms; } /* pause-before: 30ms; pause-after: 40ms */
> h3 { pause-after: 10ms; } /* pause-before: unspecified; pause-after: 10ms */
> ```
### <a id="collapsed-pauses"></a>8.3.  Collapsing pauses

The pause defines the minimum distance of the aural "box" to the aural "boxes" before and after it. Adjoining pauses are merged by selecting the strongest named break and the longest absolute time interval. For example, "strong" is selected when merging "strong" and "weak", "1s" is selected when merging "1s" and "250ms", and "strong" and "250ms" take effect additively when merging "strong" and "250ms".

The following pauses are adjoining:

- <a id="ref-for-propdef-cue-after①"></a>

  <a id="ref-for-propdef-rest-after"></a>

  <a id="ref-for-propdef-pause-after⑤"></a>

  The [pause-after](#propdef-pause-after) of an aural "box" and the <a id="ref-for-propdef-pause-after⑥"></a>pause-after of its last child, provided the former has no [rest-after](#propdef-rest-after) and no [cue-after](#propdef-cue-after).

- <a id="ref-for-propdef-cue-before①"></a>

  <a id="ref-for-propdef-rest-before"></a>

  <a id="ref-for-propdef-pause-before⑤"></a>

  The [pause-before](#propdef-pause-before) of an aural "box" and the <a id="ref-for-propdef-pause-before⑥"></a>pause-before of its first child, provided the former has no [rest-before](#propdef-rest-before) and no [cue-before](#propdef-cue-before).

- <a id="ref-for-propdef-pause-before⑦"></a>

  <a id="ref-for-propdef-pause-after⑦"></a>

  The [pause-after](#propdef-pause-after) of an aural "box" and the [pause-before](#propdef-pause-before) of its next sibling.

- <a id="ref-for-propdef-speak⑥"></a>

  <a id="ref-for-propdef-cue-after②"></a>

  <a id="ref-for-propdef-cue-before②"></a>

  <a id="ref-for-propdef-rest-after①"></a>

  <a id="ref-for-propdef-rest-before①"></a>

  <a id="ref-for-propdef-voice-duration①"></a>

  <a id="ref-for-propdef-pause-after⑧"></a>

  <a id="ref-for-propdef-pause-before⑧"></a>

  The [pause-before](#propdef-pause-before) and [pause-after](#propdef-pause-after) of an aural "box", if the "box" has a [voice-duration](#propdef-voice-duration) of "0ms" and no [rest-before](#propdef-rest-before) or [rest-after](#propdef-rest-after) and no [cue-before](#propdef-cue-before) or [cue-after](#propdef-cue-after), or if the "box" has no rendered content at all (see [speak](#propdef-speak)).

A collapsed pause is considered adjoining to another pause if any of its component pauses is adjoining to that pause.

<a id="ref-for-propdef-pause④"></a>

<a id="ref-for-propdef-cue①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [pause](#propdef-pause) has been moved from between the element’s contents and any [cue](#propdef-cue) to outside the <a id="ref-for-propdef-cue②"></a>cue. This is not backwards compatible with the informative CSS2.1 Aural appendix [\[CSS2\]](#biblio-css2).

## <a id="rest-props"></a>9.  Rest properties

<a id="ref-for-propdef-rest-before②"></a>

<a id="ref-for-propdef-rest-after②"></a>

### <a id="rest-props-rest-before-after"></a>9.1.  The [rest-before](#propdef-rest-before) and [rest-after](#propdef-rest-after) properties

| Field               | Definition                                                                                                                                                                                                                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-rest-before"></a>rest-before, <a id="propdef-rest-after"></a>rest-after                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⓪"></a><a id="ref-for-time-value①"></a>[\<time\>](https://www.w3.org/TR/css-values-4/#time-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one②①"></a>\| x-weak <a id="ref-for-comb-one②②"></a>\| weak <a id="ref-for-comb-one②③"></a>\| medium <a id="ref-for-comb-one②④"></a>\| strong <a id="ref-for-comb-one②⑤"></a>\| x-strong |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                               |

<a id="ref-for-propdef-rest-before③"></a>

<a id="ref-for-propdef-rest-after③"></a>

<a id="ref-for-aural-box-model⑤"></a>

The [rest-before](#propdef-rest-before) and [rest-after](#propdef-rest-after) properties specify a prosodic boundary (silence with a specific duration) that occurs before (or after) the speech synthesis rendition of an element within the [aural box model](#aural-box-model).

<a id="ref-for-propdef-rest②"></a>

<a id="ref-for-aural-box-model⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`break` element](https://www.w3.org/TR/speech-synthesis11/#edef_break) from the SSML markup language [\[SSML\]](#biblio-ssml), the application of [rest](#propdef-rest) prosodic boundaries within the [aural box model](#aural-box-model) of CSS Speech requires special considerations (e.g. interspersed audio cues, additive adjacent rests).

<a id="ref-for-time-value②"></a>

<a id="valdef-rest-before-time"></a>[\<time\>](https://www.w3.org/TR/css-values-4/#time-value)

Expresses the rest in absolute time units (seconds and milliseconds, e.g. "+3s", "250ms"). Only non-negative values are allowed.

<a id="valdef-rest-before-none"></a>none

Equivalent to 0ms. (No prosodic break is produced by the speech processor.)

<a id="valdef-rest-before-x-weak"></a>x-weak, <a id="valdef-rest-before-weak"></a>weak, <a id="valdef-rest-before-medium"></a>medium, <a id="valdef-rest-before-strong"></a>strong, and <a id="valdef-rest-before-x-strong"></a>x-strong

Expresses the rest by the strength of the prosodic break in speech output. The exact time is implementation-dependent. The values indicate monotonically non-decreasing (conceptually increasing) break strength between elements.

<a id="ref-for-propdef-cue-before③"></a>

<a id="ref-for-propdef-cue-after③"></a>

As opposed to [pause properties](#pause-props), the rest is inserted between the element’s content and any [cue-before](#propdef-cue-before) or [cue-after](#propdef-cue-after) content. Adjoining rests are treated additively, and do not collapse.

<a id="ref-for-propdef-rest③"></a>

### <a id="rest-props-rest"></a>9.2.  The [rest](#propdef-rest) shorthand property

| Field               | Definition                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-rest"></a>rest                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-propdef-rest-after④"></a><a id="ref-for-propdef-rest-before④"></a>[\<'rest-before'\>](#propdef-rest-before) [\<'rest-after'\>](#propdef-rest-after)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                 |

<a id="ref-for-propdef-rest④"></a>

<a id="ref-for-propdef-rest-before⑤"></a>

<a id="ref-for-propdef-rest-after⑤"></a>

The [rest](#propdef-rest) property is a shorthand for [rest-before](#propdef-rest-before) and [rest-after](#propdef-rest-after). If two values are given, the first value is <a id="ref-for-propdef-rest-before⑥"></a>rest-before and the second is <a id="ref-for-propdef-rest-after⑥"></a>rest-after. If only one value is given, it applies to both properties.

## <a id="cue-props"></a>10.  Cue properties

<a id="ref-for-propdef-cue-before④"></a>

<a id="ref-for-propdef-cue-after④"></a>

### <a id="cue-props-cue-before-after"></a>10.1.  The [cue-before](#propdef-cue-before) and [cue-after](#propdef-cue-after) properties

| Field               | Definition                                                                                                                                                                                                                                                                                |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-cue-before"></a>cue-before, <a id="propdef-cue-after"></a>cue-after                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⑥"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-typedef-voice-volume-decibel④"></a><a id="ref-for-value-def-uri"></a>[\<uri\>](https://drafts.csswg.org/css2/#value-def-uri) [\<decibel\>](#typedef-voice-volume-decibel)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [\|](https://www.w3.org/TR/css-values-4/#comb-one) none |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                               |

<a id="ref-for-propdef-cue-before⑤"></a>

<a id="ref-for-propdef-cue-after⑤"></a>

The [cue-before](#propdef-cue-before) and [cue-after](#propdef-cue-after) properties specify auditory icons (i.e. pre-recorded / pre-generated sound clips) to be played before (or after) the element within the [aural box model](#aural-model).

<a id="ref-for-aural-box-model⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property may appear related to the [`audio` element](https://www.w3.org/TR/speech-synthesis11/#edef_audio) from the SSML markup language [\[SSML\]](#biblio-ssml), there are in fact major discrepancies. For example, the [aural box model](#aural-box-model) means that audio cues are associated to the element’s volume level; and CSS Speech’s auditory icons provide limited functionality compared to SSML’s `audio` element.

<a id="ref-for-value-def-uri①"></a>

<a id="valdef-cue-before-uri"></a>[\<uri\>](https://drafts.csswg.org/css2/#value-def-uri)

The URI designates an auditory icon resource. When a user agent is not able to render the specified auditory icon (e.g. missing file resource, or unsupported audio codec), it is recommended to produce an alternative cue, such as a bell sound.

none

Specifies that no auditory icon is used.

<a id="ref-for-typedef-voice-volume-decibel⑤"></a>

[\<decibel\>](#typedef-voice-volume-decibel)

<a id="ref-for-aural-box-model⑧"></a>

<a id="ref-for-propdef-voice-volume⑤"></a>

Represents a change (positive or negative) relative to the computed value of the [voice-volume](#propdef-voice-volume) property within the [aural box model](#aural-box-model) of the selected element. (As a result, the volume level of an audio cue changes when the <a id="ref-for-propdef-voice-volume⑥"></a>voice-volume property changes). When omitted, the implied value computes to 0dB.

<a id="ref-for-propdef-voice-volume⑦"></a>

<a id="ref-for-valdef-voice-volume-silent④"></a>

<a id="ref-for-typedef-voice-volume-decibel⑥"></a>

When the computed value of the [voice-volume](#propdef-voice-volume) property is [silent](#valdef-voice-volume-silent), the audio cue is also set to <a id="ref-for-valdef-voice-volume-silent⑤"></a>silent (regardless of this specified [\<decibel\>](#typedef-voice-volume-decibel) value). Otherwise (when not <a id="ref-for-valdef-voice-volume-silent⑥"></a>silent), <a id="ref-for-propdef-voice-volume⑧"></a>voice-volume values are always specified relatively to the volume level keywords (see the definition of <a id="ref-for-propdef-voice-volume⑨"></a>voice-volume), which map to a user-calibrated scale of "preferred" loudness settings. If the inherited <a id="ref-for-propdef-voice-volume①⓪"></a>voice-volume value already contains a decibel offset, the dB offset specific to the audio cue is combined additively.

<a id="ref-for-valdef-voice-volume-silent⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is a difference between an audio cue whose volume is set to [silent](#valdef-voice-volume-silent) and one whose value is none. In the former case, the audio cue takes up the same time as if it had been played, but no sound is generated. In the latter case, the there is no manifestation of the audio cue at all (i.e. no time is allocated for the cue in the aural dimension).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7f061363"></a>
>
> Examples of property values:
>
> ```text
> a
> {
>   cue-before: url(/audio/bell.aiff) -3dB;
>   cue-after: url(dong.wav);
> }
> 
> h1
> {
>   cue-before: url(../clips-1/pop.au) +6dB;
>   cue-after: url(../clips-2/pop.au) 6dB;
> }
> 
> div.caution { cue-before: url(./audio/caution.wav) +8dB; }
> ```
### <a id="cue-props-volume"></a>10.2.  Relation between audio cues and speech synthesis volume levels

<em>This section is non-normative.</em>

<a id="ref-for-aural-box-model⑨"></a>

<a id="ref-for-typedef-voice-volume-decibel⑦"></a>

<a id="ref-for-propdef-voice-volume①①"></a>

<a id="ref-for-valdef-voice-volume-silent⑧"></a>

The volume levels of audio cues and of speech synthesis within the [aural box model](#aural-box-model) of a selected element are related. For example, the desired effect of an audio cue whose volume level is set at +0dB (as specified by the [\<decibel\>](#typedef-voice-volume-decibel) value) is that its perceived loudness during playback is close to that of the speech synthesis rendition of the selected element, as dictated by the computed value of the [voice-volume](#propdef-voice-volume) property. Note that a [silent](#valdef-voice-volume-silent) computed value for the <a id="ref-for-propdef-voice-volume①②"></a>voice-volume property results in audio cues being "forcefully" silenced as well (i.e. regardless of the specified audio cue <a id="ref-for-typedef-voice-volume-decibel⑧"></a>\<decibel\> value)

<a id="ref-for-propdef-voice-volume①③"></a>

<a id="ref-for-propdef-voice-family"></a>

The volume keywords of the [voice-volume](#propdef-voice-volume) property are user-calibrated to match requirements not known at authoring time (e.g. auditory environment, personal preferences). Therefore, in order to achieve this approximate loudness alignment of audio cues and speech synthesis, authors should ensure that the volume level of audio cues (on average, as there may be discrete variations of perceived loudness due to changes in the audio stream, such as intonation, stress, etc.) matches the output of a speech synthesis rendition based on the [voice-family](#propdef-voice-family) intended for use, given "typical" listening conditions (i.e. default system volume levels, centered equalization across the frequency spectrum). As speech processors are capable of directly controlling the waveform amplitude of generated text-to-speech audio, and because user agents are able to adjust the volume output of audio cues (i.e. amplify or attenuate audio signals based on the intrinsic waveform amplitude of digitized sound clips), this sets a baseline that enables implementations to manage the loudness of both TTS and cue audio streams within the aural box model, relative to user-calibrated volume levels (see the keywords defined in the <a id="ref-for-propdef-voice-volume①④"></a>voice-volume property).

Due to the complex relationship between perceived audio characteristics (e.g. loudness) and the processing applied to the digitized audio signal (e.g. signal compression), we refer to a simple scenario whereby the attenuation is indicated in decibels, typically ranging from 0dB (i.e. maximum audio input, near clipping threshold) to -60dB (i.e. total silence). Given this context, a "standard" audio clip would oscillate between these values, the loudest peak levels would be close to -3dB (to avoid distortion), and the relevant audible passages would have average (RMS) volume levels as high as possible (i.e. not too quiet, to avoid background noise during amplification). This would roughly provide an audio experience that could be seamlessly combined with text-to-speech output (i.e. there would be no discernible difference in volume levels when switching from pre-recorded audio to speech synthesis). Although there exists no industry-wide standard to support such convention, different TTS engines tend to generate comparably-loud audio signals when no gain or attenuation is specified. For voice and soft music, -15dB RMS seems to be pretty standard.

<a id="ref-for-propdef-cue③"></a>

### <a id="cue-props-cue"></a>10.3.  The [cue](#propdef-cue) shorthand property

| Field               | Definition                                                                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-cue"></a>cue                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt③"></a><a id="ref-for-propdef-cue-after⑥"></a><a id="ref-for-propdef-cue-before⑥"></a>[\<'cue-before'\>](#propdef-cue-before) [\<'cue-after'\>](#propdef-cue-after)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | N/A (see individual properties)                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                             |

<a id="ref-for-propdef-cue④"></a>

<a id="ref-for-propdef-cue-before⑦"></a>

<a id="ref-for-propdef-cue-after⑦"></a>

The [cue](#propdef-cue) property is a shorthand for [cue-before](#propdef-cue-before) and [cue-after](#propdef-cue-after). If two values are given the first value is <a id="ref-for-propdef-cue-before⑧"></a>cue-before and the second is <a id="ref-for-propdef-cue-after⑧"></a>cue-after. If only one value is given, it applies to both properties.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef7757db"></a>
>
> Example of shorthand notation:
>
> ```text
> h1
> {
>   cue-before: url(pop.au);
>   cue-after: url(pop.au);
> }
> /* ...is equivalent to: */
> h1
> {
>   cue: url(pop.au);
> }
> ```
## <a id="voice-char-props"></a>11.  Voice characteristic properties

<a id="ref-for-propdef-voice-family①"></a>

### <a id="voice-props-voice-family"></a>11.1.  The [voice-family](#propdef-voice-family) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-family"></a>voice-family                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-zero-plus"></a><a id="ref-for-comb-comma"></a><a id="ref-for-comb-one②⑦"></a>\[\[\<family-name\> [\|](https://www.w3.org/TR/css-values-4/#comb-one) \<generic-voice\>\][,](https://www.w3.org/TR/css-values-4/#comb-comma)\][\*](https://www.w3.org/TR/css-values-4/#mult-zero-plus) \[\<family-name\> <a id="ref-for-comb-one②⑧"></a>\| \<generic-voice\>\] <a id="ref-for-comb-one②⑨"></a>\| preserve |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | implementation-dependent                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                 |

<a id="ref-for-propdef-voice-family②"></a>

<a id="ref-for-propdef-font-family"></a>

The [voice-family](#propdef-voice-family) property specifies a prioritized list of component values that are separated by commas to indicate that they are alternatives. (This is analogous to [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) in visual style sheets.) Each component value potentially designates a speech synthesis voice instance, by specifying match criteria. See the [voice selection](#voice-selection) section on this topic.

<a id="ref-for-typedef-generic-voice"></a>

<a id="ref-for-typedef-voice-family-age"></a>

<a id="ref-for-typedef-voice-family-gender"></a>

<a id="ref-for-integer-value"></a>

<a id="typedef-generic-voice"></a>[\<generic-voice\>](#typedef-generic-voice) = \[[\<age\>](#typedef-voice-family-age)? [\<gender\>](#typedef-voice-family-gender) [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)?\]

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`voice` element](https://www.w3.org/TR/speech-synthesis11/#edef_voice) from the SSML markup language [\[SSML\]](#biblio-ssml), CSS Speech does not provide an equivalent to SSML’s sophisticated voice language selection. This technical limitation may be alleviated in a future revision of the Speech module.

<a id="ref-for-family-name-value"></a>

<a id="valdef-voice-family-family-name"></a>[\<family-name\>](https://www.w3.org/TR/css-fonts-4/#family-name-value)

<a id="ref-for-css-css-identifier"></a>

<a id="ref-for-string"></a>

<a id="ref-for-propdef-font-family①"></a>

Values are specific voice instances (e.g., Mike, comedian, mary, carlos2, "valley girl"). Like [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) names, voice names must either be given quoted as [strings](https://infra.spec.whatwg.org/#string), or unquoted as a sequence of one or more [CSS identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As a result, most punctuation characters, or digits at the start of each token, must be escaped in unquoted voice names.

If a sequence of identifiers is given as a voice name, the computed value is the name converted to a string by joining all the identifiers in the sequence by single spaces.

<a id="ref-for-valdef-voice-family-male"></a>

<a id="ref-for-valdef-voice-family-female"></a>

<a id="ref-for-valdef-voice-family-neutral"></a>

<a id="ref-for-css-wide-keywords①"></a>

<a id="ref-for-valdef-voice-family-preserve"></a>

Voice names that happen to be the same as the gender keywords ([male](#valdef-voice-family-male), [female](#valdef-voice-family-female) and [neutral](#valdef-voice-family-neutral)) or that happen to match the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) or [preserve](#valdef-voice-family-preserve) must be quoted to disambiguate with these keywords. The keyword default is reserved for future use and must also be quoted when used as voice names.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In [\[SSML\]](#biblio-ssml), voice names are space-separated and cannot contain whitespace characters.

It is recommended to quote voice names that contain white space, digits, or punctuation characters other than hyphens—even if these voice names are valid in unquoted form—in order to improve code clarity. For example:

```text
voice-family: "john doe", "Henry	the-8th";
```
<a id="ref-for-typedef-voice-family-age①"></a>

<a id="typedef-voice-family-age"></a>[\<age\>](#typedef-voice-family-age)

Possible values are <a id="valdef-voice-family-child"></a>child, <a id="valdef-voice-family-young"></a>young and <a id="valdef-voice-family-old"></a>old, indicating the preferred age category to match during voice selection.

<a id="ref-for-valdef-voice-family-child"></a>

<a id="ref-for-valdef-voice-family-young"></a>

<a id="ref-for-valdef-voice-family-old"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A recommended mapping with [\[SSML\]](#biblio-ssml) ages is: [child](#valdef-voice-family-child) = 6 y/o, [young](#valdef-voice-family-young) = 24 y/o, [old](#valdef-voice-family-old) = 75 y/o. More flexible age ranges may be used by the processor-dependent voice-matching algorithm.

<a id="ref-for-typedef-voice-family-gender①"></a>

<a id="typedef-voice-family-gender"></a>[\<gender\>](#typedef-voice-family-gender)

One of the keywords <a id="valdef-voice-family-male"></a>male, <a id="valdef-voice-family-female"></a>female, or <a id="valdef-voice-family-neutral"></a>neutral, specifying a male, female, or neutral voice, respectively.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The interpretation of the relationship between a person’s age or gender, and a recognizable type of voice, cannot realistically be defined in a universal manner as it effectively depends on numerous criteria (cultural, linguistic, biological, etc.). The functionality provided by this specification therefore represent a simplified model that can be reasonably applied to a broad variety of speech contexts, albeit at the cost of a certain degree of approximation. Future versions of this specification may refine the level of precision of the voice-matching algorithm, as speech processor implementations become more standardized.

<a id="ref-for-integer-value①"></a>

<a id="valdef-voice-family-integer"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)

An integer indicating the preferred variant (e.g. "the second male child voice"). Only positive integers (i.e. excluding zero) are allowed. The value 1 refers to the first of all matching voices.

<a id="valdef-voice-family-preserve"></a>preserve

<a id="ref-for-valdef-voice-family-preserve①"></a>

<a id="ref-for-valdef-all-inherit"></a>

<a id="ref-for-propdef-voice-family③"></a>

Indicates that the [voice-family](#propdef-voice-family) value gets inherited and used regardless of any potential language change within the content markup (see the section below about voice selection and language handling). This value behaves as [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) when applied to the root element. Note: Descendants of the element automatically inherit the [preserve](#valdef-voice-family-preserve) value, unless it is explicitly overridden by other <a id="ref-for-propdef-voice-family④"></a>voice-family values (e.g. name, gender, age).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63e6e87d"></a>
>
> Examples of invalid declarations:
>
> ```text
> voice-family: john/doe; /* forward slash character should be escaped */
> voice-family: john "doe"; /* identifier sequence cannot contain strings */
> voice-family: john!; /* exclamation mark should be escaped */
> voice-family: john@doe; /* "at" character should be escaped */
> voice-family: #john; /* identifier cannot start with hash character */
> voice-family: john 1st; /* identifier cannot start with digit */
> ```
#### <a id="voice-selection"></a>11.1.1.  Voice selection, content language

<a id="ref-for-propdef-voice-family⑤"></a>

The [voice-family](#propdef-voice-family) property is used to guide the selection of the speech synthesis voice instance. As part of this selection process, speech-capable user agents must also take into account the language of the selected element within the markup content. The "name", "gender", "age", and preferred "variant" (index) are voice selection hints that get carried down the content hierarchy as the <a id="ref-for-propdef-voice-family⑥"></a>voice-family property value gets inherited by descendant elements. At any point within the content structure, the language takes precedence (i.e. has a higher priority) over the specified CSS voice characteristics.

The following list outlines the voice selection algorithm (note that the definition of "language" is loose here, in order to cater for dialectic variations):

- If only a single voice instance is available for the language of the selected content, then this voice must be used, regardless of the specified CSS voice characteristics.

- <a id="ref-for-propdef-voice-family⑦"></a>

  If several voice instances are available for the language of the selected content, then the chosen voice is the one that most closely matches the specified name, or gender, age, and preferred voice variant. The actual definition of "best match" is processor-dependent. For example, in a system that only has male and female adult voices available, a reasonable match for "voice-family: young male" may well be a higher-pitched female voice, as this tone of voice would be closer to that of a young boy. If no voice instance matches the characteristics provided by any of the [voice-family](#propdef-voice-family) component values, the first available voice instance (amongst those suitable for the language of the selected content) must be used.

- If no voice is available for the language of the selected content, it is recommended that user agents let the user know about the lack of appropriate TTS voice.

<a id="ref-for-valdef-voice-family-preserve②"></a>

The speech synthesizer voice must be re-evaluated (i.e. the selection process must take place once again) whenever any of the CSS voice characteristics change within the content flow. The voice must also be re-calculated whenever the content language changes, unless the [preserve](#valdef-voice-family-preserve) keyword is used (this may be useful in cases where embedded foreign language text can be spoken using a voice not designed for this language, as demonstrated by the example below).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Dynamically computing a voice may lead to unexpected lag, so user agents should try to resolve concrete voice instances in the document tree before the playback starts.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-10e6a43e"></a>
>
> Examples of property values:
>
> ```text
> h1 { voice-family: announcer, old male; }
> p.romeo  { voice-family: romeo, young male; }
> p.juliet { voice-family: juliet, young female; }
> p.mercutio { voice-family: young male; }
> p.tybalt { voice-family: young male; }
> p.nurse { voice-family: amelie; }
> 
> ...
> 
> <p class="romeo" xml:lang="en-US">
>   The French text below will be spoken with an English voice:
>   <span style="voice-family: preserve;" xml:lang="fr-FR">Bonjour monsieur !</span>
> 
>   The English text below will be spoken with a voice different
>   than that corresponding to the class "romeo"
>   (which is inherited from the "p" parent element):
>   <span style="voice-family: female;">Hello sir!</span>
> </p>
> ```
<a id="ref-for-propdef-voice-rate"></a>

### <a id="voice-props-voice-rate"></a>11.2.  The [voice-rate](#propdef-voice-rate) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-rate"></a>voice-rate                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-percentage-value"></a><a id="ref-for-comb-any③"></a><a id="ref-for-comb-one③⓪"></a>\[normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) x-slow <a id="ref-for-comb-one③①"></a>\| slow <a id="ref-for-comb-one③②"></a>\| medium <a id="ref-for-comb-one③③"></a>\| fast <a id="ref-for-comb-one③④"></a>\| x-fast\] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to default value                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | a keyword value, and optionally also a percentage relative to the keyword (if not 100%)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                               |

<a id="ref-for-propdef-voice-rate①"></a>

The [voice-rate](#propdef-voice-rate) property manipulates the rate of generated synthetic speech in terms of words per minute.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`rate` attribute of the `prosody` element](https://www.w3.org/TR/speech-synthesis11/#edef_prosody) from the SSML markup language [\[SSML\]](#biblio-ssml), there are notable discrepancies. For example, CSS Speech rate keywords and percentage modifiers are not mutually-exclusive, due to how values are inherited and combined for selected elements.

<a id="valdef-voice-rate-normal"></a>normal

Represents the default rate produced by the speech synthesizer for the currently active voice. This is processor-specific and depends on the language and dialect, and on the "personality" of the voice.

<a id="valdef-voice-rate-x-slow"></a>x-slow, <a id="valdef-voice-rate-slow"></a>slow, <a id="valdef-voice-rate-medium"></a>medium, <a id="valdef-voice-rate-fast"></a>fast and <a id="valdef-voice-rate-x-fast"></a>x-fast

A sequence of monotonically non-decreasing speaking rates that are implementation- and voice-specific. For example, typical values for the English language are (in words per minute) x-slow = 80, slow = 120, medium = between 180 and 200, fast = 500.

<a id="ref-for-percentage-value①"></a>

<a id="valdef-voice-rate-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-percentage"></a>

Only non-negative [percentage](https://www.w3.org/TR/css-values-4/#percentage) values are allowed. This represents a change relative to the given keyword value (see enumeration above), or to the default value for the root element, or otherwise to the inherited speaking rate (which may itself be a combination of a keyword value and of a percentage, in which case percentages are combined multiplicatively). For example, 50% means that the speaking rate gets multiplied by 0.5 (half the value). Percentages above 100% result in faster speaking rates (relative to the base keyword), whereas percentages below 100% result in slower speaking rates.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0f17967a"></a>
>
> Examples of inherited values:
>
> ```text
> <body>
>   <e1>
>     <e2>
>       <e3>
>         ...
>       </e3>
>     </e2>
>   </e1>
> </body>
> 
> body { voice-rate: inherit; } /* the initial value is 'normal'
>                                  (the actual speaking rate value
>                                  depends on the active voice) */
> 
> e1 { voice-rate: +50%; } /* the computed value is
>                             ['normal' and 50%], which will resolve
>                             to the rate corresponding to 'normal'
>                             multiplied by 0.5 (half the speaking rate) */
> 
> e2 { voice-rate: fast 120%; } /* the computed value is
>                                  ['fast' and 120%], which will resolve
>                                  to the rate corresponding to 'fast'
>                                  multiplied by 1.2 */
> 
> e3 { voice-rate: normal; /* "resets" the speaking rate to the intrinsic voice value,
>                             the computed value is 'normal' (see comment below for actual value) */
>      voice-family: "another-voice"; } /* because the voice is different,
>                                          the calculated speaking rate may vary
>                                          compared to "body" (even though the computed
>                                          'voice-rate' value is the same) */
> ```
<a id="ref-for-propdef-voice-pitch①"></a>

### <a id="voice-props-voice-pitch"></a>11.3.  The [voice-pitch](#propdef-voice-pitch) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-pitch"></a>voice-pitch                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-percentage-value②"></a><a id="ref-for-typedef-voice-pitch-semitones"></a><a id="ref-for-comb-any④"></a><a id="ref-for-comb-one③⑤"></a><a id="ref-for-comb-all"></a><a id="ref-for-frequency-value"></a>[\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) absolute [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[\[x-low <a id="ref-for-comb-one③⑥"></a>\| low <a id="ref-for-comb-one③⑦"></a>\| medium <a id="ref-for-comb-one③⑧"></a>\| high <a id="ref-for-comb-one③⑨"></a>\| x-high\] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[<a id="ref-for-frequency-value①"></a>\<frequency\> <a id="ref-for-comb-one④⓪"></a>\| [\<semitones\>](#typedef-voice-pitch-semitones) <a id="ref-for-comb-one④①"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)\]\] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to inherited value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | one of the predefined pitch keywords if only the keyword is specified by itself, otherwise an absolute frequency calculated by converting the keyword value (if any) to a fixed frequency based on the current voice-family and by applying the specified relative offset (if any)                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |

<a id="ref-for-propdef-voice-pitch②"></a>

<a id="ref-for-propdef-voice-family⑧"></a>

The [voice-pitch](#propdef-voice-pitch) property specifies the "baseline" pitch of the generated speech output, which depends on the used [voice-family](#propdef-voice-family) instance, and varies across speech synthesis processors (it approximately corresponds to the average pitch of the output). For example, the common pitch for a male voice is around 120Hz, whereas it is around 210Hz for a female voice.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`pitch` attribute of the `prosody` element](https://www.w3.org/TR/speech-synthesis11/#edef_prosody) from the SSML markup language [\[SSML\]](#biblio-ssml), there are notable discrepancies. For example, CSS Speech pitch keywords and relative changes (frequency, semitone or percentage) are not mutually-exclusive, due to how values are inherited and combined for selected elements.

<a id="ref-for-frequency-value②"></a>

<a id="valdef-voice-pitch-frequency"></a>[\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value)

<a id="ref-for-valdef-voice-pitch-absolute"></a>

A value in frequency units (Hertz or kiloHertz, e.g. 100Hz, +2kHz). Values are restricted to positive numbers when the [absolute](#valdef-voice-pitch-absolute) keyword is specified. Otherwise (when the <a id="ref-for-valdef-voice-pitch-absolute①"></a>absolute keyword is not specified), a negative value represents a decrement, and a positive value represents an increment, relative to the inherited value. For example, 2kHz is a positive offset (strictly equivalent to +2kHz), and +2kHz absolute is an absolute frequency (strictly equivalent to 2kHz absolute).

<a id="valdef-voice-pitch-absolute"></a>absolute

If specified, this keyword indicates that the specified frequency represents an absolute value. If a negative frequency is specified, the computed frequency will be zero.

<a id="ref-for-typedef-voice-pitch-semitones①"></a>

<a id="valdef-voice-pitch-semitones"></a>[\<semitones\>](#typedef-voice-pitch-semitones)

<a id="ref-for-voice-pitch-semitone"></a>

<a id="ref-for-dimension①"></a>

<a id="ref-for-typedef-voice-pitch-semitones②"></a>

Specifies a relative change (decrement or increment) to the inherited value. The syntax of <a id="typedef-voice-pitch-semitones"></a>[\<semitones\>](#typedef-voice-pitch-semitones) allowed values is a [dimension](https://www.w3.org/TR/css-values-4/#dimension) with the unit identifier st (semitones). A <a id="voice-pitch-semitone"></a>semitone interval corresponds to the step between each note on an equal temperament chromatic scale. A [semitone](#voice-pitch-semitone) can therefore be quantified as the difference between two consecutive pitch frequencies on such scale. The ratio between two consecutive frequencies separated by exactly one <a id="ref-for-voice-pitch-semitone①"></a>semitone is the twelfth root of two (approximately 11011/10393, which equals exactly 1.0594631). As a result, the value in Hertz corresponding to a semitone offset is relative to the initial frequency the offset is applied to. (In other words, a <a id="ref-for-voice-pitch-semitone②"></a>semitone doesn’t correspond to a fixed numerical value in Hertz.)

<a id="ref-for-percentage-value③"></a>

<a id="valdef-voice-pitch-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-percentage①"></a>

Positive and negative [percentage](https://www.w3.org/TR/css-values-4/#percentage) values are allowed, to represent an increment or decrement (respectively) relative to the inherited value. Computed values are calculated by adding (or subtracting) the specified fraction of the inherited value, to (from) the inherited value. For example, 50% (which is equivalent to +50%) with a inherited value of 200Hz results in `200 + (200*0.5)` = 300Hz. Conversely, -50% results in `200-(200*0.5)` = 100Hz.

<a id="valdef-voice-pitch-x-low"></a>x-low, <a id="valdef-voice-pitch-low"></a>low, <a id="valdef-voice-pitch-medium"></a>medium, <a id="valdef-voice-pitch-high"></a>high, <a id="valdef-voice-pitch-x-high"></a>x-high

A sequence of monotonically non-decreasing pitch levels that are implementation and voice specific. When the computed value for a given element is only a keyword (i.e. no relative offset is specified), then the corresponding absolute frequency will be re-evaluated on a voice change. Conversely, the application of a relative offset requires the calculation of the resulting frequency based on the current voice at the point at which the relative offset is specified, so the computed frequency will inherit absolutely regardless of any voice change further down the style cascade. Authors should therefore only use keyword values in cases where they wish that voice changes trigger the re-evaluation of the conversion from a keyword to a concrete, voice-dependent frequency.

Computed absolute frequencies that are negative are clamped to zero Hertz. Speech-capable user agents are likely to support a specific range of values rather than the full range of possible calculated numerical values for frequencies. The actual values in user agents may therefore be clamped to implementation-dependent minimum and maximum boundaries. For example, although the 0Hz frequency can be legitimately calculated, it may be clamped to a more meaningful value in the context of the speech synthesizer.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0279cd20"></a>
>
> Examples of property values:
>
> ```text
> h1 { voice-pitch: 250Hz; } /* positive offset relative to the inherited absolute frequency */
> h1 { voice-pitch: +250Hz; } /* identical to the line above */
> h2 { voice-pitch: +30Hz absolute; } /* not an increment */
> h2 { voice-pitch: absolute 30Hz; } /* identical to the line above */
> h3 { voice-pitch: -20Hz; } /* negative offset (decrement) relative to the inherited absolute frequency */
> h4 { voice-pitch: -20Hz absolute; } /* illegal syntax => value ignored ("absolute" keyword not allowed with negative frequency) */
> h5 { voice-pitch: -3.5st; } /* semitones, negative offset */
> h6 { voice-pitch: 25%; } /* this means "add a quarter of the inherited value, to the inherited value" */
> h6 { voice-pitch: +25%; } /* identical to the line above */
> ```
<a id="ref-for-propdef-voice-range①"></a>

### <a id="voice-props-voice-range"></a>11.4.  The [voice-range](#propdef-voice-range) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-range"></a>voice-range                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-percentage-value④"></a><a id="ref-for-typedef-voice-pitch-semitones③"></a><a id="ref-for-comb-any⑤"></a><a id="ref-for-comb-one④②"></a><a id="ref-for-comb-all①"></a><a id="ref-for-frequency-value③"></a>[\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) absolute [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[\[x-low <a id="ref-for-comb-one④③"></a>\| low <a id="ref-for-comb-one④④"></a>\| medium <a id="ref-for-comb-one④⑤"></a>\| high <a id="ref-for-comb-one④⑥"></a>\| x-high\] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[<a id="ref-for-frequency-value④"></a>\<frequency\> <a id="ref-for-comb-one④⑦"></a>\| [\<semitones\>](#typedef-voice-pitch-semitones) <a id="ref-for-comb-one④⑧"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)\]\] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to inherited value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | one of the predefined pitch keywords if only the keyword is specified by itself, otherwise an absolute frequency calculated by converting the keyword value (if any) to a fixed frequency based on the current voice-family and by applying the specified relative offset (if any)                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |

<a id="ref-for-propdef-voice-range②"></a>

The [voice-range](#propdef-voice-range) property specifies the variability in the "baseline" pitch, i.e. how much the fundamental frequency may deviate from the average pitch of the speech output. The dynamic pitch range of the generated speech generally increases for a highly animated voice, for example when variations in inflection are used to convey meaning and emphasis in speech. Typically, a low range produces a flat, monotonic voice, whereas a high range produces an animated voice.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the functionality provided by this property is similar to the [`range` attribute of the `prosody` element](https://www.w3.org/TR/speech-synthesis11/#edef_prosody) from the SSML markup language [\[SSML\]](#biblio-ssml), there are notable discrepancies. For example, CSS Speech pitch range keywords and relative changes (frequency, semitone or percentage) are not mutually-exclusive, due to how values are inherited and combined for selected elements.

<a id="ref-for-frequency-value⑤"></a>

[\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value)

<a id="ref-for-valdef-voice-range-absolute"></a>

A value in frequency units (Hertz or kiloHertz, e.g. 100Hz, +2kHz). Values are restricted to positive numbers when the [absolute](#valdef-voice-range-absolute) keyword is specified. Otherwise (when the <a id="ref-for-valdef-voice-range-absolute①"></a>absolute keyword is not specified), a negative value represents a decrement, and a positive value represents an increment, relative to the inherited value. For example, 2kHz is a positive offset (strictly equivalent to +2kHz), and +2kHz absolute is an absolute frequency (strictly equivalent to 2kHz absolute).

<a id="valdef-voice-range-absolute"></a>absolute

If specified, this keyword indicates that the specified frequency represents an absolute value. If a negative frequency is specified, the computed frequency will be zero.

<a id="ref-for-typedef-voice-pitch-semitones④"></a>

<a id="valdef-voice-range-semitones"></a>[\<semitones\>](#typedef-voice-pitch-semitones)

<a id="ref-for-voice-pitch-semitone③"></a>

Specifies a relative change (decrement or increment) to the inherited value as a [semitone](#voice-pitch-semitone).

<a id="ref-for-percentage-value⑤"></a>

<a id="valdef-voice-range-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-percentage②"></a>

Positive and negative [percentage](https://www.w3.org/TR/css-values-4/#percentage) values represent an increment or decrement (respectively) relative to the inherited value. Computed values are calculated by adding (or subtracting) the specified fraction of the inherited value, to (from) the inherited value. For example, 50% (which is equivalent to +50%) with a inherited value of 200Hz results in `200 + (200*0.5)` = 300Hz. Conversely, -50% results in `200-(200*0.5)` = 100Hz.

<a id="valdef-voice-range-x-low"></a>x-low, <a id="valdef-voice-range-low"></a>low, <a id="valdef-voice-range-medium"></a>medium, <a id="valdef-voice-range-high"></a>high, <a id="valdef-voice-range-x-high"></a>x-high

A sequence of monotonically non-decreasing pitch levels that are implementation and voice specific. When the computed value for a given element is only a keyword (i.e. no relative offset is specified), then the corresponding absolute frequency will be re-evaluated on a voice change. Conversely, the application of a relative offset requires the calculation of the resulting frequency based on the current voice at the point at which the relative offset is specified, so the computed frequency will inherit absolutely regardless of any voice change further down the style cascade. Authors should therefore only use keyword values in cases where they wish that voice changes trigger the re-evaluation of the conversion from a keyword to a concrete, voice-dependent frequency.

Computed absolute frequencies that are negative are clamped to zero Hertz. Speech-capable user agents are likely to support a specific range of values rather than the full range of possible calculated numerical values for frequencies. The actual values in user agents may therefore be clamped to implementation-dependent minimum and maximum boundaries. For example: although the 0Hz frequency can be legitimately calculated, it may be clamped to a more meaningful value in the context of the speech synthesizer.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fdd5062c"></a>
>
> Examples of inherited values:
>
> ```text
> <body>
>   <e1>
>     <e2>
>       <e3>
>         <e4>
>           <e5>
>             <e6>
>               ...
>             </e6>
>           </e5>
>         </e4>
>       </e3>
>     </e2>
>   </e1>
> </body>
> 
> 
> 
> body { voice-range: inherit; } /* the initial value is 'medium'
>                                (the actual frequency value
>                                depends on the current voice) */
> 
> e1 { voice-range: +25%; } /* the computed value is
>                              ['medium' + 25%] which resolves
>                              to the frequency corresponding to 'medium'
>                              plus 0.25 times the frequency
>                              corresponding to 'medium' */
> 
> e2 { voice-range: +10Hz; } /* the computed value is
>                               [FREQ + 10Hz] where "FREQ" is the absolute frequency
>                               calculated in the "e1" rule above. */
> 
> e3 { voice-range: inherit; /* this could be omitted,
>                               but we explicitly specify it for clarity purposes */
> 
>      voice-family: "another-voice"; } /* this voice change would have resulted in
>                                          the re-evaluation of the initial 'medium' keyword
>                                          inherited by the "body" element
>                                          (i.e. conversion from a voice-dependent keyword value
>                                          to a concrete, absolute frequency),
>                                          but because relative offsets were applied down the style
>                                          cascade, the inherited value is actually the frequency
>                                          calculated at the "e2" rule above. */
> 
> e4 { voice-range: 200Hz absolute; } /* override with an absolute frequency
>                                        which doesn’t depend on the current voice */
> 
> e5 { voice-range: 2st; } /* the computed value is an absolute frequency,
>                             which is the result of the
>                             calculation: 200Hz + two semitones
>                             (reminder: the actual frequency corresponding to a semitone
>                             depends on the base value to which it applies) */
> 
> e6 { voice-range: inherit; /* this could be omitted,
>                               but we explicitly specify it for clarity purposes */
> 
>      voice-family: "yet-another-voice"; } /* despite the voice change,
>                                              the computed value is the same as
>                                              for "e5" (i.e. an absolute frequency value,
>                                              independent from the current voice) */
> ```
<a id="ref-for-propdef-voice-stress①"></a>

### <a id="voice-props-voice-stress"></a>11.5.  The [voice-stress](#propdef-voice-stress) property

| Field               | Definition                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-stress"></a>voice-stress                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④⑨"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) strong <a id="ref-for-comb-one⑤⓪"></a>\| moderate <a id="ref-for-comb-one⑤①"></a>\| none <a id="ref-for-comb-one⑤②"></a>\| reduced |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                 |

<a id="ref-for-propdef-voice-stress②"></a>

The [voice-stress](#propdef-voice-stress) property manipulates the strength of emphasis, which is normally applied using a combination of pitch change, timing changes, loudness and other acoustic differences. The precise meaning of the values therefore depend on the language being spoken.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The functionality provided by this property is similar to the [`emphasis` element](https://www.w3.org/TR/speech-synthesis11/#edef_emphasis) from the SSML markup language [\[SSML\]](#biblio-ssml).

<a id="valdef-voice-stress-normal"></a>normal  
Represents the default emphasis produced by the speech synthesizer.

<a id="valdef-voice-stress-none"></a>none  
Prevents the synthesizer from emphasizing text it would normally emphasize.

<a id="valdef-voice-stress-moderate"></a>moderate and <a id="valdef-voice-stress-strong"></a>strong  
<a id="ref-for-valdef-voice-stress-normal"></a>

These values are monotonically non-decreasing in strength. Their application results in more emphasis than what the speech synthesizer would normally produce (i.e. more than the value corresponding to [normal](#valdef-voice-stress-normal)).

<a id="valdef-voice-stress-reduced"></a>reduced  
Effectively the opposite of emphasizing a word.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4d45cfc2"></a>
>
> Examples of property values, with HTML sample:
>
> ```text
> .default-emphasis { voice-stress: normal; }
> .lowered-emphasis { voice-stress: reduced; }
> .removed-emphasis { voice-stress: none; }
> .normal-emphasis { voice-stress: moderate; }
> .huge-emphasis { voice-stress: strong; }
> 
> ...
> 
> <p>This is a big car.</p>
> <!-- The speech output from the line above is identical to the line below: -->
> <p>This is a <em class="default-emphasis">big</em> car.</p>
> 
> <p>This car is <em class="lowered-emphasis">massive</em>!</p>
> <!-- The "em" below is totally de-emphasized, whereas the emphasis in the line above is only reduced: -->
> <p>This car is <em class="removed-emphasis">massive</em>!</p>
> 
> <!-- The lines below demonstrate increasing levels of emphasis: -->
> <p>This is a <em class="normal-emphasis">big</em> car!</p>
> <p>This is a <em class="huge-emphasis">big</em> car!!!</p>
> ```
## <a id="duration-props"></a>12.  Voice duration property

<a id="ref-for-propdef-voice-duration②"></a>

### <a id="mixing-props-voice-duration"></a>12.1.  The [voice-duration](#propdef-voice-duration) property

| Field               | Definition                                                                          |
|---------------------|-------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-voice-duration"></a>voice-duration                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑤③"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) \<time\> |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                         |

<a id="ref-for-propdef-voice-duration③"></a>

<a id="ref-for-valdef-voice-duration-auto"></a>

<a id="ref-for-propdef-voice-rate②"></a>

<a id="ref-for-time-value③"></a>

The [voice-duration](#propdef-voice-duration) property specifies how long it should take to render the selected element’s content (not including [audio cues](#cue-props), [pauses](#pause-props) and [rests](#rest-props) ). Unless the value [auto](#valdef-voice-duration-auto) is specified, this property takes precedence over the [voice-rate](#propdef-voice-rate) property, and should be used to determine a suitable speaking rate for the voice. An element for which the <a id="ref-for-propdef-voice-duration④"></a>voice-duration property value is not <a id="ref-for-valdef-voice-duration-auto①"></a>auto can have descendants for which the <a id="ref-for-propdef-voice-duration⑤"></a>voice-duration and <a id="ref-for-propdef-voice-rate③"></a>voice-rate properties are specified, but these must be ignored. In other words, when a [\<time\>](https://www.w3.org/TR/css-values-4/#time-value) is specified for the <a id="ref-for-propdef-voice-duration⑥"></a>voice-duration of a selected element, it applies to the entire element subtree (children cannot override the property).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The functionality provided by this property is similar to the the [`duration` attribute of the `prosody` element](https://www.w3.org/TR/speech-synthesis11/#edef_prosody) from the SSML markup language [\[SSML\]](#biblio-ssml).

<a id="valdef-voice-duration-auto"></a>auto

<a id="ref-for-propdef-voice-rate④"></a>

Resolves to a used value corresponding to the duration of the speech synthesis when using the inherited [voice-rate](#propdef-voice-rate).

<a id="ref-for-time-value④"></a>

<a id="valdef-voice-duration-time"></a>[\<time\>](https://www.w3.org/TR/css-values-4/#time-value)

Specifies a value in absolute time units (seconds and milliseconds, e.g. "+3s", "250ms"). Only non-negative values are allowed.

## <a id="lists"></a>13.  List items and counters styles

<a id="ref-for-propdef-list-style-type"></a>

<a id="ref-for-propdef-content"></a>

<a id="ref-for-propdef-list-style-image"></a>

The [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type) property of [\[CSS2\]](#biblio-css2) specifies three types of list item markers: glyphs, numbering systems, and alphabetic systems. The values allowed for this property are also used for the counter() function of the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property. The CSS Speech module defines how to render these styles in the aural dimension, using speech synthesis. The [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image) property of \[CSS2\] is ignored, and instead the <a id="ref-for-propdef-list-style-type①"></a>list-style-type is used.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The speech rendering of new features from the CSS Lists and Counters Module Level 3 [\[CSS3LIST\]](#biblio-css3list) is not covered in this level of CSS Speech, but may be defined in a future specification.

disc, circle, square

For these list item styles, the user agent defines (possibly based on user preferences) what equivalent phrase is spoken or what audio cue is played. List items with graphical bullets are therefore announced appropriately in an implementation-dependent manner.

<a id="ref-for-armenian"></a>

<a id="ref-for-georgian"></a>

<a id="ref-for-upper-roman"></a>

<a id="ref-for-lower-roman"></a>

<a id="ref-for-decimal-leading-zero"></a>

<a id="ref-for-decimal"></a>

[decimal](https://www.w3.org/TR/css-counter-styles-3/#decimal), [decimal-leading-zero](https://www.w3.org/TR/css-counter-styles-3/#decimal-leading-zero), [lower-roman](https://www.w3.org/TR/css-counter-styles-3/#lower-roman), [upper-roman](https://www.w3.org/TR/css-counter-styles-3/#upper-roman), [georgian](https://www.w3.org/TR/css-counter-styles-3/#georgian), [armenian](https://www.w3.org/TR/css-counter-styles-3/#armenian)

For these list item styles, corresponding numbers are spoken as-is by the speech synthesizer, and may be complemented with additional audio cues or speech phrases in the document’s language (i.e. with the same TTS voice used to speak the list item content) in order to indicate the presence of list items. For example, when using the English language, the list item counter could be prefixed with the word "Item", which would result in list items being announced with "Item one", "Item two", etc.

<a id="ref-for-lower-greek"></a>

<a id="ref-for-upper-alpha"></a>

<a id="ref-for-upper-latin"></a>

<a id="ref-for-lower-alpha"></a>

<a id="ref-for-lower-latin"></a>

[lower-latin](https://www.w3.org/TR/css-counter-styles-3/#lower-latin), [lower-alpha](https://www.w3.org/TR/css-counter-styles-3/#lower-alpha), [upper-latin](https://www.w3.org/TR/css-counter-styles-3/#upper-latin), [upper-alpha](https://www.w3.org/TR/css-counter-styles-3/#upper-alpha), [lower-greek](https://www.w3.org/TR/css-counter-styles-3/#lower-greek)

<a id="ref-for-upper-latin①"></a>

<a id="ref-for-lower-greek①"></a>

These list item styles are spelled out letter-by-letter by the speech synthesizer, in the document language (i.e. with the same TTS voice used to speak the list item content). For example, [lower-greek](https://www.w3.org/TR/css-counter-styles-3/#lower-greek) in English would be read out as "alpha", "beta", "gamma", etc. Similarly, [upper-latin](https://www.w3.org/TR/css-counter-styles-3/#upper-latin) in French would be read out as /a/, /be/, /se/, etc. (phonetic notation)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is common for user agents such as screen readers to announce the nesting depth of list items, or more generally, to indicate additional structural information pertaining to complex hierarchical content. The verbosity of these additional audio cues and/or speech output can usually be controlled by users, and contribute to increasing usability. These navigation aids are implementation-dependent, but it is recommended that user agents supporting the CSS Speech module ensure that these additional audio cues and speech output don’t generate redundancies or create inconsistencies (for example: duplicated or different list item numbering scheme).

## <a id="content"></a>14.  Inserted and replaced content

<em>This section is non-normative.</em>

<a id="ref-for-propdef-content①"></a>

Sometimes, authors will want to specify a mapping from the source text into another string prior to the application of the regular pronunciation rules. This may be used for uncommon abbreviations or acronyms which are unlikely to be recognized by the synthesizer. The [content](https://www.w3.org/TR/css-content-3/#propdef-content) property can be used to replace one string by another. The functionality provided by this property is similar to the [`alias` attribute of the `sub` element](https://www.w3.org/TR/speech-synthesis11/#edef_sub) from the SSML markup language [\[SSML\]](#biblio-ssml).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ac939a61"></a> In this example, the abbreviation is rendered using the content of the title attribute instead of the element’s content.
>
> ```text
> /* This replaces the content of the selected element
> by the string "World Wide Web Consortium". */
> abbr { content: attr(title); }
> ...
> 
> <abbr title="World Wide Web Consortium">W3C</abbr>
> ```
In a similar way, text strings in a document can be replaced by a previously recorded version.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5056e9b"></a> In this example—assuming the format is supported, the file is available, and the UA is configured to do so—a recording of Sir John Gielgud’s declamation of the famous monologue is played. Otherwise the UA falls back to render the text using synthesized speech.
>
> ```text
> .hamlet { content: url(./audio/gielgud.wav); }
> ...
> 
> <div class="hamlet">
> To be, or not to be: that is the question:
> </div>
> ```
<a id="ref-for-selectordef-before①"></a>

<a id="ref-for-selectordef-after①"></a>

Furthermore, authors (or users via a user stylesheet) may add some information to ease the understanding of structures during non-visual interaction with the document. They can do so by using the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements. Note that different stylesheets can be used to define the level of verbosity for additional information spoken by screen readers.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5d2d6ce"></a>
>
> This example inserts the string "Start list: " before a list and the string "List item: " before the content of each list item. Likewise, the string "List end: " gets inserted after the list to inform the user that the list speech output is over.
>
> ```text
> ul::before { content: "Start list: "; }
> ul::after  { content: "List end. "; }
> li::before { content: "List item: "; }
> ```
Detailed information can be found in the CSS3 Generated and Replaced Content module [\[CSS3GENCON\]](#biblio-css3gencon).

## <a id="pronunciation"></a>15.  Pronunciation, phonemes 

<em>This section is non-normative.</em>

CSS does not specify how to define the pronunciation (expressed using a well-defined phonetic alphabet) of a particular piece of text within the markup document. A "phonemes" property was described in earlier drafts of this specification, but objections were raised due to breaking the principle of separation between content and presentation. (The "phonemes" authored within aural CSS stylesheets would have needed to be updated each time text changed within the markup document.) The "phonemes" functionality is therefore considered out-of-scope in CSS (the presentation layer) and should be addressed in the markup / content layer.

The ["pronunciation"](http://microformats.org/wiki/rel-pronunciation) `rel` value allows importing pronunciation lexicons in HTML documents using the `link` element (similar to how CSS stylesheets can be included). The W3C PLS (Pronunciation Lexicon Specification) [\[PRONUNCIATION-LEXICON\]](#biblio-pronunciation-lexicon) is one format that can be used to describe such a lexicon.

Additionally, an attribute-based mechanism can be used within the markup to author text-pronunciation associations At the time of writing, such mechanism isn’t formally defined in the W3C HTML standard(s). However, the [EPUB 3.0 specification](http://idpf.org/epub/30) allows (x)HTML5 documents to contain attributes derived from the [\[SSML\]](#biblio-ssml) specification, that describe how to pronounce text based on a particular phonetic alphabet.

### <a id="glossary"></a>Glossary

The following terms and abbreviations are used in this module.

<a id="ua"></a>UA  
<a id="user-agent"></a>user agent  
A program that reads and/or writes CSS style sheets on behalf of a user in either or both of these categories: programs whose purpose is to render documents (e.g., browsers) and programs whose purpose is to create style sheets (e.g., editors). A UA may fall into both categories. (There are other programs that read or write style sheets, but this module gives no rules for them.)

<a id="document"></a>document  
A tree-structured document with elements and attributes, such as an SGML or XML document [\[XML11\]](#biblio-xml11).

<a id="style-sheet"></a>style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet)

## <a id="ack"></a> Appendix D — Acknowledgements

The editors would like to thank the members of the W3C Voice Browser and Cascading Style Sheets working groups for their assistance in preparing this specification. Special thanks to Ellen Eide (IBM) for her detailed comments, and to Elika Etemad (Fantasai) for her thorough reviews.

## <a id="changes"></a> Appendix E — Changes

The following changes have been made since the [2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-speech-20120320/):

- <a id="ref-for-valdef-speak-always②"></a>

  <a id="ref-for-valdef-speak-never③"></a>

  <a id="ref-for-propdef-speak⑦"></a>

  Renamed the none and normal values of [speak](#propdef-speak) to [never](#valdef-speak-never) and [always](#valdef-speak-always) for clarity. See [Issue 510](https://github.com/w3c/csswg-drafts/issues/510).

- <a id="ref-for-propdef-visibility①"></a>

  <a id="ref-for-propdef-speak⑧"></a>

  <a id="ref-for-valdef-speak-auto③"></a>

  Made the [auto](#valdef-speak-auto) value of [speak](#propdef-speak) respond to [visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility). See [Issue 511](https://github.com/w3c/csswg-drafts/issues/511).

In addition there have been some minor editorial fixes. and the source has been converted to [Bikeshed](https://speced.github.io/bikeshed/) format.

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

### <a id="w3c-cr-exit-criteria"></a> CR exit criteria

For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:

independent  
each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.

interoperable  
passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.

implementation  
a user agent which:

1.  implements the specification.
2.  is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or "nightly build"). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
3.  is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).

The specification will remain Candidate Recommendation for at least six months.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- absolute
  - [value for voice-pitch](#valdef-voice-pitch-absolute), in § 11.3
  - [value for voice-range](#valdef-voice-range-absolute), in § 11.4
- [\<age\>](#typedef-voice-family-age), in § 11.1
- [always](#valdef-speak-always), in § 7.1
- [aural box model](#aural-box-model), in § 5
- auto
  - [value for speak](#valdef-speak-auto), in § 7.1
  - [value for voice-duration](#valdef-voice-duration-auto), in § 12.1
- [center](#valdef-voice-balance-center), in § 6.2
- [child](#valdef-voice-family-child), in § 11.1
- [cue](#propdef-cue), in § 10.3
- [cue-after](#propdef-cue-after), in § 10.1
- [cue-before](#propdef-cue-before), in § 10.1
- \<decibel\>
  - [type for voice-volume](#typedef-voice-volume-decibel), in § 6.1
  - [value for voice-volume](#valdef-voice-volume-decibel), in § 6.1
- [digits](#valdef-speak-as-digits), in § 7.2
- [document](#document), in § 15
- [\<family-name\>](#valdef-voice-family-family-name), in § 11.1
- [fast](#valdef-voice-rate-fast), in § 11.2
- [female](#valdef-voice-family-female), in § 11.1
- [\<frequency\>](#valdef-voice-pitch-frequency), in § 11.3
- [\<gender\>](#typedef-voice-family-gender), in § 11.1
- [\<generic-voice\>](#typedef-generic-voice), in § 11.1
- high
  - [value for voice-pitch](#valdef-voice-pitch-high), in § 11.3
  - [value for voice-range](#valdef-voice-range-high), in § 11.4
- [\<integer\>](#valdef-voice-family-integer), in § 11.1
- [left](#valdef-voice-balance-left), in § 6.2
- [leftwards](#valdef-voice-balance-leftwards), in § 6.2
- [literal-punctuation](#valdef-speak-as-literal-punctuation), in § 7.2
- [loud](#valdef-voice-volume-loud), in § 6.1
- low
  - [value for voice-pitch](#valdef-voice-pitch-low), in § 11.3
  - [value for voice-range](#valdef-voice-range-low), in § 11.4
- [male](#valdef-voice-family-male), in § 11.1
- medium
  - [value for pause-before, pause-after](#valdef-pause-before-medium), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-medium), in § 9.1
  - [value for voice-pitch](#valdef-voice-pitch-medium), in § 11.3
  - [value for voice-range](#valdef-voice-range-medium), in § 11.4
  - [value for voice-rate](#valdef-voice-rate-medium), in § 11.2
  - [value for voice-volume](#valdef-voice-volume-medium), in § 6.1
- [moderate](#valdef-voice-stress-moderate), in § 11.5
- [neutral](#valdef-voice-family-neutral), in § 11.1
- [never](#valdef-speak-never), in § 7.1
- none
  - [value for pause-before, pause-after](#valdef-pause-before-none), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-none), in § 9.1
  - [value for voice-stress](#valdef-voice-stress-none), in § 11.5
- [no-punctuation](#valdef-speak-as-no-punctuation), in § 7.2
- normal
  - [value for speak-as](#valdef-speak-as-normal), in § 7.2
  - [value for voice-rate](#valdef-voice-rate-normal), in § 11.2
  - [value for voice-stress](#valdef-voice-stress-normal), in § 11.5
- [\<number\>](#valdef-voice-balance-number), in § 6.2
- [old](#valdef-voice-family-old), in § 11.1
- [pause](#propdef-pause), in § 8.2
- [pause-after](#propdef-pause-after), in § 8.1
- [pause-before](#propdef-pause-before), in § 8.1
- \<percentage\>
  - [value for voice-pitch](#valdef-voice-pitch-percentage), in § 11.3
  - [value for voice-range](#valdef-voice-range-percentage), in § 11.4
  - [value for voice-rate](#valdef-voice-rate-percentage), in § 11.2
- [preserve](#valdef-voice-family-preserve), in § 11.1
- [reduced](#valdef-voice-stress-reduced), in § 11.5
- [rest](#propdef-rest), in § 9.2
- [rest-after](#propdef-rest-after), in § 9.1
- [rest-before](#propdef-rest-before), in § 9.1
- [right](#valdef-voice-balance-right), in § 6.2
- [rightwards](#valdef-voice-balance-rightwards), in § 6.2
- [semitone](#voice-pitch-semitone), in § 11.3
- \<semitones\>
  - [type for voice-pitch](#typedef-voice-pitch-semitones), in § 11.3
  - [value for voice-pitch](#valdef-voice-pitch-semitones), in § 11.3
  - [value for voice-range](#valdef-voice-range-semitones), in § 11.4
- [silent](#valdef-voice-volume-silent), in § 6.1
- [slow](#valdef-voice-rate-slow), in § 11.2
- [soft](#valdef-voice-volume-soft), in § 6.1
- [speak](#propdef-speak), in § 7.1
- [speak-as](#propdef-speak-as), in § 7.2
- [spell-out](#valdef-speak-as-spell-out), in § 7.2
- strong
  - [value for pause-before, pause-after](#valdef-pause-before-strong), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-strong), in § 9.1
  - [value for voice-stress](#valdef-voice-stress-strong), in § 11.5
- [style sheet](#style-sheet), in § 15
- \<time\>
  - [value for pause-before, pause-after](#valdef-pause-before-time), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-time), in § 9.1
  - [value for voice-duration](#valdef-voice-duration-time), in § 12.1
- [UA](#ua), in § 15
- [\<uri\>](#valdef-cue-before-uri), in § 10.1
- [user agent](#user-agent), in § 15
- [voice-balance](#propdef-voice-balance), in § 6.2
- [voice-duration](#propdef-voice-duration), in § 12.1
- [voice-family](#propdef-voice-family), in § 11.1
- [voice-pitch](#propdef-voice-pitch), in § 11.3
- [voice-range](#propdef-voice-range), in § 11.4
- [voice-rate](#propdef-voice-rate), in § 11.2
- [voice-stress](#propdef-voice-stress), in § 11.5
- [voice-volume](#propdef-voice-volume), in § 6.1
- weak
  - [value for pause-before, pause-after](#valdef-pause-before-weak), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-weak), in § 9.1
- [x-fast](#valdef-voice-rate-x-fast), in § 11.2
- x-high
  - [value for voice-pitch](#valdef-voice-pitch-x-high), in § 11.3
  - [value for voice-range](#valdef-voice-range-x-high), in § 11.4
- [x-loud](#valdef-voice-volume-x-loud), in § 6.1
- x-low
  - [value for voice-pitch](#valdef-voice-pitch-x-low), in § 11.3
  - [value for voice-range](#valdef-voice-range-x-low), in § 11.4
- [x-slow](#valdef-voice-rate-x-slow), in § 11.2
- [x-soft](#valdef-voice-volume-x-soft), in § 6.1
- x-strong
  - [value for pause-before, pause-after](#valdef-pause-before-x-strong), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-x-strong), in § 9.1
- x-weak
  - [value for pause-before, pause-after](#valdef-pause-before-x-weak), in § 8.1
  - [value for rest-before, rest-after](#valdef-rest-before-x-weak), in § 9.1
- [young](#valdef-voice-family-young), in § 11.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="term-for-propdef-border"></a>border
- \[CSS-BOX-4\] defines the following terms:
  - <a id="term-for-propdef-margin"></a>margin
  - <a id="term-for-propdef-padding"></a>padding
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="term-for-valdef-all-inherit"></a>inherit
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="term-for-armenian"></a>armenian
  - <a id="term-for-decimal"></a>decimal
  - <a id="term-for-decimal-leading-zero"></a>decimal-leading-zero
  - <a id="term-for-georgian"></a>georgian
  - <a id="term-for-lower-alpha"></a>lower-alpha
  - <a id="term-for-lower-greek"></a>lower-greek
  - <a id="term-for-lower-latin"></a>lower-latin
  - <a id="term-for-lower-roman"></a>lower-roman
  - <a id="term-for-upper-alpha"></a>upper-alpha
  - <a id="term-for-upper-latin"></a>upper-latin
  - <a id="term-for-upper-roman"></a>upper-roman
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-valdef-display-none"></a>none
  - <a id="term-for-propdef-visibility"></a>visibility
  - <a id="term-for-valdef-visibility-visible"></a>visible
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="term-for-family-name-value"></a>\<family-name\>
  - <a id="term-for-propdef-font-family"></a>font-family
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="term-for-propdef-left"></a>left
  - <a id="term-for-propdef-right"></a>right
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-mult-zero-plus"></a>\*
  - <a id="term-for-comb-comma"></a>,
  - <a id="term-for-frequency-value"></a>\<frequency\>
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
  - <a id="term-for-time-value"></a>\<time\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-css-identifier"></a>css identifier
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-dimension"></a>dimension
  - <a id="term-for-number"></a>number
  - <a id="term-for-percentage"></a>percentage
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[CSS22\] defines the following terms:
  - <a id="term-for-value-def-uri"></a>\<uri\>
  - <a id="term-for-valdef-media-all"></a>all
  - <a id="term-for-valdef-media-screen"></a>screen
  - <a id="term-for-valdef-media-speech"></a>speech
- \[CSS3GENCON\] defines the following terms:
  - <a id="term-for-propdef-content"></a>content
- \[CSS3LIST\] defines the following terms:
  - <a id="term-for-propdef-list-style-image"></a>list-style-image
  - <a id="term-for-propdef-list-style-type"></a>list-style-type
- \[HTML\] defines the following terms:
  - <a id="term-for-the-span-element"></a>span
- \[INFRA\] defines the following terms:
  - <a id="term-for-string"></a>string
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="term-for-valdef-media-aural"></a>aural

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 27 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 1 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 1 December 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 19 October 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-css3gencon"></a>\[CSS3GENCON\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css3list"></a>\[CSS3LIST\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-ssml"></a>\[SSML\]  
Daniel Burnett; Zhi Wei Shuang. [Speech Synthesis Markup Language (SSML) Version 1.1](https://www.w3.org/TR/speech-synthesis11/). 7 September 2010. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;speech-synthesis11&#x2F;](https://www.w3.org/TR/speech-synthesis11/)

<a id="biblio-xml11"></a>\[XML11\]  
Tim Bray; et al. [Extensible Markup Language (XML) 1.1 (Second Edition)](https://www.w3.org/TR/xml11/). 16 August 2006. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml11&#x2F;](https://www.w3.org/TR/xml11/)

### <a id="informative"></a>Informative References

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-pronunciation-lexicon"></a>\[PRONUNCIATION-LEXICON\]  
Paolo Baggia. [Pronunciation Lexicon Specification (PLS) Version 1.0](https://www.w3.org/TR/pronunciation-lexicon/). 14 October 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;pronunciation-lexicon&#x2F;](https://www.w3.org/TR/pronunciation-lexicon/)

<a id="biblio-ssml-sayas"></a>\[SSML-SAYAS\]  
Daniel Burnett; et al. [SSML 1.0 say-as attribute values](https://www.w3.org/TR/ssml-sayas/). 26 May 2005. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;ssml-sayas&#x2F;](https://www.w3.org/TR/ssml-sayas/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                                               | Initial                         | Applies to   | Inh. | %ages                    | Canonical order | Com­puted value                                                                                                                                                                                                                                                                     |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------|--------------|------|--------------------------|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-cue⑤"></a></span><a href="#propdef-cue">cue</a>&#xA;      </strong> | \<'cue-before'\> \<'cue-after'\>?                                                                                                                                   | N/A (see individual properties) | all elements | no   | N/A                      | per grammar     | N/A (see individual properties)                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-cue-after⑨"></a></span><a href="#propdef-cue-after">cue-after</a>&#xA;      </strong> | \<uri\> \<decibel\>? \| none                                                                                                                                        | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-cue-before⑨"></a></span><a href="#propdef-cue-before">cue-before</a>&#xA;      </strong> | \<uri\> \<decibel\>? \| none                                                                                                                                        | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-pause⑤"></a></span><a href="#propdef-pause">pause</a>&#xA;      </strong> | \<'pause-before'\> \<'pause-after'\>?                                                                                                                               | N/A (see individual properties) | all elements | no   | N/A                      | per grammar     | N/A (see individual properties)                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-pause-after⑨"></a></span><a href="#propdef-pause-after">pause-after</a>&#xA;      </strong> | \<time\> \| none \| x-weak \| weak \| medium \| strong \| x-strong                                                                                                  | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-pause-before⑨"></a></span><a href="#propdef-pause-before">pause-before</a>&#xA;      </strong> | \<time\> \| none \| x-weak \| weak \| medium \| strong \| x-strong                                                                                                  | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-rest⑤"></a></span><a href="#propdef-rest">rest</a>&#xA;      </strong> | \<'rest-before'\> \<'rest-after'\>?                                                                                                                                 | N/A (see individual properties) | all elements | no   | N/A                      | per grammar     | N/A (see individual properties)                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-rest-after⑦"></a></span><a href="#propdef-rest-after">rest-after</a>&#xA;      </strong> | \<time\> \| none \| x-weak \| weak \| medium \| strong \| x-strong                                                                                                  | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-rest-before⑦"></a></span><a href="#propdef-rest-before">rest-before</a>&#xA;      </strong> | \<time\> \| none \| x-weak \| weak \| medium \| strong \| x-strong                                                                                                  | none                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-speak⑨"></a></span><a href="#propdef-speak">speak</a>&#xA;      </strong> | auto \| never \| always                                                                                                                                             | auto                            | all elements | yes  | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-speak-as③"></a></span><a href="#propdef-speak-as">speak-as</a>&#xA;      </strong> | normal \| spell-out \|\| digits \|\| \[ literal-punctuation \| no-punctuation \]                                                                                    | normal                          | all elements | yes  | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-voice-balance①②"></a></span><a href="#propdef-voice-balance">voice-balance</a>&#xA;      </strong> | \<number\> \| left \| center \| right \| leftwards \| rightwards                                                                                                    | center                          | all elements | yes  | N/A                      | per grammar     | the specified value resolved to a \<number\> between -100 and 100 (inclusive)                                                                                                                                                                                                      |
| <strong><span><a id="ref-for-propdef-voice-duration⑦"></a></span><a href="#propdef-voice-duration">voice-duration</a>&#xA;      </strong> | auto \| \<time\>                                                                                                                                                    | auto                            | all elements | no   | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-voice-family⑨"></a></span><a href="#propdef-voice-family">voice-family</a>&#xA;      </strong> | \[\[\<family-name\> \| \<generic-voice\>\],\]\* \[\<family-name\> \| \<generic-voice\>\] \| preserve                                                                | implementation-dependent        | all elements | yes  | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-voice-pitch③"></a></span><a href="#propdef-voice-pitch">voice-pitch</a>&#xA;      </strong> | \<frequency\> &#x26;&#x26; absolute \| \[\[x-low \| low \| medium \| high \| x-high\] \|\| \[\<frequency\> \| \<semitones\> \| \<percentage\>\]\] | medium                          | all elements | yes  | refer to inherited value | per grammar     | one of the predefined pitch keywords if only the keyword is specified by itself, otherwise an absolute frequency calculated by converting the keyword value (if any) to a fixed frequency based on the current voice-family and by applying the specified relative offset (if any) |
| <strong><span><a id="ref-for-propdef-voice-range③"></a></span><a href="#propdef-voice-range">voice-range</a>&#xA;      </strong> | \<frequency\> &#x26;&#x26; absolute \| \[\[x-low \| low \| medium \| high \| x-high\] \|\| \[\<frequency\> \| \<semitones\> \| \<percentage\>\]\] | medium                          | all elements | yes  | refer to inherited value | per grammar     | one of the predefined pitch keywords if only the keyword is specified by itself, otherwise an absolute frequency calculated by converting the keyword value (if any) to a fixed frequency based on the current voice-family and by applying the specified relative offset (if any) |
| <strong><span><a id="ref-for-propdef-voice-rate⑤"></a></span><a href="#propdef-voice-rate">voice-rate</a>&#xA;      </strong> | \[normal \| x-slow \| slow \| medium \| fast \| x-fast\] \|\| \<percentage\>                                                                                        | normal                          | all elements | yes  | refer to default value   | per grammar     | a keyword value, and optionally also a percentage relative to the keyword (if not 100%)                                                                                                                                                                                            |
| <strong><span><a id="ref-for-propdef-voice-stress③"></a></span><a href="#propdef-voice-stress">voice-stress</a>&#xA;      </strong> | normal \| strong \| moderate \| none \| reduced                                                                                                                     | normal                          | all elements | yes  | N/A                      | per grammar     | specified value                                                                                                                                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-voice-volume①⑤"></a></span><a href="#propdef-voice-volume">voice-volume</a>&#xA;      </strong> | silent \| \[\[x-soft \| soft \| medium \| loud \| x-loud\] \|\| \<decibel\>\]                                                                                       | medium                          | all elements | yes  | N/A                      | per grammar     | silent, or a keyword value and optionally also a decibel offset (if not zero)                                                                                                                                                                                                      |

