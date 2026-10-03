Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Display Module Level 3](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Display Module Level 3

Source snapshot: https://www.w3.org/TR/2026/CRD-css-display-3-20260605/

Snapshot SHA-256: 872cdd7c49fdbfba1ff7ea5b6689145c168b17d09542f11af717a55bc16ab9ed

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 5 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Display Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-display"></a>

This module describes how the CSS formatting box tree is generated from the document element tree and defines the [display](#propdef-display) property that controls it.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-display” in the title, like this: “\[css-display\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-display%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

A [preliminary implementation report](http://test.csswg.org/harness/results/css-display-3_dev/grouped/) is available. Further tests will be added during the CR period.

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-selectordef-first-letter"></a>

  Application of [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) in the presence of run-ins

- <a id="ref-for-propdef-display①"></a>

  [display: run-in](#propdef-display)

- <a id="ref-for-propdef-display②"></a>

  All multi-keyword values of [display](#propdef-display)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<em>This section is normative.</em>

<a id="ref-for-elements"></a>

<a id="ref-for-text-nodes"></a>

CSS takes a source document organized as a <a id="element-tree"></a><a id="css-element-tree"></a>tree of <a id="elements"></a><a id="css-element"></a>elements (which can contain a mix of other [elements](#elements) and [text nodes](#text-nodes)) and <a id="text-nodes"></a><a id="css-text-node"></a>text nodes (which can contain text), and renders it onto a [canvas](https://www.w3.org/TR/CSS2/intro.html#canvas) such as your screen, a piece of paper, or an audio stream. Although any such source document can be rendered with CSS, the most commonly used type is the DOM. [\[DOM\]](#biblio-dom) (Some of these more complex tree types might have additional types of nodes, such as the comment nodes in the DOM. For the purposes of CSS, all of these additional types of nodes are ignored, as if they didn’t exist.)

<a id="ref-for-box-tree"></a>

<a id="ref-for-elements①"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-text-nodes①"></a>

To do this, it generates an intermediary structure, the <a id="box-tree"></a><a id="css-box-tree"></a>box tree, which represents the formatting structure of the rendered document. Each <a id="box"></a><a id="css-box"></a>box in the [box tree](#box-tree) represents its corresponding [element](#elements) (or [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element)) in space and/or time on the canvas, while each <a id="css-text-sequence"></a><a id="text-run"></a><a id="css-text-run"></a>text sequence in the <a id="ref-for-box-tree①"></a>box tree likewise represents the corresponding contents of its [text nodes](#text-nodes).

<a id="ref-for-box-tree②"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-elements②"></a>

<a id="ref-for-text-nodes②"></a>

To create the [box tree](#box-tree), CSS first uses [cascading and inheritance](https://www.w3.org/TR/css-cascade/), to assign a [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) for each CSS property to each [element](#elements) and [text node](#text-nodes) in the source tree. (See [\[CSS-CASCADE-3\]](#biblio-css-cascade-3).)

<a id="ref-for-elements③"></a>

<a id="ref-for-box"></a>

<a id="ref-for-propdef-display③"></a>

<a id="ref-for-principal-box"></a>

<a id="ref-for-box-tree③"></a>

<a id="ref-for-valdef-display-none"></a>

<a id="ref-for-valdef-display-contents"></a>

Then, for each [element](#elements), CSS generates zero or more [boxes](#box) as specified by that element’s [display](#propdef-display) property. Typically, an element generates a single <a id="ref-for-box①"></a>box, the [principal box](#principal-box), which represents itself and contains its contents in the [box tree](#box-tree). However, some <a id="ref-for-propdef-display④"></a>display values (e.g. <a id="ref-for-propdef-display⑤"></a>display: list-item) generate more than one box (e.g. a [principal block box](https://www.w3.org/TR/CSS2/visuren.html#principal-box) and a child [marker box](https://www.w3.org/TR/CSS2/generate.html#lists)). And some values (such as [none](#valdef-display-none) or [contents](#valdef-display-contents)) cause the <a id="ref-for-elements④"></a>element and/or its descendants to not generate any <a id="ref-for-box②"></a>boxes at all. <a id="ref-for-box③"></a>Boxes are often referred to by their <a id="ref-for-propdef-display⑥"></a>display type—​e.g. a <a id="ref-for-box④"></a>box generated by an element with <a id="ref-for-propdef-display⑦"></a>display: block is called a “block box” or just a “block”.

<a id="ref-for-box⑤"></a>

<a id="ref-for-elements⑤"></a>

<a id="ref-for-inherited-property"></a>

<a id="ref-for-principal-box①"></a>

<a id="ref-for-propdef-border"></a>

<a id="ref-for-table-grid-box"></a>

<a id="ref-for-table-wrapper-box"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

A [box](#box) is assigned the same styles as its generating [element](#elements), unless otherwise indicated. In general, [inherited properties](https://www.w3.org/TR/css-cascade-5/#inherited-property) are assigned to the [principal box](#principal-box), and then inherit through the box tree to any other boxes generated by the same element. Non-inherited properties default to applying to the <a id="ref-for-principal-box②"></a>principal box, but when the element generates multiple boxes, are sometimes defined to apply to a different box: for example, the [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) properties applied to a table element are applied to its [table grid box](https://www.w3.org/TR/css-tables-3/#table-grid-box), not to its <a id="ref-for-principal-box③"></a>principal [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box). If the value computation process alters the styles of those boxes, and the element’s style is requested (such as through <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>), the element reflects, for each property, the value from the box to which that property was applied.

<a id="ref-for-text-nodes③"></a>

<a id="ref-for-css-text-sequence"></a>

Similarly, each contiguous sequence of sibling [text nodes](#text-nodes) generates a [text sequence](#css-text-sequence) containing their text contents, which is assigned the same styles as the generating <a id="ref-for-text-nodes④"></a>text nodes. If the sequence contains no text, however, it does not generate a <a id="ref-for-css-text-sequence①"></a>text sequence.

<a id="ref-for-principal-box④"></a>

<a id="ref-for-box⑥"></a>

<a id="ref-for-valdef-display-run-in"></a>

<a id="ref-for-anonymous"></a>

In constructing the box tree, boxes generated by an element are descendants of the principal box of any ancestor elements. In the general case, the direct <a id="css-parent-box"></a>parent box of an element’s [principal box](#principal-box) is the <a id="ref-for-principal-box⑤"></a>principal box of its nearest ancestor element that generates a [box](#box); however, there are some exceptions, such as for [run-in](#valdef-display-run-in) boxes, display types (like tables) that generate multiple container boxes, and intervening [anonymous boxes](#anonymous).

<a id="ref-for-anonymous①"></a>

<a id="ref-for-box-tree④"></a>

<a id="ref-for-element-tree"></a>

An <a id="anonymous"></a><a id="css-anonymous"></a>anonymous box is a box that is not associated with any element. [Anonymous boxes](#anonymous) are generated in certain circumstances to fix up the [box tree](#box-tree) when it requires a particular nested structure that is not provided by the boxes generated from the [element tree](#element-tree). For example, a [table cell box](https://www.w3.org/TR/CSS2/tables.html#table-display) requires a particular type of parent box (the [table row box](https://www.w3.org/TR/CSS2/tables.html#table-display)), and will generate an <a id="ref-for-anonymous②"></a>anonymous [table row box](https://www.w3.org/TR/CSS2/tables.html#table-display) around itself if its parent is not a [table row box](https://www.w3.org/TR/CSS2/tables.html#table-display). (See [\[CSS2\]](#biblio-css2) § [17.2.1](https://www.w3.org/TR/CSS2/tables.html#anonymous-boxes).) Unlike element-generated boxes, whose styles inherit strictly through the element tree, anonymous boxes (which only exist in the <a id="ref-for-box-tree⑤"></a>box tree) [inherit](https://www.w3.org/TR/css-cascade/#inheriting) through their <a id="ref-for-box-tree⑥"></a>box tree parentage.

<a id="ref-for-box⑦"></a>

<a id="ref-for-css-text-sequence②"></a>

<a id="ref-for-fragment"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-block-box"></a>

<a id="ref-for-fragmentation"></a>

<a id="ref-for-display-type"></a>

<a id="ref-for-box-fragment"></a>

In the course of layout, [boxes](#box) and [text sequences](#css-text-sequence) can be broken into multiple [fragments](https://www.w3.org/TR/css-break-3/#fragment). This happens, for example, when an [inline box](#inline-box) and/or <a id="ref-for-css-text-sequence③"></a>text sequence is broken across lines, or when a [block box](#block-box) is broken across pages or columns, in a process called [fragmentation](https://www.w3.org/TR/css-break-4/#fragmentation). It can also happen due to bidi reordering of text (see [Applying the Bidirectional Reordering Algorithm](https://www.w3.org/TR/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-direction)) or higher-level [display type](#display-type) box splitting, e.g. [block-in-inline splitting](https://www.w3.org/TR/CSS2/visuren.html#img-anon-block) (see [CSS2§9.2](https://www.w3.org/TR/CSS2/visuren.html#box-gen)) or [column-spanner-in-block](https://www.w3.org/TR/css-multicol-1/#spanning-columns) splitting (see [CSS Multi-column Layout](https://www.w3.org/TR/css-multicol-1/#spanning-columns)). A <a id="ref-for-box⑧"></a>box therefore consists of one or more [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment), and a <a id="ref-for-css-text-sequence④"></a>text sequence consists of one or more <a id="ref-for-fragment①"></a>text fragments. See [\[CSS-BREAK-3\]](#biblio-css-break-3) for more information on <a id="ref-for-fragmentation①"></a>fragmentation.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Many of the CSS specs were written before this terminology was ironed out, or refer to things incorrectly, so view older specs with caution when they’re using these terms. It should be possible to infer from context which term they really mean. Please [report errors](#sotd) in specs when you find them, so they can be corrected.

<a id="ref-for-propdef-display⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Further information on the “aural” box tree and its interaction with the [display](#propdef-display) property can be found in the [CSS Speech Module](https://www.w3.org/TR/css-speech-1/#aural-model). [\[CSS-SPEECH-1\]](#biblio-css-speech-1)

Tests

- [empty-text-baseline-001.html](https://wpt.fyi/results/css/css-display/empty-text-baseline-001.html) [(live test)](http://wpt.live/css/css-display/empty-text-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/empty-text-baseline-001.html)
- [empty-text-baseline-002.html](https://wpt.fyi/results/css/css-display/empty-text-baseline-002.html) [(live test)](http://wpt.live/css/css-display/empty-text-baseline-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/empty-text-baseline-002.html)

### <a id="placement"></a>1.1.  Module interactions

<a id="ref-for-propdef-display⑨"></a>

This module replaces and extends the definition of the [display](#propdef-display) property defined in [\[CSS2\]](#biblio-css2) section 9.2.4.

None of the properties in this module apply to the `::first-line` or `::first-letter` pseudo-elements.

Tests

- [display-first-letter-001.html](https://wpt.fyi/results/css/css-display/display-first-letter-001.html) [(live test)](http://wpt.live/css/css-display/display-first-letter-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-first-letter-001.html)
- [display-first-line-001.html](https://wpt.fyi/results/css/css-display/display-first-line-001.html) [(live test)](http://wpt.live/css/css-display/display-first-line-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-first-line-001.html)
- [display-first-line-002.html](https://wpt.fyi/results/css/css-display/display-first-line-002.html) [(live test)](http://wpt.live/css/css-display/display-first-line-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-first-line-002.html)
- [display-inline-dynamic-001.html](https://wpt.fyi/results/css/css-display/display-inline-dynamic-001.html) [(live test)](http://wpt.live/css/css-display/display-inline-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-inline-dynamic-001.html)

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-display①⓪"></a>

## <a id="the-display-properties"></a>2.  Box Layout Modes: the [display](#propdef-display) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-display"></a>display                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-display-legacy"></a><a id="ref-for-typedef-display-box"></a><a id="ref-for-typedef-display-internal"></a><a id="ref-for-typedef-display-listitem"></a><a id="ref-for-comb-one"></a><a id="ref-for-typedef-display-inside"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-display-outside"></a>\[ [\<display-outside\>](#typedef-display-outside) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<display-inside\>](#typedef-display-inside) \] [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<display-listitem\>](#typedef-display-listitem) <a id="ref-for-comb-one①"></a>\| [\<display-internal\>](#typedef-display-internal) <a id="ref-for-comb-one②"></a>\| [\<display-box\>](#typedef-display-box) <a id="ref-for-comb-one③"></a>\| [\<display-legacy\>](#typedef-display-legacy) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | inline                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-display-box①"></a><a id="ref-for-typedef-display-internal①"></a><a id="ref-for-valdef-display-list-item"></a><a id="ref-for-outer-display-type"></a><a id="ref-for-inner-display-type"></a>a pair of keywords representing the [inner](#inner-display-type) and [outer](#outer-display-type) display types plus optional [list-item](#valdef-display-list-item) flag, or a [\<display-internal\>](#typedef-display-internal) or [\<display-box\>](#typedef-display-box) keyword; see prose in a variety of specs for computation rules                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |

<a id="ref-for-propdef-display①①"></a>

User agents are expected to support this property on all media, including non-visual ones. The [display](#propdef-display) property defines an element’s <a id="display-type"></a>display type, which consists of the two basic qualities of how an element generates boxes:

- <a id="ref-for-replaced-element"></a>

  <a id="ref-for-formatting-context"></a>

  the <a id="inner-display-type"></a>inner display type, which defines (if it is a [non-replaced element](#replaced-element)) the kind of [formatting context](#formatting-context) it generates, dictating how its descendant boxes are laid out. (The inner display of a <a id="ref-for-replaced-element①"></a>replaced element is outside the scope of CSS.)

- <a id="ref-for-principal-box⑥"></a>

  <a id="ref-for-flow-layout"></a>

  the <a id="outer-display-type"></a>outer display type, which dictates how the [principal box](#principal-box) itself participates in [flow layout](#flow-layout).

<a id="ref-for-css-text-sequence⑤"></a>

<a id="ref-for-display-type①"></a>

[Text sequences](#css-text-sequence) have no [display type](#display-type).

<a id="ref-for-propdef-display①②"></a>

<a id="ref-for-valdef-display-list-item①"></a>

<a id="ref-for-selectordef-marker"></a>

<a id="ref-for-valdef-display-none①"></a>

Some [display](#propdef-display) values have additional side-effects: such as [list-item](#valdef-display-list-item), which also generates a [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element, and [none](#valdef-display-none), which causes the element’s entire subtree to be left out of the box tree.

<a id="ref-for-propdef-display①③"></a>

<a id="ref-for-document-language"></a>

<a id="ref-for-valdef-display-none②"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> The [display](#propdef-display) property has no effect on an element’s semantics: these are defined by the [document language](https://www.w3.org/TR/selectors-4/#document-language) and <em>are not affected by CSS</em>. Aside from the [none](#valdef-display-none) value, which also affects the aural/speech output [\[CSS-SPEECH-1\]](#biblio-css-speech-1) and interactivity of an element and its descendants, the <a id="ref-for-propdef-display①④"></a>display property only affects visual layout: its purpose is to allow designers freedom to change the layout behavior of an element <em>without</em> affecting the underlying document semantics.

Values are defined as follows:

<a id="typedef-display-outside"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="typedef-display-inside"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="typedef-display-listitem"></a>

<a id="ref-for-typedef-display-outside①"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-comb-all①"></a>

<a id="typedef-display-internal"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="typedef-display-box"></a>

<a id="ref-for-comb-one②③"></a>

<a id="typedef-display-legacy"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-comb-one②⑥"></a>

```text
<display-outside>  = block | inline | run-in
<display-inside>   = flow | flow-root | table | flex | grid | ruby
<display-listitem> = <display-outside>? && [ flow | flow-root ]? && list-item
<display-internal> = table-row-group | table-header-group |
                     table-footer-group | table-row | table-cell |
                     table-column-group | table-column | table-caption |
                     ruby-base | ruby-text | ruby-base-container |
                     ruby-text-container
<display-box>      = contents | none
<display-legacy>   = inline-block | inline-table | inline-flex | inline-grid
```
<a id="ref-for-propdef-display①⑤"></a>

The following informative table summarizes the values of [display](#propdef-display):

<a id="display-value-summary"></a>

| <a id="ref-for-propdef-display①⑥"></a>Short [display](#propdef-display)                       | <a id="ref-for-propdef-display①⑦"></a>Full [display](#propdef-display) | Generated box                                                                                                                                                                                                                                     |
|----------------------------------------------------------------------------|-----------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <a id="ref-for-valdef-display-none③"></a>[none](#valdef-display-none)                            | —                                                   | <a id="ref-for-box-tree⑦"></a>subtree omitted from [box tree](#box-tree)                                                                                                                                                                                     |
| <a id="ref-for-valdef-display-contents①"></a>[contents](#valdef-display-contents)                    | —                                                   | <a id="ref-for-box-tree⑧"></a>element replaced by contents in [box tree](#box-tree)                                                                                                                                                                          |
| <a id="ref-for-valdef-display-block"></a>[block](#valdef-display-block)                          | block flow                                          | <a id="ref-for-block-box①"></a><a id="ref-for-block-container"></a><a id="ref-for-block-level"></a>[block-level](#block-level) [block container](#block-container) aka [block box](#block-box)                                                                                              |
| <a id="ref-for-valdef-display-flow-root"></a>[flow-root](#valdef-display-flow-root)                  | block flow-root                                     | <a id="ref-for-bfc"></a><a id="ref-for-block-formatting-context"></a><a id="ref-for-block-container①"></a><a id="ref-for-block-level①"></a>[block-level](#block-level) [block container](#block-container) that establishes a new [block formatting context](#block-formatting-context) ([BFC](#bfc))            |
| <a id="ref-for-valdef-display-inline"></a>[inline](#valdef-display-inline)                        | inline flow                                         | <a id="ref-for-inline-box①"></a>[inline box](#inline-box)                                                                                                                                                                                                      |
| <a id="ref-for-valdef-display-inline-block"></a>[inline-block](#valdef-display-inline-block)            | inline flow-root                                    | <a id="ref-for-block-container②"></a><a id="ref-for-inline-level"></a>[inline-level](#inline-level) [block container](#block-container) aka <a id="inline-block"></a>inline block                                                                                                       |
| <a id="ref-for-valdef-display-run-in①"></a>[run-in](#valdef-display-run-in)                        | run-in flow                                         | <a id="ref-for-inline-box②"></a><a id="ref-for-run-in"></a>[run-in box](#run-in) ([inline box](#inline-box) with special box-tree-munging rules)                                                                                                                       |
| <a id="ref-for-valdef-display-list-item②"></a>[list-item](#valdef-display-list-item)                  | block flow list-item                                | <a id="ref-for-block-box②"></a>[block box](#block-box) with additional [marker box](https://www.w3.org/TR/CSS2/generate.html#lists)                                                                                                                           |
| inline list-item                                                           | inline flow list-item                               | <a id="ref-for-inline-box③"></a>[inline box](#inline-box) with additional [marker box](https://www.w3.org/TR/CSS2/generate.html#lists)                                                                                                                         |
| <a id="ref-for-valdef-display-flex"></a>[flex](#valdef-display-flex)                            | block flex                                          | <a id="ref-for-flex-container"></a><a id="ref-for-block-level②"></a>[block-level](#block-level) [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)                                                                                                           |
| <a id="ref-for-valdef-display-inline-flex"></a>[inline-flex](#valdef-display-inline-flex)              | inline flex                                         | <a id="ref-for-flex-container①"></a><a id="ref-for-inline-level①"></a>[inline-level](#inline-level) [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)                                                                                                         |
| <a id="ref-for-valdef-display-grid"></a>[grid](#valdef-display-grid)                            | block grid                                          | <a id="ref-for-grid-container"></a><a id="ref-for-block-level③"></a>[block-level](#block-level) [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)                                                                                                              |
| <a id="ref-for-valdef-display-inline-grid"></a>[inline-grid](#valdef-display-inline-grid)              | inline grid                                         | <a id="ref-for-grid-container①"></a><a id="ref-for-inline-level②"></a>[inline-level](#inline-level) [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)                                                                                                            |
| <a id="ref-for-valdef-display-ruby"></a>[ruby](#valdef-display-ruby)                            | inline ruby                                         | <a id="ref-for-ruby-container"></a><a id="ref-for-inline-level③"></a>[inline-level](#inline-level) [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container)                                                                                                            |
| block ruby                                                                 | block ruby                                          | <a id="ref-for-ruby-container①"></a><a id="ref-for-block-box③"></a>[block box](#block-box) containing [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container)                                                                                                       |
| <a id="ref-for-valdef-display-table"></a>[table](#valdef-display-table)                          | block table                                         | <a id="ref-for-table-grid-box①"></a><a id="ref-for-table-wrapper-box①"></a><a id="ref-for-block-level④"></a>[block-level](#block-level) [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box) containing [table grid box](https://www.w3.org/TR/css-tables-3/#table-grid-box)   |
| <a id="ref-for-valdef-display-inline-table"></a>[inline-table](#valdef-display-inline-table)            | inline table                                        | <a id="ref-for-table-grid-box②"></a><a id="ref-for-table-wrapper-box②"></a><a id="ref-for-inline-level④"></a>[inline-level](#inline-level) [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box) containing [table grid box](https://www.w3.org/TR/css-tables-3/#table-grid-box) |
| <a id="ref-for-typedef-display-internal②"></a>[\<display-internal\>](#typedef-display-internal) types | —                                                   | [layout-specific internal box](#layout-specific-display)                                                                                                                                                                                          |

<a id="ref-for-propdef-display①⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Following the precedence rules of “most backwards-compatible, then shortest”, serialization of equivalent [display](#propdef-display) values uses the “Short <a id="ref-for-propdef-display①⑨"></a>display” column. [\[CSSOM\]](#biblio-cssom)

Tests

- [display-interpolation.html](https://wpt.fyi/results/css/css-display/animations/display-interpolation.html) [(live test)](http://wpt.live/css/css-display/animations/display-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/animations/display-interpolation.html)
- [display-math-on-non-mathml-elements.html](https://wpt.fyi/results/css/css-display/display-math-on-non-mathml-elements.html) [(live test)](http://wpt.live/css/css-display/display-math-on-non-mathml-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-math-on-non-mathml-elements.html)
- [display-math-on-pseudo-elements-001.html](https://wpt.fyi/results/css/css-display/display-math-on-pseudo-elements-001.html) [(live test)](http://wpt.live/css/css-display/display-math-on-pseudo-elements-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-math-on-pseudo-elements-001.html)
- [display-math-on-pseudo-elements-002.html](https://wpt.fyi/results/css/css-display/display-math-on-pseudo-elements-002.html) [(live test)](http://wpt.live/css/css-display/display-math-on-pseudo-elements-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-math-on-pseudo-elements-002.html)
- [display-none-root-hit-test-crash.html](https://wpt.fyi/results/css/css-display/display-none-root-hit-test-crash.html) [(live test)](http://wpt.live/css/css-display/display-none-root-hit-test-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-none-root-hit-test-crash.html)
- [inheritance.html](https://wpt.fyi/results/css/css-display/inheritance.html) [(live test)](http://wpt.live/css/css-display/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/inheritance.html)
- [display-computed.html](https://wpt.fyi/results/css/css-display/parsing/display-computed.html) [(live test)](http://wpt.live/css/css-display/parsing/display-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/parsing/display-computed.html)
- [display-invalid.html](https://wpt.fyi/results/css/css-display/parsing/display-invalid.html) [(live test)](http://wpt.live/css/css-display/parsing/display-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/parsing/display-invalid.html)
- [display-valid.html](https://wpt.fyi/results/css/css-display/parsing/display-valid.html) [(live test)](http://wpt.live/css/css-display/parsing/display-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/parsing/display-valid.html)
- [select-4-option-optgroup-display-none.html](https://wpt.fyi/results/css/css-display/select-4-option-optgroup-display-none.html) [(live test)](http://wpt.live/css/css-display/select-4-option-optgroup-display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/select-4-option-optgroup-display-none.html)
- [textarea-display.html](https://wpt.fyi/results/css/css-display/textarea-display.html) [(live test)](http://wpt.live/css/css-display/textarea-display.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/textarea-display.html)

<a id="ref-for-valdef-display-block①"></a>

<a id="ref-for-valdef-display-inline①"></a>

<a id="ref-for-valdef-display-run-in②"></a>

### <a id="outer-role"></a>2.1.  Outer Display Roles for Flow Layout: the [block](#valdef-display-block), [inline](#valdef-display-inline), and [run-in](#valdef-display-run-in) keywords

<a id="ref-for-typedef-display-outside②"></a>

<a id="ref-for-outer-display-type①"></a>

<a id="ref-for-principal-box⑦"></a>

<a id="ref-for-flow-layout①"></a>

The [\<display-outside\>](#typedef-display-outside) keywords specify the element’s [outer display type](#outer-display-type), which is essentially its [principal box’s](#principal-box) role in [flow layout](#flow-layout). They are defined as follows:

<a id="valdef-display-block"></a>block  
<a id="ref-for-flow-layout②"></a>

The element generates a box that is <a id="block-level-box"></a>block-level when placed in [flow layout](#flow-layout). [\[CSS2\]](#biblio-css2)

<a id="valdef-display-inline"></a>inline  
<a id="ref-for-flow-layout③"></a>

The element generates a box that is <a id="inline-level-box"></a>inline-level when placed in [flow layout](#flow-layout). [\[CSS2\]](#biblio-css2)

<a id="valdef-display-run-in"></a>run-in  
<a id="ref-for-inline-level-box"></a>

<a id="ref-for-run-in①"></a>

The element generates an [run-in box](#run-in), which is a type of [inline-level box](#inline-level-box) with special behavior that attempts to merge it into a subsequent block container. See [§ 5 Run-In Layout](#run-in-layout) for details.

<a id="ref-for-outer-display-type②"></a>

<a id="ref-for-replaced-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Outer display types](#outer-display-type) do affect [replaced elements](#replaced-element).

<a id="ref-for-typedef-display-outside③"></a>

<a id="ref-for-typedef-display-inside①"></a>

<a id="ref-for-inner-display-type①"></a>

<a id="ref-for-valdef-display-flow"></a>

If a [\<display-outside\>](#typedef-display-outside) value is specified but [\<display-inside\>](#typedef-display-inside) is omitted, the element’s [inner display type](#inner-display-type) defaults to [flow](#valdef-display-flow).

Tests

- [display-change-iframe.html](https://wpt.fyi/results/css/css-display/display-change-iframe.html) [(live test)](http://wpt.live/css/css-display/display-change-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-change-iframe.html)
- [display-change-object-iframe.html](https://wpt.fyi/results/css/css-display/display-change-object-iframe.html) [(live test)](http://wpt.live/css/css-display/display-change-object-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-change-object-iframe.html)
- after-content-display-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/after-content-display-004.xht)
- anonymous-box-generation-002.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/anonymous-box-generation-002.xht)
- background-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-applies-to-011.xht)
- background-attachment-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-attachment-applies-to-011.xht)
- background-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-color-applies-to-011.xht)
- background-image-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-image-applies-to-011.xht)
- background-position-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-position-applies-to-011.xht)
- background-repeat-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/background-repeat-applies-to-011.xht)
- before-content-display-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/before-content-display-004.xht)
- border-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-applies-to-011.xht)
- border-bottom-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-bottom-applies-to-011.xht)
- border-bottom-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-bottom-color-applies-to-011.xht)
- border-bottom-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-bottom-style-applies-to-011.xht)
- border-bottom-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-bottom-width-applies-to-011.xht)
- border-collapse-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-collapse-applies-to-004.xht)
- border-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-color-applies-to-011.xht)
- border-left-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-left-applies-to-011.xht)
- border-left-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-left-color-applies-to-011.xht)
- border-left-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-left-style-applies-to-011.xht)
- border-left-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-left-width-applies-to-011.xht)
- border-right-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-right-applies-to-011.xht)
- border-right-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-right-color-applies-to-011.xht)
- border-right-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-right-style-applies-to-011.xht)
- border-right-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-right-width-applies-to-011.xht)
- border-spacing-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-spacing-applies-to-004.xht)
- border-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-style-applies-to-011.xht)
- border-top-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-top-applies-to-011.xht)
- border-top-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-top-color-applies-to-011.xht)
- border-top-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-top-style-applies-to-011.xht)
- border-top-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-top-width-applies-to-011.xht)
- border-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/border-width-applies-to-011.xht)
- bottom-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/bottom-applies-to-011.xht)
- caption-side-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/caption-side-applies-to-004.xht)
- clear-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/clear-applies-to-011.xht)
- clear-runin-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/clear-runin-001.xht)
- color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/color-applies-to-011.xht)
- [counter-increment-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/counter-increment-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/counter-increment-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/counter-increment-applies-to-011.xht)
- [counter-reset-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/counter-reset-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/counter-reset-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/counter-reset-applies-to-011.xht)
- cursor-applies-to-011.xht (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/cursor-applies-to-011.xht)
- direction-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/direction-applies-to-011.xht)
- display-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/display-004.xht)
- empty-cells-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/empty-cells-applies-to-004.xht)
- first-line-pseudo-009.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/first-line-pseudo-009.xht)
- float-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/float-applies-to-011.xht)
- font-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-applies-to-004.xht)
- font-family-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-family-applies-to-004.xht)
- font-size-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-size-applies-to-004.xht)
- [font-style-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/font-style-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/font-style-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-style-applies-to-004.xht)
- [font-variant-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/font-variant-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/font-variant-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-variant-applies-to-004.xht)
- [font-weight-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/font-weight-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/font-weight-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/font-weight-applies-to-004.xht)
- [height-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/height-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/height-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/height-applies-to-011.xht)
- left-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/left-applies-to-011.xht)
- [letter-spacing-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/letter-spacing-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/letter-spacing-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/letter-spacing-applies-to-004.xht)
- line-height-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/line-height-applies-to-011.xht)
- [list-style-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/list-style-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/list-style-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/list-style-applies-to-011.xht)
- list-style-image-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/list-style-image-applies-to-011.xht)
- list-style-position-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/list-style-position-applies-to-011.xht)
- [list-style-type-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/list-style-type-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/list-style-type-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/list-style-type-applies-to-011.xht)
- margin-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/margin-applies-to-011.xht)
- margin-bottom-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/margin-bottom-applies-to-011.xht)
- margin-left-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/margin-left-applies-to-011.xht)
- margin-right-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/margin-right-applies-to-011.xht)
- margin-top-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/margin-top-applies-to-011.xht)
- [max-height-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/max-height-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/max-height-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/max-height-applies-to-011.xht)
- [max-width-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/max-width-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/max-width-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/max-width-applies-to-011.xht)
- [min-height-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/min-height-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/min-height-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/min-height-applies-to-011.xht)
- [min-width-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/min-width-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/min-width-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/min-width-applies-to-011.xht)
- outline-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/outline-applies-to-011.xht)
- outline-color-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/outline-color-applies-to-011.xht)
- outline-style-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/outline-style-applies-to-011.xht)
- outline-width-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/outline-width-applies-to-011.xht)
- overflow-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/overflow-applies-to-011.xht)
- padding-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/padding-applies-to-011.xht)
- padding-bottom-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/padding-bottom-applies-to-011.xht)
- padding-left-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/padding-left-applies-to-011.xht)
- padding-right-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/padding-right-applies-to-011.xht)
- padding-top-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/padding-top-applies-to-011.xht)
- position-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/position-applies-to-011.xht)
- [quotes-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/quotes-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/quotes-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/quotes-applies-to-011.xht)
- right-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/right-applies-to-011.xht)
- run-in-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-001.xht)
- run-in-002.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-002.xht)
- run-in-003.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-003.xht)
- run-in-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-004.xht)
- run-in-005.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-005.xht)
- run-in-006.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-006.xht)
- run-in-007.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-007.xht)
- run-in-008.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-008.xht)
- run-in-009.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-009.xht)
- run-in-010.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-010.xht)
- run-in-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-011.xht)
- run-in-012.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-012.xht)
- run-in-013.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-013.xht)
- [run-in-abspos-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-abspos-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-abspos-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-abspos-between-001.xht)
- [run-in-abspos-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-abspos-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-abspos-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-abspos-between-002.xht)
- [run-in-abspos-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-abspos-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-abspos-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-abspos-between-003.xht)
- [run-in-basic-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-001.xht)
- [run-in-basic-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-002.xht)
- [run-in-basic-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-003.xht)
- [run-in-basic-004.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-004.xht)
- [run-in-basic-005.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-005.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-005.xht)
- [run-in-basic-006.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-006.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-006.xht)
- [run-in-basic-007.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-007.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-007.xht)
- [run-in-basic-008.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-008.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-008.xht)
- [run-in-basic-009.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-009.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-009.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-009.xht)
- [run-in-basic-010.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-010.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-010.xht)
- [run-in-basic-011.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-011.xht)
- [run-in-basic-012.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-012.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-012.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-012.xht)
- [run-in-basic-013.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-013.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-013.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-013.xht)
- [run-in-basic-014.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-014.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-014.xht)
- [run-in-basic-015.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-015.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-015.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-015.xht)
- [run-in-basic-016.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-016.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-016.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-016.xht)
- [run-in-basic-017.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-017.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-017.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-017.xht)
- [run-in-basic-018.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-basic-018.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-basic-018.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-basic-018.xht)
- [run-in-block-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-block-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-block-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-block-between-001.xht)
- [run-in-block-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-block-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-block-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-block-between-002.xht)
- [run-in-block-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-block-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-block-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-block-between-003.xht)
- [run-in-breaking-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-breaking-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-breaking-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-breaking-001.xht)
- [run-in-breaking-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-breaking-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-breaking-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-breaking-002.xht)
- [run-in-clear-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-clear-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-clear-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-clear-001.xht)
- [run-in-clear-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-clear-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-clear-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-clear-002.xht)
- [run-in-contains-abspos-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-abspos-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-abspos-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-abspos-001.xht)
- [run-in-contains-block-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-001.xht)
- [run-in-contains-block-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-002.xht)
- [run-in-contains-block-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-003.xht)
- [run-in-contains-block-004.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-004.xht)
- [run-in-contains-block-005.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-005.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-005.xht)
- [run-in-contains-block-inside-inline-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-inside-inline-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-inside-inline-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-inside-inline-001.xht)
- [run-in-contains-block-inside-inline-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-inside-inline-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-inside-inline-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-inside-inline-002.xht)
- [run-in-contains-block-inside-inline-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-block-inside-inline-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-block-inside-inline-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-block-inside-inline-003.xht)
- [run-in-contains-float-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-float-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-float-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-float-001.xht)
- [run-in-contains-inline-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-001.xht)
- [run-in-contains-inline-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-002.xht)
- [run-in-contains-inline-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-003.xht)
- [run-in-contains-inline-004.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-004.xht)
- [run-in-contains-inline-005.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-005.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-005.xht)
- [run-in-contains-inline-006.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-006.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-006.xht)
- [run-in-contains-inline-007.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-007.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-007.xht)
- [run-in-contains-inline-block-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-block-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-block-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-block-001.xht)
- [run-in-contains-inline-table-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-inline-table-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-inline-table-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-inline-table-001.xht)
- [run-in-contains-relpos-block-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-relpos-block-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-relpos-block-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-relpos-block-001.xht)
- [run-in-contains-relpos-block-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-relpos-block-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-relpos-block-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-relpos-block-002.xht)
- [run-in-contains-relpos-block-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-relpos-block-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-relpos-block-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-relpos-block-003.xht)
- [run-in-contains-run-in-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-run-in-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-run-in-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-run-in-001.xht)
- [run-in-contains-run-in-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-run-in-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-run-in-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-run-in-002.xht)
- [run-in-contains-run-in-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-run-in-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-run-in-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-run-in-003.xht)
- [run-in-contains-table-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-001.xht)
- [run-in-contains-table-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-002.xht)
- [run-in-contains-table-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-003.xht)
- [run-in-contains-table-caption-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-caption-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-caption-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-caption-001.xht)
- [run-in-contains-table-cell-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-cell-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-cell-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-cell-001.xht)
- [run-in-contains-table-column-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-column-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-column-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-column-001.xht)
- [run-in-contains-table-column-group-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-column-group-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-column-group-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-column-group-001.xht)
- [run-in-contains-table-inside-inline-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-inside-inline-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-inside-inline-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-inside-inline-001.xht)
- [run-in-contains-table-inside-inline-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-inside-inline-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-inside-inline-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-inside-inline-002.xht)
- [run-in-contains-table-inside-inline-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-inside-inline-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-inside-inline-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-inside-inline-003.xht)
- [run-in-contains-table-row-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-row-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-row-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-row-001.xht)
- [run-in-contains-table-row-group-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-contains-table-row-group-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-contains-table-row-group-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-contains-table-row-group-001.xht)
- [run-in-display-none-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-display-none-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-display-none-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-display-none-between-001.xht)
- [run-in-display-none-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-display-none-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-display-none-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-display-none-between-002.xht)
- [run-in-display-none-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-display-none-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-display-none-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-display-none-between-003.xht)
- [run-in-fixedpos-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-fixedpos-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-fixedpos-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-fixedpos-between-001.xht)
- [run-in-fixedpos-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-fixedpos-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-fixedpos-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-fixedpos-between-002.xht)
- [run-in-fixedpos-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-fixedpos-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-fixedpos-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-fixedpos-between-003.xht)
- [run-in-float-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-float-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-float-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-float-between-001.xht)
- [run-in-float-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-float-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-float-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-float-between-002.xht)
- [run-in-float-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-float-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-float-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-float-between-003.xht)
- [run-in-inherit-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inherit-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inherit-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inherit-001.xht)
- run-in-inheritance-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inheritance-001.xht)
- [run-in-inline-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-between-001.xht)
- [run-in-inline-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-between-002.xht)
- [run-in-inline-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-between-003.xht)
- [run-in-inline-block-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-block-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-block-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-block-between-001.xht)
- [run-in-inline-block-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-block-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-block-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-block-between-002.xht)
- [run-in-inline-block-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-block-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-block-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-block-between-003.xht)
- [run-in-inline-table-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-table-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-table-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-table-between-001.xht)
- [run-in-inline-table-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-table-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-table-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-table-between-002.xht)
- [run-in-inline-table-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-inline-table-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-inline-table-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-inline-table-between-003.xht)
- run-in-linebox-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-linebox-001.xht)
- run-in-linebox-002.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-linebox-002.xht)
- [run-in-listitem-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-listitem-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-listitem-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-listitem-between-001.xht)
- [run-in-listitem-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-listitem-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-listitem-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-listitem-between-002.xht)
- [run-in-listitem-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-listitem-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-listitem-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-listitem-between-003.xht)
- [run-in-relpos-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-relpos-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-relpos-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-relpos-between-001.xht)
- [run-in-relpos-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-relpos-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-relpos-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-relpos-between-002.xht)
- [run-in-relpos-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-relpos-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-relpos-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-relpos-between-003.xht)
- [run-in-replaced-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-replaced-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-replaced-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-replaced-001.xht)
- [run-in-restyle-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-restyle-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-restyle-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-restyle-001.xht)
- [run-in-restyle-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-restyle-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-restyle-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-restyle-002.xht)
- [run-in-restyle-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-restyle-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-restyle-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-restyle-003.xht)
- [run-in-run-in-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-001.xht)
- [run-in-run-in-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-002.xht)
- [run-in-run-in-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-003.xht)
- [run-in-run-in-between-004.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-004.xht)
- [run-in-run-in-between-005.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-005.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-005.xht)
- [run-in-run-in-between-006.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-006.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-006.xht)
- [run-in-run-in-between-007.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-007.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-007.xht)
- [run-in-run-in-between-008.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-run-in-between-008.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-run-in-between-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-run-in-between-008.xht)
- [run-in-table-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-between-001.xht)
- [run-in-table-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-between-002.xht)
- [run-in-table-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-between-003.xht)
- [run-in-table-cell-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-cell-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-cell-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-cell-between-001.xht)
- [run-in-table-cell-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-cell-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-cell-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-cell-between-002.xht)
- [run-in-table-cell-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-cell-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-cell-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-cell-between-003.xht)
- [run-in-table-row-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-row-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-row-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-row-between-001.xht)
- [run-in-table-row-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-row-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-row-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-row-between-002.xht)
- [run-in-table-row-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-table-row-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-table-row-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-table-row-between-003.xht)
- [run-in-text-between-001.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-text-between-001.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-text-between-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-text-between-001.xht)
- [run-in-text-between-002.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-text-between-002.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-text-between-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-text-between-002.xht)
- [run-in-text-between-003.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-text-between-003.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-text-between-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-text-between-003.xht)
- [run-in-text-between-004.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-text-between-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-text-between-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-text-between-004.xht)
- [run-in-text-between-005.xht](https://wpt.fyi/results/css/css-display/run-in/run-in-text-between-005.xht) [(live test)](http://wpt.live/css/css-display/run-in/run-in-text-between-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/run-in-text-between-005.xht)
- table-anonymous-block-001.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/table-anonymous-block-001.xht)
- table-layout-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/table-layout-applies-to-004.xht)
- text-align-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/text-align-applies-to-004.xht)
- [text-decoration-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/text-decoration-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/text-decoration-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/text-decoration-applies-to-004.xht)
- text-indent-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/text-indent-applies-to-004.xht)
- [text-transform-applies-to-004.xht](https://wpt.fyi/results/css/css-display/run-in/text-transform-applies-to-004.xht) [(live test)](http://wpt.live/css/css-display/run-in/text-transform-applies-to-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/text-transform-applies-to-004.xht)
- top-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/top-applies-to-011.xht)
- unicode-bidi-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/unicode-bidi-applies-to-011.xht)
- vertical-align-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/vertical-align-applies-to-011.xht)
- visibility-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/visibility-applies-to-011.xht)
- white-space-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/white-space-applies-to-004.xht)
- [width-applies-to-011.xht](https://wpt.fyi/results/css/css-display/run-in/width-applies-to-011.xht) [(live test)](http://wpt.live/css/css-display/run-in/width-applies-to-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/width-applies-to-011.xht)
- word-spacing-applies-to-004.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/word-spacing-applies-to-004.xht)
- z-index-applies-to-011.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/run-in/z-index-applies-to-011.xht)

