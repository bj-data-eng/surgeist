Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Text Module Level 4](https://www.w3.org/TR/2024/WD-css-text-4-20240529/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium . W3C ® liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Text Module Level 4

Source snapshot: https://www.w3.org/TR/2024/WD-css-text-4-20240529/

Snapshot SHA-256: be6e92f95bd4b660c50bcb32f7ada80e0d804ffb71597f765b34f91c6a9325b1

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 3 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Text Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module defines properties for text manipulation and specifies their processing model. It covers line breaking, justification and alignment, white space handling, and text transformation.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-text” in the title, like this: “\[css-text\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-text%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

This module describes the typesetting controls of CSS; that is, the features of CSS that control the translation of source text to formatted, line-wrapped text. Various CSS properties provide control over [case transformation](#transforming), [white space collapsing](#white-space-processing), [text wrapping](#white-space-property), [line breaking rules](#line-breaking) and [hyphenation](#hyphenation), [alignment and justification](#justification), [spacing](#spacing), and [indentation](#edge-effects). See [Additions Since Level 3](#changes-L3) for additions since [Level 3](#biblio-css-text-3).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Font selection is covered in the [CSS Fonts Module](https://www.w3.org/TR/css-fonts-3/). [\[CSS-FONTS-3\]](#biblio-css-fonts-3)
>
> <a id="decoration"></a> <a id="text-decoration"></a> <a id="line-decoration"></a> <a id="text-decoration-line"></a> <a id="text-decoration-color"></a> <a id="text-decoration-style"></a> <a id="text-decoration-skip"></a> <a id="text-underline-position"></a> <a id="emphasis-marks"></a> <a id="text-emphasis-style"></a> <a id="text-emphasis-color"></a> <a id="text-emphasis"></a> <a id="text-emphasis-position"></a> <a id="text-shadow"></a> Features for decorating text, such as [underlines](https://www.w3.org/TR/css-text-decor-3/#line-decoration), [emphasis marks](https://www.w3.org/TR/css-text-decor-3/#emphasis-marks), and [shadows](https://www.w3.org/TR/css-text-decor-3/#text-shadow-property), (previously part of this module) are covered in the [CSS Text Decoration Module](https://www.w3.org/TR/css-text-decor-3/). [\[CSS-TEXT-DECOR-3\]](#biblio-css-text-decor-3)
>
> [Bidirectional](https://www.w3.org/TR/css-writing-modes-4/#text-direction) and [vertical](https://www.w3.org/TR/css-writing-modes-4/#vertical-intro) text are addressed in the [CSS Writing Modes Module](https://www.w3.org/TR/css-writing-modes-4/). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4).

Further information about the typesetting requirements of various languages and writing systems around the world can be found in the [Internationalization Working Group](https://www.w3.org/International/core/)’s [Language Enablement Index](https://www.w3.org/TR/typography/). [\[TYPOGRAPHY\]](#biblio-typography)

### <a id="placement"></a>1.1.  Module Interactions

This module, together with the [CSS Text Decoration Module](https://www.w3.org/TR/css-text-decor-3/), replaces and extends the text-level features defined in [Cascading Style Sheets Level 2 chapter 16](https://www.w3.org/TR/CSS2/text.html). [\[CSS-TEXT-DECOR-3\]](#biblio-css-text-decor-3) [\[CSS2\]](#biblio-css2)

In addition to the terms defined below, other terminology and concepts used in this specification are defined in [Cascading Style Sheets Level 2](https://www.w3.org/TR/CSS2/) and the [CSS Writing Modes Module](https://www.w3.org/TR/css-writing-modes-4/). [\[CSS2\]](#biblio-css2) and [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4).

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="languages"></a>1.3.  Languages and Typesetting

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> <strong>Authors should accurately language-tag their content&#xA;&#x9;for the best typographic behavior.</strong>

<a id="ref-for-content-language"></a>

Many typographic effects vary by linguistic context. Language and writing system conventions can affect line breaking, hyphenation, justification, glyph selection, and many other typographic effects. <strong>In CSS, language-specific typographic tailorings&#xA;&#x9;are only applied when the <a href="#content-language">content language</a> is known (declared).</strong> Therefore, higher quality typography requires authors to communicate to the UA the correct linguistic context of the text in the document.

<a id="ref-for-doclanguage"></a>

<a id="ref-for-content-language①"></a>

The <a id="content-language"></a>content language of an element is the (human) language the element is declared to be in, according to the rules of the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage). Note that it is possible for the [content language](#content-language) of an element to be unknown—​e.g. untagged content, or content in a <a id="ref-for-doclanguage①"></a>document language that does not have a language-tagging facility, is considered to have an unknown <a id="ref-for-content-language②"></a>content language.

<a id="ref-for-content-language③"></a>

<a id="ref-for-language"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors can declare the [content language](#content-language) using the global `lang` attribute in HTML or the universal `xml:lang` attribute in XML. See the [rules for determining the content language of an HTML element](https://html.spec.whatwg.org/multipage/dom.html#language) in HTML, and the [rules for determining the content language of an XML element](https://www.w3.org/TR/xml/#sec-lang-tag) in XML 1.0. [\[HTML\]](#biblio-html) [\[XML10\]](#biblio-xml10)

<a id="ref-for-content-language④"></a>

<a id="ref-for-doclanguage②"></a>

The [content language](#content-language) an element is declared to be in also identifies the specific written form of that language used in that element, known as the <a id="content-writing-system"></a>content writing system. Depending on the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage)’s facilities for identifying the <a id="ref-for-content-language⑤"></a>content language, this information can be explicit or implied. See the normative [Appendix F: Identifying the Content Writing System](#script-tagging).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some languages have more than one writing system tradition; in other cases a language can be transliterated into a foreign writing system. Authors should [subtag](#script-tagging) such cases so that the UA can adapt appropriately.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bcb0a1b4"></a> For example, Korean (`ko`) can be written in Hangul (`-Hang`), Hanja (`-Hani`), or a combination (`-Kore`). Historical documents written solely in Hanja do not use word spaces and are formatted more like modern Chinese than modern Korean. In other words, for typographic purposes `ko-Hani` behaves more like `zh-Hant` than `ko` (`ko-Kore`).
>
> As another example Japanese (`ja`) is typically written in a combination (`-Japn`) of Hiragana (`-Hira`), Katakana (`-Kana`), and Kanji (`-Hani`). However, it can also be “romanized” into Latin (`-Latn`) for special purposes like language-learning textbooks, in which case it should be formatted more like English than Japanese.
>
> As a third example contemporary Mongolian is written in two scripts: Cyrillic (`-Cyrl`, officially used in Mongolia) and Mongolian (`-Mong`, more common in Inner Mongolia, part of China). These have very different formatting requirements, with Cyrillic behaving similar to Latin and Greek, and Mongolian deriving from both Arabic and Chinese writing conventions.

### <a id="characters"></a>1.4.  Characters and Letters

<a id="ref-for-character"></a>

The basic unit of typesetting is the <a id="character"></a>character. However, because writing systems are not always as simple as the basic English alphabet, what a [character](#character) actually is depends on the context in which the term is used. For example, in Hangul (the Korean writing system), each square representation of a syllable (e.g. 한=Han) can be considered a <a id="ref-for-character①"></a>character. However, the square symbol is really composed of multiple letters each representing a phoneme (e.g. ㅎ=h, ㅏ=a, ㄴ=n) and these also could each be considered a <a id="ref-for-character②"></a>character.

<a id="ref-for-character③"></a>

A basic unit of computer text encoding, for any given encoding, is also called a [character](#character), and depending on the encoding, a single encoding <a id="ref-for-character④"></a>character might correspond to the entire pre-composed syllabic <a id="ref-for-character⑤"></a>character (e.g. 한), to the individual phonemic <a id="ref-for-character⑥"></a>character (e.g. ㅎ), or to smaller units such as a base letterform (e.g. ㅇ) and any combining marks that vary it (e.g. extra strokes that represent aspiration).

<a id="ref-for-character⑦"></a>

In turn, a single encoding [character](#character) can be represented in the data stream as one or more bytes; and in programming environments one byte is sometimes also called a <a id="ref-for-character⑧"></a>character.

<a id="ref-for-character⑨"></a>

Therefore the term [character](#character) is fairly ambiguous where technical precision is required.

<a id="ref-for-character①⓪"></a>

<a id="ref-for-typographic-character-unit"></a>

For text layout, we will refer to the <a id="typographic-character-unit"></a>typographic character unit as the basic unit of text. Even within the realm of text layout, the relevant [character](#character) unit depends on the operation. For example, line-breaking and letter-spacing will segment a sequence of Thai characters that include U+0E33  ำ THAI CHARACTER SARA AM differently; or the behavior of a conjunct consonant in a script such as Devanagari may depend on the font in use. So the [typographic character](#typographic-character-unit) represents a unit of the writing system—​such as a Latin alphabetic letter (including its diacritics), Hangul syllable, Chinese ideographic character, Myanmar syllable cluster—​that is indivisible with respect to a particular typographic operation (line-breaking, first-letter effects, tracking, justification, vertical arrangement, etc.).

<a id="ref-for-typographic-character-unit①"></a>

[Unicode Standard Annex \#29: Text Segmentation](http://www.unicode.org/reports/tr29/) defines a unit called the <a id="grapheme-cluster"></a>grapheme cluster which approximates the [typographic character](#typographic-character-unit). [\[UAX29\]](#biblio-uax29) A UA must use the <em>extended grapheme cluster</em> (not <em>legacy grapheme cluster</em>), as defined in UAX29, as the basis for its <a id="ref-for-typographic-character-unit②"></a>typographic character unit. However, the UA should tailor the definitions as required by typographic tradition since the default rules are not always appropriate or ideal—​and is expected to tailor them differently depending on the operation as needed.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The rules for such tailorings are out of scope for CSS.

<a id="ref-for-typographic-character-unit③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8c80c59b"></a> The following are some examples of [typographic character unit](#typographic-character-unit) tailorings required by standard typesetting practice:
>
> - <a id="ref-for-grapheme-cluster"></a>
>
>   <a id="ref-for-typographic-character-unit④"></a>
>
>   In some scripts such as Myanmar or Devanagari, the [typographic character unit](#typographic-character-unit) for both justification and line-breaking is an entire syllable, which can include more than one Unicode [grapheme cluster](#grapheme-cluster). [\[UAX29\]](#biblio-uax29)
>
> - <a id="ref-for-grapheme-cluster①"></a>
>
>   <a id="ref-for-typographic-character-unit⑤"></a>
>
>   In other scripts such as Thai or Lao, even though for line-breaking the [typographic character](#typographic-character-unit) matches Unicode’s default [grapheme clusters](#grapheme-cluster), for letter-spacing the relevant unit is <em>less</em> than a Unicode <a id="ref-for-grapheme-cluster②"></a>grapheme cluster, and may require decomposition or other substitutions before spacing can be inserted. [\[UAX29\]](#biblio-uax29)
>
>   For instance, to properly letter-space the Thai word คำ (U+0E04 + U+0E33), the U+0E33 needs to be decomposed into U+0E4D + U+0E32, and then the extra letter-space inserted before the U+0E32: คํ า.
>
>   A slightly more complex example is น&#xE49;ำ (U+0E19 + U+0E49 + U+0E33). In this case, normal Thai shaping will first decompose the U+0E33 into U+0E4D + U+0E32 and then swap the U+0E4D with the U+0E49, giving U+0E19 + U+0E4D + U+0E49 + U+0E32. As before the extra letter-space is then inserted before the U+0E32: นํ&#xE49; า.
>
> - <a id="ref-for-typographic-character-unit⑥"></a>
>
>   <a id="ref-for-valdef-text-orientation-upright"></a>
>
>   Vertical typesetting can also require tailoring. For example, when typesetting [upright](https://www.w3.org/TR/css-writing-modes-4/#valdef-text-orientation-upright) text, Tibetan tsek and shad marks are kept with the preceding grapheme cluster, rather than treated as an independent [typographic character unit](#typographic-character-unit). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

<a id="ref-for-typographic-character-unit⑦"></a>

<a id="ref-for-unicode-general-category"></a>

A <a id="typographic-letter-unit"></a>typographic letter unit (or <a id="letter"></a>letter for the purpose of this specification) is a [typographic character unit](#typographic-character-unit) belonging to one of the Letter or Number [general categories](#unicode-general-category). See [Appendix E: Characters and Properties](#character-properties) for how to determine the Unicode properties of a <a id="ref-for-typographic-character-unit⑧"></a>typographic character unit.

<a id="ref-for-typographic-character-unit⑨"></a>

<a id="ref-for-grapheme-cluster③"></a>

The rendering characteristics of a [typographic character unit](#typographic-character-unit) divided by an element boundary is undefined. Ideally each component should be rendered according to the formatting requirements of its respective element’s properties while maintaining correct shaping and positioning of the <a id="ref-for-typographic-character-unit①⓪"></a>typographic character unit as a whole. However, depending on the nature of the formatting differences between its parts and the capabilities of the font technology in use, this is not always possible. Therefore such a <a id="ref-for-typographic-character-unit①①"></a>typographic character unit may be rendered as belonging to either side of the boundary, or as some approximation of belonging to both. Authors are forewarned that dividing [grapheme clusters](#grapheme-cluster) or ligatures by element boundaries may give inconsistent or undesired results.

### <a id="text-encoding"></a>1.5.  Text Processing

CSS is built on [Unicode](https://www.unicode.org/versions/latest/). [\[UNICODE\]](#biblio-unicode) UAs that support Unicode must adhere to all normative requirements of the Unicode Core Standard, except where explicitly overridden by CSS. UAs implemented on the basis of a non-Unicode text encoding model are still expected to fulfill the same text handling requirements by assuming an appropriate mapping and analogous behavior.

<a id="ref-for-inline-box"></a>

<a id="ref-for-out-of-flow"></a>

For the purpose of determining adjacency for text processing (such as white space processing, text transformation, line-breaking, etc.), and thus in general within this specification, intervening [inline box](https://www.w3.org/TR/css-display-3/#inline-box) boundaries and [out-of-flow](https://www.w3.org/TR/css-display-3/#out-of-flow) elements must be ignored. With respect to text shaping, however, see [§ 8.7 Shaping Across Element Boundaries](#boundary-shaping).

## <a id="transforming"></a>2.  Transforming Text

<a id="ref-for-propdef-text-transform"></a>

### <a id="text-transform-property"></a>2.1.  Case Transforms: the [text-transform](#propdef-text-transform) property<a id="text-transform"></a><a id="caps-prop"></a>



| Field               | Definition                                                                                                                                                                                                                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-transform"></a>text-transform                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[capitalize <a id="ref-for-comb-one①"></a>\| uppercase <a id="ref-for-comb-one②"></a>\| lowercase \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) full-width <a id="ref-for-comb-any①"></a>\|\| full-size-kana <a id="ref-for-comb-one③"></a>\| math-auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                             |



This property transforms text for styling purposes. It has no effect on the underlying content, and must not affect the content of a plain text copy &#x26; paste operation.

<a id="ref-for-propdef-text-transform①"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors must not rely on <a href="#propdef-text-transform">text-transform</a> for semantic purposes;&#xA;&#x9;rather the correct casing and semantics should be encoded&#xA;&#x9;in the source document text and markup.</strong>

Values have the following meanings:

<a id="valdef-text-transform-none"></a>none  
No effects.

<a id="valdef-text-transform-capitalize"></a>capitalize  
<a id="ref-for-typographic-letter-unit"></a>

Puts the first [typographic letter unit](#typographic-letter-unit) of each word, if lowercase, in titlecase; other characters are unaffected.

<a id="valdef-text-transform-uppercase"></a>uppercase  
<a id="ref-for-letter"></a>

Puts all [letters](#letter) in uppercase.

<a id="valdef-text-transform-lowercase"></a>lowercase  
<a id="ref-for-letter①"></a>

Puts all [letters](#letter) in lowercase.

<a id="valdef-text-transform-full-width"></a>full-width  
<a id="ref-for-full-width"></a>

<a id="ref-for-typographic-character-unit①②"></a>

Puts all [typographic character units](#typographic-character-unit) in [full-width](#full-width) form. If a character does not have a corresponding <a id="ref-for-full-width①"></a>full-width form, it is left as is. This value is typically used to typeset Latin letters and digits as if they were ideographic characters.

<a id="valdef-text-transform-full-size-kana"></a>full-size-kana  
<a id="ref-for-kana-full-size"></a>

<a id="ref-for-kana-small"></a>

Converts all [small Kana](#kana-small) characters to the equivalent [full-size Kana](#kana-full-size). This value is typically used for ruby annotation text, where authors may want all small Kana to be drawn as large Kana to compensate for legibility issues at the small font sizes typically used in ruby.

<a id="valdef-text-transform-math-auto"></a>math-auto  
See [MathML Core § 4.2 New text-transform value](https://www.w3.org/TR/mathml-core/#new-text-transform-values).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5b7dad2f"></a> The following example converts the ASCII characters used in abbreviations in Japanese text to their full-width variants so that they lay out and line break like ideographs:
>
> ```text
> abbr:lang(ja) { text-transform: full-width; }
> ```
<a id="ref-for-propdef-text-transform②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The purpose of [text-transform](#propdef-text-transform) is to allow for presentational casing transformations without affecting the semantics of the document. Note in particular that <a id="ref-for-propdef-text-transform③"></a>text-transform casing operations are lossy, and can distort the meaning of a text. While accessibility interfaces may wish to convey the apparent casing of the rendered text to the user, the transformed text cannot be relied on to accurately represent the underlying meaning of the document.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3f5b224"></a> In this example, the first line of text is capitalized as a visual effect.
>
> ```text
> section > p:first-of-type::first-line {
>   text-transform: uppercase;
> }
> ```
>
> This effect cannot be written into the source document because the position of the line break depends on layout. But also, the capitalization is not reflecting a semantic distinction and is not intended to affect the paragraph’s reading; therefore it belongs in the presentation layer.

<a id="ref-for-ruby"></a>

<a id="ref-for-kana-small①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-65db0dde"></a> In this example, the [ruby](https://www.w3.org/TR/css-ruby-1/#ruby) annotations, which are half the size of the main paragraph text, are transformed to use regular-size kana in place of [small kana](#kana-small).
>
> ```text
> rt { font-size: 50%; text-transform: full-size-kana; }
> :is(h1, h2, h3, h4) rt { text-transform: none; /* unset for large text*/ }
> ```
>
> <a id="ref-for-kana-small②"></a>
>
> Note that while this makes such letters easier to see at small type sizes, the transformation distorts the text: the reader needs to mentally substitute [small kana](#kana-small) in the appropriate places—​not unlike reading a Latin inscription where all “U”s look like “V”s.
>
> <a id="ref-for-propdef-text-transform④"></a>
>
> For example, if [text-transform: full-size-kana](#propdef-text-transform) were applied to the following source, the annotation would read “じゆう” (jiyū), which means “liberty”, instead of “じゅう” (jū), which means “ten”, the correct reading and meaning for the annotated “十”.
>
> ```text
> <ruby>十<rt>じゅう</ruby>
> ```
#### <a id="text-transform-mapping"></a>2.1.1.  Mapping Rules

<a id="ref-for-valdef-text-transform-capitalize"></a>

<a id="ref-for-propdef-text-transform⑤"></a>

For [capitalize](#valdef-text-transform-capitalize), what constitutes a “word“ is UA-dependent; [\[UAX29\]](#biblio-uax29) is suggested (but not required) for determining such word boundaries. Out-of-flow elements and inline element boundaries must not introduce a [text-transform](#propdef-text-transform) word boundary and must be ignored when determining such word boundaries.

<a id="ref-for-valdef-text-transform-capitalize①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors cannot depend on [capitalize](#valdef-text-transform-capitalize) to follow language-specific titlecasing conventions (such as skipping articles in English).

<a id="ref-for-content-language⑥"></a>

<a id="ref-for-doclanguage③"></a>

The UA must use the full case mappings for Unicode characters, including any conditional casing rules, as defined in the Default Case Algorithms section of The Unicode Standard. [\[UNICODE\]](#biblio-unicode) If (and only if) the [content language](#content-language) of the element is, according to the rules of the [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage), known, then any appropriate language-specific rules must be applied as well. These minimally include, but are not limited to, the language-specific rules in Unicode’s [SpecialCasing.txt](http://www.unicode.org/Public/UNIDATA/SpecialCasing.txt).

<a id="ref-for-content-language⑦"></a>

<a id="ref-for-content-writing-system"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f454e7b9"></a> For example, in Turkish there are two “i”s, one with a dot—​“İ” and “i”—​and one without—​“I” and “ı”. Thus the usual case mappings between “I” and “i” are replaced with a different set of mappings to their respective dotless/dotted counterparts, which do not exist in English. This mapping must only take effect if the [content language](#content-language) is Turkish written in its modern Latin-based [writing system](#content-writing-system) (or another Turkic language that uses Turkish casing rules); in other languages, the usual mapping of “I” and “i” is required. This rule is thus conditionally defined in Unicode’s SpecialCasing.txt file.

<a id="ref-for-full-width②"></a>

The definition of <a id="full-width"></a>full-width and <a id="half-width"></a>half-width forms can be found in [Unicode Standard Annex \#11: East Asian Width](https://www.unicode.org/reports/tr11/). [\[UAX11\]](#biblio-uax11) The mapping to [full-width](#full-width) form is defined by taking code points with the `<wide>` or the `<narrow>` tag in their `Decomposition_Mapping` in [Unicode Standard Annex \#44: Unicode Character Database](https://www.unicode.org/reports/tr44/). [\[UAX44\]](#biblio-uax44) For the `<narrow>` tag, the mapping is from the code point to the decomposition (minus `<narrow>` tag), and for the `<wide>` tag, the mapping is from the decomposition (minus the `<wide>` tag) back to the original code point.

<a id="ref-for-kana-small③"></a>

<a id="ref-for-kana-full-size①"></a>

The mappings for [small Kana](#kana-small) to [full-size Kana](#kana-full-size) are defined in [Appendix G: Small Kana Mappings](#small-kana).

<a id="ref-for-propdef-word-space-transform"></a>

### <a id="word-space-transform"></a>2.2.  Expanding Between Words: the [word-space-transform](#propdef-word-space-transform) property<a id="word-boundary-expansion"></a>



| Field               | Definition                                                                                                                                                                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-word-space-transform"></a>word-space-transform                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-comb-all"></a><a id="ref-for-comb-one④"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ space <a id="ref-for-comb-one⑤"></a>\| ideographic-space \] [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) auto-phrase[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                         |



Some languages and writing systems have alternative ways of delimiting words, either using different separating characters, or sometimes no visible character at all. This property allows authors to change the rendering from one style to another without needing to change the markup.

<a id="valdef-word-space-transform-none"></a>none  
This property has no effect.

<a id="valdef-word-space-transform-space"></a>space  
<a id="ref-for-css-text-sequence"></a>

<a id="ref-for-expandable-separators"></a>

[Expandable separators](#expandable-separators) within the child [text](https://www.w3.org/TR/css-display-3/#css-text-sequence) of this element are replaced by U+0020 SPACE.

<a id="valdef-word-space-transform-ideographic-space"></a>ideographic-space  
<a id="ref-for-css-text-sequence①"></a>

<a id="ref-for-expandable-separators①"></a>

[Expandable separators](#expandable-separators) within the child [text](https://www.w3.org/TR/css-display-3/#css-text-sequence) of this element are replaced by U+3000 IDEOGRAPHIC SPACE.

<a id="valdef-word-space-transform-auto-phrase"></a>auto-phrase  
<a id="ref-for-virtual-expandable-separator"></a>

<a id="ref-for-other-space-separators"></a>

<a id="ref-for-word-separator"></a>

<a id="ref-for-phrase-boundary-detection"></a>

<a id="ref-for-content-language⑧"></a>

If the [content language](#content-language) is known and the user agent supports linguistic analysis for this language, the user agent must [detect phrase boundaries](#phrase-boundary-detection). If a [word-separator character](#word-separator), [other space separator](#other-space-separators), or U+200B ZERO WIDTH SPACE character does not already occur at that boundary, then the UA must insert a [virtual expandable separator](#virtual-expandable-separator).

<a id="ref-for-content-language⑨"></a>

<a id="ref-for-phrase-boundary-detection①"></a>

<a id="ref-for-virtual-expandable-separator①"></a>

If this value is omitted, or if the [content language](#content-language) is unknown, or if the user agent does not support [detecting phrase boundaries](#phrase-boundary-detection) for that language, there are no [virtual expandable separator](#virtual-expandable-separator).

For the purpose of this property, <a id="expandable-separators"></a>expandable separators are any of:

- U+200B ZERO WIDTH SPACE characters

- <a id="ref-for-the-wbr-element"></a>

  <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> elements

- <a id="ref-for-virtual-expandable-separator②"></a>

  [virtual expandable separators](#virtual-expandable-separator)

<a id="ref-for-expandable-separators②"></a>

A <a id="virtual-expandable-separator"></a>virtual expandable separator is a UA-detected syntactic boundary in the text that represents an [expandable separator](#expandable-separators) not otherwise occuring in the source document. It has no effect other than for this property.

<a id="ref-for-expandable-separators③"></a>

<a id="ref-for-forced-line-break"></a>

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-propdef-border"></a>

<a id="ref-for-propdef-padding"></a>

The user agent must not replace [expandable separators](#expandable-separators) immediately preceding or following a [forced line break](#forced-line-break) (ignoring any intervening inline box boundaries, and associated [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)/[border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border)/[padding](https://www.w3.org/TR/css-box-4/#propdef-padding)).

<a id="ref-for-virtual-expandable-separator③"></a>

<a id="ref-for-used-value"></a>

<a id="ref-for-propdef-word-space-transform①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [virtual expandable separators](#virtual-expandable-separator) [are placed in the outermost element that participates in an inline box boundary](#boundary-outermost), if one would coincide with boundary of an inline box whose parent box has a [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of [word-space-transform: none](#propdef-word-space-transform), that particular <a id="ref-for-virtual-expandable-separator④"></a>virtual expandable separator is not expanded, since it would be placed in the parent box.

<a id="ref-for-propdef-text-transform⑥"></a>

Like [text-transform](#propdef-text-transform), this property transforms text for styling purposes only. It has no effect on the underlying content, and must not affect the content of a plain text copy &#x26; paste operation.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="wakachigaki"></a>
>
> Unlike books for adults, Japanese books for young children often feature spaces between sentence segments, to facilitate reading. People with dyslexia also tend to find this style easier to read.
>
> Absent any particular styling, the following sentence would be rendered as depicted below.
>
> ```text
> <p>むかしむかし、<wbr>あるところに、<wbr>おじいさんと<wbr>おばあさんが<wbr>すんでいました。
> ```
>
> `  むかしむかし、あるところに、おじいさんとおばあさんがすんでいました。  `
>
> ------------------------------------------------------------------------
>
> Phrase-based spacing can be achieved with the following css:
>
> ```text
> p {
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  むかしむかし、　あるところに、　おじいさんと　おばあさんが　すんでいました。  `
>
> ------------------------------------------------------------------------
>
> Another common variant additionally restricts the allowable line breaks to these phrase boundaries. Using the same markup, this is easily achieved with the following css:
>
> ```text
> p {
>   word-break: keep-all;
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  むかしむかし、　あるところに、　おじいさんと　おばあさんが　すんでいました。  `

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="classes"></a>
>
> <a id="ref-for-the-wbr-element①"></a>
>
> In addition to making the source code more readable, using <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> rather than U+200B in the markup also allow authors to classify the delimiters into different groups.
>
> <a id="ref-for-the-wbr-element②"></a>
>
> In the following example, <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> elements are either unmarked when they delimit a word, or marked with class `p` when they also delimit a phrase.
>
> ```text
> <p>らいしゅう<wbr>の<wbr>じゅぎょう<wbr>に<wbr class=p
> >たいこ<wbr>と<wbr>ばち<wbr>を<wbr class=p
> >もって<wbr>きて<wbr>ください。
> ```
>
> Using this, it is possible not only to enable the rather common phrase-based spacing, but also word-by-word spacing that is likely to be preferred by people with dyslexia to reduce ambiguities, or other variants such as a combination of phrase-based spacing and of word-based wrapping.
>
> Usual rendering
>
> `  らいしゅうのじゅぎょうにたいことばちをもってきてください。  `
>
> ------------------------------------------------------------------------
>
> Phrase spacing
>
> ```text
> p wbr.p {
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  らいしゅうのじゅぎょうに　たいことばちを　もってきてください。  `
>
> ------------------------------------------------------------------------
>
> Word spacing
>
> ```text
> p wbr {
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  らいしゅう　の　じゅぎょう　に　たいこ　と　ばち　を　もって　きて　ください。  `
>
> ------------------------------------------------------------------------
>
> Phrase spacing, word wrapping
>
> ```text
> p {
>   word-break: keep-all;
> }
> p wbr.p {
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  らいしゅうのじゅぎょうに　たいことばちを　もってきてください。  `
>
> ------------------------------------------------------------------------
>
> Word spacing and wrapping
>
> ```text
> p {
>   word-break: keep-all;
> }
> p wbr {
>   word-space-transform: ideographic-space;
> }
> ```
>
> `  らいしゅう　の　じゅぎょう　に　たいこ　と　ばち　を　もって　きて　ください。  `

### <a id="text-transform-order"></a>2.3.  Order of Operations

When multiple transformations need to be applied, they are applied in the following order:

1.  <a id="ref-for-propdef-word-space-transform②"></a>

    [word-space-transform](#propdef-word-space-transform)

2.  <a id="ref-for-valdef-text-transform-lowercase"></a>

    <a id="ref-for-valdef-text-transform-uppercase"></a>

    <a id="ref-for-valdef-text-transform-capitalize②"></a>

    [capitalize](#valdef-text-transform-capitalize), [uppercase](#valdef-text-transform-uppercase), and [lowercase](#valdef-text-transform-lowercase)

3.  <a id="ref-for-valdef-text-transform-full-width"></a>

    [full-width](#valdef-text-transform-full-width)

4.  <a id="ref-for-valdef-text-transform-full-size-kana"></a>

    [full-size-kana](#valdef-text-transform-full-size-kana)

<a id="ref-for-valdef-text-transform-full-width①"></a>

<a id="ref-for-preserved-white-space"></a>

<a id="ref-for-white-space"></a>

Word space transformation and text transformation happen after [§ 4.3.1 Phase I: Collapsing and Transformation](#white-space-phase-1) but before [§ 4.3.2 Phase II: Trimming and Positioning](#white-space-phase-2). This means for instance that [full-width](#valdef-text-transform-full-width) only transforms spaces (U+0020) to U+3000 IDEOGRAPHIC SPACE within [preserved](#preserved-white-space) [white space](#white-space).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As defined in [Appendix A: Text Processing Order of Operations](#order), transforming affects line-breaking and other formatting operations.

<a id="ref-for-propdef-white-space"></a>

## <a id="white-space-property"></a>3.  White Space and Wrapping: the [white-space](#propdef-white-space) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-white-space"></a>white-space                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-white-space-trim"></a><a id="ref-for-propdef-text-wrap-mode"></a><a id="ref-for-comb-any②"></a><a id="ref-for-propdef-white-space-collapse"></a><a id="ref-for-comb-one⑥"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) pre <a id="ref-for-comb-one⑦"></a>\| pre-wrap <a id="ref-for-comb-one⑧"></a>\| pre-line <a id="ref-for-comb-one⑨"></a>\| [\<'white-space-collapse'\>](#propdef-white-space-collapse) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'text-wrap-mode'\>](#propdef-text-wrap-mode) <a id="ref-for-comb-any③"></a>\|\| [\<'white-space-trim'\>](#propdef-white-space-trim) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |



<a id="ref-for-propdef-white-space-collapse①"></a>

<a id="ref-for-propdef-text-wrap-mode①"></a>

<a id="ref-for-propdef-white-space-trim①"></a>

This property is a shorthand for [white-space-collapse](#propdef-white-space-collapse), [text-wrap-mode](#propdef-text-wrap-mode), and [white-space-trim](#propdef-white-space-trim). It specifies two things:

- <a id="ref-for-white-space①"></a>

  whether and how [white space](#white-space) is collapsed; see [White Space Processing](#white-space-processing)

- <a id="ref-for-soft-wrap-opportunity"></a>

  <a id="ref-for-wrapping"></a>

  whether lines may [wrap](#wrapping) at unforced [soft wrap opportunities](#soft-wrap-opportunity); see [§ 5 Text Wrapping](#text-wrapping) and [Line Breaking](#line-breaking)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This shorthand combines both inheritable and non-inheritable properties. If this is a problem, please inform the CSSWG.

<a id="ref-for-longhand"></a>

<a id="ref-for-initial-value"></a>

Unless otherwise specified, any omitted [longhand](https://www.w3.org/TR/css-cascade-5/#longhand) is set to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-longhand①"></a>

The following table gives the normative mapping of the values of the [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property)’s special keywords to their equivalent [longhand](https://www.w3.org/TR/css-cascade-5/#longhand) values.



| <a id="ref-for-propdef-white-space①"></a>[white-space](#propdef-white-space) | <a id="ref-for-propdef-white-space-collapse②"></a>[white-space-collapse](#propdef-white-space-collapse)           | <a id="ref-for-propdef-text-wrap-mode②"></a>[text-wrap-mode](#propdef-text-wrap-mode) | <a id="ref-for-propdef-white-space-trim②"></a>[white-space-trim](#propdef-white-space-trim) |
|--------------------------------------------------------|------------------------------------------------------------------------------------|--------------------------------------------------------------|------------------------------------------------------------------|
| <strong><dfn><span><a id="valdef-white-space-normal"></a></span>normal</dfn> &#xA;      </strong>                                    | <a id="ref-for-valdef-white-space-collapse-collapse"></a>[collapse](#valdef-white-space-collapse-collapse)               | <a id="ref-for-valdef-text-wrap-mode-wrap"></a>[wrap](#valdef-text-wrap-mode-wrap)       | none                                                             |
| <strong><dfn><span><a id="valdef-white-space-pre"></a></span>pre</dfn> &#xA;      </strong>                                    | <a id="ref-for-valdef-white-space-collapse-preserve"></a>[preserve](#valdef-white-space-collapse-preserve)               | <a id="ref-for-valdef-text-wrap-mode-nowrap"></a>[nowrap](#valdef-text-wrap-mode-nowrap)   | none                                                             |
| <strong><dfn><span><a id="valdef-white-space-pre-wrap"></a></span>pre-wrap</dfn> &#xA;      </strong>                                    | <a id="ref-for-valdef-white-space-collapse-preserve①"></a>[preserve](#valdef-white-space-collapse-preserve)               | <a id="ref-for-valdef-text-wrap-mode-wrap①"></a>[wrap](#valdef-text-wrap-mode-wrap)       | none                                                             |
| <strong><dfn><span><a id="valdef-white-space-pre-line"></a></span>pre-line</dfn> &#xA;      </strong>                                    | <a id="ref-for-valdef-white-space-collapse-preserve-breaks"></a>[preserve-breaks](#valdef-white-space-collapse-preserve-breaks) | <a id="ref-for-valdef-text-wrap-mode-wrap②"></a>[wrap](#valdef-text-wrap-mode-wrap)       | none                                                             |



<a id="ref-for-preserved-white-space①"></a>

<a id="ref-for-other-space-separators①"></a>

<a id="ref-for-hang"></a>

<a id="ref-for-intrinsic-sizing"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases, [preserved white space](#preserved-white-space) and [other space separators](#other-space-separators) can [hang](#hang) when at the end of the line; this can affect whether they are measured for [intrinsic sizing](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizing).

<a id="ref-for-propdef-white-space②"></a>

The following informative table summarizes the behavior of various [white-space](#propdef-white-space) values:



|                     | New Lines | Spaces and Tabs | Text Wrapping | <a id="ref-for-spaces"></a>End-of-line [spaces](#spaces) | <a id="ref-for-other-space-separators②"></a>End-of-line [other space separators](#other-space-separators) |
|---------------------|-----------|-----------------|---------------|--------------------------------------------------|----------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-valdef-white-space-normal"></a></span><a href="#valdef-white-space-normal">normal</a> &#xA;      </strong> | Collapse  | Collapse        | Wrap          | Remove                                           | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre"></a></span><a href="#valdef-white-space-pre">pre</a> &#xA;      </strong> | Preserve  | Preserve        | No wrap       | Preserve                                         | No wrap                                                                          |
| <strong><span><a id="ref-for-valdef-white-space-nowrap"></a></span><a href="https://www.w3.org/TR/css-text-3/#valdef-white-space-nowrap">nowrap</a> &#xA;      </strong> | Collapse  | Collapse        | No wrap       | Remove                                           | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre-wrap"></a></span><a href="#valdef-white-space-pre-wrap">pre-wrap</a> &#xA;      </strong> | Preserve  | Preserve        | Wrap          | Hang                                             | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-collapse-break-spaces"></a></span><a href="#valdef-white-space-collapse-break-spaces">break-spaces</a> &#xA;      </strong> | Preserve  | Preserve        | Wrap          | Wrap                                             | Wrap                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre-line"></a></span><a href="#valdef-white-space-pre-line">pre-line</a> &#xA;      </strong> | Preserve  | Collapse        | Wrap          | Remove                                           | Hang                                                                             |



## <a id="white-space-processing"></a>4.  White Space Processing &#x26; Control Characters

<a id="ref-for-white-space②"></a>

<a id="ref-for-tabs"></a>

<a id="ref-for-spaces①"></a>

<a id="ref-for-propdef-white-space-collapse③"></a>

<a id="ref-for-propdef-white-space-trim③"></a>

The source text of a document often contains formatting that is not relevant to the final rendering: for example, [breaking the source into segments](https://rhodesmill.org/brandon/2012/one-sentence-per-line/) (lines) for ease of editing or adding [white space characters](#white-space) such as [tabs](#tabs) and [spaces](#spaces) to indent the source code. CSS white space processing allows the author to control interpretation of such formatting: to preserve or collapse it away when rendering the document. White space processing in CSS (which is controlled with the [white-space-collapse](#propdef-white-space-collapse) and [white-space-trim](#propdef-white-space-trim) properties) interprets <a id="ref-for-white-space③"></a>white space characters only for rendering: it has no effect on the underlying document data.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the document language, segments can be separated by a particular newline sequence (such as a line feed or CRLF pair), or delimited by some other mechanism, such as the SGML `RECORD-START` and `RECORD-END` tokens.

<a id="ref-for-propdef-white-space③"></a>

<a id="segment-normalization"></a> For CSS processing, each document language–defined “segment break” or “newline sequence”—​or if none are defined, each line feed (U+000A)—​in the text is treated as a <a id="segment-break"></a>segment break, which is then interpreted for rendering as specified by the [white-space](#propdef-white-space) property.

<a id="ref-for-normalize-newlines"></a>

<a id="ref-for-segment-break"></a>

In the case of HTML, [newlines](https://html.spec.whatwg.org/multipage/syntax.html#newlines) are [normalized](https://infra.spec.whatwg.org/#normalize-newlines) to line feed characters (U+000A) for representation in the DOM, so when an HTML document is represented as a DOM tree each line feed (U+000A) is treated as a [segment break](#segment-break). [\[HTML\]](#biblio-html) [\[DOM\]](#biblio-dom)

<a id="ref-for-segment-break①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In most common CSS implementations, HTML does not get styled directly. Instead, it is processed into a DOM tree, which is then styled. Unlike HTML, the DOM does not give any particular meaning to carriage returns (U+000D), so they are not treated as [segment breaks](#segment-break). If carriage returns (U+000D) are inserted into the DOM by means other than HTML parsing, they then get treated as defined below.

<a id="ref-for-segment-break②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A document parser might not only normalize any [segment breaks](#segment-break), but also collapse other space characters or otherwise process white space according to markup rules. Because CSS processing occurs <em>after</em> the parsing stage, it is not possible to restore these characters for styling. Therefore, some of the behavior specified below can be affected by these limitations and may be user agent dependent.

<a id="ref-for-collapsible-white-space"></a>

<a id="ref-for-white-space④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Anonymous blocks consisting entirely of [collapsible](#collapsible-white-space) [white space](#white-space) are removed from the rendering tree. Thus any such <a id="ref-for-white-space⑤"></a>white space surrounding a block-level element is collapsed away. See [CSS 2.1 § 9.2.2.1 Anonymous inline boxes](https://www.w3.org/TR/CSS21/visuren.html#anonymous). [\[CSS2\]](#biblio-css2)

<a id="ref-for-unicode-general-category①"></a>

<a id="ref-for-segment-break③"></a>

<a id="ref-for-unicode-script"></a>

Control characters ([Unicode category](#unicode-general-category) `Cc`)—​other than tabs (U+0009), line feeds (U+000A), carriage returns (U+000D) and sequences that form a [segment break](#segment-break)—​must be rendered as a visible glyph which the UA must synthesize if the glyphs found in the font are not visible, and must be otherwise treated as any other character of the Other Symbols (`So`) <a id="ref-for-unicode-general-category②"></a>general category and Common [script](#unicode-script). The UA may use a glyph provided by a font specifically for the control character, substitute the glyphs provided for the corresponding symbol in the Control Pictures block, generate a visual representation of its code point value, or use some other method to provide an appropriate visible glyph. As required by Unicode, unsupported `Default_ignorable` characters must be ignored for text rendering. [\[UNICODE\]](#biblio-unicode)

Carriage returns (U+000D) are treated identically to spaces (U+0020) in all respects.

<a id="ref-for-normalize-newlines①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For HTML documents, carriage returns present in the source code are converted to line feeds at the parsing stage (see [HTML § 13.2.3.5 Preprocessing the input stream](https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream) and the definition of [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines) in [Infra](https://infra.spec.whatwg.org/) and therefore do no appear as U+000D CARRIAGE RETURN to CSS. [\[HTML\]](#biblio-html) [\[INFRA\]](#biblio-infra)) However, the character <em>is</em> preserved—​and the above rule observable—​when encoded using an escape sequence (`&#x0d;`).

<a id="ref-for-propdef-white-space-collapse④"></a>

### <a id="white-space-collapsing"></a>4.1.  White Space Collapsing: the [white-space-collapse](#propdef-white-space-collapse) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-257c8a37"></a> This section is still under discussion and may change in future drafts.



| Field               | Definition                                                                                                                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-white-space-collapse"></a>white-space-collapse                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a>collapse [\|](https://www.w3.org/TR/css-values-4/#comb-one) discard <a id="ref-for-comb-one①①"></a>\| preserve <a id="ref-for-comb-one①②"></a>\| preserve-breaks <a id="ref-for-comb-one①③"></a>\| preserve-spaces <a id="ref-for-comb-one①④"></a>\| break-spaces |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | collapse                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                             |



This property specifies whether and how [white space](#white-space-processing) is collapsed. Values have the following meanings, which must be interpreted according to the [White Space Processing Rules](#white-space-rules):

<a id="valdef-white-space-collapse-collapse"></a>collapse  
<a id="ref-for-white-space⑥"></a>

This value directs user agents to collapse sequences of [white space](#white-space) into a single character (or [in some cases](#line-break-transform), no character).

<a id="valdef-white-space-collapse-preserve"></a>preserve  
<a id="ref-for-forced-line-break①"></a>

<a id="ref-for-segment-break④"></a>

<a id="ref-for-white-space⑦"></a>

This value prevents user agents from collapsing sequences of [white space](#white-space). [Segment breaks](#segment-break) such as line feeds are preserved as [forced line breaks](#forced-line-break).

<a id="valdef-white-space-collapse-preserve-breaks"></a>preserve-breaks  
<a id="ref-for-forced-line-break②"></a>

<a id="ref-for-segment-break⑤"></a>

<a id="ref-for-white-space⑧"></a>

<a id="ref-for-valdef-white-space-collapse-collapse①"></a>

Like [collapse](#valdef-white-space-collapse-collapse), this value collapses consecutive [white space characters](#white-space), but preserves [segment breaks](#segment-break) in the source as [forced line breaks](#forced-line-break).

<a id="valdef-white-space-collapse-preserve-spaces"></a>preserve-spaces  
<a id="ref-for-spaces②"></a>

<a id="ref-for-segment-break⑥"></a>

<a id="ref-for-tabs①"></a>

<a id="ref-for-white-space⑨"></a>

This value prevents user agents from collapsing sequences of [white space](#white-space), and converts [tabs](#tabs) and [segment breaks](#segment-break) to [spaces](#spaces). (This value is intended to represent the behavior of `xml:space="preserve"` in SVG.)

<a id="valdef-white-space-collapse-break-spaces"></a>break-spaces  
<a id="ref-for-valdef-white-space-collapse-preserve②"></a>

The behavior is identical to that of [preserve](#valdef-white-space-collapse-preserve), except that:

- <a id="ref-for-other-space-separators③"></a>

  <a id="ref-for-white-space①⓪"></a>

  <a id="ref-for-preserved-white-space②"></a>

  Any sequence of [preserved](#preserved-white-space) [white space](#white-space) or [other space separators](#other-space-separators) always takes up space, including at the end of the line.

- <a id="ref-for-other-space-separators④"></a>

  <a id="ref-for-white-space①①"></a>

  <a id="ref-for-preserved-white-space③"></a>

  <a id="ref-for-soft-wrap-opportunity①"></a>

  A [soft wrap opportunity](#soft-wrap-opportunity) exists after every [preserved](#preserved-white-space) [white space](#white-space) character and after every [other space separator](#other-space-separators) (including between adjacent spaces).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value does not guarantee that there will never be any overflow due to white space: for example, if the line length is so short that even a single white space character does not fit, overflow is unavoidable.

<a id="valdef-white-space-collapse-discard"></a>discard  
This value directs user agents to “discard” all white space in the element.

<a id="ref-for-propdef-word-space-transform③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e3dd9c3a"></a> Does this preserve line break opportunities or no? Do we need a distinct "hide" value? If it preserves line break opportunities, maybe it should be replaced with a [word-space-transform](#propdef-word-space-transform) value?

<a id="ref-for-white-space①②"></a>

[White space](#white-space) that was not removed or collapsed due to white space processing is called <a id="preserved-white-space"></a>preserved white space.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6af3efc0"></a>
>
> The following style rules implement MathML’s white space processing:
>
> ```text
> @namespace m "http://www.w3.org/1998/Math/MathML";
> m|* {
>   white-space-collapse: discard;
> }
> m|mi, m|mn, m|mo, m|ms, m|mtext {
>   white-space-trim: discard-inner;
> }
> ```
<a id="ref-for-propdef-white-space-trim④"></a>

### <a id="white-space-trim"></a>4.2.  White Space Trimming: the [white-space-trim](#propdef-white-space-trim) property<a id="text-space-trim"></a>



| Field               | Definition                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-white-space-trim"></a>white-space-trim                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any④"></a><a id="ref-for-comb-one①⑤"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) discard-before [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) discard-after <a id="ref-for-comb-any⑤"></a>\|\| discard-inner |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-block-container"></a><a id="ref-for-inline-box①"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) and [block containers](https://www.w3.org/TR/css-display-3/#block-container)                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                              |



This property allows authors to specify trimming behavior at the beginning and end of a box. Values have the following meanings:

<a id="valdef-white-space-trim-discard-before"></a>discard-before  
This value directs the UA to collapse all collapsible whitespace immediately before the start of the element.

<a id="valdef-white-space-trim-discard-after"></a>discard-after  
This value directs the UA to collapse all collapsible whitespace immediately after the end of the element.

<a id="valdef-white-space-trim-discard-inner"></a>discard-inner  
<a id="ref-for-segment-break⑦"></a>

For block containers this value directs UAs to discard all whitespace at the beginning of the element up to and including the last [segment break](#segment-break) before the first non-white-space character in the element as well as to discard all white space at the end of the element starting with the first <a id="ref-for-segment-break⑧"></a>segment break after the last non-white-space character in the element. For other elements this value directs UAs to discard all whitespace at the beginning and end of the element.

<a id="ref-for-white-space①③"></a>

<a id="ref-for-propdef-white-space-trim⑤"></a>

<a id="ref-for-soft-wrap-opportunity②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Discarding [document white space](#white-space) using [white-space-trim](#propdef-white-space-trim) can change where [soft wrap opportunities](#soft-wrap-opportunity) occur in the text.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-94e0c859"></a>
>
> The following style rules render DT elements as a comma-separated list, even if they are coded on separate lines of the source document:
>
> ```text
> dt { display: inline; }
> dt + dt:before { content: ", "; white-space-trim: discard-before; }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a4aedd3c"></a>
>
> The following style rule removes source-formatting white space adjacent to the opening/closing tags of a preformatted block, but not any indentation or interleaved white space applied to the actual contents of the element:
>
> ```text
> pre { white-space: pre; white-space-trim: discard-inner; }
> ```
>
> This results in the following two source-code snippets:
>
> ```text
> <pre>
> 
>   some
> preformatted
> 
>   text
> 
> </pre>
> ```
>
> ```text
> <pre>  some
> preformatted
> 
>   text</pre>
> ```
>
> rendering identically as:
>
> ```text
>   some
> preformatted
> 
>   text
> ```
>
> If instead we apply it to an inline element:
>
> ```text
> span { white-space: normal; white-space-trim: discard-inner; }
> ```
>
> ```text
> start[<span>
> 
>   some
> inline
>   text
> 
> </span>]end
> ```
>
> ```text
> start[<span>  some
> inline
>   text</span>]end
> ```
>
> this directs the UA to discard <em>all</em> of the leading/trailing white space before the actual contents of the element:
>
> ```text
> start[some inline text]end
> ```
<a id="ref-for-propdef-white-space-trim⑥"></a>

White space processing for [white-space-trim](#propdef-white-space-trim) takes place before [§ 4.3.1 Phase I: Collapsing and Transformation](#white-space-phase-1).

### <a id="white-space-rules"></a>4.3.  The White Space Processing Rules

Except where specified otherwise, white space processing in CSS affects only the <a id="white-space"></a>document white space characters: <a id="spaces"></a>spaces (U+0020), <a id="tabs"></a>tabs (U+0009), and [segment breaks](#white-space-processing).

<a id="ref-for-white-space①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The set of characters considered [document white space](#white-space) (part of the document content) and those considered syntactic white space (part of the CSS syntax) are not necessarily identical. However, since both include spaces (U+0020), tabs (U+0009), and line feeds (U+000A) most authors won’t notice any differences.

<a id="ref-for-unicode-general-category③"></a>

Besides space (U+0020) and no-break space (U+00A0), Unicode defines a number of additional space separator characters. [\[UNICODE\]](#biblio-unicode) In this specification all characters in the Unicode [general category](#unicode-general-category) Zs except space (U+0020) and no-break space (U+00A0) are collectively referred to as <a id="other-space-separators"></a>other space separators.

#### <a id="white-space-phase-1"></a>4.3.1.  Phase I: Collapsing and Transformation

<a id="ref-for-propdef-white-space-trim⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [white-space-trim](#propdef-white-space-trim) is taken into account prior to this phase.

<a id="ref-for-inline-formatting-context"></a>

<a id="ref-for-white-space①⑤"></a>

<a id="ref-for-line-breaking-process"></a>

For each inline (including anonymous inlines; see [CSS 2.1 § 9.2.2.1 Anonymous inline boxes](https://www.w3.org/TR/CSS21/visuren.html#anonymous) [\[CSS2\]](#biblio-css2)) within an [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context), [white space characters](#white-space) are processed as follows prior to [line breaking](#line-breaking-process) and [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction), ignoring <a id="bidi-formatting-characters"></a>bidi formatting characters (characters with the `Bidi_Control` property [\[UAX9\]](#biblio-uax9)) as if they were not there:

- <a id="ref-for-white-space①⑥"></a>

  <a id="ref-for-valdef-white-space-collapse-preserve-breaks①"></a>

  <a id="ref-for-valdef-white-space-collapse-collapse②"></a>

  <a id="ref-for-propdef-white-space-collapse⑤"></a>

  <a id="collapse"></a> If [white-space-collapse](#propdef-white-space-collapse) is set to [collapse](#valdef-white-space-collapse-collapse) or [preserve-breaks](#valdef-white-space-collapse-preserve-breaks), [white space characters](#white-space) are considered <a id="collapsible-white-space"></a>collapsible and are processed by performing the following steps:

  1.  <a id="ref-for-segment-break⑨"></a>

      <a id="ref-for-tabs②"></a>

      <a id="ref-for-spaces③"></a>

      Any sequence of collapsible [spaces](#spaces) and [tabs](#tabs) immediately preceding or following a [segment break](#segment-break) is removed.

  2.  <a id="ref-for-segment-break①⓪"></a>

      Collapsible [segment breaks](#segment-break) are transformed for rendering according to the [segment break transformation rules](#line-break-transform).

  3.  <a id="ref-for-tabs③"></a>

      <a id="ref-for-collapsible-white-space①"></a>

      Every [collapsible](#collapsible-white-space) [tab](#tabs) is converted to a collapsible space (U+0020).

  4.  <a id="ref-for-soft-wrap-opportunity③"></a>

      <a id="ref-for-spaces④"></a>

      <a id="ref-for-collapsible-white-space②"></a>

      Any [collapsible](#collapsible-white-space) [space](#spaces) immediately following another <a id="ref-for-collapsible-white-space③"></a>collapsible <a id="ref-for-spaces⑤"></a>space—​even one outside the boundary of the inline containing that <a id="ref-for-spaces⑥"></a>space, provided both <a id="ref-for-spaces⑦"></a>spaces are within the same inline formatting context—​is collapsed to have zero advance width. (It is invisible, but retains its [soft wrap opportunity](#soft-wrap-opportunity), if any.)

- <a id="ref-for-spaces⑧"></a>

  <a id="ref-for-segment-break①①"></a>

  <a id="ref-for-tabs④"></a>

  <a id="ref-for-valdef-white-space-collapse-preserve-spaces"></a>

  <a id="ref-for-propdef-white-space-collapse⑥"></a>

  If [white-space-collapse](#propdef-white-space-collapse) is set to [preserve-spaces](#valdef-white-space-collapse-preserve-spaces), each [tab](#tabs) and [segment break](#segment-break) is converted to a [space](#spaces).

- <a id="ref-for-valdef-white-space-collapse-break-spaces①"></a>

  <a id="ref-for-tabs⑤"></a>

  <a id="ref-for-spaces⑨"></a>

  <a id="ref-for-soft-wrap-opportunity④"></a>

  <a id="ref-for-valdef-white-space-collapse-preserve-spaces①"></a>

  <a id="ref-for-valdef-white-space-collapse-preserve③"></a>

  <a id="ref-for-propdef-white-space-collapse⑦"></a>

  If [white-space-collapse](#propdef-white-space-collapse) is set to [preserve](#valdef-white-space-collapse-preserve) or [preserve-spaces](#valdef-white-space-collapse-preserve-spaces), any sequence of spaces is treated as a sequence of non-breaking spaces except that a [soft wrap opportunity](#soft-wrap-opportunity) exists at the end of each maximal sequence of [spaces](#spaces) and/or [tabs](#tabs). For [break-spaces](#valdef-white-space-collapse-break-spaces), a <a id="ref-for-soft-wrap-opportunity⑤"></a>soft wrap opportunity exists after every <a id="ref-for-spaces①⓪"></a>space and every <a id="ref-for-tabs⑥"></a>tab.

<a id="ref-for-spaces①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="egbidiwscollapse"></a> The following example illustrates the interaction of white-space collapsing and bidirectionality. Consider the following markup fragment, taking special note of [spaces](#spaces) (with varied backgrounds and borders for emphasis and identification):
>
> ```text
> <ltr>A <rtl> B </rtl> C</ltr>
> ```
>
> <a id="ref-for-propdef-white-space④"></a>
>
> <a id="ref-for-valdef-white-space-normal①"></a>
>
> where the `<ltr>` element represents a left-to-right embedding and the `<rtl>` element represents a right-to-left embedding. If the [white-space](#propdef-white-space) property is set to [normal](#valdef-white-space-normal), the white-space processing model will result in the following:
>
> - <a id="ref-for-spaces①②"></a>
>
>   The [space](#spaces) before the B ( ) will collapse with the <a id="ref-for-spaces①③"></a>space after the A ( ).
>
> - <a id="ref-for-spaces①④"></a>
>
>   The [space](#spaces) before the C ( ) will collapse with the <a id="ref-for-spaces①⑤"></a>space after the B ( ).
>
> <a id="ref-for-spaces①⑥"></a>
>
> This will leave two [spaces](#spaces), one after the A in the left-to-right embedding level, and one after the B in the right-to-left embedding level. The text will then be ordered according to the Unicode bidirectional algorithm, with the end result being:
>
> ```text
> A  BC
> ```
>
> <a id="ref-for-spaces①⑦"></a>
>
> Note that there will be two [spaces](#spaces) between A and B, and none between B and C. This is best avoided by putting <a id="ref-for-spaces①⑧"></a>spaces outside the element instead of just inside the opening and closing tags and, where practical, by relying on implicit bidirectionality instead of explicit embedding levels.

#### <a id="white-space-phase-2"></a>4.3.2.  Phase II: Trimming and Positioning

<a id="ref-for-wrapping①"></a>

<a id="ref-for-propdef-text-wrap-mode③"></a>

<a id="ref-for-propdef-text-wrap-style"></a>

Then, the entire block is rendered. Inlines are laid out, taking [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) into account, and [wrapping](#wrapping) as specified by the [text-wrap-mode](#propdef-text-wrap-mode) and [text-wrap-style](#propdef-text-wrap-style) property. As each line is laid out,

1.  <a id="ref-for-spaces①⑨"></a>

    <a id="ref-for-collapsible-white-space④"></a>

    A sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) at the beginning of a line is removed.

2.  <a id="ref-for-propdef-tab-size"></a>

    <a id="ref-for-block-container①"></a>

    <a id="ref-for-tab-stop"></a>

    <a id="ref-for-tabs⑦"></a>

    <a id="ref-for-preserved-white-space④"></a>

    <a id="ref-for-tab-size-dfn"></a>

    If the [tab size](#tab-size-dfn) is zero, [preserved](#preserved-white-space) [tabs](#tabs) are not rendered. Otherwise, each <a id="ref-for-preserved-white-space⑤"></a>preserved <a id="ref-for-tabs⑧"></a>tab is rendered as a horizontal shift that lines up the start edge of the next glyph with the next [tab stop](#tab-stop). If this distance is less than 0.5ch, then the subsequent <a id="ref-for-tab-stop①"></a>tab stop is used instead. <a id="tab-stop"></a>Tab stops occur at points that are multiples of the <a id="ref-for-tab-size-dfn①"></a>tab size from the starting content edge of the <a id="ref-for-preserved-white-space⑥"></a>preserved <a id="ref-for-tabs⑨"></a>tab’s nearest [block container](https://www.w3.org/TR/css-display-3/#block-container) ancestor. The <a id="ref-for-tab-size-dfn②"></a>tab size is given by the [tab-size](#propdef-tab-size) property.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: See the Unicode [rules on how tabulation (U+0009) interacts with bidi](http://unicode.org/reports/tr9/#L1). [\[UAX9\]](#biblio-uax9)

3.  <a id="ref-for-valdef-white-space-collapse-preserve-breaks②"></a>

    <a id="ref-for-valdef-white-space-collapse-collapse③"></a>

    <a id="ref-for-propdef-white-space-collapse⑧"></a>

    <a id="ref-for-spaces②⓪"></a>

    <a id="ref-for-collapsible-white-space⑤"></a>

    A sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) at the end of a line is removed, as well as any trailing U+1680   OGHAM SPACE MARK whose [white-space-collapse](#propdef-white-space-collapse) property is [collapse](#valdef-white-space-collapse-collapse) or [preserve-breaks](#valdef-white-space-collapse-preserve-breaks).

    <a id="ref-for-collapsible-white-space⑥"></a>

    <a id="ref-for-spaces②①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Due to Unicode Bidirectional Algorithm rule [L1](http://unicode.org/reports/tr9/#L1), a sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) located at the end of the line prior to [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) will also be at the end of the line after reordering. [\[UAX9\]](#biblio-uax9) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

4.  <a id="ref-for-tabs①⓪"></a>

    <a id="ref-for-preserved-white-space⑦"></a>

    <a id="ref-for-other-space-separators⑤"></a>

    <a id="ref-for-white-space①⑦"></a>

    If there remains any sequence of [white space](#white-space), [other space separators](#other-space-separators), and/or [preserved](#preserved-white-space) [tabs](#tabs) at the end of a line (after [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)):

    - <a id="ref-for-hang①"></a>

      <a id="ref-for-valdef-white-space-collapse-preserve-breaks③"></a>

      <a id="ref-for-valdef-white-space-collapse-collapse④"></a>

      <a id="ref-for-propdef-white-space-collapse⑨"></a>

      If [white-space-collapse](#propdef-white-space-collapse) is [collapse](#valdef-white-space-collapse-collapse) or [preserve-breaks](#valdef-white-space-collapse-preserve-breaks), the UA must [hang](#hang) this sequence (unconditionally).

    - <a id="ref-for-conditionally-hang"></a>

      <a id="ref-for-forced-line-break③"></a>

      <a id="ref-for-hang②"></a>

      <a id="ref-for-valdef-text-wrap-mode-nowrap①"></a>

      <a id="ref-for-propdef-text-wrap-mode④"></a>

      <a id="ref-for-valdef-white-space-collapse-preserve④"></a>

      <a id="ref-for-propdef-white-space-collapse①⓪"></a>

      If [white-space-collapse](#propdef-white-space-collapse) is [preserve](#valdef-white-space-collapse-preserve) and [text-wrap-mode](#propdef-text-wrap-mode) is not [nowrap](#valdef-text-wrap-mode-nowrap), the UA must (unconditionally) [hang](#hang) this sequence, unless the sequence is followed by a [forced line break](#forced-line-break), in which case it must [conditionally hang](#conditionally-hang) the sequence instead. It may also visually collapse the character advance widths of any that would otherwise overflow.

      <a id="ref-for-hang③"></a>

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > Note: [Hanging](#hang) the white space rather than collapsing it allows users to see the space when selecting or editing text.

    - <a id="ref-for-hang④"></a>

      <a id="ref-for-other-space-separators⑥"></a>

      <a id="ref-for-tabs①①"></a>

      <a id="ref-for-spaces②②"></a>

      <a id="ref-for-valdef-white-space-collapse-break-spaces②"></a>

      <a id="ref-for-propdef-white-space-collapse①①"></a>

      If [white-space-collapse](#propdef-white-space-collapse) is set to [break-spaces](#valdef-white-space-collapse-break-spaces), [spaces](#spaces), [tabs](#tabs), and [other space separators](#other-space-separators) are treated the same as other visible characters: they cannot [hang](#hang) nor have their advance width collapsed.

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > Note: Such characters therefore take up space, and depending on the available space and applicable line breaking controls will either overflow or cause the line to wrap.

    <a id="ref-for-propdef-white-space-collapse①②"></a>

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-a72a5cd8"></a> What should happen here for [white-space-collapse: preserve-spaces](#propdef-white-space-collapse)?

<a id="ref-for-conditionally-hang①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-af2745cd"></a> This example shows that [conditionally hanging](#conditionally-hang) white space at the end of lines with forced breaks provides symmetry with the start of the line. An underline is added to help visualize the spaces.
>
> ```text
> p {
>   white-space: pre-wrap;
>   width: 5ch;
>   border: solid 1px;
>   font-family: monospace;
>   text-align: center;
> }
> ```
>
> ```text
> <p> 0 </p>
> ```
>
> The sample above would be rendered as follows:
>
> 0
>
> <a id="ref-for-spaces②③"></a>
>
> Since the final [space](#spaces) is before a forced line break and does not overflow, it does not hang, and centering works as expected.

<a id="ref-for-hang⑤"></a>

<a id="ref-for-spaces②④"></a>

<a id="ref-for-conditionally-hang②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e231f51f"></a> This example illustrates the difference between [hanging](#hang) [spaces](#spaces) at the end of lines without forced breaks, and [conditionally hanging](#conditionally-hang) them at the end of lines with forced breaks. An underline is added to help visualize the <a id="ref-for-spaces②⑤"></a>spaces.
>
> ```text
> p {
>   white-space: pre-wrap;
>   width: 3ch;
>   border: solid 1px;
>   font-family: monospace;
> }
> ```
>
> ```text
> <p> 0 0 0 0 </p>
> ```
>
> The sample above would be rendered as follows:
>
> 0  
> 0 0  
> 0
>
> If `p { text-align: right; }` was added, the result would be as follows:
>
> 0   
> 0 0   
> 0
>
> <a id="ref-for-preserved-white-space⑧"></a>
>
> <a id="ref-for-spaces②⑥"></a>
>
> <a id="ref-for-hang⑥"></a>
>
> <a id="ref-for-conditionally-hang③"></a>
>
> As the [preserved](#preserved-white-space) [spaces](#spaces) at the end of lines without a forced break must [hang](#hang), they are not considered when placing the rest of the line during text alignment. When aligning towards the end, this means any such <a id="ref-for-spaces②⑦"></a>spaces will overflow, and will not prevent the rest of the line’s content from being flush with the edge of the line. On the other hand, preserved spaces at the end of a line <em>with</em> a forced break [conditionally hang](#conditionally-hang). Since the space at the end of the last line would not overflow in this example, it does not <a id="ref-for-hang⑦"></a>hang and therefore is considered during text alignment.

<a id="ref-for-hang⑧"></a>

<a id="ref-for-conditionally-hang④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6c2f4658"></a> In the following example, there is not enough room on any line to fit the end-of-line spaces, so they [hang](#hang) on all lines: the one on the line without a forced break because it must, as well as the one on the line with a forced break, because it [conditionally hangs](#conditionally-hang) and overflows. An underline is added to help visualize the spaces.
>
> ```text
> p {
>   white-space: pre-wrap;
>   width: 3ch;
>   border: solid 1px;
>   font-family: monospace;
> }
> ```
>
> ```text
> <p>0 0 0 0 </p>
> ```
>
> 0 0  
> 0 0
>
> <a id="ref-for-conditionally-hang⑤"></a>
>
> The last line is not wrapped before the last `0` because characters that [conditionally hang](#conditionally-hang) are not considered when measuring the line’s contents for fit.

#### <a id="line-break-transform"></a>4.3.3.  Segment Break Transformation Rules

<a id="ref-for-propdef-white-space-collapse①③"></a>

<a id="ref-for-valdef-white-space-collapse-collapse⑤"></a>

<a id="ref-for-segment-break①②"></a>

<a id="ref-for-collapsible-white-space⑦"></a>

<a id="ref-for-valdef-white-space-collapse-preserve-spaces②"></a>

<a id="ref-for-spaces②⑧"></a>

When [white-space-collapse](#propdef-white-space-collapse) is not [collapse](#valdef-white-space-collapse-collapse), [segment breaks](#segment-break) are not [collapsible](#collapsible-white-space). For values other than <a id="ref-for-valdef-white-space-collapse-collapse⑥"></a>collapse or [preserve-spaces](#valdef-white-space-collapse-preserve-spaces) (which transforms them into [spaces](#spaces)), <a id="ref-for-segment-break①③"></a>segment breaks are instead transformed into a preserved line feed (U+000A).

<a id="ref-for-propdef-white-space-collapse①④"></a>

<a id="ref-for-valdef-white-space-collapse-collapse⑦"></a>

<a id="ref-for-segment-break①④"></a>

<a id="ref-for-collapsible-white-space⑧"></a>

When [white-space-collapse](#propdef-white-space-collapse) is [collapse](#valdef-white-space-collapse-collapse), [segment breaks](#segment-break) are [collapsible](#collapsible-white-space), and are collapsed as follows:

1.  <a id="ref-for-segment-break①⑤"></a>

    First, any collapsible [segment break](#segment-break) immediately following another collapsible <a id="ref-for-segment-break①⑥"></a>segment break is removed.

2.  <a id="ref-for-segment-break①⑦"></a>

    Then any remaining [segment break](#segment-break) is either transformed into a space (U+0020) or removed depending on the context before and after the break. The rules for this operation are UA-defined in this level.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-c0106ca7"></a> Should we define this for Level 4?

    <a id="ref-for-tabs①②"></a>

    <a id="ref-for-spaces②⑨"></a>

    <a id="ref-for-segment-break①⑧"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The white space processing rules have already removed any [tabs](#tabs) and [spaces](#spaces) around the [segment break](#segment-break) before this context is evaluated.

<a id="ref-for-spaces③⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e2e8c9a5"></a> The purpose of the segment break transformation rules (and white space collapsing in general) is to “unbreak” text that has been [broken into segments](#white-space-processing) to make the document source code easier to work with. In languages that use word separators, such as English and Korean, “unbreaking” a line requires joining the two lines with a [space](#spaces).
>
> ```text
> Here is an English paragraph
> that is broken into multiple lines
> in the source code so that it can
> be more easily read and edited
> in a text editor.
> ```
>
> Here is an English paragraph that is broken into multiple lines in the source code so that it can be more easily read and edited in a text editor.
>
> <a id="ref-for-spaces③①"></a>
>
> Eliminating a line break in English requires maintaining a [space](#spaces) in its place.
>
> In languages that have no word separators, such as Chinese, “unbreaking” a line requires joining the two lines with no intervening space.
>
> ```text
> 這個段落是那麼長，
> 在一行寫不行。最好
> 用三行寫。
> ```
>
> 這個段落是那麼長，在一行寫不行。最好用三行寫。
>
> <a id="ref-for-white-space①⑧"></a>
>
> Eliminating a line break in Chinese requires eliminating any intervening [white space](#white-space).
>
> The segment break transformation rules can use adjacent context to either transform the segment break into a space or eliminate it entirely.

<a id="ref-for-segment-break①⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Historically, HTML and CSS have unconditionally converted [segment breaks](#segment-break) to spaces, which has prevented content authored in languages such as Chinese from being able to break lines within the source. Thus UA heuristics need to be conservative about where they discard <a id="ref-for-segment-break②⓪"></a>segment breaks even as they strive to improve support for such languages.

<a id="ref-for-propdef-tab-size①"></a>

### <a id="tab-size-property"></a>4.4.  Tab Character Size: the [tab-size](#propdef-tab-size) property<a id="tab-size"></a>



| Field               | Definition                                                                                                                                                                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-tab-size"></a>tab-size                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a><a id="ref-for-comb-one①⑥"></a><a id="ref-for-number-value"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 8                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified number or absolute length                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                    |



<a id="ref-for-preserved-white-space⑨"></a>

<a id="ref-for-number-value①"></a>

<a id="ref-for-block-container②"></a>

<a id="ref-for-tabs①③"></a>

<a id="ref-for-propdef-letter-spacing"></a>

<a id="ref-for-propdef-word-spacing"></a>

This property determines the <a id="tab-size-dfn"></a>tab size used to render [preserved](#preserved-white-space) tab characters (U+0009). A [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) represents the measure as a multiple of the advance width of the space character (U+0020) of the nearest [block container](https://www.w3.org/TR/css-display-3/#block-container) ancestor of the <a id="ref-for-preserved-white-space①⓪"></a>preserved [tab](#tabs), including its associated [letter-spacing](#propdef-letter-spacing) and [word-spacing](#propdef-word-spacing). Negative values are not allowed.

## <a id="text-wrapping"></a>5.  Text Wrapping<a id="text-wrap"></a>

<a id="ref-for-preserved-white-space①①"></a>

When inline-level content is laid out into lines, it is broken across line boxes. Such a break is called a <a id="line-break"></a>line break. When a line is broken due to explicit line-breaking controls (such as a [preserved](#preserved-white-space) newline character), or due to the start or end of a block, it is a <a id="forced-line-break"></a>forced line break. When a line is broken due to content <a id="wrapping"></a>wrapping (i.e. when the UA creates unforced line breaks in order to fit the content within the measure), it is a <a id="soft-wrap-break"></a>soft wrap break. The process of breaking inline-level content into lines is called <a id="line-breaking-process"></a>line breaking.

<a id="ref-for-propdef-white-space⑤"></a>

<a id="ref-for-soft-wrap-opportunity⑥"></a>

Wrapping is only performed at an allowed break point, called a <a id="soft-wrap-opportunity"></a>soft wrap opportunity. When wrapping is enabled (see [white-space](#propdef-white-space)), the UA must minimize the amount of content overflowing a line by wrapping the line at a [soft wrap opportunity](#soft-wrap-opportunity), if one exists.

<a id="ref-for-soft-wrap-opportunity⑦"></a>

<a id="ref-for-propdef-text-wrap-mode⑤"></a>

<a id="ref-for-propdef-text-wrap-style①"></a>

<a id="ref-for-propdef-wrap-before"></a>

<a id="ref-for-propdef-wrap-after"></a>

<a id="ref-for-propdef-wrap-inside"></a>

Where text is allowed to wrap is controlled by the [line-breaking rules and controls](#line-breaking); <em>whether</em> it is allowed to wrap and how multiple [soft wrap opportunities](#soft-wrap-opportunity) within a line are prioritized is controlled by the [text-wrap-mode](#propdef-text-wrap-mode), [text-wrap-style](#propdef-text-wrap-style), [wrap-before](#propdef-wrap-before), [wrap-after](#propdef-wrap-after), and [wrap-inside](#propdef-wrap-inside) properties.

<a id="ref-for-propdef-text-wrap-mode⑥"></a>

### <a id="text-wrap-mode"></a>5.1.  Deciding Whether to Wrap: the [text-wrap-mode](#propdef-text-wrap-mode) property



| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-wrap-mode"></a>text-wrap-mode                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑦"></a>wrap [\|](https://www.w3.org/TR/css-values-4/#comb-one) nowrap |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | wrap                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                          |



> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3d83054d"></a> The name of this property is a placeholder, pending the CSSWG finding a better name.

<a id="ref-for-longhand②"></a>

<a id="ref-for-propdef-white-space⑥"></a>

<a id="ref-for-propdef-text-wrap"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property is a [longhand](https://www.w3.org/TR/css-cascade-5/#longhand) of both [white-space](#propdef-white-space) and [text-wrap](#propdef-text-wrap).

<a id="ref-for-wrapping②"></a>

<a id="ref-for-soft-wrap-opportunity⑧"></a>

This property specifies whether lines may [wrap](#wrapping) at unforced [soft wrap opportunities](#soft-wrap-opportunity). Possible values:

<a id="valdef-text-wrap-mode-wrap"></a>wrap  
<a id="ref-for-inline-axis"></a>

<a id="ref-for-soft-wrap-opportunity⑨"></a>

Content may break across lines at allowed [soft wrap opportunities](#soft-wrap-opportunity), as determined by the line-breaking rules in effect, in order to minimize [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) overflow.

<a id="ref-for-soft-wrap-opportunity①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: See [§ 5.6 Line Breaking Details](#line-break-details) for more information about rules and constrains on [soft wrap opportunities](#soft-wrap-opportunity).

<a id="valdef-text-wrap-mode-nowrap"></a>nowrap  
Inline-level content does not break across lines; content that does not fit within the block container overflows it.

<a id="ref-for-propdef-text-wrap-mode⑦"></a>

<a id="ref-for-preserved-white-space①②"></a>

<a id="ref-for-segment-break②①"></a>

<a id="ref-for-forced-line-break④"></a>

Regardless of the [text-wrap-mode](#propdef-text-wrap-mode) value, [preserved](#preserved-white-space) [segment breaks](#segment-break), and any Unicode character with the `BK`, `CR`, `LF`, and `NL` line breaking class, must be treated as [forced line breaks](#forced-line-break). [\[UAX14\]](#biblio-uax14)

<a id="ref-for-forced-line-break⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The bidi implications of such [forced line breaks](#forced-line-break) are defined by the [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/). [\[UAX9\]](#biblio-uax9)

<a id="ref-for-propdef-wrap-inside①"></a>

### <a id="wrap-inside"></a>5.2.  Controlling Breaks Within Boxes: the [wrap-inside](#propdef-wrap-inside) property



| Field               | Definition                                                                         |
|---------------------|------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-wrap-inside"></a>wrap-inside                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑧"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box②"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                           |



<a id="valdef-wrap-inside-auto"></a>auto  
Lines may break at allowed break points within the box, as determined by the line-breaking rules in effect.

<a id="valdef-wrap-inside-avoid"></a>avoid  
<a id="ref-for-valdef-wrap-inside-auto"></a>

Line breaking is suppressed within the box: the UA may only break within the box if there are no other valid break points in the line. If the text breaks, line-breaking restrictions are honored as for [auto](#valdef-wrap-inside-auto).

<a id="ref-for-valdef-wrap-inside-avoid"></a>

If boxes with [avoid](#valdef-wrap-inside-avoid) are nested and the UA must break within these boxes, a break in an outer box must be used before a break within an inner box may be used.

#### <a id="example-avoid"></a>5.2.1.  Example of using 'wrap-inside: avoid' in presenting a footer

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0e0fea94"></a>
>
> The priority of breakpoints can be set to reflect the intended grouping of text.
>
> Given the rules
>
> ```text
> footer { wrap-inside: avoid; }
> venue { wrap-inside: avoid; }
> date { wrap-inside: avoid; }
> place { wrap-inside: avoid; }
> ```
>
> and the following markup:
>
> ```text
> <footer>
> <venue>27th Internationalization and Unicode Conference</venue>
> &#8226; <date>April 7, 2005</date> &#8226;
> <place>Berlin, Germany</place>
> </footer>
> ```
>
> In a narrow window the footer could be broken as
>
> ```text
> 27th Internationalization and Unicode Conference •
> April 7, 2005 • Berlin, Germany
> ```
>
> or in a narrower window as
>
> ```text
> 27th Internationalization and Unicode
> Conference • April 7, 2005 •
> Berlin, Germany
> ```
>
> but not as
>
> ```text
> 27th Internationalization and Unicode Conference • April
> 7, 2005 • Berlin, Germany
> ```
<a id="ref-for-propdef-wrap-before①"></a>

<a id="ref-for-propdef-wrap-after①"></a>

### <a id="wrap-before"></a>5.3.  Controlling Breaks Between Boxes: the [wrap-before](#propdef-wrap-before)/[wrap-after](#propdef-wrap-after) properties



| Field               | Definition                                                                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-wrap-before"></a>wrap-before, <a id="propdef-wrap-after"></a>wrap-after                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑨"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid <a id="ref-for-comb-one②⓪"></a>\| avoid-line <a id="ref-for-comb-one②①"></a>\| avoid-flex <a id="ref-for-comb-one②②"></a>\| line <a id="ref-for-comb-one②③"></a>\| flex |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item"></a><a id="ref-for-inline-level"></a>[inline-level](https://www.w3.org/TR/css-display-3/#inline-level) boxes and [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item)                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                 |



<a id="ref-for-flex-line"></a>

These properties specify modifications to break opportunities in line breaking (and [flex line](https://www.w3.org/TR/css-flexbox-1/#flex-line) breaking [\[CSS3-FLEXBOX\]](#biblio-css3-flexbox)). Possible values:

<a id="valdef-wrap-before-auto"></a>auto  
Lines may break at allowed break points before and after the box, as determined by the line-breaking rules in effect.

<a id="valdef-wrap-before-avoid"></a>avoid  
<a id="ref-for-valdef-wrap-before-auto"></a>

Line breaking is suppressed immediately before/after the box: the UA may only break there if there are no other valid break points in the line. If the text breaks, line-breaking restrictions are honored as for [auto](#valdef-wrap-before-auto).

<a id="valdef-wrap-before-avoid-line"></a>avoid-line  
<a id="ref-for-valdef-wrap-before-avoid"></a>

Same as [avoid](#valdef-wrap-before-avoid), but only for line breaks.

<a id="valdef-wrap-before-avoid-flex"></a>avoid-flex  
<a id="ref-for-valdef-wrap-before-avoid①"></a>

Same as [avoid](#valdef-wrap-before-avoid), but only for flex line breaks.

<a id="valdef-wrap-before-line"></a>line  
<a id="ref-for-inline-level①"></a>

Force a line break immediately before/after the box if the box is an [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) box.

<a id="valdef-wrap-before-flex"></a>flex  
<a id="ref-for-multi-line-flex-container"></a>

<a id="ref-for-flex-item①"></a>

<a id="ref-for-flex-line①"></a>

Force a [flex line](https://www.w3.org/TR/css-flexbox-1/#flex-line) break immediately before/after the box if the box is a [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item) in a [multi-line flex container](https://www.w3.org/TR/css-flexbox-1/#multi-line-flex-container).

<a id="ref-for-inline-level②"></a>

<a id="ref-for-inline-box③"></a>

<a id="ref-for-block-level"></a>

<a id="ref-for-block-box"></a>

<a id="ref-for-fragmentation-context"></a>

Forced line breaks on [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) boxes propagate upward through any parent [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) the same way forced breaks on [block-level](https://www.w3.org/TR/css-display-3/#block-level) boxes propagate upward through any parent [block boxes](https://www.w3.org/TR/css-display-3/#block-box) in the same [fragmentation context](https://www.w3.org/TR/css-break-4/#fragmentation-context). [\[CSS3-BREAK\]](#biblio-css3-break)

<a id="ref-for-propdef-text-wrap-style②"></a>

### <a id="text-wrap-style"></a>5.4.  Selecting How to Wrap: the [text-wrap-style](#propdef-text-wrap-style) property



| Field               | Definition                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-wrap-style"></a>text-wrap-style                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②④"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) balance <a id="ref-for-comb-one②⑤"></a>\| stable <a id="ref-for-comb-one②⑥"></a>\| pretty                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-formatting-context①"></a><a id="ref-for-block-container③"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container) hat establish an [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                    |



<a id="ref-for-propdef-text-wrap-mode⑧"></a>

<a id="ref-for-soft-wrap-opportunity①①"></a>

When wrapping is allowed (see [text-wrap-mode](#propdef-text-wrap-mode)), this property selects between several approaches for wrapping lines, trading off between speed, quality and style of layout, or stability. It does not change which [soft wrap opportunity](#soft-wrap-opportunity) exist, but changes how the user agent selects among them. Possible values:

<a id="valdef-text-wrap-style-auto"></a>auto  
<a id="ref-for-valdef-text-wrap-style-balance"></a>

<a id="ref-for-soft-wrap-opportunity①②"></a>

The exact algorithm for selecting which [soft wrap opportunity](#soft-wrap-opportunity) to break at is UA-defined. The algorithm <em>may</em> consider multiple lines when making break decisions. The UA <em>may</em> bias for speed over best layout. The UA <em>must not</em> attempt to even out all lines (including the last) as for [balance](#valdef-text-wrap-style-balance). This value selects the UA’s preferred (or most Web-compatible) wrapping algorithm.

<a id="valdef-text-wrap-style-balance"></a>balance  
<a id="ref-for-propdef-text-wrap①"></a>

<a id="ref-for-valdef-text-wrap-style-auto"></a>

Line breaks are chosen to balance the remaining (empty) space in each line box, if better balance than [auto](#valdef-text-wrap-style-auto) is possible. This should avoid changing—​and in the case of 5 or fewer lines must not change—​the number of line boxes the block would contain if [text-wrap](#propdef-text-wrap) were set to <a id="ref-for-valdef-text-wrap-style-auto①"></a>auto.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The height of the line boxes may nevertheless change, due to changes in which content appears together on one line.

<a id="ref-for-inline-size"></a>

The remaining space to consider is that which remains after placing floats and inline content, but before any adjustments due to text justification. Line boxes are balanced when the standard deviation from the average [inline-size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of the remaining space in each line box is reduced over the block (including lines that end in a forced break).

<a id="ref-for-forced-line-break⑥"></a>

<a id="ref-for-propdef-line-clamp"></a>

Groups of lines separated by a [forced line break](#forced-line-break) are processed separately. If the element is affected by [line-clamp](https://www.w3.org/TR/css-overflow-4/#propdef-line-clamp), the claming effect is applied first, then the remaining lines are balanced.

The exact algorithm is UA-defined.

<a id="ref-for-valdef-text-wrap-style-auto②"></a>

UAs may treat this value as [auto](#valdef-text-wrap-style-auto) if there are more than ten lines to balance.

<a id="valdef-text-wrap-style-stable"></a>stable  
<a id="ref-for-valdef-text-wrap-style-auto③"></a>

Specifies that content on subsequent lines <em>should not</em> be considered when making break decisions so that when editing text any content before the cursor remains stable; otherwise equivalent to [auto](#valdef-text-wrap-style-auto),

<a id="valdef-text-wrap-style-pretty"></a>pretty  
<a id="ref-for-valdef-text-wrap-style-auto④"></a>

Specifies the UA <em>should</em> bias for better layout over speed, and is expected to consider multiple lines, when making break decisions. Otherwise equivalent to [auto](#valdef-text-wrap-style-auto),

<a id="ref-for-valdef-text-wrap-style-auto⑤"></a>

<a id="ref-for-valdef-text-wrap-style-pretty"></a>

<a id="ref-for-valdef-text-wrap-style-balance①"></a>

<a id="ref-for-valdef-text-wrap-style-stable"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [auto](#valdef-text-wrap-style-auto) value will typically map to Web browsers’ speedy legacy line breaking, which has so far used first-fit/greedy algorithms that can often give sub-optimal results. UAs can experiment with better line breaking algorithms with this default value, but as optimal results often take more time, [pretty](#valdef-text-wrap-style-pretty) is offered as an opt-in to take more time for better results. The <a id="ref-for-valdef-text-wrap-style-pretty①"></a>pretty value is intended for body text, where the last line is expected to be a bit shorter than the average line; the [balance](#valdef-text-wrap-style-balance) value is intended for titles and captions, where equal-length lines of text tend to be preferred; and the [stable](#valdef-text-wrap-style-stable) is intended for sections that are, or are likely become toggled as, editable.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="last-line-limits"></a> See [thread](https://www.w3.org/mid/0BD85DFF-A147-44EF-B18A-FF03C3D67EF0@verou.me). Issue is about requiring a minimum length for lines. Common measures seem to be
>
> - At least as long as the text-indent.
> - At least X characters.
> - Percentage-based.
>
> <a id="ref-for-integer-value"></a>
>
> Suggestion for value space is match-indent \| \<length\> \| \<percentage\> (with Xch given as an example to make that use case clear). Alternately [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) could actually count the characters.
>
> It’s unclear how this would interact with text balancing (above); one earlier proposal had them be the same property (with 100% meaning full balancing).
>
> People have requested word-based limits, but since this is really dependent on the length of the word, character-based is better.

<a id="ref-for-propdef-text-wrap②"></a>

### <a id="text-wrap-shorthand"></a>5.5.  Joint Wrapping Control: the [text-wrap](#propdef-text-wrap) shorthand property



| Field               | Definition                                                                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-wrap"></a>text-wrap                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-text-wrap-style③"></a><a id="ref-for-comb-any⑥"></a><a id="ref-for-propdef-text-wrap-mode⑨"></a>[\<'text-wrap-mode'\>](#propdef-text-wrap-mode) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'text-wrap-style'\>](#propdef-text-wrap-style) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | wrap                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                       |



<a id="ref-for-propdef-text-wrap-mode①⓪"></a>

<a id="ref-for-propdef-text-wrap-style④"></a>

<a id="ref-for-longhand③"></a>

<a id="ref-for-initial-value①"></a>

This property is a shorthand for the [text-wrap-mode](#propdef-text-wrap-mode) and [text-wrap-style](#propdef-text-wrap-style) properties. Any omitted [longhand](https://www.w3.org/TR/css-cascade-5/#longhand) is set to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

### <a id="line-break-details"></a>5.6.  Line Breaking Details

<a id="ref-for-line-break"></a>

When determining [line breaks](#line-break):

- <a id="ref-for-line-breaking-process①"></a>

  The interaction of [line breaking](#line-breaking-process) and bidirectional text is defined by [CSS Writing Modes 4 § 2.4 Applying the Bidirectional Reordering Algorithm](https://www.w3.org/TR/css-writing-modes-4/#bidi-algo) and the Unicode Bidirectional Algorithm ([UAX9§3.4 Reordering Resolved Levels](http://unicode.org/reports/tr9/#Reordering_Resolved_Levels) in particular). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4) [\[UAX9\]](#biblio-uax9)

- <a id="ref-for-propdef-overflow-wrap"></a>

  <a id="ref-for-propdef-line-break"></a>

  Except where explicitly defined otherwise (e.g. for [line-break: anywhere](#propdef-line-break) or [overflow-wrap: anywhere](#propdef-overflow-wrap)) line breaking behavior defined for the `CM`, and `SG`, `WJ`, `ZW`, `GL`, and `ZWJ` Unicode line breaking classes must be honored. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-propdef-word-break"></a>

  <a id="ref-for-soft-wrap-opportunity①③"></a>

  <a id="ref-for-propdef-line-break①"></a>

  <a id="ref-for-word-separator①"></a>

  UAs that allow wrapping at punctuation other than [word separators](#word-separator) in writing systems that use them <em>should</em> prioritize breakpoints. (For example, if breaks after slashes are given a lower priority than spaces, the sequence “check /etc” will never break between the "/" and the "e".) As long as care is taken to avoid such awkward breaks, allowing breaks at appropriate punctuation other than <a id="ref-for-word-separator②"></a>word separators is recommended, as it results in more even-looking margins, particularly in narrow measures. The UA may use the width of the containing block, the text’s language, the [line-break](#propdef-line-break) value, and other factors in assigning priorities: CSS does not define prioritization of [soft wrap opportunities](#soft-wrap-opportunity). Prioritization of <a id="ref-for-word-separator③"></a>word separators is not expected, however, if [word-break: break-all](#propdef-word-break) is specified (since this value explicitly requests line breaking behavior not based on breaking at <a id="ref-for-word-separator④"></a>word separators)—​and is forbidden under <a id="ref-for-propdef-line-break②"></a>line-break: anywhere.

- <a id="ref-for-soft-wrap-opportunity①④"></a>

  <a id="ref-for-forced-line-break⑦"></a>

  Out-of-flow elements and inline element boundaries do not introduce a [forced line break](#forced-line-break) or [soft wrap opportunity](#soft-wrap-opportunity) in the flow.

- <a id="ref-for-atomic-inline"></a>

  <a id="ref-for-soft-wrap-opportunity①⑤"></a>

  <a id="atomic-compat-wrap"></a> For Web-compatibility there is a [soft wrap opportunity](#soft-wrap-opportunity) before and after each replaced element or other [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline), even when adjacent to a character that would normally suppress them, including U+00A0 NO-BREAK SPACE. However, with the exception of U+00A0 NO-BREAK SPACE, there must be no <a id="ref-for-soft-wrap-opportunity①⑥"></a>soft wrap opportunity between <a id="ref-for-atomic-inline①"></a>atomic inlines and adjacent characters belonging to the Unicode GL, WJ, or ZWJ line breaking classes. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-propdef-overflow-wrap①"></a>

  <a id="ref-for-propdef-word-break①"></a>

  <a id="ref-for-propdef-line-break③"></a>

  <a id="ref-for-propdef-white-space⑦"></a>

  <a id="ref-for-atomic-inline②"></a>

  <a id="ref-for-soft-wrap-opportunity①⑦"></a>

  For [soft wrap opportunities](#soft-wrap-opportunity) created by characters that disappear at the line break (e.g. U+0020 SPACE), properties on the box directly containing that character control the line breaking at that opportunity. For <a id="ref-for-soft-wrap-opportunity①⑧"></a>soft wrap opportunities defined by the boundary between two characters or [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline), the [white-space](#propdef-white-space) property on the nearest common ancestor of the two characters controls breaking; which elements’ [line-break](#propdef-line-break), [word-break](#propdef-word-break), and [overflow-wrap](#propdef-overflow-wrap) properties control the determination of <a id="ref-for-soft-wrap-opportunity①⑨"></a>soft wrap opportunities at such boundaries is undefined in this level.

- <a id="ref-for-soft-wrap-opportunity②⓪"></a>

  For [soft wrap opportunities](#soft-wrap-opportunity) before the first or after the last character of a box, the break occurs immediately before/after the box (at its margin edge) rather than breaking the box between its content edge and the content.

- Line breaking in/around Ruby is defined in [CSS Ruby Annotation Layout 1 § 3.4 Breaking Across Lines](https://www.w3.org/TR/css-ruby-1/#line-breaks). [\[CSS-RUBY-1\]](#biblio-css-ruby-1)

- <a id="ref-for-hyphenate"></a>

  <a id="ref-for-propdef-overflow-wrap②"></a>

  <a id="ref-for-propdef-line-break④"></a>

  <a id="ref-for-propdef-word-break②"></a>

  <a id="ref-for-soft-wrap-opportunity②①"></a>

  <a id="ref-for-wrapping③"></a>

  <a id="word-break-shaping"></a> When shaping scripts such as Arabic [wrap](#wrapping) at unforced [soft wrap opportunities](#soft-wrap-opportunity) within words (such as when breaking due to [word-break: break-all](#propdef-word-break), [line-break: anywhere](#propdef-line-break), [overflow-wrap: break-word](#propdef-overflow-wrap), <a id="ref-for-propdef-overflow-wrap③"></a>overflow-wrap: anywhere, or when [hyphenating](#hyphenate)) the characters must still be shaped (their joining forms chosen) as if the word were still whole.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-433d41a3"></a> For example, if the word “نوشتن” is broken between the “ش” and “ت”, the “ش” still takes its initial form (“ﺷ”), and the “ت” its medial form (“ﺘ”)—​forming as in “ﻧﻮﺷ \| ﺘﻦ”, not as in “نوش \| تن”.

## <a id="line-breaking"></a>6.  Line Breaking and Word Boundaries

<a id="ref-for-soft-wrap-opportunity②②"></a>

<a id="ref-for-spaces③②"></a>

In most writing systems, in the absence of hyphenation a [soft wrap opportunity](#soft-wrap-opportunity) occurs only at word boundaries. Many such systems use [spaces](#spaces) or punctuation to explicitly separate words, and <a id="ref-for-soft-wrap-opportunity②③"></a>soft wrap opportunities can be identified by these characters.

<a id="ref-for-soft-wrap-opportunity②④"></a>

Scripts such as Thai, Lao, and Khmer, however, do not use spaces or punctuation to separate words. Although the zero width space (U+200B) can be used as an explicit word delimiter in these scripts, this practice is not common. As a result, a lexical resource is needed to correctly identify [soft wrap opportunities](#soft-wrap-opportunity) in such texts.

<a id="ref-for-soft-wrap-opportunity②⑤"></a>

<a id="ref-for-content-language①⓪"></a>

<a id="ref-for-word-boundary-detection"></a>

In some other writing systems, [soft wrap opportunities](#soft-wrap-opportunity) are based on orthographic syllable boundaries, not word boundaries. Some of these systems, notably Brahmic scripts such as Javanese and Balinese, require analysis of the text to find breaking opportunities. Unlike languages that use characters from the SA line breaking class, this analysis does not depend on the [content language](#content-language) nor requires (language specific) [word boundary detection](#word-boundary-detection) or a lexical resource.

<a id="ref-for-typographic-letter-unit①"></a>

In others such as Chinese (as well as Japanese, Yi, and sometimes also Korean), each syllable tends to correspond to a single [typographic letter unit](#typographic-letter-unit), and thus line breaking conventions allow the line to break anywhere <em>except</em> between certain character combinations. Additionally the level of strictness in these restrictions varies with the typesetting style.

<a id="ref-for-soft-wrap-opportunity②⑥"></a>

While CSS does not fully define where [soft wrap opportunities](#soft-wrap-opportunity) occur, some controls are provided to distinguish common variations:

- <a id="ref-for-propdef-line-break⑤"></a>

  The [line-break](#propdef-line-break) property allows choosing various levels of “strictness” for line breaking restrictions.

- <a id="ref-for-propdef-word-break③"></a>

  The [word-break](#propdef-word-break) property controls what types of letters are glommed together to form unbreakable “words”, causing CJK characters to behave like non-CJK text or vice versa, enabling control over word detection in South East Asian Languages, or allowing words to be grouped into phrases…

- <a id="ref-for-propdef-hyphens"></a>

  The [hyphens](#propdef-hyphens) property controls whether automatic hyphenation is allowed to break words in scripts that hyphenate.

- <a id="ref-for-propdef-overflow-wrap④"></a>

  The [overflow-wrap](#propdef-overflow-wrap) property allows the UA to take a break anywhere in otherwise-unbreakable strings that would otherwise overflow.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Unicode Standard Annex \#14: Unicode Line Breaking Algorithm](https://www.unicode.org/reports/tr14/) defines a baseline behavior for line breaking for all scripts in Unicode, which is expected to be further tailored. [\[UAX14\]](#biblio-uax14) More information on line breaking conventions can be found in [Requirements for Japanese Text Layout](https://www.w3.org/TR/jlreq/) [\[JLREQ\]](#biblio-jlreq) and Formatting Rules for Japanese Documents [\[JIS4051\]](#biblio-jis4051) for Japanese, [Requirements for Chinese Text Layout](https://www.w3.org/TR/clreq/) [\[CLREQ\]](#biblio-clreq) and General Rules for Punctuation [\[ZHMARK\]](#biblio-zhmark) for Chinese. See also the [Internationalization Working Group](https://www.w3.org/International/)’s [Language Enablement Index](https://www.w3.org/TR/typography/#blocks_paragraphs) which includes more information on additional languages. [\[TYPOGRAPHY\]](#biblio-typography) Any guidance on additional appropriate references would be much appreciated.

<a id="ref-for-propdef-word-break④"></a>

### <a id="word-break-property"></a>6.1.  Breaking Rules for Letters: the [word-break](#propdef-word-break) property<a id="word-break"></a>



| Field               | Definition                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-word-break"></a>word-break                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⑦"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) break-all <a id="ref-for-comb-one②⑧"></a>\| keep-all <a id="ref-for-comb-one②⑨"></a>\| manual <a id="ref-for-comb-one③⓪"></a>\| auto-phrase <a id="ref-for-comb-one③①"></a>\| break-word |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                              |



<a id="ref-for-soft-wrap-opportunity②⑦"></a>

<a id="ref-for-white-space①⑨"></a>

<a id="ref-for-other-space-separators⑦"></a>

<a id="ref-for-valdef-word-break-auto-phrase"></a>

<a id="ref-for-propdef-line-break⑥"></a>

This property specifies [soft wrap opportunities](#soft-wrap-opportunity) between and within “words”, i.e. where it is “normal” and permissible to break lines of text. It focuses on breaks between letters, and does not define whether and how <a id="ref-for-soft-wrap-opportunity②⑧"></a>soft wrap opportunities are created by [white space](#white-space) and [other space separators](#other-space-separators) (though [auto-phrase](#valdef-word-break-auto-phrase) may suppress some), nor around punctuation. (See [line-break](#propdef-line-break) for controls affecting punctuation and small kana.)

<a id="ref-for-propdef-word-break⑤"></a>

<a id="ref-for-soft-wrap-opportunity②⑨"></a>

<a id="ref-for-typographic-letter-unit②"></a>

<a id="ref-for-letter②"></a>

<a id="ref-for-typographic-character-unit①③"></a>

In particular, [word-break](#propdef-word-break) controls whether a [soft wrap opportunity](#soft-wrap-opportunity) generally exists between adjacent [typographic letter units](#typographic-letter-unit), treating non-[letter](#letter) [typographic character units](#typographic-character-unit) belonging to the `NU`, `AL`, `AI`, or `ID` Unicode line breaking classes as <a id="ref-for-typographic-letter-unit③"></a>typographic letter units for this purpose (only). [\[UAX14\]](#biblio-uax14)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f0d22488"></a> For example, in some styles of CJK typesetting, English words are allowed to break between any two letters, rather than only at spaces or hyphenation points; this can be enabled with word-break:break-all.
>
> ![A snippet of Japanese text with English in it. The word 'caption' is broken into 'capt' and 'ion' across two lines.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/break-all.png)
>
> An example of English text embedded in Japanese being broken at an arbitrary point in the word.
>
> <a id="ref-for-propdef-word-break⑥"></a>
>
> As another example, Korean has two styles of line-breaking: between any two Korean syllables ([word-break: normal](#propdef-word-break)) or, like English, mainly at spaces (<a id="ref-for-propdef-word-break⑦"></a>word-break: keep-all).
>
> ```text
> 각 줄의 마지막에 한글이 올 때 줄 나눔 기
> 준을 “글자” 또는 “어절” 단위로 한다.
> ```
>
> ```text
> 각 줄의 마지막에 한글이 올 때 줄 나눔
> 기준을 “글자” 또는 “어절” 단위로 한다.
> ```
>
> <a id="ref-for-word-separator⑤"></a>
>
> <a id="ref-for-propdef-word-break⑧"></a>
>
> Ethiopic similarly has two styles of line-breaking, either only breaking at [word separators](#word-separator) ([word-break: normal](#propdef-word-break)), or also allowing breaks between letters within a word (<a id="ref-for-propdef-word-break⑨"></a>word-break: break-all).
>
> ```text
> ተወልዱ፡ኵሉ፡ሰብእ፡ግዑዛን፡ወዕሩያን፡
> በማዕረግ፡ወብሕግ።ቦሙ፡ኅሊና፡ወዐቅል፡
> ወይትጌበሩ፡አሐዱ፡ምስለ፡አሀዱ፡
> በመንፈሰ፡እኍና።
> ```
>
> ```text
> ተወልዱ፡ኵሉ፡ሰብእ፡ግዑዛን፡ወዕሩያን፡በማ
> ዕረግ፡ወብሕግ።ቦሙ፡ኅሊና፡ወዐቅል፡ወይትጌ
> በሩ፡አሐዱ፡ምስለ፡አሀዱ፡በመንፈሰ፡እኍና።
> ```
Values have the following meanings:

<a id="valdef-word-break-normal"></a>normal  
Words break according to their customary rules, as described [above](#line-breaking). Korean, which commonly exhibits two different behaviors, allows breaks between any two consecutive Hangul/Hanja. For Ethiopic, which also exhibits two different behaviors, such breaks within words are not allowed.

<a id="ref-for-soft-wrap-opportunity③⓪"></a>

Some writing systems require specific processing to obtain the customarily expected [soft wrap opportunities](#soft-wrap-opportunity), as described in [§ 6.1.1 Analytical Word Breaking](#analytical-word-breaking).

<a id="valdef-word-break-break-all"></a>break-all  
<a id="ref-for-typographic-character-unit①④"></a>

<a id="ref-for-typographic-letter-unit④"></a>

<a id="ref-for-valdef-word-break-normal"></a>

<a id="ref-for-soft-wrap-opportunity③①"></a>

Breaking is allowed within “words”: specifically, in addition to [soft wrap opportunities](#soft-wrap-opportunity) allowed for [normal](#valdef-word-break-normal), any [typographic letter units](#typographic-letter-unit) (and any [typographic character units](#typographic-character-unit) resolving to the `NU` (“numeric”), `AL` (“alphabetic”), or `SA` (“Southeast Asian”) line breaking classes [\[UAX14\]](#biblio-uax14)) are instead treated as `ID` (“ideographic characters”) for the purpose of line-breaking. Hyphenation is not applied.

<a id="ref-for-soft-wrap-opportunity③②"></a>

<a id="ref-for-propdef-line-break⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value does not affect whether there are [soft wrap opportunities](#soft-wrap-opportunity) around punctuation characters. To allow breaks anywhere, see [line-break: anywhere](#propdef-line-break).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This option enables the other common behavior for Ethiopic. It is also often used in a context where the text consists predominantly of CJK characters with only short non-CJK excerpts, and it is desired that the text be better distributed on each line.

<a id="valdef-word-break-keep-all"></a>keep-all  
<a id="ref-for-valdef-word-break-normal①"></a>

<a id="ref-for-valdef-line-break-anywhere"></a>

<a id="ref-for-propdef-line-break⑧"></a>

<a id="ref-for-typographic-character-unit①⑤"></a>

<a id="ref-for-typographic-letter-unit⑤"></a>

<a id="ref-for-soft-wrap-opportunity③③"></a>

Breaking is forbidden within “words”: implicit [soft wrap opportunities](#soft-wrap-opportunity) between [typographic letter units](#typographic-letter-unit) (or other [typographic character units](#typographic-character-unit) belonging to the `NU`, `AL`, `AI`, or `ID` Unicode line breaking classes [\[UAX14\]](#biblio-uax14)) are suppressed, i.e. breaks are prohibited between pairs of such characters (regardless of [line-break](#propdef-line-break) settings other than [anywhere](#valdef-line-break-anywhere)) except where opportunities exist due to [§ 6.1.1.1 Lexical Word Breaking](#lexical-breaking). Otherwise this option is equivalent to [normal](#valdef-word-break-normal). In this style, sequences of CJK characters do not break.

<a id="ref-for-spaces③③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the other common behavior for Korean (which uses [spaces](#spaces) between words), and is also useful for mixed-script text where CJK snippets are mixed into another language that uses <a id="ref-for-spaces③④"></a>spaces for separation.

<a id="valdef-word-break-manual"></a>manual  
<a id="ref-for-soft-wrap-opportunity③④"></a>

<a id="ref-for-valdef-line-break-anywhere①"></a>

<a id="ref-for-propdef-line-break⑨"></a>

<a id="ref-for-typographic-character-unit①⑥"></a>

<a id="ref-for-valdef-word-break-normal②"></a>

Behaves the same as [normal](#valdef-word-break-normal), except that [§ 6.1.1.1 Lexical Word Breaking](#lexical-breaking) must <em>not</em> be performed. Specifically, [typographic character units](#typographic-character-unit) with class SA in [\[UAX14\]](#biblio-uax14) must be treated as if they had class AL (i.e. assuming a value of [line-break](#propdef-line-break) other than [anywhere](#valdef-line-break-anywhere), there is no [soft wrap opportunity](#soft-wrap-opportunity) between pairs of such characters).

<a id="ref-for-soft-wrap-opportunity③⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value does not affect syllable-based [soft wrap opportunities](#soft-wrap-opportunity) within words in languages such as Balinese. (See [§ 6.1.1.2 Orthographic Breaking](#orthographic-breaking) for some discussion of such writing systems.)

<a id="ref-for-valdef-word-break-keep-all"></a>

<a id="ref-for-valdef-word-break-normal③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-35b47d14"></a> alternatively, this value could be based on [keep-all](#valdef-word-break-keep-all) rather than [normal](#valdef-word-break-normal). Yet another variant is to merge this behavior with <a id="ref-for-valdef-word-break-keep-all①"></a>keep-all.

<a id="ref-for-propdef-word-break①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some Southeast Asian languages are commonly misdetected by user agents. With [word-break: normal](#propdef-word-break), they would then apply inappropriate language-specific logic to find wrapping opportunities, and place them inappropriately.
> For instance, many languages other than Thai are written using the Thai script, and line-breaking them as if they were Thai will generally produce inadequate—​and possibly confusing—​results.
>
> <a id="ref-for-propdef-word-break①①"></a>
>
> When this occurs, this value enables authors to turn off the word boundary detection built into user agents for [word-break: normal](#propdef-word-break), so that they can manually mark up the text to indicate wrapping opportunities and obtain sensible results.
>
> <a id="ref-for-the-wbr-element③"></a>
>
> <a id="ref-for-soft-wrap-opportunity③⑥"></a>
>
> > <strong data-conversion-semantic="advisement">Advisement</strong>
> >
> > Authors using this value are expected to manually indicate word boundaries for Southeast Asian languages, using <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> or U+200B. Otherwise, there will be no [soft wrap opportunity](#soft-wrap-opportunity) and the text may overflow.

<a id="valdef-word-break-auto-phrase"></a>auto-phrase  
<a id="ref-for-valdef-word-break-normal④"></a>

Behaves the same as [normal](#valdef-word-break-normal), except that this value directs the user agent to perform language-specific content analysis to prioritize keeping natural phrases (of multiple words) together.

<a id="ref-for-content-language①①"></a>

<a id="ref-for-phrase-boundary-detection②"></a>

<a id="ref-for-valdef-word-break-normal⑤"></a>

<a id="ref-for-soft-wrap-opportunity③⑦"></a>

If the [content language](#content-language) of the element is unknown, or if the user agent does not know how to [detect phrase boundaries](#phrase-boundary-detection) for that particular language, this value must behave as [normal](#valdef-word-break-normal). Otherwise, the user agent should <a id="ref-for-phrase-boundary-detection③"></a>detect phrase boundaries and suppress [soft wrap opportunities](#soft-wrap-opportunity) within each phrase.

<a id="ref-for-content-language①②"></a>

<a id="ref-for-phrase-boundary-detection④"></a>

<a id="ref-for-hyphenation-opportunity"></a>

<a id="ref-for-propdef-hyphens①"></a>

Regardless of the [content language](#content-language) and support for [phrase boundary detection](#phrase-boundary-detection), [hyphenation opportunities](#hyphenation-opportunity) are suppressed as if [hyphens: none](#propdef-hyphens) had been specified.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Whether a word boundary detection system designed for one language is suitable for some or all dialects of that language is somewhat subjective, and this specifications leaves it at the discretion of the user agent. Even if a detection system is not able to cope with all nuances of a particular dialect, it may be reasonable to claim support if the detection correctly recognizes word boundaries most of the time. However, the user agent would do a disservice to authors and users if it claimed support for languages where it fails to detect most word boundaries or has a high error rate.
> <a id="ref-for-valdef-word-break-auto-phrase①"></a>
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-cf343edc"></a> If a user agent has a word-boundary detection system for Cantonese that is not suitable for the broader set of Chinese languages, [auto-phrase](#valdef-word-break-auto-phrase) should have an effect on content marked as lang=yue, lang=zh-yue, or lang=zh-HK, but not lang=zh or lang=zh-Hant.
> > <a id="ref-for-valdef-word-break-auto-phrase②"></a>
> >
> > However, if the user agent supports a generic word-boundary detection system that is suitable for Chinese in general, [auto-phrase](#valdef-word-break-auto-phrase) should have an effect on content marked with the broad lang=zh characterization, as well as any more specific ones, such as lang=zh-yue, lang=zh-Hant-HK, lang=zh-Hans-SG, or lang=zh-hak.

Symbols that line-break the same way as letters of a particular category are affected the same way as those letters.

<a id="ref-for-soft-wrap-opportunity③⑧"></a>

User agents must not, in response to any value of this property, suppress [soft wrap opportunities](#soft-wrap-opportunity) which are:

- <a id="ref-for-the-wbr-element④"></a>

  introduced by the <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> HTML element or U+200B ZERO WIDTH SPACE

- <a id="ref-for-propdef-line-break①⓪"></a>

  required by the [line-break](#propdef-line-break) property

- [surrounding atomic inlines](#atomic-compat-wrap)

<a id="ref-for-propdef-overflow-wrap⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: To control additional break opportunities available only in the case of overflow, see [overflow-wrap](#propdef-overflow-wrap).

<a id="ref-for-propdef-word-break①②"></a>

<a id="ref-for-intrinsic-size"></a>

The effects of [word-break](#propdef-word-break) are taken into account when computing [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cc487741"></a> Here’s a mixed-script sample text:
>
> ```text
> 这是一些汉字 and some Latin و کمی خط عربی และตัวอย่างการเขียนภาษาไทย በጽሑፍ፡ማራዘሙን፡አንዳንድ፡
> ```
>
> The break-points are determined as follows (indicated by ‘·’):
>
> <a id="ref-for-propdef-word-break①③"></a>
>
> [word-break: normal](#propdef-word-break)
>
> ```text
> 这·是·一·些·汉·字·and·some·Latin·و·کمی·خط·عربی·และ·ตัวอย่าง·การเขียน·ภาษาไทย·በጽሑፍ፡·ማራዘሙን፡·አንዳንድ፡
> ```
>
> <a id="ref-for-propdef-word-break①④"></a>
>
> [word-break: break-all](#propdef-word-break)
>
> ```text
> 这·是·一·些·汉·字·a·n·d·s·o·m·e·L·a·t·i·n·و·ﮐ·ﻤ·ﻰ·ﺧ·ﻁ·ﻋ·ﺮ·ﺑ·ﻰ·แ·ล·ะ·ตั·ว·อ·ย่·า·ง·ก·า·ร·เ·ขี·ย·น·ภ·า·ษ·า·ไ·ท·ย·በ·ጽ·ሑ·ፍ፡·ማ·ራ·ዘ·ሙ·ን፡·አ·ን·ዳ·ን·ድ፡
> ```
>
> <a id="ref-for-propdef-word-break①⑤"></a>
>
> [word-break: keep-all](#propdef-word-break)
>
> ```text
> 这是一些汉字·and·some·Latin·و·کمی·خط·عربی·และ·ตัวอย่าง·การเขียน·ภาษาไทย·በጽሑፍ፡·ማራዘሙን፡·አንዳንድ፡
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="jp-title-break"></a>
>
> Japanese is usually typeset allowing line breaks between each syllable. However, it is sometimes preferred to suppress these wrapping opportunities and to only allow wrapping at the end of certain sentence fragments. This is most commonly done in very short pieces of text, such as headings and table or figure captions.
>
> <a id="ref-for-the-wbr-element⑤"></a>
>
> <a id="ref-for-propdef-word-break①⑥"></a>
>
> This can be achieved manually by marking the allowed wrapping points with <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> or U+200B ZERO WIDTH SPACE, and suppressing the other ones using [word-break: keep-all](#propdef-word-break).
>
> <a id="ref-for-propdef-word-break①⑦"></a>
>
> <a id="ref-for-content-language①③"></a>
>
> Alternatively, in user agents that support detection of phrases boundaries in Japanese, the same results can be achieved automatically by using [word-break: auto-phrase](#propdef-word-break) (assuming the [content language](#content-language) is specified).
>
> **Table 15**
>
> Static transcription of the source’s live browser demonstration. Browser text is source content; this Markdown does not run the demonstration. Original demonstration HTML/CSS follows each case. See the [source demonstration](https://www.w3.org/TR/2024/WD-css-text-4-20240529/#jp-title-break).
>
> **Source demonstration stylesheet**
>
> The source encloses these demonstration cells in the `jp-title-break` container.
>
> ```css
> #jp-title-break td samp,
> #jp-title-break td pre {
> 	font-size: 1.5em;
> 	width: 6em;
> 	line-height: 2;
> 	padding: 0.5em 1em;
> 	display: block;
> 	margin: auto;
> 	border: solid gray 1px;
> 	background: white;
> }
> ```
>
> ```css
> samp[lang="ja"] { font-family: "MS Gothic", "Osaka-Mono", monospace }
> ```
>
> **Example 1**
>
> **Sample markup and style rule**
>
> ```text
> <h1>
> 窓ぎわの<wbr>トットちゃん
> </h1>
> ```
>
> ```text
> h1 {
>   word-break: normal;
> }
> ```
>
> **Expected rendering**
>
> ```text
> 窓ぎわのトッ
> トちゃん
> ```
>
> **Result in your browser**
>
> `  窓ぎわのトットちゃん  `
>
> **Original browser-demonstration HTML**
>
> ```html
> <samp lang="ja"> 窓ぎわの<wbr>トットちゃん </wbr></samp>
> ```
>
> **Example 2**
>
> **Sample markup and style rule**
>
> ```text
> <h1>
> 窓ぎわの<wbr>トットちゃん
> </h1>
> ```
>
> ```text
> h1 {
>   word-break: keep-all;
> }
> ```
>
> **Expected rendering**
>
> ```text
> 窓ぎわの
> トットちゃん
> ```
>
> **Result in your browser**
>
> `  窓ぎわのトットちゃん  `
>
> **Original browser-demonstration HTML**
>
> ```html
> <samp lang="ja" style="word-break:keep-all"> 窓ぎわの<wbr>トットちゃん </wbr></samp>
> ```
>
> **Example 3**
>
> **Sample markup and style rule**
>
> ```text
> <h1 lang=ja>
> 窓ぎわのトットちゃん
> </h1>
> ```
>
> ```text
> h1 {
>   word-break: auto-phrase;
> }
> ```
>
> **Expected rendering**
>
> ```text
> 窓ぎわの
> トットちゃん
> ```
>
> **Result in your browser**
>
> `  窓ぎわのトットちゃん  `
>
> **Original browser-demonstration HTML**
>
> ```html
> <samp lang="ja" style="word-break:auto-phrase"> 窓ぎわのトットちゃん </samp>
> ```

<a id="ref-for-valdef-word-break-break-all"></a>

When shaping scripts such as Arabic are allowed to break within words due to [break-all](#valdef-word-break-break-all) the characters must still be shaped as if the word were [not broken](#word-break-shaping).

<a id="ref-for-propdef-word-break①⑧"></a>

<a id="ref-for-propdef-overflow-wrap⑥"></a>

For compatibility with legacy content, the [word-break](#propdef-word-break) property also supports a deprecated <a id="valdef-word-break-break-word"></a>break-word keyword. When specified, this has the same effect as <a id="ref-for-propdef-word-break①⑨"></a>word-break: normal and [overflow-wrap: anywhere](#propdef-overflow-wrap), regardless of the actual value of the <a id="ref-for-propdef-overflow-wrap⑦"></a>overflow-wrap property.

#### <a id="analytical-word-breaking"></a>6.1.1.  Analytical Word Breaking

##### <a id="lexical-breaking"></a>6.1.1.1.  Lexical Word Breaking

<a id="ref-for-valdef-word-break-normal⑥"></a>

<a id="ref-for-typographic-character-unit①⑦"></a>

<a id="ref-for-word-boundary-detection①"></a>

<a id="ref-for-soft-wrap-opportunity③⑨"></a>

To provide the expected [normal](#valdef-word-break-normal) behavior for Southeast Asian languages, [typographic character units](#typographic-character-unit) with line breaking class SA in [\[UAX14\]](#biblio-uax14) must be treated as if they had class AL. However, the user agent must additionally analyze the content of a run of such characters to [detect word boundaries](#word-boundary-detection) and treat each boundary as a [soft wrap opportunities](#soft-wrap-opportunity).

<a id="ref-for-content-language①④"></a>

As various languages can be written in scripts which use the characters with class SA, if the [content language](#content-language) is known, the user agent should use this information to tailor its analysis.

##### <a id="orthographic-breaking"></a>6.1.1.2.  Orthographic Breaking

<a id="ref-for-soft-wrap-opportunity④⓪"></a>

For scripts (such as Balinese) that use line breaking based on orthographic syllables, the UA must analyze the content to find the correct points to insert [soft wrap opportunities](#soft-wrap-opportunity).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing, Unicode does not define how to perform this analysis, and treats all characters in these writing systems as having line breaking class AL. However, [a proposal](https://www.unicode.org/L2/L2022/22080r-line-break-ortho-bnd.pdf) has been made to solve that issue. Even though it is not final, it can be informative to consult. [\[L2-22-080R\]](#biblio-l2-22-080r)

##### <a id="fallback-breaking"></a>6.1.1.3.  Fallback Breaking

<a id="ref-for-content-language①⑤"></a>

<a id="ref-for-soft-wrap-opportunity④①"></a>

<a id="ref-for-typographic-letter-unit⑥"></a>

In order to avoid unexpected overflow, if the user agent is unable to perform the requisite lexical or orthographic analysis for line breaking any [content language](#content-language) that requires it—​for example due to lacking a dictionary for languages written in characters with class SA—​it must assume a [soft wrap opportunity](#soft-wrap-opportunity) between pairs of [typographic letter units](#typographic-letter-unit) in that writing system.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This provision is not triggered merely when the UA fails to find a word boundary in a particular text run; the text run may well be a single unbreakable word. It applies for example when a text run is composed of Khmer characters (U+1780 to U+17FF) if the user agent does not know how to determine word boundaries in Khmer.

#### <a id="phrase-pref"></a>6.1.2.  Expressing User Preferences for Phrase-based Line Breaking

<a id="ref-for-valdef-word-break-auto-phrase③"></a>

<a id="ref-for-declared-value"></a>

<a id="ref-for-propdef-word-break②⓪"></a>

<a id="ref-for-cascade-origin-user"></a>

User agents may activate language-specific content analysis described in [auto-phrase](#valdef-word-break-auto-phrase) in response to user preferences. User agents with this behavior must do this by setting the [declared value](https://www.w3.org/TR/css-cascade-5/#declared-value) of [word-break](#propdef-word-break) to <a id="ref-for-valdef-word-break-auto-phrase④"></a>auto-phrase in the [user origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-user).

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-cascade-origin-author"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This allows authors to detect whether or not this feature is enabled by calling <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>, or to override it in the [author origin](https://www.w3.org/TR/css-cascade-5/#cascade-origin-author) where appropriate.

<a id="ref-for-propdef-line-break①①"></a>

### <a id="line-break-property"></a>6.2.  Line Breaking Strictness: the [line-break](#propdef-line-break) property



| Field               | Definition                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-line-break"></a>line-break                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③②"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) loose <a id="ref-for-comb-one③③"></a>\| normal <a id="ref-for-comb-one③④"></a>\| strict <a id="ref-for-comb-one③⑤"></a>\| anywhere |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                  |



<a id="ref-for-wrapping④"></a>

This property specifies the strictness of line-breaking rules applied within an element: especially how [wrapping](#wrapping) interacts with punctuation and symbols. Values have the following meanings:

<a id="valdef-line-break-auto"></a>auto  
The UA determines the set of line-breaking restrictions to use, and it may vary the restrictions based on the length of the line; e.g., use a less restrictive set of line-break rules for short lines.

<a id="valdef-line-break-loose"></a>loose  
Breaks text using the least restrictive set of line-breaking rules. Typically used for short lines, such as in newspapers.

<a id="valdef-line-break-normal"></a>normal  
Breaks text using the most common set of line-breaking rules.

<a id="valdef-line-break-strict"></a>strict  
Breaks text using the most stringent set of line-breaking rules.

<a id="valdef-line-break-anywhere"></a>anywhere  
<a id="ref-for-propdef-word-break②①"></a>

<a id="ref-for-preserved-white-space①③"></a>

<a id="ref-for-typographic-character-unit①⑧"></a>

<a id="ref-for-soft-wrap-opportunity④②"></a>

There is a [soft wrap opportunity](#soft-wrap-opportunity) around every [typographic character unit](#typographic-character-unit), including around any punctuation character or [preserved white spaces](#preserved-white-space), or in the middle of words, disregarding any prohibition against line breaks, even those introduced by characters with the `GL`, `WJ`, or `ZWJ` line breaking classes or mandated by the [word-break](#propdef-word-break) property. [\[UAX14\]](#biblio-uax14) The different wrapping opportunities must not be prioritized. Hyphenation is not applied.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value triggers the line breaking rules typically seen in terminals.

<a id="ref-for-soft-wrap-opportunity④③"></a>

<a id="ref-for-typographic-character-unit①⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This values only creates [soft wrap opportunities](#soft-wrap-opportunity) <em>between</em> [typographic character units](#typographic-character-unit), not within them. Consequently, the `CM` and `SG` Unicode line breaking classes must be honored. [\[UAX14\]](#biblio-uax14)

<a id="ref-for-valdef-line-break-anywhere②"></a>

<a id="ref-for-preserved-white-space①④"></a>

<a id="ref-for-propdef-white-space⑧"></a>

<a id="ref-for-valdef-white-space-break-spaces"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [anywhere](#valdef-line-break-anywhere) only allows [preserved white spaces](#preserved-white-space) at the end of the line to be wrapped to the next line when [white-space](#propdef-white-space) is set to [break-spaces](https://www.w3.org/TR/css-text-3/#valdef-white-space-break-spaces), because in other cases:
> - <a id="ref-for-valdef-white-space-pre-line①"></a>
>
>   <a id="ref-for-valdef-white-space-normal②"></a>
>
>   <a id="ref-for-preserved-white-space①⑤"></a>
>
>   [preserved white space](#preserved-white-space) at the end/start of the line is discarded ([normal](#valdef-white-space-normal), [pre-line](#valdef-white-space-pre-line))
>
> - <a id="ref-for-valdef-white-space-pre①"></a>
>
>   <a id="ref-for-valdef-white-space-nowrap①"></a>
>
>   wrapping is forbidden altogether ([nowrap](https://www.w3.org/TR/css-text-3/#valdef-white-space-nowrap), [pre](#valdef-white-space-pre))
>
> - <a id="ref-for-valdef-white-space-pre-wrap①"></a>
>
>   <a id="ref-for-hang⑨"></a>
>
>   <a id="ref-for-preserved-white-space①⑥"></a>
>
>   the [preserved white space](#preserved-white-space) [hang](#hang) ([pre-wrap](#valdef-white-space-pre-wrap)).
>
> <a id="ref-for-preserved-white-space①⑦"></a>
>
> <a id="ref-for-propdef-white-space⑨"></a>
>
> <a id="ref-for-valdef-white-space-collapse-break-spaces③"></a>
>
> When it does have an effect on [preserved white space](#preserved-white-space), with [white-space: break-spaces](#propdef-white-space), it allows breaking before the first space of a sequence, which [break-spaces](#valdef-white-space-collapse-break-spaces) on its own does not.

<a id="ref-for-valdef-line-break-loose"></a>

<a id="ref-for-valdef-line-break-normal"></a>

<a id="ref-for-valdef-line-break-strict"></a>

CSS distinguishes between four levels of strictness in the rules for text wrapping. The precise set of rules in effect for each of [loose](#valdef-line-break-loose), [normal](#valdef-line-break-normal), and [strict](#valdef-line-break-strict) is up to the UA and should follow language conventions. However, this specification does require that:

- <a id="ref-for-valdef-line-break-loose①"></a>

  <a id="ref-for-valdef-line-break-normal①"></a>

  <a id="ref-for-valdef-line-break-strict①"></a>

  The following breaks are forbidden in [strict](#valdef-line-break-strict) line breaking and allowed in [normal](#valdef-line-break-normal) and [loose](#valdef-line-break-loose):

  - breaks before Japanese small kana or the Katakana-Hiragana prolonged sound mark, i.e. characters from the Unicode line breaking class `CJ`. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-writing-system-japanese"></a>

  <a id="ref-for-writing-system-chinese"></a>

  <a id="ref-for-content-writing-system①"></a>

  <a id="ref-for-valdef-line-break-loose②"></a>

  <a id="ref-for-valdef-line-break-normal②"></a>

  The following breaks are allowed for [normal](#valdef-line-break-normal) and [loose](#valdef-line-break-loose) line breaking if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese), and are otherwise forbidden:

  - breaks before certain CJK hyphen-like characters:  
    〜 U+301C, ゠ U+30A0

- <a id="ref-for-propdef-word-break②②"></a>

  <a id="ref-for-valdef-line-break-loose③"></a>

  The following breaks are allowed for [loose](#valdef-line-break-loose) line breaking if the preceding character belongs to the Unicode line breaking class `ID` [\[UAX14\]](#biblio-uax14) (including when the preceding character is treated as `ID` due to [word-break: break-all](#propdef-word-break)), and are otherwise forbidden:

  - breaks before hyphens:  
    ‐ U+2010, – U+2013

- <a id="ref-for-valdef-line-break-loose④"></a>

  <a id="ref-for-valdef-line-break-strict②"></a>

  <a id="ref-for-valdef-line-break-normal③"></a>

  The following breaks are forbidden for [normal](#valdef-line-break-normal) and [strict](#valdef-line-break-strict) line breaking and allowed in [loose](#valdef-line-break-loose):

  - breaks before iteration marks:  
    々 U+3005, 〻 U+303B, ゝ U+309D, ゞ U+309E, ヽ U+30FD, ヾ U+30FE
  - breaks between inseparable characters (such as ‥ U+2025, … U+2026) i.e. characters from the Unicode line breaking class `IN`. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-writing-system-japanese①"></a>

  <a id="ref-for-writing-system-chinese①"></a>

  <a id="ref-for-content-writing-system②"></a>

  <a id="ref-for-valdef-line-break-loose⑤"></a>

  The following breaks are allowed for [loose](#valdef-line-break-loose) if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese) and are otherwise forbidden:

  - breaks before certain centered punctuation marks:  
    ・ U+30FB, ： U+FF1A, ； U+FF1B, ･ U+FF65, ‼ U+203C, ⁇ U+2047, ⁈ U+2048, ⁉ U+2049, ！ U+FF01, ？ U+FF1F

  - <a id="ref-for-unicode-east-asian-width"></a>

    breaks before suffixes:  
    Characters with the Unicode line breaking class `PO` [\[UAX14\]](#biblio-uax14) and the [East Asian Width property](#unicode-east-asian-width) [\[UAX11\]](#biblio-uax11) `Ambiguous`, `Fullwidth`, or `Wide`.

  - <a id="ref-for-unicode-east-asian-width①"></a>

    breaks after prefixes:  
    Characters with the Unicode line breaking class `PR` [\[UAX14\]](#biblio-uax14) and the [East Asian Width property](#unicode-east-asian-width) [\[UAX11\]](#biblio-uax11) `Ambiguous`, `Fullwidth`, or `Wide`.

<a id="ref-for-propdef-line-break①②"></a>

<a id="ref-for-writing-system-chinese②"></a>

<a id="ref-for-writing-system-japanese②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The requirements listed above only create distinctions in CJK text. In an implementation that matches only the rules above, and no additional rules, [line-break](#propdef-line-break) would only affect CJK code points unless the writing system is tagged as [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese). Future levels may add additional specific rules for other writing systems and languages as their requirements become known.

<a id="ref-for-valdef-line-break-strict③"></a>

<a id="ref-for-valdef-line-break-normal④"></a>

<a id="ref-for-valdef-line-break-loose⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-038e656c"></a> As UAs can add additional distinctions between [strict](#valdef-line-break-strict)/[normal](#valdef-line-break-normal)/[loose](#valdef-line-break-loose) modes, these values can exhibit differences in other writing systems as well. For example, a UA with sufficiently-advanced Thai language processing ability could choose to map different levels of strictness in Thai line-breaking to these keywords, e.g. disallowing breaks within compound words in <a id="ref-for-valdef-line-break-strict④"></a>strict mode (e.g. breaking ตัวอย&#xE48;างการเขียนภาษาไทย as ตัวอย&#xE48;าง·การเขียน·ภาษาไทย) while allowing more breaks in <a id="ref-for-valdef-line-break-loose⑦"></a>loose (ตัวอย&#xE48;าง·การ·เขียน·ภาษา·ไทย).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While user agents can refer to [\[UAX14\]](#biblio-uax14) as a starting point for their line-breaking implementation, the following deviations could be desirable for maximum interoperability with existing implementations:
>
> - <a id="ref-for-unicode-general-category④"></a>
>
>   Not introducing a line break opportunity between U+0021 (Exclamation Mark, `!`) and a letter ([Unicode general category](#unicode-general-category) `L`). This prevents a break in the string “!important”.
>
> - <a id="ref-for-unicode-general-category⑤"></a>
>
>   Not introducing a line break opportunity between U+002F (Solidus, `/`) and a letter ([Unicode general category](#unicode-general-category) `L`). This prevents a break in dates such as “23/Jan/2024”.
>
> - <a id="ref-for-unicode-general-category⑥"></a>
>
>   Not introducing a line break opportunity between U+007C (Vertical Line, `|`) and a letter ([Unicode general category](#unicode-general-category) `L`).
>
> - <a id="ref-for-unicode-general-category⑦"></a>
>
>   Not introducing a line break opportunity between U+002D (Hyphen-Minus \`-\`) and a digit ([Unicode general category](#unicode-general-category) `Nd`) if the codepoint prior to the hyphen was \_not\_ a letter or digit (<a id="ref-for-unicode-general-category⑧"></a>Unicode general category `L` or `Nd`). This prevents breaking after the minus sign before a number such as in “-13” whilst allowing breaks after the hyphen in “ABCD-1234” and “1234-5678” which may appear in long URLs, for example.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSSWG recognizes that in a future edition of the specification finer control over line breaking may be necessary to satisfy high-end publishing requirements.

### <a id="hyphenation"></a>6.3.  Hyphenation: Morphological Breaking Within Words

<a id="ref-for-propdef-hyphens②"></a>

#### <a id="hyphens-property"></a>6.3.1.  Hyphenation Control: the [hyphens](#propdef-hyphens) property

<a id="hyphenate"></a>Hyphenation is the controlled splitting of words where they usually would not be allowed to break to improve the layout of paragraphs, typically splitting words at syllabic or morphemic boundaries and often visually indicating the split (usually by inserting a hyphen, U+2010). In some cases, hyphenation may also alter the spelling of a word. Regardless, hyphenation is a rendering effect only: it must have no effect on the underlying document content or on text selection or searching.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-17746cb4"></a> Hyphenation practices vary across languages, and can involve not just inserting a hyphen before the line break, but inserting a hyphen after the break (or both), inserting a different character than U+2010, or changing the spelling of the word.
>
> **Table 17**
>
> Representation note: complete merged-header paths are explicit; inherited span values are repeated where they apply. Native HTML span and row-header accessibility semantics are not available in GFM.
>
> | Language | Unbroken | Before | After |
> | --- | --- | --- | --- |
> | **English** | Unbroken | Un‐ | broken |
> | **Dutch** | cafeetje | café‐ | tje |
> | **Hungarian** | Összeg | Ösz‐ | szeg |
> | **Mandarin** | tú’àn | tú‐ | àn |
> | **Mandarin** | àizēng‐fēnmíng | àizēng‐ | ‐fēnmíng |
> | **Uyghur** | ![ \[isolated DAL + isolated ALEF + initial MEEM + medial YEH + final DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/uyghur-unbroken.svg) | ![\[isolated DAL + isolated ALEF + initial MEEM + final YEH + hyphen \]](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/uyghur-hyphenate-joined-before.svg) | ![\[ isolated DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/uyghur-hyphenate-joined-after.svg) |
> | **Cree** | ![\[ᑲᓯᑕᓂᐘᓂᓂᐠ\] (CANADIAN SYLLABICS KA + CANADIAN SYLLABICS SI + CANADIAN SYLLABICS TA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS WEST-CREE WA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS FINAL GRAVE)](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/cree.svg) | ![\[ᑲᓯᑕᓂ᐀\] (CANADIAN SYLLABICS KA + CANADIAN SYLLABICS SI + CANADIAN SYLLABICS TA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS HYPHEN)](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/cree-before.svg) | ![\[ᐘᓂᓂᐠ\] (CANADIAN SYLLABICS WEST-CREE WA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS FINAL GRAVE)](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/cree-after.svg) |

<a id="ref-for-soft-wrap-opportunity④④"></a>

<a id="ref-for-hyphenate①"></a>

<a id="ref-for-hyphenation-opportunity①"></a>

<a id="ref-for-propdef-hyphens③"></a>

Hyphenation occurs when the line breaks at a valid <a id="hyphenation-opportunity"></a>hyphenation opportunity, which is a type of [soft wrap opportunity](#soft-wrap-opportunity) that exists within a word where [hyphenation](#hyphenate) is allowed. In CSS [hyphenation opportunities](#hyphenation-opportunity) are controlled with the [hyphens](#propdef-hyphens) property. CSS Text Level 3 does not define the exact rules for <a id="ref-for-hyphenate②"></a>hyphenation; however UAs are strongly encouraged to optimize their choice of break points and to chose language-appropriate hyphenation points.

<a id="ref-for-soft-wrap-opportunity④⑤"></a>

<a id="ref-for-hyphenation-opportunity②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [soft wrap opportunity](#soft-wrap-opportunity) introduced by the U+002D - HYPHEN-MINUS character or the U+2010 ‐ HYPHEN character is not a [hyphenation opportunity](#hyphenation-opportunity), as no visual indication of the split is <em>created</em> when wrapping: these characters are visible whether the line is wrapped at that point or not.

<a id="ref-for-min-content"></a>

Hyphenation opportunities <em>are</em> considered when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This allows tables to hyphenate their contents instead of overflowing their containing block, which is particularly important in long-word languages like German.



| Field               | Definition                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphens"></a>hyphens                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) manual <a id="ref-for-comb-one③⑦"></a>\| auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | manual                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                     |



<a id="ref-for-hyphenate③"></a>

<a id="ref-for-soft-wrap-opportunity④⑥"></a>

This property controls whether [hyphenation](#hyphenate) is allowed to create more [soft wrap opportunities](#soft-wrap-opportunity) within a line of text. Values have the following meanings:

<a id="valdef-hyphens-none"></a>none  
<a id="ref-for-hyphenation-opportunity③"></a>

Words are not hyphenated, even if characters inside the word explicitly define [hyphenation opportunities](#hyphenation-opportunity).

<a id="ref-for-soft-wrap-opportunity④⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not suppress the existing [soft wrap opportunities](#soft-wrap-opportunity) introduced by always visible characters such as U+002D - HYPHEN-MINUS or U+2010 ‐ HYPHEN.

<a id="valdef-hyphens-manual"></a>manual  
<a id="ref-for-hyphenation-opportunity④"></a>

Words are only hyphenated where there are characters inside the word that explicitly suggest [hyphenation opportunities](#hyphenation-opportunity). The UA must use the appropriate language-specific hyphenation character(s) and should apply any appropriate spelling changes just as for automatic hyphenation at the same point.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b6db0d14"></a> In Unicode, U+00AD is a conditional "soft hyphen" and U+2010 is an unconditional hyphen. Unicode Standard Annex \#14 describes the [role of soft hyphens in](http://unicode.org/reports/tr14/#SoftHyphen) Unicode line breaking. [\[UAX14\]](#biblio-uax14) In HTML, &#x26;shy; represents the soft hyphen character, which suggests a hyphenation opportunity.
> ```text
> ex&shy;ample
> ```
<a id="valdef-hyphens-auto"></a>auto  
<a id="ref-for-hyphenation-opportunity⑤"></a>

Words may be broken at [hyphenation opportunities](#hyphenation-opportunity) determined automatically by a language-appropriate hyphenation resource in addition to those indicated explicitly by a conditional hyphen. Automatic <a id="ref-for-hyphenation-opportunity⑥"></a>hyphenation opportunities elsewhere within a word must be ignored if the word contains a conditional hyphen (&#x26;shy; or U+00AD SOFT HYPHEN), in favor of the conditional hyphen(s). However, if, even after breaking at such opportunities, a portion of that word is still too long to fit on one line, an automatic hyphenation opportunity may be used.

<a id="ref-for-content-language①⑥"></a>

Correct automatic hyphenation requires a hyphenation resource appropriate to the language of the text being broken. The UA must therefore only automatically hyphenate text for which the [content language](#content-language) is known and for which it has an appropriate hyphenation resource.

<a id="ref-for-content-language①⑦"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should correctly tag their content’s [language](#content-language) (e.g. using the HTML `lang` attribute or XML `xml:lang` attribute) in order to obtain correct automatic hyphenation.

The UA may use language-tailored heuristics to exclude certain words from automatic hyphenation. For example, a UA might try to avoid hyphenation in proper nouns by excluding words matching certain capitalization and punctuation patterns. Such heuristics are not defined by this specification. (Note that such heuristics will need to vary by language: English and German, for example, have very different capitalization conventions.)

<a id="ref-for-propdef-hyphens④"></a>

For the purpose of the [hyphens](#propdef-hyphens) property, what constitutes a “word” is UA-dependent. However, inline element boundaries and out-of-flow elements must be ignored when determining word boundaries.

<a id="ref-for-hyphenation-opportunity⑦"></a>

Any glyphs shown due to hyphenation at a [hyphenation opportunity](#hyphenation-opportunity) created by a conditional hyphen character (such as U+00AD SOFT HYPHEN) are represented by that character and are styled according to the properties applied to it.

When shaping scripts such as Arabic are allowed to break within words due to hyphenation, the characters must still be shaped as if the word were [not broken](#word-break-shaping).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4faf88c1"></a> For example, if the Uyghur word “داميدى” were hyphenated, it would appear as ![\[isolated DAL + isolated ALEF + initial MEEM + medial YEH + hyphen + line-break + final DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/uyghur-hyphenate-joined.svg) not as ![\[isolated DAL + isolated ALEF + initial MEEM + final YEH + hyphen + line-break + isolated DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/uyghur-hyphenate-unjoined.svg) .

<a id="ref-for-propdef-hyphenate-character"></a>

#### <a id="hyphenate-character"></a>6.3.2.  Hyphens: the [hyphenate-character](#propdef-hyphenate-character) property



| Field               | Definition                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphenate-character"></a>hyphenate-character                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-string-value"></a><a id="ref-for-comb-one③⑧"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                     |



This property specifies the string that is shown between parts of hyphenated words. Values have the following meanings:

<a id="valdef-hyphenate-character-auto"></a>auto

<a id="ref-for-content-language①⑧"></a>

Specifies that the user agent should find an appropriate string based on the [content language](#content-language)’s typographic conventions, possibly from the same source as the hyphenation dictionary.

<a id="ref-for-string-value①"></a>

<a id="valdef-hyphenate-character-string"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value)

<a id="ref-for-typographic-character-unit②⓪"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-content-language①⑨"></a>

Specifies the string that appears at the hyphenation break when hyphenating. (The position of this string is not affected: the UA must insert the string according to the typographic conventions of the [content language](#content-language), defaulting to immediately before the hyphenation break.) The UA <em>may</em> truncate the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) to a limited number of [typographic character units](#typographic-character-unit); it must not truncate only part of a <a id="ref-for-typographic-character-unit②①"></a>typographic character unit.

<a id="ref-for-hyphenation-opportunity⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifying the empty string "" is valid, and causes the UA to break at [hyphenation opportunities](#hyphenation-opportunity) without inserting a visible hyphenation character.

<a id="ref-for-propdef-hyphenate-character①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-011cd0e8"></a> The hyphen character (U+2010) is most typically used to indicate that a word has been split. However, [hyphenate-character](#propdef-hyphenate-character) can be used to specify a different type of hyphen when necessary.
>
> ```text
> [lang]:lang(ojs) { hyphenate-character: "᐀" /* CANADIAN SYLLABICS HYPHEN (U+1400) */ }
> ```
<a id="ref-for-propdef-hyphenate-character②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Both hyphens triggered by automatic hyphenation and hyphens triggered by soft hyphens are rendered according to [hyphenate-character](#propdef-hyphenate-character).

<a id="ref-for-propdef-hyphenate-limit-zone"></a>

#### <a id="hyphenate-size-limits"></a>6.3.3.  Hyphenation Size Limit: the [hyphenate-limit-zone](#propdef-hyphenate-limit-zone) property



| Field               | Definition                                                                                                               |
|---------------------|--------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphenate-limit-zone"></a>hyphenate-limit-zone                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-block-container④"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container)                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refers to length of the line box                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                   |



<a id="ref-for-propdef-hyphenate-limit-zone①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9a19edc4"></a>Is [hyphenate-limit-zone](#propdef-hyphenate-limit-zone) a good name? Comments/suggestions?

This property specifies the maximum amount of unfilled space (before justification) that may be left in the line box before hyphenation is triggered to pull part of a word from the next line back up into the current line.

<a id="ref-for-propdef-hyphenate-limit-chars"></a>

#### <a id="hyphenate-char-limits"></a>6.3.4.  Hyphenation Character Limits: the [hyphenate-limit-chars](#propdef-hyphenate-limit-chars) property



| Field               | Definition                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphenate-limit-chars"></a>hyphenate-limit-chars                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-integer-value①"></a><a id="ref-for-comb-one③⑨"></a>\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) \][{1,3}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-hyphenate-limit-chars-auto"></a>three values, each either the [auto](#valdef-hyphenate-limit-chars-auto) keyword or an integer                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                             |



This property specifies the minimum number of characters in a hyphenated word. If the word does not meet the required minimum number of characters in the word / before the hyphen / after the hyphen, then the word must not be hyphenated. Nonspacing combining marks (Unicode General Category Mn) and intra-word punctuation (Unicode General Category P\*) do not count towards the minimum.

<a id="ref-for-valdef-hyphenate-limit-chars-auto①"></a>

If three values are specified, the first value is the required minimum for the total characters in a word, the second value is the minimum for characters before the hyphenation point, and the third value is the minimum for characters after the hyphenation point. If the third value is missing, it is the same as the second. If the second value is missing, then it is [auto](#valdef-hyphenate-limit-chars-auto). The <a id="valdef-hyphenate-limit-chars-auto"></a>auto value means that the UA chooses a value that adapts to the current layout.

<a id="ref-for-valdef-hyphenate-limit-chars-auto②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unless the UA is able to calculate a better value, it is suggested that [auto](#valdef-hyphenate-limit-chars-auto) means 2 for before and after, and 5 for the word total.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4ddcbe59"></a> In the example below, the minimum size of a hyphenated word is left to the UA (which means it may vary depending on the language, the length of the line, or other factors), but the minimum number of characters before and after the hyphenation point is set to 3.
>
> ```text
> p { hyphenate-limit-chars: auto 3; }
> ```
<a id="ref-for-propdef-hyphenate-limit-lines"></a>

<a id="ref-for-propdef-hyphenate-limit-last"></a>

#### <a id="hyphenate-line-limits"></a>6.3.5.  Hyphenation Line Limits: the [hyphenate-limit-lines](#propdef-hyphenate-limit-lines) and [hyphenate-limit-last](#propdef-hyphenate-limit-last) properties



| Field               | Definition                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphenate-limit-lines"></a>hyphenate-limit-lines                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-integer-value②"></a><a id="ref-for-comb-one④⓪"></a>no-limit [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | no-limit                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword or integer                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                             |



This property indicates the maximum number of successive hyphenated lines in an element. The no-limit value means that there is no limit.

In some cases, user agents may not be able to honor the specified value. (See overflow-wrap.) It is not defined whether hyphenation introduced by such emergency breaking influences nearby hyphenation points.



| Field               | Definition                                                                                                                                                             |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hyphenate-limit-last"></a>hyphenate-limit-last                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④①"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) always <a id="ref-for-comb-one④②"></a>\| column <a id="ref-for-comb-one④③"></a>\| page <a id="ref-for-comb-one④④"></a>\| spread |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-block-container⑤"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container)                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                               |



This property indicates hyphenation behavior at the end of elements, column, pages, and spreads. A spread is a set of two pages that are visible to the reader at the same time. Values have the following meanings:

<a id="valdef-hyphenate-limit-lines-none"></a>none  
No restrictions imposed.

<a id="valdef-hyphenate-limit-lines-always"></a>always  
The last full line of the element, or the last line before any column, page, or spread break inside the element should not be hyphenated.

<a id="valdef-hyphenate-limit-lines-column"></a>column  
The last line before any column, page, or spread break inside the element should not be hyphenated.

<a id="valdef-hyphenate-limit-lines-page"></a>page  
The last line before page or spread break inside the element should not be hyphenated.

<a id="valdef-hyphenate-limit-lines-spread"></a>spread  
The last line before any spread break inside the element should not be hyphenated.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3e572423"></a>
>
> ```text
> p { hyphenate-limit-last: always }
> div.chapter {  hyphenate-limit-last: spread }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-72bf355c"></a>
>
> <a id="ref-for-propdef-hyphenate-limit-last①"></a>
>
> A paragraph may be formatted like this when [hyphenate-limit-last: none](#propdef-hyphenate-limit-last) is set:
>
> ```text
> This is just a
> simple example
> to show Antarc-
> tica.
> ```
>
> With 'hyphenate-limit-last: always' one would get:
>
> ```text
> This is just a
> simple example
> to        show
> Antarctica.
> ```
<a id="ref-for-propdef-overflow-wrap⑧"></a>

<a id="ref-for-propdef-word-wrap"></a>

### <a id="overflow-wrap-property"></a>6.4.  Overflow Wrapping: the [overflow-wrap](#propdef-overflow-wrap)/[word-wrap](#propdef-word-wrap) property<a id="overflow-wrap"></a>



| Field               | Definition                                                                                                             |
|---------------------|------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow-wrap"></a>overflow-wrap, <a id="propdef-word-wrap"></a>word-wrap                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④⑤"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) break-word <a id="ref-for-comb-one④⑥"></a>\| anywhere |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                               |



<a id="ref-for-propdef-white-space①⓪"></a>

<a id="ref-for-wrapping⑤"></a>

This property specifies whether the UA may break at otherwise disallowed points within a line to prevent overflow, when an otherwise-unbreakable string is too long to fit within the line box. It only has an effect when [white-space](#propdef-white-space) allows [wrapping](#wrapping). Possible values:

<a id="valdef-overflow-wrap-normal"></a>normal  
<a id="ref-for-propdef-word-break②③"></a>

Lines may break only at allowed break points. However, the restrictions introduced by [word-break: keep-all](#propdef-word-break) may be relaxed to match <a id="ref-for-propdef-word-break②④"></a>word-break: normal if there are no otherwise-acceptable break points in the line.

<a id="ref-for-propdef-word-break②⑤"></a>

Also, the restrictions introduced by [word-break: auto-phrase](#propdef-word-break) are relaxed if there are no otherwise-acceptable break points in the line:

- <a id="ref-for-valdef-word-break-normal⑦"></a>

  <a id="ref-for-soft-wrap-opportunity④⑧"></a>

  If suppressing [soft wrap opportunities](#soft-wrap-opportunity) within a particular phrase would cause that phrase to overflow even when placed on an otherwise empty line, the user agent must fall back to the same <a id="ref-for-soft-wrap-opportunity④⑨"></a>soft wrap opportunities as [normal](#valdef-word-break-normal) within that phrase.

- <a id="ref-for-hyphenation-opportunity⑨"></a>

  If that is not enough to prevent overflow, suppression of [hyphenation opportunities](#hyphenation-opportunity) must also be abandoned within each line that would overflow.

- As an intermediary measure, user agents may also detect multiple levels of phrases, choosing to shorter ones (possibly down to individual words) when longer ones would lead to overflow.

<a id="ref-for-soft-wrap-opportunity⑤⓪"></a>

<a id="ref-for-propdef-word-break②⑥"></a>

<a id="ref-for-min-content①"></a>

The [soft wrap opportunities](#soft-wrap-opportunity) obtained by relaxing the restrictions introduced by [word-break: keep-all](#propdef-word-break) and <a id="ref-for-propdef-word-break②⑦"></a>word-break: auto-phrase are <em>not</em> considered when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

<a id="valdef-overflow-wrap-anywhere"></a>anywhere  
<a id="ref-for-min-content②"></a>

<a id="ref-for-valdef-overflow-wrap-anywhere"></a>

<a id="ref-for-soft-wrap-opportunity⑤①"></a>

<a id="ref-for-character①①"></a>

An otherwise unbreakable sequence of [characters](#character) may be broken at an arbitrary point if there are no otherwise-acceptable break points in the line. Shaping characters are still shaped as if the word were not broken, and grapheme clusters must stay together as one unit. No hyphenation character is inserted at the break point. [Soft wrap opportunities](#soft-wrap-opportunity) introduced by [anywhere](#valdef-overflow-wrap-anywhere) <em>are considered</em> when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

<a id="ref-for-propdef-word-break②⑧"></a>

<a id="ref-for-soft-wrap-opportunity⑤②"></a>

<a id="ref-for-propdef-overflow-wrap⑨"></a>

In the case of [word-break: auto-phrase](#propdef-word-break), these additional [soft wrap opportunities](#soft-wrap-opportunity) are only introduced if relaxing the restrictions introduced by <a id="ref-for-propdef-word-break②⑨"></a>word-break: auto-phrase as described in [overflow-wrap: normal](#propdef-overflow-wrap) is insufficient to prevent overflow.

<a id="valdef-overflow-wrap-break-word"></a>break-word  
<a id="ref-for-min-content③"></a>

<a id="ref-for-valdef-overflow-wrap-break-word"></a>

<a id="ref-for-soft-wrap-opportunity⑤③"></a>

<a id="ref-for-valdef-overflow-wrap-anywhere①"></a>

As for [anywhere](#valdef-overflow-wrap-anywhere) except that [soft wrap opportunities](#soft-wrap-opportunity) introduced by [break-word](#valdef-overflow-wrap-break-word) are <em>not</em> considered when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

<a id="ref-for-propdef-overflow-wrap①⓪"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a2390e95"></a> Do we need to add a `none` value to [overflow-wrap](#propdef-overflow-wrap) to opt out of relaxing the keep-all' and auto-phrase'' restrictions as allowed by <a id="ref-for-propdef-overflow-wrap①①"></a>overflow-wrap: normal?

<a id="ref-for-propdef-word-wrap①"></a>

<a id="ref-for-legacy-name-alias"></a>

<a id="ref-for-propdef-overflow-wrap①②"></a>

For legacy reasons, UAs must treat [word-wrap](#propdef-word-wrap) as a [legacy name alias](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias) of the [overflow-wrap](#propdef-overflow-wrap) property.

## <a id="justification"></a>7.  Alignment and Justification

Alignment and justification controls how inline content is distributed within a line box.

<a id="ref-for-propdef-text-align"></a>

### <a id="text-align-property"></a>7.1.  Text Alignment: the [text-align](#propdef-text-align) shorthand



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align"></a>text-align                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-string-value②"></a><a id="ref-for-comb-one④⑦"></a>start [\|](https://www.w3.org/TR/css-values-4/#comb-one) end <a id="ref-for-comb-one④⑧"></a>\| left <a id="ref-for-comb-one④⑨"></a>\| right <a id="ref-for-comb-one⑤⓪"></a>\| center <a id="ref-for-comb-one⑤①"></a>\| [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) <a id="ref-for-comb-one⑤②"></a>\| justify <a id="ref-for-comb-one⑤③"></a>\| match-parent <a id="ref-for-comb-one⑤④"></a>\| justify-all |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | start                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                            |



<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-text-align-all"></a>

<a id="ref-for-propdef-text-align-last"></a>

<a id="ref-for-valdef-text-align-justify-all"></a>

<a id="ref-for-valdef-text-align-match-parent"></a>

<a id="ref-for-valdef-text-align-last-auto"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets the [text-align-all](#propdef-text-align-all) and [text-align-last](#propdef-text-align-last) properties and describes how the inline-level content of a block is aligned along the inline axis if the content does not completely fill the line box. Values other than [justify-all](#valdef-text-align-justify-all) or [match-parent](#valdef-text-align-match-parent) are assigned to <a id="ref-for-propdef-text-align-all①"></a>text-align-all and reset <a id="ref-for-propdef-text-align-last①"></a>text-align-last to [auto](#valdef-text-align-last-auto).

Values have the following meanings:

<a id="valdef-text-align-start"></a>start

<a id="ref-for-start"></a>

Inline-level content is aligned to the [start](https://www.w3.org/TR/css-writing-modes-4/#start) edge of the line box.

<a id="valdef-text-align-end"></a>end

<a id="ref-for-end"></a>

Inline-level content is aligned to the [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge of the line box.

<a id="valdef-text-align-left"></a>left

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-line-left"></a>

Inline-level content is aligned to the [line-left](https://www.w3.org/TR/css-writing-modes-4/#line-left) edge of the line box. (In vertical writing modes, this can be either the physical top or bottom, depending on [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode).) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

<a id="valdef-text-align-right"></a>right

<a id="ref-for-propdef-writing-mode①"></a>

<a id="ref-for-line-right"></a>

Inline-level content is aligned to the [line-right](https://www.w3.org/TR/css-writing-modes-4/#line-right) edge of the line box. (In vertical writing modes, this can be either the physical top or bottom, depending on [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode).) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

<a id="valdef-text-align-center"></a>center

Inline-level content is centered within the line box.

<a id="ref-for-string-value③"></a>

<a id="valdef-text-align-string"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value)

The string must be a single character; otherwise the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore). When applied to a table cell, specifies the <a id="valdef-text-align-alignment-character"></a>alignment character around which the cell’s contents will align. See [below](#character-alignment) for further details and how this value combines with keywords.

<a id="valdef-text-align-justify"></a>justify

<a id="ref-for-valdef-text-align-start"></a>

<a id="ref-for-propdef-text-align-last②"></a>

<a id="ref-for-propdef-text-justify"></a>

Text is justified according to the method specified by the [text-justify](#propdef-text-justify) property, in order to exactly fill the line box. Unless otherwise specified by [text-align-last](#propdef-text-align-last), the last line before a forced break or the end of the block is [start](#valdef-text-align-start)-aligned.

<a id="valdef-text-align-justify-all"></a>justify-all

<a id="ref-for-valdef-text-align-justify"></a>

<a id="ref-for-propdef-text-align-last③"></a>

<a id="ref-for-propdef-text-align-all②"></a>

Sets both [text-align-all](#propdef-text-align-all) and [text-align-last](#propdef-text-align-last) to [justify](#valdef-text-align-justify), forcing the last line to justify as well.

<a id="valdef-text-align-match-parent"></a>match-parent

<a id="ref-for-root-element"></a>

<a id="ref-for-valdef-text-align-right"></a>

<a id="ref-for-valdef-text-align-left"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-valdef-text-align-end"></a>

<a id="ref-for-valdef-text-align-start①"></a>

<a id="ref-for-inherited-value"></a>

<a id="ref-for-valdef-all-inherit"></a>

This value behaves the same as [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) (computes to its parent’s computed value) except that an [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) of [start](#valdef-text-align-start) or [end](#valdef-text-align-end) is interpreted against the parent’s [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) value and results in a computed value of either [left](#valdef-text-align-left) or [right](#valdef-text-align-right). Computes to <a id="ref-for-valdef-text-align-start②"></a>start when specified on the [root element](https://www.w3.org/TR/css-display-3/#root-element).

<a id="ref-for-propdef-text-align①"></a>

<a id="ref-for-propdef-text-align-all③"></a>

<a id="ref-for-propdef-text-align-last④"></a>

<a id="ref-for-valdef-text-align-match-parent①"></a>

When specified on the [text-align](#propdef-text-align) shorthand, sets both [text-align-all](#propdef-text-align-all) and [text-align-last](#propdef-text-align-last) to [match-parent](#valdef-text-align-match-parent).

<a id="ref-for-line-box"></a>

A block of text is a stack of [line boxes](https://www.w3.org/TR/css-inline-3/#line-box). This property specifies how the inline-level boxes within each line box align with respect to the start and end sides of the line box. Alignment is not with respect to the [viewport](https://www.w3.org/TR/CSS21/visuren.html#viewport) or containing block.

<a id="ref-for-valdef-text-align-justify①"></a>

<a id="ref-for-propdef-text-justify①"></a>

<a id="ref-for-white-space②⓪"></a>

<a id="ref-for-collapsible-white-space⑨"></a>

<a id="ref-for-justification-opportunity"></a>

<a id="ref-for-tab-stop②"></a>

In the case of [justify](#valdef-text-align-justify), the UA may stretch or shrink any inline boxes by [adjusting](#text-justify-property) their text. (See [text-justify](#propdef-text-justify).) If an element’s [white space](#white-space) is not [collapsible](#collapsible-white-space), then the UA is not required to adjust its text for the purpose of justification and may instead treat the text as having no [justification opportunities](#justification-opportunity). If the UA chooses to adjust the text, then it must ensure that [tab stops](#tab-stop) continue to line up as required by the [white space processing rules](#white-space-rules).

<a id="ref-for-start①"></a>

<a id="ref-for-end①"></a>

If (after justification, if any) the inline contents of a line box are too long to fit within it, then the contents are [start](https://www.w3.org/TR/css-writing-modes-4/#start)-aligned: any content that doesn’t fit overflows the line box’s [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge.

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

See [§ 9.3 Bidirectionality and Line Boxes](#bidi-linebox) for details on how to determine the [start](https://www.w3.org/TR/css-writing-modes-4/#start) and [end](https://www.w3.org/TR/css-writing-modes-4/#end) edges of a line box.

### <a id="character-alignment"></a>7.2.  Character-based Alignment in a Table Column

When multiple cells in a column have an alignment character specified, the alignment character of each such cell in the column is centered along a single column-parallel axis and the rest of the text in the column shifted accordingly. (Note that the strings do not have to be the same for each cell, although they usually are.)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-97356692"></a>Is this intended to say that it’s the centers of the alignment characters that should be aligned? It’s not clear that’s what it says, but that (or a different behavior) needs to be specified, to describe what happens when different occurrences of the alignment character are in different fonts. (Further, is that the intended behavior? Probably the most significant use case to consider is bold vs. non-bold text, which only varies slightly in width.) \[[feedback](https://lists.w3.org/Archives/Public/www-style/2016Jan/0233.html)\] \[minutes face-to-face 2016-02-02 10:00 AM\]

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6a3cc5d3"></a>
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
> +---------------------+
> | Long distance calls |
> +---------------------+
> |         $1.30       |
> |         $2.50       |
> |        $10.80       |
> |       $111.01       |
> |        $85.         |
> |        N/A          |
> |          $.05       |
> |          $.06       |
> +---------------------+
> ```
<a id="ref-for-string-value④"></a>

<a id="ref-for-valdef-text-align-right①"></a>

A keyword value may be specified in conjunction with the [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) value; if it is not given, it defaults to [right](#valdef-text-align-right). This value is used:

- when character-based alignment is applied to boxes that are not table cells.

- when the text wraps to multiple lines (at unforced break points).

- <a id="ref-for-valdef-text-align-end①"></a>

  <a id="ref-for-valdef-text-align-start③"></a>

  <a id="ref-for-valdef-text-align-center"></a>

  <a id="ref-for-valdef-text-align-right②"></a>

  <a id="ref-for-valdef-text-align-left①"></a>

  when a character-aligned cell spans more than one column. In this case the keyword alignment value is used to determine which column’s axis to align with: the leftmost column for [left](#valdef-text-align-left), the rightmost column for [right](#valdef-text-align-right) and [center](#valdef-text-align-center), the startmost column for [start](#valdef-text-align-start), the endmost column for [end](#valdef-text-align-end).

- <a id="ref-for-valdef-text-align-center①"></a>

  when the column is wide enough that the character alignment alone does not determine the positions of its character-aligned contents. In this case the keyword alignment of the first cell in the column with a specified alignment character is used to slide the position of the character-aligned contents to match the keyword alignment insofar as possible without changing the width of the column. For [center](#valdef-text-align-center), the UA may center the aligned contents using its extremes, center the alignment axis itself (insofar as possible), or optically center the aligned contents some other way (such as by taking a weighted average of the extent of the cells' contents to either side of the axis).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Right alignment is used by default for character-based alignment because numbering systems are almost all left-to-right even in right-to-left writing systems, and the primary use case of character-based alignment is for numerical alignment.

If the alignment character appears more than once in the text, the first instance is used for alignment. If the alignment character does not appear in a cell at all, the string is aligned as if the alignment character had been inserted at the end of its contents.

<a id="ref-for-propdef-text-align②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f823e64b"></a>This needs to specify what text is searched for the alignment character. Is it only in-flow text whose containing block is the cell? Or is text within any in-flow descendants in the block formatting context established by the cell considered? If so, is it considered only as long as its [text-align](#propdef-text-align) property is consistent with the cell’s? (Consistent in the alignment character, or fully consistent?)

<a id="ref-for-string-value⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a4eeca60"></a>This behavior of aligning as though the alignment character had been inserted at the end of the contents of the cell, combined with center-of-character alignment, will produce gaps on the end-side of lines that are alone on a line with [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) text-alignment, when none of the lines of the column has the alignment character, or, more importantly, when some of the lines do have the alignment character, but the column is not laid out at its max-content width. This is probably undesirable.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c1a95522"></a>When the alignment character is inserted at the end of the contents, which font is used? (In particular, if the alignment character might be within a descendant block, is it the font of the block or the font of the table cell? Or if the insertion is at a forced break within an inline, does it use the font of the inline or the font of the block or cell?)

Character-based alignment occurs before table cell width computation so that auto width computations can leave enough space for alignment. Whether column-spanning cells participate in the alignment prior to or after width computation is undefined. If width constraints on the cell contents prevent full alignment throughout the column, the resulting alignment is undefined.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f3db70cb"></a>This should have a formal definition of how character alignment affects the min-content and max-content intrinsic widths (of table columns and all content that can be inside table columns). Max-content intrinsic widths need to be split into three numbers (assuming that it’s the centers of the alignment character that are aligned): one for widths without alignment characters, one for widths on the inline-start side of the center of the alignment character, one for widths on the inline-end side of the center of the alignment character. This operates based on all segments of text between forced breaks for max-content widths. For min-content widths, segments of text between forced breaks that contain optional breaks within them should clearly contribute only to the without-alignment-character width. However, it’s less clear whether all min-content widths should work this way, or whether segments between forced breaks that do not have optional breaks (and perhaps only those that actually contain the alignment character) should contribute to start-side-of-alignment-character and end-side-of-alignment-character min-content widths instead; this choice is a tradeoff between the meaning of min-content sizing of a table meaning the narrowest reasonable size versus honoring alignment characters in more cases. Another option might be to use whether line-breaking of optional breaks is allowed as a control for which behavior to use.

<a id="ref-for-string-value⑥"></a>

<a id="ref-for-propdef-text-align③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b38ae96a"></a>Formally defining the intrinsic width contributions of column-spanning cells with [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) values of [text-align](#propdef-text-align) is a complicated (although straightforward) extension of the decisions made for intrinsic width contributions of non-column-spanning cells; this should also be formally defined. Contributions end up being made to the split intrinsic widths of the startmost or endmost column (whichever is used for alignment), and to the without-alignment-character intrinsic widths of the other spanned columns.

<a id="ref-for-propdef-text-align-all④"></a>

### <a id="text-align-all-property"></a>7.3.  Default Text Alignment: the [text-align-all](#propdef-text-align-all) property<a id="text-align-all"></a>



| Field               | Definition                                                                                                                                                                                                                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align-all"></a>text-align-all                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-string-value⑦"></a><a id="ref-for-comb-one⑤⑤"></a>start [\|](https://www.w3.org/TR/css-values-4/#comb-one) end <a id="ref-for-comb-one⑤⑥"></a>\| left <a id="ref-for-comb-one⑤⑦"></a>\| right <a id="ref-for-comb-one⑤⑧"></a>\| center <a id="ref-for-comb-one⑤⑨"></a>\| [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) <a id="ref-for-comb-one⑥⓪"></a>\| justify <a id="ref-for-comb-one⑥①"></a>\| match-parent |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | start                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-align-match-parent②"></a>keyword as specified, except for [match-parent](#valdef-text-align-match-parent) which computes as defined above                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                     |



<a id="ref-for-propdef-text-align④"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-valdef-text-align-last-auto①"></a>

<a id="ref-for-propdef-text-align-last⑤"></a>

This longhand of the [text-align](#propdef-text-align) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) specifies the inline alignment of all lines of inline content in the block container, except for last lines overridden by a non-[auto](#valdef-text-align-last-auto) value of [text-align-last](#propdef-text-align-last). See <a id="ref-for-propdef-text-align⑤"></a>text-align for a full description of values.

<a id="ref-for-propdef-text-align⑥"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should use the [text-align](#propdef-text-align) shorthand instead of this property.

<a id="ref-for-propdef-text-align-last⑥"></a>

### <a id="text-align-last-property"></a>7.4.  Last Line Alignment: the [text-align-last](#propdef-text-align-last) property<a id="text-align-last"></a>



| Field               | Definition                                                                                                                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align-last"></a>text-align-last                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑥②"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) start <a id="ref-for-comb-one⑥③"></a>\| end <a id="ref-for-comb-one⑥④"></a>\| left <a id="ref-for-comb-one⑥⑤"></a>\| right <a id="ref-for-comb-one⑥⑥"></a>\| center <a id="ref-for-comb-one⑥⑦"></a>\| justify <a id="ref-for-comb-one⑥⑧"></a>\| match-parent |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-align-match-parent③"></a>keyword as specified, except for [match-parent](#valdef-text-align-match-parent) which computes as defined above                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                        |



<a id="ref-for-forced-line-break⑧"></a>

This property describes how the last line of a block or a line right before a [forced line break](#forced-line-break) is aligned.

<a id="ref-for-propdef-text-align-all⑤"></a>

<a id="ref-for-valdef-text-align-justify②"></a>

<a id="ref-for-valdef-text-align-start④"></a>

<a id="ref-for-propdef-text-align⑦"></a>

If <a id="valdef-text-align-last-auto"></a>auto is specified, content on the affected line is aligned per [text-align-all](#propdef-text-align-all) unless <a id="ref-for-propdef-text-align-all⑥"></a>text-align-all is set to [justify](#valdef-text-align-justify), in which case it is [start](#valdef-text-align-start)-aligned. All other values are interpreted as described for [text-align](#propdef-text-align).

<a id="ref-for-propdef-text-justify②"></a>

### <a id="text-justify-property"></a>7.5.  Justification Method: the [text-justify](#propdef-text-justify) property<a id="text-justify"></a>



| Field               | Definition                                                                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-justify"></a>text-justify                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any⑦"></a><a id="ref-for-comb-one⑥⑨"></a>\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one⑦⓪"></a>\| inter-word <a id="ref-for-comb-one⑦①"></a>\| inter-character <a id="ref-for-comb-one⑦②"></a>\| ruby \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) no-compress |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-justify-distribute"></a>specified keyword (except for the [distribute](#valdef-text-justify-distribute) legacy value)                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                    |



<a id="ref-for-valdef-text-align-justify③"></a>

<a id="ref-for-propdef-text-align⑧"></a>

This property selects the justification method used when a line’s alignment is set to [justify](#valdef-text-align-justify) (see [text-align](#propdef-text-align)). The property applies to text, but is inherited from block containers to the root inline box containing their inline-level contents. It takes the following values:

<a id="valdef-text-justify-auto"></a>auto  
<a id="ref-for-content-language②⓪"></a>

<a id="ref-for-content-writing-system③"></a>

The UA determines the justification algorithm to follow, based on a balance between performance and adequate presentation quality. Since justification rules vary by [writing system](#content-writing-system) and [language](#content-language), UAs should, where possible, use a justification algorithm appropriate to the text.

<a id="ref-for-word-separator⑥"></a>

<a id="ref-for-typographic-letter-unit⑦"></a>

<a id="ref-for-content-language②①"></a>

<a id="ref-for-valdef-text-justify-inter-word"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0bd57ccb"></a> For example, the UA could use by default a justification method that is a simple universal compromise for all writing systems—​such as primarily expanding [word separators](#word-separator) and between CJK [typographic letter units](#typographic-letter-unit) along with secondarily expanding between Southeast Asian <a id="ref-for-typographic-letter-unit⑧"></a>typographic letter units. Then, in cases where the [content language](#content-language) of the paragraph is known, it could choose a more language-tailored justification behavior e.g. following the [Requirements for Japanese Text Layout](https://www.w3.org/TR/jlreq/) for Japanese [\[JLREQ\]](#biblio-jlreq), using cursive elongation for Arabic, using [inter-word](#valdef-text-justify-inter-word) for German, etc.

<a id="fig-text-justify-cursive"></a>

![Two lines of calligraphic Arabic end together due to a mix of compressed and swash forms.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/text-justify-cursive.png "Swash forms elongate the first line
				            while a compressed contextual ligature shortens the second,
				            allowing both to end precisely together.")

An example of cursively-justified Arabic text, rendered by [Tasmeem](https://www.decotype.com/). Like English, Arabic can be justified by adjusting the spacing between words, but in most styles it can also be justified by calligraphically elongating or compressing the letterforms themselves. In this example, the upper text is extended to fill the line by the use of elongated (kashida) forms and swash forms, while the bottom line is compressed slightly by using a stacked combination for the characters between ت and م. By employing traditional calligraphic techniques, a typesetter can justify the line while preserving flow and color, providing a very high quality justification effect. However, this is by its nature a very script-specific effect.

<a id="fig-text-justify-compromise"></a> ![Extra space is partly to spaces and partly among CJK and Thai letters.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/text-justify-compromise.png)

<a id="ref-for-propdef-text-justify③"></a>

Mixed-script text with [text-justify: auto](#propdef-text-justify): this interpretation uses a universal-compromise justification method, expanding at spaces as well as between CJK and Southeast Asian letters. This effectively uses inter-word + inter-ideograph spacing for lines that have word-separators and/or CJK characters and falls back to inter-cluster behavior for lines that don’t or for which the space stretches too far.

<a id="valdef-text-justify-none"></a>none  
<a id="ref-for-justification-opportunity①"></a>

Justification is disabled: there are no [justification opportunities](#justification-opportunity) within the text.

<a id="fig-text-justify-none"></a> ![No extra space is inserted.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/text-justify-none.png)

<a id="ref-for-propdef-text-justify④"></a>

Mixed-script text with [text-justify: none](#propdef-text-justify)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is intended for use in user stylesheets to improve readability or for accessibility purposes.

<a id="valdef-text-justify-inter-word"></a>inter-word  
<a id="ref-for-propdef-word-spacing①"></a>

<a id="ref-for-word-separator⑦"></a>

Justification adjusts spacing at [word separators](#word-separator) only (effectively varying the used [word-spacing](#propdef-word-spacing) on the line). This behavior is typical for languages that separate words using spaces, like English or Korean.

<a id="fig-text-justify-interword"></a> ![Extra space is equally distributed mainly to spaces.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/text-justify-interword.png)

<a id="ref-for-propdef-text-justify⑤"></a>

Mixed-script text with [text-justify: inter-word](#propdef-text-justify)

<a id="valdef-text-justify-inter-character"></a>inter-character  
<a id="ref-for-propdef-letter-spacing①"></a>

<a id="ref-for-typographic-character-unit②②"></a>

Justification adjusts spacing between each pair of adjacent [typographic character units](#typographic-character-unit) (effectively varying the used [letter-spacing](#propdef-letter-spacing) on the line). This value is sometimes used in East Asian systems such as Japanese.

<a id="fig-text-justify-distribute"></a> ![Extra space is equally distributed at points between spaces and letters of all writing systems.](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/text-justify-distribute.png)

<a id="ref-for-propdef-text-justify⑥"></a>

Mixed-script text with [text-justify: inter-character](#propdef-text-justify)

<a id="ref-for-valdef-text-justify-inter-character"></a>

<a id="ref-for-css-legacy-value-alias"></a>

For legacy reasons, UAs must also support the alternate keyword <a id="valdef-text-justify-distribute"></a>distribute which must compute to [inter-character](#valdef-text-justify-inter-character), thus having the exact same meaning and behavior. UAs may treat this as a [legacy value alias](https://www.w3.org/TR/css-cascade-5/#css-legacy-value-alias).

<a id="valdef-text-justify-ruby"></a>ruby  
<a id="ref-for-valdef-text-justify-auto"></a>

Justification adjusts spacing as for [auto](#valdef-text-justify-auto) <em>except</em>:

- <a id="ref-for-justification-opportunity②"></a>

  <a id="ref-for-word-separator⑧"></a>

  [Justification opportunities](#justification-opportunity) are disabled at [word separators](#word-separator).

- <a id="ref-for-justification-opportunity③"></a>

  <a id="ref-for-bopomofo-characters"></a>

  [Justification opportunities](#justification-opportunity) are disabled between [Bopomofo characters](https://www.w3.org/TR/css-ruby-1/#bopomofo-characters)

<a id="ref-for-ruby-annotation-box"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is intended for use in [ruby annotations](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-box), providing a reasonable default alignment. See [\[CSS-RUBY-1\]](#biblio-css-ruby-1).

<a id="valdef-text-justify-no-compress"></a>no-compress  
<a id="ref-for-propdef-text-autospace"></a>

<a id="ref-for-propdef-text-spacing-trim"></a>

Justification must not compress spacing controlled by [text-spacing-trim](#propdef-text-spacing-trim) or [text-autospace](#propdef-text-autospace). (If this value is not specified, the justification process may reduce such spacing except when the spacing is at the start or end of the line.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An example of compression rules is given for Japanese in 3.8 Line Adjustment in [\[JLREQ\]](#biblio-jlreq).

<a id="ref-for-propdef-text-spacing"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a6b9619a"></a> This keyword used to be part of [text-spacing](#propdef-text-spacing); it might need renaming to be more specific now that it’s here, as it implies that e.g. U+0020 cannot be compressed. [\[Issue \#7079\]](https://github.com/w3c/csswg-drafts/issues/7079)

<a id="ref-for-content-language②②"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Since optimal justification is [language](#content-language)-sensitive, authors should correctly language-tag their content for the best results.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The guidelines in this level of CSS do not describe a complete justification algorithm. They are merely a minimum set of requirements that a complete algorithm should meet. Limiting the set of requirements gives UAs some latitude in choosing a justification algorithm that meets their needs and desired balance of quality, speed, and complexity.

#### <a id="expanding-text"></a>7.5.1.  Expanding and Compressing Text

When justifying text, the user agent takes the remaining space between the ends of a line’s contents and the edges of its line box, and distributes that space throughout its content so that the contents exactly fill the line box. The user agent may alternatively distribute negative space, putting more content on the line than would otherwise fit under normal spacing conditions.

<a id="ref-for-typographic-character-unit②③"></a>

<a id="ref-for-word-separator⑨"></a>

<a id="ref-for-justification-opportunity④"></a>

<a id="ref-for-propdef-text-justify⑦"></a>

A <a id="justification-opportunity"></a>justification opportunity<a id="expansion-opportunity"></a> is a point where the justification algorithm may alter spacing within the text. A justification opportunity can be provided by a single [typographic character unit](#typographic-character-unit) (such as a [word separator](#word-separator)), or by the juxtaposition of two <a id="ref-for-typographic-character-unit②④"></a>typographic character units. As with controls for [soft wrap opportunities](#line-break-details), whether a <a id="ref-for-typographic-character-unit②⑤"></a>typographic character unit provides a [justification opportunity](#justification-opportunity) is controlled by the [text-justify](#propdef-text-justify) value of its parent; similarly, whether a <a id="ref-for-justification-opportunity⑤"></a>justification opportunity exists between two consecutive <a id="ref-for-typographic-character-unit②⑥"></a>typographic character units is determined by the <a id="ref-for-propdef-text-justify⑧"></a>text-justify value of their nearest common ancestor.

<a id="ref-for-propdef-letter-spacing②"></a>

<a id="ref-for-propdef-word-spacing②"></a>

<a id="ref-for-word-separator①⓪"></a>

<a id="ref-for-justification-opportunity⑥"></a>

<a id="ref-for-typographic-character-unit②⑦"></a>

Space distributed by justification is <em>in addition to</em> the spacing defined by the [letter-spacing](#propdef-letter-spacing) or [word-spacing](#propdef-word-spacing) properties. When such additional space is distributed to a [word separator](#word-separator) [justification opportunity](#justification-opportunity), it is applied under the same rules as for <a id="ref-for-propdef-word-spacing③"></a>word-spacing. Similarly, when space is distributed to a <a id="ref-for-justification-opportunity⑦"></a>justification opportunity between two [typographic character units](#typographic-character-unit), should be applied under the same rules as for <a id="ref-for-propdef-letter-spacing③"></a>letter-spacing.

<a id="ref-for-justification-opportunity⑧"></a>

<a id="ref-for-typographic-character-unit②⑧"></a>

<a id="ref-for-valdef-text-justify-inter-character①"></a>

A justification algorithm may divide [justification opportunities](#justification-opportunity) into different priority levels. All <a id="ref-for-justification-opportunity⑨"></a>justification opportunities within a given level are expanded or compressed at the same priority, regardless of which [typographic character units](#typographic-character-unit) created that opportunity. For example, if <a id="ref-for-justification-opportunity①⓪"></a>justification opportunities between two Han characters and between two Latin letters are defined to be at the same level (as they are in the [inter-character](#valdef-text-justify-inter-character) justification style), they are not treated differently because they originate from different <a id="ref-for-typographic-character-unit②⑨"></a>typographic character units. It is not defined in this level whether or how other factors (such as font size, letter-spacing, glyph shape, position within the line, etc.) may influence the distribution of space to <a id="ref-for-justification-opportunity①①"></a>justification opportunities within the line.

The UA may enable or break optional ligatures or use other font features such as alternate glyphs or glyph compression to help justify the text under any method. This behavior is not controlled by this level of CSS. However, UAs <em>must not</em> break required ligatures or otherwise disable features required to correctly shape complex scripts.

<a id="ref-for-justification-opportunity①②"></a>

<a id="ref-for-valdef-text-align-justify④"></a>

If a [justification opportunity](#justification-opportunity) exists within a line, and [text alignment](#text-align-property) specifies full justification ([justify](#valdef-text-align-justify)) for that line, it must be justified.

#### <a id="justify-symbols"></a>7.5.2.  Handling Symbols and Punctuation

<a id="ref-for-justification-opportunity①③"></a>

<a id="ref-for-typographic-character-unit③⓪"></a>

<a id="ref-for-typographic-letter-unit⑨"></a>

When determining [justification opportunities](#justification-opportunity), a [typographic character unit](#typographic-character-unit) from the Unicode Symbols (S\*) and Punctuation (P\*) classes is generally treated the same as a [typographic letter unit](#typographic-letter-unit) of the same script (or, if the character’s script property is Common, then as a <a id="ref-for-typographic-letter-unit①⓪"></a>typographic letter unit of the dominant script).

<a id="ref-for-justification-opportunity①④"></a>

However, by typographic tradition there may be additional rules controlling the justification of symbols and punctuation. Therefore, the UA may reassign specific characters or introduce additional levels of prioritization to handle [justification opportunities](#justification-opportunity) involving symbols and punctuation.

<a id="ref-for-justification-opportunity①⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3ce55d62"></a> For example, there are traditionally no [justification opportunities](#justification-opportunity) between consecutive U+2014 — EM DASH, U+2015 ― HORIZONTAL BAR, U+2026 … HORIZONTAL ELLIPSIS, or U+2025 ‥ TWO DOT LEADER characters [\[JLREQ\]](#biblio-jlreq); thus a UA might assign these characters to a “never” prioritization level. As another example, certain full-width punctuation characters (such as U+301A 〚 LEFT WHITE SQUARE BRACKET) are considered to contain a <a id="ref-for-justification-opportunity①⑥"></a>justification opportunity in Japanese. The UA might therefore assign these characters to a higher prioritization level than the opportunities between ideographic characters.

#### <a id="justify-limits"></a>7.5.3.  Unexpandable Text

<a id="ref-for-propdef-text-align-last⑦"></a>

<a id="ref-for-valdef-text-align-justify⑤"></a>

<a id="ref-for-valdef-text-align-center②"></a>

If the inline contents of a line cannot be stretched to the full width of the line box, then they must be aligned as specified by the [text-align-last](#propdef-text-align-last) property. (If <a id="ref-for-propdef-text-align-last⑧"></a>text-align-last is [justify](#valdef-text-align-justify), then they must be aligned as for [center](#valdef-text-align-center).)

#### <a id="justify-cursive"></a>7.5.4.  Cursive Scripts

<a id="ref-for-typographic-letter-unit①①"></a>

<a id="ref-for-cursive-script"></a>

<a id="ref-for-justification-opportunity①⑦"></a>

Justification <em>must not</em> introduce gaps between the joined [typographic letter units](#typographic-letter-unit) of [cursive scripts](#cursive-script) such as Arabic. If it is able, the UA <em>may</em> translate space distributed to [justification opportunities](#justification-opportunity) within a run of such <a id="ref-for-typographic-letter-unit①②"></a>typographic letter units into some form of cursive elongation for that run. It otherwise <em>must</em> assume that no <a id="ref-for-justification-opportunity①⑧"></a>justification opportunity exists between any pair of <a id="ref-for-typographic-letter-unit①③"></a>typographic letter units in <a id="ref-for-cursive-script①"></a>cursive script (regardless of whether they join).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5407996d"></a> The following are examples of unacceptable justification:
>
> ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-spaced.png)
>
> Adding gaps between every pair of Arabic letters
>
> ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-unjoined.png)
>
> Adding gaps between every pair of unjoined Arabic letters

Some font designs allow for the use of the tatweel character for justification. A UA that performs tatweel-based justification must properly handle the rules for its use. Note that correct insertion of tatweel characters depends on context, including the letter-combinations involved, location within the word, and location of the word within the line.

<a id="ref-for-valdef-text-justify-auto①"></a>

#### <a id="justify-algos"></a>7.5.5.  Minimum Requirements for [auto](#valdef-text-justify-auto) Justification

<a id="ref-for-valdef-text-justify-auto②"></a>

<a id="ref-for-justification-opportunity①⑨"></a>

For [auto](#valdef-text-justify-auto) justification, this specification does not define what all of the [justification opportunities](#justification-opportunity) are, how they are prioritized, or when and how multiple levels of <a id="ref-for-justification-opportunity②⓪"></a>justification opportunities interact. However, it does require that:

- <a id="ref-for-justification-opportunity②①"></a>

  <a id="ref-for-content-language②③"></a>

  Unless contraindicated by the typographic traditions of the [content language](#content-language) or adjacent symbols/punctuation, each of the following provides a [justification opportunity](#justification-opportunity):

  - <a id="ref-for-word-separator①①"></a>

    [Word separators](#word-separator)

  - <a id="ref-for-block-scripts"></a>

    <a id="ref-for-typographic-character-unit③①"></a>

    The boundary between a [typographic character unit](#typographic-character-unit) of any [block scripts](#block-scripts) and any other <a id="ref-for-typographic-character-unit③②"></a>typographic character unit

  - <a id="ref-for-clustered-scripts"></a>

    <a id="ref-for-typographic-character-unit③③"></a>

    The boundary between a [typographic character unit](#typographic-character-unit) of any [clustered scripts](#clustered-scripts) and any other <a id="ref-for-typographic-character-unit③④"></a>typographic character unit

- <a id="ref-for-clustered-scripts①"></a>

  <a id="ref-for-block-scripts①"></a>

  <a id="ref-for-letter③"></a>

  All [letters](#letter) belonging to all [block scripts](#block-scripts) are treated the same, and all <a id="ref-for-letter④"></a>letters belonging to all [clustered scripts](#clustered-scripts) are treated the same. For example, no distinction is made between the justification opportunity between a Han letter followed by another Han letter, vs. the justification opportunity between a Han letter followed by a Hangul letter.

Further information on text justification can be found in (or submitted to) [“Approaches to Full Justification”](https://www.w3.org/International/articles/typography/justification), which indexes by writing system and language, and is maintained by the [W3C Internationalization Working Group](https://www.w3.org/International/). [\[JUSTIFY\]](#biblio-justify)

<a id="ref-for-propdef-text-group-align"></a>

### <a id="text-group-align-property"></a>7.6.  Aligning a block of text within its container: the [text-group-align](#propdef-text-group-align) property



| Field               | Definition                                                                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-group-align"></a>text-group-align                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑦③"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) start <a id="ref-for-comb-one⑦④"></a>\| end <a id="ref-for-comb-one⑦⑤"></a>\| left <a id="ref-for-comb-one⑦⑥"></a>\| right <a id="ref-for-comb-one⑦⑦"></a>\| center |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-block-container⑥"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container)                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                       |



This property aligns the contents of the line boxes as a group while maintaining their text alignment.

<a id="ref-for-in-flow"></a>

<a id="ref-for-block-formatting-context"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="group-alignment"></a>Group alignment is performed by finding the line box with the shortest remaining space and adding that amount of space as padding to one or both sides of the line box, reducing the amount of space available for its contents; [text alignment](#text-align-property) is then applied to its contents within the remaining space. All descendant [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) line boxes within the same [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context) are considered both when searching for the shortest remaining space and when adding the padding; the contents of descendants that establish [independent formatting contexts](https://www.w3.org/TR/css-display-3/#independent-formatting-context) are skipped.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5a886932"></a> A variant of this property is inherited, and applies on each block container individually, only affecting the line boxes that are direct children of that block. This is less useful, but probably easier to implement.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6a6c35c0"></a> Somehow also moving the floats that originate in the same block container by the same amount would make things line up more nicely, which would be especially valuable in CJK layout. Exactly how that works, and how it interacts with intruding floats from ancestor elements is left as an exercise for the reader.

Values have the following meanings:

<a id="valdef-text-group-align-none"></a>none  
<a id="ref-for-group-alignment"></a>

Text alignment happens normally: [group alignment](#group-alignment) is not performed.

<a id="valdef-text-group-align-start"></a>start  
<a id="ref-for-inline-end"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-group-alignment①"></a>

Inline-level content is [group-aligned](#group-alignment) to the [inline start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) side, by padding the [inline end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) side of each line box.

<a id="valdef-text-group-align-end"></a>end  
<a id="ref-for-inline-start①"></a>

<a id="ref-for-inline-end①"></a>

<a id="ref-for-group-alignment②"></a>

Inline-level content is [group-aligned](#group-alignment) to the [inline end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) side, by padding the [inline start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) side of each line box.

<a id="valdef-text-group-align-left"></a>left  
<a id="ref-for-line-right①"></a>

<a id="ref-for-line-left①"></a>

<a id="ref-for-group-alignment③"></a>

Inline-level content is [group-aligned](#group-alignment) to the [line-left](https://www.w3.org/TR/css-writing-modes-4/#line-left) side, by padding the [line-right](https://www.w3.org/TR/css-writing-modes-4/#line-right) side of each line box.

<a id="valdef-text-group-align-right"></a>right  
<a id="ref-for-line-left②"></a>

<a id="ref-for-line-right②"></a>

<a id="ref-for-group-alignment④"></a>

Inline-level content is [group-aligned](#group-alignment) to the [line-right](https://www.w3.org/TR/css-writing-modes-4/#line-right) side, by padding the [line-left](https://www.w3.org/TR/css-writing-modes-4/#line-left) side of each line box.

<a id="valdef-text-group-align-center"></a>center  
<a id="ref-for-group-alignment⑤"></a>

Inline-level content is [group-aligned](#group-alignment) to the center, by padding both sides of each line box, half the spacing to each side.

## <a id="spacing"></a>8.  Spacing

<a id="ref-for-propdef-word-spacing④"></a>

<a id="ref-for-propdef-letter-spacing④"></a>

<a id="ref-for-propdef-line-padding"></a>

<a id="ref-for-word-separator①②"></a>

<a id="ref-for-typographic-character-unit③⑤"></a>

<a id="ref-for-propdef-text-spacing-trim①"></a>

<a id="ref-for-propdef-text-autospace①"></a>

CSS offers control over regular text spacing via the [word-spacing](#propdef-word-spacing), [letter-spacing](#propdef-letter-spacing), and [line-padding](#propdef-line-padding) properties, which specify additional space around [word separators](#word-separator) between [typographic character units](#typographic-character-unit), or at the start/end of the line, respectively. It also provides contextual control over spacing via the [text-spacing-trim](#propdef-text-spacing-trim) property, which allows for contextual fullwidth vs halfwidth setting of CJK punctuation; and the [text-autospace](#propdef-text-autospace) property, which allows automatic insertion of extra space at script changes or around punctuation.

<a id="ref-for-propdef-word-spacing⑤"></a>

### <a id="word-spacing-property"></a>8.1.  Word Spacing: the [word-spacing](#propdef-word-spacing) property<a id="word-spacing"></a>



| Field               | Definition                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-word-spacing"></a>word-spacing                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②"></a><a id="ref-for-comb-one⑦⑧"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-propdef-font-size"></a>relative to computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), i.e. 1em                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | an absolute length and/or a percentage                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                 |



This property specifies additional spacing between “words”. Values are interpreted as defined below:

<a id="valdef-word-spacing-normal"></a>normal  
No additional spacing is applied. Computes to zero.

<a id="valdef-word-spacing-length-percentage"></a>\<length-percentage\>  
Specifies extra spacing <em>in addition to</em> the intrinsic inter-word spacing defined by the font.

<a id="ref-for-propdef-font-size①"></a>

<a id="ref-for-em"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Percentages inherit intact, and are resolved against the computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) of the current element (and thus represent a size relative to the size of the text to which they apply), unlike [em](https://www.w3.org/TR/css-values-4/#em) units which are resolved against the computed <a id="ref-for-propdef-font-size②"></a>font-size of the element from which they inherit, as an absolute length.

<a id="ref-for-word-separator①③"></a>

Additional spacing is applied to each [word separator](#word-separator) left in the text after the [white space processing rules](#white-space-rules) have been applied, and should be applied half on each side of the character unless otherwise dictated by typographic tradition. Values may be negative, but there may be implementation-dependent limits.

<a id="ref-for-typographic-character-unit③⑥"></a>

<a id="word-separator"></a>Word-separator characters are [typographic character units](#typographic-character-unit) whose primary purpose and general usage is to separate words. In Unicode this includes (but is not exhaustively defined as) the space (U+0020), the no-break space (U+00A0), the Ethiopic word space (U+1361), the Aegean word separators (U+10100,U+10101), the Ugaritic word divider (U+1039F), and the Phoenician word separator (U+1091F). [\[UNICODE\]](#biblio-unicode)

<a id="ref-for-word-separator①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Neither punctuation in general, nor fixed-width spaces (such as U+3000 and U+2000 through U+200A), are considered [word-separator characters](#word-separator), because even though they frequently happen to separate words, their primary purpose is not to separate words.

<a id="ref-for-word-separator①⑤"></a>

If there are no [word-separator characters](#word-separator), or if a word-separating character has a zero advance width (such as U+200B ZERO WIDTH SPACE) then the user agent must not create an additional spacing between words.

<a id="ref-for-propdef-letter-spacing⑤"></a>

### <a id="letter-spacing-property"></a>8.2.  Tracking: the [letter-spacing](#propdef-letter-spacing) property<a id="letter-spacing"></a>



| Field               | Definition                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-letter-spacing"></a>letter-spacing                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage③"></a><a id="ref-for-comb-one⑦⑨"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box④"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) and text                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-propdef-font-size③"></a>relative to computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), i.e. 1em                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | an absolute length and/or a percentage                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                 |



<a id="ref-for-typographic-character-unit③⑦"></a>

<a id="ref-for-propdef-word-spacing⑥"></a>

This property specifies additional spacing (commonly called <a id="tracking"></a>tracking) between adjacent [typographic character units](#typographic-character-unit). Letter-spacing is applied after [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) and is in addition to [kerning](https://www.w3.org/TR/css-fonts-3/#font-kerning-prop) and [word-spacing](#propdef-word-spacing). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4) [\[CSS-FONTS-3\]](#biblio-css-fonts-3) Depending on the justification rules in effect, user agents may further increase or decrease the space between <a id="ref-for-typographic-character-unit③⑧"></a>typographic character units in order to [justify text](#text-justify-property).

Values have the following meanings:

<a id="valdef-letter-spacing-normal"></a>normal  
No additional spacing is applied. Computes to zero.

<a id="valdef-letter-spacing-length-percentage"></a>\<length-percentage\>  
<a id="ref-for-typographic-character-unit③⑨"></a>

Specifies <em>additional</em> spacing between [typographic character units](#typographic-character-unit). Values may be negative, but there may be implementation-dependent limits.

<a id="ref-for-propdef-font-size④"></a>

<a id="ref-for-em①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Percentages inherit intact, and are resolved against the computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) of the current element (and thus represent a size relative to the size of the text to which they apply), unlike [em](https://www.w3.org/TR/css-values-4/#em) units which are resolved against the computed <a id="ref-for-propdef-font-size⑤"></a>font-size of the element from which they inherit, as an absolute length.

<a id="ref-for-propdef-letter-spacing⑥"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-dom-window-getcomputedstyle①"></a>

<a id="ref-for-valdef-letter-spacing-normal"></a>

For [legacy reasons](https://github.com/w3c/csswg-drafts/issues/1484), a computed [letter-spacing](#propdef-letter-spacing) of zero yields a [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) (<code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> return value) of [normal](#valdef-letter-spacing-normal).

<a id="ref-for-propdef-letter-spacing⑦"></a>

<a id="ref-for-atomic-inline③"></a>

<a id="ref-for-typographic-character-unit④⓪"></a>

For the purpose of [letter-spacing](#propdef-letter-spacing), each consecutive run of [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) (such as images and inline blocks) is treated as a single [typographic character unit](#typographic-character-unit).

Letter-spacing must not be applied at the beginning of a line. Whether letter-spacing is applied at the end of a line is undefined in this level.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7880704e"></a> When letter-spacing is not applied at the beginning or end of a line, text always fits flush with the edge of the block.
>
> ```text
> p { letter-spacing: 1em; }
> ```
>
> ```text
> <p>abc</p>
> ```
>
> a　b　c
>
> a　b　c
>
> UAs therefore <em>really should not</em> [\[RFC6919\]](#biblio-rfc6919) append letter spacing to the right or trailing edge of a line:
>
> a　b　c　

<a id="ref-for-typographic-character-unit④①"></a>

Letter spacing between two [typographic character units](#typographic-character-unit) effectively “belongs” to the innermost element that contains the two <a id="ref-for-typographic-character-unit④②"></a>typographic character units: the total letter spacing between two adjacent <a id="ref-for-typographic-character-unit④③"></a>typographic character units (after bidi reordering) is specified by and rendered within the innermost element that <em>contains</em> the boundary between the two <a id="ref-for-typographic-character-unit④④"></a>typographic character units. However, the UA may instead attach letter-spacing at element boundaries to one or the other <a id="ref-for-typographic-character-unit④⑤"></a>typographic character unit using the letter-spacing value pertaining to its containing element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This secondary behavior is permitted in this level due to Web-compat concerns.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-58b27a9a"></a> An inline box is expected to only include letter spacing between characters completely contained within that element, thus excluding letter spacing on the right or trailing edge of the element:
>
> ```text
> p { letter-spacing: 1em; }
> ```
>
> ```text
> <p>a<span>bb</span>c</p>
> ```
>
> a　b　b　c
>
> a　b　b　c
>
> <a id="ref-for-propdef-letter-spacing⑧"></a>
>
> Consequently a given value of [letter-spacing](#propdef-letter-spacing) is expected to only affect the spacing between characters completely contained within the element for which it is specified:
>
> ```text
> p    { letter-spacing: 1em; }
> span { letter-spacing: 2em; }
> ```
>
> ```text
> <p>a<span>bb</span>c</p>
> ```
>
> a　b　　b　c
>
> <a id="ref-for-propdef-letter-spacing⑨"></a>
>
> This further implies that applying [letter-spacing](#propdef-letter-spacing) to an element containing only a single character has no effect on the rendered result:
>
> ```text
> p    { letter-spacing: 1em; }
> span { letter-spacing: 2em; }
> ```
>
> ```text
> <p>a<span>b</span>c</p>
> ```
>
> a　b　c
>
> Since letter spacing is inserted <strong>after</strong> RTL reordering, the letter spacing applied to the inner span below likewise has no effect, since after reordering the "c" doesn’t end up next to "א":
>
> ```text
> p    { letter-spacing: 1em; }
> span { letter-spacing: 2em; }
> ```
>
> ```text
> <!-- abc followed by Hebrew letters alef (א), bet (ב) and gimel (ג) -->
> <!-- Reordering will display these in reverse order. -->
> <p>ab<span>cא</span>בג</p>
> ```
>
> a　b　c　א　ב　ג

Letter spacing ignores invisible zero-width formatting characters (such as those from the Unicode Cf category). Spacing must be added as if those characters did not exist in the document.

<a id="ref-for-propdef-letter-spacing①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-35549407"></a> For example, [letter-spacing](#propdef-letter-spacing) applied to `A&#x200B;B` is identical to `AB`, regardless of where any element boundaries might fall.

<a id="ref-for-propdef-letter-spacing①①"></a>

<a id="ref-for-propdef-font-feature-settings"></a>

When the effective spacing between two characters is not zero (due to either [justification](#text-justify-property) or a non-zero value of [letter-spacing](#propdef-letter-spacing)), user agents should not apply optional ligatures, i.e. those that are not defined as required for fundamentally correct glyph shaping. However, ligatures and other font features specified via the low-level [font-feature-settings](https://www.w3.org/TR/css-fonts-4/#propdef-font-feature-settings) property take precedence over this rule. See [CSS Fonts Module Level 3 § feature-precedence](https://www.w3.org/TR/css-fonts-3/#feature-precedence).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d2a0b994"></a> For example, if the word “filial” is letter-spaced, an “fi” ligature should not be used as it will prevent even spacing of the text.
>
> <strong>filial</strong> vs <strong>ﬁlial</strong>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In OpenType, required ligatures are expected to be associated to the `rlig` feature. All other ligatures are therefore considered optional. In some cases, however, UA or platform heuristics apply additional ligatures in order to handle broken fonts; this specification does not define or override such exceptional handling.

#### <a id="cursive-tracking"></a>8.2.1.  Cursive Scripts

<a id="ref-for-cursive-script②"></a>

<a id="ref-for-typographic-letter-unit①④"></a>

If it is able, the UA <em>may</em> apply letter spacing to [cursive scripts](#cursive-script) by translating the total extra space to be distributed to a run of such letters into some form of cursive elongation (or compression, for negative tracking values) for that run that results in an equivalent total expansion (or compression) of the run. Otherwise, if the UA cannot expand text from a <a id="ref-for-cursive-script③"></a>cursive script without breaking its cursive connections, it <em>must not</em> apply spacing between any pair of that script’s [typographic letter units](#typographic-letter-unit) at all (effectively treating each word as a single <a id="ref-for-typographic-letter-unit①⑤"></a>typographic letter unit for the purpose of letter-spacing). Both cases will result in an effective spacing of zero between such letters; however the former will preserve the sense of stretching out the text.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9902d8b5"></a> Below are some appropriate and inappropriate examples of spacing out Arabic text.
>
> 
>
> | Column 1                                                                                    | Column 2 | Column 3            |
> |---------------------------------------------------------------------------------------------|----------|---------------------|
> | ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-original.png)   | —        | Original text       |
> | ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-spaced.png)     | BAD      | <strong> Even distribution of space between each letter. <em>Notice this breaks cursive joins!</em> &#xA;      </strong> |
> | ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-kashida.png)    | OK       | <strong> Distributing ∑<var>letter-spacing</var> by typographically-appropriate cursive elongation. <em>The resulting text is as long as the previous evenly-spaced example.</em> &#xA;      </strong> |
> | ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-suppressed.png) | OK       | <strong><span><a id="ref-for-spaces③⑤"></a></span><span><a id="ref-for-propdef-letter-spacing①③"></a></span><span><a id="ref-for-propdef-letter-spacing①②"></a></span> Suppressing <a href="#propdef-letter-spacing">letter-spacing</a> between Arabic letters. <em>Notice <a href="#propdef-letter-spacing">letter-spacing</a> is nonetheless applied&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;to non-Arabic characters (like <a href="#spaces">spaces</a>).</em> &#xA;      </strong> |
> | ![](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/arabic-stretch-unjoined.png)   | BAD      | <strong><span><a id="ref-for-propdef-letter-spacing①④"></a></span> Applying <a href="#propdef-letter-spacing">letter-spacing</a> only between non-joined letters. <em>This distorts typographic color and obfuscates word boundaries.</em> &#xA;    </strong> |
>
> 

<a id="ref-for-propdef-letter-spacing①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Proper cursive elongation or compression of a text can vary depending on the script, typeface, language, location within a word, location within a line, implementation complexity, font capabilities, and calligraphic preferences, and may not be possible in certain cases at all. It may involve the use of shortening ligatures, swash variants, contextual forms, elongation glyphs such as U+0640 ـ ARABIC TATWEEL, or other microtypography. It is outside the scope of CSS to define rules for these effects. Authors should avoid applying [letter-spacing](#propdef-letter-spacing) to cursive scripts unless they are prepared to accept non-interoperable results.

<a id="ref-for-propdef-line-padding①"></a>

### <a id="line-padding-property"></a>8.3.  Line Start/End Padding: the [line-padding](#propdef-line-padding) property



| Field               | Definition                                                                         |
|---------------------|------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-line-padding"></a>line-padding                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box⑤"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | absolute length                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                             |



<a id="ref-for-propdef-letter-spacing①⑥"></a>

<a id="ref-for-typographic-character-unit④⑥"></a>

<a id="ref-for-inline-box⑥"></a>

<a id="ref-for-inline-level③"></a>

<a id="ref-for-css-text-sequence②"></a>

<a id="ref-for-atomic-inline④"></a>

<a id="ref-for-justification-opportunity②②"></a>

Whereas [letter-spacing](#propdef-letter-spacing) adjusts spacing between [typographic character units](#typographic-character-unit) and does not apply at the start or end of a line, this property adjusts spacing only at the start/end of a line. The extra spacing is applied only by the <em>innermost</em> [inline box](https://www.w3.org/TR/css-display-3/#inline-box) at the start/end of the line box, and is inserted between that <a id="ref-for-inline-box⑦"></a>inline box’s content edge and the adjacent [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content ([text](https://www.w3.org/TR/css-display-3/#css-text-sequence) <em>or</em> [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline)). This extra space is not a [justification opportunity](#justification-opportunity).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4c81cd27"></a> Given the following HTML and CSS:
>
> ```text
> p    { line-padding: 0.5em; line-height: 1; text-align: center }
> span { background: black; color: white; }
> em   { background: green; color: white; }
> 
> <p><span>Here is <em>some text</em></span>
> ```
>
> Line-padding will be inserted such that an extra 0.5em of inline background will be visible on each side of each line. If it renders such that there is a break between “some” and “text”, the additional padding will be: on the first line, black on the left and green on the right, and on the second line, green on both sides.
>
> <a id="line-padding-rendering"></a>
>
> ```text
> Here is some
> text
> ```
<a id="ref-for-propdef-text-autospace②"></a>

### <a id="text-autospace-property"></a>8.4.  Automatic Contextual Spacing: the [text-autospace](#propdef-text-autospace) property



| Field               | Definition                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-autospace"></a>text-autospace                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-autospace"></a><a id="ref-for-comb-one⑧⓪"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<autospace\>](#typedef-autospace) <a id="ref-for-comb-one⑧①"></a>\| auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                       |



<a id="ref-for-inline-formatting-context②"></a>

Controls spacing between adjacent characters on the same line within the same [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context) using a set of character-class-based rules, allowing for automatic control over inter-script spacing and for spacing around punctuation.

Values are defined as follows:

<a id="typedef-autospace"></a>

<a id="ref-for-typedef-autospace①"></a>

<a id="ref-for-comb-one⑧②"></a>

<a id="ref-for-comb-any⑧"></a>

<a id="ref-for-comb-any⑨"></a>

<a id="ref-for-comb-any①⓪"></a>

<a id="ref-for-comb-one⑧③"></a>

```text
<autospace> = no-autospace |
              [ ideograph-alpha || ideograph-numeric || punctuation ]
              || [ insert | replace ]
```
<a id="valdef-text-autospace-normal"></a>normal  
Same behavior as ideograph-alpha ideograph-numeric.

<a id="valdef-text-autospace-no-autospace"></a>no-autospace  
No automatic space is inserted.

<a id="valdef-text-autospace-insert"></a>insert  
<a id="ref-for-unicode-general-category⑨"></a>

The specified spacing is automatically inserted if there are no space characters of any kind ([Unicode general category](#unicode-general-category) `Z`) already there.

<a id="ref-for-valdef-text-autospace-insert"></a>

<a id="ref-for-valdef-text-autospace-replace"></a>

If neither [insert](#valdef-text-autospace-insert) nor [replace](#valdef-text-autospace-replace) are specified, the behavior is the same as <a id="ref-for-valdef-text-autospace-insert①"></a>insert.

<a id="valdef-text-autospace-replace"></a>replace  
<a id="ref-for-valdef-text-autospace-insert②"></a>

<a id="ref-for-unicode-general-category①⓪"></a>

<a id="ref-for-spaces③⑥"></a>

The specified spacing is automatically inserted even if there is already a [space](#spaces) (U+0020) at that point; additionally, the <a id="ref-for-spaces③⑦"></a>space (U+0020) is removed. Other types of space characters ([Unicode general category](#unicode-general-category) `Z`) suppress automatic spacing, as for [insert](#valdef-text-autospace-insert).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is for correcting text which is using the easy-to-type U+0020 instead of proper spacing.

<a id="valdef-text-autospace-ideograph-alpha"></a>ideograph-alpha  
<a id="ref-for-non-ideographic-letters"></a>

<a id="ref-for-ideographs"></a>

Creates extra spacing between runs of [ideographs](#ideographs) and [non-ideographic letters](#non-ideographic-letters), see [§ 8.4.1 Inter-script Spacing](#inter-script-spacing).

<a id="valdef-text-autospace-ideograph-numeric"></a>ideograph-numeric  
<a id="ref-for-non-ideographic-numerals"></a>

<a id="ref-for-ideographs①"></a>

Creates extra spacing between runs of [ideographs](#ideographs) and [non-ideographic numerals](#non-ideographic-numerals), see [§ 8.4.1 Inter-script Spacing](#inter-script-spacing).

<a id="valdef-text-autospace-punctuation"></a>punctuation  
Creates extra non-breaking spacing around punctuation as required by language-specific typographic conventions.

In this level, if the element’s content language is French, narrow no-break space (U+202F) and no-break space (U+00A0) is inserted where required by [French typographic guidelines](https://web.archive.org/web/20201112013601/http://unicode.org/udhr/n/notes_fra.html). Otherwise this value has no effect. However future specifications may add automatic spacing behavior for other languages.

<a id="valdef-text-autospace-auto"></a>auto  
The user agent chooses a set of typographically high quality spacing values. Different user agents running on different platforms may pick different values.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These spacing values may or may not match OS platform conventions.

<a id="ref-for-propdef-word-spacing⑦"></a>

<a id="ref-for-propdef-letter-spacing①⑦"></a>

<a id="ref-for-propdef-text-autospace③"></a>

This property is additive with the [word-spacing](#propdef-word-spacing) and [letter-spacing](#propdef-letter-spacing) properties. That is, the amount of spacing contributed by the <a id="ref-for-propdef-letter-spacing①⑧"></a>letter-spacing setting (if any) is added to the spacing created by [text-autospace](#propdef-text-autospace). The same applies to <a id="ref-for-propdef-word-spacing⑧"></a>word-spacing.

At element boundaries, the amount of extra spacing introduced between characters is determined by and rendered within the innermost element that contains the boundary.

#### <a id="inter-script-spacing"></a>8.4.1.  Inter-script Spacing

<a id="ref-for-valdef-text-autospace-ideograph-alpha"></a>

<a id="ref-for-valdef-text-autospace-ideograph-numeric"></a>

<a id="ref-for-margin"></a>

<a id="ref-for-border"></a>

<a id="ref-for-padding"></a>

The [ideograph-alpha](#valdef-text-autospace-ideograph-alpha) and [ideograph-numeric](#valdef-text-autospace-ideograph-numeric) values introduce spacing at the boundary between particular classes of characters when they are directly adjoining on a line, i.e. without any intervening non-zero [margin](https://www.w3.org/TR/css-box-4/#margin), [border](https://www.w3.org/TR/css-box-4/#border), or [padding](https://www.w3.org/TR/css-box-4/#padding) or intervening characters (such as a quotation mark or a space). The amount of space introduced by these keywords is 1/8 of the CJK advance measure, i.e 0.125ic.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Spacing conventions vary, but values typically range from 1/4ic to as low as 1/8ic, with 1/4ic being more common in historical contexts due to metal type limitations and 1/6ic or thinner being more common in proportional typesetting. Because these spaces are inserted by default (through the initial value, normal), CSS uses 1/8ic in order to be conservative in its interference. A future level of this module may introduce control over the amount of spacing.

<a id="ref-for-propdef-text-spacing-trim②"></a>

### <a id="text-spacing-trim-property"></a>8.5.  CJK Punctuation Spacing: the [text-spacing-trim](#propdef-text-spacing-trim) property



| Field               | Definition                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-spacing-trim"></a>text-spacing-trim                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑧④"></a><a id="ref-for-typedef-spacing-trim"></a>[\<spacing-trim\>](#typedef-spacing-trim) [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                |



<a id="ref-for-inline-formatting-context③"></a>

Controls spacing around CJK punctuation characters on the same line within the same [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context) using a set of character-class-based rules, allowing them to be set halfwidth or fullwidth based on their position and neighbors within the line.

Values are defined as follows:

<a id="typedef-spacing-trim"></a>

<a id="ref-for-typedef-spacing-trim①"></a>

<a id="ref-for-comb-one⑧⑤"></a>

<a id="ref-for-comb-one⑧⑥"></a>

<a id="ref-for-comb-one⑧⑦"></a>

<a id="ref-for-comb-one⑧⑧"></a>

<a id="ref-for-comb-one⑧⑨"></a>

```text
<spacing-trim> = space-all | normal | space-first | trim-start | trim-both | trim-all
```
<a id="valdef-text-spacing-trim-space-all"></a>space-all  
All fullwidth punctuation characters are set with full-width glyphs (spaced).

<a id="valdef-text-spacing-trim-normal"></a>normal  
<a id="ref-for-fullwidth-closing-punctuation"></a>

<a id="ref-for-fullwidth-opening-punctuation"></a>

Set [fullwidth opening punctuation](#fullwidth-opening-punctuation) with full-width glyphs (spaced) at the start of each line; set [fullwidth closing punctuation](#fullwidth-closing-punctuation) with half-width glyphs (flush) at the end of each line if it does not otherwise fit prior to justification, else set the punctuation with full-width glyphs; and collapse spacing between punctuation glyphs [as described below](#fullwidth-collapsing).

<a id="valdef-text-spacing-trim-trim-both"></a>trim-both  
<a id="ref-for-fullwidth-closing-punctuation①"></a>

<a id="ref-for-fullwidth-opening-punctuation①"></a>

Set [fullwidth opening punctuation](#fullwidth-opening-punctuation) with half-width glyphs (flush) at the start of each line; set [fullwidth closing punctuation](#fullwidth-closing-punctuation) with half-width glyphs (flush) at the end of each line; and collapse spacing between punctuation glyphs [as described below](#fullwidth-collapsing).

<a id="valdef-text-spacing-trim-space-first"></a>space-first  
<a id="ref-for-forced-line-break⑨"></a>

<a id="ref-for-block-container⑦"></a>

<a id="ref-for-fullwidth-opening-punctuation②"></a>

Set [fullwidth opening punctuation](#fullwidth-opening-punctuation) with full-width glyphs (spaced) on the first line of the [block container](https://www.w3.org/TR/css-display-3/#block-container) and each line after a [forced line break](#forced-line-break). Otherwise as normal.

> <strong data-conversion-semantic="note">Note</strong>
>
> This value exists for compat requirements.
> <a id="ref-for-valdef-text-spacing-trim-trim-both"></a>
>
> <a id="ref-for-valdef-text-spacing-trim-space-all"></a>
>
> This value exists to manage formatting of some existing Chinese and Japanese content, for which [trim-both](#valdef-text-spacing-trim-trim-both) would have been appropriate typographically, except that they are already written to expect the first line to be set as for [space-all](#valdef-text-spacing-trim-space-all).
>
> <a id="ref-for-propdef-hanging-punctuation"></a>
>
> <a id="ref-for-propdef-text-indent"></a>
>
> <a id="ref-for-valdef-text-spacing-trim-trim-both①"></a>
>
> Specifically, due to the lack of reliable [hanging-punctuation](#propdef-hanging-punctuation) support across UAs, existing content (especially ePub content) uses U+3000 ideographic space in place of [text-indent](#propdef-text-indent), but omits it when the paragraph begins with punctuation that is desired to hang in the indent in order to create the hanging punctuation effect. Using [trim-both](#valdef-text-spacing-trim-trim-both) on the first line would thus trim away the effective indent in such content and thus obscure that line’s distinction as the first line of a new paragraph.
>
> <a id="ref-for-propdef-hanging-punctuation①"></a>
>
> <a id="ref-for-propdef-text-indent①"></a>
>
> Note that this typesetting practice of using ideographic spaces for indentation (sometimes and not always) is contrary to the separation of content and style offered by HTML and CSS. Using [hanging-punctuation](#propdef-hanging-punctuation) and [text-indent](#propdef-text-indent) to control paragraph formatting rather than tweaking the text content of the document preserves the text’s true semantics in the document source and allows the style sheet designer to freely switch among the various spacing/indentation styles without needing to alter the content. See [§ 8.5.3 Japanese Paragraph-start Conventions in CSS](#japanese-start-edges) for examples.
>
> <a id="ref-for-valdef-text-spacing-trim-normal"></a>
>
> <a id="ref-for-valdef-text-spacing-trim-trim-start"></a>
>
> <a id="ref-for-valdef-text-spacing-trim-trim-both②"></a>
>
> <a id="ref-for-valdef-text-spacing-trim-space-all①"></a>
>
> Additionally, the behavior at the end of lines is aligned with the [normal](#valdef-text-spacing-trim-normal) and [trim-start](#valdef-text-spacing-trim-trim-start) values rather than [trim-both](#valdef-text-spacing-trim-trim-both), in that it only trims the glyphs if they do not otherwise fit prior to justification. While improving the typography in fewer cases, it is closer to the legacy behavior of [space-all](#valdef-text-spacing-trim-space-all) which reduces compatibility concerns.

<a id="valdef-text-spacing-trim-trim-start"></a>trim-start  
<a id="ref-for-fullwidth-opening-punctuation③"></a>

Set [fullwidth opening punctuation](#fullwidth-opening-punctuation) with half-width glyphs (flush) at the start of each line. Otherwise as normal.

<a id="valdef-text-spacing-trim-trim-all"></a>trim-all  
<a id="ref-for-fullwidth-middle-dot-punctuation"></a>

<a id="ref-for-fullwidth-closing-punctuation②"></a>

<a id="ref-for-fullwidth-opening-punctuation④"></a>

Set [fullwidth opening punctuation](#fullwidth-opening-punctuation), [fullwidth closing punctuation](#fullwidth-closing-punctuation), and [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation) with half-width glyphs, without regards for the position within the line nor for adjacent characters.

<a id="valdef-text-spacing-trim-auto"></a>auto  
The user agent chooses a set of typographically high quality spacing values. Different user agents running on different platforms may pick different values.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These spacing values may or may not match OS platform conventions.

<a id="ref-for-valdef-text-spacing-trim-auto"></a>

<a id="ref-for-valdef-text-spacing-trim-trim-both③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-366d9875"></a> Do we need [auto](#valdef-text-spacing-trim-auto)? It would be weird for the author to choose platform-dependent behavior at the start of the first line, and it should otherwise use [trim-both](#valdef-text-spacing-trim-trim-both).

> <strong data-conversion-semantic="note">Note</strong>
>
> Here is an informal summary of what the various values do:
>
> **Table 36**
>
> Representation note: complete merged-header paths are explicit; inherited span values are repeated where they apply. Native HTML span and row-header accessibility semantics are not available in GFM.
>
> | Value | Trim at line start | Trim at line end | Trim adjacent pairs | Trim everywhere |
> | --- | --- | --- | --- | --- |
> | <a id="ref-for-valdef-text-spacing-trim-space-all②"></a> **[space-all](#valdef-text-spacing-trim-space-all)** | no | no | no | no |
> | <a id="ref-for-valdef-text-spacing-trim-normal①"></a> **[normal](#valdef-text-spacing-trim-normal)** | no | only if would not fit | yes | no |
> | <a id="ref-for-valdef-text-spacing-trim-space-first"></a> **[space-first](#valdef-text-spacing-trim-space-first)** | yes except on the first line | only if would not fit | yes | no |
> | <a id="ref-for-valdef-text-spacing-trim-trim-start①"></a> **[trim-start](#valdef-text-spacing-trim-trim-start)** | yes | only if would not fit | yes | no |
> | <a id="ref-for-valdef-text-spacing-trim-trim-both④"></a> **[trim-both](#valdef-text-spacing-trim-trim-both)** | yes | yes | yes | no |
> | <a id="ref-for-valdef-text-spacing-trim-trim-all"></a> **[trim-all](#valdef-text-spacing-trim-trim-all)** | yes | yes | yes | yes |
> | <a id="ref-for-valdef-text-spacing-trim-auto①"></a> **[auto](#valdef-text-spacing-trim-auto)** | user-agent specific / platform dependent | user-agent specific / platform dependent | user-agent specific / platform dependent | user-agent specific / platform dependent |

#### <a id="fullwidth-collapsing"></a>8.5.1.  Fullwidth Punctuation Collapsing

<a id="ref-for-propdef-text-spacing-trim③"></a>

Typically, fullwidth characters have glyphs with the same advance width as a standard Han character (e.g. 水 U+6C34). However, many fullwidth punctuation glyphs only take up part of the fullwidth design space. Thus such punctuation are not always set fullwidth. Several values of [text-spacing-trim](#propdef-text-spacing-trim) allow the author to control when such characters are set half-width (typically half the width of an ideograph) and when they are set full-width.

In order to set the text as specified, the UA will need to either

- trim (kern) the blank half of the glyphs, if they are given full-width and must be set half-width, or
- add space to the glyphs, if they are given half-width and must be set full-width.

The UA <em>may</em> use the OpenType `halt` and `vhal` features if implemented by a font in order to perform the requisite trimming of a particular glyph. The UA <em>must not</em> use the `hwid` feature or otherwise substitute halfwidth forms as switching to halfwidth glyphs can change the glyph shape which is not acceptable here.

Some fonts use proportional glyphs for fullwidth punctuation characters. If there is no support in the font for distinguishing fullwidth vs halfwidth glyph shapes (e.g. through font features), then for such proportional glyphs, the given advance width is considered simultaneously full-width and half-width: the UA must not add or remove space to these glyphs.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The advance width of a standard Han character can be determined either from font metrics such as the OpenType `ideo` and `idtp` baselines for the opposite writing mode, or by taking the advance width of a Han character such as 水 U+6C34. (The opposite writing mode must be used because some fonts are compressed so that the characters are not square.) More information on OpenType metrics can be found [in the OpenType spec](https://docs.microsoft.com/en-us/typography/opentype/spec/baselinetags#ideoembox). Note that if 水 U+6C34, 卜 U+535C, and 一 U+4E00 do not all have the same advance width, the font has proportional ideographs and the fullwidth advance width cannot be reliably determined by measuring glyphs.

Some fonts have fullwidth punctuation characters whose blank are too small to trim (kern.) UA may choose not to trim (kern) when the UA determined the trimming (kerning) may cause glyph cut-offs, collisions, or excessive kerning through glyph bounding box, glyph metrics, or font features.

<a id="ref-for-propdef-text-spacing-trim④"></a>

<a id="ref-for-valdef-text-spacing-trim-trim-all①"></a>

<a id="ref-for-valdef-text-spacing-trim-space-all③"></a>

If [text-spacing-trim](#propdef-text-spacing-trim) is [trim-all](#valdef-text-spacing-trim-trim-all), the UA must collapse the space typically associated with such full width glyphs regardless of the context in which they appear. Otherwise, unless <a id="ref-for-propdef-text-spacing-trim⑤"></a>text-spacing-trim is set to [space-all](#valdef-text-spacing-trim-space-all) (or the font has proportional fullwidth punctuation glyphs), the UA must collapse the space typically associated with such full width glyphs when placed adjacently on a line as follows:

- <a id="ref-for-fullwidth-opening-punctuation⑤"></a>

  Set [fullwidth opening punctuation](#fullwidth-opening-punctuation) half-width if the previous character is any of:

  - <a id="ref-for-fullwidth-opening-punctuation⑥"></a>

    a [fullwidth opening punctuation](#fullwidth-opening-punctuation)

  - <a id="ref-for-fullwidth-middle-dot-punctuation①"></a>

    a [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation)

  - an ideographic space (U+3000)

  - <a id="ref-for-fullwidth-closing-punctuation③"></a>

    <a id="ref-for-propdef-font-size⑥"></a>

    a [fullwidth closing punctuation](#fullwidth-closing-punctuation) of an equivalent or larger [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size)

  - <a id="ref-for-unicode-general-category①①"></a>

    a character belonging to Unicode [general category](#unicode-general-category) `Ps`

  Otherwise set it full-width.

- <a id="ref-for-fullwidth-closing-punctuation④"></a>

  Set [fullwidth closing punctuation](#fullwidth-closing-punctuation) half-width if the next character is any of:

  - <a id="ref-for-fullwidth-closing-punctuation⑤"></a>

    a [fullwidth closing punctuation](#fullwidth-closing-punctuation)

  - <a id="ref-for-fullwidth-middle-dot-punctuation②"></a>

    a [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation)

  - an ideographic space (U+3000)

  - <a id="ref-for-fullwidth-opening-punctuation⑦"></a>

    <a id="ref-for-propdef-font-size⑦"></a>

    a [fullwidth opening punctuation](#fullwidth-opening-punctuation) of a larger [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size)

  - <a id="ref-for-unicode-general-category①②"></a>

    a character belonging to Unicode [general category](#unicode-general-category) `Pe`

  Otherwise set it full-width.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eec02af5"></a> The following example table lists the punctuation pairs affected by adjacent-pairs trimming. It uses halfwidth equivalents to approximate the trimming effect.
>
> 
>
> | Combination         | Sample Pair                         | Looks Like        |
> |---------------------|-------------------------------------|-------------------|
> | <strong>Opening—Opening &#xA;       </strong> | <samp lang="ja">〔</samp>+<samp lang="ja">（</samp> | <samp lang="ja">〔(</samp> |
> | <strong>Middle Dot—Opening &#xA;       </strong> | <samp lang="ja">・</samp>+<samp lang="ja">（</samp> | <samp lang="ja">・(</samp> |
> | <strong>Closing—Opening &#xA;       </strong> | <samp lang="ja">〕</samp>+<samp lang="ja">（</samp> | <samp lang="ja">〕(</samp> |
> | <strong>Ideographic Space—Opening &#xA;       </strong> | <samp lang="ja">　</samp>+<samp lang="ja">（</samp> | <samp lang="ja">　(</samp> |
> | <strong>Closing—Closing &#xA;       </strong> | <samp lang="ja">）</samp>+<samp lang="ja">〕</samp> | <samp lang="ja">)〕</samp> |
> | <strong>Closing—Middle Dot &#xA;       </strong> | <samp lang="ja">）</samp>+<samp lang="ja">・</samp> | <samp lang="ja">)・</samp> |
> | <strong>Closing—Ideographic Space &#xA;       </strong> | <samp lang="ja">）</samp>+<samp lang="ja">　</samp> | <samp lang="ja">)　</samp> |
>
> Demonstration of adjacent-pairs punctuation trimming
>
> 

#### <a id="text-spacing-classes"></a>8.5.2.  Text Spacing Character Classes

In the context of this property the following definitions apply:

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-fbe5fc02"></a> Classes and Unicode code points are under review, and at least some changes are needed to accommodate more recent additions to Unicode. [\[Issue \#9503\]](https://github.com/w3c/csswg-drafts/issues/9503)

<a id="ideographs"></a>ideographs  
<a id="ref-for-typographic-character-unit④⑦"></a>

Includes all [typographic character units](#typographic-character-unit) [\[CSS-TEXT-3\]](#biblio-css-text-3) whose base character is listed below:

- <a id="ref-for-unicode-general-category①③"></a>

  All characters in the range of U+3041 to U+30FF, except those that belong to Unicode Punctuation \[P\*\] [general category](#unicode-general-category).

- CJK Strokes (U+31C0 to U+31EF).

- Katakana Phonetic Extensions (U+31F0 to U+31FF).

- <a id="ref-for-unicode-script①"></a>

  All characters that have the Han [script property](#unicode-script).

<a id="non-ideographic-letters"></a>non-ideographic letters  
<a id="ref-for-unicode-general-category①④"></a>

<a id="ref-for-typographic-character-unit④⑧"></a>

Includes all [typographic character units](#typographic-character-unit) that belong to Unicode Letters \[L\*\] and Mark \[M\*\] [general category](#unicode-general-category), except when any of the following conditions are met:

- <a id="ref-for-ideographs②"></a>

  is defined as [ideograph](#ideographs).

- is categorized as East Asian Fullwidth (F) by [\[UAX11\]](#biblio-uax11).

- <a id="ref-for-propdef-text-combine-upright"></a>

  <a id="ref-for-propdef-text-orientation"></a>

  is upright in vertical text flow using the [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) property or the [text-combine-upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-combine-upright) property.

<a id="non-ideographic-numerals"></a>non-ideographic numerals  
<a id="ref-for-unicode-general-category①⑤"></a>

<a id="ref-for-typographic-character-unit④⑨"></a>

Includes all [typographic character](#typographic-character-unit) units that belong to the Unicode Decimal Digit Number \[Nd\] [general category](#unicode-general-category), except when any of the following conditions are met:

- is categorized as East Asian Fullwidth (F) by [\[UAX11\]](#biblio-uax11).

- <a id="ref-for-propdef-text-combine-upright①"></a>

  <a id="ref-for-propdef-text-orientation①"></a>

  is upright in vertical text flow using the [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) property or the [text-combine-upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-combine-upright) property.

<a id="fullwidth-opening-punctuation"></a>fullwidth opening punctuation  
Includes any opening punctuation character (Unicode category `Ps`) that belongs to the CJK Symbols and Punctuation block (U+3000–U+303F) or is categorized as East Asian Fullwidth (F) by [\[UAX11\]](#biblio-uax11). Also includes LEFT SINGLE QUOTATION MARK (U+2018) and LEFT DOUBLE QUOTATION MARK (U+201C). When trimmed, the left (for horizontal text) or top (for vertical text) half is kerned.

<a id="fullwidth-closing-punctuation"></a>fullwidth closing punctuation  
<a id="ref-for-fullwidth-dot-punctuation"></a>

<a id="ref-for-fullwidth-colon-punctuation"></a>

Includes any closing punctuation character (Unicode category `Pe`) that belongs to the CJK Symbols and Punctuation block (U+3000–U+303F) or is categorized as East Asian Fullwidth (F) by [\[UAX11\]](#biblio-uax11). Also includes RIGHT SINGLE QUOTATION MARK (U+2019) and RIGHT DOUBLE QUOTATION MARK (U+201D). May also include [fullwidth colon punctuation](#fullwidth-colon-punctuation) and/or [fullwidth dot punctuation](#fullwidth-dot-punctuation) ([see below](#fullwidth-ambiguous)). When trimmed, the right (for horizontal text) or bottom (for vertical text) half is kerned.

<a id="fullwidth-middle-dot-punctuation"></a>fullwidth middle dot punctuation  
<a id="ref-for-fullwidth-dot-punctuation①"></a>

<a id="ref-for-fullwidth-colon-punctuation①"></a>

Includes MIDDLE DOT (U+00B7), HYPHENATION POINT (U+2027), and KATAKANA MIDDLE DOT (U+30FB). May also include [fullwidth colon punctuation](#fullwidth-colon-punctuation) and/or [fullwidth dot punctuation](#fullwidth-dot-punctuation) ([see below](#fullwidth-ambiguous)).

<a id="fullwidth-colon-punctuation"></a>fullwidth colon punctuation  
Includes FULLWIDTH COLON (U+FF1A) and FULLWIDTH SEMICOLON (U+FF1B).

<a id="fullwidth-dot-punctuation"></a>fullwidth dot punctuation  
Includes IDEOGRAPHIC COMMA (U+3001), IDEOGRAPHIC FULL STOP (U+3002), FULLWIDTH COMMA (U+FF0C), FULLWIDTH FULL STOP (U+FF0E).

<a id="ref-for-fullwidth-colon-punctuation②"></a>

<a id="ref-for-fullwidth-dot-punctuation②"></a>

<a id="ref-for-fullwidth-closing-punctuation⑥"></a>

<a id="ref-for-fullwidth-middle-dot-punctuation③"></a>

<a id="fullwidth-ambiguous"></a> Whether [fullwidth colon punctuation](#fullwidth-colon-punctuation) and [fullwidth dot punctuation](#fullwidth-dot-punctuation) should be considered [fullwidth closing punctuation](#fullwidth-closing-punctuation) or [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation) depends on where in the glyph’s box the punctuation is drawn. If the punctuation is centered, then it should be considered middle dot punctuation. If the punctuation is drawn to one side (left in horizontal text, top in vertical text) and the other half is therefore blank then the punctuation should be considered closing punctuation and trimmed accordingly.

<a id="ref-for-fullwidth-colon-punctuation③"></a>

<a id="ref-for-fullwidth-dot-punctuation③"></a>

<a id="ref-for-fullwidth-closing-punctuation⑦"></a>

<a id="ref-for-fullwidth-middle-dot-punctuation④"></a>

The UA must classify [fullwidth colon punctuation](#fullwidth-colon-punctuation) and [fullwidth dot punctuation](#fullwidth-dot-punctuation) under either the [fullwidth closing punctuation](#fullwidth-closing-punctuation) category or the [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation) category as appropriate. The UA may rely on language conventions and the writing mode (horizontal vs. vertical), and/or font information to determine this categorization. The UA may also add additional characters to any category as appropriate.

> <strong data-conversion-semantic="note">Note</strong>
>
> The following informative table summarizes language conventions for classifying fullwidth colon and dot punctuation:
>
> 
>
> |                     | colon punctuation | dot punctuation |
> |---------------------|-------------------|-----------------|
> | <strong>Simplified Chinese (horizontal) &#xA;       </strong> | closing           | closing         |
> | <strong>Simplified Chinese (vertical) &#xA;       </strong> | closing           | closing         |
> | <strong>Traditional Chinese &#xA;       </strong> | middle dot        | middle dot      |
> | <strong>Korean &#xA;       </strong> | middle dot        | closing         |
> | <strong>Japanese &#xA;       </strong> | middle dot        | closing         |
>
> 
>
> Note that for Chinese fonts at least, the author observes that the standard convention is often not followed.

#### <a id="japanese-start-edges"></a>8.5.3.  Japanese Paragraph-start Conventions in CSS

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-74dbaaeb"></a> Japanese has three common start-edge typesetting schemes, which are distinguished by their handling of opening brackets.
>
> ![The first scheme aligns opening brackets flush with the indent edge on the first line and with the start edge of other lines. The second scheme gives the opening bracket its full width, so that it is effectively indented half an em from the indent edge and from the start edge of other lines. The third scheme aligns the opening brackets flush with the start edge of lines, but hangs them inside the indent on the first line (resulting in an effective half-em indent instead of the full em for paragraphs that begin with an opening bracket).](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/opening-brackets-at-line-head.png)
>
> Positioning of opening brackets at line head [\[JLREQ\]](#biblio-jlreq)
>
> Assuming a UA style sheet of `p { margin: 1em 0; }`, CSS can achieve the Japanese typesetting styles with the following rules:
>
> - Brackets flush with indent, flush with other lines (first scheme):
>
>   ```text
>   p { /* Flush alignment */
>     margin: 0;
>     text-indent: 1em;
>     text-spacing-trim: trim-both;
>   }
>   ```
>
> - Brackets preserve fullwidth spacing on all lines (second scheme):
>
>   ```text
>   p { /* Fullwidth alignment */
>     margin: 0;
>     text-indent: 1em;
>     text-spacing-trim: space-all;
>   }
>   ```
>
> - Brackets hang in indent, flush with other lines (third scheme):
>
>   ```text
>   p { /* Hanging alignment */
>     margin: 0;
>     text-indent: 1em;
>     text-spacing-trim: trim-both;
>     hanging-punctuation: first;
>   }
>   ```
<a id="ref-for-propdef-text-spacing①"></a>

### <a id="text-spacing-property"></a>8.6.  Character Class Spacing Shorthand: the [text-spacing](#propdef-text-spacing) property



| Field               | Definition                                                                                                                                                                                                                                                                                        |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-spacing"></a>text-spacing                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-autospace②"></a><a id="ref-for-comb-any①①"></a><a id="ref-for-typedef-spacing-trim②"></a><a id="ref-for-comb-one⑨⓪"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto <a id="ref-for-comb-one⑨①"></a>\| [\<spacing-trim\>](#typedef-spacing-trim) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<autospace\>](#typedef-autospace) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                       |



<a id="ref-for-propdef-text-spacing-trim⑥"></a>

<a id="ref-for-propdef-text-autospace④"></a>

This property is a shorthand for setting [text-spacing-trim](#propdef-text-spacing-trim) and [text-autospace](#propdef-text-autospace) in a single declaration. Values are defined as follows:

<a id="valdef-text-spacing-none"></a>none

<a id="ref-for-valdef-text-autospace-no-autospace"></a>

<a id="ref-for-propdef-text-autospace⑤"></a>

<a id="ref-for-valdef-text-spacing-trim-space-all④"></a>

<a id="ref-for-propdef-text-spacing-trim⑦"></a>

Turns off all text-spacing features: sets [text-spacing-trim](#propdef-text-spacing-trim) to [space-all](#valdef-text-spacing-trim-space-all) and [text-autospace](#propdef-text-autospace) to [no-autospace](#valdef-text-autospace-no-autospace).

<a id="valdef-text-spacing-auto"></a>auto

<a id="ref-for-valdef-text-autospace-auto"></a>

<a id="ref-for-propdef-text-autospace⑥"></a>

<a id="ref-for-propdef-text-spacing-trim⑧"></a>

Sets both [text-spacing-trim](#propdef-text-spacing-trim) and [text-autospace](#propdef-text-autospace) to [auto](#valdef-text-autospace-auto).

<a id="ref-for-typedef-spacing-trim③"></a>

<a id="valdef-text-spacing-spacing-trim"></a>[\<spacing-trim\>](#typedef-spacing-trim)

<a id="ref-for-initial-value②"></a>

<a id="ref-for-propdef-text-autospace⑦"></a>

<a id="ref-for-typedef-autospace③"></a>

<a id="ref-for-propdef-text-spacing-trim⑨"></a>

Sets [text-spacing-trim](#propdef-text-spacing-trim) to the specified value. If no [\<autospace\>](#typedef-autospace) value is given, [text-autospace](#propdef-text-autospace) is set to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-typedef-autospace④"></a>

<a id="valdef-text-spacing-autospace"></a>[\<autospace\>](#typedef-autospace)

<a id="ref-for-initial-value③"></a>

<a id="ref-for-propdef-text-spacing-trim①⓪"></a>

<a id="ref-for-typedef-spacing-trim④"></a>

<a id="ref-for-propdef-text-autospace⑧"></a>

Sets [text-autospace](#propdef-text-autospace) to the specified value. If no [\<spacing-trim\>](#typedef-spacing-trim) value is given, [text-spacing-trim](#propdef-text-spacing-trim) is set to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-valdef-text-spacing-trim-normal②"></a>

<a id="ref-for-propdef-text-spacing-trim①①"></a>

<a id="ref-for-propdef-text-autospace⑨"></a>

<a id="ref-for-propdef-text-spacing②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As [normal](#valdef-text-spacing-trim-normal) is the initial value of both [text-spacing-trim](#propdef-text-spacing-trim) and [text-autospace](#propdef-text-autospace), [text-spacing: normal](#propdef-text-spacing) resets both to their initial values.

### <a id="boundary-shaping"></a>8.7.  Shaping Across Element Boundaries

<a id="ref-for-typographic-character-unit⑤⓪"></a>

Text shaping <em>must</em> be broken at inline box boundaries when any of the following are true for any box whose boundary separates the two [typographic character units](#typographic-character-unit):

- <a id="ref-for-typographic-character-unit⑤①"></a>

  <a id="ref-for-propdef-padding①"></a>

  <a id="ref-for-propdef-border①"></a>

  <a id="ref-for-propdef-margin①"></a>

  Any of [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)/[border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border)/[padding](https://www.w3.org/TR/css-box-4/#propdef-padding) separating the two [typographic character units](#typographic-character-unit) in the inline axis is non-zero.

- <a id="ref-for-valdef-alignment-baseline-baseline"></a>

  <a id="ref-for-propdef-vertical-align"></a>

  [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) is not [baseline](https://www.w3.org/TR/css-inline-3/#valdef-alignment-baseline-baseline).

- <a id="ref-for-bidi-isolate"></a>

  The boundary is a [bidi isolation boundary](https://www.w3.org/TR/css-writing-modes-4/#bidi-isolate).

Text shaping <em>must not</em> be broken across inline box boundaries when there is no effective change in formatting, or if the only formatting changes do not affect the glyphs (as in applying [text decoration](https://www.w3.org/TR/css-text-decor-3/)).

Text shaping <em>should not</em> be broken across inline box boundaries otherwise, if it is reasonable and possible for that case given the limitations of the font technology.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d57c8f22"></a> An example of reasonable and possible shaping across boundaries is Arabic shaping: in many systems this is performed by the font engine, allowing the font to provide variant glyphs with potentially very sophisticated contextual shaping. It’s not generally possible to rely on this system across a font change unless the font engine has an API to provide context, but it is straightforward and therefore quite reasonable for an engine to work around this limitation by, for example, using the zero-width-joiner (U+200D) or zero-width-non-joiner (U+200C) as appropriate to solicit the correct choice of initial/medial/final/isolated glyph.
>
> An example of possible but not reasonable shaping across boundaries is handling a font that is sensitive to 20 characters of context on either side to choose its glyphs: passing all the text before <em>and after</em> the string in question, even through multiple inline boundaries with formatting changes, is complicated. The UA <em>could</em> handle such cases, but is not required to, as they are not typical or fundamentally required by any modern writing system.
>
> An example of impossible shaping across boundaries is a change in font weight partway through the word “and” in a font where a ligature would replace all three letters of the word “and” with an ampersand glyph (“&#x26;”).

## <a id="edge-effects"></a>9.  Edge Effects

<a id="ref-for-propdef-text-indent②"></a>

<a id="ref-for-propdef-hanging-punctuation②"></a>

Edge effects control the indentation of lines with respect to other lines in the block ([text-indent](#propdef-text-indent)) and how content is measured at the start and end edges of a line ([hanging-punctuation](#propdef-hanging-punctuation)).

<a id="ref-for-propdef-text-indent③"></a>

### <a id="text-indent-property"></a>9.1.  First Line Indentation: the [text-indent](#propdef-text-indent) property<a id="text-indent"></a>



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-indent"></a>text-indent                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-comb-all①"></a><a id="ref-for-typedef-length-percentage④"></a>\[ [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \] [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) hanging[?](https://www.w3.org/TR/css-values-4/#mult-opt) <a id="ref-for-comb-all②"></a>&#x26;&#x26; each-line<a id="ref-for-mult-opt②"></a>? |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-inner-size"></a><a id="ref-for-inline-axis①"></a>refers to block container’s own [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [inner size](https://www.w3.org/TR/css-sizing-3/#inner-size)                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑤"></a>computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value, plus any specified keywords                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                                                                                                        |



<a id="ref-for-start③"></a>

This property specifies the indentation applied to lines of inline content in a block. The indent is treated as a margin applied to the [start](https://www.w3.org/TR/css-writing-modes-4/#start) edge of the line box.

<a id="ref-for-valdef-text-indent-each-line"></a>

<a id="ref-for-valdef-text-indent-hanging"></a>

<a id="ref-for-first-formatted-line"></a>

Unless otherwise specified by the [each-line](#valdef-text-indent-each-line) and/or [hanging](#valdef-text-indent-hanging) keywords, only lines that are the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of an element are affected. [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) For example, the first line of an anonymous block box is only affected if it is the first child of its parent element.

Values have the following meanings:

<a id="valdef-text-indent-length"></a>\<length\>  
Gives the amount of the indent as an absolute length.

<a id="valdef-text-indent-percentage"></a>\<percentage\>  
<a id="ref-for-logical-width"></a>

Gives the amount of the indent as a percentage of the block container’s own [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width).

<a id="ref-for-intrinsic-size-contribution"></a>

Percentages must be treated as 0 for the purpose of calculating [intrinsic size contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution), but are always resolved normally when performing layout.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This can lead to the element overflowing. It is not recommended to use percentage indents and intrinsic sizing together.

<a id="valdef-text-indent-each-line"></a>each-line  
<a id="ref-for-soft-wrap-break"></a>

<a id="ref-for-forced-line-break①⓪"></a>

Indentation affects the first line of each block container and each line after a [forced line break](#forced-line-break) (but not lines after a [soft wrap break](#soft-wrap-break)).

<a id="valdef-text-indent-hanging"></a>hanging  
Inverts which lines are affected.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7dde3c71"></a>
>
> <a id="ref-for-propdef-text-align⑨"></a>
>
> <a id="ref-for-valdef-text-align-start⑤"></a>
>
> <a id="ref-for-propdef-text-indent④"></a>
>
> If [text-align](#propdef-text-align) is [start](#valdef-text-align-start) and [text-indent](#propdef-text-indent) is 5em in left-to-right text with no floats present, then first line of text will start 5em into the block:
>
> ```text
>      Since CSS1 it has been possible to
> indent the first line of a block element
> 5em by setting the 'text-indent' property 
> to '5em'.
> ```
>
> <a id="ref-for-valdef-text-indent-hanging①"></a>
>
> If we add the [hanging](#valdef-text-indent-hanging) keyword, then the first line will start flush, but other lines will be indented 5em:
>
> ```text
> In CSS3 we can instead indent all other
>      lines of the block element by 5em
>      by setting the 'text-indent' property
>      to 'hanging 5em'.
> ```
<a id="ref-for-propdef-text-indent⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f29e1f90"></a> Since the [text-indent](#propdef-text-indent) property only affects the “first formatted line”, a line after a forced break will not be indented.
>
> ```text
>    For example, in the middle of
> this paragraph is an equation,
> which is centered:
>              x + y = z
> The first line after the equation
> is flush (else it would look like
> we started a new paragraph).
> ```
>
> <a id="ref-for-propdef-text-indent⑥"></a>
>
> However, sometimes (as in poetry or code), it is appropriate to indent each line that happens to be long enough to wrap. In the following example, [text-indent](#propdef-text-indent) is given a value of 3em hanging each-line, giving the third line of the poem a hanging indent where it soft-wraps at the block’s right boundary:
>
> ```text
> In a short line of text
> There need be no wrapping,
> But when we go on and on and on  
>    and on,
> Sometimes a soft break
> Can help us stay on the page.
> ```
<a id="ref-for-propdef-text-indent⑦"></a>

<a id="ref-for-propdef-display"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the [text-indent](#propdef-text-indent) property inherits, when specified on a block element, it will affect descendant inline-block elements. For this reason, it is often wise to specify <a id="ref-for-propdef-text-indent⑧"></a>text-indent: 0 on elements that are specified [display: inline-block](https://www.w3.org/TR/css-display-3/#propdef-display).

### <a id="hanging"></a>9.2.  Hanging Glyphs

<a id="ref-for-hang①⓪"></a>

<a id="ref-for-intrinsic-size①"></a>

<a id="ref-for-min-content④"></a>

<a id="ref-for-max-content"></a>

When a glyph at the start or end edge of a line <a id="hang"></a>hangs, it is not considered when measuring the line’s contents for fit, alignment, or justification. Depending on the line’s alignment/justification, this can result in the mark being placed outside the line box. The [hanging](#hang) glyph is also not taken into account when computing [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) ([min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) and [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content)), and any sizes derived thereof. (The interaction of this measurement and kerning is currently UA-defined; the CSSWG [welcomes advice](https://github.com/w3c/csswg-drafts/issues/2397) on this point.)

<a id="ref-for-hang①①"></a>

<a id="ref-for-hang①②"></a>

<a id="ref-for-inline-box⑧"></a>

<a id="ref-for-hanging-glyph"></a>

<a id="ref-for-ink-overflow"></a>

<a id="ref-for-scrollable-overflow"></a>

A <a id="hanging-glyph"></a>[hanging](#hang) glyph is still enclosed inside its parent inline box and still participates in text justification: its character advance is just not measured when determining how much content fits on the line, how much the line’s contents need to be expanded or compressed for justification, or how to position the content within the line box for text alignment. Effectively, the [hanging](#hang) glyph character advance is re-interpreted as an additional negative margin on the affected edge of its parent [inline box](https://www.w3.org/TR/css-display-3/#inline-box); the line is otherwise laid out as usual. An overflowing [hanging glyph](#hanging-glyph) should typically be considered [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) so as to avoid creating unnecessary scrollbars, but the UA may treat it as [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) when the content is editable or in other circumstances where treating it as <a id="ref-for-scrollable-overflow①"></a>scrollable overflow would be useful to the user. [\[CSS-OVERFLOW-3\]](#biblio-css-overflow-3)

<a id="ref-for-hang①③"></a>

<a id="ref-for-conditionally-hang⑥"></a>

<a id="ref-for-min-content⑤"></a>

<a id="ref-for-max-content①"></a>

In some cases, a glyph at the end of a line can <a id="conditionally-hang"></a>conditionally hang: it [hangs](#hang) only if it does not otherwise fit in the line prior to justification. It is not considered when measuring the line’s contents for fit; however, any part of it that does not fit is considered to <a id="ref-for-hang①④"></a>hang. Glyphs that [conditionally hang](#conditionally-hang) are not taken into account when computing [min-content sizes](https://www.w3.org/TR/css-sizing-3/#min-content) and any sizes derived thereof, but they are taken into account for [max-content sizes](https://www.w3.org/TR/css-sizing-3/#max-content) and any sizes derived thereof.

<a id="ref-for-hang①⑤"></a>

Non-zero inline-axis borders or padding between a [hang](#hang)able glyph and the edge of the line prevent the glyph from hanging. For example, a period at the end of an inline box with end padding does not <a id="ref-for-hang①⑥"></a>hang at the end edge of a line.

<a id="ref-for-hang①⑦"></a>

Multiple adjacent glyphs can hang together, however specific limits on how many are allowed to hang may be specified (e.g. at most one punctuation character may [hang](#hang) at each edge of the line).

<a id="ref-for-propdef-hanging-punctuation③"></a>

#### <a id="hanging-punctuation-property"></a>9.2.1.  Hanging Punctuation: the [hanging-punctuation](#propdef-hanging-punctuation) property<a id="hanging-punctuation"></a>



| Field               | Definition                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hanging-punctuation"></a>hanging-punctuation                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any①②"></a><a id="ref-for-comb-one⑨②"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ first [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ force-end <a id="ref-for-comb-one⑨③"></a>\| allow-end \] <a id="ref-for-comb-any①③"></a>\|\| last \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                    |



<a id="ref-for-hang①⑧"></a>

This property determines whether a punctuation mark, if one is present, [hangs](#hang) and may be placed outside the line box (or in the indent) at the start or at the end of a line of text.

<a id="ref-for-propdef-hanging-punctuation④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If there is not sufficient padding on the block container, [hanging-punctuation](#propdef-hanging-punctuation) can trigger overflow.

Values have the following meanings:

<a id="valdef-hanging-punctuation-none"></a>none  
<a id="ref-for-hang①⑨"></a>

No punctuation character is made to [hang](#hang).

<a id="valdef-hanging-punctuation-first"></a>first  
<a id="ref-for-hang②⓪"></a>

<a id="ref-for-first-formatted-line①"></a>

An opening bracket, quote, or ideographic space at the start of the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of an element [hangs](#hang). This applies to all characters in the Unicode categories Ps, Pf, Pi plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK and the IDEOGRAPHIC SPACE U+3000.

<a id="valdef-hanging-punctuation-last"></a>last  
<a id="ref-for-hang②①"></a>

A closing bracket or quote at the end of the last formatted line of an element [hangs](#hang). This applies to all characters in the Unicode categories Pe, Pf, Pi plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK.

<a id="valdef-hanging-punctuation-force-end"></a>force-end  
<a id="ref-for-hang②②"></a>

<a id="ref-for-stop-or-comma"></a>

A [stop or comma](#stop-or-comma) at the end of a line [hangs](#hang).

<a id="valdef-hanging-punctuation-allow-end"></a>allow-end  
<a id="ref-for-conditionally-hang⑦"></a>

<a id="ref-for-stop-or-comma①"></a>

A [stop or comma](#stop-or-comma) at the end of a line [conditionally hangs](#conditionally-hang).

<a id="ref-for-hang②③"></a>

At most one punctuation character may [hang](#hang) at each edge of the line.

<a id="ref-for-hang②④"></a>

<a id="stop-or-comma"></a>Stops and commas allowed to [hang](#hang) include:



| Column 1 | Column 2 | Column 3                        |
|----------|----------|---------------------------------|
| U+002C   | ,        | COMMA                           |
| U+002E   | .        | FULL STOP                       |
| U+060C   | ،        | ARABIC COMMA                    |
| U+06D4   | ۔        | ARABIC FULL STOP                |
| U+3001   | 、       | IDEOGRAPHIC COMMA               |
| U+3002   | 。       | IDEOGRAPHIC FULL STOP           |
| U+FF0C   | ，       | FULLWIDTH COMMA                 |
| U+FF0E   | ．       | FULLWIDTH FULL STOP             |
| U+FE50   | ﹐       | SMALL COMMA                     |
| U+FE51   | ﹑       | SMALL IDEOGRAPHIC COMMA         |
| U+FE52   | ﹒       | SMALL FULL STOP                 |
| U+FF61   | ｡        | HALFWIDTH IDEOGRAPHIC FULL STOP |
| U+FF64   | ､        | HALFWIDTH IDEOGRAPHIC COMMA     |



The UA may include other characters as appropriate.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSS Working Group would appreciate if UAs including other characters would [inform the working group](#sotd) of such additions.

<a id="ref-for-valdef-hanging-punctuation-allow-end"></a>

<a id="ref-for-valdef-hanging-punctuation-force-end"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-68d23fbc"></a> The [allow-end](#valdef-hanging-punctuation-allow-end) and [force-end](#valdef-hanging-punctuation-force-end) are two variations of hanging punctuation used in East Asia.
>
> ![hanging-punctuation: allow-end](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/hanging-punctuation-allow-end.png)
>
> ```text
> p {
>   text-align: justify;
>   hanging-punctuation: allow-end;
> }
> ```
>
> ![hanging-punctuation: force-end](https://www.w3.org/TR/2024/WD-css-text-4-20240529/images/hanging-punctuation-force-end.png)
>
> ```text
> p {
>   text-align: justify;
>   hanging-punctuation: force-end;
> }
> ```
>
> <a id="ref-for-valdef-hanging-punctuation-allow-end①"></a>
>
> <a id="ref-for-valdef-hanging-punctuation-force-end①"></a>
>
> The punctuation at the end of the first line for [allow-end](#valdef-hanging-punctuation-allow-end) does not hang, because it fits without hanging. However, if [force-end](#valdef-hanging-punctuation-force-end) is used, it is forced to hang. The justification measures the line without the hanging punctuation. Therefore when the line is expanded, the punctuation is pushed outside the line.

### <a id="bidi-linebox"></a>9.3.  Bidirectionality and Line Boxes

<a id="ref-for-start④"></a>

<a id="ref-for-end③"></a>

<a id="ref-for-inline-base-direction"></a>

<a id="ref-for-line-box①"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-bidi-paragraph"></a>

<a id="ref-for-propdef-text-align-all⑦"></a>

<a id="ref-for-propdef-text-align-last⑨"></a>

<a id="ref-for-propdef-text-indent⑨"></a>

<a id="ref-for-propdef-hanging-punctuation⑤"></a>

The [start](https://www.w3.org/TR/css-writing-modes-4/#start) and [end](https://www.w3.org/TR/css-writing-modes-4/#end) sides of a line box are determined by the [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) of the line box. Although they usually match, the <a id="ref-for-inline-base-direction①"></a>inline base direction of a [line box](https://www.w3.org/TR/css-inline-3/#line-box) is distinct from the <a id="ref-for-inline-base-direction②"></a>inline base direction of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) or the [bidi paragraph](https://www.w3.org/TR/css-writing-modes-4/#bidi-paragraph). The <a id="ref-for-line-box②"></a>line box’s <a id="ref-for-inline-base-direction③"></a>inline base direction affects [text-align-all](#propdef-text-align-all), [text-align-last](#propdef-text-align-last), [text-indent](#propdef-text-indent), and [hanging-punctuation](#propdef-hanging-punctuation)—​i.e. the position and alignment of its contents with respect to its edges. It does not affect the formatting or ordering of inline content (which is controlled by the [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/) as applied by [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/) [\[UAX9\]](#biblio-uax9) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)).

<a id="ref-for-line-box③"></a>

<a id="ref-for-inline-base-direction④"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-unicode-bidi"></a>

In most cases, a [line box](https://www.w3.org/TR/css-inline-3/#line-box)’s [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is given by its [containing block](https://www.w3.org/TR/css-display-3/#containing-block)’s computed [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction). However, if its <a id="ref-for-containing-block②"></a>containing block has [unicode-bidi: plaintext](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4):

- <a id="ref-for-inline-base-direction⑤"></a>

  <a id="ref-for-line-box④"></a>

  <a id="ref-for-bidi-paragraph①"></a>

  If the [bidi paragraph](https://www.w3.org/TR/css-writing-modes-4/#bidi-paragraph) to which the [line box](https://www.w3.org/TR/css-inline-3/#line-box) belongs (that is, the <a id="ref-for-bidi-paragraph②"></a>bidi paragraph for which the line box holds content) has strong directionality, the line box’s [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is that direction.

- <a id="ref-for-propdef-direction②"></a>

  <a id="ref-for-inline-base-direction⑥"></a>

  <a id="ref-for-atomic-inline⑤"></a>

  <a id="ref-for-line-box⑤"></a>

  If the [line box](https://www.w3.org/TR/css-inline-3/#line-box) is empty (i.e. contains no [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) or characters other than the newline character, if any) or otherwise has no strong directionality (contains only weak or neutral characters), its [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is taken from the preceding line box (if any), or, if this is the first line box in the containing block, from the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the containing block. (This can result in an RTL line box whose contents have an LTR base direction.)

<a id="ref-for-propdef-display①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0a5e92f2"></a> In the following example, assuming the `<block>` is a start-aligned preformatted block ([display: block; white-space: pre; text-align: start](https://www.w3.org/TR/css-display-3/#propdef-display)), every other line is right-aligned:
>
> ```text
> <block style="unicode-bidi: plaintext">
> français
> فارسی
> français
> فارسی
> français
> فارسی
> </block>
> ```
<a id="ref-for-propdef-text-align①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a82b4049"></a> Because neutral characters (such as punctuation) and isolated runs are skipped when [finding the inline base direction of a plaintext bidi paragraph](http://unicode.org/reports/tr9/#P2), the line box in the following example will be left-to-right (and thus left-aligned given [text-align: start](#propdef-text-align)), as dictated by the first strong character, ‘h’:
>
> ```text
> <para style="display: block; direction: rtl; unicode-bidi:plaintext">
> “<quote style="unicode-bidi:plaintext">שלום!</quote>”, they said.
> </para>
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-478dd3e8"></a>
>
> ```text
> <textarea style="direction: rtl; unicode-bidi:plaintext">
> 
> Hello!
> 
> </textarea>
> ```
>
> <a id="ref-for-propdef-unicode-bidi①"></a>
>
> <a id="ref-for-propdef-direction③"></a>
>
> Because of [unicode-bidi: plaintext](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), the “Hello!” is typeset LTR (i.e. with the exclamation mark on the right side) and left-aligned, ignoring the containing block’s RTL [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction). This makes the empty line following it LTR as well, which means that a caret on that line should appear at its left edge. The empty first line, however, is right-aligned: having no preceding line, it assumes the RTL direction of its containing block.

## <a id="order"></a> Appendix A: Text Processing Order of Operations

<em>This appendix is normative.</em>

The following list defines the order of text operations. (Implementations are not bound to this order as long as the resulting layout is the same.)

1.  [§ 4.2 White Space Trimming: the white-space-trim property](#white-space-trim)

2.  [white space processing](#white-space-phase-1) part I (pre-wrapping)

3.  [§ 2.2 Expanding Between Words: the word-space-transform property](#word-space-transform) and [text transformation](#transforming)

4.  [text combination](https://www.w3.org/TR/css-writing-modes-4/#text-combine-upright) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

5.  [text orientation](https://www.w3.org/TR/css-writing-modes-4/#text-orientation) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

6.  <a id="ref-for-wrapping⑥"></a>

    [text wrapping](#wrapping) while applying per line:

    - [indentation](#text-indent-property)

    - [bidirectional reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) [\[CSS2\]](#biblio-css2) / [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

    - [white space processing](#white-space-phase-2) part II

    - [font/glyph selection and positioning](https://www.w3.org/TR/css-fonts-3/) [\[CSS-FONTS-3\]](#biblio-css-fonts-3)

    - <a id="ref-for-propdef-letter-spacing①⑨"></a>

      <a id="ref-for-propdef-word-spacing⑨"></a>

      <a id="ref-for-propdef-text-spacing③"></a>

      <a id="ref-for-propdef-line-padding②"></a>

      [letter-spacing](#propdef-letter-spacing), [word-spacing](#propdef-word-spacing), [text-spacing](#propdef-text-spacing), and [line-padding](#propdef-line-padding)

    - [hanging punctuation](#hanging-punctuation-property)

7.  [justification](#justification) (which may affect glyph selection and/or text wrapping, looping back into that step)

8.  [text alignment](#text-align-property)

9.  [text group alignment](#text-group-align-property)

## <a id="plaintext"></a> Appendix B: Conversion to Plaintext

<em>This appendix is normative</em> for the purpose of plaintext copy-paste operations.

When a CSS-rendered document is converted to a plaintext format, it is expected that:

- <a id="ref-for-propdef-text-transform⑦"></a>

  The [text-transform](#propdef-text-transform) property has no effect.

- <a id="ref-for-forced-line-break①①"></a>

  <a id="ref-for-block"></a>

  <a id="ref-for-white-space②①"></a>

  <a id="ref-for-collapsible-white-space①⓪"></a>

  <a id="ref-for-propdef-white-space-trim⑧"></a>

  [white-space-trim](#propdef-white-space-trim) and [§ 4.3.1 Phase I: Collapsing and Transformation](#white-space-phase-1) is applied and any sequence of [collapsible](#collapsible-white-space) [white space](#white-space) at the beginning of a [block](https://www.w3.org/TR/css-display-3/#block) or immediately following a [forced line break](#forced-line-break) is removed.

## <a id="default-stylesheet"></a> Appendix C: Default UA Stylesheet

<em>This appendix is informative,</em> and is to help UA developers to implement a default stylesheet for HTML, but UA developers are free to ignore or modify as appropriate.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a0119720"></a>
>
> ```text
> /* make option elements align together */
> option { text-align: match-parent; }
> 
> /* do not allow white space to collapse in textarea */
> textarea { white-space-collapse: preserve !important; }
> 
> /* preserve character grid in preformatted text */
> pre, code, kbd, samp, tt { text-spacing: none; }
> 
> /* Avoid hanging punctuation inheriting into preformatted blocks */
> pre { hanging-punctuation: none; }
> ```
## <a id="script-groups"></a> Appendix D: Scripts and Spacing

<em>This appendix is normative.</em>

<a id="ref-for-unicode-script②"></a>

<a id="ref-for-justification-opportunity②③"></a>

Typographic behavior varies somewhat by language, but varies drastically by writing system. This appendix categorizes some common [scripts](#unicode-script) in Unicode 6.0 according to their justification and spacing behavior. Category descriptions are descriptive, not prescriptive; the determining factor is the prioritization of [justification opportunities](#justification-opportunity).

<a id="block-scripts"></a>block scripts  
<a id="ref-for-writing-system-japanese③"></a>

<a id="ref-for-writing-system-korean"></a>

<a id="ref-for-writing-system-chinese③"></a>

<a id="ref-for-content-writing-system④"></a>

<a id="ref-for-unicode-east-asian-width②"></a>

<a id="ref-for-unicode-script③"></a>

CJK and by extension all Wide characters (see [East Asian Width](https://www.unicode.org/reports/tr11/) [\[UAX11\]](#biblio-uax11)). The following [Unicode scripts](#unicode-script) are included: Bopomofo, Han, Hangul, Hiragana, Katakana, and Yi. Characters of the [East Asian Width property](#unicode-east-asian-width) `Wide` and `Fullwidth` are also included, but `Ambiguous` characters are included only if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese), [Korean](#writing-system-korean), or [Japanese](#writing-system-japanese).

<a id="clustered-scripts"></a>clustered scripts  
<a id="ref-for-unicode-script④"></a>

Clustered scripts have discrete units and break only at word boundaries, but do not use visible word separators. They prioritize stretching spaces, but comfortably admit inter-character spacing for justification. The clustered scripts include, but are not limited to, the following [Unicode scripts](#unicode-script): Khmer, Lao, Myanmar, New Tai Lue, Tai Le, Tai Tham, Tai Viet, Thai

<a id="cursive-script"></a>cursive scripts  
<a id="ref-for-unicode-script⑤"></a>

<a id="ref-for-propdef-letter-spacing②⓪"></a>

Cursive scripts do not admit gaps between their letters for either justification or [letter-spacing](#propdef-letter-spacing). The following [Unicode scripts](#unicode-script) are included: Arabic, Hanifi Rohingya, Mandaic, Mongolian, N’Ko, Phags Pa, Syriac

<a id="ref-for-cursive-script④"></a>

<a id="ref-for-typographic-character-unit⑤②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Indic scripts with baseline connectors (such as Devanagari and Gujarati) <em>are not</em> considered [cursive scripts](#cursive-script), and <em>do</em> admit such gaps between [typographic character units](#typographic-character-unit). See [Indic Layout Requirements](https://www.w3.org/TR/ilreq/). [\[ILREQ\]](#biblio-ilreq)

User agents should update this list as they update their Unicode support to handle as-yet-unencoded cursive scripts in future versions of Unicode, and are encouraged to ask the CSSWG to update this spec accordingly.

## <a id="character-properties"></a> Appendix E: Characters and Properties

<em>This appendix is normative.</em>

Unicode defines four code point-level properties that are referenced in CSS typesetting:

<a id="unicode-east-asian-width"></a>[East Asian width property](http://www.unicode.org/reports/tr11/#Definitions)  
Defined in Unicode Standard Annex \#11 [\[UAX11\]](#biblio-uax11) and given as the `East_Asian_Width` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44).

<a id="unicode-general-category"></a>[general category](http://www.unicode.org/reports/tr44/#General_Category_Values)  
Defined in Unicode Standard Annex \#44 [\[UAX44\]](#biblio-uax44) and given as the `General_Category` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) \[UAX44\].

<a id="unicode-script"></a>[script property](http://www.unicode.org/reports/tr24/#Values)  
Defined in Unicode Standard Annex \#24 [\[UAX24\]](#biblio-uax24) and given as the `Script` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44). (UAs must include any ScriptExtensions.txt assignments in this mapping.)

<a id="unicode-vertical-orientation"></a>[Vertical Orientation](http://www.unicode.org/reports/tr50/)  
Defined in Unicode Standard Annex \#50 [\[UAX50\]](#biblio-uax50) as the Vertical_Orientation property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44).

<a id="ref-for-typographic-character-unit⑤③"></a>

<a id="ref-for-grapheme-cluster④"></a>

Unicode defines properties for individual code points, but sometimes it is necessary to determine the properties of a [typographic character unit](#typographic-character-unit). For the purposes of CSS Text, the properties of a <a id="ref-for-typographic-character-unit⑤④"></a>typographic character unit are given by the base character of its first [grapheme cluster](#grapheme-cluster)—​except in two cases:

- <a id="ref-for-grapheme-cluster⑤"></a>

  [Grapheme clusters](#grapheme-cluster) formed with an Enclosing Mark (`Me`) of the Common script are considered to be Other Symbols (`So`) in the Common script. They are assumed to have the same Unicode properties as the replacement character (U+FFFD).

- <a id="ref-for-grapheme-cluster⑥"></a>

  [Grapheme clusters](#grapheme-cluster) formed with a Space Separator (`Zs`) as the base are considered to be Modifier Symbols (`Sk`). They are assumed to have the same East Asian Width property as the base, but take their other properties from the first combining character in the sequence.

## <a id="script-tagging"></a> Appendix F: Identifying the Content Writing System

<em>This appendix is normative.</em>

While most languages have a preferred writing system, some have multiple, and most can also be transcribed into one or more foreign writing systems. As a common example, most languages have at least one Latin transcription, and can thus be written in the Latin writing system. Transcribed texts typically adopt the typographic conventions of the writing system: for example Japanese “romaji” and Chinese Pinyin use Latin letters and word spaces, and follow Latin line-breaking and justification practices accordingly. As another example, historical ideographic Korean (`ko-Hani`) does not use word spaces, and should therefore be typeset similar to Chinese rather than modern Korean.

<a id="ref-for-doclanguage④"></a>

<a id="ref-for-content-language②④"></a>

In HTML or any other [document language](https://www.w3.org/TR/CSS21/conform.html#doclanguage) using BCP47 tags for identifying languages to declare the [content language](#content-language), authors can disambiguate or indicate the use of an atypical writing system with script subtags. [\[BCP47\]](#biblio-bcp47) For example, to indicate use of the Latin writing system for languages which don’t natively use it, the `-Latn` script subtag can be added, e.g. `ja-Latn` for Japanese romaji. Other subtags exist for other writing systems, see ISO’s Code for the Representation of Names of Scripts and the [ISO15924 script tag registry](http://unicode.org/iso15924/iso15924-codes.html). [\[ISO15924\]](#biblio-iso15924)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a2325afd"></a> Some common/historical examples of using BCP47 tags with script subtags:
>
> `zh-Latn`  
> Chinese, written in Latin transcription.
>
> `ko-Hani`  
> Korean, written in Hanja (Chinese ideographic characters).
>
> `tr-Arab`  
> Turkish, written in Arabic script.
>
> `mn-Cyrl`  
> Mongolian, written in Cyrillic.
>
> `mn-Mong`  
> Mongolian, written in traditional Mongolian script.

However, BCP47 script subtags are not typically used (and are in fact discouraged) for languages strongly associated with a single writing system: instead that writing system is expected to be implied when no other is specified. [\[BCP47\]](#biblio-bcp47) IANA maintains a database of various languages’ most common writing system via the `Suppress-Script` field in its [language subtag registry](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry) for this purpose.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: More advice on language tagging can be found in the [Internationalization Working Group](https://www.w3.org/International/core/)’s [“Language tags in HTML and XML”](https://www.w3.org/International/articles/language-tags/) and [“Choosing a Language Tag”](https://www.w3.org/International/questions/qa-choosing-language-tags).

<a id="ref-for-content-language②⑤"></a>

When no writing system is explicitly indicated, UAs should assume the most common writing system of the declared [content language](#content-language) for language-sensitive typographic behaviors such as line-breaking or justification. However, UAs must not assume that writing system if the author has explicitly declared a different one. If the UA has no language-specific knowledge of a particular language and writing system combination, it must use the typographic conventions of the declared writing system (assuming the conventions of a different language if necessary), not the conventions of the declared language in an assumed writing system, which would be inappropriate to the declared writing system.

The full correspondence between languages and their most common writing systems is out of scope for this document. However, user agents must assume at least the following:

- <a id="ref-for-content-writing-system⑤"></a>

  <a id="ref-for-content-language②⑥"></a>

  If the [content language](#content-language) is Chinese and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language②⑦"></a>content language if the <a id="ref-for-content-writing-system⑥"></a>writing system to specified to be one of the Hant, Hans, Hani, Hanb, or Bopo ISO script codes, then the <a id="ref-for-content-writing-system⑦"></a>writing system is <a id="writing-system-chinese"></a>Chinese.

- <a id="ref-for-content-writing-system⑧"></a>

  <a id="ref-for-content-language②⑧"></a>

  If the [content language](#content-language) is Japanese and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language②⑨"></a>content language if the <a id="ref-for-content-writing-system⑨"></a>writing system to specified to be one of the Jpan, Hrkt, Hira, or Kana ISO script codes, then the <a id="ref-for-content-writing-system①⓪"></a>writing system is <a id="writing-system-japanese"></a>Japanese.

- <a id="ref-for-content-writing-system①①"></a>

  <a id="ref-for-content-language③⓪"></a>

  If the [content language](#content-language) is Korean and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language③①"></a>content language if the <a id="ref-for-content-writing-system①②"></a>writing system to specified to be one of the Kore, Hang, or Jamo ISO script codes, then the <a id="ref-for-content-writing-system①③"></a>writing system is <a id="writing-system-korean"></a>Korean.

- <a id="ref-for-content-language③②"></a>

  <a id="ref-for-content-writing-system①④"></a>

  The [writing system](#content-writing-system) is only considered to be <a id="writing-system-known"></a>unknown if the [content language](#content-language) itself is unknown, or if it explicitly indicates an unknown writing system.

  <a id="ref-for-content-writing-system①⑤"></a>

  <a id="ref-for-content-language③③"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Mere omission of the [writing system](#content-writing-system) information when the [content language](#content-language) is declared means the that the <a id="ref-for-content-writing-system①⑥"></a>writing system is implied, not unknown.

## <a id="small-kana"></a> Appendix G: Small Kana Mappings

<em>This appendix is normative.</em>



| <a id="kana-small"></a>Small | <a id="kana-full-size"></a>Full-size |
|--------------------------|------------------------------|
| ぁ U+3041                | あ U+3042                    |
| ぃ U+3043                | い U+3044                    |
| ぅ U+3045                | う U+3046                    |
| ぇ U+3047                | え U+3048                    |
| ぉ U+3049                | お U+304A                    |
| ゕ U+3095                | か U+304B                    |
| ゖ U+3096                | け U+3051                    |
| 𛄲 U+1B132               | こ U+3053                    |
| っ U+3063                | つ U+3064                    |
| ゃ U+3083                | や U+3084                    |
| ゅ U+3085                | ゆ U+3086                    |
| ょ U+3087                | よ U+3088                    |
| ゎ U+308E                | わ U+308F                    |
| 𛅐 U+1B150               | ゐ U+3090                    |
| 𛅑 U+1B151               | ゑ U+3091                    |
| 𛅒 U+1B152               | を U+3092                    |
| ァ U+30A1                | ア U+30A2                    |
| ィ U+30A3                | イ U+30A4                    |
| ゥ U+30A5                | ウ U+30A6                    |
| ェ U+30A7                | エ U+30A8                    |
| ォ U+30A9                | オ U+30AA                    |
| ヵ U+30F5                | カ U+30AB                    |
| ㇰ U+31F0                | ク U+30AF                    |
| ヶ U+30F6                | ケ U+30B1                    |
| 𛅕 U+1B155               | コ U+30B3                    |
| ㇱ U+31F1                | シ U+30B7                    |
| ㇲ U+31F2                | ス U+30B9                    |
| ッ U+30C3                | ツ U+30C4                    |
| ㇳ U+31F3                | ト U+30C8                    |
| ㇴ U+31F4                | ヌ U+30CC                    |
| ㇵ U+31F5                | ハ U+30CF                    |
| ㇶ U+31F6                | ヒ U+30D2                    |
| ㇷ U+31F7                | フ U+30D5                    |
| ㇸ U+31F8                | ヘ U+30D8                    |
| ㇹ U+31F9                | ホ U+30DB                    |
| ㇺ U+31FA                | ム U+30E0                    |
| ャ U+30E3                | ヤ U+30E4                    |
| ュ U+30E5                | ユ U+30E6                    |
| ョ U+30E7                | ヨ U+30E8                    |
| ㇻ U+31FB                | ラ U+30E9                    |
| ㇼ U+31FC                | リ U+30EA                    |
| ㇽ U+31FD                | ル U+30EB                    |
| ㇾ U+31FE                | レ U+30EC                    |
| ㇿ U+31FF                | ロ U+30ED                    |
| ヮ U+30EE                | ワ U+30EF                    |
| 𛅤 U+1B164               | ヰ U+30F0                    |
| 𛅥 U+1B165               | ヱ U+30F1                    |
| 𛅦 U+1B166               | ヲ U+30F2                    |
| 𛅧 U+1B167               | ン U+30F3                    |
| ｧ U+FF67                 | ｱ U+FF71                     |
| ｨ U+FF68                 | ｲ U+FF72                     |
| ｩ U+FF69                 | ｳ U+FF73                     |
| ｪ U+FF6A                 | ｴ U+FF74                     |
| ｫ U+FF6B                 | ｵ U+FF75                     |
| ｯ U+FF6F                 | ﾂ U+FF82                     |
| ｬ U+FF6C                 | ﾔ U+FF94                     |
| ｭ U+FF6D                 | ﾕ U+FF95                     |
| ｮ U+FF6E                 | ﾖ U+FF96                     |

Small Kana Map to Full-size Kana



## <a id="word-phrase-detection"></a> Appendix H: Word and Phrase Detection

<i>This appendix is normative.</i>

A few operations in this specification depend on automated <a id="word-boundary-detection"></a>word boundary detection or <a id="phrase-boundary-detection"></a>phrase boundary detection.

Both operations are similar: the user agent performs linguistic analysis of the text to identify language-specific meaningful sequences of characters. They differ only targetting different units of text: either words or phrases.

While easily understood in a broad sense, both concepts of word and phrase are difficult to precisely define, especially in a way that works accross multiple languages. Nonetheless, in this context:

- A word is a recognizable semantic unit which may comprise one or more characters or syllables.
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: See [What is a word?](https://www.w3.org/International/articles/typography/linebreak.en#whatisword) for some discussion of this concept.
- A phrase is a short grouping of one or several words standing together as a conceptual or grammatical unit, forming a component of a clause or sentence.
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This is not to be confused with the French word <em lang="fr">phrase</em> which means sentence rather than phrase in the English sense used here. In Japanese, this corresponds to the concept of 文節.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-208c3735"></a> A phrase could be a noun with its article and prepositions or postpositions, a phrasal verb, a compound verb tense…

<a id="ref-for-propdef-word-space-transform④"></a>

<a id="ref-for-propdef-word-break③⓪"></a>

The start and end position of each detected word or phrase is called a <a id="word-boundary"></a>word boundary or <a id="phrase-boundary"></a>phrase boundary. By themselves, word and phrase boundaries are not observable, but properties like [word-space-transform](#propdef-word-space-transform) or [word-break](#propdef-word-break) can cause visible effects at those positions.

The specific algorithm to detect word or phrases and to place their boundaries is UA-dependent, and may take into account a variety of factors or approaches, such as dictionary-based lexical analysis, identification of punctuation or other delimiting characters, morphological analysis, machine learning methods…

The following constraints must nevertheless be respected:

- <a id="ref-for-typographic-character-unit⑤⑤"></a>

  The user agent must not place a word or phrase boundary between characters that compose a single [typographic character unit](#typographic-character-unit).

- The user agent must not place a word or phrase boundary adjacent to any characters with the line breaking class GL, WJ, or ZWJ; when two would-be words or phrases are separated by such characters, they must be treated as a single one. [\[UAX14\]](#biblio-uax14)

- If a word or phrase is immediately followed by one or more of the following characters, the user agent <em>must</em> consider them to be part of the preceeding word or phrase:
  - <a id="ref-for-word-separator①⑥"></a>

    [word-separator characters](#word-separator)

  - <a id="ref-for-other-space-separators⑧"></a>

    [other space separators](#other-space-separators)

  - U+200B ZERO WIDTH SPACE characters

- Punctuation is not a phrase by itself: it should to be attached to the relevant side of an adjacent linguistic word or phrase. For example, enclosing punctuation such as brackets and quotes are part of the phrase they enclose; commas and semicolons are suffixed to the preceding phrase; U+00BF INVERTED QUESTION MARK is attached to the subsequent word; etc.

  However, unconventional use of punctuation, such as smileys, kaomoji, or Perl snippets, may cause the user agent to need to deviate from this principle.

- <a id="ref-for-phrase-boundary"></a>

  <a id="ref-for-word-boundary"></a>

  <a id="boundary-outermost"></a> Inline box boundaries and out-of-flow elements must be ignored when determining word or phrase boundaries. However, if a [word](#word-boundary) or [phrase boundary](#phrase-boundary) is found at the same position as one or more inline box boundaries, the <a id="ref-for-word-boundary①"></a>word or <a id="ref-for-phrase-boundary①"></a>phrase boundary must be inserted in the outermost element that participates in this inline box boundary.

  <a id="ref-for-word-boundary-detection②"></a>

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-08ba2ce6"></a> In the following example, the red “`|`” indicates reasonable positions for a user agent [detecting word boundaries](#word-boundary-detection) to place them:
  > ```text
  > กรุงเทพ|คือ|สวยงาม
  > ```
  >
  > If that sentence had contained some inline markup, the following example shows the correct position to place the word boundaries:
  >
  > ```text
  > กรุงเทพ|คือ|<em>สวยงาม</em>
  > ```
  >
  > The following example shows <em>incorrect</em> positions:
  >
  > ```text
  > กรุงเทพ|คือ<em>|สวยงาม</em>
  > ```
  >
  > The following shows the correct positions in a more contrived situation:
  >
  > ```text
  > กรุงเทพ|<b><u>คือ</u>|<em>สวยงาม</em></b>
  > ```
## <a id="sec"></a> Security Considerations<a id="priv-sec"></a>

This specification introduces no new security considerations.

## <a id="priv"></a> Privacy Considerations

This specification leaks the user’s installed hyphenation and line-breaking dictionaries.

## <a id="acknowledgements"></a> Acknowledgements

This specification would not have been possible without the help from: Addison Phillips, Aharon Lanin, Alan Stearns, Ambrose Li, Arnold Schrijver, Arye Gittelman, Ayman Aldahleh, Ben Errez, Bert Bos, Chris Lilley, Chris Pratley, Chris Thrasher, Chris Wilson, Dave Hyatt, David Baron, Emilio Cobos Álvarez, Eric LeVine, Etan Wexler, Frank Tang, Håkon Wium Lie, IM Mincheol, Ian Hickson, James Clark, Javier Fernandez, John Daggett, Jonathan Kew, Ken Lunde, Laurie Anna Edlund, Marcin Sawicki, Martin Dürst, Martin Heijdra, Masafumi Yabe, Masayasu Ishikawa, Michael Jochimsen, Michel Suignard, Mike Bemford, Myles Maxfield, Nat McCully, Paul Nelson, Pierre-Anthony Lemieux, Rahul Sonnad, Randy Edmunds, Richard Ishida, Shinyu Murakami, Stephen Deach, Steve Zilles, Takao Suzuki, Tantek Çelik, Xidorn Quan, Yaniv Feinberg.

## <a id="changes"></a> Changes

<strong>This draft is kept in sync with <a href="#biblio-css-text-3" title="CSS Text Module Level 3">&#x5B;CSS-TEXT-3&#x5D;</a>, see <a href="https://www.w3.org/TR/css-text-3/#changes"><cite>CSS Text 3</cite> §  Changes</a>.</strong>  
Changes specific to Level 4 are listed below.

Significant changes since the [19 February 2024 Working Draft](https://www.w3.org/TR/2024/WD-css-text-4-20240219/) include:

- <a id="ref-for-propdef-text-spacing-trim①②"></a>

  <a id="ref-for-valdef-text-spacing-trim-trim-both⑤"></a>

  Renamed the [text-spacing-trim](#propdef-text-spacing-trim) value trim-auto to [trim-both](#valdef-text-spacing-trim-trim-both). ([Issue 10161](https://github.com/w3c/csswg-drafts/issues/10161))

- <a id="ref-for-propdef-text-wrap-style⑤"></a>

  Loosened requirement about not changing lines for [text-wrap-style: balance](#propdef-text-wrap-style) for cases with large numbers of lines. ([Issue 10186](https://github.com/w3c/csswg-drafts/issues/10186))

- <a id="ref-for-propdef-text-wrap-style⑥"></a>

  <a id="ref-for-propdef-line-clamp①"></a>

  Define interraction between [text-wrap-style: balance](#propdef-text-wrap-style) and [line-clamp](https://www.w3.org/TR/css-overflow-4/#propdef-line-clamp) ([Issue 9310](https://github.com/w3c/csswg-drafts/issues/9310))

- <a id="ref-for-valdef-text-transform-math-auto"></a>

  <a id="ref-for-propdef-text-transform⑧"></a>

  Add the [math-auto](#valdef-text-transform-math-auto) value for [text-transform](#propdef-text-transform). ([Issue 5386](https://github.com/w3c/csswg-drafts/issues/5386))

Significant changes since the [20 October 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-text-4-20231020/) include:

- <a id="ref-for-forced-line-break①②"></a>

  <a id="ref-for-propdef-text-spacing-trim①③"></a>

  Restored accidentally deleted “and each line after a [forced line break](#forced-line-break)” from definition of [text-spacing-trim: space-first](#propdef-text-spacing-trim). ([Issue 9532](https://github.com/w3c/csswg-drafts/issues/9532))

- <a id="ref-for-valdef-text-spacing-trim-normal③"></a>

  <a id="ref-for-propdef-text-spacing-trim①④"></a>

  Make [normal](#valdef-text-spacing-trim-normal) the initial value of [text-spacing-trim](#propdef-text-spacing-trim). ([Issue 9511](https://github.com/w3c/csswg-drafts/issues/9511))

- <a id="ref-for-valdef-text-spacing-trim-space-first①"></a>

  Change the end-of-line behavior of [space-first](#valdef-text-spacing-trim-space-first). ([Issue 9736](https://github.com/w3c/csswg-drafts/issues/9736))

- <a id="ref-for-valdef-text-spacing-trim-space-first②"></a>

  [space-first](#valdef-text-spacing-trim-space-first) applies after line-breaks. ([Issue 9532](https://github.com/w3c/csswg-drafts/issues/9532))

- Clarify details of fullwidth punctuation collapsing. ([Issue 9225](https://github.com/w3c/csswg-drafts/issues/9225))

- Prevent pre from inheriting hanging-punctuation by default with a user-agent style rule. ([Issue 9689](https://github.com/w3c/csswg-drafts/issues/9689))

Significant changes since the [29 March 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-text-4-20230329/) include:

- <a id="ref-for-propdef-text-wrap③"></a>

  <a id="ref-for-propdef-text-wrap-mode①①"></a>

  <a id="ref-for-propdef-text-wrap-style⑦"></a>

  <a id="ref-for-propdef-white-space①①"></a>

  Recast the [text-wrap](#propdef-text-wrap) property as a shorthand of two new properties, [text-wrap-mode](#propdef-text-wrap-mode) and [text-wrap-style](#propdef-text-wrap-style), and making <a id="ref-for-propdef-text-wrap-mode①②"></a>text-wrap-mode rather than <a id="ref-for-propdef-text-wrap④"></a>text-wrap a longhand of the [white-space](#propdef-white-space) property.

- Updated [Appendix G: Small Kana Mappings](#small-kana) to Unicode 15.0. ([Issue 8442](https://github.com/w3c/csswg-drafts/issues/8442))

- <a id="ref-for-propdef-word-break③①"></a>

  <a id="ref-for-valdef-word-space-transform-auto-phrase"></a>

  <a id="ref-for-propdef-word-space-transform⑤"></a>

  Redesign of the [word-break: auto-phrase](#propdef-word-break) <a id="ref-for-propdef-word-break③②"></a>word-break: manual features (from an earlier attempt as a standalone `word-boundary-detection` property), and add an [auto-phrase](#valdef-word-space-transform-auto-phrase) value to [word-space-transform](#propdef-word-space-transform) (since it can no longer depend on `word-boundary-detection`). and update supporting related mechanics.

- <a id="ref-for-propdef-word-space-transform⑥"></a>

  Rename `word-boundary-expansion` to [word-space-transform](#propdef-word-space-transform)

- <a id="ref-for-valdef-text-spacing-trim-trim-all②"></a>

  <a id="ref-for-propdef-text-spacing-trim①⑤"></a>

  Add [trim-all](#valdef-text-spacing-trim-trim-all) to [text-spacing-trim](#propdef-text-spacing-trim). ([Issue 8482](https://github.com/w3c/csswg-drafts/issues/8482))

- Non-tailorable Unicode line breaking controls other than NBSP take precedence over our rule about atomic inlines. ([Issue 8972](https://github.com/w3c/csswg-drafts/issues/8972))

Significant changes since the [1 March 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-text-4-20230301/) include:

- <a id="ref-for-propdef-white-space①②"></a>

  Completed the translation of [white-space](#propdef-white-space) to multiple longhands by

  - <a id="ref-for-valdef-white-space-collapse-break-spaces④"></a>

    <a id="ref-for-propdef-white-space-collapse①⑤"></a>

    adding [break-spaces](#valdef-white-space-collapse-break-spaces) to [white-space-collapse](#propdef-white-space-collapse) so that all shorthand values can be represented in the longhands ([Issue 8256](https://github.com/w3c/csswg-drafts/issues/8256))

  - <a id="ref-for-propdef-white-space①③"></a>

    integrating all longhand keywords into the [white-space](#propdef-white-space) shorthand

  - updating prose accordingly

- <a id="ref-for-propdef-white-space-collapse①⑥"></a>

  <a id="ref-for-propdef-white-space-trim⑨"></a>

  Renamed text-space-collapse and text-space-trim to [white-space-collapse](#propdef-white-space-collapse) and [white-space-trim](#propdef-white-space-trim). ([Issue 8273](https://github.com/w3c/csswg-drafts/issues/8273))

Significant changes since the [31 December 2022 Working Draft](https://www.w3.org/TR/2022/WD-css-text-4-20221231/) include:

- <a id="ref-for-propdef-text-spacing④"></a>

  Redesigned [text-spacing](#propdef-text-spacing) by:

  - Removing non-useful keyword combinations (Issues [4246](https://github.com/w3c/csswg-drafts/issues/4246), [8288](https://github.com/w3c/csswg-drafts/issues/8288))

  - <a id="ref-for-valdef-text-spacing-trim-space-first③"></a>

    Making [space-first](#valdef-text-spacing-trim-space-first) the initial value ([Issue 2462](https://github.com/w3c/csswg-drafts/issues/2462))

  - Splitting into longhands (Issues [4246](https://github.com/w3c/csswg-drafts/issues/4246), [7183](https://github.com/w3c/csswg-drafts/issues/7183), [8288](https://github.com/w3c/csswg-drafts/issues/8288))

  - Ensuring the ability to turn each feature “off”. (Issues [6950](https://github.com/w3c/csswg-drafts/issues/6950), [8288](https://github.com/w3c/csswg-drafts/issues/8288))

  - Add keywords to allow replacing incorrect space characters in the source. (Issues [318](https://github.com/w3c/csswg-drafts/issues/318), [7183](https://github.com/w3c/csswg-drafts/issues/7183), [8263](https://github.com/w3c/csswg-drafts/issues/8263))

  - <a id="ref-for-propdef-hanging-punctuation⑥"></a>

    Allow [hanging-punctuation](#propdef-hanging-punctuation) to hang leading ideographic spaces to compensate for plaintext-derived source text practices. ([Issue 2462](https://github.com/w3c/csswg-drafts/issues/2462))

  - <a id="ref-for-valdef-text-justify-no-compress"></a>

    <a id="ref-for-propdef-text-justify⑨"></a>

    Move [no-compress](#valdef-text-justify-no-compress) to [text-justify](#propdef-text-justify) since it controls justification more than spacing. ([Issue 7079](https://github.com/w3c/csswg-drafts/issues/7079))

- <a id="ref-for-propdef-text-spacing⑤"></a>

  Extended contextual characters evaluated in [text-spacing](#propdef-text-spacing) to include characters from the `Pe` and `Ps` categories. ([Issue 6091](https://github.com/w3c/csswg-drafts/issues/6091))

- <a id="ref-for-propdef-white-space-collapse①⑦"></a>

  Renamed text-space-collapse back to [white-space-collapse](#propdef-white-space-collapse). ([Issue 8273](https://github.com/w3c/browser-specs/issues/8273))

Significant changes since the [5 May 2022 Working Draft](https://www.w3.org/TR/2022/WD-css-text-4-20220505/) include:

- <a id="ref-for-propdef-text-justify①⓪"></a>

  <a id="ref-for-valdef-text-justify-ruby"></a>

  Added [ruby](#valdef-text-justify-ruby) value to [text-justify](#propdef-text-justify). ([Issue 771](https://github.com/w3c/csswg-drafts/issues/771) [Issue 779](https://github.com/w3c/csswg-drafts/issues/779))

- <a id="ref-for-propdef-text-spacing⑥"></a>

  Switched [text-spacing: normal](#propdef-text-spacing) to use trim-end instead of allow-end. ([Issue 7055](https://github.com/w3c/csswg-drafts/issues/7055))

- <a id="ref-for-propdef-text-spacing⑦"></a>

  Switched [text-spacing: normal](#propdef-text-spacing) to also apply ideograph-alpha and ideograph-numeric, updated UA default stylesheet to exclude these from monospace contexts, specified that non-zero margin/border/padding inhibits space insertion, and defined the amount of inserted space as 0.125ic. ([Issue 6950](https://github.com/w3c/csswg-drafts/issues/6950))

- <a id="ref-for-root-element①"></a>

  <a id="ref-for-valdef-text-align-start⑥"></a>

  <a id="ref-for-propdef-text-align①①"></a>

  Defined [text-align: match-parent](#propdef-text-align) to compute to [start](#valdef-text-align-start) on the [root element](https://www.w3.org/TR/css-display-3/#root-element) for simplicity of implementation. ([Issue 6542](https://github.com/w3c/csswg-drafts/issues/6542))

- <a id="ref-for-valdef-text-justify-inter-character②"></a>

  <a id="ref-for-css-legacy-value-alias①"></a>

  <a id="ref-for-valdef-text-justify-distribute①"></a>

  Allow [distribute](#valdef-text-justify-distribute) keyword to be a [legacy value alias](https://www.w3.org/TR/css-cascade-5/#css-legacy-value-alias) or to simply compute to [inter-character](#valdef-text-justify-inter-character); this allows UAs to do whichever is easier, since the distinction does not matter for backwards-compatibility. ([Issue 7322](https://github.com/w3c/csswg-drafts/issues/7322))

- <a id="ref-for-valdef-white-space-trim-discard-inner"></a>

  <a id="ref-for-propdef-white-space-trim①⓪"></a>

  Renamed trim-inner value of [white-space-trim](#propdef-white-space-trim) to [discard-inner](#valdef-white-space-trim-discard-inner) for consistency with other values. ([Issue 448](https://github.com/w3c/csswg-drafts/issues/448))

Significant changes since the [2019 Working Draft](https://www.w3.org/TR/2019/WD-css-text-4-20191113/) include:

- Integrating the full text of [\[CSS-TEXT-3\]](#biblio-css-text-3).

- <a id="ref-for-propdef-font-size⑧"></a>

  <a id="ref-for-propdef-letter-spacing②①"></a>

  <a id="ref-for-propdef-word-spacing①⓪"></a>

  Adding percentages to [word-spacing](#propdef-word-spacing) and [letter-spacing](#propdef-letter-spacing) to represent sizes relative to the current [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size). ([Issue 2165](https://github.com/w3c/csswg-drafts/issues/2165))

- <a id="ref-for-the-textarea-element"></a>

  Suggesting a UA rule to prevent spaces in <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code> from collapsing. ([Issue 6309](https://github.com/w3c/csswg-drafts/issues/6309))

### <a id="changes-L3"></a> Additions Since Level 3

New features in Level 4:

- <a id="ref-for-propdef-word-break③③"></a>

  [word-break: auto-phrase](#propdef-word-break), for automaticly determining phrases to keep together while line breaking

- <a id="ref-for-propdef-word-space-transform⑦"></a>

  [word-space-transform](#propdef-word-space-transform), for transforming word separators

- <a id="ref-for-propdef-white-space①④"></a>

  breaking up of the [white-space](#propdef-white-space) property into multiple longhands:

  - <a id="ref-for-propdef-white-space-collapse①⑧"></a>

    <a id="ref-for-valdef-white-space-collapse-preserve-spaces③"></a>

    <a id="ref-for-valdef-white-space-collapse-discard"></a>

    [white-space-collapse](#propdef-white-space-collapse) and its [preserve-spaces](#valdef-white-space-collapse-preserve-spaces) and [discard](#valdef-white-space-collapse-discard) values

  - <a id="ref-for-propdef-white-space-trim①①"></a>

    [white-space-trim](#propdef-white-space-trim), for trimming excess white space at the boundaries of an element

  - <a id="ref-for-propdef-text-wrap-mode①③"></a>

    [text-wrap-mode](#propdef-text-wrap-mode) to control whether wrapping occurs or not

- <a id="ref-for-propdef-text-wrap-style⑧"></a>

  <a id="ref-for-valdef-text-wrap-style-balance②"></a>

  <a id="ref-for-valdef-text-wrap-style-stable①"></a>

  <a id="ref-for-valdef-text-wrap-style-pretty②"></a>

  [text-wrap-style](#propdef-text-wrap-style) and its [balance](#valdef-text-wrap-style-balance), [stable](#valdef-text-wrap-style-stable), and [pretty](#valdef-text-wrap-style-pretty) values

- <a id="ref-for-propdef-wrap-before②"></a>

  <a id="ref-for-propdef-wrap-after②"></a>

  <a id="ref-for-propdef-wrap-inside②"></a>

  [wrap-before](#propdef-wrap-before), [wrap-after](#propdef-wrap-after), and [wrap-inside](#propdef-wrap-inside), to avoid or force wrapping (similar to the break-\* properties for pagination)

- <a id="ref-for-propdef-hyphenate-character③"></a>

  [hyphenate-character](#propdef-hyphenate-character), to explicitly control the hyphenation character

- <a id="ref-for-propdef-hyphenate-limit-zone②"></a>

  <a id="ref-for-propdef-hyphenate-limit-chars①"></a>

  <a id="ref-for-propdef-hyphenate-limit-lines①"></a>

  <a id="ref-for-propdef-hyphenate-limit-last②"></a>

  [hyphenate-limit-zone](#propdef-hyphenate-limit-zone), [hyphenate-limit-chars](#propdef-hyphenate-limit-chars), [hyphenate-limit-lines](#propdef-hyphenate-limit-lines), [hyphenate-limit-last](#propdef-hyphenate-limit-last), for better control over automatic hyphenation

- <a id="ref-for-string-value⑧"></a>

  <a id="ref-for-propdef-text-align①②"></a>

  [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) values for [text-align](#propdef-text-align) for aligning on, e.g., a decimal point

- <a id="ref-for-propdef-text-group-align①"></a>

  <a id="ref-for-propdef-text-align①③"></a>

  [text-group-align](#propdef-text-group-align) for group-aligning a set of line boxes whose contents are aligned by [text-align](#propdef-text-align)

- <a id="ref-for-propdef-line-padding③"></a>

  [line-padding](#propdef-line-padding) for inserting spaces within the inline box at the start/end of lines

- <a id="ref-for-propdef-text-spacing⑧"></a>

  [text-spacing](#propdef-text-spacing) for automatic spacing around punctuation and script changes

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

- [alignment character](#valdef-text-align-alignment-character), in § 7.1
- [allow-end](#valdef-hanging-punctuation-allow-end), in § 9.2.1
- [always](#valdef-hyphenate-limit-lines-always), in § 6.3.5
- anywhere
  - [value for line-break](#valdef-line-break-anywhere), in § 6.2
  - [value for overflow-wrap](#valdef-overflow-wrap-anywhere), in § 6.4
- auto
  - [value for hyphenate-character](#valdef-hyphenate-character-auto), in § 6.3.2
  - [value for hyphenate-limit-chars](#valdef-hyphenate-limit-chars-auto), in § 6.3.4
  - [value for hyphens](#valdef-hyphens-auto), in § 6.3.1
  - [value for line-break](#valdef-line-break-auto), in § 6.2
  - [value for text-align-last](#valdef-text-align-last-auto), in § 7.4
  - [value for text-autospace](#valdef-text-autospace-auto), in § 8.4
  - [value for text-justify](#valdef-text-justify-auto), in § 7.5
  - [value for text-spacing](#valdef-text-spacing-auto), in § 8.6
  - [value for text-spacing-trim](#valdef-text-spacing-trim-auto), in § 8.5
  - [value for text-wrap-style](#valdef-text-wrap-style-auto), in § 5.4
  - [value for wrap-before, wrap-after](#valdef-wrap-before-auto), in § 5.3
  - [value for wrap-inside](#valdef-wrap-inside-auto), in § 5.2
- auto-phrase
  - [value for word-break](#valdef-word-break-auto-phrase), in § 6.1
  - [value for word-space-transform](#valdef-word-space-transform-auto-phrase), in § 2.2
- \<autospace\>
  - [(type)](#typedef-autospace), in § 8.4
  - [value for text-spacing](#valdef-text-spacing-autospace), in § 8.6
- avoid
  - [value for wrap-before, wrap-after](#valdef-wrap-before-avoid), in § 5.3
  - [value for wrap-inside](#valdef-wrap-inside-avoid), in § 5.2
- [avoid-flex](#valdef-wrap-before-avoid-flex), in § 5.3
- [avoid-line](#valdef-wrap-before-avoid-line), in § 5.3
- [balance](#valdef-text-wrap-style-balance), in § 5.4
- [bidi formatting characters](#bidi-formatting-characters), in § 4.3.1
- [block scripts](#block-scripts), in § Unnumbered section
- [break-all](#valdef-word-break-break-all), in § 6.1
- [break-spaces](#valdef-white-space-collapse-break-spaces), in § 4.1
- break-word
  - [value for overflow-wrap](#valdef-overflow-wrap-break-word), in § 6.4
  - [value for word-break](#valdef-word-break-break-word), in § 6.1
- [capitalize](#valdef-text-transform-capitalize), in § 2.1
- center
  - [value for text-align](#valdef-text-align-center), in § 7.1
  - [value for text-group-align](#valdef-text-group-align-center), in § 7.6
- [character](#character), in § 1.4
- [Chinese](#writing-system-chinese), in § Unnumbered section
- [clustered scripts](#clustered-scripts), in § Unnumbered section
- [collapse](#valdef-white-space-collapse-collapse), in § 4.1
- [collapsible](#collapsible-white-space), in § 4.3.1
- [collapsible white space](#collapsible-white-space), in § 4.3.1
- [column](#valdef-hyphenate-limit-lines-column), in § 6.3.5
- [conditionally hang](#conditionally-hang), in § 9.2
- [content language](#content-language), in § 1.3
- [content writing system](#content-writing-system), in § 1.3
- [cursive script](#cursive-script), in § Unnumbered section
- [detecting phrase boundaries](#phrase-boundary-detection), in § Unnumbered section
- [detecting word boundaries](#word-boundary-detection), in § Unnumbered section
- [detect phrase boundaries](#phrase-boundary-detection), in § Unnumbered section
- [detect word boundaries](#word-boundary-detection), in § Unnumbered section
- [discard](#valdef-white-space-collapse-discard), in § 4.1
- [discard-after](#valdef-white-space-trim-discard-after), in § 4.2
- [discard-before](#valdef-white-space-trim-discard-before), in § 4.2
- [discard-inner](#valdef-white-space-trim-discard-inner), in § 4.2
- [distribute](#valdef-text-justify-distribute), in § 7.5
- [document white space](#white-space), in § 4.3
- [document white space characters](#white-space), in § 4.3
- [each-line](#valdef-text-indent-each-line), in § 9.1
- [East Asian Width property](#unicode-east-asian-width), in § Unnumbered section
- end
  - [value for text-align](#valdef-text-align-end), in § 7.1
  - [value for text-group-align](#valdef-text-group-align-end), in § 7.6
- [expandable separators](#expandable-separators), in § 2.2
- [first](#valdef-hanging-punctuation-first), in § 9.2.1
- [flex](#valdef-wrap-before-flex), in § 5.3
- [forced line break](#forced-line-break), in § 5
- [force-end](#valdef-hanging-punctuation-force-end), in § 9.2.1
- [full-size](#kana-full-size), in § Unnumbered section
- [full-size kana](#kana-full-size), in § Unnumbered section
- [full-size-kana](#valdef-text-transform-full-size-kana), in § 2.1
- full-width
  - [definition of](#full-width), in § 2.1.1
  - [value for text-transform](#valdef-text-transform-full-width), in § 2.1
- [fullwidth closing punctuation](#fullwidth-closing-punctuation), in § 8.5.2
- [fullwidth colon punctuation](#fullwidth-colon-punctuation), in § 8.5.2
- [fullwidth dot punctuation](#fullwidth-dot-punctuation), in § 8.5.2
- [fullwidth middle dot punctuation](#fullwidth-middle-dot-punctuation), in § 8.5.2
- [fullwidth opening punctuation](#fullwidth-opening-punctuation), in § 8.5.2
- [General Category](#unicode-general-category), in § Unnumbered section
- [grapheme cluster](#grapheme-cluster), in § 1.4
- [group-align](#group-alignment), in § 7.6
- [group-aligned](#group-alignment), in § 7.6
- [group alignment](#group-alignment), in § 7.6
- [half-width](#half-width), in § 2.1.1
- [hang](#hang), in § 9.2
- [hanging](#valdef-text-indent-hanging), in § 9.1
- [hanging glyph](#hanging-glyph), in § 9.2
- [hanging-punctuation](#propdef-hanging-punctuation), in § 9.2.1
- [hyphenate](#hyphenate), in § 6.3.1
- [hyphenate-character](#propdef-hyphenate-character), in § 6.3.2
- [hyphenate-limit-chars](#propdef-hyphenate-limit-chars), in § 6.3.4
- [hyphenate-limit-last](#propdef-hyphenate-limit-last), in § 6.3.5
- [hyphenate-limit-lines](#propdef-hyphenate-limit-lines), in § 6.3.5
- [hyphenate-limit-zone](#propdef-hyphenate-limit-zone), in § 6.3.3
- [hyphenation](#hyphenate), in § 6.3.1
- [hyphenation opportunity](#hyphenation-opportunity), in § 6.3.1
- [hyphens](#propdef-hyphens), in § 6.3.1
- [ideograph-alpha](#valdef-text-autospace-ideograph-alpha), in § 8.4
- [ideographic-space](#valdef-word-space-transform-ideographic-space), in § 2.2
- [ideograph-numeric](#valdef-text-autospace-ideograph-numeric), in § 8.4
- [ideographs](#ideographs), in § 8.5.2
- [insert](#valdef-text-autospace-insert), in § 8.4
- [inter-character](#valdef-text-justify-inter-character), in § 7.5
- [inter-word](#valdef-text-justify-inter-word), in § 7.5
- [Japanese](#writing-system-japanese), in § Unnumbered section
- [justification opportunity](#justification-opportunity), in § 7.5.1
- [justify](#valdef-text-align-justify), in § 7.1
- [justify-all](#valdef-text-align-justify-all), in § 7.1
- [keep-all](#valdef-word-break-keep-all), in § 6.1
- [known](#writing-system-known), in § Unnumbered section
- [Korean](#writing-system-korean), in § Unnumbered section
- [last](#valdef-hanging-punctuation-last), in § 9.2.1
- left
  - [value for text-align](#valdef-text-align-left), in § 7.1
  - [value for text-group-align](#valdef-text-group-align-left), in § 7.6
- [\<length\>](#valdef-text-indent-length), in § 9.1
- \<length-percentage\>
  - [value for letter-spacing](#valdef-letter-spacing-length-percentage), in § 8.2
  - [value for word-spacing](#valdef-word-spacing-length-percentage), in § 8.1
- [letter](#letter), in § 1.4
- [letter-spacing](#propdef-letter-spacing), in § 8.2
- [line](#valdef-wrap-before-line), in § 5.3
- [line break](#line-break), in § 5
- [line-break](#propdef-line-break), in § 6.2
- [line breaking](#line-breaking-process), in § 5
- [line breaking process](#line-breaking-process), in § 5
- [line-padding](#propdef-line-padding), in § 8.3
- [loose](#valdef-line-break-loose), in § 6.2
- [lowercase](#valdef-text-transform-lowercase), in § 2.1
- manual
  - [value for hyphens](#valdef-hyphens-manual), in § 6.3.1
  - [value for word-break](#valdef-word-break-manual), in § 6.1
- [match-parent](#valdef-text-align-match-parent), in § 7.1
- [math-auto](#valdef-text-transform-math-auto), in § 2.1
- [no-autospace](#valdef-text-autospace-no-autospace), in § 8.4
- [no-compress](#valdef-text-justify-no-compress), in § 7.5
- none
  - [value for hanging-punctuation](#valdef-hanging-punctuation-none), in § 9.2.1
  - [value for hyphenate-limit-lines](#valdef-hyphenate-limit-lines-none), in § 6.3.5
  - [value for hyphens](#valdef-hyphens-none), in § 6.3.1
  - [value for text-group-align](#valdef-text-group-align-none), in § 7.6
  - [value for text-justify](#valdef-text-justify-none), in § 7.5
  - [value for text-spacing](#valdef-text-spacing-none), in § 8.6
  - [value for text-transform](#valdef-text-transform-none), in § 2.1
  - [value for word-space-transform](#valdef-word-space-transform-none), in § 2.2
- [non-ideographic letters](#non-ideographic-letters), in § 8.5.2
- [non-ideographic numerals](#non-ideographic-numerals), in § 8.5.2
- normal
  - [value for letter-spacing](#valdef-letter-spacing-normal), in § 8.2
  - [value for line-break](#valdef-line-break-normal), in § 6.2
  - [value for overflow-wrap](#valdef-overflow-wrap-normal), in § 6.4
  - [value for text-autospace](#valdef-text-autospace-normal), in § 8.4
  - [value for text-spacing-trim](#valdef-text-spacing-trim-normal), in § 8.5
  - [value for white-space](#valdef-white-space-normal), in § 3
  - [value for word-break](#valdef-word-break-normal), in § 6.1
  - [value for word-spacing](#valdef-word-spacing-normal), in § 8.1
- [nowrap](#valdef-text-wrap-mode-nowrap), in § 5.1
- [other space separators](#other-space-separators), in § 4.3
- [overflow-wrap](#propdef-overflow-wrap), in § 6.4
- [page](#valdef-hyphenate-limit-lines-page), in § 6.3.5
- [\<percentage\>](#valdef-text-indent-percentage), in § 9.1
- [phrase boundary](#phrase-boundary), in § Unnumbered section
- [phrase boundary detection](#phrase-boundary-detection), in § Unnumbered section
- [pre](#valdef-white-space-pre), in § 3
- [pre-line](#valdef-white-space-pre-line), in § 3
- [preserve](#valdef-white-space-collapse-preserve), in § 4.1
- [preserve-breaks](#valdef-white-space-collapse-preserve-breaks), in § 4.1
- [preserved](#preserved-white-space), in § 4.1
- [preserved white space](#preserved-white-space), in § 4.1
- [preserve-spaces](#valdef-white-space-collapse-preserve-spaces), in § 4.1
- [pretty](#valdef-text-wrap-style-pretty), in § 5.4
- [pre-wrap](#valdef-white-space-pre-wrap), in § 3
- [punctuation](#valdef-text-autospace-punctuation), in § 8.4
- [replace](#valdef-text-autospace-replace), in § 8.4
- right
  - [value for text-align](#valdef-text-align-right), in § 7.1
  - [value for text-group-align](#valdef-text-group-align-right), in § 7.6
- [ruby](#valdef-text-justify-ruby), in § 7.5
- [Script property](#unicode-script), in § Unnumbered section
- [segment break](#segment-break), in § 4
- [small](#kana-small), in § Unnumbered section
- [small kana](#kana-small), in § Unnumbered section
- [soft wrap break](#soft-wrap-break), in § 5
- [soft wrap opportunity](#soft-wrap-opportunity), in § 5
- [space](#valdef-word-space-transform-space), in § 2.2
- [space-all](#valdef-text-spacing-trim-space-all), in § 8.5
- [space-first](#valdef-text-spacing-trim-space-first), in § 8.5
- [spaces](#spaces), in § 4.3
- \<spacing-trim\>
  - [(type)](#typedef-spacing-trim), in § 8.5
  - [value for text-spacing](#valdef-text-spacing-spacing-trim), in § 8.6
- [spread](#valdef-hyphenate-limit-lines-spread), in § 6.3.5
- [stable](#valdef-text-wrap-style-stable), in § 5.4
- start
  - [value for text-align](#valdef-text-align-start), in § 7.1
  - [value for text-group-align](#valdef-text-group-align-start), in § 7.6
- [stop or comma](#stop-or-comma), in § 9.2.1
- [strict](#valdef-line-break-strict), in § 6.2
- \<string\>
  - [value for hyphenate-character](#valdef-hyphenate-character-string), in § 6.3.2
  - [value for text-align](#valdef-text-align-string), in § 7.1
- [tabs](#tabs), in § 4.3
- [tab size](#tab-size-dfn), in § 4.4
- [tab-size](#propdef-tab-size), in § 4.4
- [tab stop](#tab-stop), in § 4.3.2
- [text-align](#propdef-text-align), in § 7.1
- [text-align-all](#propdef-text-align-all), in § 7.3
- [text-align-last](#propdef-text-align-last), in § 7.4
- [text-autospace](#propdef-text-autospace), in § 8.4
- [text-group-align](#propdef-text-group-align), in § 7.6
- [text-indent](#propdef-text-indent), in § 9.1
- [text-justify](#propdef-text-justify), in § 7.5
- [text-spacing](#propdef-text-spacing), in § 8.6
- [text-spacing-trim](#propdef-text-spacing-trim), in § 8.5
- [text-transform](#propdef-text-transform), in § 2.1
- [text-wrap](#propdef-text-wrap), in § 5.5
- [text-wrap-mode](#propdef-text-wrap-mode), in § 5.1
- [text-wrap-style](#propdef-text-wrap-style), in § 5.4
- [tracking](#tracking), in § 8.2
- [trim-all](#valdef-text-spacing-trim-trim-all), in § 8.5
- [trim-both](#valdef-text-spacing-trim-trim-both), in § 8.5
- [trim-start](#valdef-text-spacing-trim-trim-start), in § 8.5
- [typographic character](#typographic-character-unit), in § 1.4
- [typographic character unit](#typographic-character-unit), in § 1.4
- [typographic letter unit](#typographic-letter-unit), in § 1.4
- [Unicode category](#unicode-general-category), in § Unnumbered section
- [Unicode East Asian Width](#unicode-east-asian-width), in § Unnumbered section
- [Unicode General Category](#unicode-general-category), in § Unnumbered section
- [Unicode Script](#unicode-script), in § Unnumbered section
- [Unicode Vertical Orientation](#unicode-vertical-orientation), in § Unnumbered section
- [unknown](#writing-system-known), in § Unnumbered section
- [uppercase](#valdef-text-transform-uppercase), in § 2.1
- [Vertical Orientation](#unicode-vertical-orientation), in § Unnumbered section
- [virtual expandable separator](#virtual-expandable-separator), in § 2.2
- [white space](#white-space), in § 4.3
- [white-space](#propdef-white-space), in § 3
- [white space characters](#white-space), in § 4.3
- [white-space-collapse](#propdef-white-space-collapse), in § 4.1
- [white-space-trim](#propdef-white-space-trim), in § 4.2
- [word boundary](#word-boundary), in § Unnumbered section
- [word boundary detection](#word-boundary-detection), in § Unnumbered section
- [word-break](#propdef-word-break), in § 6.1
- [word separator](#word-separator), in § 8.1
- [word-separator character](#word-separator), in § 8.1
- [word-space-transform](#propdef-word-space-transform), in § 2.2
- [word-spacing](#propdef-word-spacing), in § 8.1
- [word-wrap](#propdef-word-wrap), in § 6.4
- wrap
  - [definition of](#wrapping), in § 5
  - [value for text-wrap-mode](#valdef-text-wrap-mode-wrap), in § 5.1
- [wrap-after](#propdef-wrap-after), in § 5.3
- [wrap-before](#propdef-wrap-before), in § 5.3
- [wrap-inside](#propdef-wrap-inside), in § 5.2
- [wrapping](#wrapping), in § 5
- [writing system](#content-writing-system), in § 1.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="e1674793"></a>border
- \[CSS-BOX-4\] defines the following terms:
  - <a id="30e036e4"></a>border
  - <a id="253362bb"></a>margin
  - <a id="a3a070bd"></a>padding
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="7eb0e25a"></a>fragmentation context
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="515ba43c"></a>author origin
  - <a id="92922499"></a>declared value
  - <a id="d0dc95c3"></a>inherit
  - <a id="4905669f"></a>inherited value
  - <a id="6b448e93"></a>initial value
  - <a id="19fd0eed"></a>legacy name alias
  - <a id="7a6ac42f"></a>legacy value alias
  - <a id="36261173"></a>longhand
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
  - <a id="e308a45f"></a>user origin
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="f2938b9c"></a>atomic inline
  - <a id="f26fdc63"></a>block
  - <a id="f89f6132"></a>block box
  - <a id="05c40e8e"></a>block container
  - <a id="e4f1fc8b"></a>block formatting context
  - <a id="ecb27d09"></a>block-level
  - <a id="6b4fc208"></a>containing block
  - <a id="2ccfe434"></a>display
  - <a id="e8976716"></a>in-flow
  - <a id="b091c3a0"></a>independent formatting context
  - <a id="d41bfa8c"></a>inline box
  - <a id="c7eac2b9"></a>inline formatting context
  - <a id="4f918eb5"></a>inline-level
  - <a id="06fd3b4c"></a>out-of-flow
  - <a id="8b4f8a45"></a>root element
  - <a id="7ea71f53"></a>text sequence
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="693c2890"></a>font-feature-settings
  - <a id="297dfe3a"></a>font-size
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="ea4fcd78"></a>baseline
  - <a id="a9330658"></a>line box
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="00d2e365"></a>ink overflow
  - <a id="e3488cd0"></a>scrollable overflow
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="8c4b9d8f"></a>line-clamp
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="99a0ef70"></a>first formatted line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="3e692bb0"></a>bopomofo characters
  - <a id="8fd8cc1b"></a>ruby
  - <a id="42a3c4cb"></a>ruby annotation
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="de48a940"></a>inner size
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="59e3c405"></a>intrinsic size contribution
  - <a id="7cb1c6db"></a>intrinsic sizing
  - <a id="8a39af7f"></a>max-content size
  - <a id="6a444fd6"></a>min-content size
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="a5688715"></a>break-spaces
  - <a id="0b73a963"></a>nowrap
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="d73c993d"></a>\<integer\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="1d798932"></a>\<string\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="eefce2af"></a>em
  - <a id="3bafef5e"></a>{a,b}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="23cd6e75"></a>unicode-bidi
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="18bb33cd"></a>bidi paragraph
  - <a id="1b250473"></a>bidi-isolate
  - <a id="e112902f"></a>end
  - <a id="9933fc3f"></a>inline base direction
  - <a id="905ff85d"></a>inline end
  - <a id="e31b81f6"></a>inline start
  - <a id="82ddda8c"></a>inline-axis
  - <a id="5ca3e367"></a>inline-size
  - <a id="4f19c3e6"></a>line-left
  - <a id="10d0d189"></a>line-right
  - <a id="99a9e10b"></a>logical width
  - <a id="90c7548c"></a>start
  - <a id="9fc17679"></a>text-combine-upright
  - <a id="8664e85f"></a>text-orientation
  - <a id="cec0d4db"></a>upright
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="f1f51cad"></a>document language
- \[CSS3-FLEXBOX\] defines the following terms:
  - <a id="9f6d5ab0"></a>flex item
  - <a id="dcaa31ad"></a>flex line
  - <a id="97651ccf"></a>multi-line flex container
- \[CSSOM-1\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
  - <a id="fc19454a"></a>resolved value
- \[HTML\] defines the following terms:
  - <a id="d4dbbbf0"></a>language
  - <a id="fc736137"></a>textarea
  - <a id="90d63fe4"></a>wbr
- \[INFRA\] defines the following terms:
  - <a id="be82f2e7"></a>normalize newlines

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 1 April 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 1 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 3 September 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3-break"></a>\[CSS3-BREAK\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css3-flexbox"></a>\[CSS3-FLEXBOX\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mathml-core"></a>\[MATHML-CORE\]  
David Carlisle; Frédéric Wang. [MathML Core](https://www.w3.org/TR/mathml-core/). 27 November 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mathml-core&#x2F;](https://www.w3.org/TR/mathml-core/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-uax11"></a>\[UAX11\]  
Ken Lunde 小林劍󠄁. [East Asian Width](https://www.unicode.org/reports/tr11/tr11-41.html). 17 July 2023. Unicode Standard Annex \#11. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr11&#x2F;tr11-41&#x2E;html](https://www.unicode.org/reports/tr11/tr11-41.html)

<a id="biblio-uax14"></a>\[UAX14\]  
Robin Leroy. [Unicode Line Breaking Algorithm](https://www.unicode.org/reports/tr14/tr14-51.html). 15 August 2023. Unicode Standard Annex \#14. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr14&#x2F;tr14-51&#x2E;html](https://www.unicode.org/reports/tr14/tr14-51.html)

<a id="biblio-uax24"></a>\[UAX24\]  
Ken Whistler. [Unicode Script Property](https://www.unicode.org/reports/tr24/tr24-36.html). 14 August 2023. Unicode Standard Annex \#24. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr24&#x2F;tr24-36&#x2E;html](https://www.unicode.org/reports/tr24/tr24-36.html)

<a id="biblio-uax29"></a>\[UAX29\]  
Josh Hadley. [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/tr29-43.html). 16 August 2023. Unicode Standard Annex \#29. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;tr29-43&#x2E;html](https://www.unicode.org/reports/tr29/tr29-43.html)

<a id="biblio-uax44"></a>\[UAX44\]  
Ken Whistler. [Unicode Character Database](https://www.unicode.org/reports/tr44/tr44-32.html). 6 September 2023. Unicode Standard Annex \#44. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr44&#x2F;tr44-32&#x2E;html](https://www.unicode.org/reports/tr44/tr44-32.html)

<a id="biblio-uax50"></a>\[UAX50\]  
Ken Lunde 小林劍󠄁; Koji Ishii 石井宏治. [Unicode Vertical Text Layout](https://www.unicode.org/reports/tr50/tr50-29.html). 17 July 2023. Unicode Standard Annex \#50. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr50&#x2F;tr50-29&#x2E;html](https://www.unicode.org/reports/tr50/tr50-29.html)

<a id="biblio-uax9"></a>\[UAX9\]  
Manish Goregaokar मनीष गोरेगांवकर; Robin Leroy. [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/tr9-48.html). 15 August 2023. Unicode Standard Annex \#9. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr9&#x2F;tr9-48&#x2E;html](https://www.unicode.org/reports/tr9/tr9-48.html)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

### <a id="informative"></a>Informative References

<a id="biblio-bcp47"></a>\[BCP47\]  
A. Phillips, Ed.; M. Davis, Ed.. [Tags for Identifying Languages](https://www.rfc-editor.org/rfc/rfc5646). September 2009. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc5646](https://www.rfc-editor.org/rfc/rfc5646)

<a id="biblio-clreq"></a>\[CLREQ\]  
Bobby Tung; et al. [Requirements for Chinese Text Layout - 中文排版需求](https://www.w3.org/TR/clreq/). 1 November 2023. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;clreq&#x2F;](https://www.w3.org/TR/clreq/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 5 May 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-ilreq"></a>\[ILREQ\]  
Swaran Lata. [Indic Layout Requirements](https://www.w3.org/TR/ilreq/). 29 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;ilreq&#x2F;](https://www.w3.org/TR/ilreq/)

<a id="biblio-iso15924"></a>\[ISO15924\]  
Code for the representation of names of scripts. International Organization for Standardization. 1998. ISO 15924:1998. Draft International Standard

<a id="biblio-jis4051"></a>\[JIS4051\]  
Formatting rules for Japanese documents (『日本語文書の組版方法』). Japanese Standards Association. 2004. JIS X 4051:2004. In Japanese

<a id="biblio-jlreq"></a>\[JLREQ\]  
Hiroyuki Chiba; et al. [Requirements for Japanese Text Layout 日本語組版処理の要件(日本語版)](https://www.w3.org/TR/jlreq/). 11 August 2020. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;jlreq&#x2F;](https://www.w3.org/TR/jlreq/)

<a id="biblio-justify"></a>\[JUSTIFY\]  
Elika Etemad; Richard Ishida. [Approches to Full Justification](https://www.w3.org/International/articles/typography/justification). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;International&#x2F;articles&#x2F;typography&#x2F;justification](https://www.w3.org/International/articles/typography/justification)

<a id="biblio-l2-22-080r"></a>\[L2-22-080R\]  
Norbert Lindenberg; Elika Etemad; Vaishnavi Murthy Yerkadithaya. [Line breaking at orthographic syllable boundaries](https://www.unicode.org/L2/L2022/22080r-line-break-ortho-bnd.pdf). Proposal. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;L2&#x2F;L2022&#x2F;22080r-line-break-ortho-bnd&#x2E;pdf](https://www.unicode.org/L2/L2022/22080r-line-break-ortho-bnd.pdf)

<a id="biblio-rfc6919"></a>\[RFC6919\]  
R. Barnes; S. Kent; E. Rescorla. [Further Key Words for Use in RFCs to Indicate Requirement Levels](https://www.rfc-editor.org/rfc/rfc6919). 1 April 2013. Experimental. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc6919](https://www.rfc-editor.org/rfc/rfc6919)

<a id="biblio-typography"></a>\[TYPOGRAPHY\]  
Richard Ishida. [Language enablement index](https://www.w3.org/TR/typography/). 22 March 2024. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;typography&#x2F;](https://www.w3.org/TR/typography/)

<a id="biblio-xml10"></a>\[XML10\]  
Tim Bray; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/xml/). 26 November 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml&#x2F;](https://www.w3.org/TR/xml/)

<a id="biblio-zhmark"></a>\[ZHMARK\]  
General Rules for Punctuation (《标点符号用法》). 2011. GB/T 15834―2011. In Chinese.

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                                                     | Initial                   | Applies to                                                  | Inh.                      | %ages                                                  | Anim­ation type            | Canonical order | Com­puted value                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------|---------------------------|-------------------------------------------------------------|---------------------------|--------------------------------------------------------|---------------------------|-----------------|-------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-hanging-punctuation⑦"></a></span><a href="#propdef-hanging-punctuation">hanging-punctuation</a>&#xA;      </strong> | none \| \[ first \|\| \[ force-end \| allow-end \] \|\| last \]                                                           | none                      | text                                                        | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-hyphenate-character④"></a></span><a href="#propdef-hyphenate-character">hyphenate-character</a>&#xA;      </strong> | auto \| \<string\>                                                                                                        | auto                      | text                                                        | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-hyphenate-limit-chars②"></a></span><a href="#propdef-hyphenate-limit-chars">hyphenate-limit-chars</a>&#xA;      </strong> | \[ auto \| \<integer\> \]{1,3}                                                                                            | auto                      | text                                                        | yes                       | n/a                                                    | by computed value type    | per grammar     | three values, each either the auto keyword or an integer                      |
| <strong><span><a id="ref-for-propdef-hyphenate-limit-last③"></a></span><a href="#propdef-hyphenate-limit-last">hyphenate-limit-last</a>&#xA;      </strong> | none \| always \| column \| page \| spread                                                                                | none                      | block containers                                            | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-hyphenate-limit-lines②"></a></span><a href="#propdef-hyphenate-limit-lines">hyphenate-limit-lines</a>&#xA;      </strong> | no-limit \| \<integer\>                                                                                                   | no-limit                  | block containers                                            | yes                       | n/a                                                    | by computed value type    | per grammar     | specified keyword or integer                                                  |
| <strong><span><a id="ref-for-propdef-hyphenate-limit-zone③"></a></span><a href="#propdef-hyphenate-limit-zone">hyphenate-limit-zone</a>&#xA;      </strong> | \<length-percentage\>                                                                                                     | 0                         | block containers                                            | yes                       | refers to length of the line box                       | by computed value type    | per grammar     | computed \<length-percentage\> value                                          |
| <strong><span><a id="ref-for-propdef-hyphens⑤"></a></span><a href="#propdef-hyphens">hyphens</a>&#xA;      </strong> | none \| manual \| auto                                                                                                    | manual                    | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-letter-spacing②②"></a></span><a href="#propdef-letter-spacing">letter-spacing</a>&#xA;      </strong> | normal \| \<length-percentage\>                                                                                           | normal                    | inline boxes and text                                       | yes                       | relative to computed font-size, i.e. 1em               | by computed value type    | n/a             | an absolute length and/or a percentage                                        |
| <strong><span><a id="ref-for-propdef-line-break①③"></a></span><a href="#propdef-line-break">line-break</a>&#xA;      </strong> | auto \| loose \| normal \| strict \| anywhere                                                                             | auto                      | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-line-padding④"></a></span><a href="#propdef-line-padding">line-padding</a>&#xA;      </strong> | \<length\>                                                                                                                | 0                         | inline boxes                                                | yes                       | N/A                                                    | by computed value type    | per grammar     | absolute length                                                               |
| <strong><span><a id="ref-for-propdef-overflow-wrap①③"></a></span><a href="#propdef-overflow-wrap">overflow-wrap</a>&#xA;      </strong> | normal \| break-word \| anywhere                                                                                          | normal                    | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-tab-size②"></a></span><a href="#propdef-tab-size">tab-size</a>&#xA;      </strong> | \<number \[0,∞\]\> \| \<length \[0,∞\]\>                                                                                  | 8                         | text                                                        | yes                       | n/a                                                    | by computed value type    | n/a             | the specified number or absolute length                                       |
| <strong><span><a id="ref-for-propdef-text-align①④"></a></span><a href="#propdef-text-align">text-align</a>&#xA;      </strong> | start \| end \| left \| right \| center \| \<string\> \| justify \| match-parent \| justify-all                           | start                     | block containers                                            | yes                       | see individual properties                              | discrete                  | n/a             | see individual properties                                                     |
| <strong><span><a id="ref-for-propdef-text-align-all⑧"></a></span><a href="#propdef-text-align-all">text-align-all</a>&#xA;      </strong> | start \| end \| left \| right \| center \| \<string\> \| justify \| match-parent                                          | start                     | block containers                                            | yes                       | n/a                                                    | discrete                  | n/a             | keyword as specified, except for match-parent which computes as defined above |
| <strong><span><a id="ref-for-propdef-text-align-last①⓪"></a></span><a href="#propdef-text-align-last">text-align-last</a>&#xA;      </strong> | auto \| start \| end \| left \| right \| center \| justify \| match-parent                                                | auto                      | block containers                                            | yes                       | n/a                                                    | discrete                  | n/a             | keyword as specified, except for match-parent which computes as defined above |
| <strong><span><a id="ref-for-propdef-text-autospace①⓪"></a></span><a href="#propdef-text-autospace">text-autospace</a>&#xA;      </strong> | normal \| \<autospace\> \| auto                                                                                           | normal                    | text                                                        | yes                       | N/A                                                    | discrete                  | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-text-group-align②"></a></span><a href="#propdef-text-group-align">text-group-align</a>&#xA;      </strong> | none \| start \| end \| left \| right \| center                                                                           | none                      | block containers                                            | no                        | N/A                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-text-indent①⓪"></a></span><a href="#propdef-text-indent">text-indent</a>&#xA;      </strong> | \[ \<length-percentage\> \] &#x26;&#x26; hanging? &#x26;&#x26; each-line?             | 0                         | block containers                                            | yes                       | refers to block container’s own inline-axis inner size | by computed value type    | per grammar     | computed \<length-percentage\> value, plus any specified keywords             |
| <strong><span><a id="ref-for-propdef-text-justify①①"></a></span><a href="#propdef-text-justify">text-justify</a>&#xA;      </strong> | \[ auto \| none \| inter-word \| inter-character \| ruby \] \|\| no-compress                                              | auto                      | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword (except for the distribute legacy value)                    |
| <strong><span><a id="ref-for-propdef-text-spacing⑨"></a></span><a href="#propdef-text-spacing">text-spacing</a>&#xA;      </strong> | none \| auto \| \<spacing-trim\> \|\| \<autospace\>                                                                       | see individual properties | text                                                        | yes                       | N/A                                                    | discrete                  | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-text-spacing-trim①⑥"></a></span><a href="#propdef-text-spacing-trim">text-spacing-trim</a>&#xA;      </strong> | \<spacing-trim\> \| auto                                                                                                  | normal                    | text                                                        | yes                       | N/A                                                    | discrete                  | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-text-transform⑨"></a></span><a href="#propdef-text-transform">text-transform</a>&#xA;      </strong> | none \| \[capitalize \| uppercase \| lowercase \] \|\| full-width \|\| full-size-kana \| math-auto                        | none                      | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-text-wrap⑤"></a></span><a href="#propdef-text-wrap">text-wrap</a>&#xA;      </strong> | \<'text-wrap-mode'\> \|\| \<'text-wrap-style'\>                                                                           | wrap                      | see individual properties                                   | see individual properties | see individual properties                              | see individual properties | per grammar     | see individual properties                                                     |
| <strong><span><a id="ref-for-propdef-text-wrap-mode①④"></a></span><a href="#propdef-text-wrap-mode">text-wrap-mode</a>&#xA;      </strong> | wrap \| nowrap                                                                                                            | wrap                      | text                                                        | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-text-wrap-style⑨"></a></span><a href="#propdef-text-wrap-style">text-wrap-style</a>&#xA;      </strong> | auto \| balance \| stable \| pretty                                                                                       | auto                      | block containers hat establish an inline formatting context | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-white-space①⑤"></a></span><a href="#propdef-white-space">white-space</a>&#xA;      </strong> | normal \| pre \| pre-wrap \| pre-line \| \<'white-space-collapse'\> \|\| \<'text-wrap-mode'\> \|\| \<'white-space-trim'\> | normal                    | text                                                        | individual properties     | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-white-space-collapse①⑨"></a></span><a href="#propdef-white-space-collapse">white-space-collapse</a>&#xA;      </strong> | collapse \| discard \| preserve \| preserve-breaks \| preserve-spaces \| break-spaces                                     | collapse                  | text                                                        | yes                       | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-white-space-trim①②"></a></span><a href="#propdef-white-space-trim">white-space-trim</a>&#xA;      </strong> | none \| discard-before \|\| discard-after \|\| discard-inner                                                              | none                      | inline boxes and block containers                           | no                        | n/a                                                    | discrete                  | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-word-break③④"></a></span><a href="#propdef-word-break">word-break</a>&#xA;      </strong> | normal \| break-all \| keep-all \| manual \| auto-phrase \| break-word                                                    | normal                    | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-word-space-transform⑧"></a></span><a href="#propdef-word-space-transform">word-space-transform</a>&#xA;      </strong> | none \| \[ space \| ideographic-space \] &#x26;&#x26; auto-phrase?                                      | none                      | text                                                        | yes                       | N/A                                                    | discrete                  | per grammar     | as specified                                                                  |
| <strong><span><a id="ref-for-propdef-word-spacing①①"></a></span><a href="#propdef-word-spacing">word-spacing</a>&#xA;      </strong> | normal \| \<length-percentage\>                                                                                           | normal                    | text                                                        | yes                       | relative to computed font-size, i.e. 1em               | by computed value type    | n/a             | an absolute length and/or a percentage                                        |
| <strong><span><a id="ref-for-propdef-word-wrap②"></a></span><a href="#propdef-word-wrap">word-wrap</a>&#xA;      </strong> | normal \| break-word \| anywhere                                                                                          | normal                    | text                                                        | yes                       | n/a                                                    | discrete                  | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-wrap-after③"></a></span><a href="#propdef-wrap-after">wrap-after</a>&#xA;      </strong> | auto \| avoid \| avoid-line \| avoid-flex \| line \| flex                                                                 | auto                      | inline-level boxes and flex items                           | no                        | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-wrap-before③"></a></span><a href="#propdef-wrap-before">wrap-before</a>&#xA;      </strong> | auto \| avoid \| avoid-line \| avoid-flex \| line \| flex                                                                 | auto                      | inline-level boxes and flex items                           | no                        | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-wrap-inside③"></a></span><a href="#propdef-wrap-inside">wrap-inside</a>&#xA;      </strong> | auto \| avoid                                                                                                             | auto                      | inline boxes                                                | no                        | n/a                                                    | discrete                  | per grammar     | specified keyword                                                             |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is still under discussion and may change in future drafts. [↵](#issue-257c8a37)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Does this preserve line break opportunities or no? Do we need a distinct "hide" value? If it preserves line break opportunities, maybe it should be replaced with a [word-space-transform](#propdef-word-space-transform) value? [↵](#issue-e3dd9c3a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What should happen here for [white-space-collapse: preserve-spaces](#propdef-white-space-collapse)? [↵](#issue-a72a5cd8)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should we define this for Level 4? [↵](#issue-c0106ca7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The name of this property is a placeholder, pending the CSSWG finding a better name. [↵](#issue-3d83054d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> See [thread](https://www.w3.org/mid/0BD85DFF-A147-44EF-B18A-FF03C3D67EF0@verou.me). Issue is about requiring a minimum length for lines. Common measures seem to be
>
> - At least as long as the text-indent.
> - At least X characters.
> - Percentage-based.
>
> Suggestion for value space is match-indent \| \<length\> \| \<percentage\> (with Xch given as an example to make that use case clear). Alternately [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) could actually count the characters.
>
> It’s unclear how this would interact with text balancing (above); one earlier proposal had them be the same property (with 100% meaning full balancing).
>
> People have requested word-based limits, but since this is really dependent on the length of the word, character-based is better.
>
> [↵](#last-line-limits)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> alternatively, this value could be based on [keep-all](#valdef-word-break-keep-all) rather than [normal](#valdef-word-break-normal). Yet another variant is to merge this behavior with keep-all. [↵](#issue-35b47d14)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is [hyphenate-limit-zone](#propdef-hyphenate-limit-zone) a good name? Comments/suggestions? [↵](#issue-9a19edc4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need to add a `none` value to [overflow-wrap](#propdef-overflow-wrap) to opt out of relaxing the keep-all' and auto-phrase'' restrictions as allowed by overflow-wrap: normal? [↵](#issue-a2390e95)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is this intended to say that it’s the centers of the alignment characters that should be aligned? It’s not clear that’s what it says, but that (or a different behavior) needs to be specified, to describe what happens when different occurrences of the alignment character are in different fonts. (Further, is that the intended behavior? Probably the most significant use case to consider is bold vs. non-bold text, which only varies slightly in width.) \[[feedback](https://lists.w3.org/Archives/Public/www-style/2016Jan/0233.html)\] \[minutes face-to-face 2016-02-02 10:00 AM\] [↵](#issue-97356692)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This needs to specify what text is searched for the alignment character. Is it only in-flow text whose containing block is the cell? Or is text within any in-flow descendants in the block formatting context established by the cell considered? If so, is it considered only as long as its [text-align](#propdef-text-align) property is consistent with the cell’s? (Consistent in the alignment character, or fully consistent?) [↵](#issue-f823e64b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This behavior of aligning as though the alignment character had been inserted at the end of the contents of the cell, combined with center-of-character alignment, will produce gaps on the end-side of lines that are alone on a line with [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) text-alignment, when none of the lines of the column has the alignment character, or, more importantly, when some of the lines do have the alignment character, but the column is not laid out at its max-content width. This is probably undesirable. [↵](#issue-a4eeca60)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> When the alignment character is inserted at the end of the contents, which font is used? (In particular, if the alignment character might be within a descendant block, is it the font of the block or the font of the table cell? Or if the insertion is at a forced break within an inline, does it use the font of the inline or the font of the block or cell?) [↵](#issue-c1a95522)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This should have a formal definition of how character alignment affects the min-content and max-content intrinsic widths (of table columns and all content that can be inside table columns). Max-content intrinsic widths need to be split into three numbers (assuming that it’s the centers of the alignment character that are aligned): one for widths without alignment characters, one for widths on the inline-start side of the center of the alignment character, one for widths on the inline-end side of the center of the alignment character. This operates based on all segments of text between forced breaks for max-content widths. For min-content widths, segments of text between forced breaks that contain optional breaks within them should clearly contribute only to the without-alignment-character width. However, it’s less clear whether all min-content widths should work this way, or whether segments between forced breaks that do not have optional breaks (and perhaps only those that actually contain the alignment character) should contribute to start-side-of-alignment-character and end-side-of-alignment-character min-content widths instead; this choice is a tradeoff between the meaning of min-content sizing of a table meaning the narrowest reasonable size versus honoring alignment characters in more cases. Another option might be to use whether line-breaking of optional breaks is allowed as a control for which behavior to use. [↵](#issue-f3db70cb)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Formally defining the intrinsic width contributions of column-spanning cells with [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) values of [text-align](#propdef-text-align) is a complicated (although straightforward) extension of the decisions made for intrinsic width contributions of non-column-spanning cells; this should also be formally defined. Contributions end up being made to the split intrinsic widths of the startmost or endmost column (whichever is used for alignment), and to the without-alignment-character intrinsic widths of the other spanned columns. [↵](#issue-b38ae96a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This keyword used to be part of [text-spacing](#propdef-text-spacing); it might need renaming to be more specific now that it’s here, as it implies that e.g. U+0020 cannot be compressed. [\[Issue \#7079\]](https://github.com/w3c/csswg-drafts/issues/7079) [↵](#issue-a6b9619a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> A variant of this property is inherited, and applies on each block container individually, only affecting the line boxes that are direct children of that block. This is less useful, but probably easier to implement. [↵](#issue-5a886932)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Somehow also moving the floats that originate in the same block container by the same amount would make things line up more nicely, which would be especially valuable in CJK layout. Exactly how that works, and how it interacts with intruding floats from ancestor elements is left as an exercise for the reader. [↵](#issue-6a6c35c0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need [auto](#valdef-text-spacing-trim-auto)? It would be weird for the author to choose platform-dependent behavior at the start of the first line, and it should otherwise use [trim-both](#valdef-text-spacing-trim-trim-both). [↵](#issue-366d9875)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Classes and Unicode code points are under review, and at least some changes are needed to accommodate more recent additions to Unicode. [\[Issue \#9503\]](https://github.com/w3c/csswg-drafts/issues/9503) [↵](#issue-fbe5fc02)
