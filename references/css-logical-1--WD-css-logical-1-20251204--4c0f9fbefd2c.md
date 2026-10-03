Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Logical Properties and Values Module Level 1](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Logical Properties and Values Module Level 1

Source snapshot: https://www.w3.org/TR/2025/WD-css-logical-1-20251204/

Snapshot SHA-256: 4c0f9fbefd2c66a86f220d5daa1514facca3ad7dbae68a4c1a82b569de370edc

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 23 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Logical Properties and Values Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module introduces logical properties and values that provide the author with the ability to control layout through logical, rather than physical, direction and dimension mappings. The module defines logical properties and values for the features defined in [\[CSS2\]](#biblio-css2). These properties are writing-mode relative equivalents of their corresponding physical properties.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-logical” in the title, like this: “\[css-logical\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-logical%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: See [\[css-writing-modes-4\]](#biblio-css-writing-modes-4) for a proper introduction to writing modes; this module assumes familiarity with its terminology.

<a id="ref-for-writing-mode"></a>

Because different writing systems are written in different directions, a variety of [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) exist: left to right, top to bottom; right to left, top to bottom; bottom to top, right to left; etc. logical concepts like the “start” of a page or line map differently to physical concepts like the “top” of a line or “left edge” of a paragraph. Some aspects of a layout are actually relative to the writing directions, and thus will vary when the page is translated to a different system; others are inherently relative to the page’s physical orientation.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6f979823"></a> For example, lists, headings, and paragraphs are typically left-aligned in English; but actually they are start-aligned, because in Arabic the same constructs are right-aligned, and a multilingual document will need to accommodate both writing systems accordingly.
>
> The following code exemplifies how using logical syntax can help you write code that works across different writing systems:
>
> ![](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/images/example01.png)
>
> Rendering of the below code in a compatible browser
>
> ```text
> <blockquote dir="auto">Quotation in English</blockquote>
> <blockquote dir="auto">اقتباس في العربية</blockquote>
> ```
>
> ```text
> blockquote {
>     text-align: start; /* left in latin, right in arabic */
>     margin-inline-start: 0px; /* margin-left in latin, margin-right in arabic */
>     border-inline-start: 5px solid gray; /* border-left in latin, border-right in arabic */
>     padding-inline-start: 5px; /* padding-left in latin, padding-right in arabic */
> }
> ```
>
> Documents might need both logical and physical properties. For instance the drop shadows on buttons on a page must remain consistent throughout, so their offset will be chosen based on visual considerations and physical directions, and not vary by writing system.

<a id="ref-for-flow-relative"></a>

Since CSS was originally designed with only physical coordinates in its controls, this module introduces text-flow–relative equivalents so that declarations in a CSS style sheet can be expressed in [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) terms. It defines the mapping and cascading of equivalent properties, some new properties and values equivalent to those in CSS2.1, and the principles used to derive their syntaxes. Future CSS specifications are expected to incorporate both sets of coordinates in their property and value definitions, so this module will not track the introduction of <a id="ref-for-flow-relative①"></a>flow-relative variants of newer CSS features.

<a id="ref-for-used-value"></a>

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-text-orientation"></a>

<a id="ref-for-flow-relative②"></a>

[CSS Writing Modes](https://www.w3.org/TR/css-writing-modes/)’ [Abstract Box Terminology](https://www.w3.org/TR/css-writing-modes-3/#abstract-box) section defines how to map between flow-relative and physical terms. This mapping, which depends on the [used values](https://www.w3.org/TR/css-cascade-5/#used-value) of [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation), controls the interpretation of [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) keywords and properties.

<a id="mapping-diagram"></a>

![](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/diagrams/sizing-ltr-tb.svg)

Correspondence of physical and flow-relative terms in typical English text layout

![](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/diagrams/sizing-ttb-rl.svg)

Correspondence of physical and flow-relative terms in vertical Chinese text layout

<a id="ref-for-propdef-text-orientation①"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-writing-mode①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Due to its interaction with [text-orientation: upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation), the [used](https://www.w3.org/TR/css-cascade-5/#used-value) [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) depends on the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) and <a id="ref-for-propdef-text-orientation②"></a>text-orientation.

Tests

General tests for logical properties

- [getComputedStyle-listing.html](https://wpt.fyi/results/css/css-logical/getComputedStyle-listing.html) [(live test)](http://wpt.live/css/css-logical/getComputedStyle-listing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/getComputedStyle-listing.html)
- [inheritance.html](https://wpt.fyi/results/css/css-logical/inheritance.html) [(live test)](http://wpt.live/css/css-logical/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/inheritance.html)
- [logicalprops-quirklength.html](https://wpt.fyi/results/css/css-logical/logicalprops-quirklength.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-quirklength.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-quirklength.html)

------------------------------------------------------------------------

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3d880eb1"></a> <strong>Things That Are Unstable</strong> Since implementation of parts of this module is effectively required for shipping an implementation of [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/) on the Web (in order to correctly implement the default HTML styles), the CSSWG resolved that the requisite features in [§ 2 Flow-Relative Values: block-start, block-end, inline-start, inline-end](#directional-keywords) and [§ 4 Flow-Relative Box Model Properties](#box) are approved for shipping. (See [FPWD announcement](https://lists.w3.org/Archives/Public/www-style/2017Dec/0043.html) for additional background.)
>
> However, there are a few significant open issues:
>
> - The logical keyword on shorthands, because the name of the keyword may change or it may be replaced by some other syntactic marker. (This feature will be deferred from this level for further development if there is no clearly satisfactory mechanism proposed, see [Issue 1282](https://github.com/w3c/csswg-drafts/issues/1282).)
>
> - Whether flow-relative longhands inherit from their namesake on the parent, or are mapped to a physical property and inherit from that property. (See [Issue 3029](https://github.com/w3c/csswg-drafts/issues/3029).)
>
> - <a id="ref-for-propdef-margin"></a>
>
>   Whether shorthands like [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) expand to both sets of longhands, or only the ones that were set. (See [Issue 3030](https://github.com/w3c/csswg-drafts/issues/3030).)
>
> Comments, suggestions, and use cases are welcome on these issues. Please file them in GitHub, tweet them to @csswg, or send them to www-style@w3.org.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="directional-keywords"></a>2.  Flow-Relative Values: block-start, block-end, inline-start, inline-end

<a id="ref-for-physical"></a>

<a id="ref-for-directional-keyword"></a>

<a id="ref-for-flow-relative③"></a>

Many CSS properties have historically accepted <a id="directional-keyword"></a>directional keyword values that are [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) (top, bottom, left, right). This specification introduces [directional keyword](#directional-keyword) values that are [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative): block-start, block-end, inline-start, inline-end.

A property’s effect can be either 1-dimensional or 2-dimensional. When contextually constrained to one dimension, the flow-relative keywords are abbreviated (to start and end).

<a id="ref-for-flow-relative④"></a>

CSS Level 2 properties are here redefined to also accept [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) directional keywords. Such values can be used in place of the corresponding physical values. For properties that take multiple keywords, combinations of flow-relative and physical values are not allowed (unless otherwise specified in a future specification).

<a id="ref-for-flow-relative⑤"></a>

<a id="ref-for-line-relative"></a>

<a id="ref-for-physical①"></a>

<a id="ref-for-writing-mode①"></a>

<a id="ref-for-containing-block"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Newer CSS specifications are expected in most cases to define [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) or [line-relative](https://www.w3.org/TR/css-writing-modes-4/#line-relative) values instead of or in addition to any [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) ones. In general, the mapping of such relative values are expected to use the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) when affecting the box itself, and that of the box itself when affecting its contents. Regardless, which <a id="ref-for-writing-mode②"></a>writing modes is used for the mapping needs to be explicitly defined.

<a id="ref-for-propdef-caption-side"></a>

### <a id="caption-side"></a>2.1.  Logical Values for the [caption-side](https://www.w3.org/TR/CSS2/tables.html#propdef-caption-side) Property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-caption-side①"></a>

[caption-side](https://www.w3.org/TR/CSS2/tables.html#propdef-caption-side)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[New values:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

inline-start [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-end

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

Tests

- [caption-side-no-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/caption-side-no-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/caption-side-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/caption-side-no-interpolation.html)

<a id="ref-for-propdef-caption-side②"></a>

<a id="ref-for-line-relative①"></a>

These two values are added only for implementations that support left and right values for [caption-side](https://www.w3.org/TR/CSS2/tables.html#propdef-caption-side). The left and right values themselves are defined to be [line-relative](https://www.w3.org/TR/css-writing-modes-4/#line-relative).

<a id="ref-for-block-start"></a>

<a id="ref-for-block-end"></a>

The existing top and bottom values are idiosyncratically redefined as assigning to the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) and [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) sides of the table, respectively.

<a id="ref-for-writing-mode③"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-table-wrapper-box"></a>

The mapping on this property uses the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the caption’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) (that is, the [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box)).

<a id="ref-for-propdef-float"></a>

<a id="ref-for-propdef-clear"></a>

### <a id="float-clear"></a>2.2.  Flow-Relative Values for the [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) Properties

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-clear①"></a>

<a id="ref-for-propdef-float①"></a>

[float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float), [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[New values:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①"></a>

inline-start [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-end

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

Tests

- [float-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/float-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/float-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/float-interpolation.html)
- [logical-values-float-clear-1.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear-1.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear-1.html)
- [logical-values-float-clear-2.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear-2.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear-2.html)
- [logical-values-float-clear-3.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear-3.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear-3.html)
- [logical-values-float-clear-4.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear-4.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear-4.html)
- [logical-values-float-clear-reftest.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear-reftest.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear-reftest.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear-reftest.html)
- [logical-values-float-clear.html](https://wpt.fyi/results/css/css-logical/logical-values-float-clear.html) [(live test)](http://wpt.live/css/css-logical/logical-values-float-clear.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-float-clear.html)

<a id="ref-for-writing-mode④"></a>

<a id="ref-for-containing-block②"></a>

The mapping on these properties uses the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block).

<a id="ref-for-flow-relative⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These properties are 1-dimensional in CSS2, but are planned to be expanded to two dimensions, and therefore are given unabbreviated [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) keywords.

<a id="ref-for-propdef-text-align"></a>

### <a id="text-align"></a>2.3.  Flow-Relative Values for the [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) Property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-text-align①"></a>

[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[New values:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one②"></a>

start [\|](https://www.w3.org/TR/css-values-4/#comb-one) end

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

These values are normatively defined in [\[css-text-3\]](#biblio-css-text-3).

## <a id="page"></a>3.  Flow-Relative Page Classifications

In CSS, all pages are classified by user agents as either left pages or right pages. [\[CSS2\]](#biblio-css2) Which page is first in a spread, however, depends on whether the page progression is left-to-right or right-to-left.

<a id="ref-for-propdef-page-break-after"></a>

<a id="ref-for-propdef-page-break-before"></a>

To allow control of page breaking to the page that is on the earlier or later side of a spread, rather than to the left or right side of a spread, this module introduces the following additional keywords for the [page-break-after](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-after) and [page-break-before](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-before) properties [\[CSS2\]](#biblio-css2):

<a id="valdef-logical-page-recto"></a>recto  
<a id="ref-for-propdef-left"></a>

<a id="ref-for-propdef-right"></a>

Equivalent to [right](https://www.w3.org/TR/css-position-3/#propdef-right) in left-to-right page progressions and [left](https://www.w3.org/TR/css-position-3/#propdef-left) in right-to-left page progressions.

<a id="valdef-logical-page-verso"></a>verso  
<a id="ref-for-propdef-right①"></a>

<a id="ref-for-propdef-left①"></a>

Equivalent to [left](https://www.w3.org/TR/css-position-3/#propdef-left) in left-to-right page progressions and [right](https://www.w3.org/TR/css-position-3/#propdef-right) in right-to-left page progressions.

These values are computed as specified and are further defined in [\[css-break-3\]](#biblio-css-break-3).

Although authors typically place page numbers using physical placements, the contents of headers often follows conventions depending on which page in the spread is earlier. Therefore the following flow-relative [page selectors](https://www.w3.org/TR/CSS2/page.html#page-selectors) are also added to support flow-relative page selection:

<a id="valdef-logical-page-selector-recto"></a>:recto  
Equivalent to ':right' in left-to-right page progressions and ':left' in right-to-left page progressions.

<a id="valdef-logical-page-selector-verso"></a>:verso  
Equivalent to ':left' in left-to-right page progressions and ':right' in right-to-left page progressions.

The flow-relative page selectors have specificity equal to the ':left' and ':right' page selectors.

## <a id="box"></a>4.  Flow-Relative Box Model Properties

<a id="ref-for-flow-relative⑦"></a>

<a id="ref-for-physical②"></a>

For many formatting effects, the axis or direction affected is encoded in the property name rather than in its value. The type of directional or axis mapping ([flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) or [physical](https://www.w3.org/TR/css-writing-modes-4/#physical)) of each such property is called its <a id="mapping-logic"></a>mapping logic. Historically, all properties have been encoded in <a id="ref-for-physical③"></a>physical terms; this specification introduces new CSS properties that are <a id="ref-for-flow-relative⑧"></a>flow-relative equivalents of CSS2’s <a id="ref-for-physical④"></a>physical box model properties.

<a id="ref-for-flow-relative⑨"></a>

<a id="ref-for-line-relative②"></a>

<a id="ref-for-physical⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Newer CSS specifications are expected in most cases to define [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) or [line-relative](https://www.w3.org/TR/css-writing-modes-4/#line-relative) properties instead of or in addition to any [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) ones.

<a id="ref-for-flow-relative①⓪"></a>

<a id="ref-for-physical⑥"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-logical-property-group"></a>

<a id="ref-for-longhand"></a>

Each set of parallel [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties and [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties (ignoring [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) properties) related by setting equivalent styles on the various sides or dimensions of a box, forms a <a id="logical-property-group"></a>logical property group. For example, the padding-\* properties form a single [logical property group](#logical-property-group), the margin-\* properties form a separate <a id="ref-for-logical-property-group①"></a>logical property group, the border-\*-style properties form another <a id="ref-for-logical-property-group②"></a>logical property group, etc. (Each [longhand property](https://www.w3.org/TR/css-cascade-5/#longhand) can belong to at most one <a id="ref-for-logical-property-group③"></a>logical property group.)

<a id="ref-for-logical-property-group④"></a>

<a id="ref-for-flow-relative①①"></a>

<a id="ref-for-physical⑦"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-writing-mode⑤"></a>

<a id="ref-for-specified-value"></a>

<a id="ref-for-cascade"></a>

Within each [logical property group](#logical-property-group), corresponding [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) and [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties are paired using the element’s own [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode). Although the [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) of each property remains distinct, paired properties share a <a id="ref-for-computed-value②"></a>computed value. This shared value is determined by [cascading](https://www.w3.org/TR/css-cascade-6/#cascade) the declarations of both properties together as one; in other words, the <a id="ref-for-computed-value③"></a>computed value of both properties in the pair is derived from the <a id="ref-for-specified-value①"></a>specified value of the property declared with higher priority in the CSS <a id="ref-for-cascade①"></a>cascade. [\[CSS-CASCADE-3\]](#biblio-css-cascade-3)

<a id="ref-for-propdef-writing-mode②"></a>

<a id="ref-for-propdef-direction②"></a>

<a id="ref-for-propdef-text-orientation③"></a>

<a id="ref-for-flow-relative①②"></a>

<a id="ref-for-physical⑧"></a>

<a id="ref-for-logical-property-group⑤"></a>

<a id="ref-for-computed-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that this requires implementations to maintain relative order of declarations within a CSS declaration block, which was not previously required for CSS cascading. It also requires that [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) be computed as a prerequisite for cascading together the [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) and [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) declarations of a [logical property group](#logical-property-group) to find their [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0ead529c"></a> For example, given the following rule:
>
> ```css
> p {
>   margin-inline-start: 1px;
>   margin-left: 2px;
>   margin-inline-end: 3px;
> }
> ```
>
> <a id="ref-for-propdef-writing-mode③"></a>
>
> <a id="ref-for-valdef-writing-mode-horizontal-tb"></a>
>
> <a id="ref-for-propdef-direction③"></a>
>
> <a id="ref-for-valdef-direction-ltr"></a>
>
> <a id="ref-for-propdef-margin-left"></a>
>
> <a id="ref-for-propdef-margin-inline-start"></a>
>
> <a id="ref-for-valdef-direction-rtl"></a>
>
> <a id="ref-for-propdef-margin-inline-end"></a>
>
> In a paragraph with computed [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) being [horizontal-tb](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-horizontal-tb) and computed [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) being [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr), the computed value of [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) is 2px, since for that <a id="ref-for-propdef-writing-mode④"></a>writing-mode and <a id="ref-for-propdef-direction④"></a>direction, [margin-inline-start](#propdef-margin-inline-start) and <a id="ref-for-propdef-margin-left①"></a>margin-left share a computed value, and the declaration of <a id="ref-for-propdef-margin-left②"></a>margin-left is after the declaration of <a id="ref-for-propdef-margin-inline-start①"></a>margin-inline-start. However, if the computed <a id="ref-for-propdef-direction⑤"></a>direction were instead [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl), the computed value of <a id="ref-for-propdef-margin-left③"></a>margin-left is 3px, since [margin-inline-end](#propdef-margin-inline-end) and <a id="ref-for-propdef-margin-left④"></a>margin-left share a computed value, and the declaration of <a id="ref-for-propdef-margin-inline-end①"></a>margin-inline-end is after the declaration of <a id="ref-for-propdef-margin-left⑤"></a>margin-left.

[\[CSSOM\]](#biblio-cssom) APIs that return computed values (such as `getComputedStyle()`) must return the same value for each individual property in such a pair.

<a id="ref-for-writing-mode⑥"></a>

<a id="ref-for-flow-relative①③"></a>

<a id="ref-for-physical⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the element’s own [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) for mapping every [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) property to its [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) equivalent simplifies the cascading calculations and gives a straightforward model for authors to reason about. However, it is problematic in many cases, see for example [this discussion](https://www.w3.org/mid/20161108202634.GA7235@mail.internode.on.net). Authors may need to use nested elements to get the correct mapping behavior when changing an element’s <a id="ref-for-writing-mode⑦"></a>writing mode from its parent.

<a id="ref-for-inline-start"></a>

<a id="ref-for-valdef-direction-rtl①"></a>

<a id="ref-for-propdef-margin-inline-start②"></a>

<a id="ref-for-valdef-direction-ltr①"></a>

Inheritance of each property is from its corresponding property on the parent. For example, although the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) margin of an[rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl) box is its right margin, [margin-inline-start](#propdef-margin-inline-start) on this box will inherit the <a id="ref-for-propdef-margin-inline-start③"></a>margin-inline-start of an [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) parent even though that happens to be the parent’s <em>left</em> margin.

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-all"></a>

<a id="ref-for-propdef-margin①"></a>

<a id="ref-for-valdef-all-inherit"></a>

Unless otherwise specified, [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that encompass both logical and physical longhands (such as the [all](https://www.w3.org/TR/css-cascade-5/#propdef-all) shorthand) set their physical longhands last. For example, <a id="ref-for-propdef-all①"></a>all: inherit will set all of the [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) properties to [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit), but since the physical longhands are set last, the child’s margins will inherit from their physical counterparts in the parent.

Tests

- [animation-001.html](https://wpt.fyi/results/css/css-logical/animation-001.html) [(live test)](http://wpt.live/css/css-logical/animation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animation-001.html)
- [animation-002.html](https://wpt.fyi/results/css/css-logical/animation-002.html) [(live test)](http://wpt.live/css/css-logical/animation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animation-002.html)
- [animation-004.html](https://wpt.fyi/results/css/css-logical/animation-004.html) [(live test)](http://wpt.live/css/css-logical/animation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animation-004.html)
- [logicalprops-with-deferred-writing-mode.html](https://wpt.fyi/results/css/css-logical/logicalprops-with-deferred-writing-mode.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-with-deferred-writing-mode.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-with-deferred-writing-mode.html)
- [logicalprops-with-variables.html](https://wpt.fyi/results/css/css-logical/logicalprops-with-variables.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-with-variables.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-with-variables.html)

<a id="ref-for-propdef-block-size"></a>

<a id="ref-for-propdef-inline-size"></a>

<a id="ref-for-propdef-min-block-size"></a>

<a id="ref-for-propdef-min-inline-size"></a>

<a id="ref-for-propdef-max-block-size"></a>

<a id="ref-for-propdef-max-inline-size"></a>

### <a id="dimension-properties"></a>4.1.  Logical Height and Logical Width: the [block-size](#propdef-block-size)/[inline-size](#propdef-inline-size), [min-block-size](#propdef-min-block-size)/[min-inline-size](#propdef-min-inline-size), and [max-block-size](#propdef-max-block-size)/[max-inline-size](#propdef-max-inline-size) properties

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-block-size"></a>block-size, <a id="propdef-inline-size"></a>inline-size

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-width"></a>

[\<'width'\>](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-height"></a>

Same as [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-propdef-height①"></a>

Same as [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

size

Tests

- [cascading-001.html](https://wpt.fyi/results/css/css-logical/cascading-001.html) [(live test)](http://wpt.live/css/css-logical/cascading-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/cascading-001.html)
- [logical-box-size.html](https://wpt.fyi/results/css/css-logical/logical-box-size.html) [(live test)](http://wpt.live/css/css-logical/logical-box-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-size.html)
- [logicalprops-block-size-vlr.html](https://wpt.fyi/results/css/css-logical/logicalprops-block-size-vlr.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-block-size-vlr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-block-size-vlr.html)
- [logicalprops-block-size.html](https://wpt.fyi/results/css/css-logical/logicalprops-block-size.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-block-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-block-size.html)
- [logicalprops-inline-size-vlr.html](https://wpt.fyi/results/css/css-logical/logicalprops-inline-size-vlr.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-inline-size-vlr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-inline-size-vlr.html)
- [logicalprops-inline-size.html](https://wpt.fyi/results/css/css-logical/logicalprops-inline-size.html) [(live test)](http://wpt.live/css/css-logical/logicalprops-inline-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logicalprops-inline-size.html)
- [block-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/block-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/block-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/block-size-computed.html)
- [block-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/block-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/block-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/block-size-invalid.html)
- [block-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/block-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/block-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/block-size-valid.html)
- [inline-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/inline-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/inline-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inline-size-computed.html)
- [inline-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/inline-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inline-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inline-size-invalid.html)
- [inline-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/inline-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inline-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inline-size-valid.html)

<a id="ref-for-propdef-height②"></a>

<a id="ref-for-propdef-width③"></a>

<a id="ref-for-propdef-writing-mode⑤"></a>

These properties correspond to the [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode).

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-min-block-size"></a>min-block-size, <a id="propdef-min-inline-size"></a>min-inline-size

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-min-width"></a>

[\<'min-width'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-propdef-height③"></a>

same as [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-min-width①"></a>

<a id="ref-for-propdef-min-height"></a>

Same as [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

min-size

Tests

- [min-block-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/min-block-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-block-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-block-size-computed.html)
- [min-block-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/min-block-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-block-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-block-size-invalid.html)
- [min-block-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/min-block-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-block-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-block-size-valid.html)
- [min-inline-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/min-inline-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-inline-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-inline-size-computed.html)
- [min-inline-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/min-inline-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-inline-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-inline-size-invalid.html)
- [min-inline-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/min-inline-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/min-inline-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/min-inline-size-valid.html)

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-writing-mode⑥"></a>

These properties correspond to the [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) and [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode).

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-max-block-size"></a>max-block-size, <a id="propdef-max-inline-size"></a>max-inline-size

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-max-width"></a>

[\<'max-width'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-width⑤"></a>

<a id="ref-for-propdef-height④"></a>

same as [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-max-width①"></a>

<a id="ref-for-propdef-max-height"></a>

Same as [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

max-size

Tests

- [max-block-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/max-block-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-block-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-block-size-computed.html)
- [max-block-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/max-block-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-block-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-block-size-invalid.html)
- [max-block-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/max-block-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-block-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-block-size-valid.html)
- [max-inline-size-computed.html](https://wpt.fyi/results/css/css-logical/parsing/max-inline-size-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-inline-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-inline-size-computed.html)
- [max-inline-size-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/max-inline-size-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-inline-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-inline-size-invalid.html)
- [max-inline-size-valid.html](https://wpt.fyi/results/css/css-logical/parsing/max-inline-size-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/max-inline-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/max-inline-size-valid.html)

<a id="ref-for-propdef-max-height①"></a>

<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-propdef-writing-mode⑦"></a>

These properties correspond to the [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) and [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode).

<a id="ref-for-propdef-margin-block-start"></a>

<a id="ref-for-propdef-margin-block-end"></a>

<a id="ref-for-propdef-margin-inline-start④"></a>

<a id="ref-for-propdef-margin-inline-end②"></a>

<a id="ref-for-propdef-margin-block"></a>

<a id="ref-for-propdef-margin-inline"></a>

### <a id="margin-properties"></a>4.2.  Flow-Relative Margins: the [margin-block-start](#propdef-margin-block-start), [margin-block-end](#propdef-margin-block-end), [margin-inline-start](#propdef-margin-inline-start), [margin-inline-end](#propdef-margin-inline-end) properties and [margin-block](#propdef-margin-block) and [margin-inline](#propdef-margin-inline) shorthands

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-margin-block-start"></a>margin-block-start, <a id="propdef-margin-block-end"></a>margin-block-end, <a id="propdef-margin-inline-start"></a>margin-inline-start, <a id="propdef-margin-inline-end"></a>margin-inline-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-margin-top"></a>

[\<'margin-top'\>](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-margin-top①"></a>

Same as [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding margin-\* properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-margin②"></a>

[margin](https://www.w3.org/TR/css-box-4/#propdef-margin)

Tests

- [logical-box-margin.html](https://wpt.fyi/results/css/css-logical/logical-box-margin.html) [(live test)](http://wpt.live/css/css-logical/logical-box-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-margin.html)
- [margin-block-inline-computed.html](https://wpt.fyi/results/css/css-logical/parsing/margin-block-inline-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/margin-block-inline-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/margin-block-inline-computed.html)
- [margin-block-inline-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/margin-block-inline-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/margin-block-inline-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/margin-block-inline-invalid.html)
- [margin-block-inline-shorthand.html](https://wpt.fyi/results/css/css-logical/parsing/margin-block-inline-shorthand.html) [(live test)](http://wpt.live/css/css-logical/parsing/margin-block-inline-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/margin-block-inline-shorthand.html)
- [margin-block-inline-valid.html](https://wpt.fyi/results/css/css-logical/parsing/margin-block-inline-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/margin-block-inline-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/margin-block-inline-valid.html)

<a id="ref-for-propdef-margin-top②"></a>

<a id="ref-for-propdef-margin-bottom"></a>

<a id="ref-for-propdef-margin-left⑥"></a>

<a id="ref-for-propdef-margin-right"></a>

<a id="ref-for-propdef-writing-mode⑧"></a>

<a id="ref-for-propdef-direction⑥"></a>

<a id="ref-for-propdef-text-orientation④"></a>

These properties correspond to the [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top), [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom), [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left), and [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-margin-block"></a>margin-block, <a id="propdef-margin-inline"></a>margin-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range"></a>

<a id="ref-for-propdef-margin-top③"></a>

[\<'margin-top'\>](https://www.w3.org/TR/css-box-4/#propdef-margin-top)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [margin-block-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/margin-block-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/margin-block-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/margin-block-interpolation.html)
- [margin-inline-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/margin-inline-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/margin-inline-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/margin-inline-interpolation.html)

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-margin-block-start①"></a>

<a id="ref-for-propdef-margin-block-end①"></a>

<a id="ref-for-propdef-margin-inline-start⑤"></a>

<a id="ref-for-propdef-margin-inline-end③"></a>

<a id="ref-for-start"></a>

<a id="ref-for-end"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [margin-block-start](#propdef-margin-block-start) &#x26; [margin-block-end](#propdef-margin-block-end) and [margin-inline-start](#propdef-margin-inline-start) &#x26; [margin-inline-end](#propdef-margin-inline-end), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge style, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge style. If only one value is given, it applies to both the <a id="ref-for-start①"></a>start and <a id="ref-for-end①"></a>end edges.

<a id="ref-for-propdef-inset-block-start"></a>

<a id="ref-for-propdef-inset-block-end"></a>

<a id="ref-for-propdef-inset-inline-start"></a>

<a id="ref-for-propdef-inset-inline-end"></a>

<a id="ref-for-propdef-inset-block"></a>

<a id="ref-for-propdef-inset-inline"></a>

<a id="ref-for-propdef-inset"></a>

### <a id="position-properties"></a>4.3.  Flow-Relative Offsets: the [inset-block-start](#propdef-inset-block-start), [inset-block-end](#propdef-inset-block-end), [inset-inline-start](#propdef-inset-inline-start), [inset-inline-end](#propdef-inset-inline-end) properties and [inset-block](#propdef-inset-block), [inset-inline](#propdef-inset-inline), and [inset](#propdef-inset) shorthands

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-left②"></a>

<a id="ref-for-propdef-bottom"></a>

<a id="ref-for-propdef-right②"></a>

<a id="ref-for-propdef-inset-block-start①"></a>

<a id="ref-for-propdef-inset-block-end①"></a>

<a id="ref-for-propdef-inset-inline-start①"></a>

<a id="ref-for-propdef-inset-inline-end①"></a>

<a id="ref-for-propdef-inset-block①"></a>

<a id="ref-for-propdef-inset-inline①"></a>

<a id="ref-for-propdef-inset①"></a>

The [top](https://www.w3.org/TR/css-position-3/#propdef-top), [left](https://www.w3.org/TR/css-position-3/#propdef-left), [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom), [right](https://www.w3.org/TR/css-position-3/#propdef-right) physical properties, their [inset-block-start](#propdef-inset-block-start), [inset-block-end](#propdef-inset-block-end), [inset-inline-start](#propdef-inset-inline-start), [inset-inline-end](#propdef-inset-inline-end) flow-relative correspondents, and the [inset-block](#propdef-inset-block), [inset-inline](#propdef-inset-inline), and [inset](#propdef-inset) shorthands, are collectively known as the <a id="inset-properties"></a>inset properties.

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-inset-block-start"></a>inset-block-start, <a id="propdef-inset-block-end"></a>inset-block-end, <a id="propdef-inset-inline-start"></a>inset-inline-start, <a id="propdef-inset-inline-end"></a>inset-inline-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-top①"></a>

[\<'top'\>](https://www.w3.org/TR/css-position-3/#propdef-top)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

positioned elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-left③"></a>

<a id="ref-for-propdef-bottom①"></a>

<a id="ref-for-propdef-right③"></a>

<a id="ref-for-propdef-top②"></a>

Same as corresponding [top](https://www.w3.org/TR/css-position-3/#propdef-top)/[right](https://www.w3.org/TR/css-position-3/#propdef-right)/[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)/[left](https://www.w3.org/TR/css-position-3/#propdef-left) properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-inset②"></a>

[inset](#propdef-inset)

Tests

- [logical-box-inset.html](https://wpt.fyi/results/css/css-logical/logical-box-inset.html) [(live test)](http://wpt.live/css/css-logical/logical-box-inset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-inset.html)
- [inset-block-inline-computed.html](https://wpt.fyi/results/css/css-logical/parsing/inset-block-inline-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-block-inline-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-block-inline-computed.html)
- [inset-block-inline-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/inset-block-inline-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-block-inline-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-block-inline-invalid.html)
- [inset-block-inline-shorthand.html](https://wpt.fyi/results/css/css-logical/parsing/inset-block-inline-shorthand.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-block-inline-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-block-inline-shorthand.html)
- [inset-block-inline-valid.html](https://wpt.fyi/results/css/css-logical/parsing/inset-block-inline-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-block-inline-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-block-inline-valid.html)

<a id="ref-for-propdef-top③"></a>

<a id="ref-for-propdef-bottom②"></a>

<a id="ref-for-propdef-left④"></a>

<a id="ref-for-propdef-right④"></a>

<a id="ref-for-propdef-writing-mode⑨"></a>

<a id="ref-for-propdef-direction⑦"></a>

<a id="ref-for-propdef-text-orientation⑤"></a>

These properties correspond to the [top](https://www.w3.org/TR/css-position-3/#propdef-top), [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom), [left](https://www.w3.org/TR/css-position-3/#propdef-left), and [right](https://www.w3.org/TR/css-position-3/#propdef-right) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-inset-block"></a>inset-block, <a id="propdef-inset-inline"></a>inset-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range①"></a>

<a id="ref-for-propdef-top④"></a>

[\<'top'\>](https://www.w3.org/TR/css-position-3/#propdef-top)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-shorthand-property③"></a>

<a id="ref-for-propdef-inset-block-start②"></a>

<a id="ref-for-propdef-inset-block-end②"></a>

<a id="ref-for-propdef-inset-inline-start②"></a>

<a id="ref-for-propdef-inset-inline-end②"></a>

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [inset-block-start](#propdef-inset-block-start) &#x26; [inset-block-end](#propdef-inset-block-end) and [inset-inline-start](#propdef-inset-inline-start) &#x26; [inset-inline-end](#propdef-inset-inline-end), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge style, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge style. If only one value is given, it applies to both the <a id="ref-for-start③"></a>start and <a id="ref-for-end③"></a>end edges.

<strong>Table 11 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-inset"></a>inset

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range②"></a>

<a id="ref-for-propdef-top⑤"></a>

[\<'top'\>](https://www.w3.org/TR/css-position-3/#propdef-top)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [inset-computed.html](https://wpt.fyi/results/css/css-logical/parsing/inset-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-computed.html)
- [inset-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/inset-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-invalid.html)
- [inset-shorthand.html](https://wpt.fyi/results/css/css-logical/parsing/inset-shorthand.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-shorthand.html)
- [inset-valid.html](https://wpt.fyi/results/css/css-logical/parsing/inset-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/inset-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/inset-valid.html)

<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-propdef-top⑥"></a>

<a id="ref-for-propdef-right⑤"></a>

<a id="ref-for-propdef-bottom③"></a>

<a id="ref-for-propdef-left⑤"></a>

<a id="ref-for-longhand①"></a>

<a id="ref-for-propdef-margin③"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets the [top](https://www.w3.org/TR/css-position-3/#propdef-top), [right](https://www.w3.org/TR/css-position-3/#propdef-right), [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom), and [left](https://www.w3.org/TR/css-position-3/#propdef-left) properties. Values are assigned to its [sub-properties](https://www.w3.org/TR/css-cascade-5/#longhand) as for [margin](https://www.w3.org/TR/css-box-4/#propdef-margin).

<a id="ref-for-propdef-padding-block-start"></a>

<a id="ref-for-propdef-padding-block-end"></a>

<a id="ref-for-propdef-padding-inline-start"></a>

<a id="ref-for-propdef-padding-inline-end"></a>

<a id="ref-for-propdef-padding-block"></a>

<a id="ref-for-propdef-padding-inline"></a>

### <a id="padding-properties"></a>4.4.  Flow-Relative Padding: the [padding-block-start](#propdef-padding-block-start), [padding-block-end](#propdef-padding-block-end), [padding-inline-start](#propdef-padding-inline-start), [padding-inline-end](#propdef-padding-inline-end) properties and [padding-block](#propdef-padding-block) and [padding-inline](#propdef-padding-inline) shorthands

<strong>Table 12 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-padding-block-start"></a>padding-block-start, <a id="propdef-padding-block-end"></a>padding-block-end, <a id="propdef-padding-inline-start"></a>padding-inline-start, <a id="propdef-padding-inline-end"></a>padding-inline-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-padding-top"></a>

[\<'padding-top'\>](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

0

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-padding-top①"></a>

Same as [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

As for the corresponding physical property

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding padding-\* properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-padding"></a>

[padding](https://www.w3.org/TR/css-box-4/#propdef-padding)

Tests

- [logical-box-padding.html](https://wpt.fyi/results/css/css-logical/logical-box-padding.html) [(live test)](http://wpt.live/css/css-logical/logical-box-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-padding.html)
- [padding-block-inline-computed.html](https://wpt.fyi/results/css/css-logical/parsing/padding-block-inline-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/padding-block-inline-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/padding-block-inline-computed.html)
- [padding-block-inline-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/padding-block-inline-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/padding-block-inline-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/padding-block-inline-invalid.html)
- [padding-block-inline-shorthand.html](https://wpt.fyi/results/css/css-logical/parsing/padding-block-inline-shorthand.html) [(live test)](http://wpt.live/css/css-logical/parsing/padding-block-inline-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/padding-block-inline-shorthand.html)
- [padding-block-inline-valid.html](https://wpt.fyi/results/css/css-logical/parsing/padding-block-inline-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/padding-block-inline-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/padding-block-inline-valid.html)

<a id="ref-for-propdef-padding-top②"></a>

<a id="ref-for-propdef-padding-bottom"></a>

<a id="ref-for-propdef-padding-left"></a>

<a id="ref-for-propdef-padding-right"></a>

<a id="ref-for-propdef-writing-mode①⓪"></a>

<a id="ref-for-propdef-direction⑧"></a>

<a id="ref-for-propdef-text-orientation⑥"></a>

These properties correspond to the [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top), [padding-bottom](https://www.w3.org/TR/css-box-4/#propdef-padding-bottom), [padding-left](https://www.w3.org/TR/css-box-4/#propdef-padding-left), and [padding-right](https://www.w3.org/TR/css-box-4/#propdef-padding-right) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 13 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-padding-block"></a>padding-block, <a id="propdef-padding-inline"></a>padding-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range③"></a>

<a id="ref-for-propdef-padding-top③"></a>

[\<'padding-top'\>](https://www.w3.org/TR/css-box-4/#propdef-padding-top)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [padding-block-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/padding-block-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/padding-block-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/padding-block-interpolation.html)
- [padding-inline-interpolation.html](https://wpt.fyi/results/css/css-logical/animations/padding-inline-interpolation.html) [(live test)](http://wpt.live/css/css-logical/animations/padding-inline-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/animations/padding-inline-interpolation.html)

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-propdef-padding-block-start①"></a>

<a id="ref-for-propdef-padding-block-end①"></a>

<a id="ref-for-propdef-padding-inline-start①"></a>

<a id="ref-for-propdef-padding-inline-end①"></a>

<a id="ref-for-start④"></a>

<a id="ref-for-end④"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [padding-block-start](#propdef-padding-block-start) &#x26; [padding-block-end](#propdef-padding-block-end) and [padding-inline-start](#propdef-padding-inline-start) &#x26; [padding-inline-end](#propdef-padding-inline-end), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge style, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge style. If only one value is given, it applies to both the <a id="ref-for-start⑤"></a>start and <a id="ref-for-end⑤"></a>end edges.

### <a id="border-properties"></a>4.5.  Flow-Relative Borders

<a id="ref-for-propdef-border-block-start-width"></a>

<a id="ref-for-propdef-border-block-end-width"></a>

<a id="ref-for-propdef-border-inline-start-width"></a>

<a id="ref-for-propdef-border-inline-end-width"></a>

<a id="ref-for-propdef-border-block-width"></a>

<a id="ref-for-propdef-border-inline-width"></a>

#### <a id="border-width"></a>4.5.1.  Flow-Relative Border Widths: the [border-block-start-width](#propdef-border-block-start-width), [border-block-end-width](#propdef-border-block-end-width), [border-inline-start-width](#propdef-border-inline-start-width), [border-inline-end-width](#propdef-border-inline-end-width) properties and [border-block-width](#propdef-border-block-width) and [border-inline-width](#propdef-border-inline-width) shorthands

<strong>Table 14 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-start-width"></a>border-block-start-width, <a id="propdef-border-block-end-width"></a>border-block-end-width, <a id="propdef-border-inline-start-width"></a>border-inline-start-width, <a id="propdef-border-inline-end-width"></a>border-inline-end-width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-width"></a>

[\<'border-top-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

medium

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-width①"></a>

Same as [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding border-\*-width properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-width"></a>

[border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)

Tests

- [logical-box-border-width.html](https://wpt.fyi/results/css/css-logical/logical-box-border-width.html) [(live test)](http://wpt.live/css/css-logical/logical-box-border-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-border-width.html)

<a id="ref-for-propdef-border-top-width②"></a>

<a id="ref-for-propdef-border-bottom-width"></a>

<a id="ref-for-propdef-border-left-width"></a>

<a id="ref-for-propdef-border-right-width"></a>

<a id="ref-for-propdef-writing-mode①①"></a>

<a id="ref-for-propdef-direction⑨"></a>

<a id="ref-for-propdef-text-orientation⑦"></a>

These properties correspond to the [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width), [border-bottom-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width), [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width), and [border-right-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 15 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-width"></a>border-block-width, <a id="propdef-border-inline-width"></a>border-inline-width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range④"></a>

<a id="ref-for-propdef-border-top-width③"></a>

[\<'border-top-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [border-block-width-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-width-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-width-computed.html)
- [border-block-width-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-width-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-width-invalid.html)
- [border-block-width-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-width-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-width-valid.html)
- [border-inline-width-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-width-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-width-computed.html)
- [border-inline-width-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-width-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-width-invalid.html)
- [border-inline-width-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-width-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-width-valid.html)

<a id="ref-for-shorthand-property⑥"></a>

<a id="ref-for-propdef-border-block-start-width①"></a>

<a id="ref-for-propdef-border-block-end-width①"></a>

<a id="ref-for-propdef-border-inline-start-width①"></a>

<a id="ref-for-propdef-border-inline-end-width①"></a>

<a id="ref-for-start⑥"></a>

<a id="ref-for-end⑥"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-width](#propdef-border-block-start-width) &#x26; [border-block-end-width](#propdef-border-block-end-width) and [border-inline-start-width](#propdef-border-inline-start-width) &#x26; [border-inline-end-width](#propdef-border-inline-end-width), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge width, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge width. If only one value is given, it applies to both the <a id="ref-for-start⑦"></a>start and <a id="ref-for-end⑦"></a>end edges.

<a id="ref-for-propdef-border-block-start-style"></a>

<a id="ref-for-propdef-border-block-end-style"></a>

<a id="ref-for-propdef-border-inline-start-style"></a>

<a id="ref-for-propdef-border-inline-end-style"></a>

<a id="ref-for-propdef-border-block-style"></a>

<a id="ref-for-propdef-border-inline-style"></a>

#### <a id="border-style"></a>4.5.2.  Flow-Relative Border Styles: the [border-block-start-style](#propdef-border-block-start-style), [border-block-end-style](#propdef-border-block-end-style), [border-inline-start-style](#propdef-border-inline-start-style), [border-inline-end-style](#propdef-border-inline-end-style) properties and [border-block-style](#propdef-border-block-style) and [border-inline-style](#propdef-border-inline-style) shorthands

<strong>Table 16 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-start-style"></a>border-block-start-style, <a id="propdef-border-block-end-style"></a>border-block-end-style, <a id="propdef-border-inline-start-style"></a>border-inline-start-style, <a id="propdef-border-inline-end-style"></a>border-inline-end-style

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-style"></a>

[\<'border-top-style'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-style)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-style①"></a>

Same as [border-top-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-style)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding border-\*-style properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-style"></a>

[border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style)

Tests

- [logical-box-border-style.html](https://wpt.fyi/results/css/css-logical/logical-box-border-style.html) [(live test)](http://wpt.live/css/css-logical/logical-box-border-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-border-style.html)
- [border-block-style-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-style-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-style-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-style-computed.html)
- [border-block-style-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-style-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-style-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-style-invalid.html)
- [border-block-style-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-style-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-style-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-style-valid.html)
- [border-inline-style-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-style-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-style-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-style-computed.html)
- [border-inline-style-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-style-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-style-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-style-invalid.html)
- [border-inline-style-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-style-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-style-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-style-valid.html)

<a id="ref-for-propdef-border-top-style②"></a>

<a id="ref-for-propdef-border-bottom-style"></a>

<a id="ref-for-propdef-border-left-style"></a>

<a id="ref-for-propdef-border-right-style"></a>

<a id="ref-for-propdef-writing-mode①②"></a>

<a id="ref-for-propdef-direction①⓪"></a>

<a id="ref-for-propdef-text-orientation⑧"></a>

These properties correspond to the [border-top-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-style), [border-bottom-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-style), [border-left-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-style), and [border-right-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-style) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 17 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-style"></a>border-block-style, <a id="propdef-border-inline-style"></a>border-inline-style

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range⑤"></a>

<a id="ref-for-propdef-border-top-style③"></a>

[\<'border-top-style'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-style)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-propdef-border-block-start-style①"></a>

<a id="ref-for-propdef-border-block-end-style①"></a>

<a id="ref-for-propdef-border-inline-start-style①"></a>

<a id="ref-for-propdef-border-inline-end-style①"></a>

<a id="ref-for-start⑧"></a>

<a id="ref-for-end⑧"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-style](#propdef-border-block-start-style) &#x26; [border-block-end-style](#propdef-border-block-end-style) and [border-inline-start-style](#propdef-border-inline-start-style) &#x26; [border-inline-end-style](#propdef-border-inline-end-style), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge style, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge style. If only one value is given, it applies to both the <a id="ref-for-start⑨"></a>start and <a id="ref-for-end⑨"></a>end edges.

<a id="ref-for-propdef-border-block-start-color"></a>

<a id="ref-for-propdef-border-block-end-color"></a>

<a id="ref-for-propdef-border-inline-start-color"></a>

<a id="ref-for-propdef-border-inline-end-color"></a>

<a id="ref-for-propdef-border-block-color"></a>

<a id="ref-for-propdef-border-inline-color"></a>

#### <a id="border-color"></a>4.5.3.  Flow-Relative Border Colors: the [border-block-start-color](#propdef-border-block-start-color), [border-block-end-color](#propdef-border-block-end-color), [border-inline-start-color](#propdef-border-inline-start-color), [border-inline-end-color](#propdef-border-inline-end-color) properties and [border-block-color](#propdef-border-block-color) and [border-inline-color](#propdef-border-inline-color) shorthands

<strong>Table 18 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-start-color"></a>border-block-start-color, <a id="propdef-border-block-end-color"></a>border-block-end-color, <a id="propdef-border-inline-start-color"></a>border-inline-start-color, <a id="propdef-border-inline-end-color"></a>border-inline-end-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-color"></a>

[\<'border-top-color'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

currentcolor

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-color①"></a>

Same as [border-top-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding border-\*-color properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-color"></a>

[border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color)

Tests

- [logical-box-border-color.html](https://wpt.fyi/results/css/css-logical/logical-box-border-color.html) [(live test)](http://wpt.live/css/css-logical/logical-box-border-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-border-color.html)
- [border-block-color-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-color-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-color-computed.html)
- [border-block-color-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-color-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-color-invalid.html)
- [border-block-color-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-color-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-color-valid.html)
- [border-inline-color-computed.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-color-computed.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-color-computed.html)
- [border-inline-color-invalid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-color-invalid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-color-invalid.html)
- [border-inline-color-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-color-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-color-valid.html)

<a id="ref-for-propdef-border-top-color②"></a>

<a id="ref-for-propdef-border-bottom-color"></a>

<a id="ref-for-propdef-border-left-color"></a>

<a id="ref-for-propdef-border-right-color"></a>

<a id="ref-for-propdef-writing-mode①③"></a>

<a id="ref-for-propdef-direction①①"></a>

<a id="ref-for-propdef-text-orientation⑨"></a>

These properties correspond to the [border-top-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color), [border-bottom-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-color), [border-left-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-color), and [border-right-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-color) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 19 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-color"></a>border-block-color, <a id="propdef-border-inline-color"></a>border-inline-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range⑥"></a>

<a id="ref-for-propdef-border-top-color③"></a>

[\<'border-top-color'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-shorthand-property⑧"></a>

<a id="ref-for-propdef-border-block-start-color①"></a>

<a id="ref-for-propdef-border-block-end-color①"></a>

<a id="ref-for-propdef-border-inline-start-color①"></a>

<a id="ref-for-propdef-border-inline-end-color①"></a>

<a id="ref-for-start①⓪"></a>

<a id="ref-for-end①⓪"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-color](#propdef-border-block-start-color) &#x26; [border-block-end-color](#propdef-border-block-end-color) and [border-inline-start-color](#propdef-border-inline-start-color) &#x26; [border-inline-end-color](#propdef-border-inline-end-color), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-3/#start) edge color, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-3/#end) edge color. If only one value is given, it applies to both the <a id="ref-for-start①①"></a>start and <a id="ref-for-end①①"></a>end edges.

<a id="ref-for-propdef-border-block-start"></a>

<a id="ref-for-propdef-border-block-end"></a>

<a id="ref-for-propdef-border-inline-start"></a>

<a id="ref-for-propdef-border-inline-end"></a>

<a id="ref-for-propdef-border-block"></a>

<a id="ref-for-propdef-border-inline"></a>

#### <a id="border-shorthands"></a>4.5.4.  Flow-Relative Border Shorthands: the [border-block-start](#propdef-border-block-start), [border-block-end](#propdef-border-block-end), [border-inline-start](#propdef-border-inline-start), [border-inline-end](#propdef-border-inline-end) properties and [border-block](#propdef-border-block) and [border-inline](#propdef-border-inline) shorthands

<strong>Table 20 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block-start"></a>border-block-start, <a id="propdef-border-block-end"></a>border-block-end, <a id="propdef-border-inline-start"></a>border-inline-start, <a id="propdef-border-inline-end"></a>border-inline-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-color"></a>

<a id="ref-for-propdef-border-top-style④"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-propdef-border-top-width④"></a>

[\<'border-top-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'border-top-style'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-style) <a id="ref-for-comb-any①"></a>\|\| [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [logical-box-border-shorthands.html](https://wpt.fyi/results/css/css-logical/logical-box-border-shorthands.html) [(live test)](http://wpt.live/css/css-logical/logical-box-border-shorthands.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-border-shorthands.html)
- [border-block-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-block-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-block-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-block-valid.html)

<a id="ref-for-propdef-border-top"></a>

<a id="ref-for-propdef-border-bottom"></a>

<a id="ref-for-propdef-border-left"></a>

<a id="ref-for-propdef-border-right"></a>

<a id="ref-for-propdef-writing-mode①④"></a>

<a id="ref-for-propdef-direction①②"></a>

<a id="ref-for-propdef-text-orientation①⓪"></a>

These properties correspond to the [border-top](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top), [border-bottom](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom), [border-left](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left), and [border-right](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).

<strong>Table 21 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-block"></a>border-block, <a id="propdef-border-inline"></a>border-inline

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-block-start①"></a>

[\<'border-block-start'\>](#propdef-border-block-start)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [border-inline-valid.html](https://wpt.fyi/results/css/css-logical/parsing/border-inline-valid.html) [(live test)](http://wpt.live/css/css-logical/parsing/border-inline-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/parsing/border-inline-valid.html)

<a id="ref-for-shorthand-property⑨"></a>

<a id="ref-for-propdef-border-block-start②"></a>

<a id="ref-for-propdef-border-block-end①"></a>

<a id="ref-for-propdef-border-inline-start①"></a>

<a id="ref-for-propdef-border-inline-end①"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start](#propdef-border-block-start) &#x26; [border-block-end](#propdef-border-block-end) or [border-inline-start](#propdef-border-inline-start) &#x26; [border-inline-end](#propdef-border-inline-end), respectively, both to the same style.

<a id="ref-for-propdef-border-start-start-radius"></a>

<a id="ref-for-propdef-border-start-end-radius"></a>

<a id="ref-for-propdef-border-end-start-radius"></a>

<a id="ref-for-propdef-border-end-end-radius"></a>

### <a id="border-radius-properties"></a>4.6.  Flow-Relative Corner Rounding: the [border-start-start-radius](#propdef-border-start-start-radius), [border-start-end-radius](#propdef-border-start-end-radius), [border-end-start-radius](#propdef-border-end-start-radius), [border-end-end-radius](#propdef-border-end-end-radius) properties

<strong>Table 22 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-border-start-start-radius"></a>border-start-start-radius, <a id="propdef-border-start-end-radius"></a>border-start-end-radius, <a id="propdef-border-end-start-radius"></a>border-end-start-radius, <a id="propdef-border-end-end-radius"></a>border-end-end-radius

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-left-radius"></a>

[\<'border-top-left-radius'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-left-radius)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-left-radius①"></a>

Same as [border-top-left-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-left-radius)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-left-radius②"></a>

Same as [border-top-left-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-left-radius)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-top-left-radius③"></a>

Same as [border-top-left-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-left-radius)

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

Same as corresponding physical border-\*-radius properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

[Logical property group:](https://drafts.csswg.org/css-logical-1/#logical-property-group)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-border-radius"></a>

[border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius)

Tests

- [logical-box-border-radius.html](https://wpt.fyi/results/css/css-logical/logical-box-border-radius.html) [(live test)](http://wpt.live/css/css-logical/logical-box-border-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-box-border-radius.html)

<a id="ref-for-propdef-border-top-left-radius④"></a>

<a id="ref-for-propdef-border-bottom-left-radius"></a>

<a id="ref-for-propdef-border-top-right-radius"></a>

<a id="ref-for-propdef-border-bottom-right-radius"></a>

<a id="ref-for-propdef-writing-mode①⑤"></a>

<a id="ref-for-propdef-direction①③"></a>

<a id="ref-for-propdef-text-orientation①①"></a>

These properties correspond to the [border-top-left-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-left-radius), [border-bottom-left-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-left-radius), [border-top-right-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-right-radius), and [border-bottom-right-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-right-radius) properties. The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation), with the first start/end giving the block axis side, and the second the inline-axis side (i.e. patterned as 'border-<var>block</var>-<var>inline</var>-radius').

<a id="ref-for-propdef-margin④"></a>

<a id="ref-for-propdef-padding①"></a>

<a id="ref-for-propdef-border-width①"></a>

<a id="ref-for-propdef-border-style①"></a>

<a id="ref-for-propdef-border-color①"></a>

### <a id="logical-shorthand-keyword"></a>4.7.  Four-Directional Shorthand Properties: the [margin](https://www.w3.org/TR/css-box-4/#propdef-margin), [padding](https://www.w3.org/TR/css-box-4/#propdef-padding), [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width), [border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style), and [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color) shorthands

The shorthand properties for margin, padding, and border set values for physical properties by default. But authors can specify the <a id="valdef-margin-logical"></a>logical keyword at the beginning of the property value to indicate that the values map to the flow-relative properties instead of the physical ones.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-cfb9f389"></a> The proposed syntax for this feature is [under discussion](https://github.com/w3c/csswg-drafts/issues/1282) and is almost guaranteed to change from what is described here. This section remains in the draft to promote discussion of alternatives, to document the affected properties, and to specify the expected impact on the interpretation of whatever syntactic switch is ultimately chosen.

<a id="ref-for-valdef-margin-logical"></a>

The following [\[CSS2\]](#biblio-css2) shorthand properties accept the [logical](#valdef-margin-logical) keyword:

- <a id="ref-for-propdef-inset③"></a>

  [inset](#propdef-inset)

- <a id="ref-for-propdef-margin⑤"></a>

  [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)

- <a id="ref-for-propdef-padding②"></a>

  [padding](https://www.w3.org/TR/css-box-4/#propdef-padding)

- <a id="ref-for-propdef-border-width②"></a>

  [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)

- <a id="ref-for-propdef-border-style②"></a>

  [border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style)

- <a id="ref-for-propdef-border-color②"></a>

  [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color)

- <a id="ref-for-propdef-scroll-padding"></a>

  [scroll-padding](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-padding)

- <a id="ref-for-propdef-scroll-margin"></a>

  [scroll-margin](https://www.w3.org/TR/css-scroll-snap-1/#propdef-scroll-margin)

The syntax for these properties is effectively changed by replacing

<a id="ref-for-mult-num-range⑦"></a>

```text
<value-type>{1,4}
```
with

<a id="ref-for-mult-opt"></a>

<a id="ref-for-mult-num-range⑧"></a>

```text
logical? <value-type>{1,4}
```
<a id="ref-for-valdef-margin-logical①"></a>

When the [logical](#valdef-margin-logical) keyword is present in the value, the values that follow are assigned to its flow-relative longhands as follows:

- <a id="ref-for-longhand②"></a>

  If only one value is set, the value applies to all four flow-relative [longhands](https://www.w3.org/TR/css-cascade-5/#longhand).

- If two values are set, the first is for block-start and block-end, the second is for inline-start and inline-end.

- If three values are set, the first is for block-start, the second is for inline-start and inline-end, and the third is for block-end.

- If four values are set, they apply to the block-start, inline-start, block-end, and inline-end sides in that order.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-119016c9"></a> In the following example, the two rules are equivalent:
>
> ```css
> blockquote {
>   margin: logical 1em 2em 3em 4em;
> }
> blockquote {
>   margin-block-start:  1em;
>   margin-inline-start: 2em;
>   margin-block-end:    3em;
>   margin-inline-end:   4em;
> }
> ```
## <a id="acknowledgements"></a>5. Acknowledgements

Cameron McCormack, David Baron, Oriol Brufau, Shinyu Murakami, Tab Atkins

## <a id="changes"></a>6.  Changes

Changes since the [27 August 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-logical-1-20180827/) include:

- Defined inheritance of flow-relative properties.

- <a id="ref-for-propdef-resize"></a>

  Moved flow-relative values of [resize](https://www.w3.org/TR/css-ui-4/#propdef-resize) to [\[CSS-UI-4\]](#biblio-css-ui-4).

- Editorially rewrote large parts of the specification for clarity.

- <a id="ref-for-propdef-text-orientation①②"></a>

  Added [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) to list of mapping dependencies.

- <a id="ref-for-propdef-border-block①"></a>

  <a id="ref-for-propdef-border-inline①"></a>

  Clarified that [border-inline](#propdef-border-inline) and [border-block](#propdef-border-block) set both affected sides to the same value.

- Clarified the mapping for border-radius.

- Referred to physical margin, border, and padding properties in the elements they apply to, so ruby base containers and ruby annotation containers are excluded.

- Made margin and padding properties refer to \*-top properties for consistency.

- <a id="ref-for-inset-properties"></a>

  Defined the term [inset properties](#inset-properties).

- Miscellaneous minor clarifications.

- Added Web Platform Tests coverage.

Changes between the earlier Editors Drafts and the [18 May 2017 First Public Working Draft](https://www.w3.org/TR/2017/WD-css-logical-1-20170518/) include:

- <a id="ref-for-writing-mode⑧"></a>

  Making all properties cascade using the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) specified on the element, not on its parent.

- <a id="ref-for-propdef-margin⑥"></a>

  Making the ordering of longhands within [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)-like shorthands put inline-start before inline-end.

- Adding the \*-inline and \*-block shorthand forms for margins/borders/padding.

- Renaming the offset-\* properties to inset-\* and marking an issue for discussion.

- Adding an Introduction section.

- Updating to current terminology of CSS Writing Modes.

- Miscellaneous prose cleanup.

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

Tests

Features from other specs

- [logical-values-resize.html](https://wpt.fyi/results/css/css-logical/logical-values-resize.html) [(live test)](http://wpt.live/css/css-logical/logical-values-resize.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-logical/logical-values-resize.html)

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

- [block-size](#propdef-block-size), in § 4.1
- [border-block](#propdef-border-block), in § 4.5.4
- [border-block-color](#propdef-border-block-color), in § 4.5.3
- [border-block-end](#propdef-border-block-end), in § 4.5.4
- [border-block-end-color](#propdef-border-block-end-color), in § 4.5.3
- [border-block-end-style](#propdef-border-block-end-style), in § 4.5.2
- [border-block-end-width](#propdef-border-block-end-width), in § 4.5.1
- [border-block-start](#propdef-border-block-start), in § 4.5.4
- [border-block-start-color](#propdef-border-block-start-color), in § 4.5.3
- [border-block-start-style](#propdef-border-block-start-style), in § 4.5.2
- [border-block-start-width](#propdef-border-block-start-width), in § 4.5.1
- [border-block-style](#propdef-border-block-style), in § 4.5.2
- [border-block-width](#propdef-border-block-width), in § 4.5.1
- [border-end-end-radius](#propdef-border-end-end-radius), in § 4.6
- [border-end-start-radius](#propdef-border-end-start-radius), in § 4.6
- [border-inline](#propdef-border-inline), in § 4.5.4
- [border-inline-color](#propdef-border-inline-color), in § 4.5.3
- [border-inline-end](#propdef-border-inline-end), in § 4.5.4
- [border-inline-end-color](#propdef-border-inline-end-color), in § 4.5.3
- [border-inline-end-style](#propdef-border-inline-end-style), in § 4.5.2
- [border-inline-end-width](#propdef-border-inline-end-width), in § 4.5.1
- [border-inline-start](#propdef-border-inline-start), in § 4.5.4
- [border-inline-start-color](#propdef-border-inline-start-color), in § 4.5.3
- [border-inline-start-style](#propdef-border-inline-start-style), in § 4.5.2
- [border-inline-start-width](#propdef-border-inline-start-width), in § 4.5.1
- [border-inline-style](#propdef-border-inline-style), in § 4.5.2
- [border-inline-width](#propdef-border-inline-width), in § 4.5.1
- [border-start-end-radius](#propdef-border-start-end-radius), in § 4.6
- [border-start-start-radius](#propdef-border-start-start-radius), in § 4.6
- [directional keyword](#directional-keyword), in § 2
- [inline-size](#propdef-inline-size), in § 4.1
- [inset](#propdef-inset), in § 4.3
- [inset-block](#propdef-inset-block), in § 4.3
- [inset-block-end](#propdef-inset-block-end), in § 4.3
- [inset-block-start](#propdef-inset-block-start), in § 4.3
- [inset-inline](#propdef-inset-inline), in § 4.3
- [inset-inline-end](#propdef-inset-inline-end), in § 4.3
- [inset-inline-start](#propdef-inset-inline-start), in § 4.3
- [inset properties](#inset-properties), in § 4.3
- [logical](#valdef-margin-logical), in § 4.7
- [logical property group](#logical-property-group), in § 4
- [mapping logic](#mapping-logic), in § 4
- [margin-block](#propdef-margin-block), in § 4.2
- [margin-block-end](#propdef-margin-block-end), in § 4.2
- [margin-block-start](#propdef-margin-block-start), in § 4.2
- [margin-inline](#propdef-margin-inline), in § 4.2
- [margin-inline-end](#propdef-margin-inline-end), in § 4.2
- [margin-inline-start](#propdef-margin-inline-start), in § 4.2
- [max-block-size](#propdef-max-block-size), in § 4.1
- [max-inline-size](#propdef-max-inline-size), in § 4.1
- [min-block-size](#propdef-min-block-size), in § 4.1
- [min-inline-size](#propdef-min-inline-size), in § 4.1
- [padding-block](#propdef-padding-block), in § 4.4
- [padding-block-end](#propdef-padding-block-end), in § 4.4
- [padding-block-start](#propdef-padding-block-start), in § 4.4
- [padding-inline](#propdef-padding-inline), in § 4.4
- [padding-inline-end](#propdef-padding-inline-end), in § 4.4
- [padding-inline-start](#propdef-padding-inline-start), in § 4.4
- [:recto](#valdef-logical-page-selector-recto), in § 3
- [recto](#valdef-logical-page-recto), in § 3
- [:verso](#valdef-logical-page-selector-verso), in § 3
- [verso](#valdef-logical-page-verso), in § 3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="f077c73e"></a>border-bottom
  - <a id="3bf45619"></a>border-bottom-color
  - <a id="f003c574"></a>border-bottom-left-radius
  - <a id="d6b02eef"></a>border-bottom-right-radius
  - <a id="429a88c6"></a>border-bottom-style
  - <a id="bb899432"></a>border-bottom-width
  - <a id="65b3a7bc"></a>border-color
  - <a id="a1ccbe05"></a>border-left
  - <a id="e2ddaaa8"></a>border-left-color
  - <a id="4bbbeaad"></a>border-left-style
  - <a id="aaf0980c"></a>border-left-width
  - <a id="3a4a9318"></a>border-radius
  - <a id="e851603c"></a>border-right
  - <a id="4e28efd4"></a>border-right-color
  - <a id="f6755377"></a>border-right-style
  - <a id="47e9abf9"></a>border-right-width
  - <a id="b6bb4b13"></a>border-style
  - <a id="87179886"></a>border-top
  - <a id="d1764055"></a>border-top-color
  - <a id="1e11542f"></a>border-top-left-radius
  - <a id="57d168af"></a>border-top-right-radius
  - <a id="092642af"></a>border-top-style
  - <a id="051410c7"></a>border-top-width
  - <a id="064303ba"></a>border-width
- \[CSS-BOX-4\] defines the following terms:
  - <a id="253362bb"></a>margin
  - <a id="ba64c9f5"></a>margin-bottom
  - <a id="836161df"></a>margin-left
  - <a id="76c02e00"></a>margin-right
  - <a id="58404105"></a>margin-top
  - <a id="a3a070bd"></a>padding
  - <a id="c8473aa4"></a>padding-bottom
  - <a id="c22cb630"></a>padding-left
  - <a id="0a9e7084"></a>padding-right
  - <a id="f72f0a02"></a>padding-top
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="d3b48763"></a>all
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="36261173"></a>longhand
  - <a id="8f27be0f"></a>longhand property
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="d5e08d9c"></a>specified value
  - <a id="b49aeda5"></a>sub-property
  - <a id="1a2b1083"></a>used value
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="aa433d97"></a>cascade
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="0923db9e"></a>containing block
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="f411d42d"></a>bottom
  - <a id="ebcbc56d"></a>left
  - <a id="a5bae6ee"></a>right
  - <a id="f99d4ae2"></a>top
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="f205ddd4"></a>scroll-margin
  - <a id="2f6cb60c"></a>scroll-padding
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="49731d1d"></a>width
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="1b178ec1"></a>table wrapper box
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="36e5f32e"></a>text-align
- \[CSS-UI-4\] defines the following terms:
  - <a id="c034f069"></a>resize
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="3bafef5e"></a>{A,B}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="ac7161d9"></a>end
  - <a id="04e5ac3a"></a>start
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="303c8d41"></a>flow-relative
  - <a id="8ebdd273"></a>horizontal-tb
  - <a id="0da67e16"></a>inline-start
  - <a id="4e782d0d"></a>line-relative
  - <a id="24c75626"></a>ltr
  - <a id="e1f6e4b9"></a>physical
  - <a id="dea08a34"></a>rtl
  - <a id="8664e85f"></a>text-orientation
  - <a id="eb6008ce"></a>writing mode
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="d66e759b"></a>caption-side
  - <a id="019a586e"></a>clear
  - <a id="da486c10"></a>float
  - <a id="0f0ab49f"></a>max-height
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
  - <a id="96f69bf6"></a>page-break-after
  - <a id="8781a4dc"></a>page-break-before

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-3"></a>\[CSS-CASCADE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 6 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 18 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-scroll-snap-1"></a>\[CSS-SCROLL-SNAP-1\]  
Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 11 March 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-1&#x2F;](https://www.w3.org/TR/css-scroll-snap-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
François Remy; Greg Whitworth; David Baron. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 27 July 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 30 September 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

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

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

## <a id="property-index"></a>Property Index

<strong>Table 23 — structured row/cell transcription</strong>

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

<strong>Column 10 (header cell; scope col):</strong>

Logical property group

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-block-size①"></a>

[block-size](#propdef-block-size)

<strong>Column 2 (data cell):</strong>

\<'width'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

Same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as height, width

<strong>Column 10 (data cell):</strong>

size

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block②"></a>

[border-block](#propdef-border-block)

<strong>Column 2 (data cell):</strong>

\<'border-block-start'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-color①"></a>

[border-block-color](#propdef-border-block-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-end②"></a>

[border-block-end](#propdef-border-block-end)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\> \|\| \<'border-top-style'\> \|\| \<color\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-end-color②"></a>

[border-block-end-color](#propdef-border-block-end-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>

<strong>Column 3 (data cell):</strong>

currentcolor

<strong>Column 4 (data cell):</strong>

Same as border-top-color

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-color properties

<strong>Column 10 (data cell):</strong>

border-color

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-end-style②"></a>

[border-block-end-style](#propdef-border-block-end-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

Same as border-top-style

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-style properties

<strong>Column 10 (data cell):</strong>

border-style

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-end-width②"></a>

[border-block-end-width](#propdef-border-block-end-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>

<strong>Column 3 (data cell):</strong>

medium

<strong>Column 4 (data cell):</strong>

Same as border-top-width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-width properties

<strong>Column 10 (data cell):</strong>

border-width

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-start③"></a>

[border-block-start](#propdef-border-block-start)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\> \|\| \<'border-top-style'\> \|\| \<color\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-start-color②"></a>

[border-block-start-color](#propdef-border-block-start-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>

<strong>Column 3 (data cell):</strong>

currentcolor

<strong>Column 4 (data cell):</strong>

Same as border-top-color

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-color properties

<strong>Column 10 (data cell):</strong>

border-color

<strong>Row 11</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-start-style②"></a>

[border-block-start-style](#propdef-border-block-start-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

Same as border-top-style

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-style properties

<strong>Column 10 (data cell):</strong>

border-style

<strong>Row 12</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-start-width②"></a>

[border-block-start-width](#propdef-border-block-start-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>

<strong>Column 3 (data cell):</strong>

medium

<strong>Column 4 (data cell):</strong>

Same as border-top-width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-width properties

<strong>Column 10 (data cell):</strong>

border-width

<strong>Row 13</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-style①"></a>

[border-block-style](#propdef-border-block-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 14</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-block-width①"></a>

[border-block-width](#propdef-border-block-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 15</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-end-end-radius①"></a>

[border-end-end-radius](#propdef-border-end-end-radius)

<strong>Column 2 (data cell):</strong>

\<'border-top-left-radius'\>

<strong>Column 3 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 4 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding physical border-\*-radius properties

<strong>Column 10 (data cell):</strong>

border-radius

<strong>Row 16</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-end-start-radius①"></a>

[border-end-start-radius](#propdef-border-end-start-radius)

<strong>Column 2 (data cell):</strong>

\<'border-top-left-radius'\>

<strong>Column 3 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 4 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding physical border-\*-radius properties

<strong>Column 10 (data cell):</strong>

border-radius

<strong>Row 17</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline②"></a>

[border-inline](#propdef-border-inline)

<strong>Column 2 (data cell):</strong>

\<'border-block-start'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 18</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-color①"></a>

[border-inline-color](#propdef-border-inline-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 19</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-end②"></a>

[border-inline-end](#propdef-border-inline-end)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\> \|\| \<'border-top-style'\> \|\| \<color\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 20</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-end-color②"></a>

[border-inline-end-color](#propdef-border-inline-end-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>

<strong>Column 3 (data cell):</strong>

currentcolor

<strong>Column 4 (data cell):</strong>

Same as border-top-color

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-color properties

<strong>Column 10 (data cell):</strong>

border-color

<strong>Row 21</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-end-style②"></a>

[border-inline-end-style](#propdef-border-inline-end-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

Same as border-top-style

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-style properties

<strong>Column 10 (data cell):</strong>

border-style

<strong>Row 22</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-end-width②"></a>

[border-inline-end-width](#propdef-border-inline-end-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>

<strong>Column 3 (data cell):</strong>

medium

<strong>Column 4 (data cell):</strong>

Same as border-top-width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-width properties

<strong>Column 10 (data cell):</strong>

border-width

<strong>Row 23</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-start②"></a>

[border-inline-start](#propdef-border-inline-start)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\> \|\| \<'border-top-style'\> \|\| \<color\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 24</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-start-color②"></a>

[border-inline-start-color](#propdef-border-inline-start-color)

<strong>Column 2 (data cell):</strong>

\<'border-top-color'\>

<strong>Column 3 (data cell):</strong>

currentcolor

<strong>Column 4 (data cell):</strong>

Same as border-top-color

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-color properties

<strong>Column 10 (data cell):</strong>

border-color

<strong>Row 25</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-start-style②"></a>

[border-inline-start-style](#propdef-border-inline-start-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

Same as border-top-style

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-style properties

<strong>Column 10 (data cell):</strong>

border-style

<strong>Row 26</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-start-width②"></a>

[border-inline-start-width](#propdef-border-inline-start-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>

<strong>Column 3 (data cell):</strong>

medium

<strong>Column 4 (data cell):</strong>

Same as border-top-width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding border-\*-width properties

<strong>Column 10 (data cell):</strong>

border-width

<strong>Row 27</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-style①"></a>

[border-inline-style](#propdef-border-inline-style)

<strong>Column 2 (data cell):</strong>

\<'border-top-style'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 28</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-inline-width①"></a>

[border-inline-width](#propdef-border-inline-width)

<strong>Column 2 (data cell):</strong>

\<'border-top-width'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 29</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-start-end-radius①"></a>

[border-start-end-radius](#propdef-border-start-end-radius)

<strong>Column 2 (data cell):</strong>

\<'border-top-left-radius'\>

<strong>Column 3 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 4 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding physical border-\*-radius properties

<strong>Column 10 (data cell):</strong>

border-radius

<strong>Row 30</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-border-start-start-radius①"></a>

[border-start-start-radius](#propdef-border-start-start-radius)

<strong>Column 2 (data cell):</strong>

\<'border-top-left-radius'\>

<strong>Column 3 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 4 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

Same as border-top-left-radius

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding physical border-\*-radius properties

<strong>Column 10 (data cell):</strong>

border-radius

<strong>Row 31</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inline-size①"></a>

[inline-size](#propdef-inline-size)

<strong>Column 2 (data cell):</strong>

\<'width'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

Same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as height, width

<strong>Column 10 (data cell):</strong>

size

<strong>Row 32</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset④"></a>

[inset](#propdef-inset)

<strong>Column 2 (data cell):</strong>

\<'top'\>{1,4}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 33</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-block②"></a>

[inset-block](#propdef-inset-block)

<strong>Column 2 (data cell):</strong>

\<'top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 34</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-block-end③"></a>

[inset-block-end](#propdef-inset-block-end)

<strong>Column 2 (data cell):</strong>

\<'top'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

positioned elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding top/right/bottom/left properties

<strong>Column 10 (data cell):</strong>

inset

<strong>Row 35</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-block-start③"></a>

[inset-block-start](#propdef-inset-block-start)

<strong>Column 2 (data cell):</strong>

\<'top'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

positioned elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding top/right/bottom/left properties

<strong>Column 10 (data cell):</strong>

inset

<strong>Row 36</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-inline②"></a>

[inset-inline](#propdef-inset-inline)

<strong>Column 2 (data cell):</strong>

\<'top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 37</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-inline-end③"></a>

[inset-inline-end](#propdef-inset-inline-end)

<strong>Column 2 (data cell):</strong>

\<'top'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

positioned elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding top/right/bottom/left properties

<strong>Column 10 (data cell):</strong>

inset

<strong>Row 38</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-inset-inline-start③"></a>

[inset-inline-start](#propdef-inset-inline-start)

<strong>Column 2 (data cell):</strong>

\<'top'\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

positioned elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding top/right/bottom/left properties

<strong>Column 10 (data cell):</strong>

inset

<strong>Row 39</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-block①"></a>

[margin-block](#propdef-margin-block)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 40</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-block-end②"></a>

[margin-block-end](#propdef-margin-block-end)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as margin-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding margin-\* properties

<strong>Column 10 (data cell):</strong>

margin

<strong>Row 41</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-block-start②"></a>

[margin-block-start](#propdef-margin-block-start)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as margin-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding margin-\* properties

<strong>Column 10 (data cell):</strong>

margin

<strong>Row 42</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-inline①"></a>

[margin-inline](#propdef-margin-inline)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 43</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-inline-end④"></a>

[margin-inline-end](#propdef-margin-inline-end)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as margin-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding margin-\* properties

<strong>Column 10 (data cell):</strong>

margin

<strong>Row 44</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-margin-inline-start⑥"></a>

[margin-inline-start](#propdef-margin-inline-start)

<strong>Column 2 (data cell):</strong>

\<'margin-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as margin-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding margin-\* properties

<strong>Column 10 (data cell):</strong>

margin

<strong>Row 45</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-max-block-size①"></a>

[max-block-size](#propdef-max-block-size)

<strong>Column 2 (data cell):</strong>

\<'max-width'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as max-height, max-width

<strong>Column 10 (data cell):</strong>

max-size

<strong>Row 46</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-max-inline-size①"></a>

[max-inline-size](#propdef-max-inline-size)

<strong>Column 2 (data cell):</strong>

\<'max-width'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as max-height, max-width

<strong>Column 10 (data cell):</strong>

max-size

<strong>Row 47</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-min-block-size①"></a>

[min-block-size](#propdef-min-block-size)

<strong>Column 2 (data cell):</strong>

\<'min-width'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as min-height, min-width

<strong>Column 10 (data cell):</strong>

min-size

<strong>Row 48</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-min-inline-size①"></a>

[min-inline-size](#propdef-min-inline-size)

<strong>Column 2 (data cell):</strong>

\<'min-width'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

same as height and width

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as min-height, min-width

<strong>Column 10 (data cell):</strong>

min-size

<strong>Row 49</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-block①"></a>

[padding-block](#propdef-padding-block)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 50</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-block-end②"></a>

[padding-block-end](#propdef-padding-block-end)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as padding-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding padding-\* properties

<strong>Column 10 (data cell):</strong>

padding

<strong>Row 51</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-block-start②"></a>

[padding-block-start](#propdef-padding-block-start)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as padding-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding padding-\* properties

<strong>Column 10 (data cell):</strong>

padding

<strong>Row 52</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-inline①"></a>

[padding-inline](#propdef-padding-inline)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>{1,2}

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Column 10 (data cell):</strong>

<strong>Row 53</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-inline-end②"></a>

[padding-inline-end](#propdef-padding-inline-end)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as padding-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding padding-\* properties

<strong>Column 10 (data cell):</strong>

padding

<strong>Row 54</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-padding-inline-start②"></a>

[padding-inline-start](#propdef-padding-inline-start)

<strong>Column 2 (data cell):</strong>

\<'padding-top'\>

<strong>Column 3 (data cell):</strong>

0

<strong>Column 4 (data cell):</strong>

Same as padding-top

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

As for the corresponding physical property

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

Same as corresponding padding-\* properties

<strong>Column 10 (data cell):</strong>

padding

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <strong>Things That Are Unstable</strong> Since implementation of parts of this module is effectively required for shipping an implementation of [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/) on the Web (in order to correctly implement the default HTML styles), the CSSWG resolved that the requisite features in [§ 2 Flow-Relative Values: block-start, block-end, inline-start, inline-end](#directional-keywords) and [§ 4 Flow-Relative Box Model Properties](#box) are approved for shipping. (See [FPWD announcement](https://lists.w3.org/Archives/Public/www-style/2017Dec/0043.html) for additional background.)
>
> However, there are a few significant open issues:
>
> - The logical keyword on shorthands, because the name of the keyword may change or it may be replaced by some other syntactic marker. (This feature will be deferred from this level for further development if there is no clearly satisfactory mechanism proposed, see [Issue 1282](https://github.com/w3c/csswg-drafts/issues/1282).)
> - Whether flow-relative longhands inherit from their namesake on the parent, or are mapped to a physical property and inherit from that property. (See [Issue 3029](https://github.com/w3c/csswg-drafts/issues/3029).)
> - Whether shorthands like [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) expand to both sets of longhands, or only the ones that were set. (See [Issue 3030](https://github.com/w3c/csswg-drafts/issues/3030).)
>
> Comments, suggestions, and use cases are welcome on these issues. Please file them in GitHub, tweet them to @csswg, or send them to www-style@w3.org. [↵](#issue-3d880eb1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The proposed syntax for this feature is [under discussion](https://github.com/w3c/csswg-drafts/issues/1282) and is almost guaranteed to change from what is described here. This section remains in the draft to promote discussion of alternatives, to document the affected properties, and to specify the expected impact on the interpretation of whatever syntactic switch is ultimately chosen. [↵](#issue-cfb9f389)
