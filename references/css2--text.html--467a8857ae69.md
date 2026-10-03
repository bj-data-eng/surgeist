Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Text](https://www.w3.org/TR/2011/REC-CSS2-20110607/text.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Text

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/text.html

Snapshot SHA-256: 467a8857ae69f2b325394ea400d14bd1280f85956c0a2e7deabc39e612b1380f

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q16.0"></a>

# 16 Text

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

The properties defined in the following sections affect the visual presentation of characters, spaces, words, and paragraphs.

<a id="indentation-prop"></a>

## 16.1 Indentation: the ['text-indent'](css2--text.html--467a8857ae69.md#propdef-text-indent) property

<a id="propdef-text-indent"></a>

<strong>'text-indent'</strong>

|                       |                                                                                                                                                                                                                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                                      |
| <em>Applies to:</em>   | block containers                                                                                                                                                                                                                                                                       |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                    |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                                                                                                                     |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                   |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                                                                                                                     |

This property specifies the indentation of the first line of text in a block container. More precisely, it specifies the indentation of the first box that flows into the block's first [line box](css2--visuren.html--3f334c530cf4.md#line-box). The box is indented with respect to the left (or right, for right-to-left layout) edge of the line box. User agents must render this indentation as blank space.

'Text-indent' only affects a line if it is the [first formatted line](css2--selector.html--f66f7c932788.md#first-line-pseudo) of an element. For example, the first line of an anonymous block box is only affected if it is the first child of its parent element.

Values have the following meanings:

<a id="x1"></a>

[\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length)

The indentation is a fixed length.

<a id="x2"></a>

[\<percentage\> ](css2--syndata.html--02e71c159e14.md#value-def-percentage)

The indentation is a percentage of the containing block width.

The value of ['text-indent'](css2--text.html--467a8857ae69.md#propdef-text-indent) may be negative, but there may be implementation-specific limits. If the value of ['text-indent'](css2--text.html--467a8857ae69.md#propdef-text-indent) is either negative or exceeds the width of the block, that <em>first box</em>, described above, can overflow the block. The value of ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) will affect whether such text that overflows the block is visible.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following example causes a '3em' text indent.
>
> ```text
> 
> p { text-indent: 3em }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the 'text-indent' property inherits, when specified on a block element, it will affect descendant inline-block elements. For this reason, it is often wise to specify '`text-indent: 0`' on elements that are specified '`display:inline-block`'.

<a id="alignment-prop"></a>

## 16.2 Alignment: the ['text-align'](css2--text.html--467a8857ae69.md#propdef-text-align) property

<a id="propdef-text-align"></a>

<strong>'text-align'</strong>

|                       |                                                                                                                              |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | left \| right \| center \| justify \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | a nameless value that acts as 'left' if 'direction' is 'ltr', 'right' if 'direction' is 'rtl'                                |
| <em>Applies to:</em>   | block containers                                                                                                             |
| <em>Inherited:</em>   | yes                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                         |
| <em>Computed value:</em>   | the initial value or as specified                                                                                            |

This property describes how inline-level content of a block container is aligned. Values have the following meanings:

left, right, center, justify  
Left, right, center, and justify text, respectively, as described in [the section on inline formatting](css2--visuren.html--3f334c530cf4.md#inline-formatting).

A block of text is a stack of [line boxes](css2--visuren.html--3f334c530cf4.md#line-box). In the case of 'left', 'right' and 'center', this property specifies how the inline-level boxes within each line box align with respect to the line box's left and right sides; alignment is not with respect to the [viewport](css2--visuren.html--3f334c530cf4.md#viewport). In the case of 'justify', this property specifies that the inline-level boxes are to be made flush with both sides of the line box if possible, by expanding or contracting the contents of inline boxes, else aligned as for the initial value. (See also ['letter-spacing'](css2--text.html--467a8857ae69.md#propdef-letter-spacing) and ['word-spacing'](css2--text.html--467a8857ae69.md#propdef-word-spacing).)

If an element has a computed value for 'white-space' of 'pre' or 'pre-wrap', then neither the glyphs of that element's text content nor its white space may be altered for the purpose of justification.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS may add a way to justify text with 'white-space: pre-wrap' in the future.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, note that since ['text-align'](css2--text.html--467a8857ae69.md#propdef-text-align) is inherited, all block-level elements inside DIV elements with a class name of 'important' will have their inline content centered.
>
> ```text
> 
> div.important { text-align: center }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>
The actual justification algorithm used depends on the user-agent and the language/script 
of the text.</em>

<a id="x4"></a>

<em><span title="conformance"><a href="css2--conform.html--de58593b67d7.md#conformance">Conforming user agents</a></span> may
interpret the value 'justify' as 'left' or 'right', depending on
whether the element's default writing direction is left-to-right or
right-to-left, respectively.</em>

<a id="decoration"></a>

## 16.3 Decoration

<a id="lining-striking-props"></a>

### 16.3.1 Underlining, overlining, striking, and blinking: the ['text-decoration'](css2--text.html--467a8857ae69.md#propdef-text-decoration) property

<a id="propdef-text-decoration"></a>

<strong>'text-decoration'</strong>

|                       |                                                                                                                                                              |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | none \| \[ underline \|\| overline \|\| line-through \|\| blink \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                         |
| <em>Applies to:</em>   | all elements                                                                                                                                                 |
| <em>Inherited:</em>   | no (see prose)                                                                                                                                               |
| <em>Percentages:</em>   | N/A                                                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                         |
| <em>Computed value:</em>   | as specified                                                                                                                                                 |

This property describes decorations that are added to the text of an element using the element's color. When specified on or propagated to an inline element, it affects all the boxes generated by that element, and is further propagated to any in-flow block-level boxes that split the inline (see [section 9.2.1.1](css2--visuren.html--3f334c530cf4.md#anonymous-block-level)). But, in CSS 2.1, it is undefined whether the decoration propagates into block-level tables. For block containers that establish an [inline formatting context,](css2--visuren.html--3f334c530cf4.md#inline-formatting) the decorations are propagated to an anonymous inline element that wraps all the in-flow inline-level children of the block container. For all other elements it is propagated to any in-flow children. Note that text decorations are not propagated to floating and absolutely positioned descendants, nor to the contents of atomic inline-level descendants such as inline blocks and inline tables.

Underlines, overlines, and line-throughs are applied only to text (including white space, letter spacing, and word spacing): margins, borders, and padding are skipped. User agents must not render these text decorations on content that is not text. For example, images and inline blocks must not be underlined.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> If an element E has both 'visibility:
hidden' and 'text-decoration: underline', the underline is invisible
(although any decoration of E's parent <strong>is</strong> visible.) 
However, CSS 2.1 does not specify if the underline is visible or
invisible in E's children:</em>
>
> ```text
> 
> <span style="visibility: hidden; text-decoration: underline">
>  <span style="visibility: visible">
>   underlined or not?
>  </span>
> </span>
> ```
>
> <em>This is expected to be specified in level 3 of CSS.</em>

The 'text-decoration' property on descendant elements cannot have any effect on the decoration of the ancestor. In determining the position of and thickness of text decoration lines, user agents may consider the font sizes of and dominant baselines of descendants, but must use the same baseline and thickness on each line. Relatively positioning a descendant moves all text decorations affecting it along with the descendant's text; it does not affect calculation of the decoration's initial position on that line.

Values have the following meanings:

none  
Produces no text decoration.

underline  
Each line of text is underlined.

overline  
Each line of text has a line above it.

line-through  
Each line of text has a line through the middle.

blink  
Text blinks (alternates between visible and invisible). [Conforming user agents](css2--conform.html--de58593b67d7.md#conformance) may simply not blink the text. Note that not blinking the text is one technique to satisfy [checkpoint 3.3 of WAI-UAAG](https://www.w3.org/TR/UAAG/guidelines.html#tech-on-off-blinking-text).

The color(s) required for the text decoration must be derived from the ['color'](css2--colors.html--5784063d2778.md#propdef-color) property value of the element on which 'text-decoration' is set. The color of decorations must remain the same even if descendant elements have different ['color'](css2--colors.html--5784063d2778.md#propdef-color) values.

Some user agents have implemented text-decoration by propagating the decoration to the descendant elements as opposed to preserving a constant thickness and line position as described above. This was arguably allowed by the looser wording in CSS2. SVG1, CSS1-only, and CSS2-only user agents may implement the older model and still claim conformance to this part of CSS 2.1. (This does not apply to UAs developed after this specification was released.)

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the following example for HTML, the text content of all A elements acting as hyperlinks (whether visited or not) will be underlined:
>
> ```text
> 
> a:visited,a:link { text-decoration: underline }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the following style sheet and document fragment:
>
> ```text
> 
>    blockquote { text-decoration: underline; color: blue; }
>    em { display: block; }
>    cite { color: fuchsia; }
> ```
>
> ```text
> 
>    <blockquote>
>     <p>
>      <span>
>       Help, help!
>       <em> I am under a hat! </em>
>       <cite> —GwieF </cite>
>      </span>
>     </p>
>    </blockquote>
> ```
>
> ...the underlining for the blockquote element is propagated to an anonymous inline element that surrounds the span element, causing the text "Help, help!" to be blue, with the blue underlining from the anonymous inline underneath it, the color being taken from the blockquote element. The `<em>text</em>` in the em block is also underlined, as it is in an in-flow block to which the underline is propagated. The final line of text is fuchsia, but the underline underneath it is still the blue underline from the anonymous inline element.
>
> ![Sample rendering of the above underline example](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/underline-example.png)
>
> This diagram shows the boxes involved in the example above. The rounded aqua line represents the anonymous inline element wrapping the inline contents of the paragraph element, the rounded blue line represents the span element, and the orange lines represent the blocks.

<a id="spacing-props"></a>

## 16.4 Letter and word spacing: the ['letter-spacing'](css2--text.html--467a8857ae69.md#propdef-letter-spacing) and ['word-spacing'](css2--text.html--467a8857ae69.md#propdef-word-spacing) properties

<a id="propdef-letter-spacing"></a>

<strong>'letter-spacing'</strong>

|                       |                                                                                                                                                                                              |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                 |
| <em>Inherited:</em>   | yes                                                                                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                         |
| <em>Computed value:</em>   | 'normal' or absolute length                                                                                                                                                                  |

This property specifies spacing behavior between text characters. Values have the following meanings:

normal

The spacing is the normal spacing for the current font. This value allows the user agent to alter the space between characters in order to justify text.

<a id="x7"></a>

[\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length)

This value indicates inter-character space <em>in
addition to</em> the default space between characters. Values may be negative, but there may be implementation-specific limits. User agents may not further increase or decrease the inter-character space in order to justify text.

Character spacing algorithms are user agent-dependent.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, the space between characters in BLOCKQUOTE elements is increased by '0.1em'.
>
> ```text
> 
> blockquote { letter-spacing: 0.1em }
> ```
>
> In the following example, the user agent is not permitted to alter inter-character space:
>
> ```text
> 
> blockquote { letter-spacing: 0cm }   /* Same as '0' */
> ```
<a id="x8"></a>

When the resultant space between two characters is not the same as the default space, user agents should not use ligatures.

<a id="propdef-word-spacing"></a>

<strong>'word-spacing'</strong>

|                       |                                                                                                                                                                                              |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                 |
| <em>Inherited:</em>   | yes                                                                                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                         |
| <em>Computed value:</em>   | for 'normal' the value '0'; otherwise the absolute length                                                                                                                                    |

This property specifies spacing behavior between words. Values have the following meanings:

normal

The normal inter-word space, as defined by the current font and/or the UA.

<a id="x10"></a>

[\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length)

This value indicates inter-word space <em>in
addition to</em> the default space between words. Values may be negative, but there may be implementation-specific limits.

Word spacing algorithms are user agent-dependent. Word spacing is also influenced by justification (see the ['text-align'](css2--text.html--467a8857ae69.md#propdef-text-align) property). Word spacing affects each space (U+0020) and non-breaking space (U+00A0), left in the text after the white space processing rules have been applied. The effect of the property on other word-separator characters is undefined. However general punctuation, characters with zero advance width (such as the zero with space U+200B) and fixed-width spaces (such as U+3000 and U+2000 through U+200A) are not affected.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, the word-spacing between each word in H1 elements is increased by '1em'.
>
> ```text
> 
> h1 { word-spacing: 1em }
> ```
<a id="caps-prop"></a>

## 16.5 Capitalization: the ['text-transform'](css2--text.html--467a8857ae69.md#propdef-text-transform) property

<a id="propdef-text-transform"></a>

<strong>'text-transform'</strong>

|                       |                                                                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | capitalize \| uppercase \| lowercase \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                   |
| <em>Applies to:</em>   | all elements                                                                                                                           |
| <em>Inherited:</em>   | yes                                                                                                                                    |
| <em>Percentages:</em>   | N/A                                                                                                                                    |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                   |
| <em>Computed value:</em>   | as specified                                                                                                                           |

This property controls capitalization effects of an element's text. Values have the following meanings:

capitalize  
Puts the first character of each word in uppercase; other characters are unaffected.

uppercase  
Puts all characters of each word in uppercase.

lowercase  
Puts all characters of each word in lowercase.

none  
No capitalization effects.

The actual transformation in each case is written language dependent. See BCP 47 ([\[BCP47\]](css2--refs.html--f208d881d0b7.md#ref-BCP47)) for ways to find the language of an element.

Only characters belonging to "bicameral scripts" [\[UNICODE\]](css2--refs.html--f208d881d0b7.md#ref-UNICODE) are affected.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, all text in an H1 element is transformed to uppercase text.
>
> ```text
> 
> h1 { text-transform: uppercase }
> ```
<a id="white-space-prop"></a>

## 16.6 White space: the ['white-space'](css2--text.html--467a8857ae69.md#propdef-white-space) property

<a id="propdef-white-space"></a>

<strong>'white-space'</strong>

|                       |                                                                                                                                           |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| pre \| nowrap \| pre-wrap \| pre-line \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                                    |
| <em>Applies to:</em>   | all elements                                                                                                                              |
| <em>Inherited:</em>   | yes                                                                                                                                       |
| <em>Percentages:</em>   | N/A                                                                                                                                       |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                      |
| <em>Computed value:</em>   | as specified                                                                                                                              |

This property declares how white space inside the element is handled. Values have the following meanings:

normal  
This value directs user agents to collapse sequences of white space, and break lines as necessary to fill line boxes.

pre  
This value prevents user agents from collapsing sequences of white space. Lines are only broken at preserved newline characters.

nowrap  
This value collapses white space as for 'normal', but suppresses line breaks within text.

pre-wrap  
This value prevents user agents from collapsing sequences of white space. Lines are broken at preserved newline characters, and as necessary to fill line boxes.

pre-line  
This value directs user agents to collapse sequences of white space. Lines are broken at preserved newline characters, and as necessary to fill line boxes.

Newlines in the source can be represented by a carriage return (U+000D), a linefeed (U+000A) or both (U+000D U+000A) or by some other mechanism that identifies the beginning and end of document segments, such as the SGML RECORD-START and RECORD-END tokens. The CSS 'white-space' processing model assumes all newlines have been normalized to line feeds. UAs that recognize other newline representations must apply the white space processing rules as if this normalization has taken place. If no newline rules are specified for the document language, each carriage return (U+000D) and CRLF sequence (U+000D U+000A) in the document text is treated as single line feed character. This default normalization rule also applies to generated content.

UAs must recognize line feeds (U+000A) as newline characters. UAs may additionally treat other forced break characters as newline characters per UAX14.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following examples show what [white space](css2--syndata.html--02e71c159e14.md#whitespace) behavior is expected from the PRE and P elements and the "nowrap" attribute in HTML.
>
> ```text
> 
> pre        { white-space: pre }
> p          { white-space: normal }
> td[nowrap] { white-space: nowrap }
> ```
>
> In addition, the effect of an HTML PRE element with the <em>non-standard</em> "wrap" attribute is demonstrated by the following example:
>
> ```text
> 
> pre[wrap]  { white-space: pre-wrap }
> ```
<a id="white-space-model"></a>

### 16.6.1 The 'white-space' processing model

For each inline element (including anonymous inline elements), the following steps are performed, treating bidi formatting characters as if they were not there:

1.  Each tab (U+0009), carriage return (U+000D), or space (U+0020) character surrounding a linefeed (U+000A) character is removed if 'white-space' is set to 'normal', 'nowrap', or 'pre-line'.
2.  If 'white-space' is set to 'pre' or 'pre-wrap', any sequence of spaces (U+0020) unbroken by an element boundary is treated as a sequence of non-breaking spaces. However, for 'pre-wrap', a line breaking opportunity exists at the end of the sequence.
3.  If 'white-space' is set to 'normal' or 'nowrap', linefeed characters are transformed for rendering purpose into one of the following characters: a space character, a zero width space character (U+200B), or no character (i.e., not rendered), according to UA-specific algorithms based on the content script.
4.  If 'white-space' is set to 'normal', 'nowrap', or 'pre-line',
    1.  every tab (U+0009) is converted to a space (U+0020)
    2.  any space (U+0020) following another space (U+0020) — even a space before the inline, if that space also has 'white-space' set to 'normal', 'nowrap' or 'pre-line' — is removed.

Then, the block container's inlines are laid out. Inlines are laid out, taking bidi reordering into account, and wrapping as specified by the 'white-space' property. When wrapping, line breaking opportunities are determined based on the text prior to the white space collapsing steps above.

As each line is laid out,

1.  If a space (U+0020) at the beginning of a line has 'white-space' set to 'normal', 'nowrap', or 'pre-line', it is removed.
2.  All tabs (U+0009) are rendered as a horizontal shift that lines up the start edge of the next glyph with the next tab stop. Tab stops occur at points that are multiples of 8 times the width of a space (U+0020) rendered in the block's font from the block's starting content edge.
3.  If a space (U+0020) at the end of a line has 'white-space' set to 'normal', 'nowrap', or 'pre-line', it is also removed.
4.  If spaces (U+0020) or tabs (U+0009) at the end of a line have 'white-space' set to 'pre-wrap', UAs may visually collapse them.

Floated and absolutely-positioned elements do not introduce a line breaking opportunity.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>
CSS 2.1 does not fully define where line breaking opportunities occur.</em>

<a id="egbidiwscollapse"></a>

### 16.6.2 Example of bidirectionality with white space collapsing

Given the following markup fragment, taking special note of spaces (with varied backgrounds and borders for emphasis and identification):

```text

 
     <ltr>A <rtl> B </rtl> C</ltr>

```
...where the `<ltr>` element represents a left-to-right embedding and the `<rtl>` element represents a right-to-left embedding, and assuming that the 'white-space' property is set to 'normal', the above processing model would result in the following:

- The space before the B ( ) would collapse with the space after the A ( ).
- The space before the C ( ) would collapse with the space after the B ( ).

This would leave two spaces, one after the A in the left-to-right embedding level, and one after the B in the right-to-left embedding level. This is then rendered according to the Unicode bidirectional algorithm, with the end result being:

```text


     A  BC

```
Note that there are two spaces between A and B, and none between B and C. This can sometimes be avoided by using the natural bidirectionality of characters instead of explicit embedding levels. Also, it is good to avoid spaces immediately inside start and end tags, as these tend to do weird things when dealing with white space collapsing.

<a id="ctrlchars"></a>

### 16.6.3 Control and combining characters' details

Control characters other than U+0009 (tab), U+000A (line feed), U+0020 (space), and U+202x (bidi formatting characters) are treated as characters to render in the same way as any normal character.

Combining characters should be treated as part of the character with which they are supposed to combine. For example, :first-letter styles the entire glyph if you have content like "`o<span>&#x308;</span>`"; it does not just match the base character.
