Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [CSS Text Module Level 3](https://www.w3.org/TR/2012/WD-css3-text-20121113/).

Original copyright notice: Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Text Module Level 3

Source snapshot: https://www.w3.org/TR/2012/WD-css3-text-20121113/

Snapshot SHA-256: cc1cba0eebc9b4eed5b292763b75a57c142af82797ca767a0ac3944220fb6bef

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# CSS Text Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2012 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

## <a id="abstract"></a>Abstract

This CSS3 module defines properties for text manipulation and specifies their processing model. It covers line breaking, justification and alignment, white space handling, and text transformation.

## <a id="status"></a>Status of This Document

<em>This section describes the status of this document at the time of
   its publication. Other documents may supersede this document. A list of
   current W3C publications and the latest revision of this technical report
   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports
   index at http://www.w3.org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

This CSS module has been produced as a combined effort of the [W3C Internationalization Activity](https://www.w3.org/International/Activity), and the [Style Activity](https://www.w3.org/Style/Activity) and is maintained by the [CSS Working Group](https://www.w3.org/Style/CSS/members). It also includes contributions made by participants in the [XSL Working Group](https://www.w3.org/Style/XSL/Group/) ([members only](http://cgi.w3.org/MemberAccess/AccessRequest)).

This document was produced by a group operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

<strong>Feedback on this draft should be posted to the (<a href="http://lists.w3.org/Archives/Public/www-style/">archived</a>) public
   mailing list <a href="mailto:www-style@w3.org">www-style@w3.org</a></strong> (see [instructions](https://www.w3.org/Mail/Request)) <strong>with
   <kbd>&#x5B;css3-text&#x5D;</kbd> in the subject line.</strong> You are strongly encouraged to complain if you see something stupid in this draft. The editors will do their best to respond to all feedback.

The following features are at risk and may be cut from the spec during its CR period if there are no (correct) implementations:

- the ‘`full-width`’ value of ‘[`text-transform`](#text-transform0)’
- the \<length\> values of the ‘[`tab-size`](#tab-size1)’ property
- the ‘`start end`’ and ‘`<string>`’ values of ‘[`text-align`](#text-align0)’
- the ‘[`text-justify`](#text-justify0)’ property, particularly its ‘`kashida`’ value
- the percentage values of ‘[`word-spacing`](#word-spacing0)’
- minimum and maximum limits of ‘[`word-spacing`](#word-spacing0)’ and ‘[`letter-spacing`](#letter-spacing0)’
- the ‘[`hanging-punctuation`](#hanging-punctuation0)’ property

## <a id="contents"></a> Table of Contents

## <a id="intro"></a>1.  Introduction

\[document here\]

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This draft describes features that are specific to certain scripts. There is an ongoing discussion about where these features belong: in existing CSS properties, in new CSS properties, or perhaps in other specifications.

<a id="decoration"></a>

<a id="text-decoration"></a>

<a id="line-decoration"></a>

<a id="text-decoration-line"></a>

<a id="text-decoration-color"></a>

<a id="text-decoration-style"></a>

<a id="text-decoration-skip"></a>

<a id="text-underline-position"></a>

<a id="emphasis-marks"></a>

<a id="text-emphasis-style"></a>

<a id="text-emphasis-color"></a>

<a id="text-emphasis"></a>

<a id="text-emphasis-position"></a>

<a id="text-shadow"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Text decoration has moved to CSS Text Decoration Module Level 3 [\[CSS3-TEXT-DECOR\]](#CSS3-TEXT-DECOR).

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the text-level features defined in [\[CSS21\]](#CSS21) chapter 16.

### <a id="values"></a>1.2.  Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#CSS21). Value types not defined in this specification are defined in CSS Level 2 Revision 1 [\[CSS21\]](#CSS21). Other CSS modules may expand the definitions of these value types: for example [\[CSS3COLOR\]](#CSS3COLOR), when combined with this module, expands the definition of the \<color\> value type as used in this specification.

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [inherit](https://www.w3.org/TR/CSS21/cascade.html#value-def-inherit) keyword as their property value. For readability it has not been repeated explicitly.

### <a id="terms"></a>1.3.  Terminology

<a id="grapheme-cluster"></a>A <a id="grapheme-cluster0"></a>grapheme cluster is what a language user considers to be a character or a basic unit of the script. The term is described in detail in the Unicode Technical Report: Text Boundaries [\[UAX29\]](#UAX29). This specification uses the <em>extended grapheme cluster</em> definition in [\[UAX29\]](#UAX29) (not the <em>legacy grapheme
   cluster</em> definition). The UA may further tailor the definition as allowed by Unicode. Within this specification, the ambiguous term <a id="character"></a>character is used as a friendlier synonym for [<i>grapheme cluster</i>](#grapheme-cluster0). See [Characters and Properties](http://dev.w3.org/csswg/css3-writing-modes/#character-properties) for how to determine the Unicode properties of a character.

<a id="letter"></a>A <a id="letter0"></a>letter for the purpose of this specification is a [<i>character</i>](#character) belonging to one of the Letter or Number general categories in Unicode. [\[UAX44\]](#UAX44)

The rendering characteristics of a [<i>character</i>](#character) divided by an element boundary is undefined: it may be rendered as belonging to either side of the boundary, or as some approximation of belonging to both. Authors are forewarned that dividing grapheme clusters by element boundaries may give inconsistent or undesired results.

The <a id="content-language"></a>content language of an element is the (human) language the element is declared to be in, according to the rules of the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage). For example, the rules for determining the [<i>content language</i>](#content-language) of an HTML element use the `lang` attribute and are defined in [\[HTML5\]](#HTML5), and the rules for determining the [<i>content language</i>](#content-language) of an XML element use the `xml:lang` attribute and are [defined](https://www.w3.org/TR/REC-xml/#sec-lang-tag) in [\[XML10\]](#XML10). Note that it is possible for the [<i>content language</i>](#content-language) of an element to be unknown.

Other terminology and concepts used in this specification are defined in [\[CSS21\]](#CSS21) and [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES).

## <a id="transforming"></a>2.  Transforming Text

<a id="caps-prop"></a>

### <a id="text-transform"></a>2.1.   Transforming Text: the ‘[`text-transform`](#text-transform0)’ property

|                   |                                                            |
|-------------------|------------------------------------------------------------|
| Name:             | <a id="text-transform0"></a>text-transform                          |
| [Value](#values): | none \| capitalize \| uppercase \| lowercase \| full-width |
| Initial:          | none                                                       |
| Applies to:       | all elements                                               |
| Inherited:        | yes                                                        |
| Percentages:      | N/A                                                        |
| Media:            | visual                                                     |
| Computed value:   | as specified                                               |

This property transforms text for styling purposes. (It has no effect on the underlying content.) Values have the following meanings:

<a id="none"></a>‘`none`’  
No effects.

<a id="capitalize"></a>‘`capitalize`’  
Puts the first [<i>letter</i>](#letter0) of each word in titlecase; other characters are unaffected.

<a id="uppercase"></a>‘`uppercase`’  
Puts all [<i>letters</i>](#letter0) in uppercase.

<a id="lowercase"></a>‘`lowercase`’  
Puts all [<i>letters</i>](#letter0) in lowercase.

<a id="full-width"></a>‘`full-width`’  
Puts all characters in fullwidth form. If the character does not have a corresponding fullwidth form, it is left as is. This value is typically used to typeset Latin characters and digits like ideographic characters.

The case mapping rules for the character repertoire specified by the Unicode Standard can be found on the Unicode Consortium Web site [\[UNICODE\]](#UNICODE). The UA must use the full case mappings for Unicode characters, including any conditional casing rules, as defined in Default Case Algorithm section. If (and only if) the [<i>content language</i>](#content-language) of the element is, according to the rules of the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage), known, then any appropriate language-specific rules must be applied as well. These minimally include, but are not limited to, the language-specific rules in Unicode's [SpecialCasing.txt](http://www.unicode.org/Public/UNIDATA/SpecialCasing.txt).

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, in Turkish there are two “i”s, one with a dot—“İ” and “i”— and one without—“I” and “ı”. Thus the usual case mappings between “I” and “i” are replaced with a different set of mappings to their respective undotted/dotted counterparts, which do not exist in English. This mapping must only take effect if the [<i>content language</i>](#content-language) is Turkish (or another Turkic language that uses Turkish casing rules); in other languages, the usual mapping of “I” and “i” is required. This rule is thus conditionally defined in Unicode's SpecialCasing.txt file.

The definition of "word" used for ‘`capitalize`’ is UA-dependent; [\[UAX29\]](#UAX29) is suggested (but not required) for determining such word boundaries. Authors should not expect ‘`capitalize`’ to follow language-specific titlecasing conventions (such as skipping articles in English).

The definition of fullwidth and halfwidth forms can be found on the Unicode consortium web site at [\[UAX11\]](#UAX11). The mapping to fullwidth form is defined by taking code points with the \<wide\> or the \<narrow\> tag in their Decomposition_Mapping in [\[UAX44\]](#UAX44). For the \<narrow\> tag, the mapping is from the code point to the decomposition (minus \<narrow\> tag), and for the \<wide\> tag, the mapping is from the decomposition (minus the \<wide\> tag) back to the original code point.

Text transformation happens after [white space processing](#white-space-rules), which means that ‘`full-width`’ transforms only preserved U+0020 spaces to U+3000.

> <strong data-conversion-semantic="example">Example</strong>
>
> The following example converts the ASCII characters in abbreviations in Japanese to their fullwidth variants so that they lay out and line break like ideographs:
>
> ```text
> abbr:lang(ja) { text-transform: full-width; }
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> CSS may introduce the ability to create custom mapping tables for less common text transforms, such as by an ‘`@text-transform`’ rule similar to ‘`@counter-style`’ from [\[CSS3LIST\]](#CSS3LIST).

<a id="white-space-collapsing"></a>

<a id="text-wrap"></a>

## <a id="white-space"></a>3.  White Space and Wrapping: the ‘[`white-space`](#white-space0)’ property

This property specifies two things:

- whether and how [white space](#white-space-processing) inside the element is collapsed
- whether lines may [<i>wrap</i>](#wrapping) at unforced [<i>soft wrap opportunities</i>](#soft-wrap-opportunity)

|                   |                                                 |
|-------------------|-------------------------------------------------|
| Name:             | <a id="white-space0"></a>white-space                  |
| [Value](#values): | normal \| pre \| nowrap \| pre-wrap \| pre-line |
| Initial:          | not defined for shorthand properties            |
| Applies to:       | all elements                                    |
| Inherited:        | yes                                             |
| Percentages:      | N/A                                             |
| Media:            | visual                                          |
| Computed value:   | see individual properties                       |

Values have the following meanings, which must be interpreted according to the [White Space Processing](#white-space-rules) and [Line Breaking](#line-breaking) rules:

<a id="normal"></a>‘`normal`’  
This value directs user agents to collapse sequences of white space into a single character (or [in some cases](#line-break-transform), no character). Lines may wrap at allowed [<i>soft wrap opportunities</i>](#soft-wrap-opportunity), as determined by the line-breaking rules in effect, in order to minimize overflow.

<a id="pre"></a>‘`pre`’  
This value prevents user agents from collapsing sequences of white space. [<i>Segment breaks</i>](#segment-break) such as line feeds and carriage returns are preserved as [<i>forced line breaks</i>](#forced-line-break). Lines only break at [<i>forced line breaks</i>](#forced-line-break); content that does not fit within the block container overflows it.

<a id="nowrap"></a>‘`nowrap`’  
Like ‘`normal`’, this value collapses white space; but like ‘`pre`’, it does not allow wrapping.

<a id="pre-wrap"></a>‘`pre-wrap`’  
Like ‘`pre`’, this value preserves white space; but like ‘`normal`’, it allows wrapping.

<a id="pre-line"></a>‘`pre-line`’  
Like ‘`normal`’, this value collapses consecutive spaces and allows wrapping, but preserves [<i>segment breaks</i>](#segment-break) in the source as [<i>forced line breaks</i>](#forced-line-break).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> There have been requests for the ability to "discard" white space; the current definition has no facility for this.

The following informative table summarizes the behavior of various ‘[`white-space`](#white-space0)’ values:

|                     | New Lines | Spaces and Tabs | Text Wrapping |
|---------------------|-----------|-----------------|---------------|
| ‘`normal`’ | Collapse  | Collapse        | Wrap          |
| ‘`pre`’ | Preserve  | Preserve        | No wrap       |
| ‘`nowrap`’ | Collapse  | Collapse        | No wrap       |
| ‘`pre-wrap`’ | Preserve  | Preserve        | Wrap          |
| ‘`pre-line`’ | Preserve  | Collapse        | Wrap          |

See [White Space Processing Rules](#white-space-processing) for details on how white space collapses. An informative summary of collapsing (‘`normal`’ and ‘`nowrap`’) is presented below:

- A sequence of segment breaks and other white space between two Chinese, Japanese, or Yi characters collapses into nothing.
- A zero width space before or after a white space sequence containing a segment break causes the entire sequence of white space to collapse into a zero width space.
- Otherwise, consecutive white space collapses into a single space.

See [Line Breaking](#line-breaking) for details on wrapping behavior.

## <a id="white-space-processing"></a>4.  White Space Processing Details

The source text of a document often contains formatting that is not relevant to the final rendering: for example, [breaking the source into segments](http://rhodesmill.org/brandon/2012/one-sentence-per-line/) (lines) for ease of editing or adding white space characters such as tabs and spaces to indent the source code. CSS white space processing allows the author to control interpretation of such formatting: to preserve or collapse it away when rendering the document. White space processing in CSS interprets white space characters only for rendering: it has no effect on the underlying document data.

White space processing in CSS is controlled with the ‘[`white-space`](#white-space0)’ property.

<a id="segment-normalization"></a> CSS does not define document segmentation rules. Segments could be separated by a particular newline seqence (such as a line feed or CRLF pair), or delimited by some other mechanism, such as the SGML RECORD-START and RECORD-END tokens. For CSS processing, each document language–defined segment break, CRLF sequence (U+000D U+000A), carriage return (U+000D), and line feed (U+000A) in the text is treated as a <a id="segment-break"></a>segment break, which is then interpreted for rendering as specified by the ‘[`white-space`](#white-space0)’ property.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the document parser may have not only normalized any segment breaks, but also collapsed other space characters or otherwise processed white space according to markup rules. Because CSS processing occurs <em>after</em> the parsing stage, it is not possible to restore these characters for styling. Therefore, some of the behavior specified below can be affected by these limitations and may be user agent dependent.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that anonymous inlines consisting entirely of [<i>collapsible</i>](#collapsible) white space are removed from the rendering tree. See [\[CSS21\]](#CSS21) section [9.2.2.1](https://www.w3.org/TR/CSS21/visuren.html#anonymous)

Control characters (Unicode class Cc) other than tab (U+0009), line feed (U+000A), and carriage return (U+000D) are ignored for the purpose of rendering.

### <a id="white-space-rules"></a>4.1.  The White Space Processing Rules

White space processing affects only spaces (U+0020), tabs (U+0009), and [segment breaks](#segment-normalization).

For each inline (including anonymous inlines) within an inline formatting context, white space characters are handled as follows, ignoring bidi formatting characters as if they were not there:

- <a id="collapse"></a>

  If ‘[`white-space`](#white-space0)’ is set to ‘`normal`’, ‘`nowrap`’, or ‘`pre-line`’, white space characters are considered <a id="collapsible"></a>collapsible and are processed by performing the following steps:

  1.  All spaces and tabs immediately preceding or following a segment break are removed.
  2.  Segment breaks are transformed for rendering according to the [line break transformation rules](#line-break-transform).
  3.  Every tab is converted to a space (U+0020).
  4.  Any space immediately following another collapsible space —even one outside the boundary of the inline containing the space, provided they are within the same inline formatting context—is collapsed to have zero advance width. (It is invisible, but retains its [<i>soft wrap opportunity</i>](#soft-wrap-opportunity), if any.)

- If ‘[`white-space`](#white-space0)’ is set to ‘`pre-wrap`’, any sequence of spaces is treated as a sequence of non-breaking spaces. However, a [<i>soft wrap opportunity</i>](#soft-wrap-opportunity) exists at the end of the sequence.

Then, the entire block is rendered. Inlines are laid out, taking bidi reordering into account, and [<i>wrapping</i>](#wrapping) as specified by the ‘[`white-space`](#white-space0)’ property.

As each line is laid out,

1.  A sequence of collapsible spaces at the beginning of a line is removed.
2.  Each tab is rendered as a horizontal shift that lines up the start edge of the next glyph with the next tab stop. <a id="tab-stops"></a>Tab stops occur at points that are multiples of the [<i>tab size</i>](#tab-size0) from the block's starting content edge. The <a id="tab-size0"></a>tab size is given by the ‘[`tab-size`](#tab-size1)’ property.
3.  A sequence of [<i>collapsible</i>](#collapsible) spaces at the end of a line is removed.
4.  If spaces or tabs at the end of a line are non-collapsible but have ‘`text-wrap`’ set to ‘`normal`’ the UA may visually collapse their character advance widths.

White space that was not removed or collapsed during the white space processing steps is called <a id="preserved"></a>preserved white space.

> <strong data-conversion-semantic="example">Example</strong>
>
> #### <a id="egbidiwscollapse"></a>4.1.1.  Example of bidirectionality with white space collapsing
>
> Consider the following markup fragment, taking special note of spaces (with varied backgrounds and borders for emphasis and identification):
>
> ```text
> <ltr>A <rtl> B </rtl> C</ltr>
> ```
>
> where the `<ltr>` element represents a left-to-right embedding and the `<rtl>` element represents a right-to-left embedding. If the ‘`text-space-collapse`’ property is set to ‘`collapse`’, the above processing model would result in the following:
>
> - The space before the B ( ) would collapse with the space after the A ( ).
> - The space before the C ( ) would collapse with the space after the B ( ).
>
> This would leave two spaces, one after the A in the left-to-right embedding level, and one after the B in the right-to-left embedding level. This is then ordered according to the Unicode bidirectional algorithm, with the end result being:
>
> ```text
> A  BC
> ```
>
> Note that there are two spaces between A and B, and none between B and C. This is best avoided by putting spaces outside the element instead of just inside the opening and closing tags and, where practical, by relying on implicit bidirectionality instead of explicit embedding levels.

#### <a id="line-break-transform"></a>4.1.2.  Line Break Transformation Rules

When ‘[`white-space`](#white-space0)’ is ‘`pre`’, ‘`pre-wrap`’, or ‘`pre-line`’, [<i>segment
   breaks</i>](#segment-break) are not [<i>collapsible</i>](#collapsible) and are instead transformed into a preserved line feed (U+000A).

For other values of ‘[`white-space`](#white-space0)’, [<i>segment breaks</i>](#segment-break) are [<i>collapsible</i>](#collapsible), and are either transformed into a space (U+0020) or removed depending on the context before and after the break:

- If the character immediately before or immediately after the segment break is the zero-width space character (U+200B), then the break is removed, leaving behind the zero-width space.
- Otherwise, if the East Asian Width property [\[UAX11\]](#UAX11) of both the character before and after the line feed is F, W, or H (not A), and neither side is Hangul, then the segment break is removed.
- Otherwise, the segment break is converted to a space (U+0020).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the white space processing rules have already removed any tabs and spaces after the segment break before these checks take place.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Comments on how well this would work in practice would be very much appreciated, particularly from people who work with Thai and similar scripts. Note that browser implementations do not currently follow these rules (although IE does in some cases transform the break).

### <a id="tab-size"></a>4.2.  Tab Character Size: the ‘[`tab-size`](#tab-size1)’ property

|                   |                                               |
|-------------------|-----------------------------------------------|
| Name:             | <a id="tab-size1"></a>tab-size                   |
| [Value](#values): | \<integer\> \| \<length\>                     |
| Initial:          | 8                                             |
| Applies to:       | block containers                              |
| Inherited:        | yes                                           |
| Percentages:      | N/A                                           |
| Media:            | visual                                        |
| Computed value:   | the specified integer or length made absolute |

This property determines the [<i>tab size</i>](#tab-size0) used to render preserved tab characters (U+0009). Integers represent the measure as multiples of the space character's advance width (U+0020). Negative values are not allowed.

## <a id="line-breaking"></a>5.  Line Breaking and Word Boundaries

When inline-level content is laid out into lines, it is broken across line boxes. Such a break is called a <a id="line-break0"></a>line break. When a line is broken due to explicit line-breaking controls, or due to the start or end of a block, it is a <a id="forced-line-break"></a>forced line break. When a line is broken due to content <a id="wrapping"></a>wrapping (i.e. when the UA creates unforced line breaks in order to fit the content within the measure), it is a <a id="soft-wrap-break"></a>soft wrap break. The process of breaking inline-level content into lines is called <a id="line-breaking0"></a>line breaking.

Wrapping is only performed at an allowed break point, called a <a id="soft-wrap-opportunity"></a>soft wrap opportunity.

In most writing systems, in the absence of hyphenation a [<i>soft wrap opportunity</i>](#soft-wrap-opportunity) occurs only at word boundaries. Many such systems use spaces or punctuation to explicitly separate words, and [<i>soft
   wrap opportunities</i>](#soft-wrap-opportunity) can be identified by these characters. Scripts such as Thai, Lao, and Khmer, however, do not use spaces or punctuation to separate words. Although the zero width space (U+200B) can be used as an explicit word delimiter in these scripts, this practice is not common. As a result, a lexical resource is needed to correctly identify [<i>soft wrap opportunities</i>](#soft-wrap-opportunity) in such texts.

In several other writing systems, (including Chinese, Japanese, Yi, and sometimes also Korean) a [<i>soft wrap
   opportunity</i>](#soft-wrap-opportunity) is based on syllable boundaries, not word boundaries. In these systems a line can break anywhere <em>except</em> between certain character combinations. Additionally the level of strictness in these restrictions can vary with the typesetting style.

CSS does not fully define where [<i>soft
   wrap opportunities</i>](#soft-wrap-opportunity) occur, however some controls are provided to distinguish common variations.

> <strong data-conversion-semantic="note">Note</strong>
>
> Further information on line breaking conventions can be found in [\[JLREQ\]](#JLREQ) and [\[JIS4051\]](#JIS4051) for Japanese, [\[ZHMARK\]](#ZHMARK) for Chinese, and in [\[UAX14\]](#UAX14) for all scripts in Unicode.
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > Any guidance for appropriate references here would be much appreciated.

### <a id="line-break-details"></a>5.1.  Line Breaking Details

When determining [<i>line breaks</i>](#line-break0):

- Regardless of the ‘[`white-space`](#white-space0)’ value, lines always break at each [<i>preserved</i>](#preserved) forced break character: for all values, line-breaking behavior defined for the BK, CR, LF, CM, NL, and SG line breaking classes in [\[UAX14\]](#UAX14) must be honored.
- When ‘[`white-space`](#white-space0)’ allows wrapping, line breaking behavior defined for the WJ, ZW, and GL line-breaking classes in [\[UAX14\]](#UAX14) must be honored.
- UAs that allow wrapping at punctuation other than spaces should prioritize breakpoints. For example, if breaks after slashes are given a lower priority than spaces, the sequence "check /etc" will never break between the ‘`/`’ and the ‘`e`’. The UA may use the width of the containing block, the text's language, and other factors in assigning priorities. As long as care is taken to avoid such awkward breaks, allowing breaks at appropriate punctuation other than spaces is recommended, as it results in more even-looking margins, particularly in narrow measures.
- Out-of-flow elements do not introduce a [<i>forced line break</i>](#forced-line-break) or [<i>soft wrap opportunity</i>](#soft-wrap-opportunity) in the flow.
- The line breaking behavior of a replaced element or other atomic inline is equivalent to that of the Object Replacement Character (U+FFFC).
- For [<i>soft wrap
    opportunities</i>](#soft-wrap-opportunity) created by characters that disappear at the line break (e.g. U+0020 SPACE), properties on the element containing that character control the line breaking at that opportunity. For [<i>soft wrap opportunities</i>](#soft-wrap-opportunity) defined by the boundary between two characters, the properties on the element containing the boundary control breaking.
- For [<i>soft wrap
    opportunities</i>](#soft-wrap-opportunity) before the first or after the last character of a box, the break occurs immediately before/after the box (at its margin edge) rather than breaking the box between its content edge and the content.
- For line breaking in/around [ruby](https://www.w3.org/TR/css3-ruby/), the base text is considered part of the same inline formatting context as its surrouding content, but the ruby text is not: i.e. line breaking opportunities between the ruby element and its surrounding content are determined as if the ruby base were inline and the ruby text were not there.

### <a id="line-break"></a>5.2.  Breaking Rules for Punctuation: the ‘[`line-break`](#line-break1)’ property

|                   |                                   |
|-------------------|-----------------------------------|
| Name:             | <a id="line-break1"></a>line-break     |
| [Value](#values): | auto \| loose \| normal \| strict |
| Initial:          | auto                              |
| Applies to:       | all elements                      |
| Inherited:        | yes                               |
| Percentages:      | N/A                               |
| Media:            | visual                            |
| Computed value:   | specified value                   |

This property specifies the strictness of line-breaking rules applied within an element: particularly how [<i>wrapping</i>](#wrapping) interacts with punctuation and symbols. Values have the following meanings:

<a id="auto"></a>‘`auto`’  
The UA determines the set of line-breaking restrictions to use, and it may vary the restrictions based on the length of the line; e.g., use a less restrictive set of line-break rules for short lines.

<a id="loose"></a>‘`loose`’  
Breaks text using the least restrictive set of line-breaking rules. Typically used for short lines, such as in newspapers.

<a id="normal0"></a>‘`normal`’  
Breaks text using the most common set of line-breaking rules.

<a id="strict"></a>‘`strict`’  
Breaks text using the most stringent set of line-breaking rules.

CSS distinguishes between three levels of strictness in the rules for text wrapping. The precise set of rules in effect for each level is up to the UA and should follow language conventions. However, this specification does recommend that:

- Following breaks be forbidden in ‘`strict`’ line breaking and allowed in ‘`normal`’ and ‘`loose`’:
  - breaks before Japanese [small kana](#small-kana)
  - breaks before the Katakana-Hiragana prolonged sound mark: ー U+30FC, ｰ U+FF70

  If the [<i>content language</i>](#content-language) is Chinese, Japanese, or Korean, then additionally:
  - breaks before hyphens:  
    ‐ U+2010, – U+2013, 〜 U+301C, ゠ U+30A0
- Following breaks be forbidden in ‘`normal`’ and ‘`strict`’ line breaking and allowed in ‘`loose`’:
  - breaks before iteration marks:  
    々 U+3005, 〻 U+303B, ゝ U+309D, ゞ U+309E, ヽ U+30FD, ヾ U+30FE
  - breaks between some inseparable characters:  
    ‥ U+2025, … U+2026

  If the [<i>content language</i>](#content-language) is Chinese, Japanese, or Korean, then additionally:
  - breaks before certain centered punctuation marks:  
    : U+003A, ; U+003B, ・ U+30FB, ： U+FF1A, ； U+FF1B, ･ U+FF65, ! U+0021, ? U+003F, ‼ U+203C, ⁇ U+2047, ⁈ U+2048, ⁉ U+2049, ！ U+FF01, ？ U+FF1F
  - breaks before postfixes:  
    % U+0025, ¢ U+00A2, ° U+00B0, ‰ U+2030, ′ U+2032, ″ U+2033, ℃ U+2103, ％ U+FF05, ￠ U+FFE0
  - breaks after prefixes:  
    \$ U+0024, £ U+00A3, ¥ U+00A5, € U+20AC, № U+2116, ＄ U+FF04, ￡ U+FFE1, ￥ U+FFE5

> <strong data-conversion-semantic="note">Note</strong>
>
> In the recommended list above, no distinction is made among the levels of strictness in non-CJK text: only CJK codepoints are affected, unless the text is marked as Chinese or Japanese, in which case some additional common codepoints are affected. However a future level of CSS may add behaviors affecting non-CJK text.

Support for this property is <em>optional</em>. It is recommended for UAs that wish to support CJK typography and strongly recommended for UAs in the Japanese market.

> <strong data-conversion-semantic="note">Note</strong>
>
> The CSSWG recognizes that in a future edition of the specification finer control over line breaking may be necessary to satisfy high-end publishing requirements.

### <a id="word-break"></a>5.3.  Breaking Rules for Letters: the ‘[`word-break`](#word-break0)’ property

|                   |                                 |
|-------------------|---------------------------------|
| Name:             | <a id="word-break0"></a>word-break   |
| [Value](#values): | normal \| keep-all \| break-all |
| Initial:          | normal                          |
| Applies to:       | all elements                    |
| Inherited:        | yes                             |
| Percentages:      | N/A                             |
| Media:            | visual                          |
| Computed value:   | specified value                 |

This property specifies [<i>soft wrap
   opportunities</i>](#soft-wrap-opportunity) between letters. Values have the following meanings:

<a id="normal1"></a>‘`normal`’  
Words break according to their usual rules.

<a id="break-all"></a>‘`break-all`’  
In addition to ‘`normal`’ [<i>soft wrap opportunities</i>](#soft-wrap-opportunity), lines may break between any two [<i>letters</i>](#letter0) (except where forbidden by the ‘[`line-break`](#line-break1)’ property). Hyphenation is not applied. This option is used mostly in a context where the text is predominantly using CJK characters with few non-CJK excerpts and it is desired that the text be better distributed on each line.

<a id="keep-all"></a>‘`keep-all`’  
Implicit [<i>soft wrap
    opportunities</i>](#soft-wrap-opportunity) between [<i>letters</i>](#letter0) are suppressed, i.e. breaks are prohibited between pairs of letters (except where opportunities exist due to dictionary-based breaking). Otherwise this option is equivalent to ‘`normal`’. In this style, sequences of CJK characters do not break.

> <strong data-conversion-semantic="note">Note</strong>
>
> This is sometimes seen in Korean (which uses spaces between words), and is also useful for mixed-script text where CJK snippets are mixed into another language that uses spaces for separation.

Symbols that line-break the same way as letters of a particular category are affected the same way as those letters.

> <strong data-conversion-semantic="example">Example</strong>
>
> Here's a mixed-script sample text:
>
> ```text
> 这是一些汉字, and some Latin, و کمی نوشتنن عربی, และตัวอย่างการเขียนภาษาไทย.
> ```
>
> The break-points are determined as follows (indicated by ‘·’):
>
> ‘`word-break: normal`’  
> ```text
> 这·是·一·些·汉·字,·and·some·Latin,·و·کمی·نوشتنن·عربی·และ·ตัวอย่าง·การเขียน·ภาษาไทย.
> ```
>
> ‘`word-break: break-all`’  
> ```text
> 这·是·一·些·汉·字,·a·n·d·s·o·m·e·L·a·t·i·n,·و·ﮐ·ﻤ·ﻰ·ﻧ·ﻮ·ﺷ·ﺘ·ﻦ·ﻋ·ﺮ·ﺑ·ﻰ,·แ·ล·ะ·ตั·ว·อ·ย่·า·ง·ก·า·ร·เ·ขี·ย·น·ภ·า·ษ·า·ไ·ท·ย.
> ```
>
> ‘`word-break: keep-all`’  
> ```text
> 这是一些汉字,·and·some·Latin,·و·کمی·نوشتنن·عربی,·และตัวอย่างการเขียนภาษาไทย.
> ```
When shaping scripts such as Arabic are allowed to break within words due to ‘`break-all`’, the characters must still be shaped as if the word were not broken.

## <a id="hyphenation"></a>6. Hyphenation

<a id="hyphenation0"></a>Hyphenation allows the controlled splitting of words to improve the layout of paragraphs, typically splitting words at syllabic or morphemic boundaries and visually indicating the split (usually with a hyphen). Hyphenation occurs when the line breaks at a valid <a id="hyphenation-opportunity"></a>hyphenation opportunity, which creates a [<i>soft wrap opportunity</i>](#soft-wrap-opportunity) within the word.

Hyphenation in CSS is controlled with the ‘[`hyphens`](#hyphens0)’ property. CSS Text Level 3 does not define the exact rules for hyphenation, however UAs are strongly encouraged to optimize their line-breaking implementation to choose good break points and appropriate hyphenation points.

Hyphenation opportunities <em>are</em> considered when calculating ‘`min-content`’ intrinsic sizes.

### <a id="hyphens"></a>6.1. Hyphenation Control: the ‘[`hyphens`](#hyphens0)’ property

|                   |                            |
|-------------------|----------------------------|
| Name:             | <a id="hyphens0"></a>hyphens |
| [Value:](#values) | none \| manual \| auto     |
| Initial:          | manual                     |
| Applies to:       | all elements               |
| Inherited:        | yes                        |
| Percentages:      | N/A                        |
| Media:            | visual                     |
| Computed value:   | specified value            |

This property controls whether hyphenation is allowed to create more [<i>soft wrap opportunities</i>](#soft-wrap-opportunity) within a line of text. Values have the following meanings:

<a id="none0"></a>‘`none`’  
Words are not hyphenated, even if characters inside the word explicitly define hyphenation opportunities.

<a id="manual"></a>‘`manual`’  
Words are only hyphenated where there are characters inside the word that explicitly suggest hyphenation opportunities.

> <strong data-conversion-semantic="example">Example</strong>
>
> In Unicode, U+00AD is a conditional "soft hyphen" and U+2010 is an unconditional hyphen. Unicode Standard Annex \#14 describes the [role of soft hyphens in](http://unicode.org/reports/tr14/#SoftHyphen) Unicode line breaking. [\[UAX14\]](#UAX14) In HTML, &#x26;shy; represents the soft hyphen character which suggests a hyphenation opportunity.
>
> ```text
> ex&shy;ample
> ```
<a id="auto0"></a>‘`auto`’  
Words may be broken at appropriate hyphenation points either as determined by hyphenation characters inside the word or as determined automatically by a language-appropriate hyphenation resource. Conditional hyphenation characters inside a word, if present, take priority over automatic resources when determining hyphenation opportunities within the word.

Correct automatic hyphenation requires a hyphenation resource appropriate to the language of the text being broken. The UA is therefore only required to automatically hyphenate text for which the author has declared a language (e.g. via HTML `lang` or XML `xml:lang`) and for which it has an appropriate hyphenation resource.

When shaping scripts such as Arabic are allowed to break within words due to hyphenation, the characters must still be shaped as if the word were not broken.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, if the word “نوشتنن” were hyphenated, it would appear as “ﻧﻮﺷ-ﺘﻦ” not as “ﻧﻮﺵ-ﺗﻦ”.

### <a id="overflow-wrap"></a>6.2.  Overflow Wrapping: the ‘[`word-wrap`](#word-wrap)’/‘[`overflow-wrap`](#overflow-wrap0)’ property

|                   |                                                               |
|-------------------|---------------------------------------------------------------|
| Name:             | <a id="overflow-wrap0"></a>overflow-wrap/<a id="word-wrap"></a>word-wrap |
| [Value](#values): | normal \| break-word                                          |
| Initial:          | normal                                                        |
| Applies to:       | all elements                                                  |
| Inherited:        | yes                                                           |
| Percentages:      | N/A                                                           |
| Media:            | visual                                                        |
| Computed value:   | specified value                                               |

This property specifies whether the UA may arbitrarily break within a word to prevent overflow when an otherwise-unbreakable string is too long to fit within the line box. It only has an effect when ‘[`white-space`](#white-space0)’ allows [<i>wrapping</i>](#wrapping). Possible values:

<a id="normal2"></a>‘`normal`’  
Lines may break only at allowed break points. However, the restrictions introduced by ‘

```text
word-break:
    keep-all
```
’ may be relaxed to match ‘

```text
word-break:
    normal
```
’ if there are no otherwise-acceptable break points in the line.

<a id="break-word"></a>‘`break-word`’  
An unbreakable "word" may be broken at an arbitrary point if there are no otherwise-acceptable break points in the line. Shaping characters are still shaped as if the word were not broken, and grapheme clusters must together stay as one unit. No hyphenation character is inserted at the break point.

[<i>Soft wrap opportunities</i>](#soft-wrap-opportunity) not part of ‘`overflow-wrap: normal`’ line breaking are not considered when calculating ‘`min-content`’ intrinsic sizes.

For legacy reasons, UAs must treat ‘[`word-wrap`](#word-wrap)’ as an alternate name for the ‘[`overflow-wrap`](#overflow-wrap0)’ property, as if it were a shorthand of ‘[`overflow-wrap`](#overflow-wrap0)’.

## <a id="justification"></a>7.  Alignment and Justification

### <a id="text-align"></a>7.1.  Text Alignment: the ‘[`text-align`](#text-align0)’ property

|                   |                                                                                                             |
|-------------------|-------------------------------------------------------------------------------------------------------------|
| Name:             | <a id="text-align0"></a>text-align                                                                               |
| [Value](#values): | \[ \[ start \| end \| left \| right \| center \] \|\| \<string\> \] \| justify \| match-parent \| start end |
| Initial:          | start                                                                                                       |
| Applies to:       | block containers                                                                                            |
| Inherited:        | yes                                                                                                         |
| Percentages:      | N/A                                                                                                         |
| Media:            | visual                                                                                                      |
| Computed value:   | specified value, except for ‘`match-parent`’ (see prose)                                                 |

This property describes how inline contents of a block are aligned along the inline axis if the contents do not completely fill the line box. Values have the following meanings:

<a id="start"></a>‘`start`’  
The inline contents are aligned to the start edge of the line box.

<a id="end"></a>‘`end`’  
The inline contents are aligned to the end edge of the line box.

<a id="left"></a>‘`left`’  
The inline contents are aligned to the [line left](https://www.w3.org/TR/css3-writing-modes/#line-left) edge of the line box. (Note that in vertical writing modes, this will be either the physical top or bottom.) [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES)

<a id="right"></a>‘`right`’  
The inline contents are aligned to the [line right](https://www.w3.org/TR/css3-writing-modes/#line-right) edge of the line box. (Note that in vertical writing modes, this will be either the physical top or bottom.) [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES)

<a id="center"></a>‘`center`’  
The inline contents are centered within the line box.

<a id="justify"></a>‘`justify`’  
The text is justified according to the method specified by the [‘`text-justify`’](#text-justify) property.

<a id="ltstringgt"></a>‘<code><span title="&lt;string&gt;"><a href="https://www.w3.org/TR/CSS21/syndata.html#value-def-string"><span>&lt;string&gt;</span></a></span></code>’  
The string must be a single [<i>character</i>](#character); otherwise the declaration must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore). When applied to a table cell, specifies the <a id="alignment-character"></a>alignment character around which the cell's contents will align. See [below](#character-alignment) for further details and how this value combines with keywords.

<a id="match-parent"></a>‘`match-parent`’  
This value behaves the same as ‘`inherit`’ except that an inherited ‘`start`’ or ‘`end`’ keyword is calculated against its parent's ‘`direction`’ value and results in a computed value of either ‘`left`’ or ‘`right`’.

<a id="start-end"></a>‘`start end`’  
Specifies ‘`start`’ alignment of the first line and any line immediately after a [<i>forced line break</i>](#forced-line-break); and ‘`end`’ alignment of any remaining lines not affected by ‘[`text-align-last`](#text-align-last0)’.

A block of text is a stack of [line boxes](https://www.w3.org/TR/CSS21/visuren.html#line-box). In the case of ‘`start`’, ‘`end`’, ‘`left`’, ‘`right`’ and ‘`center`’, this property specifies how the inline-level boxes within each line box align with respect to the start and end sides of the line box: alignment is not with respect to the [viewport](https://www.w3.org/TR/CSS21/visuren.html#viewport) or containing block.

In the case of ‘`justify`’, the UA may stretch or shrink any inline boxes by [adjusting](#text-justify) their text in addition to shifting their positions. (See also ‘[`text-justify`](#text-justify0)’, ‘[`letter-spacing`](#letter-spacing0)’, and ‘[`word-spacing`](#word-spacing0)’.) If an element's white space is not [collapsible](#collapse), then the UA is not required to adjust its text for the purpose of justification and may instead treat the text as having no [<i>expansion
   opportunities</i>](#expansion-opportunity). If the UA chooses to adjust the text, then it must ensure that tab stops continue to line up as required by the [white space processing rules](http://dev.w3.org/csswg/css3-text/#white-space-rules).

#### <a id="bidi-linebox"></a>7.1.1.  Bidirectionality and Line Boxes

The start and end edges of a line box are determined by the inline base direction of the line box. In most cases, this is given by its containing block's computed ‘`direction`’. However if its containing block has ‘

```text
unicode-bidi:
   plaintext
```
’ [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES), the inline base direction the line box must be determined by the base direction of the bidi paragraph to which it belongs: that is, the bidi paragraph for which the line box holds content. An empty line box (i.e. one that contains no atomic inlines or characters other than the line-breaking character, if any), takes its inline base direction from the preceding line box (if any), or, if this is the first line box in the containing block, then from the ‘`direction`’ property of the containing block.

> <strong data-conversion-semantic="example">Example</strong>
>
> In the following example, assuming the `<block>` is a preformatted block (‘
>
> ```text
> display: block; white-space:
>     pre
> ```
>
> ’) inheriting ‘`text-align: start`’, every other line is right-aligned:
>
> ```text
> <block style="unicode-bidi: plaintext">
>   Latin
>   و·کمی
>   Latin
>   و·کمی
>   Latin
>   و·کمی
> </block>
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the inline base direction determined here applies to the line box itself, and not to its contents. It affects ‘[`text-align`](#text-align0)’, ‘[`text-align-last`](#text-align-last0)’, ‘[`text-indent`](#text-indent0)’, and ‘[`hanging-punctuation`](#hanging-punctuation0)’, i.e. the position and alignment of its contents with respect to its edges. It does not affect the formatting or ordering of its content.

> <strong data-conversion-semantic="example">Example</strong>
>
> In the following example:
>
> ```text
> 
> <para style="display: block; direction: rtl; unicode-bidi:plaintext">
> <quote style="unicode-bidi:plaintext">שלום!</quote>", he said.
> </para>
> ```
>
> The result should be a left-aligned line looking like this:
>
> ```text
> "!שלום", he said.
> ```
>
> The line is left-aligned (despite the containing block having ‘`direction: rtl`’) because the containing block (the `<para>`) has ‘`unicode-bidi:plaintext`’, and the line box belongs to a bidi paragraph that is LTR. This is because that paragraph's first character with a strong direction is the LTR "h" from "he". The RTL "שלום!" does precede the "he", but it sits in its own bidi-isolated paragraph that is <em>not</em> immediately contained by the `<para>`, and is thus irrelevent to the line box's alignment. From from the standpoint of the bidi paragraph immediately contained by the `<para>` containing block, the `<quote>`’s bidi-isolated paragraph inside it is, by definition, just a neutral U+FFFC character, so the immediately-contained paragraph becomes LTR by virtue of the "he" following it.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
> <fieldset style="direction: rtl">
> <textarea style="unicode-bidi:plaintext">
> 
> Hello!
> 
> </textarea>
> </fieldset>
> ```
>
> As expected, the "Hello!" should be displayed LTR (i.e. with the exclamation mark on the right end, despite the `<textarea>`‘` s  `’‘`direction:rtl`’‘
>
> ```text
> ) and left-aligned.
>     This makes the empty line following it left-aligned as well, which means
>     that the caret on that line should appear at its left edge. The first
>     empty line, on the other hand, should be right-aligned, due to the RTL
>     direction of its containing paragraph, the <textarea>.
>     
> ```
#### <a id="character-alignment"></a>7.1.2. Character-based Alignment in a Table Column

When multiple cells in a column have an [<i>alignment character</i>](#alignment-character) specified, the alignment character of each such cell in the column is centered along a single column-parallel axis and the rest of the text in the column shifted accordingly. (Note that the strings do not have to be the same for each cell, although they usually are.)

> <strong data-conversion-semantic="example">Example</strong>
>
> The following style sheet:
>
> ```text
> TD { text-align: "." center }
> ```
>
> will cause the column of dollar figures in the following HTML table:
>
> ```text
> 
> <TABLE>
> <COL width="40">
> <TR> <TH>Long distance calls
> <TR> <TD> $1.30
> <TR> <TD> $2.50
> <TR> <TD> $10.80
> <TR> <TD> $111.01
> <TR> <TD> $85.
> <TR> <TD> N/A
> <TR> <TD> $.05
> <TR> <TD> $.06
> </TABLE>
> ```
>
> to align along the decimal point. The table might be rendered as follows:
>
> ```text
> 
> +---------------------+
> | Long distance calls |
> +---------------------+
> |        $11.30       |
> |        $22.50       |
> |         $0.80       |
> |    $200567.01       |
> |        $85.         |
> |        N/A          |
> |          $.05       |
> |          $.06       |
> +---------------------+
> ```
A keyword value may be specified in conjunction with the \<string\> value; if it is not given, it defaults to ’‘`right`’‘

```text
. This value is used:
   
```
- when character-based alignment is applied to boxes that are not table cells.
- when the text wraps to multiple lines (at unforced break points).
- when a character-aligned cell spans more than one column. In this case the keyword alignment value is used to determine which column’s axis to align with: the leftmost column for ‘`left`’, the rightmost column for ‘`right`’ and ‘`center`’, the startmost column for ‘`start`’, the endmost column for ‘`end`’.
- when the column is wide enough that the character alignment alone does not determine the positions of its character-aligned contents. In this case the keyword alignment of the first cell in the column with a specified alignment character is used to slide the position of the character-aligned contents to match the keyword alignment insofar as possible without changing the width of the column. For ‘`center`’, the UA may center the aligned contents using its extremes, center the alignment axis itself (insofar as possible), or optically center the aligned contents some other way (such as by taking a weighted average of the extent of the cells' contents to either side of the axis).

> <strong data-conversion-semantic="note">Note</strong>
>
> Right alignment is used by default for character-based alignment because numbering systems are almost all left-to-right even in right-to-left writing systems, and the primary use case of character-based alignment is for numerical alignment.

If the alignment character appears more than once in the text, the first instance is used for alignment. If the alignment character does not appear in a cell at all, the string is aligned as if the alignment character had been inserted at the end of its contents.

Character-based alignment occurs before table cell width computation so that auto width computations can leave enough space for alignment. Whether column-spanning cells participate in the alignment prior to or after width computation is undefined. If width constraints on the cell contents prevent full alignment throughout the column, the resulting alignment is undefined.

### <a id="text-align-last"></a>7.2.  Last Line Alignment: the ‘[`text-align-last`](#text-align-last0)’ property

|                   |                                                            |
|-------------------|------------------------------------------------------------|
| Name:             | <a id="text-align-last0"></a>text-align-last                         |
| [Value](#values): | auto \| start \| end \| left \| right \| center \| justify |
| Initial:          | auto                                                       |
| Applies to:       | block containers                                           |
| Inherited:        | yes                                                        |
| Percentages:      | N/A                                                        |
| Media:            | visual                                                     |
| Computed value:   | specified value                                            |

This property describes how the last line of a block or a line right before a [<i>forced line break</i>](#forced-line-break) is aligned. If a line is also the first line of the block or the first line after a [<i>forced line break</i>](#forced-line-break), then, unless ‘[`text-align`](#text-align0)’ assigns an explicit first line alignment (via ‘`start end`’), ‘[`text-align-last`](#text-align-last0)’ takes precedence over ‘[`text-align`](#text-align0)’.

If <a id="auto1"></a>‘`auto`’ is specified, content on the affected line is aligned per ‘[`text-align`](#text-align0)’ unless ‘[`text-align`](#text-align0)’ is set to ‘`justify`’. In this case, content is justified if ‘[`text-justify`](#text-justify0)’ is ‘`distribute`’ and start-aligned otherwise. All other values have the same meanings as in ‘[`text-align`](#text-align0)’.

### <a id="text-justify"></a>7.3.  Justification Method: the ‘[`text-justify`](#text-justify0)’ property

|                   |                                                                                         |
|-------------------|-----------------------------------------------------------------------------------------|
| Name:             | <a id="text-justify0"></a>text-justify                                                         |
| [Value](#values): | auto \| none \| inter-word \| inter-ideograph \| inter-cluster \| distribute \| kashida |
| Initial:          | auto                                                                                    |
| Applies to:       | block containers and, optionally, inline elements                                       |
| Inherited:        | yes                                                                                     |
| Percentages:      | N/A                                                                                     |
| Media:            | visual                                                                                  |
| Computed value:   | specified value                                                                         |

This property selects the justification method used when a line's alignment is set to ‘`justify`’ (see ‘[`text-align`](#text-align0)’), primarily by controlling which scripts' characters are adjusted together or separately. The property applies to block containers, but the UA may (but is not required to) also support it on inline elements. It takes the following values:

<a id="fig-text-justify"></a>

![Examples of text-justify values commonly used in East Asian scripts](https://www.w3.org/TR/2012/WD-css3-text-20121113/text-justify-east-asia.png)

Values of ‘[`text-justify`](#text-justify0)’: ‘`inter-word`’, ‘`inter-cluster`’, ‘`inter-ideograph`’, and ‘`distribute`’

<a id="fig-text-justify-kashida"></a>

![One possible example of rendering for text-justify: kashida](https://www.w3.org/TR/2012/WD-css3-text-20121113/text-justify-kashida.png)

One possible example of rendering for ‘[`text-justify`](#text-justify0)’: ‘`kashida`’

<a id="auto2"></a>‘`auto`’

The UA determines the justification algorithm to follow, based on a balance between performance and adequate presentation quality.

> <strong data-conversion-semantic="note">Note</strong>
>
> One possible algorithm is to determine the behavior based on the language of the paragraph: the UA can then choose appropriate value for the language, like ‘`inter-ideograph`’ for CJK, or ‘`inter-word`’ for English. Another possibility is to use a justification method that is a universal compromise for all scripts, e.g. the ‘`inter-cluster`’ method with block scripts raised to first priority.

<a id="none1"></a>‘`none`’

Justification is disabled. <strong data-conversion-semantic="note">Note:</strong> This value is intended for use in user stylesheets to improve readability or for accessibility purposes.

<a id="inter-word"></a>‘`inter-word`’

Justification primarily changes spacing at word separators. This value is typically used for languages that separate words using spaces, like English or Korean.

<a id="inter-ideograph"></a>‘`inter-ideograph`’

Justification primarily changes spacing at word separators and between characters in [block scripts](#block-scripts). This value is typically used for CJK languages.

<a id="inter-cluster"></a>‘`inter-cluster`’

Justification primarily changes spacing at word separators and between characters in [clustered scripts](#clustered-scripts). This value is typically used for Southeast Asian scripts such as Thai.

<a id="distribute"></a>‘`distribute`’

Justification primarily changes spacing both at word separators and between characters in all scripts equally (except those in the connected and cursive categories). This value is sometimes used in e.g. Japanese.

<a id="text-kashida-space"></a>

<a id="kashida-prop"></a>

<a id="kashida"></a>‘`kashida`’

Justification primarily stretches [cursive scripts](#cursive-scripts) through the use of kashida or other calligraphic elongation. This value is <em>optional</em> for conformance to CSS3 Text. (UAs that do not support cursive elongation must [treat the value as invalid](https://www.w3.org/TR/css-2010/#partial).)

When justifying text, the user agent takes the remaining space between the ends of a line's contents and the edges of its line box, and distributes that space throughout its contents so that the contents exactly fill the line box. If the ‘[`letter-spacing`](#letter-spacing0)’ and ‘[`word-spacing`](#word-spacing0)’ property values allow it, the user agent may also distribute negative space, putting more content on the line than would otherwise fit under normal spacing conditions. The exact justification algorithm is UA-dependent; however, CSS provides some general guidelines which should be followed when any justification method other than ‘[`auto`](#auto1)’ is specified.

> <strong data-conversion-semantic="note">Note</strong>
>
> The guidelines in this level of CSS do not describe a complete justification algorithm. They are merely a minimum set of requirements that a complete algorithm should meet. Limiting the set of requirements gives UAs some latitude in choosing a justification algorithm that meets their needs.
>
> For instance, a basic but fast ‘`inter-word`’ justification algorithm might use a simple greedy method for determining line breaks, then distribute leftover space using the [spacing limits provided](#spacing). This algorithm could follow the guidelines by expanding word spaces first, expanding between letters only if ‘[`word-spacing`](#word-spacing0)’ hit a limit.
>
> A more sophisticated but slower ‘`inter-word`’ justification algorithm might use a Knuth/Plass method where [<i>expansion opportunities</i>](#expansion-opportunity) and limits were assigned weights and assessed with other line breaking considerations. This algorithm could follow the guidelines by giving more weight to word spaces than letter spacing.

CSS defines <a id="expansion-opportunity"></a>expansion opportunities as points where the justification algorithm may alter spacing within the text. These [<i>expansion opportunities</i>](#expansion-opportunity) fall into priority levels as defined by the justification method. Within a line, expansion and compression should primarily target the first-priority expansion opportunities; lower priority expansion opportunities are adjusted at a lower priority as needed.

Expansion and compression limits are given by the [letter-spacing](#letter-spacing) and [word-spacing](#word-spacing) properties. How any remaining space is distributed once all [<i>expansion
   opportunities</i>](#expansion-opportunity) reach their limits is up to the UA. If the inline contents of a line cannot be stretched to the full width of the line box, then they must be aligned as specified by the ‘[`text-align-last`](#text-align-last0)’ property. (If ‘[`text-align-last`](#text-align-last0)’ is ‘`justify`’, then they must be aligned as for ‘`center`’ if ‘[`text-justify`](#text-justify0)’ is ‘`distribute`’ and as ‘`start`’ otherwise.)

The [<i>expansion opportunity</i>](#expansion-opportunity) priorities for values of ‘[`text-justify`](#text-justify0)’ are given in the table below. Since justification behavior varies by writing system, [<i>expansion opportunities</i>](#expansion-opportunity) are organized by [script categories](#script-groups). An [<i>expansion opportunity</i>](#expansion-opportunity) exists between two [<i>letters</i>](#letter0) at a priority level when at least one of them belongs to a script category at that level and the other does not belong to a higher priority level. All scripts in the same priority level must be treated exactly the same. Word separators (spaces) and other symbols and punctuation are treated specially, see below.

|                                 | ‘`inter-word`’ | ‘`inter-ideograph`’ | ‘`distribute`’ | ‘`inter-cluster`’ | ‘`kashida`’ | ‘[`auto`](#auto1)’ |
|---------------------------------|---------------------|---------------------|---------------------|---------------------|---------------------|-------------------------------|
| [block](#block-scripts)         | 2                   | <strong>1</strong> | <strong>1</strong> | 3                   | 3                   | <strong>2</strong>\*         |
| [clustered](#clustered-scripts) | 2                   | 2                   | <strong>1</strong> | <strong>2</strong> | 3                   | <strong>2</strong>\*         |
| [cursive](#cursive-scripts)     | 2                   | 2                   | 2                   | 3                   | <strong>1</strong> | 3\*                           |
| [discrete](#discrete-scripts)   | 2                   | 2                   | <strong>1</strong> | 3                   | 3                   | 3\*                           |
| [connected](#connected-scripts) | never               | never               | never               | never               | never               | never                         |
| spaces                          | <strong>1</strong> | <strong>1</strong> | <strong>1</strong> | <strong>1</strong> | 2                   | <strong>1</strong>\*         |
| symbols                         | 2                   | <strong>1</strong> | <strong>1</strong> | <strong>2</strong> | 3                   | \*                            |

Prioritization of Expansion Points

<a id="auto-justify"></a>\* The ‘[`auto`](#auto1)’ column defined above is informative; it suggests a prioritization that presents a universal compromise among justification methods.

<a id="justify-spaces"></a>The <a id="spaces"></a>spaces category represents [<i>expansion
   opportunities</i>](#expansion-opportunity) at [word separators](#word-separator). (See [‘`word-spacing`’](#word-spacing).) Except when ‘[`text-justify`](#text-justify0)’ is ‘`distribute`’, the UA may treat spaces differently than other [<i>expansion
   opportunities</i>](#expansion-opportunity) in the same priority, but must not change their priority with respect to [<i>expansion
   opportunities</i>](#expansion-opportunity) in other priority levels. For example, in Japanese ‘`inter-ideograph`’ justification (which treats CJK characters at a higher priority than Latin characters), word spaces traditionally have a higher priority than inter-CJK spacing, and the UA may split the 1st-priority level to implement that. However the UA is not allowed to drop either spaces or CJK characters to the same priority as Latin characters.

<a id="justify-symbols"></a>The <a id="punctuation-symbols"></a>symbols category represents the [<i>expansion
   opportunity</i>](#expansion-opportunity) existing at or between any pair of characters from the Unicode Symbols (S\*) and Punctuation (P\*) classes. The default justification priority of these [<i>expansion opportunities</i>](#expansion-opportunity) is given above. However, there may be additional rules controlling their justification behavior due to typographic tradition. Therefore, the UA may reassign specific characters or introduce additional levels of prioritization to handle [<i>expansion
   opportunities</i>](#expansion-opportunity) involving symbols and punctuation. For example, there are traditionally no [<i>expansion
   opportunities</i>](#expansion-opportunity) between consecutive EM DASH U+2014, HORIZONTAL BAR U+2015, HORIZONTAL ELLIPSIS U+2026, or TWO DOT LEADER U+2025 characters [\[JLREQ\]](#JLREQ); thus a UA might assign these characters to the "never" prioritization level. As another example, certain fullwidth punctuation characters are considered to contain an [<i>expansion
   opportunity</i>](#expansion-opportunity) (see ‘`text-spacing`’). The UA might therefore assign these characters to a higher prioritization level than the opportunities between ideographic characters.

<a id="justify-cursive"></a>For justification of [<i>cursive scripts</i>](#cursive-scripts0), words may be expanded through kashida elongation or other cursive expansion processes. Kashida may be applied in discrete units or continuously, and the prioritization of kashida opportunities is UA-dependent: for example, the UA may apply more at the end of the line. The UA should not apply kashida to fonts for which it is inappropriate. It may instead rely on other justification methods that lengthen or shorten Arabic segments (e.g. by substituting in swash forms or optional ligatures). Because elongation rules depend on the typeface style, the UA should rely on on the font whenever possible rather than inserting kashida based on a font-independent ruleset. The UA should limit elongation so that, e.g. in multi-script lines a short stretch of Arabic will not be forced to soak up too much of the extra space by itself. If the UA does not support cursive elongation, then, as with connected scripts, no [<i>expansion
   opportunities</i>](#expansion-opportunity) exist between characters of these scripts.

The UA may enable or break optional ligatures or use other font features such as alternate glyphs or glyph compression to help justify the text under any method. This behavior is not controlled by this level of CSS.

> <strong data-conversion-semantic="example">Example</strong>
>
> 3.8 Line Adjustment in [\[JLREQ\]](#JLREQ) gives an example of a set of rules for how a text formatter can justify Japanese text. It describes rules for cases where the ‘[`text-justify`](#text-justify0)’ property is ‘`inter-ideograph`’ and the ‘`text-spacing`’ property does not specify ‘`no-compress`’.
>
> It produces an effect similar to cases where the computed value of ‘`text-spacing`’ property does not specify ‘`trim-end`’ or ‘`space-end`’. If the UA wants to prohibit this behavior, rule b. of 3.8.3 should be omitted.
>
> Note that the rules described in the document specifically target Japanese. Therefore they may produce non-optimal results when used to justify other languages such as English. To make the rules more applicable to other scripts, the UA could, for instance, omit the rule to compress half-width spaces (rule a. of 3.8.3).

## <a id="spacing"></a>8.  Spacing

CSS offers control over text spacing via the ‘[`word-spacing`](#word-spacing0)’ and ‘[`letter-spacing`](#letter-spacing0)’ properties. While in CSS1 and CSS2 these could only be ‘`normal`’ (justifiable) or a fixed length, CSS3 can indicate range constraints to control flexibility in justification. In addition the ‘[`word-spacing`](#word-spacing0)’ property can now be specified in percentages, making it possible to, for example, double or eliminate word spacing.

> <strong data-conversion-semantic="example">Example</strong>
>
> In the following example, word spacing is halved, but may expand up to its full amount if needed for text justification.
>
> ```text
> p { word-spacing: -50% 0%; }
> ```
<a id="spacing-limits"></a>The <a id="ltspacing-limitsgt"></a>\<spacing-limits\> value type, which represents optimum, minimum, and maximum spacing in ‘[`word-spacing`](#word-spacing0)’ and ‘[`letter-spacing`](#letter-spacing0)’, is defined as

<a id="ltspacing-limits"></a>

```text
<spacing-limits> = [ normal | <length> | <percentage>]{1,3}
```
If three values are specified, they represent the optimum, minimum, and maximum in that order. If only two values are specified, then the first represents both the optimum and the minimum, and the second represents the maximum. If just one value is specified, then it represents the optimum, minimum, and maximum. The values are interpreted as defined below:

<a id="normal3"></a>‘`normal`’  
Specifies normal spacing as defined by the current font and/or the user agent; see [below](#normal-spacing). A ‘`normal`’ optimum spacing value computes to zero.

<a id="ltlengthgt"></a>‘`<length>`’  
Specifies extra spacing <em>in addition to</em> the intrinsic inter-character/inter-word spacing defined by the font. Values may be negative, but there may be implementation-dependent limits.

<a id="ltpercentagegt"></a>‘`<percentage>`’  
Specifies the additional spacing as a percentage of the affected character's <i>advance measure</i>. Only valid on ‘[`word-spacing`](#word-spacing0)’.

In the absence of justification the optimum spacing is be used. The text justification process may alter the spacing from its optimum (see the [‘`text-justify`’](#text-justify) property, above) but must not violate the minimum spacing limit and should also avoid exceeding the maximum. The UA may also use the difference between the minimum/maximum limits and the optimum as input into a weighting algorithm for justification.

The minimum is treated as a hard constraint: if the maximum is less than the minimum, then the used it is set to the minimum. Likewise for the optimum. Similarly if the maximum is less than the optimum, then the used optimum is set to the used maximum.

<a id="normal-spacing"></a>Normal spacing: Although ‘`normal`’ minimum and maximum spacing limits are UA-defined, they must be defined relative to the optimum so that the limits increase and decrease with changes to the optimum spacing. Normal limits may also vary according to the value of the [‘`text-justify`’](#text-justify) property, the element's language, some measure of the amount of text on a line (e.g. block width divided by font size), and/or other factors.

### <a id="word-spacing"></a>8.1.  Word Spacing: the ‘[`word-spacing`](#word-spacing0)’ property

|                   |                                                                                                                                        |
|-------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| Name:             | <a id="word-spacing0"></a>word-spacing                                                                                                        |
| [Value](#values): | [\<spacing-limits\>](#spacing-limits)                                                                                                  |
| Initial:          | normal                                                                                                                                 |
| Applies to:       | all elements                                                                                                                           |
| Inherited:        | yes                                                                                                                                    |
| Percentages:      | refers to width of the affected glyph                                                                                                  |
| Media:            | visual                                                                                                                                 |
| Computed value:   | an optimum, minimum, and maximum value, each consisting of either an absolute length, a percentage, or the keyword ‘`normal`’ |

This property specifies the minimum, maximum, and optimal spacing between “words”.

Additional spacing is applied to each word-separator character left in the text after the [white space processing rules](#white-space-rules) have been applied, and should be applied half on each side of the character unless otherwise dictated by typographic tradition.

> <strong data-conversion-semantic="example">Example</strong>
>
> The following example will make all the spaces between words in Arabic be rendered as zero-width, and double the width of each space in English:
>
> ```text
> :lang(ar) { word-spacing: -100%; }
> :lang(en) { word-spacing: 100%; }
> ```
>
> The following example will <em>add</em> half the the width of the “0” glyph to word spacing character [\[CSS3VAL\]](#CSS3VAL):
>
> ```text
> p { word-spacing: 0.5ch; }
> ```
<a id="word-separator"></a>Word-separator characters include the space (U+0020), the no-break space (U+00A0), the Ethiopic word space (U+1361), the Aegean word separators (U+10100,U+10101), the Ugaritic word divider (U+1039F), the Phoenician Word Separator (U+1091F), and the Tibetan tsek (U+0F0B, U+0F0C). If there are no word-separator characters, or if the word-separating character has a zero advance width (such as the zero width space U+200B) then the user agent must not create an additional spacing between words. General punctuation and fixed-width spaces (such as U+3000 and U+2000 through U+200A) are not considered word-separator characters.

### <a id="letter-spacing"></a>8.2.  Tracking: the ‘[`letter-spacing`](#letter-spacing0)’ property

|                   |                                                                                                                         |
|-------------------|-------------------------------------------------------------------------------------------------------------------------|
| Name:             | <a id="letter-spacing0"></a>letter-spacing                                                                                       |
| [Value](#values): | [\<spacing-limits\>](#spacing-limits)                                                                                   |
| Initial:          | normal                                                                                                                  |
| Applies to:       | all elements                                                                                                            |
| Inherited:        | yes                                                                                                                     |
| Percentages:      | N/A                                                                                                                     |
| Media:            | visual                                                                                                                  |
| Computed value:   | an optimum, minimum, and maximum value, each consisting of either an absolute length or the keyword ‘`normal`’ |

This property specifies the minimum, maximum, and optimal spacing between [<i>characters</i>](#character). Letter-spacing is applied in addition to any word-spacing.

Letter-spacing must not be applied at the beginning or at the end of a line. At element boundaries, the total letter spacing between two characters is given by and rendered within the innermost element that <em>contains</em> the boundary.

For the purpose of letter-spacing, each consecutive run of atomic inlines (such as image and/or inline blocks) is treated as a single [<i>character</i>](#character).

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, given the markup
>
> ```text
> <P>a<LS>b<Z>cd</Z><Y>ef</Y></LS>g</P>
> ```
>
> and the style sheet
>
> ```text
> LS { letter-spacing: 1em; }
> Z { letter-spacing: 0.3em; }
> Y { letter-spacing: 0.4em; }
> ```
>
> the spacing would be
>
> ```text
> a[0]b[1em]c[0.3em]d[1em]e[0.4em]f[0]g
> ```
UAs may apply letter-spacing to cursive scripts. In this case, UAs should extend the space between disjoint characters as specified above <em>and</em> extend the visible connection between cursively connected characters by the same amount (rather than leaving a gap). The UA may use glyph substitution or other font capabilities to spread out the letters. If the UA cannot expand a cursive script without breaking the cursive connections, it should not apply letter-spacing between characters of that script at all.

Letter-spacing ignores zero-width characters (such as those from the Unicode Cf category). For example, ‘[`letter-spacing`](#letter-spacing0)’ applied to `A&zwsp;B` is identical to `AB`.

When the effective letter-spacing between two characters is not zero (due to either [justification](#text-justify) or a non-zero specified optimum), user agents should not apply optional ligatures.

## <a id="edge-effects"></a>9.  Edge Effects

Edge effects control the indentation of lines with respect to other lines in the block (‘[`text-indent`](#text-indent0)’) and how content is aligned to the start and end edges of a line (‘[`hanging-punctuation`](#hanging-punctuation0)’).

### <a id="text-indent"></a>9.1.  First Line Indentation: the ‘[`text-indent`](#text-indent0)’ property

|                   |                                                                                                                                                                                                                               |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Name:             | <a id="text-indent0"></a>text-indent                                                                                                                                                                                                |
| [Value](#values): | \[ [\<length\>](https://www.w3.org/TR/CSS21/syndata.html#value-def-length) \| [\<percentage\>](https://www.w3.org/TR/CSS21/syndata.html#value-def-percentage) \] &#x26;&#x26; \[ hanging \|\| each-line \]? |
| Initial:          | 0                                                                                                                                                                                                                             |
| Applies to:       | block containers                                                                                                                                                                                                              |
| Inherited:        | yes                                                                                                                                                                                                                           |
| Percentages:      | refers to width of containing block                                                                                                                                                                                           |
| Media:            | visual                                                                                                                                                                                                                        |
| Computed value:   | the percentage as specified or the absolute length, plus any keywords as specified                                                                                                                                            |

This property specifies the indentation applied to lines of inline content in a block. The indent is treated as a margin applied to the start edge of the line box. Unless otherwise specified via the ‘[`each-line`](#each-line)’ and/or ‘[`hanging`](#hanging)’ keywords, only lines that are the [first formatted line](https://www.w3.org/TR/CSS21/selector.html#first-line-pseudo) of an element are affected. For example, the first line of an anonymous block box is only affected if it is the first child of its parent element.

Values have the following meanings:

‘`<length>`’  
Gives the amount of the indent as an absolute length.

‘`<percentage>`’  
Gives the amount of the indent as a percentage of the containing block's logical width.

<a id="each-line"></a>‘`each-line`’  
Indentation affects the first line of the block container as well as each line after a [<i>forced line
    break</i>](#forced-line-break), but does not affect lines after a [<i>soft wrap break</i>](#soft-wrap-break).

<a id="hanging"></a>‘`hanging`’  
Inverts which lines are affected.

> <strong data-conversion-semantic="example">Example</strong>
>
> If ‘[`text-align`](#text-align0)’ is ‘`start`’ and ‘[`text-indent`](#text-indent0)’ is ‘`5em`’ in left-to-right text with no floats present, then first line of text will start 5em into the block:
>
> ```text
>      Since CSS1 it has been possible
> to indent the first line of a block
> element using the 'text-indent'
> property.
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note that since the ‘[`text-indent`](#text-indent0)’ property inherits, when specified on a block element, it will affect descendant inline-block elements. For this reason, it is often wise to specify ‘
>
> ```text
> text-indent:
>    0
> ```
>
> ’ on elements that are specified ‘
>
> ```text
> display:
>    inline-block
> ```
>
> ’.

### <a id="hanging-punctuation"></a>9.2.  Hanging Punctuation: the ‘[`hanging-punctuation`](#hanging-punctuation0)’ property

|                   |                                                                 |
|-------------------|-----------------------------------------------------------------|
| Name:             | <a id="hanging-punctuation0"></a>hanging-punctuation                          |
| [Value](#values): | none \| \[ first \|\| \[ force-end \| allow-end \] \|\| last \] |
| Initial:          | none                                                            |
| Applies to:       | inline elements                                                 |
| Inherited:        | yes                                                             |
| Percentages:      | N/A                                                             |
| Media:            | visual                                                          |
| Computed value:   | as specified                                                    |

This property determines whether a punctuation mark, if one is present, [<i>hangs</i>](#hangs) and may be placed outside the line box (or in the indent) at the start or at the end of a line of text.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that if there is not sufficient padding on the block container, ‘[`hanging-punctuation`](#hanging-punctuation0)’ can trigger overflow.

When a punctuation mark <a id="hangs"></a>hangs, it is not considered when measuring the line's contents for fit, alignment, or justification. Depending on the line's alignment, this may (or may not) result in the mark being placed outside the line box.

Values have the following meanings:

<a id="none2"></a>‘`none`’  
No character [<i>hangs</i>](#hangs).

<a id="first"></a>‘`first`’  
An opening bracket or quote at the start of the [first formatted line](https://www.w3.org/TR/CSS21/selector.html#first-line-pseudo) of an element [<i>hangs</i>](#hangs). This applies to all characters in the Unicode categories Ps, Pf, Pi.

<a id="last"></a>‘`last`’  
A closing bracket or quote at the end of the <i>last formatted
    line</i> of an element [<i>hangs</i>](#hangs). This applies to all characters in the Unicode categories Pe, Pf, Pi.

<a id="force-end"></a>‘`force-end`’  
A stop or comma at the end of a line [<i>hangs</i>](#hangs).

<a id="allow-end"></a>‘`allow-end`’  
A stop or comma at the end of a line [<i>hangs</i>](#hangs) if it does not otherwise fit prior to justification.

Non-zero start and end borders/padding between a [<i>hang</i>](#hangs)eable mark and the edge of the line prevent the mark from hanging. For example, a period at the end of an inline box with end padding does not [<i>hang</i>](#hangs) at the end edge of a line. At most one punctuation character may [<i>hang</i>](#hangs) at each edge of the line.

A [<i>hanging</i>](#hanging) punctuation mark is still enclosed inside its inline box and participates in text justification: its character advance width is just not measured when determining how much content fits on the line, how much the line's contents need to be expanded or compressed for justification, or how to position the content within the line box for text alignment.

<a id="stops-and-commas"></a>Stops and commas allowed to [<i>hang</i>](#hangs) include:

|        |     |                                 |
|--------|-----|---------------------------------|
| U+002C | ,   | COMMA                           |
| U+002E | .   | FULL STOP                       |
| U+060C | ،   | ARABIC COMMA                    |
| U+06D4 | ۔   | ARABIC FULL STOP                |
| U+3001 | 、  | IDEOGRAPHIC COMMA               |
| U+3002 | 。  | IDEOGRAPHIC FULL STOP           |
| U+FF0C | ，  | FULLWIDTH COMMA                 |
| U+FF0E | ．  | FULLWIDTH FULL STOP             |
| U+FE50 | ﹐  | SMALL COMMA                     |
| U+FE51 | ﹑  | SMALL IDEOGRAPHIC COMMA         |
| U+FE52 | ﹒  | SMALL FULL STOP                 |
| U+FF61 | ｡   | HALFWIDTH IDEOGRAPHIC FULL STOP |
| U+FF64 | ､   | HALFWIDTH IDEOGRAPHIC COMMA     |

The UA may include other characters as appropriate.

> <strong data-conversion-semantic="note">Note</strong>
>
> The CSS Working Group would appreciate if UAs including other characters would [inform the working group](#status) of such additions.

Support for this property is <em>optional</em>. It is recommended for UAs that wish to support CJK typography, particularly those in the Japanese market.

> <strong data-conversion-semantic="example">Example</strong>
>
> The ‘`allow-end`’ and ‘`force-end`’ are two variations of hanging punctuation used in East Asia.
>
> ![hanging-punctuation: allow-end](https://www.w3.org/TR/2012/WD-css3-text-20121113/hanging-punctuation-allow-end.png)
>
> ```text
> p {
>    text-align: justify;
>    hanging-punctuation: allow-end;
> }
> ```
>
> ![hanging-punctuation: force-end](https://www.w3.org/TR/2012/WD-css3-text-20121113/hanging-punctuation-force-end.png)
>
> ```text
> p {
>    text-align: justify;
>    hanging-punctuation: force-end;
> }
> ```
>
> The punctuation at the end of the first line for ‘`allow-end`’ does not hang, because it fits without hanging. However, if ‘`force-end`’ is used, it is forced to hang. The justification measures the line without the hanging punctuation. Therefore when the line is expanded, the punctuation is pushed outside the line.

## <a id="conformance"></a>10.  Conformance

### <a id="conventions"></a>10.1.  Document Conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#RFC2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

### <a id="conformance-classes"></a>10.2.  Conformance Classes

Conformance to CSS Text Level 3 is defined for three conformance classes:

<a id="style-sheet"></a>style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

<a id="renderer"></a>renderer  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

<a id="authoring-tool"></a>authoring tool  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to CSS Text Level 3 if all of its declarations that use properties defined in this module have values that are valid according to the generic CSS grammar and the individual grammars of each property as given in this module.

A renderer is conformant to CSS Text Level 3 if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by CSS Text Level 3 by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to CSS Text Level 3 if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="partial"></a>10.3.  Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

### <a id="experimental"></a>10.4.  Experimental Implementations

To avoid clashes with future CSS features, the CSS2.1 specification reserves a [prefixed syntax](https://www.w3.org/TR/CSS21/syndata.html#vendor-keywords) for proprietary and experimental extensions to CSS.

Prior to a specification reaching the Candidate Recommendation stage in the W3C process, all implementations of a CSS feature are considered experimental. The CSS Working Group recommends that implementations use a vendor-prefixed syntax for such features, including those in W3C Working Drafts. This avoids incompatibilities with future changes in the draft.

### <a id="testing"></a>10.5. Non-Experimental Implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group's website at [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

### <a id="cr-exit-criteria"></a>10.6.  CR Exit Criteria

For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:

independent  
each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.

interoperable  
passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.

implementation  
a user agent which:

1.  implements the specification.
2.  is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or “nightly build”). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
3.  is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).

The specification will remain Candidate Recommendation for at least six months.

## <a id="acknowledgements"></a> Appendix A: Acknowledgements

This specification would not have been possible without the help from: Ayman Aldahleh, Bert Bos, Tantek Çelik, Stephen Deach, John Daggett, Martin Dürst, Laurie Anna Edlund, Ben Errez, Yaniv Feinberg, Arye Gittelman, Ian Hickson, Martin Heijdra, Richard Ishida, Masayasu Ishikawa, Michael Jochimsen, Eric LeVine, Ambrose Li, Håkon Wium Lie, Chris Lilley, Ken Lunde, Nat McCully, Shinyu Murakami, Paul Nelson, Chris Pratley, Marcin Sawicki, Arnold Schrijver, Rahul Sonnad, Michel Suignard, Takao Suzuki, Frank Tang, Chris Thrasher, Etan Wexler, Chris Wilson, Masafumi Yabe and Steve Zilles.

## <a id="appendix-b-references"></a>Appendix B: References

### <a id="normative-ref"></a>Normative references

<a id="CSS21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification.](css2--REC-CSS2-20110607--1e43327015ed.md) 7 June 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-CSS2-20110607](css2--REC-CSS2-20110607--1e43327015ed.md)

<a id="CSS3-FONTS"></a>\[CSS3-FONTS\]  
John Daggett. [CSS Fonts Module Level 3.](https://www.w3.org/TR/2012/WD-css3-fonts-20120823/) 23 August 2012. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2012&#x2F;WD-css3-fonts-20120823&#x2F;](https://www.w3.org/TR/2012/WD-css3-fonts-20120823/)

<a id="CSS3-WRITING-MODES"></a>\[CSS3-WRITING-MODES\]  
Elika J. Etemad; Koji Ishii. [CSS Writing Modes Module Level 3.](https://www.w3.org/TR/2012/WD-css3-writing-modes-20120501/) 1 May 2012. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2012&#x2F;WD-css3-writing-modes-20120501&#x2F;](https://www.w3.org/TR/2012/WD-css3-writing-modes-20120501/)

<a id="RFC2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels.](http://www.ietf.org/rfc/rfc2119.txt) Internet RFC 2119. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;ietf&#x2E;org&#x2F;rfc&#x2F;rfc2119&#x2E;txt](http://www.ietf.org/rfc/rfc2119.txt)

<a id="UAX11"></a>\[UAX11\]  
Asmus Freytag. [East Asian Width.](http://www.unicode.org/reports/tr11/) 17 January 2012. Unicode Standard Annex \#11. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr11&#x2F;](http://www.unicode.org/reports/tr11/)

<a id="UAX14"></a>\[UAX14\]  
Asmus Freytag. [Line Breaking Properties.](http://www.unicode.org/unicode/reports/tr14/) 17 January 2012. Unicode Standard Annex \#14. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;unicode&#x2F;reports&#x2F;tr14&#x2F;](http://www.unicode.org/unicode/reports/tr14/)

<a id="UAX29"></a>\[UAX29\]  
Mark Davis. [Unicode Text Segmentation.](http://www.unicode.org/reports/tr29/) 24 January 2012. Unicode Standard Annex \#29. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;](http://www.unicode.org/reports/tr29/)

<a id="UAX44"></a>\[UAX44\]  
Mark Davis; Ken Whistler. [Unicode Character Database.](http://www.unicode.org/reports/tr44/) 23 January 2012. Unicode Standard Annex \#44. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr44&#x2F;](http://www.unicode.org/reports/tr44/)

<a id="UNICODE"></a>\[UNICODE\]  
The Unicode Consortium. [The Unicode Standard.](http://www.unicode.org/standard/versions/enumeratedversions.html) 2003. Defined by: The Unicode Standard, Version 4.0 (Boston, MA, Addison-Wesley, ISBN 0-321-18578-1), as updated from time to time by the publication of new versions URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;standard&#x2F;versions&#x2F;enumeratedversions&#x2E;html](http://www.unicode.org/standard/versions/enumeratedversions.html)

### <a id="informative-ref"></a>Informative references

<a id="CSS3-TEXT-DECOR"></a>\[CSS3-TEXT-DECOR\]  
Elika J. Etemad; Koji Ishii. [CSS Text Decoration Module Level 3.](https://www.w3.org/TR/2012/WD-css-text-decor-3-20121113/) 13 November 2012. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2012&#x2F;WD-css-text-decor-3-20121113&#x2F;](https://www.w3.org/TR/2012/WD-css-text-decor-3-20121113/)

<a id="CSS3COLOR"></a>\[CSS3COLOR\]  
Tantek Çelik; Chris Lilley; L. David Baron. [CSS Color Module Level 3.](https://www.w3.org/TR/2011/REC-css3-color-20110607) 7 June 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-css3-color-20110607](https://www.w3.org/TR/2011/REC-css3-color-20110607)

<a id="CSS3LIST"></a>\[CSS3LIST\]  
Tab Atkins Jr. [CSS Lists and Counters Module Level 3.](https://www.w3.org/TR/2011/WD-css3-lists-20110524) 24 May 2011. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;WD-css3-lists-20110524](https://www.w3.org/TR/2011/WD-css3-lists-20110524)

<a id="CSS3VAL"></a>\[CSS3VAL\]  
Håkon Wium Lie; Tab Atkins; Elika J. Etemad. [CSS Values and Units Module Level 3.](https://www.w3.org/TR/2012/CR-css3-values-20120828/) 28 August 2012. W3C Candidate Recommendation. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2012&#x2F;CR-css3-values-20120828&#x2F;](https://www.w3.org/TR/2012/CR-css3-values-20120828/)

<a id="HTML5"></a>\[HTML5\]  
Ian Hickson. [HTML5.](https://www.w3.org/TR/2011/WD-html5-20110525/) 25 May 2011. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;WD-html5-20110525&#x2F;](https://www.w3.org/TR/2011/WD-html5-20110525/)

<a id="JIS4051"></a>\[JIS4051\]  
Formatting rules for Japanese documents (『日本語文書の組版方法』). Japanese Standards Association. 2004. JIS X 4051:2004. In Japanese

<a id="JLREQ"></a>\[JLREQ\]  
Yasuhiro Anan; et al. [Requirements for Japanese Text Layout.](https://www.w3.org/TR/2012/NOTE-jlreq-20120403/) 3 April 2012. W3C Working Group Note. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2012&#x2F;NOTE-jlreq-20120403&#x2F;](https://www.w3.org/TR/2012/NOTE-jlreq-20120403/)

<a id="XML10"></a>\[XML10\]  
C. M. Sperberg-McQueen; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition).](https://www.w3.org/TR/2008/REC-xml-20081126/) 26 November 2008. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2008&#x2F;REC-xml-20081126&#x2F;](https://www.w3.org/TR/2008/REC-xml-20081126/)

<a id="ZHMARK"></a>\[ZHMARK\]  
标点符号用法 (Punctuation Mark Usage). 1995. 中华人民共和国国家标准

## <a id="changes"></a>Appendix C: Changes

### <a id="recent-changes"></a> Changes from the [August 2012 CSS3 Text WD](https://www.w3.org/TR/2012/WD-css3-text-20120814/)

Major changes include:

- Shifted text decoration chapter to a separate Text Decoration module [\[CSS3-TEXT-DECOR\]](#CSS3-TEXT-DECOR)

Significant details updated:

- Shifted spaces higher in priority than clustered scripts for ‘`inter-cluster`’ value of ‘[`text-justify`](#text-justify0)’.
- Defined line breaking behavior for ruby and atomic inlines.
- Added Korean to Chinese and Japanese in ‘[`line-break`](#line-break1)’ special rules.
- Added missing halfwidth codepoint to ‘[`line-break`](#line-break1)’ rules.

## <a id="default-stylesheet"></a> Appendix D: Default UA Stylesheet

This appendix is informative, and is to help UA developers to implement default stylesheet, but UA developers are free to ignore or change.

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
> /* make list items and option elements align together */
> li, option { text-align: match-parent; }
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> If you find any issues, recommendations to add, or corrections, please send the information to <www-style@w3.org> with `[css3-text]` in the subject line.

## <a id="script-groups"></a>Appendix E: Scripts and Spacing

<em>This appendix is informative (non-normative).</em>

Typographic behavior varies somewhat by language, but varies drastically by writing system. This appendix categorizes some common scripts in Unicode 6.0 according to their justification and spacing behavior. Category descriptions are descriptive, not prescriptive; the determining factor is the prioritization of [<i>expansion opportunities</i>](#expansion-opportunity).

<a id="block-scripts"></a><a id="block-scripts0"></a>block scripts  
CJK and by extension all Wide characters. (See [\[UAX11\]](#UAX11)) The following scripts are included: Bopomofo, Han, Hangul, Hiragana, Katakana, Yi

<a id="clustered-scripts"></a><a id="clustered-scripts0"></a>clustered scripts  
Scripts that have discrete units but do not use spaces between words, such as many Southeast Asian systems. The following scripts are included: Javanese, Khmer, Lao, Myanmar, Thai, <strong data-conversion-semantic="issue">Issue:</strong> This list is likely incomplete. What else fits here?

<a id="connected-scripts"></a><a id="connected-scripts0"></a>connected scripts  
Devanagari, Ogham, and other scripts that use spaces between words and baseline connectors within words. By extension this category also includes any other Indic scripts whose typographic behavior is similar to Devanagari. The following scripts are included: Bengali, Brahmi, Devanagari, Gujarati, Gurmukhi, Kannada, Malayalam, Oriya?, Ogham, Tamil?, Telugu

<a id="cursive-scripts"></a><a id="cursive-scripts0"></a>cursive scripts  
Arabic and similar inherently cursive scripts. The following scripts are included: Arabic, Mongolian, N'Ko, Phags Pa, Syriac

<a id="discrete-scripts"></a><a id="discrete-scripts0"></a>discrete scripts  
Scripts that use spaces or visible word-separating punctuation between words and have discrete, unconnected (in print) units within words. The following scripts are included: Armenian, Bamum?, Braille, Canadian Aboriginal, Cherokee, Coptic, Cyrillic, Deseret, Ethiopic Greek, Hebrew, Kharoshthi, Latin, Lisu, Osmanya, Shavian, Tifinagh, Vai?

UAs should treat unrecognized scripts as <i>discrete</i>.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This listing should ideally be exhaustive wrt Unicode. Please [send](#status) suggestions and corrections to the CSS Working Group.

> <strong data-conversion-semantic="note">Note</strong>
>
> Guidelines for classification consider letter-spacing and justification:
>
> 1.  If the script is cursive and may expand cursively but must not space between letters, it is <i>cursive</i>.
> 2.  If the script primarily flexes word separators, it is either <i>discrete</i> or <i>connected</i>. <i>Discrete</i> scripts can space between letters. <i>Connected</i> scripts must not space between letters (typically because that would break the connections or otherwise look bad).
> 3.  If the script primarily expands equally between its "letters" in native typesettings, it is either <i>block</i> or <i>clustered</i>. The exact classification depends on whether it always spaces when mixed with CJK and sometimes stays together when mixed with Thai and related scripts (<i>block</i>) or sometimes spaces when mixed with CJK and always spaces with Thai (<i>clustered</i>).

## <a id="small-kana"></a>Appendix F: Small Kana

| A         | I         | U         | E         | O         |
|-----------|-----------|-----------|-----------|-----------|
| ぁ U+3041 | ぃ U+3043 | ぅ U+3045 | ぇ U+3047 | ぉ U+3049 |
| ゕ U+3095 |           |           | ゖ U+3096 |           |
|           |           | っ U+3063 |           |           |
| ゃ U+3083 |           | ゅ U+3085 |           | ょ U+3087 |
| ゎ U+308E |           |           |           |           |
| ァ U+30A1 | ィ U+30A3 | ゥ U+30A5 | ェ U+30A7 | ォ U+30A9 |
| ヵ U+30F5 |           | ㇰ U+31F0 | ヶ U+30F6 |           |
|           | ㇱ U+31F1 | ㇲ U+31F2 |           |           |
|           |           | ッ U+30C3 |           | ㇳ U+31F3 |
|           |           | ㇴ U+31F4 |           |           |
| ㇵ U+31F5 | ㇶ U+31F6 | ㇷ U+31F7 | ㇸ U+31F8 | ㇹ U+31F9 |
|           |           | ㇺ U+31FA |           |           |
| ャ U+30E3 |           | ュ U+30E5 |           | ョ U+30E7 |
| ㇻ U+31FB | ㇼ U+31FC | ㇽ U+31FD | ㇾ U+31FE | ㇿ U+31FF |
| ヮ U+30EE |           |           |           |           |
| ｧ U+FF67  | ｨ U+FF68  | ｩ U+FF69  | ｪ U+FF6A  | ｫ U+FF6B  |
|           |           | ｯ U+FF6F  |           |           |
| ｬ U+FF6C  |           | ｭ U+FF6D  |           | ｮ U+FF6E  |

Small Kana

## <a id="appendix-g-text-processing-order-of-oper"></a>Appendix G: Text Processing Order of Operations

The following list defines the order of text operations. (Implementations are not bound to this order as long as the resulting layout is the same.)

1.  [text combination](https://www.w3.org/TR/css3-writing-modes/#text-combine-horizontal) [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES)
2.  [white space processing](#white-space-rules) part I (pre-wrapping)
3.  [text transformation](#text-transform)
4.  [default spacing](#spacing)
5.  [text wrapping](#wrapping) while applying per line:
    1.  [indentation](#text-indent)
    2.  [bidirectional reordering](https://www.w3.org/TR/css3-writing-modes/#text-direction) [\[CSS21\]](#CSS21) / [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES)
    3.  [white space processing](#white-space-rules) part II
    4.  [text orientation](https://www.w3.org/TR/css3-writing-modes/#text-orientation) [\[CSS3-WRITING-MODES\]](#CSS3-WRITING-MODES)
    5.  [font/glyph selection and kerning](https://www.w3.org/TR/css3-fonts/) [\[CSS21\]](#CSS21) / [\[CSS3-FONTS\]](#CSS3-FONTS)
    6.  [hanging punctuation](#hanging-punctuation)
6.  [justification](#justification) (which may affect glyph selection and/or text wrapping, looping back into that step)
7.  [text alignment](#text-align)

## <a id="appendix-h-full-property-index"></a>Appendix H: Full Property Index

| Property                                     | Values                                                                                                      | Initial                              | Applies to                                        | Inh. | Percentages                           | Media  |
|----------------------------------------------|-------------------------------------------------------------------------------------------------------------|--------------------------------------|---------------------------------------------------|------|---------------------------------------|--------|
| [hanging-punctuation](#hanging-punctuation0) | none \| \[ first \|\| \[ force-end \| allow-end \] \|\| last \]                                             | none                                 | inline elements                                   | yes  | N/A                                   | visual |
| [hyphens](#hyphens0)                         | none \| manual \| auto                                                                                      | manual                               | all elements                                      | yes  | N/A                                   | visual |
| [letter-spacing](#letter-spacing0)           | \<spacing-limits\>                                                                                          | normal                               | all elements                                      | yes  | N/A                                   | visual |
| [line-break](#line-break1)                   | auto \| loose \| normal \| strict                                                                           | auto                                 | all elements                                      | yes  | N/A                                   | visual |
| overflow-wrap/word-wrap                      | normal \| break-word                                                                                        | normal                               | all elements                                      | yes  | N/A                                   | visual |
| [tab-size](#tab-size1)                       | \<integer\> \| \<length\>                                                                                   | 8                                    | block containers                                  | yes  | N/A                                   | visual |
| [text-align-last](#text-align-last0)         | auto \| start \| end \| left \| right \| center \| justify                                                  | auto                                 | block containers                                  | yes  | N/A                                   | visual |
| [text-align](#text-align0)                   | \[ \[ start \| end \| left \| right \| center \] \|\| \<string\> \] \| justify \| match-parent \| start end | start                                | block containers                                  | yes  | N/A                                   | visual |
| [text-indent](#text-indent0)                 | \[ \<length\> \| \<percentage\> \] &#x26;&#x26; \[ hanging \|\| each-line \]?             | 0                                    | block containers                                  | yes  | refers to width of containing block   | visual |
| [text-justify](#text-justify0)               | auto \| none \| inter-word \| inter-ideograph \| inter-cluster \| distribute \| kashida                     | auto                                 | block containers and, optionally, inline elements | yes  | N/A                                   | visual |
| [text-transform](#text-transform0)           | none \| capitalize \| uppercase \| lowercase \| full-width                                                  | none                                 | all elements                                      | yes  | N/A                                   | visual |
| [white-space](#white-space0)                 | normal \| pre \| nowrap \| pre-wrap \| pre-line                                                             | not defined for shorthand properties | all elements                                      | yes  | N/A                                   | visual |
| [word-break](#word-break0)                   | normal \| keep-all \| break-all                                                                             | normal                               | all elements                                      | yes  | N/A                                   | visual |
| [word-spacing](#word-spacing0)               | \<spacing-limits\>                                                                                          | normal                               | all elements                                      | yes  | refers to width of the affected glyph | visual |
