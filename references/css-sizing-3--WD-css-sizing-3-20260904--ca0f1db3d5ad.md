Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 3](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 3

Source snapshot: https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/

Snapshot SHA-256: ca0f1db3d5ad0c9799bf9dc04b9c30b4d9adf8d4d6f222787e4759ed81a6bbad

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 7 source tables are presented as readable Markdown tables or explicit labeled layouts: 6 ordinary table conversions, 1 complex-table layout. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Box Sizing Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-sizing” in the title, like this: “\[css-sizing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-sizing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-column-width"></a>

  Additions to [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

CSS layout has several different concepts of automatic sizing that are used in various layout calculations. This section defines some more precise terminology to help connect the layout behaviors of this spec to the calculations used in other modules, and some new keywords for the [width](#propdef-width) and [height](#propdef-height) properties to allow authors to assign elements the dimensions resulting from these size calculations.

Tests

General sizing tests

- [dynamic-available-size-iframe.html](https://wpt.fyi/results/css/css-sizing/dynamic-available-size-iframe.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-available-size-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-available-size-iframe.html)
- [dynamic-change-inline-size-001.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-001.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-001.html)
- [dynamic-change-inline-size-002.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-002.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-002.html)
- [dynamic-change-inline-size-003.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-003.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-003.html)
- [dynamic-change-inline-size-004.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-004.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-004.html)
- [frameset-intrinsic-crash.html](https://wpt.fyi/results/css/css-sizing/frameset-intrinsic-crash.html) [(live test)](http://wpt.live/css/css-sizing/frameset-intrinsic-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/frameset-intrinsic-crash.html)
- [inheritance-001.html](https://wpt.fyi/results/css/css-sizing/inheritance-001.html) [(live test)](http://wpt.live/css/css-sizing/inheritance-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/inheritance-001.html)
- [inheritance-002.html](https://wpt.fyi/results/css/css-sizing/inheritance-002.html) [(live test)](http://wpt.live/css/css-sizing/inheritance-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/inheritance-002.html)
- [min-width-max-width-precedence.html](https://wpt.fyi/results/css/css-sizing/min-width-max-width-precedence.html) [(live test)](http://wpt.live/css/css-sizing/min-width-max-width-precedence.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-width-max-width-precedence.html)
- [min-width-max-width-precedence.html](https://wpt.fyi/results/css/css-sizing/min-width-max-width-precedence.html) [(live test)](http://wpt.live/css/css-sizing/min-width-max-width-precedence.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-width-max-width-precedence.html)
- [replaced-max-size-saturation.html](https://wpt.fyi/results/css/css-sizing/replaced-max-size-saturation.html) [(live test)](http://wpt.live/css/css-sizing/replaced-max-size-saturation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-max-size-saturation.html)
- [responsive-iframe-no-match-element.html](https://wpt.fyi/results/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html) [(live test)](http://wpt.live/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html)
- [textarea-large-padding-crash.html](https://wpt.fyi/results/css/css-sizing/textarea-large-padding-crash.html) [(live test)](http://wpt.live/css/css-sizing/textarea-large-padding-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/textarea-large-padding-crash.html)

------------------------------------------------------------------------

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

This module extends the [width](#propdef-width), [height](#propdef-height), [min-width](#propdef-min-width), [min-height](#propdef-min-height), [max-width](#propdef-max-width), [max-height](#propdef-max-height), and [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width) features defined in [\[CSS2\]](#biblio-css2) chapter 10 and in [\[CSS3COL\]](#biblio-css3col)

<a id="ref-for-propdef-box-sizing"></a>

The definition of the [box-sizing](#propdef-box-sizing) property in this module supersedes the one in [\[CSS-UI-3\]](#biblio-css-ui-3).

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terms"></a>2.  Terminology

Some key terminology related to coordinate axises and dimensions is defined in [CSS Writing Modes 3 § 6 Abstract Box Terminology](https://www.w3.org/TR/css-writing-modes-3/#abstract-box).

<a id="size"></a>size  
<a id="ref-for-height"></a>

<a id="ref-for-width"></a>

<a id="ref-for-inline-size"></a>

<a id="ref-for-block-size"></a>

A one- or two-dimensional measurement: a [block size](#block-size) and/or [inline size](#inline-size); alternatively a [width](#width) and/or [height](#height).

![In left-to-right, top-to-bottom horizontal English, the horizontal width and inline size are synonymous, and vertical height and block size are synonymous.](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/images/sizing-ltr-tb.svg)![In top-to-bottom, right-to-left vertical Japanese, the horizontal width and block size are synonymous, and vertical height and inline size are synonymous.](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/images/sizing-ttb-rl.svg)

<a id="ref-for-width①"></a>

<a id="ref-for-height①"></a>

<a id="ref-for-inline-size①"></a>

<a id="ref-for-block-size①"></a>

<a id="ref-for-writing-mode"></a>

Whether the [width](#width) or [height](#height) corresponds to an [inline size](#inline-size) or [block size](#block-size) depends on the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

<a id="inner-size"></a>inner size  
<a id="ref-for-box"></a>

<a id="ref-for-size"></a>

The [content-box](https://www.w3.org/TR/css2/box.html#box-dimensions) [size](#size) of a [box](https://www.w3.org/TR/css-display-3/#box).

![](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/images/inner-size.svg)

Inner size

<a id="outer-size"></a>outer size  
<a id="ref-for-box①"></a>

<a id="ref-for-size①"></a>

The [margin-box](https://www.w3.org/TR/css2/box.html#box-dimensions) [size](#size) of a [box](https://www.w3.org/TR/css-display-3/#box).

![](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/images/outer-size.svg)

Outer size

<a id="definite"></a>definite size  
<a id="ref-for-definite"></a>

<a id="ref-for-percentage-value"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-length-value"></a>

A size that can be determined without performing layout; that is, a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), a measure of text (without consideration of line-wrapping), a size of the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block), or a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or other formula (such the [“stretch-fit” sizing of non-replaced blocks](https://www.w3.org/TR/CSS2/visudet.html#blockwidth) [\[CSS2\]](#biblio-css2)) that is resolved solely against [definite](#definite) sizes.

<a id="ref-for-containing-block"></a>

<a id="ref-for-definite①"></a>

Additionally, the size of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) of an absolutely positioned element is always [definite](#definite) with respect to that element.

<a id="indefinite"></a>indefinite size  
<a id="ref-for-available"></a>

<a id="ref-for-indefinite"></a>

<a id="ref-for-definite②"></a>

A size that is not [definite](#definite). [Indefinite](#indefinite) [available space](#available) is essentially infinite.

<a id="ref-for-intrinsic-sizing"></a>

<a id="ref-for-valdef-width-max-content"></a>

<a id="ref-for-size-containment"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [intrinsic sizing](#intrinsic-sizing) keywords such as [max-content](#valdef-width-max-content) are indefinite, even if they can be determined without laying out the children e.g. due to [size containment](https://www.w3.org/TR/css-contain-2/#size-containment) or lack of children.

<a id="available"></a>available space  
<a id="ref-for-max-content-constraint"></a>

<a id="ref-for-min-content-constraint"></a>

<a id="ref-for-available①"></a>

<a id="ref-for-indefinite①"></a>

<a id="ref-for-definite③"></a>

<a id="ref-for-containing-block①"></a>

A size representing the space into which a box is laid out, as determined by the rules of the formatting context in which it participates. The space available to a box is usually either a measurement of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) (if that is [definite](#definite)) or an infinite size (when it is [indefinite](#indefinite)). [Available space](#available) can alternatively be either a [min-content constraint](#min-content-constraint) or a [max-content constraint](#max-content-constraint), which forces boxes laid into it to be laid out under that constraint.

Tests

- [available-height-for-replaced-content-001.html](https://wpt.fyi/results/css/css-sizing/available-height-for-replaced-content-001.html) [(live test)](http://wpt.live/css/css-sizing/available-height-for-replaced-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/available-height-for-replaced-content-001.html)
- [table-percentage-max-width-beside-float.html](https://wpt.fyi/results/css/css-sizing/table-percentage-max-width-beside-float.html) [(live test)](http://wpt.live/css/css-sizing/table-percentage-max-width-beside-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/table-percentage-max-width-beside-float.html)
- [table-percentage-min-width-below-float.html](https://wpt.fyi/results/css/css-sizing/table-percentage-min-width-below-float.html) [(live test)](http://wpt.live/css/css-sizing/table-percentage-min-width-below-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/table-percentage-min-width-below-float.html)
- [table-percentage-min-width-beside-float.html](https://wpt.fyi/results/css/css-sizing/table-percentage-min-width-beside-float.html) [(live test)](http://wpt.live/css/css-sizing/table-percentage-min-width-beside-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/table-percentage-min-width-beside-float.html)

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

Some sizing algorithms do not work well with an infinite size. In these cases, the [fallback size](#fallback) is used instead. Unless otherwise specified, this is the size of the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block).

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

The box’s “ideal” [size](#size) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Usually the narrowest [inline size](#inline-size) it could take while fitting around its contents if <em>none</em> of the soft wrap opportunities within the box were taken. (See [§ 5 Intrinsic Size Determination](#intrinsic).)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “preferred width” in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and the “maximum cell width” in [CSS2.1§17.5.2.2](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout).

<a id="max-content-block-size"></a>max-content block size  
<a id="ref-for-size⑥"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-block-size②"></a>

The box’s “ideal” [size](#size) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Usually the [block size](#block-size) of the content after layout.

<a id="ref-for-max-content"></a>

<a id="ref-for-min-content"></a>

If the ideal [max-content size](#max-content) would be smaller than the [min-content size](#min-content) (e.g. due to the use of negative margins) the effective <a id="ref-for-max-content①"></a>max-content size is floored by the <a id="ref-for-min-content①"></a>min-content size.

<a id="min-content"></a>min-content size  
<a id="ref-for-min-content-constraint①"></a>

<a id="ref-for-size⑦"></a>

Nominally, the smallest [size](#size) a box could take that doesn’t lead to overflow that could be avoided by choosing a larger <a id="ref-for-size⑧"></a>size. Formally, the size of the box when sized under a [min-content constraint](#min-content-constraint), see [§ 5 Intrinsic Size Determination](#intrinsic).

<a id="min-content-inline-size"></a>min-content inline size  
<a id="ref-for-min-content②"></a>

<a id="ref-for-inline-axis②"></a>

<a id="ref-for-inline-size③"></a>

The [min-content size](#min-content) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Typically, the [inline size](#inline-size) that would fit around its contents if <em>all</em> soft wrap opportunities within the box were taken.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “preferred minimum width” in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and the “minimum content width” in [CSS2.1§17.5.2.2](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout).

<a id="min-content-block-size"></a>min-content block size  
<a id="ref-for-min-content③"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-max-content-block-size"></a>

The [min-content size](#min-content) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). For [block containers](https://www.w3.org/TR/css-display-4/#block-container), tables, and [inline boxes](https://www.w3.org/TR/css-display-4/#inline-box), this is equivalent to the [max-content block size](#max-content-block-size).

<a id="fit-content-size"></a>fit-content size  
<a id="fit-content-inline-size"></a>fit-content inline size  
<a id="fit-content-block-size"></a>fit-content block size  
<a id="ref-for-max-content④"></a>

<a id="ref-for-min-content⑥"></a>

<a id="ref-for-min-content-constraint②"></a>

<a id="ref-for-stretch-fit-size①"></a>

<a id="ref-for-max-content③"></a>

<a id="ref-for-min-content⑤"></a>

<a id="ref-for-max-content②"></a>

<a id="ref-for-stretch-fit-size"></a>

<a id="ref-for-min-content④"></a>

<a id="ref-for-definite⑤"></a>

<a id="ref-for-available⑥"></a>

If the [available space](#available) in a given axis is [definite](#definite), equal to <code>clamp(<a href="#min-content">min-content size</a>, <a href="#stretch-fit-size">stretch-fit size</a>, <a href="#max-content">max-content size</a>)</code> (i.e. <code>max(<a href="#min-content">min-content size</a>, min(<a href="#max-content">max-content size</a>, <a href="#stretch-fit-size">stretch-fit size</a>))</code>). When sizing under a [min-content constraint](#min-content-constraint), equal to the [min-content size](#min-content). Otherwise, equal to the [max-content size](#max-content) in that axis.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is called the “shrink-to-fit” width in [CSS2.1§10.3.5](https://www.w3.org/TR/CSS2/visudet.html#float-width) and [CSS Multi-column Layout § 3.4](https://www.w3.org/TR/css3-multicol/#pseudo-algorithm).

<a id="intrinsic-size"></a>intrinsic size  
<a id="ref-for-min-content⑦"></a>

<a id="ref-for-max-content⑤"></a>

A [max-content size](#max-content) or [min-content size](#min-content), i.e. a size arising primarily from the size of the content. (Some uses of this term may refer also to sizes derived primarily from one of these two sizes.)

<a id="ref-for-replaced-element"></a>

<a id="ref-for-intrinsic-size"></a>

<a id="ref-for-natural-dimensions"></a>

[Replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) frequently derive their [intrinsic size](#intrinsic-size) from their [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions).

<a id="ref-for-fit-content-size"></a>

<a id="ref-for-containing-block②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1abdc974"></a> The following example applies [fit-content sizing](#fit-content-size) to the width of the inner `<div>`, and places it within [containing blocks](https://www.w3.org/TR/css-display-4/#containing-block) of varying sizes:
>
> - <a id="ref-for-min-content⑧"></a>
>
>   In the narrowest containing block (2ch), the box takes its [min-content size](#min-content)—​overflowing its containing block in order to fully contain all its content, which has wrapped as narrowly as possible.
>
> - <a id="ref-for-stretch-fit-size②"></a>
>
>   In the middle containing block (9ch), the box takes its [stretch-fit size](#stretch-fit-size)—​filling the containing block exactly while its content wraps to fit inside.
>
> - <a id="ref-for-max-content⑥"></a>
>
>   In the widest containing block (16ch), the box takes its [max-content size](#max-content)—​smaller than the containing block, but fitting around its unwrapped contents exactly.
>
> ```text
> <!DOCTYPE html>
> <style>div > div { width: fit-content; }</style>
> <div style="width: 2ch;">
>   <div>abc def ehg</div>
> </div>
> <div style="width: 9ch;">
>   <div>abc def ehg</div>
> </div>
> <div style="width: 16ch;">
>   <div>abc def ehg</div>
> </div>
> ```
>
> ![Illustrating the above example: a red box as wide as “abc”, another a little bit wider than “abc def”, and a third as wide as the entire string on a single line.](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/images/fit-content.png)
>
> <a id="ref-for-propdef-min-width①"></a>
>
> <a id="ref-for-propdef-max-width①"></a>
>
> <a id="ref-for-propdef-width②"></a>
>
> <a id="ref-for-min-width"></a>
>
> <a id="ref-for-max-width"></a>
>
> <a id="ref-for-width②"></a>
>
> <a id="ref-for-used-value"></a>
>
> Using the [min-width](#propdef-min-width) or [max-width](#propdef-max-width) property in place of [width](#propdef-width) here would apply these same sizes as the [minimum width](#min-width) or [maximum width](#max-width) (respectively), constraining the [preferred width](#width) to find the [used width](https://www.w3.org/TR/css-cascade-5/#used-value). See [§ 3 Specifying Box Sizes](#specifying-sizes) and [§ 3.2 Sizing Values: the \<length-percentage\>, auto \| none, stretch, min-content, max-content, and fit-content values](#sizing-values).

### <a id="contributions"></a>2.2.  Intrinsic Size Contributions

<a id="max-content-contribution"></a>max-content contribution  
<a id="ref-for-max-content⑦"></a>

<a id="ref-for-containing-block③"></a>

The size that a box contributes to its [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s [max-content size](#max-content).

<a id="min-content-contribution"></a>min-content contribution  
<a id="ref-for-min-content⑨"></a>

<a id="ref-for-containing-block④"></a>

The size that a box contributes to its [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s [min-content size](#min-content).

<a id="intrinsic-size-contribution"></a>intrinsic size contribution  
<a id="ref-for-min-content-contribution"></a>

<a id="ref-for-max-content-contribution"></a>

A [max-content contribution](#max-content-contribution), [min-content contribution](#min-content-contribution), or similarly-calculated content-based size contribution.

<a id="ref-for-outer-size①"></a>

Intrinsic size contributions are based on the [outer size](#outer-size) of the box; for this purpose auto margins are treated as zero.

<a id="ref-for-max-content-contribution①"></a>

<a id="ref-for-min-content-contribution①"></a>

If the ideal [max-content contribution](#max-content-contribution) would be smaller than the [min-content contribution](#min-content-contribution) (e.g. due to the use of negative margins) the effective <a id="ref-for-max-content-contribution②"></a>max-content contribution is floored by the <a id="ref-for-min-content-contribution②"></a>min-content contribution.

### <a id="constraints"></a>2.3.  Intrinsic Size Constraints

<a id="max-content-constraint"></a>max-content constraint  
<a id="ref-for-max-content-contribution③"></a>

<a id="ref-for-containing-block⑤"></a>

A sizing constraint imposed by the box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) that causes it to produce its [max-content contribution](#max-content-contribution).

<a id="min-content-constraint"></a>min-content constraint  
<a id="ref-for-min-content-contribution③"></a>

<a id="ref-for-containing-block⑥"></a>

A sizing constraint imposed by the box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) that causes it to produce its [min-content contribution](#min-content-contribution).

<a id="preferred-aspect-ratio"></a>preferred aspect ratio  
<a id="ref-for-content-box"></a>

<a id="ref-for-natural-aspect-ratio"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

A width:height ratio inherent to a box, which biases various sizing algorithms to produce a size consistent with that aspect ratio insofar as possible while honoring other sizing inputs. Unless otherwise specified, a box’s [preferred aspect ratio](#preferred-aspect-ratio) is its [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) if it has one and is applied to its [content box](https://www.w3.org/TR/css-box-4/#content-box). Most boxes do not have a <a id="ref-for-preferred-aspect-ratio①"></a>preferred aspect ratio.

Tests

- [replaced-fractional-height-from-aspect-ratio.html](https://wpt.fyi/results/css/css-sizing/replaced-fractional-height-from-aspect-ratio.html) [(live test)](http://wpt.live/css/css-sizing/replaced-fractional-height-from-aspect-ratio.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-fractional-height-from-aspect-ratio.html)

## <a id="specifying-sizes"></a>3. <a id="size-keywords"></a> Specifying Box Sizes

### <a id="sizing-properties"></a>3.1.  Sizing Properties

<a id="ref-for-preferred-size"></a>

<a id="ref-for-min-width①"></a>

<a id="ref-for-max-width①"></a>

This section defines the <a id="sizing-property"></a>sizing properties, which specify the [preferred](#preferred-size), [minimum](#min-width), and [maximum](#max-width) sizes of the box to which they are applied. Their potential values are defined in the next section, [§ 3.2 Sizing Values: the \<length-percentage\>, auto \| none, stretch, min-content, max-content, and fit-content values](#sizing-values).

<a id="ref-for-flow-relative"></a>

<a id="ref-for-propdef-inline-size"></a>

<a id="ref-for-propdef-block-size"></a>

<a id="ref-for-writing-mode①"></a>

<a id="ref-for-physical"></a>

<a id="ref-for-propdef-width③"></a>

<a id="ref-for-propdef-height②"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) variants ([inline-size](#propdef-inline-size), [block-size](#propdef-block-size), etc.) are mapped using the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element itself, and interact with their [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) counterparts ([width](#propdef-width), [height](#propdef-height), etc.) as defined in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1). See also [CSS Writing Modes 3 § 6 Abstract Box Terminology](https://www.w3.org/TR/css-writing-modes-3/#abstract-box).

<a id="ref-for-min-width②"></a>

<a id="ref-for-max-width②"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-inner-size"></a>

In CSS, while the [minimum](#min-width) and [maximum](#max-width) sizes both constrain the [used size](https://www.w3.org/TR/css-cascade-5/#used-value), the <a id="ref-for-min-width③"></a>minimum size constraint is always the strongest constraint. Furthermore, the [inner size](#inner-size) is always floored at zero.

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-propdef-height③"></a>

<a id="ref-for-propdef-inline-size①"></a>

<a id="ref-for-propdef-block-size①"></a>

#### <a id="preferred-size-properties"></a>3.1.1.  Preferred Size Properties: the [width](#propdef-width), [height](#propdef-height), [inline-size](#propdef-inline-size), and [block-size](#propdef-block-size) properties

| Field               | Definition                                                                                                                                                                         |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-width"></a>width, <a id="propdef-height"></a>height, <a id="propdef-inline-size"></a>inline-size, <a id="propdef-block-size"></a>block-size                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-box-size"></a><a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<box-size\>](#typedef-box-size)                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline"></a><a id="ref-for-non-replaced"></a>all elements except [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) [inlines](https://www.w3.org/TR/css-display-4/#inline) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑦"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-4/#containing-block)                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-funcdef-width-fit-content"></a>by computed value type, recursing into [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content)                                        |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-size"></a>[size](https://drafts.csswg.org/css-sizing-4/#propdef-size)                                                                                                     |

Tests

- [height-composition.html](https://wpt.fyi/results/css/css-sizing/animation/height-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/height-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/height-composition.html)
- [height-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/height-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/height-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/height-interpolation.html)
- [height-no-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/height-no-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/height-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/height-no-interpolation.html)
- [width-composition.html](https://wpt.fyi/results/css/css-sizing/animation/width-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/width-composition.html)
- [width-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/width-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/width-interpolation.html)
- [percentage-height-replaced-content-in-auto-cb.html](https://wpt.fyi/results/css/css-sizing/percentage-height-replaced-content-in-auto-cb.html) [(live test)](http://wpt.live/css/css-sizing/percentage-height-replaced-content-in-auto-cb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/percentage-height-replaced-content-in-auto-cb.html)
- [height-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/height-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/height-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/height-invalid.html)
- [height-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/height-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/height-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/height-valid.html)
- [width-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/width-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/width-invalid.html)
- [width-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/width-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/width-valid.html)

<a id="ref-for-propdef-width⑤"></a>

<a id="ref-for-propdef-height④"></a>

<a id="ref-for-physical①"></a>

<a id="ref-for-propdef-inline-size②"></a>

<a id="ref-for-propdef-block-size②"></a>

<a id="ref-for-flow-relative①"></a>

<a id="ref-for-sizing-property"></a>

The [width](#propdef-width) and [height](#propdef-height) ([physical](https://www.w3.org/TR/css-writing-modes-4/#physical)) and [inline-size](#propdef-inline-size) and [block-size](#propdef-block-size) ([flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative)) are [sizing properties](#sizing-property) that specify the <a id="preferred-size"></a>preferred <a id="width"></a>width and <a id="height"></a>height (<a id="ref-for-physical②"></a>physical) or <a id="inline-size"></a>inline size and <a id="block-size"></a>block size (<a id="ref-for-flow-relative②"></a>flow-relative) of the box, respectively.

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-propdef-min-inline-size"></a>

<a id="ref-for-propdef-min-block-size"></a>

#### <a id="min-size-properties"></a>3.1.2.  Minimum Size Properties: the [min-width](#propdef-min-width), [min-height](#propdef-min-height), [min-inline-size](#propdef-min-inline-size), and [min-block-size](#propdef-min-block-size) properties

| Field               | Definition                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-min-width"></a>min-width, <a id="propdef-min-height"></a>min-height, <a id="propdef-min-inline-size"></a>min-inline-size, <a id="propdef-min-block-size"></a>min-block-size           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-box-size①"></a><a id="ref-for-comb-one①"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<box-size\>](#typedef-box-size)              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height⑤"></a><a id="ref-for-propdef-width⑥"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height)                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑧"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-4/#containing-block)                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-funcdef-width-fit-content①"></a>by computed value, recursing into [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content)       |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | min-size                                                                                                                                     |

Tests

- [min-height-composition.html](https://wpt.fyi/results/css/css-sizing/animation/min-height-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/min-height-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/min-height-composition.html)
- [min-height-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/min-height-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/min-height-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/min-height-interpolation.html)
- [min-width-composition.html](https://wpt.fyi/results/css/css-sizing/animation/min-width-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/min-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/min-width-composition.html)
- [min-width-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/min-width-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/min-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/min-width-interpolation.html)
- [button-min-width.html](https://wpt.fyi/results/css/css-sizing/button-min-width.html) [(live test)](http://wpt.live/css/css-sizing/button-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/button-min-width.html)
- [grid-item-image-percentage-min-height-computes-as-0.html](https://wpt.fyi/results/css/css-sizing/grid-item-image-percentage-min-height-computes-as-0.html) [(live test)](http://wpt.live/css/css-sizing/grid-item-image-percentage-min-height-computes-as-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/grid-item-image-percentage-min-height-computes-as-0.html)
- [min-height-computed.html](https://wpt.fyi/results/css/css-sizing/parsing/min-height-computed.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-height-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-height-computed.html)
- [min-height-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/min-height-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-height-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-height-invalid.html)
- [min-height-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/min-height-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-height-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-height-valid.html)
- [min-width-computed.html](https://wpt.fyi/results/css/css-sizing/parsing/min-width-computed.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-width-computed.html)
- [min-width-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/min-width-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-width-invalid.html)
- [min-width-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/min-width-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/min-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/min-width-valid.html)

<a id="ref-for-propdef-min-width③"></a>

<a id="ref-for-propdef-min-height②"></a>

<a id="ref-for-physical③"></a>

<a id="ref-for-propdef-min-inline-size①"></a>

<a id="ref-for-propdef-min-block-size①"></a>

<a id="ref-for-flow-relative③"></a>

<a id="ref-for-sizing-property①"></a>

The [min-width](#propdef-min-width) and [min-height](#propdef-min-height) ([physical](https://www.w3.org/TR/css-writing-modes-4/#physical)) and [min-inline-size](#propdef-min-inline-size) and [min-block-size](#propdef-min-block-size) ([flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative)) are [sizing properties](#sizing-property) that specify the <a id="min-width"></a>minimum width (“min width”) and <a id="min-height"></a>minimum height (“min height”) or <a id="min-inline-size"></a>minimum inline size (“min inline size”) and <a id="min-block-size"></a>minimum block size (“min block size”) of the box, respectively.

<a id="ref-for-valdef-width-auto②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The initial value of [auto](#valdef-width-auto) is new; in [\[CSS2\]](#biblio-css2) the initial value was zero.

<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-propdef-max-height①"></a>

<a id="ref-for-propdef-max-inline-size"></a>

<a id="ref-for-propdef-max-block-size"></a>

#### <a id="max-size-properties"></a>3.1.3.  Maximum Size Properties: the [max-width](#propdef-max-width), [max-height](#propdef-max-height), [max-inline-size](#propdef-max-inline-size), and [max-block-size](#propdef-max-block-size) properties

| Field               | Definition                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-max-width"></a>max-width, <a id="propdef-max-height"></a>max-height, <a id="propdef-max-inline-size"></a>max-inline-size, <a id="propdef-max-block-size"></a>max-block-size           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-box-size②"></a><a id="ref-for-comb-one②"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<box-size\>](#typedef-box-size)              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height⑥"></a><a id="ref-for-propdef-width⑦"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height)                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block⑨"></a>relative to width/height of [containing block](https://www.w3.org/TR/css-display-4/#containing-block)                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-funcdef-width-fit-content②"></a>by computed value, recursing into [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content)       |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | max-size                                                                                                                                     |

Tests

- [max-height-composition.html](https://wpt.fyi/results/css/css-sizing/animation/max-height-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/max-height-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/max-height-composition.html)
- [max-height-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/max-height-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/max-height-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/max-height-interpolation.html)
- [max-width-composition.html](https://wpt.fyi/results/css/css-sizing/animation/max-width-composition.html) [(live test)](http://wpt.live/css/css-sizing/animation/max-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/max-width-composition.html)
- [max-width-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/max-width-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/max-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/max-width-interpolation.html)
- [block-image-percentage-max-height-inside-inline.html](https://wpt.fyi/results/css/css-sizing/block-image-percentage-max-height-inside-inline.html) [(live test)](http://wpt.live/css/css-sizing/block-image-percentage-max-height-inside-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-image-percentage-max-height-inside-inline.html)
- [image-percentage-max-height-in-anonymous-block.html](https://wpt.fyi/results/css/css-sizing/image-percentage-max-height-in-anonymous-block.html) [(live test)](http://wpt.live/css/css-sizing/image-percentage-max-height-in-anonymous-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-percentage-max-height-in-anonymous-block.html)
- [nested-flexbox-image-percentage-max-height-computes-as-none.html](https://wpt.fyi/results/css/css-sizing/nested-flexbox-image-percentage-max-height-computes-as-none.html) [(live test)](http://wpt.live/css/css-sizing/nested-flexbox-image-percentage-max-height-computes-as-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/nested-flexbox-image-percentage-max-height-computes-as-none.html)
- [max-height-computed.html](https://wpt.fyi/results/css/css-sizing/parsing/max-height-computed.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-height-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-height-computed.html)
- [max-height-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/max-height-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-height-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-height-invalid.html)
- [max-height-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/max-height-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-height-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-height-valid.html)
- [max-width-computed.html](https://wpt.fyi/results/css/css-sizing/parsing/max-width-computed.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-width-computed.html)
- [max-width-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/max-width-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-width-invalid.html)
- [max-width-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/max-width-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/max-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/max-width-valid.html)

<a id="ref-for-propdef-max-width③"></a>

<a id="ref-for-propdef-max-height②"></a>

<a id="ref-for-physical④"></a>

<a id="ref-for-propdef-max-inline-size①"></a>

<a id="ref-for-propdef-max-block-size①"></a>

<a id="ref-for-flow-relative④"></a>

<a id="ref-for-sizing-property②"></a>

The [max-width](#propdef-max-width) and [max-height](#propdef-max-height) ([physical](https://www.w3.org/TR/css-writing-modes-4/#physical)) and [max-inline-size](#propdef-max-inline-size) and [max-block-size](#propdef-max-block-size) ([flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative)) are [sizing properties](#sizing-property) that specify the <a id="max-width"></a>maximum width (“max width”) and <a id="max-height"></a>maximum height (“max height”) or <a id="max-inline-size"></a>maximum inline size (“max inline size) and <a id="max-block-size"></a>maximum block size (“max block size) of the box, respectively.

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-valdef-width-auto③"></a>

<a id="ref-for-valdef-max-width-none"></a>

<a id="ref-for-valdef-width-stretch"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-max-content①"></a>

<a id="ref-for-valdef-width-fit-content"></a>

### <a id="sizing-values"></a>3.2. <a id="width-height-keywords"></a> Sizing Values: the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), [auto](#valdef-width-auto) \| [none](#valdef-max-width-none), [stretch](#valdef-width-stretch), [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content](#valdef-width-fit-content) values

<a id="ref-for-sizing-property③"></a>

<a id="ref-for-valdef-width-auto④"></a>

<a id="ref-for-valdef-max-width-none①"></a>

<a id="ref-for-typedef-box-size③"></a>

The following values are used in the [sizing properties](#sizing-property). Values other than [auto](#valdef-width-auto) and [none](#valdef-max-width-none) are grouped under the [\<box-size\>](#typedef-box-size) production:

<a id="typedef-box-size"></a>

<a id="ref-for-typedef-box-size④"></a>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

```text
<box-size> = <length-percentage> | stretch | min-content | max-content | fit-content
```
<a id="ref-for-typedef-box-size⑤"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The equivalent of [\<box-size\>](#typedef-box-size) for Level 2 would be just [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) alone.

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="valdef-width-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-border-box"></a>

<a id="ref-for-content-box①"></a>

<a id="ref-for-propdef-box-sizing①"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-length-value①"></a>

Specifies the size of the box using [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) and/or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value). The [box-sizing](#propdef-box-sizing) property indicates whether the [content box](https://www.w3.org/TR/css-box-4/#content-box) or [border box](https://www.w3.org/TR/css-box-4/#border-box) is measured.

<a id="ref-for-containing-block①⓪"></a>

Percentages are resolved against the width/height, as appropriate, of the box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block). If, in a particular axis, the <a id="ref-for-containing-block①①"></a>containing block’s size depends on the box’s size, see the relevant layout module for special rules on how to resolve percentages.

Negative values are invalid.

Tests

- [thin-element-render.html](https://wpt.fyi/results/css/css-sizing/thin-element-render.html) [(live test)](http://wpt.live/css/css-sizing/thin-element-render.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/thin-element-render.html)

<a id="valdef-width-auto"></a>auto

<a id="ref-for-inline-size④"></a>

<a id="ref-for-block-size③"></a>

<a id="ref-for-propdef-height⑦"></a>

<a id="ref-for-propdef-width⑧"></a>

For [width](#propdef-width)/[height](#propdef-height), specifies an <a id="automatic-size"></a>automatic size (<a id="automatic-block-size"></a>automatic [block size](#block-size)/<a id="automatic-inline-size"></a>automatic [inline size](#inline-size)). See the relevant layout module for how to calculate this.

<a id="ref-for-propdef-min-width④"></a>

<a id="ref-for-propdef-min-height③"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-display-type"></a>

<a id="ref-for-propdef-aspect-ratio"></a>

<a id="ref-for-valdef-aspect-ratio-auto"></a>

For [min-width](#propdef-min-width)/[min-height](#propdef-min-height), specifies an <a id="automatic-minimum-size"></a>automatic minimum size. Unless otherwise defined by the relevant layout module, however, it resolves to a used value of 0. For backwards-compatibility, the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of this keyword is zero for boxes of all [\[CSS2\]](#biblio-css2) [display types](https://www.w3.org/TR/css-display-4/#display-type) (block and inline boxes, inline blocks, and all table display types) when [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) is [auto](https://www.w3.org/TR/css-sizing-4/#valdef-aspect-ratio-auto). It also resolves to zero when no box is generated.

<a id="valdef-max-width-none"></a>none

No limit on the size of the box.

<a id="valdef-width-stretch"></a>stretch

<a id="ref-for-containing-block①②"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-stretch-fit-size③"></a>

Applies [stretch-fit sizing](#stretch-fit-size), attempting to match the size of the box’s [margin box](https://www.w3.org/TR/css-box-4/#margin-box) to the size of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block). See [§ 4.2 Stretch-fit Sizing: filling the containing block](#stretch-fit-sizing).

Tests

- [block-height-001.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-001.html)
- [flex-line-001.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-line-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-line-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-line-001.html)
- [flex-line-002.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-line-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-line-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-line-002.html)
- [flex-line-003.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-line-003.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-line-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-line-003.html)
- [flex-line-004.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-line-004.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-line-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-line-004.html)
- [flex-line-005.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-line-005.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-line-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-line-005.html)
- [min-width-1.html](https://wpt.fyi/results/css/css-sizing/stretch/min-width-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/min-width-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/min-width-1.html)
- [parsing.html](https://wpt.fyi/results/css/css-sizing/stretch/parsing.html) [(live test)](http://wpt.live/css/css-sizing/stretch/parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/parsing.html)
- [positioned-non-replaced-1.html](https://wpt.fyi/results/css/css-sizing/stretch/positioned-non-replaced-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/positioned-non-replaced-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/positioned-non-replaced-1.html)
- [positioned-replaced-1.html](https://wpt.fyi/results/css/css-sizing/stretch/positioned-replaced-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/positioned-replaced-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/positioned-replaced-1.html)
- [positioned-replaced-2.html](https://wpt.fyi/results/css/css-sizing/stretch/positioned-replaced-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/positioned-replaced-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/positioned-replaced-2.html)
- [positioned-replaced-3.html](https://wpt.fyi/results/css/css-sizing/stretch/positioned-replaced-3.html) [(live test)](http://wpt.live/css/css-sizing/stretch/positioned-replaced-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/positioned-replaced-3.html)
- [stretch-block-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-block-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-block-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-block-size-001.html)
- [stretch-block-size-002.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-block-size-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-block-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-block-size-002.html)
- [stretch-block-size-003.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-block-size-003.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-block-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-block-size-003.html)
- [stretch-inline-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-inline-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-inline-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-inline-size-001.html)
- [stretch-inline-size-002.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-inline-size-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-inline-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-inline-size-002.html)
- [stretch-inline-size-003.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-inline-size-003.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-inline-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-inline-size-003.html)
- [stretch-max-block-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-max-block-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-max-block-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-max-block-size-001.html)
- [stretch-max-inline-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-max-inline-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-max-inline-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-max-inline-size-001.html)
- [stretch-min-block-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-min-block-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-min-block-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-min-block-size-001.html)
- [stretch-min-inline-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-min-inline-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-min-inline-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-min-inline-size-001.html)
- [stretch-quirk-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-quirk-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-quirk-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-quirk-001.html)

<a id="valdef-width-min-content"></a>min-content

<a id="ref-for-automatic-size"></a>

<a id="ref-for-block-size④"></a>

<a id="ref-for-min-content①⓪"></a>

Use the [min-content size](#min-content) in the relevant axis; for a box’s [block size](#block-size), unless otherwise specified, this is equivalent to its [automatic size](#automatic-size).

Tests

- [clone-intrinsic-size.html](https://wpt.fyi/results/css/css-sizing/clone-intrinsic-size.html) [(live test)](http://wpt.live/css/css-sizing/clone-intrinsic-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/clone-intrinsic-size.html)
- [clone-nowrap-intrinsic-size-bidi.html](https://wpt.fyi/results/css/css-sizing/clone-nowrap-intrinsic-size-bidi.html) [(live test)](http://wpt.live/css/css-sizing/clone-nowrap-intrinsic-size-bidi.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/clone-nowrap-intrinsic-size-bidi.html)
- [clone-nowrap-intrinsic-size.html](https://wpt.fyi/results/css/css-sizing/clone-nowrap-intrinsic-size.html) [(live test)](http://wpt.live/css/css-sizing/clone-nowrap-intrinsic-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/clone-nowrap-intrinsic-size.html)
- [min-content-negative-margin-crash.html](https://wpt.fyi/results/css/css-sizing/min-content-negative-margin-crash.html) [(live test)](http://wpt.live/css/css-sizing/min-content-negative-margin-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-content-negative-margin-crash.html)
- [replaced-max-height-min-content.html](https://wpt.fyi/results/css/css-sizing/replaced-max-height-min-content.html) [(live test)](http://wpt.live/css/css-sizing/replaced-max-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-max-height-min-content.html)
- [replaced-max-size-saturation.html](https://wpt.fyi/results/css/css-sizing/replaced-max-size-saturation.html) [(live test)](http://wpt.live/css/css-sizing/replaced-max-size-saturation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-max-size-saturation.html)
- [replaced-max-width-min-content.html](https://wpt.fyi/results/css/css-sizing/replaced-max-width-min-content.html) [(live test)](http://wpt.live/css/css-sizing/replaced-max-width-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-max-width-min-content.html)
- [replaced-min-height-min-content.html](https://wpt.fyi/results/css/css-sizing/replaced-min-height-min-content.html) [(live test)](http://wpt.live/css/css-sizing/replaced-min-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-min-height-min-content.html)
- [replaced-min-width-min-content.html](https://wpt.fyi/results/css/css-sizing/replaced-min-width-min-content.html) [(live test)](http://wpt.live/css/css-sizing/replaced-min-width-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-min-width-min-content.html)
- [shrink-to-fit-sizing-max-width-min-content.html](https://wpt.fyi/results/css/css-sizing/shrink-to-fit-sizing-max-width-min-content.html) [(live test)](http://wpt.live/css/css-sizing/shrink-to-fit-sizing-max-width-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/shrink-to-fit-sizing-max-width-min-content.html)
- [slice-intrinsic-size.html](https://wpt.fyi/results/css/css-sizing/slice-intrinsic-size.html) [(live test)](http://wpt.live/css/css-sizing/slice-intrinsic-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/slice-intrinsic-size.html)
- [slice-nowrap-intrinsic-size-bidi.html](https://wpt.fyi/results/css/css-sizing/slice-nowrap-intrinsic-size-bidi.html) [(live test)](http://wpt.live/css/css-sizing/slice-nowrap-intrinsic-size-bidi.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/slice-nowrap-intrinsic-size-bidi.html)
- [slice-nowrap-intrinsic-size.html](https://wpt.fyi/results/css/css-sizing/slice-nowrap-intrinsic-size.html) [(live test)](http://wpt.live/css/css-sizing/slice-nowrap-intrinsic-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/slice-nowrap-intrinsic-size.html)
- [svg-no-ar-max-height-min-content.html](https://wpt.fyi/results/css/css-sizing/svg-no-ar-max-height-min-content.html) [(live test)](http://wpt.live/css/css-sizing/svg-no-ar-max-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-no-ar-max-height-min-content.html)
- [svg-no-ar-min-height-min-content.html](https://wpt.fyi/results/css/css-sizing/svg-no-ar-min-height-min-content.html) [(live test)](http://wpt.live/css/css-sizing/svg-no-ar-min-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-no-ar-min-height-min-content.html)

<a id="valdef-width-max-content"></a>max-content

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-block-size⑤"></a>

<a id="ref-for-max-content⑧"></a>

Use the [max-content size](#max-content) in the relevant axis; for a box’s [block size](#block-size), unless otherwise specified, this is equivalent to its [automatic size](#automatic-size).

<a id="valdef-width-fit-content"></a>fit-content

<a id="ref-for-valdef-width-stretch①"></a>

<a id="ref-for-valdef-width-min-content①"></a>

<a id="ref-for-valdef-width-max-content②"></a>

<a id="ref-for-fit-content-size①"></a>

Use the [fit-content size](#fit-content-size) in the relevant axis, i.e. <code>min(<a href="#valdef-width-max-content">max-content</a>, max(<a href="#valdef-width-min-content">min-content</a>, <a href="#valdef-width-stretch">stretch</a>))</code>.

Tests

- [fit-content-block-size-abspos.html](https://wpt.fyi/results/css/css-sizing/fit-content-block-size-abspos.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-block-size-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-block-size-abspos.html)
- [fit-content-block-size-fixedpos.html](https://wpt.fyi/results/css/css-sizing/fit-content-block-size-fixedpos.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-block-size-fixedpos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-block-size-fixedpos.html)
- [fit-content-contribution-001.html](https://wpt.fyi/results/css/css-sizing/fit-content-contribution-001.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-contribution-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-contribution-001.html)
- [fit-content-min-inline-size.html](https://wpt.fyi/results/css/css-sizing/fit-content-min-inline-size.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-min-inline-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-min-inline-size.html)
- [fit-content-percentage-padding.html](https://wpt.fyi/results/css/css-sizing/fit-content-percentage-padding.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-percentage-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-percentage-padding.html)
- [float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-001.html](https://wpt.fyi/results/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-001.html) [(live test)](http://wpt.live/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-001.html)
- [float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-002.html](https://wpt.fyi/results/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-002.html) [(live test)](http://wpt.live/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-002.html)
- [float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-003.html](https://wpt.fyi/results/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-003.html) [(live test)](http://wpt.live/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-003.html)
- [float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-004.html](https://wpt.fyi/results/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-004.html) [(live test)](http://wpt.live/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-004.html)

Tests

- [block-size-with-min-or-max-content-1a.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-1a.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-1a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-1a.html)
- [block-size-with-min-or-max-content-1b.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-1b.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-1b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-1b.html)
- [block-size-with-min-or-max-content-2.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-2.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-2.html)
- [block-size-with-min-or-max-content-3.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-3.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-3.html)
- [block-size-with-min-or-max-content-4.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-4.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-4.html)
- [block-size-with-min-or-max-content-5.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-5.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-5.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-5.html)
- [block-size-with-min-or-max-content-6.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-6.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-6.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-6.html)
- [block-size-with-min-or-max-content-7.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-7.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-7.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-7.html)
- [block-size-with-min-or-max-content-table-1a.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-table-1a.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-table-1a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-table-1a.html)
- [block-size-with-min-or-max-content-table-1b.html](https://wpt.fyi/results/css/css-sizing/block-size-with-min-or-max-content-table-1b.html) [(live test)](http://wpt.live/css/css-sizing/block-size-with-min-or-max-content-table-1b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-size-with-min-or-max-content-table-1b.html)
- [hori-block-size-small-or-larger-than-container-with-min-or-max-content-1.html](https://wpt.fyi/results/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-1.html) [(live test)](http://wpt.live/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-1.html)
- [hori-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html](https://wpt.fyi/results/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html) [(live test)](http://wpt.live/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html)
- [hori-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html](https://wpt.fyi/results/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html) [(live test)](http://wpt.live/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/hori-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html)
- [image-min-max-content-intrinsic-size-change-001.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-001.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-001.html)
- [image-min-max-content-intrinsic-size-change-002.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-002.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-002.html)
- [image-min-max-content-intrinsic-size-change-003.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-003.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-003.html)
- [image-min-max-content-intrinsic-size-change-004.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-004.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-004.html)
- [image-min-max-content-intrinsic-size-change-005.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-005.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-005.html)
- [image-min-max-content-intrinsic-size-change-006.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-006.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-006.html)
- [image-min-max-content-intrinsic-size-change-007.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-007.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-007.html)
- [image-min-max-content-intrinsic-size-change-008.html](https://wpt.fyi/results/css/css-sizing/image-min-max-content-intrinsic-size-change-008.html) [(live test)](http://wpt.live/css/css-sizing/image-min-max-content-intrinsic-size-change-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-min-max-content-intrinsic-size-change-008.html)
- [keyword-sizes-for-intrinsic-contributions.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-for-intrinsic-contributions.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-for-intrinsic-contributions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-for-intrinsic-contributions.html)
- [keyword-sizes-for-intrinsic-contributions-002.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-for-intrinsic-contributions-002.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-for-intrinsic-contributions-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-for-intrinsic-contributions-002.html)
- [keyword-sizes-on-abspos.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-abspos.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-abspos.html)
- [keyword-sizes-on-flex-item-001.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-flex-item-001.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-flex-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-flex-item-001.html)
- [keyword-sizes-on-flex-item-002.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-flex-item-002.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-flex-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-flex-item-002.html)
- [keyword-sizes-on-floated-element.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-floated-element.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-floated-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-floated-element.html)
- [keyword-sizes-on-inline-block.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-inline-block.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-inline-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-inline-block.html)
- [keyword-sizes-on-replaced-element.html](https://wpt.fyi/results/css/css-sizing/keyword-sizes-on-replaced-element.html) [(live test)](http://wpt.live/css/css-sizing/keyword-sizes-on-replaced-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/keyword-sizes-on-replaced-element.html)
- [min-content-min-width-000.html](https://wpt.fyi/results/css/css-sizing/min-content-min-width-000.html) [(live test)](http://wpt.live/css/css-sizing/min-content-min-width-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-content-min-width-000.html)
- [percentage-min-width.html](https://wpt.fyi/results/css/css-sizing/percentage-min-width.html) [(live test)](http://wpt.live/css/css-sizing/percentage-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/percentage-min-width.html)
- [vert-block-size-small-or-larger-than-container-with-min-or-max-content-1.html](https://wpt.fyi/results/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-1.html) [(live test)](http://wpt.live/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-1.html)
- [vert-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html](https://wpt.fyi/results/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html) [(live test)](http://wpt.live/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html)
- [vert-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html](https://wpt.fyi/results/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html) [(live test)](http://wpt.live/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/vert-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html)

<a id="ref-for-inner-size①"></a>

In all cases, the used value is floored to preserve a non-negative [inner size](#inner-size).

<a id="ref-for-valdef-width-stretch②"></a>

<a id="ref-for-valdef-width-min-content②"></a>

<a id="ref-for-valdef-width-max-content③"></a>

<a id="ref-for-valdef-width-fit-content①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [stretch](#valdef-width-stretch), [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content](#valdef-width-fit-content) values are new in Level 3.

<a id="ref-for-propdef-flex-basis"></a>

<a id="ref-for-propdef-width⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis) property hereby also gains these new keywords, as its values are defined by reference to [\<'width'\>](#propdef-width).

<a id="ref-for-valdef-width-auto⑤"></a>

#### <a id="behave-auto"></a>3.2.1.  “Behaving as [auto](#valdef-width-auto)”

<a id="ref-for-propdef-width①⓪"></a>

<a id="ref-for-propdef-height⑧"></a>

<a id="ref-for-valdef-width-auto⑥"></a>

<a id="ref-for-indefinite④"></a>

To have a common term for both when [width](#propdef-width)/[height](#propdef-height) computes to [auto](#valdef-width-auto) and when it is defined to behave as if <a id="ref-for-valdef-width-auto⑦"></a>auto were specified (as in the case of [block percentage heights](https://www.w3.org/TR/CSS2/visudet.html#the-height-property) resolving against an [indefinite](#indefinite) size, see [CSS2§10.5](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)), the property is said to <a id="behave-as-auto"></a>behave as auto in both of these cases.

<a id="ref-for-propdef-width①①"></a>

<a id="ref-for-propdef-height⑨"></a>

<a id="ref-for-valdef-width-auto⑧"></a>

<a id="ref-for-behave-as-auto"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Legacy spec prose defining layout behavior, particularly in [\[CSS2\]](#biblio-css2), might explicitly refer to [width](#propdef-width)/[height](#propdef-height) having a computed value of [auto](#valdef-width-auto) as a condition; some of these cases should be interpreted as meaning [behaves as auto](#behave-as-auto), and reported to the CSSWG for updating.

Tests

- [margin-collapse-with-indefinite-block-size-001.html](https://wpt.fyi/results/css/css-sizing/margin-collapse-with-indefinite-block-size-001.html) [(live test)](http://wpt.live/css/css-sizing/margin-collapse-with-indefinite-block-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/margin-collapse-with-indefinite-block-size-001.html)
- [margin-collapse-with-indefinite-block-size-002.html](https://wpt.fyi/results/css/css-sizing/margin-collapse-with-indefinite-block-size-002.html) [(live test)](http://wpt.live/css/css-sizing/margin-collapse-with-indefinite-block-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/margin-collapse-with-indefinite-block-size-002.html)
- [margin-collapse-with-indefinite-block-size-003.html](https://wpt.fyi/results/css/css-sizing/margin-collapse-with-indefinite-block-size-003.html) [(live test)](http://wpt.live/css/css-sizing/margin-collapse-with-indefinite-block-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/margin-collapse-with-indefinite-block-size-003.html)
- [margin-collapse-with-indefinite-block-size-004.html](https://wpt.fyi/results/css/css-sizing/margin-collapse-with-indefinite-block-size-004.html) [(live test)](http://wpt.live/css/css-sizing/margin-collapse-with-indefinite-block-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/margin-collapse-with-indefinite-block-size-004.html)
- [margin-collapse-with-indefinite-block-size-005.html](https://wpt.fyi/results/css/css-sizing/margin-collapse-with-indefinite-block-size-005.html) [(live test)](http://wpt.live/css/css-sizing/margin-collapse-with-indefinite-block-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/margin-collapse-with-indefinite-block-size-005.html)

<a id="ref-for-automatic-size②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6215c343"></a> Replace this section with references to the new term [automatic size](#automatic-size).

#### <a id="the-contain-floats-value"></a>3.2.2.  Containing or Excluding Floats

<em>This section is non-normative.</em>

<a id="ref-for-block-box"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-formatting-context"></a>

Although [block box](https://www.w3.org/TR/css-display-4/#block-box) boundaries are typically pervious to floats, sometimes an author needs them to contain their own (descendant) floats or to exclude floats from outside. For Block layout, specifying [display: flow-root](https://www.w3.org/TR/css-display-3/#propdef-display) will make the box a [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context) root, which has this behavior.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Boxes participating in Flex, Grid, or Table layout will automatically have this behavior.

<a id="ref-for-propdef-box-sizing②"></a>

### <a id="box-sizing"></a>3.3.  Box Edges for Sizing: the [box-sizing](#propdef-box-sizing) property

| Field               | Definition                                                                                                          |
|---------------------|---------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-sizing"></a>box-sizing                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑦"></a>content-box [\|](https://www.w3.org/TR/css-values-4/#comb-one) border-box                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | content-box                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-height①⓪"></a><a id="ref-for-propdef-width①②"></a>all elements that accept [width](#propdef-width) or [height](#propdef-height) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                            |

Tests

- [box-sizing-replaced-001.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-replaced-001.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-replaced-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-replaced-001.xht)
- [box-sizing-replaced-002.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-replaced-002.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-replaced-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-replaced-002.xht)
- [box-sizing-replaced-003.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-replaced-003.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-replaced-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-replaced-003.xht)
- [box-sizing-computed.html](https://wpt.fyi/results/css/css-sizing/parsing/box-sizing-computed.html) [(live test)](http://wpt.live/css/css-sizing/parsing/box-sizing-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/box-sizing-computed.html)
- [box-sizing-invalid.html](https://wpt.fyi/results/css/css-sizing/parsing/box-sizing-invalid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/box-sizing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/box-sizing-invalid.html)
- [box-sizing-valid.html](https://wpt.fyi/results/css/css-sizing/parsing/box-sizing-valid.html) [(live test)](http://wpt.live/css/css-sizing/parsing/box-sizing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/parsing/box-sizing-valid.html)

<a id="ref-for-propdef-box-sizing③"></a>

<a id="ref-for-length-value②"></a>

<a id="ref-for-percentage-value②"></a>

<a id="ref-for-content-box②"></a>

<a id="ref-for-border-box①"></a>

<a id="ref-for-sizing-property④"></a>

<a id="ref-for-propdef-flex-basis①"></a>

The [box-sizing](#propdef-box-sizing) property defines whether fixed sizes (such as [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s and [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s) are assigned to the [content box](https://www.w3.org/TR/css-box-4/#content-box) or to the [border box](https://www.w3.org/TR/css-box-4/#border-box). It affects the interpretation of all [sizing properties](#sizing-property), including [flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis).

Values have the following meanings:

<a id="valdef-box-sizing-content-box"></a>content-box  
<a id="ref-for-propdef-height①①"></a>

<a id="ref-for-propdef-width①③"></a>

<a id="ref-for-content-box③"></a>

<a id="ref-for-inner-size②"></a>

<a id="ref-for-typedef-length-percentage⑦"></a>

<a id="ref-for-sizing-property⑤"></a>

Sizes specified on [sizing properties](#sizing-property) as [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) represent the box’s [inner sizes](#inner-size), excluding the margins/border/padding: they are applied to the [content box](https://www.w3.org/TR/css-box-4/#content-box). The padding and border of the box are laid out and drawn <em>outside</em> the specified [width](#propdef-width) and [height](#propdef-height).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the behavior of width and height as specified by CSS2.1, and is thus the default.

Tests

- [box-sizing-content-box-001.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-content-box-001.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-content-box-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-content-box-001.xht)
- [box-sizing-content-box-002.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-content-box-002.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-content-box-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-content-box-002.xht)
- [box-sizing-content-box-003.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-content-box-003.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-content-box-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-content-box-003.xht)

<a id="valdef-box-sizing-border-box"></a>border-box  
<a id="ref-for-inner-size③"></a>

<a id="ref-for-height②"></a>

<a id="ref-for-width③"></a>

<a id="ref-for-content-box④"></a>

<a id="ref-for-propdef-height①②"></a>

<a id="ref-for-propdef-width①④"></a>

<a id="ref-for-margin"></a>

<a id="ref-for-border"></a>

<a id="ref-for-padding"></a>

<a id="ref-for-border-box②"></a>

<a id="ref-for-sizing-property⑥"></a>

<a id="ref-for-typedef-length-percentage⑧"></a>

Any [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values in the [sizing properties](#sizing-property) are applied to the [border box](https://www.w3.org/TR/css-box-4/#border-box), thus representing the box’s visually-apparent sizes: the [padding](https://www.w3.org/TR/css-box-4/#padding) and [border](https://www.w3.org/TR/css-box-4/#border) of the box (but not its [margins](https://www.w3.org/TR/css-box-4/#margin)) are essentially laid out and drawn <em>inside</em> the used [width](#propdef-width) and [height](#propdef-height), with the [content box](https://www.w3.org/TR/css-box-4/#content-box) sized to fill the remaining space. More specifically, the <a id="ref-for-content-box⑤"></a>content box [width](#width) and [height](#height) are calculated by subtracting the <a id="ref-for-border①"></a>border and <a id="ref-for-padding①"></a>padding in the corresponding axis from the specified <a id="ref-for-typedef-length-percentage⑨"></a>\<length-percentage\>, and flooring the result at zero (as the [inner size](#inner-size) of a box cannot be negative).

<a id="ref-for-sizing-property⑦"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-border-box③"></a>

Used values of the [sizing properties](#sizing-property), as exposed for instance through <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>, also refer to the [border box](https://www.w3.org/TR/css-box-4/#border-box).

Tests

- [border-box-and-max-content-001.html](https://wpt.fyi/results/css/css-sizing/border-box-and-max-content-001.html) [(live test)](http://wpt.live/css/css-sizing/border-box-and-max-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/border-box-and-max-content-001.html)
- [border-box-and-max-content-002.html](https://wpt.fyi/results/css/css-sizing/border-box-and-max-content-002.html) [(live test)](http://wpt.live/css/css-sizing/border-box-and-max-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/border-box-and-max-content-002.html)
- [border-box-and-max-content-003.html](https://wpt.fyi/results/css/css-sizing/border-box-and-max-content-003.html) [(live test)](http://wpt.live/css/css-sizing/border-box-and-max-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/border-box-and-max-content-003.html)
- [box-sizing-border-box-001.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-border-box-001.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-border-box-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-border-box-001.xht)
- [box-sizing-border-box-002.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-border-box-002.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-border-box-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-border-box-002.xht)
- [box-sizing-border-box-003.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-border-box-003.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-border-box-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-border-box-003.xht)
- [box-sizing-border-box-004.xht](https://wpt.fyi/results/css/css-sizing/box-sizing-border-box-004.xht) [(live test)](http://wpt.live/css/css-sizing/box-sizing-border-box-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/box-sizing-border-box-004.xht)
- [table-child-percentage-height-with-border-box.html](https://wpt.fyi/results/css/css-sizing/table-child-percentage-height-with-border-box.html) [(live test)](http://wpt.live/css/css-sizing/table-child-percentage-height-with-border-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/table-child-percentage-height-with-border-box.html)

<a id="ref-for-propdef-box-sizing④"></a>

<a id="ref-for-typedef-length-percentage①⓪"></a>

<a id="ref-for-funcdef-width-fit-content③"></a>

<a id="ref-for-valdef-width-auto⑨"></a>

<a id="ref-for-valdef-width-min-content③"></a>

Values affected by [box-sizing](#propdef-box-sizing) include both raw [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values and those used in functional notations such as [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content) [\[css-sizing-4\]](#biblio-css-sizing-4). In contrast, non-quantitative values such as [auto](#valdef-width-auto) and [min-content](#valdef-width-min-content) are not influenced by the <a id="ref-for-propdef-box-sizing⑤"></a>box-sizing property (unless otherwise specified).

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
> <a id="ref-for-inner-size④"></a>
>
> <a id="ref-for-propdef-padding"></a>
>
> <a id="ref-for-propdef-border"></a>
>
> <a id="ref-for-propdef-width①⑤"></a>
>
> The [inner size](#inner-size) can’t be less than zero, so if the [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) + [border](https://www.w3.org/TR/css-borders-4/#propdef-border) is greater than the specified border-box size, the box will end up larger than specified. In this case, the content-box size will floor at 0px so the border-box size ends up at 120px, even though [width: 100px](#propdef-width) is specified for the border box:
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

<a id="ref-for-width④"></a>

<a id="ref-for-height③"></a>

<a id="ref-for-min-width④"></a>

<a id="ref-for-min-height"></a>

<a id="ref-for-max-width③"></a>

<a id="ref-for-max-height"></a>

<a id="ref-for-inner-size⑤"></a>

<a id="ref-for-content-box⑥"></a>

<a id="ref-for-box②"></a>

In legacy CSS specifications, the terms [width](#width), [height](#height), [minimum (min) width](#min-width), [minimum (min) height](#min-height), [maximum (max) width](#max-width), and [maximum (max) height](#max-height) generally refer to the [inner](#inner-size) size ([content-box](https://www.w3.org/TR/css-box-4/#content-box) size) of a [box](https://www.w3.org/TR/css-display-3/#box) unless otherwise indicated.

Refer to [CSS User Interface 3 § 3.1 Changing the Box Model: the box-sizing property](https://www.w3.org/TR/css-ui-3/#box-sizing) for an explicit disambiguation of these terms for the [Visual formatting model details](https://www.w3.org/TR/CSS21/visudet.html) section of [\[CSS2\]](#biblio-css2).

<a id="ref-for-inner-size⑥"></a>

<a id="ref-for-outer-size②"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-sizing-property⑧"></a>

> <strong data-conversion-semantic="advisement">Advisement</strong>
>
> To avoid ambiguities, specification authors should avoid ambiguous uses of terms such as width or height without further qualification, and should explicitly refer and link to the [inner](#inner-size) size, the [outer](#outer-size) size, the size of the [border-box](https://www.w3.org/TR/css2/box.html#box-dimensions), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [sizing properties](#sizing-property), etc, as appropriate for each case.

<a id="ref-for-valdef-column-width-stretch"></a>

<a id="ref-for-valdef-column-width-min-content"></a>

<a id="ref-for-valdef-column-width-max-content"></a>

<a id="ref-for-valdef-column-width-fit-content"></a>

### <a id="column-sizing"></a>3.4.  New Column Sizing Values: the [stretch](#valdef-column-width-stretch), [min-content](#valdef-column-width-min-content), [max-content](#valdef-column-width-max-content), and [fit-content](#valdef-column-width-fit-content) values

| Field               | Definition                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-column-width②"></a>[column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width)                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-typedef-box-size⑥"></a>[\<box-size\>](#typedef-box-size)                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①①"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                       |

<a id="ref-for-propdef-column-width③"></a>

When used as values for [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width), the new keywords specify the optimal column width:

<a id="valdef-column-width-stretch"></a>stretch  
<a id="ref-for-stretch-fit-inline-size"></a>

Specifies the optimal column width as the [stretch-fit inline size](#stretch-fit-inline-size) of the multi-column container.

<a id="valdef-column-width-min-content"></a>min-content  
<a id="ref-for-min-content-inline-size"></a>

Specifies the optimal column width as the [min-content inline size](#min-content-inline-size) of the multi-column container’s contents.

<a id="valdef-column-width-max-content"></a>max-content  
<a id="ref-for-max-content-inline-size"></a>

Specifies the optimal column width as the [max-content inline size](#max-content-inline-size) of the multi-column container’s contents.

<a id="valdef-column-width-fit-content"></a>fit-content  
<a id="ref-for-stretch-fit-inline-size①"></a>

<a id="ref-for-min-content-inline-size①"></a>

<a id="ref-for-max-content-inline-size①"></a>

Specifies the optimal column width as <code>min(<a href="#max-content-inline-size">max-content inline size</a>, max(<a href="#min-content-inline-size">min-content inline size</a>, <a href="#stretch-fit-inline-size">stretch-fit inline size</a>))</code>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The column width never varies by column. When the column width is informed by the multi-column container’s contents (as in the keywords above), all of its contents are taken under consideration and the calculated width is shared by all the columns.

## <a id="extrinsic"></a>4.  Extrinsic Size Determination

<a id="extrinsic-sizing"></a>Extrinsic sizing determines sizes based on the context of an element, without regard for its contents.

### <a id="percentage-sizing"></a>4.1.  Percentage Sizing

<a id="ref-for-containing-block①③"></a>

Percentages specify sizing of a box with respect to the box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block).

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

<a id="ref-for-containing-block①④"></a>

See [§ 5.2.1 Intrinsic Contributions of Percentage-Sized Boxes](#cyclic-percentage-contribution) for details on how to resolve percentages when the size of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) depends on the size of its content.

Tests

- [percentage-height-in-flexbox.html](https://wpt.fyi/results/css/css-sizing/percentage-height-in-flexbox.html) [(live test)](http://wpt.live/css/css-sizing/percentage-height-in-flexbox.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/percentage-height-in-flexbox.html)
- [range-percent-intrinsic-size-1.html](https://wpt.fyi/results/css/css-sizing/range-percent-intrinsic-size-1.html) [(live test)](http://wpt.live/css/css-sizing/range-percent-intrinsic-size-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/range-percent-intrinsic-size-1.html)
- [range-percent-intrinsic-size-2.html](https://wpt.fyi/results/css/css-sizing/range-percent-intrinsic-size-2.html) [(live test)](http://wpt.live/css/css-sizing/range-percent-intrinsic-size-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/range-percent-intrinsic-size-2.html)
- [range-percent-intrinsic-size-2a.html](https://wpt.fyi/results/css/css-sizing/range-percent-intrinsic-size-2a.html) [(live test)](http://wpt.live/css/css-sizing/range-percent-intrinsic-size-2a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/range-percent-intrinsic-size-2a.html)

### <a id="stretch-fit-sizing"></a>4.2.  Stretch-fit Sizing: filling the containing block

<a id="ref-for-principal-box"></a>

<a id="ref-for-outer-size③"></a>

<a id="ref-for-containing-block①⑤"></a>

<a id="ref-for-propdef-min-height④"></a>

<a id="ref-for-propdef-min-width⑤"></a>

<a id="ref-for-propdef-max-height③"></a>

<a id="ref-for-propdef-max-width④"></a>

Stretch-fit sizing tries to set the box’s used size to the length necessary to make its [principal box](https://www.w3.org/TR/css-display-4/#principal-box)’s [outer size](#outer-size) as close to filling the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) as possible while still respecting the constraints imposed by [min-height](#propdef-min-height)/[min-width](#propdef-min-width)/[max-height](#propdef-max-height)/[max-width](#propdef-max-width).

<a id="ref-for-definite⑨"></a>

If used in an axis where percentage sizes can resolve to a [definite](#definite) value

<a id="ref-for-margin-box①"></a>

<a id="ref-for-containing-block①⑥"></a>

<a id="ref-for-inner-size⑦"></a>

Sizes the [margin box](https://www.w3.org/TR/css-box-4/#margin-box) to fill the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) exactly, treating auto margins as zero. (If this would make the [inner size](#inner-size) negative, it instead sizes the <a id="ref-for-margin-box②"></a>margin box so that the <a id="ref-for-inner-size⑧"></a>inner size is zero.)

<a id="ref-for-in-flow"></a>

<a id="ref-for-block-level-box"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-line-box"></a>

<a id="ref-for-containing-block①⑦"></a>

<a id="ref-for-inline-size⑤"></a>

For [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) [block-level boxes](https://www.w3.org/TR/css-display-4/#block-level-box) that form an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context), use the space available to [line boxes](https://www.w3.org/TR/css-inline-3/#line-box) (i.e. excluding floats) in place of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s [inline size](#inline-size).

<a id="ref-for-in-flow①"></a>

<a id="ref-for-block-level-box①"></a>

<a id="ref-for-block-axis②"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-propdef-border①"></a>

<a id="ref-for-propdef-padding①"></a>

<a id="ref-for-independent-formatting-context①"></a>

<a id="ref-for-block-end"></a>

For [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) [block-level boxes](https://www.w3.org/TR/css-display-4/#block-level-box) when resolving the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) size: if the ancestor element percentages resolve against does not have a [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) [border](https://www.w3.org/TR/css-borders-4/#propdef-border) or [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) and is not an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context), treat the element’s <a id="ref-for-block-start①"></a>block-start margin as zero for the purpose of calculating this size. Do the same for the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) margin, analogously.

<a id="ref-for-stretch-fit-size④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This simulates the effect of margins collapsing with the parent’s margin. It doesn’t actually suppress the margins, so if anything prevents the element from actually collapsing with its parent, the [stretch-fit size](#stretch-fit-size) might actually be too large to fit in the parent perfectly.

Otherwise

<a id="ref-for-preferred-size-properties⑤"></a>

<a id="ref-for-behave-as-auto①"></a>

<a id="ref-for-min-size-properties④"></a>

<a id="ref-for-max-size-properties⑤"></a>

<a id="ref-for-valdef-max-width-none②"></a>

In a [preferred size property](#preferred-size-properties), [behaves as auto](#behave-as-auto). In a [min size property](#min-size-properties), behaves as 0. In a [max size property](#max-size-properties), behaves as [none](#valdef-max-width-none).

<a id="ref-for-block-box①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2ccaaa06"></a> For example, given the following HTML representing two [block boxes](https://www.w3.org/TR/css-display-4/#block-box):
>
> ```text
> <div class="parent">
>   <div class="child">text</div>
> </div>
> ```
>
> <a id="ref-for-outer-size④"></a>
>
> <a id="ref-for-inner-size⑨"></a>
>
> In the following case, the [outer height](#outer-size) of the child box will exactly match the height of the parent box (200px), but its [inner height](#inner-size) will be 20px less, to account for its margins.
>
> ```text
> .parent { height: 200px; border: solid; }
> .child { height: stretch; margin: 10px; }
> ```
>
> On the other hand, in this case we can assume that the child’s margins will collapse with the parent, so the inner box will be 200px tall, exactly filling the parent.
>
> ```text
> .outer { height: 200px; margin: 0; }
> .inner { height: stretch; margin: 10px; }
> ```
>
> <a id="ref-for-valdef-width-auto①⓪"></a>
>
> (The top margins will in fact collapse, but the bottom margins do not collapse, because the bottom margin of a box is not adjoining to the bottom margin of a parent with a non-[auto](#valdef-width-auto) height, see [CSS 2 § 8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins). Luckily, an overflowing bottom margin doesn’t have any visible effect.)

<a id="ref-for-propdef-width①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-81a6389a"></a> Similarly, [width: stretch](#propdef-width) causes the box to fill its container, being 20px narrower than the width of "some more text" (due to the 10px margin):
>
> ```text
> <div class="parent">
>   <div class="child">text</div>
> </div>
> some more text
> ```
>
> ```text
> .parent { float: left; margin: 0; }
> .child { width: stretch; margin: 10px; }
> ```
<a id="ref-for-behave-as-auto②"></a>

<a id="ref-for-propdef-height①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a3676cf7"></a> On the other hand, in this example the container’s height is indefinite, which would cause a percentage height on the child to [behave as auto](#behave-as-auto), so [height: stretch](#propdef-height) <a id="ref-for-behave-as-auto③"></a>behaves as auto as well.
>
> ```text
> .parent { height: auto; margin: 0; }
> .child { height: stretch; margin: 10px; }
> ```
Tests

- [abspos-1.html](https://wpt.fyi/results/css/css-sizing/stretch/abspos-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/abspos-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/abspos-1.html)
- [abspos-2.html](https://wpt.fyi/results/css/css-sizing/stretch/abspos-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/abspos-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/abspos-2.html)
- [aspect-ratio-1.html](https://wpt.fyi/results/css/css-sizing/stretch/aspect-ratio-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/aspect-ratio-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/aspect-ratio-1.html)
- [aspect-ratio-2.html](https://wpt.fyi/results/css/css-sizing/stretch/aspect-ratio-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/aspect-ratio-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/aspect-ratio-2.html)
- [auto-margins-1.html](https://wpt.fyi/results/css/css-sizing/stretch/auto-margins-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/auto-margins-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/auto-margins-1.html)
- [auto-margins-2.html](https://wpt.fyi/results/css/css-sizing/stretch/auto-margins-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/auto-margins-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/auto-margins-2.html)
- [bfc-next-to-float-1.html](https://wpt.fyi/results/css/css-sizing/stretch/bfc-next-to-float-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/bfc-next-to-float-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/bfc-next-to-float-1.html)
- [bfc-next-to-float-2.html](https://wpt.fyi/results/css/css-sizing/stretch/bfc-next-to-float-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/bfc-next-to-float-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/bfc-next-to-float-2.html)
- [block-height-002.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-002.html)
- [block-height-003.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-003.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-003.html)
- [block-height-004.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-004.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-004.html)
- [block-height-005.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-005.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-005.html)
- [block-height-006.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-006.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-006.html)
- [block-height-007.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-007.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-007.html)
- [block-height-008.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-008.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-008.html)
- [block-height-009.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-009.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-009.html)
- [block-height-010.html](https://wpt.fyi/results/css/css-sizing/stretch/block-height-010.html) [(live test)](http://wpt.live/css/css-sizing/stretch/block-height-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/block-height-010.html)
- [cache-miss-001.html](https://wpt.fyi/results/css/css-sizing/stretch/cache-miss-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/cache-miss-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/cache-miss-001.html)
- [cache-miss-002.html](https://wpt.fyi/results/css/css-sizing/stretch/cache-miss-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/cache-miss-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/cache-miss-002.html)
- [content-contribution-001.html](https://wpt.fyi/results/css/css-sizing/stretch/content-contribution-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/content-contribution-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/content-contribution-001.html)
- [fixed-table-1.html](https://wpt.fyi/results/css/css-sizing/stretch/fixed-table-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/fixed-table-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/fixed-table-1.html)
- [flex-basis-1.html](https://wpt.fyi/results/css/css-sizing/stretch/flex-basis-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flex-basis-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flex-basis-1.html)
- [flexbox-auto-minimum-001.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-auto-minimum-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-auto-minimum-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-auto-minimum-001.html)
- [flexbox-auto-minimum-002.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-auto-minimum-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-auto-minimum-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-auto-minimum-002.html)
- [flexbox-flex-base-size-001.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-flex-base-size-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-flex-base-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-flex-base-size-001.html)
- [flexbox-flex-base-size-002.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-flex-base-size-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-flex-base-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-flex-base-size-002.html)
- [flexbox-stretch-minimum-001.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-stretch-minimum-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-stretch-minimum-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-stretch-minimum-001.html)
- [flexbox-stretch-minimum-002.html](https://wpt.fyi/results/css/css-sizing/stretch/flexbox-stretch-minimum-002.html) [(live test)](http://wpt.live/css/css-sizing/stretch/flexbox-stretch-minimum-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/flexbox-stretch-minimum-002.html)
- [indefinite-1.html](https://wpt.fyi/results/css/css-sizing/stretch/indefinite-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/indefinite-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/indefinite-1.html)
- [indefinite-2.html](https://wpt.fyi/results/css/css-sizing/stretch/indefinite-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/indefinite-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/indefinite-2.html)
- [indefinite-3.html](https://wpt.fyi/results/css/css-sizing/stretch/indefinite-3.html) [(live test)](http://wpt.live/css/css-sizing/stretch/indefinite-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/indefinite-3.html)
- [indefinite-4.html](https://wpt.fyi/results/css/css-sizing/stretch/indefinite-4.html) [(live test)](http://wpt.live/css/css-sizing/stretch/indefinite-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/indefinite-4.html)
- [replaced-next-to-float-1.html](https://wpt.fyi/results/css/css-sizing/stretch/replaced-next-to-float-1.html) [(live test)](http://wpt.live/css/css-sizing/stretch/replaced-next-to-float-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/replaced-next-to-float-1.html)
- [replaced-next-to-float-2.html](https://wpt.fyi/results/css/css-sizing/stretch/replaced-next-to-float-2.html) [(live test)](http://wpt.live/css/css-sizing/stretch/replaced-next-to-float-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/replaced-next-to-float-2.html)
- [stretch-table-001.html](https://wpt.fyi/results/css/css-sizing/stretch/stretch-table-001.html) [(live test)](http://wpt.live/css/css-sizing/stretch/stretch-table-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/stretch/stretch-table-001.html)

<a id="ref-for-stretch-fit-size⑤"></a>

<a id="ref-for-valdef-align-self-stretch"></a>

<a id="ref-for-self-align"></a>

<a id="ref-for-multi-line-flex-container"></a>

<a id="ref-for-flex-layout"></a>

<a id="ref-for-containing-block①⑧"></a>

<a id="ref-for-cross-size"></a>

<a id="ref-for-fit-content-size②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [stretch-fit size](#stretch-fit-size) is usually, but not always, equivalent to [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) [self-alignment](https://www.w3.org/TR/css-align-3/#self-align). For example, in [multi-line](https://www.w3.org/TR/css-flexbox-1/#multi-line-flex-container) [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout), the <a id="ref-for-stretch-fit-size⑥"></a>stretch-fit size resolves against the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) directly, and contributes that size to the [cross size](https://www.w3.org/TR/css-flexbox-1/#cross-size) of the line—​whereas <a id="ref-for-valdef-align-self-stretch①"></a>stretch contributes the [fit-content size](#fit-content-size)—​before finally resolving against the <a id="ref-for-cross-size①"></a>cross size of the line.

## <a id="intrinsic"></a>5.  Intrinsic Size Determination

<a id="intrinsic-sizing"></a>Intrinsic sizing determines sizes based on the contents of an element, without regard for its context.

Tests

- [canvas-intrinsic-dynamic.html](https://wpt.fyi/results/css/css-sizing/canvas-intrinsic-dynamic.html) [(live test)](http://wpt.live/css/css-sizing/canvas-intrinsic-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/canvas-intrinsic-dynamic.html)
- [intrinsic-percent-replaced-dynamic-001.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-001.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-001.html)
- [intrinsic-percent-replaced-dynamic-002.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-002.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-002.html)
- [intrinsic-percent-replaced-dynamic-003.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-003.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-003.html)
- [intrinsic-percent-replaced-dynamic-004.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-004.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-004.html)
- [intrinsic-percent-replaced-dynamic-005.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-005.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-005.html)
- [intrinsic-percent-replaced-dynamic-006.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-006.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-006.html)
- [intrinsic-percent-replaced-dynamic-007.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-007.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-007.html)
- [intrinsic-percent-replaced-dynamic-008.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-008.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-008.html)
- [intrinsic-percent-replaced-dynamic-009.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-009.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-009.html)
- [intrinsic-percent-replaced-dynamic-010.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-010.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-010.html)
- [intrinsic-percent-replaced-dynamic-011.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-011.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-011.html)
- [intrinsic-percent-replaced-dynamic-012.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-dynamic-012.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-dynamic-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-dynamic-012.html)
- [intrinsic-ratio-replaced-box-sizing.html](https://wpt.fyi/results/css/css-sizing/intrinsic-ratio-replaced-box-sizing.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-ratio-replaced-box-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-ratio-replaced-box-sizing.html)
- [intrinsic-size-fallback-replaced.html](https://wpt.fyi/results/css/css-sizing/intrinsic-size-fallback-replaced.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-size-fallback-replaced.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-size-fallback-replaced.html)

### <a id="intrinsic-sizes"></a>5.1.  Intrinsic Sizes

<a id="ref-for-min-content①①"></a>

<a id="ref-for-valdef-width-auto①①"></a>

<a id="ref-for-preferred-size①"></a>

<a id="ref-for-min-width⑤"></a>

<a id="ref-for-max-width④"></a>

The [min-content size](#min-content) of a box in each axis is the size it would have if it was a float given an [auto](#valdef-width-auto) [preferred size](#preferred-size) in that axis (and no [minimum](#min-width) or [maximum size](#max-width) in that axis) and if its containing block was <em>zero</em>-sized in that axis. (In other words, the minimum size it has when sized as “shrink-to-fit”.)

<a id="ref-for-max-content⑨"></a>

<a id="ref-for-valdef-width-auto①②"></a>

<a id="ref-for-preferred-size②"></a>

<a id="ref-for-min-width⑥"></a>

<a id="ref-for-max-width⑤"></a>

The [max-content size](#max-content) of a box in each axis is the size it would have if it was a float given an [auto](#valdef-width-auto) [preferred size](#preferred-size) in that axis (and no [minimum](#min-width) or [maximum size](#max-width) in that axis), and if its containing block was <em>infinitely</em>-sized in that axis. (In other words, the maximum size it has when sized as “shrink-to-fit”.)

<a id="ref-for-min-content①②"></a>

<a id="ref-for-max-content①⓪"></a>

<a id="ref-for-intrinsic-size①"></a>

The [min-content size](#min-content) and [max-content size](#max-content) are collectively referred to as the [intrinsic sizes](#intrinsic-size).

<a id="ref-for-preferred-aspect-ratio②"></a>

<a id="ref-for-valdef-width-auto①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When the box has a [preferred aspect ratio](#preferred-aspect-ratio), size constraints in the opposite dimension will transfer through and can affect the [auto](#valdef-width-auto) size in the considered one. See [CSS2§10](https://www.w3.org/TR/CSS2/visudet.html).

<a id="ref-for-intrinsic-size②"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-natural-size"></a>

This specification does not define how to determine the sizes of floats. Please refer to [\[CSS2\]](#biblio-css2). However, the [intrinsic sizes](#intrinsic-size) of [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) without [natural sizes](https://www.w3.org/TR/css-images-3/#natural-size) are defined below:

<a id="ref-for-preferred-aspect-ratio③"></a>

If it has a non-degenerate [preferred aspect ratio](#preferred-aspect-ratio):

<a id="ref-for-min-content①③"></a>

For the [min-content size](#min-content), use zero.

<a id="ref-for-max-content①①"></a>

For the [max-content size](#max-content):

- <a id="ref-for-available⑦"></a>

  <a id="ref-for-definite①⓪"></a>

  <a id="ref-for-inline-axis③"></a>

  <a id="ref-for-stretch-fit②"></a>

  If the [available space](#available) is [definite](#definite) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), use the [stretch fit](#stretch-fit) into that size for the inline size and calculate the block size using the aspect ratio.

- <a id="ref-for-length-value③"></a>

  <a id="ref-for-computed-value①"></a>

  <a id="ref-for-propdef-min-width⑥"></a>

  <a id="ref-for-propdef-min-height⑤"></a>

  Otherwise if the box has a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) as its [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) for [min-width](#propdef-min-width) or [min-height](#propdef-min-height), use that size and calculate the other dimension using the aspect ratio; if both dimensions have a <a id="ref-for-length-value④"></a>\<length\> minimum, choose the one that results in the larger overall size.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This case was previous calculated from a 300x150 default size, rather than the box’s min size. This is believed to be a better behavior, and likely to be Web-compatible, but please send feedback to the CSSWG if there are any problems.

  Tests
  - [replaced-fractional-height-from-aspect-ratio-2.html](https://wpt.fyi/results/css/css-sizing/replaced-fractional-height-from-aspect-ratio-2.html) [(live test)](http://wpt.live/css/css-sizing/replaced-fractional-height-from-aspect-ratio-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-fractional-height-from-aspect-ratio-2.html)

- <a id="ref-for-inline-size⑥"></a>

  <a id="ref-for-initial-containing-block②"></a>

  Otherwise use an [inline size](#inline-size) matching the corresponding dimension of the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) and calculate the other dimension using the aspect ratio.

<a id="ref-for-preferred-aspect-ratio④"></a>

If it has no [preferred aspect ratio](#preferred-aspect-ratio):

<a id="ref-for-max-content①②"></a>

<a id="ref-for-min-content①④"></a>

For both the [min-content size](#min-content) and [max-content size](#max-content):

- <a id="ref-for-length-value⑤"></a>

  <a id="ref-for-computed-value②"></a>

  <a id="ref-for-min-width⑦"></a>

  <a id="ref-for-propdef-min-width⑦"></a>

  <a id="ref-for-propdef-min-height⑥"></a>

  If the box has a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) as its [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [minimum size](#min-width) ([min-width](#propdef-min-width)/[min-height](#propdef-min-height)) in that dimension, use that size.

  <a id="ref-for-valdef-width-auto①④"></a>

  <a id="ref-for-min-size-properties①"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This author-controllable behavior is made possible by the new [auto](#valdef-width-auto) value for the [min size properties](#min-size-properties). This is believed to be a better behavior, but it is not yet clear if it is Web-compatible, so please send feedback to the CSSWG if there are any problems.

- Otherwise, use 300px for the width and/or 150px for the height as needed.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This does not imply an aspect ratio.

<a id="ref-for-propdef-height①④"></a>

<a id="ref-for-propdef-width①⑦"></a>

<a id="ref-for-behave-as-auto④"></a>

<a id="ref-for-max-content①③"></a>

Since a block-level or inline-level replaced element whose [height](#propdef-height) or [width](#propdef-width) [behaves as auto](#behave-as-auto) is effectively defined to use its [max-content size](#max-content) ([CSS2§10.3.2](https://www.w3.org/TR/CSS2/visudet.html#inline-replaced-width)), this specification applies the rules above to the undefined case of a replaced element whose <a id="ref-for-propdef-height①⑤"></a>height and <a id="ref-for-propdef-width①⑧"></a>width both <a id="ref-for-behave-as-auto⑤"></a>behave as auto.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define how to determine the size of a float. Please refer to [\[CSS2\]](#biblio-css2), the relevant CSS specification for that display type, and/or existing implementations for further details. A future specification will define this in detail, replacing the CSS2 “definition”, such as it is.

------------------------------------------------------------------------

<a id="ref-for-valdef-width-auto①⑤"></a>

<a id="ref-for-intrinsic-size③"></a>

<a id="ref-for-valdef-width-min-content④"></a>

<a id="ref-for-valdef-width-max-content④"></a>

<a id="ref-for-sizing-property⑨"></a>

Although the [auto](#valdef-width-auto) size of text input controls such as HTML’s `<input type=text>` and `<textarea>` elements is typically a fixed size, the contents of such elements can be used to determine a content-based [intrinsic size](#intrinsic-size), as for non-replaced block containers. The [min-content](#valdef-width-min-content) and [max-content](#valdef-width-max-content) keywords of the [sizing properties](#sizing-property) thus represent content-based sizes for form controls which render their value as text contained within their box, allowing such controls to size to fit their visible contents similarly to regular non-replaced elements.

<a id="ref-for-concept-textarea-raw-value"></a>

<a id="ref-for-the-textarea-element"></a>

<a id="ref-for-the-input-element"></a>

<a id="ref-for-css-text-sequence"></a>

<a id="ref-for-soft-wrap-opportunity"></a>

<a id="ref-for-intrinsic-size④"></a>

The content in this case is defined to be the input control’s values (the [raw value](https://html.spec.whatwg.org/multipage/form-elements.html#concept-textarea-raw-value) in the case of <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element">textarea</a></code>, or the [value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value) in the case of <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code>), possibly transformed to a more human-readable and/or localized display format, which is then treated as child [text sequences](https://www.w3.org/TR/css-display-4/#css-text-sequence) of the input control, allowing [soft wrap opportunities](https://www.w3.org/TR/css-text-4/#soft-wrap-opportunity) only where the input control would actually allow wrapping (whether keyed off of CSS properties or other, UA-internal constraints). If the input control has designated placeholder text to be overlaid in its value display area, then that text is also measured for the purpose of calculating the content-based size—​whether or not the placeholder text is visible at the moment. (Thus the content-based [intrinsic size](#intrinsic-size) of the input control is the larger of the size to fit the placeholder text and the size to fit the value.)

<a id="ref-for-min-content①⑤"></a>

<a id="ref-for-max-content①④"></a>

The UA may enforce a minimum (such as the size required to contain a single zero-width character, or the smallest usable size of a touch target) on the form control’s [min-content](#min-content) and [max-content sizes](#max-content) to ensure sufficient space for the caret and otherwise maintain usability of the form control.

<a id="ref-for-the-iframe-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This might be extended to <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> or other content-containing replaced elements (see [discussion](https://github.com/w3c/csswg-drafts/issues/1771)), but text inputs are a major use-case; and being document-internal, have the least additional complications.

Tests

- [aspect-ratio-affects-container-width-when-height-changes.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio-affects-container-width-when-height-changes.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio-affects-container-width-when-height-changes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio-affects-container-width-when-height-changes.html)
- [calc-margins-block.html](https://wpt.fyi/results/css/css-sizing/calc-margins-block.html) [(live test)](http://wpt.live/css/css-sizing/calc-margins-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/calc-margins-block.html)
- [calc-margins-fieldset-content.html](https://wpt.fyi/results/css/css-sizing/calc-margins-fieldset-content.html) [(live test)](http://wpt.live/css/css-sizing/calc-margins-fieldset-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/calc-margins-fieldset-content.html)
- [calc-margins-fieldset-legend.html](https://wpt.fyi/results/css/css-sizing/calc-margins-fieldset-legend.html) [(live test)](http://wpt.live/css/css-sizing/calc-margins-fieldset-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/calc-margins-fieldset-legend.html)
- [calc-margins-flex.html](https://wpt.fyi/results/css/css-sizing/calc-margins-flex.html) [(live test)](http://wpt.live/css/css-sizing/calc-margins-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/calc-margins-flex.html)
- [calc-margins-table-caption.html](https://wpt.fyi/results/css/css-sizing/calc-margins-table-caption.html) [(live test)](http://wpt.live/css/css-sizing/calc-margins-table-caption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/calc-margins-table-caption.html)
- [image-fractional-height-with-wide-aspect-ratio.html](https://wpt.fyi/results/css/css-sizing/image-fractional-height-with-wide-aspect-ratio.html) [(live test)](http://wpt.live/css/css-sizing/image-fractional-height-with-wide-aspect-ratio.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-fractional-height-with-wide-aspect-ratio.html)
- [intrinsic-fixed-width-with-max-content-height.html](https://wpt.fyi/results/css/css-sizing/intrinsic-fixed-width-with-max-content-height.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-fixed-width-with-max-content-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-fixed-width-with-max-content-height.html)
- [intrinsic-fixed-width-with-min-content-height.html](https://wpt.fyi/results/css/css-sizing/intrinsic-fixed-width-with-min-content-height.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-fixed-width-with-min-content-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-fixed-width-with-min-content-height.html)
- [intrinsic-percent-replaced-001.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-001.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-001.html)
- [intrinsic-percent-replaced-002.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-002.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-002.html)
- [intrinsic-percent-replaced-003.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-003.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-003.html)
- [intrinsic-percent-replaced-004.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-004.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-004.html)
- [intrinsic-percent-replaced-005.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-005.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-005.html)
- [intrinsic-percent-replaced-006.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-006.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-006.html)
- [intrinsic-percent-replaced-007.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-007.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-007.html)
- [intrinsic-percent-replaced-008.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-008.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-008.html)
- [intrinsic-percent-replaced-009.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-009.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-009.html)
- [intrinsic-percent-replaced-010.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-010.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-010.html)
- [intrinsic-percent-replaced-011.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-011.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-011.html)
- [intrinsic-percent-replaced-012.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-012.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-012.html)
- [intrinsic-percent-replaced-013.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-013.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-013.html)
- [intrinsic-percent-replaced-014.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-014.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-014.html)
- [intrinsic-percent-replaced-015.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-015.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-015.html)
- [intrinsic-percent-replaced-016.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-016.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-016.html)
- [intrinsic-percent-replaced-017.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-017.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-017.html)
- [intrinsic-percent-replaced-018.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-018.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-018.html)
- [intrinsic-percent-replaced-019.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-019.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-019.html)
- [intrinsic-percent-replaced-020.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-020.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-020.html)
- [intrinsic-percent-replaced-021.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-021.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-021.html)
- [intrinsic-percent-replaced-022.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-022.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-022.html)
- [intrinsic-percent-replaced-023.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-023.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-023.html)
- [intrinsic-percent-replaced-024.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-024.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-024.html)
- [intrinsic-percent-replaced-025.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-025.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-025.html)
- [intrinsic-percent-replaced-026.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-026.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-026.html)
- [intrinsic-percent-replaced-027.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-027.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-027.html)
- [intrinsic-size-fallback-video.html](https://wpt.fyi/results/css/css-sizing/intrinsic-size-fallback-video.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-size-fallback-video.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-size-fallback-video.html)
- [max-content-input-001.html](https://wpt.fyi/results/css/css-sizing/max-content-input-001.html) [(live test)](http://wpt.live/css/css-sizing/max-content-input-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/max-content-input-001.html)
- [ortho-writing-mode-001.html](https://wpt.fyi/results/css/css-sizing/ortho-writing-mode-001.html) [(live test)](http://wpt.live/css/css-sizing/ortho-writing-mode-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/ortho-writing-mode-001.html)
- [orthogonal-writing-mode-float-in-inline.html](https://wpt.fyi/results/css/css-sizing/orthogonal-writing-mode-float-in-inline.html) [(live test)](http://wpt.live/css/css-sizing/orthogonal-writing-mode-float-in-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/orthogonal-writing-mode-float-in-inline.html)
- [replaced-aspect-ratio-intrinsic-size-001.html](https://wpt.fyi/results/css/css-sizing/replaced-aspect-ratio-intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-sizing/replaced-aspect-ratio-intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-aspect-ratio-intrinsic-size-001.html)
- [replaced-aspect-ratio-intrinsic-size-002.html](https://wpt.fyi/results/css/css-sizing/replaced-aspect-ratio-intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-sizing/replaced-aspect-ratio-intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-aspect-ratio-intrinsic-size-002.html)
- [replaced-aspect-ratio-stretch-fit-001.html](https://wpt.fyi/results/css/css-sizing/replaced-aspect-ratio-stretch-fit-001.html) [(live test)](http://wpt.live/css/css-sizing/replaced-aspect-ratio-stretch-fit-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-aspect-ratio-stretch-fit-001.html)
- [replaced-aspect-ratio-stretch-fit-002.html](https://wpt.fyi/results/css/css-sizing/replaced-aspect-ratio-stretch-fit-002.html) [(live test)](http://wpt.live/css/css-sizing/replaced-aspect-ratio-stretch-fit-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-aspect-ratio-stretch-fit-002.html)
- [replaced-aspect-ratio-stretch-fit-003.html](https://wpt.fyi/results/css/css-sizing/replaced-aspect-ratio-stretch-fit-003.html) [(live test)](http://wpt.live/css/css-sizing/replaced-aspect-ratio-stretch-fit-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-aspect-ratio-stretch-fit-003.html)
- [svg-intrinsic-size-001.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-001.html)
- [svg-intrinsic-size-002.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-002.html)
- [svg-intrinsic-size-003.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-003.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-003.html)
- [svg-intrinsic-size-004.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-004.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-004.html)
- [svg-intrinsic-size-005.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-005.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-005.html)
- [svg-intrinsic-size-006.html](https://wpt.fyi/results/css/css-sizing/svg-intrinsic-size-006.html) [(live test)](http://wpt.live/css/css-sizing/svg-intrinsic-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/svg-intrinsic-size-006.html)
- [whitespace-and-break.html](https://wpt.fyi/results/css/css-sizing/whitespace-and-break.html) [(live test)](http://wpt.live/css/css-sizing/whitespace-and-break.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/whitespace-and-break.html)

### <a id="intrinsic-contribution"></a>5.2.  Intrinsic Contributions

<a id="ref-for-min-content-contribution④"></a>

<a id="ref-for-max-content-contribution④"></a>

<a id="ref-for-valdef-width-auto①⑥"></a>

A box’s [min-content contribution](#min-content-contribution)/[max-content contribution](#max-content-contribution) in each axis is the size of the content box of a hypothetical [auto](#valdef-width-auto)-sized float that contains only that box, if that hypothetical float’s containing block is zero-sized/infinitely-sized.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define precisely how to determine these sizes. Please refer to [\[CSS2\]](#biblio-css2), the relevant CSS specification for that display type, the [rules for handling percentages](#percentage-sizing) (below), and/or existing implementations for further details.

Tests

- [intrinsic-percent-non-replaced-001.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-001.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-001.html)
- [intrinsic-percent-non-replaced-002.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-002.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-002.html)
- [intrinsic-percent-non-replaced-003.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-003.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-003.html)
- [intrinsic-percent-non-replaced-004.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-004.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-004.html)
- [intrinsic-percent-non-replaced-005.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-005.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-005.html)

<a id="ref-for-valdef-align-self-stretch②"></a>

<a id="ref-for-self-align①"></a>

<a id="ref-for-stretch-fit-size⑦"></a>

For this purpose, [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) [self-alignment](https://www.w3.org/TR/css-align-3/#self-align) and [stretch-fit sizing](#stretch-fit-size) (when they are able to resolve extrinsically) are considered definite the same way as resolveable percentages.

#### <a id="cyclic-percentage-contribution"></a>5.2.1.  Intrinsic Contributions of Percentage-Sized Boxes

<a id="ref-for-containing-block①⑨"></a>

<a id="ref-for-intrinsic-size-contribution"></a>

<a id="ref-for-automatic-minimum-size"></a>

Sometimes the size of a percentage-sized box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) depends on the [intrinsic size contribution](#intrinsic-size-contribution) of the box itself, creating a cyclic dependency. When calculating the <a id="ref-for-intrinsic-size-contribution①"></a>intrinsic size contribution of such a box (including any calculations for a content-based [automatic minimum size](#automatic-minimum-size)), a percentage value that resolves against a size in the same axis as the <a id="ref-for-intrinsic-size-contribution②"></a>intrinsic size contribution (a <a id="cyclic-percentage-size"></a>cyclic percentage size) is resolved specially:

1.  <a id="ref-for-valdef-width-auto①⑦"></a>

    <a id="ref-for-initial-value"></a>

    <a id="ref-for-intrinsic-size-contribution③"></a>

    <a id="ref-for-cyclic-percentage-size"></a>

    <a id="ref-for-css-contain-a-percentage"></a>

    <a id="ref-for-propdef-max-height④"></a>

    <a id="ref-for-propdef-height①⑥"></a>

    <a id="ref-for-propdef-max-width⑤"></a>

    <a id="ref-for-propdef-width①⑨"></a>

    <a id="ref-for-preferred-size-properties①"></a>

    <a id="ref-for-max-size-properties①"></a>

    <a id="ref-for-non-replaced①"></a>

    <a id="non-replaced-percentage-contribution"></a> If the box is [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced), then the entire value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) ([width](#propdef-width)/[max-width](#propdef-max-width)/[height](#propdef-height)/[max-height](#propdef-max-height)) specified as an expression [containing a percentage](https://www.w3.org/TR/css-values-4/#css-contain-a-percentage) (such as 10% or calc(10px + 0%)) that is [cyclic](#cyclic-percentage-size) is treated <em>for the purpose of calculating the box’s <a href="#intrinsic-size-contribution">intrinsic size contributions</a> only</em> as that property’s [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value). For example, given a box with <a id="ref-for-propdef-width②⓪"></a>width: calc(20px + 50%), its max-content contribution is calculated as if its <a id="ref-for-propdef-width②①"></a>width were [auto](#valdef-width-auto). (The percentage is honored as usual, however, during the actual sizing of the box itself; see below.)

2.  <a id="ref-for-initial-value①"></a>

    <a id="ref-for-max-content-contribution⑤"></a>

    <a id="ref-for-cyclic-percentage-size①"></a>

    <a id="ref-for-preferred-size-properties②"></a>

    <a id="ref-for-max-size-properties②"></a>

    <a id="ref-for-replaced-element②"></a>

    <a id="replaced-percentage-max-contribution"></a> Likewise, if the box is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element), then the entire value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) specified as an expression containing a percentage that is [cyclic](#cyclic-percentage-size) is treated <em>for the purpose of calculating the box’s <a href="#max-content-contribution">max-content contributions</a> only</em> as that property’s [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

3.  <a id="ref-for-min-width⑧"></a>

    <a id="ref-for-typedef-length-percentage①②"></a>

    <a id="ref-for-preferred-aspect-ratio⑤"></a>

    <a id="ref-for-min-content-contribution⑤"></a>

    <a id="ref-for-propdef-max-height⑤"></a>

    <a id="ref-for-propdef-height①⑦"></a>

    <a id="ref-for-propdef-max-width⑥"></a>

    <a id="ref-for-propdef-width②②"></a>

    <a id="ref-for-preferred-size-properties③"></a>

    <a id="ref-for-max-size-properties③"></a>

    <a id="ref-for-cyclic-percentage-size②"></a>

    <a id="ref-for-replaced-element③"></a>

    <a id="replaced-percentage-min-contribution"></a> If the box is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element), a [cyclic percentage](#cyclic-percentage-size) in the value of any [max size property](#max-size-properties) or [preferred size property](#preferred-size-properties) ([width](#propdef-width)/[max-width](#propdef-max-width)/[height](#propdef-height)/[max-height](#propdef-max-height)), is resolved against zero when calculating the [min-content contribution](#min-content-contribution) in the corresponding axis. (See [§ 5.2.2 Compressible Replaced Elements](#min-content-zero) for a list of which elements in HTML this applies to.) If the box also has a [preferred aspect ratio](#preferred-aspect-ratio), then this <a id="ref-for-min-content-contribution⑥"></a>min-content contribution is floored by any [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [minimum size](#min-width) from the opposite axis—​resolving any such percentage against zero—​transferred through the <a id="ref-for-preferred-aspect-ratio⑥"></a>preferred aspect ratio.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-1b92b23f"></a> Should we resolve transferred percentages against their containing block instead of zero before transferring them? See [discussion](https://github.com/w3c/csswg-drafts/issues/6341).

    <a id="ref-for-min-content-contribution⑦"></a>

    <a id="ref-for-the-select-element"></a>

    The UA may additionally floor the [min-content contribution](#min-content-contribution) based on UI considerations, such as ensuring certain UI elements remain visible (for example, the dropdown arrow on a <code><a href="https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element">select</a></code>).

    <a id="ref-for-min-content-contribution⑧"></a>

    <a id="ref-for-min-width⑨"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The [min-content contribution](#min-content-contribution) is, as always, also floored by the [minimum size](#min-width) in its own axis.

    <a id="ref-for-automatic-minimum-size①"></a>

    <a id="ref-for-definite①①"></a>

    This rule also applies when calculating a content-based [automatic minimum size](#automatic-minimum-size) or its corresponding size contribution, yielding a [definite](#definite) “specified size suggestion”.

    Tests
    - [intrinsic-percent-replaced-028.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-replaced-028.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-replaced-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-replaced-028.html)

    <a id="ref-for-the-input-element①"></a>

    <a id="ref-for-propdef-width②③"></a>

    <a id="ref-for-min-content-contribution⑨"></a>

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-b06093ff"></a> For example, an <code><a href="https://html.spec.whatwg.org/multipage/input.html#the-input-element">input</a></code> assigned [width: calc(50% + 50px)](#propdef-width) has a [min-content contribution](#min-content-contribution) of 50px, plus any horizontal margin/border/padding.

4.  <a id="ref-for-intrinsic-size-contribution④"></a>

    <a id="ref-for-cyclic-percentage-size③"></a>

    <a id="ref-for-gutter"></a>

    <a id="ref-for-padding②"></a>

    <a id="ref-for-margin①"></a>

    <a id="ref-for-min-size-properties②"></a>

    <a id="min-percentage-contribution"></a> For the [min size properties](#min-size-properties), as well as for [margins](https://www.w3.org/TR/css-box-4/#margin) and [paddings](https://www.w3.org/TR/css-box-4/#padding) (and [gutters](https://www.w3.org/TR/css-gaps-1/#gutter)), a [cyclic percentage](#cyclic-percentage-size) is resolved against zero for determining [intrinsic size contributions](#intrinsic-size-contribution).

Tests

- [inline-intrinsic-size-calc.html](https://wpt.fyi/results/css/css-sizing/inline-intrinsic-size-calc.html) [(live test)](http://wpt.live/css/css-sizing/inline-intrinsic-size-calc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/inline-intrinsic-size-calc.html)
- [intrinsic-percent-non-replaced-006.html](https://wpt.fyi/results/css/css-sizing/intrinsic-percent-non-replaced-006.html) [(live test)](http://wpt.live/css/css-sizing/intrinsic-percent-non-replaced-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/intrinsic-percent-non-replaced-006.html)

**Table 6**

Summary of the Cyclic-Percentage Intrinsic Size Contribution Rules (Above)

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Element Type / Contribution Type | <a id="ref-for-replaced-element④"></a> [Replaced](https://www.w3.org/TR/css-display-4/#replaced-element) / <a id="ref-for-min-content-contribution①⓪"></a> [min-content](#min-content-contribution) | [Replaced](https://www.w3.org/TR/css-display-4/#replaced-element) / <a id="ref-for-max-content-contribution⑥"></a> [max-content](#max-content-contribution) | <a id="ref-for-non-replaced②"></a> [Non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) / <a id="ref-for-min-content-contribution①①"></a> [min-content](#min-content-contribution) | [Non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) / <a id="ref-for-max-content-contribution⑦"></a> [max-content](#max-content-contribution) |
| --- | --- | --- | --- | --- |
| <a id="ref-for-padding-properties"></a> <a id="ref-for-margin-properties"></a> <a id="ref-for-min-size-properties③"></a> [min size](#min-size-properties) &#x26; [margin](https://www.w3.org/TR/css-box-4/#margin-properties)/[padding](https://www.w3.org/TR/css-box-4/#padding-properties) | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) | [zeroᵈ](#min-percentage-contribution) |
| <a id="ref-for-preferred-size-properties④"></a> <a id="ref-for-max-size-properties④"></a> [max](#max-size-properties) &#x26; [preferred size](#preferred-size-properties) | [zeroᶜ](#replaced-percentage-min-contribution) | [initialᵇ](#replaced-percentage-max-contribution) | [initialᵃ](#non-replaced-percentage-contribution) | [initialᵃ](#non-replaced-percentage-contribution) |

Then, unless otherwise specified, when calculating the used sizes and positions of the containing block’s <em>contents</em>:

- <a id="ref-for-block-axis③"></a>

  <a id="ref-for-min-width①⓪"></a>

  <a id="ref-for-propdef-block-size③"></a>

  <a id="ref-for-propdef-max-block-size②"></a>

  <a id="ref-for-propdef-flex-basis②"></a>

  <a id="ref-for-flex-layout①"></a>

  <a id="ref-for-behave-as-auto⑥"></a>

  If the cyclic dependency was introduced due to a [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) size other than a [minimum size](#min-width) on the containing block (i.e. a [block-size](#propdef-block-size) or [max-block-size](#propdef-max-block-size) in most layout modes, or a [flex-basis](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-basis) in [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout)) that causes it to depend on the size of its contents, the box’s percentage is not resolved and instead [behaves as auto](#behave-as-auto).

  <a id="ref-for-grid-item"></a>

  <a id="ref-for-flex-item"></a>

  <a id="ref-for-main-axis"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: [Grid items](https://www.w3.org/TR/css-grid-2/#grid-item) in both axes, [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) in the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis), and children of <a id="ref-for-flex-item①"></a>flex items in both axes do allow percentages to resolve in this case.

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
> <a id="ref-for-propdef-width②④"></a>
>
> <a id="ref-for-definite①②"></a>
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
> <a id="ref-for-valdef-width-auto①⑧"></a>
>
> because the percentage [block size](#block-size) ([height](#propdef-height), in this case) on block-level elements is defined to not resolve inside content-sized containing blocks, the percentage height on the `<aside>` is ignored, that is, it behaves exactly as if [auto](#valdef-width-auto) were specified.

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
> <a id="ref-for-behave-as-auto⑦"></a>
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
> <a id="ref-for-valdef-width-auto①⑨"></a>
>
> <a id="ref-for-behave-as-auto⑧"></a>
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
> <a id="ref-for-behave-as-auto⑨"></a>
>
> <a id="ref-for-valdef-width-min-content⑤"></a>
>
> This is a variation on the first code block, and follows a similar path; the `<aside>` initially wants to compute to 200px tall (200% of the 100px containing block height). When we calculate the effects of [min-height](#propdef-min-height), the percentage [behaves as auto](#behave-as-auto), causing it to become 150px tall, and the total [min-content](#valdef-width-min-content) height of the containing block to be 180px tall. Since this is larger than 100px, the `<article>` gets clamped to 180px, the percentage resolves against this new height, and the `<aside>` ends up being 360px tall, overflowing the `<article>`

#### <a id="min-content-zero"></a>5.2.2.  Compressible Replaced Elements

<a id="ref-for-replaced-element⑤"></a>

<a id="ref-for-min-content-contribution①②"></a>

<a id="ref-for-propdef-width②⑤"></a>

<a id="ref-for-propdef-height②③"></a>

<a id="ref-for-propdef-max-width⑦"></a>

<a id="ref-for-propdef-max-height⑥"></a>

In addition to the [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) listed in [HTML§14.4](https://html.spec.whatwg.org/multipage/rendering.html#replaced-elements) [\[HTML\]](#biblio-html), the following HTML elements are also considered to be <a id="ref-for-replaced-element⑥"></a>replaced elements for the purpose of the [percentage-sized replaced element rule](#replaced-percentage-min-contribution) above, and can have their [min-content contribution](#min-content-contribution) compressed when their [width](#propdef-width)/[height](#propdef-height) or [max-width](#propdef-max-width)/[max-height](#propdef-max-height) is expressed with a cyclic percentage size:

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

Changes since the [17 December 2021 Working Draft](https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/) include:

- <a id="ref-for-valdef-width-stretch③"></a>

  <a id="ref-for-valdef-width-fit-content②"></a>

  Moved fit-content() to Level 4, pulled [fit-content](#valdef-width-fit-content) and [stretch](#valdef-width-stretch) from Level 4. ([Issue 10601](https://github.com/w3c/csswg-drafts/issues/10601))

- <a id="ref-for-sizing-property①⓪"></a>

  <a id="ref-for-flow-relative⑤"></a>

  Imported definitions of the [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) [sizing properties](#sizing-property) from [\[CSS-LOGICAL-1\]](#biblio-css-logical-1). ([Issue 10189](https://github.com/w3c/csswg-drafts/issues/10189))

- <a id="ref-for-stretch-fit-size⑧"></a>

  Clarified details for [stretch-fit sizing](#stretch-fit-size). ([Issue 11044](https://github.com/w3c/csswg-drafts/issues/11044), [Issue 11006](https://github.com/w3c/csswg-drafts/issues/11006), [Issue 11076](https://github.com/w3c/csswg-drafts/issues/11076), [Issue 4028](https://github.com/w3c/csswg-drafts/issues/4028), [Issue 13260](https://github.com/w3c/csswg-drafts/issues/13260), [Issue 11489](https://github.com/w3c/csswg-drafts/issues/11489))

- <a id="ref-for-typedef-box-size⑦"></a>

  Introduced [\<box-size\>](#typedef-box-size) grammar production to consolidate sizing values. ([Issue 13478](https://github.com/w3c/csswg-drafts/issues/13478))

- <a id="ref-for-preferred-aspect-ratio⑦"></a>

  Clarified that rules for finding the intrinsic size of replaced elements with a [preferred aspect ratio](#preferred-aspect-ratio) only apply when the aspect ratio is non-degenerate. ([Issue 12612](https://github.com/w3c/csswg-drafts/issues/12612))

- <a id="ref-for-min-width①①"></a>

  <a id="ref-for-valdef-width-auto②⓪"></a>

  <a id="ref-for-resolved-value①"></a>

  <a id="ref-for-propdef-aspect-ratio①"></a>

  Defined that [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) [\[css-sizing-4\]](#biblio-css-sizing-4) preserves the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of an [auto](#valdef-width-auto) [minimum size](#min-width) as <a id="ref-for-valdef-width-auto②①"></a>auto. ([Issue 11716](https://github.com/w3c/csswg-drafts/issues/11716))

- <a id="ref-for-min-content-contribution①③"></a>

  <a id="ref-for-min-content①⑥"></a>

  <a id="ref-for-max-content-contribution⑧"></a>

  <a id="ref-for-max-content①⑤"></a>

  Clarified that the [max-content size](#max-content) and [max-content contribution](#max-content-contribution) are floored by the [min-content size](#min-content)/[min-content contribution](#min-content-contribution). ([Issue 12076](https://github.com/w3c/csswg-drafts/issues/12076))

- Clarified handling of cyclic percentages in intrinsic size contribution calculations. ([Issue 6822](https://github.com/w3c/csswg-drafts/issues/6822))

- Clarified that stretch sizes are handled similarly to percentages for the purpose of intrinsic size calculations. ([Issue 10619](https://github.com/w3c/csswg-drafts/issues/10619))

- Clarified indefiniteness of intrinsic size keywords. ([Issue 7206](https://github.com/w3c/csswg-drafts/issues/7206))

- <a id="ref-for-sizing-property①①"></a>

  Added logical property groups to the [sizing properties](#sizing-property). ([Issue 2822](https://github.com/w3c/csswg-drafts/issues/2822))

- Added Web Platform Tests coverage.

- Various other minor editorial fixes and improvements.

Changes since the [18 December 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-3-20201218/) include:

- <a id="ref-for-valdef-contain-intrinsic-width-auto"></a>

  <a id="ref-for-propdef-contain-intrinsic-size"></a>

  Fixed the order of [contain-intrinsic-size](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size) values when [auto](https://drafts.csswg.org/css-sizing-4/#valdef-contain-intrinsic-width-auto) is combined with other values so that parsing is unambiguous. ([Issue 6391](https://github.com/w3c/csswg-drafts/issues/6391))

- <a id="ref-for-propdef-contain-intrinsic-size①"></a>

  Clarified which elements are allowed to not have a last remembered size for [contain-intrinsic-size: auto](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size). ([Issue 6220](https://github.com/w3c/csswg-drafts/issues/6220))

- <a id="ref-for-valdef-content-visibility-auto"></a>

  <a id="ref-for-propdef-contain-intrinsic-size②"></a>

  Limited [contain-intrinsic-size: auto](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size) to when content-visibility is [auto](https://www.w3.org/TR/css-contain-2/#valdef-content-visibility-auto). ([Issue 6308](https://github.com/w3c/csswg-drafts/issues/6308))

Changes since the [18 December 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-3-20201218/) include:

- Fixed various errors in definition of max-content sizes of replaced elements in [§ 5.1 Intrinsic Sizes](#intrinsic-sizes). ([Issue 6072](https://github.com/w3c/csswg-drafts/issues/6072))

- <a id="ref-for-fit-content-size③"></a>

  <a id="ref-for-min-content-constraint③"></a>

  Added missing statement handling [min-content constraint](#min-content-constraint) to definition of [fit-content size](#fit-content-size).

- <a id="ref-for-intrinsic-size⑤"></a>

  Renamed replaced element “intrinsic” dimensions to “natural” dimensions in order to avoid confusion with [intrinsic sizes](#intrinsic-size) (see [Issue 4961](https://github.com/w3c/csswg-drafts/issues/4961)).

- Various other minor editorial fixes and improvements.

Major changes since the [22 May 2019 Working Draft](https://www.w3.org/TR/2019/WD-css-sizing-3-20190522/) include:

- <a id="ref-for-valdef-width-max-content⑤"></a>

  <a id="ref-for-valdef-width-min-content⑥"></a>

  Defined that [min-content](#valdef-width-min-content) and [max-content](#valdef-width-max-content) do not necessarily behave the same as the property’s initial value if otherwise specified (by the relevant layout module). ([Issue 3973](https://github.com/w3c/csswg-drafts/issues/3973))

- <a id="ref-for-funcdef-width-fit-content④"></a>

  Switched intrinsic contribution of [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content) to treat its argument as that argument would be treated alone for intrinsic contribution calculations and resolve the fit-content formula accordingly, rather than having special behavior for <a id="ref-for-funcdef-width-fit-content⑤"></a>fit-content() resolution when calculating intrinsic contributions. ([Issue 3731](https://github.com/w3c/csswg-drafts/issues/3731))

- <a id="ref-for-length-value⑥"></a>

  <a id="ref-for-min-width①②"></a>

  <a id="ref-for-max-content①⑥"></a>

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

- <a id="ref-for-valdef-width-auto②②"></a>

  Compute new keywords to the initial value, not to a potentially non-existent [auto](#valdef-width-auto), when applied to the block axis.

- Specify that percent sizes on replaced elements zero out their min-content contribution.

- Fix confusing/wrong definition of percentage sizes resolved against a dependent containing block. (This may require further work.)

- <a id="ref-for-indefinite⑤"></a>

  <a id="ref-for-valdef-width-fit-content③"></a>

  <a id="ref-for-valdef-width-stretch④"></a>

  Deferred the [stretch](#valdef-width-stretch) and [fit-content](#valdef-width-fit-content) keywords to Level 4 to allow for further consideration of their behavior in [indefinite](#indefinite) containing blocks.

- <a id="ref-for-propdef-box-sizing⑧"></a>

  <a id="ref-for-propdef-max-height⑦"></a>

  <a id="ref-for-propdef-min-height①⓪"></a>

  <a id="ref-for-propdef-min-width⑧"></a>

  <a id="ref-for-propdef-height②④"></a>

  <a id="ref-for-propdef-width②⑥"></a>

  Pulled in full definitions for all of the sizing properties (rather than diffing them): [width](#propdef-width), [height](#propdef-height), [min-width](#propdef-min-width), [min-height](#propdef-min-height), max-width', [max-height](#propdef-max-height), and [box-sizing](#propdef-box-sizing).

### <a id="changes-3"></a> Additions since CSS Level 2

In addition to substantially more detail to the various automatic and content-based sizing algorithms, the following new features have been added since [\[CSS2\]](#biblio-css2):

- <a id="ref-for-propdef-box-sizing⑨"></a>

  The [box-sizing](#propdef-box-sizing) property (originally defined in [\[CSS-UI-3\]](#biblio-css-ui-3), then moved here).

- <a id="ref-for-sizing-property①②"></a>

  <a id="ref-for-valdef-width-fit-content④"></a>

  <a id="ref-for-valdef-width-max-content⑥"></a>

  <a id="ref-for-valdef-width-min-content⑦"></a>

  <a id="ref-for-valdef-width-stretch⑤"></a>

  The [stretch](#valdef-width-stretch), [min-content](#valdef-width-min-content), [max-content](#valdef-width-max-content), and [fit-content](#valdef-width-fit-content) values of the [sizing properties](#sizing-property).

- <a id="ref-for-propdef-min-height①①"></a>

  <a id="ref-for-propdef-min-width⑨"></a>

  <a id="ref-for-valdef-width-auto②③"></a>

  The [auto](#valdef-width-auto) initial value of the [min-width](#propdef-min-width) and [min-height](#propdef-min-height) properties (originally defined in [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1), then moved here).

## <a id="acknowledgments"></a> Acknowledgments

Special thanks go to L. David Baron, Aaron Gustafson, Daniel Holbert, and Mats Palmgren for their contributions to this module.

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-the-iframe-element①"></a>

<a id="ref-for-replaced-element⑦"></a>

In order to support automatic layout, CSS sizes boxes to fit their contents. In conjunction with various [\[DOM\]](#biblio-dom) and [\[CSSOM\]](#biblio-cssom) APIs which can return the size of those boxes to script, this can expose information about those contents. However, this information is more directly and easily available by inspecting the DOM for the contents, rather than indirecting through the box’s size. Containers that can’t have their contents inspected (such as cross-origin <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>s) also do not expose sizing information to the outer page, except insofar as [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) such as images expose their natural size and/or aspect ratio.

## <a id="security"></a> Security Considerations

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
- [block size](#block-size), in § 3.1.1
- [block-size](#propdef-block-size), in § 3.1.1
- [border-box](#valdef-box-sizing-border-box), in § 3.3
- [\<box-size\>](#typedef-box-size), in § 3.2
- [box-sizing](#propdef-box-sizing), in § 3.3
- [content-box](#valdef-box-sizing-content-box), in § 3.3
- [cyclic percentage](#cyclic-percentage-size), in § 5.2.1
- [cyclic percentage size](#cyclic-percentage-size), in § 5.2.1
- [definite](#definite), in § 2
- [definite size](#definite), in § 2
- [Extrinsic sizing](#extrinsic-sizing), in § 4
- [fallback](#fallback), in § 2
- [fallback size](#fallback), in § 2
- fit-content
  - [value for column-width](#valdef-column-width-fit-content), in § 3.4
  - [value for width, min-width, max-width, height, min-height, max-height](#valdef-width-fit-content), in § 3.2
- [fit-content block size](#fit-content-block-size), in § 2.1
- [fit-content inline size](#fit-content-inline-size), in § 2.1
- [fit-content size](#fit-content-size), in § 2.1
- height
  - [(property)](#propdef-height), in § 3.1.1
  - [definition of](#height), in § 3.1.1
- [indefinite](#indefinite), in § 2
- [indefinite size](#indefinite), in § 2
- [inline size](#inline-size), in § 3.1.1
- [inline-size](#propdef-inline-size), in § 3.1.1
- [inner block size](#inner-size), in § 2
- [inner height](#inner-size), in § 2
- [inner inline size](#inner-size), in § 2
- [inner size](#inner-size), in § 2
- [inner width](#inner-size), in § 2
- [intrinsic size](#intrinsic-size), in § 2.1
- [intrinsic size constraint](#constraints), in § 2.2
- [intrinsic size contribution](#intrinsic-size-contribution), in § 2.2
- [Intrinsic sizing](#intrinsic-sizing), in § 5
- [\<length-percentage \[0,∞\]\>](#valdef-width-length-percentage-0), in § 3.2
- [max block size](#max-block-size), in § 3.1.3
- [max-block-size](#propdef-max-block-size), in § 3.1.3
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
- [maximum block size](#max-block-size), in § 3.1.3
- [maximum height](#max-height), in § 3.1.3
- [maximum inline size](#max-inline-size), in § 3.1.3
- [maximum size](#max-width), in § 3.1.3
- [maximum width](#max-width), in § 3.1.3
- [max inline size](#max-inline-size), in § 3.1.3
- [max-inline-size](#propdef-max-inline-size), in § 3.1.3
- [max size](#max-width), in § 3.1.3
- [max size property](#max-size-properties), in § 3.1.2
- [max width](#max-width), in § 3.1.3
- [max-width](#propdef-max-width), in § 3.1.3
- [min block size](#min-block-size), in § 3.1.2
- [min-block-size](#propdef-min-block-size), in § 3.1.2
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
- [minimum block size](#min-block-size), in § 3.1.2
- [minimum height](#min-height), in § 3.1.2
- [minimum inline size](#min-inline-size), in § 3.1.2
- [minimum size](#min-width), in § 3.1.2
- [minimum width](#min-width), in § 3.1.2
- [min inline size](#min-inline-size), in § 3.1.2
- [min-inline-size](#propdef-min-inline-size), in § 3.1.2
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
- [preferred block size](#block-size), in § 3.1.1
- [preferred height](#height), in § 3.1.1
- [preferred inline size](#inline-size), in § 3.1.1
- [preferred size](#preferred-size), in § 3.1.1
- [preferred size property](#preferred-size-properties), in § 3.1
- [preferred width](#width), in § 3.1.1
- [size](#size), in § 2
- [sizing property](#sizing-property), in § 3.1
- stretch
  - [value for column-width](#valdef-column-width-stretch), in § 3.4
  - [value for width, min-width, max-width, height, min-height, max-height](#valdef-width-stretch), in § 3.2
- [stretch fit](#stretch-fit), in § 2
- [stretch-fit block size](#stretch-fit-block-size), in § 2.1
- [stretch-fit inline size](#stretch-fit-inline-size), in § 2.1
- [stretch-fit size](#stretch-fit-size), in § 2.1
- width
  - [(property)](#propdef-width), in § 3.1.1
  - [definition of](#width), in § 3.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="5f80981e"></a>self-alignment
  - <a id="598fa031"></a>stretch
- \[CSS-BORDERS-4\] defines the following terms:
  - <a id="f169a45f"></a>border
- \[CSS-BOX-4\] defines the following terms:
  - <a id="30e036e4"></a>border
  - <a id="85c399c0"></a>border box
  - <a id="f72f5cb4"></a>content box
  - <a id="d049494a"></a>margin
  - <a id="0778a939"></a>margin box
  - <a id="62f136c0"></a>margin properties
  - <a id="a2be8c84"></a>padding
  - <a id="1c1b0c95"></a>padding properties
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="1a2b1083"></a>used value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="a6218d6a"></a>auto
  - <a id="3e4b15e8"></a>size containment
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="5c159f8f"></a>box
  - <a id="2ccfe434"></a>display
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="45f9eae9"></a>block box
  - <a id="8d18d112"></a>block container
  - <a id="91b1f11d"></a>block-level box
  - <a id="0923db9e"></a>containing block
  - <a id="af3737f8"></a>display type
  - <a id="ae223697"></a>formatting context
  - <a id="6658d41f"></a>in-flow
  - <a id="57aa8824"></a>independent formatting context
  - <a id="d1ebdd75"></a>initial containing block
  - <a id="2e557886"></a>inline
  - <a id="f089a6e1"></a>inline box
  - <a id="e89ddbcb"></a>non-replaced
  - <a id="7a605ac8"></a>principal box
  - <a id="380d5174"></a>replaced
  - <a id="a9db5d6d"></a>replaced element
  - <a id="e4657f7f"></a>text sequence
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="34dea1dc"></a>cross size
  - <a id="9f6d5ab0"></a>flex item
  - <a id="07e702cf"></a>flex layout
  - <a id="50463bf6"></a>flex-basis
  - <a id="98f2297b"></a>main axis
  - <a id="97651ccf"></a>multi-line flex container
- \[CSS-GAPS-1\] defines the following terms:
  - <a id="07db51c0"></a>gutter
- \[CSS-GRID-2\] defines the following terms:
  - <a id="ba30fc9a"></a>grid item
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="ffedca23"></a>natural aspect ratio
  - <a id="487e1aa9"></a>natural dimension
  - <a id="c0cc78c8"></a>natural size
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="a9330658"></a>line box
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="7777143d"></a>column-width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="d1cbb104"></a>aspect-ratio
  - <a id="b495d7b2"></a>auto (for aspect-ratio)
  - <a id="9acf31c7"></a>auto (for contain-intrinsic-width)
  - <a id="29688800"></a>contain-intrinsic-size
  - <a id="2b34d13e"></a>fit-content()
  - <a id="7b40d8b7"></a>size
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="a464ce7e"></a>soft wrap opportunity
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="510ff601"></a>contain a percentage
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4eb9d37e"></a>\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="599428b5"></a>block-axis
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="303c8d41"></a>flow-relative
  - <a id="a6eb24bb"></a>inline axis
  - <a id="e1f6e4b9"></a>physical
  - <a id="eb6008ce"></a>writing mode
- \[CSSOM\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
  - <a id="fc19454a"></a>resolved value
- \[HTML\] defines the following terms:
  - <a id="bfff6250"></a>button
  - <a id="87fcd40c"></a>iframe
  - <a id="d7d642a2"></a>input
  - <a id="83d1e90e"></a>marquee
  - <a id="b23654ab"></a>meter
  - <a id="b90b2ad5"></a>progress
  - <a id="b3bf6f52"></a>raw value
  - <a id="85188fb3"></a>select
  - <a id="fc736137"></a>textarea
  - <a id="66aee777"></a>type

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 30 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-borders-4"></a>\[CSS-BORDERS-4\]  
Elika Etemad; et al. [CSS Borders and Box Decorations Module Level 4](https://www.w3.org/TR/css-borders-4/). 16 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-borders-4&#x2F;](https://www.w3.org/TR/css-borders-4/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 5 June 2026. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 14 October 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-gaps-1"></a>\[CSS-GAPS-1\]  
Kevin Babbitt. [CSS Gaps Module Level 1](https://www.w3.org/TR/css-gaps-1/). 24 June 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-gaps-1&#x2F;](https://www.w3.org/TR/css-gaps-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-multicol-2"></a>\[CSS-MULTICOL-2\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://www.w3.org/TR/css-multicol-2/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-2&#x2F;](https://www.w3.org/TR/css-multicol-2/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 14 August 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 7 April 2026. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

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

<a id="biblio-css3col"></a>\[CSS3COL\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Non-Normative References

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Elika Etemad; Rossen Atanassov. [CSS Logical Properties and Values Module Level 1](https://www.w3.org/TR/css-logical-1/). 4 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

## <a id="property-index"></a>Property Index

| Name                | Value                     | Initial     | Applies to                               | Inh. | %ages                                        | Anim­ation type                                       | Canonical order | Com­puted value                                           | Logical property group |
|---------------------|---------------------------|-------------|------------------------------------------|------|----------------------------------------------|------------------------------------------------------|-----------------|----------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-block-size④"></a></span><a href="#propdef-block-size">block-size</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed | size                   |
| <strong><span><a id="ref-for-propdef-box-sizing①⓪"></a></span><a href="#propdef-box-sizing">box-sizing</a>&#xA;      </strong> | content-box \| border-box | content-box | all elements that accept width or height | no   | N/A                                          | discrete                                             | per grammar     | specified keyword                                        |                        |
| <strong><span><a id="ref-for-propdef-height②⑤"></a></span><a href="#propdef-height">height</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed | size                   |
| <strong><span><a id="ref-for-propdef-inline-size③"></a></span><a href="#propdef-inline-size">inline-size</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed | size                   |
| <strong><span><a id="ref-for-propdef-max-block-size③"></a></span><a href="#propdef-max-block-size">max-block-size</a>&#xA;      </strong> | none \| \<box-size\>      | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | max-size               |
| <strong><span><a id="ref-for-propdef-max-height⑧"></a></span><a href="#propdef-max-height">max-height</a>&#xA;      </strong> | none \| \<box-size\>      | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | max-size               |
| <strong><span><a id="ref-for-propdef-max-inline-size②"></a></span><a href="#propdef-max-inline-size">max-inline-size</a>&#xA;      </strong> | none \| \<box-size\>      | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | max-size               |
| <strong><span><a id="ref-for-propdef-max-width⑧"></a></span><a href="#propdef-max-width">max-width</a>&#xA;      </strong> | none \| \<box-size\>      | none        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | max-size               |
| <strong><span><a id="ref-for-propdef-min-block-size②"></a></span><a href="#propdef-min-block-size">min-block-size</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | min-size               |
| <strong><span><a id="ref-for-propdef-min-height①②"></a></span><a href="#propdef-min-height">min-height</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | min-size               |
| <strong><span><a id="ref-for-propdef-min-inline-size②"></a></span><a href="#propdef-min-inline-size">min-inline-size</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | min-size               |
| <strong><span><a id="ref-for-propdef-min-width①⓪"></a></span><a href="#propdef-min-width">min-width</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements that accept width or height | no   | relative to width/height of containing block | by computed value, recursing into fit-content()      | per grammar     | as specified, with \<length-percentage\> values computed | min-size               |
| <strong><span><a id="ref-for-propdef-width②⑦"></a></span><a href="#propdef-width">width</a>&#xA;      </strong> | auto \| \<box-size\>      | auto        | all elements except non-replaced inlines | no   | relative to width/height of containing block | by computed value type, recursing into fit-content() | per grammar     | as specified, with \<length-percentage\> values computed | size                   |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This spec needs illustrations! See [issue](https://github.com/w3c/csswg-drafts/issues/1938). [↵](#issue-664b2cc3)

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
