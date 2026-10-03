Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Text Module Level 3](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Text Module Level 3

Source snapshot: https://www.w3.org/TR/2026/CRD-css-text-3-20260814/

Snapshot SHA-256: 17c26f1b41455947f106dac81736944b12da328ee4892c7c5b601f5b65ced55a

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 22 source tables are presented as readable Markdown tables or explicit labeled layouts: 18 ordinary table conversions, 2 complex-table layouts, 2 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Live HTML/CSS demonstrations are represented by static source code and text, not equivalent browser appearance. Incidental whitespace in sample-display elements may collapse as in HTML; exact source markup is retained, and true preformatted/code blocks stay literal.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Text Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module defines properties for text manipulation and specifies their processing model. It covers line breaking, justification and alignment, white space handling, and text transformation.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-text” in the title, like this: “\[css-text\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-text%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-text-transform"></a>

  <a id="ref-for-valdef-text-transform-full-width"></a>

  the [full-width](#valdef-text-transform-full-width) value of [text-transform](#propdef-text-transform)

- <a id="ref-for-propdef-text-transform①"></a>

  <a id="ref-for-valdef-text-transform-full-size-kana"></a>

  the [full-size-kana](#valdef-text-transform-full-size-kana) value of [text-transform](#propdef-text-transform)

- <a id="ref-for-propdef-tab-size"></a>

  the \<length\> values of the [tab-size](#propdef-tab-size) property

- <a id="ref-for-propdef-text-justify"></a>

  the [text-justify](#propdef-text-justify) property

- <a id="ref-for-propdef-hanging-punctuation"></a>

  the [hanging-punctuation](#propdef-hanging-punctuation) property

- Writing-system specific adjustments to line-breaking

- Trimming trailing Ogham space marks

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

This module describes the typesetting controls of CSS; that is, the features of CSS that control the translation of source text to formatted, line-wrapped text. Various CSS properties provide control over [case transformation](#transforming), [white space collapsing](#white-space-processing), [text wrapping](#white-space-property), [line breaking rules](#line-breaking) and [hyphenation](#hyphenation), [alignment and justification](#justification), [spacing](#spacing), and [indentation](#edge-effects).

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

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="languages"></a>1.3.  Languages and Typesetting

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> <strong>Authors should accurately language-tag their content
	for the best typographic behavior.</strong>

<a id="ref-for-content-language"></a>

Many typographic effects vary by linguistic context. Language and writing system conventions can affect line breaking, hyphenation, justification, glyph selection, and many other typographic effects. <strong>In CSS, language-specific typographic tailorings
	are only applied when the <a href="#content-language">content language</a> is known (declared).</strong> Therefore, higher quality typography requires authors to communicate to the UA the correct linguistic context of the text in the document.

<a id="ref-for-doclanguage"></a>

<a id="ref-for-content-language①"></a>

The <a id="content-language"></a>content language of an element is the (human) language the element is declared to be in, according to the rules of the [document language](https://www.w3.org/TR/CSS2/conform.html#doclanguage). Note that it is possible for the [content language](#content-language) of an element to be unknown—​e.g. untagged content, or content in a <a id="ref-for-doclanguage①"></a>document language that does not have a language-tagging facility, is considered to have an unknown <a id="ref-for-content-language②"></a>content language.

<a id="ref-for-content-language③"></a>

<a id="ref-for-language"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors can declare the [content language](#content-language) using the global `lang` attribute in HTML or the universal `xml:lang` attribute in XML. See the [rules for determining the content language of an HTML element](https://html.spec.whatwg.org/multipage/dom.html#language) in HTML, and the [rules for determining the content language of an XML element](https://www.w3.org/TR/xml/#sec-lang-tag) in XML 1.0. [\[HTML\]](#biblio-html) [\[XML10\]](#biblio-xml10)

<a id="ref-for-content-language④"></a>

<a id="ref-for-doclanguage②"></a>

The [content language](#content-language) an element is declared to be in also identifies the specific written form of that language used in that element, known as the <a id="content-writing-system"></a>content writing system. Depending on the [document language](https://www.w3.org/TR/CSS2/conform.html#doclanguage)’s facilities for identifying the <a id="ref-for-content-language⑤"></a>content language, this information can be explicit or implied. See the normative [Appendix F: Identifying the Content Writing System](#script-tagging).

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

For the purpose of determining adjacency for text processing (such as white space processing, text transformation, line-breaking, etc.), and thus in general within this specification, intervening [inline box](https://www.w3.org/TR/css-display-4/#inline-box) boundaries and [out-of-flow](https://www.w3.org/TR/css-display-4/#out-of-flow) elements must be ignored. With respect to text shaping, however, see [§ 7.3 Shaping Across Element Boundaries](#boundary-shaping).

## <a id="transforming"></a>2.  Transforming Text

<a id="ref-for-propdef-text-transform②"></a>

### <a id="text-transform-property"></a>2.1. <a id="caps-prop"></a><a id="text-transform"></a> Case Transforms: the [text-transform](#propdef-text-transform) property

| Field               | Definition                                                                                                                                                                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-transform"></a>text-transform                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[capitalize <a id="ref-for-comb-one①"></a>\| uppercase <a id="ref-for-comb-one②"></a>\| lowercase \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) full-width <a id="ref-for-comb-any①"></a>\|\| full-size-kana |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                             |

This property transforms text for styling purposes. It has no effect on the underlying content, and must not affect the content of a plain text copy &#x26; paste operation.

<a id="ref-for-propdef-text-transform③"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors must not rely on <a href="#propdef-text-transform">text-transform</a> for semantic purposes;
	rather the correct casing and semantics should be encoded
	in the source document text and markup.</strong>

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

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5b7dad2f"></a> The following example converts the ASCII characters used in abbreviations in Japanese text to their full-width variants so that they lay out and line break like ideographs:
>
> ```text
> abbr:lang(ja) { text-transform: full-width; }
> ```
<a id="ref-for-propdef-text-transform④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The purpose of [text-transform](#propdef-text-transform) is to allow for presentational casing transformations without affecting the semantics of the document. Note in particular that <a id="ref-for-propdef-text-transform⑤"></a>text-transform casing operations are lossy, and can distort the meaning of a text. While accessibility interfaces may wish to convey the apparent casing of the rendered text to the user, the transformed text cannot be relied on to accurately represent the underlying meaning of the document.

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
> <a id="ref-for-propdef-text-transform⑥"></a>
>
> For example, if [text-transform: full-size-kana](#propdef-text-transform) were applied to the following source, the annotation would read “じゆう” (jiyū), which means “liberty”, instead of “じゅう” (jū), which means “ten”, the correct reading and meaning for the annotated “十”.
>
> ```text
> <ruby>十<rt>じゅう</ruby>
> ```
#### <a id="text-transform-mapping"></a>2.1.1.  Mapping Rules

<a id="ref-for-valdef-text-transform-capitalize"></a>

<a id="ref-for-out-of-flow①"></a>

<a id="ref-for-inline-box①"></a>

<a id="ref-for-propdef-text-transform⑦"></a>

For [capitalize](#valdef-text-transform-capitalize), what constitutes a “word“ is UA-dependent; [\[UAX29\]](#biblio-uax29) is suggested (but not required) for determining such word boundaries. [Out-of-flow boxes](https://www.w3.org/TR/css-display-3/#out-of-flow) and [inline box](https://www.w3.org/TR/css-display-4/#inline-box) boundaries must not introduce a [text-transform](#propdef-text-transform) word boundary and must be ignored when determining such word boundaries.

<a id="ref-for-valdef-text-transform-capitalize①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors cannot depend on [capitalize](#valdef-text-transform-capitalize) to follow language-specific titlecasing conventions (such as skipping articles in English).

<a id="ref-for-content-language⑥"></a>

<a id="ref-for-doclanguage③"></a>

The UA must use the full case mappings for Unicode characters, including any conditional casing rules, as defined in the Default Case Algorithms section of The Unicode Standard. [\[UNICODE\]](#biblio-unicode) If (and only if) the [content language](#content-language) of the element is, according to the rules of the [document language](https://www.w3.org/TR/CSS2/conform.html#doclanguage), known, then any appropriate language-specific rules must be applied as well. These minimally include, but are not limited to, the language-specific rules in Unicode’s [SpecialCasing.txt](http://www.unicode.org/Public/UNIDATA/SpecialCasing.txt).

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

#### <a id="text-transform-order"></a>2.1.2.  Order of Operations

When multiple values are specified and therefore multiple transformations need to be applied, they are applied in the following order:

1.  <a id="ref-for-valdef-text-transform-lowercase"></a>

    <a id="ref-for-valdef-text-transform-uppercase"></a>

    <a id="ref-for-valdef-text-transform-capitalize②"></a>

    [capitalize](#valdef-text-transform-capitalize), [uppercase](#valdef-text-transform-uppercase), and [lowercase](#valdef-text-transform-lowercase)

2.  <a id="ref-for-valdef-text-transform-full-width①"></a>

    [full-width](#valdef-text-transform-full-width)

3.  <a id="ref-for-valdef-text-transform-full-size-kana①"></a>

    [full-size-kana](#valdef-text-transform-full-size-kana)

<a id="ref-for-valdef-text-transform-full-width②"></a>

<a id="ref-for-preserved-white-space"></a>

<a id="ref-for-white-space"></a>

Text transformation happens after [§ 4.1.1 Phase I: Collapsing and Transformation](#white-space-phase-1) but before [§ 4.1.2 Phase II: Trimming and Positioning](#white-space-phase-2). This means that [full-width](#valdef-text-transform-full-width) only transforms spaces (U+0020) to U+3000 IDEOGRAPHIC SPACE within [preserved](#preserved-white-space) [white space](#white-space).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As defined in [Appendix A: Text Processing Order of Operations](#order), transforming text affects line-breaking and other formatting operations.

<a id="ref-for-propdef-white-space"></a>

## <a id="white-space-property"></a>3. <a id="text-wrap"></a><a id="white-space-collapsing"></a> White Space and Wrapping: the [white-space](#propdef-white-space) property

| Field               | Definition                                                                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-white-space"></a>white-space                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) pre <a id="ref-for-comb-one④"></a>\| nowrap <a id="ref-for-comb-one⑤"></a>\| pre-wrap <a id="ref-for-comb-one⑥"></a>\| break-spaces <a id="ref-for-comb-one⑦"></a>\| pre-line |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                       |

This property specifies two things:

- <a id="ref-for-white-space①"></a>

  whether and how [white space](#white-space) is collapsed

- <a id="ref-for-soft-wrap-opportunity"></a>

  <a id="ref-for-wrapping"></a>

  whether lines may [wrap](#wrapping) at unforced [soft wrap opportunities](#soft-wrap-opportunity)

Values have the following meanings, which must be interpreted according to the [White Space Processing](#white-space-rules) and [Line Breaking](#line-breaking) rules:

<a id="valdef-white-space-normal"></a>normal  
<a id="ref-for-inline-axis"></a>

<a id="ref-for-soft-wrap-opportunity①"></a>

<a id="ref-for-wrapping①"></a>

<a id="ref-for-white-space②"></a>

This value directs user agents to collapse sequences of [white space](#white-space) into a single character (or [in some cases](#line-break-transform), no character). Lines may [wrap](#wrapping) at allowed [soft wrap opportunities](#soft-wrap-opportunity), as determined by the line-breaking rules in effect, in order to minimize [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) overflow.

<a id="valdef-white-space-pre"></a>pre  
<a id="ref-for-forced-line-break"></a>

<a id="ref-for-segment-break"></a>

<a id="ref-for-white-space③"></a>

This value prevents user agents from collapsing sequences of [white space](#white-space). [Segment breaks](#segment-break) such as line feeds are preserved as [forced line breaks](#forced-line-break). Lines only break at <a id="ref-for-forced-line-break①"></a>forced line breaks; content that does not fit within the block container overflows it.

<a id="valdef-white-space-nowrap"></a>nowrap  
<a id="ref-for-wrapping②"></a>

<a id="ref-for-valdef-white-space-pre"></a>

<a id="ref-for-white-space④"></a>

<a id="ref-for-valdef-white-space-normal"></a>

Like [normal](#valdef-white-space-normal), this value collapses [white space](#white-space); but like [pre](#valdef-white-space-pre), it does not allow [wrapping](#wrapping).

<a id="valdef-white-space-pre-wrap"></a>pre-wrap  
<a id="ref-for-wrapping③"></a>

<a id="ref-for-valdef-white-space-normal①"></a>

<a id="ref-for-white-space⑤"></a>

<a id="ref-for-valdef-white-space-pre①"></a>

Like [pre](#valdef-white-space-pre), this value preserves [white space](#white-space); but like [normal](#valdef-white-space-normal), it allows [wrapping](#wrapping).

<a id="valdef-white-space-break-spaces"></a>break-spaces  
<a id="ref-for-valdef-white-space-pre-wrap"></a>

The behavior is identical to that of [pre-wrap](#valdef-white-space-pre-wrap), except that:

- <a id="ref-for-other-space-separators"></a>

  <a id="ref-for-white-space⑥"></a>

  <a id="ref-for-preserved-white-space①"></a>

  Any sequence of [preserved](#preserved-white-space) [white space](#white-space) or [other space separators](#other-space-separators) always takes up space, including at the end of the line.

- <a id="ref-for-other-space-separators①"></a>

  <a id="ref-for-white-space⑦"></a>

  <a id="ref-for-preserved-white-space②"></a>

  <a id="ref-for-soft-wrap-opportunity②"></a>

  A [soft wrap opportunity](#soft-wrap-opportunity) exists after every [preserved](#preserved-white-space) [white space](#white-space) character and after every [other space separator](#other-space-separators) (including between adjacent spaces).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value does not guarantee that there will never be any overflow due to white space: for example, if the line length is so short that even a single white space character does not fit, overflow is unavoidable.

<a id="valdef-white-space-pre-line"></a>pre-line  
<a id="ref-for-forced-line-break②"></a>

<a id="ref-for-segment-break①"></a>

<a id="ref-for-wrapping④"></a>

<a id="ref-for-white-space⑧"></a>

<a id="ref-for-valdef-white-space-normal②"></a>

Like [normal](#valdef-white-space-normal), this value collapses consecutive [white space characters](#white-space) and allows [wrapping](#wrapping), but it preserves [segment breaks](#segment-break) in the source as [forced line breaks](#forced-line-break).

<a id="ref-for-white-space⑨"></a>

[White space](#white-space) that was not removed or collapsed due to white space processing is called <a id="preserved-white-space"></a>preserved white space.

<a id="ref-for-preserved-white-space③"></a>

<a id="ref-for-other-space-separators②"></a>

<a id="ref-for-hang"></a>

<a id="ref-for-intrinsic-sizing"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases, [preserved white space](#preserved-white-space) and [other space separators](#other-space-separators) can [hang](#hang) when at the end of the line; this can affect whether they are measured for [intrinsic sizing](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizing).

<a id="ref-for-propdef-white-space①"></a>

The following informative table summarizes the behavior of various [white-space](#propdef-white-space) values:

|                     | New Lines | Spaces and Tabs | Text Wrapping | <a id="ref-for-spaces"></a>End-of-line [spaces](#spaces) | <a id="ref-for-other-space-separators③"></a>End-of-line [other space separators](#other-space-separators) |
|---------------------|-----------|-----------------|---------------|--------------------------------------------------|----------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-valdef-white-space-normal③"></a></span><a href="#valdef-white-space-normal">normal</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Collapse  | Collapse        | Wrap          | Remove                                           | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre②"></a></span><a href="#valdef-white-space-pre">pre</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Preserve  | Preserve        | No wrap       | Preserve                                         | No wrap                                                                          |
| <strong><span><a id="ref-for-valdef-white-space-nowrap"></a></span><a href="#valdef-white-space-nowrap">nowrap</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Collapse  | Collapse        | No wrap       | Remove                                           | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre-wrap①"></a></span><a href="#valdef-white-space-pre-wrap">pre-wrap</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Preserve  | Preserve        | Wrap          | Hang                                             | Hang                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-break-spaces"></a></span><a href="#valdef-white-space-break-spaces">break-spaces</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Preserve  | Preserve        | Wrap          | Wrap                                             | Wrap                                                                             |
| <strong><span><a id="ref-for-valdef-white-space-pre-line"></a></span><a href="#valdef-white-space-pre-line">pre-line</a>&#xA;&#x9;&#x9;&#x9;&#x9;&#xA;      </strong> | Preserve  | Collapse        | Wrap          | Remove                                           | Hang                                                                             |

<a id="ref-for-white-space①⓪"></a>

See [White Space Processing Rules](#white-space-processing) for details on how [white space](#white-space) collapses.

See [Line Breaking](#line-breaking) for details on wrapping behavior.

## <a id="white-space-processing"></a>4.  White Space Processing &#x26; Control Characters

<a id="ref-for-white-space①①"></a>

<a id="ref-for-tabs"></a>

<a id="ref-for-spaces①"></a>

<a id="ref-for-propdef-white-space②"></a>

The source text of a document often contains formatting that is not relevant to the final rendering: for example, [breaking the source into segments](https://rhodesmill.org/brandon/2012/one-sentence-per-line/) (lines) for ease of editing or adding [white space characters](#white-space) such as [tabs](#tabs) and [spaces](#spaces) to indent the source code. CSS white space processing allows the author to control interpretation of such formatting: to preserve or collapse it away when rendering the document. White space processing in CSS (which is controlled with the [white-space](#propdef-white-space) property) interprets <a id="ref-for-white-space①②"></a>white space characters only for rendering: it has no effect on the underlying document data.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the document language, segments can be separated by a particular newline sequence (such as a line feed or CRLF pair), or delimited by some other mechanism, such as the SGML `RECORD-START` and `RECORD-END` tokens.

<a id="ref-for-propdef-white-space③"></a>

<a id="segment-normalization"></a> For CSS processing, each document language–defined “segment break” or “newline sequence”—​or if none are defined, each line feed (U+000A)—​in the text is treated as a <a id="segment-break"></a>segment break, which is then interpreted for rendering as specified by the [white-space](#propdef-white-space) property.

<a id="ref-for-normalize-newlines"></a>

<a id="ref-for-segment-break②"></a>

In the case of HTML, [newlines](https://html.spec.whatwg.org/multipage/syntax.html#newlines) are [normalized](https://infra.spec.whatwg.org/#normalize-newlines) to line feed characters (U+000A) for representation in the DOM, so when an HTML document is represented as a DOM tree each line feed (U+000A) is treated as a [segment break](#segment-break). [\[HTML\]](#biblio-html) [\[DOM\]](#biblio-dom)

<a id="ref-for-segment-break③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In most common CSS implementations, HTML does not get styled directly. Instead, it is processed into a DOM tree, which is then styled. Unlike HTML, the DOM does not give any particular meaning to carriage returns (U+000D), so they are not treated as [segment breaks](#segment-break). If carriage returns (U+000D) are inserted into the DOM by means other than HTML parsing, they then get treated as defined below.

<a id="ref-for-segment-break④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A document parser might not only normalize any [segment breaks](#segment-break), but also collapse other space characters or otherwise process white space according to markup rules. Because CSS processing occurs <em>after</em> the parsing stage, it is not possible to restore these characters for styling. Therefore, some of the behavior specified below can be affected by these limitations and may be user agent dependent.

<a id="ref-for-collapsible-white-space"></a>

<a id="ref-for-white-space①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Anonymous blocks consisting entirely of [collapsible](#collapsible-white-space) [white space](#white-space) are removed from the rendering tree. Thus any such <a id="ref-for-white-space①④"></a>white space surrounding a block-level element is collapsed away. See [CSS 2 § 9.2.2.1 Anonymous inline boxes](https://www.w3.org/TR/CSS2/visuren.html#anonymous). [\[CSS2\]](#biblio-css2)

<a id="ref-for-unicode-general-category①"></a>

<a id="ref-for-segment-break⑤"></a>

<a id="ref-for-unicode-script"></a>

Control characters ([Unicode category](#unicode-general-category) `Cc`)—​other than tabs (U+0009), line feeds (U+000A), carriage returns (U+000D) and sequences that form a [segment break](#segment-break)—​must be rendered as a visible glyph which the UA must synthesize if the glyphs found in the font are not visible, and must be otherwise treated as any other character of the Other Symbols (`So`) <a id="ref-for-unicode-general-category②"></a>general category and Common [script](#unicode-script). The UA may use a glyph provided by a font specifically for the control character, substitute the glyphs provided for the corresponding symbol in the Control Pictures block, generate a visual representation of its code point value, or use some other method to provide an appropriate visible glyph. As required by Unicode, unsupported `Default_ignorable` characters must be ignored for text rendering. [\[UNICODE\]](#biblio-unicode)

Carriage returns (U+000D) are treated identically to spaces (U+0020) in all respects.

<a id="ref-for-normalize-newlines①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For HTML documents, carriage returns present in the source code are converted to line feeds at the parsing stage (see [HTML § 13.2.3.5 Preprocessing the input stream](https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream) and the definition of [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines) in [Infra](https://infra.spec.whatwg.org/) and therefore do no appear as U+000D CARRIAGE RETURN to CSS. [\[HTML\]](#biblio-html) [\[INFRA\]](#biblio-infra)) However, the character <em>is</em> preserved—​and the above rule observable—​when encoded using an escape sequence (`&#x0d;`).

### <a id="white-space-rules"></a>4.1.  The White Space Processing Rules

Except where specified otherwise, white space processing in CSS affects only the <a id="white-space"></a>document white space characters: <a id="spaces"></a>spaces (U+0020), <a id="tabs"></a>tabs (U+0009), and [segment breaks](#white-space-processing).

<a id="ref-for-white-space①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The set of characters considered [document white space](#white-space) (part of the document content) and those considered syntactic white space (part of the CSS syntax) are not necessarily identical. However, since both include spaces (U+0020), tabs (U+0009), and line feeds (U+000A) most authors won’t notice any differences.

<a id="ref-for-unicode-general-category③"></a>

Besides space (U+0020) and no-break space (U+00A0), Unicode defines a number of additional space separator characters. [\[UNICODE\]](#biblio-unicode) In this specification all characters in the Unicode [general category](#unicode-general-category) Zs except space (U+0020) and no-break space (U+00A0) are collectively referred to as <a id="other-space-separators"></a>other space separators.

#### <a id="white-space-phase-1"></a>4.1.1.  Phase I: Collapsing and Transformation

<a id="ref-for-inline-formatting-context"></a>

<a id="ref-for-white-space①⑥"></a>

<a id="ref-for-line-breaking-process"></a>

For each inline (including anonymous inlines; see [CSS 2 § 9.2.2.1 Anonymous inline boxes](https://www.w3.org/TR/CSS2/visuren.html#anonymous) [\[CSS2\]](#biblio-css2)) within an [inline formatting context](https://www.w3.org/TR/css-display-4/#inline-formatting-context), [white space characters](#white-space) are processed as follows prior to [line breaking](#line-breaking-process) and [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction), ignoring <a id="bidi-formatting-characters"></a>bidi formatting characters (characters with the `Bidi_Control` property [\[UAX9\]](#biblio-uax9)) as if they were not there:

- <a id="ref-for-white-space①⑦"></a>

  <a id="ref-for-valdef-white-space-pre-line①"></a>

  <a id="ref-for-valdef-white-space-nowrap①"></a>

  <a id="ref-for-valdef-white-space-normal④"></a>

  <a id="ref-for-propdef-white-space④"></a>

  <a id="collapse"></a> If [white-space](#propdef-white-space) is set to [normal](#valdef-white-space-normal), [nowrap](#valdef-white-space-nowrap), or [pre-line](#valdef-white-space-pre-line), [white space characters](#white-space) are considered <a id="collapsible-white-space"></a>collapsible and are processed by performing the following steps:

  1.  <a id="ref-for-segment-break⑥"></a>

      <a id="ref-for-tabs①"></a>

      <a id="ref-for-spaces②"></a>

      Any sequence of collapsible [spaces](#spaces) and [tabs](#tabs) immediately preceding or following a [segment break](#segment-break) is removed.

  2.  <a id="ref-for-segment-break⑦"></a>

      Collapsible [segment breaks](#segment-break) are transformed for rendering according to the [segment break transformation rules](#line-break-transform).

  3.  <a id="ref-for-tabs②"></a>

      <a id="ref-for-collapsible-white-space①"></a>

      Every [collapsible](#collapsible-white-space) [tab](#tabs) is converted to a collapsible space (U+0020).

  4.  <a id="ref-for-soft-wrap-opportunity③"></a>

      <a id="ref-for-spaces③"></a>

      <a id="ref-for-collapsible-white-space②"></a>

      Any [collapsible](#collapsible-white-space) [space](#spaces) immediately following another <a id="ref-for-collapsible-white-space③"></a>collapsible <a id="ref-for-spaces④"></a>space—​even one outside the boundary of the inline containing that <a id="ref-for-spaces⑤"></a>space, provided both <a id="ref-for-spaces⑥"></a>spaces are within the same inline formatting context—​is collapsed to have zero advance width. (It is invisible, but retains its [soft wrap opportunity](#soft-wrap-opportunity), if any.)

- <a id="ref-for-tabs③"></a>

  <a id="ref-for-spaces⑦"></a>

  <a id="ref-for-soft-wrap-opportunity④"></a>

  <a id="ref-for-valdef-white-space-break-spaces①"></a>

  <a id="ref-for-valdef-white-space-pre-wrap②"></a>

  <a id="ref-for-valdef-white-space-pre③"></a>

  <a id="ref-for-propdef-white-space⑤"></a>

  If [white-space](#propdef-white-space) is set to [pre](#valdef-white-space-pre), [pre-wrap](#valdef-white-space-pre-wrap), or [break-spaces](#valdef-white-space-break-spaces), any sequence of spaces is treated as a sequence of non-breaking spaces. However, for <a id="ref-for-valdef-white-space-pre-wrap③"></a>pre-wrap, a [soft wrap opportunity](#soft-wrap-opportunity) exists at the end of a sequence of [spaces](#spaces) and/or [tabs](#tabs), while for <a id="ref-for-valdef-white-space-break-spaces②"></a>break-spaces, a <a id="ref-for-soft-wrap-opportunity⑤"></a>soft wrap opportunity exists after every <a id="ref-for-spaces⑧"></a>space and every <a id="ref-for-tabs④"></a>tab.

<a id="ref-for-spaces⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="egbidiwscollapse"></a> The following example illustrates the interaction of white-space collapsing and bidirectionality. Consider the following markup fragment, taking special note of [spaces](#spaces) (with varied backgrounds and borders for emphasis and identification):
>
> ```text
> <ltr>A <rtl> B </rtl> C</ltr>
> ```
>
> <a id="ref-for-propdef-white-space⑥"></a>
>
> <a id="ref-for-valdef-white-space-normal⑤"></a>
>
> where the `<ltr>` element represents a left-to-right embedding and the `<rtl>` element represents a right-to-left embedding. If the [white-space](#propdef-white-space) property is set to [normal](#valdef-white-space-normal), the white-space processing model will result in the following:
>
> - <a id="ref-for-spaces①⓪"></a>
>
>   The [space](#spaces) before the B ( ) will collapse with the <a id="ref-for-spaces①①"></a>space after the A ( ).
>
> - <a id="ref-for-spaces①②"></a>
>
>   The [space](#spaces) before the C ( ) will collapse with the <a id="ref-for-spaces①③"></a>space after the B ( ).
>
> <a id="ref-for-spaces①④"></a>
>
> This will leave two [spaces](#spaces), one after the A in the left-to-right embedding level, and one after the B in the right-to-left embedding level. The text will then be ordered according to the Unicode bidirectional algorithm, with the end result being:
>
> ```text
> A  BC
> ```
>
> <a id="ref-for-spaces①⑤"></a>
>
> Note that there will be two [spaces](#spaces) between A and B, and none between B and C. This is best avoided by putting <a id="ref-for-spaces①⑥"></a>spaces outside the element instead of just inside the opening and closing tags and, where practical, by relying on implicit bidirectionality instead of explicit embedding levels.

#### <a id="white-space-phase-2"></a>4.1.2.  Phase II: Trimming and Positioning

<a id="ref-for-wrapping⑤"></a>

<a id="ref-for-propdef-white-space⑦"></a>

Then, the entire block is rendered. Inlines are laid out, taking [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) into account, and [wrapping](#wrapping) as specified by the [white-space](#propdef-white-space) property. As each line is laid out,

1.  <a id="ref-for-spaces①⑦"></a>

    <a id="ref-for-collapsible-white-space④"></a>

    A sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) at the beginning of a line is removed.

2.  <a id="ref-for-propdef-tab-size①"></a>

    <a id="ref-for-block-container"></a>

    <a id="ref-for-tab-stop"></a>

    <a id="ref-for-tabs⑤"></a>

    <a id="ref-for-preserved-white-space④"></a>

    <a id="ref-for-tab-size-dfn"></a>

    If the [tab size](#tab-size-dfn) is zero, [preserved](#preserved-white-space) [tabs](#tabs) are not rendered. Otherwise, each <a id="ref-for-preserved-white-space⑤"></a>preserved <a id="ref-for-tabs⑥"></a>tab is rendered as a horizontal shift that lines up the start edge of the next glyph with the next [tab stop](#tab-stop). If this distance is less than 0.5ch, then the subsequent <a id="ref-for-tab-stop①"></a>tab stop is used instead. <a id="tab-stop"></a>Tab stops occur at points that are multiples of the <a id="ref-for-tab-size-dfn①"></a>tab size from the starting content edge of the <a id="ref-for-preserved-white-space⑥"></a>preserved <a id="ref-for-tabs⑦"></a>tab’s nearest [block container](https://www.w3.org/TR/css-display-4/#block-container) ancestor. The <a id="ref-for-tab-size-dfn②"></a>tab size is given by the [tab-size](#propdef-tab-size) property.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: See the Unicode [rules on how tabulation (U+0009) interacts with bidi](http://unicode.org/reports/tr9/#L1). [\[UAX9\]](#biblio-uax9)

3.  <a id="ref-for-valdef-white-space-pre-line②"></a>

    <a id="ref-for-valdef-white-space-nowrap②"></a>

    <a id="ref-for-valdef-white-space-normal⑥"></a>

    <a id="ref-for-propdef-white-space⑧"></a>

    <a id="ref-for-spaces①⑧"></a>

    <a id="ref-for-collapsible-white-space⑤"></a>

    A sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) at the end of a line is removed, as well as any trailing U+1680   OGHAM SPACE MARK whose [white-space](#propdef-white-space) property is [normal](#valdef-white-space-normal), [nowrap](#valdef-white-space-nowrap), or [pre-line](#valdef-white-space-pre-line).

    <a id="ref-for-collapsible-white-space⑥"></a>

    <a id="ref-for-spaces①⑨"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Due to Unicode Bidirectional Algorithm rule [L1](http://unicode.org/reports/tr9/#L1), a sequence of [collapsible](#collapsible-white-space) [spaces](#spaces) located at the end of the line prior to [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) will also be at the end of the line after reordering. [\[UAX9\]](#biblio-uax9) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

4.  <a id="ref-for-tabs⑧"></a>

    <a id="ref-for-preserved-white-space⑦"></a>

    <a id="ref-for-other-space-separators④"></a>

    <a id="ref-for-white-space①⑧"></a>

    If there remains any sequence of [white space](#white-space), [other space separators](#other-space-separators), and/or [preserved](#preserved-white-space) [tabs](#tabs) at the end of a line (after [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)):

    - <a id="ref-for-hang①"></a>

      <a id="ref-for-valdef-white-space-pre-line③"></a>

      <a id="ref-for-valdef-white-space-nowrap③"></a>

      <a id="ref-for-valdef-white-space-normal⑦"></a>

      <a id="ref-for-propdef-white-space⑨"></a>

      If [white-space](#propdef-white-space) is set to [normal](#valdef-white-space-normal), [nowrap](#valdef-white-space-nowrap), or [pre-line](#valdef-white-space-pre-line), the UA must [hang](#hang) this sequence (unconditionally).

    - <a id="ref-for-conditionally-hang"></a>

      <a id="ref-for-forced-line-break③"></a>

      <a id="ref-for-hang②"></a>

      <a id="ref-for-valdef-white-space-pre-wrap④"></a>

      <a id="ref-for-propdef-white-space①⓪"></a>

      If [white-space](#propdef-white-space) is set to [pre-wrap](#valdef-white-space-pre-wrap), the UA must (unconditionally) [hang](#hang) this sequence, unless the sequence is followed by a [forced line break](#forced-line-break), in which case it must [conditionally hang](#conditionally-hang) the sequence instead. It may also visually collapse the character advance widths of any that would otherwise overflow.

      <a id="ref-for-hang③"></a>

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > Note: [Hanging](#hang) the white space rather than collapsing it allows users to see the space when selecting or editing text.

    - <a id="ref-for-hang④"></a>

      <a id="ref-for-other-space-separators⑤"></a>

      <a id="ref-for-tabs⑨"></a>

      <a id="ref-for-spaces②⓪"></a>

      <a id="ref-for-valdef-white-space-break-spaces③"></a>

      <a id="ref-for-propdef-white-space①①"></a>

      If [white-space](#propdef-white-space) is set to [break-spaces](#valdef-white-space-break-spaces), [spaces](#spaces), [tabs](#tabs), and [other space separators](#other-space-separators) are treated the same as other visible characters: they cannot [hang](#hang) nor have their advance width collapsed.

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > Note: Such characters therefore take up space, and depending on the available space and applicable line breaking controls will either overflow or cause the line to wrap.

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
> <a id="ref-for-spaces②①"></a>
>
> Since the final [space](#spaces) is before a forced line break and does not overflow, it does not hang, and centering works as expected.

<a id="ref-for-hang⑤"></a>

<a id="ref-for-spaces②②"></a>

<a id="ref-for-conditionally-hang②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e231f51f"></a> This example illustrates the difference between [hanging](#hang) [spaces](#spaces) at the end of lines without forced breaks, and [conditionally hanging](#conditionally-hang) them at the end of lines with forced breaks. An underline is added to help visualize the <a id="ref-for-spaces②③"></a>spaces.
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
> <a id="ref-for-spaces②④"></a>
>
> <a id="ref-for-hang⑥"></a>
>
> <a id="ref-for-conditionally-hang③"></a>
>
> As the [preserved](#preserved-white-space) [spaces](#spaces) at the end of lines without a forced break must [hang](#hang), they are not considered when placing the rest of the line during text alignment. When aligning towards the end, this means any such <a id="ref-for-spaces②⑤"></a>spaces will overflow, and will not prevent the rest of the line’s content from being flush with the edge of the line. On the other hand, preserved spaces at the end of a line <em>with</em> a forced break [conditionally hang](#conditionally-hang). Since the space at the end of the last line would not overflow in this example, it does not <a id="ref-for-hang⑦"></a>hang and therefore is considered during text alignment.

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

#### <a id="line-break-transform"></a>4.1.3.  Segment Break Transformation Rules

<a id="ref-for-propdef-white-space①②"></a>

<a id="ref-for-valdef-white-space-pre④"></a>

<a id="ref-for-valdef-white-space-pre-wrap⑤"></a>

<a id="ref-for-valdef-white-space-break-spaces④"></a>

<a id="ref-for-valdef-white-space-pre-line④"></a>

<a id="ref-for-segment-break⑧"></a>

<a id="ref-for-collapsible-white-space⑦"></a>

When [white-space](#propdef-white-space) is [pre](#valdef-white-space-pre), [pre-wrap](#valdef-white-space-pre-wrap), [break-spaces](#valdef-white-space-break-spaces), or [pre-line](#valdef-white-space-pre-line), [segment breaks](#segment-break) are not [collapsible](#collapsible-white-space) and are instead transformed into a preserved line feed (U+000A).

<a id="ref-for-propdef-white-space①③"></a>

<a id="ref-for-segment-break⑨"></a>

<a id="ref-for-collapsible-white-space⑧"></a>

For other values of [white-space](#propdef-white-space), [segment breaks](#segment-break) are [collapsible](#collapsible-white-space), and are collapsed as follows:

1.  <a id="ref-for-segment-break①⓪"></a>

    First, any collapsible [segment break](#segment-break) immediately following another collapsible <a id="ref-for-segment-break①①"></a>segment break is removed.

2.  <a id="ref-for-segment-break①②"></a>

    Then any remaining [segment break](#segment-break) is either transformed into a space (U+0020) or removed depending on the context before and after the break. The rules for this operation are UA-defined in this level.

    <a id="ref-for-tabs①⓪"></a>

    <a id="ref-for-spaces②⑥"></a>

    <a id="ref-for-segment-break①③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The white space processing rules have already removed any [tabs](#tabs) and [spaces](#spaces) around the [segment break](#segment-break) before this context is evaluated.

<a id="ref-for-spaces②⑦"></a>

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
> <a id="ref-for-spaces②⑧"></a>
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
> <a id="ref-for-white-space①⑨"></a>
>
> Eliminating a line break in Chinese requires eliminating any intervening [white space](#white-space).
>
> The segment break transformation rules can use adjacent context to either transform the segment break into a space or eliminate it entirely.

<a id="ref-for-segment-break①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Historically, HTML and CSS have unconditionally converted [segment breaks](#segment-break) to spaces, which has prevented content authored in languages such as Chinese from being able to break lines within the source. Thus UA heuristics need to be conservative about where they discard <a id="ref-for-segment-break①⑤"></a>segment breaks even as they strive to improve support for such languages.

<a id="ref-for-propdef-tab-size②"></a>

### <a id="tab-size-property"></a>4.2. <a id="tab-size"></a> Tab Character Size: the [tab-size](#propdef-tab-size) property

| Field               | Definition                                                                                                                                                                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-tab-size"></a>tab-size                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-number-value"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 8                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified number or absolute length                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                    |

<a id="ref-for-preserved-white-space⑨"></a>

<a id="ref-for-number-value①"></a>

<a id="ref-for-block-container①"></a>

<a id="ref-for-tabs①①"></a>

<a id="ref-for-propdef-letter-spacing"></a>

<a id="ref-for-propdef-word-spacing"></a>

This property determines the <a id="tab-size-dfn"></a>tab size used to render [preserved](#preserved-white-space) tab characters (U+0009). A [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) represents the measure as a multiple of the advance width of the space character (U+0020) of the nearest [block container](https://www.w3.org/TR/css-display-4/#block-container) ancestor of the <a id="ref-for-preserved-white-space①⓪"></a>preserved [tab](#tabs), including its associated [letter-spacing](#propdef-letter-spacing) and [word-spacing](#propdef-word-spacing). Negative values are not allowed.

## <a id="line-breaking"></a>5.  Line Breaking and Word Boundaries

<a id="ref-for-preserved-white-space①①"></a>

When inline-level content is laid out into lines, it is broken across line boxes. Such a break is called a <a id="line-break"></a>line break. When a line is broken due to explicit line-breaking controls (such as a [preserved](#preserved-white-space) newline character), or due to the start or end of a block, it is a <a id="forced-line-break"></a>forced line break. When a line is broken due to content <a id="wrapping"></a>wrapping (i.e. when the UA creates unforced line breaks in order to fit the content within the measure), it is a <a id="soft-wrap-break"></a>soft wrap break. The process of breaking inline-level content into lines is called <a id="line-breaking-process"></a>line breaking.

<a id="ref-for-propdef-white-space①④"></a>

<a id="ref-for-soft-wrap-opportunity⑥"></a>

<a id="ref-for-content-language⑧"></a>

Wrapping is only performed at an allowed break point, called a <a id="soft-wrap-opportunity"></a>soft wrap opportunity. When wrapping is enabled (see [white-space](#propdef-white-space)), the UA must minimize the amount of content overflowing a line by wrapping the line at a [soft wrap opportunity](#soft-wrap-opportunity), if one exists. Valid <a id="ref-for-soft-wrap-opportunity⑦"></a>soft wrap opportunities depend on the [content language](#content-language) and writing system, as well as on CSS properties that control them.

<a id="ref-for-soft-wrap-opportunity⑧"></a>

<a id="ref-for-spaces②⑨"></a>

In most writing systems, in the absence of hyphenation a [soft wrap opportunity](#soft-wrap-opportunity) occurs only at word boundaries. Many such systems (such as English written using the Latin alphabet) use [spaces](#spaces) or punctuation to explicitly separate words, and <a id="ref-for-soft-wrap-opportunity⑨"></a>soft wrap opportunities can be identified by these characters.

<a id="ref-for-soft-wrap-opportunity①⓪"></a>

Scripts such as Thai, Lao, and Khmer, however, do not use spaces or punctuation to separate words. Although the zero width space (U+200B) can be used as an explicit word delimiter in these scripts, this practice is not common. As a result, a lexical resource is needed to correctly identify [soft wrap opportunities](#soft-wrap-opportunity) in such texts.

<a id="ref-for-soft-wrap-opportunity①①"></a>

<a id="ref-for-content-language⑨"></a>

In some other writing systems, notably Brahmic scripts such as Javanese and Balinese, [soft wrap opportunities](#soft-wrap-opportunity) are based on orthographic syllable boundaries, not word boundaries. Although orthographic syllable breaking does not depend on the [content language](#content-language) or require lexical resource, it nonetheless requires analysis of the text to find breaking opportunities.

<a id="ref-for-typographic-letter-unit①"></a>

In others such as Chinese (as well as Japanese, Yi, and sometimes also Korean), each syllable tends to correspond to a single [typographic letter unit](#typographic-letter-unit), and thus line breaking conventions allow the line to break anywhere <em>except</em> between certain character combinations. Additionally the level of strictness in these restrictions varies with the typesetting style.

<a id="ref-for-soft-wrap-opportunity①②"></a>

While CSS does not fully define where [soft wrap opportunities](#soft-wrap-opportunity) occur, some controls are provided to distinguish common variations:

- <a id="ref-for-propdef-line-break"></a>

  The [line-break](#propdef-line-break) property allows choosing various levels of “strictness” for line breaking restrictions.

- <a id="ref-for-propdef-word-break"></a>

  The [word-break](#propdef-word-break) property controls what types of letters are glommed together to form unbreakable “words”, causing CJK characters to behave like non-CJK text or vice versa.

- <a id="ref-for-propdef-hyphens"></a>

  The [hyphens](#propdef-hyphens) property controls whether automatic hyphenation is allowed to break words in scripts that hyphenate.

- <a id="ref-for-propdef-overflow-wrap"></a>

  The [overflow-wrap](#propdef-overflow-wrap) property allows the UA to take a break anywhere in otherwise-unbreakable strings that would otherwise overflow.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Unicode Standard Annex \#14: Unicode Line Breaking Algorithm](https://www.unicode.org/reports/tr14/) defines a baseline behavior for line breaking for all scripts in Unicode, which is expected to be further tailored. [\[UAX14\]](#biblio-uax14) More information on line breaking conventions can be found in [Requirements for Japanese Text Layout](https://www.w3.org/TR/jlreq/) [\[JLREQ\]](#biblio-jlreq) and Formatting Rules for Japanese Documents [\[JIS4051\]](#biblio-jis4051) for Japanese, [Requirements for Chinese Text Layout](https://www.w3.org/TR/clreq/) [\[CLREQ\]](#biblio-clreq) and General Rules for Punctuation [\[ZHMARK\]](#biblio-zhmark) for Chinese. See also the [Internationalization Working Group](https://www.w3.org/International/)’s [Language Enablement Index](https://www.w3.org/TR/typography/#blocks_paragraphs) which includes more information on additional languages. [\[TYPOGRAPHY\]](#biblio-typography) Any guidance on additional appropriate references would be much appreciated.

<a id="ref-for-propdef-word-break①"></a>

### <a id="word-break-property"></a>5.1. <a id="word-break"></a> Breaking Rules for Letters: the [word-break](#propdef-word-break) property

| Field               | Definition                                                                                                                                             |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-word-break"></a>word-break                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑨"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) keep-all <a id="ref-for-comb-one①⓪"></a>\| break-all <a id="ref-for-comb-one①①"></a>\| break-word |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                               |

<a id="ref-for-soft-wrap-opportunity①③"></a>

<a id="ref-for-typographic-letter-unit②"></a>

<a id="ref-for-letter②"></a>

<a id="ref-for-typographic-character-unit①③"></a>

<a id="ref-for-white-space②⓪"></a>

<a id="ref-for-other-space-separators⑥"></a>

<a id="ref-for-propdef-line-break①"></a>

This property specifies [soft wrap opportunities](#soft-wrap-opportunity) between letters, i.e. where it is “normal” and permissible to break lines of text. Specifically it controls whether a <a id="ref-for-soft-wrap-opportunity①④"></a>soft wrap opportunity generally exists between adjacent [typographic letter units](#typographic-letter-unit), treating non-[letter](#letter) [typographic character units](#typographic-character-unit) belonging to the `NU`, `AL`, `AI`, or `ID` Unicode line breaking classes as <a id="ref-for-typographic-letter-unit③"></a>typographic letter units for this purpose (only). [\[UAX14\]](#biblio-uax14) It does not affect rules governing the <a id="ref-for-soft-wrap-opportunity①⑤"></a>soft wrap opportunities created by [white space](#white-space) (as well as by [other space separators](#other-space-separators)) and around punctuation. (See [line-break](#propdef-line-break) for controls affecting punctuation and small kana.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f0d22488"></a> For example, in some styles of CJK typesetting, English words are allowed to break between any two letters, rather than only at spaces or hyphenation points; this can be enabled with word-break:break-all.
>
> ![A snippet of Japanese text with English in it. The word 'caption' is broken into 'capt' and 'ion' across two lines.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/break-all.png)
>
> An example of English text embedded in Japanese being broken at an arbitrary point in the word.
>
> <a id="ref-for-propdef-word-break②"></a>
>
> As another example, Korean has two styles of line-breaking: between any two Korean syllables ([word-break: normal](#propdef-word-break)) or, like English, mainly at spaces (<a id="ref-for-propdef-word-break③"></a>word-break: keep-all).
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
> <a id="ref-for-word-separator"></a>
>
> <a id="ref-for-propdef-word-break④"></a>
>
> Ethiopic similarly has two styles of line-breaking, either only breaking at [word separators](#word-separator) ([word-break: normal](#propdef-word-break)), or also allowing breaks between letters within a word (<a id="ref-for-propdef-word-break⑤"></a>word-break: break-all).
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
<a id="ref-for-propdef-overflow-wrap①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: To enable additional break opportunities only in the case of overflow, see [overflow-wrap](#propdef-overflow-wrap).

Values have the following meanings:

<a id="valdef-word-break-normal"></a>normal  
Words break according to their customary rules, as described [above](#line-breaking). Korean, which commonly exhibits two different behaviors, allows breaks between any two consecutive Hangul/Hanja. For Ethiopic, which also exhibits two different behaviors, such breaks within words are not allowed.

<a id="valdef-word-break-break-all"></a>break-all  
<a id="ref-for-typographic-character-unit①④"></a>

<a id="ref-for-typographic-letter-unit④"></a>

<a id="ref-for-valdef-word-break-normal"></a>

<a id="ref-for-soft-wrap-opportunity①⑥"></a>

Breaking is allowed within “words”: specifically, in addition to [soft wrap opportunities](#soft-wrap-opportunity) allowed for [normal](#valdef-word-break-normal), any [typographic letter units](#typographic-letter-unit) (and any [typographic character units](#typographic-character-unit) resolving to the `NU` (“numeric”), `AL` (“alphabetic”), or `SA` (“Southeast Asian”) line breaking classes [\[UAX14\]](#biblio-uax14)) are instead treated as `ID` (“ideographic characters”) for the purpose of line-breaking. Hyphenation is not applied.

<a id="ref-for-soft-wrap-opportunity①⑦"></a>

<a id="ref-for-propdef-line-break②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value does not affect whether there are [soft wrap opportunities](#soft-wrap-opportunity) around punctuation characters. To allow breaks anywhere, see [line-break: anywhere](#propdef-line-break).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This option enables the other common behavior for Ethiopic. It is also often used in a context where the text consists predominantly of CJK characters with only short non-CJK excerpts, and it is desired that the text be better distributed on each line.

<a id="valdef-word-break-keep-all"></a>keep-all  
<a id="ref-for-valdef-word-break-normal①"></a>

<a id="ref-for-valdef-line-break-anywhere"></a>

<a id="ref-for-propdef-line-break③"></a>

<a id="ref-for-typographic-character-unit①⑤"></a>

<a id="ref-for-typographic-letter-unit⑤"></a>

<a id="ref-for-soft-wrap-opportunity①⑧"></a>

Breaking is forbidden within “words”: implicit [soft wrap opportunities](#soft-wrap-opportunity) between [typographic letter units](#typographic-letter-unit) (or other [typographic character units](#typographic-character-unit) belonging to the `NU`, `AL`, `AI`, or `ID` Unicode line breaking classes [\[UAX14\]](#biblio-uax14)) are suppressed, i.e. breaks are prohibited between pairs of such characters (regardless of [line-break](#propdef-line-break) settings other than [anywhere](#valdef-line-break-anywhere)) except where opportunities exist due to dictionary-based breaking. Otherwise this option is equivalent to [normal](#valdef-word-break-normal). In this style, sequences of CJK characters do not break.

<a id="ref-for-spaces③⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the other common behavior for Korean (which uses [spaces](#spaces) between words), and is also useful for mixed-script text where CJK snippets are mixed into another language that uses <a id="ref-for-spaces③①"></a>spaces for separation.

Symbols that line-break the same way as letters of a particular category are affected the same way as those letters.

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
> <a id="ref-for-propdef-word-break⑥"></a>
>
> [word-break: normal](#propdef-word-break)
>
> ```text
> 这·是·一·些·汉·字·and·some·Latin·و·کمی·خط·عربی·และ·ตัวอย่าง·การเขียน·ภาษาไทย·በጽሑፍ፡·ማራዘሙን፡·አንዳንድ፡
> ```
>
> <a id="ref-for-propdef-word-break⑦"></a>
>
> [word-break: break-all](#propdef-word-break)
>
> ```text
> 这·是·一·些·汉·字·a·n·d·s·o·m·e·L·a·t·i·n·و·ﮐ·ﻤ·ﻰ·ﺧ·ﻁ·ﻋ·ﺮ·ﺑ·ﻰ·แ·ล·ะ·ตั·ว·อ·ย่·า·ง·ก·า·ร·เ·ขี·ย·น·ภ·า·ษ·า·ไ·ท·ย·በ·ጽ·ሑ·ፍ፡·ማ·ራ·ዘ·ሙ·ን፡·አ·ን·ዳ·ን·ድ፡
> ```
>
> <a id="ref-for-propdef-word-break⑧"></a>
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
> Japanese is usually typeset allowing line breaks between syllables within words. However, it is sometimes preferred to suppress these wrapping opportunities and to only allow wrapping at the end of certain sentence fragments. This is most commonly done in very short pieces of text, such as headings and table or figure captions.
>
> <a id="ref-for-the-wbr-element"></a>
>
> <a id="ref-for-propdef-word-break⑨"></a>
>
> This can be achieved by marking the allowed wrapping points with <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code> or U+200B ZERO WIDTH SPACE, and suppressing the other ones using [word-break: keep-all](#propdef-word-break).
>
> <a id="ref-for-propdef-word-break①⓪"></a>
>
> For instance, the following markup can produce either of the renderings below, depending on the value of the [word-break](#propdef-word-break) property:
>
> ```text
> <h1>窓ぎわの<wbr>トットちゃん</h1>
> ```
>
>
> These are static transcriptions of the source’s live browser demonstration. The “browser” text below is the source content, not a measured rendering. Original demonstration HTML/CSS is included so clipping, direction, and line-breaking are not lost. See the [source demonstration](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/#jp-title-break).
>
> **Demonstration stylesheet from the source**
>
> The source places these demonstration cells inside the `jp-title-break` container.
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
> 	color: black;
> 	background: white;
> }
> ```
>
> **`h1 { word-break: normal }`**
>
> **Expected rendering**
>
> ```text
> 窓ぎわのトットちゃ
> ん
> ```
>
> **Result in your browser**
>
> ```text
> 窓ぎわのトットちゃん
> ```
>
> **Original browser-demonstration HTML**
>
> ```html
> <samp lang="ja">
> 							窓ぎわの<wbr>トットちゃん
> 						</wbr></samp>
> ```
>
> **`h1 { word-break: keep-all }`**
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
> ```text
> 窓ぎわのトットちゃん
> ```
>
> **Original browser-demonstration HTML**
>
> ```html
> <samp lang="ja" style="word-break:keep-all">
> 							窓ぎわの<wbr>トットちゃん
> 						</wbr></samp>
> ```
>
<a id="ref-for-propdef-word-break①①"></a>

<a id="ref-for-propdef-overflow-wrap②"></a>

For compatibility with legacy content, the [word-break](#propdef-word-break) property also supports a deprecated <a id="valdef-word-break-break-word"></a>break-word keyword. When specified, this has the same effect as <a id="ref-for-propdef-word-break①②"></a>word-break: normal and [overflow-wrap: anywhere](#propdef-overflow-wrap), regardless of the actual value of the <a id="ref-for-propdef-overflow-wrap③"></a>overflow-wrap property.

<a id="ref-for-propdef-word-break①③"></a>

<a id="ref-for-intrinsic-size"></a>

The effects of [word-break](#propdef-word-break) are taken into account when computing [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size).

<a id="ref-for-propdef-line-break④"></a>

### <a id="line-break-property"></a>5.2.  Line Breaking Strictness: the [line-break](#propdef-line-break) property

| Field               | Definition                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-line-break"></a>line-break                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①②"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) loose <a id="ref-for-comb-one①③"></a>\| normal <a id="ref-for-comb-one①④"></a>\| strict <a id="ref-for-comb-one①⑤"></a>\| anywhere |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                  |

<a id="ref-for-wrapping⑥"></a>

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
<a id="ref-for-propdef-word-break①④"></a>

<a id="ref-for-preserved-white-space①②"></a>

<a id="ref-for-typographic-character-unit①⑥"></a>

<a id="ref-for-soft-wrap-opportunity①⑨"></a>

There is a [soft wrap opportunity](#soft-wrap-opportunity) around every [typographic character unit](#typographic-character-unit), including around any punctuation character or [preserved white spaces](#preserved-white-space), or in the middle of words, disregarding any prohibition against line breaks, even those introduced by characters with the `GL`, `WJ`, or `ZWJ` line breaking classes or mandated by the [word-break](#propdef-word-break) property. [\[UAX14\]](#biblio-uax14) The different wrapping opportunities must not be prioritized. Hyphenation is not applied.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value triggers the line breaking rules typically seen in terminals.

<a id="ref-for-valdef-line-break-anywhere①"></a>

<a id="ref-for-preserved-white-space①③"></a>

<a id="ref-for-propdef-white-space①⑤"></a>

<a id="ref-for-valdef-white-space-break-spaces⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [anywhere](#valdef-line-break-anywhere) only allows [preserved white spaces](#preserved-white-space) at the end of the line to be wrapped to the next line when [white-space](#propdef-white-space) is set to [break-spaces](#valdef-white-space-break-spaces), because in other cases:
> - <a id="ref-for-valdef-white-space-pre-line⑤"></a>
>
>   <a id="ref-for-valdef-white-space-normal⑧"></a>
>
>   <a id="ref-for-preserved-white-space①④"></a>
>
>   [preserved white space](#preserved-white-space) at the end/start of the line is discarded ([normal](#valdef-white-space-normal), [pre-line](#valdef-white-space-pre-line))
>
> - <a id="ref-for-valdef-white-space-pre⑤"></a>
>
>   <a id="ref-for-valdef-white-space-nowrap④"></a>
>
>   wrapping is forbidden altogether ([nowrap](#valdef-white-space-nowrap), [pre](#valdef-white-space-pre))
>
> - <a id="ref-for-valdef-white-space-pre-wrap⑥"></a>
>
>   <a id="ref-for-hang⑨"></a>
>
>   <a id="ref-for-preserved-white-space①⑤"></a>
>
>   the [preserved white space](#preserved-white-space) [hang](#hang) ([pre-wrap](#valdef-white-space-pre-wrap)).
>
> <a id="ref-for-preserved-white-space①⑥"></a>
>
> <a id="ref-for-propdef-white-space①⑥"></a>
>
> <a id="ref-for-valdef-white-space-break-spaces⑥"></a>
>
> When it does have an effect on [preserved white space](#preserved-white-space), with [white-space: break-spaces](#propdef-white-space), it allows breaking before the first space of a sequence, which [break-spaces](#valdef-white-space-break-spaces) on its own does not.

<a id="ref-for-valdef-line-break-loose"></a>

<a id="ref-for-valdef-line-break-normal"></a>

<a id="ref-for-valdef-line-break-strict"></a>

CSS distinguishes between four levels of strictness in the rules for text wrapping. The precise set of rules in effect for each of [loose](#valdef-line-break-loose), [normal](#valdef-line-break-normal), and [strict](#valdef-line-break-strict) is up to the UA and should follow language conventions. However, for these three keywords, this specification does require that:

- <a id="ref-for-writing-system-japanese"></a>

  <a id="ref-for-writing-system-chinese"></a>

  <a id="ref-for-content-writing-system①"></a>

  <a id="ref-for-valdef-line-break-loose①"></a>

  <a id="ref-for-valdef-line-break-normal①"></a>

  The following breaks are allowed for [normal](#valdef-line-break-normal) and [loose](#valdef-line-break-loose) line breaking if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese), and are otherwise forbidden:

  - breaks before certain CJK hyphen-like characters:  
    〜 U+301C, ゠ U+30A0

- <a id="ref-for-propdef-word-break①⑤"></a>

  <a id="ref-for-valdef-line-break-loose②"></a>

  The following breaks are allowed for [loose](#valdef-line-break-loose) line breaking if the preceding character belongs to the Unicode line breaking class `ID` [\[UAX14\]](#biblio-uax14) (including when the preceding character is treated as `ID` due to [word-break: break-all](#propdef-word-break)), and are otherwise forbidden:

  - breaks before hyphens:  
    ‐ U+2010, – U+2013

- <a id="ref-for-valdef-line-break-loose③"></a>

  <a id="ref-for-valdef-line-break-strict①"></a>

  <a id="ref-for-valdef-line-break-normal②"></a>

  The following breaks are forbidden for [normal](#valdef-line-break-normal) and [strict](#valdef-line-break-strict) line breaking and allowed in [loose](#valdef-line-break-loose):

  - breaks before Japanese small kana or the Katakana-Hiragana prolonged sound mark, i.e. characters from the Unicode line breaking class `CJ`. [\[UAX14\]](#biblio-uax14)
  - breaks before iteration marks:  
    々 U+3005, 〻 U+303B, ゝ U+309D, ゞ U+309E, ヽ U+30FD, ヾ U+30FE
  - breaks between inseparable characters (such as ‥ U+2025, … U+2026) i.e. characters from the Unicode line breaking class `IN`. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-writing-system-japanese①"></a>

  <a id="ref-for-writing-system-chinese①"></a>

  <a id="ref-for-content-writing-system②"></a>

  <a id="ref-for-valdef-line-break-loose④"></a>

  The following breaks are allowed for [loose](#valdef-line-break-loose) if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese) and are otherwise forbidden:

  - breaks before certain centered punctuation marks:  
    ・ U+30FB, ： U+FF1A, ； U+FF1B, ･ U+FF65, ‼ U+203C, ⁇ U+2047, ⁈ U+2048, ⁉ U+2049, ！ U+FF01, ？ U+FF1F

  - <a id="ref-for-unicode-east-asian-width"></a>

    breaks before suffixes:  
    Characters with the Unicode line breaking class `PO` [\[UAX14\]](#biblio-uax14) and the [East Asian Width property](#unicode-east-asian-width) [\[UAX11\]](#biblio-uax11) `Ambiguous`, `Fullwidth`, or `Wide`.

  - <a id="ref-for-unicode-east-asian-width①"></a>

    breaks after prefixes:  
    Characters with the Unicode line breaking class `PR` [\[UAX14\]](#biblio-uax14) and the [East Asian Width property](#unicode-east-asian-width) [\[UAX11\]](#biblio-uax11) `Ambiguous`, `Fullwidth`, or `Wide`.

<a id="ref-for-propdef-line-break⑤"></a>

<a id="ref-for-writing-system-chinese②"></a>

<a id="ref-for-writing-system-japanese②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The requirements listed above only create distinctions in CJK text. In an implementation that matches only the rules above, and no additional rules, [line-break](#propdef-line-break) would only affect CJK code points unless the writing system is tagged as [Chinese](#writing-system-chinese) or [Japanese](#writing-system-japanese). Future levels may add additional specific rules for other writing systems and languages as their requirements become known.

<a id="ref-for-valdef-line-break-strict②"></a>

<a id="ref-for-valdef-line-break-normal③"></a>

<a id="ref-for-valdef-line-break-loose⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-038e656c"></a> As UAs can add additional distinctions between [strict](#valdef-line-break-strict)/[normal](#valdef-line-break-normal)/[loose](#valdef-line-break-loose) modes, these values can exhibit differences in other writing systems as well. For example, a UA with sufficiently-advanced Thai language processing ability could choose to map different levels of strictness in Thai line-breaking to these keywords, e.g. disallowing breaks within compound words in <a id="ref-for-valdef-line-break-strict③"></a>strict mode (e.g. breaking ตัวอย&#xE48;างการเขียนภาษาไทย as ตัวอย&#xE48;าง·การเขียน·ภาษาไทย) while allowing more breaks in <a id="ref-for-valdef-line-break-loose⑥"></a>loose (ตัวอย&#xE48;าง·การ·เขียน·ภาษา·ไทย).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSSWG recognizes that in a future edition of the specification finer control over line breaking may be necessary to satisfy high-end publishing requirements.

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

<a id="ref-for-propdef-hyphens①"></a>

### <a id="hyphenation"></a>5.3. <a id="hyphens-property"></a> Hyphenation: the [hyphens](#propdef-hyphens) property

<a id="hyphenate"></a>Hyphenation is the controlled splitting of words where they usually would not be allowed to break to improve the layout of paragraphs, typically splitting words at syllabic or morphemic boundaries and often visually indicating the split (usually by inserting a hyphen, U+2010). In some cases, hyphenation may also alter the spelling of a word. Regardless, hyphenation is a rendering effect only: it must have no effect on the underlying document content or on text selection or searching.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-17746cb4"></a> Hyphenation practices vary across languages, and can involve not just inserting a hyphen before the line break, but inserting a hyphen after the break (or both), inserting a different character than U+2010, or changing the spelling of the word.
>
> **Table 8**
>
> Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.
>
> | Language | Unbroken | Before | After |
> | --- | --- | --- | --- |
> | English | Unbroken | Un‐ | broken |
> | Dutch | cafeetje | café‐ | tje |
> | Hungarian | Összeg | Ösz‐ | szeg |
> | Mandarin | tú’àn | tú‐ | àn |
> | Mandarin | àizēng‐fēnmíng | àizēng‐ | ‐fēnmíng |
> | Uyghur | ![ \[isolated DAL + isolated ALEF + initial MEEM + medial YEH + final DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/uyghur-unbroken.svg) | ![\[isolated DAL + isolated ALEF + initial MEEM + final YEH + hyphen \]](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/uyghur-hyphenate-joined-before.svg) | ![\[ isolated DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/uyghur-hyphenate-joined-after.svg) |
> | Cree | ![\[ᑲᓯᑕᓂᐘᓂᓂᐠ\] (CANADIAN SYLLABICS KA + CANADIAN SYLLABICS SI + CANADIAN SYLLABICS TA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS WEST-CREE WA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS FINAL GRAVE)](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/cree.svg) | ![\[ᑲᓯᑕᓂ᐀\] (CANADIAN SYLLABICS KA + CANADIAN SYLLABICS SI + CANADIAN SYLLABICS TA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS HYPHEN)](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/cree-before.svg) | ![\[ᐘᓂᓂᐠ\] (CANADIAN SYLLABICS WEST-CREE WA + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS NI + CANADIAN SYLLABICS FINAL GRAVE)](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/cree-after.svg) |

<a id="ref-for-soft-wrap-opportunity②⓪"></a>

<a id="ref-for-hyphenate"></a>

<a id="ref-for-hyphenation-opportunity"></a>

<a id="ref-for-propdef-hyphens②"></a>

Hyphenation occurs when the line breaks at a valid <a id="hyphenation-opportunity"></a>hyphenation opportunity, which is a type of [soft wrap opportunity](#soft-wrap-opportunity) that exists within a word where [hyphenation](#hyphenate) is allowed. In CSS [hyphenation opportunities](#hyphenation-opportunity) are controlled with the [hyphens](#propdef-hyphens) property. CSS Text Level 3 does not define the exact rules for <a id="ref-for-hyphenate①"></a>hyphenation; however UAs are strongly encouraged to optimize their choice of break points and to chose language-appropriate hyphenation points.

<a id="ref-for-soft-wrap-opportunity②①"></a>

<a id="ref-for-hyphenation-opportunity①"></a>

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
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) manual <a id="ref-for-comb-one①⑦"></a>\| auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | manual                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                     |

<a id="ref-for-hyphenate②"></a>

<a id="ref-for-soft-wrap-opportunity②②"></a>

This property controls whether [hyphenation](#hyphenate) is allowed to create more [soft wrap opportunities](#soft-wrap-opportunity) within a line of text. Values have the following meanings:

<a id="valdef-hyphens-none"></a>none  
<a id="ref-for-hyphenation-opportunity②"></a>

Words are not hyphenated, even if characters inside the word explicitly define [hyphenation opportunities](#hyphenation-opportunity).

<a id="ref-for-soft-wrap-opportunity②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This does not suppress the existing [soft wrap opportunities](#soft-wrap-opportunity) introduced by always visible characters such as U+002D - HYPHEN-MINUS or U+2010 ‐ HYPHEN.

<a id="valdef-hyphens-manual"></a>manual  
<a id="ref-for-hyphenation-opportunity③"></a>

Words are only hyphenated where there are characters inside the word that explicitly suggest [hyphenation opportunities](#hyphenation-opportunity). The UA must use the appropriate language-specific hyphenation character(s) and should apply any appropriate spelling changes just as for automatic hyphenation at the same point.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b6db0d14"></a> In Unicode, U+00AD is a conditional "soft hyphen" and U+2010 is an unconditional hyphen. Unicode Standard Annex \#14 describes the [role of soft hyphens in](http://unicode.org/reports/tr14/#SoftHyphen) Unicode line breaking. [\[UAX14\]](#biblio-uax14) In HTML, &#x26;shy; represents the soft hyphen character, which suggests a hyphenation opportunity.
> ```text
> ex&shy;ample
> ```
<a id="valdef-hyphens-auto"></a>auto  
<a id="ref-for-hyphenation-opportunity④"></a>

Words may be broken at [hyphenation opportunities](#hyphenation-opportunity) determined automatically by a language-appropriate hyphenation resource in addition to those indicated explicitly by a conditional hyphen. Automatic <a id="ref-for-hyphenation-opportunity⑤"></a>hyphenation opportunities elsewhere within a word must be ignored if the word contains a conditional hyphen (&#x26;shy; or U+00AD SOFT HYPHEN), in favor of the conditional hyphen(s). However, if, even after breaking at such opportunities, a portion of that word is still too long to fit on one line, an automatic hyphenation opportunity may be used.

<a id="ref-for-content-language①⓪"></a>

Correct automatic hyphenation requires a hyphenation resource appropriate to the language of the text being broken. The UA must therefore only automatically hyphenate text for which the [content language](#content-language) is known and for which it has an appropriate hyphenation resource.

<a id="ref-for-content-language①①"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should correctly tag their content’s [language](#content-language) (e.g. using the HTML `lang` attribute or XML `xml:lang` attribute) in order to obtain correct automatic hyphenation.

The UA may use language-tailored heuristics to exclude certain words from automatic hyphenation. For example, a UA might try to avoid hyphenation in proper nouns by excluding words matching certain capitalization and punctuation patterns. Such heuristics are not defined by this specification. (Note that such heuristics will need to vary by language: English and German, for example, have very different capitalization conventions.)

<a id="ref-for-propdef-hyphens③"></a>

<a id="ref-for-out-of-flow②"></a>

For the purpose of the [hyphens](#propdef-hyphens) property, what constitutes a “word” is UA-dependent. However, inline element boundaries and [out-of-flow](https://www.w3.org/TR/css-display-4/#out-of-flow) elements must be ignored when determining word boundaries.

<a id="ref-for-hyphenation-opportunity⑥"></a>

Any glyphs shown due to hyphenation at a [hyphenation opportunity](#hyphenation-opportunity) created by a conditional hyphen character (such as U+00AD SOFT HYPHEN) are represented by that character and are styled according to the properties applied to it.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When shaping scripts such as Arabic are allowed to break within words due to hyphenation, the characters are still shaped as if the word were [not broken](#word-break-shaping).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4faf88c1"></a> For example, if the Uyghur word “داميدى” were hyphenated, it would appear as ![\[isolated DAL + isolated ALEF + initial MEEM + medial YEH + hyphen + line-break + final DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/uyghur-hyphenate-joined.svg) not as ![\[isolated DAL + isolated ALEF + initial MEEM + final YEH + hyphen + line-break + isolated DAL + isolated ALEF MAKSURA\]](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/uyghur-hyphenate-unjoined.svg) .

<a id="ref-for-propdef-overflow-wrap④"></a>

<a id="ref-for-propdef-word-wrap"></a>

### <a id="overflow-wrap-property"></a>5.4. <a id="overflow-wrap"></a> Overflow Wrapping: the [overflow-wrap](#propdef-overflow-wrap) ([word-wrap](#propdef-word-wrap)) property

| Field               | Definition                                                                                                             |
|---------------------|------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-overflow-wrap"></a>overflow-wrap                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑧"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) break-word <a id="ref-for-comb-one①⑨"></a>\| anywhere |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                               |

<a id="ref-for-propdef-white-space①⑦"></a>

<a id="ref-for-wrapping⑦"></a>

This property specifies whether the UA may break at otherwise disallowed points within a line to prevent overflow, when an otherwise-unbreakable string is too long to fit within the line box. It only has an effect when [white-space](#propdef-white-space) allows [wrapping](#wrapping). Possible values:

<a id="valdef-overflow-wrap-normal"></a>normal  
<a id="ref-for-propdef-word-break①⑥"></a>

Lines may break only at allowed break points. However, the restrictions introduced by [word-break: keep-all](#propdef-word-break) may be relaxed to match <a id="ref-for-propdef-word-break①⑦"></a>word-break: normal if there are no otherwise-acceptable break points in the line.

<a id="valdef-overflow-wrap-anywhere"></a>anywhere  
<a id="ref-for-min-content①"></a>

<a id="ref-for-valdef-overflow-wrap-anywhere"></a>

<a id="ref-for-soft-wrap-opportunity②④"></a>

<a id="ref-for-character①①"></a>

An otherwise unbreakable sequence of [characters](#character) may be broken at an arbitrary point if there are no otherwise-acceptable break points in the line. Shaping characters are still shaped as if the word were not broken, and grapheme clusters must stay together as one unit. No hyphenation character is inserted at the break point. [Soft wrap opportunities](#soft-wrap-opportunity) introduced by [anywhere](#valdef-overflow-wrap-anywhere) <em>are considered</em> when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

<a id="valdef-overflow-wrap-break-word"></a>break-word  
<a id="ref-for-min-content②"></a>

<a id="ref-for-valdef-overflow-wrap-break-word"></a>

<a id="ref-for-soft-wrap-opportunity②⑤"></a>

<a id="ref-for-valdef-overflow-wrap-anywhere①"></a>

As for [anywhere](#valdef-overflow-wrap-anywhere) except that [soft wrap opportunities](#soft-wrap-opportunity) introduced by [break-word](#valdef-overflow-wrap-break-word) are <em>not</em> considered when calculating [min-content intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#min-content).

<a id="ref-for-legacy-name-alias"></a>

<a id="ref-for-propdef-overflow-wrap⑤"></a>

For legacy reasons, UAs must treat <a id="propdef-word-wrap"></a>word-wrap as a [legacy name alias](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias) of the [overflow-wrap](#propdef-overflow-wrap) property.

### <a id="line-break-details"></a>5.5.  Line Breaking Details

<a id="ref-for-line-break"></a>

When determining [line breaks](#line-break):

- <a id="ref-for-line-breaking-process①"></a>

  <a id="line-breaking-bidi"></a> The interaction of [line breaking](#line-breaking-process) and bidirectional text is defined by [CSS Writing Modes 4 § 2.4 Applying the Bidirectional Reordering Algorithm](https://www.w3.org/TR/css-writing-modes-4/#bidi-algo) and the Unicode Bidirectional Algorithm ([UAX9§3.4 Reordering Resolved Levels](http://unicode.org/reports/tr9/#Reordering_Resolved_Levels) in particular). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4) [\[UAX9\]](#biblio-uax9)

- <a id="ref-for-forced-line-break④"></a>

  <a id="ref-for-propdef-white-space①⑧"></a>

  <a id="ref-for-segment-break①⑥"></a>

  <a id="unicode-forced-breaks"></a> Preserved [segment breaks](#segment-break), and—​regardless of the [white-space](#propdef-white-space) value—​any Unicode character with the `BK` and `NL` line breaking class, must be treated as [forced line breaks](#forced-line-break). [\[UAX14\]](#biblio-uax14)

  <a id="ref-for-forced-line-break⑤"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The bidi implications of such [forced line breaks](#forced-line-break) are defined by the [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/). [\[UAX9\]](#biblio-uax9)

- <a id="ref-for-propdef-overflow-wrap⑥"></a>

  <a id="ref-for-propdef-line-break⑥"></a>

  <a id="unicode-glue"></a> Except where explicitly defined otherwise (e.g. for [line-break: anywhere](#propdef-line-break) or [overflow-wrap: anywhere](#propdef-overflow-wrap)) line breaking behavior defined for the `WJ`, `ZW`, `GL`, and `ZWJ` Unicode line breaking classes must be honored. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-typographic-character-unit①⑦"></a>

  <a id="ref-for-soft-wrap-opportunity②⑥"></a>

  <a id="unicode-unbreakable"></a> CSS never allows [soft wrap opportunities](#soft-wrap-opportunity) within [typographic character units](#typographic-character-unit). Thus, the `CM` and `SG` Unicode line breaking classes must always be honored. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-propdef-word-break①⑧"></a>

  <a id="ref-for-soft-wrap-opportunity②⑦"></a>

  <a id="ref-for-propdef-line-break⑦"></a>

  <a id="ref-for-word-separator①"></a>

  <a id="break-prioritization"></a> UAs that allow wrapping at punctuation other than [word separators](#word-separator) in writing systems that use them <em>should</em> prioritize breakpoints. (For example, if breaks after slashes are given a lower priority than spaces, the sequence “check /etc” will never break between the "/" and the "e".) As long as care is taken to avoid such awkward breaks, allowing breaks at appropriate punctuation other than <a id="ref-for-word-separator②"></a>word separators is recommended, as it results in more even-looking margins, particularly in narrow measures. The UA may use the width of the containing block, the text’s language, the [line-break](#propdef-line-break) value, and other factors in assigning priorities: CSS does not define prioritization of [soft wrap opportunities](#soft-wrap-opportunity). Prioritization of <a id="ref-for-word-separator③"></a>word separators is not expected, however, if [word-break: break-all](#propdef-word-break) is specified (since this value explicitly requests line breaking behavior not based on breaking at <a id="ref-for-word-separator④"></a>word separators)—​and is forbidden under <a id="ref-for-propdef-line-break⑧"></a>line-break: anywhere.

- <a id="ref-for-soft-wrap-opportunity②⑧"></a>

  <a id="ref-for-forced-line-break⑥"></a>

  <a id="ref-for-inline-box②"></a>

  <a id="ref-for-out-of-flow③"></a>

  <a id="ignored-boxes"></a> [Out-of-flow boxes](https://www.w3.org/TR/css-display-3/#out-of-flow) and [inline box](https://www.w3.org/TR/css-display-4/#inline-box) boundaries do not introduce a [forced line break](#forced-line-break) or [soft wrap opportunity](#soft-wrap-opportunity) in the flow.

- <a id="ref-for-atomic-inline"></a>

  <a id="ref-for-soft-wrap-opportunity②⑨"></a>

  <a id="atomic-compat-wrap"></a> For Web-compatibility there is a [soft wrap opportunity](#soft-wrap-opportunity) before and after each replaced element or other [atomic inline](https://www.w3.org/TR/css-display-4/#atomic-inline), even when adjacent to a character that would normally suppress them, including U+00A0 NO-BREAK SPACE. However, with the exception of U+00A0 NO-BREAK SPACE, there must be no <a id="ref-for-soft-wrap-opportunity③⓪"></a>soft wrap opportunity between <a id="ref-for-atomic-inline①"></a>atomic inlines and adjacent characters belonging to the Unicode GL, WJ, or ZWJ line breaking classes. [\[UAX14\]](#biblio-uax14)

- <a id="ref-for-propdef-overflow-wrap⑦"></a>

  <a id="ref-for-propdef-word-break①⑨"></a>

  <a id="ref-for-propdef-line-break⑨"></a>

  <a id="ref-for-propdef-white-space①⑨"></a>

  <a id="ref-for-atomic-inline②"></a>

  <a id="ref-for-soft-wrap-opportunity③①"></a>

  <a id="line-breaking-scope"></a> For [soft wrap opportunities](#soft-wrap-opportunity) created by characters that disappear at the line break (e.g. U+0020 SPACE), properties on the box directly containing that character control the line breaking at that opportunity. For <a id="ref-for-soft-wrap-opportunity③②"></a>soft wrap opportunities defined by the boundary between two characters or [atomic inlines](https://www.w3.org/TR/css-display-4/#atomic-inline), the [white-space](#propdef-white-space) property on the nearest common ancestor of the two characters controls breaking; which elements’ [line-break](#propdef-line-break), [word-break](#propdef-word-break), and [overflow-wrap](#propdef-overflow-wrap) properties control the determination of <a id="ref-for-soft-wrap-opportunity③③"></a>soft wrap opportunities at such boundaries is undefined in this level.

- <a id="ref-for-soft-wrap-opportunity③④"></a>

  <a id="line-breaking-box-edges"></a> For [soft wrap opportunities](#soft-wrap-opportunity) before the first or after the last character of a box, the break occurs immediately before/after the box (at its margin edge) rather than breaking the box between its content edge and the content.

- <a id="line-breaking-ruby"></a> Line breaking in/around Ruby is defined in [CSS Ruby Annotation Layout 1 § 3.4 Breaking Across Lines](https://www.w3.org/TR/css-ruby-1/#line-breaks). [\[CSS-RUBY-1\]](#biblio-css-ruby-1)

- <a id="ref-for-hyphenate③"></a>

  <a id="ref-for-propdef-overflow-wrap⑧"></a>

  <a id="ref-for-propdef-line-break①⓪"></a>

  <a id="ref-for-propdef-word-break②⓪"></a>

  <a id="ref-for-soft-wrap-opportunity③⑤"></a>

  <a id="ref-for-wrapping⑧"></a>

  <a id="word-break-shaping"></a> When shaping scripts such as Arabic [wrap](#wrapping) at unforced [soft wrap opportunities](#soft-wrap-opportunity) within words (such as when breaking due to [word-break: break-all](#propdef-word-break), [line-break: anywhere](#propdef-line-break), [overflow-wrap: break-word](#propdef-overflow-wrap), <a id="ref-for-propdef-overflow-wrap⑨"></a>overflow-wrap: anywhere, or when [hyphenating](#hyphenate)) the characters must still be shaped (their joining forms chosen) as if the word were still whole.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-433d41a3"></a> For example, if the word “نوشتن” is broken between the “ش” and “ت”, the “ش” still takes its initial form (“ﺷ”), and the “ت” its medial form (“ﺘ”)—​forming as in “ﻧﻮﺷ \| ﺘﻦ”, not as in “نوش \| تن”.

- <a id="ref-for-typographic-letter-unit⑥"></a>

  <a id="ref-for-soft-wrap-opportunity③⑥"></a>

  <a id="ref-for-content-language①②"></a>

  <a id="fallback-breaking"></a> In order to avoid unexpected overflow, if the user agent is unable to perform the requisite lexical or orthographic analysis for line breaking any [content language](#content-language) that requires it—​for example due to lacking a dictionary for certain languages—​it must assume a [soft wrap opportunity](#soft-wrap-opportunity) between pairs of [typographic letter units](#typographic-letter-unit) in that writing system.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This provision is not triggered merely when the UA fails to find a word boundary in a particular text run; the text run may well be a single unbreakable word. It applies for example when a text run is composed of Khmer characters (U+1780 to U+17FF) if the user agent does not know how to determine word boundaries in Khmer.

## <a id="justification"></a>6.  Alignment and Justification

Alignment and justification controls how inline content is distributed within a line box.

<a id="ref-for-propdef-text-align"></a>

### <a id="text-align-property"></a>6.1. <a id="text-align"></a> Text Alignment: the [text-align](#propdef-text-align) shorthand

| Field               | Definition                                                                                                                                                                                                                                                             |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align"></a>text-align                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⓪"></a>start [\|](https://www.w3.org/TR/css-values-4/#comb-one) end <a id="ref-for-comb-one②①"></a>\| left <a id="ref-for-comb-one②②"></a>\| right <a id="ref-for-comb-one②③"></a>\| center <a id="ref-for-comb-one②④"></a>\| justify <a id="ref-for-comb-one②⑤"></a>\| match-parent <a id="ref-for-comb-one②⑥"></a>\| justify-all |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | start                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                    |

<a id="ref-for-shorthand-property"></a>

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

<a id="valdef-text-align-justify"></a>justify  
<a id="ref-for-valdef-text-align-start"></a>

<a id="ref-for-forced-line-break⑦"></a>

<a id="ref-for-propdef-text-align-last②"></a>

<a id="ref-for-propdef-text-justify①"></a>

Text is justified according to the method specified by the [text-justify](#propdef-text-justify) property, in order to exactly fill the line box. Unless otherwise specified by [text-align-last](#propdef-text-align-last), the last line before a [forced line break](#forced-line-break) is [start](#valdef-text-align-start)-aligned.

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

This value behaves the same as [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) (computes to its parent’s computed value) except that an [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) of [start](#valdef-text-align-start) or [end](#valdef-text-align-end) is interpreted against the parent’s [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) value and results in a computed value of either [left](#valdef-text-align-left) or [right](#valdef-text-align-right). Computes to <a id="ref-for-valdef-text-align-start②"></a>start when specified on the [root element](https://www.w3.org/TR/css-display-4/#root-element).

<a id="ref-for-propdef-text-align①"></a>

<a id="ref-for-propdef-text-align-all③"></a>

<a id="ref-for-propdef-text-align-last④"></a>

<a id="ref-for-valdef-text-align-match-parent①"></a>

When specified on the [text-align](#propdef-text-align) shorthand, sets both [text-align-all](#propdef-text-align-all) and [text-align-last](#propdef-text-align-last) to [match-parent](#valdef-text-align-match-parent).

<a id="ref-for-line-box"></a>

A block of text is a stack of [line boxes](https://www.w3.org/TR/css-inline-3/#line-box). This property specifies how the inline-level boxes within each line box align with respect to the start and end sides of the line box. Alignment is not with respect to the [viewport](https://www.w3.org/TR/CSS2/visuren.html#viewport) or containing block.

<a id="ref-for-valdef-text-align-justify①"></a>

<a id="ref-for-propdef-text-justify②"></a>

<a id="ref-for-white-space②①"></a>

<a id="ref-for-collapsible-white-space⑨"></a>

<a id="ref-for-justification-opportunity"></a>

<a id="ref-for-tab-stop②"></a>

In the case of [justify](#valdef-text-align-justify), the UA may stretch or shrink any inline boxes by [adjusting](#text-justify-property) their text. (See [text-justify](#propdef-text-justify).) If an element’s [white space](#white-space) is not [collapsible](#collapsible-white-space), then the UA is not required to adjust its text for the purpose of justification and may instead treat the text as having no [justification opportunities](#justification-opportunity). If the UA chooses to adjust the text, then it must ensure that [tab stops](#tab-stop) continue to line up as required by the [white space processing rules](#white-space-rules).

<a id="ref-for-start①"></a>

<a id="ref-for-end①"></a>

If (after justification, if any) the inline contents of a line box are too long to fit within it, then the contents are [start](https://www.w3.org/TR/css-writing-modes-4/#start)-aligned: any content that doesn’t fit overflows the line box’s [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge.

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

See [§ 8.3 Bidirectionality and Line Boxes](#bidi-linebox) for details on how to determine the [start](https://www.w3.org/TR/css-writing-modes-4/#start) and [end](https://www.w3.org/TR/css-writing-modes-4/#end) edges of a line box.

<a id="ref-for-propdef-text-align-all④"></a>

### <a id="text-align-all-property"></a>6.2. <a id="text-align-all"></a> Default Text Alignment: the [text-align-all](#propdef-text-align-all) property

| Field               | Definition                                                                                                                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align-all"></a>text-align-all                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⑦"></a>start [\|](https://www.w3.org/TR/css-values-4/#comb-one) end <a id="ref-for-comb-one②⑧"></a>\| left <a id="ref-for-comb-one②⑨"></a>\| right <a id="ref-for-comb-one③⓪"></a>\| center <a id="ref-for-comb-one③①"></a>\| justify <a id="ref-for-comb-one③②"></a>\| match-parent |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | start                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-align-match-parent②"></a>keyword as specified, except for [match-parent](#valdef-text-align-match-parent) which computes as defined above                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                             |

<a id="ref-for-propdef-text-align②"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-valdef-text-align-last-auto①"></a>

<a id="ref-for-propdef-text-align-last⑤"></a>

This longhand of the [text-align](#propdef-text-align) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) specifies the inline alignment of all lines of inline content in the block container, except for last lines overridden by a non-[auto](#valdef-text-align-last-auto) value of [text-align-last](#propdef-text-align-last). See <a id="ref-for-propdef-text-align③"></a>text-align for a full description of values.

<a id="ref-for-propdef-text-align④"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should use the [text-align](#propdef-text-align) shorthand instead of this property.

<a id="ref-for-propdef-text-align-last⑥"></a>

### <a id="text-align-last-property"></a>6.3. <a id="text-align-last"></a> Last Line Alignment: the [text-align-last](#propdef-text-align-last) property

| Field               | Definition                                                                                                                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-align-last"></a>text-align-last                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③③"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) start <a id="ref-for-comb-one③④"></a>\| end <a id="ref-for-comb-one③⑤"></a>\| left <a id="ref-for-comb-one③⑥"></a>\| right <a id="ref-for-comb-one③⑦"></a>\| center <a id="ref-for-comb-one③⑧"></a>\| justify <a id="ref-for-comb-one③⑨"></a>\| match-parent |
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

<a id="ref-for-valdef-text-align-start③"></a>

<a id="ref-for-propdef-text-align⑤"></a>

If <a id="valdef-text-align-last-auto"></a>auto is specified, content on the affected line is aligned per [text-align-all](#propdef-text-align-all) unless <a id="ref-for-propdef-text-align-all⑥"></a>text-align-all is set to [justify](#valdef-text-align-justify), in which case it is [start](#valdef-text-align-start)-aligned. All other values are interpreted as described for [text-align](#propdef-text-align).

<a id="ref-for-propdef-text-justify③"></a>

### <a id="text-justify-property"></a>6.4. <a id="text-justify"></a> Justification Method: the [text-justify](#propdef-text-justify) property

| Field               | Definition                                                                                                                                             |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-justify"></a>text-justify                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one④①"></a>\| inter-word <a id="ref-for-comb-one④②"></a>\| inter-character |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-justify-distribute"></a>specified keyword (except for the [distribute](#valdef-text-justify-distribute) legacy value)                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                               |

<a id="ref-for-valdef-text-align-justify③"></a>

<a id="ref-for-propdef-text-align⑥"></a>

This property selects the justification method used when a line’s alignment is set to [justify](#valdef-text-align-justify) (see [text-align](#propdef-text-align)). The property applies to text, but is inherited from block containers to the root inline box containing their inline-level contents. It takes the following values:

<a id="valdef-text-justify-auto"></a>auto  
<a id="ref-for-content-language①③"></a>

<a id="ref-for-content-writing-system③"></a>

The UA determines the justification algorithm to follow, based on a balance between performance and adequate presentation quality. Since justification rules vary by [writing system](#content-writing-system) and [language](#content-language), UAs should, where possible, use a justification algorithm appropriate to the text.

<a id="ref-for-word-separator⑤"></a>

<a id="ref-for-typographic-letter-unit⑦"></a>

<a id="ref-for-content-language①④"></a>

<a id="ref-for-valdef-text-justify-inter-word"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0bd57ccb"></a> For example, the UA could use by default a justification method that is a simple universal compromise for all writing systems—​such as primarily expanding [word separators](#word-separator) and between CJK [typographic letter units](#typographic-letter-unit) along with secondarily expanding between Southeast Asian <a id="ref-for-typographic-letter-unit⑧"></a>typographic letter units. Then, in cases where the [content language](#content-language) of the paragraph is known, it could choose a more language-tailored justification behavior e.g. following the [Requirements for Japanese Text Layout](https://www.w3.org/TR/jlreq/) for Japanese [\[JLREQ\]](#biblio-jlreq), using cursive elongation for Arabic, using [inter-word](#valdef-text-justify-inter-word) for German, etc.

<a id="fig-text-justify-cursive"></a>

![Two lines of calligraphic Arabic end together due to a mix of compressed and swash forms.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/text-justify-cursive.png "Swash forms elongate the first line
				            while a compressed contextual ligature shortens the second,
				            allowing both to end precisely together.")

An example of cursively-justified Arabic text, rendered by [Tasmeem](https://www.decotype.com/). Like English, Arabic can be justified by adjusting the spacing between words, but in most styles it can also be justified by calligraphically elongating or compressing the letterforms themselves. In this example, the upper text is extended to fill the line by the use of elongated (kashida) forms and swash forms, while the bottom line is compressed slightly by using a stacked combination for the characters between ت and م. By employing traditional calligraphic techniques, a typesetter can justify the line while preserving flow and color, providing a very high quality justification effect. However, this is by its nature a very script-specific effect.

<a id="fig-text-justify-compromise"></a> ![Extra space is partly to spaces and partly among CJK and Thai letters.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/text-justify-compromise.png)

<a id="ref-for-propdef-text-justify④"></a>

Mixed-script text with [text-justify: auto](#propdef-text-justify): this interpretation uses a universal-compromise justification method, expanding at spaces as well as between CJK and Southeast Asian letters. This effectively uses inter-word + inter-ideograph spacing for lines that have word-separators and/or CJK characters and falls back to inter-cluster behavior for lines that don’t or for which the space stretches too far.

<a id="valdef-text-justify-none"></a>none  
<a id="ref-for-justification-opportunity①"></a>

Justification is disabled: there are no [justification opportunities](#justification-opportunity) within the text.

<a id="fig-text-justify-none"></a> ![No extra space is inserted.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/text-justify-none.png)

<a id="ref-for-propdef-text-justify⑤"></a>

Mixed-script text with [text-justify: none](#propdef-text-justify)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is intended for use in user stylesheets to improve readability or for accessibility purposes.

<a id="valdef-text-justify-inter-word"></a>inter-word  
<a id="ref-for-propdef-word-spacing①"></a>

<a id="ref-for-word-separator⑥"></a>

Justification adjusts spacing at [word separators](#word-separator) only (effectively varying the used [word-spacing](#propdef-word-spacing) on the line). This behavior is typical for languages that separate words using spaces, like English or Korean.

<a id="fig-text-justify-interword"></a> ![Extra space is equally distributed mainly to spaces.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/text-justify-interword.png)

<a id="ref-for-propdef-text-justify⑥"></a>

Mixed-script text with [text-justify: inter-word](#propdef-text-justify)

<a id="valdef-text-justify-inter-character"></a>inter-character  
<a id="ref-for-propdef-letter-spacing①"></a>

<a id="ref-for-typographic-character-unit①⑧"></a>

Justification adjusts spacing between each pair of adjacent [typographic character units](#typographic-character-unit) (effectively varying the used [letter-spacing](#propdef-letter-spacing) on the line). This value is sometimes used in East Asian systems such as Japanese.

<a id="fig-text-justify-distribute"></a> ![Extra space is equally distributed at points between spaces and letters of all writing systems.](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/text-justify-distribute.png)

<a id="ref-for-propdef-text-justify⑦"></a>

Mixed-script text with [text-justify: inter-character](#propdef-text-justify)

<a id="ref-for-valdef-text-justify-inter-character"></a>

<a id="ref-for-css-legacy-value-alias"></a>

For legacy reasons, UAs must also support the alternate keyword <a id="valdef-text-justify-distribute"></a>distribute which must compute to [inter-character](#valdef-text-justify-inter-character), thus having the exact same meaning and behavior. UAs may treat this as a [legacy value alias](https://www.w3.org/TR/css-cascade-5/#css-legacy-value-alias).

<a id="ref-for-content-language①⑤"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Since optimal justification is [language](#content-language)-sensitive, authors should correctly language-tag their content for the best results.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The guidelines in this level of CSS do not describe a complete justification algorithm. They are merely a minimum set of requirements that a complete algorithm should meet. Limiting the set of requirements gives UAs some latitude in choosing a justification algorithm that meets their needs and desired balance of quality, speed, and complexity.

#### <a id="expanding-text"></a>6.4.1.  Expanding and Compressing Text

When justifying text, the user agent takes the remaining space between the ends of a line’s contents and the edges of its line box, and distributes that space throughout its content so that the contents exactly fill the line box. The user agent may alternatively distribute negative space, putting more content on the line than would otherwise fit under normal spacing conditions.

<a id="ref-for-typographic-character-unit①⑨"></a>

<a id="ref-for-word-separator⑦"></a>

<a id="ref-for-justification-opportunity②"></a>

<a id="ref-for-propdef-text-justify⑧"></a>

A <a id="justification-opportunity"></a><a id="expansion-opportunity"></a>justification opportunity is a point where the justification algorithm may alter spacing within the text. A justification opportunity can be provided by a single [typographic character unit](#typographic-character-unit) (such as a [word separator](#word-separator)), or by the juxtaposition of two <a id="ref-for-typographic-character-unit②⓪"></a>typographic character units. As with controls for [soft wrap opportunities](#line-break-details), whether a <a id="ref-for-typographic-character-unit②①"></a>typographic character unit provides a [justification opportunity](#justification-opportunity) is controlled by the [text-justify](#propdef-text-justify) value of its parent; similarly, whether a <a id="ref-for-justification-opportunity③"></a>justification opportunity exists between two consecutive <a id="ref-for-typographic-character-unit②②"></a>typographic character units is determined by the <a id="ref-for-propdef-text-justify⑨"></a>text-justify value of their nearest common ancestor.

<a id="ref-for-propdef-letter-spacing②"></a>

<a id="ref-for-propdef-word-spacing②"></a>

<a id="ref-for-word-separator⑧"></a>

<a id="ref-for-justification-opportunity④"></a>

<a id="ref-for-typographic-character-unit②③"></a>

Space distributed by justification is <em>in addition to</em> the spacing defined by the [letter-spacing](#propdef-letter-spacing) or [word-spacing](#propdef-word-spacing) properties. When such additional space is distributed to a [word separator](#word-separator) [justification opportunity](#justification-opportunity), it is applied under the same rules as for <a id="ref-for-propdef-word-spacing③"></a>word-spacing. Similarly, when space is distributed to a <a id="ref-for-justification-opportunity⑤"></a>justification opportunity between two [typographic character units](#typographic-character-unit), should be applied under the same rules as for <a id="ref-for-propdef-letter-spacing③"></a>letter-spacing.

<a id="ref-for-justification-opportunity⑥"></a>

<a id="ref-for-typographic-character-unit②④"></a>

<a id="ref-for-valdef-text-justify-inter-character①"></a>

A justification algorithm may divide [justification opportunities](#justification-opportunity) into different priority levels. All <a id="ref-for-justification-opportunity⑦"></a>justification opportunities within a given level are expanded or compressed at the same priority, regardless of which [typographic character units](#typographic-character-unit) created that opportunity. For example, if <a id="ref-for-justification-opportunity⑧"></a>justification opportunities between two Han characters and between two Latin letters are defined to be at the same level (as they are in the [inter-character](#valdef-text-justify-inter-character) justification style), they are not treated differently because they originate from different <a id="ref-for-typographic-character-unit②⑤"></a>typographic character units. It is not defined in this level whether or how other factors (such as font size, letter-spacing, glyph shape, position within the line, etc.) may influence the distribution of space to <a id="ref-for-justification-opportunity⑨"></a>justification opportunities within the line.

The UA may enable or break optional ligatures or use other font features such as alternate glyphs or glyph compression to help justify the text under any method. This behavior is not controlled by this level of CSS. However, UAs <em>must not</em> break required ligatures or otherwise disable features required to correctly shape complex scripts.

<a id="ref-for-justification-opportunity①⓪"></a>

<a id="ref-for-valdef-text-align-justify④"></a>

If a [justification opportunity](#justification-opportunity) exists within a line, and [text alignment](#text-align-property) specifies full justification ([justify](#valdef-text-align-justify)) for that line, it must be justified.

#### <a id="justify-symbols"></a>6.4.2.  Handling Symbols and Punctuation

<a id="ref-for-justification-opportunity①①"></a>

<a id="ref-for-typographic-character-unit②⑥"></a>

<a id="ref-for-typographic-letter-unit⑨"></a>

When determining [justification opportunities](#justification-opportunity), a [typographic character unit](#typographic-character-unit) from the Unicode Symbols (S\*) and Punctuation (P\*) classes is generally treated the same as a [typographic letter unit](#typographic-letter-unit) of the same script (or, if the character’s script property is Common, then as a <a id="ref-for-typographic-letter-unit①⓪"></a>typographic letter unit of the dominant script).

<a id="ref-for-justification-opportunity①②"></a>

However, by typographic tradition there may be additional rules controlling the justification of symbols and punctuation. Therefore, the UA may reassign specific characters or introduce additional levels of prioritization to handle [justification opportunities](#justification-opportunity) involving symbols and punctuation.

<a id="ref-for-justification-opportunity①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3ce55d62"></a> For example, there are traditionally no [justification opportunities](#justification-opportunity) between consecutive U+2014 — EM DASH, U+2015 ― HORIZONTAL BAR, U+2026 … HORIZONTAL ELLIPSIS, or U+2025 ‥ TWO DOT LEADER characters [\[JLREQ\]](#biblio-jlreq); thus a UA might assign these characters to a “never” prioritization level. As another example, certain full-width punctuation characters (such as U+301A 〚 LEFT WHITE SQUARE BRACKET) are considered to contain a <a id="ref-for-justification-opportunity①④"></a>justification opportunity in Japanese. The UA might therefore assign these characters to a higher prioritization level than the opportunities between ideographic characters.

#### <a id="justify-limits"></a>6.4.3.  Unexpandable Text

<a id="ref-for-propdef-text-align-last⑦"></a>

<a id="ref-for-valdef-text-align-justify⑤"></a>

<a id="ref-for-valdef-text-align-center"></a>

If the inline contents of a line cannot be stretched to the full width of the line box, then they must be aligned as specified by the [text-align-last](#propdef-text-align-last) property. (If <a id="ref-for-propdef-text-align-last⑧"></a>text-align-last is [justify](#valdef-text-align-justify), then they must be aligned as for [center](#valdef-text-align-center).)

#### <a id="justify-cursive"></a>6.4.4.  Cursive Scripts

<a id="ref-for-typographic-letter-unit①①"></a>

<a id="ref-for-cursive-script"></a>

<a id="ref-for-justification-opportunity①⑤"></a>

Justification <em>must not</em> introduce gaps between the joined [typographic letter units](#typographic-letter-unit) of [cursive scripts](#cursive-script) such as Arabic. If it is able, the UA <em>may</em> translate space distributed to [justification opportunities](#justification-opportunity) within a run of such <a id="ref-for-typographic-letter-unit①②"></a>typographic letter units into some form of cursive elongation for that run. It otherwise <em>must</em> assume that no <a id="ref-for-justification-opportunity①⑥"></a>justification opportunity exists between any pair of <a id="ref-for-typographic-letter-unit①③"></a>typographic letter units in <a id="ref-for-cursive-script①"></a>cursive script (regardless of whether they join).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5407996d"></a> The following are examples of unacceptable justification:
>
> ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-spaced.png)
>
> Adding gaps between every pair of Arabic letters
>
> ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-unjoined.png)
>
> Adding gaps between every pair of unjoined Arabic letters

Some font designs allow for the use of the tatweel character for justification. A UA that performs tatweel-based justification must properly handle the rules for its use. Note that correct insertion of tatweel characters depends on context, including the letter-combinations involved, location within the word, and location of the word within the line.

<a id="ref-for-valdef-text-justify-auto"></a>

#### <a id="justify-algos"></a>6.4.5.  Minimum Requirements for [auto](#valdef-text-justify-auto) Justification

<a id="ref-for-valdef-text-justify-auto①"></a>

<a id="ref-for-justification-opportunity①⑦"></a>

For [auto](#valdef-text-justify-auto) justification, this specification does not define what all of the [justification opportunities](#justification-opportunity) are, how they are prioritized, or when and how multiple levels of <a id="ref-for-justification-opportunity①⑧"></a>justification opportunities interact. However, it does require that:

- <a id="ref-for-justification-opportunity①⑨"></a>

  <a id="ref-for-content-language①⑥"></a>

  Unless contraindicated by the typographic traditions of the [content language](#content-language) or adjacent symbols/punctuation, each of the following provides a [justification opportunity](#justification-opportunity):

  - <a id="ref-for-word-separator⑨"></a>

    [Word separators](#word-separator)

  - <a id="ref-for-block-scripts"></a>

    <a id="ref-for-typographic-character-unit②⑦"></a>

    The boundary between a [typographic character unit](#typographic-character-unit) of any [block scripts](#block-scripts) and any other <a id="ref-for-typographic-character-unit②⑧"></a>typographic character unit

  - <a id="ref-for-clustered-scripts"></a>

    <a id="ref-for-typographic-character-unit②⑨"></a>

    The boundary between a [typographic character unit](#typographic-character-unit) of any [clustered scripts](#clustered-scripts) and any other <a id="ref-for-typographic-character-unit③⓪"></a>typographic character unit

- <a id="ref-for-clustered-scripts①"></a>

  <a id="ref-for-block-scripts①"></a>

  <a id="ref-for-letter③"></a>

  All [letters](#letter) belonging to all [block scripts](#block-scripts) are treated the same, and all <a id="ref-for-letter④"></a>letters belonging to all [clustered scripts](#clustered-scripts) are treated the same. For example, no distinction is made between the justification opportunity between a Han letter followed by another Han letter, vs. the justification opportunity between a Han letter followed by a Hangul letter.

Further information on text justification can be found in (or submitted to) [“Approaches to Full Justification”](https://www.w3.org/International/articles/typography/justification), which indexes by writing system and language, and is maintained by the [W3C Internationalization Working Group](https://www.w3.org/International/). [\[JUSTIFY\]](#biblio-justify)

## <a id="spacing"></a>7.  Spacing

<a id="ref-for-propdef-word-spacing④"></a>

<a id="ref-for-propdef-letter-spacing④"></a>

<a id="ref-for-word-separator①⓪"></a>

<a id="ref-for-typographic-character-unit③①"></a>

CSS offers control over text spacing via the [word-spacing](#propdef-word-spacing) and [letter-spacing](#propdef-letter-spacing) properties, which specify additional space around [word separators](#word-separator) or between [typographic character units](#typographic-character-unit), respectively.

<a id="ref-for-propdef-word-spacing⑤"></a>

### <a id="word-spacing-property"></a>7.1. <a id="word-spacing"></a> Word Spacing: the [word-spacing](#propdef-word-spacing) property

| Field               | Definition                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-word-spacing"></a>word-spacing                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a><a id="ref-for-comb-one④③"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | an absolute length                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                         |

This property specifies additional spacing between “words”. Values are interpreted as defined below:

<a id="valdef-word-spacing-normal"></a>normal  
No additional spacing is applied. Computes to zero.

<a id="valdef-word-spacing-length"></a>\<length\>  
Specifies extra spacing <em>in addition to</em> the intrinsic inter-word spacing defined by the font.

<a id="ref-for-word-separator①①"></a>

Additional spacing is applied to each [word separator](#word-separator) left in the text after the [white space processing rules](#white-space-rules) have been applied, and should be applied half on each side of the character unless otherwise dictated by typographic tradition. Values may be negative, but there may be implementation-dependent limits.

<a id="ref-for-typographic-character-unit③②"></a>

<a id="word-separator"></a>Word-separator characters are [typographic character units](#typographic-character-unit) whose primary purpose and general usage is to separate words. In Unicode this includes (but is not exhaustively defined as) the space (U+0020), the no-break space (U+00A0), the Ethiopic word space (U+1361), the Aegean word separators (U+10100,U+10101), the Ugaritic word divider (U+1039F), and the Phoenician word separator (U+1091F). [\[UNICODE\]](#biblio-unicode)

<a id="ref-for-word-separator①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Neither punctuation in general, nor fixed-width spaces (such as U+3000 and U+2000 through U+200A), are considered [word-separator characters](#word-separator), because even though they frequently happen to separate words, their primary purpose is not to separate words.

<a id="ref-for-word-separator①③"></a>

If there are no [word-separator characters](#word-separator), or if a word-separating character has a zero advance width (such as U+200B ZERO WIDTH SPACE) then the user agent must not create an additional spacing between words.

<a id="ref-for-propdef-letter-spacing⑤"></a>

### <a id="letter-spacing-property"></a>7.2. <a id="letter-spacing"></a> Tracking: the [letter-spacing](#propdef-letter-spacing) property

| Field               | Definition                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-letter-spacing"></a>letter-spacing                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value②"></a><a id="ref-for-comb-one④④"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box③"></a>[inline boxes](https://www.w3.org/TR/css-display-4/#inline-box) and text                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | an absolute length                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | n/a                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                         |

<a id="ref-for-typographic-character-unit③③"></a>

<a id="ref-for-propdef-word-spacing⑥"></a>

This property specifies additional spacing (commonly called <a id="tracking"></a>tracking) between adjacent [typographic character units](#typographic-character-unit). Letter-spacing is applied after [bidi reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) and is in addition to [kerning](https://www.w3.org/TR/css-fonts-3/#font-kerning-prop) and [word-spacing](#propdef-word-spacing). [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4) [\[CSS-FONTS-3\]](#biblio-css-fonts-3) Depending on the justification rules in effect, user agents may further increase or decrease the space between <a id="ref-for-typographic-character-unit③④"></a>typographic character units in order to [justify text](#text-justify-property).

Values have the following meanings:

<a id="valdef-letter-spacing-normal"></a>normal  
No additional spacing is applied. Computes to zero.

<a id="valdef-letter-spacing-length"></a>\<length\>  
<a id="ref-for-typographic-character-unit③⑤"></a>

Specifies <em>additional</em> spacing between [typographic character units](#typographic-character-unit). Values may be negative, but there may be implementation-dependent limits.

<a id="ref-for-propdef-letter-spacing⑥"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-valdef-letter-spacing-normal"></a>

For [legacy reasons](https://github.com/w3c/csswg-drafts/issues/1484), a computed [letter-spacing](#propdef-letter-spacing) of zero yields a [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) (<code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> return value) of [normal](#valdef-letter-spacing-normal).

<a id="ref-for-typographic-character-unit③⑥"></a>

Letter-spacing is applied to each [typographic character unit](#typographic-character-unit) by inserting half of the additional spacing on each side; except it is not applied at the beginning or end of a line.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d10964dd"></a> Letter-spacing is not applied at the beginning or end of a line so that text always fits flush with the edge of the block.
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
> Applying letter spacing to the right or trailing edge of a line is incorrect:
>
> a　b　c　

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f16ff7e4"></a> As each letter with letter-spacing places half of the space assigned to it before and half after, this adds up to the full amount of spacing between adjacent letters. No spacing is added at the start or end of the line.
>
> ```text
> p { letter-spacing: 1em; }
> ```
>
> ```text
> <p>aa<span>bb</span>cc</p>
> ```
>
> aabbcc
>
> When the value varies between adjacent inlines, the effective spacing between adjacent letters of different inlines amounts to the average between the two.
>
> ```text
> p    { letter-spacing: 1em; }
> span { letter-spacing: 2em; }
> ```
>
> ```text
> <p>aa<span>bb</span>cc</p>
> ```
>
> aabbcc
>
> Letter spacing is inserted <strong>after</strong> RTL reordering. This affects which letter has spacing suppressed on one side due to being at the edge of the line.
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
> abcאבג

<a id="ref-for-propdef-letter-spacing⑦"></a>

<a id="ref-for-atomic-inline③"></a>

<a id="ref-for-typographic-character-unit③⑦"></a>

For the purpose of [letter-spacing](#propdef-letter-spacing), each consecutive run of [atomic inlines](https://www.w3.org/TR/css-display-4/#atomic-inline) (such as images and inline blocks) is treated as a single [typographic character unit](#typographic-character-unit).

Letter spacing ignores invisible zero-width formatting characters (such as those from the Unicode Cf category). Spacing must be added as if those characters did not exist in the document.

<a id="ref-for-propdef-letter-spacing⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-35549407"></a> For example, [letter-spacing](#propdef-letter-spacing) applied to `A&#x200B;B` is identical to `AB`, regardless of where any element boundaries might fall.

<a id="ref-for-propdef-letter-spacing⑨"></a>

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

#### <a id="cursive-tracking"></a>7.2.1.  Cursive Scripts

<a id="ref-for-cursive-script②"></a>

<a id="ref-for-typographic-letter-unit①④"></a>

If it is able, the UA <em>may</em> apply letter spacing to [cursive scripts](#cursive-script) by translating the total extra space to be distributed to a run of such letters into some form of cursive elongation (or compression, for negative tracking values) for that run that results in an equivalent total expansion (or compression) of the run. Otherwise, if the UA cannot expand text from a <a id="ref-for-cursive-script③"></a>cursive script without breaking its cursive connections, it <em>must not</em> apply spacing between any pair of that script’s [typographic letter units](#typographic-letter-unit) at all (effectively treating each word as a single <a id="ref-for-typographic-letter-unit①⑤"></a>typographic letter unit for the purpose of letter-spacing). Both cases will result in an effective spacing of zero between such letters; however the former will preserve the sense of stretching out the text.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9902d8b5"></a> Below are some appropriate and inappropriate examples of spacing out Arabic text.
>
> | Column 1                                                                                     | Column 2 | Column 3            |
> |----------------------------------------------------------------------------------------------|----------|---------------------|
> | ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-original.png)   | —        | Original text       |
> | ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-spaced.png)     | BAD      | <strong>&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;Even distribution of space between each letter.&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;<em>Notice this breaks cursive joins!</em>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> |
> | ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-kashida.png)    | OK       | <strong>&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;Distributing ∑<var>letter-spacing</var>&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;by typographically-appropriate cursive elongation.&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;<em>The resulting text is as long as the previous evenly-spaced example.</em>&#xA;&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> |
> | ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-suppressed.png) | OK       | <strong><span><a id="ref-for-spaces③②"></a></span><span><a id="ref-for-propdef-letter-spacing①①"></a></span><span><a id="ref-for-propdef-letter-spacing①⓪"></a></span>&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;Suppressing <a href="#propdef-letter-spacing">letter-spacing</a> between Arabic letters.&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;<em>Notice <a href="#propdef-letter-spacing">letter-spacing</a> is nonetheless applied&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;to non-Arabic characters (like <a href="#spaces">spaces</a>).</em>&#xA;&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> |
> | ![](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/arabic-stretch-unjoined.png)   | BAD      | <strong><span><a id="ref-for-propdef-letter-spacing①②"></a></span>&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;Applying <a href="#propdef-letter-spacing">letter-spacing</a> only between non-joined letters.&#xA;&#x9;&#x9;&#x9;&#x9;&#x9;<em>This distorts typographic color and obfuscates word boundaries.</em>&#xA;&#x9;&#x9;&#xA;    </strong> |

<a id="ref-for-propdef-letter-spacing①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Proper cursive elongation or compression of a text can vary depending on the script, typeface, language, location within a word, location within a line, implementation complexity, font capabilities, and calligraphic preferences, and may not be possible in certain cases at all. It may involve the use of shortening ligatures, swash variants, contextual forms, elongation glyphs such as U+0640 ـ ARABIC TATWEEL, or other microtypography. It is outside the scope of CSS to define rules for these effects. Authors should avoid applying [letter-spacing](#propdef-letter-spacing) to cursive scripts unless they are prepared to accept non-interoperable results.

### <a id="boundary-shaping"></a>7.3.  Shaping Across Element Boundaries

<a id="ref-for-typographic-character-unit③⑧"></a>

Text shaping <em>must</em> be broken at inline box boundaries when any of the following are true for any box whose boundary separates the two [typographic character units](#typographic-character-unit):

- <a id="ref-for-typographic-character-unit③⑨"></a>

  <a id="ref-for-propdef-padding"></a>

  <a id="ref-for-propdef-border"></a>

  <a id="ref-for-propdef-margin"></a>

  Any of [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)/[border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border)/[padding](https://www.w3.org/TR/css-box-4/#propdef-padding) separating the two [typographic character units](#typographic-character-unit) in the inline axis is non-zero.

- <a id="ref-for-initial-value"></a>

  <a id="ref-for-propdef-vertical-align"></a>

  [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) is not its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

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

## <a id="edge-effects"></a>8.  Edge Effects

<a id="ref-for-propdef-text-indent"></a>

<a id="ref-for-propdef-hanging-punctuation①"></a>

Edge effects control the indentation of lines with respect to other lines in the block ([text-indent](#propdef-text-indent)) and how content is measured at the start and end edges of a line ([hanging-punctuation](#propdef-hanging-punctuation)).

<a id="ref-for-propdef-text-indent①"></a>

### <a id="text-indent-property"></a>8.1. <a id="text-indent"></a> First Line Indentation: the [text-indent](#propdef-text-indent) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-indent"></a>text-indent                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-comb-all"></a><a id="ref-for-typedef-length-percentage"></a>\[ [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \] [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) hanging[?](https://www.w3.org/TR/css-values-4/#mult-opt) <a id="ref-for-comb-all①"></a>&#x26;&#x26; each-line<a id="ref-for-mult-opt①"></a>? |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-inner-size"></a><a id="ref-for-inline-axis①"></a>refers to block container’s own [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [inner size](https://www.w3.org/TR/css-sizing-3/#inner-size)                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value, plus any specified keywords                                                                                                                                                                                                                         |
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

<a id="ref-for-forced-line-break⑨"></a>

Indentation affects the first line of each block container and each line after a [forced line break](#forced-line-break) (but not lines after a [soft wrap break](#soft-wrap-break)).

<a id="valdef-text-indent-hanging"></a>hanging  
Inverts which lines are affected.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7dde3c71"></a>
>
> <a id="ref-for-propdef-text-align⑦"></a>
>
> <a id="ref-for-valdef-text-align-start④"></a>
>
> <a id="ref-for-propdef-text-indent②"></a>
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
<a id="ref-for-propdef-text-indent③"></a>

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
> <a id="ref-for-propdef-text-indent④"></a>
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
<a id="ref-for-propdef-text-indent⑤"></a>

<a id="ref-for-propdef-display"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the [text-indent](#propdef-text-indent) property inherits, when specified on a block element, it will affect descendant inline-block elements. For this reason, it is often wise to specify <a id="ref-for-propdef-text-indent⑥"></a>text-indent: 0 on elements that are specified [display: inline-block](https://www.w3.org/TR/css-display-4/#propdef-display).

### <a id="hanging"></a>8.2.  Hanging Glyphs

<a id="ref-for-hang①⓪"></a>

<a id="ref-for-intrinsic-size①"></a>

<a id="ref-for-min-content③"></a>

<a id="ref-for-max-content"></a>

When a glyph at the start or end edge of a line <a id="hang"></a>hangs, it is not considered when measuring the line’s contents for fit, alignment, or justification. Depending on the line’s alignment/justification, this can result in the mark being placed outside the line box. A [hanging glyph](#hang) is also not taken into account when computing [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) ([min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) and [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content)), and any sizes derived thereof. (The interaction of this measurement and kerning is currently UA-defined; the CSSWG [welcomes advice](https://github.com/w3c/csswg-drafts/issues/2397) on this point.)

<a id="ref-for-hang①①"></a>

<a id="ref-for-inline-box④"></a>

A [hanging glyph](#hang) is still enclosed inside its parent inline box and still participates in text justification: its character advance is just not measured when determining how much content fits on the line, how much the line’s contents need to be expanded or compressed for justification, or how to position the content within the line box for text alignment. Effectively, the <a id="ref-for-hang①②"></a>hanging glyph character advance is re-interpreted as an additional negative margin on the affected edge of its parent [inline box](https://www.w3.org/TR/css-display-4/#inline-box); the line is otherwise laid out as usual.

<a id="ref-for-hang①③"></a>

<a id="ref-for-ink-overflow"></a>

<a id="ref-for-scrollable-overflow"></a>

An overflowing [hanging glyph](#hang) should typically be considered [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) so as to avoid creating unnecessary scrollbars, but the UA may treat it as [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) when the content is editable or in other circumstances where treating it as <a id="ref-for-scrollable-overflow①"></a>scrollable overflow would be useful to the user. [\[CSS-OVERFLOW-3\]](#biblio-css-overflow-3)

<a id="ref-for-hang①④"></a>

<a id="ref-for-conditionally-hang⑥"></a>

<a id="ref-for-min-content④"></a>

<a id="ref-for-max-content①"></a>

In some cases, a glyph at the end of a line can <a id="conditionally-hang"></a>conditionally hang: it [hangs](#hang) only if it does not otherwise fit in the line prior to justification. It is not considered when measuring the line’s contents for fit; however, any part of it that does not fit is considered to <a id="ref-for-hang①⑤"></a>hang. Glyphs that [conditionally hang](#conditionally-hang) are not taken into account when computing [min-content sizes](https://www.w3.org/TR/css-sizing-3/#min-content) and any sizes derived thereof, but they are taken into account for [max-content sizes](https://www.w3.org/TR/css-sizing-3/#max-content) and any sizes derived thereof.

When conditionally hangable glyphs at the end of the line are immediately preceded by (unconditionally) hangable glyphs, the unconditionally hangable glyphs hang if and only if all subsequent conditionally hangable glyphs do indeed hang; otherwise, the (unconditionally) hangable glyphs are effectively not at the end of the line and do not hang.

<a id="ref-for-hang①⑥"></a>

Non-zero inline-axis borders or padding between a [hang](#hang)able glyph and the edge of the line prevent the glyph from hanging. For example, a period at the end of an inline box with end padding does not <a id="ref-for-hang①⑦"></a>hang at the end edge of a line.

<a id="ref-for-hang①⑧"></a>

Multiple adjacent glyphs can hang together, however specific limits on how many are allowed to hang may be specified (e.g. at most one punctuation character may [hang](#hang) at each edge of the line).

<a id="ref-for-propdef-hanging-punctuation②"></a>

#### <a id="hanging-punctuation-property"></a>8.2.1. <a id="hanging-punctuation"></a> Hanging Punctuation: the [hanging-punctuation](#propdef-hanging-punctuation) property

| Field               | Definition                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-hanging-punctuation"></a>hanging-punctuation                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any②"></a><a id="ref-for-comb-one④⑤"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ first [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ force-end <a id="ref-for-comb-one④⑥"></a>\| allow-end \] <a id="ref-for-comb-any③"></a>\|\| last \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                    |

<a id="ref-for-hang①⑨"></a>

This property determines whether a punctuation mark, if one is present, [hangs](#hang) and may be placed outside the line box (or in the indent) at the start or at the end of a line of text.

<a id="ref-for-propdef-hanging-punctuation③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If there is not sufficient padding on the block container, [hanging-punctuation](#propdef-hanging-punctuation) can trigger overflow.

Values have the following meanings:

<a id="valdef-hanging-punctuation-none"></a>none  
<a id="ref-for-hang②⓪"></a>

No punctuation character is made to [hang](#hang).

<a id="valdef-hanging-punctuation-first"></a>first  
<a id="ref-for-hang②①"></a>

<a id="ref-for-first-formatted-line①"></a>

An opening bracket, quote, or ideographic space at the start of the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of an element [hangs](#hang). This applies to all characters in the Unicode categories Ps, Pf, Pi plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK and the IDEOGRAPHIC SPACE U+3000.

<a id="valdef-hanging-punctuation-last"></a>last  
<a id="ref-for-hang②②"></a>

A closing bracket or quote at the end of the last formatted line of an element [hangs](#hang). This applies to all characters in the Unicode categories Pe, Pf, Pi plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK.

<a id="valdef-hanging-punctuation-force-end"></a>force-end  
<a id="ref-for-hang②③"></a>

<a id="ref-for-stop-or-comma"></a>

A [stop or comma](#stop-or-comma) at the end of a line [hangs](#hang).

<a id="valdef-hanging-punctuation-allow-end"></a>allow-end  
<a id="ref-for-conditionally-hang⑦"></a>

<a id="ref-for-stop-or-comma①"></a>

A [stop or comma](#stop-or-comma) at the end of a line [conditionally hangs](#conditionally-hang).

<a id="ref-for-hang②④"></a>

At most one punctuation character may [hang](#hang) at each edge of the line.

<a id="ref-for-hang②⑤"></a>

<a id="stop-or-comma"></a>Stops and commas allowed to [hang](#hang) include:

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
> Note: The CSS Working Group would appreciate if UAs including other characters would [inform the working group](#sotd) of such additions.

<a id="ref-for-valdef-hanging-punctuation-allow-end"></a>

<a id="ref-for-valdef-hanging-punctuation-force-end"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-68d23fbc"></a> The [allow-end](#valdef-hanging-punctuation-allow-end) and [force-end](#valdef-hanging-punctuation-force-end) are two variations of hanging punctuation used in East Asia.
>
> ![hanging-punctuation: allow-end](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/hanging-punctuation-allow-end.png)
>
> ```text
> p {
>   text-align: justify;
>   hanging-punctuation: allow-end;
> }
> ```
>
> ![hanging-punctuation: force-end](https://www.w3.org/TR/2026/CRD-css-text-3-20260814/images/hanging-punctuation-force-end.png)
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

### <a id="bidi-linebox"></a>8.3.  Bidirectionality and Line Boxes

<a id="ref-for-start④"></a>

<a id="ref-for-end③"></a>

<a id="ref-for-inline-base-direction"></a>

<a id="ref-for-line-box①"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-bidi-paragraph"></a>

<a id="ref-for-propdef-text-align-all⑦"></a>

<a id="ref-for-propdef-text-align-last⑨"></a>

<a id="ref-for-propdef-text-indent⑦"></a>

<a id="ref-for-propdef-hanging-punctuation④"></a>

The [start](https://www.w3.org/TR/css-writing-modes-4/#start) and [end](https://www.w3.org/TR/css-writing-modes-4/#end) sides of a line box are determined by the [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) of the line box. Although they usually match, the <a id="ref-for-inline-base-direction①"></a>inline base direction of a [line box](https://www.w3.org/TR/css-inline-3/#line-box) is distinct from the <a id="ref-for-inline-base-direction②"></a>inline base direction of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) or the [bidi paragraph](https://www.w3.org/TR/css-writing-modes-4/#bidi-paragraph). The <a id="ref-for-line-box②"></a>line box’s <a id="ref-for-inline-base-direction③"></a>inline base direction affects [text-align-all](#propdef-text-align-all), [text-align-last](#propdef-text-align-last), [text-indent](#propdef-text-indent), and [hanging-punctuation](#propdef-hanging-punctuation)—​i.e. the position and alignment of its contents with respect to its edges. It does not affect the formatting or ordering of inline content (which is controlled by the [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/) as applied by [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/) [\[UAX9\]](#biblio-uax9) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)).

<a id="ref-for-line-box③"></a>

<a id="ref-for-inline-base-direction④"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-unicode-bidi"></a>

In most cases, a [line box](https://www.w3.org/TR/css-inline-3/#line-box)’s [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is given by its [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s computed [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction). However, if its <a id="ref-for-containing-block②"></a>containing block has [unicode-bidi: plaintext](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4):

- <a id="ref-for-inline-base-direction⑤"></a>

  <a id="ref-for-line-box④"></a>

  <a id="ref-for-bidi-paragraph①"></a>

  If the [bidi paragraph](https://www.w3.org/TR/css-writing-modes-4/#bidi-paragraph) to which the [line box](https://www.w3.org/TR/css-inline-3/#line-box) belongs (that is, the <a id="ref-for-bidi-paragraph②"></a>bidi paragraph for which the line box holds content) has strong directionality, the line box’s [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is that direction.

- <a id="ref-for-propdef-direction②"></a>

  <a id="ref-for-inline-base-direction⑥"></a>

  <a id="ref-for-atomic-inline④"></a>

  <a id="ref-for-line-box⑤"></a>

  If the [line box](https://www.w3.org/TR/css-inline-3/#line-box) is empty (i.e. contains no [atomic inlines](https://www.w3.org/TR/css-display-4/#atomic-inline) or characters other than the newline character, if any) or otherwise has no strong directionality (contains only weak or neutral characters), its [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) is taken from the preceding line box (if any), or, if this is the first line box in the containing block, from the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the containing block. (This can result in an RTL line box whose contents have an LTR base direction.)

<a id="ref-for-propdef-display①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0a5e92f2"></a> In the following example, assuming the `<block>` is a start-aligned preformatted block ([display: block; white-space: pre; text-align: start](https://www.w3.org/TR/css-display-4/#propdef-display)), every other line is right-aligned:
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
<a id="ref-for-propdef-text-align⑧"></a>

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

1.  [white space processing](#white-space-phase-1) part I (pre-wrapping)

2.  [text transformation](#transforming)

3.  [text combination](https://www.w3.org/TR/css-writing-modes-4/#text-combine-upright) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

4.  [text orientation](https://www.w3.org/TR/css-writing-modes-4/#text-orientation) [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

5.  <a id="ref-for-wrapping⑨"></a>

    [text wrapping](#wrapping) while applying per line:

    - [indentation](#text-indent-property)

    - [bidirectional reordering](https://www.w3.org/TR/css-writing-modes-4/#text-direction) [\[CSS2\]](#biblio-css2) / [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)

    - [white space processing](#white-space-phase-2) part II

    - [font/glyph selection and positioning](https://www.w3.org/TR/css-fonts-3/) [\[CSS-FONTS-3\]](#biblio-css-fonts-3)

    - <a id="ref-for-propdef-letter-spacing①④"></a>

      <a id="ref-for-propdef-word-spacing⑦"></a>

      [letter-spacing](#propdef-letter-spacing) and [word-spacing](#propdef-word-spacing)

    - [hanging punctuation](#hanging-punctuation-property)

6.  [justification](#justification) (which may affect glyph selection and/or text wrapping, looping back into that step)

7.  [text alignment](#text-align-property)

## <a id="plaintext"></a> Appendix B: Conversion to Plaintext

<em>This appendix is normative</em> for the purpose of plaintext copy-paste operations.

When a CSS-rendered document is converted to a plaintext format, it is expected that:

- <a id="ref-for-propdef-text-transform⑧"></a>

  The [text-transform](#propdef-text-transform) property has no effect.

- <a id="ref-for-forced-line-break①⓪"></a>

  <a id="ref-for-block"></a>

  <a id="ref-for-white-space②②"></a>

  <a id="ref-for-collapsible-white-space①⓪"></a>

  [§ 4.1.1 Phase I: Collapsing and Transformation](#white-space-phase-1) is applied and any sequence of [collapsible](#collapsible-white-space) [white space](#white-space) at the beginning of a [block](https://www.w3.org/TR/css-display-4/#block) or immediately following a [forced line break](#forced-line-break) is removed.

## <a id="default-stylesheet"></a> Appendix C: Default UA Stylesheet

<em>This appendix is informative,</em> and is to help UA developers to implement a default stylesheet for HTML, but UA developers are free to ignore or modify as appropriate.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6277cda4"></a>
>
> ```text
> /* make option elements align together */
> option { text-align: match-parent; }
> 
> /* Avoid hanging punctuation inheriting into preformatted blocks */
> pre { hanging-punctuation: none; }
> ```
## <a id="script-groups"></a> Appendix D: Scripts and Spacing

<em>This appendix is normative.</em>

<a id="ref-for-unicode-script①"></a>

<a id="ref-for-justification-opportunity②⓪"></a>

Typographic behavior varies somewhat by language, but varies drastically by writing system. This appendix categorizes some common [scripts](#unicode-script) in Unicode 6.0 according to their justification and spacing behavior. Category descriptions are descriptive, not prescriptive; the determining factor is the prioritization of [justification opportunities](#justification-opportunity).

<a id="block-scripts"></a>block scripts  
<a id="ref-for-writing-system-japanese③"></a>

<a id="ref-for-writing-system-korean"></a>

<a id="ref-for-writing-system-chinese③"></a>

<a id="ref-for-content-writing-system④"></a>

<a id="ref-for-unicode-east-asian-width②"></a>

<a id="ref-for-unicode-script②"></a>

CJK and by extension all Wide characters (see [East Asian Width](https://www.unicode.org/reports/tr11/) [\[UAX11\]](#biblio-uax11)). The following [Unicode scripts](#unicode-script) are included: Bopomofo, Han, Hangul, Hiragana, Katakana, and Yi. Characters of the [East Asian Width property](#unicode-east-asian-width) `Wide` and `Fullwidth` are also included, but `Ambiguous` characters are included only if the [writing system](#content-writing-system) is [Chinese](#writing-system-chinese), [Korean](#writing-system-korean), or [Japanese](#writing-system-japanese).

<a id="clustered-scripts"></a>clustered scripts  
<a id="ref-for-unicode-script③"></a>

Clustered scripts have discrete units and break only at word boundaries, but do not use visible word separators. They prioritize stretching spaces, but comfortably admit inter-character spacing for justification. The clustered scripts include, but are not limited to, the following [Unicode scripts](#unicode-script): Khmer, Lao, Myanmar, New Tai Lue, Tai Le, Tai Tham, Tai Viet, Thai

<a id="cursive-script"></a>cursive scripts  
<a id="ref-for-unicode-script④"></a>

<a id="ref-for-propdef-letter-spacing①⑤"></a>

Cursive scripts do not admit gaps between their letters for either justification or [letter-spacing](#propdef-letter-spacing). The following [Unicode scripts](#unicode-script) are included: Arabic, Hanifi Rohingya, Mandaic, Mongolian, N’Ko, Phags Pa, Syriac

<a id="ref-for-cursive-script④"></a>

<a id="ref-for-typographic-character-unit④⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Indic scripts with baseline connectors (such as Devanagari and Gujarati) <em>are not</em> considered [cursive scripts](#cursive-script), and <em>do</em> admit such gaps between [typographic character units](#typographic-character-unit). See [Indic Layout Requirements](https://www.w3.org/TR/ilreq/). [\[ILREQ\]](#biblio-ilreq)

User agents should update this list as they update their Unicode support to handle as-yet-unencoded cursive scripts in future versions of Unicode, and are encouraged to ask the CSSWG to update this spec accordingly.

## <a id="character-properties"></a> Appendix E: Characters and Properties

<em>This appendix is normative.</em>

Unicode defines four code point-level properties that are referenced in CSS typesetting:

<a id="unicode-east-asian-width"></a>East Asian width property  
Defined in [Unicode Standard Annex \#11](http://www.unicode.org/reports/tr11/#Definitions) [\[UAX11\]](#biblio-uax11) and given as the `East_Asian_Width` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44).

<a id="unicode-general-category"></a>general category  
Defined in [Unicode Standard Annex \#44](http://www.unicode.org/reports/tr44/#General_Category_Values) [\[UAX44\]](#biblio-uax44) and given as the `General_Category` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44).

<a id="unicode-script"></a>script property  
Defined in [Unicode Standard Annex \#24](http://www.unicode.org/reports/tr24/#Values) [\[UAX24\]](#biblio-uax24) and given as the `Script` property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44). (UAs must include any ScriptExtensions.txt assignments in this mapping.)

<a id="unicode-vertical-orientation"></a>Vertical Orientation  
Defined in [Unicode Standard Annex \#50](http://www.unicode.org/reports/tr50/) [\[UAX50\]](#biblio-uax50) as the Vertical_Orientation property in the [Unicode Character Database](https://www.unicode.org/reports/tr44/) [\[UAX44\]](#biblio-uax44).

<a id="ref-for-typographic-character-unit④①"></a>

<a id="ref-for-grapheme-cluster④"></a>

Unicode defines properties for individual code points, but sometimes it is necessary to determine the properties of a [typographic character unit](#typographic-character-unit). For the purposes of CSS Text, the properties of a <a id="ref-for-typographic-character-unit④②"></a>typographic character unit are given by the base character of its first [grapheme cluster](#grapheme-cluster)—​except in two cases:

- <a id="ref-for-grapheme-cluster⑤"></a>

  [Grapheme clusters](#grapheme-cluster) formed with an Enclosing Mark (`Me`) of the Common script are considered to be Other Symbols (`So`) in the Common script. They are assumed to have the same Unicode properties as the replacement character (U+FFFD).

- <a id="ref-for-grapheme-cluster⑥"></a>

  [Grapheme clusters](#grapheme-cluster) formed with a Space Separator (`Zs`) as the base are considered to be Modifier Symbols (`Sk`). They are assumed to have the same East Asian Width property as the base, but take their other properties from the first combining character in the sequence.

## <a id="script-tagging"></a> Appendix F: Identifying the Content Writing System

<em>This appendix is normative.</em>

While most languages have a preferred writing system, some have multiple, and most can also be transcribed into one or more foreign writing systems. As a common example, most languages have at least one Latin transcription, and can thus be written in the Latin writing system. Transcribed texts typically adopt the typographic conventions of the writing system: for example Japanese “romaji” and Chinese Pinyin use Latin letters and word spaces, and follow Latin line-breaking and justification practices accordingly. As another example, historical ideographic Korean (`ko-Hani`) does not use word spaces, and should therefore be typeset similar to Chinese rather than modern Korean.

<a id="ref-for-doclanguage④"></a>

<a id="ref-for-content-language①⑦"></a>

In HTML or any other [document language](https://www.w3.org/TR/CSS2/conform.html#doclanguage) using BCP47 tags for identifying languages to declare the [content language](#content-language), authors can disambiguate or indicate the use of an atypical writing system with script subtags. [\[BCP47\]](#biblio-bcp47) For example, to indicate use of the Latin writing system for languages which don’t natively use it, the `-Latn` script subtag can be added, e.g. `ja-Latn` for Japanese romaji. Other subtags exist for other writing systems, see ISO’s Code for the Representation of Names of Scripts and the [ISO15924 script tag registry](http://unicode.org/iso15924/iso15924-codes.html). [\[ISO15924\]](#biblio-iso15924)

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

<a id="ref-for-content-language①⑧"></a>

When no writing system is explicitly indicated, UAs should assume the most common writing system of the declared [content language](#content-language) for language-sensitive typographic behaviors such as line-breaking or justification. However, UAs must not assume that writing system if the author has explicitly declared a different one. If the UA has no language-specific knowledge of a particular language and writing system combination, it must use the typographic conventions of the declared writing system (assuming the conventions of a different language if necessary), not the conventions of the declared language in an assumed writing system, which would be inappropriate to the declared writing system.

The full correspondence between languages and their most common writing systems is out of scope for this document. However, user agents must assume at least the following:

- <a id="ref-for-content-writing-system⑤"></a>

  <a id="ref-for-content-language①⑨"></a>

  If the [content language](#content-language) is Chinese and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language②⓪"></a>content language if the <a id="ref-for-content-writing-system⑥"></a>writing system to specified to be one of the Hant, Hans, Hani, Hanb, or Bopo ISO script codes, then the <a id="ref-for-content-writing-system⑦"></a>writing system is <a id="writing-system-chinese"></a>Chinese.

- <a id="ref-for-content-writing-system⑧"></a>

  <a id="ref-for-content-language②①"></a>

  If the [content language](#content-language) is Japanese and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language②②"></a>content language if the <a id="ref-for-content-writing-system⑨"></a>writing system to specified to be one of the Jpan, Hrkt, Hira, or Kana ISO script codes, then the <a id="ref-for-content-writing-system①⓪"></a>writing system is <a id="writing-system-japanese"></a>Japanese.

- <a id="ref-for-content-writing-system①①"></a>

  <a id="ref-for-content-language②③"></a>

  If the [content language](#content-language) is Korean and the [writing system](#content-writing-system) is unspecified, or for any <a id="ref-for-content-language②④"></a>content language if the <a id="ref-for-content-writing-system①②"></a>writing system to specified to be one of the Kore, Hang, or Jamo ISO script codes, then the <a id="ref-for-content-writing-system①③"></a>writing system is <a id="writing-system-korean"></a>Korean.

- <a id="ref-for-content-language②⑤"></a>

  <a id="ref-for-content-writing-system①④"></a>

  The [writing system](#content-writing-system) is only considered to be <a id="writing-system-known"></a>unknown if the [content language](#content-language) itself is unknown, or if it explicitly indicates an unknown writing system.

  <a id="ref-for-content-writing-system①⑤"></a>

  <a id="ref-for-content-language②⑥"></a>

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

## <a id="priv"></a> Privacy Considerations

This specification leaks the user’s installed hyphenation and line-breaking dictionaries.

## <a id="sec"></a> Security Considerations

This specification introduces no new security considerations.

## <a id="acknowledgements"></a> Acknowledgements

This specification would not have been possible without the help from: Addison Phillips, Aharon Lanin, Alan Stearns, Ambrose Li, Arnold Schrijver, Arye Gittelman, Ayman Aldahleh, Ben Errez, Bert Bos, Chris Lilley, Chris Pratley, Chris Thrasher, Chris Wilson, Dave Hyatt, David Baron, Emilio Cobos Álvarez, Eric LeVine, Etan Wexler, Frank Tang, Håkon Wium Lie, IM Mincheol, Ian Hickson, James Clark, Javier Fernandez, John Daggett, Jonathan Kew, Ken Lunde, Laurie Anna Edlund, Marcin Sawicki, Martin Dürst, Martin Heijdra, Masafumi Yabe, Masayasu Ishikawa, Michael Jochimsen, Michel Suignard, Mike Bemford, Myles Maxfield, Nat McCully, Paul Nelson, Rahul Sonnad, Richard Ishida, Shinyu Murakami, Stephen Deach, Steve Zilles, Takao Suzuki, Tantek Çelik, Xidorn Quan, Yaniv Feinberg.

## <a id="changes"></a> Changes

### <a id="recent-changes"></a> Recent Changes

The following changes have been made since the [June 2026 Candidate Recommendation Draft](https://www.w3.org/TR/2026/CRD-css-text-3-20260608/):

- <a id="ref-for-propdef-letter-spacing①⑥"></a>

  Changed the [letter-spacing](#propdef-letter-spacing) model from inserting space [between characters](https://www.w3.org/TR/2026/CRD-css-text-3-20260608/#letter-spacing) to [around characters](#letter-spacing). ([Issue 10193](https://github.com/w3c/csswg-drafts/issues/10193))

- <a id="ref-for-propdef-vertical-align①"></a>

  <a id="ref-for-initial-value①"></a>

  <a id="ref-for-valdef-alignment-baseline-baseline"></a>

  Clarified that shaping breaks when [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) is not its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value), rather than mentioning [baseline](https://www.w3.org/TR/css-inline-3/#valdef-alignment-baseline-baseline) specifically, to avoid confusion given further developments to <a id="ref-for-propdef-vertical-align②"></a>vertical-align. ([Issue 13277](https://github.com/w3c/csswg-drafts/issues/13277))

- Clarified the interaction between conditionally and unconditionally hangable glyphs at the end of a line. ([Issue 9724](https://github.com/w3c/csswg-drafts/issues/9724))

The following changes have been made since the [September 2024 Candidate Recommendation Draft](https://www.w3.org/TR/2024/CRD-css-text-3-20240930/):

- <a id="ref-for-propdef-line-break①①"></a>

  Disallowed breaks before small kana in [line-break: normal](#propdef-line-break). ([Issue 10363](https://github.com/w3c/csswg-drafts/issues/10363))

- Backported `hanging-punctuation: none` rule to [Appendix C: Default UA Stylesheet](#default-stylesheet) from Level 4.

- Synchronized text with Level 4 editorial changes.

The following normative changes have been made since the [September 2023 Candidate Recommendation Draft](https://www.w3.org/TR/2023/CRD-css-text-3-20230903/):

- <a id="ref-for-propdef-text-align-last①⓪"></a>

  Corrected the Computed Value line for [text-align-last](#propdef-text-align-last). ([Issue 7331](https://github.com/w3c/csswg-drafts/issues/7331))

- <a id="ref-for-propdef-white-space②⓪"></a>

  <a id="ref-for-atomic-inline⑤"></a>

  <a id="ref-for-soft-wrap-opportunity③⑦"></a>

  Disambiguate soft wrap opportunities around replaced elements. ([Issue 9964](https://github.com/w3c/csswg-drafts/issues/9964))

  > For [soft wrap opportunities](#soft-wrap-opportunity) defined by the boundary between two characters <u>or [atomic inlines](https://www.w3.org/TR/css-display-4/#atomic-inline)</u> , the [white-space](#propdef-white-space) property on the nearest common ancestor of the two characters controls breaking;

The following normative changes have been made since the [February 2023 Candidate Recommendation Draft](https://www.w3.org/TR/2023/CRD-css-text-3-20230213/).

- Update [Appendix G: Small Kana Mappings](#small-kana) to Unicode 15.0. ([Issue 8442](https://github.com/w3c/csswg-drafts/issues/8442))

- <a id="ref-for-atomic-inline⑦"></a>

  <a id="ref-for-soft-wrap-opportunity③⑨"></a>

  <a id="ref-for-atomic-inline⑥"></a>

  <a id="ref-for-soft-wrap-opportunity③⑧"></a>

  Non-tailorable Unicode line breaking controls other than NBSP take precedence over our rule about atomic inlines. ([Issue 8972](https://github.com/w3c/csswg-drafts/issues/8972))

  > For Web-compatibility there is a [soft wrap opportunity](#soft-wrap-opportunity) before and after each replaced element or other [atomic inline](https://www.w3.org/TR/css-display-4/#atomic-inline), even when adjacent to a character that would normally suppress them, ~~such as~~ <u>including</u> U+00A0 NO-BREAK SPACE. <u>However, with the exception of U+00A0 NO-BREAK SPACE, there must be no [soft wrap opportunity](#soft-wrap-opportunity) between [atomic inlines](https://www.w3.org/TR/css-display-4/#atomic-inline) and adjacent characters belonging to the Unicode GL, WJ, or ZWJ line breaking classes. [\[UAX14\]](#biblio-uax14)</u>

The following normative changes have been made since the [December 2020 Candidate Recommendation](https://www.w3.org/TR/2020/CR-css-text-3-20201222/).

- <a id="ref-for-propdef-hanging-punctuation⑤"></a>

  Allow [hanging-punctuation: first](#propdef-hanging-punctuation) to hang U+300 IDEOGRAPHIC SPACE in order to accommodate plaintext indentation habits. ([Issue 2462](https://github.com/w3c/csswg-drafts/issues/2462))

  > first  
  > <a id="ref-for-hang②⑥"></a>
  >
  > <a id="ref-for-first-formatted-line②"></a>
  >
  > An opening bracket <u>,</u> ~~or~~ quote <u>, or ideographic space</u> at the start of the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of an element [hangs](#hang). This applies to all characters in the Unicode categories Ps, Pf, Pi plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK plus the ASCII quote marks U+0027 ' APOSTROPHE and U+0022 " QUOTATION MARK <u>and the IDEOGRAPHIC SPACE U+3000</u> .

- <a id="ref-for-valdef-text-justify-distribute①"></a>

  <a id="ref-for-valdef-text-justify-inter-character②"></a>

  <a id="ref-for-css-legacy-value-alias①"></a>

  Define that [distribute](#valdef-text-justify-distribute) computes to [inter-character](#valdef-text-justify-inter-character), rather than merely behave the same; allow <a id="ref-for-valdef-text-justify-distribute②"></a>distribute to be implemented as a [legacy value alias](https://www.w3.org/TR/css-cascade-5/#css-legacy-value-alias), since this is easier for some engines and does not matter for compatibility. ([Issue 6156](https://github.com/w3c/csswg-drafts/issues/6156), [Issue 7322](https://github.com/w3c/csswg-drafts/issues/7322))

- <a id="ref-for-hyphenation-opportunity⑧"></a>

  <a id="ref-for-hyphenation-opportunity⑦"></a>

  Clarify that language-specific hyphenation rules also apply to explicit [hyphenation opportunities](#hyphenation-opportunity). ([Issue 5973](https://github.com/w3c/csswg-drafts/issues/5973))

  > Words are only hyphenated where there are characters inside the word that explicitly suggest [hyphenation opportunities](#hyphenation-opportunity). <u>The UA must use the appropriate language-specific hyphenation character(s) and should apply any appropriate spelling changes just as for automatic hyphenation at the same point.</u>

- <a id="ref-for-valdef-text-align-match-parent④"></a>

  <a id="ref-for-root-element①"></a>

  <a id="ref-for-valdef-text-align-start⑤"></a>

  <a id="ref-for-principal-writing-mode"></a>

  Define [match-parent](#valdef-text-align-match-parent) on the [root element](https://www.w3.org/TR/css-display-4/#root-element) to compute to [start](#valdef-text-align-start) instead of computing against the [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode). ([Issue 6542](https://github.com/w3c/csswg-drafts/issues/6542))

- <a id="ref-for-propdef-text-transform⑨"></a>

  Make authoring advice regarding [text-transform](#propdef-text-transform) a normative recommendation. ([Issue 8279](https://github.com/w3c/csswg-drafts/issues/8279))

  > <a id="ref-for-propdef-text-transform①⓪"></a>
  >
  > ~~Note: The [text-transform](#propdef-text-transform) property only affects the presentation layer; correct casing for semantic purposes is expected to be represented in the source document.~~
  >
  > <a id="ref-for-propdef-text-transform①①"></a>
  >
  > <u>Advisement: Authors must not rely on [text-transform](#propdef-text-transform) for semantic purposes; rather the correct casing and semantics should be encoded in the source document text and markup.</u>

In addition there have been some minor editorial fixes.

### <a id="old-changes"></a> Older Changes

See also [earlier list of changes](https://www.w3.org/TR/2020/CR-css-text-3-20201222/#changes) covering the 2020 and 2019 Working Drafts prior to that Candidate Recommendation and the [Disposition of Comments](https://drafts.csswg.org/css-text-3/issues-lc-2013) covering all comments between 2013 and 2020.

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

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong>
        UAs MUST provide an accessible alternative.
    </strong>

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

- [allow-end](#valdef-hanging-punctuation-allow-end), in § 8.2.1
- anywhere
  - [value for line-break](#valdef-line-break-anywhere), in § 5.2
  - [value for overflow-wrap](#valdef-overflow-wrap-anywhere), in § 5.4
- auto
  - [value for hyphens](#valdef-hyphens-auto), in § 5.3
  - [value for line-break](#valdef-line-break-auto), in § 5.2
  - [value for text-align-last](#valdef-text-align-last-auto), in § 6.3
  - [value for text-justify](#valdef-text-justify-auto), in § 6.4
- [bidi formatting characters](#bidi-formatting-characters), in § 4.1.1
- [block scripts](#block-scripts), in § Unnumbered section
- [break-all](#valdef-word-break-break-all), in § 5.1
- [break-spaces](#valdef-white-space-break-spaces), in § 3
- break-word
  - [value for overflow-wrap](#valdef-overflow-wrap-break-word), in § 5.4
  - [value for word-break](#valdef-word-break-break-word), in § 5.1
- [capitalize](#valdef-text-transform-capitalize), in § 2.1
- [center](#valdef-text-align-center), in § 6.1
- [character](#character), in § 1.4
- [Chinese](#writing-system-chinese), in § Unnumbered section
- [clustered scripts](#clustered-scripts), in § Unnumbered section
- [collapsible](#collapsible-white-space), in § 4.1.1
- [collapsible white space](#collapsible-white-space), in § 4.1.1
- [conditionally hang](#conditionally-hang), in § 8.2
- [content language](#content-language), in § 1.3
- [content writing system](#content-writing-system), in § 1.3
- [cursive script](#cursive-script), in § Unnumbered section
- [distribute](#valdef-text-justify-distribute), in § 6.4
- [document white space](#white-space), in § 4.1
- [document white space characters](#white-space), in § 4.1
- [each-line](#valdef-text-indent-each-line), in § 8.1
- [East Asian Width property](#unicode-east-asian-width), in § Unnumbered section
- [end](#valdef-text-align-end), in § 6.1
- [first](#valdef-hanging-punctuation-first), in § 8.2.1
- [forced line break](#forced-line-break), in § 5
- [force-end](#valdef-hanging-punctuation-force-end), in § 8.2.1
- [full-size](#kana-full-size), in § Unnumbered section
- [full-size kana](#kana-full-size), in § Unnumbered section
- [full-size-kana](#valdef-text-transform-full-size-kana), in § 2.1
- full-width
  - [definition of](#full-width), in § 2.1.1
  - [value for text-transform](#valdef-text-transform-full-width), in § 2.1
- [General Category](#unicode-general-category), in § Unnumbered section
- [grapheme cluster](#grapheme-cluster), in § 1.4
- [half-width](#half-width), in § 2.1.1
- [hang](#hang), in § 8.2
- [hanging](#valdef-text-indent-hanging), in § 8.1
- [hanging glyph](#hang), in § 8.2
- [hanging-punctuation](#propdef-hanging-punctuation), in § 8.2.1
- [hyphenate](#hyphenate), in § 5.3
- [hyphenation](#hyphenate), in § 5.3
- [hyphenation opportunity](#hyphenation-opportunity), in § 5.3
- [hyphens](#propdef-hyphens), in § 5.3
- [inter-character](#valdef-text-justify-inter-character), in § 6.4
- [inter-word](#valdef-text-justify-inter-word), in § 6.4
- [Japanese](#writing-system-japanese), in § Unnumbered section
- [justification opportunity](#justification-opportunity), in § 6.4.1
- [justify](#valdef-text-align-justify), in § 6.1
- [justify-all](#valdef-text-align-justify-all), in § 6.1
- [keep-all](#valdef-word-break-keep-all), in § 5.1
- [known](#writing-system-known), in § Unnumbered section
- [Korean](#writing-system-korean), in § Unnumbered section
- [last](#valdef-hanging-punctuation-last), in § 8.2.1
- [left](#valdef-text-align-left), in § 6.1
- \<length\>
  - [value for letter-spacing](#valdef-letter-spacing-length), in § 7.2
  - [value for text-indent](#valdef-text-indent-length), in § 8.1
  - [value for word-spacing](#valdef-word-spacing-length), in § 7.1
- [letter](#letter), in § 1.4
- [letter-spacing](#propdef-letter-spacing), in § 7.2
- [line break](#line-break), in § 5
- [line-break](#propdef-line-break), in § 5.2
- [line breaking](#line-breaking-process), in § 5
- [line breaking process](#line-breaking-process), in § 5
- [loose](#valdef-line-break-loose), in § 5.2
- [lowercase](#valdef-text-transform-lowercase), in § 2.1
- [manual](#valdef-hyphens-manual), in § 5.3
- [match-parent](#valdef-text-align-match-parent), in § 6.1
- none
  - [value for hanging-punctuation](#valdef-hanging-punctuation-none), in § 8.2.1
  - [value for hyphens](#valdef-hyphens-none), in § 5.3
  - [value for text-justify](#valdef-text-justify-none), in § 6.4
  - [value for text-transform](#valdef-text-transform-none), in § 2.1
- normal
  - [value for letter-spacing](#valdef-letter-spacing-normal), in § 7.2
  - [value for line-break](#valdef-line-break-normal), in § 5.2
  - [value for overflow-wrap](#valdef-overflow-wrap-normal), in § 5.4
  - [value for white-space](#valdef-white-space-normal), in § 3
  - [value for word-break](#valdef-word-break-normal), in § 5.1
  - [value for word-spacing](#valdef-word-spacing-normal), in § 7.1
- [nowrap](#valdef-white-space-nowrap), in § 3
- [other space separators](#other-space-separators), in § 4.1
- [overflow-wrap](#propdef-overflow-wrap), in § 5.4
- [\<percentage\>](#valdef-text-indent-percentage), in § 8.1
- [pre](#valdef-white-space-pre), in § 3
- [pre-line](#valdef-white-space-pre-line), in § 3
- [preserved](#preserved-white-space), in § 3
- [preserved white space](#preserved-white-space), in § 3
- [pre-wrap](#valdef-white-space-pre-wrap), in § 3
- [right](#valdef-text-align-right), in § 6.1
- [Script property](#unicode-script), in § Unnumbered section
- [segment break](#segment-break), in § 4
- [small](#kana-small), in § Unnumbered section
- [small kana](#kana-small), in § Unnumbered section
- [soft wrap break](#soft-wrap-break), in § 5
- [soft wrap opportunity](#soft-wrap-opportunity), in § 5
- [spaces](#spaces), in § 4.1
- [start](#valdef-text-align-start), in § 6.1
- [stop or comma](#stop-or-comma), in § 8.2.1
- [strict](#valdef-line-break-strict), in § 5.2
- [tabs](#tabs), in § 4.1
- [tab size](#tab-size-dfn), in § 4.2
- [tab-size](#propdef-tab-size), in § 4.2
- [tab stop](#tab-stop), in § 4.1.2
- [text-align](#propdef-text-align), in § 6.1
- [text-align-all](#propdef-text-align-all), in § 6.2
- [text-align-last](#propdef-text-align-last), in § 6.3
- [text-indent](#propdef-text-indent), in § 8.1
- [text-justify](#propdef-text-justify), in § 6.4
- [text-transform](#propdef-text-transform), in § 2.1
- [tracking](#tracking), in § 7.2
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
- [white space](#white-space), in § 4.1
- [white-space](#propdef-white-space), in § 3
- [white space characters](#white-space), in § 4.1
- [word-break](#propdef-word-break), in § 5.1
- [word separator](#word-separator), in § 7.1
- [word-separator character](#word-separator), in § 7.1
- [word-spacing](#propdef-word-spacing), in § 7.1
- [word-wrap](#propdef-word-wrap), in § 5.4
- [wrap](#wrapping), in § 5
- [wrapping](#wrapping), in § 5
- [writing system](#content-writing-system), in § 1.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="e1674793"></a>border
- \[CSS-BOX-4\] defines the following terms:
  - <a id="253362bb"></a>margin
  - <a id="a3a070bd"></a>padding
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="d0dc95c3"></a>inherit
  - <a id="4905669f"></a>inherited value
  - <a id="6b448e93"></a>initial value
  - <a id="19fd0eed"></a>legacy name alias
  - <a id="7a6ac42f"></a>legacy value alias
  - <a id="980ac56a"></a>shorthand property
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="7b249d0f"></a>out-of-flow box
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="d04cd927"></a>atomic inline
  - <a id="67c2e6e6"></a>block
  - <a id="8d18d112"></a>block container
  - <a id="0923db9e"></a>containing block
  - <a id="e8c16097"></a>display
  - <a id="f089a6e1"></a>inline box
  - <a id="6345c890"></a>inline formatting context
  - <a id="ff5f937c"></a>out-of-flow
  - <a id="143ef105"></a>root element
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="693c2890"></a>font-feature-settings
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="ea4fcd78"></a>baseline
  - <a id="a9330658"></a>line box
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="00d2e365"></a>ink overflow
  - <a id="e3488cd0"></a>scrollable overflow
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="99a0ef70"></a>first formatted line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="8fd8cc1b"></a>ruby
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="de48a940"></a>inner size
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="59e3c405"></a>intrinsic size contribution
  - <a id="7cb1c6db"></a>intrinsic sizing
  - <a id="8a39af7f"></a>max-content size
  - <a id="6a444fd6"></a>min-content size
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
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
  - <a id="82ddda8c"></a>inline-axis
  - <a id="4f19c3e6"></a>line-left
  - <a id="10d0d189"></a>line-right
  - <a id="99a9e10b"></a>logical width
  - <a id="953ffdad"></a>principal writing mode
  - <a id="90c7548c"></a>start
  - <a id="cec0d4db"></a>upright
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="0e782f73"></a>document language
- \[CSSOM-1\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
  - <a id="fc19454a"></a>resolved value
- \[HTML\] defines the following terms:
  - <a id="d4dbbbf0"></a>language
  - <a id="90d63fe4"></a>wbr
- \[INFRA\] defines the following terms:
  - <a id="be82f2e7"></a>normalize newlines

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 5 June 2026. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 22 April 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-uax11"></a>\[UAX11\]  
Ken Lunde 小林剣. [East Asian Width](https://www.unicode.org/reports/tr11/tr11-44.html). 24 July 2025. Unicode Standard Annex \#11. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr11&#x2F;tr11-44&#x2E;html](https://www.unicode.org/reports/tr11/tr11-44.html)

<a id="biblio-uax14"></a>\[UAX14\]  
Robin Leroy. [Unicode Line Breaking Algorithm](https://www.unicode.org/reports/tr14/tr14-55.html). 5 September 2025. Unicode Standard Annex \#14. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr14&#x2F;tr14-55&#x2E;html](https://www.unicode.org/reports/tr14/tr14-55.html)

<a id="biblio-uax24"></a>\[UAX24\]  
Ken Whistler. [Unicode Script Property](https://www.unicode.org/reports/tr24/tr24-39.html). 31 July 2025. Unicode Standard Annex \#24. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr24&#x2F;tr24-39&#x2E;html](https://www.unicode.org/reports/tr24/tr24-39.html)

<a id="biblio-uax29"></a>\[UAX29\]  
Josh Hadley. [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/tr29-47.html). 17 August 2025. Unicode Standard Annex \#29. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;tr29-47&#x2E;html](https://www.unicode.org/reports/tr29/tr29-47.html)

<a id="biblio-uax44"></a>\[UAX44\]  
Ken Whistler. [Unicode Character Database](https://www.unicode.org/reports/tr44/tr44-36.html). 27 August 2025. Unicode Standard Annex \#44. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr44&#x2F;tr44-36&#x2E;html](https://www.unicode.org/reports/tr44/tr44-36.html)

<a id="biblio-uax50"></a>\[UAX50\]  
Ken Lunde 小林剣; Koji Ishii 石井宏治. [Unicode Vertical Text Layout](https://www.unicode.org/reports/tr50/tr50-33.html). 24 July 2025. Unicode Standard Annex \#50. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr50&#x2F;tr50-33&#x2E;html](https://www.unicode.org/reports/tr50/tr50-33.html)

<a id="biblio-uax9"></a>\[UAX9\]  
Manish Goregaokar मनीष गोरेगांवकर; Robin Leroy. [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/tr9-51.html). 13 August 2025. Unicode Standard Annex \#9. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr9&#x2F;tr9-51&#x2E;html](https://www.unicode.org/reports/tr9/tr9-51.html)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-bcp47"></a>\[BCP47\]  
A. Phillips, Ed.; M. Davis, Ed.. [Tags for Identifying Languages](https://www.rfc-editor.org/info/rfc5646/). September 2009. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;info&#x2F;rfc5646&#x2F;](https://www.rfc-editor.org/info/rfc5646/)

<a id="biblio-clreq"></a>\[CLREQ\]  
Fuqiao Xue; Richard Ishida. [Requirements for Chinese Text Layout - 中文排版需求](https://www.w3.org/TR/clreq/). 3 July 2026. DNOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;clreq&#x2F;](https://www.w3.org/TR/clreq/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 5 May 2022. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-ilreq"></a>\[ILREQ\]  
Swaran Lata. [Indic Layout Requirements](https://www.w3.org/TR/ilreq/). 29 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;ilreq&#x2F;](https://www.w3.org/TR/ilreq/)

<a id="biblio-iso15924"></a>\[ISO15924\]  
Code for the representation of names of scripts.. 1998. ISO 15924:1998. Draft International Standard.

<a id="biblio-jis4051"></a>\[JIS4051\]  
Formatting rules for Japanese documents (日本語文書の組版方法).. 2004. JIS X 4051:2004. In Japanese.

<a id="biblio-jlreq"></a>\[JLREQ\]  
Hiroyuki Chiba; et al. [Requirements for Japanese Text Layout 日本語組版処理の要件(日本語版)](https://www.w3.org/TR/jlreq/). 11 August 2020. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;jlreq&#x2F;](https://www.w3.org/TR/jlreq/)

<a id="biblio-justify"></a>\[JUSTIFY\]  
Elika Etemad; Richard Ishida. [Approches to Full Justification](https://www.w3.org/International/articles/typography/justification). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;International&#x2F;articles&#x2F;typography&#x2F;justification](https://www.w3.org/International/articles/typography/justification)

<a id="biblio-typography"></a>\[TYPOGRAPHY\]  
Richard Ishida. [Language enablement index](https://www.w3.org/TR/typography/). 15 November 2024. DNOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;typography&#x2F;](https://www.w3.org/TR/typography/)

<a id="biblio-xml10"></a>\[XML10\]  
Tim Bray; et al. [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/xml/). 26 November 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;xml&#x2F;](https://www.w3.org/TR/xml/)

<a id="biblio-zhmark"></a>\[ZHMARK\]  
General Rules for Punctuation (《标点符号用法》).. 2011. GB/T 15834―2011. In Chinese..

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                         | Initial | Applies to            | Inh. | %ages                                                  | Anim­ation type         | Canonical order | Com­puted value                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------|---------|-----------------------|------|--------------------------------------------------------|------------------------|-----------------|-------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-hanging-punctuation⑥"></a></span><a href="#propdef-hanging-punctuation">hanging-punctuation</a>&#xA;      </strong> | none \| \[ first \|\| \[ force-end \| allow-end \] \|\| last \]                                               | none    | text                  | yes  | n/a                                                    | discrete               | per grammar     | specified keyword(s)                                                          |
| <strong><span><a id="ref-for-propdef-hyphens④"></a></span><a href="#propdef-hyphens">hyphens</a>&#xA;      </strong> | none \| manual \| auto                                                                                        | manual  | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-letter-spacing①⑦"></a></span><a href="#propdef-letter-spacing">letter-spacing</a>&#xA;      </strong> | normal \| \<length\>                                                                                          | normal  | inline boxes and text | yes  | n/a                                                    | by computed value type | n/a             | an absolute length                                                            |
| <strong><span><a id="ref-for-propdef-line-break①②"></a></span><a href="#propdef-line-break">line-break</a>&#xA;      </strong> | auto \| loose \| normal \| strict \| anywhere                                                                 | auto    | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-overflow-wrap①⓪"></a></span><a href="#propdef-overflow-wrap">overflow-wrap</a>&#xA;      </strong> | normal \| break-word \| anywhere                                                                              | normal  | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-tab-size③"></a></span><a href="#propdef-tab-size">tab-size</a>&#xA;      </strong> | \<number \[0,∞\]\> \| \<length \[0,∞\]\>                                                                      | 8       | text                  | yes  | n/a                                                    | by computed value type | n/a             | the specified number or absolute length                                       |
| <strong><span><a id="ref-for-propdef-text-align⑨"></a></span><a href="#propdef-text-align">text-align</a>&#xA;      </strong> | start \| end \| left \| right \| center \| justify \| match-parent \| justify-all                             | start   | block containers      | yes  | see individual properties                              | discrete               | n/a             | see individual properties                                                     |
| <strong><span><a id="ref-for-propdef-text-align-all⑧"></a></span><a href="#propdef-text-align-all">text-align-all</a>&#xA;      </strong> | start \| end \| left \| right \| center \| justify \| match-parent                                            | start   | block containers      | yes  | n/a                                                    | discrete               | n/a             | keyword as specified, except for match-parent which computes as defined above |
| <strong><span><a id="ref-for-propdef-text-align-last①①"></a></span><a href="#propdef-text-align-last">text-align-last</a>&#xA;      </strong> | auto \| start \| end \| left \| right \| center \| justify \| match-parent                                    | auto    | block containers      | yes  | n/a                                                    | discrete               | n/a             | keyword as specified, except for match-parent which computes as defined above |
| <strong><span><a id="ref-for-propdef-text-indent⑧"></a></span><a href="#propdef-text-indent">text-indent</a>&#xA;      </strong> | \[ \<length-percentage\> \] &#x26;&#x26; hanging? &#x26;&#x26; each-line? | 0       | block containers      | yes  | refers to block container’s own inline-axis inner size | by computed value type | per grammar     | computed \<length-percentage\> value, plus any specified keywords             |
| <strong><span><a id="ref-for-propdef-text-justify①⓪"></a></span><a href="#propdef-text-justify">text-justify</a>&#xA;      </strong> | auto \| none \| inter-word \| inter-character                                                                 | auto    | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword (except for the distribute legacy value)                    |
| <strong><span><a id="ref-for-propdef-text-transform①②"></a></span><a href="#propdef-text-transform">text-transform</a>&#xA;      </strong> | none \| \[capitalize \| uppercase \| lowercase \] \|\| full-width \|\| full-size-kana                         | none    | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-white-space②①"></a></span><a href="#propdef-white-space">white-space</a>&#xA;      </strong> | normal \| pre \| nowrap \| pre-wrap \| break-spaces \| pre-line                                               | normal  | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-word-break②①"></a></span><a href="#propdef-word-break">word-break</a>&#xA;      </strong> | normal \| keep-all \| break-all \| break-word                                                                 | normal  | text                  | yes  | n/a                                                    | discrete               | n/a             | specified keyword                                                             |
| <strong><span><a id="ref-for-propdef-word-spacing⑧"></a></span><a href="#propdef-word-spacing">word-spacing</a>&#xA;      </strong> | normal \| \<length\>                                                                                          | normal  | text                  | yes  | N/A                                                    | by computed value type | n/a             | an absolute length                                                            |

