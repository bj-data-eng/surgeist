Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Text Decoration Module Level 4](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Text Decoration Module Level 4

Source snapshot: https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/

Snapshot SHA-256: be6b9a7763ec76ae648f527cca1866ddf107f8f1dc3a607d5aeb24911252c421

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 25 source tables are presented as readable Markdown tables or explicit labeled layouts: 22 ordinary table conversions, 1 complex-table layout, 2 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Text Decoration Module Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module contains the features of CSS relating to text decoration, such as underlines, text shadows, and emphasis marks.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-text-decor” in the title, like this: “\[css-text-decor\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-text-decor%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This subsection is non-normative.</em>

This module covers text decoration, i.e. decorating the glyphs of the text once typeset according to font and typographic rules. (See [\[CSS-TEXT-3\]](#biblio-css-text-3) and [\[CSS-FONTS-3\]](#biblio-css-fonts-3).) Such features are traditionally used not only for purely decorative purposes, but also in some cases to show emphasis, for honorifics, and to indicate editorial changes such as insertions, deletions, and misspellings.

CSS Levels 1 and 2 only defined very basic [line decorations](#line-decoration) (underlines, overlines, and strike-throughs) appropriate to Western typographical traditions. Level 3 of this module added the ability to change the color, style, position, and continuity of these decorations, and also introduced [emphasis marks](#emphasis-marks) (traditionally used in East Asian typography), and [shadows](#text-shadow-property) (which were proposed then deferred from Level 2). Level 4 introduces additional controls over these decorations.

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the text-decorating features defined in [\[CSS-TEXT-DECOR-3\]](#biblio-css-text-decor-3).

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-selectordef-first-letter"></a>

<a id="ref-for-pseudo-element"></a>

All of the properties in this module can be applied to the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) [pseudo-elements](https://www.w3.org/TR/selectors-4/#pseudo-element).

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="terms"></a>1.3. Terminology

<a id="ref-for-typographic-character-unit"></a>

<a id="ref-for-typographic-letter-unit"></a>

<a id="ref-for-letter"></a>

<a id="ref-for-content-language"></a>

The terms [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) (<a id="character"></a>character), [typographic letter unit](https://www.w3.org/TR/css-text-3/#typographic-letter-unit) ([letter](https://www.w3.org/TR/css-syntax-3/#letter)), and [content language](https://www.w3.org/TR/css-text-3/#content-language) as used in this specification are defined in [\[CSS-TEXT-3\]](#biblio-css-text-3). Other terminology and concepts used in this specification are defined in [\[CSS2\]](#biblio-css2) and [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4).

## <a id="line-decoration"></a>2.  Line Decoration: Underline, Overline, and Strike-Through

<a id="ref-for-inline-box"></a>

<a id="ref-for-box"></a>

<a id="ref-for-box-fragment"></a>

<a id="ref-for-in-flow"></a>

<a id="ref-for-block-level"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-inline-formatting-context"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-inline-level"></a>

<a id="ref-for-ruby-container"></a>

<a id="ref-for-ruby-base-box"></a>

The following properties describe line decorations that are added to the content of an element. When specified on or propagated to an [inline box](https://www.w3.org/TR/css-display-3/#inline-box), that [box](https://www.w3.org/TR/css-display-3/#box) becomes a <a id="decorating-box"></a>decorating box for that decoration, applying the decoration to all its [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment). The decoration is then further propagated to any [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) [block-level](https://www.w3.org/TR/css-display-3/#block-level) boxes that split the inline (see [CSS2.1 section 9.2.1.1](https://www.w3.org/TR/CSS21/visuren.html#anonymous-block-level)). When specified on or propagated to a [block container](https://www.w3.org/TR/css-display-3/#block-container) that establishes an [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context), the decorations are propagated to an [anonymous](https://www.w3.org/TR/css-display-3/#anonymous) inline box that wraps all the <a id="ref-for-in-flow①"></a>in-flow [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) children of the <a id="ref-for-block-container①"></a>block container. When specified on or propagated to a [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container), the decorations are propagated only to the [ruby base](https://www.w3.org/TR/css-ruby-1/#ruby-base-box). For all other box types, the decorations are propagated to all <a id="ref-for-in-flow②"></a>in-flow children.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that text decorations are not propagated to any out-of-flow descendants, nor to the contents of atomic inline-level descendants such as inline blocks and inline tables. They are also not propagated to inline children of inline boxes, although the decoration is <em>applied</em> to such boxes.

<a id="ref-for-non-replaced"></a>

<a id="ref-for-inline-box①"></a>

<a id="ref-for-atomic-inline"></a>

<a id="ref-for-decorating-box"></a>

Underlines, overlines, and line-throughs are drawn only for [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), and are drawn across all text (including white space, letter spacing, and word spacing) except spacing (white space, letter spacing, and word spacing) at the beginning and end of a line. [Atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline), such as images and inline blocks, are not decorated. Margins, borders, and padding of the [decorating box](#decorating-box) are always skipped, however the margins, border, and padding of descendant <a id="ref-for-inline-box②"></a>inline boxes are not.

<a id="ref-for-decorating-box①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that CSS 2.1 required skipping margins, borders, and padding always. In Level 3 and beyond, by default only the margins, borders, and padding of the [decorating box](#decorating-box) are skipped. In the future CSS2.1 may be updated to match this new default.

<a id="ref-for-relative-position"></a>

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-text-shadow"></a>

<a id="ref-for-atomic-inline①"></a>

<a id="ref-for-non-replaced①"></a>

<a id="ref-for-inline-box③"></a>

<a id="ref-for-decorating-box②"></a>

[Relatively positioning](https://www.w3.org/TR/css-position-3/#relative-position) a descendant moves all text decorations applied to it along with the descendant’s text; it does not affect calculation of the decoration’s initial position on that line. The [visibility](https://www.w3.org/TR/css-display-3/#propdef-visibility) property, [text-shadow](#propdef-text-shadow), filters, and other graphical transformations likewise also affect all text decorations applied to that box—including decorations propagated from an ancestor box—and do not affect the calculation of their initial positions or thicknesses. (In the case of line decorations drawn over an [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline) or across the margins/borders/padding of a [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) [inline box](https://www.w3.org/TR/css-display-3/#inline-box), they are analogously associated with the affected atomic inline / non-replaced inline box rather than with the [decorating box](#decorating-box).)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d6d0ccd7"></a> In the following style sheet and document fragment:
>
> ```text
> blockquote { text-decoration: underline; color: blue; }
> em { display: block; }
> cite { color: fuchsia; }
> ```
>
> ```text
> <blockquote>
>  <p>
>   <span>
>    Help, help!
>    <em> I am under a hat! </em>
> 
>    <cite> —GwieF </cite>
>   </span>
>  </p>
> </blockquote>
> ```
>
> ...the underlining for the blockquote element is propagated to an anonymous inline box that surrounds the span element, causing the text "Help, help!" to be blue, with the blue underlining from the anonymous inline underneath it, the color being taken from the blockquote element. The `<em>text</em>` in the em block is also underlined, as it is in an in-flow block to which the underline is propagated. The final line of text is fuchsia, but the underline underneath it is still the blue underline from the anonymous inline element. ![Sample rendering of the above underline example](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-example.png) This diagram shows the boxes involved in the example above. The rounded aqua line represents the anonymous inline element wrapping the inline contents of the paragraph element, the rounded blue line represents the span element, and the orange lines represent the blocks.

<a id="ref-for-propdef-display"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Line decorations are propagated through the box tree, not through inheritance, and thus have no effect on descendants when specified on an element with [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display).

<a id="ref-for-propdef-text-decoration-line"></a>

### <a id="text-decoration-line-property"></a>2.1.  Text Decoration Lines: the [text-decoration-line](#propdef-text-decoration-line) property

| Field               | Definition                                                                                                                                                                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-line"></a>text-decoration-line                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ underline [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) overline <a id="ref-for-comb-any①"></a>\|\| line-through <a id="ref-for-comb-any②"></a>\|\| blink \] <a id="ref-for-comb-one①"></a>\| spelling-error <a id="ref-for-comb-one②"></a>\| grammar-error |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no (but see prose, above)                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                |

<a id="ref-for-longhand"></a>

<a id="ref-for-propdef-text-decoration"></a>

This property, which is a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, specifies what line decorations, if any, are added by the element. Values other than text-decoration-line cause the element to originate the indicated text decorations, and to apply and propagate it as described [above](#line-decoration).

<a id="ref-for-cascade"></a>

<a id="ref-for-propdef-text-decoration①"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-longhand①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unless it is desired for the color, style, and thickness of the lines to be set by declarations lower in the [cascade](https://www.w3.org/TR/css-cascade-6/#cascade), it is safer to use the [text-decoration](#propdef-text-decoration) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) instead of this [longhand](https://www.w3.org/TR/css-cascade-5/#longhand).

Values have the following meanings:

<a id="valdef-text-decoration-line-none"></a>none  
Neither produces nor inhibits text decoration.

<a id="valdef-text-decoration-line-underline"></a>underline  
Each line of text is underlined.

<a id="valdef-text-decoration-line-overline"></a>overline  
Each line of text has a line over it (i.e. on the opposite side from an underline).

<a id="valdef-text-decoration-line-line-through"></a>line-through  
Each line of text has a line through the middle.

<a id="valdef-text-decoration-line-blink"></a>blink  
The text blinks (alternates between visible and invisible). Conforming user agents may simply not blink the text. Note that not blinking the text is one technique to satisfy [checkpoint 3.3 of WAI-UAAG](https://www.w3.org/TR/UAAG/guidelines.html#tech-on-off-blinking-text). This value is <strong>deprecated</strong> in favor of Animations [\[CSS3-ANIMATIONS\]](#biblio-css3-animations).

<a id="valdef-text-decoration-line-spelling-error"></a>spelling-error  
This value indicates the type of text decoration used by the user agent to highlight spelling mistakes. Its appearance is UA-defined, and may be platform-dependent. <strong data-conversion-semantic="note">Note:</strong> It is often rendered as a red wavy underline.

<a id="valdef-text-decoration-line-grammar-error"></a>grammar-error  
This value indicates the type of text decoration used by the user agent to highlight grammar mistakes. Its appearance is UA defined, and may be platform-dependent. <strong data-conversion-semantic="note">Note:</strong> It is often rendered as a green wavy underline.

<a id="ref-for-vertical-writing-mode"></a>

<a id="ref-for-propdef-text-underline-position"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In [vertical writing modes](https://www.w3.org/TR/css-writing-modes-4/#vertical-writing-mode), [text-underline-position](#propdef-text-underline-position) can cause the underline and overline to switch sides. This allows the position of underlines to key off of language-specific preferences automatically.

<a id="ref-for-valdef-text-decoration-line-spelling-error"></a>

<a id="ref-for-valdef-text-decoration-line-grammar-error"></a>

<a id="ref-for-longhand②"></a>

<a id="ref-for-propdef-text-decoration②"></a>

<a id="ref-for-propdef-text-underline-position①"></a>

<a id="ref-for-propdef-color"></a>

<a id="ref-for-propdef-stroke"></a>

<a id="ref-for-propdef-fill"></a>

Since [spelling-error](#valdef-text-decoration-line-spelling-error) and [grammar-error](#valdef-text-decoration-line-grammar-error) decorations are entirely UA-defined, the UA <em>may</em> disregard the other [sub-properties](https://www.w3.org/TR/css-cascade-5/#longhand) of [text-decoration](#propdef-text-decoration), as well any other properties typically affecting the appearance of line decorations (such as [text-underline-position](#propdef-text-underline-position), [color](https://www.w3.org/TR/css-color-4/#propdef-color), [stroke](https://www.w3.org/TR/fill-stroke-3/#propdef-stroke), or [fill](https://www.w3.org/TR/fill-stroke-3/#propdef-fill)) when rendering these decorations. However, to the extent that honoring any of these properties would be meaningful and practical given the UA’s chosen rendering, the UA <em>should</em> apply them as modifications to its default styling.

<a id="ref-for-propdef-text-decoration-style"></a>

### <a id="text-decoration-style-property"></a>2.2.  Text Decoration Style: the [text-decoration-style](#propdef-text-decoration-style) property

| Field               | Definition                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-style"></a>text-decoration-style                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③"></a>solid [\|](https://www.w3.org/TR/css-values-4/#comb-one) double <a id="ref-for-comb-one④"></a>\| dotted <a id="ref-for-comb-one⑤"></a>\| dashed <a id="ref-for-comb-one⑥"></a>\| wavy |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | solid                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                |

<a id="ref-for-longhand③"></a>

<a id="ref-for-propdef-text-decoration③"></a>

<a id="ref-for-propdef-text-decoration-line①"></a>

This property, which is a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, sets the line-drawing style of underlines, overlines, and line-throughs specified on the element with [text-decoration-line](#propdef-text-decoration-line), and affects all decorations originating from this element even if descendant boxes specify a different style.

Values have the same meaning as for the [border-style properties](https://www.w3.org/TR/css-backgrounds-3/#the-border-style) [\[CSS-BACKGROUNDS-3\]](#biblio-css-backgrounds-3). <a id="valdef-text-decoration-style-wavy"></a>wavy indicates a wavy line.

<a id="ref-for-propdef-text-decoration-color"></a>

### <a id="text-decoration-color-property"></a>2.3.  Text Decoration Color: the [text-decoration-color](#propdef-text-decoration-color) property

| Field               | Definition                                                                       |
|---------------------|----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-color"></a>text-decoration-color                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color"></a>[\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | currentcolor                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | computed color                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                           |

<a id="ref-for-longhand④"></a>

<a id="ref-for-propdef-text-decoration④"></a>

<a id="ref-for-propdef-text-decoration-line②"></a>

This property, which is a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, sets the color of underlines, overlines, and line-throughs specified on the element with [text-decoration-line](#propdef-text-decoration-line), and affects all decorations originating from this element even if descendant boxes specify a different color.

<a id="ref-for-propdef-text-decoration-thickness"></a>

### <a id="text-decoration-thickness-property"></a>2.4.  Text Decoration Line Thickness: the [text-decoration-thickness](#propdef-text-decoration-thickness) property<a id="text-decoration-width-property"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-thickness"></a>text-decoration-thickness                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-percentage-value"></a><a id="ref-for-length-value"></a><a id="ref-for-comb-one⑦"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) from-font <a id="ref-for-comb-one⑧"></a>\| [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) <a id="ref-for-comb-one⑨"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword or absolute length                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                            |

<a id="ref-for-longhand⑤"></a>

<a id="ref-for-propdef-text-decoration⑤"></a>

<a id="ref-for-propdef-text-decoration-line③"></a>

This property, which is a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, sets the stroke thickness of underlines, overlines, and line-throughs specified on the element with [text-decoration-line](#propdef-text-decoration-line), and affects all decorations originating from this element even if descendant boxes specify a different thickness.

Values have the following meanings:

<a id="valdef-text-decoration-thickness-auto"></a>auto

The UA chooses an appropriate thickness for text decoration lines; see below.

<a id="valdef-text-decoration-thickness-from-font"></a>from-font

<a id="ref-for-valdef-text-decoration-thickness-auto"></a>

<a id="ref-for-first-available-font"></a>

If the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) has metrics indicating a preferred underline width, use that width, otherwise behaves as [auto](#valdef-text-decoration-thickness-auto).

<a id="ref-for-length-value①"></a>

<a id="valdef-text-decoration-thickness-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the thickness of text decoration lines as a fixed length. The UA must floor the actual value at one device pixel.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A length will inherit as a fixed value, and will not scale with the font.

<a id="ref-for-percentage-value①"></a>

<a id="valdef-text-decoration-thickness-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

Specifies the thickness of text decoration lines as a percentage of 1em. The UA must floor the actual value at one device pixel.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A percentage will inherit as a relative value, and will therefore scale with changes in the font as it inherits.

#### <a id="text-decoration-thickness"></a>2.4.1.  Automatic Thickness of Text Decoration Lines

<a id="ref-for-valdef-text-decoration-thickness-auto①"></a>

Some font formats (such as OpenType) can offer information about the appropriate thickness of a line decoration. The UA should use such font-based information when choosing [auto](#valdef-text-decoration-thickness-auto) line thicknesses wherever appropriate.

### <a id="line-position"></a>2.5.  Determining the Position and Thickness of Line Decorations

<a id="ref-for-propdef-text-underline-offset"></a>

<a id="ref-for-propdef-text-decoration-thickness①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d1c311e6"></a> This section is copied over from early drafts of Text Decoration Level 3. It is still under review, and needs integration with [text-underline-offset](#propdef-text-underline-offset) and [text-decoration-thickness](#propdef-text-decoration-thickness).

<a id="ref-for-decorating-box③"></a>

<a id="ref-for-propdef-text-decoration-skip"></a>

<a id="ref-for-considered-text"></a>

Since line decorations can span elements with varying font sizes and vertical alignments, the best position for a line decoration is not necessarily the ideal position dictated by the [decorating box](#decorating-box). Instead, it’s calculated, per line, from all text decorated by the <a id="ref-for-decorating-box④"></a>decorating box on that line, the <a id="considered-text"></a>considered text. However, descendants of the <a id="ref-for-decorating-box⑤"></a>decorating box that are skipped due to [text-decoration-skip](#propdef-text-decoration-skip), descendant inlines with <a id="ref-for-propdef-text-decoration-skip①"></a>text-decoration-skip: ink, and any descendants that do not participate in the <a id="ref-for-decorating-box⑥"></a>decorating box’s inline formatting context are excluded from the set of [considered text](#considered-text).

<a id="ref-for-over"></a>

<a id="ref-for-under"></a>

The line decoration positions are then calculated per line as follows (treating [over](https://www.w3.org/TR/css-writing-modes-4/#over)-positioned underlines as <a id="ref-for-over①"></a>over lines and [under](https://www.w3.org/TR/css-writing-modes-4/#under)-positioned overlines as <a id="ref-for-under①"></a>under lines):

<a id="ref-for-over②"></a>

[over](https://www.w3.org/TR/css-writing-modes-4/#over) lines

<a id="ref-for-considered-text①"></a>

<a id="ref-for-over③"></a>

Align the line decoration with respect to the highest [over](https://www.w3.org/TR/css-writing-modes-4/#over) EM-box edge of the [considered text](#considered-text).

alphabetic underlines

<a id="ref-for-propdef-vertical-align"></a>

<a id="ref-for-valdef-alignment-baseline-baseline"></a>

<a id="ref-for-considered-text②"></a>

The alphabetic underline position is calculated by taking the ideal offset (from the alphabetic baseline) of each run of [considered text](#considered-text), averaging those, and then using the lowest alphabetic baseline to actually position the line. (Alphabetic baselines can differ between [baseline](https://www.w3.org/TR/css-inline-3/#valdef-alignment-baseline-baseline)-aligned boxes if the dominant baseline is non-alphabetic.) To prevent superscripts and subscripts from throwing this position off-kilter, an inline with a non-initial computed [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) is treated as having the ideal underline position of its parent.

<a id="ref-for-under②"></a>

non-alphabetic [under](https://www.w3.org/TR/css-writing-modes-4/#under) lines

<a id="ref-for-considered-text③"></a>

<a id="ref-for-under③"></a>

Position the line decoration with respect to the lowest [under](https://www.w3.org/TR/css-writing-modes-4/#under) EM-box edge of the [considered text](#considered-text).

line-throughs

<a id="ref-for-propdef-vertical-align①"></a>

<a id="ref-for-considered-text④"></a>

<a id="ref-for-propdef-font-size"></a>

Line-throughs essentially use the same sort of averaging as for alphabetic underlines, but recompute the position when drawing across a descendant with a different computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size). (This ensures that the text remains effectively “crossed out” despite any font size changes.) For each run of [considered text](#considered-text) with the same <a id="ref-for-propdef-font-size①"></a>font-size, compute an ideal position averaged from its font metrics. To prevent superscripts and subscripts from throwing this position off-kilter, an inline with a non-initial computed [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) is treated as having the ideal underline position of its parent. Position the portion of the line across each decorated fragment at that position.

<a id="ref-for-considered-text⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3dd812b9"></a> For simplicity, line-throughs should draw over each element at that element’s preferred/averaged position. This can produce some undesirable jumpiness, but there doesn’t appear to be any way to avoid that which is correct in all instances, and all attempts are worryingly complex. What position should line-throughs adopt over elements that have a different font-size, but no [considered text](#considered-text)?

CSS does not define the thickness of line decorations. In determining the thickness of text decoration lines, user agents may consider the font sizes, faces, and weights of descendants to provide an appropriately averaged thickness.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-081ed4f4"></a> The following figure shows the averaging for underline:
>
> ![In the first rendering of the underlined text '1st a' with 'st' as a superscript, both the '1st' and the 'a' are rendered in a small font. In the second rendering, the 'a' is rendered in a larger font. In the third, both '1st' and 'a' are large.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-averaging.gif)
>
> In the three fragments of underlined text, the underline is drawn consecutively lower and thicker as the ratio of large text to small text increases.
>
> Using the same example, a line-through would in the second fragment, instead of averaging the two font sizes, split the line-through into two segments:
>
> ![](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/linethrough-averaging.gif)
>
> In both cases, however, the superscript, due to the vertical-alignment shift, has no effect on the position of the line.

<a id="ref-for-propdef-text-decoration⑥"></a>

### <a id="text-decoration-property"></a>2.6.  Text Decoration Shorthand: the [text-decoration](#propdef-text-decoration) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration"></a>text-decoration                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-text-decoration-color①"></a><a id="ref-for-propdef-text-decoration-style①"></a><a id="ref-for-propdef-text-decoration-thickness②"></a><a id="ref-for-comb-any③"></a><a id="ref-for-propdef-text-decoration-line④"></a>[\<'text-decoration-line'\>](#propdef-text-decoration-line) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'text-decoration-thickness'\>](#propdef-text-decoration-thickness) <a id="ref-for-comb-any④"></a>\|\| [\<'text-decoration-style'\>](#propdef-text-decoration-style) <a id="ref-for-comb-any⑤"></a>\|\| [\<'text-decoration-color'\>](#propdef-text-decoration-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                       |

<a id="ref-for-propdef-text-decoration-line⑤"></a>

<a id="ref-for-propdef-text-decoration-thickness③"></a>

<a id="ref-for-propdef-text-decoration-style②"></a>

<a id="ref-for-propdef-text-decoration-color②"></a>

This property is a shorthand for setting [text-decoration-line](#propdef-text-decoration-line), [text-decoration-thickness](#propdef-text-decoration-thickness), [text-decoration-style](#propdef-text-decoration-style), and [text-decoration-color](#propdef-text-decoration-color) in one declaration. Omitted values are set to their initial values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4a6e8734"></a> The following example underlines unvisited links with a solid blue underline in CSS1 and CSS2 UAs and a navy dotted underline in CSS3 UAs.
>
> ```text
> :link {
>   color: blue;
>   text-decoration: underline;
>   text-decoration: navy dotted underline; /* Ignored in CSS1/CSS2 UAs */
> }
> ```
<a id="ref-for-propdef-text-underline-position②"></a>

<a id="ref-for-propdef-text-decoration⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The shorthand purposefully omits the [text-underline-position](#propdef-text-underline-position) property, which is a language/writing-system–dependent setting that keys off the content, so that it can cascade and inherit independently from the (uninherited) stylistic settings of the [text-decoration](#propdef-text-decoration) shorthand.

<a id="ref-for-propdef-text-underline-position③"></a>

### <a id="text-underline-position-property"></a>2.7.  Text Underline Position: the [text-underline-position](#propdef-text-underline-position) property

| Field               | Definition                                                                                                                                                                                                                            |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-underline-position"></a>text-underline-position                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any⑥"></a><a id="ref-for-comb-one①⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ from-font <a id="ref-for-comb-one①①"></a>\| under \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ left <a id="ref-for-comb-one①②"></a>\| right \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                              |

<a id="ref-for-longhand⑥"></a>

<a id="ref-for-propdef-text-decoration⑧"></a>

<a id="ref-for-underline-zero-position"></a>

<a id="ref-for-propdef-text-underline-offset①"></a>

This property, which is <em>not</em> a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, sets the position of an underline with respect to the text, and defines its [zero position](#underline-zero-position) for further adjustment by [text-underline-offset](#propdef-text-underline-offset). It affects all decorations originating from this element, even if descendant boxes specify a different position. It does not affect underlines specified by ancestor elements.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7303f01e"></a> The following example styles modern Chinese, Japanese, and Korean texts with the appropriate underline positions in both horizontal and vertical text:
>
> ```text
> :root:lang(ja), [lang|=ja], :root:lang(ko), [lang|=ko] { text-underline-position: under right; }
> :root:lang(zh), [lang|=zh] { text-underline-position: under left; }
> ```
<a id="ref-for-underline-left"></a>

<a id="ref-for-underline-right"></a>

<a id="ref-for-underline-auto"></a>

If [left](#underline-left) or [right](#underline-right) is specified alone, [auto](#underline-auto) is also implied. Values have the following meanings:

<a id="underline-auto"></a>auto  
The user agent may use any algorithm to determine the underline’s position; however it must be placed at or under the alphabetic baseline.

<a id="ref-for-underline-under"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is suggested that the default underline position be close to the alphabetic baseline, unless that would either cross subscripted (or otherwise lowered) text or draw over glyphs from Asian scripts such as Han or Tibetan for which an alphabetic underline is too high: in such cases, shifting the underline lower or aligning to the em box edge as described for [under](#underline-under) may be more appropriate.

![In a typical Latin font, the underline is positioned slightly below the alphabetic baseline, leaving a gap between the line and the bottom of most Latin letters, but crossing through descenders such as the stem of a 'p'.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-position-alphabetic.png "text-underline-position: alphabetic")

A typical “alphabetic” underline is positioned just below the alphabetic baseline

<a id="valdef-text-underline-position-from-font"></a>from-font  
<a id="ref-for-valdef-text-underline-offset-auto"></a>

<a id="ref-for-first-available-font①"></a>

If the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) has metrics indicating a preferred underline offset, use that offset, otherwise behaves as [auto](#valdef-text-underline-offset-auto).

<a id="underline-under"></a>under  
<a id="ref-for-typographic-mode"></a>

<a id="ref-for-underline-right①"></a>

<a id="ref-for-underline-left①"></a>

The underline is positioned <i>under</i> the element’s text content. In this case the underline usually does not cross the descenders. (This is sometimes called “accounting” underline.) This value can be combined with [left](#underline-left) or [right](#underline-right) if a particular side is preferred in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

![In a typical Latin font, the underline is far enough below the text that it does not cross the bottom of a 'g'.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-position-under.png "text-underline-position: under")

<a id="ref-for-propdef-text-underline-position④"></a>

[text-underline-position: under](#propdef-text-underline-position)

<a id="ref-for-propdef-text-underline-position⑤"></a>

<a id="ref-for-propdef-text-decoration⑨"></a>

<a id="ref-for-underline-under①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95685877"></a> Because [text-underline-position](#propdef-text-underline-position) inherits, and is not reset by the [text-decoration](#propdef-text-decoration) shorthand, the following example switches the document to use [under](#underline-under) underlining, which can be more appropriate for writing systems with long, complicated descenders. It is also often useful for mathematical or chemical texts that use many subscripts.
> ```text
> :root { text-underline-position: under; }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The under value does not guarantee that the underline will not conflict with glyphs, as some fonts have descenders or diacritics that extend below the font’s descent metrics.

<a id="underline-left"></a>left  
<a id="ref-for-underline-under②"></a>

<a id="ref-for-typographic-mode①"></a>

In vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode), the underline is aligned as for [under](#underline-under), except it is always aligned to the left edge of the text. If this causes the underline to be drawn on the "over" side of the text, then an overline also switches sides and is drawn on the "under" side.

<a id="underline-right"></a>right  
<a id="ref-for-underline-under③"></a>

<a id="ref-for-typographic-mode②"></a>

In vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode), the underline is aligned as for [under](#underline-under), except it is always aligned to the right edge of the text. If this causes the underline to be drawn on the "over" side of the text, then an overline also switches sides and is drawn on the "under" side.

<a id="fig-text-underline-position"></a>

| Column 1                                                                                                                                                                                                                                                 | Column 2                                                                                                                                                                                                                                                     |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ![In mixed Japanese-Latin vertical text, 'text-underline-position: left' places the underline on the left side of the text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-position-left.png "text-underline-position: left") | ![In mixed Japanese-Latin vertical text, 'text-underline-position: right' places the underline on the right side of the text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-position-right.png "text-underline-position: right") |
| <a id="ref-for-underline-left②"></a>[left](#underline-left)                                                                                                                                                                                                               | <a id="ref-for-underline-right②"></a>[right](#underline-right)                                                                                                                                                                                                                 |

<a id="ref-for-typographic-mode③"></a>

<a id="ref-for-propdef-text-underline-position⑥"></a>

<a id="ref-for-underline-left③"></a>

<a id="ref-for-underline-right③"></a>

<a id="ref-for-underline-auto①"></a>

In vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode), the [text-underline-position](#propdef-text-underline-position) values [left](#underline-left) and [right](#underline-right) allow placing the underline on either side of the text. (In horizontal <a id="ref-for-typographic-mode④"></a>typographic modes, both values are treated as [auto](#underline-auto).)

<a id="ref-for-propdef-text-underline-offset②"></a>

### <a id="underline-offset"></a>2.8.  Text Underline Offset: the [text-underline-offset](#propdef-text-underline-offset) property

| Field               | Definition                                                                                                                                                                                                                                                                   |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-underline-offset"></a>text-underline-offset                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-percentage-value②"></a><a id="ref-for-length-value②"></a><a id="ref-for-comb-one①③"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) <a id="ref-for-comb-one①④"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword or absolute length                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                            |

<a id="ref-for-longhand⑦"></a>

<a id="ref-for-propdef-text-decoration①⓪"></a>

<a id="ref-for-underline-zero-position①"></a>

This property, which is <em>not</em> a [sub-property](https://www.w3.org/TR/css-cascade-5/#longhand) of the [text-decoration](#propdef-text-decoration) shorthand, sets the offset of underlines from their [zero position](#underline-zero-position). Positive offsets represent distances outward from the text; negative offsets inward. It affects all decorations originating from this element, even if descendant boxes specify a different position. It does not affect underlines specified by ancestor elements.

Values have the following meanings:

<a id="valdef-text-underline-offset-auto"></a>auto

The UA chooses an appropriate offset for underlines.

<a id="ref-for-propdef-text-underline-position⑦"></a>

<a id="ref-for-valdef-text-underline-position-from-font"></a>

However, this offset must be zero if the computed value of [text-underline-position](#propdef-text-underline-position) is [from-font](#valdef-text-underline-position-from-font) and the UA was able to extract an appropriate metric to use from the font.

<a id="ref-for-length-value③"></a>

<a id="valdef-text-underline-offset-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the offset of underlines as a fixed length.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A length will inherit as a fixed value, and will not scale with the font.

<a id="ref-for-percentage-value③"></a>

<a id="valdef-text-underline-offset-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

Specifies the offset of underlines as a percentage of 1em.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A percentage will inherit as a relative value, and will therefore scale with changes in the font as it inherits.

<a id="ref-for-propdef-text-decoration-line⑥"></a>

<a id="ref-for-valdef-text-decoration-line-spelling-error①"></a>

<a id="ref-for-valdef-text-decoration-line-grammar-error①"></a>

<a id="ref-for-propdef-text-underline-position⑧"></a>

When the value of the [text-decoration-line](#propdef-text-decoration-line) property is either [spelling-error](#valdef-text-decoration-line-spelling-error) or [grammar-error](#valdef-text-decoration-line-grammar-error), the UA may ignore the value of [text-underline-position](#propdef-text-underline-position).

#### <a id="line-offset-zero"></a>2.8.1.  Underline Offset Origin (Zero Position)

<a id="ref-for-propdef-text-underline-position⑨"></a>

The <a id="underline-zero-position"></a>zero position of the underline depends on the value of [text-underline-position](#propdef-text-underline-position) as detailed below.

<a id="ref-for-propdef-text-underline-position①⓪"></a>

<a id="ref-for-propdef-text-underline-offset③"></a>

| <a id="ref-for-propdef-text-underline-position①①"></a>[text-underline-position](#propdef-text-underline-position) | Zero Position                                                               | Positive Direction                                                           |
|--------------------------------------------------------------------------------|-----------------------------------------------------------------------------|------------------------------------------------------------------------------|
| <a id="ref-for-underline-auto②"></a>[auto](#underline-auto)                                     | alphabetic baseline                                                         | <a id="ref-for-under④"></a>[under](https://www.w3.org/TR/css-writing-modes-4/#under) |
| <a id="ref-for-valdef-text-underline-position-from-font①"></a>[from-font](#valdef-text-underline-position-from-font)      | position specified by the font metrics, falling back to alphabetic baseline | <a id="ref-for-under⑤"></a>[under](https://www.w3.org/TR/css-writing-modes-4/#under) |
| <a id="ref-for-underline-under④"></a>[under](#underline-under)                                   | text-under edge                                                             | <a id="ref-for-under⑥"></a>[under](https://www.w3.org/TR/css-writing-modes-4/#under) |
| <a id="ref-for-underline-left④"></a>[left](#underline-left)                                     | text-under (left) edge                                                      | <a id="ref-for-under⑦"></a>[under](https://www.w3.org/TR/css-writing-modes-4/#under) |
| <a id="ref-for-underline-right④"></a>[right](#underline-right)                                   | text-over (right) edge                                                      | <a id="ref-for-over④"></a>[over](https://www.w3.org/TR/css-writing-modes-4/#over)   |

Interaction of [text-underline-position](#propdef-text-underline-position) and [text-underline-offset](#propdef-text-underline-offset)

The underline is aligned to the outside of the specified position (extending its thickness in the positive direction only).

<a id="ref-for-propdef-text-underline-offset④"></a>

Any automatic adjustments made to accommodate descendant content are maintained; the [text-underline-offset](#propdef-text-underline-offset) is in addition to those.

#### <a id="line-auto-offset"></a>2.8.2.  Using Font Metrics for Automatic Positioning

<a id="ref-for-valdef-text-underline-offset-auto①"></a>

<a id="ref-for-valdef-text-underline-position-from-font②"></a>

<a id="ref-for-propdef-text-underline-position①②"></a>

Some font formats (such as OpenType) can offer information about the appropriate position of a line decoration. The UA should use such font-based information in its choice of [auto](#valdef-text-underline-offset-auto) offset wherever appropriate, and must use such information when [from-font](#valdef-text-underline-position-from-font) is specified for [text-underline-position](#propdef-text-underline-position).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Typically, OpenType font metrics give the position of an alphabetic underline; in some cases (especially in CJK fonts), it gives the position of a under left underline. (In this case, the font’s underline metrics typically touch the bottom edge of the em box). The UA may but is not required to correct for incorrect font metrics.

### <a id="text-line-constancy"></a>2.9.  Text Decoration Line Uniformity

<a id="ref-for-propdef-text-underline-position①③"></a>

<a id="ref-for-propdef-text-underline-offset⑤"></a>

<a id="ref-for-propdef-text-decoration-thickness④"></a>

The exact position and thickness of line decorations depends on the values of [text-underline-position](#propdef-text-underline-position), [text-underline-offset](#propdef-text-underline-offset), and [text-decoration-thickness](#propdef-text-decoration-thickness) as defined above, and is otherwise UA-defined. However, for underlines and overlines the UA must use a single thickness and position on each line for the decorations deriving from a single <i>decorating box</i>.

![A single underline drawn under varying font sizes and vertical positions must be a single line.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-single.png) vs. ![Drawing multiple line segments, each with the position and thickness appropriate to the decorated text, is incorrect.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-broken.png)

Correct and incorrect rendering of `<u>A<sup>B</sup><big>C</big>D</u>`

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, since line decorations can span elements with varying font sizes and vertical alignments, the best position for a line decoration is not necessarily the ideal position dictated by the <i>decorating box</i>. For example, an overline positioned to a small font will effectively become a line-through if the element contains text in a significantly larger font-size. Even for underlines, if the text is not aligned to the alphabetic baseline (for example, in vertical typesetting styles, text is aligned by its central baseline by default [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4)) an underline will cut through descendant text of a larger font-size. UA consideration of descendant content will therefore result in better typography.
>
> ![](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/leftline-cross.png) ![](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/leftline-under.png)
>
> Due to the central baseline alignment of vertical text, a left-side underline on small vertical text will cut through the text of a child with a larger font size. The underline is not allowed to be broken, but adjusting its position further to the left properly accommodates all of the underlined text.

<a id="ref-for-propdef-vertical-align②"></a>

<a id="ref-for-valdef-alignment-baseline-baseline①"></a>

<a id="ref-for-propdef-font-variant-position"></a>

UAs <em>must</em> adjust line positions to match the shifted metrics of <i>decorating boxes</i> shifted with [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) values other than [baseline](https://www.w3.org/TR/css-inline-3/#valdef-alignment-baseline-baseline) [\[CSS2\]](#biblio-css2) or subscripted/superscripted via [font-variant-position](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant-position) [\[CSS-FONTS-3\]](#biblio-css-fonts-3), but <em>must not</em> adjust the line position or thickness in response to descendants of a <i>decorating box</i> that are so styled (even though it <em>may</em> adjust the position to accommodate descendants that are not so styled, such as those merely typeset in a different font size as noted above). This allows superscripts and subscripts to be properly decorated (underlined, struck through, etc.) but prevents them from distorting or breaking the positioning of such decorations on their ancestors.

![An underline for just the superscript 'st' in '1st' is drawn just below the superscript, whereas an underline for the entire text is drawn at the appropriate position for full-size text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/underline-superscript.png)

Example of underline applied to superscripted text vs. underline applied to text containing a superscript

<a id="ref-for-propdef-text-decoration-skip②"></a>

### <a id="text-decoration-skipping"></a>2.10.  Text Decoration Line Continuity: the [text-decoration-skip](#propdef-text-decoration-skip) shorthand and its sub-properties

<a id="ref-for-propdef-text-decoration-skip-ink"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e6a42afa"></a> The CSSWG resolved to be split skipping functionality into individual properties along the lines of [text-decoration-skip-ink](#propdef-text-decoration-skip-ink), to improve its cascading behavior. See [discussion](https://github.com/w3c/csswg-drafts/issues/843) and [resolution](https://lists.w3.org/Archives/Public/www-style/2017Feb/0049.html). This section is a rough draft and has not yet been vetted by the CSSWG

| Field               | Definition                                                                      |
|---------------------|---------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip"></a>text-decoration-skip                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑤"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | See individual properties                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                        |

<a id="ref-for-propdef-text-decoration-skip③"></a>

<a id="ref-for-propdef-text-decoration-skip-self"></a>

<a id="ref-for-propdef-text-decoration-skip-box"></a>

<a id="ref-for-propdef-text-decoration-skip-inset"></a>

<a id="ref-for-propdef-text-decoration-skip-spaces"></a>

<a id="ref-for-propdef-text-decoration-skip-ink①"></a>

<a id="ref-for-decorating-box⑦"></a>

<a id="ref-for-valdef-text-decoration-skip-none"></a>

The [text-decoration-skip](#propdef-text-decoration-skip) property and its sub-properties ([text-decoration-skip-self](#propdef-text-decoration-skip-self), [text-decoration-skip-box](#propdef-text-decoration-skip-box), [text-decoration-skip-inset](#propdef-text-decoration-skip-inset), [text-decoration-skip-spaces](#propdef-text-decoration-skip-spaces), [text-decoration-skip-ink](#propdef-text-decoration-skip-ink)) control interruptions in line decorations for which the element or an ancestor is the [decorating box](#decorating-box). The <a id="valdef-text-decoration-skip-none"></a>none value sets all sub-properties to [none](#valdef-text-decoration-skip-none), and the <a id="valdef-text-decoration-skip-auto"></a>auto value sets all sub-properties to their initial values.

<a id="ref-for-valdef-text-decoration-skip-none①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-070668ae"></a> Is this [none](#valdef-text-decoration-skip-none) definition Web-compatible? Do we also need to add an ink value for Web-compat?

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that these properties inherit and that descendant elements can have a different setting.

The following addition is made to the default UA stylesheet for HTML:

```text
ins, del { text-decoration-skip: none; }
```
<a id="ref-for-propdef-text-decoration-line⑦"></a>

<a id="ref-for-valdef-text-decoration-line-spelling-error②"></a>

<a id="ref-for-valdef-text-decoration-line-grammar-error②"></a>

When the value of the [text-decoration-line](#propdef-text-decoration-line) property is either [spelling-error](#valdef-text-decoration-line-spelling-error) or [grammar-error](#valdef-text-decoration-line-grammar-error), the UA may ignore any or all of these properties.

<a id="ref-for-propdef-text-decoration-skip-self①"></a>

#### <a id="text-decoration-skip-self-property"></a>2.10.1.  Skipping Spaces: the [text-decoration-skip-self](#propdef-text-decoration-skip-self) property

| Field               | Definition                                                                         |
|---------------------|------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip-self"></a>text-decoration-skip-self                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) objects |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | objects                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                           |

<a id="ref-for-propdef-text-decoration-skip④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a29a7026"></a> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG.

This property specifies whether any text decoration lines drawn by its ancestors are propagated to or drawn across the element. Values have the following meanings:

<a id="valdef-text-decoration-skip-self-none"></a>none  
<a id="ref-for-decorating-box⑧"></a>

Skip nothing: line decorations from ancestor [decorating boxes](#decorating-box) are propagated to or drawn across this box, as appropriate.

<a id="valdef-text-decoration-skip-self-objects"></a>objects  
Skip this element (its entire margin box) if it is an atomic inline (such as an image or inline-block).

<a id="ref-for-propdef-text-decoration-skip-box①"></a>

#### <a id="text-decoration-skip-box-property"></a>2.10.2.  Skipping Spaces: the [text-decoration-skip-box](#propdef-text-decoration-skip-box) property

| Field               | Definition                                                                     |
|---------------------|--------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip-box"></a>text-decoration-skip-box                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑦"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) all |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                       |

<a id="ref-for-propdef-text-decoration-skip⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a29a7026①"></a> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG.

This property specifies what parts of the element’s box area any text decoration affecting the element must skip over. It controls only text decoration lines drawn by its ancestors. Values have the following meanings:

<a id="valdef-text-decoration-skip-box-none"></a>none  
<a id="ref-for-decorating-box⑨"></a>

Skip nothing: line decorations from ancestor [decorating boxes](#decorating-box) are drawn from margin edge to margin edge.

<a id="valdef-text-decoration-skip-box-all"></a>all  
<a id="ref-for-decorating-box①⓪"></a>

When drawing text decoration lines applied to an ancestor [decorating box](#decorating-box), skip over the box’s own margin, border, and padding areas and only draw line decorations within its content area.

<a id="ref-for-decorating-box①①"></a>

This value only has an effect for decorations imposed by an ancestor; a [decorating box](#decorating-box) never draws over its own box decoration.

<a id="ref-for-propdef-text-decoration-skip-inset①"></a>

#### <a id="text-decoration-skip-inset-property"></a>2.10.3.  Inset Edges: the [text-decoration-skip-inset](#propdef-text-decoration-skip-inset) property

| Field               | Definition                                                                      |
|---------------------|---------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip-inset"></a>text-decoration-skip-inset                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑧"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                        |

<a id="ref-for-propdef-text-decoration-skip⑥"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a29a7026②"></a> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG.

This property specifies what parts of the element’s box area any text decoration affecting the element must skip over. It controls all text decoration lines drawn by the element, but not any text decoration lines drawn by its ancestors. Values have the following meanings:

<a id="valdef-text-decoration-skip-inset-none"></a>none  
Skip nothing: text-decoration is drawn from box edge to box edge.

<a id="valdef-text-decoration-skip-inset-auto"></a>auto  
<a id="ref-for-decorating-box①②"></a>

The UA must place the start and end of the line inwards slightly from the content edge of the [decorating box](#decorating-box) so that, e.g. two underlined elements side-by-side do not appear to have a single underline. The size of the inset is up to the user agent (e.g. half a line thickness) but must not be zero. (This is important in Chinese, where underlining is a form of punctuation.)

![An underline below a series of Chinese characters has a gap between two adjacent underlining elements.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/decoration-skip-inset.png "text-decoration-skip-inset: auto")

<a id="ref-for-propdef-text-decoration-skip-inset②"></a>

[text-decoration-skip-inset: auto](#propdef-text-decoration-skip-inset) for `<u>石井</u><u>艾俐俐</u>`

<a id="ref-for-propdef-text-decoration-skip⑦"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-505740d9"></a> This might want to be a standalone property rather than part of the [text-decoration-skip](#propdef-text-decoration-skip) set. See also [Issue 4557](https://github.com/w3c/csswg-drafts/issues/4557), about controlling the line length explicitly.

<a id="ref-for-propdef-text-decoration-skip-spaces①"></a>

#### <a id="text-decoration-skip-spaces-property"></a>2.10.4.  Skipping Spaces: the [text-decoration-skip-spaces](#propdef-text-decoration-skip-spaces) property

| Field               | Definition                                                                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip-spaces"></a>text-decoration-skip-spaces                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any⑦"></a><a id="ref-for-comb-one①⑨"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) all <a id="ref-for-comb-one②⓪"></a>\| \[ start [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) end \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | start end                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                     |

<a id="ref-for-valdef-text-decoration-skip-spaces-none"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4229bfce"></a> Should the initial value be [none](#valdef-text-decoration-skip-spaces-none) for Web-compat? If not, INS and DEL at least should be assigned <a id="ref-for-valdef-text-decoration-skip-spaces-none①"></a>none in the UA default stylesheet. See also [Issue 4653](https://github.com/w3c/csswg-drafts/issues/4653).

This property specifies whether text decoration skips any spaces. It controls all text decoration lines drawn by the element and also any text decoration lines drawn by its ancestors. Values have the following meanings:

<a id="valdef-text-decoration-skip-spaces-none"></a>none  
<a id="ref-for-spacer"></a>

[Spacers](#spacer) are not skipped. They are decorated just like any other character.

<a id="valdef-text-decoration-skip-spaces-all"></a>all  
<a id="ref-for-propdef-word-spacing"></a>

<a id="ref-for-propdef-letter-spacing"></a>

<a id="ref-for-spacer①"></a>

Skip all [spacers](#spacer), plus any adjacent [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) or [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing).

<a id="valdef-text-decoration-skip-spaces-start"></a>start  
<a id="ref-for-propdef-word-spacing①"></a>

<a id="ref-for-propdef-letter-spacing①"></a>

<a id="ref-for-spacer②"></a>

Skip all [spacers](#spacer), plus any adjacent [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) or [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), when located at the start of the line.

<a id="valdef-text-decoration-skip-spaces-end"></a>end  
<a id="ref-for-propdef-word-spacing②"></a>

<a id="ref-for-propdef-letter-spacing②"></a>

<a id="ref-for-spacer③"></a>

Skip all [spacers](#spacer), plus any adjacent [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) or [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), when located at the end of the line.

<a id="ref-for-typographic-character-unit①"></a>

For the purpose of this property, a <a id="spacer"></a>spacer is any [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) with the Unicode White_Space property [\[UAX44\]](#biblio-uax44) except U+202F NARROW NO-BREAK SPACE, or any [word separator](https://www.w3.org/TR/css-text-3/#word-separator).

<a id="ref-for-propdef-text-decoration-skip-ink②"></a>

#### <a id="text-decoration-skip-ink-property"></a>2.10.5.  Skipping Glyphs: the [text-decoration-skip-ink](#propdef-text-decoration-skip-ink) property

| Field               | Definition                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-decoration-skip-ink"></a>text-decoration-skip-ink                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②①"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one②②"></a>\| all |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-decoration-skip-ink-auto"></a>[auto](#valdef-text-decoration-skip-ink-auto)                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                  |

This property controls how overlines and underlines are drawn when they cross over a glyph. It affects all decorations originating from this element even if descendant boxes specify a different style.

When enabled, decoration lines skip over where glyphs are drawn: interrupt the decoration line to let the shape of the text show through where the text decoration would otherwise cross over a glyph. The UA must also skip a small distance to either side of the glyph outline.

![An alphabetic underline through Myanmar text skips around descenders and the vertical strokes of combining characters that drop below the alphabetic baseline.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/decoration-skip-ink.png "Skipping Descenders When Drawing an Underline")

Skipping Glyph Ink

<a id="ref-for-valdef-text-decoration-skip-ink-auto①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-01cb5447"></a> Ideographic scripts do not want to skip when [auto](#valdef-text-decoration-skip-ink-auto). How can we define this behavior? Are there more scripts wanting not to skip? Need some normative text describe how <a id="ref-for-valdef-text-decoration-skip-ink-auto②"></a>auto works. See [telcon minutes](https://lists.w3.org/Archives/Public/www-style/2017Feb/0069.html), [alreq#86](https://github.com/w3c/alreq/issues/86), [csswg#1288](https://github.com/w3c/csswg-drafts/issues/1288)

This property only applies to overlines and underlines; line-throughs are always continuous.

<a id="valdef-text-decoration-skip-ink-auto"></a>auto  
UAs <em>may</em> interrupt underlines and overlines where the line would cross glyph ink and to some distance to either side of the glyph outline. UAs <em>should</em> consider the script of the text (see note below) when determining whether to apply ink-skipping behavior to a given range of content.

<a id="valdef-text-decoration-skip-ink-all"></a>all  
UAs <em>must</em> interrupt underlines and overlines where the line would cross glyph ink and to some distance to either side of the glyph outline.

<a id="valdef-text-decoration-skip-ink-none"></a>none  
UA <em>must</em> draw continuous underlines and overlines, without interruptions when they cross over a glyph.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementation experience shows that ink-skipping behavior often produces undesirable results when underlined text includes ideographic characters, as the underline position (depending on the font and user agent involved) often clashes with almost all the glyphs, such that only occasional fragments of the line remain to be rendered.
>
> <a id="ref-for-propdef-text-underline-position①④"></a>
>
> <a id="ref-for-propdef-text-underline-offset⑥"></a>
>
> In principle, this could be resolved by authors using [text-underline-position: under](#propdef-text-underline-position) (or possibly [text-underline-offset](#propdef-text-underline-offset)) to move the underline to a lower position that does not clash with the glyphs, but this is not always feasible, even if the user agent supports these properties and the author is aware of their potential. In particular, when a page contains arbitrary user-generated content, the author responsible for the design may not know whether CJK content will be present. And with mixed-script content, an underline position designed to work well for CJK content may look bad if the majority of the text is non-CJK.
>
> <a id="ref-for-valdef-text-decoration-skip-ink-auto③"></a>
>
> Therefore, when [auto](#valdef-text-decoration-skip-ink-auto) is in effect, a UA that implements ink-skipping <em>should</em> refrain from doing so in CJK contexts. (Authors who <em>do</em> want ink-skipping applied to CJK content can use the always value to explicitly request this.)
>
> Primarily, this means <em>not</em> applying ink-skipping for characters whose Unicode [Script property](http://unicode.org/Public/UCD/latest/ucd/Scripts.txt) is any of the CJK scripts Han, Hiragana, Katakana, Bopomofo, or Hangul, or for characters whose Script property is Inherited or Common, and whose [ScriptExtensions property](http://unicode.org/Public/UCD/latest/ucd/ScriptExtensions.txt) includes one or more of the CJK scripts.
>
> In addition, characters with a Unicode script property of Common and Inherited (primarily generic punctuation and symbols) need to be considered, as these may be used as part of a run of CJK-script content, and it is desirable to treat all text within a given script run in a consistent way. Therefore, the UA <em>should</em> resolve the text into script runs as described in the [“Implementation Notes”](https://www.unicode.org/reports/tr24/#Usage_Model) of [\[UAX24\]](#biblio-uax24) “Unicode Script Property”, in particular subsections 5.1 and 5.2. After applying the heuristics described there (or a similar analysis of scripts), the UA <em>should</em> disable ink-skipping for all ranges of text that are determined to be in a CJK script.
>
> <a id="ref-for-valdef-text-decoration-skip-ink-auto④"></a>
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > <a id="issue-da3f50fc"></a> Are there other (non-CJK) scripts where it would be preferable to disable ink-skipping by default (when [auto](#valdef-text-decoration-skip-ink-auto) is in effect)? Perhaps Yi? Arabic? (See also discussion in [Issue 1288](https://github.com/w3c/csswg-drafts/issues/1288).)

#### <a id="ink-skip-shape"></a>2.10.6.  Shaping Interruptions

When the UA interrupts underlines or overlines at glyph boundaries, the shape of the line at that boundary should follow the shape of the glyph.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this specification intentionally does not mandate a particular method for “following the shape” of the glyph so that UAs can take appropriate measures to handle aesthetic and performance considerations. For example, a UA could assume square line endings below a certain size threshold for performance reasons; or use trapezoidal endings to approximate curves, especially on thinner line decorations. In terms of aesthetic considerations, the UA might also consider what happens when the glyph boundary intersects only part of the line thickness or is slanted close to the horizontal—following the curve exactly could result in typographically-awkward wisps of underline. Whether to show the line within enclosed areas of a glyph is yet another consideration.
>
> ![Take, for example, the word “goal” with an underline striking through the bottom loop of the “g”. Depending on the position and thickness of the underline, we might see the entire thickness of the underline, or only part of it within the “g”. This example shows a masked-out underline in two positions. In the left pair the underline passes through the center of the bowl of the “g”: the full thickness of the underline shows through the center, filling it. In the right pair the underline is slightly lower, and thus the portion of the underline within the “g” can only show a partial thickness.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/skip-ink-wisp.png "Wispy Leftovers of a Masked-Out Underline")
>
> Hiding the portion of the underline within the bowl gives a cleaner look to the type, while the curved ends of the underline outside it suggest the continuity of the underline through the letter by hugging its outer contour.

## <a id="emphasis-marks"></a>3.  Additional Controls for Emphasis Marks

East Asian documents traditionally use small symbols next to each glyph to emphasize a run of text. For example:

![Example of emphasis in Japanese appearing over the text](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-ja.png)

Accent emphasis (shown in blue for clarity) applied to Japanese text

<a id="ref-for-propdef-text-emphasis"></a>

<a id="ref-for-propdef-text-emphasis-style"></a>

<a id="ref-for-propdef-text-emphasis-color"></a>

<a id="ref-for-propdef-text-emphasis-position"></a>

The [text-emphasis](#propdef-text-emphasis) shorthand, and its [text-emphasis-style](#propdef-text-emphasis-style) and [text-emphasis-color](#propdef-text-emphasis-color) longhands, can be used to apply such marks to the text. The [text-emphasis-position](#propdef-text-emphasis-position) property, which inherits separately, allows setting the emphasis marks’ position with respect to the text.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5d2e97f6"></a> See also [issue about continuity in size/position](https://github.com/w3c/csswg-drafts/issues/1892).

<a id="ref-for-propdef-text-emphasis-style①"></a>

### <a id="text-emphasis-style-property"></a>3.1.  Emphasis Mark Style: the [text-emphasis-style](#propdef-text-emphasis-style) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-emphasis-style"></a>text-emphasis-style                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-string-value"></a><a id="ref-for-comb-any⑧"></a><a id="ref-for-comb-one②③"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ \[ filled <a id="ref-for-comb-one②④"></a>\| open \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ dot <a id="ref-for-comb-one②⑤"></a>\| circle <a id="ref-for-comb-one②⑥"></a>\| double-circle <a id="ref-for-comb-one②⑦"></a>\| triangle <a id="ref-for-comb-one②⑧"></a>\| sesame \] \] <a id="ref-for-comb-one②⑨"></a>\| [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-text-emphasis-style-none"></a>the keyword [none](#valdef-text-emphasis-style-none), a pair of keywords representing the shape and fill, or a string                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                        |

This property applies emphasis marks to the element’s text. Values have the following meanings:

<a id="valdef-text-emphasis-style-none"></a>none

No emphasis marks.

<a id="valdef-text-emphasis-style-filled"></a>filled

The shape is filled with solid color.

<a id="valdef-text-text-emphasis-open"></a>open

The shape is hollow.

<a id="valdef-text-emphasis-style-dot"></a>dot

Display small circles as marks. The filled dot is U+2022 '•', and the open dot is U+25E6 '◦'.

<a id="valdef-text-emphasis-style-circle"></a>circle

Display large circles as marks. The filled circle is U+25CF '●', and the open circle is U+25CB '○'.

<a id="valdef-text-emphasis-style-double-circle"></a>double-circle

Display double circles as marks. The filled double-circle is U+25C9 '◉', and the open double-circle is U+25CE '◎'.

<a id="valdef-text-emphasis-style-triangle"></a>triangle

Display triangles as marks. The filled triangle is U+25B2 '▲', and the open triangle is U+25B3 '△'.

<a id="valdef-text-emphasis-style-sesame"></a>sesame

Display sesames as marks. The filled sesame is U+FE45 '﹅', and the open sesame is U+FE46 '﹆'.

<a id="ref-for-string-value①"></a>

<a id="valdef-text-emphasis-style-string"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value)

Display the given string as marks. Authors should not specify more than one <i>character</i> in \<string\>. The UA may truncate or ignore strings consisting of more than one grapheme cluster.

<a id="ref-for-valdef-text-emphasis-style-filled"></a>

<a id="ref-for-valdef-text-text-emphasis-open"></a>

<a id="ref-for-valdef-text-emphasis-style-circle"></a>

<a id="ref-for-typographic-mode⑤"></a>

<a id="ref-for-valdef-text-emphasis-style-sesame"></a>

If a shape keyword is specified but neither of [filled](#valdef-text-emphasis-style-filled) nor [open](#valdef-text-text-emphasis-open) is specified, <a id="ref-for-valdef-text-emphasis-style-filled①"></a>filled is assumed. If only <a id="ref-for-valdef-text-emphasis-style-filled②"></a>filled or <a id="ref-for-valdef-text-text-emphasis-open①"></a>open is specified, the shape keyword computes to [circle](#valdef-text-emphasis-style-circle) in horizontal [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode) and [sesame](#valdef-text-emphasis-style-sesame) in vertical <a id="ref-for-typographic-mode⑥"></a>typographic modes.

<a id="ref-for-valdef-font-variant-east-asian-ruby"></a>

<a id="ref-for-typographic-mode⑦"></a>

<a id="ref-for-writing-mode"></a>

The marks should be drawn using the element’s font settings with the addition of the [ruby](https://www.w3.org/TR/css-fonts-4/#valdef-font-variant-east-asian-ruby) feature and the size scaled down 50%. However, since not all fonts have all these glyphs, and some fonts use inappropriate sizes for emphasis marks in these code points, the UA may opt to use a font known to be good for emphasis marks, or the marks may instead be synthesized by the UA. Marks must remain upright in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode): like CJK characters, they do not rotate to match the writing mode. The orientation of marks in horizontal <a id="ref-for-typographic-mode⑧"></a>typographic modes of vertical [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) is undefined in this level (but may be defined in a future level if definitive use cases arise).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: One example of good fonts for emphasis marks is Adobe’s open source [Kenten Generic OpenType Font](https://github.com/adobe-fonts/kenten-generic), which is specially designed for the emphasis marks.

<a id="ref-for-typographic-character-unit②"></a>

The marks are drawn once for each [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit). However, emphasis marks are <em>not</em> drawn for:

- <a id="ref-for-character"></a>

  [Word separators](https://www.w3.org/TR/css-text-3/#word-separator) or other [characters](#character) that belong to the Unicode separator classes (Z\*). (But note that emphasis marks <em>are</em> drawn for a space that combines with any combining characters.)

- <a id="ref-for-unicode-general-category"></a>

  <a id="ref-for-character①"></a>

  Punctuation--specifically, any [characters](#character) that belong to the Unicode P\* [general category](https://www.w3.org/TR/css-text-3/#unicode-general-category) and do not `NFKD` normalize [\[UAX15\]](#biblio-uax15) to any of the following symbols:

  |                 |        |                                    |
  |-----------------|--------|------------------------------------|
  | \#              | U+0023 | NUMBER SIGN                        |
  | %               | U+0025 | PERCENT SIGN                       |
  | ‰               | U+2030 | PER MILLE SIGN                     |
  | ‱               | U+2031 | PER TEN THOUSAND SIGN              |
  | ٪               | U+066A | ARABIC PERCENT SIGN                |
  | ؉               | U+0609 | ARABIC-INDIC PER MILLE SIGN        |
  | ؊               | U+060A | ARABIC-INDIC PER TEN THOUSAND SIGN |
  | &#x26; | U+0026 | AMPERSAND                          |
  | ⁊               | U+204A | TIRONIAN SIGN ET                   |
  | @               | U+0040 | COMMERCIAL AT                      |
  | §               | U+00A7 | SECTION SIGN                       |
  | ¶               | U+00B6 | PILCROW SIGN                       |
  | ⁋               | U+204B | REVERSED PILCROW SIGN              |
  | ⁓               | U+2053 | SWUNG DASH                         |
  | 〽              | U+303D | PART ALTERNATION MARK              |

- Characters belonging to the Unicode classes for control codes and unassigned characters (Cc, Cf, Cn).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Control over which characters are marked will be added in Level 4. (The list of punctuation may also be further refined, particularly for non-CJK punctuation.)

<a id="ref-for-propdef-text-emphasis-color①"></a>

### <a id="text-emphasis-color-property"></a>3.2.  Emphasis Mark Color: the [text-emphasis-color](#propdef-text-emphasis-color) property

| Field               | Definition                                                                       |
|---------------------|----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-emphasis-color"></a>text-emphasis-color                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color①"></a>[\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | currentcolor                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | computed color                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                           |

This property specifies the foreground color of the emphasis marks.

<a id="ref-for-valdef-color-currentcolor"></a>

<a id="ref-for-propdef-color①"></a>

<a id="ref-for-propdef-text-emphasis-color②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) keyword computes to itself and is resolved to the value of [color](https://www.w3.org/TR/css-color-4/#propdef-color) after inheritance is performed. This means [text-emphasis-color](#propdef-text-emphasis-color) by default matches the text <a id="ref-for-propdef-color②"></a>color even as <a id="ref-for-propdef-color③"></a>color changes across elements.

<a id="ref-for-propdef-text-emphasis①"></a>

### <a id="text-emphasis-property"></a>3.3.  Emphasis Mark Shorthand: the [text-emphasis](#propdef-text-emphasis) property

| Field               | Definition                                                                                                                                                                                                                        |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-emphasis"></a>text-emphasis                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-text-emphasis-color③"></a><a id="ref-for-comb-any⑨"></a><a id="ref-for-propdef-text-emphasis-style②"></a>[\<'text-emphasis-style'\>](#propdef-text-emphasis-style) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'text-emphasis-color'\>](#propdef-text-emphasis-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                       |

<a id="ref-for-propdef-text-emphasis-style③"></a>

<a id="ref-for-propdef-text-emphasis-color④"></a>

This property is a shorthand for setting [text-emphasis-style](#propdef-text-emphasis-style) and [text-emphasis-color](#propdef-text-emphasis-color) in one declaration. Omitted values are set to their initial values.

<a id="ref-for-propdef-text-emphasis-position①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [text-emphasis-position](#propdef-text-emphasis-position) is not reset in this shorthand. This is because typically the shape and color vary, but the position is consistent for a particular language throughout the document. Therefore the position should inherit independently.

<a id="ref-for-propdef-text-emphasis-position②"></a>

### <a id="text-emphasis-position-property"></a>3.4.  Emphasis Mark Position: the [text-emphasis-position](#propdef-text-emphasis-position) property

| Field               | Definition                                                                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-emphasis-position"></a>text-emphasis-position                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-comb-all"></a><a id="ref-for-comb-one③⓪"></a>\[ over [\|](https://www.w3.org/TR/css-values-4/#comb-one) under \] [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) \[ right <a id="ref-for-comb-one③①"></a>\| left \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | over right                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                            |

<a id="ref-for-valdef-text-emphasis-position-right"></a>

This property describes where emphasis marks are drawn at. If \[ right \| left \] is omitted, it defaults to [right](#valdef-text-emphasis-position-right). The values have following meanings:

<a id="valdef-text-emphasis-position-over"></a>over  
<a id="ref-for-typographic-mode⑨"></a>

Draw marks over the text in horizontal [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

<a id="valdef-text-emphasis-position-under"></a>under  
<a id="ref-for-typographic-mode①⓪"></a>

Draw marks under the text in horizontal [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

<a id="valdef-text-emphasis-position-right"></a>right  
<a id="ref-for-typographic-mode①①"></a>

Draw marks to the right of the text in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

<a id="valdef-text-emphasis-position-left"></a>left  
<a id="ref-for-typographic-mode①②"></a>

Draw marks to the left of the text in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

<a id="ref-for-propdef-text-emphasis-position③"></a>

Emphasis marks are drawn exactly as if each character was assigned the mark as its ruby annotation text with the ruby position given by [text-emphasis-position](#propdef-text-emphasis-position) and the ruby alignment as centered. Note that this position may be adjusted if it would conflict with underline or overline decorations.

The effect of emphasis marks on the line height is the same as for ruby text.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, the preferred position of emphasis marks depends on the language. In Japanese for example, the preferred position is over right. In Chinese, on the other hand, the preferred position is under right. The informative table below summarizes the preferred emphasis mark positions for Chinese and Japanese:
>
> **Table 21**
>
> Preferred emphasis mark and ruby position
>
> Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.
>
> | Language | Preferred position / Horizontal | Preferred position / Vertical | Illustration | Illustration |
> | --- | --- | --- | --- | --- |
> | Japanese | over | right | ![Emphasis marks appear over each emphasized character in horizontal Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-ja.png "Emphasis (shown in blue for clarity) applied above a fragment of Japanese text") | ![Emphasis marks appear on the right of each emphasized character in vertical Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-v.gif "Emphasis applied on the right of a fragment of Japanese text") |
> | Korean | over | right | ![Emphasis marks appear over each emphasized character in horizontal Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-ja.png "Emphasis (shown in blue for clarity) applied above a fragment of Japanese text") | ![Emphasis marks appear on the right of each emphasized character in vertical Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-v.gif "Emphasis applied on the right of a fragment of Japanese text") |
> | Mongolian | over | right | ![Emphasis marks appear over each emphasized character in horizontal Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-ja.png "Emphasis (shown in blue for clarity) applied above a fragment of Japanese text") | ![Emphasis marks appear on the right of each emphasized character in vertical Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-v.gif "Emphasis applied on the right of a fragment of Japanese text") |
> | Chinese | under | right | ![Emphasis marks appear below each emphasized character in horizontal Simplified Chinese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-zh.png "Emphasis (shown in blue for clarity) applied below a fragment of Chinese text") | ![Emphasis marks appear on the right of each emphasized character in vertical Japanese text.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-v.gif "Emphasis applied on the right of a fragment of Japanese text") |

<a id="ref-for-ruby-annotation-box"></a>

If emphasis marks are applied to characters for which ruby is drawn in the same position as the emphasis mark, the emphasis marks are placed outside the ruby. This includes [auto-hidden](https://www.w3.org/TR/css-ruby-1/#autohide) and empty [ruby annotations](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-box).

![In this example, emphasis marks are applied to 4 characters, two of which have ruby. The dots are placed above each character (aligned with the ruby) for the bare characters, and above the ruby text for the annotated characters.](https://www.w3.org/TR/2022/WD-css-text-decor-4-20220504/images/text-emphasis-ruby.png)

Emphasis marks applied to 4 characters, with ruby also on 2 of them

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-47f78eba"></a> Some editors prefer to hide emphasis marks when they conflict with ruby. In HTML, this can be done with the following style rule:
>
> ```text
> ruby { text-emphasis: none; }
> ```
>
> Some other editors prefer to hide ruby when they conflict with emphasis marks. In HTML, this can be done with the following pattern:
>
> ```text
> em { text-emphasis: dot; } /* Set text-emphasis for <em> elements */
> em rt { display: none; }   /* Hide ruby inside <em> elements */
> ```
<a id="ref-for-propdef-text-emphasis-skip"></a>

### <a id="text-emphasis-skip"></a>3.5.  Emphasis Mark Skip: the [text-emphasis-skip](#propdef-text-emphasis-skip) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c3697583"></a>This section is under brainstorming. It’s also not yet clear if this property is needed quite yet, despite differences in desired behavior among publications.

| Field               | Definition                                                                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-emphasis-skip"></a>text-emphasis-skip                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any①⓪"></a>spaces [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) punctuation <a id="ref-for-comb-any①①"></a>\|\| symbols <a id="ref-for-comb-any①②"></a>\|\| narrow |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | spaces punctuation                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                  |

This property describes for which characters marks are drawn. The values have following meanings:

<a id="valdef-text-emphasis-skip-spaces"></a>spaces  
<a id="ref-for-character②"></a>

Skip [word separators](https://www.w3.org/TR/css-text/#word-separator) or other [characters](#character) belonging to the Unicode separator category (Z\*). (But note that emphasis marks <em>are</em> drawn for a space that combines with any combining characters.)

<a id="valdef-text-emphasis-skip-punctuation"></a>punctuation  
<a id="ref-for-valdef-text-emphasis-skip-symbols"></a>

Skip punctuation. Punctuation in this definition includes characters belonging to the Unicode P\* category that are not defined as [symbols](#valdef-text-emphasis-skip-symbols) (see below).

<a id="valdef-text-emphasis-skip-symbols"></a>symbols  
<a id="ref-for-character③"></a>

<a id="ref-for-unicode-general-category①"></a>

<a id="ref-for-typographic-character-unit③"></a>

Skip symbols. Symbols in this definition includes all [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) belonging to the Unicode S\* [general category](https://www.w3.org/TR/css-text-3/#unicode-general-category) as well as any which are `NFKD`-equivalent [\[UAX15\]](#biblio-uax15) to the following [characters](#character) from the Unicode Po category:

|                 |        |                                    |
|-----------------|--------|------------------------------------|
| \#              | U+0023 | NUMBER SIGN                        |
| %               | U+0025 | PERCENT SIGN                       |
| ‰               | U+2030 | PER MILLE SIGN                     |
| ‱               | U+2031 | PER TEN THOUSAND SIGN              |
| ٪               | U+066A | ARABIC PERCENT SIGN                |
| ؉               | U+0609 | ARABIC-INDIC PER MILLE SIGN        |
| ؊               | U+060A | ARABIC-INDIC PER TEN THOUSAND SIGN |
| &#x26; | U+0026 | AMPERSAND                          |
| ⁊               | U+204A | TIRONIAN SIGN E\[\[                |
| @               | U+0040 | COMMERCIAL AT                      |
| §               | U+00A7 | SECTION SIGN                       |
| ¶               | U+00B6 | PILCROW SIGN                       |
| ⁋               | U+204B | REVERSED PILCROW SIGN              |
| ⁓               | U+2053 | SWUNG DASH                         |
| 〽️              | U+303D | PART ALTERNATION MARK              |

<a id="valdef-text-emphasis-skip-narrow"></a>narrow  
Skip characters where the `East_Asian_Width` property [\[UAX11\]](#biblio-uax11) of the Unicode database [\[UAX44\]](#biblio-uax44) is not F (Fullwidth) or W (Wide).

Characters belonging to the Unicode classes for control codes and unassigned characters (Cc, Cf, Cn) are skipped regardless of the value of this property.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-37d9e55a"></a>This syntax requires UA to implement drawing marks for spaces. Is there any use case for doing so? If not, should we modify the syntax not to allow drawing marks for spaces?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e5b775b7"></a> See also [discussion of the initial value](https://github.com/w3c/csswg-drafts/issues/839).

<a id="ref-for-propdef-text-shadow①"></a>

## <a id="text-shadow-property"></a>4.  Text Shadows: the [text-shadow](#propdef-text-shadow) property

| Field               | Definition                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-shadow"></a>text-shadow                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-typedef-shadow"></a><a id="ref-for-comb-one③②"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<shadow\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-shadow)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | text                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-box-shadow-none"></a>either the keyword [none](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-none) or a list, each item consisting of four absolute lengths plus a computed color and optionally also an inset keyword                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | [as shadow list](https://www.w3.org/TR/web-animations-1/#animating-shadow-lists)                                                                                                                                                           |

<a id="ref-for-propdef-box-shadow"></a>

This property accepts a comma-separated list of shadow effects to be applied to the text of the element. Values are interpreted as for [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow) [\[CSS-BACKGROUNDS-3\]](#biblio-css-backgrounds-3). Each layer shadows the element’s text and all its text decorations (composited together). The shadow effects are applied front-to-back: the first shadow is on top. The shadows may thus overlay each other.

<a id="ref-for-propdef-box-shadow①"></a>

<a id="ref-for-spread-distance"></a>

Unlike [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow), the [spread distance](https://www.w3.org/TR/css-backgrounds-3/#spread-distance) is strictly interpreted as outset distance from any point of the glyph outline, and therefore, similar to the blur radius, creates rounded, rather than sharp, corners. Negative spread values are invalid.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-62da5bfc"></a> Leave corner shaping undefined? [\[Issue \#7250\]](https://github.com/w3c/csswg-drafts/issues/7250)

<a id="ref-for-shadow-inset"></a>

<a id="ref-for-propdef-box-shadow②"></a>

<a id="outer-text-shadows"></a>Outer text shadows (specified without the [inset](https://www.w3.org/TR/css-backgrounds-3/#shadow-inset) keyword) shadow the text—including any text stroke [\[FILL-STROKE-3\]](#biblio-fill-stroke-3)—as if it were cut and raised above the surrounding canvas. Unlike [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow), outer text shadows are not clipped to the shadowed shape and may show through if the text is partially-transparent.

<a id="ref-for-shadow-inset①"></a>

<a id="inner-text-shadows"></a>Inner text shadows (specified with the [inset](https://www.w3.org/TR/css-backgrounds-3/#shadow-inset) keyword) shadow the canvas—and any text stroke [\[FILL-STROKE-3\]](#biblio-fill-stroke-3)—as if the text were cut and dropped below the surrounding canvas. They are therefore only drawn within the inner edge of the stroke.

<a id="ref-for-outer-text-shadows"></a>

<a id="ref-for-inner-text-shadows"></a>

[Outer text shadows](#outer-text-shadows) must be painted at a stack level between the element’s border/background (if present) and the elements text and text decoration. [Inner text shadows](#inner-text-shadows) must be painted over the text and its decorations. UAs should avoid painting text shadows over text in adjacent elements belonging to the same stack level and stacking context. (This may mean that the exact stack level of the shadows depends on whether the element has a border or background: the exact stacking behavior of text shadows is thus UA-defined.)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-796f8a28"></a> Stacking relationship to stroke? [\[Issue \#7251\]](https://github.com/w3c/csswg-drafts/issues/7251)

<a id="ref-for-propdef-box-shadow③"></a>

<a id="ref-for-scrollable-overflow-region"></a>

Like [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow), text shadows do not influence layout, and do not trigger scrolling or increase the size of the [scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region).

The text-shadow property applies to both the `::first-line` and `::first-letter` pseudo-elements.

## <a id="painting"></a>5.  Painting Text Decorations

### <a id="painting-order"></a>5.1.  Painting Order of Text Decorations

As in [\[CSS2\]](#biblio-css2), text decorations are drawn immediately over/under the text they decorate, in the following order (bottommost first):

- <a id="ref-for-propdef-text-shadow②"></a>

  shadows ([text-shadow](#propdef-text-shadow))

- <a id="ref-for-propdef-text-decoration①①"></a>

  underlines ([text-decoration](#propdef-text-decoration))

- <a id="ref-for-propdef-text-decoration①②"></a>

  overlines ([text-decoration](#propdef-text-decoration))

- text

- <a id="ref-for-propdef-text-emphasis②"></a>

  emphasis marks ([text-emphasis](#propdef-text-emphasis))

- <a id="ref-for-propdef-text-decoration①③"></a>

  line-through ([text-decoration](#propdef-text-decoration))

Where line decorations are drawn across box decorations or atomic inlines, they are drawn over non-positioned content and just below any positioned descendants (immediately below layer \#8 in CSS2.1 Appendix E).

### <a id="overflow"></a>5.2.  Overflow of Text Decorations

<a id="ref-for-box①"></a>

<a id="ref-for-ink-overflow"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

Text decorations that leak outside a [box](https://www.w3.org/TR/css-display-3/#box) are considered [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow): they do not extend the [scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region). [\[css-overflow-3\]](#biblio-css-overflow-3)

## <a id="acknowledgements"></a> Appendix A: Acknowledgements

This specification would not have been possible without the help from: Ayman Aldahleh, Bert Bos, Tantek Çelik, Stephen Deach, John Daggett, Martin Dürst, Laurie Anna Edlund, Ben Errez, Yaniv Feinberg, Arye Gittelman, Ian Hickson, Martin Heijdra, Richard Ishida, Masayasu Ishikawa, Michael Jochimsen, Eric LeVine, Ambrose Li, Håkon Wium Lie, Chris Lilley, Ken Lunde, Nat McCully, Shinyu Murakami, Paul Nelson, Chris Pratley, Marcin Sawicki, Arnold Schrijver, Rahul Sonnad, Michel Suignard, Takao Suzuki, Frank Tang, Chris Thrasher, Etan Wexler, Chris Wilson, Masafumi Yabe and Steve Zilles.

## <a id="default-stylesheet"></a> Appendix B: Default UA Stylesheet

This appendix is informative, and is to help UA developers to implement default stylesheet, but UA developers are free to ignore or change.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a9219bae"></a>
>
> ```text
> /* typical styling of HTML */
> blink {
>   text-decoration-line: blink;
> }
> s, strike, del {
>   text-decoration: line-through;
> }
> u, ins, :link, :visited {
>   text-decoration: underline;
> }
> abbr[title], acronym[title] {
>   text-decoration: dotted underline;
> }
> 
> /* disable inheritance of text-emphasis marks to ruby text:
>   emphasis marks should only apply to base text */
> rt { text-emphasis: none; }
> 
> /* set language-appropriate default emphasis mark position */
> :root:lang(zh), [lang|=zh] { text-emphasis-position: under right; }
> [lang|=ja], [lang|=ko]     { text-emphasis-position: over right; }
> 
> /* set language-appropriate default underline position */
> :root:lang(ja), [lang|=ja],
> :root:lang(mn), [lang|=mn],
> :root:lang(ko), [lang|=ko] { text-underline-position: right; }
> :root:lang(zh), [lang|=zh] { text-underline-position: left;  }
> /* auto is chosen (implied) above instead of under
>    due to content-compatibility concerns */
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4a420eee"></a> If you find any issues, recommendations to add, or corrections, please send the information to <www-style@w3.org> with `[css-text-decor]` in the subject line.

<a id="ref-for-propdef-text-decoration-line⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef2cba97"></a> While [text-decoration-line: blink](#propdef-text-decoration-line) can’t be fully reproduced with other existing properties, authors can achieve a very similar effect with the following CSS:
>
> ```text
> @keyframes blink {
>   0% {
>     visibility: hidden;
>     animation-timing-function: step-end;
>   }
>   25%, 100% {
>     visibility: visible;
>   }
> }
> blink {
>   animation: blink 1s infinite;
> }
> ```
## <a id="changes"></a>Appendix C: Changes

### <a id="changes-2020"></a> Changes since the 6 May 2020 Working Draft

Significant changes since the [6 May 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-text-decor-4-20200506/):

- <a id="ref-for-propdef-text-shadow③"></a>

  <a id="ref-for-shadow-inset②"></a>

  <a id="ref-for-spread-distance①"></a>

  Added [spread distance](https://www.w3.org/TR/css-backgrounds-3/#spread-distance) and [inset](https://www.w3.org/TR/css-backgrounds-3/#shadow-inset) to [text-shadow](#propdef-text-shadow). ([Issue 6074](https://github.com/w3c/csswg-drafts/issues/6074), [Issue 6971](https://github.com/w3c/csswg-drafts/issues/6971))

- <a id="ref-for-propdef-text-decoration-skip-ink③"></a>

  Clarified that [text-decoration-skip-ink](#propdef-text-decoration-skip-ink) affects only decorations initiated by the element. ([Issue 2817](https://github.com/w3c/csswg-drafts/issues/2817))

- Explicitly noted properties applying to text in “Applies to” line. ([Issue 5303](https://github.com/w3c/csswg-drafts/issues/5303))

### <a id="additions-l3"></a> Additions Since Level 3

The following features have been added since [Level 3](https://www.w3.org/TR/css-text-decor-3/):

- <a id="ref-for-propdef-text-decoration-line⑨"></a>

  <a id="ref-for-valdef-text-decoration-line-grammar-error③"></a>

  <a id="ref-for-valdef-text-decoration-line-spelling-error③"></a>

  Added [spelling-error](#valdef-text-decoration-line-spelling-error) and [grammar-error](#valdef-text-decoration-line-grammar-error) values to [text-decoration-line](#propdef-text-decoration-line).

- <a id="ref-for-propdef-text-underline-offset⑦"></a>

  <a id="ref-for-propdef-text-decoration-thickness⑤"></a>

  Added [text-decoration-thickness](#propdef-text-decoration-thickness) and [text-underline-offset](#propdef-text-underline-offset) properties.

- <a id="ref-for-propdef-text-underline-position①⑤"></a>

  <a id="ref-for-valdef-text-underline-position-from-font③"></a>

  Added [from-font](#valdef-text-underline-position-from-font) value to [text-underline-position](#propdef-text-underline-position).

- <a id="ref-for-propdef-text-decoration-skip⑧"></a>

  Drafted [text-decoration-skip](#propdef-text-decoration-skip) property and its longhands.

- <a id="ref-for-propdef-text-emphasis-skip①"></a>

  Drafted [text-emphasis-skip](#propdef-text-emphasis-skip) property.

- <a id="ref-for-propdef-text-shadow④"></a>

  <a id="ref-for-shadow-inset③"></a>

  <a id="ref-for-spread-distance②"></a>

  Added [spread distance](https://www.w3.org/TR/css-backgrounds-3/#spread-distance) and [inset](https://www.w3.org/TR/css-backgrounds-3/#shadow-inset) to [text-shadow](#propdef-text-shadow).

## <a id="priv-sec"></a>6. Privacy and Security Considerations

This specification introduces no new privacy or security considerations.

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

- all
  - [value for text-decoration-skip-box](#valdef-text-decoration-skip-box-all), in § 2.10.2
  - [value for text-decoration-skip-ink](#valdef-text-decoration-skip-ink-all), in § 2.10.5
  - [value for text-decoration-skip-spaces](#valdef-text-decoration-skip-spaces-all), in § 2.10.4
- auto
  - [value for text-decoration-skip](#valdef-text-decoration-skip-auto), in § 2.10
  - [value for text-decoration-skip-ink](#valdef-text-decoration-skip-ink-auto), in § 2.10.5
  - [value for text-decoration-skip-inset](#valdef-text-decoration-skip-inset-auto), in § 2.10.3
  - [value for text-decoration-thickness](#valdef-text-decoration-thickness-auto), in § 2.4
  - [value for text-underline-offset](#valdef-text-underline-offset-auto), in § 2.8
  - [value for text-underline-position](#underline-auto), in § 2.7
- [blink](#valdef-text-decoration-line-blink), in § 2.1
- [character](#character), in § 1.3
- [circle](#valdef-text-emphasis-style-circle), in § 3.1
- [considered text](#considered-text), in § 2.5
- [decorating box](#decorating-box), in § 2
- [dot](#valdef-text-emphasis-style-dot), in § 3.1
- [double-circle](#valdef-text-emphasis-style-double-circle), in § 3.1
- [end](#valdef-text-decoration-skip-spaces-end), in § 2.10.4
- [filled](#valdef-text-emphasis-style-filled), in § 3.1
- from-font
  - [value for text-decoration-thickness](#valdef-text-decoration-thickness-from-font), in § 2.4
  - [value for text-underline-position](#valdef-text-underline-position-from-font), in § 2.7
- [grammar-error](#valdef-text-decoration-line-grammar-error), in § 2.1
- [Inner text shadows](#inner-text-shadows), in § 4
- left
  - [value for text-emphasis-position](#valdef-text-emphasis-position-left), in § 3.4
  - [value for text-underline-position](#underline-left), in § 2.7
- \<length\>
  - [value for text-decoration-thickness](#valdef-text-decoration-thickness-length), in § 2.4
  - [value for text-underline-offset](#valdef-text-underline-offset-length), in § 2.8
- [line-through](#valdef-text-decoration-line-line-through), in § 2.1
- [narrow](#valdef-text-emphasis-skip-narrow), in § 3.5
- none
  - [value for text-decoration-line](#valdef-text-decoration-line-none), in § 2.1
  - [value for text-decoration-skip](#valdef-text-decoration-skip-none), in § 2.10
  - [value for text-decoration-skip-box](#valdef-text-decoration-skip-box-none), in § 2.10.2
  - [value for text-decoration-skip-ink](#valdef-text-decoration-skip-ink-none), in § 2.10.5
  - [value for text-decoration-skip-inset](#valdef-text-decoration-skip-inset-none), in § 2.10.3
  - [value for text-decoration-skip-self](#valdef-text-decoration-skip-self-none), in § 2.10.1
  - [value for text-decoration-skip-spaces](#valdef-text-decoration-skip-spaces-none), in § 2.10.4
  - [value for text-emphasis-style](#valdef-text-emphasis-style-none), in § 3.1
- [objects](#valdef-text-decoration-skip-self-objects), in § 2.10.1
- [open](#valdef-text-text-emphasis-open), in § 3.1
- [Outer text shadows](#outer-text-shadows), in § 4
- [over](#valdef-text-emphasis-position-over), in § 3.4
- [overline](#valdef-text-decoration-line-overline), in § 2.1
- \<percentage\>
  - [value for text-decoration-thickness](#valdef-text-decoration-thickness-percentage), in § 2.4
  - [value for text-underline-offset](#valdef-text-underline-offset-percentage), in § 2.8
- [punctuation](#valdef-text-emphasis-skip-punctuation), in § 3.5
- right
  - [value for text-emphasis-position](#valdef-text-emphasis-position-right), in § 3.4
  - [value for text-underline-position](#underline-right), in § 2.7
- [sesame](#valdef-text-emphasis-style-sesame), in § 3.1
- [spacer](#spacer), in § 2.10.4
- [spaces](#valdef-text-emphasis-skip-spaces), in § 3.5
- [spelling-error](#valdef-text-decoration-line-spelling-error), in § 2.1
- [start](#valdef-text-decoration-skip-spaces-start), in § 2.10.4
- [\<string\>](#valdef-text-emphasis-style-string), in § 3.1
- [symbols](#valdef-text-emphasis-skip-symbols), in § 3.5
- [text-decoration](#propdef-text-decoration), in § 2.6
- [text-decoration-color](#propdef-text-decoration-color), in § 2.3
- [text-decoration-line](#propdef-text-decoration-line), in § 2.1
- [text-decoration-skip](#propdef-text-decoration-skip), in § 2.10
- [text-decoration-skip-box](#propdef-text-decoration-skip-box), in § 2.10.2
- [text-decoration-skip-ink](#propdef-text-decoration-skip-ink), in § 2.10.5
- [text-decoration-skip-inset](#propdef-text-decoration-skip-inset), in § 2.10.3
- [text-decoration-skip-self](#propdef-text-decoration-skip-self), in § 2.10.1
- [text-decoration-skip-spaces](#propdef-text-decoration-skip-spaces), in § 2.10.4
- [text-decoration-style](#propdef-text-decoration-style), in § 2.2
- [text-decoration-thickness](#propdef-text-decoration-thickness), in § 2.4
- [text-emphasis](#propdef-text-emphasis), in § 3.3
- [text-emphasis-color](#propdef-text-emphasis-color), in § 3.2
- [text-emphasis-position](#propdef-text-emphasis-position), in § 3.4
- [text-emphasis-skip](#propdef-text-emphasis-skip), in § 3.5
- [text-emphasis-style](#propdef-text-emphasis-style), in § 3.1
- [text-shadow](#propdef-text-shadow), in § 4
- [text-underline-offset](#propdef-text-underline-offset), in § 2.8
- [text-underline-position](#propdef-text-underline-position), in § 2.7
- [triangle](#valdef-text-emphasis-style-triangle), in § 3.1
- under
  - [value for text-emphasis-position](#valdef-text-emphasis-position-under), in § 3.4
  - [value for text-underline-position](#underline-under), in § 2.7
- [underline](#valdef-text-decoration-line-underline), in § 2.1
- [underline zero position](#underline-zero-position), in § 2.8.1
- [wavy](#valdef-text-decoration-style-wavy), in § 2.2
- [zero position](#underline-zero-position), in § 2.8.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="term-for-typedef-shadow"></a>\<shadow\>
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
  - <a id="term-for-shadow-inset"></a>inset
  - <a id="term-for-box-shadow-none"></a>none
  - <a id="term-for-spread-distance"></a>spread distance
- \[css-break-4\] defines the following terms:
  - <a id="term-for-box-fragment"></a>box fragment
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-longhand"></a>longhand
  - <a id="term-for-shorthand-property"></a>shorthand
  - <a id="term-for-longhand①"></a>sub-property
- \[css-cascade-6\] defines the following terms:
  - <a id="term-for-cascade"></a>cascade
- \[css-color-4\] defines the following terms:
  - <a id="term-for-typedef-color"></a>\<color\>
  - <a id="term-for-propdef-color"></a>color
  - <a id="term-for-valdef-color-currentcolor"></a>currentcolor
- \[css-display-3\] defines the following terms:
  - <a id="term-for-anonymous"></a>anonymous
  - <a id="term-for-atomic-inline"></a>atomic inline
  - <a id="term-for-block-container"></a>block container
  - <a id="term-for-block-level"></a>block-level
  - <a id="term-for-box"></a>box
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-in-flow"></a>in-flow
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-inline-formatting-context"></a>inline formatting context
  - <a id="term-for-inline-level"></a>inline-level
  - <a id="term-for-non-replaced"></a>non-replaced
  - <a id="term-for-propdef-visibility"></a>visibility
- \[css-fonts-4\] defines the following terms:
  - <a id="term-for-first-available-font"></a>first available font
  - <a id="term-for-propdef-font-size"></a>font-size
  - <a id="term-for-propdef-font-variant-position"></a>font-variant-position
  - <a id="term-for-valdef-font-variant-east-asian-ruby"></a>ruby
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-valdef-alignment-baseline-baseline"></a>baseline
  - <a id="term-for-propdef-vertical-align"></a>vertical-align
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-ink-overflow"></a>ink overflow
  - <a id="term-for-scrollable-overflow-region"></a>scrollable overflow region
- \[css-position-3\] defines the following terms:
  - <a id="term-for-relative-position"></a>relatively position
- \[css-pseudo-4\] defines the following terms:
  - <a id="term-for-selectordef-first-letter"></a>::first-letter
  - <a id="term-for-selectordef-first-line"></a>::first-line
- \[css-ruby-1\] defines the following terms:
  - <a id="term-for-ruby-annotation-box"></a>ruby annotation
  - <a id="term-for-ruby-base-box"></a>ruby base
  - <a id="term-for-ruby-container"></a>ruby container
- \[css-syntax-3\] defines the following terms:
  - <a id="term-for-letter"></a>letter
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="term-for-content-language"></a>content language
  - <a id="term-for-unicode-general-category"></a>general category
  - <a id="term-for-propdef-letter-spacing"></a>letter-spacing
  - <a id="term-for-typographic-character-unit"></a>typographic character unit
  - <a id="term-for-typographic-letter-unit"></a>typographic letter unit
  - <a id="term-for-propdef-word-spacing"></a>word-spacing
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="term-for-over"></a>over
  - <a id="term-for-typographic-mode"></a>typographic mode
  - <a id="term-for-under"></a>under
  - <a id="term-for-vertical-writing-mode"></a>vertical writing mode
  - <a id="term-for-writing-mode"></a>writing mode
- \[FILL-STROKE-3\] defines the following terms:
  - <a id="term-for-propdef-fill"></a>fill
  - <a id="term-for-propdef-stroke"></a>stroke
- \[selectors-4\] defines the following terms:
  - <a id="term-for-pseudo-element"></a>pseudo-element

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 15 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 23 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 2 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-fill-stroke-3"></a>\[FILL-STROKE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Fill and Stroke Module Level 3](https://www.w3.org/TR/fill-stroke-3/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;fill-stroke-3&#x2F;](https://www.w3.org/TR/fill-stroke-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 21 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-uax11"></a>\[UAX11\]  
Ken Lunde 小林劍󠄁. [East Asian Width](https://www.unicode.org/reports/tr11/tr11-39.html). 23 August 2021. Unicode Standard Annex \#11. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr11&#x2F;tr11-39&#x2E;html](https://www.unicode.org/reports/tr11/tr11-39.html)

<a id="biblio-uax15"></a>\[UAX15\]  
Ken Whistler. [Unicode Normalization Forms](https://www.unicode.org/reports/tr15/tr15-51.html). 27 August 2021. Unicode Standard Annex \#15. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr15&#x2F;tr15-51&#x2E;html](https://www.unicode.org/reports/tr15/tr15-51.html)

<a id="biblio-uax24"></a>\[UAX24\]  
Ken Whistler. [Unicode Script Property](https://www.unicode.org/reports/tr24/tr24-32.html). 27 August 2021. Unicode Standard Annex \#24. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr24&#x2F;tr24-32&#x2E;html](https://www.unicode.org/reports/tr24/tr24-32.html)

<a id="biblio-uax44"></a>\[UAX44\]  
Ken Whistler; Laurențiu Iancu. [Unicode Character Database](https://www.unicode.org/reports/tr44/tr44-28.html). 30 August 2021. Unicode Standard Annex \#44. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr44&#x2F;tr44-28&#x2E;html](https://www.unicode.org/reports/tr44/tr44-28.html)

### <a id="informative"></a>Informative References

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 13 August 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css3-animations"></a>\[CSS3-ANIMATIONS\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                             | Initial                   | Applies to                | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-text-decoration①④"></a></span><a href="#propdef-text-decoration">text-decoration</a>&#xA;      </strong> | \<'text-decoration-line'\> \|\| \<'text-decoration-thickness'\> \|\| \<'text-decoration-style'\> \|\| \<'text-decoration-color'\> | see individual properties | see individual properties | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                                                                                                                   |
| <strong><span><a id="ref-for-propdef-text-decoration-color③"></a></span><a href="#propdef-text-decoration-color">text-decoration-color</a>&#xA;      </strong> | \<color\>                                                                                                                         | currentcolor              | all elements              | no                        | n/a                       | by computed value type    | per grammar     | computed color                                                                                                                              |
| <strong><span><a id="ref-for-propdef-text-decoration-line①⓪"></a></span><a href="#propdef-text-decoration-line">text-decoration-line</a>&#xA;      </strong> | none \| \[ underline \|\| overline \|\| line-through \|\| blink \] \| spelling-error \| grammar-error                             | none                      | all elements              | no (but see prose, above) | n/a                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-decoration-skip⑨"></a></span><a href="#propdef-text-decoration-skip">text-decoration-skip</a>&#xA;      </strong> | none \| auto                                                                                                                      | See individual properties | all elements              | yes                       | N/A                       | discrete                  | per grammar     | See individual properties                                                                                                                   |
| <strong><span><a id="ref-for-propdef-text-decoration-skip-box②"></a></span><a href="#propdef-text-decoration-skip-box">text-decoration-skip-box</a>&#xA;      </strong> | none \| all                                                                                                                       | none                      | all elements              | yes                       | N/A                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-decoration-skip-ink④"></a></span><a href="#propdef-text-decoration-skip-ink">text-decoration-skip-ink</a>&#xA;      </strong> | auto \| none \| all                                                                                                               | auto                      | all elements              | yes                       | N/A                       | discrete                  | per grammar     | specified keyword                                                                                                                           |
| <strong><span><a id="ref-for-propdef-text-decoration-skip-inset③"></a></span><a href="#propdef-text-decoration-skip-inset">text-decoration-skip-inset</a>&#xA;      </strong> | none \| auto                                                                                                                      | none                      | all elements              | yes                       | N/A                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-decoration-skip-self②"></a></span><a href="#propdef-text-decoration-skip-self">text-decoration-skip-self</a>&#xA;      </strong> | none \| objects                                                                                                                   | objects                   | all elements              | yes                       | N/A                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-decoration-skip-spaces②"></a></span><a href="#propdef-text-decoration-skip-spaces">text-decoration-skip-spaces</a>&#xA;      </strong> | none \| all \| \[ start \|\| end \]                                                                                               | start end                 | all elements              | yes                       | N/A                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-decoration-style③"></a></span><a href="#propdef-text-decoration-style">text-decoration-style</a>&#xA;      </strong> | solid \| double \| dotted \| dashed \| wavy                                                                                       | solid                     | all elements              | no                        | n/a                       | discrete                  | per grammar     | specified keyword                                                                                                                           |
| <strong><span><a id="ref-for-propdef-text-decoration-thickness⑥"></a></span><a href="#propdef-text-decoration-thickness">text-decoration-thickness</a>&#xA;      </strong> | auto \| from-font \| \<length\> \| \<percentage\>                                                                                 | auto                      | all elements              | no                        | N/A                       | by computed value         | per grammar     | specified keyword or absolute length                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-emphasis③"></a></span><a href="#propdef-text-emphasis">text-emphasis</a>&#xA;      </strong> | \<'text-emphasis-style'\> \|\| \<'text-emphasis-color'\>                                                                          | see individual properties | see individual properties | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                                                                                                                   |
| <strong><span><a id="ref-for-propdef-text-emphasis-color⑤"></a></span><a href="#propdef-text-emphasis-color">text-emphasis-color</a>&#xA;      </strong> | \<color\>                                                                                                                         | currentcolor              | text                      | yes                       | n/a                       | by computed value type    | per grammar     | computed color                                                                                                                              |
| <strong><span><a id="ref-for-propdef-text-emphasis-position④"></a></span><a href="#propdef-text-emphasis-position">text-emphasis-position</a>&#xA;      </strong> | \[ over \| under \] &#x26;&#x26; \[ right \| left \]?                                                           | over right                | text                      | yes                       | n/a                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-emphasis-skip②"></a></span><a href="#propdef-text-emphasis-skip">text-emphasis-skip</a>&#xA;      </strong> | spaces \|\| punctuation \|\| symbols \|\| narrow                                                                                  | spaces punctuation        | text                      | yes                       | N/A                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-emphasis-style④"></a></span><a href="#propdef-text-emphasis-style">text-emphasis-style</a>&#xA;      </strong> | none \| \[ \[ filled \| open \] \|\| \[ dot \| circle \| double-circle \| triangle \| sesame \] \] \| \<string\>                  | none                      | text                      | yes                       | n/a                       | discrete                  | per grammar     | the keyword none, a pair of keywords representing the shape and fill, or a string                                                           |
| <strong><span><a id="ref-for-propdef-text-shadow⑤"></a></span><a href="#propdef-text-shadow">text-shadow</a>&#xA;      </strong> | none \| \<shadow\>#                                                                                                               | none                      | text                      | yes                       | n/a                       | as shadow list            | per grammar     | either the keyword none or a list, each item consisting of four absolute lengths plus a computed color and optionally also an inset keyword |
| <strong><span><a id="ref-for-propdef-text-underline-offset⑧"></a></span><a href="#propdef-text-underline-offset">text-underline-offset</a>&#xA;      </strong> | auto \| \<length\> \| \<percentage\>                                                                                              | auto                      | all elements              | yes                       | N/A                       | by computed value         | per grammar     | specified keyword or absolute length                                                                                                        |
| <strong><span><a id="ref-for-propdef-text-underline-position①⑥"></a></span><a href="#propdef-text-underline-position">text-underline-position</a>&#xA;      </strong> | auto \| \[ from-font \| under \] \|\| \[ left \| right \]                                                                         | auto                      | all elements              | yes                       | n/a                       | discrete                  | per grammar     | specified keyword(s)                                                                                                                        |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is copied over from early drafts of Text Decoration Level 3. It is still under review, and needs integration with [text-underline-offset](#propdef-text-underline-offset) and [text-decoration-thickness](#propdef-text-decoration-thickness). [↵](#issue-d1c311e6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> For simplicity, line-throughs should draw over each element at that element’s preferred/averaged position. This can produce some undesirable jumpiness, but there doesn’t appear to be any way to avoid that which is correct in all instances, and all attempts are worryingly complex. What position should line-throughs adopt over elements that have a different font-size, but no [considered text](#considered-text)? [↵](#issue-3dd812b9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG resolved to be split skipping functionality into individual properties along the lines of [text-decoration-skip-ink](#propdef-text-decoration-skip-ink), to improve its cascading behavior. See [discussion](https://github.com/w3c/csswg-drafts/issues/843) and [resolution](https://lists.w3.org/Archives/Public/www-style/2017Feb/0049.html). This section is a rough draft and has not yet been vetted by the CSSWG [↵](#issue-e6a42afa)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is this [none](#valdef-text-decoration-skip-none) definition Web-compatible? Do we also need to add an ink value for Web-compat? [↵](#issue-070668ae)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG. [↵](#issue-a29a7026)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG. [↵](#issue-a29a7026%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG resolved to split [text-decoration-skip](#propdef-text-decoration-skip) into sub-properties, but this value set has not yet been vetted by the CSSWG. [↵](#issue-a29a7026%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This might want to be a standalone property rather than part of the [text-decoration-skip](#propdef-text-decoration-skip) set. See also [Issue 4557](https://github.com/w3c/csswg-drafts/issues/4557), about controlling the line length explicitly. [↵](#issue-505740d9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should the initial value be [none](#valdef-text-decoration-skip-spaces-none) for Web-compat? If not, INS and DEL at least should be assigned none in the UA default stylesheet. See also [Issue 4653](https://github.com/w3c/csswg-drafts/issues/4653). [↵](#issue-4229bfce)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Ideographic scripts do not want to skip when [auto](#valdef-text-decoration-skip-ink-auto). How can we define this behavior? Are there more scripts wanting not to skip? Need some normative text describe how auto works. See [telcon minutes](https://lists.w3.org/Archives/Public/www-style/2017Feb/0069.html), [alreq#86](https://github.com/w3c/alreq/issues/86), [csswg#1288](https://github.com/w3c/csswg-drafts/issues/1288) [↵](#issue-01cb5447)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Are there other (non-CJK) scripts where it would be preferable to disable ink-skipping by default (when [auto](#valdef-text-decoration-skip-ink-auto) is in effect)? Perhaps Yi? Arabic? (See also discussion in [Issue 1288](https://github.com/w3c/csswg-drafts/issues/1288).) [↵](#issue-da3f50fc)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> See also [issue about continuity in size/position](https://github.com/w3c/csswg-drafts/issues/1892). [↵](#issue-5d2e97f6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is under brainstorming. It’s also not yet clear if this property is needed quite yet, despite differences in desired behavior among publications. [↵](#issue-c3697583)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This syntax requires UA to implement drawing marks for spaces. Is there any use case for doing so? If not, should we modify the syntax not to allow drawing marks for spaces? [↵](#issue-37d9e55a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> See also [discussion of the initial value](https://github.com/w3c/csswg-drafts/issues/839). [↵](#issue-e5b775b7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Leave corner shaping undefined? [\[Issue \#7250\]](https://github.com/w3c/csswg-drafts/issues/7250) [↵](#issue-62da5bfc)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Stacking relationship to stroke? [\[Issue \#7251\]](https://github.com/w3c/csswg-drafts/issues/7251) [↵](#issue-796f8a28)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If you find any issues, recommendations to add, or corrections, please send the information to <www-style@w3.org> with `[css-text-decor]` in the subject line. [↵](#issue-4a420eee)
