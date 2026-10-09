Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 3](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/).

Original copyright notice: Copyright © 2021 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 3

Source snapshot: https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/

Snapshot SHA-256: 88e7fb4fefc1ab7ec3f75b1058e6d9e003c937eb3c8049294adf113fdf3a4fc5

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 1 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Box Sizing Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-sizing” in the title, like this: “\[css-sizing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-sizing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-column-width"></a>

  Additions to [column-width](https://www.w3.org/TR/css-multicol-1/#propdef-column-width)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

CSS layout has several different concepts of automatic sizing that are used in various layout calculations. This section defines some more precise terminology to help connect the layout behaviors of this spec to the calculations used in other modules, and some new keywords for the [width](#propdef-width) and [height](#propdef-height) properties to allow authors to assign elements the dimensions resulting from these size calculations.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-664b2cc3"></a> This spec needs illustrations! See [issue](https://github.com/w3c/csswg-drafts/issues/1938).

### <a id="placement"></a>1.1.  Module interactions

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-column-width①"></a>

This module extends the [width](#propdef-width), [height](#propdef-height), [min-width](#propdef-min-width), [min-height](#propdef-min-height), [max-width](#propdef-max-width), [max-height](#propdef-max-height), and [column-width](https://www.w3.org/TR/css-multicol-1/#propdef-column-width) features defined in [\[CSS2\]](#biblio-css2) chapter 10 and in [\[CSS3COL\]](#biblio-css3col)

<a id="ref-for-propdef-box-sizing"></a>

The definition of the [box-sizing](#propdef-box-sizing) property in this module supersedes the one in [\[CSS-UI-3\]](#biblio-css-ui-3).

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terms"></a>2.  Terminology

Some key terminology related to coordinate axises and dimensions is defined in [CSS Writing Modes 3 § 6 Abstract Box Terminology](https://www.w3.org/TR/css-writing-modes-3/#abstract-box).

<a id="size"></a>size  
<a id="ref-for-height"></a>

<a id="ref-for-width"></a>

<a id="ref-for-inline-size"></a>

<a id="ref-for-block-size"></a>

A one- or two-dimensional measurement: a [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) and/or [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size); alternatively a [width](#width) and/or [height](#height).

![In left-to-right, top-to-bottom horizontal English, the horizontal width and inline size are synonymous, and vertical height and block size are synonymous.](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/images/sizing-ltr-tb.svg) ![In top-to-bottom, right-to-left vertical Japanese, the horizontal width and block size are synonymous, and vertical height and inline size are synonymous.](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/images/sizing-ttb-rl.svg)

<a id="ref-for-width①"></a>

<a id="ref-for-height①"></a>

<a id="ref-for-inline-size①"></a>

<a id="ref-for-block-size①"></a>

<a id="ref-for-writing-mode"></a>

Whether the [width](#width) or [height](#height) corresponds to an [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) or [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) depends on the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

<a id="inner-size"></a>inner size  
<a id="ref-for-box"></a>

<a id="ref-for-size"></a>

The [content-box](https://www.w3.org/TR/css2/box.html#box-dimensions) [size](#size) of a [box](https://www.w3.org/TR/css-display-3/#box).

![](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/images/inner-size.svg)

Inner size

<a id="outer-size"></a>outer size  
<a id="ref-for-box①"></a>

<a id="ref-for-size①"></a>

The [margin-box](https://www.w3.org/TR/css2/box.html#box-dimensions) [size](#size) of a [box](https://www.w3.org/TR/css-display-3/#box).

![](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/images/outer-size.svg)

Outer size

<a id="definite"></a>definite size  
<a id="ref-for-definite"></a>

<a id="ref-for-percentage-value"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-length-value"></a>

A size that can be determined without performing layout; that is, a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), a measure of text (without consideration of line-wrapping), a size of the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), or a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or other formula (such the [“stretch-fit” sizing of non-replaced blocks](https://www.w3.org/TR/CSS2/visudet.html#blockwidth) [\[CSS2\]](#biblio-css2)) that is resolved solely against [definite](#definite) sizes.

<a id="ref-for-containing-block"></a>

<a id="ref-for-definite①"></a>

Additionally, the size of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) of an absolutely positioned element is always [definite](#definite) with respect to that element.

<a id="indefinite"></a>indefinite size  
<a id="ref-for-available"></a>

<a id="ref-for-indefinite"></a>

<a id="ref-for-definite②"></a>

A size that is not [definite](#definite). [Indefinite](#indefinite) [available space](#available) is essentially infinite.

<a id="available"></a>available space  
<a id="ref-for-max-content-constraint"></a>

<a id="ref-for-min-content-constraint"></a>

<a id="ref-for-available①"></a>

<a id="ref-for-indefinite①"></a>

<a id="ref-for-definite③"></a>

<a id="ref-for-containing-block①"></a>

A size representing the space into which a box is laid out, as determined by the rules of the formatting context in which it participates. The space available to a box is usually either a measurement of its [containing block](https://www.w3.org/TR/css-display-3/#containing-block) (if that is [definite](#definite)) or an infinite size (when it is [indefinite](#indefinite)). [Available space](#available) can alternatively be either a [min-content constraint](#min-content-constraint) or a [max-content constraint](#max-content-constraint), which forces boxes laid into it to be laid out under that constraint.

<a id="stretch-fit"></a>stretch fit  
<a id="ref-for-stretch-fit"></a>

The [stretch fit](#stretch-fit) into a given size is that size, minus the box’s computed margins (not collapsed, treating auto as zero), border, and padding in the given dimension (such that the outer size is a perfect fit), and flooring at zero (so that the inner size is not negative).

<a id="ref-for-valdef-width-auto"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the formula used to calculate the [auto](#valdef-width-auto) widths of non-replaced blocks in normal flow in [CSS2.1§10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth).

<a id="fallback"></a>fallback size  
<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-fallback"></a>

Some sizing algorithms do not work well with an infinite size. In these cases, the [fallback size](#fallback) is used instead. Unless otherwise specified, this is the size of the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block).

### <a id="auto-box-sizes"></a>2.1.  Auto Box Sizes

<a id="ref-for-valdef-width-auto①"></a>

There are four types of automatically-determined sizes in CSS (sizes resulting from [auto](#valdef-width-auto) sizing rules, depending on context):

<a id="stretch-fit-size"></a>stretch-fit size  
<a id="stretch-fit-inline-size"></a>stretch-fit inline size  
<a id="stretch-fit-block-size"></a>stretch-fit block size  
<a id="ref-for-indefinite②"></a>

<a id="ref-for-definite④"></a>

<a id="ref-for-stretch-fit①"></a>

<a id="ref-for-available②"></a>

<a id="ref-for-outer-size"></a>

<a id="ref-for-size②"></a>

The [size](#size) a box would take if its [outer size](#outer-size) filled the [available space](#available) in the given axis; in other words, the [stretch fit](#stretch-fit) into the <a id="ref-for-available③"></a>available space, if that is [definite](#definite). Undefined if the <a id="ref-for-available④"></a>available space is [indefinite](#indefinite).

<a id="ref-for-inline-axis"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), this is called the “available width” in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and computed by the rules in [CSS2.1§10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth).

<a id="ref-for-available⑤"></a>

<a id="ref-for-indefinite③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Calculations involving this size need to specify a fallback behavior for when the [available space](#available) is [indefinite](#indefinite) if that happens to be possible.

<a id="max-content"></a>max-content size  
<a id="ref-for-size③"></a>

A box’s “ideal” [size](#size) in a given axis when given infinite available space. Usually this is the smallest <a id="ref-for-size④"></a>size the box could take in that axis while still fitting around its contents, i.e. minimizing unfilled space while avoiding overflow.

<a id="max-content-inline-size"></a>max-content inline size  
<a id="ref-for-size⑤"></a>

<a id="ref-for-inline-axis①"></a>

<a id="ref-for-inline-size②"></a>

The box’s “ideal” [size](#size) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Usually the narrowest [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) it could take while fitting around its contents if <em>none</em> of the soft wrap opportunities within the box were taken. (See [§ 5 Intrinsic Size Determination](#intrinsic).)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “preferred width” in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and the “maximum cell width” in [CSS2.1§17.5.2.2](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout).

<a id="max-content-block-size"></a>max-content block size  
<a id="ref-for-size⑥"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-block-size②"></a>

The box’s “ideal” [size](#size) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Usually the [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) of the content after layout.

<a id="min-content"></a>min-content size  
<a id="ref-for-min-content-constraint①"></a>

<a id="ref-for-size⑦"></a>

Nominally, the smallest [size](#size) a box could take that doesn’t lead to overflow that could be avoided by choosing a larger <a id="ref-for-size⑧"></a>size. Formally, the size of the box when sized under a [min-content constraint](#min-content-constraint), see [§ 5 Intrinsic Size Determination](#intrinsic).

<a id="min-content-inline-size"></a>min-content inline size  
<a id="ref-for-min-content"></a>

<a id="ref-for-inline-axis②"></a>

<a id="ref-for-inline-size③"></a>

The [min-content size](#min-content) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Typically, the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) that would fit around its contents if <em>all</em> soft wrap opportunities within the box were taken.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “preferred minimum width” in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and the “minimum content width” in [CSS2.1§17.5.2.2](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout).

<a id="min-content-block-size"></a>min-content block size  
<a id="ref-for-min-content①"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-max-content-block-size"></a>

The [min-content size](#min-content) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). For [block containers](https://www.w3.org/TR/css-display-3/#block-container), tables, and [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box), this is equivalent to the [max-content block size](#max-content-block-size).

<a id="fit-content-size"></a>fit-content size  
<a id="fit-content-inline-size"></a>fit-content inline size  
<a id="fit-content-block-size"></a>fit-content block size  
<a id="ref-for-max-content②"></a>

<a id="ref-for-min-content④"></a>

<a id="ref-for-min-content-constraint②"></a>

<a id="ref-for-stretch-fit-size①"></a>

<a id="ref-for-max-content①"></a>

<a id="ref-for-min-content③"></a>

<a id="ref-for-max-content"></a>

<a id="ref-for-stretch-fit-size"></a>

<a id="ref-for-min-content②"></a>

<a id="ref-for-definite⑤"></a>

<a id="ref-for-available⑥"></a>

If the [available space](#available) in a given axis is [definite](#definite), equal to <code>clamp(<a href="#min-content">min-content&#x20;size</a>,&#x20;<a href="#stretch-fit-size">stretch-fit&#x20;size</a>,&#x20;<a href="#max-content">max-content&#x20;size</a>)</code> (i.e. <code>max(<a href="#min-content">min-content&#x20;size</a>,&#x20;min(<a href="#max-content">max-content&#x20;size</a>,&#x20;<a href="#stretch-fit-size">stretch-fit&#x20;size</a>))</code>). When sizing under a [min-content constraint](#min-content-constraint), equal to the [min-content size](#min-content). Otherwise, equal to the [max-content size](#max-content) in that axis.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “shrink-to-fit” width in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and [CSS Multi-column Layout § 3.4](https://www.w3.org/TR/css3-multicol/#pseudo-algorithm).

<a id="intrinsic-size"></a>intrinsic size  
<a id="ref-for-min-content⑤"></a>

<a id="ref-for-max-content③"></a>

A [max-content size](#max-content) or [min-content size](#min-content), i.e. a size arising primarily from the size of the content. (Some uses of this term may refer also to sizes derived primarily from one of these two sizes.)

<a id="ref-for-replaced-element"></a>

<a id="ref-for-intrinsic-size"></a>

<a id="ref-for-natural-dimensions"></a>

[Replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) frequently derive their [intrinsic size](#intrinsic-size) from their [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions).

### <a id="contributions"></a>2.2.  Intrinsic Size Contributions

<a id="max-content-contribution"></a>max-content contribution  
<a id="ref-for-max-content④"></a>

<a id="ref-for-containing-block②"></a>

The size that a box contributes to its [containing block](https://www.w3.org/TR/css-display-3/#containing-block)’s [max-content size](#max-content).

<a id="min-content-contribution"></a>min-content contribution  
<a id="ref-for-min-content⑥"></a>

<a id="ref-for-containing-block③"></a>

The size that a box contributes to its [containing block](https://www.w3.org/TR/css-display-3/#containing-block)’s [min-content size](#min-content).

<a id="intrinsic-size-contribution"></a>intrinsic size contribution  
<a id="ref-for-min-content-contribution"></a>

<a id="ref-for-max-content-contribution"></a>

A [max-content contribution](#max-content-contribution), [min-content contribution](#min-content-contribution), or similarly-calculated content-based size contribution.

<a id="ref-for-outer-size①"></a>

Intrinsic size contributions are based on the [outer size](#outer-size) of the box; for this purpose auto margins are treated as zero.

### <a id="constraints"></a>2.3.  Intrinsic Size Constraints

<a id="max-content-constraint"></a>max-content constraint  
<a id="ref-for-max-content-contribution①"></a>

<a id="ref-for-containing-block④"></a>

A sizing constraint imposed by the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block) that causes it to produce its [max-content contribution](#max-content-contribution).

<a id="min-content-constraint"></a>min-content constraint  
<a id="ref-for-min-content-contribution①"></a>

<a id="ref-for-containing-block⑤"></a>

A sizing constraint imposed by the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block) that causes it to produce its [min-content contribution](#min-content-contribution).

<a id="preferred-aspect-ratio"></a>preferred aspect ratio  
<a id="ref-for-content-box"></a>

<a id="ref-for-natural-aspect-ratio"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

A width:height ratio inherent to a box, which biases various sizing algorithms to produce a size consistent with that aspect ratio insofar as possible while honoring other sizing inputs. Unless otherwise specified, a box’s [preferred aspect ratio](#preferred-aspect-ratio) is its [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) if it has one and is applied to its [content box](https://www.w3.org/TR/css-box-4/#content-box). Most boxes do not have a <a id="ref-for-preferred-aspect-ratio①"></a>preferred aspect ratio.

## <a id="specifying-sizes"></a>3.  Specifying Box Sizes<a id="size-keywords"></a>

### <a id="sizing-properties"></a>3.1.  Sizing Properties

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-propdef-height②"></a>

<a id="ref-for-propdef-min-width①"></a>

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-propdef-max-width①"></a>

<a id="ref-for-propdef-max-height①"></a>

This section defines the <a id="sizing-property"></a>sizing properties [width](#propdef-width), [height](#propdef-height), [min-width](#propdef-min-width), [min-height](#propdef-min-height), [max-width](#propdef-max-width), and [max-height](#propdef-max-height). Their potential values are defined in the next section, [§ 3.2 Sizing Values: the \<length-percentage\>, auto \| none, min-content, max-content, and fit-content() values](#sizing-values).

<a id="ref-for-flow-relative"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Additional [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) aliases to these properties are defined in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1).

<a id="ref-for-propdef-width③"></a>

<a id="ref-for-propdef-height③"></a>

<a id="ref-for-at-ruledef-page"></a>

<a id="ref-for-descdef-page-size"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6eb2c542"></a> We would like to define shorthands for each pair of sizing properties (e.g. [width](#propdef-width) and [height](#propdef-height)) but there is a naming conflict with the [@page](https://www.w3.org/TR/css-page-3/#at-ruledef-page) [size](https://www.w3.org/TR/css-page-3/#descdef-page-size) descriptor [\[CSS-PAGE-3\]](#biblio-css-page-3), so this has been deferred to Level 4. Suggestions on how to resolve this problem are welcome, see [discussion](https://github.com/w3c/csswg-drafts/issues/820).

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-propdef-height④"></a>

#### <a id="preferred-size-properties"></a>3.1.1.  Preferred Size Properties: the [width](#propdef-width) and [height](#propdef-height) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-width"></a>width, <a id="propdef-height"></a>height                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a><a id="ref-for-typedef-length-percentage"></a><a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one①"></a>\| min-content <a id="ref-for-comb-one②"></a>\| max-content <a id="ref-for-comb-one③"></a>\| fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline"></a><a id="ref-for-non-replaced"></a>all elements except [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) [inlines](https://www.w3.org/TR/css-display-3/#inline)                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑥"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-3/#containing-block)                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-valdef-width-fit-content-length-percentage"></a>by computed value type, recursing into [fit-content()](#valdef-width-fit-content-length-percentage)                                                                                                                                                                                                                                                                                |



<a id="ref-for-propdef-width⑤"></a>

<a id="ref-for-propdef-height⑤"></a>

<a id="ref-for-physical"></a>

The [width](#propdef-width) and [height](#propdef-height) ([physical](https://www.w3.org/TR/css-writing-modes-4/#physical)) properties specify the <a id="preferred-size"></a>preferred <a id="width"></a>width and <a id="height"></a>height of the box, respectively.

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-min-height②"></a>

#### <a id="min-size-properties"></a>3.1.2.  Minimum Size Properties: the [min-width](#propdef-min-width) and [min-height](#propdef-min-height) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-min-width"></a>min-width, <a id="propdef-min-height"></a>min-height                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage④"></a><a id="ref-for-typedef-length-percentage③"></a><a id="ref-for-comb-one④"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one⑤"></a>\| min-content <a id="ref-for-comb-one⑥"></a>\| max-content <a id="ref-for-comb-one⑦"></a>\| fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height⑥"></a><a id="ref-for-propdef-width⑥"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑦"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-3/#containing-block)                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑤"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animatable:</a>&#xA;      </strong> | <a id="ref-for-valdef-width-fit-content-length-percentage①"></a>by computed value, recursing into [fit-content()](#valdef-width-fit-content-length-percentage)                                                                                                                                                                                                                                                                                     |



<a id="ref-for-propdef-min-width③"></a>

<a id="ref-for-propdef-min-height③"></a>

The [min-width](#propdef-min-width) and [min-height](#propdef-min-height) properties specify the <a id="min-width"></a>minimum width (or “min width”) and <a id="min-height"></a>minimum height (or “min height”) of the box, respectively.

<a id="ref-for-valdef-width-auto②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The initial value of [auto](#valdef-width-auto) is new; in [\[CSS2\]](#biblio-css2) the initial value was zero.

<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-propdef-max-height②"></a>

#### <a id="max-size-properties"></a>3.1.3.  Maximum Size Properties: the [max-width](#propdef-max-width) and [max-height](#propdef-max-height) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-max-width"></a>max-width, <a id="propdef-max-height"></a>max-height                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑦"></a><a id="ref-for-typedef-length-percentage⑥"></a><a id="ref-for-comb-one⑧"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one⑨"></a>\| min-content <a id="ref-for-comb-one①⓪"></a>\| max-content <a id="ref-for-comb-one①①"></a>\| fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height⑦"></a><a id="ref-for-propdef-width⑦"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑧"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-3/#containing-block)                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑧"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animatable:</a>&#xA;      </strong> | <a id="ref-for-valdef-width-fit-content-length-percentage②"></a>by computed value, recursing into [fit-content()](#valdef-width-fit-content-length-percentage)                                                                                                                                                                                                                                                                                     |



<a id="ref-for-propdef-max-width③"></a>

<a id="ref-for-propdef-max-height③"></a>

The [max-width](#propdef-max-width) and [max-height](#propdef-max-height) properties specify the <a id="max-width"></a>maximum width (or “max width”) and <a id="max-height"></a>maximum height (or “max height”) of the box, respectively.

<a id="ref-for-typedef-length-percentage⑨"></a>

<a id="ref-for-valdef-width-auto③"></a>

<a id="ref-for-valdef-max-width-none"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-max-content"></a>

<a id="ref-for-valdef-width-fit-content-length-percentage③"></a>

### <a id="sizing-values"></a>3.2.  Sizing Values: the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), [auto](#valdef-width-auto) \| [none](#valdef-max-width-none), [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content()](#valdef-width-fit-content-length-percentage) values<a id="width-height-keywords"></a>

<a id="ref-for-typedef-length-percentage①⓪"></a>

<a id="valdef-width-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-border-box"></a>

<a id="ref-for-content-box①"></a>

<a id="ref-for-propdef-box-sizing①"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-length-value①"></a>

Specifies the size of the box using [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) and/or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value). The [box-sizing](#propdef-box-sizing) property indicates whether the [content box](https://www.w3.org/TR/css-box-4/#content-box) or [border box](https://www.w3.org/TR/css-box-4/#border-box) is measured.

<a id="ref-for-containing-block⑨"></a>

Percentages are resolved against the width/height, as appropriate, of the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block). If, in a particular axis, the <a id="ref-for-containing-block①⓪"></a>containing block’s size depends on the box’s size, see the relevant layout module for special rules on how to resolve percentages.

Negative values are invalid.

<a id="valdef-width-auto"></a>auto

<a id="ref-for-inline-size④"></a>

<a id="ref-for-block-size③"></a>

<a id="ref-for-propdef-height⑧"></a>

<a id="ref-for-propdef-width⑧"></a>

For [width](#propdef-width)/[height](#propdef-height), specifies an <a id="automatic-size"></a>automatic size (<a id="automatic-block-size"></a>automatic [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size)/<a id="automatic-inline-size"></a>automatic [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size)). See the relevant layout module for how to calculate this.

<a id="ref-for-propdef-min-width④"></a>

<a id="ref-for-propdef-min-height④"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-display-type"></a>

For [min-width](#propdef-min-width)/[min-height](#propdef-min-height), specifies an <a id="automatic-minimum-size"></a>automatic minimum size. Unless otherwise defined by the relevant layout module, however, it resolves to a used value of 0. For backwards-compatibility, the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of this keyword is zero for boxes of all [\[CSS2\]](#biblio-css2) [display types](https://www.w3.org/TR/css-display-3/#display-type): block and inline boxes, inline blocks, and all the table layout boxes. It also resolves to zero when no box is generated.

<a id="valdef-max-width-none"></a>none

No limit on the size of the box.

<a id="valdef-width-min-content"></a>min-content

<a id="ref-for-automatic-size"></a>

<a id="ref-for-block-size④"></a>

<a id="ref-for-min-content⑦"></a>

Use the [min-content size](#min-content) in the relevant axis; for a box’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size), unless otherwise specified, this is equivalent to its [automatic size](#automatic-size).

<a id="valdef-width-max-content"></a>max-content

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-block-size⑤"></a>

<a id="ref-for-max-content⑤"></a>

Use the [max-content size](#max-content) in the relevant axis; for a box’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size), unless otherwise specified, this is equivalent to its [automatic size](#automatic-size).

<a id="ref-for-typedef-length-percentage①①"></a>

<a id="valdef-width-fit-content-length-percentage"></a>fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage))

<a id="ref-for-typedef-length-percentage①③"></a>

<a id="ref-for-typedef-length-percentage①②"></a>

<a id="ref-for-valdef-width-min-content①"></a>

<a id="ref-for-valdef-width-max-content①"></a>

<a id="ref-for-available⑦"></a>

Use the fit-content formula with the [available space](#available) replaced by the specified argument, i.e. <code>min(<a href="#valdef-width-max-content">max-content</a>,&#x20;max(<a href="#valdef-width-min-content">min-content</a>,&#x20;<a href="https://www.w3.org/TR/css-values-4/#typedef-length-percentage">&lt;length-percentage&gt;</a>))</code>, where the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) argument is resolved exactly as for <a id="ref-for-typedef-length-percentage①④"></a>\<length-percentage\> values standing alone.

<a id="ref-for-typedef-length-percentage①⑤"></a>

Negative [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values are invalid.

<a id="ref-for-inner-size"></a>

In all cases, the used value is floored to preserve a non-negative [inner size](#inner-size).

<a id="ref-for-valdef-width-min-content②"></a>

<a id="ref-for-valdef-width-max-content②"></a>

<a id="ref-for-valdef-width-fit-content-length-percentage④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content()](#valdef-width-fit-content-length-percentage) values are new in Level 3.

<a id="ref-for-propdef-flex-basis"></a>

<a id="ref-for-propdef-width⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis) property hereby also gains these new keywords, as its values are defined by reference to \<[width](#propdef-width)\>.

<a id="ref-for-valdef-width-stretch"></a>

<a id="ref-for-valdef-width-fit-content"></a>

<a id="ref-for-stretch-fit-size②"></a>

<a id="ref-for-fit-content-size"></a>

<a id="ref-for-valdef-width-contain"></a>

<a id="ref-for-preferred-aspect-ratio②"></a>

<a id="ref-for-indefinite④"></a>

<a id="ref-for-available⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This section previously defined [stretch](https://www.w3.org/TR/css-sizing-4/#valdef-width-stretch) and [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) as keywords representing the [stretch-fit size](#stretch-fit-size) and [fit-content size](#fit-content-size), respectively. These keywords have been deferred to Level 4 (along with an additional [contain](https://www.w3.org/TR/css-sizing-4/#valdef-width-contain) keyword that behaves similarly to <a id="ref-for-valdef-width-stretch①"></a>stretch but preserves the [preferred aspect ratio](#preferred-aspect-ratio), if any) to better work out the implications in situations with [indefinite](#indefinite) [available space](#available).

<a id="ref-for-valdef-width-auto④"></a>

#### <a id="behave-auto"></a>3.2.1.  “Behaving as [auto](#valdef-width-auto)”

<a id="ref-for-propdef-width①⓪"></a>

<a id="ref-for-propdef-height⑨"></a>

<a id="ref-for-valdef-width-auto⑤"></a>

<a id="ref-for-indefinite⑤"></a>

To have a common term for both when [width](#propdef-width)/[height](#propdef-height) computes to [auto](#valdef-width-auto) and when it is defined to behave as if <a id="ref-for-valdef-width-auto⑥"></a>auto were specified (as in the case of [block percentage heights](https://www.w3.org/TR/CSS2/visudet.html#the-height-property) resolving against an [indefinite](#indefinite) size, see [CSS2§10.5](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)), the property is said to <a id="behave-as-auto"></a>behave as auto in both of these cases.

<a id="ref-for-propdef-width①①"></a>

<a id="ref-for-propdef-height①⓪"></a>

<a id="ref-for-valdef-width-auto⑦"></a>

<a id="ref-for-behave-as-auto"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Legacy spec prose defining layout behavior, particularly in [\[CSS2\]](#biblio-css2), might explicitly refer to [width](#propdef-width)/[height](#propdef-height) having a computed value of [auto](#valdef-width-auto) as a condition; some of these cases should be interpreted as meaning [behaves as auto](#behave-as-auto), and reported to the CSSWG for updating.

<a id="ref-for-automatic-size②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6215c343"></a> Replace this section with references to the new term [automatic size](#automatic-size).

#### <a id="the-contain-floats-value"></a>3.2.2.  Containing or Excluding Floats

<em>This section is non-normative.</em>

<a id="ref-for-block-box"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-formatting-context"></a>

Although [block box](https://www.w3.org/TR/css-display-3/#block-box) boundaries are typically pervious to floats, sometimes an author needs them to contain their own (descendant) floats or to exclude floats from outside. For Block layout, specifying [display: flow-root](https://www.w3.org/TR/css-display-3/#propdef-display) will make the box a [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context) root, which has this behavior.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Boxes participating in Flex, Grid, or Table layout will automatically have this behavior.

<a id="ref-for-propdef-box-sizing②"></a>

### <a id="box-sizing"></a>3.3.  Box Edges for Sizing: the [box-sizing](#propdef-box-sizing) property



| Field               | Definition                                                                                                          |
|---------------------|---------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-sizing"></a>box-sizing                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①②"></a>content-box [\|](https://www.w3.org/TR/css-values-4/#comb-one) border-box                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | content-box                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height①①"></a><a id="ref-for-propdef-width①②"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                            |



<a id="ref-for-propdef-box-sizing③"></a>

<a id="ref-for-length-value②"></a>

<a id="ref-for-percentage-value②"></a>

<a id="ref-for-content-box②"></a>

<a id="ref-for-border-box①"></a>

<a id="ref-for-sizing-property"></a>

<a id="ref-for-propdef-flex-basis①"></a>

The [box-sizing](#propdef-box-sizing) property defines whether fixed sizes (such as [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s and [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s) are assigned to the [content box](https://www.w3.org/TR/css-box-4/#content-box) or to the [border box](https://www.w3.org/TR/css-box-4/#border-box). It affects the interpretation of all [sizing properties](#sizing-property), including [flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis).

Values have the following meanings:

<a id="valdef-box-sizing-content-box"></a>content-box  
<a id="ref-for-propdef-height①②"></a>

<a id="ref-for-propdef-width①③"></a>

<a id="ref-for-content-box③"></a>

<a id="ref-for-inner-size①"></a>

<a id="ref-for-typedef-length-percentage①⑥"></a>

<a id="ref-for-sizing-property①"></a>

Sizes specified on [sizing properties](#sizing-property) as [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) represent the box’s [inner sizes](#inner-size), excluding the margins/border/padding: they are applied to the [content box](https://www.w3.org/TR/css-box-4/#content-box). The padding and border of the box are laid out and drawn <em>outside</em> the specified [width](#propdef-width) and [height](#propdef-height).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the behavior of width and height as specified by CSS2.1, and is thus the default.

<a id="valdef-box-sizing-border-box"></a>border-box  
<a id="ref-for-content-box④"></a>

<a id="ref-for-propdef-height①③"></a>

<a id="ref-for-propdef-width①④"></a>

<a id="ref-for-border-box②"></a>

<a id="ref-for-typedef-length-percentage①⑦"></a>

<a id="ref-for-sizing-property②"></a>

Sizes specified on [sizing properties](#sizing-property) as [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) represent the box’s visually-apparent sizes, including the borders/padding (but not margin): they are applied to the [border box](https://www.w3.org/TR/css-box-4/#border-box). The padding and border of the box are laid out and drawn <em>inside</em> the specified [width](#propdef-width) and [height](#propdef-height), with the [content box](https://www.w3.org/TR/css-box-4/#content-box) sized to fill the remaining space, floored at zero.

<a id="ref-for-typedef-length-percentage①⑧"></a>

The content width and height are calculated by subtracting the border and padding widths of the respective sides from the specified [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage). As the content width and height [cannot be negative](#sizing-values), this computation is floored at zero.

<a id="ref-for-dom-window-getcomputedstyle"></a>

Used values, as exposed for instance through <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>, also refer to the border box.

<a id="ref-for-propdef-box-sizing④"></a>

<a id="ref-for-typedef-length-percentage①⑨"></a>

<a id="ref-for-valdef-width-fit-content-length-percentage⑤"></a>

<a id="ref-for-valdef-width-auto⑧"></a>

<a id="ref-for-valdef-width-min-content③"></a>

Values affected by [box-sizing](#propdef-box-sizing) include both raw [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values and those used in functional notations such as [fit-content()](#valdef-width-fit-content-length-percentage). In contrast, non-quantitative values such as [auto](#valdef-width-auto) and [min-content](#valdef-width-min-content) are not influenced by the <a id="ref-for-propdef-box-sizing⑤"></a>box-sizing property (unless otherwise specified).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8af7a68e"></a> For example, the following properties set the content-box size of the box to 100px, with the border-box size calculating to 120px:
>
> ```css
> .box {
>   box-sizing:   content-box; /* default */
>   width:        100px;
>   padding-left: 10px;
>   border-left:  10px solid;
> }
> ```
>
> <a id="ref-for-valdef-box-sizing-border-box"></a>
>
> On the other hand, by changing to [border-box](#valdef-box-sizing-border-box), the border-box is set to 100px, with the content-box size calculating to 80px:
>
> ```css
> .box {
>   box-sizing:   border-box;
>   width:        100px;
>   padding-left: 10px;
>   border-left:  10px solid;
> }
> ```
>
> <a id="ref-for-inner-size②"></a>
>
> <a id="ref-for-propdef-padding"></a>
>
> <a id="ref-for-propdef-border"></a>
>
> <a id="ref-for-propdef-width①⑤"></a>
>
> The [inner size](#inner-size) can’t be less than zero, so if the [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) + [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) is greater than the specified border-box size, the box will end up larger than specified. In this case, the content-box size will floor at 0px so the border-box size ends up at 120px, even though [width: 100px](#propdef-width) is specified for the border box:
>
> ```css
> .box {
>   box-sizing:   border-box;
>   width:        100px;
>   padding-left: 60px;
>   border-left:  60px solid;
>   /* padding + border = 120px */
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0700ac27"></a> This example uses box-sizing to evenly horizontally split two divs with fixed size borders inside a div container, which would otherwise require additional markup.
>
> sample CSS:
>
> ```text
> div.container {
>   width:38em;
>   border:1em solid black;
> }
> 
> div.split {
>   box-sizing:border-box;
>   width:50%;
>   border:1em silver ridge;
>   float:left;
> }
> ```
>
> sample HTML fragment:
>
> ```text
> <div class="container">
> <div class="split">This div occupies the left half.</div>
> <div class="split">This div occupies the right half.</div>
> </div>
> ```
>
> demonstration of sample CSS and HTML:
>
> <a id="ref-for-propdef-box-sizing⑥"></a>
>
> This div should occupy the left half.
>
> This div should occupy the right half.
>
> The two divs above should appear side by side, each (including borders) 50% of the content width of their container. If instead they are stacked one on top of the other then your browser does not support [box-sizing](#propdef-box-sizing).

<a id="ref-for-the-button-element"></a>

<a id="ref-for-valdef-box-sizing-border-box①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Certain HTML elements, such as <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code>, default to [border-box](#valdef-box-sizing-border-box) behavior. See HTML for details on which elements have this behavior.

<a id="ref-for-width②"></a>

<a id="ref-for-height②"></a>

<a id="ref-for-min-width"></a>

<a id="ref-for-min-height"></a>

<a id="ref-for-max-width"></a>

<a id="ref-for-max-height"></a>

<a id="ref-for-inner-size③"></a>

<a id="ref-for-content-box⑤"></a>

<a id="ref-for-box②"></a>

In legacy CSS specifications, the terms [width](#width), [height](#height), [minimum (min) width](#min-width), [minimum (min) height](#min-height), [maximum (max) width](#max-width), and [maximum (max) height](#max-height) generally refer to the [inner](#inner-size) size ([content-box](https://www.w3.org/TR/css-box-4/#content-box) size) of a [box](https://www.w3.org/TR/css-display-3/#box) unless otherwise indicated.

Refer to [CSS User Interface 3 § 3.1 Changing the Box Model: the box-sizing property](https://www.w3.org/TR/css-ui-3/#box-sizing) for an explicit disambiguation of these terms for the [Visual formatting model details](https://www.w3.org/TR/CSS21/visudet.html) section of [\[CSS2\]](#biblio-css2).

<a id="ref-for-inner-size④"></a>

<a id="ref-for-outer-size②"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-sizing-property③"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> To avoid ambiguities, specification authors should avoid ambiguous uses of terms such as width or height without further qualification, and should explicitly refer and link to the [inner](#inner-size) size, the [outer](#outer-size) size, the size of the [border-box](https://www.w3.org/TR/css2/box.html#box-dimensions), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [sizing properties](#sizing-property), etc, as appropriate for each case.

<a id="ref-for-valdef-column-width-min-content"></a>

<a id="ref-for-valdef-column-width-max-content"></a>

<a id="ref-for-valdef-column-width-fit-content-length-percentage"></a>

### <a id="column-sizing"></a>3.4.  New Column Sizing Values: the [min-content](#valdef-column-width-min-content), [max-content](#valdef-column-width-max-content), and [fit-content()](#valdef-column-width-fit-content-length-percentage) values



| Field               | Definition                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-column-width②"></a>[column-width](https://www.w3.org/TR/css-multicol-1/#propdef-column-width)                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②⓪"></a><a id="ref-for-comb-one①③"></a>min-content [\|](https://www.w3.org/TR/css-values-4/#comb-one) max-content <a id="ref-for-comb-one①④"></a>\| fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②①"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                     |



<a id="ref-for-propdef-column-width③"></a>

When used as values for [column-width](https://www.w3.org/TR/css-multicol-1/#propdef-column-width), the new keywords specify the optimal column width:

<a id="valdef-column-width-min-content"></a>min-content

<a id="ref-for-min-content-inline-size"></a>

Specifies the optimal column width as the [min-content inline size](#min-content-inline-size) of the multi-column container’s contents.

<a id="valdef-column-width-max-content"></a>max-content

<a id="ref-for-max-content-inline-size"></a>

Specifies the optimal column width as the [max-content inline size](#max-content-inline-size) of the multi-column container’s contents.

<a id="ref-for-typedef-length-percentage②②"></a>

<a id="valdef-column-width-fit-content-length-percentage"></a>fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage))

<a id="ref-for-typedef-length-percentage②③"></a>

<a id="ref-for-min-content⑧"></a>

<a id="ref-for-max-content⑥"></a>

Specifies the optimal column width as <code>min(<a href="#max-content">max-content&#x20;size</a>,&#x20;max(<a href="#min-content">min-content&#x20;size</a>,&#x20;<a href="https://www.w3.org/TR/css-values-4/#typedef-length-percentage">&lt;length-percentage&gt;</a>))</code>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The column width never varies by column. When the column width is informed by the multi-column container’s contents (as in the keywords above), all of its contents are taken under consideration and the calculated width is shared by all the columns.

## <a id="extrinsic"></a>4.  Extrinsic Size Determination

<a id="extrinsic-sizing"></a>Extrinsic sizing determines sizes based on the context of an element, without regard for its contents.

### <a id="percentage-sizing"></a>4.1.  Percentage Sizing

<a id="ref-for-containing-block①①"></a>

Percentages specify sizing of a box with respect to the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-65abfbed"></a> For example, in the following markup:
>
> ```html
> <article style="height: 60em">
>   <figure style="height: 50%;">
>     <img style="height: 50%;">
>   </figure>
> </article>
> ```
>
> - <a id="ref-for-definite⑥"></a>
>
>   the `<figure>` would be 30em tall = 50% of the [definite](#definite) 60em height of the `<article>`
>
> - <a id="ref-for-definite⑦"></a>
>
>   the `<img>` would be 15em tall = 50% of the `<figure>`’s height (which is itself [definite](#definite) because it’s a percentage resolved against a <a id="ref-for-definite⑧"></a>definite length)

<a id="ref-for-containing-block①②"></a>

See [§ 5.2.1 Intrinsic Contributions of Percentage-Sized Boxes](#cyclic-percentage-contribution) for details on how to resolve percentages when the size of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) depends on the size of its content.

## <a id="intrinsic"></a>5.  Intrinsic Size Determination

<a id="intrinsic-sizing"></a>Intrinsic sizing determines sizes based on the contents of an element, without regard for its context.

### <a id="intrinsic-sizes"></a>5.1.  Intrinsic Sizes

<a id="ref-for-min-content⑨"></a>

<a id="ref-for-valdef-width-auto⑨"></a>

<a id="ref-for-min-width①"></a>

<a id="ref-for-max-width①"></a>

The [min-content size](#min-content) of a box in each axis is the size it would have if it was a float given an [auto](#valdef-width-auto) size in that axis (and no [minimum](#min-width) or [maximum size](#max-width) in that axis) and if its containing block was <em>zero</em>-sized in that axis. (In other words, the minimum size it has when sized as “shrink-to-fit”.)

<a id="ref-for-max-content⑦"></a>

<a id="ref-for-valdef-width-auto①⓪"></a>

<a id="ref-for-min-width②"></a>

<a id="ref-for-max-width②"></a>

The [max-content size](#max-content) of a box in each axis is the size it would have if it was a float given an [auto](#valdef-width-auto) size in that axis (and no [minimum](#min-width) or [maximum size](#max-width) in that axis), and if its containing block was <em>infinitely</em>-sized in that axis. (In other words, the maximum size it has when sized as “shrink-to-fit”.)

<a id="ref-for-min-content①⓪"></a>

<a id="ref-for-max-content⑧"></a>

<a id="ref-for-intrinsic-size①"></a>

The [min-content size](#min-content) and [max-content size](#max-content) are collectively referred to as the [intrinsic sizes](#intrinsic-size).

<a id="ref-for-preferred-aspect-ratio③"></a>

<a id="ref-for-valdef-width-auto①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When the box has a [preferred aspect ratio](#preferred-aspect-ratio), size constraints in the opposite dimension will transfer through and can affect the [auto](#valdef-width-auto) size in the considered one. See [CSS2§10](https://www.w3.org/TR/CSS2/visudet.html).

<a id="ref-for-intrinsic-size②"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-natural-size"></a>

This specification does not define how to determine the sizes of floats. Please refer to [\[CSS2\]](#biblio-css2). However, the [intrinsic sizes](#intrinsic-size) of [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) without [natural sizes](https://www.w3.org/TR/css-images-3/#natural-size) are defined below:

<a id="ref-for-preferred-aspect-ratio④"></a>

If it has a [preferred aspect ratio](#preferred-aspect-ratio):

<a id="ref-for-min-content①①"></a>

For the [min-content size](#min-content), use zero.

<a id="ref-for-max-content⑨"></a>

For the [max-content size](#max-content):

- <a id="ref-for-available⑨"></a>

  <a id="ref-for-definite⑨"></a>

  <a id="ref-for-inline-axis③"></a>

  <a id="ref-for-stretch-fit②"></a>

  If the [available space](#available) is [definite](#definite) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), use the [stretch fit](#stretch-fit) into that size for the inline size and calculate the block size using the aspect ratio.

- <a id="ref-for-length-value③"></a>

  <a id="ref-for-computed-value①"></a>

  <a id="ref-for-propdef-min-width⑤"></a>

  <a id="ref-for-propdef-min-height⑤"></a>

  Otherwise if the box has a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) as its [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) for [min-width](#propdef-min-width) or [min-height](#propdef-min-height), use that size and calculate the other dimension using the aspect ratio; if both dimensions have a <a id="ref-for-length-value④"></a>\<length\> minimum, choose the one that results in the larger overall size.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This case was previous calculated from a 300x150 default size, rather than the box’s min size. This is believed to be a better behavior, and likely to be Web-compatible, but please send feedback to the CSSWG if there are any problems.

- <a id="ref-for-inline-size⑤"></a>

  <a id="ref-for-initial-containing-block②"></a>

  Otherwise use an [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) matching the corresponding dimension of the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block) and calculate the other dimension using the aspect ratio.

<a id="ref-for-preferred-aspect-ratio⑤"></a>

If it has no [preferred aspect ratio](#preferred-aspect-ratio):

<a id="ref-for-max-content①⓪"></a>

<a id="ref-for-min-content①②"></a>

For both the [min-content size](#min-content) and [max-content size](#max-content):

- <a id="ref-for-length-value⑤"></a>

  <a id="ref-for-computed-value②"></a>

  <a id="ref-for-min-width③"></a>

  <a id="ref-for-propdef-min-width⑥"></a>

  <a id="ref-for-propdef-min-height⑥"></a>

  If the box has a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) as its [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [minimum size](#min-width) ([min-width](#propdef-min-width)/[min-height](#propdef-min-height)) in that dimension, use that size.

  <a id="ref-for-valdef-width-auto①②"></a>

  <a id="ref-for-min-size-properties③"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This author-controllable behavior is made possible by the new [auto](#valdef-width-auto) value for the [min size properties](#min-size-properties). This is believed to be a better behavior, but it is not yet clear if it is Web-compatible, so please send feedback to the CSSWG if there are any problems.

- Otherwise, use 300px for the width and/or 150px for the height as needed.

<a id="ref-for-propdef-height①④"></a>

<a id="ref-for-propdef-width①⑥"></a>

<a id="ref-for-behave-as-auto①"></a>

<a id="ref-for-max-content①①"></a>

Since a block-level or inline-level replaced element whose [height](#propdef-height) or [width](#propdef-width) [behaves as auto](#behave-as-auto) is effectively defined to use its [max-content size](#max-content) ([CSS2§10.3.2](https://www.w3.org/TR/CSS2/visudet.html#inline-replaced-width)), this specification applies the rules above to the undefined case of a replaced element whose <a id="ref-for-propdef-height①⑤"></a>height and <a id="ref-for-propdef-width①⑦"></a>width both <a id="ref-for-behave-as-auto②"></a>behave as auto.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define how to determine the size of a float. Please refer to [\[CSS2\]](#biblio-css2), the relevant CSS specification for that display type, and/or existing implementations for further details. A future specification will define this in detail, replacing the CSS2 “definition”, such as it is.

------------------------------------------------------------------------

<a id="ref-for-valdef-width-auto①③"></a>

<a id="ref-for-intrinsic-size③"></a>

<a id="ref-for-valdef-width-min-content④"></a>

<a id="ref-for-valdef-width-max-content③"></a>

<a id="ref-for-sizing-property④"></a>

Although the [auto](#valdef-width-auto) size of text input controls such as HTML’s `<input type=text>` and `<textarea>` elements is typically a fixed size, the contents of such elements can be used to determine a content-based [intrinsic size](#intrinsic-size), as for non-replaced block containers. The [min-content](#valdef-width-min-content) and [max-content](#valdef-width-max-content) keywords of the [sizing properties](#sizing-property) thus represent content-based sizes for form controls which render their value as text contained within their box, allowing such controls to size to fit their visible contents similarly to regular non-replaced elements.

<a id="ref-for-concept-textarea-raw-value"></a>

<a id="ref-for-the-textarea-element"></a>

<a id="ref-for-the-input-element"></a>

<a id="ref-for-text-run"></a>

<a id="ref-for-soft-wrap-opportunity"></a>

<a id="ref-for-intrinsic-size④"></a>

The content in this case is defined to be the input control’s values (the [raw value](https://html.spec.whatwg.org/multipage/form-elements.html#concept-textarea-raw-value) in the case of <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code>, or the [value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value) in the case of <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>), possibly transformed to a more human-readable and/or localized display format, which is then treated as child [text runs](https://www.w3.org/TR/css-display-3/#text-run) of the input control, allowing [soft wrap opportunities](https://www.w3.org/TR/css-text-3/#soft-wrap-opportunity) only where the input control would actually allow wrapping (whether keyed off of CSS properties or other, UA-internal constraints). If the input control has designated placeholder text to be overlaid in its value display area, then that text is also measured for the purpose of calculating the content-based size—whether or not the placeholder text is visible at the moment. (Thus the content-based [intrinsic size](#intrinsic-size) of the input control is the larger of the size to fit the placeholder text and the size to fit the value.)

<a id="ref-for-min-content①③"></a>

<a id="ref-for-max-content①②"></a>

The UA may enforce a minimum (such as the size required to contain a single zero-width character, or the smallest usable size of a touch target) on the form control’s [min-content](#min-content) and [max-content sizes](#max-content) to ensure sufficient space for the caret and otherwise maintain usability of the form control.

<a id="ref-for-the-iframe-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This might be extended to <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> or other content-containing replaced elements (see [discussion](https://github.com/w3c/csswg-drafts/issues/1771)), but text inputs are a major use-case; and being document-internal, have the least additional complications.

### <a id="intrinsic-contribution"></a>5.2.  Intrinsic Contributions

<a id="ref-for-min-content-contribution②"></a>

<a id="ref-for-max-content-contribution②"></a>

<a id="ref-for-valdef-width-auto①④"></a>

A box’s [min-content contribution](#min-content-contribution)/[max-content contribution](#max-content-contribution) in each axis is the size of the content box of a hypothetical [auto](#valdef-width-auto)-sized float that contains only that box, if that hypothetical float’s containing block is zero-sized/infinitely-sized.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define precisely how to determine these sizes. Please refer to [\[CSS2\]](#biblio-css2), the relevant CSS specification for that display type, the [rules for handling percentages](#percentage-sizing) (below), and/or existing implementations for further details.

#### <a id="cyclic-percentage-contribution"></a>5.2.1.  Intrinsic Contributions of Percentage-Sized Boxes

<a id="ref-for-containing-block①③"></a>

<a id="ref-for-intrinsic-size-contribution"></a>

<a id="ref-for-automatic-minimum-size"></a>

Sometimes the size of a percentage-sized box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block) depends on the [intrinsic size contribution](#intrinsic-size-contribution) of the box itself, creating a cyclic dependency. When calculating the <a id="ref-for-intrinsic-size-contribution①"></a>intrinsic size contribution of such a box (including any calculations for a content-based [automatic minimum size](#automatic-minimum-size)), a percentage value that resolves against a size in the same axis as the <a id="ref-for-intrinsic-size-contribution②"></a>intrinsic size contribution (a <a id="cyclic-percentage-size"></a>cyclic percentage size) is resolved specially:

1.  <a id="ref-for-valdef-width-auto①⑤"></a>

    <a id="ref-for-initial-value"></a>

    <a id="ref-for-intrinsic-size-contribution③"></a>

    <a id="ref-for-cyclic-percentage-size"></a>

    <a id="ref-for-propdef-max-height④"></a>

    <a id="ref-for-propdef-height①⑥"></a>

    <a id="ref-for-propdef-max-width④"></a>

    <a id="ref-for-propdef-width①⑧"></a>

    <a id="ref-for-preferred-size-properties④"></a>

    <a id="ref-for-max-size-properties④"></a>

    <a id="ref-for-non-replaced①"></a>

    <a id="non-replaced-percentage-contribution"></a> If the box is [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced), then the entire value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) ([width](#propdef-width)/[max-width](#propdef-max-width)/[height](#propdef-height)/[max-height](#propdef-max-height)) specified as an expression containing a percentage (such as 10% or calc(10px + 0%)) that is [cyclic](#cyclic-percentage-size) is treated <em>for the purpose of calculating the box’s <a href="#intrinsic-size-contribution">intrinsic size contributions</a> only</em> as that property’s [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value). For example, given a box with <a id="ref-for-propdef-width①⑨"></a>width: calc(20px + 50%), its max-content contribution is calculated as if its <a id="ref-for-propdef-width②⓪"></a>width were [auto](#valdef-width-auto). (The percentage is honored as usual, however, during the actual sizing of the box itself; see below.)

2.  <a id="ref-for-initial-value①"></a>

    <a id="ref-for-max-content-contribution③"></a>

    <a id="ref-for-cyclic-percentage-size①"></a>

    <a id="ref-for-preferred-size-properties①"></a>

    <a id="ref-for-max-size-properties①"></a>

    <a id="ref-for-replaced-element②"></a>

    <a id="replaced-percentage-max-contribution"></a> Likewise, if the box is [replaced](https://www.w3.org/TR/css-display-3/#replaced-element), then the entire value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) specified as an expression containing a percentage that is [cyclic](#cyclic-percentage-size) is treated <em>for the purpose of calculating the box’s <a href="#max-content-contribution">max-content contributions</a> only</em> as that property’s [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

3.  <a id="ref-for-min-width④"></a>

    <a id="ref-for-typedef-length-percentage②④"></a>

    <a id="ref-for-preferred-aspect-ratio⑥"></a>

    <a id="ref-for-min-content-contribution③"></a>

    <a id="ref-for-propdef-max-height⑤"></a>

    <a id="ref-for-propdef-height①⑦"></a>

    <a id="ref-for-propdef-max-width⑤"></a>

    <a id="ref-for-propdef-width②①"></a>

    <a id="ref-for-preferred-size-properties②"></a>

    <a id="ref-for-max-size-properties②"></a>

    <a id="ref-for-cyclic-percentage-size②"></a>

    <a id="ref-for-replaced-element③"></a>

    <a id="replaced-percentage-min-contribution"></a> If the box is [replaced](https://www.w3.org/TR/css-display-3/#replaced-element), a [cyclic percentage](#cyclic-percentage-size) in the value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) ([width](#propdef-width)/[max-width](#propdef-max-width)/[height](#propdef-height)/[max-height](#propdef-max-height)), is resolved against zero when calculating the [min-content contribution](#min-content-contribution) in the corresponding axis. (See [§ 5.2.2 Compressible Replaced Elements](#min-content-zero) for a list of which elements in HTML this applies to.) If the box also has a [preferred aspect ratio](#preferred-aspect-ratio), then this <a id="ref-for-min-content-contribution④"></a>min-content contribution is floored by any [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [minimum size](#min-width) from the opposite axis—resolving any such percentage against zero—transferred through the <a id="ref-for-preferred-aspect-ratio⑦"></a>preferred aspect ratio.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-1b92b23f"></a> Should we resolve transferred percentages against their containing block instead of zero before transferring them? See [discussion](https://github.com/w3c/csswg-drafts/issues/6341).

    <a id="ref-for-min-content-contribution⑤"></a>

    <a id="ref-for-the-select-element"></a>

    The UA may additionally floor the [min-content contribution](#min-content-contribution) based on UI considerations, such as ensuring certain UI elements remain visible (for example, the dropdown arrow on a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code>).

    <a id="ref-for-min-content-contribution⑥"></a>

    <a id="ref-for-min-width⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The [min-content contribution](#min-content-contribution) is, as always, also floored by the [minimum size](#min-width) in its own axis.

    <a id="ref-for-automatic-minimum-size①"></a>

    <a id="ref-for-definite①⓪"></a>

    This rule also applies when calculating a content-based [automatic minimum size](#automatic-minimum-size) or its corresponding size contribution, yielding a [definite](#definite) “specified size suggestion”.

    <a id="ref-for-the-input-element①"></a>

    <a id="ref-for-propdef-width②②"></a>

    <a id="ref-for-min-content-contribution⑦"></a>

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-b06093ff"></a> For example, an <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> assigned [width: calc(50% + 50px)](#propdef-width) has a [min-content contribution](#min-content-contribution) of 50px, plus any horizontal margin/border/padding.

4.  <a id="ref-for-intrinsic-size-contribution④"></a>

    <a id="ref-for-cyclic-percentage-size③"></a>

    <a id="ref-for-gutter"></a>

    <a id="ref-for-padding"></a>

    <a id="ref-for-margin"></a>

    <a id="ref-for-min-size-properties①"></a>

    <a id="min-percentage-contribution"></a> For the [min size properties](#min-size-properties), as well as for [margins](https://www.w3.org/TR/css-box-4/#margin) and [paddings](https://www.w3.org/TR/css-box-4/#padding) (and [gutters](https://www.w3.org/TR/css-align-3/#gutter)), a [cyclic percentage](#cyclic-percentage-size) is resolved against zero for determining [intrinsic size contributions](#intrinsic-size-contribution).

**Table 6**

Summary of the Cyclic-Percentage Intrinsic Size Contribution Rules (Above)

Representation note: complete merged-header paths are explicit; inherited span values are repeated where they apply. Native HTML span and row-header accessibility semantics are not available in GFM.

**Source header labels**

Element Type / <a id="ref-for-replaced-element④"></a> [Replaced](https://www.w3.org/TR/css-display-3/#replaced-element) / <a id="ref-for-non-replaced②"></a> [Non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced)

Contribution Type / <a id="ref-for-min-content-contribution⑧"></a> [min-content](#min-content-contribution) / <a id="ref-for-max-content-contribution④"></a> [max-content](#max-content-contribution) / <a id="ref-for-min-content-contribution⑨"></a> [min-content](#min-content-contribution) / <a id="ref-for-max-content-contribution⑤"></a> [max-content](#max-content-contribution)

| Element Type / Contribution Type | [Replaced](https://www.w3.org/TR/css-display-3/#replaced-element) / [min-content](#min-content-contribution) | [Replaced](https://www.w3.org/TR/css-display-3/#replaced-element) / [max-content](#max-content-contribution) | [Non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) / [min-content](#min-content-contribution) | [Non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) / [max-content](#max-content-contribution) |
| --- | --- | --- | --- | --- |
| <a id="ref-for-padding-properties"></a> <a id="ref-for-margin-properties"></a> <a id="ref-for-min-size-properties②"></a> **[min size](#min-size-properties) &#x26; [margin](https://www.w3.org/TR/css-box-4/#margin-properties)/[padding](https://www.w3.org/TR/css-box-4/#padding-properties)** | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) |
| <a id="ref-for-preferred-size-properties③"></a> <a id="ref-for-max-size-properties③"></a> **[max](#max-size-properties) &#x26; [preferred size](#preferred-size-properties)** | [zeroᶜ](#replaced-percentage-min-contribution) | [initialᵇ](#replaced-percentage-max-contribution) | [initialᵃ](#non-replaced-percentage-contribution) | [initialᵃ](#non-replaced-percentage-contribution) |

Then, unless otherwise specified, when calculating the used sizes and positions of the containing block’s <em>contents</em>:

- <a id="ref-for-propdef-block-size"></a>

  <a id="ref-for-propdef-max-block-size"></a>

  <a id="ref-for-behave-as-auto③"></a>

  If the cyclic dependency was introduced due to a [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size) or [max-block-size](https://www.w3.org/TR/css-logical-1/#propdef-max-block-size) on the containing block that causes it to depend on the size of its contents, the box’s percentage is not resolved and instead [behaves as auto](#behave-as-auto).

  <a id="ref-for-grid-item"></a>

  <a id="ref-for-flex-item"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: [Grid items](https://www.w3.org/TR/css-grid-2/#grid-item) and [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) do allow percentages to resolve in this case.

- Otherwise, the percentage is resolved against the containing block’s size. (The containing block’s size is not re-resolved based on the resulting size of the box; the contents might thus overflow or underflow the containing block).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These rules specify the previously-undefined behavior of this cyclic case in [CSS2§10.2](https://www.w3.org/TR/CSS2/visudet.html#the-width-property), [CSS2§8.3](https://www.w3.org/TR/CSS2/box.html#margin-properties), and [CSS2§8.4](https://www.w3.org/TR/CSS2/box.padding-properties). Note also, the behavior in [CSS2§10.5](https://www.w3.org/TR/CSS2/visudet.html#the-height-property) is superseded in their respective specifications for layout modes (such as [flex layout](https://www.w3.org/TR/css-flexbox/)) not described in CSS2.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a57016da"></a> For example, in the following markup:
>
> ```html
> <article style="width: min-content">
>   <aside style="width: 50%;">
>   LOOOOOOOOOOOOOOOOOOOONG
>   </aside>
> </article>
> ```
>
> <a id="ref-for-propdef-width②③"></a>
>
> <a id="ref-for-definite①①"></a>
>
> When calculating the width of the outer `<article>`, the inner `<aside>` behaves as [width: auto](#propdef-width), so the `<article>` sets itself to the width of the long word. Since the `<article>`’s width didn’t depend on "real" layout, though, it’s treated as [definite](#definite) for resolving the `<aside>`, whose width resolves to half that of the `<article>`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a6fc9bae"></a> In this example,
>
> ```html
> <article style="height:auto">
>   <aside style="height: 50%;">
>     <div class=block style="height: 150px;"></div>
>   </aside>
>   <section style="height: 30px;"></section>
> </article>
> ```
>
> <a id="ref-for-block-size⑥"></a>
>
> <a id="ref-for-propdef-height①⑧"></a>
>
> <a id="ref-for-valdef-width-auto①⑥"></a>
>
> because the percentage [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) ([height](#propdef-height), in this case) on block-level elements is defined to not resolve inside content-sized containing blocks, the percentage height on the `<aside>` is ignored, that is, it behaves exactly as if [auto](#valdef-width-auto) were specified.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8559f728"></a>
>
> <a id="ref-for-propdef-height①⑨"></a>
>
> <a id="ref-for-propdef-min-height⑦"></a>
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > <a id="issue-9b3707fe"></a> Letting percentages still resolve against a definite [height](#propdef-height) when the min-height is intrinsic is an open issue. (CSS2 has a general statement about "height depending on contents", which this technically is, even though CSS2 didn’t have content-dependent keywords for [min-height](#propdef-min-height). Since this is new, we think we could have this different behavior.)
>
> The following examples illustrate how block-axis percentages resolve against a containing block whose size depends on its contents.
>
> ```html
> <article style="height:100px; min-height: min-content;">
>   <aside style="height: 50%;">
>     <div style="height: 150px;"></div>
>   </aside>
>   <section style="height: 30px;"></section>
> </article>
> ```
>
> <a id="ref-for-propdef-height②⓪"></a>
>
> <a id="ref-for-behave-as-auto④"></a>
>
> The initial height of the `<article>` is 100px, as specified, which would make the `<aside>` 50px tall when it resolved its percentage. However, we must calculate the min-height, by substituting it in for [height](#propdef-height). This causes the percentage on the `<aside>` to [behave as auto](#behave-as-auto), so the `<aside>` ends up 150px tall. The total height of the contents is thus 180px. This is larger than the specified 100px height, so the `<article>` gets adjusted to 180px tall.
>
> Then, since the percentage could <em>originally</em> resolve against the (100px) height, it now resolves against the 180px height, so the `<aside>` ends up being 90px tall.
>
> ```html
> <article style="height:auto; min-height: min-content;">
>   <aside style="height: 50%;">
>     <div class=block style="height: 150px;"></div>
>   </aside>
>   <section style="height: 30px;"></section>
> </article>
> ```
>
> <a id="ref-for-valdef-width-auto①⑦"></a>
>
> <a id="ref-for-behave-as-auto⑤"></a>
>
> <a id="ref-for-propdef-min-height⑧"></a>
>
> <a id="ref-for-propdef-height②①"></a>
>
> In this case, the percentage on the `<aside>` won’t normally resolve, because the containing block’s height is [auto](#valdef-width-auto) (and thus depends on the size of its contents). Instead it [behaves as auto](#behave-as-auto), resulting in a height of 150px for the `<aside>`, and an initial height of 180px for the `<article>` The [min-height](#propdef-min-height) doesn’t change this; [height: min-content;](#propdef-height) acts similarly to <a id="ref-for-propdef-height②②"></a>height: auto; and results in the same sizes.
>
> ```html
> <article style="height:100px; min-height: min-content;">
>   <aside style="height: 200%;">
>     <div style="height: 150px;"></div>
>   </aside>
>   <section style="height: 30px;"></section>
> </article>
> ```
>
> <a id="ref-for-propdef-min-height⑨"></a>
>
> <a id="ref-for-behave-as-auto⑥"></a>
>
> <a id="ref-for-valdef-width-min-content⑤"></a>
>
> This is a variation on the first code block, and follows a similar path; the `<aside>` initially wants to compute to 200px tall (200% of the 100px containing block height). When we calculate the effects of [min-height](#propdef-min-height), the percentage [behaves as auto](#behave-as-auto), causing it to become 150px tall, and the total [min-content](#valdef-width-min-content) height of the containing block to be 180px tall. Since this is larger than 100px, the `<article>` gets clamped to 180px, the percentage resolves against this new height, and the `<aside>` ends up being 360px tall, overflowing the `<article>`

#### <a id="min-content-zero"></a>5.2.2.  Compressible Replaced Elements

<a id="ref-for-replaced-element⑤"></a>

<a id="ref-for-min-content-contribution①⓪"></a>

<a id="ref-for-propdef-width②④"></a>

<a id="ref-for-propdef-height②③"></a>

<a id="ref-for-propdef-max-width⑥"></a>

<a id="ref-for-propdef-max-height⑥"></a>

In addition to the [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) listed in [HTML§14.4](https://html.spec.whatwg.org/multipage/rendering.html#replaced-elements) [\[HTML\]](#biblio-html), the following HTML elements are also considered to be <a id="ref-for-replaced-element⑥"></a>replaced elements for the purpose of the [percentage-sized replaced element rule](#replaced-percentage-min-contribution) above, and can have their [min-content contribution](#min-content-contribution) compressed when their [width](#propdef-width)/[height](#propdef-height) or [max-width](#propdef-max-width)/[max-height](#propdef-max-height) is expressed with a cyclic percentage size:

- <a id="ref-for-the-input-element②"></a>

  <a id="ref-for-attr-input-type"></a>

  <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> with any <code><a href="https://html.spec.whatwg.org/multipage/input.html#attr-input-type">type</a></code> that is not "button-like"; this can vary depending on the UA.

  <a id="ref-for-the-button-element①"></a>

  A type is "button-like" in a particular UA if it displays similar to a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element">button</a></code> element, where it can contains actual content that determines the layout of the element. In most UAs, the "button", "reset", "submit", and "color" types are button-like; the "file" type is also partially button-like in some UAs, when it’s displayed as a combination of a text input (shrinkable) and a button (button-like, and thus not shrinkable).

- <a id="ref-for-the-select-element①"></a>

  <a id="ref-for-the-textarea-element①"></a>

  <a id="ref-for-the-progress-element"></a>

  <a id="ref-for-the-meter-element"></a>

  <a id="ref-for-the-marquee-element"></a>

  <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-progress-element">progress</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-meter-element">meter</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/obsolete.html#the-marquee-element">marquee</a></code>.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d999780f"></a> Tracking web-compat &#x26; implementation progress of applying this to max-width/height in [Issue 6348](https://github.com/w3c/csswg-drafts/issues/6348). [\[Issue \#6348\]](https://github.com/w3c/csswg-drafts/issues/6348)

## <a id="changes"></a> Changes

### <a id="changes-recent"></a> Recent Changes

Changes since the [18 December 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-3-20201218/) include:

- Fixed various errors in definition of max-content sizes of replaced elements in [§ 5.1 Intrinsic Sizes](#intrinsic-sizes). ([Issue 6072](https://github.com/w3c/csswg-drafts/issues/6072))

- <a id="ref-for-fit-content-size①"></a>

  <a id="ref-for-min-content-constraint③"></a>

  Added missing statement handling [min-content constraint](#min-content-constraint) to definition of [fit-content size](#fit-content-size).

- <a id="ref-for-intrinsic-size⑤"></a>

  Renamed replaced element “intrinsic” dimensions to “natural” dimensions in order to avoid confusion with [intrinsic sizes](#intrinsic-size) (see [Issue 4961](https://github.com/w3c/csswg-drafts/issues/4961)).

- Various other minor editorial fixes and improvements.

Major changes since the [22 May 2019 Working Draft](https://www.w3.org/TR/2019/WD-css-sizing-3-20190522/) include:

- <a id="ref-for-valdef-width-max-content④"></a>

  <a id="ref-for-valdef-width-min-content⑥"></a>

  Defined that [min-content](#valdef-width-min-content) and [max-content](#valdef-width-max-content) do not necessarily behave the same as the property’s initial value if otherwise specified (by the relevant layout module). ([Issue 3973](https://github.com/w3c/csswg-drafts/issues/3973))

- <a id="ref-for-valdef-width-fit-content-length-percentage⑥"></a>

  Switched intrinsic contribution of [fit-content()](#valdef-width-fit-content-length-percentage) to treat its argument as that argument would be treated alone for intrinsic contribution calculations and resolve the fit-content formula accordingly, rather than having special behavior for <a id="ref-for-valdef-width-fit-content-length-percentage⑦"></a>fit-content() resolution when calculating intrinsic contributions. ([Issue 3731](https://github.com/w3c/csswg-drafts/issues/3731))

- <a id="ref-for-length-value⑥"></a>

  <a id="ref-for-min-width⑥"></a>

  <a id="ref-for-max-content①③"></a>

  Changed the [max-content size](#max-content) of replaced boxes without an intrinsic size to use their [minimum size](#min-width) in place of ICB or 300px×150px only when it is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), see [§ 5.1 Intrinsic Sizes](#intrinsic-sizes). ([Issue 4217](https://github.com/w3c/csswg-drafts/issues/4217))

- Switched default sizing of an object with a natural aspect ratio to use the ICB size instead of 300px×150px. ([Issue 4218](https://github.com/w3c/csswg-drafts/issues/4218))

- <a id="ref-for-preferred-aspect-ratio⑧"></a>

  Defined [preferred aspect ratio](#preferred-aspect-ratio) and used it in place of “intrinsic aspect ratio” where appropriate.

- Miscellaneous minor / editorial fixes.

Major changes since the [4 March 2018 Working Draft](https://www.w3.org/TR/2018/WD-css-sizing-3-20180304/) include:

- <a id="ref-for-propdef-box-sizing⑦"></a>

  Imported the [box-sizing](#propdef-box-sizing) definition from [CSS UI Level 3](https://www.w3.org/TR/css-ui-3/).

- <a id="ref-for-cyclic-percentage-size④"></a>

  More rigorously specified handling of [cyclic percentages](#cyclic-percentage-size). ([\#1132](https://github.com/w3c/csswg-drafts/issues/1132), [\#2384](https://github.com/w3c/csswg-drafts/issues/2384), [\#2297](https://github.com/w3c/csswg-drafts/issues/2297), [\#2674](https://github.com/w3c/csswg-drafts/issues/2674))

- Changed the \*-content values applied to the bock axis to not compute to the property’s initial value, but to rather “behave as” the property’s initial value. ([\#2708](https://github.com/w3c/csswg-drafts/issues/2708))

- Fixed miscellaneous trivial errors.

Major changes since the [7 February 2017 Working Draft](https://www.w3.org/TR/2017/WD-css-sizing-3-20170207/) include:

- More accurate definition of min-content and max-content sizes for replaced elements.

- <a id="ref-for-valdef-width-auto①⑧"></a>

  Compute new keywords to the initial value, not to a potentially non-existent [auto](#valdef-width-auto), when applied to the block axis.

- Specify that percent sizes on replaced elements zero out their min-content contribution.

- Fix confusing/wrong definition of percentage sizes resolved against a dependent containing block. (This may require further work.)

- <a id="ref-for-indefinite⑥"></a>

  <a id="ref-for-valdef-width-fit-content①"></a>

  <a id="ref-for-valdef-width-stretch②"></a>

  Deferred the [stretch](https://www.w3.org/TR/css-sizing-4/#valdef-width-stretch) and [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) keywords to Level 4 to allow for further consideration of their behavior in [indefinite](#indefinite) containing blocks.

- <a id="ref-for-propdef-box-sizing⑧"></a>

  <a id="ref-for-propdef-max-height⑦"></a>

  <a id="ref-for-propdef-min-height①⓪"></a>

  <a id="ref-for-propdef-min-width⑦"></a>

  <a id="ref-for-propdef-height②④"></a>

  <a id="ref-for-propdef-width②⑤"></a>

  Pulled in full definitions for all of the sizing properties (rather than diffing them): [width](#propdef-width), [height](#propdef-height), [min-width](#propdef-min-width), [min-height](#propdef-min-height), max-width', [max-height](#propdef-max-height), and [box-sizing](#propdef-box-sizing).

### <a id="changes-3"></a> Additions since CSS Level 2

In addition to substantially more detail to the various automatic and content-based sizing algorithms, the following new features have been added since [\[CSS2\]](#biblio-css2):

- <a id="ref-for-propdef-box-sizing⑨"></a>

  The [box-sizing](#propdef-box-sizing) property (originally defined in [\[CSS-UI-3\]](#biblio-css-ui-3), then moved here).

- <a id="ref-for-sizing-property⑤"></a>

  <a id="ref-for-valdef-width-fit-content-length-percentage⑧"></a>

  <a id="ref-for-valdef-width-max-content⑤"></a>

  <a id="ref-for-valdef-width-min-content⑦"></a>

  The [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content()](#valdef-width-fit-content-length-percentage) values of the [sizing properties](#sizing-property).

- <a id="ref-for-propdef-min-height①①"></a>

  <a id="ref-for-propdef-min-width⑧"></a>

  <a id="ref-for-valdef-width-auto①⑨"></a>

  The [auto](#valdef-width-auto) initial value of the [min-width](#propdef-min-width) and [min-height](#propdef-min-height) properties (originally defined in [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1), then moved here).

## <a id="acknowledgments"></a> Acknowledgments

Special thanks go to L. David Baron, Aaron Gustafson, Daniel Holbert, and Mats Palmgren for their contributions to this module.

## <a id="priv-sec"></a> Privacy and Security Considerations

<a id="ref-for-the-iframe-element①"></a>

<a id="ref-for-replaced-element⑦"></a>

In order to support automatic layout, CSS sizes boxes to fit their contents. In conjunction with various [\[DOM\]](#biblio-dom) and [\[CSSOM\]](#biblio-cssom) APIs which can return the size of those boxes to script, this can expose information about those contents. However, this information is more directly and easily available by inspecting the DOM for the contents, rather than indirecting through the box’s size. Containers that can’t have their contents inspected (such as cross-origin <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>s) also do not expose sizing information to the outer page, except insofar as [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) such as images expose their natural size and/or aspect ratio.

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

- [auto](#valdef-width-auto), in § 3.2
- [automatic block size](#automatic-block-size), in § 3.2
- [automatic inline size](#automatic-inline-size), in § 3.2
- [automatic minimum size](#automatic-minimum-size), in § 3.2
- [automatic size](#automatic-size), in § 3.2
- [available](#available), in § 2
- [available block space](#available), in § 2
- [available inline space](#available), in § 2
- [available space](#available), in § 2
- [behave as auto](#behave-as-auto), in § 3.2.1
- [behaves as auto](#behave-as-auto), in § 3.2.1
- [behaving as auto](#behave-as-auto), in § 3.2.1
- [border-box](#valdef-box-sizing-border-box), in § 3.3
- [box-sizing](#propdef-box-sizing), in § 3.3
- [content-box](#valdef-box-sizing-content-box), in § 3.3
- [cyclic percentage](#cyclic-percentage-size), in § 5.2.1
- [cyclic percentage size](#cyclic-percentage-size), in § 5.2.1
- [definite](#definite), in § 2
- [definite size](#definite), in § 2
- [Extrinsic sizing](#extrinsic-sizing), in § 4
- [fallback](#fallback), in § 2
- [fallback size](#fallback), in § 2
- [fit-content block size](#fit-content-block-size), in § 2.1
- [fit-content inline size](#fit-content-inline-size), in § 2.1
- fit-content(\<length-percentage\>)
  - [value for column-width](#valdef-column-width-fit-content-length-percentage), in § 3.4
  - [value for width, min-width, max-width, height, min-height, max-height](#valdef-width-fit-content-length-percentage), in § 3.2
- [fit-content size](#fit-content-size), in § 2.1
- height
  - [(property)](#propdef-height), in § 3.1.1
  - [definition of](#height), in § 3.1.1
- [indefinite](#indefinite), in § 2
- [indefinite size](#indefinite), in § 2
- [inner block size](#inner-size), in § 2
- [inner height](#inner-size), in § 2
- [inner inline size](#inner-size), in § 2
- [inner size](#inner-size), in § 2
- [inner width](#inner-size), in § 2
- [intrinsic size](#intrinsic-size), in § 2.1
- [intrinsic size constraint](#constraints), in § 2.2
- [intrinsic size contribution](#intrinsic-size-contribution), in § 2.2
- [Intrinsic sizing](#intrinsic-sizing), in § 5
- [\<length-percentage\>](#valdef-width-length-percentage), in § 3.2
- max-content
  - [definition of](#max-content), in § 2.1
  - [value for column-width](#valdef-column-width-max-content), in § 3.4
  - [value for width, min-width, max-width, height, min-height, max-height](#valdef-width-max-content), in § 3.2
- [max-content block size](#max-content-block-size), in § 2.1
- [max-content block-size contribution](#max-content-contribution), in § 2.2
- [max-content constraint](#max-content-constraint), in § 2.3
- [max-content contribution](#max-content-contribution), in § 2.2
- [max-content inline size](#max-content-inline-size), in § 2.1
- [max-content inline-size contribution](#max-content-contribution), in § 2.2
- [max-content size](#max-content), in § 2.1
- [max height](#max-height), in § 3.1.3
- [max-height](#propdef-max-height), in § 3.1.3
- [maximum height](#max-height), in § 3.1.3
- [maximum size](#max-width), in § 3.1.3
- [maximum width](#max-width), in § 3.1.3
- [max size](#max-width), in § 3.1.3
- [max size property](#max-size-properties), in § 3.1.2
- [max width](#max-width), in § 3.1.3
- [max-width](#propdef-max-width), in § 3.1.3
- min-content
  - [definition of](#min-content), in § 2.1
  - [value for column-width](#valdef-column-width-min-content), in § 3.4
  - [value for width, min-width, max-width, height, min-height, max-height](#valdef-width-min-content), in § 3.2
- [min-content block size](#min-content-block-size), in § 2.1
- [min-content block-size contribution](#min-content-contribution), in § 2.2
- [min-content constraint](#min-content-constraint), in § 2.3
- [min-content contribution](#min-content-contribution), in § 2.2
- [min-content inline size](#min-content-inline-size), in § 2.1
- [min-content inline-size contribution](#min-content-contribution), in § 2.2
- [min-content size](#min-content), in § 2.1
- [min height](#min-height), in § 3.1.2
- [min-height](#propdef-min-height), in § 3.1.2
- [minimum height](#min-height), in § 3.1.2
- [minimum size](#min-width), in § 3.1.2
- [minimum width](#min-width), in § 3.1.2
- [min size](#min-width), in § 3.1.2
- [min size property](#min-size-properties), in § 3.1.1
- [min width](#min-width), in § 3.1.2
- [min-width](#propdef-min-width), in § 3.1.2
- [none](#valdef-max-width-none), in § 3.2
- [outer block size](#outer-size), in § 2
- [outer height](#outer-size), in § 2
- [outer inline size](#outer-size), in § 2
- [outer size](#outer-size), in § 2
- [outer width](#outer-size), in § 2
- [preferred aspect ratio](#preferred-aspect-ratio), in § 2.3
- [preferred height](#height), in § 3.1.1
- [preferred size](#preferred-size), in § 3.1.1
- [preferred size property](#preferred-size-properties), in § 3.1
- [preferred width](#width), in § 3.1.1
- [size](#size), in § 2
- [sizing property](#sizing-property), in § 3.1
- [stretch fit](#stretch-fit), in § 2
- [stretch-fit block size](#stretch-fit-block-size), in § 2.1
- [stretch-fit inline size](#stretch-fit-inline-size), in § 2.1
- [stretch-fit size](#stretch-fit-size), in § 2.1
- width
  - [(property)](#propdef-width), in § 3.1.1
  - [definition of](#width), in § 3.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-align-3\] defines the following terms:
  - <a id="term-for-gutter"></a>gutter
- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-border"></a>border
- \[css-box-4\] defines the following terms:
  - <a id="term-for-border-box"></a>border box
  - <a id="term-for-content-box"></a>content box
  - <a id="term-for-margin"></a>margin
  - <a id="term-for-margin-properties"></a>margin properties
  - <a id="term-for-padding"></a>padding
  - <a id="term-for-padding-properties"></a>padding properties
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-initial-value"></a>initial value
- \[css-display-3\] defines the following terms:
  - <a id="term-for-block-box"></a>block box
  - <a id="term-for-block-container"></a>block container
  - <a id="term-for-box"></a>box
  - <a id="term-for-containing-block"></a>containing block
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-display-type"></a>display type
  - <a id="term-for-formatting-context"></a>formatting context
  - <a id="term-for-initial-containing-block"></a>initial containing block
  - <a id="term-for-inline"></a>inline
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-non-replaced"></a>non-replaced
  - <a id="term-for-replaced-element"></a>replaced
  - <a id="term-for-replaced-element①"></a>replaced element
  - <a id="term-for-text-run"></a>text run
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="term-for-flex-item"></a>flex item
  - <a id="term-for-propdef-flex-basis"></a>flex-basis
- \[css-grid-2\] defines the following terms:
  - <a id="term-for-grid-item"></a>grid item
- \[css-images-3\] defines the following terms:
  - <a id="term-for-natural-aspect-ratio"></a>natural aspect ratio
  - <a id="term-for-natural-dimensions"></a>natural dimension
  - <a id="term-for-natural-size"></a>natural size
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="term-for-propdef-block-size"></a>block-size
  - <a id="term-for-propdef-max-block-size"></a>max-block-size
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="term-for-at-ruledef-page"></a>@page
  - <a id="term-for-descdef-page-size"></a>size
- \[css-sizing-4\] defines the following terms:
  - <a id="term-for-valdef-width-contain"></a>contain
  - <a id="term-for-valdef-width-fit-content"></a>fit-content
  - <a id="term-for-valdef-width-stretch"></a>stretch
- \[css-text-3\] defines the following terms:
  - <a id="term-for-soft-wrap-opportunity"></a>soft wrap opportunity
- \[css-values-4\] defines the following terms:
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
  - <a id="term-for-block-size"></a>block size
  - <a id="term-for-flow-relative"></a>flow-relative
  - <a id="term-for-inline-axis"></a>inline axis
  - <a id="term-for-inline-size"></a>inline size
  - <a id="term-for-physical"></a>physical
  - <a id="term-for-writing-mode"></a>writing mode
- \[CSS3COL\] defines the following terms:
  - <a id="term-for-propdef-column-width"></a>column-width
- \[CSSOM\] defines the following terms:
  - <a id="term-for-dom-window-getcomputedstyle"></a>getComputedStyle(elt)
  - <a id="term-for-resolved-value"></a>resolved value
- \[HTML\] defines the following terms:
  - <a id="term-for-the-button-element"></a>button
  - <a id="term-for-the-iframe-element"></a>iframe
  - <a id="term-for-the-input-element"></a>input
  - <a id="term-for-the-marquee-element"></a>marquee
  - <a id="term-for-the-meter-element"></a>meter
  - <a id="term-for-the-progress-element"></a>progress
  - <a id="term-for-concept-textarea-raw-value"></a>raw value
  - <a id="term-for-the-select-element"></a>select
  - <a id="term-for-the-textarea-element"></a>textarea
  - <a id="term-for-attr-input-type"></a>type

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 3 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 October 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3col"></a>\[CSS3COL\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 12 October 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                             | Initial     | Applies to                               | Inh. | %ages                                        | Ani­mat­able                                      | Anim­ation type                                       | Canonical order | Com­puted value                                           |
|---------------------|---------------------------------------------------------------------------------------------------|-------------|------------------------------------------|------|----------------------------------------------|-------------------------------------------------|------------------------------------------------------|-----------------|----------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-box-sizing①⓪"></a></span><a href="#propdef-box-sizing">box-sizing</a>&#xA;      </strong> | content-box \| border-box                                                                         | content-box | all elements that accept width or height | no   | N/A                                          |                                                 | discrete                                             | per grammar     | specified keyword                                        |
| <strong><span><a id="ref-for-propdef-height②⑤"></a></span><a href="#propdef-height">height</a>&#xA;      </strong> | auto \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block |                                                 | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed |
| <strong><span><a id="ref-for-propdef-max-height⑧"></a></span><a href="#propdef-max-height">max-height</a>&#xA;      </strong> | none \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content() |                                                      | per grammar     | as specified, with \<length-percentage\> values computed |
| <strong><span><a id="ref-for-propdef-max-width⑦"></a></span><a href="#propdef-max-width">max-width</a>&#xA;      </strong> | none \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content() |                                                      | per grammar     | as specified, with \<length-percentage\> values computed |
| <strong><span><a id="ref-for-propdef-min-height①②"></a></span><a href="#propdef-min-height">min-height</a>&#xA;      </strong> | auto \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content() |                                                      | per grammar     | as specified, with \<length-percentage\> values computed |
| <strong><span><a id="ref-for-propdef-min-width⑨"></a></span><a href="#propdef-min-width">min-width</a>&#xA;      </strong> | auto \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content() |                                                      | per grammar     | as specified, with \<length-percentage\> values computed |
| <strong><span><a id="ref-for-propdef-width②⑥"></a></span><a href="#propdef-width">width</a>&#xA;      </strong> | auto \| \<length-percentage\> \| min-content \| max-content \| fit-content(\<length-percentage\>) | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block |                                                 | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This spec needs illustrations! See [issue](https://github.com/w3c/csswg-drafts/issues/1938). [↵](#issue-664b2cc3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We would like to define shorthands for each pair of sizing properties (e.g. [width](#propdef-width) and [height](#propdef-height)) but there is a naming conflict with the [@page](https://www.w3.org/TR/css-page-3/#at-ruledef-page) [size](https://www.w3.org/TR/css-page-3/#descdef-page-size) descriptor [\[CSS-PAGE-3\]](#biblio-css-page-3), so this has been deferred to Level 4. Suggestions on how to resolve this problem are welcome, see [discussion](https://github.com/w3c/csswg-drafts/issues/820). [↵](#issue-6eb2c542)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Replace this section with references to the new term [automatic size](#automatic-size). [↵](#issue-6215c343)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should we resolve transferred percentages against their containing block instead of zero before transferring them? See [discussion](https://github.com/w3c/csswg-drafts/issues/6341). [↵](#issue-1b92b23f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Letting percentages still resolve against a definite [height](#propdef-height) when the min-height is intrinsic is an open issue. (CSS2 has a general statement about "height depending on contents", which this technically is, even though CSS2 didn’t have content-dependent keywords for [min-height](#propdef-min-height). Since this is new, we think we could have this different behavior.) [↵](#issue-9b3707fe)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Tracking web-compat &#x26; implementation progress of applying this to max-width/height in [Issue 6348](https://github.com/w3c/csswg-drafts/issues/6348). [\[Issue \#6348\]](https://github.com/w3c/csswg-drafts/issues/6348) [↵](#issue-d999780f)