<a id="ref-for-valdef-display-flow①"></a>

<a id="ref-for-valdef-display-flow-root①"></a>

<a id="ref-for-valdef-display-table①"></a>

<a id="ref-for-valdef-display-flex①"></a>

<a id="ref-for-valdef-display-grid①"></a>

<a id="ref-for-valdef-display-ruby①"></a>

### <a id="inner-model"></a>2.2.  Inner Display Layout Models: the [flow](#valdef-display-flow), [flow-root](#valdef-display-flow-root), [table](#valdef-display-table), [flex](#valdef-display-flex), [grid](#valdef-display-grid), and [ruby](#valdef-display-ruby) keywords

<a id="ref-for-typedef-display-inside②"></a>

<a id="ref-for-inner-display-type②"></a>

<a id="ref-for-replaced-element③"></a>

The [\<display-inside\>](#typedef-display-inside) keywords specify the element’s [inner display type](#inner-display-type), which defines the type of formatting context that lays out its contents (assuming it is a [non-replaced element](#replaced-element)). They are defined as follows:

<a id="valdef-display-flow"></a>flow  
The element lays out its contents using <a id="flow-layout"></a>flow layout ([block-and-inline layout](https://www.w3.org/TR/CSS2/visuren.html)).

<a id="ref-for-outer-display-type③"></a>

<a id="ref-for-valdef-display-inline②"></a>

<a id="ref-for-valdef-display-run-in③"></a>

<a id="ref-for-block-formatting-context①"></a>

<a id="ref-for-inline-formatting-context"></a>

<a id="ref-for-inline-box④"></a>

If its [outer display type](#outer-display-type) is [inline](#valdef-display-inline) or [run-in](#valdef-display-run-in), and it is participating in a [block](#block-formatting-context) or [inline](#inline-formatting-context) formatting context, then it generates an [inline box](#inline-box).

<a id="ref-for-block-container③"></a>

Otherwise it generates a [block container](#block-container) box.

<a id="ref-for-propdef-position"></a>

<a id="ref-for-propdef-float"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-formatting-context①"></a>

<a id="ref-for-block-formatting-context②"></a>

<a id="ref-for-block-container④"></a>

<a id="ref-for-inner-display-type③"></a>

<a id="ref-for-valdef-display-flow-root②"></a>

Depending on the value of other properties (such as [position](https://www.w3.org/TR/css-position-3/#propdef-position), [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float), or [overflow](https://www.w3.org/TR/CSS2/visufx.html#propdef-overflow)) and whether it is itself participating in a block or inline [formatting context](#formatting-context), it either establishes a new [block formatting context](#block-formatting-context) for its contents or integrates its contents into its parent <a id="ref-for-formatting-context②"></a>formatting context. See [CSS2.1 Chapter 9](https://www.w3.org/TR/CSS2/visuren.html). [\[CSS2\]](#biblio-css2) A [block container](#block-container) that establishes a new <a id="ref-for-block-formatting-context③"></a>block formatting context is considered to have a used [inner display type](#inner-display-type) of [flow-root](#valdef-display-flow-root).

<a id="valdef-display-flow-root"></a>flow-root  
<a id="ref-for-block-formatting-context④"></a>

<a id="ref-for-flow-layout④"></a>

<a id="ref-for-block-container⑤"></a>

The element generates a [block container](#block-container) box, and lays out its contents using [flow layout](#flow-layout). It always establishes a new [block formatting context](#block-formatting-context) for its contents. [\[CSS2\]](#biblio-css2)

<a id="valdef-display-table"></a>table  
<a id="ref-for-table-grid-box③"></a>

<a id="ref-for-block-formatting-context⑤"></a>

<a id="ref-for-table-wrapper-box③"></a>

The element generates a principal [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box) that establishes a [block formatting context](#block-formatting-context), and which contains an additionally-generated [table grid box](https://www.w3.org/TR/css-tables-3/#table-grid-box) that establishes a table formatting context. [\[CSS2\]](#biblio-css2)

<a id="valdef-display-flex"></a>flex  
<a id="ref-for-flex-formatting-context"></a>

<a id="ref-for-flex-container②"></a>

The element generates a principal [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container) box and establishes a [flex formatting context](https://www.w3.org/TR/css-flexbox-1/#flex-formatting-context). [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1)

<a id="valdef-display-grid"></a>grid  
<a id="ref-for-grid-formatting-context"></a>

<a id="ref-for-grid-container②"></a>

The element generates a principal [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) box, and establishes a [grid formatting context](https://www.w3.org/TR/css-grid-2/#grid-formatting-context). [\[CSS-GRID-1\]](#biblio-css-grid-1)

<a id="ref-for-valdef-grid-template-rows-subgrid"></a>

<a id="ref-for-grid-formatting-context①"></a>

(Grids using [subgrid](https://www.w3.org/TR/css-grid-2/#valdef-grid-template-rows-subgrid) might not generate a new [grid formatting context](https://www.w3.org/TR/css-grid-2/#grid-formatting-context); see [\[CSS-GRID-2\]](#biblio-css-grid-2) for details.)

<a id="valdef-display-ruby"></a>ruby  
<a id="ref-for-outer-display-type④"></a>

<a id="ref-for-formatting-context③"></a>

<a id="ref-for-ruby-formatting-context"></a>

<a id="ref-for-ruby-container②"></a>

The element generates a [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container) box and establishes a [ruby formatting context](https://www.w3.org/TR/css-ruby-1/#ruby-formatting-context) in addition to integrating its base-level contents into its parent [formatting context](#formatting-context) (if it is inline) or generating a wrapper box of the appropriate [outer display type](#outer-display-type) (if it is not). [\[CSS-RUBY-1\]](#biblio-css-ruby-1)

<a id="ref-for-typedef-display-inside③"></a>

<a id="ref-for-typedef-display-outside④"></a>

<a id="ref-for-outer-display-type⑤"></a>

<a id="ref-for-valdef-display-block②"></a>

<a id="ref-for-valdef-display-ruby②"></a>

<a id="ref-for-valdef-display-inline③"></a>

If a [\<display-inside\>](#typedef-display-inside) value is specified but [\<display-outside\>](#typedef-display-outside) is omitted, the element’s [outer display type](#outer-display-type) defaults to [block](#valdef-display-block)—​except for [ruby](#valdef-display-ruby), which defaults to [inline](#valdef-display-inline).

Tests

- [display-flow-root-001.html](https://wpt.fyi/results/css/css-display/display-flow-root-001.html) [(live test)](http://wpt.live/css/css-display/display-flow-root-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-flow-root-001.html)
- [display-flow-root-002.html](https://wpt.fyi/results/css/css-display/display-flow-root-002.html) [(live test)](http://wpt.live/css/css-display/display-flow-root-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-flow-root-002.html)
- [display-none-inline-img.html](https://wpt.fyi/results/css/css-display/display-none-inline-img.html) [(live test)](http://wpt.live/css/css-display/display-none-inline-img.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-none-inline-img.html)

<a id="ref-for-valdef-display-list-item③"></a>

### <a id="list-items"></a>2.3.  Generating Marker Boxes: the [list-item](#valdef-display-list-item) keyword

<a id="ref-for-selectordef-marker①"></a>

<a id="ref-for-propdef-list-style"></a>

The <a id="valdef-display-list-item"></a>list-item keyword causes the element to generate a [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4) with the content specified by its [list-style](https://www.w3.org/TR/CSS2/generate.html#propdef-list-style) properties ([CSS 2.1§12.5 Lists](https://www.w3.org/TR/CSS2/generate.html#lists)) [\[CSS2\]](#biblio-css2) together with a principal box of the specified type for its own contents.

<a id="ref-for-inner-display-type④"></a>

<a id="ref-for-valdef-display-flow②"></a>

<a id="ref-for-outer-display-type⑥"></a>

<a id="ref-for-valdef-display-block③"></a>

If no [inner display type](#inner-display-type) value is specified, the principal box’s <a id="ref-for-inner-display-type⑤"></a>inner display type defaults to [flow](#valdef-display-flow). If no [outer display type](#outer-display-type) value is specified, the principal box’s <a id="ref-for-outer-display-type⑦"></a>outer display type defaults to [block](#valdef-display-block).

<a id="ref-for-flow-layout⑤"></a>

<a id="ref-for-valdef-display-block④"></a>

<a id="ref-for-valdef-display-inline④"></a>

<a id="ref-for-valdef-display-run-in④"></a>

<a id="ref-for-valdef-display-flow③"></a>

<a id="ref-for-valdef-display-flow-root③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In this level, as restricted in the grammar, list-items are limited to the [Flow Layout](#flow-layout) display types ([block](#valdef-display-block)/[inline](#valdef-display-inline)/[run-in](#valdef-display-run-in) with [flow](#valdef-display-flow)/[flow-root](#valdef-display-flow-root) inner types). This restriction may be relaxed in a future level of this module.

Tests

- [display-flow-root-list-item-001.html](https://wpt.fyi/results/css/css-display/display-flow-root-list-item-001.html) [(live test)](http://wpt.live/css/css-display/display-flow-root-list-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-flow-root-list-item-001.html)
- [display-list-item-height-after-dom-change.html](https://wpt.fyi/results/css/css-display/display-list-item-height-after-dom-change.html) [(live test)](http://wpt.live/css/css-display/display-list-item-height-after-dom-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-list-item-height-after-dom-change.html)

### <a id="layout-specific-display"></a>2.4.  Layout-Internal Display Types: the table-\* and ruby-\* keywords

<a id="ref-for-valdef-display-table②"></a>

<a id="ref-for-valdef-display-ruby③"></a>

<a id="ref-for-propdef-display②⓪"></a>

Some layout models, such as [table](#valdef-display-table) and [ruby](#valdef-display-ruby), have a complex internal structure, with several different roles that their children and descendants can fill. This section defines those “<a id="layout-internal"></a>layout-internal” [display](#propdef-display) values, which only have meaning within that particular layout mode.

<a id="ref-for-inner-display-type⑥"></a>

<a id="ref-for-outer-display-type⑧"></a>

<a id="ref-for-propdef-display②①"></a>

Unless otherwise specified, both the [inner display type](#inner-display-type) and the [outer display type](#outer-display-type) of elements using these [display](#propdef-display) values are set to the given keyword.

<a id="ref-for-propdef-display②②"></a>

<a id="ref-for-replaced-element④"></a>

<a id="ref-for-layout-internal"></a>

<a id="ref-for-valdef-display-inline⑤"></a>

When the [display](#propdef-display) property of a [replaced element](#replaced-element) computes to one of the [layout-internal](#layout-internal) values, it is handled as having a used value of [inline](#valdef-display-inline). White space collapsing and anonymous box generation must happen around those replaced elements based on that <a id="ref-for-valdef-display-inline⑥"></a>inline value, as if they never had a <a id="ref-for-layout-internal①"></a>layout-internal display value applied to them.

<a id="ref-for-layout-internal②"></a>

<a id="ref-for-replaced-element⑤"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors should not assign a <a href="#layout-internal">layout-internal</a> display value to <a href="#replaced-element">replaced elements</a>.</strong>

<a id="ref-for-typedef-display-internal③"></a>

The [\<display-internal\>](#typedef-display-internal) keywords are defined as follows:

<a id="valdef-display-table-row-group"></a>table-row-group, <a id="valdef-display-table-header-group"></a>table-header-group, <a id="valdef-display-table-footer-group"></a>table-footer-group, <a id="valdef-display-table-row"></a>table-row, <a id="valdef-display-table-cell"></a>table-cell, <a id="valdef-display-table-column-group"></a>table-column-group, <a id="valdef-display-table-column"></a>table-column  
The element is an <a id="internal-table-element"></a>internal table element. It generates the appropriate <a id="internal-table-box"></a>internal table box which participates in a table formatting context. See [CSS2§17.2](https://www.w3.org/TR/CSS2/tables.html#table-display) [\[CSS2\]](#biblio-css2).

<a id="ref-for-valdef-display-table-cell"></a>

<a id="ref-for-valdef-display-flow-root④"></a>

<a id="ref-for-inner-display-type⑦"></a>

[table-cell](#valdef-display-table-cell) boxes have a [flow-root](#valdef-display-flow-root) [inner display type](#inner-display-type).

<a id="valdef-display-table-caption"></a>table-caption  
<a id="ref-for-block-box④"></a>

The element generates a <a id="table-caption-box"></a>table caption box, which is a [block box](#block-box) with special behavior with respect to table and table wrapper boxes. See [CSS2§17.2](https://www.w3.org/TR/CSS2/tables.html#table-display) [\[CSS2\]](#biblio-css2).

<a id="ref-for-valdef-display-table-caption"></a>

<a id="ref-for-valdef-display-flow-root⑤"></a>

<a id="ref-for-inner-display-type⑧"></a>

[table-caption](#valdef-display-table-caption) boxes have a [flow-root](#valdef-display-flow-root) [inner display type](#inner-display-type).

<a id="valdef-display-ruby-base"></a>ruby-base, <a id="valdef-display-ruby-text"></a>ruby-text, <a id="valdef-display-ruby-base-container"></a>ruby-base-container, <a id="valdef-display-ruby-text-container"></a>ruby-text-container  
<a id="ref-for-ruby-formatting-context①"></a>

The element is an <a id="internal-ruby-element"></a>internal ruby element. It generates the appropriate <a id="internal-ruby-box"></a>internal ruby box which participates in a [ruby formatting context](https://www.w3.org/TR/css-ruby-1/#ruby-formatting-context). [\[CSS-RUBY-1\]](#biblio-css-ruby-1)

<a id="ref-for-valdef-display-ruby-base"></a>

<a id="ref-for-valdef-display-ruby-text"></a>

<a id="ref-for-valdef-display-flow④"></a>

<a id="ref-for-inner-display-type⑨"></a>

[ruby-base](#valdef-display-ruby-base) and [ruby-text](#valdef-display-ruby-text) have a [flow](#valdef-display-flow) [inner display type](#inner-display-type).

Boxes with layout-specific display types generate anonymous wrapper boxes around themselves when placed in an incompatible parent, as defined by their respective specifications.

<a id="ref-for-valdef-display-table-cell①"></a>

<a id="ref-for-valdef-display-table-row"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d11b97c3"></a> For example, Table Layout requires that a [table-cell](#valdef-display-table-cell) box must have a [table-row](#valdef-display-table-row) parent box.
>
> If it is misparented, like so:
>
> ```markup
> <div style="display:block;">
>   <div style="display:table-cell">...</div>
> </div>
> ```
>
> It will generate wrapper boxes around itself, producing a structure like:
>
> ```text
> block box
> └anonymous table box
>  └anonymous table-row-group box
>   └anonymous table-row box
>    └table-cell box
> ```
>
> <a id="ref-for-internal-table-element"></a>
>
> Even if the parent is another [internal table element](#internal-table-element), if it’s not the <em>correct</em> one, wrapper boxes will be generated. For example, in the following markup:
>
> ```markup
> <div style="display:table;">
>   <div style="display:table-row">
>     <div style="display:table-cell">...</div>
>   </div>
> </div>
> ```
>
> Anonymous wrapper box generation will produce:
>
> ```text
> table box
> └anonymous table-row-group box
>  └table-row box
>   └table-cell box
> ```
>
> This "fix-up" ensures that table layout has a predictable structure to operate on.

Tests

This section lacks tests.

------------------------------------------------------------------------

<a id="ref-for-valdef-display-none④"></a>

<a id="ref-for-valdef-display-contents②"></a>

### <a id="box-generation"></a>2.5.  Box Generation: the [none](#valdef-display-none) and [contents](#valdef-display-contents) keywords

<a id="ref-for-propdef-display②③"></a>

While [display](#propdef-display) can control the <em>types</em> of boxes an element will generate, it can also control whether an element will generate any boxes at all.

<a id="ref-for-typedef-display-box②"></a>

The [\<display-box\>](#typedef-display-box) keywords are defined as follows:

<a id="valdef-display-contents"></a>contents  
<a id="ref-for-selectordef-after"></a>

<a id="ref-for-selectordef-before"></a>

<a id="ref-for-element-tree①"></a>

<a id="ref-for-css-text-sequence⑥"></a>

<a id="ref-for-box⑨"></a>

The element itself does not generate any boxes, but its children and pseudo-elements still generate [boxes](#box) and [text sequences](#css-text-sequence) as normal. For the purposes of box generation and layout, the element must be treated as if it had been replaced in the [element tree](#element-tree) by its contents (including both its source-document children and its pseudo-elements, such as [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements, which are generated before/after the element’s children as normal).

<a id="ref-for-inheritance"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As only the box tree is affected, any semantics based on the document tree, such as selector-matching, event handling, and property [inheritance](https://www.w3.org/TR/css-cascade-5/#inheritance), are not affected. <strong>As of writing, however,
			<a href="https://github.com/w3c/csswg-drafts/issues/3040">this is not implemented correctly in major browsers</a>,
			so using this feature on the Web must be done with care
			as it can prevent accessibility tools
			from accessing the element’s semantics.</strong>

<a id="ref-for-propdef-display②④"></a>

This value computes to [display: none](#propdef-display) on replaced elements and other elements whose rendering is not entirely controlled by CSS; see [Appendix B: Effects of display: contents on Unusual Elements](#unbox) for details.

<a id="ref-for-propdef-display②⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Replaced elements and form controls are treated specially because removing only the element’s own generating box is a more-or-less undefined operation. As this behavior may be refined if use cases (and more precise rendering models) develop, authors should use [display: none](#propdef-display) rather than <a id="ref-for-propdef-display②⑥"></a>display: contents on such elements for forward-compatibility.

<a id="valdef-display-none"></a>none  
<a id="ref-for-css-text-sequence⑦"></a>

<a id="ref-for-box①⓪"></a>

<a id="ref-for-elements⑥"></a>

The [element](#elements) and its descendants generate no [boxes](#box) or [text sequences](#css-text-sequence).

<a id="ref-for-text-nodes⑤"></a>

<a id="ref-for-propdef-display②⑦"></a>

<a id="ref-for-css-text-sequence⑧"></a>

Similarly, if a [text node](#text-nodes) is defined to behave as [display: none](#propdef-display), it generates no [text sequences](#css-text-sequence).

<a id="ref-for-inner-display-type①⓪"></a>

<a id="ref-for-outer-display-type⑨"></a>

Elements with either of these values do not have [inner](#inner-display-type) or [outer display types](#outer-display-type), because they don’t generate any boxes at all.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As these values cause affected elements to not generate a box, anonymous box generation rules will ignore the elided elements entirely, as if they did not exist in the box tree.

<a id="ref-for-the-summary-element"></a>

<a id="ref-for-the-legend-element"></a>

<a id="ref-for-the-fieldset-element"></a>

Markup-based relationships, however, are not affected by these values, as they are solely rendering-time effects. For example, although they may affect which table cell <em>appears</em> in a column, they do not affect which table cell is associated with a particular column <em>element</em>. Similarly, they cannot affect which HTML <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-summary-element">summary</a></code> element is associated with a particular table or whether a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-legend-element">legend</a></code> is considered to be labelling the contents of a particular <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element">fieldset</a></code>.

Tests

- [display-contents-001-crash.html](https://wpt.fyi/results/css/css-display/display-contents-001-crash.html) [(live test)](http://wpt.live/css/css-display/display-contents-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-001-crash.html)
- [display-contents-alignment-001.html](https://wpt.fyi/results/css/css-display/display-contents-alignment-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-alignment-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-alignment-001.html)
- [display-contents-alignment-002.html](https://wpt.fyi/results/css/css-display/display-contents-alignment-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-alignment-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-alignment-002.html)
- [display-contents-before-after-001.html](https://wpt.fyi/results/css/css-display/display-contents-before-after-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-before-after-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-before-after-001.html)
- [display-contents-before-after-002.html](https://wpt.fyi/results/css/css-display/display-contents-before-after-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-before-after-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-before-after-002.html)
- [display-contents-before-after-003.html](https://wpt.fyi/results/css/css-display/display-contents-before-after-003.html) [(live test)](http://wpt.live/css/css-display/display-contents-before-after-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-before-after-003.html)
- [display-contents-block-001.html](https://wpt.fyi/results/css/css-display/display-contents-block-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-block-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-block-001.html)
- [display-contents-block-002.html](https://wpt.fyi/results/css/css-display/display-contents-block-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-block-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-block-002.html)
- [display-contents-blockify-dynamic.html](https://wpt.fyi/results/css/css-display/display-contents-blockify-dynamic.html) [(live test)](http://wpt.live/css/css-display/display-contents-blockify-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-blockify-dynamic.html)
- [display-contents-button.html](https://wpt.fyi/results/css/css-display/display-contents-button.html) [(live test)](http://wpt.live/css/css-display/display-contents-button.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-button.html)
- [display-contents-computed-style.html](https://wpt.fyi/results/css/css-display/display-contents-computed-style.html) [(live test)](http://wpt.live/css/css-display/display-contents-computed-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-computed-style.html)
- [display-contents-details-001.html](https://wpt.fyi/results/css/css-display/display-contents-details-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-details-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-details-001.html)
- [display-contents-details.html](https://wpt.fyi/results/css/css-display/display-contents-details.html) [(live test)](http://wpt.live/css/css-display/display-contents-details.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-details.html)
- [display-contents-dynamic-before-after-001.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-before-after-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-before-after-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-before-after-001.html)
- [display-contents-dynamic-before-after-first-letter-001.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-before-after-first-letter-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-before-after-first-letter-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-before-after-first-letter-001.html)
- [display-contents-dynamic-fieldset-legend-001.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-fieldset-legend-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-fieldset-legend-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-fieldset-legend-001.html)
- [display-contents-dynamic-flex-001-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-001-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-001-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-001-inline.html)
- [display-contents-dynamic-flex-001-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-001-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-001-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-001-none.html)
- [display-contents-dynamic-flex-002-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-002-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-002-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-002-inline.html)
- [display-contents-dynamic-flex-002-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-002-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-002-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-002-none.html)
- [display-contents-dynamic-flex-003-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-003-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-003-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-003-inline.html)
- [display-contents-dynamic-flex-003-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-flex-003-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-flex-003-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-flex-003-none.html)
- [display-contents-dynamic-generated-content-fieldset-001.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-generated-content-fieldset-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-generated-content-fieldset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-generated-content-fieldset-001.html)
- [display-contents-dynamic-inline-flex-001-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-inline-flex-001-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-inline-flex-001-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-inline-flex-001-inline.html)
- [display-contents-dynamic-inline-flex-001-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-inline-flex-001-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-inline-flex-001-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-inline-flex-001-none.html)
- [display-contents-dynamic-list-001-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-list-001-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-list-001-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-list-001-inline.html)
- [display-contents-dynamic-list-001-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-list-001-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-list-001-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-list-001-none.html)
- [display-contents-dynamic-multicol-001-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-multicol-001-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-multicol-001-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-multicol-001-inline.html)
- [display-contents-dynamic-multicol-001-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-multicol-001-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-multicol-001-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-multicol-001-none.html)
- [display-contents-dynamic-pseudo-insertion-001.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-pseudo-insertion-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-pseudo-insertion-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-pseudo-insertion-001.html)
- [display-contents-dynamic-table-001-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-table-001-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-table-001-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-table-001-inline.html)
- [display-contents-dynamic-table-001-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-table-001-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-table-001-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-table-001-none.html)
- [display-contents-dynamic-table-002-inline.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-table-002-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-table-002-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-table-002-inline.html)
- [display-contents-dynamic-table-002-none.html](https://wpt.fyi/results/css/css-display/display-contents-dynamic-table-002-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-dynamic-table-002-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-dynamic-table-002-none.html)
- [display-contents-fieldset-002.html](https://wpt.fyi/results/css/css-display/display-contents-fieldset-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-fieldset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-fieldset-002.html)
- [display-contents-fieldset-nested-legend.html](https://wpt.fyi/results/css/css-display/display-contents-fieldset-nested-legend.html) [(live test)](http://wpt.live/css/css-display/display-contents-fieldset-nested-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-fieldset-nested-legend.html)
- [display-contents-fieldset.html](https://wpt.fyi/results/css/css-display/display-contents-fieldset.html) [(live test)](http://wpt.live/css/css-display/display-contents-fieldset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-fieldset.html)
- [display-contents-first-letter-001.html](https://wpt.fyi/results/css/css-display/display-contents-first-letter-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-first-letter-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-first-letter-001.html)
- [display-contents-first-letter-002.html](https://wpt.fyi/results/css/css-display/display-contents-first-letter-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-first-letter-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-first-letter-002.html)
- [display-contents-first-line-001.html](https://wpt.fyi/results/css/css-display/display-contents-first-line-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-first-line-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-first-line-001.html)
- [display-contents-first-line-002.html](https://wpt.fyi/results/css/css-display/display-contents-first-line-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-first-line-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-first-line-002.html)
- [display-contents-flex-001.html](https://wpt.fyi/results/css/css-display/display-contents-flex-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-flex-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-flex-001.html)
- [display-contents-flex-002.html](https://wpt.fyi/results/css/css-display/display-contents-flex-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-flex-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-flex-002.html)
- [display-contents-flex-003.html](https://wpt.fyi/results/css/css-display/display-contents-flex-003.html) [(live test)](http://wpt.live/css/css-display/display-contents-flex-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-flex-003.html)
- [display-contents-float-001.html](https://wpt.fyi/results/css/css-display/display-contents-float-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-float-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-float-001.html)
- [display-contents-focusable-001.html](https://wpt.fyi/results/css/css-display/display-contents-focusable-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-focusable-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-focusable-001.html)
- [display-contents-inline-001.html](https://wpt.fyi/results/css/css-display/display-contents-inline-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-inline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-inline-001.html)
- [display-contents-inline-002.html](https://wpt.fyi/results/css/css-display/display-contents-inline-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-inline-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-inline-002.html)
- [display-contents-inline-flex-001.html](https://wpt.fyi/results/css/css-display/display-contents-inline-flex-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-inline-flex-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-inline-flex-001.html)
- [display-contents-line-height.html](https://wpt.fyi/results/css/css-display/display-contents-line-height.html) [(live test)](http://wpt.live/css/css-display/display-contents-line-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-line-height.html)
- [display-contents-list-001.html](https://wpt.fyi/results/css/css-display/display-contents-list-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-list-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-list-001.html)
- [display-contents-multicol-001.html](https://wpt.fyi/results/css/css-display/display-contents-multicol-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-multicol-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-multicol-001.html)
- [display-contents-oof-001.html](https://wpt.fyi/results/css/css-display/display-contents-oof-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-oof-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-oof-001.html)
- [display-contents-oof-002.html](https://wpt.fyi/results/css/css-display/display-contents-oof-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-oof-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-oof-002.html)
- [display-contents-parsing-001.html](https://wpt.fyi/results/css/css-display/display-contents-parsing-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-parsing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-parsing-001.html)
- [display-contents-pseudo-click-target.html](https://wpt.fyi/results/css/css-display/display-contents-pseudo-click-target.html) [(live test)](http://wpt.live/css/css-display/display-contents-pseudo-click-target.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-pseudo-click-target.html)
- [display-contents-root-background.html](https://wpt.fyi/results/css/css-display/display-contents-root-background.html) [(live test)](http://wpt.live/css/css-display/display-contents-root-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-root-background.html)
- [display-contents-shadow-dom-1.html](https://wpt.fyi/results/css/css-display/display-contents-shadow-dom-1.html) [(live test)](http://wpt.live/css/css-display/display-contents-shadow-dom-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-shadow-dom-1.html)
- [display-contents-shadow-host-whitespace.html](https://wpt.fyi/results/css/css-display/display-contents-shadow-host-whitespace.html) [(live test)](http://wpt.live/css/css-display/display-contents-shadow-host-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-shadow-host-whitespace.html)
- [display-contents-sharing-001.html](https://wpt.fyi/results/css/css-display/display-contents-sharing-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-sharing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-sharing-001.html)
- [display-contents-slot-attach-whitespace.html](https://wpt.fyi/results/css/css-display/display-contents-slot-attach-whitespace.html) [(live test)](http://wpt.live/css/css-display/display-contents-slot-attach-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-slot-attach-whitespace.html)
- [display-contents-state-change-001.html](https://wpt.fyi/results/css/css-display/display-contents-state-change-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-state-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-state-change-001.html)
- [display-contents-suppression-dynamic-001.html](https://wpt.fyi/results/css/css-display/display-contents-suppression-dynamic-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-suppression-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-suppression-dynamic-001.html)
- [display-contents-svg-anchor-child.html](https://wpt.fyi/results/css/css-display/display-contents-svg-anchor-child.html) [(live test)](http://wpt.live/css/css-display/display-contents-svg-anchor-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-svg-anchor-child.html)
- [display-contents-svg-elements.html](https://wpt.fyi/results/css/css-display/display-contents-svg-elements.html) [(live test)](http://wpt.live/css/css-display/display-contents-svg-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-svg-elements.html)
- [display-contents-svg-switch-child.html](https://wpt.fyi/results/css/css-display/display-contents-svg-switch-child.html) [(live test)](http://wpt.live/css/css-display/display-contents-svg-switch-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-svg-switch-child.html)
- [display-contents-table-001.html](https://wpt.fyi/results/css/css-display/display-contents-table-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-table-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-table-001.html)
- [display-contents-table-002.html](https://wpt.fyi/results/css/css-display/display-contents-table-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-table-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-table-002.html)
- [display-contents-td-001.html](https://wpt.fyi/results/css/css-display/display-contents-td-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-td-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-td-001.html)
- [display-contents-text-inherit-002.html](https://wpt.fyi/results/css/css-display/display-contents-text-inherit-002.html) [(live test)](http://wpt.live/css/css-display/display-contents-text-inherit-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-text-inherit-002.html)
- [display-contents-text-inherit.html](https://wpt.fyi/results/css/css-display/display-contents-text-inherit.html) [(live test)](http://wpt.live/css/css-display/display-contents-text-inherit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-text-inherit.html)
- [display-contents-text-only-001.html](https://wpt.fyi/results/css/css-display/display-contents-text-only-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-text-only-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-text-only-001.html)
- [display-contents-tr-001.html](https://wpt.fyi/results/css/css-display/display-contents-tr-001.html) [(live test)](http://wpt.live/css/css-display/display-contents-tr-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-tr-001.html)
- [display-contents-unusual-html-elements-none.html](https://wpt.fyi/results/css/css-display/display-contents-unusual-html-elements-none.html) [(live test)](http://wpt.live/css/css-display/display-contents-unusual-html-elements-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-unusual-html-elements-none.html)
- [display-contents-whitespace-inside-inline.html](https://wpt.fyi/results/css/css-display/display-contents-whitespace-inside-inline.html) [(live test)](http://wpt.live/css/css-display/display-contents-whitespace-inside-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-display/display-contents-whitespace-inside-inline.html)

### <a id="legacy-display"></a>2.6.  Precomposed Inline-level Display Values

<a id="ref-for-propdef-display②⑧"></a>

<a id="ref-for-typedef-display-legacy①"></a>

CSS level 2 used a single-keyword syntax for [display](#propdef-display), requiring separate keywords for block-level and inline-level variants of the same layout mode. These [\<display-legacy\>](#typedef-display-legacy) keywords map as follows:

<a id="valdef-display-inline-block"></a>inline-block  
Computes to inline flow-root.

<a id="valdef-display-inline-table"></a>inline-table  
Computes to inline table.

<a id="valdef-display-inline-flex"></a>inline-flex  
Computes to inline flex.

<a id="valdef-display-inline-grid"></a>inline-grid  
Computes to inline grid.

<a id="ref-for-specified-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although these keywords and their equivalents compute to the same value, their [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value) remain distinct.

<a id="ref-for-dom-window-getcomputedstyle①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> serialization rules will always output these precomposed keywords rather than the equivalent two-keyword pairs due to the [shortest, most backwards-compatible serialization principle](https://www.w3.org/TR/cssom-1/#serializing-css-values).

Tests

This section lacks tests.

------------------------------------------------------------------------

### <a id="transformations"></a>2.7.  Automatic Box Type Transformations

<a id="ref-for-computed-value①"></a>

<a id="ref-for-outer-display-type①⓪"></a>

<a id="ref-for-valdef-display-block⑤"></a>

<a id="ref-for-valdef-display-inline⑦"></a>

<a id="ref-for-display-type②"></a>

<a id="ref-for-valdef-display-none⑤"></a>

<a id="ref-for-valdef-display-contents③"></a>

Some layout effects require <a id="blockify"></a>blockification or <a id="inlinify"></a>inlinification of the box type, which sets the box’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [outer display type](#outer-display-type) to [block](#valdef-display-block) or [inline](#valdef-display-inline) (respectively). (This has no effect on [display types](#display-type) that generate no box at all, such as [none](#valdef-display-none) or [contents](#valdef-display-contents).) Additionally:

- <a id="ref-for-block-box⑤"></a>

  <a id="ref-for-inlinify"></a>

  <a id="ref-for-inner-display-type①①"></a>

  <a id="ref-for-valdef-display-flow-root⑥"></a>

  If a [block box](#block-box) (block flow) is [inlinified](#inlinify), its [inner display type](#inner-display-type) is set to [flow-root](#valdef-display-flow-root) so that it remains a block container.

- <a id="ref-for-inline-box⑤"></a>

  <a id="ref-for-inlinify①"></a>

  <a id="ref-for-in-flow"></a>

  If an [inline box](#inline-box) (inline flow) is [inlinified](#inlinify), it recursively <a id="ref-for-inlinify②"></a>inlinifies all of its [in-flow](#in-flow) children, so that no block-level descendants break up the inline formatting context in which it participates.

- <a id="ref-for-inline-block"></a>

  <a id="ref-for-blockify"></a>

  <a id="ref-for-valdef-display-block⑥"></a>

  <a id="ref-for-valdef-display-flow-root⑦"></a>

  For legacy reasons, if an [inline block box](#inline-block) (inline flow-root) is [blockified](#blockify), it becomes a [block](#valdef-display-block) box (losing its [flow-root](#valdef-display-flow-root) nature). For consistency, a run-in flow-root box also <a id="ref-for-blockify①"></a>blockifies to a <a id="ref-for-valdef-display-block⑦"></a>block box.

- <a id="ref-for-layout-internal③"></a>

  <a id="ref-for-blockify②"></a>

  <a id="ref-for-inner-display-type①②"></a>

  <a id="ref-for-valdef-display-flow⑤"></a>

  <a id="ref-for-block-container⑥"></a>

  <a id="ref-for-inlinify③"></a>

  If a [layout-internal](#layout-internal) box is [blockified](#blockify), its [inner display type](#inner-display-type) converts to [flow](#valdef-display-flow) so that it becomes a [block container](#block-container). [Inlinification](#inlinify) has no effect on <a id="ref-for-layout-internal④"></a>layout-internal boxes. (However, placement in such an inline context will typically cause them to be wrapped in an appropriately-typed anonymous inline-level box.)

<a id="ref-for-computed-value②"></a>

<a id="ref-for-propdef-display②⑨"></a>

<a id="ref-for-blockify③"></a>

<a id="ref-for-inlinify④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There are two methods used to fix up box types when a box is mismatched to its context. One is transformation of the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [display](#propdef-display), such as [blockification](#blockify) and [inlinification](#inlinify) described here. The other, taking place during [box tree construction](#intro) (after computed values have been determined), is the creation of intermediary anonymous boxes, such as happens in [tables](https://www.w3.org/TR/CSS2/tables.html#anonymous-boxes), [ruby](https://www.w3.org/TR/css-ruby-1/#box-fixup), and [flow](https://www.w3.org/TR/CSS21/visuren.html#box-gen) layout.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-26cc3423"></a> Some examples of computed-value fixup include:
>
> - <a id="ref-for-blockify④"></a>
>
>   Absolute positioning or floating an element [blockifies](#blockify) the box’s display type. [\[CSS2\]](#biblio-css2)
>
> - <a id="ref-for-inlinify⑤"></a>
>
>   <a id="ref-for-ruby-container③"></a>
>
>   Containment in a [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container) [inlinifies](#inlinify) the box’s display type, as described in [\[CSS-RUBY-1\]](#biblio-css-ruby-1).
>
> - <a id="ref-for-blockify⑤"></a>
>
>   <a id="ref-for-propdef-display③⓪"></a>
>
>   <a id="ref-for-valdef-display-flex②"></a>
>
>   <a id="ref-for-valdef-display-grid②"></a>
>
>   A parent with a [grid](#valdef-display-grid) or [flex](#valdef-display-flex) [display](#propdef-display) value [blockifies](#blockify) the box’s display type. [\[CSS-GRID-1\]](#biblio-css-grid-1) [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1)

### <a id="root"></a>2.8.  The Root Element’s Principal Box

<a id="ref-for-root-element"></a>

<a id="ref-for-blockify⑥"></a>

<a id="ref-for-principal-box⑧"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-initial-containing-block"></a>

The [root element](#root-element)’s display type is always [blockified](#blockify), and its [principal box](#principal-box) always establishes an [independent formatting context](#independent-formatting-context). This box’s [containing block](#containing-block) is the [initial containing block](#initial-containing-block).

<a id="ref-for-propdef-display③①"></a>

<a id="ref-for-valdef-display-contents④"></a>

<a id="ref-for-valdef-display-block⑧"></a>

Additionally, a [display](#propdef-display) of [contents](#valdef-display-contents) computes to [block](#valdef-display-block) on the root element.

<a id="ref-for-propdef-order"></a>

## <a id="order-property"></a>3.  Display Order: the [order](#propdef-order) property

| Field               | Definition                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-order"></a>order                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-integer-value"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value)                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-grid-item"></a><a id="ref-for-flex-item"></a>[flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) and [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified integer                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                           |

Tests

- [flexible-order.html](https://wpt.fyi/results/css/css-flexbox/flexible-order.html) [(live test)](http://wpt.live/css/css-flexbox/flexible-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexible-order.html)

<a id="ref-for-formatting-context④"></a>

<a id="ref-for-propdef-order①"></a>

Boxes are generally displayed and laid out in the same order as they appear in the source document. In some [formatting contexts](#formatting-context), the [order](#propdef-order) property can be used to rearrange the order of boxes to deliberately create a divergence of the logical order of elements and their spatial arrangement on the 2D visual canvas. (See [§ 3.1 Reordering and Accessibility](#order-accessibility).)

<a id="ref-for-propdef-order②"></a>

<a id="ref-for-flex-item①"></a>

<a id="ref-for-grid-item①"></a>

<a id="ref-for-integer-value①"></a>

Specifically, the [order](#propdef-order) property controls the order in which [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) or [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) appear within their container, by assigning them to ordinal groups. It takes a single <a id="valdef-order-integer"></a>[\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) value, which specifies which ordinal group the item belongs to.

<a id="ref-for-propdef-order③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c8720ec3"></a> Here’s an example of a catalog item card which has a title, a photo, and a description. Within each entry, the source document content is ordered logically with the title first, followed by the description and the photo. This provides a sensible ordering for speech rendering and in non-CSS browsers. For a more compelling visual presentation, however, [order](#propdef-order) is used to pull the image up from later in the content to the top of the card.
>
> ```css
> article.sale-item {
>   display: flex;
>   flex-flow: column;
> }
> article.sale-item > img {
>   order: -1; /* Shift image before other content (in layout order) */
>   align-self: center;
> }
> ```
>
> ```markup
> <article class="sale-item">
>   <h3>Computer Starter Kit</h3>
>   <p>This is the best computer money can buy, if you don’t have much money.
>   <ul>
>     <li>Computer
>     <li>Monitor
>     <li>Keyboard
>     <li>Mouse
>   </ul>
>   <img src="images/computer.jpg"
>     alt="You get: a white desktop computer with matching peripherals."
>     width="250" height="188">
> </article>
> ```
>
> <a id="order-example"></a> ![You get: a white desktop computer with matching keyboard and monitor.](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/images/computer.jpg)
>
> ### <a id="quiet-pubrules-1"></a>Computer Starter Kit
>
> This is the best computer money can buy, if you don’t have much money.
>
> - Computer
> - Monitor
> - Keyboard
> - Mouse
>
> An example rendering of the code above.

<a id="ref-for-flex-container③"></a>

<a id="ref-for-grid-container③"></a>

<a id="ref-for-flex-item②"></a>

<a id="ref-for-grid-item②"></a>

<a id="ref-for-propdef-order④"></a>

[Flex](https://www.w3.org/TR/css-flexbox-1/#flex-container) and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) lay out their contents in <a id="order-modified-document-order"></a>order-modified document order, starting from the lowest numbered ordinal group and going up. Items with the same ordinal group are laid out in the order they appear in the source document. This also affects the [painting order](https://www.w3.org/TR/CSS2/zindex.html) [\[CSS2\]](#biblio-css2), exactly as if the [flex](https://www.w3.org/TR/css-flexbox-1/#flex-item)/[grid items](https://www.w3.org/TR/css-grid-2/#grid-item) were reordered in the source document. Absolutely-positioned children of a <a id="ref-for-flex-container④"></a>flex/<a id="ref-for-grid-container④"></a>grid container are treated as having [order: 0](#propdef-order) for the purpose of determining their painting order relative to <a id="ref-for-flex-item③"></a>flex/<a id="ref-for-grid-item③"></a>grid items.

<a id="ref-for-flex-item④"></a>

<a id="ref-for-grid-item④"></a>

Unless otherwise specified by a future specification, this property has no effect on boxes that are not [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) or [grid items](https://www.w3.org/TR/css-grid-2/#grid-item).

### <a id="order-accessibility"></a>3.1.  Reordering and Accessibility

<a id="ref-for-propdef-order⑤"></a>

The [order](#propdef-order) property <em>does not</em> affect ordering in non-visual media (such as [speech](https://www.w3.org/TR/css-speech-1/)). Likewise, <a id="ref-for-propdef-order⑥"></a>order does not affect the default traversal order of sequential navigation modes (such as cycling through links, see e.g. [`tabindex`](https://html.spec.whatwg.org/multipage/interaction.html#attr-tabindex) [\[HTML\]](#biblio-html)).

<a id="ref-for-propdef-order⑦"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong>
	Authors <em>must</em> use <a href="#propdef-order">order</a> only for spatial, not logical, reordering of content.
	Style sheets that use <span><span><a id="ref-for-propdef-order⑧"></a></span>order</span> to perform logical reordering are non-conforming.</strong>

<a id="ref-for-propdef-order⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is so that non-visual media and non-CSS UAs, which typically present content linearly, can rely on a logical source order, while [order](#propdef-order) is used to tailor the layout order. (Since visual perception is two-dimensional and non-linear, the desired layout order is not always logical.)

<a id="ref-for-propdef-order①⓪"></a>

In order to preserve the author’s intended ordering in all presentation modes, authoring tools—​including WYSIWYG editors as well as Web-based authoring aids—​must reorder the underlying document source and not use [order](#propdef-order) to perform reordering unless the author has explicitly indicated that the spatial order should be <em>out-of-sync</em> with the underlying document order (which determines speech and navigation order).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b20b6e0a"></a> For example, a tool might offer both drag-and-drop reordering of flex items as well as handling of media queries for alternate layouts per screen size range.
>
> <a id="ref-for-propdef-order①①"></a>
>
> Since most of the time, reordering should affect all screen ranges as well as navigation and speech order, the tool would perform drag-and-drop reordering at the DOM layer. In some cases, however, the author may want different layouts per screen size. The tool could offer this functionality by using [order](#propdef-order) together with media queries, but also tie the smallest screen size’s ordering to the underlying DOM order (since this is most likely to be a logical linear presentation order) while using <a id="ref-for-propdef-order①②"></a>order to define the visual presentation order in other size ranges.
>
> <a id="ref-for-propdef-order①③"></a>
>
> This tool would be conformant, whereas a tool that only ever used [order](#propdef-order) to handle drag-and-drop reordering (however convenient it might be to implement it that way) would be non-conformant.

<a id="ref-for-propdef-order①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents, including browsers, accessible technology, and extensions, may offer spatial navigation features. This section does not preclude respecting the [order](#propdef-order) property when determining element ordering in such spatial navigation modes; indeed it would need to be considered for such a feature to work. But <a id="ref-for-propdef-order①⑤"></a>order is not the only (or even the primary) CSS property that would need to be considered for such a spatial navigation feature. A well-implemented spatial navigation feature would need to consider all the layout features of CSS that modify spatial relationships.

<a id="ref-for-propdef-visibility"></a>

## <a id="visibility"></a>4.  Invisibility: the [visibility](#propdef-visibility) property

| Field               | Definition                                                                                                          |
|---------------------|---------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-visibility"></a>visibility                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②⑦"></a>visible [\|](https://www.w3.org/TR/css-values-4/#comb-one) hidden <a id="ref-for-comb-one②⑧"></a>\| collapse |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | visible                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                            |
| <strong>Media:&#xA;      </strong> | visual                                                                                                              |

<a id="ref-for-propdef-visibility①"></a>

<a id="ref-for-invisible-box"></a>

<a id="ref-for-propdef-display③②"></a>

<a id="ref-for-valdef-display-none⑥"></a>

The [visibility](#propdef-visibility) property specifies whether the box is rendered. [Invisible](#invisible-box) boxes still affect layout. (Set the [display](#propdef-display) property to [none](#valdef-display-none) to suppress box generation altogether.). Values have the following meanings:

<a id="valdef-visibility-visible"></a>visible  
The generated box is visible, as normal.

<a id="valdef-visibility-hidden"></a>hidden  
<a id="ref-for-propdef-visibility②"></a>

<a id="ref-for-invisible-box①"></a>

Any boxes generated by the element are [invisible](#invisible-box). Descendants of the element can, however, be visible if they have [visibility: visible](#propdef-visibility).

<a id="valdef-visibility-collapse"></a>collapse  
<a id="ref-for-valdef-visibility-hidden"></a>

<a id="ref-for-invisible-box②"></a>

Indicates that the box is <a id="collapsed"></a>collapsed, which can cause it to take up less space than otherwise in a formatting-context–specific way. See [dynamic row and column effects in tables](https://www.w3.org/TR/CSS2/tables.html#dynamic-effects) [\[CSS2\]](#biblio-css2) and [collapsed flex items](https://www.w3.org/TR/css-flexbox-1/#visibility-collapse) in flex layout [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1). In all other cases, however, (i.e. unless otherwise specified) this simply makes the box [invisible](#invisible-box), just like [hidden](#valdef-visibility-hidden).

<a id="ref-for-invisible-box③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Currently, many user agents and/or accessibility tools don’t correctly implement the accessibility implications of visible elements with semantic relationships to [invisible](#invisible-box) elements, so, for example, making parent elements with special roles (such as table rows) <a id="ref-for-invisible-box④"></a>invisible while leaving child elements with special roles (such as table cells) visible can be problematic for users of those tools. Authors should avoid creating these situations until the tooling situation improves.

<a id="ref-for-propdef-pointer-events"></a>

<a id="ref-for-propdef-display③③"></a>

<a id="ref-for-propdef-speak"></a>

<a id="ref-for-valdef-speak-always"></a>

<a id="ref-for-valdef-visibility-visible"></a>

<a id="invisible-box"></a>Invisible boxes are not rendered (as if they were fully transparent), cannot be interacted with (and behave as if they had [pointer-events: none](https://www.w3.org/TR/css-ui-4/#propdef-pointer-events)), are removed from navigation (similar to [display: none](#propdef-display)), and are also not rendered to speech (except when [speak](https://www.w3.org/TR/css-speech-1/#propdef-speak) is [always](https://www.w3.org/TR/css-speech-1/#valdef-speak-always) [\[CSS-SPEECH-1\]](#biblio-css-speech-1)). However, as with <a id="ref-for-propdef-display③④"></a>display: contents, their semantic role as a container is not affected, to ensure that any [visible](#valdef-visibility-visible) descendants are properly interpreted.

<a id="ref-for-propdef-speak①"></a>

<a id="ref-for-valdef-speak-always①"></a>

<a id="ref-for-invisible-box⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [speak](https://www.w3.org/TR/css-speech-1/#propdef-speak) is [always](https://www.w3.org/TR/css-speech-1/#valdef-speak-always), an otherwise [invisible](#invisible-box) box <em>is</em> rendered to speech, and may be interacted with using non-visual/spatial methods.

<a id="ref-for-propdef-display③⑤"></a>

<a id="ref-for-propdef-visibility③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a0db1b94"></a> While temporarily hiding things with [display: none](#propdef-display) is often sufficient, doing so removes the elements from layout entirely, possibly causing unwanted movement or reflow of the page when an element is hidden or shown. [visibility: hidden](#propdef-visibility) can instead be used to keep the page’s layout stable as something is hidden and displayed.
>
> For example, here is a (deliberately simplified) possible implementation of a "spoiler" element that can be revealed by clicking on the hidden text:
>
> ```text
> <p>The symbolism earlier in the movie becomes obvious at the end,
>   when it's revealed that <spoiler-text><span>Luke is his own father</span></spoiler-text>,
>   making the wizard's cryptic riddles meaningful.
> <style>
> spoiler-text { border-bottom: 1px solid; }
> spoiler-text > span { visibility: hidden; }
> spoiler-text.shown > span { visibility: visible; }
> </style>
> <script>
> [...document.querySelectorAll("spoiler-text")].forEach(el=>{
>   el.addEventListener("click", e=>el.classList.toggle("shown"));
> });
> </script>
> ```
>
> <a id="ref-for-propdef-visibility④"></a>
>
> <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> This example is deliberately significantly simplified.
		It is missing a number of accessibility and UX features
		that a well-designed spoiler element would have
		to show off the <a href="#propdef-visibility">visibility</a> usage more plainly.
		Don’t copy this code for a real site.</strong>

Tests

This section lacks tests.

------------------------------------------------------------------------

## <a id="run-in-layout"></a>5.  Run-In Layout

A <a id="run-in"></a>run-in box is a box that <em>merges into</em> a block that comes after it, inserting itself at the beginning of that block’s inline-level content. This is useful for formatting compact headlines, definitions, and other similar things, where the appropriate DOM structure is to have a headline preceding the following prose, but the desired display is an inline headline laying out with the text.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8e61beaa"></a> For example, dictionary definitions are often formatted so that the word is inline with the definition:
>
> ```text
> <dl class='dict'>
>   <dt>dictionary
>   <dd>a book that lists the words of a language in alphabetical
>       order and gives their meaning, or that gives the equivalent
>       words in a different language.
>   <dt>glossary
>   <dd>an alphabetical list of terms or words found in or relating
>       to a specific subject, text, or dialect, with explanations; a
>       brief dictionary.
> </dl>
> <style>
> .dict > dt {
>   display: run-in;
> }
> .dict > dt::after {
>   content: ": "
> }
> </style>
> ```
>
> Which is formatted as:
>
> ```text
> dictionary: a book that lists the words of a language
> in alphabetical order and explains their meaning.
> 
> glossary: an alphabetical list of terms or words found
> in or relating to a specific subject, text, or dialect,
> with explanations; a brief dictionary.
> ```
<a id="ref-for-run-in②"></a>

<a id="ref-for-inline-level-box①"></a>

A [run-in box](#run-in) behaves exactly as any other [inline-level box](#inline-level-box), except:

- <a id="ref-for-inlinify⑥"></a>

  <a id="ref-for-inner-display-type①③"></a>

  <a id="ref-for-valdef-display-flow⑥"></a>

  <a id="ref-for-run-in③"></a>

  A [run-in box](#run-in) with a [flow](#valdef-display-flow) [inner display type](#inner-display-type) [inlinifies](#inlinify) its contents.

- <a id="ref-for-selectordef-before①"></a>

  <a id="ref-for-selectordef-marker②"></a>

  <a id="ref-for-block-formatting-context⑥"></a>

  <a id="ref-for-run-in-sequence"></a>

  If a [run-in sequence](#run-in-sequence) is immediately followed by a block box that does not establish a new [block formatting context](#block-formatting-context), it is inserted as direct children of that block box: after its [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-element’s boxes (if any), but preceding any other boxes generated by the contents of the block (including the box generated by the [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) pseudo-element, if any). This re-parenting recurses if possible (so that the run-in effectively becomes part of the deepest subsequent “paragraph” in its formatting context, collecting newly-adjacent run-ins as it goes).

  The reparented content is then formatted as if originally parented there. <strong data-conversion-semantic="note">Note:</strong> Note that only layout is affected, not inheritance, because property inheritance for non-anonymous boxes is based only on the element tree.

- <a id="ref-for-run-in-sequence①"></a>

  Otherwise (if the [run-in sequence](#run-in-sequence) is <em>not</em> followed by such a block), an anonymous block box is generated around the <a id="ref-for-run-in-sequence②"></a>run-in sequence and all immediately following inline-level content (up to, but not including, the next <a id="ref-for-run-in-sequence③"></a>run-in sequence, if any).

<a id="ref-for-run-in④"></a>

<a id="ref-for-white-space"></a>

<a id="ref-for-out-of-flow"></a>

A <a id="run-in-sequence"></a>run-in sequence is a maximal sequence of consecutive sibling [run-in boxes](#run-in) and intervening [white space](https://www.w3.org/TR/css-text-4/#white-space) and/or [out-of-flow](#out-of-flow) boxes.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This statement implies that out-of-flow boxes are reparented if they are between two run-in boxes. Another alternative would be to leave behind the intervening out-of-flow boxes, or to have out-of-flow boxes impede the running-in of earlier boxes. Implementers and authors are encouraged to contact the CSSWG if they have a preferred behavior, as this one was picked somewhat at random.

<a id="ref-for-first-formatted-line"></a>

<a id="ref-for-run-in-sequence④"></a>

This fixup occurs before the anonymous block and inline box fixup described in [CSS2§9.2](https://www.w3.org/TR/CSS21/visuren.html#box-gen), and affects the determination of the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of the affected elements as if the [run-in sequence](#run-in-sequence) were originally in its final location in the box tree.

<a id="ref-for-first-formatted-line①"></a>

<a id="ref-for-selectordef-first-letter①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As the earliest run-in represents the first text on the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) of its containing block, a [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-element applied to that block element selects the first letter of the run-in, rather than the first letter of its own contents.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This run-in model is slightly different from the one proposed in earlier revisions of [\[CSS2\]](#biblio-css2).

## <a id="glossary"></a> Appendix A: Glossary

The following terms are defined here for convenience:

<a id="root-element"></a>root element  
<a id="ref-for-the-html-element"></a>

<a id="ref-for-document-element"></a>

<a id="ref-for-concept-document-tree"></a>

<a id="ref-for-elements⑦"></a>

The [element](#elements) at the root of the [document tree](https://dom.spec.whatwg.org/#concept-document-tree). In a <a id="ref-for-concept-document-tree①"></a>document tree produced under the DOM, this is the [document element](https://dom.spec.whatwg.org/#document-element); in HTML it is the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> element. [\[DOM\]](#biblio-dom) [\[HTML\]](#biblio-html)

<a id="principal-box"></a>principal box  
<a id="ref-for-principal-box⑨"></a>

<a id="ref-for-box①①"></a>

<a id="ref-for-elements⑧"></a>

When an [element](#elements) generates one or more [boxes](#box), one of them is the [principal box](#principal-box), which contains its descendant boxes and generated content, and is also the box involved in any positioning scheme.

<a id="ref-for-valdef-display-list-item④"></a>

<a id="ref-for-valdef-display-table③"></a>

<a id="ref-for-principal-box①⓪"></a>

<a id="ref-for-table-wrapper-box④"></a>

<a id="ref-for-table-grid-box④"></a>

Some elements may generate additional boxes in addition to the principal box (such as [list-item](#valdef-display-list-item) elements, which generate an additional marker box, or [table](#valdef-display-table) elements, which generate a [principal](#principal-box) [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box) and an additional [table grid box](https://www.w3.org/TR/css-tables-3/#table-grid-box)). These additional boxes are placed with respect to the principal box.

<a id="inline-level"></a>inline-level  
<a id="ref-for-css-text-sequence⑨"></a>

Content that participates in inline layout. Specifically, inline-level boxes and [text sequences](#css-text-sequence).

<a id="block-level"></a>block-level  
<a id="ref-for-block-layout"></a>

Content that participates in [block layout](#block-layout). Specifically, block-level boxes.

<a id="inline-box"></a>inline box  
<a id="ref-for-valdef-display-flow⑦"></a>

<a id="ref-for-inner-display-type①④"></a>

<a id="ref-for-inline-level⑤"></a>

A non-replaced [inline-level](#inline-level) box whose [inner display type](#inner-display-type) is [flow](#valdef-display-flow). The contents of an inline box participate in the same inline formatting context as the inline box itself.

<a id="inline"></a>inline  
<a id="ref-for-inline-level⑥"></a>

<a id="ref-for-inline-level-box②"></a>

<a id="ref-for-inline-box⑥"></a>

Used as a shorthand for [inline box](#inline-box) or [inline-level box](#inline-level-box) where unambiguous, or as an adjective meaning [inline-level](#inline-level). The latter usage is deprecated.

<a id="atomic-inline"></a>atomic inline  
<a id="ref-for-ruby-container④"></a>

<a id="ref-for-inline-box⑦"></a>

<a id="ref-for-valdef-display-inline-table①"></a>

<a id="ref-for-valdef-display-inline-block①"></a>

An inline-level box that is replaced (such as an image) or that establishes a new formatting context (such as an [inline-block](#valdef-display-inline-block) or [inline-table](#valdef-display-inline-table)) and cannot split across lines (as [inline boxes](#inline-box) and [ruby containers](https://www.w3.org/TR/css-ruby-1/#ruby-container) can).

<a id="ref-for-inner-display-type①⑤"></a>

<a id="ref-for-valdef-display-flow⑧"></a>

Any inline-level box whose [inner display type](#inner-display-type) is not [flow](#valdef-display-flow) establishes a new formatting context of the specified <a id="ref-for-inner-display-type①⑥"></a>inner display type.

<a id="block-container"></a>block container  
<a id="ref-for-block-formatting-context⑦"></a>

<a id="ref-for-inline-formatting-context①"></a>

A block container either contains only inline-level boxes participating in an [inline formatting context](#inline-formatting-context), or contains only block-level boxes participating in a [block formatting context](#block-formatting-context) (possibly generating anonymous block boxes to ensure this constraint, as defined in [CSS2§9.2.1.1](https://www.w3.org/TR/CSS2/visuren.html#anonymous-block-level)).

<a id="ref-for-inline-formatting-context②"></a>

<a id="ref-for-root-inline-box"></a>

<a id="ref-for-root-inline-box①"></a>

A block container that contains only inline-level content establishes a new [inline formatting context](#inline-formatting-context). The element then also generates a [root inline box](https://www.w3.org/TR/css-inline-3/#root-inline-box) which wraps all of its inline content. <strong data-conversion-semantic="note">Note:</strong> Note, this [root inline box](https://www.w3.org/TR/css-inline-3/#root-inline-box) concept effectively replaces the "anonymous inline element" concept introduced in [CSS2§9.2.2.1](https://www.w3.org/TR/CSS2/visuren.html#anonymous).

<a id="ref-for-block-formatting-context⑧"></a>

<a id="ref-for-propdef-overflow①"></a>

<a id="ref-for-propdef-align-content"></a>

A block container establishes a new [block formatting context](#block-formatting-context) if its parent formatting context is <em>not</em> a <a id="ref-for-block-formatting-context⑨"></a>block formatting context; otherwise, when participating in a <a id="ref-for-block-formatting-context①⓪"></a>block formatting context itself, it either establishes a new <a id="ref-for-block-formatting-context①①"></a>block formatting context for its contents or continues the one in which it participates, as determined by the constraints of other properties (such as [overflow](https://www.w3.org/TR/CSS2/visufx.html#propdef-overflow) or [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content)).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A block container box can both establish a block formatting context <em>and</em> an inline formatting context simultaneously.

<a id="block-box"></a>block box  
<a id="ref-for-block-container⑦"></a>

<a id="ref-for-block-level-box"></a>

A [block-level box](#block-level-box) that is also a [block container](#block-container).

<a id="ref-for-block-container⑧"></a>

<a id="ref-for-block-level⑤"></a>

<a id="ref-for-inline-block①"></a>

<a id="ref-for-propdef-display③⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Not all [block container](#block-container) boxes are [block-level](#block-level) boxes: non-replaced [inline blocks](#inline-block) and non-replaced table cells, for example, are block containers but not block-level boxes. Similarly, not all block-level boxes are block containers: block-level replaced elements ([display: block](#propdef-display)) and flex containers (<a id="ref-for-propdef-display③⑦"></a>display: flex), for example, are not block containers.

<a id="block"></a>block  
<a id="ref-for-block-container⑨"></a>

<a id="ref-for-block-level-box①"></a>

<a id="ref-for-block-box⑥"></a>

Used as a shorthand for [block box](#block-box), [block-level box](#block-level-box), or [block container box](#block-container), where unambiguous.

<a id="replaced-element"></a>replaced element  
<a id="ref-for-attr-img-src"></a>

<a id="ref-for-the-img-element"></a>

An element whose content is outside the scope of the CSS formatting model, such as an image or embedded document. For example, the content of the HTML <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element is often replaced by the image that its <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#attr-img-src">src</a></code> attribute designates.

<a id="ref-for-natural-dimensions"></a>

Replaced elements often have [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions). For example, a bitmap image has a natural width and a natural height specified in absolute units (from which the natural ratio can obviously be determined). On the other hand, other objects may not have any natural dimensions (for example, a blank HTML document). See [\[css-images-3\]](#biblio-css-images-3).

<a id="ref-for-replaced-element⑥"></a>

<a id="ref-for-natural-dimensions①"></a>

User agents may consider a [replaced element](#replaced-element) to not have any [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions) if it is believed that those dimensions could leak sensitive information to a third party. For example, if an HTML document changed natural size depending on the user’s bank balance, then the UA might want to act as if that resource had no <a id="ref-for-natural-dimensions②"></a>natural dimensions.

<a id="ref-for-natural-dimensions③"></a>

<a id="ref-for-independent-formatting-context①"></a>

The content of replaced elements is not considered in the CSS formatting model; however, their [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions) are used in various layout calculations. Replaced elements always establish an [independent formatting context](#independent-formatting-context).

<a id="ref-for-replaced-element⑦"></a>

A <a id="non-replaced"></a>non-replaced element is one that is not [replaced](#replaced-element), i.e. whose rendering is dictated by the CSS model.

<a id="containing-block"></a>containing block  
<a id="ref-for-overflow"></a>

<a id="ref-for-box①③"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-box①②"></a>

A rectangle that forms the basis of sizing and positioning for the [boxes](#box) associated with it. Notably, a [containing block](#containing-block) is <em>not a <a href="#box">box</a></em> (it is a rectangle), however it is often derived from the dimensions of a <a id="ref-for-box①④"></a>box. Each <a id="ref-for-box①⑤"></a>box is given a position with respect to its <a id="ref-for-containing-block②"></a>containing block, but it is not confined by this <a id="ref-for-containing-block③"></a>containing block; it can [overflow](https://www.w3.org/TR/css-overflow-3/#overflow). The phrase “a box’s containing block” means “the <a id="ref-for-containing-block④"></a>containing block in which the box lives,” not the one it generates.

<a id="ref-for-box-box-edge"></a>

<a id="ref-for-box①⑥"></a>

<a id="ref-for-containing-block⑤"></a>

<a id="ref-for-initial-containing-block①"></a>

In general, the [edges](https://www.w3.org/TR/css-box-4/#box-box-edge) of a [box](#box) act as the [containing block](#containing-block) for descendant boxes; we say that a box “establishes” the <a id="ref-for-containing-block⑥"></a>containing block for its descendants. If properties of a <a id="ref-for-containing-block⑦"></a>containing block are referenced, they reference the values on the <a id="ref-for-box①⑦"></a>box that generated the <a id="ref-for-containing-block⑧"></a>containing block. (For the [initial containing block](#initial-containing-block), values are taken from the root element unless otherwise specified.)

See [\[CSS2\]](#biblio-css2) [Section 9.1.2](https://www.w3.org/TR/CSS2/visuren.html#containing-block) and [Section 10.1](https://www.w3.org/TR/CSS2/visudet.html#containing-block-details) and [CSS Positioned Layout 3 § 2.1 Containing Blocks of Positioned Boxes](https://www.w3.org/TR/css-position-3/#def-cb) for details.

<a id="containing-block-chain"></a>containing block chain  
<a id="ref-for-initial-containing-block②"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-block"></a>

<a id="ref-for-in-flow①"></a>

<a id="ref-for-block-container①⓪"></a>

<a id="ref-for-inline-box⑧"></a>

<a id="ref-for-containing-block⑨"></a>

A sequence of successive [containing blocks](#containing-block) that form an ancestor-descendant chain through the <a id="ref-for-containing-block①⓪"></a>containing block relation. For example, an [inline box](#inline-box)’s containing block is the content box of its closest [block container](#block-container) ancestor; if that block container is an [in-flow](#in-flow) [block](#block), then <em>its</em> containing block is formed by its parent <a id="ref-for-block-container①①"></a>block container; if that grandparent <a id="ref-for-block-container①②"></a>block container is [absolutely positioned](https://www.w3.org/TR/css-position-3/#absolute-position), then <em>its</em> containing block is the padding edges of its closest <em>positioned</em> ancestor (not necessarily its parent), and so on up to the [initial containing block](#initial-containing-block).

<a id="initial-containing-block"></a>initial containing block  
<a id="ref-for-paged-media"></a>

<a id="ref-for-small-viewport-size"></a>

<a id="ref-for-continuous-media"></a>

<a id="ref-for-principal-writing-mode"></a>

<a id="ref-for-block-formatting-context①②"></a>

<a id="ref-for-initial-containing-block③"></a>

<a id="ref-for-root-element①"></a>

<a id="ref-for-containing-block①①"></a>

The [containing block](#containing-block) of the [root element](#root-element). The [initial containing block](#initial-containing-block) establishes a [block formatting context](#block-formatting-context), and takes the [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode) of the document (see [CSS Writing Modes 4 § 8.1 Propagation to the Initial Containing Block](https://www.w3.org/TR/css-writing-modes-4/#icb)). In [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media), it has the dimensions of the [small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) and is anchored at the canvas origin. In [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), see [\[CSS-PAGE-3\]](#biblio-css-page-3) for its position and dimensions.

<a id="formatting-context"></a>formatting context  
<a id="ref-for-ruby-base-container-box"></a>

<a id="ref-for-ruby-formatting-context②"></a>

<a id="ref-for-ruby-container⑤"></a>

<a id="ref-for-inline-formatting-context③"></a>

<a id="ref-for-block-formatting-context①③"></a>

<a id="ref-for-flex-layout"></a>

<a id="ref-for-flex-formatting-context①"></a>

<a id="ref-for-formatting-context⑤"></a>

A [formatting context](#formatting-context) is the environment into which a set of related boxes are laid out. Different <a id="ref-for-formatting-context⑥"></a>formatting contexts lay out their boxes according to different rules. For example, a [flex formatting context](https://www.w3.org/TR/css-flexbox-1/#flex-formatting-context) lays out boxes according to the [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) rules [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1), whereas a [block formatting context](#block-formatting-context) lays out boxes according to the block-and-inline layout rules [\[CSS2\]](#biblio-css2). Additionally, some types of <a id="ref-for-formatting-context⑦"></a>formatting contexts interleave and co-exist: for example, an [inline formatting context](#inline-formatting-context) exists within and interacts with the <a id="ref-for-block-formatting-context①④"></a>block formatting context of the element that establishes it, and a [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container) overlays a [ruby formatting context](https://www.w3.org/TR/css-ruby-1/#ruby-formatting-context) over the <a id="ref-for-inline-formatting-context④"></a>inline formatting context in which its [ruby base container](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) participates.

<a id="ref-for-independent-formatting-context②"></a>

<a id="ref-for-formatting-context⑧"></a>

<a id="ref-for-inner-display-type①⑦"></a>

<a id="ref-for-grid-container⑤"></a>

<a id="ref-for-grid-formatting-context②"></a>

<a id="ref-for-ruby-container⑥"></a>

<a id="ref-for-ruby-formatting-context③"></a>

<a id="ref-for-block-container①③"></a>

<a id="ref-for-block-formatting-context①⑤"></a>

<a id="ref-for-inline-formatting-context⑤"></a>

<a id="ref-for-propdef-display③⑧"></a>

A box either establishes a new [independent formatting context](#independent-formatting-context) or continues the [formatting context](#formatting-context) of its containing block. In some cases, it might additionally establish a new (non-independent) co-existing formatting context. Unless otherwise specified, however, establishing a new <a id="ref-for-formatting-context⑨"></a>formatting context creates an <a id="ref-for-independent-formatting-context③"></a>independent formatting context. The type of formatting context established by the box is determined by its [inner display type](#inner-display-type). E.g. a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) establishes a new [grid formatting context](https://www.w3.org/TR/css-grid-2/#grid-formatting-context), a [ruby container](https://www.w3.org/TR/css-ruby-1/#ruby-container) establishes a new [ruby formatting context](https://www.w3.org/TR/css-ruby-1/#ruby-formatting-context), and a [block container](#block-container) can establish a new [block formatting context](#block-formatting-context) and/or a new [inline formatting context](#inline-formatting-context). See the [display](#propdef-display) property.

<a id="independent-formatting-context"></a>independent formatting context  
<a id="ref-for-formatting-context①⓪"></a>

When a box establishes an independent formatting context (whether that [formatting context](#formatting-context) is of the same type as its parent or not), it essentially creates a new, independent layout environment: except through the sizing of the box itself, the layout of its descendants is (generally) not affected by the rules and contents of the <a id="ref-for-formatting-context①①"></a>formatting context outside the box, and vice versa.

<a id="ref-for-block-formatting-context①⑥"></a>

<a id="ref-for-formatting-context①②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7e2994e7"></a> For example, in a [block formatting context](#block-formatting-context), floated boxes affect the layout of surrounding boxes. But their effects do not escape their [formatting context](#formatting-context): the box establishing their <a id="ref-for-formatting-context①③"></a>formatting context grows to fully contain them, and floats from outside that box are not allowed to protrude into and affect the contents inside the box.

<a id="ref-for-formatting-context①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bb8209a6"></a> As another example, margins do not collapse across [formatting context](#formatting-context) boundaries.

<a id="ref-for-independent-formatting-context④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Exclusions are able to affect content across [independent formatting context](#independent-formatting-context) boundaries. (At time of writing, they are the only layout feature that can.) [\[CSS3-EXCLUSIONS\]](#biblio-css3-exclusions)

<a id="ref-for-out-of-flow①"></a>

<a id="ref-for-blockify⑦"></a>

<a id="ref-for-establish-an-independent-formatting-context"></a>

<a id="ref-for-propdef-contain"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-subgrid"></a>

<a id="ref-for-grid-container⑥"></a>

Certain properties can force a box to <a id="establish-an-independent-formatting-context"></a> establish an independent formatting context in cases where it wouldn’t ordinarily. For example, making a box [out-of-flow](#out-of-flow) causes it to [blockify](#blockify) as well as to [establish an independent formatting context](#establish-an-independent-formatting-context). As another example, certain values of the [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain) property can cause a box to <a id="ref-for-establish-an-independent-formatting-context①"></a>establish an independent formatting context. Turning a block into a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) will cause it to <a id="ref-for-establish-an-independent-formatting-context②"></a>establish an independent formatting context; however turning a [subgrid](https://www.w3.org/TR/css-grid-2/#subgrid) into a <a id="ref-for-scroll-container①"></a>scroll container will not—​it continues to act as a subgrid, with its contents participating in the layout of its parent [grid container](https://www.w3.org/TR/css-grid-2/#grid-container).

<a id="ref-for-block-box⑦"></a>

<a id="ref-for-establish-an-independent-formatting-context③"></a>

<a id="ref-for-block-formatting-context①⑦"></a>

<a id="ref-for-independent-formatting-context⑤"></a>

<a id="ref-for-flex-container⑤"></a>

<a id="ref-for-inline-box⑨"></a>

A [block box](#block-box) that [establishes an independent formatting context](#establish-an-independent-formatting-context) establishes a new [block formatting context](#block-formatting-context) for its contents. In most other cases, forcing a box to <a id="ref-for-establish-an-independent-formatting-context④"></a>establish an independent formatting context is a no-op—​either the box already establishes an [independent formatting context](#independent-formatting-context) (e.g. [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container)), or it’s not possible to establish a totally independent new formatting context on that type of box (e.g. non-replaced [inline boxes](#inline-box)).

<a id="block-formatting-context"></a>block formatting context  
<a id="inline-formatting-context"></a>inline formatting context  
<a id="ref-for-inline-formatting-context⑥"></a>

<a id="ref-for-block-formatting-context①⑧"></a>

[Block](#block-formatting-context) and [inline formatting contexts](#inline-formatting-context) are defined in [CSS 2.1 Section 9.4](https://www.w3.org/TR/CSS2/visuren.html#normal-flow). <a id="ref-for-inline-formatting-context⑦"></a>Inline formatting contexts exist within (are part of their containing) <a id="ref-for-block-formatting-context①⑨"></a>block formatting contexts; for example, line boxes belonging to the <a id="ref-for-inline-formatting-context⑧"></a>inline formatting context interact with floats belonging to the <a id="ref-for-block-formatting-context②⓪"></a>block formatting context.

<a id="block-layout"></a>block layout  
<a id="ref-for-block-formatting-context②①"></a>

<a id="ref-for-block-level-box②"></a>

The layout of [block-level boxes](#block-level-box), performed within a [block formatting context](#block-formatting-context).

<a id="block-formatting-context-root"></a>block formatting context root  
<a id="ref-for-block-formatting-context②②"></a>

<a id="ref-for-block-container①④"></a>

A [block container](#block-container) that establishes a new [block formatting context](#block-formatting-context).

<a id="bfc"></a>BFC  
<a id="ref-for-block-formatting-context-root"></a>

<a id="ref-for-block-formatting-context②③"></a>

Abbreviation for [block formatting context](#block-formatting-context) or [block formatting context root](#block-formatting-context-root). Has various informal definitions referring to boxes which contain internal floats, exclude external floats, and suppress margin collapsing, and may therefore refer specifically to one of:

- <a id="ref-for-block-container①⑤"></a>

  <a id="ref-for-block-formatting-context②④"></a>

  a [block container](#block-container) that establishes a new [block formatting context](#block-formatting-context) for its contents

- <a id="ref-for-block-box⑧"></a>

  <a id="ref-for-block-level⑥"></a>

  <a id="ref-for-block-formatting-context②⑤"></a>

  a [block box](#block-box) (i.e. a [block-level](#block-level) block container) that establishes a [block formatting context](#block-formatting-context) for its contents (as distinguished from a block box which does not)

- <a id="ref-for-formatting-context①⑤"></a>

  (very loosely) any block-level box that establishes a new [formatting context](#formatting-context) (other than an inline formatting context)

<a id="out-of-flow"></a>out-of-flow  
<a id="in-flow"></a>in-flow  
<a id="ref-for-in-flow②"></a>

<a id="ref-for-propdef-position①"></a>

<a id="ref-for-absolute-position①"></a>

<a id="ref-for-propdef-float①"></a>

<a id="ref-for-out-of-flow②"></a>

A box is [out-of-flow](#out-of-flow) if it is extracted from its expected position and interaction with surrounding content and laid out using a different paradigm outside the normal flow of content in its parent formatting context. This occurs if the box is floated (via [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float)) or [absolutely positioned](https://www.w3.org/TR/css-position-3/#absolute-position) (via [position](https://www.w3.org/TR/css-position-3/#propdef-position)). A box is [in-flow](#in-flow) if it is not <a id="ref-for-out-of-flow③"></a>out-of-flow.

<a id="ref-for-formatting-context①⑥"></a>

<a id="ref-for-propdef-float②"></a>

<a id="ref-for-out-of-flow④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some [formatting contexts](#formatting-context) inhibit floating, so that an element with [float: left](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) is not necessarily [out-of-flow](#out-of-flow).

<a id="document-order"></a>document order  
The order in which boxes or content occurs in the document (which can be different from the order in which it appears for rendering). For the purpose of determining the relative order of pseudo-elements, the box-tree order is used, see [CSS Pseudo-Elements 4 § 4 Tree-Abiding Pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#treelike).

See [\[CSS2\]](#biblio-css2) [Chapter 9](https://www.w3.org/TR/CSS2/visuren.html) for a fuller definition of these terms.

<a id="ref-for-propdef-display③⑨"></a>

## <a id="unbox"></a> Appendix B: Effects of [display: contents](#propdef-display) on Unusual Elements

<em>This section is (currently) non-normative.</em>

<a id="ref-for-the-img-element①"></a>

<a id="ref-for-the-input-element"></a>

Some elements aren’t rendered purely by CSS box concepts; for example, replaced elements (such as <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code>), many form controls (such as <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>), and SVG elements.

<a id="ref-for-propdef-display④⓪"></a>

This appendix defines how they interact with [display: contents](#propdef-display).

### <a id="unbox-html"></a>HTML Elements

<a id="ref-for-the-br-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-br-element">br</a></code>

<a id="ref-for-the-wbr-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-wbr-element">wbr</a></code>

<a id="ref-for-the-meter-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-meter-element">meter</a></code>

<a id="ref-for-the-progress-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-progress-element">progress</a></code>

<a id="ref-for-canvas"></a>

<code><a href="https://html.spec.whatwg.org/multipage/canvas.html#canvas">canvas</a></code>

<a id="ref-for-the-embed-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-embed-element">embed</a></code>

<a id="ref-for-the-object-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element">object</a></code>

<a id="ref-for-audio"></a>

<code><a href="https://html.spec.whatwg.org/multipage/media.html#audio">audio</a></code>

<a id="ref-for-the-iframe-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>

<a id="ref-for-the-img-element②"></a>

<code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code>

<a id="ref-for-video"></a>

<code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code>

<a id="ref-for-frame"></a>

<code><a href="https://html.spec.whatwg.org/multipage/obsolete.html#frame">frame</a></code>

<a id="ref-for-frameset"></a>

<code><a href="https://html.spec.whatwg.org/multipage/obsolete.html#frameset">frameset</a></code>

<a id="ref-for-the-input-element①"></a>

<code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>

<a id="ref-for-the-textarea-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code>

<a id="ref-for-the-select-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code>

<a id="ref-for-propdef-display④①"></a>

[display: contents](#propdef-display) computes to <a id="ref-for-propdef-display④②"></a>display: none.

<a id="ref-for-the-legend-element①"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-legend-element">legend</a></code>

<a id="ref-for-the-legend-element②"></a>

<a id="ref-for-propdef-display④③"></a>

<a id="ref-for-rendered-legend"></a>

Per HTML, a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-legend-element">legend</a></code> with [display: contents](#propdef-display) is not a [rendered legend](https://html.spec.whatwg.org/multipage/rendering.html#rendered-legend), so it does not have magical display behavior. (Thus, it reacts to <a id="ref-for-propdef-display④④"></a>display: contents normally.)

<a id="ref-for-the-button-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code>

<a id="ref-for-the-details-element"></a>

<code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code>

<a id="ref-for-the-fieldset-element①"></a>

<code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-fieldset-element">fieldset</a></code>

<a id="ref-for-propdef-display④⑤"></a>

<a id="ref-for-principal-box①①"></a>

These elements don’t have any special behavior; [display: contents](#propdef-display) simply removes their [principal box](#principal-box), and their contents render as normal.

any other HTML element

<a id="ref-for-propdef-display④⑥"></a>

Behaves as normal for [display: contents](#propdef-display).

### <a id="unbox-svg"></a>SVG Elements

<a id="ref-for-elementdef-svg①"></a>

<a id="ref-for-elementdef-svg"></a>

An <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code> element that has CSS box layout (this includes all <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code> whose parent is an HTML element, as well as document root elements)

<a id="ref-for-propdef-display④⑦"></a>

[display: contents](#propdef-display) computes to <a id="ref-for-propdef-display④⑧"></a>display: none.

<a id="ref-for-TermRenderableElement"></a>

<a id="ref-for-container-element"></a>

All other SVG [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) that are also [renderable elements](https://www.w3.org/TR/SVG2/render.html#TermRenderableElement)

<a id="ref-for-TermTextContentChildElement"></a>

SVG [text content child elements](https://w3c.github.io/svgwg/svg2-draft/text.html#TermTextContentChildElement)

<a id="ref-for-elementdef-use"></a>

<code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-use">use</a></code>

<a id="ref-for-propdef-display④⑨"></a>

<a id="ref-for-elementdef-use①"></a>

[display: contents](#propdef-display) strips the element from the formatting tree, and hoists its contents up to display in its place. These contents include the shadow-DOM content for <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-use">use</a></code>.

any other SVG elements

<a id="ref-for-propdef-display⑤⓪"></a>

[display: contents](#propdef-display) computes to <a id="ref-for-propdef-display⑤①"></a>display: none.

<a id="ref-for-propdef-display⑤②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The intention here is that the [display: none](#propdef-display) behavior applies whenever the "rendering context" inside the element is different than the context outside of it. If the element’s child elements would not be valid children of the element’s parent, you cannot simply hoist them up the formatting tree.
>
> <a id="ref-for-elementdef-text"></a>
>
> <a id="ref-for-elementdef-text①"></a>
>
> <a id="ref-for-propdef-display⑤③"></a>
>
> <a id="ref-for-elementdef-text②"></a>
>
> <a id="ref-for-elementdef-tspan"></a>
>
> <a id="ref-for-elementdef-textPath"></a>
>
> For example, text content and text formatting elements in SVG require a <code><a href="https://www.w3.org/TR/SVG2/text.html#elementdef-text">text</a></code> element context; if you remove a <code><a href="https://www.w3.org/TR/SVG2/text.html#elementdef-text">text</a></code>, its child text content and elements are no longer valid. For that reason, [display: contents](#propdef-display) on <code><a href="https://www.w3.org/TR/SVG2/text.html#elementdef-text">text</a></code> prevents the entire text element from being rendered. In contrast, any valid content inside a <code><a href="https://www.w3.org/TR/SVG2/text.html#elementdef-tspan">tspan</a></code> or <code><a href="https://www.w3.org/TR/SVG2/text.html#elementdef-textPath">textPath</a></code> is also valid content inside the parent text formatting context, so the hoisting behavior applies for these elements.
>
> <a id="ref-for-TermNonRenderedElement"></a>
>
> <a id="ref-for-elementdef-pattern"></a>
>
> <a id="ref-for-elementdef-symbol"></a>
>
> <a id="ref-for-TermRenderedElement"></a>
>
> <a id="ref-for-elementdef-svg②"></a>
>
> <a id="ref-for-propdef-display⑤④"></a>
>
> Similarly, if hoisting would convert the children from [non-rendered elements](https://www.w3.org/TR/SVG2/render.html#TermNonRenderedElement) (e.g., a shape inside a <code><a href="https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern">pattern</a></code> or <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-symbol">symbol</a></code>) to [rendered elements](https://www.w3.org/TR/SVG2/render.html#TermRenderedElement) (e.g., a shape that is a direct child of the <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code>), that is an invalid change of rendering context. Never-rendered container elements therefore cannot be "un-boxed" by [display: contents](#propdef-display).

<a id="ref-for-TermPresentationAttribute"></a>

When an element is stripped from the formatting tree, then any SVG attributes on that element that control layout and visual formatting are ignored when rendering the contents. However, SVG [presentation attributes](https://www.w3.org/TR/SVG2/styling.html#TermPresentationAttribute)—​which map to CSS properties—​continue to affect value processing and inheritance [\[CSS-CASCADE-3\]](#biblio-css-cascade-3); thus such attributes can affect the layout and visual formatting of the element’s descendants by influencing the values of such properties on those descendants.

### <a id="unbox-mathml"></a>MathML Elements

<a id="ref-for-propdef-display⑤⑤"></a>

For all MathML elements, [display: contents](#propdef-display) computes to <a id="ref-for-propdef-display⑤⑥"></a>display: none.

## <a id="box-guidelines"></a> Appendix C: Box Construction Guidelines for Spec Authors

<em>This section is non-normative guidance for specification authors.</em>

- <a id="ref-for-blockify⑧"></a>

  <a id="ref-for-inlinify⑦"></a>

  A box cannot be [blockified](#blockify) and [inlinified](#inlinify) at the same time; if such a thing would occur, define which wins over the other.

- <a id="ref-for-blockify⑨"></a>

  <a id="ref-for-principal-box①②"></a>

  Non-principal non-anonymous boxes can’t be [blockified](#blockify): blockification affects the element’s computed values and thus determines the type of its [principal box](#principal-box).

- <a id="ref-for-blockify①⓪"></a>

  <a id="ref-for-inline-level⑦"></a>

  <a id="ref-for-anonymous③"></a>

  <a id="ref-for-block-container①⑥"></a>

  Boxes which [blockify](#blockify) their contents can’t directly contain [inline-level](#inline-level) content; any boxes or text sequences generated within such an element must be <a id="ref-for-blockify①①"></a>blockified or wrapped in an [anonymous](#anonymous) [block container](#block-container).

- <a id="ref-for-inlinify⑧"></a>

  <a id="ref-for-block-level⑦"></a>

  <a id="ref-for-inline-level⑧"></a>

  Boxes which [inlinify](#inlinify) their contents can’t directly contain [block-level](#block-level) boxes; any boxes generated within such an element must be [inline-level](#inline-level).

- <a id="ref-for-independent-formatting-context⑥"></a>

  <a id="ref-for-establish-an-independent-formatting-context⑤"></a>

  Boxes that fundamentally cannot establish an [independent formatting context](#independent-formatting-context) (such as non-replaced inlines) must not be asked to [establish an independent formatting context](#establish-an-independent-formatting-context). Blockify them first, or otherwise change their box type to one that can establish an <a id="ref-for-independent-formatting-context⑦"></a>independent formatting context.

## <a id="acknowledgments"></a> Acknowledgments

We would like to thank the many people who have attempted to separate out the disparate details of box generation over the years, most particularly Bert Bos, whose last attempt with display-model and display-role didn’t get anywhere, but primed us for the current spec; Anton Prowse, whose relentless assault on CSS2.1 Chapter 9 forced some order out of the chaos; and Oriol Brufau, who teased apart dozens of fine distinctions and errors in this spec. Honorable mentions also go to David Baron, Mats Palmgren, Ilya Streltsyn, and Boris Zbarsky for their feedback.

## <a id="changes"></a> Changes

### <a id="changes-2023"></a> Changes Since 2023 Candidate Recommendation Snapshot

Changes since the [30 March 2023 Candidate Recommendation Snapshot](https://www.w3.org/TR/2023/CR-css-display-3-20230330/) include:

- <a id="ref-for-small-viewport-size①"></a>

  <a id="ref-for-writing-mode"></a>

  <a id="ref-for-continuous-media①"></a>

  <a id="ref-for-initial-containing-block④"></a>

  <a id="change-icb-definition"></a> Inlined the CSS2.1 definition of the [initial containing block](#initial-containing-block) for [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media), cross-referenced its [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) propagation as defined in [\[CSS-WRITING-MODES-4\]](#biblio-css-writing-modes-4), and clarified that it refers to the [small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size). ([Issue 6453](https://github.com/w3c/csswg-drafts/issues/6453))

### <a id="changes-2020"></a> Changes Since 2020 Candidate Recommendation

A [Disposition of Comments](https://drafts.csswg.org/css-display-3/issues-cr-2020) since the [19 May 2020 Candidate Recommendation](https://www.w3.org/TR/2020/CR-css-display-3-20200519/) is available.

Changes since the [19 May 2020 Candidate Recommendation](https://www.w3.org/TR/2020/CR-css-display-3-20200519/) include:

- <a id="ref-for-css-text-sequence①⓪"></a>

  <a id="change-text-run-sequence"></a> Renamed “text run” to “[text sequence](#css-text-sequence)”. ([Issue 7768](https://github.com/w3c/csswg-drafts/issues/7768))

- <a id="ref-for-initial-containing-block⑤"></a>

  <a id="ref-for-block-level⑧"></a>

  <a id="ref-for-root-element②"></a>

  <a id="change-root-layout"></a> Defined [root element](#root-element) and clarified its layout as a [block-level](#block-level) in [§ 2.8 The Root Element’s Principal Box](#root) and the definition of [initial containing block](#initial-containing-block). ([PR 8095](https://github.com/w3c/csswg-drafts/pull/8095), [Issue 7207](https://github.com/w3c/csswg-drafts/issues/7207), [Issue 6480](https://github.com/w3c/csswg-drafts/issues/6480), [Issue 7786](https://github.com/w3c/csswg-drafts/issues/7786))

- <a id="ref-for-grid-item⑤"></a>

  <a id="ref-for-propdef-order①⑥"></a>

  <a id="change-move-order"></a> Pulled in the [order](#propdef-order) property definition from [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1), since it also applies to [grid items](https://www.w3.org/TR/css-grid-2/#grid-item). ([Issue 5865](https://github.com/w3c/csswg-drafts/issues/5865))

- <a id="ref-for-propdef-visibility⑤"></a>

  <a id="change-import-visibility"></a> Imported the [visibility](#propdef-visibility) property definition from [\[CSS2\]](#biblio-css2) and updated it to be more thorough and complete. ([Issue 6123](https://github.com/w3c/csswg-drafts/issues/6123))

- <a id="ref-for-propdef-display⑤⑦"></a>

  <a id="ref-for-layout-internal⑤"></a>

  <a id="ref-for-replaced-element⑧"></a>

  <a id="change-internal-replaced"></a> Defined that [replaced elements](#replaced-element) with a [layout-internal](#layout-internal) [display](#propdef-display) are treated as <a id="ref-for-propdef-display⑤⑧"></a>display: inline. ([Issue 6000](https://github.com/w3c/csswg-drafts/issues/6000))

- <a id="ref-for-computed-value③"></a>

  <a id="ref-for-typedef-display-legacy②"></a>

  <a id="change-legacy-computation"></a> Clarified that the [\<display-legacy\>](#typedef-display-legacy) values actually [compute](https://www.w3.org/TR/css-cascade-5/#computed-value) to the same value as their two-keyword equivalents. ([Issue 5575](https://github.com/w3c/csswg-drafts/issues/5575))

  > inline-…  
  > ~~Behaves as~~ <u>Computes to</u> inline ….
  >
  > <a id="ref-for-specified-value①"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > <u>Note: Although these keywords and their equivalents compute to the same value, their [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value) remain distinct.</u>
  >
  > <a id="ref-for-dom-window-getcomputedstyle②"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > <u>Note: The <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> serialization rules will always output these precomposed keywords rather than the equivalent two-keyword pairs due to the [shortest, most backwards-compatible serialization principle](https://www.w3.org/TR/cssom-1/#serializing-css-values).</u>

- <a id="ref-for-computed-value④"></a>

  <a id="ref-for-inlinify⑨"></a>

  <a id="ref-for-blockify①②"></a>

  <a id="change-blockification-computed"></a> Clarified that [blockification](#blockify) and [inlinification](#inlinify) are [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) changes. ([Issue 6251](https://github.com/w3c/csswg-drafts/issues/6251))

  > <a id="ref-for-blockify①③"></a>
  >
  > <a id="ref-for-inlinify①⓪"></a>
  >
  > <a id="ref-for-computed-value⑤"></a>
  >
  > <a id="ref-for-outer-display-type①①"></a>
  >
  > <a id="ref-for-valdef-display-block⑨"></a>
  >
  > <a id="ref-for-valdef-display-inline⑧"></a>
  >
  > Some layout effects require [blockification](#blockify) or [inlinification](#inlinify) of the box type, which sets the box’s <u>[computed](https://www.w3.org/TR/css-cascade-5/#computed-value)</u> [outer display type](#outer-display-type) to [block](#valdef-display-block) or [inline](#valdef-display-inline) (respectively).

- <a id="ref-for-block-layout①"></a>

  <a id="change-block-layout-def"></a> Added definition for [block layout](#block-layout) to glossary, for convenience.

- <a id="change-grid-refs"></a> Updated references to [CSS Grid Layout](https://www.w3.org/TR/css-grid/).

- <a id="change-2020-editorial"></a> Various minor editorial clarifications and cross-linking improvements.

### <a id="changes-2019"></a> Changes Since 2019 Candidate Recommendation

Changes since the [11 July 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-display-3-20190711/) include:

- <a id="ref-for-containing-block①②"></a>

  <a id="change-containing-block"></a> Merged in additional prose from [\[CSS2\]](#biblio-css2) into the definition of [containing block](#containing-block).

- <a id="change-dom-ignore"></a> Clarified that, for the purpose of CSS, DOM nodes other than elements and text are ignored.

  > <u>(Some source documents start from more complex trees, such as the DOM, which can have comment nodes and other types of things. For the purposes of CSS, all of these additional types of nodes are ignored, as if they didn’t exist.)</u>

- <a id="ref-for-absolute-position②"></a>

  <a id="change-abspos-glossary"></a> Improved/moved glossary definitions relating to [absolute positioning](https://www.w3.org/TR/css-position-3/#absolute-position).

Changes since the [28 August 2018 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-display-3-20180828/) include:

- <a id="ref-for-css-parent-box"></a>

  <a id="change-parent"></a> Defined box tree parentage; see [parent box](#css-parent-box).

- <a id="ref-for-absolute-position③"></a>

  <a id="change-abspos"></a> Added definition of [absolutely positioned](https://www.w3.org/TR/css-position-3/#absolute-position) to the glossary; [copied from CSS2](https://www.w3.org/TR/CSS2/visuren.html#absolutely-positioned) for easier referencing.

- <a id="change-bidi-fragment"></a> Added cross-references to various forms of fragmentation in [§ 1 Introduction](#intro).

- <a id="change-table-grid-box"></a> Renamed “table box” to “table grid box” to more easily distinguish from “table wrapper box”.

- <a id="ref-for-the-body-element"></a>

  <a id="change-body-propagation"></a> Added “unless otherwise specified” to root → initial containing block propagation, since there are some regrettable special exceptions for the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element.

### <a id="changes-wd"></a> Changes Prior to Candidate Recommendation Status

A [Disposition of Comments](https://drafts.csswg.org/css-display-3/issues-wd-2017) is available.

Changes since the [20 April 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-display-3-20180420/) include:

- <a id="ref-for-propdef-display⑤⑨"></a>

  Elements with [display: contents](#propdef-display) that behave as <a id="ref-for-propdef-display⑥⓪"></a>display: none now <em>compute to</em> <a id="ref-for-propdef-display⑥①"></a>display: none. ([Issue 2755](https://github.com/w3c/csswg-drafts/issues/2755))

- <a id="ref-for-independent-formatting-context⑧"></a>

  <a id="ref-for-formatting-context①⑦"></a>

  Distinguished “new” [formatting context](#formatting-context) from [independent formatting context](#independent-formatting-context) since certain formatting contexts layer. ([Issue 2597](https://github.com/w3c/csswg-drafts/issues/2597), [Issue 1457](https://github.com/w3c/csswg-drafts/issues/1457))

- <a id="ref-for-valdef-display-flow-root⑧"></a>

  <a id="ref-for-propdef-display⑥②"></a>

  <a id="ref-for-independent-formatting-context⑨"></a>

  <a id="ref-for-block-box⑨"></a>

  Defined that [block boxes](#block-box) that establish an [independent formatting context](#independent-formatting-context) have a used [display](#propdef-display) of [flow-root](#valdef-display-flow-root), to provide an easier point of reference. ([Issue 1550](https://github.com/w3c/csswg-drafts/issues/1550))

- <a id="ref-for-propdef-display⑥③"></a>

  Clarified that [display](#propdef-display) is not animatable (as opposed to discretely animatable). ([Issue 2938](https://github.com/w3c/csswg-drafts/issues/2938))

- Minor editorial fixes.

Changes since the [20 July 2017 Working Draft](https://www.w3.org/TR/2017/WD-css-display-3-20170720/) include:

- <a id="ref-for-blockify①④"></a>

  <a id="ref-for-valdef-display-inline-block②"></a>

  Tightened up rules for the [blockification](#blockify) of [inline-block](#valdef-display-inline-block) / inline flow-root to ensure compatibility with CSS2. ([Issue 1246](https://github.com/w3c/csswg-drafts/issues/1246)) Updated handling of run-in flow-root to match. ([Issue 1715](https://github.com/w3c/csswg-drafts/issues/1715))

- <a id="ref-for-propdef-display⑥④"></a>

  <a id="ref-for-valdef-display-list-item⑤"></a>

  Adjusted grammar of [display](#propdef-display) to list the [list-item](#valdef-display-list-item) keyword last. This affects the expected serialization order. ([Issue 1621](https://github.com/w3c/csswg-drafts/issues/1621))

- <a id="ref-for-block-formatting-context②⑥"></a>

  <a id="ref-for-inline-formatting-context⑨"></a>

  Added better definition of interaction between [block formatting contexts](#block-formatting-context) and [inline formatting contexts](#inline-formatting-context) in block containers that establish inline formatting contexts. ([Issue 1553](https://github.com/w3c/csswg-drafts/issues/1553))

- More clearly defined the way property values are reflected between an element and its boxes (in the case of an element generating multiple boxes). ([Issue 1643](https://github.com/w3c/csswg-drafts/issues/1643))

- Clarified that empty text objects are ignored for CSS rendering. ([Issue 1808](https://github.com/w3c/csswg-drafts/issues/1808))

- <a id="ref-for-propdef-display⑥⑤"></a>

  Clarified that [display](#propdef-display) has no effect on document semantics, since this is a common bug in UAs. ([Issue 2355](https://github.com/w3c/csswg-drafts/issues/2355))

- <a id="ref-for-propdef-display⑥⑥"></a>

  <a id="ref-for-computed-value⑥"></a>

  <a id="ref-for-blockify①⑤"></a>

  <a id="ref-for-inlinify①①"></a>

  Fixed error in definition of [display](#propdef-display)’s [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) (which is definitely not “as specified”, due to [blockification](#blockify) and [inlinification](#inlinify) rules triggered by various properties). ([Issue 1716](https://github.com/w3c/csswg-drafts/issues/1716))

- <a id="ref-for-document-order"></a>

  Added definition for [document order](#document-order).

- <a id="ref-for-propdef-display⑥⑦"></a>

  Added missing SVG elements to [Appendix B](#unbox)’s details on [display: contents](#propdef-display) ([Issue 2118](https://github.com/w3c/csswg-drafts/issues/2118)), clarified effect of SVG attributes ([Issue 2502](https://github.com/w3c/csswg-drafts/issues/2502)), and defined behavior for MathML ([Issue 2167](https://github.com/w3c/csswg-drafts/issues/2167)).

- Added some [guidance](#box-guidelines) to future spec authors of anonymous box construction rules. ([Issue 1643](https://github.com/w3c/csswg-drafts/issues/1643))

- Pushed the section about “becoming a formatting context” back to [CSS Containment](https://www.w3.org/TR/css-contain-1/) where it is used.

- Various minor wording fixes and clarifications.

Changes since the [26 January 2017 Working Draft](https://www.w3.org/TR/2017/WD-css-display-3-20170126/) include:

- Remove inline-list-item value that is equivalent to inline list-item.

- <a id="ref-for-text-nodes⑥"></a>

  <a id="ref-for-element-tree②"></a>

  <a id="ref-for-css-text-sequence①①"></a>

  <a id="ref-for-box-tree⑨"></a>

  <a id="ref-for-propdef-display⑥⑧"></a>

  Added the notion of “[text nodes](#text-nodes)” to the [element tree](#element-tree), and “[text runs](#css-text-sequence)” to the [box tree](#box-tree) to define behavior in the context of [display: contents](#propdef-display). ([Issue 19](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-19), [Issue 32](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-32))

- Defined that the root element is “in flow”. ([Issue 3](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-3))

- <a id="ref-for-selectordef-first-line"></a>

  <a id="ref-for-selectordef-first-letter②"></a>

  <a id="ref-for-valdef-display-run-in⑤"></a>

  Defined interaction of [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line)/[::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) and [run-in](#valdef-display-run-in). ([Issue 5](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-5), [Issue 42](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-42))

- <a id="ref-for-flow-layout⑥"></a>

  Clarified that block/inline/run-in only dictates behavior in [flow layout](#flow-layout); it is ignored in other contexts.

- Run-ins are a type of inline box, not just "like" an inline box.

- Fixed the lack of recursion in of run-in’s box-tree munging. ([Issue 45](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-45))

- <a id="ref-for-propdef-display⑥⑨"></a>

  Added an appendix on how [display: contents](#propdef-display) works on “unusual elements”. ([Issue 8](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-8), [Issue 18](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-18))

- <a id="ref-for-blockify①⑥"></a>

  <a id="ref-for-inlinify①②"></a>

  Fix [blockification](#blockify) and [inlinification](#inlinify) rules, particularly handling of layout-internal types. ([Issue 35](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-35), [Issue 57](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-57))

- Clarified interaction of various box tree fixups. ([Issue 38](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-38), [Issue 48](https://drafts.csswg.org/css-display-3/issues-wd-2017#issue-48))

- Added the definition of “becoming a formatting context”.

- Miscellaneous minor fixes and minor clarifications.

A [Disposition of Comments](https://drafts.csswg.org/css-display-3/issues-wd-2017) is also available.

Changes since the [15 October 2015 Working Draft](https://www.w3.org/TR/2015/WD-css-display-3-20151015/) include:

- Deferred the box-suppress/display-or-not property to the next level of Display, in order to provide time for further discussion of use cases.

- <a id="ref-for-propdef-display⑦⓪"></a>

  Specified the effects of [display: contents](#propdef-display) on unusual elements such as replaced elements and form controls.

- <a id="ref-for-propdef-display⑦①"></a>

  <a id="ref-for-selectordef-after①"></a>

  <a id="ref-for-selectordef-before②"></a>

  Clarified that an element’s [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudo-elements still exist if its own box is not generated due to [display: contents](#propdef-display).

- <a id="ref-for-propdef-display⑦②"></a>

  Clarified that event bubbling is not affected by [display: contents](#propdef-display).

- <a id="ref-for-selectordef-first-letter③"></a>

  Clarified interaction of run-ins with out-of-flow elements and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter).

- <a id="ref-for-inner-display-type①⑧"></a>

  <a id="ref-for-valdef-display-flow-root⑨"></a>

  <a id="ref-for-valdef-display-table-cell②"></a>

  <a id="ref-for-valdef-display-table-caption①"></a>

  Switched [table-caption](#valdef-display-table-caption) and [table-cell](#valdef-display-table-cell) to use [flow-root](#valdef-display-flow-root) as their [inner display type](#inner-display-type), since they always form a formatting context root.

- Closed off remaining issues and added at-risk list.

Changes since the [21 July 2015 Working Draft](https://www.w3.org/TR/2015/WD-css-display-3-20150721/) include:

- <a id="ref-for-out-of-flow⑤"></a>

  <a id="ref-for-in-flow③"></a>

  Added definitions for [in-flow](#in-flow) and [out-of-flow](#out-of-flow) to glossary.

Changes since the [11 September 2014 Working Draft](https://www.w3.org/TR/2014/WD-css-display-3-20140911/) include:

- <a id="ref-for-propdef-display⑦③"></a>

  Removed display-inside, display-outside, and display-extras longhands, in favor of just making [display](#propdef-display) multi-value. (This was done to impose constraints on what can be combined. Future levels of this specification may relax some or all of those restrictions if they become unnecessary or unwanted.)

- <a id="ref-for-propdef-overflow②"></a>

  <a id="ref-for-bfc①"></a>

  <a id="ref-for-display-type③"></a>

  <a id="ref-for-inner-display-type①⑨"></a>

  <a id="ref-for-valdef-display-flow-root①⓪"></a>

  <a id="ref-for-valdef-display-flow⑨"></a>

  Created the [flow](#valdef-display-flow) and [flow-root](#valdef-display-flow-root) [inner display types](#inner-display-type) to better express flow layout [display types](#display-type) and to create an explicit switch for making an element a [BFC](#bfc) root. (This should eliminate the need for hacks like ::after { clear: both; } and [overflow: hidden](https://www.w3.org/TR/CSS2/visufx.html#propdef-overflow) that are intended to accomplish this purpose.)

## <a id="priv"></a> Privacy Considerations

This specification introduces no new privacy considerations.

### <a id="sec"></a> Security Considerations

This specification introduces no new security considerations.

### <a id="security-privacy-self-review"></a> Self-Review Questionnaire

Per the [Self-Review Questionnaire: Security and Privacy: Questions to Consider](https://www.w3.org/TR/security-privacy-questionnaire/#questions)

1.  Does this specification deal with personally-identifiable information?

    No.

2.  Does this specification deal with high-value data?

    No.

3.  Does this specification introduce new state for an origin that persists across browsing sessions?

    No.

4.  Does this specification expose persistent, cross-origin state to the web?

    No.

5.  Does this specification expose any other data to an origin that it doesn’t currently have access to?

    No.

6.  Does this specification enable new script execution/loading mechanisms?

    No.

7.  Does this specification allow an origin access to a user’s location?

    No.

8.  Does this specification allow an origin access to sensors on a user’s device?

    No.

9.  Does this specification allow an origin access to aspects of a user’s local computing environment?

    No.

10. Does this specification allow an origin access to other devices?

    No.

11. Does this specification allow an origin some measure of control over a user agent’s native UI?

    No.

12. Does this specification expose temporary identifiers to the web?

    No.

13. Does this specification distinguish between behavior in first-party and third-party contexts?

    No.

14. How should this specification work in the context of a user agent’s "incognito" mode?

    No differently.

15. Does this specification persist data to a user’s local device?

    No.

16. Does this specification have a "Security Considerations" and "Privacy Considerations" section?

    Yes.

17. Does this specification allow downgrading default security characteristics?

    No.

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

- [anonymous](#anonymous), in § 1
- [anonymous box](#anonymous), in § 1
- [atomic inline](#atomic-inline), in § Unnumbered section
- [atomic inline box](#atomic-inline), in § Unnumbered section
- [BFC](#bfc), in § Unnumbered section
- block
  - [definition of](#block), in § Unnumbered section
  - [value for display, \<display-outside\>](#valdef-display-block), in § 2.1
- [block box](#block-box), in § Unnumbered section
- [block container](#block-container), in § Unnumbered section
- [block container box](#block-container), in § Unnumbered section
- [block formatting context](#block-formatting-context), in § Unnumbered section
- [block formatting context root](#block-formatting-context-root), in § Unnumbered section
- [blockification](#blockify), in § 2.7
- [blockify](#blockify), in § 2.7
- [block layout](#block-layout), in § Unnumbered section
- [block-level](#block-level), in § Unnumbered section
- [block-level box](#block-level-box), in § 2.1
- [block-level content](#block-level), in § Unnumbered section
- [box](#box), in § 1
- [box tree](#box-tree), in § 1
- [collapse](#valdef-visibility-collapse), in § 4
- [collapsed](#collapsed), in § 4
- [containing block](#containing-block), in § Unnumbered section
- [containing block chain](#containing-block-chain), in § Unnumbered section
- [contents](#valdef-display-contents), in § 2.5
- [display](#propdef-display), in § 2
- [\<display-box\>](#typedef-display-box), in § 2
- [\<display-inside\>](#typedef-display-inside), in § 2
- [\<display-internal\>](#typedef-display-internal), in § 2
- [\<display-legacy\>](#typedef-display-legacy), in § 2
- [\<display-listitem\>](#typedef-display-listitem), in § 2
- [\<display-outside\>](#typedef-display-outside), in § 2
- [display type](#display-type), in § 2
- [document order](#document-order), in § Unnumbered section
- [element](#elements), in § 1
- [element tree](#element-tree), in § 1
- [establish an independent formatting context](#establish-an-independent-formatting-context), in § Unnumbered section
- [established an independent formatting context](#establish-an-independent-formatting-context), in § Unnumbered section
- [establishes an independent formatting context](#establish-an-independent-formatting-context), in § Unnumbered section
- [establishing an independent formatting context](#establish-an-independent-formatting-context), in § Unnumbered section
- [flex](#valdef-display-flex), in § 2.2
- [flow](#valdef-display-flow), in § 2.2
- [flow layout](#flow-layout), in § 2.2
- [flow-root](#valdef-display-flow-root), in § 2.2
- [formatting context](#formatting-context), in § Unnumbered section
- [grid](#valdef-display-grid), in § 2.2
- [hidden](#valdef-visibility-hidden), in § 4
- [independent formatting context](#independent-formatting-context), in § Unnumbered section
- [in flow](#in-flow), in § Unnumbered section
- [in-flow](#in-flow), in § Unnumbered section
- [in-flow box](#in-flow), in § Unnumbered section
- [initial containing block](#initial-containing-block), in § Unnumbered section
- inline
  - [definition of](#inline), in § Unnumbered section
  - [value for display, \<display-outside\>](#valdef-display-inline), in § 2.1
- [inline block](#inline-block), in § 2
- [inline-block](#valdef-display-inline-block), in § 2.6
- [inline block box](#inline-block), in § 2
- [inline box](#inline-box), in § Unnumbered section
- [inline-flex](#valdef-display-inline-flex), in § 2.6
- [inline formatting context](#inline-formatting-context), in § Unnumbered section
- [inline-grid](#valdef-display-inline-grid), in § 2.6
- [inline-level](#inline-level), in § Unnumbered section
- [inline-level box](#inline-level-box), in § 2.1
- [inline-level content](#inline-level), in § Unnumbered section
- [inline-table](#valdef-display-inline-table), in § 2.6
- [inlinification](#inlinify), in § 2.7
- [inlinify](#inlinify), in § 2.7
- [inner](#inner-display-type), in § 2
- [inner display type](#inner-display-type), in § 2
- [\<integer\>](#valdef-order-integer), in § 3
- [internal ruby box](#internal-ruby-box), in § 2.4
- [internal ruby element](#internal-ruby-element), in § 2.4
- [internal table box](#internal-table-box), in § 2.4
- [internal table element](#internal-table-element), in § 2.4
- [invisible](#invisible-box), in § 4
- [invisible box](#invisible-box), in § 4
- [layout-internal](#layout-internal), in § 2.4
- [list-item](#valdef-display-list-item), in § 2.3
- [none](#valdef-display-none), in § 2.5
- [non-replaced](#non-replaced), in § Unnumbered section
- [non-replaced element](#non-replaced), in § Unnumbered section
- [order](#propdef-order), in § 3
- [order-modified document order](#order-modified-document-order), in § 3
- [outer](#outer-display-type), in § 2
- [outer display type](#outer-display-type), in § 2
- [out of flow](#out-of-flow), in § Unnumbered section
- [out-of-flow](#out-of-flow), in § Unnumbered section
- [out-of-flow box](#out-of-flow), in § Unnumbered section
- [parent box](#css-parent-box), in § 1
- [principal box](#principal-box), in § Unnumbered section
- [replaced](#replaced-element), in § Unnumbered section
- [replaced element](#replaced-element), in § Unnumbered section
- [root element](#root-element), in § Unnumbered section
- [ruby](#valdef-display-ruby), in § 2.2
- [ruby-base](#valdef-display-ruby-base), in § 2.4
- [ruby-base-container](#valdef-display-ruby-base-container), in § 2.4
- [ruby-text](#valdef-display-ruby-text), in § 2.4
- [ruby-text-container](#valdef-display-ruby-text-container), in § 2.4
- run-in
  - [definition of](#run-in), in § 5
  - [value for display, \<display-outside\>](#valdef-display-run-in), in § 2.1
- [run-in box](#run-in), in § 5
- [run-in sequence](#run-in-sequence), in § 5
- [table](#valdef-display-table), in § 2.2
- [table-caption](#valdef-display-table-caption), in § 2.4
- [table caption box](#table-caption-box), in § 2.4
- [table-cell](#valdef-display-table-cell), in § 2.4
- [table-column](#valdef-display-table-column), in § 2.4
- [table-column-group](#valdef-display-table-column-group), in § 2.4
- [table-footer-group](#valdef-display-table-footer-group), in § 2.4
- [table-header-group](#valdef-display-table-header-group), in § 2.4
- [table-row](#valdef-display-table-row), in § 2.4
- [table-row-group](#valdef-display-table-row-group), in § 2.4
- [text node](#text-nodes), in § 1
- [text sequence](#css-text-sequence), in § 1
- [visibility](#propdef-visibility), in § 4
- [visible](#valdef-visibility-visible), in § 4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fde954ee"></a>align-content
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="e1674793"></a>border
- \[CSS-BOX-4\] defines the following terms:
  - <a id="60669dde"></a>box edge
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="62c772f5"></a>fragment
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="04004305"></a>fragmentation
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="cbeb753c"></a>inheritance
  - <a id="fdef2996"></a>inherited property
  - <a id="d5e08d9c"></a>specified value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="5dfeee7f"></a>contain
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="cc7f0a64"></a>flex container
  - <a id="31765884"></a>flex formatting context
  - <a id="9f6d5ab0"></a>flex item
  - <a id="07e702cf"></a>flex layout
- \[CSS-GRID-2\] defines the following terms:
  - <a id="df72a52c"></a>grid container
  - <a id="6f8683f4"></a>grid formatting context
  - <a id="ba30fc9a"></a>grid item
  - <a id="bbc693a1"></a>subgrid
  - <a id="e25926df"></a>subgrid (for grid-template-rows)
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="487e1aa9"></a>natural dimension
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="8af3edff"></a>root inline box
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="24c40f7e"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="4cf3c57e"></a>absolute position
  - <a id="d9b71dda"></a>absolutely position
  - <a id="b8c34db8"></a>position
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="b6b63ba4"></a>::marker
  - <a id="99a0ef70"></a>first formatted line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="153743a1"></a>ruby base container
  - <a id="be5da72e"></a>ruby container
  - <a id="470cd324"></a>ruby formatting context
- \[CSS-SPEECH-1\] defines the following terms:
  - <a id="6cb2de13"></a>always
  - <a id="decb9188"></a>speak
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="28f05b35"></a>table grid box
  - <a id="1b178ec1"></a>table wrapper box
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="ba466582"></a>white space
- \[CSS-UI-4\] defines the following terms:
  - <a id="e2ba7ac4"></a>pointer-events
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="d73c993d"></a>\<integer\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="a959f189"></a>small viewport size
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="953ffdad"></a>principal writing mode
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="da486c10"></a>float
  - <a id="493618d2"></a>list-style
  - <a id="11b5911d"></a>overflow
- \[CSSOM\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
- \[DOM\] defines the following terms:
  - <a id="2f0ba72c"></a>document element
  - <a id="7bf10631"></a>document tree
- \[HTML\] defines the following terms:
  - <a id="2c82d279"></a>audio
  - <a id="2f0492ac"></a>body
  - <a id="efab6432"></a>br
  - <a id="bfff6250"></a>button
  - <a id="0fc40460"></a>canvas
  - <a id="1748bbeb"></a>details
  - <a id="49a6cb20"></a>embed
  - <a id="5b914404"></a>fieldset
  - <a id="88ed8860"></a>frame
  - <a id="cf87e3bd"></a>frameset
  - <a id="c3dd181e"></a>html
  - <a id="87fcd40c"></a>iframe
  - <a id="f0811ff8"></a>img
  - <a id="d7d642a2"></a>input
  - <a id="61885b99"></a>legend
  - <a id="b23654ab"></a>meter
  - <a id="99b5cef6"></a>object
  - <a id="b90b2ad5"></a>progress
  - <a id="0008a4b3"></a>rendered legend
  - <a id="85188fb3"></a>select
  - <a id="b996b941"></a>src
  - <a id="89436b29"></a>summary
  - <a id="fc736137"></a>textarea
  - <a id="aa7bbf63"></a>video
  - <a id="90d63fe4"></a>wbr
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="9e462db5"></a>continuous media
  - <a id="23af89d0"></a>paged media
- \[SELECTORS-4\] defines the following terms:
  - <a id="ad0c2f10"></a>document language
  - <a id="e8cb0e54"></a>pseudo-elements
- \[SVG2\] defines the following terms:
  - <a id="a7f9a4cb"></a>container element
  - <a id="abb19516"></a>non-rendered element
  - <a id="a17bc661"></a>pattern
  - <a id="a7e8836e"></a>presentation attributes
  - <a id="40439e69"></a>renderable element
  - <a id="685aea54"></a>rendered element
  - <a id="45c278c3"></a>svg
  - <a id="cd1011a5"></a>symbol
  - <a id="91a37aa8"></a>text
  - <a id="becb5bd6"></a>text content child element
  - <a id="81f3587f"></a>textPath
  - <a id="1628bded"></a>tspan
  - <a id="356d6b6e"></a>use

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 30 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-3"></a>\[CSS-CASCADE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 14 October 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-speech-1"></a>\[CSS-SPEECH-1\]  
Léonie Watson; Elika Etemad. [CSS Speech Module Level 1](https://www.w3.org/TR/css-speech-1/). 14 February 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-speech-1&#x2F;](https://www.w3.org/TR/css-speech-1/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
Keith Cirkel. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 16 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 29 May 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 20 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

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

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Tab Atkins Jr.; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 19 February 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 22 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-css3-exclusions"></a>\[CSS3-EXCLUSIONS\]  
Rossen Atanassov; Vincent Hardy; Alan Stearns. [CSS Exclusions Module Level 1](https://www.w3.org/TR/css3-exclusions/). 15 January 2015. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-exclusions&#x2F;](https://www.w3.org/TR/css3-exclusions/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                      | Initial | Applies to                | Inh. | %ages | Anim­ation type         | Canonical order | Com­puted value                                                                                                                                                                                              | Media  |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------|---------|---------------------------|------|-------|------------------------|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|
| <strong><span><a id="ref-for-propdef-display⑦④"></a></span><a href="#propdef-display">display</a>&#xA;      </strong> | \[ \<display-outside\> \|\| \<display-inside\> \] \| \<display-listitem\> \| \<display-internal\> \| \<display-box\> \| \<display-legacy\> | inline  | all elements              | no   | n/a   | not animatable         | per grammar     | a pair of keywords representing the inner and outer display types plus optional list-item flag, or a \<display-internal\> or \<display-box\> keyword; see prose in a variety of specs for computation rules |        |
| <strong><span><a id="ref-for-propdef-order①⑦"></a></span><a href="#propdef-order">order</a>&#xA;      </strong> | \<integer\>                                                                                                                                | 0       | flex items and grid items | no   | n/a   | by computed value type | per grammar     | specified integer                                                                                                                                                                                           |        |
| <strong><span><a id="ref-for-propdef-visibility⑥"></a></span><a href="#propdef-visibility">visibility</a>&#xA;      </strong> | visible \| hidden \| collapse                                                                                                              | visible | all elements              | yes  | N/A   | discrete               | per grammar     | as specified                                                                                                                                                                                                | visual |

