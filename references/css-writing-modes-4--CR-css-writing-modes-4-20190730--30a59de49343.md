Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Writing Modes Level 4](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/).

Original copyright notice: Copyright © 2019 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Writing Modes Level 4

Source snapshot: https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/

Snapshot SHA-256: 30a59de49343d984c8570e2c1a5fa5bda511dda4f2e83841c862caa99d155fcd

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 14 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Writing Modes Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2019 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

CSS Writing Modes Level 4 defines CSS support for various writing modes and their combinations, including left-to-right and right-to-left text ordering as well as horizontal and vertical orientations.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
		Other documents may supersede this document.
		A list of current W3C publications and the latest revision of this technical report
		can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) as a Candidate Recommendation. This document is intended to become a W3C Recommendation. This document will remain a Candidate Recommendation at least until 28 August 2019 in order to ensure the opportunity for wide review.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “css-writing-modes” in the title, preferably like this: “\[css-writing-modes\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

A draft implementation report is not yet available.

Publication as a Candidate Recommendation does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 March 2019 W3C Process Document](https://www.w3.org/2019/Process-20190301/).

For changes since the last draft, see the [Changes](#changes) section.

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-text-combine-upright"></a>

  The look-ahead/look-behind sequencing rules for [text-combine-upright](#propdef-text-combine-upright).

- [Automatic multi-column behavior](#auto-multicol) of orthogonal flows.

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="text-flow"></a>1.  Introduction to Writing Modes

CSS Writing Modes Level 4 defines CSS features to support for various international writing modes, such as left-to-right (e.g. Latin or Indic), right-to-left (e.g. Hebrew or Arabic), bidirectional (e.g. mixed Latin and Arabic) and vertical (e.g. Asian scripts).

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-text-orientation"></a>

<a id="ref-for-inline-base-direction"></a>

<a id="ref-for-block-flow-direction"></a>

A <a id="writing-mode"></a>writing mode in CSS is determined by the [writing-mode](#propdef-writing-mode), [direction](#propdef-direction), and [text-orientation](#propdef-text-orientation) properties. It is defined primarily in terms of its [inline base direction](#inline-base-direction) and [block flow direction](#block-flow-direction):

[![Latin-based writing mode](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-tb.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-tb.svg)

Latin-based writing mode

[![Mongolian-based writing mode](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-lr-reverse.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-lr-reverse.svg)

Mongolian-based writing mode

[![Han-based writing mode](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-tb.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-tb.svg) [![Han-based writing mode](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-rl.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-rl.svg)

Han-based writing mode

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-unicode-bidi"></a>

The <a id="inline-base-direction"></a>inline base direction is the primary direction in which content is ordered on a line and defines on which sides the “start” and “end” of a line are. The [direction](#propdef-direction) property specifies the inline base direction of a box and, together with the [unicode-bidi](#propdef-unicode-bidi) property and the inherent directionality of any text content, determines the ordering of inline-level content within a line.

<a id="ref-for-propdef-writing-mode①"></a>

The <a id="block-flow-direction"></a>block flow direction is the direction in which block-level boxes stack and the direction in which line boxes stack within a block container. The [writing-mode](#propdef-writing-mode) property determines the block flow direction.

<a id="ref-for-vertical-script"></a>

The <a id="typographic-mode"></a>typographic mode determines if text should apply typographic conventions specific to vertical flow for [vertical scripts](#vertical-script). This concept distinguishes vertical flow for <a id="ref-for-vertical-script①"></a>vertical scripts from rotated horizontal flow.

A <a id="horizontal-writing-mode"></a>horizontal writing mode is one with horizontal lines of text, i.e. a downward or upward block flow. A <a id="vertical-writing-mode"></a>vertical writing mode is one with vertical lines of text, i.e. a leftward or rightward block flow.

> <strong data-conversion-semantic="note">Note</strong>
>
> These terms should not be confused with <a id="vertical-block-flow"></a>vertical block flow (which is a downward or upward block flow) and <a id="horizontal-block-flow"></a>horizontal block flow (which is leftward or rightward block flow). To avoid confusion, CSS specifications avoid this latter set of terms.

Writing systems typically have one or two native writing modes. Some examples are:

- Latin-based systems are typically written using a left-to-right inline direction with a downward (top-to-bottom) block flow direction.
- Arabic-based systems are typically written using a right-to-left inline direction with a downward (top-to-bottom) block flow direction.
- Mongolian-based systems are typically written using a top-to-bottom inline direction with a rightward (left-to-right) block flow direction.
- Han-based systems are commonly written using a left-to-right inline direction with a downward (top-to-bottom) block flow direction, <strong>or</strong> a top-to-bottom inline direction with a leftward (right-to-left) block flow direction. Many magazines and newspapers will mix these two writing modes on the same page.

<a id="ref-for-propdef-text-orientation①"></a>

The [text-orientation](#propdef-text-orientation) component of the writing mode controls the glyph orientation.

> <strong data-conversion-semantic="note">Note</strong>
>
> See Unicode Technical Note \#22 [\[UTN22\]](#biblio-utn22) ([HTML version](http://fantasai.inkedblade.net/style/discuss/vertical-text/paper)) for a more in-depth introduction to writing modes and vertical text.

### <a id="placement"></a>1.1.  Module Interactions

<a id="ref-for-propdef-unicode-bidi①"></a>

<a id="ref-for-propdef-direction②"></a>

This module replaces and extends the [unicode-bidi](#propdef-unicode-bidi) and [direction](#propdef-direction) features defined in [\[CSS2\]](#biblio-css2) sections 8.6 and 9.10. The interaction of its features with other text operations in setting lines of text is described in [CSS Text 3 § Text Processing Order of Operations](https://www.w3.org/TR/css-text-3/#order).

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-writing-mode②"></a>

<a id="ref-for-propdef-direction③"></a>

<a id="ref-for-propdef-text-orientation②"></a>

<a id="ref-for-font-relative-length"></a>

<a id="ref-for-writing-mode"></a>

The [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value) of the [writing-mode](#propdef-writing-mode), [direction](#propdef-direction), and [text-orientation](#propdef-text-orientation) properties (even on elements to which these properties themselves don’t apply [\[CSS-CASCADE-4\]](#biblio-css-cascade-4)) are broadly able to influence the computed values of other, unrelated properties through calculations such as the computation of [font-relative lengths](https://www.w3.org/TR/css-values-4/#font-relative-length) or the cascade of [flow-relative properties](https://www.w3.org/TR/css-logical-1/) which purposefully depend on the computed [writing mode](#writing-mode) or on font metrics that can depend on the <a id="ref-for-writing-mode①"></a>writing mode.

### <a id="values"></a>1.2.  Value Types and Terminology

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) keywords as their property value. For readability they have not been repeated explicitly.

Other important terminology and concepts used in this specification are defined in [\[CSS2\]](#biblio-css2) and [\[CSS-TEXT-3\]](#biblio-css-text-3).

<a id="bidi"></a>

## <a id="text-direction"></a>2.  Inline Direction and Bidirectionality

While the characters in most scripts are written from left to right, certain scripts are written from right to left. In some documents, in particular those written with the Arabic or Hebrew script, and in some mixed-language contexts, text in a single (visually displayed) block may appear with mixed directionality. This phenomenon is called <a id="bidirectionality"></a>bidirectionality, or "bidi" for short.

![An example of bidirectional text is a Latin name in an Arabic sentence. The sentence overall is typeset right-to-left, but the letters in the Latin word in the middle are typeset left-to-right.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/bidi.png)

Bidirectionality

The Unicode standard ([Unicode Standard Annex \#9](http://www.unicode.org/reports/tr9/)) defines a complex algorithm for determining the proper ordering of bidirectional text. The algorithm consists of an implicit part based on character properties, as well as explicit controls for embeddings and overrides. CSS relies on this algorithm to achieve proper bidirectional rendering.

<a id="ref-for-propdef-direction④"></a>

<a id="ref-for-propdef-unicode-bidi②"></a>

Two CSS properties, [direction](#propdef-direction) and [unicode-bidi](#propdef-unicode-bidi), provide explicit embedding, isolation, and override controls in the CSS layer. Because the base directionality of a text depends on the structure and semantics of the document, the <a id="ref-for-propdef-direction⑤"></a>direction and <a id="ref-for-propdef-unicode-bidi③"></a>unicode-bidi properties should in most cases be used only to map bidi information in the markup to its corresponding CSS styles.

> <strong data-conversion-semantic="note">Note</strong>
>
> The HTML specifications ([\[HTML401\]](#biblio-html401), section 8.2, and [\[HTML5\]](#biblio-html5), section 10.3.5) define bidirectionality behavior for HTML elements.

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> <strong>If a document language provides markup features to control
    bidi, authors and users should use those features instead</strong> and not specify CSS rules to override them.

<a id="ref-for-propdef-direction⑥"></a>

### <a id="direction"></a>2.1.  Specifying Directionality: the [direction](#propdef-direction) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-direction"></a>direction

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

ltr [\|](https://www.w3.org/TR/css-values-4/#comb-one) rtl

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

ltr

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

[all elements](https://drafts.csswg.org/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-direction⑦"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Because HTML UAs can turn off CSS styling, <strong>we recommend HTML authors to use the HTML <code>dir</code> attribute and &lt;bdo&gt; element</strong> to ensure correct bidirectional layout in the absence of a style sheet. <strong>Authors <em>should not</em> use <a href="#propdef-direction">direction</a> in HTML documents.</strong>

<a id="ref-for-inline-base-direction①"></a>

<a id="ref-for-propdef-unicode-bidi④"></a>

This property specifies the [inline base direction](#inline-base-direction) or directionality of any bidi paragraph, embedding, isolate, or override established by the box. (See [unicode-bidi](#propdef-unicode-bidi).) In addition, it informs the ordering of [table](https://www.w3.org/TR/CSS2/tables.html) column layout, the direction of horizontal [overflow](https://www.w3.org/TR/CSS2/visufx.html#overflow), and the default alignment of text within a line, and other layout effects that depend on the box’s inline base direction.

Values for this property have the following meanings:

<a id="valdef-direction-ltr"></a>ltr  
<a id="ref-for-line-right"></a>

<a id="ref-for-line-left"></a>

<a id="ref-for-inline-base-direction②"></a>

This value sets [inline base direction](#inline-base-direction) (bidi directionality) to [line-left](#line-left)-to-[line-right](#line-right).

<a id="valdef-direction-rtl"></a>rtl  
<a id="ref-for-line-left①"></a>

<a id="ref-for-line-right①"></a>

<a id="ref-for-inline-base-direction③"></a>

This value sets [inline base direction](#inline-base-direction) (bidi directionality) to [line-right](#line-right)-to-[line-left](#line-left).

<a id="ref-for-propdef-direction⑧"></a>

<a id="ref-for-propdef-unicode-bidi⑤"></a>

<a id="ref-for-valdef-unicode-bidi-normal"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [direction](#propdef-direction) property has no effect on bidi reordering when specified on inline boxes whose [unicode-bidi](#propdef-unicode-bidi) value is [normal](#valdef-unicode-bidi-normal), because the box does not open an additional level of embedding with respect to the bidirectional algorithm.

<a id="ref-for-propdef-direction⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [direction](#propdef-direction) property, when specified for table column boxes, is not inherited by cells in the column since columns are not the ancestors of the cells in the document tree. Thus, CSS cannot easily capture the "dir" attribute inheritance rules described in [\[HTML401\]](#biblio-html401), section 11.3.2.1.

<a id="ref-for-propdef-unicode-bidi⑥"></a>

### <a id="unicode-bidi"></a>2.2.  Embeddings and Overrides: the [unicode-bidi](#propdef-unicode-bidi) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-unicode-bidi"></a>unicode-bidi

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) embed <a id="ref-for-comb-one②"></a>\| isolate <a id="ref-for-comb-one③"></a>\| bidi-override <a id="ref-for-comb-one④"></a>\| isolate-override <a id="ref-for-comb-one⑤"></a>\| plaintext

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

all elements, but see prose

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-unicode-bidi⑦"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Because HTML UAs can turn off CSS styling, <strong>we recommend HTML authors to use the HTML <code>dir</code> attribute, &lt;bdo&gt; element,
    and appropriate distinction of text-level vs. grouping-level HTML element types</strong> to ensure correct bidirectional layout in the absence of a style sheet. <strong>Authors <em>should not</em> use <a href="#propdef-unicode-bidi">unicode-bidi</a> in HTML documents.</strong>

<a id="ref-for-propdef-unicode-bidi⑧"></a>

<a id="ref-for-valdef-unicode-bidi-normal①"></a>

Normally (i.e. when [unicode-bidi](#propdef-unicode-bidi) is [normal](#valdef-unicode-bidi-normal)) an inline box is transparent to the unicode bidi algorithm; content is ordered as if the box’s boundaries were not there. Other values of the <a id="ref-for-propdef-unicode-bidi⑨"></a>unicode-bidi property cause inline boxes to create scopes within the algorithm, and to override the intrinsic directionality of text.

<a id="ref-for-propdef-unicode-bidi①⓪"></a>

The following informative table summarizes the box-internal and box-external effects of [unicode-bidi](#propdef-unicode-bidi):

<a id="ref-for-valdef-unicode-bidi-normal②"></a>

<a id="ref-for-propdef-unicode-bidi①①"></a>

<strong>Table 3 — structured row/cell transcription</strong>

Effect of non-[normal](#valdef-unicode-bidi-normal) values of [unicode-bidi](#propdef-unicode-bidi) on inline boxes

<strong>Row 1</strong>

<strong>Column 1 (header cell; row span 2, column span 2):</strong>

<strong>Column 3 (header cell; column span 2, scope rowgroup):</strong>

Outside

<strong>Row 2</strong>

<strong>Column 3 (header cell):</strong>

strong

<strong>Column 4 (header cell):</strong>

neutral

<strong>Row 3</strong>

<strong>Column 1 (header cell; row span 3, scope colgroup):</strong>

Inside

<strong>Column 2 (header cell):</strong>

scoped

<strong>Column 3 (data cell):</strong>

<a id="ref-for-valdef-unicode-bidi-embed"></a>

[embed](#valdef-unicode-bidi-embed)

<strong>Column 4 (data cell):</strong>

<a id="ref-for-valdef-unicode-bidi-isolate"></a>

[isolate](#valdef-unicode-bidi-isolate)

<strong>Row 4</strong>

<strong>Column 2 (header cell):</strong>

override

<strong>Column 3 (data cell):</strong>

<a id="ref-for-valdef-unicode-bidi-bidi-override"></a>

[bidi-override](#valdef-unicode-bidi-bidi-override)

<strong>Column 4 (data cell):</strong>

<a id="ref-for-valdef-unicode-bidi-isolate-override"></a>

[isolate-override](#valdef-unicode-bidi-isolate-override)

<strong>Row 5</strong>

<strong>Column 2 (header cell):</strong>

plaintext

<strong>Column 3 (data cell):</strong>

—

<strong>Column 4 (data cell):</strong>

<a id="ref-for-valdef-unicode-bidi-plaintext"></a>

[plaintext](#valdef-unicode-bidi-plaintext)

Values for this property have the following (normative) meanings:

<a id="valdef-unicode-bidi-normal"></a>normal  
The box does not open an additional level of embedding with respect to the bidirectional algorithm. For inline boxes, implicit reordering works across box boundaries.

<a id="valdef-unicode-bidi-embed"></a>embed  
<a id="ref-for-propdef-direction①⓪"></a>

If the box is inline, this value creates a <a id="directional-embedding"></a>directional embedding by opening an additional level of embedding with respect to the bidirectional algorithm. The direction of this embedding level is given by the [direction](#propdef-direction) property. Inside the box, reordering is done implicitly.

> <strong data-conversion-semantic="note">Note</strong>
>
> This value has no effect on boxes that are not inline.

<a id="valdef-unicode-bidi-isolate"></a>isolate  
<a id="ref-for-forced-paragraph-break"></a>

On an inline box, this <a id="bidi-isolate"></a>bidi-isolates its contents. This is similar to a directional embedding (and increases the embedding level accordingly) except that each sequence of inline-level boxes uninterrupted by any block boundary or [forced paragraph break](#forced-paragraph-break) is treated as an <a id="isolated-sequence"></a>isolated sequence:

- <a id="ref-for-propdef-direction①①"></a>

  the content within the sequence is ordered as if inside an independent paragraph with the base directionality specified by the box’s [direction](#propdef-direction) property.

- for the purpose of bidi resolution in its containing bidi paragraph, the sequence is treated as if it were a single Object Replacement Character (U+FFFC).

In effect, neither is the content inside the box bidi-affected by the content surrounding the box, nor is the content surrounding the box bidi-affected by the content or specified directionality of the box. However, <a id="ref-for-forced-paragraph-break①"></a>forced paragraph breaks within the box still create a corresponding break in the containing paragraph.

> <strong data-conversion-semantic="note">Note</strong>
>
> This value has no effect on boxes that are not inline.

<a id="valdef-unicode-bidi-bidi-override"></a>bidi-override  
<a id="ref-for-propdef-direction①②"></a>

<a id="ref-for-directional-embedding"></a>

This value puts the box’s immediate inline content in a <a id="directional-override"></a>directional override. For an inline, this means that the box acts like a [directional embedding](#directional-embedding) in the bidirectional algorithm, except that reordering within it is strictly in sequence according to the [direction](#propdef-direction) property; the implicit part of the bidirectional algorithm is ignored. For a block container, the override is applied to an anonymous inline box that surrounds all of its content.

<a id="valdef-unicode-bidi-isolate-override"></a>isolate-override  
<a id="ref-for-isolated-sequence"></a>

<a id="ref-for-valdef-unicode-bidi-bidi-override①"></a>

<a id="ref-for-directional-override"></a>

<a id="ref-for-valdef-unicode-bidi-isolate①"></a>

<a id="ref-for-bidi-isolate"></a>

This combines the [isolation](#bidi-isolate) behavior of [isolate](#valdef-unicode-bidi-isolate) with the [directional override](#directional-override) behavior of [bidi-override](#valdef-unicode-bidi-bidi-override): to surrounding content, it is equivalent to <a id="ref-for-valdef-unicode-bidi-isolate②"></a>isolate, but within the box content is ordered as if <a id="ref-for-valdef-unicode-bidi-bidi-override②"></a>bidi-override were specified. It effectively nests a <a id="ref-for-directional-override①"></a>directional override inside an [isolated sequence](#isolated-sequence).

<a id="valdef-unicode-bidi-plaintext"></a>plaintext  
<a id="ref-for-valdef-unicode-bidi-isolate③"></a>

<a id="ref-for-bidi-paragraph"></a>

<a id="ref-for-isolated-sequence①"></a>

<a id="ref-for-propdef-direction①③"></a>

This value behaves as [isolate](#valdef-unicode-bidi-isolate) except that for the purposes of the Unicode bidirectional algorithm, the base directionality of each of the box’s [bidi paragraphs](#bidi-paragraph) (if a block container) or [isolated sequences](#isolated-sequence) (if an inline) is determined by following the heuristic in rules P2 and P3 of the Unicode bidirectional algorithm (rather than by using the [direction](#propdef-direction) property of the box).

<a id="ref-for-valdef-unicode-bidi-normal③"></a>

Following Unicode Bidirectional Algorithm clause HL3 [\[UAX9\]](#biblio-uax9), values other than [normal](#valdef-unicode-bidi-normal) effectively insert the corresponding Unicode bidi control codes into the text stream at the start and end of the inline element before passing the paragraph to the Unicode bidirectional algorithm for reordering. (See [§ 2.4.2 CSS–Unicode Bidi Control Translation, Text Reordering](#bidi-control-codes).)

<a id="bidi-control-codes-injection-table"></a>

<a id="ref-for-propdef-unicode-bidi①②"></a>

<a id="ref-for-propdef-display"></a>

<strong>Table 4 — structured row/cell transcription</strong>

Bidi control codes injected by [unicode-bidi](#propdef-unicode-bidi) at the start/end of [display: inline](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) boxes

<strong>Row 1</strong>

<strong>Column 1 (header cell; row span 3, scope col):</strong>

<a id="ref-for-propdef-unicode-bidi①③"></a>

[unicode-bidi](#propdef-unicode-bidi) value

<strong>Column 2 (header cell; column span 4):</strong>

<a id="ref-for-propdef-direction①④"></a>

[direction](#propdef-direction) value

<strong>Row 2</strong>

<strong>Column 2 (header cell; column span 2):</strong>

<a id="ref-for-valdef-direction-ltr"></a>

[ltr](#valdef-direction-ltr)

<strong>Column 4 (header cell; column span 2):</strong>

<a id="ref-for-valdef-direction-rtl"></a>

[rtl](#valdef-direction-rtl)

<strong>Row 3</strong>

<strong>Column 2 (header cell):</strong>

start

<strong>Column 3 (header cell):</strong>

end

<strong>Column 4 (header cell):</strong>

start

<strong>Column 5 (header cell):</strong>

end

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-normal④"></a>

[normal](#valdef-unicode-bidi-normal)

<strong>Column 2 (data cell):</strong>

—

<strong>Column 3 (data cell):</strong>

—

<strong>Column 4 (data cell):</strong>

—

<strong>Column 5 (data cell):</strong>

—

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-embed①"></a>

[embed](#valdef-unicode-bidi-embed)

<strong>Column 2 (data cell):</strong>

LRE (U+202A)

<strong>Column 3 (data cell):</strong>

PDF (U+202C)

<strong>Column 4 (data cell):</strong>

RLE (U+202B)

<strong>Column 5 (data cell):</strong>

PDF (U+202C)

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-isolate④"></a>

[isolate](#valdef-unicode-bidi-isolate)

<strong>Column 2 (data cell):</strong>

LRI (U+2066)

<strong>Column 3 (data cell):</strong>

PDI (U+2069)

<strong>Column 4 (data cell):</strong>

RLI (U+2067)

<strong>Column 5 (data cell):</strong>

PDI (U+2069)

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-bidi-override③"></a>

[bidi-override](#valdef-unicode-bidi-bidi-override)\*

<strong>Column 2 (data cell):</strong>

LRO (U+202D)

<strong>Column 3 (data cell):</strong>

PDF (U+202C)

<strong>Column 4 (data cell):</strong>

RLO (U+202E)

<strong>Column 5 (data cell):</strong>

PDF (U+202C)

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-isolate-override①"></a>

[isolate-override](#valdef-unicode-bidi-isolate-override)\*

<strong>Column 2 (data cell):</strong>

FSI,LRO (U+2068,U+202D)

<strong>Column 3 (data cell):</strong>

PDF,PDI (U+202C,U+2069)

<strong>Column 4 (data cell):</strong>

FSI,RLO (U+2068,U+202E)

<strong>Column 5 (data cell):</strong>

PDF,PDI (U+202C,U+2069)

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-valdef-unicode-bidi-plaintext①"></a>

[plaintext](#valdef-unicode-bidi-plaintext)

<strong>Column 2 (data cell):</strong>

FSI (U+2068)

<strong>Column 3 (data cell):</strong>

PDI (U+2069)

<strong>Column 4 (data cell):</strong>

FSI (U+2068)

<strong>Column 5 (data cell):</strong>

PDI (U+2069)

<strong>Row 10</strong>

<strong>Column 1 (data cell; column span 5):</strong>

<a id="ref-for-propdef-unicode-bidi①④"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-root-inline-box"></a>

\* The LRO/RLO+PDF pairs are also applied to the [root inline box](https://www.w3.org/TR/css-inline-3/#root-inline-box) of a [block container](https://www.w3.org/TR/css-display-3/#block-container) if these values of [unicode-bidi](#propdef-unicode-bidi) were specified on the <a id="ref-for-block-container①"></a>block container.

<a id="ref-for-propdef-unicode-bidi①⑤"></a>

<a id="ref-for-valdef-unicode-bidi-bidi-override④"></a>

<a id="ref-for-valdef-unicode-bidi-plaintext②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Because the [unicode-bidi](#propdef-unicode-bidi) property does not inherit, setting [bidi-override](#valdef-unicode-bidi-bidi-override) or [plaintext](#valdef-unicode-bidi-plaintext) on a block box will not affect any descendant blocks. Therefore these values are best used on blocks and inlines that do not contain any block-level structures.

<a id="ref-for-propdef-unicode-bidi①⑥"></a>

<a id="ref-for-propdef-direction①⑤"></a>

<a id="ref-for-valdef-unicode-bidi-plaintext③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [unicode-bidi](#propdef-unicode-bidi) does not affect the [direction](#propdef-direction) property even in the case of [plaintext](#valdef-unicode-bidi-plaintext), and thus does not affect <a id="ref-for-propdef-direction①⑥"></a>direction-dependent layout calculations.

<a id="ref-for-propdef-unicode-bidi①⑦"></a>

<a id="ref-for-valdef-unicode-bidi-normal⑤"></a>

<a id="ref-for-valdef-all-inherit"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-display-inline"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Because the Unicode algorithm has a limit of 125 levels of embedding, care should be taken not to overuse [unicode-bidi](#propdef-unicode-bidi) values other than [normal](#valdef-unicode-bidi-normal). In particular, a value of [inherit](https://www.w3.org/TR/css-cascade-4/#valdef-all-inherit) should be used with extreme caution in deeply nested inline markup. However, for elements that are, in general, intended to be displayed as blocks, a setting of <a id="ref-for-propdef-unicode-bidi①⑧"></a>unicode-bidi: isolate is preferred to keep the element together in case the [display](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) is changed to [inline](https://www.w3.org/TR/css-display-3/#valdef-display-inline) (see example below).

### <a id="bidi-example"></a>2.3.  Example of Bidirectional Text

The following example shows an XML document with bidirectional text. It illustrates an important design principle: document language designers should take bidi into account both in the language proper (elements and attributes) and in any accompanying style sheets. The style sheets should be designed so that bidi rules are separate from other style rules, and such rules should not be overridden by other style sheets so that the document language’s bidi behavior is preserved.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-16648d44"></a>
>
> In this example, lowercase letters stand for inherently left-to-right characters and uppercase letters represent inherently right-to-left characters. The text stream is shown below in logical backing store order.
>
> ```text
> <section dir=rtl>
>   <para>HEBREW1 HEBREW2 english3 HEBREW4 HEBREW5</para>
>   <para>HEBREW6 <emphasis>HEBREW7</emphasis> HEBREW8</para>
> </section>
> <section dir=ltr>
>   <para>english9 english10 english11 HEBREW12 HEBREW13</para>
>   <para>english14 english15 english16</para>
>   <para>english17 <quote dir=rtl>HEBREW18 english19 HEBREW20</quote></para>
> </section>
> ```
>
> Since this is arbitrary XML, the style sheet is responsible for setting the writing direction. This is the style sheet:
>
> ```text
> /* Rules for bidi */
> [dir=rtl] {direction: rtl; unicode-bidi: isolate; }
> [dir=ltr] {direction: ltr; unicode-bidi: isolate; }
> 
> /* Rules for presentation */
> section, para  {display: block;}
> emphasis       {font-weight: bold;}
> quote          {font-style: italic;}
> ```
>
> If the line length is long, the formatting of this text might look like this:
>
> ```text
>                5WERBEH 4WERBEH english3 2WERBEH 1WERBEH
> 
>                                 8WERBEH 7WERBEH 6WERBEH
> 
> english9 english10 english11 13WERBEH 12WERBEH
> 
> english14 english15 english16
> 
> english17 20WERBEH english19 18WERBEH
> ```
>
> The first `<section>` element is a block with a right-to-left base direction, the second `<section>` element is a block with a left-to-right base direction. The `<para>`s are blocks that inherit the base direction from their parents. Thus, the first two `<para>`s are read starting at the top right, the final three are read starting at the top left.
>
> <a id="ref-for-propdef-unicode-bidi①⑨"></a>
>
> <a id="ref-for-valdef-unicode-bidi-normal⑥"></a>
>
> The `<emphasis>` element is inline-level, and since its value for [unicode-bidi](#propdef-unicode-bidi) is [normal](#valdef-unicode-bidi-normal) (the initial value), it has no effect on the ordering of the text.
>
> <a id="ref-for-isolated-sequence②"></a>
>
> The `<quote>` element, on the other hand, creates an [isolated sequence](#isolated-sequence) with the given internal directionality. Note that this causes `HEBREW18` to be to the right of `english19`.
>
> If lines have to be broken, the same text might format like this:
>
> ```text
>        2WERBEH 1WERBEH
>   -EH 4WERBEH english3
>                  5WERB
> 
>    -EH 7WERBEH 6WERBEH
>                  8WERB
> 
> english9 english10 en-
> glish11 12WERBEH
> 13WERBEH
> 
> english14 english15
> english16
> 
> english17 18WERBEH
> 20WERBEH english19
> ```
>
> Notice that because `HEBREW18` must be read before `english19`, it is on the line above `english19`. Just breaking the long line from the earlier formatting would not have worked.
>
> Note also that the first syllable from `english19` might have fit on the previous line, but hyphenation of left-to-right words in a right-to-left context, and vice versa, is usually suppressed to avoid having to display a hyphen in the middle of a line.

### <a id="bidi-algo"></a>2.4.  Applying the Bidirectional Reordering Algorithm

User agents that support bidirectional text must apply the Unicode bidirectional algorithm to every sequence of inline-level boxes uninterrupted by any block boundary or “[bidi type B](http://www.unicode.org/reports/tr9/#Bidirectional_Character_Types)” <a id="forced-paragraph-break"></a>forced paragraph break. This sequence forms the <a id="bidi-paragraph"></a>paragraph unit in the bidirectional algorithm.

#### <a id="bidi-para-direction"></a>2.4.1.  Bidi Paragraph Embedding Levels

<a id="ref-for-propdef-direction①⑦"></a>

In CSS, the paragraph embedding level must be set (following [UAX9 clause HL1](http://www.unicode.org/reports/tr9/#HL1)) according to the [direction](#propdef-direction) property of the paragraph’s containing block rather than by the heuristic given in steps [P2](http://www.unicode.org/reports/tr9/#P2) and [P3](http://www.unicode.org/reports/tr9/#P2) of the Unicode algorithm.

<a id="ref-for-propdef-unicode-bidi②⓪"></a>

<a id="ref-for-valdef-unicode-bidi-plaintext④"></a>

There is, however, one exception: when the computed [unicode-bidi](#propdef-unicode-bidi) of the paragraph’s containing block is [plaintext](#valdef-unicode-bidi-plaintext), the Unicode heuristics in P2 and P3 are used as described in [\[UAX9\]](#biblio-uax9), without the HL1 override.

#### <a id="bidi-control-codes"></a>2.4.2.  CSS–Unicode Bidi Control Translation, Text Reordering

<a id="ref-for-bidi-paragraph①"></a>

<a id="ref-for-propdef-unicode-bidi②①"></a>

The final order of characters within each [bidi paragraph](#bidi-paragraph) is the same as if the bidi control codes had been added as described for [unicode-bidi](#propdef-unicode-bidi) (above), markup had been stripped, and the resulting character sequence had been passed to an implementation of the Unicode bidirectional algorithm for plain text that produced the same line-breaks as the styled text.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that bidi control codes in the source text are still honored, and might not correspond to the document tree structure. This can split inlines or interfere with bidi start/end control pairing in interesting ways.

#### <a id="bidi-atomic-inlines"></a>2.4.3.  Bidi Treatment of Atomic Inlines

<a id="ref-for-replaced-element"></a>

<a id="ref-for-propdef-display②"></a>

<a id="ref-for-propdef-unicode-bidi②②"></a>

<a id="ref-for-valdef-unicode-bidi-embed②"></a>

<a id="ref-for-valdef-unicode-bidi-bidi-override⑤"></a>

<a id="ref-for-propdef-direction①⑧"></a>

In this process, [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) with [display: inline](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) are treated as neutral characters, unless their [unicode-bidi](#propdef-unicode-bidi) property is either [embed](#valdef-unicode-bidi-embed) or [bidi-override](#valdef-unicode-bidi-bidi-override), in which case they are treated as strong characters in the [direction](#propdef-direction) specified for the element. (This is so that, in case the replaced element falls back to rendering inlined text content, its bidi effect on the surrounding text is consistent with its replaced rendering.)

All other atomic inline-level boxes are treated as neutral characters always.

#### <a id="bidi-embedding-breaks"></a>2.4.4.  Paragraph Breaks Within Embeddings and Isolates

<a id="ref-for-bidi-paragraph②"></a>

<a id="ref-for-forced-paragraph-break②"></a>

If an inline box is broken around a [bidi paragraph](#bidi-paragraph) boundary (e.g. if split by a block or [forced paragraph break](#forced-paragraph-break)), then the [HL3](http://www.unicode.org/reports/tr9/#HL3) bidi control codes assigned to the end of the box are also added before the interruption and the codes assigned to the start of the box are also added after it. (In other words, any embedding levels, isolates, or overrides started by the box are closed at the paragraph break and reopened on the other side of it.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-334883cc"></a>
>
> <a id="ref-for-forced-paragraph-break③"></a>
>
> For example, where \<BR/\> is a [forced paragraph break](#forced-paragraph-break) the bidi ordering is identical between
>
> ```text
> <para>...<i1><i2>...<BR/>...</i2></i1>...</para>
> ```
>
> and
>
> ```text
> <para>...<i1><i2>...</i2></i1><BR/><i1><i2>...</i2></i1>...</para>
> ```
>
> <a id="ref-for-propdef-unicode-bidi②③"></a>
>
> for all values of [unicode-bidi](#propdef-unicode-bidi) on inline elements \<i1\> and \<i2\>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that this behavior is applied by CSS for CSS-declared bidi controls applied to the box tree; it does not apply to Unicode’s bidi formatting controls, which are defined to terminate their effect at the end of the bidi paragraph.

#### <a id="bidi-box-model"></a>2.4.5.  Reordering-induced Box Fragmentation

<a id="ref-for-inline-box"></a>

Since bidi reordering can split apart and reorder text that is logically contiguous, bidirectional text can cause an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) containing such text to be split and its fragments reordered within a line.

##### <a id="bidi-fragmentation"></a>2.4.5.1.  Conditions of Reordering-induced Box Fragmentation

<a id="ref-for-box-fragment"></a>

<a id="ref-for-fragment"></a>

<a id="ref-for-tracking"></a>

<a id="ref-for-inline-box①"></a>

When bidi reordering would split apart an inline box due to intervening content, the inline box is considered to be broken into multiple [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment). [\[CSS-BREAK-3\]](#biblio-css-break-3) The box is considered to be thus [fragmented](https://www.w3.org/TR/css3-break/#fragment) if it would be divided by intervening content on an infinitely long line, even if line breaking happens to result in both <a id="ref-for-box-fragment①"></a>box fragments being placed adjacent to each other on the line. In such cases, the nearest common ancestor of text in the two <a id="ref-for-box-fragment②"></a>box fragments (which determines certain aspects of text formatting such as [tracking](https://www.w3.org/TR/css-text-3/#tracking) and [justification](https://www.w3.org/TR/css-text/#text-justify-property) between the two <a id="ref-for-box-fragment③"></a>box fragments, see [\[CSS-TEXT-3\]](#biblio-css-text-3)) is considered to be the nearest common ancestor of the two <a id="ref-for-box-fragment④"></a>box fragments, not the [inline box](https://www.w3.org/TR/css-display-3/#inline-box) itself. However, an <a id="ref-for-inline-box②"></a>inline box is not considered to be broken into multiple <a id="ref-for-box-fragment⑤"></a>box fragments due to bidi reordering if no intervening content would force it to split. (These rules maintain the integrity of an <a id="ref-for-inline-box③"></a>inline box where possible, while keeping bidi-induced fragmentation stable across variations in line-breaking.)

<a id="ref-for-inline-box④"></a>

<a id="ref-for-box-fragment⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-951ac452"></a> In the following example, where lowercase letters represent LTR letters and uppercase letters represent RTL letters, bidi reordering causes the `<em>`’s [inline box](https://www.w3.org/TR/css-display-3/#inline-box) to be divided into two [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment) separated by text outside the `<em>`.
>
> Source code (logical order):
>
> ```text
> <p>here is <em>some MIXED</em> TEXT.</p>
> ```
>
> Rendering (visual order) in a wide containing block, resulting in two inline box fragments separated by external content:
>
> ```text
> here is some TXET DEXIM.
> ```
>
> Rendering (visual order) in a narrow containing block, resulting in two inline box fragments placed adjacent to each other:
>
> ```text
> here is some DEXIM
> TXET.
> ```
<a id="ref-for-inline-box⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f9f11e24"></a> By contrast, in this example, where the mixed-direction phrase is kept together with an isolation, only one fragment is generated—the surrounding content will never split the `<em>`’s [inline box](https://www.w3.org/TR/css-display-3/#inline-box) even inside an infinitely-long containing block:
>
> Source code (logical order):
>
> ```text
> <p>here is <em dir=rtl>some MIXED</em> TEXT.</p>
> ```
>
> Rendering (visual order) in a wide containing block, resulting in one fragment:
>
> ```text
> here is some DEXIM TXET.
> ```
>
> Rendering (visual order) in a narrow containing block, resulting in one fragment:
>
> ```text
> here is some DEXIM
> TXET.
> ```
##### <a id="bidi-fragment-boxes"></a>2.4.5.2.  Box Model of Reordering-induced Box Fragments

<a id="ref-for-start"></a>

<a id="ref-for-end"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb"></a>

For each line box, UAs must take the fragments of each inline box and assign the margins, borders and padding in visual order (not logical order). The [start](#start)-most fragment on the first line box in which the box appears has the <a id="ref-for-start①"></a>start edge’s margin, border, and padding; and the end-most fragment on the last line box in which the box appears has the [end](#end) edge’s margin, border, and padding. For example, in the [horizontal-tb](#valdef-writing-mode-horizontal-tb) writing mode:

- <a id="ref-for-valdef-direction-ltr①"></a>

  <a id="ref-for-propdef-direction①⑨"></a>

  When the parent’s [direction](#propdef-direction) property is [ltr](#valdef-direction-ltr), the left-most box fragment on the first line box in which the box appears has the left margin, left border and left padding, and the right-most box fragment on the last line box in which the box appears has the right padding, right border and right margin.

- <a id="ref-for-valdef-direction-rtl①"></a>

  <a id="ref-for-propdef-direction②⓪"></a>

  When the parent’s [direction](#propdef-direction) property is [rtl](#valdef-direction-rtl), the right-most fragment of the first line box in which the box appears has the right padding, right border and right margin, and the left-most fragment of the last line box in which the box appears has the left margin, left border and left padding.

Analogous rules hold for vertical writing modes.

<a id="ref-for-propdef-box-decoration-break"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) property can override this behavior to draw box decorations on both sides of each fragment. [\[CSS-BREAK-3\]](#biblio-css-break-3)

## <a id="vertical-modes"></a>3.  Vertical Writing Modes

In addition to extensions to CSS2.1’s support for bidirectional text, this module introduces the rules and properties needed to support vertical text layout in CSS.

### <a id="vertical-intro"></a>3.1.  Introduction to Vertical Writing

<em>This subsection is non-normative.</em>

Unlike languages that use the Latin script which are primarily laid out horizontally, Asian languages such as Chinese and Japanese can be laid out vertically. The Japanese example below shows the same text laid out horizontally and vertically. In the horizontal case, text is read from left to right, top to bottom. For the vertical case, the text is read top to bottom, right to left. Indentation from the left edge in the left-to-right horizontal case translates to indentation from the top edge in the top-to-bottom vertical case.

![A comparison of horizontal and vertical Japanese shows that although the lines rotate, the characters remain upright. Some glyphs, however change: a period mark shifts from the bottom left of its glyph box to the top right. Running headers, however, may remain laid out horizontally across the top of the page.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vert-horiz-comparison.png)

Comparison of vertical and horizontal Japanese: iBunko application (iOS)

> <strong data-conversion-semantic="note">Note</strong>
>
> For Chinese and Japanese lines are ordered either right to left or top to bottom, while for Mongolian and Manchu lines are ordered left to right.

The change from horizontal to vertical writing can affect not just the layout, but also the typesetting. For example, the position of a punctuation mark within its spacing box can change from the horizontal to the vertical case, and in some cases alternate glyphs are used.

Vertical text that includes Latin script text or text from other scripts normally displayed horizontally can display that text in a number of ways. For example, Latin words can be rotated sideways, or each letter can be oriented upright:

![A dictionary definition for ヴィルス might write the English word 'virus' rotated 90° clockwise, but stack the letters of the initialisms 'RNA' and 'DNA' upright.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vert-latin-layouts.png)

Examples of Latin in vertical Japanese: Daijirin Viewer 1.4 (iOS)

In some special cases such as two-digit numbers in dates, text is fit compactly into a single vertical character box:

<a id="fig-mac"></a>

![An excerpt from MacFan shows several possible vertical layouts for numbers: the two-digit month and day are written as horizontal-in-vertical blocks; the years are written with each character upright; except in the English phrase “for Mac 2011”, where the date is rotated to match the rotated Latin.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vert-number-layouts.png)

Mac Fan, December 2010, p.49

Layouts often involve a mixture of vertical and horizontal elements:

![Magazines often mix horizontal and vertical layout; for example, using one orientation for the main article text and a different one for sidebar or illustrative content.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vert-horiz-combination.png)

Mixture of vertical and horizontal elements

Vertical text layouts also need to handle bidirectional text layout; clockwise-rotated Arabic, for example, is laid out bottom-to-top.

<a id="ref-for-propdef-writing-mode③"></a>

### <a id="block-flow"></a>3.2.  Block Flow Direction: the [writing-mode](#propdef-writing-mode) property

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-writing-mode"></a>writing-mode

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑥"></a>

horizontal-tb [\|](https://www.w3.org/TR/css-values-4/#comb-one) vertical-rl <a id="ref-for-comb-one⑦"></a>\| vertical-lr <a id="ref-for-comb-one⑧"></a>\| sideways-rl <a id="ref-for-comb-one⑨"></a>\| sideways-lr

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

horizontal-tb

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

All elements except table row groups, table column groups, table rows, table columns, ruby base container, ruby annotation container

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

This property specifies whether lines of text are laid out horizontally or vertically and the direction in which blocks progress. Possible values:

<a id="valdef-writing-mode-horizontal-tb"></a>horizontal-tb  
<a id="ref-for-typographic-mode"></a>

<a id="ref-for-writing-mode②"></a>

<a id="ref-for-block-flow-direction①"></a>

Top-to-bottom [block flow direction](#block-flow-direction). Both the [writing mode](#writing-mode) and the [typographic mode](#typographic-mode) are horizontal.

<a id="valdef-writing-mode-vertical-rl"></a>vertical-rl  
<a id="ref-for-typographic-mode①"></a>

<a id="ref-for-writing-mode③"></a>

<a id="ref-for-block-flow-direction②"></a>

Right-to-left [block flow direction](#block-flow-direction). Both the [writing mode](#writing-mode) and the [typographic mode](#typographic-mode) are vertical.

<a id="valdef-writing-mode-vertical-lr"></a>vertical-lr  
<a id="ref-for-typographic-mode②"></a>

<a id="ref-for-writing-mode④"></a>

<a id="ref-for-block-flow-direction③"></a>

Left-to-right [block flow direction](#block-flow-direction). Both the [writing mode](#writing-mode) and the [typographic mode](#typographic-mode) are vertical.

<a id="valdef-writing-mode-sideways-rl"></a>sideways-rl  
<a id="ref-for-typographic-mode③"></a>

<a id="ref-for-writing-mode⑤"></a>

<a id="ref-for-block-flow-direction④"></a>

Right-to-left [block flow direction](#block-flow-direction). The [writing mode](#writing-mode) is vertical, while the [typographic mode](#typographic-mode) is horizontal.

<a id="valdef-writing-mode-sideways-lr"></a>sideways-lr  
<a id="ref-for-typographic-mode④"></a>

<a id="ref-for-writing-mode⑥"></a>

<a id="ref-for-block-flow-direction⑤"></a>

Left-to-right [block flow direction](#block-flow-direction). The [writing mode](#writing-mode) is vertical, while the [typographic mode](#typographic-mode) is horizontal.

<a id="ref-for-propdef-writing-mode④"></a>

<a id="ref-for-block-flow-direction⑥"></a>

<a id="ref-for-writing-mode⑦"></a>

<a id="ref-for-propdef-text-orientation③"></a>

The [writing-mode](#propdef-writing-mode) property specifies the [block flow direction](#block-flow-direction), which determines the ordering direction of block-level boxes in a block formatting context; the ordering direction of line boxes in a block container that contains inlines; the ordering direction of rows in a table; etc. By virtue of determining the stacking direction of line boxes, the <a id="ref-for-propdef-writing-mode⑤"></a>writing-mode property also determines whether the line boxes' orientation (and thus the [writing mode](#writing-mode)) is horizontal or vertical. The [text-orientation](#propdef-text-orientation) property then determines how text is laid out within the line box.

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-default-object-size"></a>

The content of [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) do not rotate due to the writing mode: images and external content such as from `<iframe>`s, for example, remain upright, and the [default object size](https://drafts.csswg.org/css-images-3/#default-object-size) of 300px×150px does not re-orient. However embedded replaced content involving text (such as MathML content or form elements) should match the replaced element’s writing mode and line orientation if the UA supports such a vertical writing mode for the replaced content.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-26ec58a4"></a>
>
> In the following example, two block elements (1 and 3) separated by an image (2) are presented in various flow writing modes.
>
> Here is a diagram of horizontal writing mode (`writing-mode: horizontal-tb`):
>
> ![Diagram of horizontal layout: blocks 1, 2, and 3 are stacked top-to-bottom](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/horizontal.png)
>
> Here is a diagram for the right-to-left vertical writing mode commonly used in East Asia (`writing-mode: vertical-rl`):
>
> ![Diagram of a right-to-left vertical layout: blocks 1, 2, and 3 are arranged side by side from right to left](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vertical-rl.png)
>
> And finally, here is a diagram for the left-to-right vertical writing mode used for Manchu and Mongolian (`writing-mode: vertical-lr`):
>
> ![Diagram of left-to-right vertical layout: blocks 1, 2, and 3 are arranged side by side from left to right](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vertical-lr.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fb72ee40"></a>
>
> <a id="ref-for-valdef-writing-mode-vertical-rl"></a>
>
> In the following example, some form controls are rendered inside a block with [vertical-rl](#valdef-writing-mode-vertical-rl) writing mode. The form controls are rendered to match the writing mode.
>
> ```text
> <style>
>   form { writing-mode: vertical-rl; }
> </style>
> ...
> <form>
> <p><label>姓名　<input value="艾俐俐"></label>
> <p><label>语言　<select><option>English
>                        <option>français
>                        <option>فارسی
>                        <option>中文
>                        <option>日本語</select></label>
> </form>
> ```
>
> ![Screenshot of vertical layout: the input element is laid lengthwise from top to bottom and its contents rendered in a vertical typographic mode, matching the labels outside it. The drop-down selection control after it slides out to the side (towards the after edge of the block) rather than downward as it would in horizontal writing modes.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vertical-form.png)

<a id="ref-for-propdef-writing-mode⑥"></a>

<a id="ref-for-propdef-display③"></a>

If a box has a different [writing-mode](#propdef-writing-mode) value than its parent box (i.e. nearest ancestor without [display: contents](https://www.w3.org/TR/CSS21/visuren.html#propdef-display)):

- <a id="ref-for-valdef-display-inline-block"></a>

  <a id="ref-for-valdef-display-inline①"></a>

  <a id="ref-for-propdef-display④"></a>

  <a id="ref-for-in-flow"></a>

  If the box would otherwise become an [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) box with a computed [display](https://www.w3.org/TR/CSS21/visuren.html#propdef-display) of [inline](https://www.w3.org/TR/css-display-3/#valdef-display-inline), its <a id="ref-for-propdef-display⑤"></a>display computes instead to [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block).

- <a id="ref-for-block-formatting-context"></a>

  <a id="ref-for-independent-formatting-context"></a>

  <a id="ref-for-block-container②"></a>

  If the box is a [block container](https://www.w3.org/TR/css-display-3/#block-container), then it establishes an [independent](https://www.w3.org/TR/css-display-3/#independent-formatting-context) [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context).

- <a id="ref-for-valdef-display-flow-root"></a>

  <a id="ref-for-valdef-display-flow"></a>

  <a id="ref-for-inner-display-type"></a>

  More generally, if its specified [inner display type](https://www.w3.org/TR/css-display-3/#inner-display-type) is [flow](https://www.w3.org/TR/css-display-3/#valdef-display-flow), then its computed <a id="ref-for-inner-display-type①"></a>inner display type becomes [flow-root](https://www.w3.org/TR/css-display-3/#valdef-display-flow-root). [\[CSS-DISPLAY-3\]](#biblio-css-display-3)

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="embedded-svg-inheritance"></a>
>
> <a id="ref-for-propdef-writing-mode⑦"></a>
>
> As all other inherited CSS properties do, the [writing-mode](#propdef-writing-mode) property inherits to SVG elements inlined (rather than linked) into the source document. This could cause unintentional side effects when, for example, an SVG image designed only for horizontal flow was embedded into a vertical flow document.
>
> Authors can prevent this from happening by adding the following rule:
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-5db07892"></a>
> >
> > ```text
> > svg { writing-mode: initial; }
> > ```
<a id="ref-for-propdef-writing-mode⑧"></a>

#### <a id="svg-writing-mode"></a>3.2.1.  Obsolete SVG1.1 [writing-mode](#propdef-writing-mode) Values

SVG1.1 [\[SVG11\]](#biblio-svg11) defines some additional values: lr, lr-tb, rl, rl-tb, tb, and tb-rl.

These values are <em>obsolete</em> in any context except SVG1 documents and are therefore <em>optional</em> for non-SVG UAs.

<a id="ref-for-propdef-writing-mode⑨"></a>

##### <a id="svg-writing-mode-css"></a>3.2.1.1.  Supporting SVG1.1 [writing-mode](#propdef-writing-mode) values in CSS syntax

UAs that wish to support these values in the context of CSS must compute them as follows:

<a id="ref-for-propdef-writing-mode①⓪"></a>

<strong>Table 6 — structured row/cell transcription</strong>

Mapping of Obsolete SVG1.1 [writing-mode](#propdef-writing-mode) values to modern CSS

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Specified

<strong>Column 2 (header cell):</strong>

Computed

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

lr

<strong>Column 2 (data cell; row span 4):</strong>

<a id="ref-for-valdef-writing-mode-horizontal-tb①"></a>

[horizontal-tb](#valdef-writing-mode-horizontal-tb)

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

lr-tb

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

rl

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

rl-tb

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

tb

<strong>Column 2 (data cell; row span 2):</strong>

<a id="ref-for-valdef-writing-mode-vertical-rl①"></a>

[vertical-rl](#valdef-writing-mode-vertical-rl)

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

tb-rl

<a id="ref-for-propdef-writing-mode①①"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The SVG1.1 values were also present in an older of the CSS [writing-mode](#propdef-writing-mode) specification, which is obsoleted by this specification. The additional tb-lr value of that revision is replaced by [vertical-lr](#valdef-writing-mode-vertical-lr).

<a id="ref-for-propdef-writing-mode①②"></a>

##### <a id="svg-writing-mode-markup"></a>3.2.1.2.  Supporting SVG1.1 [writing-mode](#propdef-writing-mode) values in presentational attributes

In order to support legacy content with presentational attributes, and to allow authors to create documents that support older clients, SVG UAs must add the following style sheet rules to their default UA stylesheet:

```text
@namespace svg "http://www.w3.org/2000/svg";
svg|*[writing-mode=lr], svg|*[writing-mode=lr-tb],
svg|*[writing-mode=rl], svg|*[writing-mode=rl-tb] {
  writing-mode: horizontal-tb; }
svg|*[writing-mode=tb], svg|*[writing-mode=tb-rl] {
  writing-mode: vertical-rl; }
```
> <strong data-conversion-semantic="note">Note</strong>
>
> Authors who wish to create forwards and backwards-compatible SVG content in CSS syntax can use the CSS forwards-compatible parsing rules to do so, e.g.
>
> ```text
> svg|text { writing-mode: tb; writing-mode: vertical-rl; }
> ```
## <a id="inline-alignment"></a>4.  Inline-level Alignment

<a id="ref-for-propdef-vertical-align"></a>

When different kinds of inline-level content are placed together on a line, the baselines of the content and the settings of the [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property control how they are aligned in the transverse direction of the line box. This section discusses what baselines are, how to find them, and how they are used together with the <a id="ref-for-propdef-vertical-align①"></a>vertical-align property to determine the alignment of inline-level content.

### <a id="intro-baselines"></a>4.1.  Introduction to Baselines

<em>This section is non-normative.</em>

<a id="ref-for-inline-axis"></a>

A <a id="baseline"></a>baseline is a line along the [inline axis](#inline-axis) of a line box along which individual glyphs of text are aligned. Baselines guide the design of glyphs in a font (for example, the bottom of most alphabetic glyphs typically align with the alphabetic baseline), and they guide the alignment of glyphs from different fonts or font sizes when typesetting.

![Picture of alphabetic text in two font sizes with the baseline and em-boxes](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/alphabetic-baseline-in-two-font-sizes.svg)

Alphabetic text in two font sizes with the baseline and em-boxes

Different writing systems prefer different baseline tables.

![Latin prefers the alphabetic baseline, on top of which most letters rest, though some have descenders that dangle below it. Indic scripts are sometimes typeset with a hanging baseline, since their glyph shapes appear to be hanging from a horizontal line. Han-based systems, whose glyphs are designed to fill a square, tend to align on their bottoms.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/script-preferred-baselines.gif)

Preferred baselines in various writing systems

A well-constructed font contains a <a id="baseline-table"></a>baseline table, which indicates the position of one or more baselines within the font’s design coordinate space. (The design coordinate space is scaled with the font size.)

![](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/baselines.gif)

In a well-designed mixed-script font, the glyphs are positioned in the coordinate space to harmonize with one another when typeset together. The baseline table is then constructed to match the shape of the glyphs, each baseline positioned to match the glyphs from its preferred scripts.

The baseline table is a property of the font, and the positions of the various baselines apply to all glyphs in the font.

<a id="ref-for-typographic-mode⑤"></a>

Different baseline tables can be provided for alignment in horizontal and vertical text. UAs should use the vertical tables in vertical [typographic modes](#typographic-mode) and the horizontal tables otherwise.

### <a id="text-baselines"></a>4.2.  Text Baselines

In this specification, only the following baselines are considered:

alphabetic  
The <a id="alphabetic-baseline"></a>alphabetic baseline, which typically aligns with the bottom of uppercase Latin glyphs.

central  
<a id="ref-for-under"></a>

<a id="ref-for-over"></a>

The <a id="central-baseline"></a>central baseline, which typically crosses the center of the em box. If the font is missing this baseline, it is assumed to be halfway between the ascender ([over](#over)) and descender ([under](#under)) edges of the em box.

<a id="ref-for-typographic-mode⑥"></a>

<a id="ref-for-central-baseline"></a>

<a id="ref-for-propdef-text-orientation④"></a>

<a id="ref-for-valdef-text-orientation-mixed"></a>

<a id="ref-for-valdef-text-orientation-upright"></a>

<a id="ref-for-alphabetic-baseline"></a>

In vertical [typographic mode](#typographic-mode), the [central baseline](#central-baseline) is used as the dominant baseline when [text-orientation](#propdef-text-orientation) is [mixed](#valdef-text-orientation-mixed) or [upright](#valdef-text-orientation-upright). Otherwise the [alphabetic baseline](#alphabetic-baseline) is used.

> <strong data-conversion-semantic="note">Note</strong>
>
> A future CSS module will deal with baselines in more detail and allow the choice of other dominant baselines and alignment options.

### <a id="replaced-baselines"></a>4.3.  Atomic Inline Baselines

If an [atomic inline](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) (such as an inline-block, inline-table, or replaced inline element) does not have a baseline, then the UA synthesizes a baseline table thus:

alphabetic  
<a id="ref-for-under①"></a>

The alphabetic baseline is assumed to be at the [under](#under) margin edge.

central  
<a id="ref-for-over①"></a>

<a id="ref-for-under②"></a>

The central baseline is assumed to be halfway between the [under](#under) and [over](#over) margin edges of the box.

<a id="ref-for-propdef-vertical-align②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property in [\[CSS2\]](#biblio-css2) defines the baseline of inline-table and inline-block boxes with some exceptions.

### <a id="baseline-alignment"></a>4.4.  Baseline Alignment

<a id="ref-for-typographic-mode⑦"></a>

The <a id="dominant-baseline"></a>dominant baseline (which [can change](#text-baselines) based on the [typographic mode](#typographic-mode)) is used in CSS for alignment in two cases:

- <strong>Aligning glyphs from different fonts within the same inline box.</strong> The glyphs are aligned by matching up the positions of the dominant baseline in their corresponding fonts.

- <a id="ref-for-percentage-value"></a>

  <a id="ref-for-length-value"></a>

  <a id="ref-for-propdef-vertical-align③"></a>

  <strong>Aligning a child inline-level box within its parent.</strong> For the [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) value of baseline, child is aligned to the parent by matching the parent’s dominant baseline to the same baseline in the child. (E.g. if the parent’s dominant baseline is alphabetic, then the child’s alphabetic baseline is matched to the parent’s alphabetic baseline, even if the child’s dominant baseline is something else.) For values of sub, super, [\<length\>](https://www.w3.org/TR/css3-values/#length-value), and [\<percentage\>](https://www.w3.org/TR/css3-values/#percentage-value), the baselines are aligned as for baseline, but the child is shifted according to the offset given by its <a id="ref-for-propdef-vertical-align④"></a>vertical-align value.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-edb77cae"></a>
  > Given following sample markup:
  >
  > ```text
  > <p><span class="outer">Ap <span class="inner">ji</span></span></p>
  > ```
  >
  > And the following style rule:
  >
  > ```text
  > span.inner { font-size: .75em; }
  > ```
  >
  > The baseline tables of the parent (`.outer`) and the child (`.inner`) will not match up due to the font size difference. Since the dominant baseline is the alphabetic baseline, the child box is aligned to its parent by matching up their alphabetic baselines.
  >
  > ![](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/baseline-align-sizes.gif)

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-3b107e11"></a>
  > <a id="ref-for-propdef-vertical-align⑤"></a>
  >
  > If we assign [vertical-align: super](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) to the `.inner` element from the example above, the same rules are used to align the `.inner` child to its parent; the only difference is in addition to the baseline alignment, the child is shifted to the superscript position.
  >
  > ```text
  > span.inner { vertical-align: super; font-size: .75em; }
  > ```
  >
  > ![In this example, the resulting alignment is equivalent to shifting the parent baseline table upwards by the superscript offset, and then aligning the child’s alphabetic baseline to the shifted position of the parent’s alphabetic baseline.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/baseline-align-super.gif)

## <a id="intro-text-layout"></a>5.  Introduction to Vertical Text Layout

Each writing system has one or more native orientations. Modern scripts can therefore be classified into three orientational categories:

<a id="horizontal-only"></a>horizontal-only  
Scripts that have horizontal, but not vertical, native orientation. Includes: Latin, Arabic, Hebrew, Devanagari

<a id="vertical-only"></a>vertical-only  
Scripts that have vertical, but not horizontal, native orientation. Includes: Mongolian, Phags Pa

<a id="bi-orientational"></a>bi-orientational  
Scripts that have both vertical and horizontal native orientation. Includes: Han, Hangul, Japanese Kana

<a id="ref-for-vertical-only"></a>

<a id="ref-for-bi-orientational"></a>

<a id="ref-for-horizontal-only"></a>

A <a id="vertical-script"></a>vertical script is one that has a native vertical orientation: i.e. one that is either [vertical-only](#vertical-only) or that is [bi-orientational](#bi-orientational). A <a id="horizontal-script"></a>horizontal script is one that has a native horizontal orientation: i.e. one that is either [horizontal-only](#horizontal-only) or that is <a id="ref-for-bi-orientational①"></a>bi-orientational. (See [Appendix A](#script-orientations) for a categorization of scripts by native orientation.)

![A Venn diagram of these distinctions would show two circles: one labelled 'vertical', the other 'horizontal'. The overlapped region would represent the bi-orientational scripts, while horizontal-only and vertical-only scripts would occupy their respective circles' exclusive regions.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/script-orientations.png)

In modern typographic systems, all glyphs are assigned a horizontal orientation, which is used when laying out text horizontally. To lay out vertical text, the UA needs to transform the text from its horizontal orientation. This transformation is the <a id="bi-orientational-transform"></a>bi-orientational transform, and there are two types:

rotate  
Rotate the glyph from horizontal to vertical [![Rotate the glyph from horizontal to vertical](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/glyph-right.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/glyph-right.svg)

translate  
Translate the glyph from horizontal to vertical [![Translate the glyph from horizontal to vertical](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/glyph-upright.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/glyph-upright.svg)

<a id="ref-for-bi-orientational-transform"></a>

Scripts with a native vertical orientation have an intrinsic [bi-orientational transform](#bi-orientational-transform), which orients them correctly in vertical text: most CJK (Chinese/Japanese/Korean) characters translate, that is, they are always upright. Characters from other scripts, such as Mongolian, rotate.

<a id="ref-for-propdef-text-orientation⑤"></a>

<a id="ref-for-valdef-text-orientation-mixed①"></a>

<a id="ref-for-valdef-text-orientation-upright①"></a>

<a id="ref-for-horizontal-only①"></a>

Scripts without a native vertical orientation can be either rotated (set sideways) or translated (set upright): the transform used is a stylistic preference depending on the text’s usage, rather than a matter of correctness. The [text-orientation](#propdef-text-orientation) property’s [mixed](#valdef-text-orientation-mixed) and [upright](#valdef-text-orientation-upright) values are provided to specify rotation vs. translation of [horizontal-only](#horizontal-only) text.

<a id="ref-for-propdef-text-orientation⑥"></a>

### <a id="text-orientation"></a>5.1.  Orienting Text: the [text-orientation](#propdef-text-orientation) property

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-text-orientation"></a>text-orientation

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①⓪"></a>

mixed [\|](https://www.w3.org/TR/css-values-4/#comb-one) upright <a id="ref-for-comb-one①①"></a>\| sideways

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

mixed

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

all elements except table row groups, rows, column groups, and columns

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-typographic-mode⑧"></a>

This property specifies the orientation of text within a line. Current values only have an effect in vertical [typographic modes](#typographic-mode): the property has no effect on boxes in horizontal <a id="ref-for-typographic-mode⑨"></a>typographic modes.

Values have the following meanings:

<a id="valdef-text-orientation-mixed"></a>mixed  
<a id="ref-for-typographic-character-unit"></a>

In vertical writing modes, [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) from horizontal-only scripts are [typeset sideways](#typeset-sideways), i.e. 90° clockwise from their standard orientation in horizontal text. <a id="ref-for-typographic-character-unit①"></a>Typographic character units from vertical scripts are typeset with their intrinsic orientation. See [Vertical Orientations](#vertical-orientations) for further details.

This value is typical for layout of dominantly vertical-script text.

<a id="valdef-text-orientation-upright"></a>upright  
<a id="ref-for-typographic-character-unit②"></a>

In vertical writing modes, [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) from horizontal-only scripts are [typeset upright](#typeset-upright), i.e. in their standard horizontal orientation. <a id="ref-for-typographic-character-unit③"></a>Typographic character units from vertical scripts are typeset with their intrinsic orientation and shaped normally. See [Vertical Orientations](#vertical-orientations) for further details.

<a id="ref-for-used-value"></a>

<a id="ref-for-propdef-direction②①"></a>

<a id="ref-for-valdef-direction-ltr②"></a>

This value causes the [used value](https://www.w3.org/TR/css-cascade-4/#used-value) of [direction](#propdef-direction) to be [ltr](#valdef-direction-ltr), and for the purposes of bidi reordering, causes all characters to be treated as strong LTR.

<a id="ref-for-used-value①"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-propdef-direction②②"></a>

<a id="ref-for-valdef-direction-rtl②"></a>

<a id="ref-for-horizontal-writing-mode"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [used value](https://www.w3.org/TR/css-cascade-4/#used-value), rather than the [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value), of [direction](#propdef-direction) is influenced so that [rtl](#valdef-direction-rtl) can inherit properly into any descendants (such as the contents of a [horizontal](#horizontal-writing-mode) inline-block) where this directional override does not apply.

<a id="valdef-text-orientation-sideways"></a>sideways  
In vertical writing modes, this causes all text to be [typeset sideways](#typeset-sideways), as if in a horizontal layout, but rotated 90° clockwise.

<a id="fig-text-orientation"></a>

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (data cell):</strong>

![text-orientation: mixed](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/text-orientation-vr.png)

<strong>Column 2 (data cell):</strong>

![text-orientation: upright](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/text-orientation-up.png)

<strong>Column 3 (data cell):</strong>

![text-orientation: sideways](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/text-orientation-sr.png)

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-mixed②"></a>

[mixed](#valdef-text-orientation-mixed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-upright②"></a>

[upright](#valdef-text-orientation-upright)

<strong>Column 3 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-sideways"></a>

[sideways](#valdef-text-orientation-sideways)

<a id="ref-for-propdef-text-orientation⑦"></a>

<a id="ref-for-propdef-writing-mode①③"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl②"></a>

[text-orientation](#propdef-text-orientation) values ([writing-mode](#propdef-writing-mode) is [vertical-rl](#valdef-writing-mode-vertical-rl))

> <strong data-conversion-semantic="note">Note</strong>
>
> Changing the value of this property may affect inline-level alignment. Refer to [Text Baselines](#text-baselines) for more details.

<a id="ref-for-valdef-text-orientation-sideways①"></a>

UAs may accept <a id="valdef-text-orientation-sideways-right"></a>sideways-right as a value that computes to [sideways](#valdef-text-orientation-sideways) if needed for backward compatibility reasons.

<a id="ref-for-valdef-text-orientation-upright③"></a>

<a id="ref-for-propdef-unicode-bidi②④"></a>

<a id="ref-for-propdef-direction②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> As of writing, major implementations do not support the automatic LTR treatment of RTL characters for [upright](#valdef-text-orientation-upright) typesetting. In such cases, authors may need to explicitly specify [unicode-bidi](#propdef-unicode-bidi) and [direction](#propdef-direction) as in the following example:
>
> ```text
> .vertical-upright-hebrew {
>     writing-mode: vertical-rl;
>     text-orientation: upright;
>     unicode-bidi: bidi-override;
>     direction: ltr;
> }
> ```
#### <a id="vertical-font-features"></a>5.1.1.  Vertical Typesetting and Font Features

<a id="ref-for-valdef-writing-mode-vertical-rl③"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr①"></a>

When typesetting text in [vertical-rl](#valdef-writing-mode-vertical-rl) and [vertical-lr](#valdef-writing-mode-vertical-lr) modes, text is typeset either “upright” or “sideways” as defined below:

<a id="typeset-upright"></a>upright typesetting  
<a id="ref-for-typographic-character-unit④"></a>

[Typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) are individually typeset upright in vertical lines with vertical font metrics. The UA must synthesize vertical font metrics for fonts that lack them. (This specification does not define heuristics for synthesizing such metrics.) Additionally, font features (such as alternate glyphs and other transformation) intended for use in vertical typesetting must be used. (E.g. the OpenType vert feature must be enabled.) Furthermore, characters from horizontal cursive scripts (such as Arabic) are shaped in their isolated forms when typeset upright.

<a id="ref-for-inline-axis①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that even when typeset “upright”, some glyphs should appear rotated. For example, dashes and enclosing punctuation should be oriented relative to the [inline axis](#inline-axis). In OpenType, this is typically handled by glyph substitution, although not all fonts have alternate glyphs for all relevant codepoints. (East Asian fonts usually provide alternates for East Asian codepoints, but Western fonts typically lack any vertical typesetting features and East Asian fonts typically lack vertical substitutions for Western codepoints.) Unicode published draft data on which characters should appear sideways as the SVO property in [this data file](http://www.unicode.org/reports/tr50/tr50-6.Orientation.txt); however, this property has been abandoned for the current revision of [\[UTR50\]](#biblio-utr50).

<a id="ref-for-typographic-character-unit⑤"></a>

<a id="ref-for-character"></a>

[Typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) which are classified as `Tr` or `Tu` in [\[UTR50\]](#biblio-utr50) are expected to have alternate glyphs or positioning for typesetting upright in vertical text. In the case of `Tr` [characters](https://www.w3.org/TR/css-text-3/#character), if such vertical alternate glyphs are missing from the font, the UA <em>may wish to</em> [\[RFC6919\]](#biblio-rfc6919) (but is not expected to) synthesize the missing glyphs by [typesetting them sideways](#typeset-sideways) etc.

<a id="typeset-sideways"></a>sideways typesetting  
<a id="ref-for-typographic-character-unit⑥"></a>

[Typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) typeset as a run rotated 90° clockwise from their upright orientation, using horizontal metrics and composition, and vertical typesetting features are not used. However, if the font has features meant to be enabled for sideways text that is typeset in vertical lines (e.g. to adjust brush stroke angles or alignment), those features are used. (An example of such a feature would be the proposed [`vrtr` OpenType font feature](http://blogs.adobe.com/CCJKType/2013/08/tale-of-three-features.html).)

#### <a id="vertical-orientations"></a>5.1.2.  Mixed Vertical Orientations

<a id="ref-for-propdef-text-orientation⑧"></a>

<a id="ref-for-valdef-text-orientation-mixed③"></a>

<a id="ref-for-typographic-character-unit⑦"></a>

[\[UTR50\]](#biblio-utr50) defines the `Vertical_Orientation` property for the default glyph orientation of mixed-orientation vertical text. When [text-orientation](#propdef-text-orientation) is [mixed](#valdef-text-orientation-mixed), the UA must determine the orientation of each [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) by its `Vertical_Orientation` property: [typeseting it upright](#typeset-upright) if its orientation property is `U`, `Tu`, or `Tr`; or [typesetting it sideways](#typeset-sideways) (90° clockwise from horizontal) if its orientation property is `R`.

<a id="ref-for-valdef-text-orientation-mixed④"></a>

<a id="ref-for-valdef-writing-mode-sideways-lr"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that UTR50 does not handle scripts that rotate -90° in vertical contexts, so they will not be typeset correctly with [mixed](#valdef-text-orientation-mixed) orientation. Use [sideways-lr](#valdef-writing-mode-sideways-lr) for such scripts.

> <strong data-conversion-semantic="note">Note</strong>
>
> The OpenType vrt2 feature, which is intended for mixed-orientation typesetting, is not used by CSS. It delegates the responsibility for orienting glyphs to the font designer. CSS instead dictates the orientation through [\[UTR50\]](#biblio-utr50) and orients glyphs by typesetting them sideways or upright as appropriate.

<a id="ref-for-propdef-glyph-orientation-vertical"></a>

#### <a id="glyph-orientation"></a>5.1.3.  Obsolete: the SVG1.1 [glyph-orientation-vertical](#propdef-glyph-orientation-vertical) property

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-glyph-orientation-vertical"></a>glyph-orientation-vertical

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one①②"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) 0deg <a id="ref-for-comb-one①③"></a>\| 90deg <a id="ref-for-comb-one①④"></a>\| 0 <a id="ref-for-comb-one①⑤"></a>\| 90

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

na/

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animatable:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

n/a

<a id="ref-for-propdef-glyph-orientation-vertical①"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-propdef-text-orientation⑨"></a>

Some SVG user agents will need to process documents containing the obsolete SVG [glyph-orientation-vertical](#propdef-glyph-orientation-vertical) property, which was defined to accept an auto keyword as well as [\<angle\>](https://www.w3.org/TR/css3-values/#angle-value) and [\<integer\>](https://www.w3.org/TR/css3-values/#integer-value) values representing multiples of 90°. While supporting this property is <em>optional</em>, UAs that do so must alias <a id="ref-for-propdef-glyph-orientation-vertical②"></a>glyph-orientation-vertical as a shorthand of [text-orientation](#propdef-text-orientation) as follows:

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-propdef-glyph-orientation-vertical③"></a>

Shorthand [glyph-orientation-vertical](#propdef-glyph-orientation-vertical) value

<strong>Column 2 (header cell):</strong>

<a id="ref-for-propdef-text-orientation①⓪"></a>

Longhand [text-orientation](#propdef-text-orientation) value

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

auto

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-mixed⑤"></a>

[mixed](#valdef-text-orientation-mixed)

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

0deg

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-upright④"></a>

[upright](#valdef-text-orientation-upright)

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

0

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-upright⑤"></a>

[upright](#valdef-text-orientation-upright)

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

90deg

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-sideways②"></a>

[sideways](#valdef-text-orientation-sideways)

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

90

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-text-orientation-sideways③"></a>

[sideways](#valdef-text-orientation-sideways)

<a id="ref-for-propdef-glyph-orientation-vertical④"></a>

UAs must ignore and treat as invalid any other values for the [glyph-orientation-vertical](#propdef-glyph-orientation-vertical) property; and treat as invalid the glyph-orientation-horizontal property in its entirety.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The 180deg and 270deg values, the radian and gradian values, and the glyph-orientation-horizontal property are not mapped because they have no known use cases nor significant amounts of dependent content, and are therefore not part of CSS, and have been likewise dropped from SVG.

## <a id="abstract-box"></a>6.  Abstract Box Terminology

<a id="ref-for-valdef-writing-mode-horizontal-tb②"></a>

CSS2.1 [\[CSS2\]](#biblio-css2) defines the box layout model of CSS in detail, but only for the [horizontal-tb](#valdef-writing-mode-horizontal-tb) writing mode. Layout is analogous in writing modes other than <a id="ref-for-valdef-writing-mode-horizontal-tb③"></a>horizontal-tb; however directional and dimensional terms in CSS2.1 must be abstracted and remapped appropriately.

<a id="ref-for-propdef-writing-mode①④"></a>

<a id="ref-for-propdef-direction②④"></a>

This section defines abstract directional and dimensional terms and their mappings in order to define box layout for other writing modes, and to provide terminology for future specs to define their layout concepts abstractly. (The next section explains how to apply them to CSS2.1 layout calculations and how to handle [orthogonal flows](#orthogonal-flows).) Although they derive from the behavior of text, these abstract mappings exist even for boxes that do not contain any line boxes: they are calculated directly from the values of the [writing-mode](#propdef-writing-mode) and [direction](#propdef-direction) properties.

There are three sets of directional terms in CSS:

<a id="physical"></a>physical  
Interpreted relative to the page, independent of writing mode. The <a id="physical-direction"></a>physical directions are <a id="physical-left"></a>left, <a id="physical-right"></a>right, <a id="physical-top"></a>top, and <a id="physical-bottom"></a>bottom.

<a id="flow-relative"></a>[flow-relative](#logical-directions)  
<a id="ref-for-inline-end"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-end①"></a>

<a id="ref-for-start②"></a>

Interpreted relative to the flow of content. The flow-relative directions are [start](#start) and [end](#end), or [block-start](#block-start), [block-end](#block-end), [inline-start](#inline-start), and [inline-end](#inline-end) if the dimension is also ambiguous.

<a id="line-relative"></a>[line-relative](#line-directions)  
<a id="ref-for-line-under"></a>

<a id="ref-for-line-over"></a>

<a id="ref-for-line-right②"></a>

<a id="ref-for-line-left②"></a>

Interpreted relative to the orientation of the line box. The line-relative directions are [line-left](#line-left), [line-right](#line-right), [line-over](#line-over), and [line-under](#line-under).

The <a id="physical-dimensions"></a>physical dimensions are <a id="width"></a>width and <a id="height"></a>height, which correspond to measurements along the <a id="x-axis"></a>x-axis (<a id="horizontal-dimension"></a>horizontal dimension) and <a id="y-axis"></a>y-axis (<a id="vertical-dimension"></a>vertical dimension), respectively. [Abstract dimensions](#abstract-axes) are identical in both flow-relative and line-relative terms, so there is only one set of these terms.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[CSS3-FLEXBOX\]](#biblio-css3-flexbox) also defines [flex-relative terms](https://www.w3.org/TR/css3-flexbox/#box-model), which are used in describing flex layout.

### <a id="abstract-axes"></a>6.1.  Abstract Dimensions

The <a id="abstract-dimensions"></a>abstract dimensions are defined below:

<a id="block-dimension"></a>block dimension  
<a id="ref-for-horizontal-dimension"></a>

<a id="ref-for-vertical-dimension"></a>

The dimension perpendicular to the flow of text within a line, i.e. the [vertical dimension](#vertical-dimension) in horizontal writing modes, and the [horizontal dimension](#horizontal-dimension) in vertical writing modes.

<a id="inline-dimension"></a>inline dimension  
<a id="ref-for-vertical-dimension①"></a>

<a id="ref-for-horizontal-dimension①"></a>

The dimension parallel to the flow of text within a line, i.e. the [horizontal dimension](#horizontal-dimension) in horizontal writing modes, and the [vertical dimension](#vertical-dimension) in vertical writing modes.

<a id="block-axis"></a>block axis  
<a id="ref-for-x-axis"></a>

<a id="ref-for-y-axis"></a>

The axis in the block dimension, i.e. the [vertical axis](#y-axis) in horizontal writing modes and the [horizontal axis](#x-axis) in vertical writing modes.

<a id="inline-axis"></a>inline axis  
<a id="ref-for-y-axis①"></a>

<a id="ref-for-x-axis①"></a>

The axis in the inline dimension, i.e. the [horizontal axis](#x-axis) in horizontal writing modes and the [vertical axis](#y-axis) in vertical writing modes.

<a id="block-size"></a>block size  
<a id="extent"></a><a id="logical-height"></a>logical height  
A measurement in the block dimension: refers to the physical height (vertical dimension) in horizontal writing modes, and to the physical width (horizontal dimension) in vertical writing modes.

<a id="inline-size"></a>inline size  
<a id="measure"></a><a id="logical-width"></a>logical width  
A measurement in the inline dimension: refers to the physical width (horizontal dimension) in horizontal writing modes, and to the physical height (vertical dimension) in vertical writing modes.

### <a id="logical-directions"></a>6.2.  Flow-relative Directions

<a id="ref-for-block-start①"></a>

<a id="ref-for-block-end①"></a>

<a id="ref-for-inline-start①"></a>

<a id="ref-for-inline-end①"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb④"></a>

The <a id="flow-relative-direction"></a>flow-relative directions, [block-start](#block-start), [block-end](#block-end), [inline-start](#inline-start), and [inline-end](#inline-end), are defined relative to the flow of content on the page. In an LTR [horizontal-tb](#valdef-writing-mode-horizontal-tb) writing mode, they correspond to the top, bottom, left, and right directions, respectively. They are defined as follows:

<a id="block-start"></a>block-start  
<a id="ref-for-valdef-writing-mode-vertical-lr②"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl④"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb⑤"></a>

<a id="ref-for-propdef-writing-mode①⑤"></a>

<a id="ref-for-block-flow-direction⑦"></a>

The side that comes earlier in the [block flow direction](#block-flow-direction), as determined by the [writing-mode](#propdef-writing-mode) property: the physical top in [horizontal-tb](#valdef-writing-mode-horizontal-tb) mode, the right in [vertical-rl](#valdef-writing-mode-vertical-rl), and the left in [vertical-lr](#valdef-writing-mode-vertical-lr).

<a id="block-end"></a>block-end  
<a id="ref-for-block-start②"></a>

The side opposite [block-start](#block-start).

<a id="inline-start"></a>inline-start  
<a id="ref-for-line-right③"></a>

<a id="ref-for-valdef-direction-rtl③"></a>

<a id="ref-for-line-left③"></a>

<a id="ref-for-valdef-direction-ltr③"></a>

<a id="ref-for-propdef-direction②⑤"></a>

The side from which text of the inline base direction would start. For boxes with a used [direction](#propdef-direction) value of [ltr](#valdef-direction-ltr), this means the [line-left](#line-left) side. For boxes with a used <a id="ref-for-propdef-direction②⑥"></a>direction value of [rtl](#valdef-direction-rtl), this means the [line-right](#line-right) side.

<a id="inline-end"></a>inline-end  
<a id="ref-for-start③"></a>

The side opposite [start](#start).

<a id="ref-for-block-start③"></a>

<a id="ref-for-inline-start②"></a>

<a id="ref-for-block-end②"></a>

<a id="ref-for-inline-end②"></a>

Where contextually unambiguous or encompassing both meanings, the terms <a id="start"></a>start and <a id="end"></a>end are used in place of [block-start](#block-start)/[inline-start](#inline-start) and [block-end](#block-end)/[inline-end](#inline-end), respectively.

<a id="ref-for-block-start④"></a>

<a id="ref-for-block-end③"></a>

<a id="ref-for-propdef-writing-mode①⑥"></a>

<a id="ref-for-inline-start③"></a>

<a id="ref-for-inline-end③"></a>

<a id="ref-for-propdef-direction②⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that while determining the [block-start](#block-start) and [block-end](#block-end) sides of a box depends only on the [writing-mode](#propdef-writing-mode) property, determining the [inline-start](#inline-start) and [inline-end](#inline-end) sides of a box depends not only on the <a id="ref-for-propdef-writing-mode①⑦"></a>writing-mode property but also the [direction](#propdef-direction) property.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bf84250f"></a>
>
> ![](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/sizing-ltr-tb.svg)
>
> Physical/logical terms as applicable to typical English text layout

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40873f56"></a>
>
> ![](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/sizing-ttb-rl.svg)
>
> Physical/logical terms as applicable to typical Chinese text layout

### <a id="line-directions"></a>6.3.  Line-relative Directions

<a id="ref-for-propdef-writing-mode①⑧"></a>

<a id="ref-for-block-start⑤"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr③"></a>

<a id="ref-for-block-end④"></a>

The <a id="line-orientation"></a>line orientation determines which side of a line box is the logical “top” (ascender side). It is given by the [writing-mode](#propdef-writing-mode) property. Usually the line-relative “top” corresponds to the [block-start](#block-start) side, but this is not always the case: in Mongolian typesetting (and thus by default in [vertical-lr](#valdef-writing-mode-vertical-lr) writing modes), the line-relative “top” corresponds to the [block-end](#block-end) side. Hence the need for distinct terminology.

![Mongolian mixed with English](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/mongolian-lr.jpg)

A primarily Mongolian document, such as the one above, is written in vertical lines stacking left to right, but lays its Latin text with the tops of the glyphs towards the right. This makes the text run in the same inline direction as Mongolian (top-to-bottom) and face the same direction it does in other East Asian layouts (which have vertical lines stacking right to left), but the glyphs' tops are facing the bottom of the line stack rather than the top, which in an English paragraph would be upside-down. (See this [Diagram of Mongolian Text Layout](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/text-flow-vectors-lr-reverse.svg).)

<a id="ref-for-propdef-text-align"></a>

<a id="ref-for-line-orientation"></a>

In addition to a line-relative “top” and “bottom” to map things like 'vertical-align: top', CSS also needs to refer to a line-relative “left” and “right” in order to map things like [text-align: left](https://www.w3.org/TR/css-text-3/#propdef-text-align). Thus there are four <a id="line-relative-direction"></a>line-relative directions, which are defined relative to the [line orientation](#line-orientation) as follows:

<a id="over"></a>over or <a id="line-over"></a>line-over  
Nominally the side that corresponds to the ascender side or “top” side of a line box. (The side overlines are typically drawn on.)

<a id="under"></a>under or <a id="line-under"></a>line-under  
<a id="ref-for-over②"></a>

Opposite of [over](#over): the line-relative “bottom” or descender side. (The side underlines are typically drawn on.)

<a id="line-left"></a>line-left  
The line-relative "left" side of a line box, which is nominally the side from which LTR text would start.

<a id="line-right"></a>line-right  
<a id="ref-for-line-left④"></a>

The line-relative "right" side of a line box, which is nominally the side from which RTL text would start. (Opposite of [line-left](#line-left).)

See the [table below](#logical-to-physical) for the exact mappings between physical and line-relative directions.

[![Line orientation compass](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-up.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-up.svg)

<a id="ref-for-valdef-writing-mode-horizontal-tb⑥"></a>

Line orientation in [horizontal-tb](#valdef-writing-mode-horizontal-tb)

[![Typical orientation in vertical](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-right.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-right.svg)

<a id="ref-for-valdef-writing-mode-vertical-rl⑤"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr④"></a>

<a id="ref-for-valdef-writing-mode-sideways-rl"></a>

Line orientation in [vertical-rl](#valdef-writing-mode-vertical-rl), [vertical-lr](#valdef-writing-mode-vertical-lr), and [sideways-rl](#valdef-writing-mode-sideways-rl)

[![Typical orientation in vertical](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-left.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/line-orient-left.svg)

<a id="ref-for-valdef-writing-mode-sideways-lr①"></a>

Line orientation in [sideways-lr](#valdef-writing-mode-sideways-lr)

<a id="ref-for-propdef-text-orientation①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="vertical-metrics"></a>
>
> ![Baseline of an upright glyph is drawn vertically from the top center](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/vertical-upright-baseline.svg)
>
> Vertical baseline of an upright glyph
>
> When [text-orientation: upright](#propdef-text-orientation), the baseline is still vertical, and the vertical baseline in the font is used, or the vertical baseline is synthesized if the font does not provide.
>
> <a id="ref-for-valdef-text-orientation-mixed⑥"></a>
>
> <a id="ref-for-valdef-text-orientation-sideways④"></a>
>
> <a id="ref-for-line-over①"></a>
>
> <a id="ref-for-line-under①"></a>
>
> Since the baseline is vertical, the definitions for [mixed](#valdef-text-orientation-mixed) or [sideways](#valdef-text-orientation-sideways) above still apply; i.e., [line-over](#line-over) is on right, and [line-under](#line-under) is on left.
>
> This is in line with font systems such as OpenType which defines the ascender on right and the descender on left in their vertical metrics.

### <a id="logical-to-physical"></a>6.4.  Abstract-to-Physical Mappings

<a id="ref-for-propdef-direction②⑧"></a>

<a id="ref-for-propdef-writing-mode①⑨"></a>

The following table summarizes the abstract-to-physical mappings (based on the <em>used</em> [direction](#propdef-direction) and [writing-mode](#propdef-writing-mode)):

<strong>Table 11 — structured row/cell transcription</strong>

Abstract-Physical Mapping

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-writing-mode②⓪"></a>

[writing-mode](#propdef-writing-mode)

<strong>Column 2 (header cell; column span 2):</strong>

<a id="ref-for-valdef-writing-mode-horizontal-tb⑦"></a>

[horizontal-tb](#valdef-writing-mode-horizontal-tb)

<strong>Column 4 (header cell; column span 2):</strong>

<a id="ref-for-valdef-writing-mode-sideways-rl①"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl⑥"></a>

[vertical-rl](#valdef-writing-mode-vertical-rl), [sideways-rl](#valdef-writing-mode-sideways-rl)

<strong>Column 6 (header cell; column span 2):</strong>

<a id="ref-for-valdef-writing-mode-vertical-lr⑤"></a>

[vertical-lr](#valdef-writing-mode-vertical-lr)

<strong>Column 8 (header cell; column span 2):</strong>

<a id="ref-for-valdef-writing-mode-sideways-lr②"></a>

[sideways-lr](#valdef-writing-mode-sideways-lr)

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-direction②⑨"></a>

[direction](#propdef-direction)

<strong>Column 2 (header cell):</strong>

<a id="ref-for-valdef-direction-ltr④"></a>

[ltr](#valdef-direction-ltr)

<strong>Column 3 (header cell):</strong>

<a id="ref-for-valdef-direction-rtl④"></a>

[rtl](#valdef-direction-rtl)

<strong>Column 4 (header cell):</strong>

<a id="ref-for-valdef-direction-ltr⑤"></a>

[ltr](#valdef-direction-ltr)

<strong>Column 5 (header cell):</strong>

<a id="ref-for-valdef-direction-rtl⑤"></a>

[rtl](#valdef-direction-rtl)

<strong>Column 6 (header cell):</strong>

<a id="ref-for-valdef-direction-ltr⑥"></a>

[ltr](#valdef-direction-ltr)

<strong>Column 7 (header cell):</strong>

<a id="ref-for-valdef-direction-rtl⑥"></a>

[rtl](#valdef-direction-rtl)

<strong>Column 8 (header cell):</strong>

<a id="ref-for-valdef-direction-ltr⑦"></a>

[ltr](#valdef-direction-ltr)

<strong>Column 9 (header cell):</strong>

<a id="ref-for-valdef-direction-rtl⑦"></a>

[rtl](#valdef-direction-rtl)

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

block-size

<strong>Column 2 (data cell; column span 2):</strong>

height

<strong>Column 4 (data cell; column span 6):</strong>

width

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

inline-size

<strong>Column 2 (data cell; column span 2):</strong>

width

<strong>Column 4 (data cell; column span 6):</strong>

height

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

block-start

<strong>Column 2 (data cell; column span 2):</strong>

top

<strong>Column 4 (data cell; column span 2):</strong>

right

<strong>Column 6 (data cell; column span 4):</strong>

left

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

block-end

<strong>Column 2 (data cell; column span 2):</strong>

bottom

<strong>Column 4 (data cell; column span 2):</strong>

left

<strong>Column 6 (data cell; column span 4):</strong>

right

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

inline-start

<strong>Column 2 (data cell):</strong>

left

<strong>Column 3 (data cell):</strong>

right

<strong>Column 4 (data cell):</strong>

top

<strong>Column 5 (data cell):</strong>

bottom

<strong>Column 6 (data cell):</strong>

top

<strong>Column 7 (data cell):</strong>

bottom

<strong>Column 8 (data cell):</strong>

bottom

<strong>Column 9 (data cell):</strong>

top

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

inline-end

<strong>Column 2 (data cell):</strong>

right

<strong>Column 3 (data cell):</strong>

left

<strong>Column 4 (data cell):</strong>

bottom

<strong>Column 5 (data cell):</strong>

top

<strong>Column 6 (data cell):</strong>

bottom

<strong>Column 7 (data cell):</strong>

top

<strong>Column 8 (data cell):</strong>

top

<strong>Column 9 (data cell):</strong>

bottom

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

over

<strong>Column 2 (data cell; column span 2):</strong>

top

<strong>Column 4 (data cell; column span 4):</strong>

right

<strong>Column 8 (data cell; column span 2):</strong>

left

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

under

<strong>Column 2 (data cell; column span 2):</strong>

bottom

<strong>Column 4 (data cell; column span 4):</strong>

left

<strong>Column 8 (data cell; column span 2):</strong>

right

<strong>Row 11</strong>

<strong>Column 1 (header cell; scope row):</strong>

line-left

<strong>Column 2 (data cell; column span 2):</strong>

left

<strong>Column 4 (data cell; column span 4):</strong>

top

<strong>Column 8 (data cell; column span 2):</strong>

bottom

<strong>Row 12</strong>

<strong>Column 1 (header cell; scope row):</strong>

line-right

<strong>Column 2 (data cell; column span 2):</strong>

right

<strong>Column 4 (data cell; column span 4):</strong>

bottom

<strong>Column 8 (data cell; column span 2):</strong>

top

<a id="ref-for-used-value②"></a>

<a id="ref-for-propdef-direction③⓪"></a>

<a id="ref-for-propdef-writing-mode②①"></a>

<a id="ref-for-propdef-text-orientation①②"></a>

<a id="ref-for-vertical-writing-mode"></a>

<a id="ref-for-valdef-text-orientation-upright⑥"></a>

<a id="ref-for-valdef-direction-ltr⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [used](https://www.w3.org/TR/css-cascade-4/#used-value) [direction](#propdef-direction) depends on the computed [writing-mode](#propdef-writing-mode) and [text-orientation](#propdef-text-orientation): in [vertical writing modes](#vertical-writing-mode), a <a id="ref-for-propdef-text-orientation①③"></a>text-orientation value of [upright](#valdef-text-orientation-upright) forces the used <a id="ref-for-propdef-direction③①"></a>direction to [ltr](#valdef-direction-ltr).

## <a id="abstract-layout"></a>7.  Abstract Box Layout

### <a id="vertical-layout"></a>7.1.  Principles of Layout in Vertical Writing Modes

CSS box layout in vertical writing modes is analogous to layout in the horizontal writing modes, following the principles outlined below:

Layout calculation rules (such as those in CSS2.1, Section 10.3) that apply to the horizontal dimension in horizontal writing modes instead apply to the vertical dimension in vertical writing modes. Likewise, layout calculation rules (such as those in CSS2.1, Section 10.6) that apply to the vertical dimension in horizontal writing modes instead apply to the horizontal dimension in vertical writing modes. Thus:

- Layout rules that refer to the width use the height instead, and vice versa.

- <a id="ref-for-flow-relative-direction"></a>

  <a id="ref-for-propdef-margin-left"></a>

  <a id="ref-for-valdef-writing-mode-vertical-rl⑦"></a>

  <a id="ref-for-propdef-margin-bottom"></a>

  Layout rules that refer to the \*-left and \*-right box properties (border, margin, padding, positioning offsets) use \*-top and \*-bottom instead, and vice versa, mapping the horizontal writing-mode rules of CSS2.1 into vertical writing-mode rules using the [flow-relative directions](#flow-relative-direction). The side of the box these properties apply to doesn’t change: only which values are inputs to which layout calculations changes. The [margin-left](https://www.w3.org/TR/css-box-3/#propdef-margin-left) property still affects the lefthand margin, for example; however in a [vertical-rl](#valdef-writing-mode-vertical-rl) writing mode it takes part in margin collapsing in place of [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom).

- <a id="ref-for-propdef-direction③②"></a>

  <a id="ref-for-propdef-text-align①"></a>

  <a id="ref-for-start④"></a>

  <a id="ref-for-end②"></a>

  Layout rules that depend on the [direction](#propdef-direction) property to choose between left and right (e.g. overflow, overconstraint resolution, the initial value for [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align), table column ordering) are abstracted to the [start](#start) and [end](#end) sides and applied appropriately.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63009bd2"></a>
>
> <a id="ref-for-valdef-writing-mode-vertical-rl⑧"></a>
>
> <a id="ref-for-valdef-text-orientation-mixed⑦"></a>
>
> <a id="ref-for-valdef-direction-rtl⑧"></a>
>
> <a id="ref-for-inline-start④"></a>
>
> <a id="ref-for-block-start⑥"></a>
>
> <a id="ref-for-propdef-margin-right"></a>
>
> <a id="ref-for-propdef-margin-left①"></a>
>
> <a id="ref-for-propdef-margin-top"></a>
>
> <a id="ref-for-propdef-margin-bottom①"></a>
>
> For example, in vertical writing modes, table rows are vertical and table columns are horizontal. In a [vertical-rl](#valdef-writing-mode-vertical-rl) [mixed](#valdef-text-orientation-mixed) [rtl](#valdef-direction-rtl) table, the first column would be on the bottom (the [inline-start](#inline-start) side), and the first row on the right (the [block-start](#block-start) side). The table’s [margin-right](https://www.w3.org/TR/css-box-3/#propdef-margin-right) and [margin-left](https://www.w3.org/TR/css-box-3/#propdef-margin-left) would collapse with margins before (on the right) and after (on the left) the table, respectively, and if the table had auto values for [margin-top](https://www.w3.org/TR/css-box-3/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-3/#propdef-margin-bottom) it would be centered vertically within its block flow.
>
> [![Diagram of a vertical-rl mixed rtl table in a vertical block formatting context, showing the ordering of rows, cells, and columns as described above.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/vertical-table.png)](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/diagrams/vertical-table.svg)
>
> <a id="ref-for-valdef-writing-mode-vertical-rl⑨"></a>
>
> Table in [vertical-rl](#valdef-writing-mode-vertical-rl) RTL writing mode

<a id="ref-for-line-left⑤"></a>

<a id="ref-for-line-right④"></a>

For features such as text alignment, floating, and list marker positioning, that primarily reference the left or right sides of the line box or its longitudinal parallels and therefore have no top or bottom equivalent, the [line-left](#line-left) and [line-right](#line-right) sides are used as the reference for the left and right sides respectively.

<a id="ref-for-propdef-vertical-align⑥"></a>

<a id="ref-for-line-over②"></a>

<a id="ref-for-line-under②"></a>

Likewise for features such as underlining, overlining, and baseline alignment (the unfortunately-named [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align)), that primarily reference the top or bottom sides of the linebox or its transversal parallels and therefore have no left or right equivalent, the [line-over](#line-over) and [line-under](#line-under) sides are used as the reference for the top and bottom sides respectively.

The details of these mappings are provided below.

### <a id="dimension-mapping"></a>7.2.  Dimensional Mapping

Certain properties behave logically as follows:

- <a id="ref-for-propdef-border-spacing"></a>

  The first and second values of the [border-spacing](https://www.w3.org/TR/CSS21/tables.html#propdef-border-spacing) property represent spacing between columns and rows respectively, not necessarily the horizontal and vertical spacing respectively. [\[CSS2\]](#biblio-css2)

- <a id="ref-for-propdef-line-height"></a>

  The [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) property always refers to the logical height. [\[CSS2\]](#biblio-css2)

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-max-width"></a>

The height properties ([height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height), [min-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-height), and [max-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-height)) refer to the physical height, and the width properties ([width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width), [min-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-min-width), and [max-width](https://www.w3.org/TR/CSS21/visudet.html#propdef-max-width)) refer to the physical width. However, the rules used to calculate box dimensions and positions are logical.

<a id="ref-for-inline-size"></a>

<a id="ref-for-inline-start⑤"></a>

<a id="ref-for-inline-end④"></a>

<a id="ref-for-block-size"></a>

<a id="ref-for-block-start⑦"></a>

<a id="ref-for-block-end⑤"></a>

For example, the calculation rules in [CSS2.1 Section 10.3](https://www.w3.org/TR/CSS2/visudet.html#Computing_widths_and_margins) are used for the inline dimension measurements: they apply to the [inline size](#inline-size) (which could be either the physical width or physical height) and to the [inline-start](#inline-start) and [inline-end](#inline-end) margins, padding, and border. Likewise the calculation rules in [CSS2.1 Section 10.6](https://www.w3.org/TR/CSS2/visudet.html#Computing_heights_and_margins) are used in the block dimension: they apply to the [block size](#block-size) and to the [block-start](#block-start) and [block-end](#block-end) margins, padding, and border. [\[CSS2\]](#biblio-css2)

<a id="ref-for-inline-size①"></a>

As a corollary, percentages on the margin and padding properties, which are always calculated with respect to the containing block width in CSS2.1, are calculated with respect to the <em><a href="#inline-size">inline size</a></em> of the containing block in CSS3.

### <a id="orthogonal-flows"></a>7.3.  Orthogonal Flows

> <strong data-conversion-semantic="note">Note</strong>
>
> We appreciate feedback in general, but we are particularly interested in feedback on this particularly complicated section.

<a id="ref-for-propdef-writing-mode②②"></a>

When a box has a different [writing-mode](#propdef-writing-mode) from its containing block two cases are possible:

- <a id="ref-for-valdef-writing-mode-vertical-lr⑥"></a>

  <a id="ref-for-valdef-writing-mode-vertical-rl①⓪"></a>

  The two writing modes are parallel to each other. (For example, [vertical-rl](#valdef-writing-mode-vertical-rl) and [vertical-lr](#valdef-writing-mode-vertical-lr)).

- <a id="ref-for-valdef-writing-mode-vertical-rl①①"></a>

  <a id="ref-for-valdef-writing-mode-horizontal-tb⑧"></a>

  The two writing modes are perpendicular to each other. (For example, [horizontal-tb](#valdef-writing-mode-horizontal-tb) and [vertical-rl](#valdef-writing-mode-vertical-rl)).

When a box has a writing mode that is perpendicular to its containing block it is said to be in, or establish, an <a id="establish-an-orthogonal-flow"></a>orthogonal flow.

To handle this case, CSS layout calculations are divided into two phases: sizing a box, and positioning the box within its flow.

- <a id="ref-for-establish-an-orthogonal-flow"></a>

  <a id="ref-for-block-size①"></a>

  <a id="ref-for-inline-size②"></a>

  In the sizing phase—calculating the width and height of the box—the dimensions of the box and the containing block are mapped to the [inline size](#inline-size) and [block size](#block-size) and calculations are performed accordingly using the writing mode of the box establishing the [orthogonal flow](#establish-an-orthogonal-flow).

- <a id="ref-for-establish-an-orthogonal-flow①"></a>

  <a id="ref-for-block-size②"></a>

  <a id="ref-for-inline-size③"></a>

  In the positioning phase—calculating the positioning offsets, margins, borders, and padding—the dimensions of the box and its containing block are mapped to the [inline size](#inline-size) and [block size](#block-size) and calculations are performed according to the writing mode of the <em>containing block</em> of the box establishing the [orthogonal flow](#establish-an-orthogonal-flow).

<a id="ref-for-establish-an-orthogonal-flow②"></a>

Since auto margins are resolved consistent with the containing block’s writing mode, a box establishing an [orthogonal flow](#establish-an-orthogonal-flow) can, once sized, be aligned or centered within its containing block just like other block-level boxes by using auto margins.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d9cb469c"></a>
>
> ![Diagram of a vertical flow box appearing in between two horizontal flow boxes.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/orthogonal.svg)
>
> An example of orthogonal flow
>
> <a id="ref-for-inline-size④"></a>
>
> <a id="ref-for-block-size③"></a>
>
> For example, if a vertical block is placed inside a horizontal block, then when calculating the physical height (which is the [inline size](#inline-size)) of the child block the physical height of the parent block is used as the child’s containing block <a id="ref-for-inline-size⑤"></a>inline size, even though the physical height is the [block size](#block-size), not the <a id="ref-for-inline-size⑥"></a>inline size, of the parent block.
>
> <a id="ref-for-inline-axis②"></a>
>
> <a id="ref-for-block-axis"></a>
>
> On the other hand, because the containing block is in a horizontal writing mode, the vertical margins on the child participate in margin-collapsing, even though they are in the [inline-axis](#inline-axis) of the child, and horizontal auto margins will expand to fill the containing block, even though they are in the [block-axis](#block-axis) of the child.

<a id="ref-for-inline-axis③"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-max-content"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="orthogonal-shrink-to-fit"></a> Note that this section requires that when a child box auto-sized in its block axis establishes an orthogonal flow, the used block size of the child is calculated to fit its content; and this resulting content-based size is used as input to the [inline-axis](#inline-axis) [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) and [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) of the parent.
>
> <a id="ref-for-block-size④"></a>
>
> <a id="ref-for-inline-size⑦"></a>
>
> This means that when applying shrink-to-fit formula to a box such as an inline-block, float, or table-cell, if its child establishes an orthogonal flow, the calculation dependency must be changed so that the sizing phase of the child runs first and its used [block size](#block-size) becomes an input to the [inline-size](#inline-size) shrink-to-fit formula of the parent.

#### <a id="orthogonal-auto"></a>7.3.1.  Available Space in Orthogonal Flows

<a id="ref-for-inline-size⑧"></a>

<a id="ref-for-block-size⑤"></a>

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-available"></a>

It is common in CSS for a containing block to have a definite [inline size](#inline-size), but not a definite [block size](#block-size). This typically happens in CSS2.1 when a containing block has an [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) height, for example: its width is given by the calculations in [10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth), but its <a id="ref-for-block-size⑥"></a>block size depends on its contents. In such cases the [available inline space](https://www.w3.org/TR/css-sizing-3/#available) is defined as the <a id="ref-for-inline-size⑨"></a>inline size of the containing block; but the <a id="ref-for-available①"></a>available block space, which would otherwise be the <a id="ref-for-block-size⑦"></a>block size of the containing block, is infinite.

<a id="ref-for-establish-an-orthogonal-flow③"></a>

<a id="ref-for-available②"></a>

<a id="ref-for-inline-size①⓪"></a>

<a id="ref-for-inline-axis④"></a>

<a id="ref-for-fallback"></a>

Putting a box in an [orthogonal flow](#establish-an-orthogonal-flow) can result in the opposite: for the box’s [available block space](https://www.w3.org/TR/css-sizing-3/#available) to be definite, but its <a id="ref-for-available③"></a>available inline space to be indefinite. In such cases a percentage of the containing block’s [inline size](#inline-size) cannot be defined, and [inline axis](#inline-axis) computations cannot be resolved. In these cases, an additional constraint is used as a [fallback](https://www.w3.org/TR/css-sizing-3/#fallback) in place of the <a id="ref-for-available④"></a>available inline space for calculations that require a definite <a id="ref-for-available⑤"></a>available inline space—the smallest of

- <a id="ref-for-min-width"></a>

  <a id="ref-for-max-width"></a>

  the size represented by the containing block’s inner [max size](https://www.w3.org/TR/css-sizing-3/#max-width) (if that is fixed) floored by its inner [min size](https://www.w3.org/TR/css-sizing-3/#min-width) (if that is fixed)

- <a id="ref-for-min-width①"></a>

  <a id="ref-for-max-width①"></a>

  <a id="ref-for-scrollport"></a>

  the nearest ancestor [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport)’s inner size if that is fixed, else / capped by its inner [max size](https://www.w3.org/TR/css-sizing-3/#max-width) if that is fixed, floored by its inner [min size](https://www.w3.org/TR/css-sizing-3/#min-width) if that is fixed

- the initial containing block’s size

See [\[CSS3-SIZING\]](#biblio-css3-sizing) for further details on CSS sizing terminology and concepts.

#### <a id="auto-multicol"></a>7.3.2.  Auto-sizing Block Containers in Orthogonal Flows

<a id="ref-for-establish-an-orthogonal-flow④"></a>

<a id="ref-for-inline-size①①"></a>

<a id="ref-for-valdef-width-auto①"></a>

If a box establishing an [orthogonal flow](#establish-an-orthogonal-flow) is either a block container or a multi-column container, for the case where the box’s [inline size](#inline-size) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto):

1.  <a id="ref-for-propdef-column-width"></a>

    <strong>Calculate the used <a href="https://www.w3.org/TR/css3-multicol/#propdef-column-width">column-width</a>:</strong>

    - <a id="ref-for-valdef-column-width-auto"></a>

      <a id="ref-for-propdef-column-width①"></a>

      <a id="ref-for-propdef-column-count"></a>

      If [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count) and [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width) are both [auto](https://www.w3.org/TR/css3-multicol/#valdef-column-width-auto), use the shrink-to-fit formula <code>min(<var>max-content</var>, max(<var>min-content</var>, <var>constraint</var>))</code>, where:

      <var>min-content</var>  
      <a id="ref-for-min-content-inline-size"></a>

      the [min-content inline size](https://www.w3.org/TR/css-sizing-3/#min-content-inline-size) of the box

      <var>max-content</var>  
      <a id="ref-for-max-content-inline-size"></a>

      the [max-content inline size](https://www.w3.org/TR/css-sizing-3/#max-content-inline-size) of the box

      <var>constraint</var>  
      <a id="ref-for-stretch-fit"></a>

      <a id="ref-for-inline-axis⑤"></a>

      the [inline-axis](#inline-axis) size that would [stretch fit](https://www.w3.org/TR/css-sizing-3/#stretch-fit) into to the smallest of

      - <a id="ref-for-min-width②"></a>

        <a id="ref-for-max-width②"></a>

        <a id="ref-for-available⑥"></a>

        the [available space](https://www.w3.org/TR/css-sizing-3/#available): the containing block’s size if that is fixed, else the size represented by the containing block’s inner [max size](https://www.w3.org/TR/css-sizing-3/#max-width) (if that is fixed) floored by its inner [min size](https://www.w3.org/TR/css-sizing-3/#min-width) (if that is fixed)

      - <a id="ref-for-min-width③"></a>

        <a id="ref-for-max-width③"></a>

        <a id="ref-for-scrollport①"></a>

        the nearest ancestor [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport)’s inner size if that is fixed, else / capped by its inner [max size](https://www.w3.org/TR/css-sizing-3/#max-width) if that is fixed, floored by its inner [min size](https://www.w3.org/TR/css-sizing-3/#min-width) if that is fixed

      - the initial containing block’s size

      <a id="ref-for-inline-size①②"></a>

      <a id="ref-for-propdef-column-width②"></a>

      <a id="ref-for-valdef-width-auto②"></a>

      <a id="ref-for-valdef-width-max-content"></a>

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > ![Diagram of automatically triggered multi-column preventing T-shaped documents](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/auto-multicol.svg)
      > <strong>Note that this requirement automatically triggers multi-column flow on all block containers.</strong> This is so that overflowing content, instead of continuing off the side of the containing block, is wrapped into columns in the flow direction of the containing block, thus avoiding T-shaped documents. Authors can control the [inline size](#inline-size) of these columns by setting [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width), or disable this behavior by setting the <a id="ref-for-inline-size①③"></a>inline size property to a non-[auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) value such as [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content).  

    - <a id="ref-for-propdef-column-gap"></a>

      <a id="ref-for-propdef-column-width③"></a>

      <a id="ref-for-valdef-column-count-auto"></a>

      <a id="ref-for-propdef-column-count①"></a>

      If [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count) is not [auto](https://www.w3.org/TR/css3-multicol/#valdef-column-count-auto) and [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width) is <a id="ref-for-valdef-column-count-auto①"></a>auto, calculate the used <a id="ref-for-propdef-column-width④"></a>column-width with the same formula, except replace <var>constraint</var> with <var>constraint</var> − (<a id="ref-for-propdef-column-count②"></a>column-count − 1) × [column-gap](https://www.w3.org/TR/css3-multicol/#propdef-column-gap).

    - <a id="ref-for-valdef-column-count-auto②"></a>

      <a id="ref-for-propdef-column-width⑤"></a>

      <a id="ref-for-propdef-column-count③"></a>

      If [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count) and [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width) are both non-[auto](https://www.w3.org/TR/css3-multicol/#valdef-column-count-auto), the used <a id="ref-for-propdef-column-width⑥"></a>column-width is the computed <a id="ref-for-propdef-column-width⑦"></a>column-width. (This is not recommended as it can cause overflow; instead it is better to set the <a id="ref-for-propdef-column-width⑧"></a>column-width and a max-block-size.)

2.  <a id="ref-for-max-content-block-size"></a>

    <a id="ref-for-stretch-fit-block-size"></a>

    <a id="ref-for-definite"></a>

    <a id="ref-for-valdef-column-width-auto①"></a>

    <a id="ref-for-propdef-column-width⑨"></a>

    <a id="ref-for-propdef-column-count④"></a>

    <a id="ref-for-valdef-width-auto③"></a>

    <a id="ref-for-block-size⑧"></a>

    <strong>Calculate the used column length:</strong> If the computed [block size](#block-size) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) and the specified value of at least one of [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count) and [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width) was [auto](https://www.w3.org/TR/css3-multicol/#valdef-column-width-auto), use the box’s <a id="ref-for-block-size⑨"></a>block size (if that is [definite](https://www.w3.org/TR/css-sizing-3/#definite)), else the [stretch-fit block size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-block-size) of the box (if that is <a id="ref-for-definite①"></a>definite), else the box’s [max-content block size](https://www.w3.org/TR/css-sizing-3/#max-content-block-size). Otherwise follow the normal rules for sizing a multi-column container.

    <a id="ref-for-min-content-block-size"></a>

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-fae7187e"></a> Should we factor in the box’s [min-content block size](https://www.w3.org/TR/css-sizing-3/#min-content-block-size) in this formula, so that e.g. a large image will not overflow the box, but cause the box to overflow the containing block?

3.  <a id="ref-for-valdef-column-count-auto③"></a>

    <a id="ref-for-propdef-column-count⑥"></a>

    <a id="ref-for-propdef-column-count⑤"></a>

    <strong>Calculate the used <a href="https://www.w3.org/TR/css3-multicol/#propdef-column-count">column-count</a>:</strong> If the computed [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count) is [auto](https://www.w3.org/TR/css3-multicol/#valdef-column-count-auto), then the used <a id="ref-for-propdef-column-count⑦"></a>column-count follows from filling the resulting multi-column layout with the box’s content.

<a id="ref-for-inline-size①④"></a>

The used [inline size](#inline-size) of the resulting multi-column container is then calculated:

1.  <a id="ref-for-max-content-inline-size①"></a>

    <a id="ref-for-inline-size①⑤"></a>

    If the content neither line-wraps nor fragments within the multi-column container, then the used [inline size](#inline-size) is the [max-content inline size](https://www.w3.org/TR/css-sizing-3/#max-content-inline-size) of the box’s contents. This criteria gives the shrink-to-fit behavior for short orthogonal flow contents without making a large blank space.

2.  <a id="ref-for-propdef-column-gap①"></a>

    <a id="ref-for-propdef-column-count⑧"></a>

    <a id="ref-for-propdef-column-width①⓪"></a>

    Otherwise it is calculated from the used [column-width](https://www.w3.org/TR/css3-multicol/#propdef-column-width), [column-count](https://www.w3.org/TR/css3-multicol/#propdef-column-count), and [column-gap](https://www.w3.org/TR/css3-multicol/#propdef-column-gap).

<a id="ref-for-block-size①⓪"></a>

<a id="ref-for-max-content-block-size①"></a>

<a id="ref-for-available⑦"></a>

The used [block size](#block-size) of the box is either the used column length (if multiple columns were used) or the [max-content block size](https://www.w3.org/TR/css-sizing-3/#max-content-block-size) of the content (if only one column was used). If the UA does not support CSS Multi-column Layout [\[CSS3COL\]](#biblio-css3col), the UA may instead calculate the box’s <a id="ref-for-block-size①①"></a>block size assuming infinite [available block space](https://www.w3.org/TR/css-sizing-3/#available), thus laying out its contents into a single column. (Note that this can, however, result in content that is clipped or otherwise inaccessible if it overflows its containing block.)

> <strong data-conversion-semantic="note">Note</strong>
>
> The automatic triggering of multi-column flow is at-risk and may be dropped during CR.

#### <a id="orthogonal-layout"></a>7.3.3.  Auto-sizing Other Orthogonal Flow Roots

<a id="ref-for-available⑧"></a>

<a id="ref-for-establish-an-orthogonal-flow⑤"></a>

In order to limit the length of lines, block containers have special auto-sizing behavior (defined [above](#auto-multicol)) when their [available inline space](https://www.w3.org/TR/css-sizing-3/#available) is infinite (which typically occurs when they establish an [orthogonal flow](#establish-an-orthogonal-flow)).

<a id="ref-for-available⑨"></a>

<a id="ref-for-max-content①"></a>

<a id="ref-for-establish-an-orthogonal-flow⑥"></a>

Other layout models simply lay out into the infinite [available inline space](https://www.w3.org/TR/css-sizing-3/#available) at their [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content). However, they pass through the infinite <a id="ref-for-available①⓪"></a>available inline space to block containers they contain, possibly triggering that special auto-sizing behavior on those block containers even though they do not themselves establish an [orthogonal flow](#establish-an-orthogonal-flow).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e2cd91dc"></a>
>
> <a id="ref-for-flex-container"></a>
>
> <a id="ref-for-establish-an-orthogonal-flow⑦"></a>
>
> <a id="ref-for-available①①"></a>
>
> <a id="ref-for-max-content②"></a>
>
> <a id="ref-for-flex-item"></a>
>
> <a id="ref-for-block-container③"></a>
>
> For example, a table or [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container) establishing an [orthogonal flow](#establish-an-orthogonal-flow) is laid out into its given [available space](https://www.w3.org/TR/css-sizing-3/#available). If its <a id="ref-for-available①②"></a>available inline space is infinite, this effectively lays the box out at its [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content). However, any of its table cells or [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) that are [block containers](https://www.w3.org/TR/css-display-3/#block-container) are laid out assuming infinite <a id="ref-for-available①③"></a>available inline space and so behave accordingly.

#### <a id="orthogonal-pagination"></a>7.3.4.  Fragmenting Orthogonal Flows

<em>This section is informative.</em>

With regards to fragmentation, the rules in CSS2.1 still hold in vertical writing modes and orthogonal flows: break opportunities do not occur inside line boxes, only between them. UAs that support [\[CSS3COL\]](#biblio-css3col) may break in the (potentially zero-width) gap between columns, however.

Note that if content spills outside the pagination stream established by the root element, the UA is not required to print such content. Authors wishing to mix writing modes with long streams of text are thus encouraged to use CSS columns to keep all content flowing in the document’s pagination direction.

> <strong data-conversion-semantic="note">Note</strong>
>
> In other words, if your document would require two scrollbars on the screen it probably won’t all print. Fix your layout, e.g. by using [columns](https://www.w3.org/TR/css3-multicol/) so that it all scrolls (and therefore paginates) in one direction if you want to make sure it’ll all print. T-shaped documents tend not to print well.

### <a id="logical-direction-layout"></a>7.4.  Flow-Relative Mappings

<a id="ref-for-propdef-float"></a>

<a id="ref-for-propdef-clear"></a>

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-bottom"></a>

<a id="ref-for-propdef-left"></a>

<a id="ref-for-propdef-right"></a>

<a id="ref-for-propdef-caption-side"></a>

<a id="ref-for-block-start⑧"></a>

<a id="ref-for-block-end⑥"></a>

Flow-relative directions are calculated with respect to the writing mode of the <em>containing block</em> of the box and used to abstract layout rules related to the box properties (margins, borders, padding) and any properties related to positioning the box within its containing block ([float](https://www.w3.org/TR/CSS21/visuren.html#propdef-float), [clear](https://www.w3.org/TR/CSS21/visuren.html#propdef-clear), [top](https://www.w3.org/TR/CSS21/visuren.html#propdef-top), [bottom](https://www.w3.org/TR/CSS21/visuren.html#propdef-bottom), [left](https://www.w3.org/TR/CSS21/visuren.html#propdef-left), [right](https://www.w3.org/TR/CSS21/visuren.html#propdef-right), [caption-side](https://www.w3.org/TR/CSS21/tables.html#propdef-caption-side)). For inline-level boxes, the writing mode of the <em>parent
    box</em> is used instead. (The left/right/top/bottom-named properties and values themselves are still mapped physically; with a special exception made for <a id="ref-for-propdef-caption-side①"></a>caption-side, whose top/top-outside and bottom/bottom-outside values are associated to the [block-start](#block-start) and [block-end](#block-end) sides of the table, respectively.)

For example, the margin that is dropped when a box’s inline dimension is [over-constrained](https://www.w3.org/TR/CSS2/visudet.html#blockwidth) is the end margin as determined by the writing mode of the containing block.

<a id="ref-for-block-start⑨"></a>

<a id="ref-for-block-end⑦"></a>

<a id="ref-for-block-start①⓪"></a>

<a id="ref-for-block-end⑧"></a>

The [margin collapsing rules](https://www.w3.org/TR/CSS2/box.html#collapsing-margins) apply exactly with the <em><a href="#block-start">block-start</a> margin</em> substituted for the top margin and the <em><a href="#block-end">block-end</a> margin</em> substituted for the bottom margin. Similarly the [block-start](#block-start) padding and border are substituted for the top padding and border, and the [block-end](#block-end) padding and border substituted for the bottom padding and border. Note this means only <a id="ref-for-block-start①①"></a>block-start and <a id="ref-for-block-end⑨"></a>block-end margins ever collapse.

Flow-relative directions are calculated with respect to the writing mode of the box and used to abstract layout related to the box’s contents:

- <a id="ref-for-start⑤"></a>

  <a id="ref-for-propdef-text-align②"></a>

  The initial value of the [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) property aligns to the [start](#start) edge of the line box.

- <a id="ref-for-start⑥"></a>

  <a id="ref-for-propdef-text-indent"></a>

  The [text-indent](https://www.w3.org/TR/css-text-3/#propdef-text-indent) property indents from the [start](#start) edge of the line box.

- <a id="ref-for-block-start①②"></a>

  <a id="ref-for-inline-start⑥"></a>

  For tables, the ordering of columns begins on the [inline-start](#inline-start) side of the table, and the ordering of rows begins on the [block-start](#block-start) side of the table.

### <a id="line-mappings"></a>7.5.  Line-Relative Mappings

<a id="ref-for-line-relative-direction"></a>

<a id="ref-for-over③"></a>

<a id="ref-for-under③"></a>

<a id="ref-for-line-left⑥"></a>

<a id="ref-for-line-right⑤"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb⑨"></a>

The [line-relative directions](#line-relative-direction) are [over](#over), [under](#under), [line-left](#line-left), and [line-right](#line-right). In an LTR [horizontal-tb](#valdef-writing-mode-horizontal-tb) writing mode, they correspond to the top, bottom, left, and right directions, respectively.

<a id="ref-for-line-right⑥"></a>

<a id="ref-for-line-left⑦"></a>

The [line-right](#line-right) and [line-left](#line-left) directions are calculated with respect to the writing mode of the box and used to interpret the left and right values of the following properties:

- <a id="ref-for-propdef-text-align③"></a>

  the [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) property [\[CSS2\]](#biblio-css2)

<a id="ref-for-line-right⑦"></a>

<a id="ref-for-line-left⑧"></a>

The [line-right](#line-right) and [line-left](#line-left) directions are calculated with respect to the writing mode of the <em>containing
    block</em> of the box and used to interpret the left and right values of the following properties:

- <a id="ref-for-propdef-float①"></a>

  the [float](https://www.w3.org/TR/CSS21/visuren.html#propdef-float) property [\[CSS2\]](#biblio-css2)

- <a id="ref-for-propdef-clear①"></a>

  the [clear](https://www.w3.org/TR/CSS21/visuren.html#propdef-clear) property [\[CSS2\]](#biblio-css2)

- <a id="ref-for-propdef-caption-side②"></a>

  the [caption-side](https://www.w3.org/TR/CSS21/tables.html#propdef-caption-side) property [\[CSS2\]](#biblio-css2)

<a id="ref-for-over④"></a>

<a id="ref-for-under④"></a>

The [over](#over) and [under](#under) directions are calculated with respect to the writing mode of the box and used to define the interpretation of the "top" (over) and "bottom" (under) sides of the line box as follows:

- <a id="ref-for-line-over③"></a>

  <a id="ref-for-over⑤"></a>

  <a id="ref-for-propdef-vertical-align⑦"></a>

  For the [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property, the "top" of the line box is its [over](#over) edge; the "bottom" of the line box is its under edge. Positive length and percentage values shift the baseline towards the [line-over](#line-over) edge. [\[CSS2\]](#biblio-css2)

- <a id="ref-for-over⑥"></a>

  <a id="ref-for-under⑤"></a>

  <a id="ref-for-propdef-text-decoration"></a>

  For the [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration) property, the underline is drawn on the [under](#under) side of the text; the overline is drawn on the [over](#over) side of the text. [\[CSS2\]](#biblio-css2) <strong data-conversion-semantic="note">Note:</strong> Note that the CSS Text Decoration Module defines this in more detail and provides additional controls for controlling the position of underlines and overlines. [\[CSS3-TEXT-DECOR\]](#biblio-css3-text-decor)

### <a id="physical-only"></a>7.6.  Purely Physical Mappings

The following values are purely physical in their definitions and do not respond to changes in writing mode:

- <a id="ref-for-propdef-clip"></a>

  <a id="ref-for-funcdef-rect"></a>

  the [rect()](https://www.w3.org/TR/css-masking-1/#funcdef-rect) notation of the [clip](https://www.w3.org/TR/CSS21/visufx.html#propdef-clip) property [\[CSS2\]](#biblio-css2)

- the background properties [\[CSS2\]](#biblio-css2) [\[CSS3BG\]](#biblio-css3bg)

- the border-image properties [\[CSS3BG\]](#biblio-css3bg)

- <a id="ref-for-propdef-text-shadow"></a>

  <a id="ref-for-propdef-box-shadow"></a>

  the offsets of the [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow) and [text-shadow](https://www.w3.org/TR/css-text-decor-3/#propdef-text-shadow) properties

## <a id="principal-flow"></a>8.  The Principal Writing Mode

<a id="ref-for-used-value③"></a>

<a id="ref-for-propdef-writing-mode②③"></a>

<a id="ref-for-propdef-direction③③"></a>

<a id="ref-for-propdef-text-orientation①④"></a>

<a id="ref-for-page-progression"></a>

The <a id="principal-writing-mode"></a>principal writing mode of the document is determined by the [used](https://www.w3.org/TR/css-cascade-4/#used-value) [writing-mode](#propdef-writing-mode), [direction](#propdef-direction), and [text-orientation](#propdef-text-orientation) values of the root element. This writing mode is used, for example, to determine the direction of scrolling and the default [page progression](https://www.w3.org/TR/css3-page/#page-progression) direction.

<a id="ref-for-the-body-element"></a>

<a id="ref-for-used-value④"></a>

<a id="ref-for-propdef-writing-mode②④"></a>

<a id="ref-for-propdef-direction③④"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-propdef-text-orientation①⑤"></a>

As a special case for handling HTML documents, if the root element has a <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> child element [\[HTML\]](#biblio-html), the [used value](https://www.w3.org/TR/css-cascade-4/#used-value) of the of [writing-mode](#propdef-writing-mode) and [direction](#propdef-direction) properties on root element are taken from the [computed](https://www.w3.org/TR/css-cascade-4/#computed-value) <a id="ref-for-propdef-writing-mode②⑤"></a>writing-mode and <a id="ref-for-propdef-direction③⑤"></a>direction of the first such child element instead of from the root element’s own values. The UA <em>may</em> also propagate the value of [text-orientation](#propdef-text-orientation) in this manner. Note that this does not affect the computed values of <a id="ref-for-propdef-writing-mode②⑥"></a>writing-mode, <a id="ref-for-propdef-direction③⑥"></a>direction, or <a id="ref-for-propdef-text-orientation①⑥"></a>text-orientation of the root element itself.

<a id="ref-for-inheritance"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Propagation is done on used values rather than computed values to avoid disrupting other aspects of style computation, such as [inheritance](https://www.w3.org/TR/css-cascade-4/#inheritance), [logical property mapping logic](https://www.w3.org/TR/css-logical-1/#box), or [length value computation](https://www.w3.org/TR/css-values-4/#lengths).

### <a id="icb"></a>8.1.  Propagation to the Initial Containing Block

<a id="ref-for-principal-writing-mode"></a>

<a id="ref-for-initial-containing-block"></a>

The [principal writing mode](#principal-writing-mode) is propagated to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block) and to the viewport, thereby affecting the layout of the root element and the scrolling direction of the viewport.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-80e9901b"></a> Does it also propagate to @page boxes?

### <a id="page-direction"></a>8.2.  Page Flow: the page progression direction

<a id="ref-for-page-progression①"></a>

<a id="ref-for-principal-writing-mode①"></a>

In paged media CSS classifies all pages as either left or right pages. The [page progression](https://www.w3.org/TR/css3-page/#page-progression) direction (see [\[CSS3PAGE\]](#biblio-css3page)), which determines whether the left or right page in a spread is first in the flow and whether the first page is by default a left or right page, depends on the [principal writing mode](#principal-writing-mode) as follows:

<strong>Table 12 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<a id="ref-for-principal-writing-mode②"></a>

[principal writing mode](#principal-writing-mode)

<strong>Column 2 (header cell):</strong>

<a id="ref-for-page-progression②"></a>

[page progression](https://www.w3.org/TR/css3-page/#page-progression)

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-direction-ltr⑨"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb①⓪"></a>

[horizontal-tb](#valdef-writing-mode-horizontal-tb) and [ltr](#valdef-direction-ltr)

<strong>Column 2 (data cell):</strong>

left-to-right

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-direction-rtl⑨"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb①①"></a>

[horizontal-tb](#valdef-writing-mode-horizontal-tb) and [rtl](#valdef-direction-rtl)

<strong>Column 2 (data cell):</strong>

right-to-left

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-writing-mode-sideways-rl②"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl①②"></a>

[vertical-rl](#valdef-writing-mode-vertical-rl) or [sideways-rl](#valdef-writing-mode-sideways-rl)

<strong>Column 2 (data cell):</strong>

right-to-left

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-valdef-writing-mode-sideways-lr③"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr⑦"></a>

[vertical-lr](#valdef-writing-mode-vertical-lr) or [sideways-lr](#valdef-writing-mode-sideways-lr)

<strong>Column 2 (data cell):</strong>

left-to-right

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unless otherwise overridden, the first page of a document begins on the second half of a spread, e.g. on the right page in a left-to-right page progression.

## <a id="text-combine"></a>9.  Glyph Composition

<a id="text-combine-horizontal"></a>

<a id="ref-for-propdef-text-combine-upright①"></a>

### <a id="text-combine-upright"></a>9.1.  Horizontal-in-Vertical Composition: the [text-combine-upright](#propdef-text-combine-upright) property

<strong>Table 13 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-text-combine-upright"></a>text-combine-upright

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://drafts.csswg.org/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-one①⑥"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) all <a id="ref-for-comb-one①⑦"></a>\| \[ digits \<integer\>[?](https://www.w3.org/TR/css-values-4/#mult-opt) \]

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://drafts.csswg.org/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Applies to:

<strong>Column 2 (data cell):</strong>

non-replaced inline elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://drafts.csswg.org/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

yes

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://drafts.csswg.org/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://drafts.csswg.org/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword, plus integer if digits

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

Canonical order:

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://drafts.csswg.org/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-typographic-character-unit⑧"></a>

This property specifies the combination of multiple [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) into the space of a single <a id="ref-for-typographic-character-unit⑨"></a>typographic character unit. If the combined text is wider than 1em, the UA must fit the contents within 1em, see below. The resulting composition is treated as a single upright glyph for the purposes of layout and decoration. This property only has an effect in vertical writing modes. Values have the following meanings:

<a id="valdef-text-combine-upright-none"></a>none

No special processing.

<a id="valdef-text-combine-upright-all"></a>all

<a id="ref-for-typographic-character-unit①⓪"></a>

Attempt to typeset horizontally all consecutive [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) within the box such that they take up the space of a single <a id="ref-for-typographic-character-unit①①"></a>typographic character unit within the vertical line box.

<a id="ref-for-integer-value①"></a>

<a id="valdef-text-combine-upright-digits-integer"></a>digits [\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)?

<a id="ref-for-typographic-character-unit①②"></a>

Attempt to typeset horizontally each maximal sequence of consecutive ASCII digits (U+0030–U+0039) that has as many or fewer digits than the specified integer such that it takes up the space of a single [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) within the vertical line box. If the integer is omitted, it computes to 2. Integers outside the range 2-4 are invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-57d55bfa"></a>
>
> In East Asian documents, the text-combine-upright effect is often used to display Latin-based strings such as components of a date or letters of an initialism, always in a horizontal writing mode regardless of the writing mode of the line:
>
> ![Diagram of tate-chu-yoko, showing the two digits of a date set halfwidth side-by-side in a vertical column of text](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/tate-chu-yoko.png)
>
> Example of horizontal-in-vertical <i lang="ja">tate-chu-yoko</i>
>
> The figure is the result of the rules
>
> ```text
> date { text-combine-upright: digits 2; }
> ```
>
> and the following markup:
>
> ```text
> <date>平成20年4月16日に</date>
> ```
>
> In Japanese, this effect is known as <i lang="ja">tate-chu-yoko</i>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-feb64026"></a>
>
> <a id="ref-for-propdef-text-combine-upright②"></a>
>
> The following example shows that applying [text-combine-upright: digits 2](#propdef-text-combine-upright) to an entire document, rather than to a segment with a known type of numeric content, can have unintended consequences:
>
> ```text
> <p>あれは10,000円ですよ！</p>
> ```
>
> ![Rendering of the above markup with 'text-combine-upright: digits': the first two digits of the number are rendered as tate-chu-yoko while the rest of the number is rendered sideways.](https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/images/bad-tate-chu-yoko.png)
>
> Example of mis-applied <i lang="ja">tate-chu-yoko</i>

#### <a id="text-combine-runs"></a>9.1.1.  Text Run Rules

<a id="ref-for-propdef-text-combine-upright③"></a>

<a id="ref-for-typographic-character-unit①③"></a>

To avoid complexity in the rendering and layout, [text-combine-upright](#propdef-text-combine-upright) can only combine plain text: consecutive [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) that are not interrupted by a box boundary.

<a id="ref-for-propdef-text-combine-upright④"></a>

<a id="ref-for-valdef-text-combine-upright-none"></a>

However, because the property inherits, the UA must ensure that the contents of the box effecting the combination are not part of an otherwise-combinable sequence that happens to begin or end outside the box; if so, then the text is laid out normally, as if [text-combine-upright](#propdef-text-combine-upright) were [none](#valdef-text-combine-upright-none). To avoid combining only part of a sequence: if the boundary of a potentially-combinable run is due only to one or more inline box boundaries, the UA must inspect any characters that appear immediately before and immediately after the run, and if these characters would, without the intervening box, form a sequence that would (if it were not too long) combine, then the candidate run does not combine.

> <strong data-conversion-semantic="note">Note</strong>
>
> The above paragraph is at-risk. Comments from implementors are welcome.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-73e9bf5b"></a>
>
> For example, given the rule
>
> ```text
> tcy { text-combine-upright: digits 4; }
> ```
>
> if the following markup were given:
>
> ```text
> <tcy>12<span>34</span></tcy>
> ```
>
> <a id="ref-for-propdef-text-combine-upright⑤"></a>
>
> no text would combine: the 12 and 34 both share an ancestor with the same [text-combine-upright](#propdef-text-combine-upright) value, and therefore are considered part of a sequence of four combinable digits interrupted by a box boundary. However in these cases:
>
> ```text
> 12<tcy><span>34</span></tcy>12<tcy><span></span>34</tcy>
> 12<tcy>34<span></span></tcy>
> ```
>
> <a id="ref-for-propdef-text-combine-upright⑥"></a>
>
> The 34 would combine, because the 12 immediately previous does not share with the 34 an ancestor with a common [text-combine-upright](#propdef-text-combine-upright), and therefore the 34 is considered to be the entirety of a sequence of two combinable digits.
>
> If we used the rule
>
> ```text
> tcy { text-combine-upright: all; }
> ```
>
> the same results would occur: the first case not combining because 1234 forms a sequence of four combinable characters interrupted by a box boundary, and the second combining 34 because it forms the entirety of a sequence of two combinable characters.

<a id="ref-for-propdef-text-combine-upright⑦"></a>

<a id="ref-for-valdef-text-combine-upright-all"></a>

<a id="ref-for-typographic-character-unit①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the value of [text-combine-upright](#propdef-text-combine-upright) ([all](#valdef-text-combine-upright-all) or digits) only affects which types of [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) can be combined and what is the maximum length of a combinable sequence. It does not otherwise change behavior.

#### <a id="text-combine-layout"></a>9.1.2.  Layout Rules

<a id="ref-for-propdef-text-combine-upright⑧"></a>

<a id="ref-for-bidi-isolate①"></a>

<a id="ref-for-propdef-letter-spacing"></a>

<a id="ref-for-valdef-display-inline-block①"></a>

<a id="ref-for-horizontal-writing-mode①"></a>

<a id="ref-for-propdef-line-height①"></a>

<a id="ref-for-white-space"></a>

When combining text as for [text-combine-upright: all](#propdef-text-combine-upright), the glyphs of the combined text are [bidi-isolated](#bidi-isolate) and composed horizontally (ignoring [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) and any forced line breaks, but using the specified font settings), similar to the contents of an [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block) box with a [horizontal writing mode](#horizontal-writing-mode) and a [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) of 1em. Any [document white space](https://www.w3.org/TR/css-text-3/#white-space) included at the start/end of the combined text <em>is</em> [processed](https://www.w3.org/TR/css-text-3/#white-space-processing) [\[CSS-TEXT-3\]](#biblio-css-text-3) as at the start/end of such an inline block. The effective size of the composition is assumed to be 1em square; anything outside the square is not measured for layout purposes. The UA should center the glyphs horizontally and vertically within the measured 1em square.

<a id="ref-for-propdef-vertical-align⑧"></a>

<a id="ref-for-typographic-character-unit①⑤"></a>

<a id="ref-for-propdef-text-orientation①⑦"></a>

The baseline of the resulting composition must be chosen such that the square is centered between the text-over and text-under baselines of its parent inline box prior to any baseline alignment shift ([vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align)). For bidi reordering, the composition is treated the same as a [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit) with [text-orientation: upright](#propdef-text-orientation). For line breaking before and after the composition, it is treated as a regular inline with its actual contents. For other text layout purposes, e.g. emphasis marks, text-decoration, spacing, etc. the resulting composition is treated as a single glyph representing the Object Replacement Character U+FFFC.

#### <a id="text-combine-compression"></a>9.1.3.  Compression Rules

<a id="ref-for-typographic-character-unit①⑥"></a>

The UA must ensure that the combined advance width of the composition fits within 1em by compressing the combined text if necessary. (This does not necessarily mean that the glyphs will fit within 1em, as some glyphs are designed to draw outside their geometric boundaries.) OpenType implementations <em>must</em> use width-specific variants (OpenType features `hwid`/`twid`/`qwid`; other glyph-width features such as `fwid` or `pwid` are not included) to compress text in cases where those variants are available for all [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) in the composition. Otherwise, the UA may use any means to compress the text, including substituting half-width, third-width, and/or quarter-width glyphs provided by the font, using other font features designed to compress text horizontally, scaling the text geometrically, or any combination thereof.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-da304b32"></a>
>
> For example, a simple OpenType-based implementation might compress the text as follows:
>
> 1.  <a id="ref-for-typographic-character-unit①⑦"></a>
>
>     Enable 1/<var>n</var>-width glyphs for combined text of <var>n</var> [typographic character units](https://www.w3.org/TR/css-text-3/#typographic-character-unit) (i.e. use OpenType `hwid` for 2 <a id="ref-for-typographic-character-unit①⑧"></a>typographic character units, `twid` for 3 <a id="ref-for-typographic-character-unit①⑨"></a>typographic character units, etc.) if the number of <a id="ref-for-typographic-character-unit②⓪"></a>typographic character units \> 1. Note that the number of <a id="ref-for-typographic-character-unit②①"></a>typographic character units ≠ number of Unicode codepoints!
>
> 2.  If the result is wider than 1em, horizontally scale the result to 1em.
>
> A different implementation that utilizes OpenType layout features might compose the text first with normal glyphs to see if that fits, then substitute in half-width or third-width forms as available and necessary, possibly adjusting its approach or combining it with scaling operations depending on the available glyph substitutions.

In some fonts, the ideographic glyphs are given a compressed design such that they are 1em wide but shorter than 1em tall. To accommodate such fonts, the UA may vertically scale the composition to match the advance height of 水 U+6C34 as rendered according to the specified font settings. In such a case the resulting composition assumes the advance height of 水 U+6C34 rather than 1em.

##### <a id="text-combine-fullwidth"></a>9.1.3.1.  Full-width Characters

<a id="ref-for-typographic-character-unit②②"></a>

<a id="ref-for-propdef-text-transform"></a>

In order to preserve typographic color when compressing the text to 1em, when the combined text consists of more than one [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit), then any full-width <a id="ref-for-typographic-character-unit②③"></a>typographic character units should first be converted to their non-full-width equivalents by reversing the algorithm defined for [text-transform: full-width](https://www.w3.org/TR/css-text-3/#propdef-text-transform) in [\[CSS-TEXT-3\]](#biblio-css-text-3) before applying other compression techniques.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b52a6513"></a>
>
> <a id="ref-for-propdef-text-transform①"></a>
>
> <a id="ref-for-propdef-text-combine-upright⑨"></a>
>
> For example, an author might apply both [text-transform](https://www.w3.org/TR/css-text-3/#propdef-text-transform) and [text-combine-upright](#propdef-text-combine-upright) to a date set in vertical text.
>
> ```text
> date { text-combine-upright: digits 2; text-transform: full-width; }
> ```
>
> Suppose this style rule is applied to a date such as.
>
> ```text
> <date>2010年2月23日</date>
> ```
>
> <a id="ref-for-typographic-character-unit②④"></a>
>
> <a id="ref-for-propdef-text-transform②"></a>
>
> The "2010" is too long to be combined (4 digits), but the "2" and "23" will be affected. Since "23" is more than one [typographic character unit](https://www.w3.org/TR/css-text-3/#typographic-character-unit), it will not be affected by [text-transform: full-width](https://www.w3.org/TR/css-text-3/#propdef-text-transform). However since the "2" is only one <a id="ref-for-typographic-character-unit②⑤"></a>typographic character unit, it will be transformed to a fullwidth "２". Since the "2010" was not combined, its digits, too, will be transformed to fullwidth "２０１０"; and being fullwidth, they will be typeset upright, giving the following result:
>
> ```text
> ２
> ０
> １
> ０
> 年
> ２
> 月
> 23
> 日
> ```
<a id="ref-for-propdef-font-variant"></a>

<a id="ref-for-propdef-font-feature-settings"></a>

<a id="ref-for-propdef-text-combine-upright①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Properties that affect glyph selection, such as the [font-variant](https://www.w3.org/TR/css-fonts-3/#propdef-font-variant) and [font-feature-settings](https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings) properties defined in [\[CSS3-FONTS\]](#biblio-css3-fonts), can potentially affect the selection of variants for characters included in combined text runs. Authors are advised to use these properties with care when [text-combine-upright](#propdef-text-combine-upright) is also used.

## <a id="priv-sec"></a>10. Privacy and Security Considerations

This specification introduces no new privacy leaks, or security considerations beyond “implement it correctly”.

## <a id="changes"></a>Changes

### <a id="changes-201805"></a> Changes since the [May 2018 CSS Writing Modes Module Level 4 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-writing-modes-4-20180524)

- <a id="ref-for-propdef-text-orientation①⑧"></a>

  Clarified that propagation of the principal writing mode from the body element to the initial containing block and viewport does affect the used value on root element as well, but not its computed value. Also, optionally allow propagating [text-orientation](#propdef-text-orientation) as well. This change was also applied to level 3. ([Issue 3066](https://github.com/w3c/csswg-drafts/issues/3066))

- <a id="ref-for-propdef-text-combine-upright①①"></a>

  Clarified that white space within—particularly at the start/end of—a [text-combine-upright](#propdef-text-combine-upright) combined text sequence is processed the same way as in an inline block. ([Issue 4139](https://github.com/w3c/csswg-drafts/issues/4139)) This change was also applied to level 3.

  > <a id="ref-for-propdef-text-combine-upright①②"></a>
  >
  > <a id="ref-for-bidi-isolate②"></a>
  >
  > <a id="ref-for-propdef-letter-spacing①"></a>
  >
  > <a id="ref-for-valdef-display-inline-block②"></a>
  >
  > <a id="ref-for-horizontal-writing-mode②"></a>
  >
  > <a id="ref-for-propdef-line-height②"></a>
  >
  > <a id="ref-for-white-space①"></a>
  >
  > When combining text as for [text-combine-upright: all](#propdef-text-combine-upright), the glyphs of the combined text are [bidi-isolated](#bidi-isolate) and composed horizontally (ignoring [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing) and any forced line breaks, but using the specified font settings), similar to the contents of an [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block) box with a [horizontal writing mode](#horizontal-writing-mode) and a [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height) of 1em. <u>Any [document white space](https://www.w3.org/TR/css-text-3/#white-space) included at the start/end of the combined text <em>is</em> [processed](https://www.w3.org/TR/css-text-3/#white-space-processing) [\[CSS-TEXT-3\]](#biblio-css-text-3) as at the start/end of such an inline block.</u>

### <a id="additions"></a> New in Level 4

This module is simply a copy of the [2015 Candidate Recommendation of CSS Writing Modes Level 3](https://www.w3.org/TR/2015/CR-css-writing-modes-3-20151215/). The difference from the [current CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/) is the set of features that were deferred from Level 3 due to later implementation uptake:

- <a id="ref-for-propdef-writing-mode②⑦"></a>

  <a id="ref-for-valdef-writing-mode-sideways-rl③"></a>

  <a id="ref-for-valdef-writing-mode-sideways-lr④"></a>

  Re-introduced [sideways-lr](#valdef-writing-mode-sideways-lr) and [sideways-rl](#valdef-writing-mode-sideways-rl) values of [writing-mode](#propdef-writing-mode)

- <a id="ref-for-propdef-text-combine-upright①③"></a>

  Re-introduced the digits value of [text-combine-upright](#propdef-text-combine-upright).

- Re-introduced [automatic multi-column behavior](#auto-multicol) of orthogonal flows.

- Clarified the [conditions for bidi reordering-induced fragmentation](#bidi-fragmentation). ([Issue 1509](https://github.com/w3c/csswg-drafts/issues/1509))

## <a id="acknowledgements"></a> Acknowledgements

L. David Baron, Brian Birtles, James Clark, John Daggett, Nami Fujii, Daisaku Hataoka, Martin Heijdra, Laurentiu Iancu, Richard Ishida, Jonathan Kew, Yasuo Kida, Tatsuo Kobayashi, Toshi Kobayashi, Ken Lunde, Shunsuke Matsuki, Nat McCully, Eric Muller, Paul Nelson, Kenzou Onozawa, Chris Pratley, Xidorn Quan, Florian Rivoal, Dwayne Robinson, Simon Sapin, Marcin Sawicki, Dirk Schulze, Hajime Shiozawa, Alan Stearns, Michel Suignard, Takao Suzuki, Gérard Talbot, Masataka Yakura, Taro Yamamoto, Steve Zilles

## <a id="script-orientations"></a>Appendix A: Vertical Scripts in Unicode

<em>This section is informative.</em>

<a id="ref-for-vertical-only①"></a>

<a id="ref-for-bi-orientational②"></a>

<a id="ref-for-horizontal-only②"></a>

This appendix lists the [vertical-only](#vertical-only) and [bi-orientational](#bi-orientational) scripts in Unicode 6.0 [\[UNICODE\]](#biblio-unicode) and their transformation from horizontal to vertical orientation. Any script not listed explicitly is assumed to be [horizontal-only](#horizontal-only). The script classification of Unicode characters is given by [\[UAX24\]](#biblio-uax24).

| Code | Name                 | Transform (Clockwise) | Vertical Intrinsic Direction |
|------|----------------------|-----------------------|------------------------------|
| Bopo | Bopomofo             | 0°                    | ttb                          |
| Egyp | Egyptian Hieroglyphs | 0°                    | ttb                          |
| Hira | Hiragana             | 0°                    | ttb                          |
| Kana | Katakana             | 0°                    | ttb                          |
| Hani | Han                  | 0°                    | ttb                          |
| Hang | Hangul               | 0°                    | ttb                          |
| Merc | Meroitic Cursive     | 0°                    | ttb                          |
| Mero | Meroitic Hieroglyphs | 0°                    | ttb                          |
| Mong | Mongolian            | 90°                   | ttb                          |
| Ogam | Ogham                | -90°                  | btt                          |
| Orkh | Old Turkic           | -90°                  | ttb                          |
| Phag | Phags Pa             | 90°                   | ttb                          |
| Yiii | Yi                   | 0°                    | ttb                          |

Vertical Scripts in Unicode

<a id="ref-for-horizontal-script"></a>

<strong>Exceptions:</strong> For the purposes of this specification, all fullwidth (F) and wide (W) characters are treated as belonging to a vertical script, and halfwidth characters (H) are treated as belonging to a [horizontal script](#horizontal-script). [\[UAX11\]](#biblio-uax11)

<a id="ref-for-vertical-only②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that for [vertical-only](#vertical-only) characters (such as Mongolian and Phags Pa letters), the glyphs in the Unicode code charts are shown in their vertical orientation. In horizontal text, they are typeset in a 90° counter-clockwise rotation from this orientation.

<a id="ref-for-valdef-text-orientation-mixed⑧"></a>

<a id="ref-for-valdef-writing-mode-sideways-lr⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Due to limitations in the current featureset of Unicode Technical Report 50 and CSS Writing Modes, vertical [mixed](#valdef-text-orientation-mixed) typesetting cannot automatically handle either Ogham or Old Turkic. For these scripts, [sideways-lr](#valdef-writing-mode-sideways-lr) can be used to typeset passages.

## <a id="conformance"></a> Conformance

### <a id="document-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conform-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="conform-responsible"></a> Requirements for Responsible Implementation of CSS

The following sections define several conformance requirements for implementing CSS responsibly, in a way that promotes interoperability in the present and future.

#### <a id="conform-partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid
        (and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)
        any at-rules, properties, property values, keywords, and other syntactic constructs
        for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

### <a id="cr-exit-criteria"></a> CR exit criteria

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

- [abstract dimensions](#abstract-dimensions), in §6.1
- [all](#valdef-text-combine-upright-all), in §9.1
- [alphabetic baseline](#alphabetic-baseline), in §4.2
- [baseline](#baseline), in §4.1
- [baseline table](#baseline-table), in §4.1
- [bidi-isolate](#bidi-isolate), in §2.2
- [bidi-isolated](#bidi-isolate), in §2.2
- [bidi isolation](#bidi-isolate), in §2.2
- [bidi-override](#valdef-unicode-bidi-bidi-override), in §2.2
- [bidi paragraph](#bidi-paragraph), in §2.4
- [bidirectionality](#bidirectionality), in §2
- [bi-orientational](#bi-orientational), in §5
- [bi-orientational transform](#bi-orientational-transform), in §5
- [block-axis](#block-axis), in §6.1
- [block axis](#block-axis), in §6.1
- [block dimension](#block-dimension), in §6.1
- [block-end](#block-end), in §6.2
- [block end](#block-end), in §6.2
- [block flow direction](#block-flow-direction), in §1
- [block size](#block-size), in §6.1
- [block-size](#block-size), in §6.1
- [block start](#block-start), in §6.2
- [block-start](#block-start), in §6.2
- [bottom](#physical-bottom), in §6
- [central baseline](#central-baseline), in §4.2
- [digits \<integer\>?](#valdef-text-combine-upright-digits-integer), in §9.1
- [direction](#propdef-direction), in §2.1
- [directional embedding](#directional-embedding), in §2.2
- [directional override](#directional-override), in §2.2
- [dominant baseline](#dominant-baseline), in §4.4
- [embed](#valdef-unicode-bidi-embed), in §2.2
- [end](#end), in §6.2
- [establish an orthogonal flow](#establish-an-orthogonal-flow), in §7.3
- [flow-relative](#flow-relative), in §6
- [flow-relative direction](#flow-relative-direction), in §6.2
- [forced paragraph break](#forced-paragraph-break), in §2.4
- [glyph-orientation-vertical](#propdef-glyph-orientation-vertical), in §5.1.3
- [height](#height), in §6
- [horizontal axis](#x-axis), in §6
- [horizontal block flow](#horizontal-block-flow), in §1
- [horizontal dimension](#horizontal-dimension), in §6
- [horizontal-only](#horizontal-only), in §5
- [horizontal script](#horizontal-script), in §5
- [horizontal-tb](#valdef-writing-mode-horizontal-tb), in §3.2
- [horizontal writing mode](#horizontal-writing-mode), in §1
- [inline axis](#inline-axis), in §6.1
- [inline-axis](#inline-axis), in §6.1
- [inline base direction](#inline-base-direction), in §1
- [inline dimension](#inline-dimension), in §6.1
- [inline end](#inline-end), in §6.2
- [inline-end](#inline-end), in §6.2
- [inline size](#inline-size), in §6.1
- [inline-size](#inline-size), in §6.1
- [inline start](#inline-start), in §6.2
- [inline-start](#inline-start), in §6.2
- [isolate](#valdef-unicode-bidi-isolate), in §2.2
- [isolated sequence](#isolated-sequence), in §2.2
- [isolate-override](#valdef-unicode-bidi-isolate-override), in §2.2
- [isolation](#bidi-isolate), in §2.2
- [left](#physical-left), in §6
- [line-left](#line-left), in §6.3
- [line orientation](#line-orientation), in §6.3
- [line-over](#line-over), in §6.3
- [line-relative](#line-relative), in §6
- [line-relative direction](#line-relative-direction), in §6.3
- [line-right](#line-right), in §6.3
- [line-under](#line-under), in §6.3
- [logical height](#logical-height), in §6.1
- [logical width](#logical-width), in §6.1
- [ltr](#valdef-direction-ltr), in §2.1
- [mixed](#valdef-text-orientation-mixed), in §5.1
- [none](#valdef-text-combine-upright-none), in §9.1
- [normal](#valdef-unicode-bidi-normal), in §2.2
- [orthogonal](#establish-an-orthogonal-flow), in §7.3
- [orthogonal flow](#establish-an-orthogonal-flow), in §7.3
- [over](#over), in §6.3
- [physical](#physical), in §6
- [physical bottom](#physical-bottom), in §6
- [physical dimensions](#physical-dimensions), in §6
- [physical direction](#physical-direction), in §6
- [physical left](#physical-left), in §6
- [physical right](#physical-right), in §6
- [physical top](#physical-top), in §6
- [plaintext](#valdef-unicode-bidi-plaintext), in §2.2
- [principal writing mode](#principal-writing-mode), in §8
- [right](#physical-right), in §6
- [rtl](#valdef-direction-rtl), in §2.1
- [sideways](#valdef-text-orientation-sideways), in §5.1
- [sideways-lr](#valdef-writing-mode-sideways-lr), in §3.2
- [sideways-right](#valdef-text-orientation-sideways-right), in §5.1
- [sideways-rl](#valdef-writing-mode-sideways-rl), in §3.2
- [start](#start), in §6.2
- [text-combine-upright](#propdef-text-combine-upright), in §9.1
- [text-orientation](#propdef-text-orientation), in §5.1
- [top](#physical-top), in §6
- [typographic mode](#typographic-mode), in §1
- [under](#under), in §6.3
- [unicode-bidi](#propdef-unicode-bidi), in §2.2
- [upright](#valdef-text-orientation-upright), in §5.1
- [vertical axis](#y-axis), in §6
- [vertical block flow](#vertical-block-flow), in §1
- [vertical dimension](#vertical-dimension), in §6
- [vertical-lr](#valdef-writing-mode-vertical-lr), in §3.2
- [vertical-only](#vertical-only), in §5
- [vertical-rl](#valdef-writing-mode-vertical-rl), in §3.2
- [vertical script](#vertical-script), in §5
- [vertical writing mode](#vertical-writing-mode), in §1
- [width](#width), in §6
- [writing mode](#writing-mode), in §1
- [writing-mode](#propdef-writing-mode), in §3.2
- [x-axis](#x-axis), in §6
- [y-axis](#y-axis), in §6

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-box-3\] defines the following terms:
  - <a id="term-for-propdef-margin-bottom"></a>margin-bottom
  - <a id="term-for-propdef-margin-left"></a>margin-left
  - <a id="term-for-propdef-margin-right"></a>margin-right
  - <a id="term-for-propdef-margin-top"></a>margin-top
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="term-for-fragment"></a>fragment
- \[css-break-4\] defines the following terms:
  - <a id="term-for-box-fragment"></a>box fragment
  - <a id="term-for-propdef-box-decoration-break"></a>box-decoration-break
- \[CSS-CASCADE-4\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-valdef-all-inherit"></a>inherit
  - <a id="term-for-inheritance"></a>inheritance
  - <a id="term-for-used-value"></a>used value
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="term-for-block-container"></a>block container
  - <a id="term-for-block-formatting-context"></a>block formatting context
  - <a id="term-for-valdef-display-flow"></a>flow
  - <a id="term-for-valdef-display-flow-root"></a>flow-root
  - <a id="term-for-in-flow"></a>in-flow
  - <a id="term-for-independent-formatting-context"></a>independent formatting context
  - <a id="term-for-initial-containing-block"></a>initial containing block
  - <a id="term-for-valdef-display-inline"></a>inline
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-valdef-display-inline-block"></a>inline-block
  - <a id="term-for-inner-display-type"></a>inner display type
  - <a id="term-for-replaced-element"></a>replaced element
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-root-inline-box"></a>root inline box
  - <a id="term-for-propdef-vertical-align"></a>vertical-align
- \[css-masking-1\] defines the following terms:
  - <a id="term-for-funcdef-rect"></a>rect()
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-scrollport"></a>scrollport
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="term-for-character"></a>character
  - <a id="term-for-white-space"></a>document white space
  - <a id="term-for-propdef-letter-spacing"></a>letter-spacing
  - <a id="term-for-propdef-text-align"></a>text-align
  - <a id="term-for-propdef-text-indent"></a>text-indent
  - <a id="term-for-propdef-text-transform"></a>text-transform
  - <a id="term-for-tracking"></a>tracking
  - <a id="term-for-typographic-character-unit"></a>typographic character unit
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="term-for-angle-value"></a>\<angle\>
  - <a id="term-for-integer-value"></a>\<integer\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-font-relative-length"></a>font-relative lengths
  - <a id="term-for-comb-one"></a>\|
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-border-spacing"></a>border-spacing
  - <a id="term-for-propdef-bottom"></a>bottom
  - <a id="term-for-propdef-caption-side"></a>caption-side
  - <a id="term-for-propdef-clear"></a>clear
  - <a id="term-for-propdef-clip"></a>clip
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-propdef-float"></a>float
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-propdef-left"></a>left
  - <a id="term-for-propdef-line-height"></a>line-height
  - <a id="term-for-propdef-max-height"></a>max-height
  - <a id="term-for-propdef-max-width"></a>max-width
  - <a id="term-for-propdef-min-height"></a>min-height
  - <a id="term-for-propdef-min-width"></a>min-width
  - <a id="term-for-propdef-right"></a>right
  - <a id="term-for-propdef-top"></a>top
  - <a id="term-for-propdef-width"></a>width
- \[CSS3-FLEXBOX\] defines the following terms:
  - <a id="term-for-flex-container"></a>flex container
  - <a id="term-for-flex-item"></a>flex item
- \[CSS3-FONTS\] defines the following terms:
  - <a id="term-for-propdef-font-feature-settings"></a>font-feature-settings
  - <a id="term-for-propdef-font-variant"></a>font-variant
- \[css3-images\] defines the following terms:
  - <a id="term-for-default-object-size"></a>default object size
- \[CSS3-SIZING\] defines the following terms:
  - <a id="term-for-valdef-width-auto"></a>auto
  - <a id="term-for-available"></a>available block space
  - <a id="term-for-available①"></a>available inline space
  - <a id="term-for-available②"></a>available space
  - <a id="term-for-definite"></a>definite
  - <a id="term-for-fallback"></a>fallback
  - <a id="term-for-max-width"></a>max size
  - <a id="term-for-valdef-width-max-content"></a>max-content
  - <a id="term-for-max-content-block-size"></a>max-content block size
  - <a id="term-for-max-content-inline-size"></a>max-content inline size
  - <a id="term-for-max-content"></a>max-content size
  - <a id="term-for-min-width"></a>min size
  - <a id="term-for-min-content-block-size"></a>min-content block size
  - <a id="term-for-min-content-inline-size"></a>min-content inline size
  - <a id="term-for-min-content"></a>min-content size
  - <a id="term-for-stretch-fit"></a>stretch fit
  - <a id="term-for-stretch-fit-block-size"></a>stretch-fit block size
- \[CSS3-TEXT-DECOR\] defines the following terms:
  - <a id="term-for-propdef-text-decoration"></a>text-decoration
  - <a id="term-for-propdef-text-shadow"></a>text-shadow
- \[CSS3BG\] defines the following terms:
  - <a id="term-for-propdef-box-shadow"></a>box-shadow
- \[CSS3COL\] defines the following terms:
  - <a id="term-for-valdef-column-width-auto"></a>auto (for column-width)
  - <a id="term-for-propdef-column-count"></a>column-count
  - <a id="term-for-propdef-column-gap"></a>column-gap
  - <a id="term-for-propdef-column-width"></a>column-width
- \[CSS3PAGE\] defines the following terms:
  - <a id="term-for-page-progression"></a>page progression
- \[HTML\] defines the following terms:
  - <a id="term-for-the-body-element"></a>body

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 11 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 8 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 26 August 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 31 July 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 12 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 31 January 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Elika Etemad; Tab Atkins Jr.. [CSS Image Values and Replaced Content Module Level 3](https://www.w3.org/TR/css3-images/). 17 April 2012. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-images&#x2F;](https://www.w3.org/TR/css3-images/)

<a id="biblio-css3-sizing"></a>\[CSS3-SIZING\]  
Tab Atkins Jr.; Elika Etemad. [CSS Intrinsic &#x26; Extrinsic Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 22 May 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css3-text-decor"></a>\[CSS3-TEXT-DECOR\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 3 July 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 17 October 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3col"></a>\[CSS3COL\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 28 May 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css3page"></a>\[CSS3PAGE\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

<a id="biblio-rfc6919"></a>\[RFC6919\]  
R. Barnes; S. Kent; E. Rescorla. [Further Key Words for Use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc6919). 1 April 2013. Experimental. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc6919](https://tools.ietf.org/html/rfc6919)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-uax11"></a>\[UAX11\]  
Ken Lunde 小林劍󠄁. [East Asian Width](https://www.unicode.org/reports/tr11/tr11-36.html). 25 January 2019. Unicode Standard Annex \#11. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr11&#x2F;tr11-36&#x2E;html](https://www.unicode.org/reports/tr11/tr11-36.html)

<a id="biblio-uax24"></a>\[UAX24\]  
Ken Whistler. [Unicode Script Property](https://www.unicode.org/reports/tr24/tr24-29.html). 6 February 2019. Unicode Standard Annex \#24. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr24&#x2F;tr24-29&#x2E;html](https://www.unicode.org/reports/tr24/tr24-29.html)

<a id="biblio-uax9"></a>\[UAX9\]  
Mark Davis; Aharon Lanin; Andrew Glass. [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/tr9-41.html). 4 February 2019. Unicode Standard Annex \#9. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr9&#x2F;tr9-41&#x2E;html](https://www.unicode.org/reports/tr9/tr9-41.html)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

<a id="biblio-utr50"></a>\[UTR50\]  
Koji Ishii 石井宏治; Ken Lunde 小林劍󠄁. [Unicode Vertical Text Layout](https://www.unicode.org/reports/tr50/tr50-22.html). 4 February 2019. Unicode Standard Annex \#50. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr50&#x2F;tr50-22&#x2E;html](https://www.unicode.org/reports/tr50/tr50-22.html)

### <a id="informative"></a>Informative References

<a id="biblio-css3-flexbox"></a>\[CSS3-FLEXBOX\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css3-fonts"></a>\[CSS3-FONTS\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-html401"></a>\[HTML401\]  
Dave Raggett; Arnaud Le Hors; Ian Jacobs. [HTML 4.01 Specification](https://www.w3.org/TR/html401/). 27 March 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;html401&#x2F;](https://www.w3.org/TR/html401/)

<a id="biblio-html5"></a>\[HTML5\]  
Ian Hickson; et al. [HTML5](https://www.w3.org/TR/html5/). 27 March 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;html5&#x2F;](https://www.w3.org/TR/html5/)

<a id="biblio-utn22"></a>\[UTN22\]  
Elika J. Etemad. [Robust Vertical Text Layout](https://unicode.org/notes/tn22/). 25 April 2005. Unicode Technical Note \#22. URL: [https&#x3A;&#x2F;&#x2F;unicode&#x2E;org&#x2F;notes&#x2F;tn22&#x2F;](https://unicode.org/notes/tn22/)

## <a id="property-index"></a>Property Index

<strong>Table 15 — structured row/cell transcription</strong>

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

Ani­mat­able

<strong>Column 8 (header cell; scope col):</strong>

Anim­ation type

<strong>Column 9 (header cell; scope col):</strong>

Canonical order

<strong>Column 10 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-direction③⑦"></a>

[direction](#propdef-direction)

<strong>Column 2 (data cell):</strong>

ltr \| rtl

<strong>Column 3 (data cell):</strong>

ltr

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

<strong>Column 8 (data cell):</strong>

not animatable

<strong>Column 9 (data cell):</strong>

n/a

<strong>Column 10 (data cell):</strong>

specified value

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-glyph-orientation-vertical⑤"></a>

[glyph-orientation-vertical](#propdef-glyph-orientation-vertical)

<strong>Column 2 (data cell):</strong>

auto \| 0deg \| 90deg \| 0 \| 90

<strong>Column 3 (data cell):</strong>

n/a

<strong>Column 4 (data cell):</strong>

n/a

<strong>Column 5 (data cell):</strong>

na/

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

n/a

<strong>Column 8 (data cell):</strong>

<strong>Column 9 (data cell):</strong>

n/a

<strong>Column 10 (data cell):</strong>

n/a

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-text-combine-upright①④"></a>

[text-combine-upright](#propdef-text-combine-upright)

<strong>Column 2 (data cell):</strong>

none \| all \| \[ digits \<integer\>? \]

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

non-replaced inline elements

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

<strong>Column 8 (data cell):</strong>

not animatable

<strong>Column 9 (data cell):</strong>

n/a

<strong>Column 10 (data cell):</strong>

specified keyword, plus integer if digits

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-text-orientation①⑨"></a>

[text-orientation](#propdef-text-orientation)

<strong>Column 2 (data cell):</strong>

mixed \| upright \| sideways

<strong>Column 3 (data cell):</strong>

mixed

<strong>Column 4 (data cell):</strong>

all elements except table row groups, rows, column groups, and columns

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

<strong>Column 8 (data cell):</strong>

not animatable

<strong>Column 9 (data cell):</strong>

n/a

<strong>Column 10 (data cell):</strong>

specified value

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-unicode-bidi②⑤"></a>

[unicode-bidi](#propdef-unicode-bidi)

<strong>Column 2 (data cell):</strong>

normal \| embed \| isolate \| bidi-override \| isolate-override \| plaintext

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

all elements, but see prose

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

<strong>Column 8 (data cell):</strong>

not animatable

<strong>Column 9 (data cell):</strong>

per grammar

<strong>Column 10 (data cell):</strong>

specified value

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-writing-mode②⑧"></a>

[writing-mode](#propdef-writing-mode)

<strong>Column 2 (data cell):</strong>

horizontal-tb \| vertical-rl \| vertical-lr \| sideways-rl \| sideways-lr

<strong>Column 3 (data cell):</strong>

horizontal-tb

<strong>Column 4 (data cell):</strong>

All elements except table row groups, table column groups, table rows, table columns, ruby base container, ruby annotation container

<strong>Column 5 (data cell):</strong>

yes

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

<strong>Column 8 (data cell):</strong>

not animatable

<strong>Column 9 (data cell):</strong>

n/a

<strong>Column 10 (data cell):</strong>

specified value

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should we factor in the box’s [min-content block size](https://www.w3.org/TR/css-sizing-3/#min-content-block-size) in this formula, so that e.g. a large image will not overflow the box, but cause the box to overflow the containing block? [↵](#issue-fae7187e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Does it also propagate to @page boxes? [↵](#issue-80e9901b)
