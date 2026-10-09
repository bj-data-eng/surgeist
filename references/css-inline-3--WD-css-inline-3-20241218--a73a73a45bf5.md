Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Inline Layout Module Level 3](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium . W3C ® liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Inline Layout Module Level 3

Source snapshot: https://www.w3.org/TR/2024/WD-css-inline-3-20241218/

Snapshot SHA-256: a73a73a45bf591c5964c5d49212de94066a2e4aeda245c7eb1f984c5fd3bb152

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="ref-for-propdef-vertical-align⑦"></a>

# <a id="title"></a>CSS Inline Layout Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

The CSS formatting model provides for a flow of elements and text inside of a container to be wrapped into lines. This module describes box model for this inline layout model and defines the block-axis alignment and sizing of inline-level content, extending the model in [\[CSS2\]](#biblio-css2). It also adds a special layout mode for drop-caps.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-inline” in the title, like this: “\[css-inline\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-inline%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-initial-letter-wrap"></a>

  the [initial-letter-wrap](#propdef-initial-letter-wrap) property

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<a id="ref-for-inline-layout"></a>

<a id="ref-for-inline-level"></a>

<a id="ref-for-block-axis"></a>

This module defines [inline layout](#inline-layout), the CSS model for laying out a mixed stream of text and [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) boxes, and defines controls for the [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) alignment and sizing of this content within each line. It also adds a [special layout mode for drop caps and similar initial letter styling](#initial-letter-styling).

<a id="ref-for-inline-level①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Line-breaking, justification, and other aspects of inline-axis positioning of [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content are handled in the [CSS Text Module](https://www.w3.org/TR/css-text/).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4c501384"></a> Many aspects of layout here depend on font metrics. While the relevant metrics exist in OpenType for Latin/Cyrillic/Greek and for CJK, they are missing for many other writing systems. For example, the visual top metric for Hebrew has no metric in the OpenType tables. For this module to work well for the world, we need fonts to provide the relevant metrics for all writing systems, and that means both that OpenType needs to allow such metrics and font designers need to provide accurate numbers. See [issue](https://github.com/w3c/csswg-drafts/issues/5244) and [liaison statement](https://lists.w3.org/Archives/Public/www-archive/2020Feb/att-0005/CSS-SC29-20200113.pdf).

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the CSS inline layout model and features defined in [\[CSS2\]](#biblio-css2) section 10.8.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="model"></a>2.  Inline Layout Model

<a id="ref-for-inline-level-box"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-fragment"></a>

<a id="ref-for-line-box"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-baseline"></a>

In <a id="inline-layout"></a>inline layout, a mixed, recursive stream of text and [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) forming an <a id="inline-formatting-context"></a>inline formatting context within a [block container](https://www.w3.org/TR/css-display-3/#block-container) are laid out by [fragmenting](https://www.w3.org/TR/css-break-3/#fragment) them into a stack of [line boxes](#line-box). Within each <a id="ref-for-line-box①"></a>line box, <a id="ref-for-inline-level-box①"></a>inline-level boxes are [aligned to each other](#alignment) along the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), typically by the [baselines](#baseline) of their text.

<a id="ref-for-block-container①"></a>

<a id="ref-for-inline-level②"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-atomic-inline"></a>

<a id="ref-for-css-text-sequence"></a>

<a id="ref-for-inline-formatting-context"></a>

<a id="ref-for-inline-layout①"></a>

<a id="ref-for-content-edge"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-inline-level-box②"></a>

Any [block container](https://www.w3.org/TR/css-display-3/#block-container) that directly contains [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content—​such as [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline), and [text sequences](https://www.w3.org/TR/css-display-3/#css-text-sequence)—​establishes an [inline formatting context](#inline-formatting-context) to lay out its contents using [inline layout](#inline-layout). The <a id="ref-for-block-container②"></a>block container’s [content edges](https://www.w3.org/TR/css-box-3/#content-edge) form the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for each of the [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) participating in its <a id="ref-for-inline-formatting-context①"></a>inline formatting context.

<a id="ref-for-block-container③"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-inline-box①"></a>

<a id="ref-for-inline-level③"></a>

<a id="ref-for-inline-formatting-context②"></a>

<a id="ref-for-root-inline-box"></a>

The [block container](https://www.w3.org/TR/css-display-3/#block-container) also generates a <a id="root-inline-box"></a>root inline box, which is an [anonymous](https://www.w3.org/TR/css-display-3/#anonymous) [inline box](https://www.w3.org/TR/css-display-3/#inline-box) that holds all of its [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) contents. (Thus, all text in an [inline formatting context](#inline-formatting-context) is directly contained by an <a id="ref-for-inline-box②"></a>inline box, whether the [root inline box](#root-inline-box) or one of its descendants.) The <a id="ref-for-root-inline-box①"></a>root inline box inherits from its parent <a id="ref-for-block-container④"></a>block container, but is otherwise unstyleable.

<a id="ref-for-inline-formatting-context③"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-margin"></a>

<a id="ref-for-border"></a>

<a id="ref-for-padding"></a>

<a id="ref-for-inline-level-box③"></a>

<a id="ref-for-inline-level④"></a>

In an [inline formatting context](#inline-formatting-context), content is laid out along the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), ordered according to the [Unicode bidirectional algorithm and its controls](https://www.w3.org/TR/css-writing-modes-3/#text-direction) [\[CSS-WRITING-MODES-3\]](#biblio-css-writing-modes-3) and distributed according to the typesetting controls in [\[CSS-TEXT-3\]](#biblio-css-text-3). <a id="ref-for-inline-axis①"></a>Inline-axis [margins](https://www.w3.org/TR/css-box-4/#margin), [borders](https://www.w3.org/TR/css-box-4/#border), and [padding](https://www.w3.org/TR/css-box-4/#padding) are respected between [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) (and their margins do not [collapse](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)). The resulting rectangular area that contains the boxes that form a single line of [inline-level content](https://www.w3.org/TR/css-display-3/#inline-level) is called a <a id="line-box"></a>line box.

<a id="ref-for-line-box②"></a>

<a id="ref-for-inline-box③"></a>

<a id="ref-for-inline-level-box④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Line boxes](#line-box) and [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) and [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) are each different things! See [\[CSS-DISPLAY-3\]](#biblio-css-display-3) for an in-depth discussion of box types and related terminology.

### <a id="line-boxes"></a>2.1.  Layout of Line Boxes

<a id="ref-for-line-box③"></a>

<a id="ref-for-inline-level⑤"></a>

<a id="ref-for-inline-formatting-context④"></a>

<a id="ref-for-inline-box④"></a>

<a id="ref-for-logical-width"></a>

<a id="ref-for-forced-line-break"></a>

<a id="ref-for-fragment①"></a>

<a id="ref-for-column-box"></a>

<a id="ref-for-multi-column-layout"></a>

<a id="ref-for-fragmentation-container"></a>

<a id="ref-for-formatting-context"></a>

<a id="ref-for-box-tree"></a>

[Line boxes](#line-box) are created as needed to hold [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content within an [inline formatting context](#inline-formatting-context). When an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) exceeds the [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of a <a id="ref-for-line-box④"></a>line box, or contains a [forced line break](https://www.w3.org/TR/css-text-3/#forced-line-break), it is split (see [CSS Text 3 § 5 Line Breaking and Word Boundaries](https://www.w3.org/TR/css-text-3/#line-breaking)) into several [fragments](https://www.w3.org/TR/css-break-3/#fragment) [\[CSS-BREAK-3\]](#biblio-css-break-3), which are partitioned across multiple <a id="ref-for-line-box⑤"></a>line boxes. Like [column boxes](https://www.w3.org/TR/css-multicol-1/#column-box) in [multi-column layout](https://www.w3.org/TR/css-multicol-1/#multi-column-layout) [\[CSS-MULTICOL-1\]](#biblio-css-multicol-1), <a id="ref-for-line-box⑥"></a>line boxes are [fragmentation containers](https://www.w3.org/TR/css-break-4/#fragmentation-container) generated by their [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context), and are not part of the CSS [box tree](https://www.w3.org/TR/css-display-3/#box-tree).

<a id="ref-for-inline-box⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) can also be [split into several fragments within the same line box due to bidirectional text processing](https://www.w3.org/TR/css-writing-modes-3/#bidi-box-model). See [\[CSS-WRITING-MODES-3\]](#biblio-css-writing-modes-3).

<a id="ref-for-line-box⑦"></a>

<a id="ref-for-block-container⑤"></a>

<a id="ref-for-block-flow-direction"></a>

<a id="ref-for-propdef-align-content"></a>

<a id="ref-for-inline-formatting-context⑤"></a>

<a id="ref-for-float"></a>

[Line boxes](#line-box) are stacked as the direct contents of the [block container box](https://www.w3.org/TR/css-display-3/#block-container) in its [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) and aligned within this container as specified by [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) [\[CSS-ALIGN-3\]](#biblio-css-align-3). Thus, an [inline formatting context](#inline-formatting-context) consists of a stack of <a id="ref-for-line-box⑧"></a>line boxes. <a id="ref-for-line-box⑨"></a>Line boxes are stacked with no separation (except as specified elsewhere, e.g. for [float](https://www.w3.org/TR/css-page-floats-3/#float) [clearance](https://www.w3.org/TR/CSS2/visuren.html#clearance)) and they never overlap.

<a id="ref-for-line-left"></a>

<a id="ref-for-line-box①⓪"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-line-right"></a>

<a id="ref-for-logical-width①"></a>

<a id="ref-for-inner-size"></a>

<a id="ref-for-block-container⑥"></a>

<a id="ref-for-x10"></a>

<a id="ref-for-initial-letter"></a>

In general, the [line-left](https://www.w3.org/TR/css-writing-modes-4/#line-left) edge of a [line box](#line-box) touches the <a id="ref-for-line-left①"></a>line-left edge of its [containing block](https://www.w3.org/TR/css-display-3/#containing-block) and the [line-right](https://www.w3.org/TR/css-writing-modes-4/#line-right) edge touches the <a id="ref-for-line-right①"></a>line-right edge of its <a id="ref-for-containing-block②"></a>containing block, and thus the [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of a line box is equal to the [inner](https://www.w3.org/TR/css-sizing-3/#inner-size) <a id="ref-for-logical-width②"></a>logical width of its <a id="ref-for-containing-block③"></a>containing block (i.e. the [block container](https://www.w3.org/TR/css-display-3/#block-container)’s [content box](https://www.w3.org/TR/CSS21/box.html#x10)). However, floating boxes or [initial letter boxes](#initial-letter) can come between the <a id="ref-for-containing-block④"></a>containing block edge and the <a id="ref-for-line-box①①"></a>line box edge, reducing the space available to, and thus the <a id="ref-for-logical-width③"></a>logical width of, any such impacted <a id="ref-for-line-box①②"></a>line boxes. (See [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification § visuren#inline-formatting](https://www.w3.org/TR/CSS21//visuren#inline-formatting)/[Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification § visuren#floats](https://www.w3.org/TR/CSS21//visuren#floats) and [§ 7 Initial Letters](#initial-letter-styling).)

<a id="ref-for-logical-height"></a>

<a id="ref-for-line-box①③"></a>

<a id="ref-for-propdef-line-height"></a>

<a id="ref-for-propdef-line-fit-edge"></a>

<a id="ref-for-block-container⑦"></a>

<a id="ref-for-propdef-text-box-trim"></a>

The [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of a [line box](#line-box) is fitted to its contents once they have been [block-axis aligned](#alignment). This fit is controlled by [line-height](#propdef-line-height) and [line-fit-edge](#propdef-line-fit-edge). The first/last line boxes in a [block container](https://www.w3.org/TR/css-display-3/#block-container) may additionally be trimmed by [text-box-trim](#propdef-text-box-trim).

![diagram showing inline boxes split across line boxes as described above](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/box-model.png)

Inline Layout Box Model

### <a id="line-layout"></a>2.2.  Layout Within Line Boxes

<a id="ref-for-inline-level-box⑤"></a>

<a id="ref-for-line-box①④"></a>

<a id="ref-for-box-fragment"></a>

As described [above](#model), user agents flow [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) into a stack of [line boxes](#line-box). Layout within each <a id="ref-for-line-box①⑤"></a>line box is performed, sizing and positioning each [box fragment](https://www.w3.org/TR/css-break-4/#box-fragment) and <a id="ref-for-line-box①⑥"></a>line box independently, as follows:

1.  <a id="ref-for-in-flow"></a>

    <a id="ref-for-inline-level-box⑥"></a>

    <a id="ref-for-line-box①⑦"></a>

    <a id="ref-for-block-axis②"></a>

    <a id="ref-for-propdef-dominant-baseline"></a>

    <a id="ref-for-propdef-vertical-align"></a>

    <a id="ref-for-line-relative-shift-values"></a>

    <a id="ref-for-propdef-baseline-shift"></a>

    <strong>Baseline Alignment:</strong> All [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) in the [line box](#line-box) are aligned to each other in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) according to [dominant-baseline](#propdef-dominant-baseline) and [vertical-align](#propdef-vertical-align). This is referred to as [baseline alignment](#alignment). Those with [line-relative values](#line-relative-shift-values) for [baseline-shift](#propdef-baseline-shift) are assumed to be aligned so as to minimize the line box height.

2.  <a id="ref-for-layout-bounds"></a>

    <a id="ref-for-inline-level-box⑦"></a>

    <a id="ref-for-line-box①⑧"></a>

    <strong>Content Size Contribution Calculation:</strong> The [layout bounds](#layout-bounds) (i.e. the size contributions) of each [inline-level box](https://www.w3.org/TR/css-display-3/#inline-level-box) in the [line box](#line-box) are calculated:

    - <a id="ref-for-margin-box"></a>

      <a id="ref-for-inline-block"></a>

      <a id="ref-for-replaced-element"></a>

      <a id="ref-for-atomic-inline①"></a>

      For [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) such as [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) and [inline blocks](https://www.w3.org/TR/css-display-3/#inline-block): this is their [margin box](https://www.w3.org/TR/css-box-3/#margin-box).

    - <a id="ref-for-padding①"></a>

      <a id="ref-for-border①"></a>

      <a id="ref-for-margin①"></a>

      <a id="ref-for-propdef-line-height①"></a>

      <a id="ref-for-propdef-line-fit-edge①"></a>

      <a id="ref-for-inline-box⑥"></a>

      <a id="ref-for-root-inline-box②"></a>

      For the [root inline box](#root-inline-box), and for [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) with [line-fit-edge: leading](#propdef-line-fit-edge): this derived from their used [line-height](#propdef-line-height), ignoring any [margin](https://www.w3.org/TR/css-box-4/#margin)/[border](https://www.w3.org/TR/css-box-4/#border)/[padding](https://www.w3.org/TR/css-box-4/#padding); see [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height).

    - <a id="ref-for-padding②"></a>

      <a id="ref-for-border②"></a>

      <a id="ref-for-margin②"></a>

      <a id="ref-for-propdef-line-fit-edge②"></a>

      <a id="ref-for-inline-box⑦"></a>

      For other [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box): this is derived from their [line-fit-edge](#propdef-line-fit-edge) metrics, and includes any [margin](https://www.w3.org/TR/css-box-4/#margin)/[border](https://www.w3.org/TR/css-box-4/#border)/[padding](https://www.w3.org/TR/css-box-4/#padding); see [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height).

3.  <a id="ref-for-line-box①⑨"></a>

    <a id="ref-for-logical-height①"></a>

    <a id="ref-for-layout-bounds①"></a>

    <a id="ref-for-inline-level-box⑧"></a>

    <strong>Line Box Sizing:</strong> The [line box](#line-box)’s [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) is sized to exactly include the aligned [layout bounds](#layout-bounds) of all its [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box).

4.  <a id="ref-for-root-inline-box③"></a>

    <a id="ref-for-aligned-subtree"></a>

    <a id="ref-for-line-relative-shift-values①"></a>

    <a id="ref-for-propdef-baseline-shift①"></a>

    <a id="ref-for-line-box②⓪"></a>

    <strong>Content Positioning:</strong> The [root inline box](#root-inline-box)’s [aligned subtree](#aligned-subtree) and boxes [line-relative values](#line-relative-shift-values) for [baseline-shift](#propdef-baseline-shift) are positioned within the [line box](#line-box).

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-e2d22611"></a> Define what to do for top/bottom/center aligned boxes that are taller than the rest of the content.

<a id="ref-for-inline-box⑧"></a>

<a id="ref-for-margin③"></a>

<a id="ref-for-padding③"></a>

<a id="ref-for-border③"></a>

<a id="ref-for-propdef-line-height②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Empty [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) still have [margins](https://www.w3.org/TR/css-box-4/#margin), [padding](https://www.w3.org/TR/css-box-4/#padding), [borders](https://www.w3.org/TR/css-box-4/#border), and a [line-height](#propdef-line-height), and thus influence these calculations just like boxes with content.

### <a id="invisible-line-boxes"></a>2.3.  Phantom Line Boxes

<a id="ref-for-line-box②①"></a>

<a id="ref-for-preserved-white-space"></a>

<a id="ref-for-inline-box⑨"></a>

<a id="ref-for-margin④"></a>

<a id="ref-for-padding④"></a>

<a id="ref-for-border④"></a>

<a id="ref-for-in-flow①"></a>

<a id="ref-for-atomic-inline②"></a>

<a id="ref-for-ruby-annotation-box"></a>

<a id="ref-for-forced-line-break①"></a>

<a id="ref-for-logical-height②"></a>

<a id="ref-for-absolute-position"></a>

[Line boxes](#line-box) that contain no text, no [preserved white space](https://www.w3.org/TR/css-text-4/#preserved-white-space), no [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) with non-zero inline-axis [margins](https://www.w3.org/TR/css-box-4/#margin), [padding](https://www.w3.org/TR/css-box-4/#padding), or [borders](https://www.w3.org/TR/css-box-4/#border), and no other [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) content (such as [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) or [ruby annotations](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-box)), and do not end with a [forced line break](https://www.w3.org/TR/css-text-4/#forced-line-break) are <a id="phantom-line-box"></a>phantom line boxes. Such boxes must be treated as zero-[height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) <a id="ref-for-line-box②②"></a>line boxes for the purposes of determining the positions of any descendant content (such as [absolutely positioned boxes](https://www.w3.org/TR/css-position-3/#absolute-position)), and both the <a id="ref-for-line-box②③"></a>line box and its <a id="ref-for-in-flow②"></a>in-flow content must be treated as not existing for any other layout or rendering purpose.

> <strong data-conversion-semantic="note">Note</strong>
>
> What’s invisible?
>
> <a id="ref-for-phantom-line-box"></a>
>
> <a id="ref-for-inline-box①⓪"></a>
>
> <a id="ref-for-out-of-flow"></a>
>
> <a id="ref-for-white-space"></a>
>
> Such [phantom line boxes](#phantom-line-box), which can still contain unstyled empty [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), [out-of-flow](https://www.w3.org/TR/css-display-3/#out-of-flow) boxes, and/or collapsed [document white space](https://www.w3.org/TR/css-text-4/#white-space), are ignored, for example, for:
>
> - [margin collapsing](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)
>
> - <a id="ref-for-first-formatted-line"></a>
>
>   finding the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line)
>
> - <a id="ref-for-propdef-text-box-trim①"></a>
>
>   applying [text-box-trim](#propdef-text-box-trim)
>
> - [fragmentation break propagation](https://www.w3.org/TR/css-break-4/#break-propagation)
>
> - etc.

<a id="ref-for-phantom-line-box①"></a>

<a id="ref-for-propdef-outline"></a>

<a id="ref-for-propdef-box-shadow"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8b05a2ec"></a> Firefox allows the inline boxes within a [phantom line box](#phantom-line-box) to accept [outline](https://www.w3.org/TR/CSS21/ui.html#propdef-outline),which allows it to make focus rings visible. As in other browsers, all other properties that could make the element visible (e.g. [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow)) seem to be ignored.

### <a id="paint-order"></a>2.4.  Painting Order

<a id="ref-for-positioned-box"></a>

<a id="ref-for-inline-level-box⑨"></a>

<a id="ref-for-document-order"></a>

<a id="ref-for-propdef-z-index"></a>

Except as specified for [positioned boxes](https://www.w3.org/TR/css-position-3/#positioned-box) (see [\[CSS-POSITION-3\]](#biblio-css-position-3)) [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) are painted in [document order](https://www.w3.org/TR/css-display-3/#document-order); the [z-index](https://www.w3.org/TR/CSS21/visuren.html#propdef-z-index) property does not generally apply.

## <a id="css-metrics"></a>3.  Baselines and Alignment Metrics

### <a id="baseline-intro"></a>3.1.  Introduction to Baselines

<a id="ref-for-inline-axis②"></a>

<a id="ref-for-baseline①"></a>

A <a id="baseline"></a>baseline is a line along the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of a line box along which individual glyphs of text are aligned. [Baselines](#baseline) guide the design of glyphs in a font (for example, the bottom of most alphabetic glyphs typically align with the alphabetic baseline), and they guide the alignment of glyphs from different fonts or font sizes when typesetting.

![Picture of alphabetic text in two font sizes with the baseline and em-boxes](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/alphabetic-baseline-in-two-font-sizes.svg)

Alphabetic text in two font sizes with the baseline and em-boxes

<a id="ref-for-baseline②"></a>

Different writing systems prefer different [baselines](#baseline).

![Latin prefers the alphabetic baseline, on top of which most letters rest, though some letters have descenders that dangle below it. Indic scripts are sometimes typeset with a hanging baseline, since their glyph shapes appear to be hanging from a horizontal line. Han-based systems, whose glyphs are designed to fill a square, tend to align on their bottoms or through their centers.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/script-preferred-baselines.gif)

Preferred baselines in various writing systems

<a id="ref-for-baseline③"></a>

A well-constructed font contains a <a id="baseline-table"></a>baseline table, which indicates the position of one or more [baselines](#baseline) within the font’s design coordinate space. (The design coordinate space is scaled with the font size.)

![](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/baselines.gif)

In a well-designed mixed-script font, the glyphs are positioned in the coordinate space to harmonize with one another when typeset together. The baseline table is then constructed to match the shape of the glyphs, each baseline positioned to match the glyphs from its preferred scripts.

<a id="ref-for-baseline-table"></a>

The [baseline table](#baseline-table) is a property of the font, and the positions of the various baselines apply to all glyphs in the font.

<a id="ref-for-baseline-table①"></a>

<a id="ref-for-typographic-mode"></a>

Different [baseline tables](#baseline-table) can be provided for alignment in horizontal and vertical text. UAs should use the vertical tables in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode) and the horizontal tables otherwise.

<a id="ref-for-baseline-table②"></a>

<a id="ref-for-propdef-font-language-override"></a>

<a id="ref-for-content-language"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Fonts can have more than one [baseline table](#baseline-table) in each axis; the UA is responsible for choosing the appropriate table in consideration of [font-language-override](https://www.w3.org/TR/css-fonts-4/#propdef-font-language-override) and the [content language](https://www.w3.org/TR/css-text-4/#content-language).

### <a id="baseline-types"></a>3.2.  Baselines and Metrics

<a id="ref-for-baseline④"></a>

<a id="ref-for-inline-layout②"></a>

CSS uses the following text-based metrics as [baselines](#baseline) for [inline layout](#inline-layout) functions such as alignment, box sizing, and initial letter layout.

<a id="ref-for-propdef-dominant-baseline①"></a>

<a id="ref-for-propdef-alignment-baseline"></a>

<a id="ref-for-propdef-text-box-edge"></a>

<a id="ref-for-propdef-line-fit-edge③"></a>

<a id="ref-for-propdef-initial-letter-align"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3bc5ecf6"></a> The CSSWG would like to know which baseline values are necessary for each property that uses them ([dominant-baseline](#propdef-dominant-baseline), [alignment-baseline](#propdef-alignment-baseline), [text-box-edge](#propdef-text-box-edge), [line-fit-edge](#propdef-line-fit-edge), [initial-letter-align](#propdef-initial-letter-align)): if any can be dropped, or any need to be added. See [Issue 859](https://github.com/w3c/csswg-drafts/issues/859).

<a id="alphabetic-baseline"></a>alphabetic  
Used in writing Latin, Cyrillic, Greek, and many other scripts, corresponds to the bottom of most, but not all, their characters, (such as “m”, “Ш”, “Δ”). Often represented as zero in font design coordinate systems; assigned to `romn` in OpenType and to `bsln` value zero in TrueType AAT.

<a id="cap-height-baseline"></a>cap-height  
Corresponds to the top of capital letters (such as “T”, “Б”, “Σ”) in Latin, Cyrillic, Greek, etc. Calculated using `sCapHeight` in OpenType.

<a id="x-height-baseline"></a>x-height  
Corresponds to the top of short lowercase letters (such as “m”, “л”, “α”) in Latin, Cyrillic, Greek, etc. Calculated using `sxHeight` in OpenType.

<a id="x-middle-baseline"></a>x-middle  
<a id="ref-for-x-height-baseline"></a>

<a id="ref-for-alphabetic-baseline"></a>

Corresponds to halfway between the [alphabetic](#alphabetic-baseline) and [x-height](#x-height-baseline) baselines.

<a id="ideographic-over-baseline"></a>ideographic-over  
<a id="ref-for-line-over"></a>

Corresponds to the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) design edge of CJK (Han/Hangul/Kana) text. Assigned to `idtp` in OpenType.

<a id="ideographic-under-baseline"></a>ideographic-under  
<a id="ref-for-line-under"></a>

Corresponds to the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) design edge of CJK (Han/Hangul/Kana) text. Assigned to `ideo` in OpenType.

<a id="central-baseline"></a>central  
<a id="ref-for-ideographic-over-baseline"></a>

<a id="ref-for-ideographic-under-baseline"></a>

Corresponds to the ideographic central baseline, halfway between the [ideographic-under](#ideographic-under-baseline) and [ideographic-over](#ideographic-over-baseline) baselines. Assigned to `bsln` value 1 in TrueType AAT.

<a id="ideographic-ink-over-baseline"></a>ideographic-ink-over  
<a id="ref-for-line-over①"></a>

Corresponds to the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) ink edge of CJK (Han/Hangul/Kana) text. Assigned to `icft` in OpenType.

<a id="ideographic-ink-under-baseline"></a>ideographic-ink-under  
Corresponds to the line-under ink edge of CJK (Han/Hangul/Kana) text. Assigned `icfb` in OpenType.

<a id="hanging-baseline"></a>hanging  
Corresponds to hanging baseline from which characters in Tibetan and similar unicameral scripts with a strong but not absolute top edge seem to “hang”. Assigned to `hang` in OpenType and to `bsln` value 3 in TrueType AAT.

<a id="math-baseline"></a>math  
Corresponds to center baseline around which mathematical characters are designed. Assigned to `math` in OpenType and `bsln` value 4 in TrueType AAT.

<a id="text-over-baseline"></a>text-over  
<a id="ref-for-x10①"></a>

<a id="ref-for-inline-box①①"></a>

<a id="ref-for-line-over②"></a>

Corresponds to the metric used as the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) edge of an [inline](https://www.w3.org/TR/css-display-3/#inline-box)’s [content box](https://www.w3.org/TR/CSS21/box.html#x10) per [\[CSS2\]](#biblio-css2).

<a id="text-under-baseline"></a>text-under  
<a id="ref-for-x10②"></a>

<a id="ref-for-inline-box①②"></a>

<a id="ref-for-line-under①"></a>

Corresponds to the metric used as the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) edge of an [inline](https://www.w3.org/TR/css-display-3/#inline-box)’s [content box](https://www.w3.org/TR/CSS21/box.html#x10) per [\[CSS2\]](#biblio-css2).

<a id="em-over-baseline"></a>em-over  
<a id="ref-for-em-under-baseline"></a>

<a id="ref-for-em-over-baseline"></a>

<a id="ref-for-ascent-metric"></a>

Corresponds to a conceptual [ascent](#ascent-metric) normalized to ensure 1em between [em-over](#em-over-baseline) and [em-under](#em-under-baseline). See [A.1: Calculating Em-over and Em-under](#baseline-synthesis-em).

<a id="em-under-baseline"></a>em-under  
<a id="ref-for-em-under-baseline①"></a>

<a id="ref-for-em-over-baseline①"></a>

<a id="ref-for-descent-metric"></a>

Corresponds to a conceptual [descent](#descent-metric) normalized to ensure 1em between [em-over](#em-over-baseline) and [em-under](#em-under-baseline). See [A.1: Calculating Em-over and Em-under](#baseline-synthesis-em).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These metrics are optical design metrics, and therefore do not necessarily correspond exactly to actual glyph outlines.

In general, these metrics are taken from the appropriate font, but if they are missing or need to be derived from a box rather than text, they must be synthesized, see [§ 3.3 Baselines of Glyphs and Boxes](#baseline-tables) and [Appendix A: Synthesizing Alignment Metrics](#baseline-synthesis).

#### <a id="ascent-descent"></a>3.2.1.  Ascent and Descent Metrics

<a id="ref-for-inline-formatting-context⑥"></a>

CSS assumes that every font has font metrics that specify a characteristic height above the baseline—​called the <a id="ascent-metric"></a>ascent metric—​and a characteristic depth below it—​called the <a id="descent-metric"></a>descent metric—​which CSS uses for laying out text and boxes in an [inline formatting context](#inline-formatting-context). Note that these are metrics of the font as a whole and need not correspond to the ascender and descender of any individual glyph.

<a id="ref-for-ascent-metric①"></a>

<a id="ref-for-descent-metric①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is recommended that implementations that use OpenType or TrueType fonts use the metrics `sTypoAscender` and `sTypoDescender` from the font’s OS/2 table (after scaling to the current element’s font size) to find the [ascent metric](#ascent-metric) and [descent metric](#descent-metric) for CSS layout. In the absence of these metrics, the "Ascent" and "Descent" metrics from the HHEA table should be used.

#### <a id="font-line-gap"></a>3.2.2.  Line Gap Metrics

<a id="ref-for-line-box②④"></a>

<a id="ref-for-logical-height③"></a>

<a id="ref-for-propdef-line-height③"></a>

<a id="ref-for-valdef-line-height-normal"></a>

Font formats can allow for a font-recommended “line gap” or “external leading” metric. This metric is referred to as the <a id="line-gap-metric"></a>line gap metric, and may be incorporated into the [line box](#line-box) [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) calculations when [line-height](#propdef-line-height) is [normal](#valdef-line-height-normal) as described in [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height).

<a id="ref-for-line-gap-metric"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In OpenType, the [line gap metric](#line-gap-metric) can be found as `sTypoLineGap` or `hhea.lineGap`.

<a id="ref-for-line-gap-metric①"></a>

UAs must floor the [line gap metric](#line-gap-metric) at zero.

### <a id="baseline-tables"></a>3.3.  Baselines of Glyphs and Boxes

<a id="ref-for-inline-level-box①⓪"></a>

<a id="ref-for-baseline⑤"></a>

<a id="ref-for-block-axis③"></a>

<a id="ref-for-shared-alignment-context"></a>

<a id="ref-for-alignment-baseline"></a>

<a id="ref-for-dominant-baseline"></a>

Each font, glyph, and [inline-level box](https://www.w3.org/TR/css-display-3/#inline-level-box) is assumed to have a [baseline](#baseline) coordinate for each <a id="ref-for-baseline⑥"></a>baseline type indicating that <a id="ref-for-baseline⑦"></a>baseline’s position on its [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). The set of such <a id="ref-for-baseline⑧"></a>baselines is called its <a id="baseline-set"></a>baseline set. The <a id="ref-for-baseline⑨"></a>baseline from this set that is used to align the box or glyph within its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context) is called its [alignment baseline](#alignment-baseline); the <a id="ref-for-baseline①⓪"></a>baseline used to align its content within itself is called its [dominant baseline](#dominant-baseline).

<a id="ref-for-baseline-set"></a>

<a id="ref-for-baseline-table③"></a>

<a id="ref-for-inline-box①③"></a>

<a id="ref-for-first-available-font"></a>

For an individual glyph, the [baseline set](#baseline-set) derives from the font’s [baseline table](#baseline-table). For an [inline box](https://www.w3.org/TR/css-display-3/#inline-box), it derives from its [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) regardless of whether the box actually contains any glyphs from that font. If the requisite metrics are missing from a font, the UA must synthesize them, see [A.2: Synthesizing Baselines (and Other Font Metrics) for Text](#baseline-synthesis-fonts).

<a id="ref-for-box"></a>

<a id="ref-for-baseline-set①"></a>

<a id="ref-for-propdef-baseline-source"></a>

<a id="ref-for-formatting-context①"></a>

<a id="ref-for-atomic-inline③"></a>

<a id="ref-for-inline-formatting-context⑦"></a>

<a id="ref-for-inline-axis③"></a>

<a id="ref-for-alignment-baseline①"></a>

<a id="ref-for-synthesize-baseline"></a>

<a id="ref-for-margin-box①"></a>

For other [boxes](https://www.w3.org/TR/css-display-3/#box), its [baseline set](#baseline-set) is nominally derived from its contents in accordance with [baseline-source](#propdef-baseline-source) and the rules of the [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context) in which it participates. For an [atomic inline box](https://www.w3.org/TR/css-display-3/#atomic-inline) with no <a id="ref-for-baseline-set②"></a>baseline set in the [inline formatting context](#inline-formatting-context)’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) its [alignment baselines](#alignment-baseline) are [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) from its [margin box](https://www.w3.org/TR/css-box-3/#margin-box), see [A.3: Synthesizing Baselines for Atomic Inlines](#baseline-synthesis-box).

## <a id="alignment"></a>4.  Baseline Alignment

<a id="ref-for-formatting-context②"></a>

<a id="ref-for-inline-layout③"></a>

<a id="ref-for-block-axis④"></a>

<a id="ref-for-baseline①①"></a>

While most CSS [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context) position content by aligning boxes with respect to their container’s edges, [inline layout](#inline-layout) positions boxes in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) by aligning them with respect to each other using their [baselines](#baseline).

<a id="ref-for-line-relative-shift-values②"></a>

<a id="ref-for-inline-level-box①①"></a>

<a id="ref-for-block-axis⑤"></a>

<a id="ref-for-alignment-baseline②"></a>

<a id="ref-for-baseline①②"></a>

<a id="ref-for-shared-alignment-context①"></a>

<a id="ref-for-post-alignment-shift"></a>

More specifically, (unless using a [line-relative shift value](#line-relative-shift-values)) each glyph or [inline-level box](https://www.w3.org/TR/css-display-3/#inline-level-box) is aligned in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) by positioning its [alignment baseline](#alignment-baseline) to match the <em>corresponding</em> [baseline](#baseline) of its parent (which is its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context)), and then is potentially shifted from that position according to its [post-alignment shift](#post-alignment-shift).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Baseline alignment always matches corresponding baselines: alphabetic to alphabetic, hanging to hanging, mathematical to mathematical, etc.

<a id="ref-for-box①"></a>

<a id="ref-for-alignment-baseline③"></a>

<a id="ref-for-propdef-alignment-baseline①"></a>

<a id="ref-for-propdef-baseline-source①"></a>

<a id="ref-for-propdef-vertical-align①"></a>

<a id="ref-for-propdef-dominant-baseline②"></a>

<a id="ref-for-dominant-baseline①"></a>

When aligning a [box](https://www.w3.org/TR/css-display-3/#box), the [alignment baseline](#alignment-baseline) is chosen according to its [alignment-baseline](#propdef-alignment-baseline) and [baseline-source](#propdef-baseline-source) values (see shorthand [vertical-align](#propdef-vertical-align)), and defaults to matching the parent’s [dominant-baseline](#propdef-dominant-baseline). For a glyph, the <a id="ref-for-alignment-baseline④"></a>alignment baseline is always determined by the parent’s [dominant baseline](#dominant-baseline).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-713d3335"></a>
>
> Given following sample markup:
>
> ```text
> <p><span class="outer">Ap <span class="inner">ਜੀ</span></span></p>
> ```
>
> And the following style rule:
>
> ```text
> .inner { font-size: 75%; }
> ```
>
> <a id="ref-for-baseline-set③"></a>
>
> <a id="ref-for-alphabetic-baseline①"></a>
>
> The [baseline sets](#baseline-set) of the parent (`.outer`) and the child (`.inner`) will not match up due to the font size difference. The child box is aligned to its parent by matching up their [alphabetic baselines](#alphabetic-baseline).
>
> ![In this example, the distance between each baseline in the baseline set is compacted 75% in the span with a 75% font size. Their alphabetic baselines, however, line up.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/baseline-align-sizes.gif)
>
> <a id="ref-for-alphabetic-baseline②"></a>
>
> <a id="ref-for-alignment-baseline⑤"></a>
>
> <a id="ref-for-dominant-baseline②"></a>
>
> <a id="ref-for-typographic-mode①"></a>
>
> The [alphabetic baseline](#alphabetic-baseline) is used here because by default a box’s [alignment baseline](#alignment-baseline) matches the [dominant baseline](#dominant-baseline) of its parent, and in horizontal [typographic mode](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode), the <a id="ref-for-dominant-baseline③"></a>dominant baseline itself defaults to the <a id="ref-for-alphabetic-baseline③"></a>alphabetic baseline.
>
> <a id="ref-for-propdef-vertical-align②"></a>
>
> If we add [vertical-align: super](#propdef-vertical-align) to the `.inner` element from the example above, the same rules are used to align the `.inner` child to its parent; but in addition to the baseline alignment, the child is shifted to the superscript position.
>
> ![In this example, the resulting alignment is equivalent to shifting the parent baseline table upwards by its superscript offset, and then aligning the child's alphabetic baseline to the shifted position of the parent's alphabetic baseline.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/baseline-align-super.gif)

<a id="ref-for-propdef-dominant-baseline③"></a>

### <a id="dominant-baseline-property"></a>4.1.  Dominant Baselines: the [dominant-baseline](#propdef-dominant-baseline) property



| Field               | Definition                                                                                                                                                                                                                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-dominant-baseline"></a>dominant-baseline                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) text-bottom <a id="ref-for-comb-one①"></a>\| alphabetic <a id="ref-for-comb-one②"></a>\| ideographic <a id="ref-for-comb-one③"></a>\| middle <a id="ref-for-comb-one④"></a>\| central <a id="ref-for-comb-one⑤"></a>\| mathematical <a id="ref-for-comb-one⑥"></a>\| hanging <a id="ref-for-comb-one⑦"></a>\| text-top |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-TermTextContentElement"></a>block containers, inline boxes, table rows, grid containers, flex containers, and SVG [text content elements](https://www.w3.org/TR/SVG2/text.html#TermTextContentElement)                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                             |



This property specifies the <a id="dominant-baseline"></a>dominant baseline, which is the default baseline type used to align content within the box.

<a id="ref-for-inline-box①④"></a>

<a id="ref-for-dominant-baseline④"></a>

<a id="ref-for-propdef-vertical-align③"></a>

<a id="ref-for-inline-level⑥"></a>

<a id="ref-for-alignment-baseline⑥"></a>

<a id="ref-for-baseline-alignment"></a>

<a id="ref-for-shared-alignment-context②"></a>

<a id="ref-for-propdef-alignment-baseline②"></a>

For [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), the [dominant baseline](#dominant-baseline) is used to align the box’s text (and, unless otherwise specified by [vertical-align](#propdef-vertical-align), any [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) child boxes) by aligning each glyph/box’s corresponding baseline to the box’s own <a id="ref-for-dominant-baseline⑤"></a>dominant baseline. For other boxes, it indicates the default [alignment baseline](#alignment-baseline) of any boxes participating in [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) in the box’s [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context); see ([alignment-baseline: baseline](#propdef-alignment-baseline) and [\[CSS-ALIGN-3\]](#biblio-css-align-3)).

Values have the following meanings:

<a id="valdef-dominant-baseline-auto"></a>auto  
<a id="ref-for-valdef-text-orientation-upright"></a>

<a id="ref-for-valdef-text-orientation-mixed"></a>

<a id="ref-for-valdef-dominant-baseline-central"></a>

<a id="ref-for-valdef-text-orientation-sideways"></a>

<a id="ref-for-propdef-text-orientation"></a>

<a id="ref-for-vertical-writing-mode"></a>

<a id="ref-for-horizontal-writing-mode"></a>

<a id="ref-for-valdef-dominant-baseline-alphabetic"></a>

Equivalent to [alphabetic](#valdef-dominant-baseline-alphabetic) in [horizontal writing modes](https://www.w3.org/TR/css-writing-modes-4/#horizontal-writing-mode) and in [vertical writing modes](https://www.w3.org/TR/css-writing-modes-4/#vertical-writing-mode) when [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) is [sideways](https://www.w3.org/TR/css-writing-modes-4/#valdef-text-orientation-sideways). Equivalent to [central](#valdef-dominant-baseline-central) in <a id="ref-for-vertical-writing-mode①"></a>vertical writing modes when <a id="ref-for-propdef-text-orientation①"></a>text-orientation is [mixed](https://www.w3.org/TR/css-writing-modes-4/#valdef-text-orientation-mixed) or [upright](https://www.w3.org/TR/css-writing-modes-4/#valdef-text-orientation-upright).

<a id="ref-for-valdef-dominant-baseline-central①"></a>

<a id="ref-for-vertical-writing-mode②"></a>

However, in SVG text, the origin point of glyphs (used for coordinate-based glyph positioning) is always handled as for [central](#valdef-dominant-baseline-central) in [vertical writing modes](https://www.w3.org/TR/css-writing-modes-4/#vertical-writing-mode).

<a id="valdef-dominant-baseline-text-bottom"></a>text-bottom  
<a id="ref-for-text-under-baseline"></a>

Use the [text-under baselines](#text-under-baseline).

<a id="valdef-dominant-baseline-alphabetic"></a>alphabetic  
<a id="ref-for-alphabetic-baseline④"></a>

Use the [alphabetic baselines](#alphabetic-baseline).

<a id="valdef-dominant-baseline-ideographic"></a>ideographic  
<a id="ref-for-ideographic-under-baseline①"></a>

Use the [ideographic-under baselines](#ideographic-under-baseline).

<a id="valdef-dominant-baseline-middle"></a>middle  
<a id="ref-for-central-baseline"></a>

<a id="ref-for-x-height-baseline①"></a>

<a id="ref-for-alphabetic-baseline⑤"></a>

<a id="ref-for-propdef-text-orientation②"></a>

<a id="ref-for-x-middle-baseline"></a>

Use the [x-middle baselines](#x-middle-baseline); except under [text-orientation: upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) (where the [alphabetic](#alphabetic-baseline) and [x-height](#x-height-baseline) baselines are essentially meaningless) use the [central baseline](#central-baseline).

<a id="valdef-dominant-baseline-central"></a>central  
<a id="ref-for-central-baseline①"></a>

Use the [central baselines](#central-baseline).

<a id="valdef-dominant-baseline-mathematical"></a>mathematical  
<a id="ref-for-math-baseline"></a>

Use the [math baselines](#math-baseline).

<a id="valdef-dominant-baseline-hanging"></a>hanging  
<a id="ref-for-hanging-baseline"></a>

Use the [hanging baselines](#hanging-baseline).

<a id="valdef-dominant-baseline-text-top"></a>text-top  
<a id="ref-for-text-over-baseline"></a>

Use the [text-over baselines](#text-over-baseline).

See [\[CSS-WRITING-MODES-3\]](#biblio-css-writing-modes-3) for an introduction to dominant baselines.

<a id="ref-for-valdef-dominant-baseline-central②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2ffa7534"></a> Define behavior for mixed vertical orientations that isn’t nonsensical when specified baseline isn’t [central](#valdef-dominant-baseline-central).

<a id="ref-for-propdef-vertical-align④"></a>

### <a id="transverse-alignment"></a>4.2.  Transverse Box Alignment: the [vertical-align](#propdef-vertical-align) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-vertical-align"></a>vertical-align                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-baseline-shift②"></a><a id="ref-for-propdef-alignment-baseline③"></a><a id="ref-for-comb-any"></a><a id="ref-for-comb-one⑧"></a>\[ first [\|](https://www.w3.org/TR/css-values-4/#comb-one) last\] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'alignment-baseline'\>](#propdef-alignment-baseline) <a id="ref-for-comb-any①"></a>\|\| [\<'baseline-shift'\>](#propdef-baseline-shift) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | baseline                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                         |



<a id="ref-for-shorthand-property"></a>

<a id="ref-for-alignment-baseline⑦"></a>

<a id="ref-for-propdef-alignment-baseline④"></a>

<a id="ref-for-baseline-alignment-preference"></a>

<a id="ref-for-propdef-baseline-source②"></a>

<a id="ref-for-post-alignment-shift①"></a>

<a id="ref-for-propdef-baseline-shift③"></a>

This [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) property specifies how an inline-level box is aligned within the line by specifying its [alignment baseline](#alignment-baseline) type ([alignment-baseline](#propdef-alignment-baseline)), [baseline alignment preference](#baseline-alignment-preference) ([baseline-source](#propdef-baseline-source)), and [post-alignment shift](#post-alignment-shift) ([baseline-shift](#propdef-baseline-shift)) in a single declaration.

<a id="ref-for-valdef-baseline-source-first"></a>

<a id="ref-for-valdef-baseline-source-last"></a>

<a id="ref-for-propdef-baseline-source③"></a>

<a id="ref-for-valdef-baseline-source-auto"></a>

If [first](#valdef-baseline-source-first) or [last](#valdef-baseline-source-last) is specified, it sets [baseline-source](#propdef-baseline-source) (which is otherwise reset to [auto](#valdef-baseline-source-auto)). Other values are as for the corresponding longhand properties, see below.

<a id="ref-for-propdef-vertical-align⑤"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should use this shorthand ([vertical-align](#propdef-vertical-align)) instead of its longhands, unless specifically needing to cascade its longhands independently or (on SVG elements) to support legacy SVG implementations.

<a id="ref-for-propdef-vertical-align⑥"></a>

<a id="ref-for-propdef-align-content①"></a>

<a id="ref-for-valdef-justify-content-normal"></a>

<a id="ref-for-valdef-baseline-shift-top"></a>

<a id="ref-for-propdef-baseline-shift④"></a>

<a id="ref-for-valdef-self-position-start"></a>

<a id="ref-for-valdef-baseline-shift-bottom"></a>

<a id="ref-for-valdef-self-position-end"></a>

<a id="ref-for-valdef-alignment-baseline-middle"></a>

<a id="ref-for-propdef-alignment-baseline⑤"></a>

<a id="ref-for-valdef-self-position-center"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [vertical-align](#propdef-vertical-align) can also affect the alignment of table cells when [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) is [normal](https://www.w3.org/TR/css-align-3/#valdef-justify-content-normal). Specifically, [top](#valdef-baseline-shift-top) ([baseline-shift: top](#propdef-baseline-shift)) maps it to [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start), [bottom](#valdef-baseline-shift-bottom) (<a id="ref-for-propdef-baseline-shift⑤"></a>baseline-shift: bottom) to [end](https://www.w3.org/TR/css-align-3/#valdef-self-position-end), and otherwise [middle](#valdef-alignment-baseline-middle) ([alignment-baseline: middle](#propdef-alignment-baseline)) to [center](https://www.w3.org/TR/css-align-3/#valdef-self-position-center). See [CSS Box Alignment 3 § 5.1.1 Block Containers (Including Table Cells)](https://www.w3.org/TR/css-align-3/#distribution-block).

<a id="ref-for-propdef-baseline-source④"></a>

#### <a id="baseline-source"></a>4.2.1.  Alignment Baseline Source: the [baseline-source](#propdef-baseline-source) longhand



| Field               | Definition                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-baseline-source"></a>baseline-source                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑨"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) first <a id="ref-for-comb-one①⓪"></a>\| last |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | inline-level boxes                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                    |



<a id="ref-for-first-baseline-set"></a>

<a id="ref-for-last-baseline-set"></a>

When an inline-level box has more than one possible source for baseline information (such as for a multi-line inline block or inline flex container) this property specifies whether the [first baseline set](https://www.w3.org/TR/css-align-3/#first-baseline-set) or [last baseline set](https://www.w3.org/TR/css-align-3/#last-baseline-set) is preferred for alignment, indicating the box’s <a id="baseline-alignment-preference"></a>baseline alignment preference. Values have the following meanings:

<a id="valdef-baseline-source-auto"></a>auto  
<a id="ref-for-first-baseline-alignment"></a>

<a id="ref-for-valdef-display-inline-block"></a>

<a id="ref-for-last-baseline-alignment"></a>

Specifies [last-baseline alignment](https://www.w3.org/TR/css-align-3/#last-baseline-alignment) for [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block), [first-baseline alignment](https://www.w3.org/TR/css-align-3/#first-baseline-alignment) for everything else.

<a id="valdef-baseline-source-first"></a>first  
<a id="ref-for-first-baseline-alignment①"></a>

Specifies [first-baseline alignment](https://www.w3.org/TR/css-align-3/#first-baseline-alignment).

<a id="valdef-baseline-source-last"></a>last  
<a id="ref-for-last-baseline-alignment①"></a>

Specifies [last-baseline alignment](https://www.w3.org/TR/css-align-3/#last-baseline-alignment).

<a id="ref-for-inline-box①⑤"></a>

See [CSS Box Alignment 3 § 9.1 Determining the Baselines of a Box](https://www.w3.org/TR/css-align-3/#baseline-export) for how to find the baselines of boxes other than [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box).

<a id="ref-for-propdef-alignment-baseline⑥"></a>

#### <a id="alignment-baseline-property"></a>4.2.2.  Alignment Baseline Type: the [alignment-baseline](#propdef-alignment-baseline) longhand



| Field               | Definition                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-alignment-baseline"></a>alignment-baseline                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①①"></a>baseline [\|](https://www.w3.org/TR/css-values-4/#comb-one) text-bottom <a id="ref-for-comb-one①②"></a>\| alphabetic <a id="ref-for-comb-one①③"></a>\| ideographic <a id="ref-for-comb-one①④"></a>\| middle <a id="ref-for-comb-one①⑤"></a>\| central <a id="ref-for-comb-one①⑥"></a>\| mathematical <a id="ref-for-comb-one①⑦"></a>\| text-top |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | baseline                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-TermTextContentElement①"></a>inline-level boxes, flex items, grid items, table cells, and SVG [text content elements](https://www.w3.org/TR/SVG2/text.html#TermTextContentElement)                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                   |



<a id="ref-for-baseline①③"></a>

<a id="ref-for-post-alignment-shift②"></a>

This property specifies the box’s <a id="alignment-baseline"></a>alignment baseline: the [baseline](#baseline) used to align the box prior to applying its [post-alignment shift](#post-alignment-shift) (if applicable).

Values are defined as follows:

<a id="valdef-alignment-baseline-baseline"></a>baseline  
<a id="ref-for-dominant-baseline⑥"></a>

Use the [dominant baseline](#dominant-baseline) choice of the parent.

<a id="valdef-alignment-baseline-text-bottom"></a>text-bottom  
<a id="ref-for-text-under-baseline①"></a>

Use the [text-under baseline](#text-under-baseline).

<a id="valdef-alignment-baseline-alphabetic"></a>alphabetic  
<a id="ref-for-alphabetic-baseline⑥"></a>

Use the [alphabetic baseline](#alphabetic-baseline).

<a id="valdef-alignment-baseline-ideographic"></a>ideographic  
<a id="ref-for-ideographic-under-baseline②"></a>

Use the [ideographic-under baseline](#ideographic-under-baseline).

<a id="valdef-alignment-baseline-middle"></a>middle  
<a id="ref-for-central-baseline②"></a>

<a id="ref-for-x-height-baseline②"></a>

<a id="ref-for-alphabetic-baseline⑦"></a>

<a id="ref-for-propdef-text-orientation③"></a>

<a id="ref-for-x-middle-baseline①"></a>

In general, use the [x-middle baselines](#x-middle-baseline); except under [text-orientation: upright](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) (where the [alphabetic](#alphabetic-baseline) and [x-height](#x-height-baseline) baselines are essentially meaningless) use the [central baseline](#central-baseline) instead.

<a id="valdef-alignment-baseline-central"></a>central  
<a id="ref-for-central-baseline③"></a>

Use the [central baseline](#central-baseline).

<a id="valdef-alignment-baseline-mathematical"></a>mathematical  
<a id="ref-for-math-baseline①"></a>

Use the [math baseline](#math-baseline).

<a id="valdef-alignment-baseline-text-top"></a>text-top  
<a id="ref-for-text-over-baseline①"></a>

Use the [text-over baseline](#text-over-baseline).

<a id="ref-for-baseline-alignment①"></a>

<a id="ref-for-baseline①④"></a>

<a id="ref-for-shared-alignment-context③"></a>

<a id="ref-for-inline-formatting-context⑧"></a>

<a id="ref-for-inline-level⑦"></a>

<a id="ref-for-box-fragment①"></a>

<a id="ref-for-inline-box①⑥"></a>

<a id="ref-for-inline-axis④"></a>

<a id="ref-for-formatting-context③"></a>

<a id="ref-for-TermCurrentTextPosition"></a>

When performing [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment), these values specify which [baseline](#baseline) of the box is aligned to the corresponding <a id="ref-for-baseline①⑤"></a>baseline of its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context). (In an [inline formatting context](#inline-formatting-context), [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment) and glyphs share an <a id="ref-for-shared-alignment-context④"></a>alignment context established by their parent [inline box](https://www.w3.org/TR/css-display-3/#inline-box) <a id="ref-for-box-fragment②"></a>fragment along its [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). For other [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context), see [CSS Box Alignment 3 § 9.2 Baseline Alignment Grouping](https://www.w3.org/TR/css-align-3/#baseline-terms).) In SVG text layout, these values instead specify the <a id="ref-for-baseline①⑥"></a>baseline that is aligned to the SVG [current text position](https://svgwg.org/svg2-draft/text.html#TermCurrentTextPosition).

##### <a id="alignment-baseline-svg-legacy"></a>4.2.2.1.  Legacy Values for SVG

SVG implementations <em>may</em> support the following aliases in order to support legacy content:

- <a id="ref-for-valdef-alignment-baseline-text-top"></a>

  <a id="valdef-alignment-baseline-text-before-edge"></a>text-before-edge aliasing [text-top](#valdef-alignment-baseline-text-top)

- <a id="ref-for-valdef-alignment-baseline-text-bottom"></a>

  <a id="valdef-alignment-baseline-text-after-edge"></a>text-after-edge aliasing [text-bottom](#valdef-alignment-baseline-text-bottom)

These values are not allowed in the [vertical-align](#propdef-vertical-align) shorthand.

<a id="ref-for-propdef-baseline-shift⑥"></a>

#### <a id="baseline-shift-property"></a>4.2.3.  Post-Alignment Shift: the [baseline-shift](#propdef-baseline-shift) longhand



| Field               | Definition                                                                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-baseline-shift"></a>baseline-shift                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑧"></a><a id="ref-for-typedef-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [\|](https://www.w3.org/TR/css-values-4/#comb-one) sub <a id="ref-for-comb-one①⑨"></a>\| super <a id="ref-for-comb-one②⓪"></a>\| top <a id="ref-for-comb-one②①"></a>\| center <a id="ref-for-comb-one②②"></a>\| bottom |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-TermTextContentElement②"></a>inline-level boxes and SVG [text content elements](https://www.w3.org/TR/SVG2/text.html#TermTextContentElement)                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-propdef-line-height④"></a>refer to the used value of [line-height](#propdef-line-height)                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>the specified keyword or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                              |



<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-valdef-baseline-shift-sub"></a>

<a id="ref-for-valdef-baseline-shift-super"></a>

<a id="ref-for-valdef-baseline-shift-top①"></a>

<a id="ref-for-valdef-baseline-shift-center"></a>

<a id="ref-for-valdef-baseline-shift-bottom①"></a>

<a id="ref-for-inline-box①⑦"></a>

<a id="ref-for-line-box②⑤"></a>

This property specifies the box’s <a id="post-alignment-shift"></a>post-alignment shift. The <a id="baseline-relative-shift-values"></a>baseline-relative shift values [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), [sub](#valdef-baseline-shift-sub), [super](#valdef-baseline-shift-super) shift the box relative to its baseline-aligned position, whereas the <a id="line-relative-shift-values"></a>line-relative shift values [top](#valdef-baseline-shift-top), [center](#valdef-baseline-shift-center), and [bottom](#valdef-baseline-shift-bottom) shift the [inline box](https://www.w3.org/TR/css-display-3/#inline-box) and its contents relative to the bounds of its [line box](#line-box).

<a id="ref-for-propdef-vertical-align⑧"></a>

<a id="ref-for-propdef-baseline-shift⑦"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> Authors should use the [vertical-align](#propdef-vertical-align) shorthand, which has existed since CSS1, instead of this [baseline-shift](#propdef-baseline-shift) longhand (except in SVG content, where conversely <a id="ref-for-propdef-baseline-shift⑧"></a>baseline-shift is more widely-supported in legacy user agents).

Values have the following meanings:

<a id="ref-for-length-value"></a>

<a id="valdef-baseline-shift-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Raise (positive value) or lower (negative value) by the specified length.

<a id="ref-for-percentage-value"></a>

<a id="valdef-baseline-shift-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-propdef-line-height⑤"></a>

Raise (positive value) or lower (negative value) by the specified percentage of the [line-height](#propdef-line-height).

<a id="valdef-baseline-shift-sub"></a>sub

<a id="ref-for-propdef-font-size"></a>

Lower by the offset appropriate for subscripts of the parent’s box. The UA may use the parent’s font metrics to find this offset; otherwise it defaults to dropping by one fifth of the parent’s used [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size).

<a id="valdef-baseline-shift-super"></a>super

<a id="ref-for-propdef-font-size①"></a>

Raise by the offset appropriate for superscripts of the parent’s box. The UA may use the parent’s font metrics to find this offset; otherwise it defaults to raising by one third of the parent’s used [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size).

<a id="valdef-baseline-shift-top"></a>top

<a id="ref-for-line-box②⑥"></a>

<a id="ref-for-aligned-subtree①"></a>

<a id="ref-for-line-over③"></a>

Align the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) edge of the [aligned subtree](#aligned-subtree) with the <a id="ref-for-line-over④"></a>line-over edge of the [line box](#line-box).

<a id="valdef-baseline-shift-center"></a>center

<a id="ref-for-line-box②⑦"></a>

<a id="ref-for-aligned-subtree②"></a>

Align the center of the [aligned subtree](#aligned-subtree) with the center of the [line box](#line-box).

<a id="valdef-baseline-shift-bottom"></a>bottom

<a id="ref-for-line-box②⑧"></a>

<a id="ref-for-aligned-subtree③"></a>

<a id="ref-for-line-under②"></a>

Align the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) edge of the [aligned subtree](#aligned-subtree) with the <a id="ref-for-line-under③"></a>line-under edge of the [line box](#line-box).

<a id="ref-for-inline-box①⑧"></a>

<a id="ref-for-layout-bounds②"></a>

<a id="ref-for-aligned-subtree④"></a>

<a id="ref-for-propdef-alignment-baseline⑦"></a>

<a id="ref-for-line-relative-shift-values③"></a>

<a id="ref-for-line-over⑤"></a>

<a id="ref-for-over"></a>

<a id="ref-for-line-under④"></a>

The <a id="aligned-subtree"></a>aligned subtree of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) contains the [layout bounds](#layout-bounds) of that box and the [aligned subtrees](#aligned-subtree) of all child <a id="ref-for-inline-box①⑨"></a>inline boxes whose computed [alignment-baseline](#propdef-alignment-baseline) value is not itself a [line-relative shift value](#line-relative-shift-values). The [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) edge of the <a id="ref-for-aligned-subtree⑤"></a>aligned subtree is the highest [over](https://www.w3.org/TR/css-writing-modes-4/#over) edge of the <a id="ref-for-layout-bounds③"></a>layout bounds in the subtree, and the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) edge is analogously the lowest.

<a id="ref-for-line-relative-shift-values④"></a>

<a id="ref-for-propdef-alignment-baseline⑧"></a>

<a id="ref-for-propdef-baseline-shift⑨"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8910737b"></a> The [line-relative shift values](#line-relative-shift-values) don’t fit perfectly in the dichotomy between [alignment-baseline](#propdef-alignment-baseline) and [baseline-shift](#propdef-baseline-shift). There’s [decent](https://github.com/w3c/csswg-drafts/issues/5180) [arguments](https://github.com/w3c/csswg-drafts/issues/5234) for either option. They’re currently drafted here, but if there’s a strong argument to move them, please file an issue for consideration.

##### <a id="baseline-shift-svg-legacy"></a>4.2.3.1.  Legacy Values for SVG

<a id="ref-for-propdef-vertical-align⑨"></a>

User agents <em>may</em> additionally support the keyword <a id="valdef-baseline-shift-baseline"></a>baseline as computing to 0 if is necessary for them to support legacy SVG content. This value is not allowed in the [vertical-align](#propdef-vertical-align) shorthand.

<a id="ref-for-valdef-baseline-shift-baseline"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6cbc9542"></a> We would prefer to remove the [baseline](#valdef-baseline-shift-baseline) value, and are looking for feedback from SVG user agents as to whether it’s necessary.

## <a id="line-height"></a>5.  Logical Heights and Inter-line Spacing

<a id="ref-for-block-axis⑥"></a>

<a id="ref-for-line-box②⑨"></a>

<a id="ref-for-inline-level⑧"></a>

<a id="ref-for-propdef-line-height⑥"></a>

<a id="ref-for-propdef-line-fit-edge④"></a>

The [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) sizing of a [line box](#line-box) depends on the sizes and [alignment](#alignment) of its [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) contents. This sizing is controlled by the [line-height](#propdef-line-height) and [line-fit-edge](#propdef-line-fit-edge) properties.

<a id="ref-for-propdef-line-height⑦"></a>

### <a id="line-height-property"></a>5.1.  Line Spacing: the [line-height](#propdef-line-height) property



| Field               | Definition                                                                                                                                                                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-line-height"></a>line-height                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage③"></a><a id="ref-for-number-value"></a><a id="ref-for-comb-one②③"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) <a id="ref-for-comb-one②④"></a>\| [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-TermTextContentElement③"></a>non-replaced inline boxes and SVG [text content elements](https://www.w3.org/TR/SVG2/text.html#TermTextContentElement)                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | computed relative to 1em                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a>the specified keyword, a number, or a computed [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                                         |



<a id="ref-for-layout-bounds④"></a>

<a id="ref-for-logical-height④"></a>

<a id="ref-for-line-box③⓪"></a>

This property specifies the box’s <a id="preferred-line-height"></a>preferred line height, which is used in calculating its “[layout bounds](#layout-bounds)”, i.e. its contribution to the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of its [line box](#line-box). (See [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height).)

<a id="ref-for-root-inline-box④"></a>

<a id="ref-for-block-container⑧"></a>

<a id="ref-for-propdef-line-height⑧"></a>

<a id="ref-for-line-box③①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because it applies to the [root inline box](#root-inline-box) when specified on a [block container](https://www.w3.org/TR/css-display-3/#block-container), [line-height](#propdef-line-height) effectively establishes the minimum height of the block’s [line boxes](#line-box).

Values for this property have the following meanings:

<a id="valdef-line-height-normal"></a>normal

<a id="ref-for-preferred-line-height"></a>

Determine the [preferred line height](#preferred-line-height) automatically based on font metrics.

<a id="ref-for-length-value②"></a>

<a id="valdef-line-height-length-0"></a>[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-preferred-line-height①"></a>

The specified length is used as the [preferred line height](#preferred-line-height). Negative values are illegal.

<a id="ref-for-number-value①"></a>

<a id="valdef-line-height-number-0"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-font-size②"></a>

<a id="ref-for-preferred-line-height②"></a>

The [preferred line height](#preferred-line-height) is this number multiplied by the element’s computed [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size). Negative values are illegal. The [computed value](https://www.w3.org/TR/CSS21/cascade.html#computed-value) is the same as the [specified value](https://www.w3.org/TR/CSS21/cascade.html#specified-value).

<a id="ref-for-percentage-value①"></a>

<a id="valdef-line-height-percentage-0"></a>[\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-propdef-font-size③"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-preferred-line-height③"></a>

The [preferred line height](#preferred-line-height) and [computed value](https://www.w3.org/TR/CSS21/cascade.html#computed-value) of the property is this percentage of the element’s computed [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size). Negative values are illegal.

<a id="ref-for-first-available-font①"></a>

<a id="ref-for-layout-bounds⑤"></a>

<a id="ref-for-inline-box②⓪"></a>

<a id="ref-for-propdef-line-height⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Metrics from fonts other than the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) only impact the [layout bounds](#layout-bounds) of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) with [line-height: normal](#propdef-line-height).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e366ece9"></a> The three rules in the example below have the same used line height:
>
> ```text
> div { line-height: 1.2; font-size: 10pt }     /* number */
> div { line-height: 1.2em; font-size: 10pt }   /* length */
> div { line-height: 120%; font-size: 10pt }    /* percentage */
> ```
>
> However, they inherit differently: the first one inherits as a number, which will lead to different line heights if descendants have different font sizes; the last two as inherit as absolute lengths, which will not be influenced by the font size on descendants.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-af10b7fe"></a> The fact that percentages compute to lengths is annoying. See also [Issue 3118](https://github.com/w3c/csswg-drafts/issues/3118) and [Issue 2165](https://github.com/w3c/csswg-drafts/issues/2165).

<a id="ref-for-propdef-line-fit-edge⑤"></a>

<a id="ref-for-valdef-line-fit-edge-leading"></a>

<a id="ref-for-inline-box②①"></a>

<a id="ref-for-propdef-line-height①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When [line-fit-edge](#propdef-line-fit-edge) is [leading](#valdef-line-fit-edge-leading), the margins, borders, and padding of [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) do not affect the line box’s height calculation. However, they are still rendered around these boxes. This means that if the size specified by [line-height](#propdef-line-height) is less than the size of the box, backgrounds and borders can “bleed” into adjoining line boxes, potentially obscuring earlier content.

<a id="ref-for-propdef-line-fit-edge⑥"></a>

### <a id="text-edges"></a>5.2.  Text Edge Metrics: the [line-fit-edge](#propdef-line-fit-edge) property



| Field               | Definition                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-line-fit-edge"></a>line-fit-edge                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-text-edge"></a><a id="ref-for-comb-one②⑤"></a>leading [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<text-edge\>](#typedef-text-edge) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | leading                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box②②"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box)                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified keyword                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                             |



> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6de82a32"></a> This is an early draft of a proposal, and might change significantly as design critiques and use cases are registered and various details and interactions with other properties are worked out. <strong>Do not ship (yet).</strong>

<a id="ref-for-inline-box②③"></a>

<a id="ref-for-block-axis⑦"></a>

<a id="ref-for-propdef-line-fit-edge⑦"></a>

<a id="ref-for-layout-bounds⑥"></a>

<a id="ref-for-root-inline-box⑤"></a>

<a id="ref-for-propdef-text-box-trim②"></a>

[Inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), whose primary purpose is to contain text, are sized in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) based on their font metrics. The [line-fit-edge](#propdef-line-fit-edge) property controls which metrics are used. These chosen metrics are used as the basis for the [layout bounds](#layout-bounds) of the <a id="ref-for-inline-box②④"></a>inline box (if it is not the [root inline box](#root-inline-box)); and also, by default, are the metrics used for [text-box-trim](#propdef-text-box-trim).

<a id="ref-for-typedef-text-edge①"></a>

The <a id="typedef-text-edge"></a>[\<text-edge\>](#typedef-text-edge) value, which identifies specific font metrics, expands to

<a id="ref-for-typedef-text-edge②"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-comb-one③⑤"></a>

```text
<text-edge> = [ text | ideographic | ideographic-ink ]
              | [ text | ideographic | ideographic-ink | cap | ex ]
                [ text | ideographic | ideographic-ink | alphabetic ]
```
<a id="ref-for-over①"></a>

<a id="ref-for-under"></a>

<a id="ref-for-valdef-line-fit-edge-text"></a>

The first value specifies the text [over](https://www.w3.org/TR/css-writing-modes-4/#over) edge; the second value specifies the text [under](https://www.w3.org/TR/css-writing-modes-4/#under) edge. If only one value is specified, both edges are assigned that same keyword if possible; else [text](#valdef-line-fit-edge-text) is assumed as the missing value.

<a id="ref-for-longhand"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-eba283e1"></a> Do we need [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) or is this shorthand enough? [\[Issue \#5236\]](https://github.com/w3c/csswg-drafts/issues/5236)

Values have the following meanings:

<a id="valdef-line-fit-edge-leading"></a>leading  
<a id="ref-for-line-box③②"></a>

<a id="ref-for-half-leading"></a>

<a id="ref-for-descent-metric②"></a>

<a id="ref-for-ascent-metric②"></a>

Use the [ascent](#ascent-metric)/[descent](#descent-metric) plus any positive [half-leading](#half-leading). Margin/padding/border is ignored for the purpose of sizing the [line box](#line-box).

<a id="valdef-line-fit-edge-text"></a>text  
<a id="ref-for-under①"></a>

<a id="ref-for-over②"></a>

<a id="ref-for-text-under-baseline②"></a>

<a id="ref-for-text-over-baseline②"></a>

Use the [text-over baseline](#text-over-baseline)/[text-under baseline](#text-under-baseline) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over)/[under](https://www.w3.org/TR/css-writing-modes-4/#under) edge.

<a id="valdef-line-fit-edge-cap"></a>cap  
<a id="ref-for-over③"></a>

<a id="ref-for-cap-height-baseline"></a>

Use the [cap-height baseline](#cap-height-baseline) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over) edge.

<a id="valdef-line-fit-edge-ex"></a>ex  
<a id="ref-for-over④"></a>

<a id="ref-for-x-height-baseline③"></a>

Use the [x-height baseline](#x-height-baseline) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over) edge.

<a id="valdef-line-fit-edge-ideographic"></a>ideographic  
<a id="ref-for-under②"></a>

<a id="ref-for-over⑤"></a>

<a id="ref-for-ideographic-under-baseline③"></a>

<a id="ref-for-ideographic-over-baseline①"></a>

Use the [ideographic-over baseline](#ideographic-over-baseline)/[ideographic-under baseline](#ideographic-under-baseline) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over)/[under](https://www.w3.org/TR/css-writing-modes-4/#under) edge.

<a id="valdef-line-fit-edge-ideographic-ink"></a>ideographic-ink  
<a id="ref-for-under③"></a>

<a id="ref-for-over⑥"></a>

<a id="ref-for-ideographic-ink-under-baseline"></a>

<a id="ref-for-ideographic-ink-over-baseline"></a>

Use the [ideographic-ink-over baseline](#ideographic-ink-over-baseline)/[ideographic-ink-under baseline](#ideographic-ink-under-baseline) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over)/[under](https://www.w3.org/TR/css-writing-modes-4/#under) edge.

<a id="valdef-line-fit-edge-alphabetic"></a>alphabetic  
<a id="ref-for-under④"></a>

<a id="ref-for-alphabetic-baseline⑧"></a>

Use the [alphabetic baseline](#alphabetic-baseline) as the [under](https://www.w3.org/TR/css-writing-modes-4/#under) edge.

<a id="ref-for-valdef-line-fit-edge-text①"></a>

<a id="ref-for-valdef-line-fit-edge-leading①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d656b674"></a> Is [text](#valdef-line-fit-edge-text) a reasonable name for the ascent/descent metrics, or can we think of something better? Ditto [leading](#valdef-line-fit-edge-leading) as a keyword. [\[Issue \#8067\]](https://github.com/w3c/csswg-drafts/issues/8067)

<a id="ref-for-propdef-line-fit-edge⑧"></a>

<a id="ref-for-valdef-line-fit-edge-leading②"></a>

<a id="ref-for-propdef-line-height①①"></a>

<a id="ref-for-layout-bounds⑦"></a>

Unless [line-fit-edge](#propdef-line-fit-edge) is [leading](#valdef-line-fit-edge-leading)—​in which case the box’s own [line-height](#propdef-line-height) is used to add spacing—​the box’s margin, padding, and border also contribute to the [layout bounds](#layout-bounds).

<a id="ref-for-valdef-line-fit-edge-leading③"></a>

<a id="ref-for-valdef-line-fit-edge-text②"></a>

<a id="ref-for-ascent-metric③"></a>

<a id="ref-for-descent-metric③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [leading](#valdef-line-fit-edge-leading) and [text](#valdef-line-fit-edge-text) values rely on the font [ascent](#ascent-metric) and [descent](#descent-metric) to make sure the text fits. Other values are more likely to result in overlap or overflow caused by ascents above the specified metrics (such as for diacritics), so authors using these values need to be careful to provide sufficient spacing for the text, particularly in multi-lingual contexts.

![Three different values of the line-fit-edge property.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/text-edge.png)

<a id="ref-for-propdef-line-fit-edge⑨"></a>

<a id="ref-for-valdef-line-fit-edge-leading④"></a>

<a id="ref-for-valdef-line-fit-edge-cap"></a>

<a id="ref-for-valdef-line-fit-edge-ex"></a>

The [line-fit-edge](#propdef-line-fit-edge) property, showing values for [leading](#valdef-line-fit-edge-leading), [cap](#valdef-line-fit-edge-cap), and [ex](#valdef-line-fit-edge-ex). The red lines indicate the layout bounds of the inline box.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5f46f8ae"></a> This illustration doesn’t match actual font metrics, it’s actually illustrating the cap-height, not the ascent. [\[Issue \#11364\]](https://github.com/w3c/csswg-drafts/issues/11364)

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-propdef-line-fit-edge①⓪"></a>
>
> <a id="ref-for-valdef-line-fit-edge-leading⑤"></a>
>
> When [line-fit-edge](#propdef-line-fit-edge) is [leading](#valdef-line-fit-edge-leading), vertical rhythm can be broken any time there is a change in font metrics or vertical alignment within a paragraph.
>
> Other values are more likely to give consistent line spacing—​as long as there is enough leading added that the half-leading on the root inline is large enough to accommodate the specified metrics of any descendants. The line box will still grow, however, to accommodate content that would otherwise overflow, to avoid overlap between lines.

<a id="ref-for-valdef-line-fit-edge-leading⑥"></a>

<a id="ref-for-half-leading①"></a>

<a id="ref-for-propdef-line-fit-edge①①"></a>

<a id="ref-for-margin⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although only [leading](#valdef-line-fit-edge-leading) applies positive [half-leading](#half-leading), in order to allow text to be set tightly, all values apply negative <a id="ref-for-half-leading②"></a>half-leading, see [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height). Half-leading is applied equally to both sides of the text; for more precise overlap control authors can use [line-fit-edge: text](#propdef-line-fit-edge) together with negative [margins](https://www.w3.org/TR/css-box-4/#margin) on the affected text.

### <a id="inline-height"></a>5.3.  Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes

<a id="ref-for-inline-box②⑤"></a>

<a id="ref-for-logical-height⑤"></a>

<a id="ref-for-line-box③③"></a>

<a id="ref-for-propdef-line-fit-edge①②"></a>

<a id="ref-for-propdef-line-height①②"></a>

<a id="ref-for-layout-bounds⑧"></a>

<a id="ref-for-propdef-inline-sizing"></a>

The contribution of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) to the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of its [line box](#line-box), here referred to as its <a id="layout-bounds"></a>layout bounds, is always calculated with respect to its own text metrics, as described below, and is controlled by [line-fit-edge](#propdef-line-fit-edge) and [line-height](#propdef-line-height). The sizes and positions of child boxes do not influence its [layout bounds](#layout-bounds) (nor its own <a id="ref-for-logical-height⑥"></a>logical height, for that matter, see [inline-sizing](#propdef-inline-sizing)).

<a id="ref-for-layout-bounds⑨"></a>

<a id="ref-for-box-box-edge"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [layout bounds](#layout-bounds) need not correspond to the box’s [edges](https://www.w3.org/TR/css-box-4/#box-box-edge).

<a id="ref-for-layout-bounds①⓪"></a>

<a id="ref-for-inline-box②⑥"></a>

<a id="ref-for-dominant-baseline⑦"></a>

<a id="ref-for-first-available-font②"></a>

To find the [layout bounds](#layout-bounds) of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box), the UA must first align all the glyphs <em>directly</em> contained in the <a id="ref-for-inline-box②⑦"></a>inline box to each other by their [dominant baselines](#dominant-baseline). (See [§ 3.3 Baselines of Glyphs and Boxes](#baseline-tables).) If the <a id="ref-for-inline-box②⑧"></a>inline box contains no glyphs at all, or if it contains only glyphs from fallback fonts, it is considered to contain a “strut” (an invisible glyph of zero width) with the metrics of the box’s [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font).

<a id="ref-for-baseline①⑦"></a>

<a id="ref-for-propdef-line-fit-edge①③"></a>

<a id="ref-for-ascent-metric④"></a>

<a id="ref-for-descent-metric④"></a>

<a id="ref-for-dominant-baseline⑧"></a>

<a id="ref-for-propdef-line-height①③"></a>

<a id="ref-for-valdef-line-height-normal①"></a>

<a id="ref-for-valdef-line-fit-edge-leading⑦"></a>

<a id="ref-for-root-inline-box⑥"></a>

<a id="ref-for-line-gap-metric②"></a>

<a id="ref-for-half-leading③"></a>

For each glyph (including the “strut”), <var>A</var> represents its ascent above the [baseline](#baseline); <var>D</var> represents its descent below. Unless [line-fit-edge](#propdef-line-fit-edge) specifies a different metric to use, <var>A</var> refers to the [ascent metric](#ascent-metric) (for the given font at its given size) and <var>D</var> to the [descent metric](#descent-metric), each adjusted to account for the [dominant baseline](#dominant-baseline)’s offset from zero. If [line-height](#propdef-line-height) computes to [normal](#valdef-line-height-normal) and either <a id="ref-for-propdef-line-fit-edge①④"></a>line-fit-edge is [leading](#valdef-line-fit-edge-leading) or this is the [root inline box](#root-inline-box), the font’s [line gap metric](#line-gap-metric) may also be incorporated into <var>A</var> and <var>D</var> by adding half to each side as [half-leading](#half-leading).

<a id="ref-for-propdef-line-height①④"></a>

<a id="ref-for-valdef-line-height-normal②"></a>

<a id="ref-for-layout-bounds①①"></a>

<strong>When its computed <a href="#propdef-line-height">line-height</a> is <a href="#valdef-line-height-normal">normal</a></strong>, the [layout bounds](#layout-bounds) of an inline box encloses all its glyphs, going from the highest <var>A</var> to the deepest <var>D</var>. (Note that glyphs in a single box can come from different fonts and thus might not all have the same <var>A</var> and <var>D</var>.)

<a id="ref-for-propdef-line-height①⑤"></a>

<a id="ref-for-valdef-line-height-normal③"></a>

<a id="ref-for-layout-bounds①②"></a>

<a id="ref-for-first-available-font③"></a>

<a id="ref-for-propdef-line-height①⑥"></a>

<a id="ref-for-leading"></a>

<a id="ref-for-propdef-line-fit-edge①⑤"></a>

<a id="ref-for-valdef-line-fit-edge-leading⑧"></a>

<a id="ref-for-root-inline-box⑦"></a>

<a id="ref-for-half-leading④"></a>

<strong>When its computed <a href="#propdef-line-height">line-height</a> is not <a href="#valdef-line-height-normal">normal</a></strong>, its [layout bounds](#layout-bounds) are derived solely from metrics of its [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) (ignoring glyphs from other fonts), and <a id="leading"></a>leading is used to adjust the effective <var>A</var> and <var>D</var> to add up to the used [line-height](#propdef-line-height). Calculate the [leading](#leading) <var>L</var> as <var>L</var> = <a id="ref-for-propdef-line-height①⑦"></a>line-height - (<var>A</var> + <var>D</var>). Half the <a id="ref-for-leading①"></a>leading (its <a id="half-leading"></a>half-leading) is added above <var>A</var> of the first available font, and the other half below <var>D</var> of the first available font, giving an effective ascent above the baseline of <var>A′</var> = <var>A</var> + <var>L</var>/2, and an effective descent of <var>D′</var> = <var>D</var> + <var>L</var>/2. However, if [line-fit-edge](#propdef-line-fit-edge) is not [leading](#valdef-line-fit-edge-leading) and this is not the [root inline box](#root-inline-box), if the [half-leading](#half-leading) is positive, treat it as zero. The <a id="ref-for-layout-bounds①③"></a>layout bounds exactly encloses this effective <var>A′</var> and <var>D′</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: <var>L</var> may be negative.

<a id="ref-for-propdef-line-fit-edge①⑥"></a>

<a id="ref-for-valdef-line-fit-edge-leading⑨"></a>

<a id="ref-for-layout-bounds①④"></a>

<a id="ref-for-margin⑥"></a>

<a id="ref-for-border⑤"></a>

<a id="ref-for-padding⑤"></a>

<a id="ref-for-inline-box②⑨"></a>

<a id="ref-for-inline-formatting-context⑨"></a>

Additionally, when [line-fit-edge](#propdef-line-fit-edge) is not [leading](#valdef-line-fit-edge-leading), the [layout bounds](#layout-bounds) are inflated by the sum of the [margin](https://www.w3.org/TR/css-box-4/#margin), [border](https://www.w3.org/TR/css-box-4/#border), and [padding](https://www.w3.org/TR/css-box-4/#padding) on each side. In order to allow negative <a id="ref-for-margin⑦"></a>margin values to have an actual effect, negative <a id="ref-for-margin⑧"></a>margins are also accumulated onto the <a id="ref-for-layout-bounds①⑤"></a>layout bounds of any descendant [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) participating in the same [inline formatting context](#inline-formatting-context).

<a id="ref-for-inline-box③⓪"></a>

<a id="ref-for-box-fragment③"></a>

<a id="ref-for-preserved-white-space①"></a>

<a id="ref-for-line-box③④"></a>

In Quirks Mode [\[QUIRKS\]](#biblio-quirks), any [inline box](https://www.w3.org/TR/css-display-3/#inline-box) [fragment](https://www.w3.org/TR/css-break-4/#box-fragment) that has zero borders and padding and that does not directly contain text or [preserved white space](https://www.w3.org/TR/css-text-4/#preserved-white-space) [\[CSS-TEXT-3\]](#biblio-css-text-3) is ignored when sizing the [line box](#line-box).

## <a id="leading-trim"></a>6.  Trimming Leading Over/Under Text

<a id="ref-for-propdef-line-height①⑧"></a>

To ensure consistent spacing in the basic case of running text, CSS line layout introduces leading both above and below the text content of each line as needed to ensure its [line-height](#propdef-line-height). In addition, the ascent and descent font metrics themselves often include extra space above and below the most typical glyph shapes in order to accommodate occasional characters and diacritics that ascend or descend beyond the typical bounds. This prevents adjacent lines of text from overlapping each other. However, all this extra spacing interferes with visual alignment and with control over effective (visually-apparent) spacing.

<a id="ref-for-propdef-text-box"></a>

The [text-box](#propdef-text-box) property allows trimming this additional space above and below the first and last lines of a block, allowing more precise control over spacing around the glyphs. By relying on font metrics rather than hard-coded lengths, this feature allows content to be resized, rewrapped, and rendered in a variety of fonts while maintaining that precise spacing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-306266d4"></a>
>
> A common problem is vertical centering. It’s easy to vertically center the text container to an icon, but because the visual boundaries of Latin text are the cap height and the alphabetic baseline, rather than the ascent and descent, this often doesn’t yield the intended visual effect.
>
> ![Consider some Latin text placed to the right of an image, to be centered between its top and bottom. Measuring from the top of the image to the top of the text box yields 13px; likewise measuring from the bottom of the image to the bottom of the text box yields 13px, theoretically perfectly centering the text. However, measuring from the top of the image to the cap-height yields 21px; and measuring from the bottom to the alphabetic baseline yields 19px, showing that visually the text is not actually centered.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/leading-trim-centering-fail.png)
>
> Measuring to the top/bottom of the text may yield equal results, but measuring to the visual bounds shows that it is not visually centered.
>
> To center the text visually, it’s necessary to assume the cap height and alphabetic baseline as the top and bottom edges of the text, respectively.
>
> ![If the text were visually centered, the distance between the top of the image and the cap height would be 20px, and the distance between the bottom of the image and the alphabetic baseline would be equally 20px.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/leading-trim-centering-goal.png)
>
> Measuring to the cap height / alphabetic baseline instead of the ascent / descent and equalizing those distances visually centers the text.
>
> <a id="ref-for-propdef-text-box-trim③"></a>
>
> By using [text-box-trim](#propdef-text-box-trim) to strip out the spacing above the cap height and below the alphabetic baseline, centering the box actually centers the text; and does so reliably, regardless of what font is used to render it.
>
> ![](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/leading-trim-centering-variants.gif)
>
> Even though different fonts have different cap heights, by using the font’s metric rather than a magic number, the layout intention is met even as the font is changed.

<a id="ref-for-propdef-text-box①"></a>

### <a id="text-box-shorthand"></a>6.1.  Shorthand for Text Box Trimming: the [text-box](#propdef-text-box) property



| Field               | Definition                                                                                                                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-box"></a>text-box                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-text-box-edge①"></a><a id="ref-for-comb-any②"></a><a id="ref-for-propdef-text-box-trim④"></a><a id="ref-for-comb-one③⑥"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<'text-box-trim'\>](#propdef-text-box-trim) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'text-box-edge'\>](#propdef-text-box-edge) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box③①"></a><a id="ref-for-block-container⑨"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container) and [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box)                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified keyword                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                               |



<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-text-box-trim⑤"></a>

<a id="ref-for-propdef-text-box-edge②"></a>

This property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting the [text-box-trim](#propdef-text-box-trim) and [text-box-edge](#propdef-text-box-edge) properties in a single declaration.

<a id="ref-for-propdef-text-box-trim⑥"></a>

<a id="ref-for-valdef-text-box-trim-none"></a>

<a id="ref-for-propdef-text-box-edge③"></a>

<a id="ref-for-valdef-text-box-edge-auto"></a>

If the single keyword <a id="valdef-text-box-normal"></a>normal is specified, it sets [text-box-trim](#propdef-text-box-trim) to [none](#valdef-text-box-trim-none) and [text-box-edge](#propdef-text-box-edge) to [auto](#valdef-text-box-edge-auto). Otherwise, omitting the <a id="ref-for-propdef-text-box-trim⑦"></a>text-box-trim value sets it to both (not the initial value), while omitting the <a id="ref-for-propdef-text-box-edge④"></a>text-box-edge value sets it to <a id="ref-for-valdef-text-box-edge-auto①"></a>auto (the initial value).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6e028676"></a> Add examples.

<a id="ref-for-propdef-text-box-trim⑧"></a>

### <a id="text-box-trim"></a>6.2.  Trimming Over/Under Text: the [text-box-trim](#propdef-text-box-trim) property



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-box-trim"></a>text-box-trim                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③⑦"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) trim-start <a id="ref-for-comb-one③⑧"></a>\| trim-end <a id="ref-for-comb-one③⑨"></a>\| trim-both |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box③②"></a>block containers and [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box)                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified keyword                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                             |



<a id="ref-for-inline-box③③"></a>

<a id="ref-for-x10③"></a>

<a id="ref-for-propdef-text-box-edge⑤"></a>

On [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), specifies whether to trim the [content box](https://www.w3.org/TR/CSS21/box.html#x10) to match the specified [text-box-edge](#propdef-text-box-edge) metric. See [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height) for details.

<a id="ref-for-block-container①⓪"></a>

<a id="ref-for-multi-column-container"></a>

<a id="ref-for-half-leading⑤"></a>

<a id="ref-for-content-edge①"></a>

<a id="ref-for-propdef-text-box-edge⑥"></a>

<a id="ref-for-line-box③⑤"></a>

<a id="ref-for-containing-block⑤"></a>

On [block containers](https://www.w3.org/TR/css-display-3/#block-container), as well as on each column of a [multi-column container](https://www.w3.org/TR/css-multicol-1/#multi-column-container), specifies whether to trim [half-leading](#half-leading) at the start/end of the box’s content to better match its [content edge](https://www.w3.org/TR/css-box-3/#content-edge) to its text content. The trimming edge in this case is specified by the start/end [text-box-edge](#propdef-text-box-edge) value of the affected [line box](#line-box)’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block).

Values have the following meanings:

<a id="valdef-text-box-trim-none"></a>none  
<a id="ref-for-block-container①①"></a>

<a id="ref-for-line-box③⑥"></a>

No special handling of the first/last [line box](#line-box) when applied to a [block container](https://www.w3.org/TR/css-display-3/#block-container).

<a id="ref-for-inline-box③④"></a>

<a id="ref-for-content-edge②"></a>

<a id="ref-for-text-over-baseline③"></a>

<a id="ref-for-text-under-baseline③"></a>

<a id="ref-for-propdef-text-box-edge⑦"></a>

When applied to an [inline box](https://www.w3.org/TR/css-display-3/#inline-box), specifies that the over/under [content edges](https://www.w3.org/TR/css-box-3/#content-edge) coincide with the [text-over](#text-over-baseline)/[text-under](#text-under-baseline) baselines regardless of [text-box-edge](#propdef-text-box-edge).

<a id="valdef-text-box-trim-trim-start"></a>trim-start  
<a id="ref-for-root-inline-box⑧"></a>

<a id="ref-for-first-formatted-line①"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-column-box①"></a>

<a id="ref-for-block-container①②"></a>

For [block containers](https://www.w3.org/TR/css-display-3/#block-container) and [column boxes](https://www.w3.org/TR/css-multicol-1/#column-box): trim the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) side of the [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) to the specified metric of its [root inline box](#root-inline-box). If there is no such line, or if there is intervening non-zero padding or borders, there is no effect.

<a id="ref-for-inline-box③⑤"></a>

<a id="ref-for-block-start①"></a>

<a id="ref-for-content-edge③"></a>

<a id="ref-for-propdef-text-box-edge⑧"></a>

For [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box): trims the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) side of the box to match its [content edge](https://www.w3.org/TR/css-box-3/#content-edge) to the metric specified by [text-box-edge](#propdef-text-box-edge).

<a id="valdef-text-box-trim-trim-end"></a>trim-end  
<a id="ref-for-root-inline-box⑨"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-column-box②"></a>

<a id="ref-for-block-container①③"></a>

For [block containers](https://www.w3.org/TR/css-display-3/#block-container) and [column boxes](https://www.w3.org/TR/css-multicol-1/#column-box): trim the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) side of the last formatted line to the specified metric of its [root inline box](#root-inline-box). If there is no such line, or if there is intervening non-zero padding or borders, there is no effect.

<a id="ref-for-inline-box③⑥"></a>

<a id="ref-for-block-end①"></a>

<a id="ref-for-content-edge④"></a>

<a id="ref-for-propdef-text-box-edge⑨"></a>

For [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box): trims the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) side of the box to match its [content edge](https://www.w3.org/TR/css-box-3/#content-edge) to the metric specified by [text-box-edge](#propdef-text-box-edge).

<a id="valdef-text-box-trim-trim-both"></a>trim-both  
<a id="ref-for-valdef-text-box-trim-trim-end"></a>

<a id="ref-for-valdef-text-box-trim-trim-start"></a>

Specifies the behavior of [trim-start](#valdef-text-box-trim-trim-start) and [trim-end](#valdef-text-box-trim-trim-end) simultaneously.

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-formatting-context④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Like [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line), this property does not apply to, or propagate through, flex, grid, or table [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context).

<a id="ref-for-block-end②"></a>

<a id="ref-for-line-under⑤"></a>

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) side does not coincide with the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) side when [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) is [vertical-lr](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-lr).

<a id="ref-for-line-box③⑦"></a>

<a id="ref-for-block-container①④"></a>

If multiple ancestors specify trimming on the same [line box](#line-box), the metric used is that of the innermost [block container](https://www.w3.org/TR/css-display-3/#block-container) that requests trimming on that side of the <a id="ref-for-line-box③⑧"></a>line box.

<a id="ref-for-propdef-text-box-trim⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Content and ink overflowing a box due to non-initial values of [text-box-trim](#propdef-text-box-trim) is handled the same as content that would overflow the box or line box otherwise.

<a id="ref-for-selectordef-first-line①"></a>

<a id="ref-for-multi-column-container①"></a>

Unlike [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line), when applying to the first (or last) formatted line of a [multi-column container](https://www.w3.org/TR/css-multicol-1/#multi-column-container), this property applies to the first (or last) formatted lines of <em>every</em> column in the <a id="ref-for-multi-column-container②"></a>multi-column container.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d1836b7a"></a> What happens if the column is split by a spanner? [\[Issue \#11363\]](https://github.com/w3c/csswg-drafts/issues/11363)

<a id="ref-for-propdef-text-box-trim①⓪"></a>

<a id="ref-for-fragmentation"></a>

<a id="ref-for-propdef-box-decoration-break"></a>

When the box to which [text-box-trim](#propdef-text-box-trim) has been applied is split by [fragmentation](https://www.w3.org/TR/css-break-4/#fragmentation) [\[CSS-BREAK-3\]](#biblio-css-break-3), whether trimming is applied per fragment or only to the start/end edges of its first/last fragments is determined by [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break).

<a id="ref-for-line-box③⑨"></a>

<a id="ref-for-propdef-text-box-trim①①"></a>

If, when printing, trimming a [line box](#line-box) would cause its content to be clipped, the UA may ignore [text-box-trim](#propdef-text-box-trim) on that edge of that <a id="ref-for-line-box④⓪"></a>line box.

<a id="ref-for-propdef-text-box-edge①⓪"></a>

### <a id="text-box-edge"></a>6.3.  Text Trimming Metrics: the [text-box-edge](#propdef-text-box-edge) property



| Field               | Definition                                                                                                                                                                         |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-box-edge"></a>text-box-edge                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-text-edge③"></a><a id="ref-for-comb-one④⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<text-edge\>](#typedef-text-edge)                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box③⑦"></a><a id="ref-for-block-container①⑤"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container) and [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the specified keyword                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                           |



<a id="ref-for-propdef-text-box-trim①②"></a>

<a id="ref-for-propdef-line-fit-edge①⑦"></a>

<a id="ref-for-valdef-line-fit-edge-leading①⓪"></a>

<a id="ref-for-x1"></a>

<a id="ref-for-valdef-line-fit-edge-text③"></a>

This property specifies the metrics to use for [text-box-trim](#propdef-text-box-trim) effects. Values have the same meanings as for [line-fit-edge](#propdef-line-fit-edge); the <a id="valdef-text-box-edge-auto"></a>auto keyword uses the value of <a id="ref-for-propdef-line-fit-edge①⑧"></a>line-fit-edge, interpreting [leading](#valdef-line-fit-edge-leading) (the [initial value](https://www.w3.org/TR/CSS21/cascade.html#x1)) as [text](#valdef-line-fit-edge-text).

<a id="ref-for-propdef-text-box-trim①③"></a>

<a id="ref-for-propdef-text-box②"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-line-fit-edge①⑨"></a>

<a id="ref-for-x1①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property can be set together with [text-box-trim](#propdef-text-box-trim) in the [text-box](#propdef-text-box) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property). Unlike [line-fit-edge](#propdef-line-fit-edge), it does not inherit; however its [initial value](https://www.w3.org/TR/CSS21/cascade.html#x1) copies from <a id="ref-for-propdef-line-fit-edge②⓪"></a>line-fit-edge, which does inherit.

<a id="ref-for-propdef-inline-sizing①"></a>

### <a id="line-fill"></a>6.4.  Inline Box Drawing Height: the [inline-sizing](#propdef-inline-sizing) property



| Field               | Definition                                                                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-inline-sizing"></a>inline-sizing                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④①"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) stretch                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-internal-ruby-boxes"></a><a id="ref-for-ruby-container"></a><a id="ref-for-inline-box③⑧"></a>[inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), but not [ruby container boxes](https://www.w3.org/TR/css-ruby-1/#ruby-container) nor [internal ruby boxes](https://www.w3.org/TR/css-ruby-1/#internal-ruby-boxes) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                    |



<a id="ref-for-propdef-text-box-trim①④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2c8f414b"></a> This has a confusing name. We need a new name. Alternatively, incorporate this into [text-box-trim](#propdef-text-box-trim)? [\[Issue \#5189\]](https://github.com/w3c/csswg-drafts/issues/5189)

<a id="ref-for-logical-height⑦"></a>

<a id="ref-for-content-area"></a>

<a id="ref-for-inline-box③⑨"></a>

This property specifies how the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of the [content area](https://www.w3.org/TR/css-box-4/#content-area) of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) is measured in relation to its contents. It has no effect on the size or position of the box’s contents, the line box, or any other content.

Values have the following meanings:

<a id="valdef-inline-sizing-normal"></a>normal  
<a id="ref-for-propdef-text-box-trim①⑤"></a>

<a id="ref-for-first-available-font④"></a>

<a id="ref-for-inline-box④⓪"></a>

<a id="ref-for-content-area①"></a>

The [content area](https://www.w3.org/TR/css-box-4/#content-area) of the [inline box](https://www.w3.org/TR/css-display-3/#inline-box) is sized and positioned to fit (possibly hypothetical) text from its [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font). If [text-box-trim](#propdef-text-box-trim) indicates trimming, then the specified metric must be used. Otherwise, this specification does not specify how. A UA may, e.g., use the maximum ascender and descender of the font. (This would ensure that glyphs with parts above or below the em-box still fall within the content area, but leads to differently sized boxes for different fonts.)

<a id="ref-for-logical-height⑧"></a>

<a id="ref-for-content-area②"></a>

<a id="ref-for-first-available-font⑤"></a>

<a id="ref-for-line-box④①"></a>

<a id="ref-for-propdef-line-height①⑨"></a>

<a id="ref-for-valdef-line-height-normal④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If more than one font is used (which happen when glyphs are found in different fonts), the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of the [content area](https://www.w3.org/TR/css-box-4/#content-area) is not affected by the glyphs from the fallback fonts, and only depends on the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font). However, these fallback glyphs can still affect the [line box](#line-box) size when [line-height](#propdef-line-height) is [normal](#valdef-line-height-normal); see [§ 5.3 Calculating the Logical Height Contributions (“Layout Bounds”) of Inline Boxes](#inline-height).

<a id="valdef-inline-sizing-stretch"></a>stretch  
<a id="ref-for-in-flow③"></a>

<a id="ref-for-outer-size"></a>

<a id="ref-for-block-axis⑧"></a>

<a id="ref-for-logical-height⑨"></a>

<a id="ref-for-inner-size①"></a>

<a id="ref-for-margin-edge"></a>

<a id="ref-for-under⑤"></a>

<a id="ref-for-over⑦"></a>

<a id="ref-for-box-box-edge①"></a>

<a id="ref-for-inline-box④①"></a>

<a id="ref-for-valdef-inline-sizing-normal"></a>

<a id="ref-for-line-box④②"></a>

Once the [line box](#line-box) has been sized and its contents positioned as for [normal](#valdef-inline-sizing-normal), the [inline box](https://www.w3.org/TR/css-display-3/#inline-box)’s [box edges](https://www.w3.org/TR/css-box-4/#box-box-edge) are shifted such that its [over](https://www.w3.org/TR/css-writing-modes-4/#over)/[under](https://www.w3.org/TR/css-writing-modes-4/#under) [margin edges](https://www.w3.org/TR/css-box-3/#margin-edge) coincide with the corresponding <a id="ref-for-line-box④③"></a>line box’s edges, stretching the <a id="ref-for-inline-box④②"></a>inline box’s [inner](https://www.w3.org/TR/css-sizing-3/#inner-size) [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) so that its [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) [outer size](https://www.w3.org/TR/css-sizing-3/#outer-size) fills the <a id="ref-for-line-box④④"></a>line box. (The sizes and positions of its [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) contents are not affected.)

<a id="ref-for-propdef-height"></a>

<a id="ref-for-inline-box④③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) property does not apply to [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box).

<a id="ref-for-propdef-line-height②⓪"></a>

<a id="ref-for-inline-box④④"></a>

<a id="ref-for-logical-height①⓪"></a>

<a id="ref-for-line-box④⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [line-height](#propdef-line-height) has no impact on the size of an [inline box](https://www.w3.org/TR/css-display-3/#inline-box), it only [affects its contribution](#inline-height) to the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of its [line box](#line-box).

## <a id="initial-letter-styling"></a>7.  Initial Letters

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-81b8cb9c"></a>The editors would appreciate any examples of drop initials in non-western scripts, especially Indic scripts.

### <a id="initial-letter-intro"></a>7.1.  An Introduction to Initial Letters

<em>This section is non-normative.</em>

Large, decorative letters have been used to start new sections of text since before the invention of printing. In fact, their use predates lowercase letters entirely.

#### <a id="drop-initial"></a>7.1.1.  Drop Initial

A <a id="dropped-initial"></a>dropped initial (or “drop cap”) is a larger-than-usual letter at the start of a paragraph, with a baseline at least one line lower than the first baseline of the paragraph. The size of the drop initial is usually indicated by how many lines it occupies. Two- and three-line drop initials are very common.

![3-line drop cap with E Acute](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Dropcap-E-acute-3line.png)

Three-line drop initial with E acute. Since the cap-height of the drop initial aligns with the cap-height of the main text, the accent extends above the paragraph.

<a id="ref-for-dropped-initial"></a>

The exact size and position of a [dropped initial](#dropped-initial) depends on the alignment of its glyph. Reference points on the drop cap must align precisely with reference points in the text. The alignment constraints for drop initials depend on the writing system.

In Western scripts, the top reference points are the cap height of the initial letter and of the first line of text. The bottom reference points are the alphabetic baseline of the initial letter and the baseline of the Nth line of text. The figure below shows a simple two-line drop cap, with the relevant reference lines marked.

![drop cap showing alignment](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Dropcap-lines.png)

Two-line drop cap showing baselines (green lines), cap-height (red line), and ascender (cyan line).

<a id="ref-for-block-start②"></a>

<a id="ref-for-block-end③"></a>

In Han-derived scripts, the initial letter extends from the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge of the glyphs on the first line to the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge of the glyphs on the Nth line.

![Japanese Vertical Initial](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Initial-2line-JapaneseVertical.png)

Two-line drop initial in vertical writing mode

In certain Indic scripts, the top alignment point is the hanging baseline, and the bottom alignment point is the text-after-edge.

![Devanagari initial letter](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Devangari-Initial.png)

<a id="ref-for-initial-letter①"></a>

Devanagari [initial letter](#initial-letter) aligned with hanging baseline. Alignment points shown in red.

#### <a id="sunk-initial"></a>7.1.2.  Sunken Initial Letters

Some styles of drop initials do not align with the first line of text. A <a id="sunken-initial"></a>sunken initial (or “sunken cap”) both sinks below the first baseline, and extends above the first line of text.

![sunken drop initial](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/SunkenCapA.png)

Sunken cap. The letter drops two lines, but is the size of a three-line initial letter.

#### <a id="raise-initial"></a>7.1.3.  Raised Initial Letters

A <a id="raised-initial"></a>raised initial (often called a “raised cap” or “stick-up cap”) “sinks” to the first text baseline.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A proper raised initial has several advantages over simply increasing the font size of a first letter. The line spacing in the rest of the paragraph will not be altered, but text will still be excluded around large descenders. And if the size of raised initial is defined to be an integral number of lines, implicit baseline grids can be maintained.

![raised cap](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/RaisedCap.png)

Raised cap. The initial letter is the size of a 3-line initial, but does not drop.

### <a id="selecting-drop-initials"></a>7.2.  Selecting Initial Letters

<em>This section is non-normative.</em>

<a id="ref-for-selectordef-first-letter"></a>

<a id="ref-for-initial-letter②"></a>

Initial letters are typically a single letter, although they may include punctuation or a sequence of characters which are perceived by the user to be a single typographic unit. The [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-element, defined in [\[SELECT\]](#biblio-select) and [\[CSS-PSEUDO-4\]](#biblio-css-pseudo-4), can be used to select the character(s) to be formatted as [initial letters](#initial-letter).

<a id="ref-for-propdef-initial-letter"></a>

Authors who need more control over which characters are included in an initial letter, or who want to apply initial-letter formatting to replaced elements or multiple words can alternately apply the [initial-letter](#propdef-initial-letter) property to the first inline-level child of a block container.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f8ada747"></a>
>
> ```text
> <p>This paragraph has a dropped “T”.
> <p><img alt="H" src="illuminated-h.svg">ere we have an illuminated “H”.
> <p><span>Words may also</span> be given initial letter styling at the beginning of a paragraph.
> ```
>
> ```text
> ::first-letter, /* style first paragraph’s T */
> img, /* style illuminated H */
> span /* style phrase inside span */
> { initial-letter: 2; }
> ```
<a id="ref-for-selectordef-first-letter①"></a>

<a id="ref-for-initial-letter③"></a>

Note that since [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) selects punctuation before or after the first letter, these characters are included in the [initial letter](#initial-letter) when <a id="ref-for-selectordef-first-letter②"></a>::first-letter is used.

![Paragraph showing both opening quote and first letter set as three-line drop cap](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/initial-letter-punctuation-quote.png)

<a id="ref-for-selectordef-first-letter③"></a>

The [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-element selects the quotation mark as well as the “M”.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8e10cdbe"></a> Should there be a way to opt out of this behavior? See [GitHub Issue 310](https://github.com/w3c/csswg-drafts/issues/310).

<a id="ref-for-propdef-initial-letter①"></a>

### <a id="sizing-drop-initials"></a>7.3.  Creating Initial Letters: the [initial-letter](#propdef-initial-letter) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-initial-letter"></a>initial-letter                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-comb-all"></a><a id="ref-for-integer-value"></a><a id="ref-for-number-value②"></a><a id="ref-for-comb-one④②"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) [\<integer \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value) <a id="ref-for-comb-one④③"></a>\| <a id="ref-for-number-value③"></a>\<number \[1,∞\]\> [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) \[ drop <a id="ref-for-comb-one④④"></a>\| raise \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | certain inline-level boxes and ::first-letter and inside ::marker boxes ([see prose](#first-most-inline-level))                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-initial-letter-normal"></a>the keyword [normal](#valdef-initial-letter-normal) or a number paired with an integer                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |



<a id="ref-for-initial-letter-initial-letter-size"></a>

<a id="ref-for-initial-letter-initial-letter-sink"></a>

This property specifies the [size](#initial-letter-initial-letter-size) and [sink](#initial-letter-initial-letter-sink) for dropped, raised, and sunken initial letters as the number of lines spanned.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aaaee605"></a> For example, the following code will create a 2-line dropped initial letter at the beginning of each paragraph that immediately follows a second-level heading:
>
> ```text
> h2 + p::first-letter { initial-letter: 2; }
> ```
It takes the following values:

<a id="valdef-initial-letter-normal"></a>normal

No special initial letter effect. Text behaves as normal.

<a id="ref-for-number-value④"></a>

<a id="valdef-initial-letter-number-1"></a>[\<number \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

This first argument defines the <a id="initial-letter-initial-letter-size"></a>size of the initial letter in terms of how many lines it occupies. Values less than one are invalid.

<a id="ref-for-integer-value①"></a>

<a id="valdef-initial-letter-integer-1"></a>[\<integer \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value)

<a id="ref-for-sunken-initial"></a>

<a id="ref-for-raised-initial"></a>

This optional second argument defines the number of lines the initial letter should <a id="initial-letter-initial-letter-sink"></a>sink. A value of 1 indicates a [raised initial](#raised-initial); values greater than 1 indicate a [sunken initial](#sunken-initial). Values less than one are invalid.

<a id="valdef-initial-letter-raise"></a>raise

<a id="ref-for-initial-letter-initial-letter-sink①"></a>

Computes to an [initial letter sink](#initial-letter-initial-letter-sink) of 1.

<a id="valdef-initial-letter-drop"></a>drop

<a id="ref-for-initial-letter-initial-letter-size①"></a>

<a id="ref-for-initial-letter-initial-letter-sink②"></a>

Computes to an [initial letter sink](#initial-letter-initial-letter-sink) equal to the [initial letter size](#initial-letter-initial-letter-size) floored to the nearest positive whole number.

<a id="ref-for-initial-letter-initial-letter-sink③"></a>

<a id="ref-for-valdef-initial-letter-drop"></a>

If the [initial letter sink](#initial-letter-initial-letter-sink) value is omitted, [drop](#valdef-initial-letter-drop) is assumed.

<a id="ref-for-valdef-initial-letter-normal①"></a>

<a id="ref-for-in-flow④"></a>

<a id="ref-for-inline-level-box①②"></a>

Values other than [normal](#valdef-initial-letter-normal) cause the affected box to become an <a id="initial-letter"></a>initial letter box, which is an [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) [inline-level box](https://www.w3.org/TR/css-display-3/#inline-level-box) with special layout behavior.

<a id="ref-for-propdef-initial-letter②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-68b34294"></a> Here are some examples of [initial-letter](#propdef-initial-letter) usage:
>
> <a id="ref-for-propdef-initial-letter③"></a>
>
> [initial-letter: 3](#propdef-initial-letter)
>
> <a id="ref-for-propdef-initial-letter④"></a>
>
> [initial-letter: 3 3](#propdef-initial-letter)
>
> <a id="ref-for-propdef-initial-letter⑤"></a>
>
> [initial-letter: 3 drop](#propdef-initial-letter)
>
> <a id="ref-for-propdef-initial-letter⑥"></a>
>
> [initial-letter: drop 3](#propdef-initial-letter)
>
> <a id="ref-for-dropped-initial①"></a>
>
> Represents a [dropped initial](#dropped-initial) 3 lines high, 3 lines deep.
>
> ![3 lines high, 3 lines deep](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/InitialLetter33.png)
>
> <a id="ref-for-propdef-initial-letter⑦"></a>
>
> [initial-letter: 3 2](#propdef-initial-letter)
>
> <a id="ref-for-sunken-initial①"></a>
>
> Represents a [sunken initial](#sunken-initial) 3 lines high, 2 lines deep.
>
> ![3 lines high, 2 lines deep](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/InitialLetter32.png)
>
> <a id="ref-for-propdef-initial-letter⑧"></a>
>
> [initial-letter: 3 1](#propdef-initial-letter)
>
> <a id="ref-for-propdef-initial-letter⑨"></a>
>
> [initial-letter: 3 raise](#propdef-initial-letter)
>
> <a id="ref-for-propdef-initial-letter①⓪"></a>
>
> [initial-letter: raise 3](#propdef-initial-letter)
>
> <a id="ref-for-raised-initial①"></a>
>
> Represents a [raised initial](#raised-initial) 3 lines high, 1 line deep.
>
> ![3 lines high, 1 line deep](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/InitialLetter31.png)
>
> <a id="ref-for-propdef-initial-letter①①"></a>
>
> [initial-letter: 2.51 3](#propdef-initial-letter)
>
> The size of the initial letter does not have to be an integral number of lines. In this case only the top aligns.
>
> ![Non-integral initial letter that only aligns at base](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/non-integer-initial.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-80ced456"></a> In conjunction with other CSS properties, initial-letter can be used to create “adjacent initial letters,” where the initial letter is adjacent to the text:
>
> ```text
> p::first-letter {
>   initial-letter: 3;
>   color: red;
>   width: 5em;
>   text-align: right;
>   margin-left: -5em;
> }
> 
> p {
>   margin-left: 5em;
> }
> ```
>
> ![Initial letter adjacent to text](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/adjacent-initial-letter.png)

#### <a id="first-most-inline-level"></a>7.3.1.  Applicability

<a id="ref-for-initial-letter④"></a>

<a id="ref-for-propdef-initial-letter①②"></a>

<a id="ref-for-selectordef-first-letter④"></a>

<a id="ref-for-valdef-list-style-position-inside"></a>

<a id="ref-for-selectordef-marker"></a>

<a id="ref-for-inline-level-box①③"></a>

<a id="ref-for-containing-block⑥"></a>

<a id="ref-for-inline-box④⑤"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-valdef-initial-letter-normal②"></a>

To give authors more control over which characters can be styled as an [initial letter](#initial-letter) and to allow the possibility of multi-character <a id="ref-for-initial-letter⑤"></a>initial letters (such as for first word or first phrase styling), the [initial-letter](#propdef-initial-letter) property applies not just to the CSS-defined [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-element, but also to [inside](https://www.w3.org/TR/css-lists-3/#valdef-list-style-position-inside)-positioned [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo-elements and to [inline-level boxes](https://www.w3.org/TR/css-display-3/#inline-level-box) that are placed at the start of the first line. Specifically, <a id="ref-for-propdef-initial-letter①③"></a>initial-letter applies to any <a id="ref-for-inline-level-box①④"></a>inline-level box—​including any such <a id="ref-for-selectordef-first-letter⑤"></a>::first-letter or <a id="ref-for-selectordef-marker①"></a>::marker box—​that is the first child of its parent box and whose ancestors (if any) that are descendants of its [containing block](https://www.w3.org/TR/css-display-3/#containing-block) are all first-child [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) that have a [computed](https://www.w3.org/TR/CSS21/cascade.html#computed-value) <a id="ref-for-propdef-initial-letter①④"></a>initial-letter value of [normal](#valdef-initial-letter-normal).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9e69d0f4"></a> For example, the `<span>`, `<em>`, and `<b>` elements in the following example are "first-most inline-level descendants" of the `<p>`, but the `<strong>` element is not:
>
> ```text
> <p><span><em><b>This</b> phrase</em> is styled
> <strong>specially</strong>.</span> …
> ```
>
> If we apply the following rules:
>
> ```text
> em { initial-letter: 2; }
> b, strong { initial-letter: 3; }
> ```
>
> <a id="ref-for-propdef-initial-letter①⑤"></a>
>
> <a id="ref-for-initial-letter⑥"></a>
>
> The [initial-letter](#propdef-initial-letter) property will take effect only on the `<em>`. The styling on `<b>` is ignored, as it has an ancestor already styled as an [initial letter](#initial-letter); and the styling on `<strong>` is ignored because it is a second sibling.
>
> The result might be rendered as
>
> ![“This phrase” becomes the dropped text spanning two lines, the remainder of the text wrapping alongside.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/firstmost-inline.png)

<a id="ref-for-propdef-initial-letter①⑥"></a>

<a id="ref-for-inline-level-box①⑤"></a>

<a id="ref-for-inline-level⑨"></a>

<a id="ref-for-used-value"></a>

<a id="ref-for-valdef-initial-letter-normal③"></a>

<a id="ref-for-initial-letter⑦"></a>

If [initial-letter](#propdef-initial-letter) is applied to an [inline-level box](https://www.w3.org/TR/css-display-3/#inline-level-box) that is not positioned at the start of the line due to bidi reordering or which is otherwise preceded by other [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content, its [used value](https://www.w3.org/TR/CSS21/cascade.html#used-value) is [normal](#valdef-initial-letter-normal), and it is not formatted as an [initial letter](#initial-letter).

<a id="ref-for-propdef-initial-letter①⑦"></a>

<a id="ref-for-ruby-base-container-box"></a>

<a id="ref-for-ruby-container①"></a>

The effect of the [initial-letter](#propdef-initial-letter) property is undefined on children of [ruby base container boxes](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and on [ruby container boxes](https://www.w3.org/TR/css-ruby-1/#ruby-container).

<a id="ref-for-propdef-initial-letter①⑧"></a>

<a id="ref-for-propdef-float"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-valdef-position-static"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-valdef-display-block"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [initial-letter](#propdef-initial-letter) property cannot apply to any element whose [float](https://drafts.csswg.org/css2/#propdef-float) is not none or [position](https://www.w3.org/TR/CSS21/visuren.html#propdef-position) is not [static](https://www.w3.org/TR/css-position-3/#valdef-position-static), because these values cause its [display](https://www.w3.org/TR/css-display-3/#propdef-display) to compute to [block](https://www.w3.org/TR/css-display-3/#valdef-display-block).

<a id="ref-for-propdef-initial-letter-align①"></a>

### <a id="aligning-initial-letter"></a>7.4.  Alignment of Initial Letters: the [initial-letter-align](#propdef-initial-letter-align) property

<a id="ref-for-propdef-initial-letter-align②"></a>

As mentioned earlier, the alignment of initial letters depends on the script used. The [initial-letter-align](#propdef-initial-letter-align) property can be used to specify the proper alignment.



| Field               | Definition                                                                                                                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-initial-letter-align"></a>initial-letter-align                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-req"></a><a id="ref-for-comb-one④⑤"></a><a id="ref-for-mult-opt①"></a>\[ border-box[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ alphabetic [\|](https://www.w3.org/TR/css-values-4/#comb-one) ideographic <a id="ref-for-comb-one④⑥"></a>\| hanging <a id="ref-for-comb-one④⑦"></a>\| leading \]<a id="ref-for-mult-opt②"></a>? \][!](https://www.w3.org/TR/css-values-4/#mult-req) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | alphabetic                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | certain inline-level boxes and ::first-letter and inside ::marker boxes ([see prose](#first-most-inline-level))                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                    |



<a id="ref-for-initial-letter⑧"></a>

<a id="ref-for-over⑧"></a>

<a id="ref-for-under⑥"></a>

<a id="ref-for-root-inline-box①⓪"></a>

This property specifies the alignment points used to size and position an [initial letter](#initial-letter). Two sets of alignment points are necessary: the [over](https://www.w3.org/TR/css-writing-modes-4/#over) and [under](https://www.w3.org/TR/css-writing-modes-4/#under) alignment points of the <a id="ref-for-initial-letter⑨"></a>initial letter are matched to corresponding <a id="ref-for-over⑨"></a>over and <a id="ref-for-under⑦"></a>under points of the [root inline box](#root-inline-box).

Values have the following meanings:

<a id="valdef-initial-letter-align-alphabetic"></a>alphabetic  
<a id="ref-for-initial-letter①⓪"></a>

<a id="ref-for-alphabetic-baseline⑨"></a>

<a id="ref-for-cap-height-baseline①"></a>

Use the [cap-height](#cap-height-baseline) and [alphabetic](#alphabetic-baseline) baselines of the surrounding text to align the [initial letter](#initial-letter).

<a id="valdef-initial-letter-align-ideographic"></a>ideographic  
<a id="ref-for-initial-letter①①"></a>

<a id="ref-for-ideographic-ink-under-baseline①"></a>

<a id="ref-for-ideographic-ink-over-baseline①"></a>

Use the [ideographic-ink-over](#ideographic-ink-over-baseline) and [ideographic-ink-under](#ideographic-ink-under-baseline) baselines of the surrounding text to align the [initial letter](#initial-letter).

<a id="valdef-initial-letter-align-hanging"></a>hanging  
<a id="ref-for-initial-letter①②"></a>

<a id="ref-for-alphabetic-baseline①⓪"></a>

<a id="ref-for-hanging-baseline①"></a>

Use the [hanging](#hanging-baseline) and [alphabetic](#alphabetic-baseline) baselines of the surrounding text to align the [initial letter](#initial-letter).

<a id="valdef-initial-letter-align-leading"></a>leading  
<a id="ref-for-initial-letter①③"></a>

<a id="ref-for-half-leading⑥"></a>

<a id="ref-for-descent-metric⑤"></a>

<a id="ref-for-ascent-metric⑤"></a>

Use the over/under half-leading edges (i.e. [ascent](#ascent-metric)/[descent](#descent-metric) + [half-leading](#half-leading)) of the surrounding text to align the [initial letter](#initial-letter).

<a id="ref-for-initial-letter①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This will essentially match the edges of the [initial letter](#initial-letter) to middle of the line gap above/below the first/last impacted lines, which is an effect sometimes used in certain types of [Indic typesetting](https://www.w3.org/TR/ilreq/#h_scripts_without_hanging_baseline) [\[ILREQ\]](#biblio-ilreq).

<a id="valdef-initial-letter-align-border-box"></a>border-box  
<a id="ref-for-under⑧"></a>

<a id="ref-for-over①⓪"></a>

<a id="ref-for-border-edge"></a>

<a id="ref-for-line-over⑥"></a>

<a id="ref-for-line-under⑥"></a>

<a id="ref-for-initial-letter①⑤"></a>

Use the [initial letter box](#initial-letter)’s [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) and [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) [border edges](https://www.w3.org/TR/css-box-3/#border-edge) as the [over](https://www.w3.org/TR/css-writing-modes-4/#over) and [under](https://www.w3.org/TR/css-writing-modes-4/#under) alignment points, respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a1ba2074"></a> The vertical writing mode example earlier (in [§ 7.1 An Introduction to Initial Letters](#initial-letter-intro)) could be coded as:
>
> ```text
> span.initial {
>   initial-letter: 2;
>   initial-letter-align: ideographic;
> }
> ```
<a id="ref-for-valdef-initial-letter-align-border-box"></a>

<a id="ref-for-initial-letter①⑥"></a>

Except when [border-box](#valdef-initial-letter-align-border-box) is specified, the alignment points of the [initial letter](#initial-letter) are automatically determined from its contents:

1.  <a id="ref-for-under⑨"></a>

    <a id="ref-for-over①①"></a>

    <a id="ref-for-initial-letter①⑦"></a>

    If the [initial letter](#initial-letter) is an atomic inline, use its [over](https://www.w3.org/TR/css-writing-modes-4/#over) and [under](https://www.w3.org/TR/css-writing-modes-4/#under) content-box edges.

2.  <a id="ref-for-ideographic-ink-under-baseline②"></a>

    <a id="ref-for-ideographic-ink-over-baseline②"></a>

    <a id="ref-for-initial-letter①⑧"></a>

    Else if the [initial letter](#initial-letter) contains any character having the Han, Hangul, Kana, or Yi Unicode script property, use the [ideographic-ink-over](#ideographic-ink-over-baseline) and [ideographic-ink-under](#ideographic-ink-under-baseline) baselines.

3.  <a id="ref-for-alphabetic-baseline①①"></a>

    <a id="ref-for-hanging-baseline②"></a>

    <a id="ref-for-initial-letter①⑨"></a>

    Else if the [initial letter](#initial-letter) contains any character having the Han, Hangul, Kana, or Yi Unicode script property, use the [hanging](#hanging-baseline) and [alphabetic](#alphabetic-baseline) baselines.

4.  <a id="ref-for-alphabetic-baseline①②"></a>

    <a id="ref-for-cap-height-baseline②"></a>

    Else use the [cap-height](#cap-height-baseline) and [alphabetic](#alphabetic-baseline) baselines.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9175a33b"></a> Correct alignment of initial letter in scripts such as Hebrew and Thai is currently not possible because OpenType lacks corresponding metrics. ([Issue 5244](https://github.com/w3c/csswg-drafts/issues/5244))
>
> ![Hebrew 2-line drop-letter alignment using the hebrew-top and alphabetic baselines](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/hebrew-initial-letter.png)

<a id="ref-for-valdef-initial-letter-align-border-box①"></a>

<a id="ref-for-initial-letter②⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The ordering of keywords in this property is fixed in case [border-box](#valdef-initial-letter-align-border-box) is expanded to \[ border-box \| alphabetic \| ideographic \| hanging \] to allow explicitly specifying the [initial letter](#initial-letter)’s alignment points.

<a id="ref-for-propdef-initial-letter-align③"></a>

#### <a id="initial-letter-align-defaults"></a>7.4.1.  UA Default Stylesheet for [initial-letter-align](#propdef-initial-letter-align)

In order to provide the better behavior by default, UAs must include in their default UA style sheet the following rules:

```text
[lang]:lang(zh, ja, ko, ii) {
  initial-letter-align: ideographic;
}
[lang]:lang(hi, mr, ne, pi, kok, brx, mai, sd, sa) {
  initial-letter-align: hanging;
}
/* Script tags override language tags */
[lang]:lang('*-Latn', '*-Cyrl') {
  initial-letter-align: alphabetic;
}
[lang]:lang('*-Hani', '*-Hant', '*-Hans') {
  initial-letter-align: ideographic;
}
```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-fbd38850"></a> This only covers the most common cross-linguistic transcription systems. Should we include any other / all script tags in the UA style sheet?

### <a id="initial-letter-layout"></a>7.5.  Initial Letter Layout

<a id="ref-for-initial-letter②①"></a>

<a id="ref-for-inline-box④⑥"></a>

<a id="ref-for-atomic-inline④"></a>

There are two types of [initial letter boxes](#initial-letter): those that arise from non-replaced [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) and those that arise from [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline).

<a id="ref-for-inline-formatting-context①⓪"></a>

For the non-atomic <a id="inline-initial-letter"></a>inline initial letter, the box and its contents participate in the same [inline formatting context](#inline-formatting-context) as the line on which it occurs, and a lot of special rules apply to give the expected sizing and alignment.

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-automatic-size"></a>

<a id="ref-for-block-axis⑨"></a>

For an <a id="atomic-initial-letter"></a>atomic initial letter, however, which is either a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) or which establishes an [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context) for its contents, the sizing of the box (aside from its [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis)) and layout of the contents within the box follows the usual rules: it is primarily the positioning of the box which is special.

#### <a id="initial-letter-properties"></a>7.5.1.  Properties Applying to Initial Letters

<a id="ref-for-inline-box④⑦"></a>

<a id="ref-for-inline-initial-letter"></a>

<a id="ref-for-propdef-vertical-align①⓪"></a>

<a id="ref-for-longhand①"></a>

<a id="ref-for-propdef-font-size④"></a>

<a id="ref-for-propdef-line-height②①"></a>

<a id="ref-for-propdef-line-fit-edge②①"></a>

<a id="ref-for-propdef-inline-sizing②"></a>

<a id="ref-for-sizing-property"></a>

<a id="ref-for-propdef-box-sizing"></a>

<a id="ref-for-initial-letter②②"></a>

All properties that apply to an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) also apply to an [inline initial letter](#inline-initial-letter) except for [vertical-align](#propdef-vertical-align) and its [sub-properties](https://www.w3.org/TR/css-cascade-5/#longhand), [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size), [line-height](#propdef-line-height), [line-fit-edge](#propdef-line-fit-edge), and [inline-sizing](#propdef-inline-sizing). Additionally, all of the [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property) and [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) also apply to [initial letters](#initial-letter) (see [\[css-sizing-3\]](#biblio-css-sizing-3)).

<a id="ref-for-atomic-inline⑤"></a>

<a id="ref-for-initial-letter②③"></a>

<a id="ref-for-propdef-vertical-align①①"></a>

<a id="ref-for-longhand②"></a>

All properties that apply to an [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline) also apply to the <a id="ref-for-atomic-inline⑥"></a>atomic inline when styled as an [initial letter](#initial-letter), except for [vertical-align](#propdef-vertical-align) and its [sub-properties](https://www.w3.org/TR/css-cascade-5/#longhand).

#### <a id="initial-letter-box"></a>7.5.2.  Margins, Borders, and Padding

<a id="ref-for-margin⑨"></a>

<a id="ref-for-padding⑥"></a>

<a id="ref-for-border⑥"></a>

<a id="ref-for-propdef-initial-letter-align④"></a>

<a id="ref-for-valdef-initial-letter-align-border-box②"></a>

<a id="ref-for-initial-letter②④"></a>

<a id="ref-for-margin-box②"></a>

<a id="ref-for-propdef-initial-letter-wrap①"></a>

Initial letters can be styled with [margins](https://www.w3.org/TR/css-box-4/#margin), [padding](https://www.w3.org/TR/css-box-4/#padding), and [borders](https://www.w3.org/TR/css-box-4/#border) just like any other box. Unless [initial-letter-align](#propdef-initial-letter-align) is [border-box](#valdef-initial-letter-align-border-box), its vertical alignment and [font sizing](#sizing-initial-letter) are not affected. However the effective exclusion area, which is typically the [initial letter](#initial-letter)’s [margin box](https://www.w3.org/TR/css-box-3/#margin-box) (see [initial-letter-wrap](#propdef-initial-letter-wrap)) is affected.

When padding and borders are zero, the initial letter may be kerned; see below.

#### <a id="sizing-initial-letter"></a>7.5.3.  Font Sizing of Initial Letters

<a id="ref-for-inline-initial-letter①"></a>

<a id="ref-for-initial-letter②⑤"></a>

<a id="ref-for-initial-letter-initial-letter-size②"></a>

<a id="ref-for-propdef-initial-letter①⑨"></a>

<a id="ref-for-propdef-initial-letter-align⑤"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-computed-value③"></a>

<a id="ref-for-propdef-font-size⑤"></a>

<a id="ref-for-em"></a>

For an [inline initial letter](#inline-initial-letter), the font size used for sizing the [initial letter](#initial-letter) contents is calculated to fulfill its specified [size](#initial-letter-initial-letter-size) (see [initial-letter](#propdef-initial-letter)) as anchored by its specified alignment points (see [initial-letter-align](#propdef-initial-letter-align)). Note that no layout is required in this calculation: it is based only on computed values and font metrics. These [used](https://www.w3.org/TR/CSS21/cascade.html#used-value) font size calculations <em>do not</em> affect the [computed](https://www.w3.org/TR/CSS21/cascade.html#computed-value) [font-size](https://www.w3.org/TR/CSS21/fonts.html#propdef-font-size), and therefore have no effect on the computation of [em](https://www.w3.org/TR/css-values-4/#em) length values, etc.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c519e9ec"></a> What about inheritance to descendants? [\[Issue \#4988\]](https://github.com/w3c/csswg-drafts/issues/4988)

<a id="ref-for-propdef-line-height②②"></a>

The line height used in these calculations is the [line-height](#propdef-line-height) of the containing block (or, in the case where a baseline grid is in use, the baseline-to-baseline spacing required by the baseline grid [\[CSS-LINE-GRID-1\]](#biblio-css-line-grid-1)). The contents of the lines spanned, and therefore any variation in their heights and positions, is not accounted for.

![Text underlay shows how initial letter alignment is not affected by the content of the spanned lines.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/infinite-text.png)

For an <var>N</var>-line drop initial in a Western script, the cap-height of the letter needs to be (<var>N</var> – 1) times the line-height, plus the cap-height of the surrounding text. Note this height is <em>not</em> the font size of the drop initial.

Actually calculating this font size is tricky. For an <var>N</var>-line drop initial, we find the drop initial font size to be:

![Font size of drop cap = ((N-1) \* line-height + \[cap-height of para\] \* \[font size of paragraph\])/\[cap-height ratio of drop initial font\]](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/InitialCapEquation.png)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-20a01c1f"></a> Update this calculation to be a) generic across writing systems / alignment points and b) handle non-integer sizes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c366e114"></a> A three-line drop initial in Adobe Minion Pro would have a font size of 61.2pt given 12pt text, 16pt line-height, and a cap-height of 651/1000 (from the font’s OS/2 table).

<a id="ref-for-atomic-initial-letter"></a>

For an [atomic initial letter](#atomic-initial-letter), the used font size is the computed font size as usual.

#### <a id="initial-letter-shaping"></a>7.5.4.  Shaping and Glyph Selection

<a id="ref-for-propdef-initial-letter②⓪"></a>

<a id="ref-for-valdef-initial-letter-normal④"></a>

<a id="ref-for-inline-initial-letter②"></a>

<a id="ref-for-initial-letter②⑥"></a>

When [initial-letter](#propdef-initial-letter) is not [normal](#valdef-initial-letter-normal), an [inline initial letter](#inline-initial-letter) is isolated for glyph shaping; however the text after it <em>should</em> shape across the <a id="ref-for-inline-initial-letter③"></a>inline initial letter box’s boundaries, assuming its presence as part of the first line’s text content. (See [CSS Text 3 § 7.3 Shaping Across Element Boundaries](https://www.w3.org/TR/css-text-3/#boundary-shaping).) For example, if the first letter of the word “يحق” were styled with <a id="ref-for-propdef-initial-letter②①"></a>initial-letter: 2 1, the first letter would be styled in its isolated form “ي”, as the [initial letter](#initial-letter), followed by the medial/final-form “ﺤﻖ”, which assumes it is preceded by the initial letter’s contents as normal text.

![Two-line Arabic drop-cap showing isolated form of the first letter, connected form of the rest of the word.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/arabic-drop-cap.png)

Two-line Arabic “Drop-cap”

#### <a id="initial-letter-box-size"></a>7.5.5.  Sizing the Initial Letter Box

<a id="ref-for-inline-initial-letter④"></a>

<a id="ref-for-initial-letter②⑦"></a>

<a id="ref-for-width"></a>

<a id="ref-for-height"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-min-width"></a>

<a id="ref-for-max-width"></a>

<a id="ref-for-propdef-box-sizing①"></a>

For an [inline initial letter](#inline-initial-letter), if the [initial letter](#initial-letter)’s [preferred width](https://www.w3.org/TR/css-sizing-3/#width)/[preferred height](https://www.w3.org/TR/css-sizing-3/#height) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), use that value (clamped as required by the [min size](https://www.w3.org/TR/css-sizing-3/#min-width) and [max size](https://www.w3.org/TR/css-sizing-3/#max-width) properties, and handling [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) as required) for that dimension of the box.

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-x10④"></a>

Otherwise it is considered to have an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) in that dimension and its [content box](https://www.w3.org/TR/CSS21/box.html#x10) is sized to fit both:

- <a id="ref-for-initial-letter-initial-letter-sink④"></a>

  The specified [sink](#initial-letter-initial-letter-sink) (i.e the space between the over alignment point and the under alignment point).

- <a id="ref-for-atomic-inline⑦"></a>

  <a id="ref-for-margin-box③"></a>

  <a id="ref-for-propdef-hanging-punctuation"></a>

  <a id="ref-for-hang"></a>

  The glyph outlines of all the glyphs it contains—​excluding any that [hang](https://www.w3.org/TR/css-text-3/#hang) (see [hanging-punctuation](https://www.w3.org/TR/css-text-4/#propdef-hanging-punctuation))—​as well as the [margin boxes](https://www.w3.org/TR/css-box-3/#margin-box) of any [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) it contains.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="initial-letter-exclusions"></a> The glyph(s) of an initial letter do not always fit within the specified sink. For example, if an initial letter has a descender, it could crash into the (n+1)th line of text. This is not desirable.
  > ![3-line drop cap with J, with descender crashing into fourth line of text](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Dropcap-J-3line-crash.png)
  > <a id="ref-for-propdef-initial-letter②②"></a>
  >
  > Incorrect: three-line initial letter ([initial-letter: drop 3](#propdef-initial-letter)) with descender. In this font, the capital “J” extends well below the baseline (shown in red).
  >
  > <a id="ref-for-line-box④⑥"></a>
  >
  > <a id="ref-for-initial-letter②⑧"></a>
  >
  > <a id="ref-for-initial-letter-initial-letter-sink⑤"></a>
  >
  > Therefore all [line boxes](#line-box) impacted by the glyph outlines of an [initial letter](#initial-letter) need to be excluded, not just those within range of the [initial letter sink](#initial-letter-initial-letter-sink).
  >
  > ![3-line drop cap with J, but four-line exclusion](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Dropcap-J-3line-exclude.png)
  > Correct: text excluded around glyph bounding box

<a id="ref-for-block-start③"></a>

<a id="ref-for-padding⑦"></a>

<a id="ref-for-border⑦"></a>

<a id="ref-for-content-edge⑤"></a>

<a id="ref-for-over①②"></a>

However, if its [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) [padding](https://www.w3.org/TR/css-box-4/#padding) and [border](https://www.w3.org/TR/css-box-4/#border) are both zero, then its <a id="ref-for-block-start④"></a>block-start [content edge](https://www.w3.org/TR/css-box-3/#content-edge) instead coincides with its [over](https://www.w3.org/TR/css-writing-modes-4/#over) alignment point exactly, and any content overflowing above that point is ignored for the purpose of layout.

<a id="ref-for-inline-initial-letter⑤"></a>

<a id="ref-for-over①③"></a>

<a id="ref-for-margin①⓪"></a>

<a id="ref-for-initial-letter②⑨"></a>

<a id="ref-for-containing-block⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If an [inline initial letter](#inline-initial-letter) has ascenders above its [over](https://www.w3.org/TR/css-writing-modes-4/#over) alignment point, and the author has not provided sufficient [margin](https://www.w3.org/TR/css-box-4/#margin) on either the [initial letter](#initial-letter) itself or its [containing block](https://www.w3.org/TR/css-display-3/#containing-block), then those ascenders might collide with preceding content.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It might be nice to automatically provide the necessary spacing by treating such ascenders as a margin that can collapse with the margin of the containing block, and thus guarantee the requisite spacing without imposing any additional space unless it becomes actually necessary. Depending on implementation complexity, this option may be explored in the future; but in the meantime, authors need to be careful to provide the requisite spacing explicitly.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8de59d00"></a> Should the hanging punctuation be included in the box instead (so that the box is drawn around the punctuation when it is made visible through borders/background), but rather only excluded when positioning the box (so that the initial letter remains flush, with the hanging punctuation properly hanging)? See [discussion](https://github.com/w3c/csswg-drafts/issues/310).

<a id="ref-for-atomic-initial-letter①"></a>

<a id="ref-for-atomic-inline⑧"></a>

<a id="ref-for-automatic-size②"></a>

<a id="ref-for-block-size"></a>

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-inline-initial-letter⑥"></a>

<a id="ref-for-valdef-initial-letter-align-border-box③"></a>

<a id="ref-for-definite①"></a>

For [atomic initial letters](#atomic-initial-letter), sizing follows the usual rules for that type of [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline). However, if the box has an [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size) [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) ([auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)), then its <a id="ref-for-block-size①"></a>block size is determined as for an [inline initial letter](#inline-initial-letter) with [border-box](#valdef-initial-letter-align-border-box) alignment, and is [definite](https://www.w3.org/TR/css-sizing-3/#definite).

#### <a id="initial-letter-content-align"></a>7.5.6.  Alignment Within an Initial Letter Box

<a id="ref-for-automatic-size③"></a>

<a id="ref-for-inline-initial-letter⑦"></a>

<a id="ref-for-propdef-text-align"></a>

<a id="ref-for-propdef-align-content②"></a>

By default (i.e. under [automatic sizing](https://www.w3.org/TR/css-sizing-3/#automatic-size)), the content box of an [inline initial letter](#inline-initial-letter) is fitted exactly to its content, and alignment properties like [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) or [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) do not apply. However, if the box is <em>not</em> sized automatically:

- <a id="ref-for-inline-axis⑤"></a>

  <a id="ref-for-initial-letter③⓪"></a>

  <a id="ref-for-propdef-text-align①"></a>

  <a id="ref-for-definite②"></a>

  <a id="ref-for-inline-size"></a>

  If the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) is honored for aligning the contents of the [initial letter](#initial-letter) within its box in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) (using its <a id="ref-for-inline-axis⑥"></a>inline-axis bearings as usual, not the bounding box of its glyph outlines).

- <a id="ref-for-block-axis①⓪"></a>

  <a id="ref-for-propdef-align-content③"></a>

  <a id="ref-for-definite③"></a>

  <a id="ref-for-block-size②"></a>

  If the [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) is honored for aligning its contents in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) (using its <a id="ref-for-block-axis①①"></a>block-axis bearings, synthesizing them if needed).

### <a id="initial-letter-position"></a>7.6.  Initial Letter Positioning and Spacing

#### <a id="initial-letter-block-position"></a>7.6.1.  Block-axis Positioning

<a id="ref-for-block-axis①②"></a>

<a id="ref-for-initial-letter③①"></a>

<a id="ref-for-line-box④⑦"></a>

<a id="ref-for-originating-line"></a>

<a id="ref-for-propdef-initial-letter-align⑥"></a>

<a id="ref-for-initial-letter-initial-letter-sink⑥"></a>

<a id="ref-for-propdef-initial-letter②③"></a>

In the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), the [initial letter](#initial-letter) is positioned with respect to the [line box](#line-box) in which it [originates](#originating-line) as required to satisfy its alignment ([initial-letter-align](#propdef-initial-letter-align)) and specified [sink](#initial-letter-initial-letter-sink) ([initial-letter](#propdef-initial-letter)):

- <a id="ref-for-initial-letter-initial-letter-size③"></a>

  <a id="ref-for-initial-letter-initial-letter-sink⑦"></a>

  <a id="ref-for-initial-letter③②"></a>

  <a id="ref-for-under①⓪"></a>

  <a id="ref-for-propdef-line-height②③"></a>

  <a id="ref-for-containing-block⑧"></a>

  <a id="ref-for-block-end④"></a>

  If its [size](#initial-letter-initial-letter-size) is greater than or equal to its [sink](#initial-letter-initial-letter-sink), the [initial letter](#initial-letter) is positioned to satisfy its [under](https://www.w3.org/TR/css-writing-modes-4/#under) alignment, and then shifted by (<a id="ref-for-initial-letter-initial-letter-sink⑧"></a>sink - 1) × [line-height](#propdef-line-height) of [containing block](https://www.w3.org/TR/css-display-3/#containing-block) towards the <a id="ref-for-containing-block⑨"></a>containing block’s [block end](https://www.w3.org/TR/css-writing-modes-4/#block-end).

- <a id="ref-for-initial-letter-initial-letter-size④"></a>

  <a id="ref-for-initial-letter-initial-letter-sink⑨"></a>

  <a id="ref-for-initial-letter③③"></a>

  <a id="ref-for-over①④"></a>

  If its [size](#initial-letter-initial-letter-size) is less than its [sink](#initial-letter-initial-letter-sink), the [initial letter](#initial-letter) is positioned to satisfy its [over](https://www.w3.org/TR/css-writing-modes-4/#over) alignment.

<a id="ref-for-initial-letter③④"></a>

<a id="ref-for-propdef-initial-letter②④"></a>

<a id="ref-for-under①①"></a>

<a id="ref-for-root-inline-box①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An [initial letter](#initial-letter) is essentially positioned such that it would sink the number of lines specified by [initial-letter](#propdef-initial-letter)’s second argument and align to the requisite [under](https://www.w3.org/TR/css-writing-modes-4/#under) alignment point if it was assumed that its containing block held only the <a id="ref-for-initial-letter③⑤"></a>initial letter itself followed by an infinite sequence of plain text as the direct contents of its [root inline box](#root-inline-box). Its position is not affected by line height inconsistencies introduced by the contents of the impacted line boxes.

![Constant-sized text underlay shows how initial letter alignment is not affected by the content of the spanned lines.](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/infinite-text.png)

<a id="ref-for-initial-letter③⑥"></a>

<a id="ref-for-logical-height①①"></a>

<a id="ref-for-line-box④⑧"></a>

<a id="ref-for-block-start⑤"></a>

<a id="ref-for-margin-edge①"></a>

<a id="ref-for-containing-block①⓪"></a>

<a id="ref-for-content-edge⑥"></a>

<a id="ref-for-originating-line①"></a>

The [initial letter](#initial-letter) does not increase the [logical height](https://www.w3.org/TR/css-writing-modes-4/#logical-height) of the [line box](#line-box) in which it participates: it can protrude above or below it. It must be positioned such that its own [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) [margin edge](https://www.w3.org/TR/css-box-3/#margin-edge) is below its [containing block](https://www.w3.org/TR/css-display-3/#containing-block)’s <a id="ref-for-block-start⑥"></a>block-start [content edge](https://www.w3.org/TR/css-box-3/#content-edge), and thus can force its [originating line box](#originating-line) (and subsequent content) to shift further away from that edge.

#### <a id="initial-letter-inline-position"></a>7.6.2.  Inline Kerning

<a id="ref-for-initial-letter③⑦"></a>

<a id="ref-for-automatic-size④"></a>

<a id="ref-for-inline-size①"></a>

<a id="ref-for-margin-box④"></a>

<a id="ref-for-line-box④⑨"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-margin①①"></a>

If the [initial letter](#initial-letter) is a non-atomic inline with an [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size) [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) and zero padding and borders, its [margin box](https://www.w3.org/TR/css-box-3/#margin-box) is kerned (negatively inset) by the distance from the start edge of its content box to the point in the content that would have been placed at the start edge of the [line box](#line-box) if it were not an <a id="ref-for-initial-letter③⑧"></a>initial letter (i.e. the distance between its glyph bounding box and its start side bearing). This inset is effectively an additional [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) [margin](https://www.w3.org/TR/css-box-4/#margin) on the box.

<a id="ref-for-propdef-initial-letter-wrap②"></a>

### <a id="initial-letter-wrapping"></a>7.7.  Initial Letter Wrapping: the [initial-letter-wrap](#propdef-initial-letter-wrap) property

<a id="ref-for-propdef-initial-letter-wrap③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [initial-letter-wrap](#propdef-initial-letter-wrap) is at risk.



| Field               | Definition                                                                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-initial-letter-wrap"></a>initial-letter-wrap                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage④"></a><a id="ref-for-comb-one④⑧"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) first <a id="ref-for-comb-one④⑨"></a>\| all <a id="ref-for-comb-one⑤⓪"></a>\| grid <a id="ref-for-comb-one⑤①"></a>\| [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | certain inline-level boxes and ::first-letter and inside ::marker boxes ([see prose](#first-most-inline-level))                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-logical-width④"></a>relative to [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of (last fragment of) initial letter                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑤"></a>specified keyword or computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                |



<a id="ref-for-initial-letter③⑨"></a>

This property specifies whether lines impacted by an [initial letter](#initial-letter) are shortened to fit the rectangular shape of the <a id="ref-for-initial-letter④⓪"></a>initial letter box or the contour of its glyph outline.

<a id="valdef-initial-letter-wrap-none"></a>none

<a id="ref-for-initial-letter④①"></a>

<a id="ref-for-margin-edge②"></a>

<a id="ref-for-inline-end"></a>

No contour-fitting is performed: each impacted line is aligned flush to the [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) [margin edge](https://www.w3.org/TR/css-box-3/#margin-edge) of the [initial letter](#initial-letter).

<a id="valdef-initial-letter-wrap-first"></a>first

<a id="ref-for-valdef-initial-letter-wrap-all"></a>

<a id="ref-for-initial-letter④②"></a>

<a id="ref-for-typographic-character-unit"></a>

<a id="ref-for-valdef-initial-letter-wrap-none"></a>

Behaves as [none](#valdef-initial-letter-wrap-none) if the first [typographic character unit](https://www.w3.org/TR/css-text-4/#typographic-character-unit) after the [initial letter](#initial-letter) belongs to Unicode General Category Zs. Otherwise behaves as for [all](#valdef-initial-letter-wrap-all) on the first line of the block containing the initial letter and as <a id="ref-for-valdef-initial-letter-wrap-none①"></a>none on the rest.

<a id="ref-for-initial-letter④③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8c68c539"></a> This example shows why contour-fitting the first line is necessary, and why it is dropped when the [initial letter](#initial-letter) is followed by a space:
>
> ![optical kerning in the presence or absence of a space after the initial letter](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/OpticalKerning.png)
>
> In the top paragraph, the initial letter "A" has a word space after it: the gap between the top of the "A" and the next letter provides the necessary word separation. In the next paragraph, the initial letter "A" is part of the first word, and leaving a gap between the top of the "A" and the next letter would create a jarring visual break within the word. In this case, the first line of text should be kerned into the initial letter’s area, as shown in the bottom paragraph.

<a id="ref-for-valdef-initial-letter-wrap-first"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-58dffa4b"></a> Do we need an unconditional [first](#valdef-initial-letter-wrap-first)? (I.e. Should we rename this value to auto and add a <a id="ref-for-valdef-initial-letter-wrap-first①"></a>first value that does not check for spaces?) See GitHub issue [410](https://github.com/w3c/csswg-drafts/issues/410)

<a id="valdef-initial-letter-wrap-all"></a>all

<a id="ref-for-start"></a>

<a id="ref-for-line-box⑤⓪"></a>

<a id="ref-for-initial-letter④④"></a>

For each line of text impacted by the [initial letter](#initial-letter), the [line box](#line-box) adjacent to the <a id="ref-for-initial-letter④⑤"></a>initial letter starts at the [start](https://www.w3.org/TR/css-writing-modes-4/#start)-most point that does not overlap the <a id="ref-for-initial-letter④⑥"></a>initial letter’s glyph outline.

<a id="ref-for-valdef-shape-outside-none"></a>

If the value of shape-outside is not [none](https://www.w3.org/TR/css-shapes-1/#valdef-shape-outside-none), shape-outside is used instead of the glyph outline.

<a id="ref-for-propdef-shape-margin"></a>

<a id="ref-for-initial-letter④⑦"></a>

<a id="ref-for-margin-edge③"></a>

In both cases, [shape-margin](https://www.w3.org/TR/css-shapes-1/#propdef-shape-margin) is applied to expand the outline, and the resulting outline is clipped by the [initial letter](#initial-letter)’s [margin edges](https://www.w3.org/TR/css-box-3/#margin-edge).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is at-risk.

<a id="valdef-initial-letter-wrap-grid"></a>grid

<a id="ref-for-propdef-justify-self"></a>

<a id="ref-for-propdef-letter-spacing"></a>

<a id="ref-for-valdef-initial-letter-wrap-none②"></a>

This value is the same as [none](#valdef-initial-letter-wrap-none), except that the exclusion area of the impacted lines is increased as necessary for its end-edge to land on the character grid, i.e. to be a multiple of (1ic + [letter-spacing](https://www.w3.org/TR/CSS21/text.html#propdef-letter-spacing)) as computed on the containing block. The [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) property can then be used to align the initial letter box within the exclusion area.

![Diagram of Japanese initial letter in vertical writing mode](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/CJK-Initial.001.png)

Diagram of Japanese initial letter in vertical writing mode

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In this example, the exclusion area for the drop initial is larger than its glyph in order to preserve inline-axis alignment.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value is also at-risk.

<a id="ref-for-length-value③"></a>

<a id="valdef-initial-letter-wrap-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-percentage-value②"></a>

<a id="valdef-initial-letter-wrap-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-valdef-initial-letter-wrap-first②"></a>

This value behaves the same as [first](#valdef-initial-letter-wrap-first) except that the adjustment to the first line is given explicitly instead of being inferred from the glyph shape.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-645378a9"></a> This really needs font-relative lengths to be relative to the used size.

<a id="ref-for-valdef-initial-letter-wrap-first③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value exists because it is easier to implement. Authors are encouraged to use the [first](#valdef-initial-letter-wrap-first) value and to set margins to control spacing, and to use this as a fallback for glyph detection if necessary.

<a id="ref-for-valdef-initial-letter-wrap-first④"></a>

<a id="ref-for-length-value④"></a>

<a id="ref-for-percentage-value③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-09c7aa2b"></a> In the following example, UAs that support [first](#valdef-initial-letter-wrap-first) will use the glyph outline plus the specified margin in order to place the first line, whereas UAs that only support [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values will pull in the first line by 40% of the initial letter’s width (and then add the margin to that point).
>
> ```text
> h1 + p:first-letter {
>   initial-letter: 3; /* 3-line drop-cap */
>   initial-letter-wrap: first;
>   margin-right: 0.1em;
> }
> @supports (not (initial-letter-wrap: first)) {
>   /* Classes auto-generated on paragraphs to match first letter. */
>   p.A:first-letter {
>     initial-letter-wrap: -40%; /* Start of glyph outline, assuming correct font. */
>   }
> }
> ```
<a id="ref-for-valdef-initial-letter-wrap-first⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e060e05e"></a> These values and related annoyance is likely unnecessary if someone submits a patch to Blink to support [first](#valdef-initial-letter-wrap-first).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c4ff3304"></a> Edit figure to show how auto behaves in varying contexts

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dffa11d5"></a>
>
> ```text
> p::first-letter {
>   initial-letter: 3;
>   initial-letter-wrap: none;
> }
> ```
>
> ![regular dropcap A](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/A-wraparound-none.png)
>
> Ordinary initial letter with no wrapping.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-60dc0ae7"></a>
>
> ```text
> p::first-letter {
>   initial-letter: 3;
>   initial-letter-wrap: all;
> }
> ```
>
> ![text wrapping around dropcap A](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/A-wraparound.png)
>
> Text follows shape of initial letter. Each line box should just touch the ink of the letter, with some offset (represented by the shaded box).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95fe5f59"></a>
>
> ```text
> p::first-letter {
>   initial-letter: 3;
>   initial-letter-wrap: first;
> }
> ```
>
> ![text wrapping around dropcap A but only on first line](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/A-wraparound-first.png)
>
> Only the first line is moved up against the ink of the initial letter.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-db88657f"></a>
>
> ```text
> p::first-letter {
>   initial-letter: 3;
>   initial-letter-wrap: all;
> }
> ```
>
> ![text wrapping around dropcap V](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/V-wraparound.png)
>
> ![text wrapping around dropcap P](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/P-wraparound.png)
>
> ![text wrapping around dropcap W](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/W-wraparound.png)

### <a id="initial-letter-line-layout"></a>7.8.  Line Layout

<a id="ref-for-initial-letter④⑧"></a>

<a id="ref-for-in-flow⑤"></a>

<a id="ref-for-block-formatting-context"></a>

<a id="ref-for-line-box⑤①"></a>

<a id="ref-for-inline-level①⓪"></a>

An [initial letter box](#initial-letter) is considered [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) in its [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context), and is part of the contents of the [line box](#line-box) in which it originates (its <a id="originating-line"></a>originating line box). Aside from the vertical axis (see [§ 7.6.1 Block-axis Positioning](#initial-letter-block-position)), its interaction with the rest of the contents of the line is as normal for [inline-level content](https://www.w3.org/TR/css-display-3/#inline-level), except in a few specific details…

#### <a id="initial-letter-inline-flow"></a>7.8.1.  Inline Flow Layout: Alignment, Justification, and White Space

<a id="ref-for-initial-letter④⑨"></a>

<a id="ref-for-inline-level①①"></a>

<a id="ref-for-originating-line②"></a>

An [initial letter box](#initial-letter) is handled similar to any other [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content participating in its [originating line box](#originating-line), including participation in alignment, justification, and white space processing.

<a id="ref-for-collapsible-white-space"></a>

<a id="ref-for-sunken-initial②"></a>

<a id="ref-for-originating-line③"></a>

<a id="ref-for-propdef-letter-spacing①"></a>

<a id="ref-for-justification-opportunity"></a>

<a id="ref-for-propdef-word-spacing"></a>

<a id="ref-for-word-separator"></a>

<a id="ref-for-typographic-character-unit①"></a>

However, to ensure consistent alignment of all the impacted lines, [collapsible white space](https://www.w3.org/TR/css-text-4/#collapsible-white-space) between a [sunken initial](#sunken-initial) and subsequent content on its [originating line](#originating-line) is collapsed away, and any [letter-spacing](https://www.w3.org/TR/CSS21/text.html#propdef-letter-spacing) or [justification opportunity](https://www.w3.org/TR/css-text-4/#justification-opportunity) that would normally be introduced by the juxtaposition of the contents of a <a id="ref-for-sunken-initial③"></a>sunken initial and the subsequent contents of the line is suppressed. (Note that this does not affect [word-spacing](https://www.w3.org/TR/CSS21/text.html#propdef-word-spacing) or the <a id="ref-for-justification-opportunity①"></a>justification opportunity introduced by a [word separator](https://www.w3.org/TR/css-text-4/#word-separator) because that space is provided by the [typographic character unit](https://www.w3.org/TR/css-text-4/#typographic-character-unit) alone and not by its juxtaposition with an adjacent character.)

#### <a id="initial-letter-indentation"></a>7.8.2.  Edge Effects: Indentation and Hanging Punctuation

<a id="ref-for-propdef-text-indent"></a>

<a id="ref-for-propdef-hanging-punctuation①"></a>

<a id="ref-for-initial-letter⑤⓪"></a>

<a id="ref-for-originating-line④"></a>

[text-indent](https://www.w3.org/TR/CSS21/text.html#propdef-text-indent) and [hanging-punctuation](https://www.w3.org/TR/css-text-4/#propdef-hanging-punctuation) apply to an [initial letter](#initial-letter)’s [originating line box](#originating-line) as usual, and cause a shift in the start of the line’s contents including the <a id="ref-for-initial-letter⑤①"></a>initial letter itself. Subsequent lines affected by the exclusion are shortened as usual, possibly more or less than otherwise depending on the resulting position of the <a id="ref-for-initial-letter⑤②"></a>initial letter.

![initial letter with text indent](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/InitialLetterWithTextIndent.png)

Initial letter with text indent.

<a id="ref-for-propdef-initial-letter②⑤"></a>

<a id="ref-for-propdef-hanging-punctuation②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a41c99e9"></a> The interaction of [initial-letter](#propdef-initial-letter) and [hanging-punctuation](https://www.w3.org/TR/css-text-4/#propdef-hanging-punctuation) is [under discussion](https://github.com/w3c/csswg-drafts/issues/310#issuecomment-396765893).

#### <a id="initial-letter-ancestors"></a>7.8.3.  Ancestor Inlines

<a id="ref-for-initial-letter⑤③"></a>

<a id="ref-for-inline-box④⑧"></a>

<a id="ref-for-margin-edge④"></a>

<a id="ref-for-propdef-letter-spacing②"></a>

If the [initial letter box](#initial-letter) is contained by [inline box](https://www.w3.org/TR/css-display-3/#inline-box) ancestors, the boundaries of those <a id="ref-for-inline-box④⑨"></a>inline boxes are drawn to exclude the <a id="ref-for-initial-letter⑤④"></a>initial letter box, as if it were outside their startmost [margin edge](https://www.w3.org/TR/css-box-3/#margin-edge). This is a purely geometric operation: it does not affect e.g. property inheritance or the effective [letter-spacing](https://www.w3.org/TR/CSS21/text.html#propdef-letter-spacing) between the <a id="ref-for-initial-letter⑤⑤"></a>initial letter box and subsequent content.

#### <a id="initial-letter-multi-line"></a>7.8.4.  Multi-line Initial Letters

If an initial letter is too long to fit on one line, it wraps (according to the usual text-wrapping rules), each line filled and formatted exactly as if it were the first line and the initial letter too long to fit any subsequent normal text. Any normal text after the initial letter starts on its last line, affected exactly as if that line were the first line.

![multi-line drop cap](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/Multi-line-initial.png)

Drop cap extends to two lines.

### <a id="initial-letter-paragraphs"></a>7.9.  Clearing Initial Letters

#### <a id="raised-sunken-caps"></a>7.9.1.  Raised and sunken caps

The margin box of an initial letter contributes to the size of its containing element. Initial letters that extend above the first line of text, known as “raised caps” or “sunken caps,” do not extend up into previous elements. Since the content box for an initial letter includes all glyph ink, this also means that accents or other ink above the cap height of an initial letter will not impinge on previous elements.

![raised cap para after normal para](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/initial-letter-drop-para-compare.png)

Raised cap (`initial-letter: 3 1`) on right; note that the position of the “C” is the same in both cases, but on the right all text is moved down relative to the initial letter.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bd8afce4"></a> Handle glyph ink above cap height of font. Proposal: Make it an exclusion area for line boxes and border boxes. Include margin specified on initial-letters as part of exclusion area in order to control spacing.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9ca3e502"></a> Draw a box model diagram here. Does the margin of the initial letter collapse with its container?

#### <a id="short-para-initial-letter"></a>7.9.2.  Short paragraphs with initial letters

A paragraph with an initial letter can have fewer lines of text than the initial letter occupies. In this case, the initial letter’s top alignment is still honored, and its exclusion area continues into any subsequent blocks. This forces the subsequent inline-level content to wrap around the initial letter—​exactly as if that block’s text were part of its own containing block. (This is similar to how floats exclude content in subsequent block boxes.)

![short para with initial letter](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/initial-letter-short-para.png)

The red text is a short paragraph with an initial letter. Note the subsequent paragraph wraps around the initial letter just as text in the paragraph with the initial letter does.

<a id="ref-for-independent-formatting-context①"></a>

<a id="ref-for-propdef-clear"></a>

If the subsequent block starts with an initial letter, establishes an [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context), or specifies [clear](https://drafts.csswg.org/css2/#propdef-clear) in the initial letter’s containing block’s start direction, then it must clear the previous block’s initial letter.

![short para with initial letter followed by para with initial](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/initial-letter-short-para-initial.png)

The red text is a short paragraph with an initial letter. The subsequent paragraph clears because it also has an initial letter.

#### <a id="initial-letter-floats"></a>7.9.3.  Interaction with floats

<a id="ref-for-initial-letter⑤⑥"></a>

<a id="ref-for-in-flow⑥"></a>

<a id="ref-for-inline-level①②"></a>

<a id="ref-for-line-box⑤②"></a>

[Initial letters](#initial-letter) are not [floats](https://www.w3.org/TR/CSS21/visuren.html#floats): they are [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content that belongs to a [line box](#line-box). Therefore:

- <a id="ref-for-initial-letter⑤⑦"></a>

  <a id="ref-for-propdef-clear①"></a>

  The [clear](https://drafts.csswg.org/css2/#propdef-clear) property does not care about [initial letters](#initial-letter): it neither applies to <a id="ref-for-initial-letter⑤⑧"></a>initial letters nor clears them when applied to nearby floats.

- <a id="ref-for-block-formatting-context①"></a>

  <a id="ref-for-margin-box⑤"></a>

  <a id="ref-for-initial-letter⑤⑨"></a>

  Like line boxes or floats, [initial letter boxes](#initial-letter) must not overlap the [margin boxes](https://www.w3.org/TR/css-box-3/#margin-box) of any floats participating in the same [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context). An overlapping <a id="ref-for-initial-letter⑥⓪"></a>initial letter box is shifted inward or downward until either it fits without overlapping or there are no more floats present.

- <a id="ref-for-originating-line⑤"></a>

  <a id="ref-for-initial-letter⑥①"></a>

  If a line box’s start edge shifts or moves down to clear a float, an [initial letter](#initial-letter) [originating](#originating-line) in it moves with it; likewise if an <a id="ref-for-initial-letter⑥②"></a>initial letter shifts inward or moves downward to clear a float, its <a id="ref-for-originating-line⑥"></a>originating line box and subsequent line boxes shorten and/or move accordingly.

- <a id="ref-for-initial-letter⑥③"></a>

  <a id="ref-for-inline-start①"></a>

  If an [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) float originates in the first line of content adjacent to an [initial letter](#initial-letter), then it moves past the <a id="ref-for-initial-letter⑥④"></a>initial letter towards the containing block edge, exactly as if the <a id="ref-for-initial-letter⑥⑤"></a>initial letter were any other inline-level content.

  <a id="ref-for-initial-letter⑥⑥"></a>

  However, if such a float originates in subsequent lines of content adjacent to a (sunk) [initial letter](#initial-letter), then that float must clear the <a id="ref-for-initial-letter⑥⑦"></a>initial letter.

![initial letter interacting with floats](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/float-interaction.png)

In the absence of an initial letter, the first line of text could abut the blue float. But the presence of the initial letter requires that the text move over.

See [CSS2§9.5](https://www.w3.org/TR/CSS21/visuren.html#floats) for more information about the layout of floats and adjacent content. [\[CSS2\]](#biblio-css2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a83ae7df"></a> Whether an inline-end float originating in subsequent lines must clear the initial letter (as inline-start floats do) is [still under discussion](https://lists.w3.org/Archives/Public/www-style/2018Jul/0019.html). There is no aesthetic reason to require it; however it’s yet unclear how the underlying layout model would distinguish between the two cases.

#### <a id="initial-letter-breaking"></a>7.9.4.  Interaction with Fragmentation (Pagination)

<a id="ref-for-fragment②"></a>

<a id="ref-for-fragmentation-container①"></a>

<a id="ref-for-initial-letter⑥⑧"></a>

<a id="ref-for-monolithic"></a>

<a id="ref-for-fragmentation①"></a>

<a id="ref-for-propdef-widows"></a>

<a id="ref-for-propdef-orphans"></a>

<a id="ref-for-forced-break"></a>

Since a single glyph cannot be [fragmented](https://www.w3.org/TR/css-break-3/#fragment) across pages (or columns or other [fragmentation containers](https://www.w3.org/TR/css-break-4/#fragmentation-container)), an [initial letter](#initial-letter) is considered [monolithic](https://www.w3.org/TR/css-break-3/#monolithic) [\[CSS-BREAK-3\]](#biblio-css-break-3) for the purpose of block-axis [fragmentation](https://www.w3.org/TR/css-break-4/#fragmentation) (breaking across pages, columns, regions, etc.). Additionally, breaks between the in-flow lines alongside an <a id="ref-for-initial-letter⑥⑨"></a>initial letter box are avoided, much as breaks between line boxes affected be [widows](https://www.w3.org/TR/CSS21/page.html#propdef-widows) and [orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans) are avoided. However, if there is a [forced break](https://www.w3.org/TR/css-break-4/#forced-break) alongside the <a id="ref-for-initial-letter⑦⓪"></a>initial letter box, then it takes precedence; but has no effect on the <a id="ref-for-initial-letter⑦①"></a>initial letter box itself.

<a id="ref-for-monolithic①"></a>

<a id="ref-for-initial-letter⑦②"></a>

<a id="ref-for-fragmentation-container②"></a>

<a id="ref-for-fragment③"></a>

As with other [monolithic](https://www.w3.org/TR/css-break-3/#monolithic) objects, if an [initial letter box](#initial-letter) occurs at the top of a [fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container) and that <a id="ref-for-fragmentation-container③"></a>fragmentation container is too short to contain it, it may be either truncated or sliced. Adjacent content, however, must be [fragmented](https://www.w3.org/TR/css-break-3/#fragment) according to its own rules, not truncated or sliced along with the <a id="ref-for-initial-letter⑦③"></a>initial letter.

## <a id="baseline-synthesis"></a> Appendix A: Synthesizing Alignment Metrics

<a id="ref-for-em-over-baseline②"></a>

<a id="ref-for-em-under-baseline②"></a>

### <a id="baseline-synthesis-em"></a> A.1: Calculating [Em-over](#em-over-baseline) and [Em-under](#em-under-baseline)

<a id="ref-for-em-over-baseline③"></a>

<a id="ref-for-em-under-baseline③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [em-over](#em-over-baseline) and [em-under](#em-under-baseline) baselines are not used by CSS. Their definitions are included in this module for consistency with the other metrics used by [Canvas TextMetrics API](https://html.spec.whatwg.org/multipage/canvas.html#textmetrics).

<a id="ref-for-em-over-baseline④"></a>

<a id="ref-for-em-under-baseline④"></a>

The [em-over](#em-over-baseline) and [em-under](#em-under-baseline) metrics are calculated as follows:

- <a id="ref-for-central-baseline④"></a>

  <a id="ref-for-ideographic-over-baseline②"></a>

  <a id="ref-for-ideographic-under-baseline④"></a>

  <a id="ref-for-em-over-baseline⑤"></a>

  <a id="ref-for-em-under-baseline⑤"></a>

  If any one of [central](#central-baseline), [ideographic-over](#ideographic-over-baseline), or [ideographic-under](#ideographic-under-baseline) is defined by the font, then [em-over](#em-over-baseline) is 0.5em over the <a id="ref-for-central-baseline⑤"></a>central baseline, and [em-under](#em-under-baseline) is 0.5em under it, with the <a id="ref-for-central-baseline⑥"></a>central baseline derived from the others if missing or undefined (see below).

- <a id="ref-for-ascent-metric⑥"></a>

  <a id="ref-for-descent-metric⑥"></a>

  <a id="ref-for-em-over-baseline⑥"></a>

  <a id="ref-for-em-under-baseline⑥"></a>

  Otherwise, the [ascent](#ascent-metric) and [descent](#descent-metric) are both proportionally augmented or reduced to add up to exactly 1em, and these normalized metrics are taken as the [em-over](#em-over-baseline) and [em-under](#em-under-baseline) metrics, respectively.

<a id="ref-for-em-over-baseline⑦"></a>

<a id="ref-for-em-under-baseline⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This calculation ensures that [em-over](#em-over-baseline) and [em-under](#em-under-baseline) are always exactly 1em apart while trying to center the glyph outlines’ “center of gravity” between them.

### <a id="baseline-synthesis-fonts"></a> A.2: Synthesizing Baselines (and Other Font Metrics) for Text

Some fonts might not contain the metrics information necessary to align text properly as described in this module. User agents may use the following strategies in the absence of a required metric:

Use related metrics  
Certain metrics are typically related, and this relationship can be used to at least heuristically derive the missing metric. If the font format itself does not define any specific calculations, the following rules may be used:

1.  <a id="ref-for-ideographic-under-baseline⑤"></a>

    <a id="ref-for-ideographic-over-baseline③"></a>

    <a id="ref-for-central-baseline⑦"></a>

    The [central baseline](#central-baseline) is defined to be halfway between the [ideographic-over](#ideographic-over-baseline) and [ideographic-under](#ideographic-under-baseline) baselines, so any two of these determines the third.

2.  <a id="ref-for-central-baseline⑧"></a>

    <a id="ref-for-ideographic-under-baseline⑥"></a>

    <a id="ref-for-ideographic-over-baseline④"></a>

    The [ideographic-over](#ideographic-over-baseline) and [ideographic-under](#ideographic-under-baseline) baselines are typically 1em apart, so if only one of the <a id="ref-for-ideographic-over-baseline⑤"></a>ideographic-over/<a id="ref-for-ideographic-under-baseline⑦"></a>ideographic-under/[central](#central-baseline) baselines are provided, this relation can be used to calculate the other two.

3.  <a id="ref-for-ideographic-under-baseline⑧"></a>

    <a id="ref-for-ideographic-over-baseline⑥"></a>

    <a id="ref-for-descent-metric⑦"></a>

    <a id="ref-for-ascent-metric⑦"></a>

    In CJK fonts the [ascent](#ascent-metric) and [descent](#descent-metric) typically match the [ideographic-over](#ideographic-over-baseline) and [ideographic-under](#ideographic-under-baseline) baselines, so can be used as a fallback when both are missing.

Measure the font  
Metrics may be derived from the glyph shapes. For example,

1.  The center of the minus sign (U+2212) can be taken as the mathematical baseline.

2.  The amount by which the lowercase “o” descends below the alphabetic baseline can be subtracted from its highest point to [measure the x-height](https://drafts.csswg.org/css-values/#ex).
    ![measuring the x height of the letter o](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/measuring-x-height-o.png)
    Measuring the x height.

3.  The amount by which the uppercase “O” descends below the alphabetic baseline can be subtracted from its highest point to measure the cap-height.

4.  The bounding box of 永 (U+6C38) can be used to find the ideographic character face edges.

5.  The top edge of the center of the Hebrew He (U+05D4 “ה”) can be taken as the Hebrew hanging baseline.

6.  <a id="ref-for-content-language①"></a>

    The top edge of the center of the letter Ka can be taken as the hanging baseline. Which Ka is used should depend on the [content language](https://www.w3.org/TR/css-text-4/#content-language):

    

    | Language | Script     | Letter      |
    |----------|------------|-------------|
    |          | Devanagari | क U+0915 KA |
    |          | Bengali    | ক U+0995    |
    |          | Gurmukhi   | ਕ U+0A15    |
    |          | Tibetan    | ཀ U+0F40    |

    

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-2765c39b"></a> Pick a default.

    ![finding the position of the hanging baseline of the letter ka](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/measuring-hanging-baseline-ka.png)
    The hanging baseline is at the top edge of the character ink.

7.  Issue: Add more notes here?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6c88f039"></a> Somebody sanity-check these heuristics please.

Use fallback values  
The following fallback values are suggested:

- x-height: .5em;
- cap-height: .66em;
- hanging baseline: .6em;

### <a id="baseline-synthesis-box"></a> A.3: Synthesizing Baselines for Atomic Inlines

<a id="ref-for-atomic-inline⑨"></a>

<a id="ref-for-replaced-element②"></a>

<a id="ref-for-baseline-set④"></a>

<a id="ref-for-inline-axis⑦"></a>

<a id="ref-for-inline-formatting-context①①"></a>

<a id="ref-for-baseline①⑧"></a>

If an [atomic inline](https://www.w3.org/TR/css-display-3/#atomic-inline) (such as an inline-block, inline-table, or [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element)) does not have a content-derived [baseline set](#baseline-set) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of the [inline formatting context](#inline-formatting-context) in which it participates, then the UA must synthesize its [baselines](#baseline) as follows in order to align it.

<a id="ref-for-baseline①⑨"></a>

<a id="ref-for-line-under⑦"></a>

<a id="ref-for-margin-edge⑤"></a>

These [baselines](#baseline) are assumed to be <strong>at its <a href="https://www.w3.org/TR/css-writing-modes-4/#line-under">line-under</a> <a href="https://www.w3.org/TR/css-box-3/#margin-edge">margin edge</a></strong>:

- <a id="ref-for-text-under-baseline④"></a>

  [text-under baseline](#text-under-baseline)

- <a id="ref-for-ideographic-under-baseline⑨"></a>

  [ideographic-under baseline](#ideographic-under-baseline)

- <a id="ref-for-ideographic-ink-under-baseline③"></a>

  [ideographic-ink-under baseline](#ideographic-ink-under-baseline)

- <a id="ref-for-alphabetic-baseline①③"></a>

  [alphabetic baseline](#alphabetic-baseline)

<a id="ref-for-baseline②⓪"></a>

<a id="ref-for-line-under⑧"></a>

<a id="ref-for-line-over⑦"></a>

<a id="ref-for-margin-edge⑥"></a>

These [baselines](#baseline) are assumed to be <strong>halfway between&#xA;&#x9;its <a href="https://www.w3.org/TR/css-writing-modes-4/#line-under">line-under</a> and <a href="https://www.w3.org/TR/css-writing-modes-4/#line-over">line-over</a> <a href="https://www.w3.org/TR/css-box-3/#margin-edge">margin edges</a></strong>:

- <a id="ref-for-central-baseline⑨"></a>

  [central baseline](#central-baseline)

- <a id="ref-for-math-baseline②"></a>

  [math baseline](#math-baseline)

- <a id="ref-for-x-middle-baseline②"></a>

  [x-middle baseline](#x-middle-baseline)

<a id="ref-for-baseline②①"></a>

<a id="ref-for-line-over⑧"></a>

<a id="ref-for-margin-edge⑦"></a>

These [baselines](#baseline) are assumed to be <strong>at its <a href="https://www.w3.org/TR/css-writing-modes-4/#line-over">line-over</a> <a href="https://www.w3.org/TR/css-box-3/#margin-edge">margin edge</a></strong>:

- <a id="ref-for-text-over-baseline④"></a>

  [text-over baseline](#text-over-baseline)

- <a id="ref-for-ideographic-over-baseline⑦"></a>

  [ideographic-over baseline](#ideographic-over-baseline)

- <a id="ref-for-ideographic-ink-over-baseline③"></a>

  [ideographic-ink-over baseline](#ideographic-ink-over-baseline)

- <a id="ref-for-cap-height-baseline③"></a>

  [cap-height baseline](#cap-height-baseline)

- <a id="ref-for-hanging-baseline③"></a>

  [hanging baseline](#hanging-baseline)

- <a id="ref-for-x-height-baseline④"></a>

  [x-height baseline](#x-height-baseline)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors can use margins (positive or negative) to adjust the alignment of replaced content within a line.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f3a3b459"></a> In this example, the author is using a set of images to display characters that don’t exist.
>
> ```text
> img[src^="/text/"] {
>   height: 1em; /* Size to match adjacent text */
>   margin-bottom: -0.2em; /* Baseline at 20% above bottom */
> }
> ...
> <p>This is some text with words written in an unencoded script:
> <img src="/text/ch3439.png" alt="...">
>   <img src="/text/ch3440.png" alt="...">
>   <img src="/text/ch3442.png" alt="...">
> ```
<a id="ref-for-baseline-table④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level of CSS may include a way of specifying a full [baseline table](#baseline-table) for replaced elements. (This will probably look like a baseline-table property that accepts \[\<baseline-keyword\> \<percentage\>\]+.)

## <a id="changes"></a> Changes

Changes since the [12 August 2024 Working Draft](https://www.w3.org/TR/2024/WD-css-inline-3-20240812/):

- <a id="ref-for-typedef-text-edge④"></a>

  Made both values of [\<text-edge\>](#typedef-text-edge) required. ([Issue 10703](https://github.com/w3c/csswg-drafts/issues/10703))

- <a id="ref-for-propdef-text-box-trim①⑥"></a>

  <a id="ref-for-propdef-text-box-edge①①"></a>

  Made [text-box-edge](#propdef-text-box-edge) inherit; [text-box-trim](#propdef-text-box-trim) references the relevant value applied to the affected line box(es). ([Issue 10904](https://github.com/w3c/csswg-drafts/issues/10904))

- <a id="ref-for-fragmentation②"></a>

  <a id="ref-for-propdef-text-box-trim①⑦"></a>

  Defined behavior of [text-box-trim](#propdef-text-box-trim) at [fragmentation](https://www.w3.org/TR/css-break-4/#fragmentation) breaks. ([Issue 5335](https://github.com/w3c/csswg-drafts/issues/5335))

- <a id="ref-for-multi-column-container③"></a>

  <a id="ref-for-propdef-text-box-trim①⑧"></a>

  Defined behavior of [text-box-trim](#propdef-text-box-trim) on [multi-column containers](https://www.w3.org/TR/css-multicol-1/#multi-column-container), and clarified its application to (and through) other formatting contexts. ([Issue 5335](https://github.com/w3c/csswg-drafts/issues/5335), [Issue 11038](https://github.com/w3c/csswg-drafts/issues/11038))

- <a id="ref-for-phantom-line-box②"></a>

  Renamed “invisible line boxes” to [phantom line boxes](#phantom-line-box) for [consistency with CSS2](https://www.w3.org/TR/CSS2/visuren.html#phantom-line-box) and to help clarify that they are “invisible” to layout, not just painting.

Changes since the [8 August 2024 Working Draft](https://www.w3.org/TR/2024/WD-css-inline-3-20240808/):

- <a id="ref-for-propdef-line-fit-edge②②"></a>

  <a id="ref-for-propdef-text-box-edge①②"></a>

  Some minor clean-up of references to [text-box-edge](#propdef-text-box-edge) left over from when it also represented [line-fit-edge](#propdef-line-fit-edge).

- <a id="ref-for-line-box⑤③"></a>

  <a id="ref-for-propdef-line-fit-edge②③"></a>

  <a id="ref-for-propdef-text-box-edge①③"></a>

  Adjusted [text-box-edge: auto](#propdef-text-box-edge) to reference [line-fit-edge](#propdef-line-fit-edge) on the affected [line box](#line-box) rather than computing to the <a id="ref-for-propdef-line-fit-edge②④"></a>line-fit-edge of the specifying element.

Changes since the [1 April 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-inline-3-20230401/):

- <a id="ref-for-shorthand-property③"></a>

  <a id="ref-for-propdef-text-box③"></a>

  <a id="ref-for-propdef-line-fit-edge②⑤"></a>

  <a id="ref-for-propdef-text-box-trim①⑨"></a>

  <a id="ref-for-propdef-text-box-edge①④"></a>

  Split [text-box-edge](#propdef-text-box-edge) into two properties—​<a id="ref-for-propdef-text-box-edge①⑤"></a>text-box-edge for controlling the [text-box-trim](#propdef-text-box-trim) edge and [line-fit-edge](#propdef-line-fit-edge) for controlling line box sizing—​and added the [text-box](#propdef-text-box) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property). (Issues [8829](https://github.com/w3c/csswg-drafts/issues/8829) and [8696](https://github.com/w3c/csswg-drafts/issues/8696))

- <a id="ref-for-propdef-text-box④"></a>

  <a id="ref-for-propdef-text-box-trim②⓪"></a>

  Added trim-\* prefix to [text-box-trim](#propdef-text-box-trim) keywords so that they make sense in the context of the [text-box](#propdef-text-box) shorthand. ([Issue 10675](https://github.com/w3c/csswg-drafts/issues/10675))

- <a id="ref-for-propdef-text-box-trim②①"></a>

  Use the innermost trim edge for [text-box-trim](#propdef-text-box-trim) when multiple ancestors request trimming. ([Issue 5426](https://github.com/w3c/csswg-drafts/issues/5426))

- <a id="ref-for-layout-bounds①⑥"></a>

  <a id="ref-for-inline-box⑤⓪"></a>

  <a id="ref-for-block-axis①③"></a>

  Apply negative [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) margins to descendants of [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) when calculating their [layout bounds](#layout-bounds) so that they can actually have the specified effect. ([Issue 8182](https://github.com/w3c/csswg-drafts/issues/8182))

- <a id="ref-for-inline-axis⑧"></a>

  <a id="ref-for-phantom-line-box③"></a>

  Corrected [phantom line boxes](#phantom-line-box) to only account for [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) box decorations. ([Issue 9344](https://github.com/w3c/csswg-drafts/issues/9344))

Changes since the [14 November 2022 Working Draft](https://www.w3.org/TR/2022/WD-css-inline-3-20221114/):

- <a id="ref-for-propdef-text-box-trim②②"></a>

  <a id="ref-for-propdef-text-box-edge①⑥"></a>

  Renamed text-edge to [text-box-edge](#propdef-text-box-edge) and leading-trim to [text-box-trim](#propdef-text-box-trim), and also renamed their initial values. ([Issue 8067](https://github.com/w3c/csswg-drafts/issues/8067))

- <a id="ref-for-line-gap-metric③"></a>

  Floor the [line gap metric](#line-gap-metric) at zero. ([Issue 5064](https://github.com/w3c/csswg-drafts/issues/5064))

Changes since the [28 August 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-inline-3-20200827/):

- <a id="ref-for-css-inheritance"></a>

  <a id="ref-for-propdef-inline-sizing③"></a>

  Fixed [inline-sizing](#propdef-inline-sizing) to be [inherited](https://drafts.csswg.org/css-cascade-5/#css-inheritance), as was originally intended. ([Issue 1974](https://github.com/w3c/csswg-drafts/issues/1974))

- <a id="ref-for-propdef-inline-sizing④"></a>

  Update [inline-sizing](#propdef-inline-sizing) “Applies to” to exclude ruby boxes.

- Editorial fixes, including missing images.

Changes since the [18 June 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-inline-3-20200618/) include:

- <a id="ref-for-propdef-initial-letter-align⑦"></a>

  <a id="ref-for-valdef-initial-letter-align-leading"></a>

  Added [leading](#valdef-initial-letter-align-leading) value to [initial-letter-align](#propdef-initial-letter-align) to handle common practices for certain Indic scripts. See [Indic Layout Requirements](https://www.w3.org/TR/ilreq/#h_scripts_without_hanging_baseline). ([Issue 864](https://github.com/w3c/csswg-drafts/issues/864))

- Make non-zero padding and border block effects of leading-trim from ancestors. ([Issue 5237](https://github.com/w3c/csswg-drafts/issues/5237))

- <a id="ref-for-propdef-initial-letter-align⑧"></a>

  <a id="ref-for-hebrew"></a>

  Remove [hebrew](https://www.w3.org/TR/css-counter-styles-3/#hebrew) value from [initial-letter-align](#propdef-initial-letter-align). ([Issue 5208](https://github.com/w3c/csswg-drafts/issues/5208))

- Use the under alignment point for initial letters whose size is less than the sink. ([Issue 5329](https://github.com/w3c/csswg-drafts/issues/5329))

- <a id="ref-for-dropped-initial②"></a>

  Collapse white space adjacent to [dropped initials](#dropped-initial). ([Issue 5120](https://github.com/w3c/csswg-drafts/issues/5120))

- <a id="ref-for-propdef-text-align②"></a>

  <a id="ref-for-raised-initial②"></a>

  <a id="ref-for-dropped-initial③"></a>

  Make [dropped initials](#dropped-initial) behave the same as [raised initials](#raised-initial) for the purpose of [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align). ([Issue 5207](https://github.com/w3c/csswg-drafts/issues/5207))

- <a id="ref-for-propdef-initial-letter-wrap④"></a>

  <a id="ref-for-propdef-shape-outside"></a>

  <a id="ref-for-propdef-margin"></a>

  <a id="ref-for-propdef-shape-margin①"></a>

  Altered interaction of [shape-margin](https://www.w3.org/TR/css-shapes-1/#propdef-shape-margin), [margin](https://www.w3.org/TR/CSS21/box.html#propdef-margin), and [shape-outside](https://www.w3.org/TR/css-shapes-1/#propdef-shape-outside) to match floats (see [initial-letter-wrap](#propdef-initial-letter-wrap)). ([Issue 5119](https://github.com/w3c/csswg-drafts/issues/5119))

- <a id="ref-for-em-under-baseline⑧"></a>

  <a id="ref-for-em-over-baseline⑧"></a>

  Added definitions for [em-over](#em-over-baseline) and [em-under](#em-under-baseline) baselines for reference by Canvas 2D. ([Issue 5312](https://github.com/w3c/csswg-drafts/issues/5312))

- <a id="ref-for-propdef-vertical-align①②"></a>

  Slight refinements to the (new) syntax of [vertical-align](#propdef-vertical-align). ([Issue 5235](https://github.com/w3c/csswg-drafts/issues/5235))

- <a id="ref-for-propdef-baseline-shift①⓪"></a>

  Define fallback shift for [baseline-shift: sub \| super](#propdef-baseline-shift). ([Issue 5225](https://github.com/w3c/csswg-drafts/issues/5225))

Changes since the [4 June 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-inline-3-20200604/) include:

- Reworked the relationship of the earlier line-sizing and text-box-trim proposals to create text-edge and a differently-structured leading-trim. ([Issue 5168](https://github.com/w3c/csswg-drafts/issues/5168))

- <a id="ref-for-propdef-baseline-shift①①"></a>

  <a id="ref-for-propdef-alignment-baseline⑨"></a>

  <a id="ref-for-propdef-vertical-align①③"></a>

  <a id="ref-for-line-relative-shift-values⑤"></a>

  Shifted the [line-relative shift values](#line-relative-shift-values) of [vertical-align](#propdef-vertical-align) from the [alignment-baseline](#propdef-alignment-baseline) longhand to the [baseline-shift](#propdef-baseline-shift) longhand. ([Issue 5180](https://github.com/w3c/csswg-drafts/issues/5180))

- Integrated text-edge into [line box height calculations](#inline-height).

- Refactored definitions of various baselines into [their own section](#css-metrics) and imported introduction and core terminology from [\[CSS-WRITING-MODES-3\]](#biblio-css-writing-modes-3).

- Imported and updated / integrated remaining baseline alignment and line box sizing prose from [\[CSS2\]](#biblio-css2).

- Defined atomic inline baseline synthesis rules for all baselines.

- <a id="ref-for-central-baseline①⓪"></a>

  Defined the [central baseline](#central-baseline) definitively as the <em>ideographic</em> central baseline.

- <a id="ref-for-initial-letter⑦④"></a>

  Defined white space collapsing between an [initial letter box](#initial-letter) and subsequent text. ([Issue 5120](https://github.com/w3c/csswg-drafts/issues/5120))

- <a id="ref-for-containing-block①①"></a>

  <a id="ref-for-initial-letter⑦⑤"></a>

  Tightened up box model definitions for [initial letter boxes](#initial-letter), including interaction with its [containing block](https://www.w3.org/TR/css-display-3/#containing-block). ([Issue 719](https://github.com/w3c/csswg-drafts/issues/719))

- Miscellaneous small fixes, clarifications, and editorial improvements.

Changes since the [8 August 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-inline-3-20180808/) include:

- Added line-sizing property to control how inter-line spacing is calculated. ([Issue 3199](https://github.com/w3c/csswg-drafts/issues/3199))

- <a id="ref-for-propdef-baseline-source⑤"></a>

  Added [baseline-source](#propdef-baseline-source) property to control whether first or last baseline is used for alignment. ([Issue 861](https://github.com/w3c/csswg-drafts/issues/861))

- Added leading-trim proposal to control the metrics used for the line-over/line-under edge in line box layout. ([Issue 3240](https://github.com/w3c/csswg-drafts/issues/3240) and [3955](https://github.com/w3c/csswg-drafts/issues/3955))

- <a id="ref-for-propdef-line-height②④"></a>

  Imported [line-height](#propdef-line-height) definition and related normative prose from [\[CSS2\]](#biblio-css2).

- Improved high-level description of inline layout in [§ 2 Inline Layout Model](#model).

- <a id="ref-for-propdef-initial-letter②⑥"></a>

  Renamed initial-letters back to [initial-letter](#propdef-initial-letter). ([Issue 862](https://github.com/w3c/csswg-drafts/issues/862))

- <a id="ref-for-valdef-initial-letter-raise"></a>

  Added the [raise](#valdef-initial-letter-raise) and sink keywords for syntactic convenience. ([Issue 2955](https://github.com/w3c/csswg-drafts/issues/2955))

- <a id="ref-for-atomic-inline①⓪"></a>

  Specified synthesis of baselines for [atomic inlines](https://www.w3.org/TR/css-display-3/#atomic-inline) that have no baseline set.

- <a id="ref-for-vertical-writing-mode③"></a>

  <a id="ref-for-valdef-alignment-baseline-text-bottom①"></a>

  <a id="ref-for-valdef-alignment-baseline-text-top①"></a>

  <a id="ref-for-valdef-alignment-baseline-middle①"></a>

  Clarified interpretation of [middle](#valdef-alignment-baseline-middle), [text-top](#valdef-alignment-baseline-text-top), and [text-bottom](#valdef-alignment-baseline-text-bottom) in [vertical writing modes](https://www.w3.org/TR/css-writing-modes-4/#vertical-writing-mode). ([Issue 4495](https://github.com/w3c/csswg-drafts/issues/4495))

- <a id="ref-for-propdef-dominant-baseline④"></a>

  <a id="ref-for-propdef-vertical-align①④"></a>

  <a id="ref-for-valdef-alignment-baseline-text-bottom②"></a>

  <a id="ref-for-valdef-alignment-baseline-text-top②"></a>

  Clarified that [text-top](#valdef-alignment-baseline-text-top)/[text-bottom](#valdef-alignment-baseline-text-bottom)/text values should be consistently interpreted across [vertical-align](#propdef-vertical-align), [dominant-baseline](#propdef-dominant-baseline), leading-trim, and drawing the content box of an inline box. ([Issue 3978](https://github.com/w3c/csswg-drafts/issues/3978))

- <a id="ref-for-valdef-dominant-baseline-auto"></a>

  <a id="ref-for-propdef-dominant-baseline⑤"></a>

  Corrected initial value of [dominant-baseline](#propdef-dominant-baseline) to [auto](#valdef-dominant-baseline-auto). ([Issue 4115](https://github.com/w3c/csswg-drafts/issues/4115))

- <a id="ref-for-propdef-vertical-align①⑤"></a>

  Improved some nuances in authoring advice regarding [vertical-align](#propdef-vertical-align) longhands vs. shorthands.

- <a id="ref-for-propdef-position①"></a>

  <a id="ref-for-propdef-float①"></a>

  <a id="ref-for-propdef-initial-letter②⑦"></a>

  Clarified interaction of [initial-letter](#propdef-initial-letter) and [float](https://drafts.csswg.org/css2/#propdef-float)/[position](https://www.w3.org/TR/CSS21/visuren.html#propdef-position).

- Reorganized the [§ 7.5 Initial Letter Layout](#initial-letter-layout) section for better readability, and tweaked some wording for clarity.

- <a id="ref-for-propdef-shape-margin②"></a>

  Defined that [shape-margin](https://www.w3.org/TR/css-shapes-1/#propdef-shape-margin) applies to the glyph outline.

- Switched baseline synthesis rules to use 永 (U+6C38) for ideographic face edges.

- <a id="ref-for-initial-letter⑦⑥"></a>

  Specified that an [initial letter](#initial-letter) is isolated wrt shaping, even though text after it remains in its connecting form. ([Issue 2399](https://github.com/w3c/csswg-drafts/issues/2399#issuecomment-635630662))

See also earlier [changes since the 24 May 2016 Working Draft](https://www.w3.org/TR/2018/WD-css-inline-3-20180808/).

## <a id="ack"></a> Acknowledgments

Special thanks goes to the initial authors, Eric A. Meyer and Michel Suignard.

In additions to the authors, this specification would not have been possible without the help from:

David Baron, Mike Bremford, David M Brown, Oriol Brufau, John Daggett, Stephen Deach, Sylvain Galineau, David Hyatt, Myles Maxfield, Shinyu Murakami, Jan Nicklas, Tess O’Connor, Sujal Parikh, Florian Rivoal, Alan Stearns, Weston Thayer, Bobby Tung, Chris Wilson, Grzegorz Zygmunt.

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

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

- [aligned subtree](#aligned-subtree), in § 4.2.3
- [alignment baseline](#alignment-baseline), in § 4.2.2
- [alignment-baseline](#propdef-alignment-baseline), in § 4.2.2
- [all](#valdef-initial-letter-wrap-all), in § 7.7
- alphabetic
  - [definition of](#alphabetic-baseline), in § 3.2
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-alphabetic), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-alphabetic), in § 4.1
  - [value for initial-letter-align](#valdef-initial-letter-align-alphabetic), in § 7.4
  - [value for line-fit-edge, \<\<text-edge\>\>](#valdef-line-fit-edge-alphabetic), in § 5.2
- [alphabetic baseline](#alphabetic-baseline), in § 3.2
- [ascent](#ascent-metric), in § 3.2.1
- [ascent metric](#ascent-metric), in § 3.2.1
- [atomic initial letter](#atomic-initial-letter), in § 7.5
- [atomic initial letter box](#atomic-initial-letter), in § 7.5
- auto
  - [value for baseline-source, vertical-align](#valdef-baseline-source-auto), in § 4.2.1
  - [value for dominant-baseline](#valdef-dominant-baseline-auto), in § 4.1
  - [value for text-box-edge](#valdef-text-box-edge-auto), in § 6.3
- baseline
  - [definition of](#baseline), in § 3.1
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-baseline), in § 4.2.2
  - [value for baseline-shift](#valdef-baseline-shift-baseline), in § 4.2.3.1
- [baseline alignment preference](#baseline-alignment-preference), in § 4.2.1
- [baseline-relative shift values](#baseline-relative-shift-values), in § 4.2.3
- [baseline-relative values](#baseline-relative-shift-values), in § 4.2.3
- [baseline set](#baseline-set), in § 3.3
- [baseline-shift](#propdef-baseline-shift), in § 4.2.3
- [baseline-source](#propdef-baseline-source), in § 4.2.1
- [baseline table](#baseline-table), in § 3.1
- [border-box](#valdef-initial-letter-align-border-box), in § 7.4
- [bottom](#valdef-baseline-shift-bottom), in § 4.2.3
- [cap](#valdef-line-fit-edge-cap), in § 5.2
- [cap-height](#cap-height-baseline), in § 3.2
- [cap-height baseline](#cap-height-baseline), in § 3.2
- [center](#valdef-baseline-shift-center), in § 4.2.3
- central
  - [definition of](#central-baseline), in § 3.2
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-central), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-central), in § 4.1
- [central baseline](#central-baseline), in § 3.2
- [descent](#descent-metric), in § 3.2.1
- [descent metric](#descent-metric), in § 3.2.1
- [dominant baseline](#dominant-baseline), in § 4.1
- [dominant-baseline](#propdef-dominant-baseline), in § 4.1
- [drop](#valdef-initial-letter-drop), in § 7.3
- [dropped initial](#dropped-initial), in § 7.1.1
- [em-over](#em-over-baseline), in § 3.2
- [em-over baseline](#em-over-baseline), in § 3.2
- [em-under](#em-under-baseline), in § 3.2
- [em-under baseline](#em-under-baseline), in § 3.2
- [ex](#valdef-line-fit-edge-ex), in § 5.2
- first
  - [value for baseline-source, vertical-align](#valdef-baseline-source-first), in § 4.2.1
  - [value for initial-letter-wrap](#valdef-initial-letter-wrap-first), in § 7.7
- [grid](#valdef-initial-letter-wrap-grid), in § 7.7
- [half-leading](#half-leading), in § 5.3
- hanging
  - [definition of](#hanging-baseline), in § 3.2
  - [value for dominant-baseline](#valdef-dominant-baseline-hanging), in § 4.1
  - [value for initial-letter-align](#valdef-initial-letter-align-hanging), in § 7.4
- [hanging baseline](#hanging-baseline), in § 3.2
- ideographic
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-ideographic), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-ideographic), in § 4.1
  - [value for initial-letter-align](#valdef-initial-letter-align-ideographic), in § 7.4
  - [value for line-fit-edge, \<\<text-edge\>\>](#valdef-line-fit-edge-ideographic), in § 5.2
- [ideographic-ink](#valdef-line-fit-edge-ideographic-ink), in § 5.2
- [ideographic-ink-over](#ideographic-ink-over-baseline), in § 3.2
- [ideographic-ink-over baseline](#ideographic-ink-over-baseline), in § 3.2
- [ideographic-ink-under](#ideographic-ink-under-baseline), in § 3.2
- [ideographic-ink-under baseline](#ideographic-ink-under-baseline), in § 3.2
- [ideographic-over](#ideographic-over-baseline), in § 3.2
- [ideographic-over baseline](#ideographic-over-baseline), in § 3.2
- [ideographic-under](#ideographic-under-baseline), in § 3.2
- [ideographic-under baseline](#ideographic-under-baseline), in § 3.2
- [initial letter](#initial-letter), in § 7.3
- [initial-letter](#propdef-initial-letter), in § 7.3
- [initial-letter-align](#propdef-initial-letter-align), in § 7.4
- [initial letter box](#initial-letter), in § 7.3
- [initial letter sink](#initial-letter-initial-letter-sink), in § 7.3
- [initial letter size](#initial-letter-initial-letter-size), in § 7.3
- [initial-letter-wrap](#propdef-initial-letter-wrap), in § 7.7
- [inline formatting context](#inline-formatting-context), in § 2
- [inline initial letter](#inline-initial-letter), in § 7.5
- [inline initial letter box](#inline-initial-letter), in § 7.5
- [inline layout](#inline-layout), in § 2
- [inline-sizing](#propdef-inline-sizing), in § 6.4
- [\<integer \[1,∞\]\>](#valdef-initial-letter-integer-1), in § 7.3
- [last](#valdef-baseline-source-last), in § 4.2.1
- [layout bounds](#layout-bounds), in § 5.3
- leading
  - [definition of](#leading), in § 5.3
  - [value for initial-letter-align](#valdef-initial-letter-align-leading), in § 7.4
  - [value for line-fit-edge, \<\<text-edge\>\>](#valdef-line-fit-edge-leading), in § 5.2
- \<length\>
  - [value for baseline-shift, vertical-align](#valdef-baseline-shift-length), in § 4.2.3
  - [value for initial-letter-wrap](#valdef-initial-letter-wrap-length), in § 7.7
- [\<length \[0,∞\]\>](#valdef-line-height-length-0), in § 5.1
- [line box](#line-box), in § 2
- [line-fit-edge](#propdef-line-fit-edge), in § 5.2
- [line gap metric](#line-gap-metric), in § 3.2.2
- [line-height](#propdef-line-height), in § 5.1
- [line-relative shift values](#line-relative-shift-values), in § 4.2.3
- [line-relative values](#line-relative-shift-values), in § 4.2.3
- [math](#math-baseline), in § 3.2
- [math baseline](#math-baseline), in § 3.2
- mathematical
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-mathematical), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-mathematical), in § 4.1
- middle
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-middle), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-middle), in § 4.1
- none
  - [value for initial-letter-wrap](#valdef-initial-letter-wrap-none), in § 7.7
  - [value for text-box-trim](#valdef-text-box-trim-none), in § 6.2
- normal
  - [value for initial-letter](#valdef-initial-letter-normal), in § 7.3
  - [value for inline-sizing](#valdef-inline-sizing-normal), in § 6.4
  - [value for line-height](#valdef-line-height-normal), in § 5.1
  - [value for text-box](#valdef-text-box-normal), in § 6.1
- [\<number \[0,∞\]\>](#valdef-line-height-number-0), in § 5.1
- [\<number \[1,∞\]\>](#valdef-initial-letter-number-1), in § 7.3
- [originate](#originating-line), in § 7.8
- [originating line](#originating-line), in § 7.8
- [originating line box](#originating-line), in § 7.8
- \<percentage\>
  - [value for baseline-shift, vertical-align](#valdef-baseline-shift-percentage), in § 4.2.3
  - [value for initial-letter-wrap](#valdef-initial-letter-wrap-percentage), in § 7.7
- [\<percentage \[0,∞\]\>](#valdef-line-height-percentage-0), in § 5.1
- [phantom line box](#phantom-line-box), in § 2.3
- [post-alignment shift](#post-alignment-shift), in § 4.2.3
- [preferred line height](#preferred-line-height), in § 5.1
- [raise](#valdef-initial-letter-raise), in § 7.3
- [raised initial](#raised-initial), in § 7.1.3
- [root inline box](#root-inline-box), in § 2
- [sink](#initial-letter-initial-letter-sink), in § 7.3
- [stretch](#valdef-inline-sizing-stretch), in § 6.4
- [sub](#valdef-baseline-shift-sub), in § 4.2.3
- [sunken initial](#sunken-initial), in § 7.1.2
- [super](#valdef-baseline-shift-super), in § 4.2.3
- [text](#valdef-line-fit-edge-text), in § 5.2
- [text-after-edge](#valdef-alignment-baseline-text-after-edge), in § 4.2.2.1
- [text-before-edge](#valdef-alignment-baseline-text-before-edge), in § 4.2.2.1
- text-bottom
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-text-bottom), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-text-bottom), in § 4.1
- [text-box](#propdef-text-box), in § 6.1
- [text-box-edge](#propdef-text-box-edge), in § 6.3
- [text-box-trim](#propdef-text-box-trim), in § 6.2
- [\<text-edge\>](#typedef-text-edge), in § 5.2
- [text-over](#text-over-baseline), in § 3.2
- [text-over baseline](#text-over-baseline), in § 3.2
- text-top
  - [value for alignment-baseline, vertical-align](#valdef-alignment-baseline-text-top), in § 4.2.2
  - [value for dominant-baseline](#valdef-dominant-baseline-text-top), in § 4.1
- [text-under](#text-under-baseline), in § 3.2
- [text-under baseline](#text-under-baseline), in § 3.2
- [top](#valdef-baseline-shift-top), in § 4.2.3
- [trim-both](#valdef-text-box-trim-trim-both), in § 6.2
- [trim-end](#valdef-text-box-trim-trim-end), in § 6.2
- [trim-start](#valdef-text-box-trim-trim-start), in § 6.2
- [vertical-align](#propdef-vertical-align), in § 4.2
- [x-height](#x-height-baseline), in § 3.2
- [x-height baseline](#x-height-baseline), in § 3.2
- [x-middle](#x-middle-baseline), in § 3.2
- [x-middle baseline](#x-middle-baseline), in § 3.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fde954ee"></a>align-content
  - <a id="0e506341"></a>alignment context
  - <a id="4d2cf2cf"></a>baseline alignment
  - <a id="515ec31f"></a>center
  - <a id="020c3333"></a>end
  - <a id="78aede31"></a>first baseline set
  - <a id="950a691a"></a>first-baseline alignment
  - <a id="80d2b689"></a>justify-self
  - <a id="cd78c41b"></a>last baseline set
  - <a id="15d5b5ce"></a>last-baseline alignment
  - <a id="50284573"></a>normal
  - <a id="35e80fd9"></a>start
  - <a id="81f3a960"></a>synthesize baseline
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="c48eaa20"></a>box-shadow
- \[CSS-BOX-3\] defines the following terms:
  - <a id="f98c718d"></a>border edge
  - <a id="4016198d"></a>content edge
  - <a id="fe3bf782"></a>margin box
  - <a id="4346f556"></a>margin edge
- \[CSS-BOX-4\] defines the following terms:
  - <a id="30e036e4"></a>border
  - <a id="60669dde"></a>box edge
  - <a id="df86efcb"></a>content area
  - <a id="6e7a78f3"></a>edge
  - <a id="d049494a"></a>margin
  - <a id="a2be8c84"></a>padding
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="62c772f5"></a>fragment
  - <a id="4ea821fb"></a>monolithic
  - <a id="4f75e4ec"></a>orphans
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="a0542bba"></a>box-decoration-break
  - <a id="2218854e"></a>forced break
  - <a id="04004305"></a>fragmentation
  - <a id="4904f647"></a>fragmentation container
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="21ec9802"></a>inherit
  - <a id="36261173"></a>longhand
  - <a id="e14541aa"></a>shorthand
  - <a id="b49aeda5"></a>sub-property
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="e3b56e79"></a>hebrew
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="61c422b4"></a>anonymous box
  - <a id="f2938b9c"></a>atomic inline
  - <a id="ea628b01"></a>atomic inline box
  - <a id="9680aa9c"></a>block
  - <a id="05c40e8e"></a>block container
  - <a id="87e40879"></a>block container box
  - <a id="e4f1fc8b"></a>block formatting context
  - <a id="5c159f8f"></a>box
  - <a id="79d4bbff"></a>box tree
  - <a id="6b4fc208"></a>containing block
  - <a id="2ccfe434"></a>display
  - <a id="a502c18f"></a>document order
  - <a id="43fd67c9"></a>formatting context
  - <a id="e8976716"></a>in-flow
  - <a id="b091c3a0"></a>independent formatting context
  - <a id="c7cc6301"></a>inline block
  - <a id="d41bfa8c"></a>inline box
  - <a id="2351701b"></a>inline-block
  - <a id="4f918eb5"></a>inline-level
  - <a id="aab2607b"></a>inline-level box
  - <a id="0ab5f38d"></a>inline-level content
  - <a id="06fd3b4c"></a>out-of-flow
  - <a id="299e10e4"></a>replaced element
  - <a id="7ea71f53"></a>text sequence
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="5da00747"></a>first available font
  - <a id="9ce65b41"></a>font-language-override
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="1e41af8f"></a>inside
- \[CSS-MULTICOL-1\] defines the following terms:
  - <a id="3126ae25"></a>column box
  - <a id="825824a2"></a>multi-column container
  - <a id="b339ce4c"></a>multi-column layout
- \[CSS-PAGE-FLOATS-3\] defines the following terms:
  - <a id="06eea7ac"></a>float
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="dec20430"></a>absolutely positioned box
  - <a id="6bef2f05"></a>positioned box
  - <a id="35f1d972"></a>static
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="b6b63ba4"></a>::marker
  - <a id="99a0ef70"></a>first formatted line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="123f21cb"></a>internal ruby boxes
  - <a id="42a3c4cb"></a>ruby annotation
  - <a id="2f1d12f3"></a>ruby base container box
  - <a id="b07db6cc"></a>ruby container box
- \[CSS-SHAPES-1\] defines the following terms:
  - <a id="148eb1d2"></a>none
  - <a id="730a72b3"></a>shape-margin
  - <a id="81e9414b"></a>shape-outside
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="37f6dbd7"></a>automatic size
  - <a id="54a1fea8"></a>box-sizing
  - <a id="66f218c1"></a>definite
  - <a id="5ad01cca"></a>height
  - <a id="de48a940"></a>inner size
  - <a id="1b6b4591"></a>max size
  - <a id="2355631a"></a>min size
  - <a id="47ea2436"></a>outer size
  - <a id="a229530b"></a>preferred height
  - <a id="22dc0499"></a>preferred width
  - <a id="2ac08cff"></a>sizing property
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="ee69494f"></a>forced line break
  - <a id="34ede11b"></a>hang
  - <a id="36e5f32e"></a>text-align
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="f7e97fd7"></a>collapsible white space
  - <a id="219a15e3"></a>content language
  - <a id="9eb5c2af"></a>document white space
  - <a id="276c9ab1"></a>forced line break
  - <a id="b3b49700"></a>hanging-punctuation
  - <a id="e3f6386a"></a>justification opportunity
  - <a id="131c8fac"></a>preserved white space
  - <a id="24e3d3d6"></a>typographic character unit
  - <a id="1a7375ef"></a>word separator
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="81b3af3e"></a>!
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="d73c993d"></a>\<integer\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="eefce2af"></a>em
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="7922a8cf"></a>block end
  - <a id="ddf25d36"></a>block flow direction
  - <a id="ecef1eb5"></a>block size
  - <a id="599428b5"></a>block-axis
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="49eecea3"></a>horizontal writing mode
  - <a id="a6eb24bb"></a>inline axis
  - <a id="18bb1084"></a>inline size
  - <a id="82ddda8c"></a>inline-axis
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="4f19c3e6"></a>line-left
  - <a id="0ad9204c"></a>line-over
  - <a id="10d0d189"></a>line-right
  - <a id="401cafe5"></a>line-under
  - <a id="e1b94ba0"></a>logical height
  - <a id="99a9e10b"></a>logical width
  - <a id="f5e69024"></a>mixed
  - <a id="9d51c0ac"></a>over
  - <a id="8804aba6"></a>sideways
  - <a id="90c7548c"></a>start
  - <a id="8664e85f"></a>text-orientation
  - <a id="ef383707"></a>typographic mode
  - <a id="99d814d1"></a>under
  - <a id="cec0d4db"></a>upright
  - <a id="e51c8aeb"></a>vertical writing mode
  - <a id="35f596e9"></a>vertical-lr
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="ab2f3e73"></a>computed value
  - <a id="088aa1a3"></a>content box
  - <a id="4b35980d"></a>font-size
  - <a id="640e68b1"></a>initial value
  - <a id="504a511e"></a>letter-spacing
  - <a id="5515c98f"></a>margin
  - <a id="708ccaca"></a>outline
  - <a id="dc76e65c"></a>position
  - <a id="9ce36abc"></a>specified value
  - <a id="627d7057"></a>text-indent
  - <a id="503c5c26"></a>used value
  - <a id="45439228"></a>widows
  - <a id="18e89fe6"></a>word-spacing
  - <a id="5455396f"></a>z-index
- \[CSS22\] defines the following terms:
  - <a id="9436e460"></a>clear
  - <a id="0570259e"></a>float
- \[SVG2\] defines the following terms:
  - <a id="08c40d52"></a>current text position
  - <a id="fada422e"></a>text content element

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 11 April 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 27 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-page-floats-3"></a>\[CSS-PAGE-FLOATS-3\]  
Johannes Wilm. [CSS Page Floats](https://www.w3.org/TR/css-page-floats-3/). 15 September 2015. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-floats-3&#x2F;](https://www.w3.org/TR/css-page-floats-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 10 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-shapes-1"></a>\[CSS-SHAPES-1\]  
Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 15 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 30 September 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 29 May 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-quirks"></a>\[QUIRKS\]  
Simon Pieters. [Quirks Mode Standard](https://quirks.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;quirks&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://quirks.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-line-grid-1"></a>\[CSS-LINE-GRID-1\]  
Elika Etemad; Koji Ishii; Alan Stearns. [CSS Line Grid Module Level 1](https://www.w3.org/TR/css-line-grid-1/). 16 September 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-line-grid-1&#x2F;](https://www.w3.org/TR/css-line-grid-1/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-ilreq"></a>\[ILREQ\]  
Swaran Lata. [Indic Layout Requirements](https://www.w3.org/TR/ilreq/). 29 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;ilreq&#x2F;](https://www.w3.org/TR/ilreq/)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                                                      | Initial    | Applies to                                                                                                  | Inh. | %ages                                                          | Anim­ation type            | Canonical order | Com­puted value                                                  |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------------------|------|----------------------------------------------------------------|---------------------------|-----------------|-----------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-alignment-baseline①⓪"></a></span><a href="#propdef-alignment-baseline">alignment-baseline</a>&#xA;      </strong> | baseline \| text-bottom \| alphabetic \| ideographic \| middle \| central \| mathematical \| text-top                      | baseline   | inline-level boxes, flex items, grid items, table cells, and SVG text content elements                      | no   | N/A                                                            | discrete                  | per grammar     | specified keyword                                               |
| <strong><span><a id="ref-for-propdef-baseline-shift①②"></a></span><a href="#propdef-baseline-shift">baseline-shift</a>&#xA;      </strong> | \<length-percentage\> \| sub \| super \| top \| center \| bottom                                                           | 0          | inline-level boxes and SVG text content elements                                                            | no   | refer to the used value of line-height                         | by computed value type    | per grammar     | the specified keyword or a computed \<length-percentage\> value |
| <strong><span><a id="ref-for-propdef-baseline-source⑥"></a></span><a href="#propdef-baseline-source">baseline-source</a>&#xA;      </strong> | auto \| first \| last                                                                                                      | auto       | inline-level boxes                                                                                          | no   | N/A                                                            | discrete                  | per grammar     | specified keyword                                               |
| <strong><span><a id="ref-for-propdef-dominant-baseline⑥"></a></span><a href="#propdef-dominant-baseline">dominant-baseline</a>&#xA;      </strong> | auto \| text-bottom \| alphabetic \| ideographic \| middle \| central \| mathematical \| hanging \| text-top               | auto       | block containers, inline boxes, table rows, grid containers, flex containers, and SVG text content elements | yes  | N/A                                                            | discrete                  | per grammar     | specified keyword                                               |
| <strong><span><a id="ref-for-propdef-initial-letter②⑧"></a></span><a href="#propdef-initial-letter">initial-letter</a>&#xA;      </strong> | normal \| \<number \[1,∞\]\> \<integer \[1,∞\]\> \| \<number \[1,∞\]\> &#x26;&#x26; \[ drop \| raise \]? | normal     | certain inline-level boxes and ::first-letter and inside ::marker boxes (see prose)                         | no   | N/A                                                            | by computed value type    | per grammar     | the keyword normal or a number paired with an integer           |
| <strong><span><a id="ref-for-propdef-initial-letter-align⑨"></a></span><a href="#propdef-initial-letter-align">initial-letter-align</a>&#xA;      </strong> | \[ border-box? \[ alphabetic \| ideographic \| hanging \| leading \]? \]!                                                  | alphabetic | certain inline-level boxes and ::first-letter and inside ::marker boxes (see prose)                         | yes  | N/A                                                            | discrete                  | per grammar     | specified keyword(s)                                            |
| <strong><span><a id="ref-for-propdef-initial-letter-wrap⑤"></a></span><a href="#propdef-initial-letter-wrap">initial-letter-wrap</a>&#xA;      </strong> | none \| first \| all \| grid \| \<length-percentage\>                                                                      | none       | certain inline-level boxes and ::first-letter and inside ::marker boxes (see prose)                         | yes  | relative to logical width of (last fragment of) initial letter | by computed value type    | per grammar     | specified keyword or computed \<length-percentage\> value       |
| <strong><span><a id="ref-for-propdef-inline-sizing⑤"></a></span><a href="#propdef-inline-sizing">inline-sizing</a>&#xA;      </strong> | normal \| stretch                                                                                                          | normal     | inline boxes, but not ruby container boxes nor internal ruby boxes                                          | yes  | n/a                                                            | discrete                  | per grammar     | specified keyword                                               |
| <strong><span><a id="ref-for-propdef-line-fit-edge②⑥"></a></span><a href="#propdef-line-fit-edge">line-fit-edge</a>&#xA;      </strong> | leading \| \<text-edge\>                                                                                                   | leading    | inline boxes                                                                                                | yes  | N/A                                                            | discrete                  | per grammar     | the specified keyword                                           |
| <strong><span><a id="ref-for-propdef-line-height②⑤"></a></span><a href="#propdef-line-height">line-height</a>&#xA;      </strong> | normal \| \<number \[0,∞\]\> \| \<length-percentage \[0,∞\]\>                                                              | normal     | non-replaced inline boxes and SVG text content elements                                                     | yes  | computed relative to 1em                                       | by computed value type    | per grammar     | the specified keyword, a number, or a computed \<length\> value |
| <strong><span><a id="ref-for-propdef-text-box⑤"></a></span><a href="#propdef-text-box">text-box</a>&#xA;      </strong> | normal \| \<'text-box-trim'\> \|\| \<'text-box-edge'\>                                                                     | normal     | block containers and inline boxes                                                                           | no   | N/A                                                            | discrete                  | per grammar     | the specified keyword                                           |
| <strong><span><a id="ref-for-propdef-text-box-edge①⑦"></a></span><a href="#propdef-text-box-edge">text-box-edge</a>&#xA;      </strong> | auto \| \<text-edge\>                                                                                                      | auto       | block containers and inline boxes                                                                           | yes  | N/A                                                            | discrete                  | per grammar     | the specified keyword                                           |
| <strong><span><a id="ref-for-propdef-text-box-trim②③"></a></span><a href="#propdef-text-box-trim">text-box-trim</a>&#xA;      </strong> | none \| trim-start \| trim-end \| trim-both                                                                                | none       | block containers and inline boxes                                                                           | no   | N/A                                                            | discrete                  | per grammar     | the specified keyword                                           |
| <strong><span><a id="ref-for-propdef-vertical-align①⑥"></a></span><a href="#propdef-vertical-align">vertical-align</a>&#xA;      </strong> | \[ first \| last\] \|\| \<'alignment-baseline'\> \|\| \<'baseline-shift'\>                                                 | baseline   | see individual properties                                                                                   | no   | N/A                                                            | see individual properties | per grammar     | see individual properties                                       |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Many aspects of layout here depend on font metrics. While the relevant metrics exist in OpenType for Latin/Cyrillic/Greek and for CJK, they are missing for many other writing systems. For example, the visual top metric for Hebrew has no metric in the OpenType tables. For this module to work well for the world, we need fonts to provide the relevant metrics for all writing systems, and that means both that OpenType needs to allow such metrics and font designers need to provide accurate numbers. See [issue](https://github.com/w3c/csswg-drafts/issues/5244) and [liaison statement](https://lists.w3.org/Archives/Public/www-archive/2020Feb/att-0005/CSS-SC29-20200113.pdf). [↵](#issue-4c501384)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define what to do for top/bottom/center aligned boxes that are taller than the rest of the content. [↵](#issue-e2d22611)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Firefox allows the inline boxes within a [phantom line box](#phantom-line-box) to accept [outline](https://www.w3.org/TR/CSS21/ui.html#propdef-outline),which allows it to make focus rings visible. As in other browsers, all other properties that could make the element visible (e.g. [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow)) seem to be ignored. [↵](#issue-8b05a2ec)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG would like to know which baseline values are necessary for each property that uses them ([dominant-baseline](#propdef-dominant-baseline), [alignment-baseline](#propdef-alignment-baseline), [text-box-edge](#propdef-text-box-edge), [line-fit-edge](#propdef-line-fit-edge), [initial-letter-align](#propdef-initial-letter-align)): if any can be dropped, or any need to be added. See [Issue 859](https://github.com/w3c/csswg-drafts/issues/859). [↵](#issue-3bc5ecf6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define behavior for mixed vertical orientations that isn’t nonsensical when specified baseline isn’t [central](#valdef-dominant-baseline-central). [↵](#issue-2ffa7534)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The [line-relative shift values](#line-relative-shift-values) don’t fit perfectly in the dichotomy between [alignment-baseline](#propdef-alignment-baseline) and [baseline-shift](#propdef-baseline-shift). There’s [decent](https://github.com/w3c/csswg-drafts/issues/5180) [arguments](https://github.com/w3c/csswg-drafts/issues/5234) for either option. They’re currently drafted here, but if there’s a strong argument to move them, please file an issue for consideration. [↵](#issue-8910737b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We would prefer to remove the [baseline](#valdef-baseline-shift-baseline) value, and are looking for feedback from SVG user agents as to whether it’s necessary. [↵](#issue-6cbc9542)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The fact that percentages compute to lengths is annoying. See also [Issue 3118](https://github.com/w3c/csswg-drafts/issues/3118) and [Issue 2165](https://github.com/w3c/csswg-drafts/issues/2165). [↵](#issue-af10b7fe)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is an early draft of a proposal, and might change significantly as design critiques and use cases are registered and various details and interactions with other properties are worked out. <strong>Do not ship (yet).</strong> [↵](#issue-6de82a32)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) or is this shorthand enough? [\[Issue \#5236\]](https://github.com/w3c/csswg-drafts/issues/5236) [↵](#issue-eba283e1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is [text](#valdef-line-fit-edge-text) a reasonable name for the ascent/descent metrics, or can we think of something better? Ditto [leading](#valdef-line-fit-edge-leading) as a keyword. [\[Issue \#8067\]](https://github.com/w3c/csswg-drafts/issues/8067) [↵](#issue-d656b674)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This illustration doesn’t match actual font metrics, it’s actually illustrating the cap-height, not the ascent. [\[Issue \#11364\]](https://github.com/w3c/csswg-drafts/issues/11364) [↵](#issue-5f46f8ae)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add examples. [↵](#issue-6e028676)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What happens if the column is split by a spanner? [\[Issue \#11363\]](https://github.com/w3c/csswg-drafts/issues/11363) [↵](#issue-d1836b7a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This has a confusing name. We need a new name. Alternatively, incorporate this into [text-box-trim](#propdef-text-box-trim)? [\[Issue \#5189\]](https://github.com/w3c/csswg-drafts/issues/5189) [↵](#issue-2c8f414b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The editors would appreciate any examples of drop initials in non-western scripts, especially Indic scripts. [↵](#issue-81b8cb9c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should there be a way to opt out of this behavior? See [GitHub Issue 310](https://github.com/w3c/csswg-drafts/issues/310). [↵](#issue-8e10cdbe)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Correct alignment of initial letter in scripts such as Hebrew and Thai is currently not possible because OpenType lacks corresponding metrics. ([Issue 5244](https://github.com/w3c/csswg-drafts/issues/5244))
>
> ![Hebrew 2-line drop-letter alignment using the hebrew-top and alphabetic baselines](https://www.w3.org/TR/2024/WD-css-inline-3-20241218/images/hebrew-initial-letter.png)
>
> [↵](#issue-9175a33b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This only covers the most common cross-linguistic transcription systems. Should we include any other / all script tags in the UA style sheet? [↵](#issue-fbd38850)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What about inheritance to descendants? [\[Issue \#4988\]](https://github.com/w3c/csswg-drafts/issues/4988) [↵](#issue-c519e9ec)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Update this calculation to be a) generic across writing systems / alignment points and b) handle non-integer sizes. [↵](#issue-20a01c1f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should the hanging punctuation be included in the box instead (so that the box is drawn around the punctuation when it is made visible through borders/background), but rather only excluded when positioning the box (so that the initial letter remains flush, with the hanging punctuation properly hanging)? See [discussion](https://github.com/w3c/csswg-drafts/issues/310). [↵](#issue-8de59d00)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need an unconditional [first](#valdef-initial-letter-wrap-first)? (I.e. Should we rename this value to auto and add a first value that does not check for spaces?) See GitHub issue [410](https://github.com/w3c/csswg-drafts/issues/410) [↵](#issue-58dffa4b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This really needs font-relative lengths to be relative to the used size. [↵](#issue-645378a9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> These values and related annoyance is likely unnecessary if someone submits a patch to Blink to support [first](#valdef-initial-letter-wrap-first). [↵](#issue-e060e05e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Edit figure to show how auto behaves in varying contexts [↵](#issue-c4ff3304)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The interaction of [initial-letter](#propdef-initial-letter) and [hanging-punctuation](https://www.w3.org/TR/css-text-4/#propdef-hanging-punctuation) is [under discussion](https://github.com/w3c/csswg-drafts/issues/310#issuecomment-396765893). [↵](#issue-a41c99e9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Handle glyph ink above cap height of font. Proposal: Make it an exclusion area for line boxes and border boxes. Include margin specified on initial-letters as part of exclusion area in order to control spacing. [↵](#issue-bd8afce4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Draw a box model diagram here. Does the margin of the initial letter collapse with its container? [↵](#issue-9ca3e502)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Whether an inline-end float originating in subsequent lines must clear the initial letter (as inline-start floats do) is [still under discussion](https://lists.w3.org/Archives/Public/www-style/2018Jul/0019.html). There is no aesthetic reason to require it; however it’s yet unclear how the underlying layout model would distinguish between the two cases. [↵](#issue-a83ae7df)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Pick a default. [↵](#issue-2765c39b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Somebody sanity-check these heuristics please. [↵](#issue-6c88f039)
