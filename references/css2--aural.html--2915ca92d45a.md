Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Aural style sheets](https://www.w3.org/TR/2011/REC-CSS2-20110607/aural.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Aural style sheets

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/aural.html

Snapshot SHA-256: 2915ca92d45afca74c29838b4ff4c79db3bac7db858a8a6d1f6babc8954ab39c

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q19.0"></a>

# Appendix A. Aural style sheets

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

This chapter is informative. UAs are not required to implement the properties of this chapter in order to conform to CSS 2.1.

<a id="aural-media-group"></a>

## A.1 The media types 'aural' and 'speech'

We expect that in a future level of CSS there will be new properties and values defined for speech output. Therefore CSS 2.1 reserves the 'speech' media type (see [chapter 7, "Media types"](css2--media.html--3a324a170379.md)), but does not yet define which properties do or do not apply to it.

The properties in this appendix apply to a media type 'aural', that was introduced in CSS2. The type 'aural' is now deprecated.

> <strong data-conversion-semantic="note">Note</strong>
>
> This means that a style sheet such as
>
> ```text
> 
> @media speech {
>   body { voice-family: Paul }
> }
> ```
>
> is valid, but that its meaning is not defined by CSS 2.1, while
>
> ```text
> 
> @media aural {
>   body { voice-family: Paul }
> }
> ```
>
> is deprecated, but defined by this appendix.

<a id="aural-intro"></a>

## A.2 Introduction to aural style sheets

<a id="x0"></a>

<a id="x1"></a>

The aural rendering of a document, already commonly used by the blind and print-impaired communities, combines speech synthesis and "auditory icons." Often such aural presentation occurs by converting the document to plain text and feeding this to a screen reader -- software or hardware that simply reads all the characters on the screen. This results in less effective presentation than would be the case if the document structure were retained. Style sheet properties for aural presentation may be used together with visual properties (mixed media) or as an aural alternative to visual presentation.

Besides the obvious accessibility advantages, there are other large markets for listening to information, including in-car use, industrial and medical documentation systems (intranets), home entertainment, and to help users learning to read or who have difficulty reading.

<a id="x2"></a>

When using aural properties, the canvas consists of a three-dimensional physical space (sound surrounds) and a temporal space (one may specify sounds before, during, and after other sounds). The CSS properties also allow authors to vary the quality of synthesized speech (voice type, frequency, inflection, etc.).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1, h2, h3, h4, h5, h6 {
>     voice-family: paul;
>     stress: 20;
>     richness: 90;
>     cue-before: url("ping.au")
> }
> p.heidi { azimuth: center-left }
> p.peter { azimuth: right }
> p.goat  { volume: x-soft }
> ```
>
> This will direct the speech synthesizer to speak headers in a voice (a kind of "audio font") called "paul", on a flat tone, but in a very rich voice. Before speaking the headers, a sound sample will be played from the given URL. Paragraphs with class "heidi" will appear to come from front left (if the sound system is capable of spatial audio), and paragraphs of class "peter" from the right. Paragraphs with class "goat" will be very soft.

<a id="angles"></a>

### A.2.1 Angles

<a id="value-def-angle"></a>

<a id="x4"></a>

Angle values are denoted by \<angle\> in the text. Their format is a [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) immediately followed by an angle unit identifier.

Angle unit identifiers are:

- <strong>deg</strong>: degrees
- <strong>grad</strong>: grads
- <strong>rad</strong>: radians

Angle values may be negative. They should be normalized to the range 0-360deg by the user agent. For example, -10deg and 350deg are equivalent.

For example, a right angle is '90deg' or '100grad' or '1.570796326794897rad'.

Like for \<length\>, the unit may be omitted, if the value is zero: '0deg' may be written as '0'.

<a id="times"></a>

### A.2.2 Times

<a id="value-def-time"></a>

<a id="x6"></a>

Time values are denoted by \<time\> in the text. Their format is a [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) immediately followed by a time unit identifier.

Time unit identifiers are:

- <strong>ms</strong>: milliseconds
- <strong>s</strong>: seconds

Time values may not be negative.

Like for \<length\>, the unit may be omitted, if the value is zero: '0s' may be written as '0'.

<a id="frequencies"></a>

### A.2.3 Frequencies

<a id="value-def-frequency"></a>

<a id="x8"></a>

Frequency values are denoted by \<frequency\> in the text. Their format is a [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) immediately followed by a frequency unit identifier.

Frequency unit identifiers are:

- <strong>Hz</strong>: Hertz
- <strong>kHz</strong>: kilohertz

Frequency values may not be negative.

For example, 200Hz (or 200hz) is a bass sound, and 6kHz is a treble sound.

Like for \<length\>, the unit may be omitted, if the value is zero: '0Hz' may be written as '0'.

<a id="volume-props"></a>

## A.3 Volume properties: ['volume'](css2--aural.html--2915ca92d45a.md#propdef-volume)

<a id="propdef-volume"></a>

<strong>'volume'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| silent \| x-soft \| soft \| medium \| loud \| x-loud \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                                                                                                                                                                         |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                   |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                            |
| <em>Percentages:</em>   | refer to inherited value                                                                                                                                                                                                                                                                                                                       |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                                                                                             |
| <em>Computed value:</em>   | number                                                                                                                                                                                                                                                                                                                                         |

<a id="x10"></a>

Volume refers to the median volume of the waveform. In other words, a highly inflected voice at a volume of 50 might peak well above that. The overall values are likely to be human adjustable for comfort, for example with a physical volume control (which would increase both the 0 and 100 values proportionately); what this property does is adjust the dynamic range.

Values have the following meanings:

<a id="x11"></a>

[<strong>&lt;number&gt;</strong> ](css2--syndata.html--02e71c159e14.md#value-def-number)

Any number between '0' and '100'. '0' represents the <em>minimum audible</em> volume level and 100 corresponds to the <em>maximum comfortable</em> level.

<a id="x12"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Percentage values are calculated relative to the inherited value, and are then clipped to the range '0' to '100'.

<strong>silent</strong>

No sound at all. The value '0' does not mean the same as 'silent'.

<strong>x-soft</strong>

Same as '0'.

<strong>soft</strong>

Same as '25'.

<strong>medium</strong>

Same as '50'.

<strong>loud</strong>

Same as '75'.

<strong>x-loud</strong>

Same as '100'.

User agents should allow the values corresponding to '0' and '100' to be set by the listener. No one setting is universally applicable; suitable values depend on the equipment in use (speakers, headphones), the environment (in car, home theater, library) and personal preferences. Some examples:

- A browser for in-car use has a setting for when there is lots of background noise. '0' would map to a fairly high level and '100' to a quite high level. The speech is easily audible over the road noise but the overall dynamic range is compressed. Cars with better insulation might allow a wider dynamic range.
- Another speech browser is being used in an apartment, late at night, or in a shared study room. '0' is set to a very quiet level and '100' to a fairly quiet level, too. As with the first example, there is a low slope; the dynamic range is reduced. The actual volumes are low here, whereas they were high in the first example.
- In a quiet and isolated house, an expensive hi-fi home theater setup. '0' is set fairly low and '100' to quite high; there is wide dynamic range.

The same author style sheet could be used in all cases, simply by mapping the '0' and '100' points suitably at the client side.

<a id="speaking-props"></a>

## A.4 Speaking properties: ['speak'](css2--aural.html--2915ca92d45a.md#propdef-speak)

<a id="propdef-speak"></a>

<strong>'speak'</strong>

|                       |                                                                                                                       |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| none \| spell-out \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                          |
| <em>Inherited:</em>   | yes                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                   |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                    |
| <em>Computed value:</em>   | as specified                                                                                                          |

This property specifies whether text will be rendered aurally and if so, in what manner. The possible values are:

<strong>none</strong>  
Suppresses aural rendering so that the element requires no time to render. Note, however, that descendants may override this value and will be spoken. (To be sure to suppress rendering of an element and its descendants, use the ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) property).

<strong>normal</strong>  
Uses language-dependent pronunciation rules for rendering an element and its children.

<strong>spell-out</strong>  
Spells the text one letter at a time (useful for acronyms and abbreviations).

Note the difference between an element whose ['volume'](css2--aural.html--2915ca92d45a.md#propdef-volume) property has a value of 'silent' and an element whose ['speak'](css2--aural.html--2915ca92d45a.md#propdef-speak) property has the value 'none'. The former takes up the same time as if it had been spoken, including any pause before and after the element, but no sound is generated. The latter requires no time and is not rendered (though its descendants may be).

<a id="pause-props"></a>

## A.5 Pause properties: ['pause-before'](css2--aural.html--2915ca92d45a.md#propdef-pause-before), ['pause-after'](css2--aural.html--2915ca92d45a.md#propdef-pause-after), and ['pause'](css2--aural.html--2915ca92d45a.md#propdef-pause)

<a id="propdef-pause-before"></a>

<strong>'pause-before'</strong>

|                       |                                                                                                                                                                                                                                                                                  |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<time\>](css2--aural.html--2915ca92d45a.md#value-def-time) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                     |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                               |
| <em>Percentages:</em>   | see prose                                                                                                                                                                                                                                                                        |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                               |
| <em>Computed value:</em>   | time                                                                                                                                                                                                                                                                             |

<a id="propdef-pause-after"></a>

<strong>'pause-after'</strong>

|                       |                                                                                                                                                                                                                                                                                  |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<time\>](css2--aural.html--2915ca92d45a.md#value-def-time) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                     |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                               |
| <em>Percentages:</em>   | see prose                                                                                                                                                                                                                                                                        |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                               |
| <em>Computed value:</em>   | time;;                                                                                                                                                                                                                                                                           |

These properties specify a pause to be observed before (or after) speaking an element's content. Values have the following meanings:

> <strong data-conversion-semantic="note">Note</strong>
>
> <strong>Note.</strong> In CSS3 pauses are inserted around the cues and content rather than between them. See [\[CSS3SPEECH\]](css2--refs.html--f208d881d0b7.md#ref-CSS3SPEECH) for details.

<a id="x16"></a>

[<strong>&lt;time&gt;</strong>](css2--aural.html--2915ca92d45a.md#value-def-time)

Expresses the pause in absolute time units (seconds and milliseconds).

<a id="x17"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Refers to the inverse of the value of the ['speech-rate'](css2--aural.html--2915ca92d45a.md#propdef-speech-rate) property. For example, if the speech-rate is 120 words per minute (i.e., a word takes half a second, or 500ms) then a ['pause-before'](css2--aural.html--2915ca92d45a.md#propdef-pause-before) of 100% means a pause of 500 ms and a ['pause-before'](css2--aural.html--2915ca92d45a.md#propdef-pause-before) of 20% means 100ms.

The pause is inserted between the element's content and any ['cue-before'](css2--aural.html--2915ca92d45a.md#propdef-cue-before) or ['cue-after'](css2--aural.html--2915ca92d45a.md#propdef-cue-after) content.

Authors should use relative units to create more robust style sheets in the face of large changes in speech-rate.

<a id="propdef-pause"></a>

<strong>'pause'</strong>

|                       |                                                                                                                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ \[[\<time\>](css2--aural.html--2915ca92d45a.md#value-def-time) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage)\]{1,2} \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                              |
| <em>Percentages:</em>   | see descriptions of 'pause-before' and 'pause-after'                                                                                                                                                                                                                                            |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                                              |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                       |

The ['pause'](css2--aural.html--2915ca92d45a.md#propdef-pause) property is a shorthand for setting ['pause-before'](css2--aural.html--2915ca92d45a.md#propdef-pause-before) and ['pause-after'](css2--aural.html--2915ca92d45a.md#propdef-pause-after). If two values are given, the first value is ['pause-before'](css2--aural.html--2915ca92d45a.md#propdef-pause-before) and the second is ['pause-after'](css2--aural.html--2915ca92d45a.md#propdef-pause-after). If only one value is given, it applies to both properties.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { pause: 20ms } /* pause-before: 20ms; pause-after: 20ms */
> h2 { pause: 30ms 40ms } /* pause-before: 30ms; pause-after: 40ms */
> h3 { pause-after: 10ms } /* pause-before unspecified; pause-after: 10ms */
> ```
<a id="cue-props"></a>

## A.6 Cue properties: ['cue-before'](css2--aural.html--2915ca92d45a.md#propdef-cue-before), ['cue-after'](css2--aural.html--2915ca92d45a.md#propdef-cue-after), and ['cue'](css2--aural.html--2915ca92d45a.md#propdef-cue)

<a id="propdef-cue-before"></a>

<strong>'cue-before'</strong>

|                       |                                                                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                         |
| <em>Inherited:</em>   | no                                                                                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                  |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                   |
| <em>Computed value:</em>   | absolute URI or 'none'                                                                                                                                                               |

<a id="propdef-cue-after"></a>

<strong>'cue-after'</strong>

|                       |                                                                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                         |
| <em>Inherited:</em>   | no                                                                                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                  |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                   |
| <em>Computed value:</em>   | absolute URI or 'none'                                                                                                                                                               |

Auditory icons are another way to distinguish semantic elements. Sounds may be played before and/or after the element to delimit it. Values have the following meanings:

<a id="x21"></a>

[<strong>&lt;uri&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-uri)

The URI must designate an auditory icon resource. If the URI resolves to something other than an audio file, such as an image, the resource should be ignored and the property treated as if it had the value 'none'.

<strong>none</strong>

No auditory icon is specified.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> a {cue-before: url("bell.aiff"); cue-after: url("dong.wav") }
> h1 {cue-before: url("pop.au"); cue-after: url("pop.au") }
> ```
<a id="propdef-cue"></a>

<strong>'cue'</strong>

|                       |                                                                                                                                                                                                                                                                                                  |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ [\<'cue-before'\>](css2--aural.html--2915ca92d45a.md#propdef-cue-before) \|\| [\<'cue-after'\>](css2--aural.html--2915ca92d45a.md#propdef-cue-after) \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                        |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                     |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                               |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                              |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                                               |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                        |

The ['cue'](css2--aural.html--2915ca92d45a.md#propdef-cue) property is a shorthand for setting ['cue-before'](css2--aural.html--2915ca92d45a.md#propdef-cue-before) and ['cue-after'](css2--aural.html--2915ca92d45a.md#propdef-cue-after). If two values are given, the first value is ['cue-before'](css2--aural.html--2915ca92d45a.md#propdef-cue-before) and the second is ['cue-after'](css2--aural.html--2915ca92d45a.md#propdef-cue-after). If only one value is given, it applies to both properties.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following two rules are equivalent:
>
> ```text
> 
> h1 {cue-before: url("pop.au"); cue-after: url("pop.au") }
> h1 {cue: url("pop.au") }
> ```
If a user agent cannot render an auditory icon (e.g., the user's environment does not permit it), we recommend that it produce an alternative cue.

Please see the sections on [the :before and :after pseudo-elements](css2--generate.html--37748b674cd2.md#before-after-content) for information on other content generation techniques. 'Cue-before' sounds and 'pause-before' gaps are inserted before content from the ':before' pseudo-element. Similarly, 'pause-after' gaps and 'cue-after' sounds are inserted after content from the ':after' pseudo-element.

<a id="mixing-props"></a>

## A.7 Mixing properties: ['play-during'](css2--aural.html--2915ca92d45a.md#propdef-play-during)

<a id="propdef-play-during"></a>

<strong>'play-during'</strong>

|                       |                                                                                                                                                                                                                     |
|-----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) \[ mix \|\| repeat \]? \| auto \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | auto                                                                                                                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                        |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                  |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                 |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                  |
| <em>Computed value:</em>   | absolute URI, rest as specified                                                                                                                                                                                     |

Similar to the ['cue-before'](css2--aural.html--2915ca92d45a.md#propdef-cue-before) and ['cue-after'](css2--aural.html--2915ca92d45a.md#propdef-cue-after) properties, this property specifies a sound to be played as a background while an element's content is spoken. Values have the following meanings:

<a id="x24"></a>

[<strong>&lt;uri&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-uri)

<a id="x25"></a>

The sound designated by this [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) is played as a background while the element's content is spoken.

<strong>mix</strong>

<a id="x26"></a>

When present, this keyword means that the sound inherited from the parent element's ['play-during'](css2--aural.html--2915ca92d45a.md#propdef-play-during) property continues to play and the sound designated by the [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) is mixed with it. If 'mix' is not specified, the element's background sound replaces the parent's.

<strong>repeat</strong>

When present, this keyword means that the sound will repeat if it is too short to fill the entire duration of the element. Otherwise, the sound plays once and then stops. This is similar to the ['background-repeat'](css2--colors.html--5784063d2778.md#propdef-background-repeat) property. If the sound is too long for the element, it is clipped once the element has been spoken.

<strong>auto</strong>

The sound of the parent element continues to play (it is not restarted, which would have been the case if this property had been inherited).

<strong>none</strong>

This keyword means that there is silence. The sound of the parent element (if any) is silent during the current element and continues after the current element.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> blockquote.sad { play-during: url("violins.aiff") }
> blockquote Q   { play-during: url("harp.wav") mix }
> span.quiet     { play-during: none }
> ```
<a id="spatial-props"></a>

## A.8 Spatial properties: ['azimuth'](css2--aural.html--2915ca92d45a.md#propdef-azimuth) and ['elevation'](css2--aural.html--2915ca92d45a.md#propdef-elevation)

Spatial audio is an important stylistic property for aural presentation. It provides a natural way to tell several voices apart, as in real life (people rarely all stand in the same spot in a room). Stereo speakers produce a lateral sound stage. Binaural headphones or the increasingly popular 5-speaker home theater setups can generate full surround sound, and multi-speaker setups can create a true three-dimensional sound stage. VRML 2.0 also includes spatial audio, which implies that in time consumer-priced spatial audio hardware will become more widely available.

<a id="propdef-azimuth"></a>

<strong>'azimuth'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<angle\>](css2--aural.html--2915ca92d45a.md#value-def-angle) \| \[\[ left-side \| far-left \| left \| center-left \| center \| center-right \| right \| far-right \| right-side \] \|\| behind \] \| leftwards \| rightwards \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | center                                                                                                                                                                                                                                                                                                                                         |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                   |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                            |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                            |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                                                                                             |
| <em>Computed value:</em>   | normalized angle                                                                                                                                                                                                                                                                                                                               |

Values have the following meanings:

<a id="x28"></a>

[<strong>&lt;angle&gt;</strong>](css2--aural.html--2915ca92d45a.md#value-def-angle)

Position is described in terms of an angle within the range '-360deg' to '360deg'. The value '0deg' means directly ahead in the center of the sound stage. '90deg' is to the right, '180deg' behind, and '270deg' (or, equivalently and more conveniently, '-90deg') to the left.

<strong>left-side</strong>

Same as '270deg'. With 'behind', '270deg'.

<strong>far-left</strong>

Same as '300deg'. With 'behind', '240deg'.

<strong>left</strong>

Same as '320deg'. With 'behind', '220deg'.

<strong>center-left</strong>

Same as '340deg'. With 'behind', '200deg'.

<strong>center</strong>

Same as '0deg'. With 'behind', '180deg'.

<strong>center-right</strong>

Same as '20deg'. With 'behind', '160deg'.

<strong>right</strong>

Same as '40deg'. With 'behind', '140deg'.

<strong>far-right</strong>

Same as '60deg'. With 'behind', '120deg'.

<strong>right-side</strong>

Same as '90deg'. With 'behind', '90deg'.

<strong>leftwards</strong>

Moves the sound to the left, relative to the current angle. More precisely, subtracts 20 degrees. Arithmetic is carried out modulo 360 degrees. Note that 'leftwards' is more accurately described as "turned counter-clockwise," since it <em>always</em> subtracts 20 degrees, even if the inherited azimuth is already behind the listener (in which case the sound actually appears to move to the right).

<strong>rightwards</strong>

Moves the sound to the right, relative to the current angle. More precisely, adds 20 degrees. See 'leftwards' for arithmetic.

This property is most likely to be implemented by mixing the same signal into different channels at differing volumes. It might also use phase shifting, digital delay, and other such techniques to provide the illusion of a sound stage. The precise means used to achieve this effect and the number of speakers used to do so are user agent-dependent; this property merely identifies the desired end result.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1   { azimuth: 30deg }
> td.a { azimuth: far-right }          /*  60deg */
> #12  { azimuth: behind far-right }   /* 120deg */
> p.comment { azimuth: behind }        /* 180deg */
> ```
If spatial-azimuth is specified and the output device cannot produce sounds <em>behind</em> the listening position, user agents should convert values in the rearwards hemisphere to forwards hemisphere values. One method is as follows:

- if 90deg \< x \<= 180deg then x := 180deg - x
- if 180deg \< x \<= 270deg then x := 540deg - x

<a id="propdef-elevation"></a>

<strong>'elevation'</strong>

|                       |                                                                                                                                                                                                                              |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<angle\>](css2--aural.html--2915ca92d45a.md#value-def-angle) \| below \| level \| above \| higher \| lower \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | level                                                                                                                                                                                                                        |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                 |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                          |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                           |
| <em>Computed value:</em>   | normalized angle                                                                                                                                                                                                             |

Values of this property have the following meanings:

<a id="x30"></a>

[<strong>&lt;angle&gt;</strong>](css2--aural.html--2915ca92d45a.md#value-def-angle)

Specifies the elevation as an angle, between '-90deg' and '90deg'. '0deg' means on the forward horizon, which loosely means level with the listener. '90deg' means directly overhead and '-90deg' means directly below.

<strong>below</strong>

Same as '-90deg'.

<strong>level</strong>

Same as '0deg'.

<strong>above</strong>

Same as '90deg'.

<strong>higher</strong>

Adds 10 degrees to the current elevation.

<strong>lower</strong>

Subtracts 10 degrees from the current elevation.

The precise means used to achieve this effect and the number of speakers used to do so are undefined. This property merely identifies the desired end result.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1   { elevation: above }
> tr.a { elevation: 60deg }
> tr.b { elevation: 30deg }
> tr.c { elevation: level }
> ```
<a id="voice-char-props"></a>

## A.9 Voice characteristic properties: ['speech-rate'](css2--aural.html--2915ca92d45a.md#propdef-speech-rate), ['voice-family'](css2--aural.html--2915ca92d45a.md#propdef-voice-family), ['pitch'](css2--aural.html--2915ca92d45a.md#propdef-pitch), ['pitch-range'](css2--aural.html--2915ca92d45a.md#propdef-pitch-range), ['stress'](css2--aural.html--2915ca92d45a.md#propdef-stress), and ['richness'](css2--aural.html--2915ca92d45a.md#propdef-richness)

<a id="propdef-speech-rate"></a>

<strong>'speech-rate'</strong>

|                       |                                                                                                                                                                                                                                                      |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| x-slow \| slow \| medium \| fast \| x-fast \| faster \| slower \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                                                                               |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                         |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                  |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                  |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                   |
| <em>Computed value:</em>   | number                                                                                                                                                                                                                                               |

This property specifies the speaking rate. Note that both absolute and relative keyword values are allowed (compare with ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size)). Values have the following meanings:

<a id="x32"></a>

[<strong>&lt;number&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-number)

Specifies the speaking rate in words per minute, a quantity that varies somewhat by language but is nevertheless widely supported by speech synthesizers.

<strong>x-slow</strong>

Same as 80 words per minute.

<strong>slow</strong>

Same as 120 words per minute

<strong>medium</strong>

Same as 180 - 200 words per minute.

<strong>fast</strong>

Same as 300 words per minute.

<strong>x-fast</strong>

Same as 500 words per minute.

<strong>faster</strong>

Adds 40 words per minute to the current speech rate.

<strong>slower</strong>

Subtracts 40 words per minutes from the current speech rate.

<a id="propdef-voice-family"></a>

<strong>'voice-family'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[\[[\<specific-voice\>](css2--aural.html--2915ca92d45a.md#value-def-specific-voice) \| [\<generic-voice\>](css2--aural.html--2915ca92d45a.md#value-def-generic-voice) \],\]\* \[[\<specific-voice\>](css2--aural.html--2915ca92d45a.md#value-def-specific-voice) \| [\<generic-voice\>](css2--aural.html--2915ca92d45a.md#value-def-generic-voice) \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | depends on user agent                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <em>Computed value:</em>   | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |

The value is a comma-separated, prioritized list of voice family names (compare with ['font-family'](css2--fonts.html--d52fc14f36c2.md#propdef-font-family)). Values have the following meanings:

<a id="value-def-generic-voice"></a>

<strong>&lt;generic-voice&gt;</strong>

Values are voice families. Possible values are 'male', 'female', and 'child'.

<a id="value-def-specific-voice"></a>

<strong>&lt;specific-voice&gt;</strong>

Values are specific instances (e.g., comedian, trinoids, carlos, lani).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { voice-family: announcer, male }
> p.part.romeo  { voice-family: romeo, male }
> p.part.juliet { voice-family: juliet, female }
> ```
Names of specific voices may be quoted, and indeed must be quoted if any of the words that make up the name does not conform to the syntax rules for [identifiers](css2--syndata.html--02e71c159e14.md#tokenization). It is also recommended to quote specific voices with a name consisting of more than one word. If quoting is omitted, any [white space](css2--syndata.html--02e71c159e14.md#whitespace) characters before and after the voice family name are ignored and any sequence of white space characters inside the voice family name is converted to a single space.

<a id="propdef-pitch"></a>

<strong>'pitch'</strong>

|                       |                                                                                                                                                                                                                                    |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<frequency\>](css2--aural.html--2915ca92d45a.md#value-def-frequency) \| x-low \| low \| medium \| high \| x-high \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                                                             |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                       |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                                                                 |
| <em>Computed value:</em>   | frequency                                                                                                                                                                                                                          |

Specifies the average pitch (a frequency) of the speaking voice. The average pitch of a voice depends on the voice family. For example, the average pitch for a standard male voice is around 120Hz, but for a female voice, it's around 210Hz.

Values have the following meanings:

<a id="x37"></a>

[<strong>&lt;frequency&gt;</strong>](css2--aural.html--2915ca92d45a.md#value-def-frequency)

Specifies the average pitch of the speaking voice in hertz (Hz).

<strong>x-low</strong>, <strong>low</strong>, <strong>medium</strong>, <strong>high</strong>, <strong>x-high</strong>

These values do not map to absolute frequencies since these values depend on the voice family. User agents should map these values to appropriate frequencies based on the voice family and user environment. However, user agents must map these values in order (i.e., 'x-low' is a lower frequency than 'low', etc.).

<a id="propdef-pitch-range"></a>

<strong>'pitch-range'</strong>

|                       |                                                                                                                                                                                    |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 50                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                       |
| <em>Inherited:</em>   | yes                                                                                                                                                                                |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                 |
| <em>Computed value:</em>   | as specified                                                                                                                                                                       |

Specifies variation in average pitch. The perceived pitch of a human voice is determined by the fundamental frequency and typically has a value of 120Hz for a male voice and 210Hz for a female voice. Human languages are spoken with varying inflection and pitch; these variations convey additional meaning and emphasis. Thus, a highly animated voice, i.e., one that is heavily inflected, displays a high pitch range. This property specifies the range over which these variations occur, i.e., how much the fundamental frequency may deviate from the average pitch.

Values have the following meanings:

<a id="x39"></a>

[<strong>&lt;number&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-number)

A value between '0' and '100'. A pitch range of '0' produces a flat, monotonic voice. A pitch range of 50 produces normal inflection. Pitch ranges greater than 50 produce animated voices.

<a id="propdef-stress"></a>

<strong>'stress'</strong>

|                       |                                                                                                                                                                                    |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 50                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                       |
| <em>Inherited:</em>   | yes                                                                                                                                                                                |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                 |
| <em>Computed value:</em>   | as specified                                                                                                                                                                       |

Specifies the height of "local peaks" in the intonation contour of a voice. For example, English is a <strong>stressed</strong> language, and different parts of a sentence are assigned primary, secondary, or tertiary stress. The value of ['stress'](css2--aural.html--2915ca92d45a.md#propdef-stress) controls the amount of inflection that results from these stress markers. This property is a companion to the ['pitch-range'](css2--aural.html--2915ca92d45a.md#propdef-pitch-range) property and is provided to allow developers to exploit higher-end auditory displays.

Values have the following meanings:

<a id="x41"></a>

[<strong>&lt;number&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-number)

A value, between '0' and '100'. The meaning of values depends on the language being spoken. For example, a level of '50' for a standard, English-speaking male voice (average pitch = 122Hz), speaking with normal intonation and emphasis would have a different meaning than '50' for an Italian voice.

<a id="propdef-richness"></a>

<strong>'richness'</strong>

|                       |                                                                                                                                                                                    |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 50                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                       |
| <em>Inherited:</em>   | yes                                                                                                                                                                                |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                                                                                                 |
| <em>Computed value:</em>   | as specified                                                                                                                                                                       |

Specifies the richness, or brightness, of the speaking voice. A rich voice will "carry" in a large room, a smooth voice will not. (The term "smooth" refers to how the wave form looks when drawn.)

Values have the following meanings:

<a id="x43"></a>

[<strong>&lt;number&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-number)

A value between '0' and '100'. The higher the value, the more the voice will carry. A lower value will produce a soft, mellifluous voice.

<a id="speech-props"></a>

## A.10 Speech properties: ['speak-punctuation'](css2--aural.html--2915ca92d45a.md#propdef-speak-punctuation) and ['speak-numeral'](css2--aural.html--2915ca92d45a.md#propdef-speak-numeral)

An additional speech property, ['speak-header'](css2--aural.html--2915ca92d45a.md#propdef-speak-header), is described below.

<a id="propdef-speak-punctuation"></a>

<strong>'speak-punctuation'</strong>

|                       |                                                                                                        |
|-----------------------|--------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | code \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                   |
| <em>Applies to:</em>   | all elements                                                                                           |
| <em>Inherited:</em>   | yes                                                                                                    |
| <em>Percentages:</em>   | N/A                                                                                                    |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                     |
| <em>Computed value:</em>   | as specified                                                                                           |

This property specifies how punctuation is spoken. Values have the following meanings:

<strong>code</strong>  
Punctuation such as semicolons, braces, and so on are to be spoken literally.

<strong>none</strong>  
Punctuation is not to be spoken, but instead rendered naturally as various pauses.

<a id="propdef-speak-numeral"></a>

<strong>'speak-numeral'</strong>

|                       |                                                                                                                |
|-----------------------|----------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | digits \| continuous \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | continuous                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                   |
| <em>Inherited:</em>   | yes                                                                                                            |
| <em>Percentages:</em>   | N/A                                                                                                            |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                             |
| <em>Computed value:</em>   | as specified                                                                                                   |

This property controls how numerals are spoken. Values have the following meanings:

<strong>digits</strong>  
Speak the numeral as individual digits. Thus, "237" is spoken "Two Three Seven".

<strong>continuous</strong>  
Speak the numeral as a full number. Thus, "237" is spoken "Two hundred thirty seven". Word representations are language-dependent.

<a id="aural-tables"></a>

## A.11 Audio rendering of tables

When a table is spoken by a speech generator, the relation between the data cells and the header cells must be expressed in a different way than by horizontal and vertical alignment. Some speech browsers may allow a user to move around in the 2-dimensional space, thus giving them the opportunity to map out the spatially represented relations. When that is not possible, the style sheet must specify at which points the headers are spoken.

<a id="speak-headers"></a>

### A.11.1 Speaking headers: the ['speak-header'](css2--aural.html--2915ca92d45a.md#propdef-speak-header) property

<a id="propdef-speak-header"></a>

<strong>'speak-header'</strong>

|                       |                                                                                                          |
|-----------------------|----------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | once \| always \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | once                                                                                                     |
| <em>Applies to:</em>   | elements that have table header information                                                              |
| <em>Inherited:</em>   | yes                                                                                                      |
| <em>Percentages:</em>   | N/A                                                                                                      |
| <em>Media:</em>   | [aural](css2--aural.html--2915ca92d45a.md#aural-media-group)                       |
| <em>Computed value:</em>   | as specified                                                                                             |

This property specifies whether table headers are spoken before every cell, or only before a cell when that cell is associated with a different header than the previous cell. Values have the following meanings:

<strong>once</strong>  
The header is spoken one time, before a series of cells.

<strong>always</strong>  
The header is spoken before every pertinent cell.

Each document language may have different mechanisms that allow authors to specify headers. For example, in HTML 4 ([\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4)), it is possible to specify header information with three different attributes ("headers", "scope", and "axis"), and the specification gives an algorithm for determining header information when these attributes have not been specified.

<a id="img-table1"></a>

![Image of a table created in MS Word](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/table1.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/table1-desc.html)

Image of a table with header cells ("San Jose" and "Seattle") that are not in the same column or row as the data they apply to.

This HTML example presents the money spent on meals, hotels and transport in two locations (San Jose and Seattle) for successive days. Conceptually, you can think of the table in terms of an n-dimensional space. The headers of this space are: location, day, category and subtotal. Some cells define marks along an axis while others give money spent at points within this space. The markup for this table is:

```text

<TABLE>
<CAPTION>Travel Expense Report</CAPTION>
<TR>
  <TH></TH>
  <TH>Meals</TH>
  <TH>Hotels</TH>
  <TH>Transport</TH>
  <TH>subtotal</TH>
</TR>
<TR>
  <TH id="san-jose" axis="san-jose">San Jose</TH>
</TR>
<TR>
  <TH headers="san-jose">25-Aug-97</TH>
  <TD>37.74</TD>
  <TD>112.00</TD>
  <TD>45.00</TD>
  <TD></TD>
</TR>
<TR>
  <TH headers="san-jose">26-Aug-97</TH>
  <TD>27.28</TD>
  <TD>112.00</TD>
  <TD>45.00</TD>
  <TD></TD>
</TR>
<TR>
  <TH headers="san-jose">subtotal</TH>
  <TD>65.02</TD>
  <TD>224.00</TD>
  <TD>90.00</TD>
  <TD>379.02</TD>
</TR>
<TR>
  <TH id="seattle" axis="seattle">Seattle</TH>
</TR>
<TR>
  <TH headers="seattle">27-Aug-97</TH>
  <TD>96.25</TD>
  <TD>109.00</TD>
  <TD>36.00</TD>
  <TD></TD>
</TR>
<TR>
  <TH headers="seattle">28-Aug-97</TH>
  <TD>35.00</TD>
  <TD>109.00</TD>
  <TD>36.00</TD>
  <TD></TD>
</TR>
<TR>
  <TH headers="seattle">subtotal</TH>
  <TD>131.25</TD>
  <TD>218.00</TD>
  <TD>72.00</TD>
  <TD>421.25</TD>
</TR>
<TR>
  <TH>Totals</TH>
  <TD>196.27</TD>
  <TD>442.00</TD>
  <TD>162.00</TD>
  <TD>800.27</TD>
</TR>
</TABLE>
```
By providing the data model in this way, authors make it possible for speech enabled-browsers to explore the table in rich ways, e.g., each cell could be spoken as a list, repeating the applicable headers before each data cell:

```text

  San Jose, 25-Aug-97, Meals:  37.74
  San Jose, 25-Aug-97, Hotels:  112.00
  San Jose, 25-Aug-97, Transport:  45.00
 ...
```
The browser could also speak the headers only when they change:

```text

San Jose, 25-Aug-97, Meals: 37.74
    Hotels: 112.00
    Transport: 45.00
  26-Aug-97, Meals: 27.28
    Hotels: 112.00
...
```
<a id="sample"></a>

## A.12 Sample style sheet for HTML

This style sheet describes a possible rendering of HTML 4:

```text

@media aural {
h1, h2, h3, 
h4, h5, h6    { voice-family: paul, male; stress: 20; richness: 90 }
h1            { pitch: x-low; pitch-range: 90 }
h2            { pitch: x-low; pitch-range: 80 }
h3            { pitch: low; pitch-range: 70 }
h4            { pitch: medium; pitch-range: 60 }
h5            { pitch: medium; pitch-range: 50 }
h6            { pitch: medium; pitch-range: 40 }
li, dt, dd    { pitch: medium; richness: 60 }
dt            { stress: 80 }
pre, code, tt { pitch: medium; pitch-range: 0; stress: 0; richness: 80 }
em            { pitch: medium; pitch-range: 60; stress: 60; richness: 50 }
strong        { pitch: medium; pitch-range: 60; stress: 90; richness: 90 }
dfn           { pitch: high; pitch-range: 60; stress: 60 }
s, strike     { richness: 0 }
i             { pitch: medium; pitch-range: 60; stress: 60; richness: 50 }
b             { pitch: medium; pitch-range: 60; stress: 90; richness: 90 }
u             { richness: 0 }
a:link        { voice-family: harry, male }
a:visited     { voice-family: betty, female }
a:active      { voice-family: betty, female; pitch-range: 80; pitch: x-high }
}
```
<a id="Emacspeak"></a>

## A.13 Emacspeak

For information, here is the list of properties implemented by Emacspeak, a speech subsystem for the Emacs editor.

- voice-family
- stress (but with a different range of values)
- richness (but with a different range of values)
- pitch (but with differently named values)
- pitch-range (but with a different range of values)

(We thank T. V. Raman for the information about implementation status of aural properties.)
