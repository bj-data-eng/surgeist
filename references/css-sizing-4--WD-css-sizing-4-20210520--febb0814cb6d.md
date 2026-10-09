Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 4](https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/).

Original copyright notice: Copyright © 2021 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 4

Source snapshot: https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/

Snapshot SHA-256: febb0814cb6dd7ad343daea040f3aac9d3c56520bd20e64c6a8cf43ecc722e25

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Box Sizing Module Level 4

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context. This is a delta spec over CSS Sizing Level 3.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;Other documents may supersede this document.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Working Draft</strong>. Publication as a Working Draft does not imply endorsement by the W3C Membership.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-sizing” in the title, like this: “\[css-sizing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-sizing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f7037e82"></a> This is a diff spec over [CSS Sizing Level 3](https://www.w3.org/TR/css-sizing-3/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR.

### <a id="placement"></a>1.1.  Module interactions

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-column-width"></a>

This module extends the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), and [column-width](https://www.w3.org/TR/css-multicol-1/#propdef-column-width) features defined in [\[CSS2\]](#biblio-css2) chapter 10 and in [\[CSS3COL\]](#biblio-css3col)

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terms"></a>2.  Terminology

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9"></a> [CSS Sizing 3 §2 Terminology](https://www.w3.org/TR/css-sizing-3/#terms)

## <a id="specifying-sizes"></a>3.  Specifying Box Sizes<a id="size-keywords"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①"></a> [CSS Sizing 3 §3 Specifying Box Sizes](https://www.w3.org/TR/css-sizing-3/#specifying-sizes)

### <a id="sizing-properties"></a>3.1.  Sizing Properties

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-0e53565a"></a> Add shorthands. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;820&#x3E;](https://github.com/w3c/csswg-drafts/issues/820)

<a id="ref-for-valdef-width-stretch"></a>

<a id="ref-for-valdef-width-fit-content"></a>

<a id="ref-for-valdef-width-contain"></a>

### <a id="sizing-values"></a>3.2.  New Sizing Values: the [stretch](#valdef-width-stretch), [fit-content](#valdef-width-fit-content), and [contain](#valdef-width-contain) keywords<a id="width-height-keywords"></a>



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-max-block-size"></a><a id="ref-for-propdef-max-inline-size"></a><a id="ref-for-propdef-max-height①"></a><a id="ref-for-propdef-max-width①"></a><a id="ref-for-propdef-min-block-size"></a><a id="ref-for-propdef-min-inline-size"></a><a id="ref-for-propdef-min-height①"></a><a id="ref-for-propdef-min-width①"></a><a id="ref-for-propdef-block-size"></a><a id="ref-for-propdef-inline-size"></a><a id="ref-for-propdef-height①"></a><a id="ref-for-propdef-width①"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [inline-size](https://www.w3.org/TR/css-logical-1/#propdef-inline-size), [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size), [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), [min-inline-size](https://www.w3.org/TR/css-logical-1/#propdef-min-inline-size), [min-block-size](https://www.w3.org/TR/css-logical-1/#propdef-min-block-size), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), [max-inline-size](https://www.w3.org/TR/css-logical-1/#propdef-max-inline-size), [max-block-size](https://www.w3.org/TR/css-logical-1/#propdef-max-block-size) |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>stretch [\|](https://www.w3.org/TR/css-values-4/#comb-one) fit-content <a id="ref-for-comb-one①"></a>\| contain                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |



<a id="valdef-width-stretch"></a>stretch  
<a id="ref-for-containing-block"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-stretch-fit-size"></a>

Applies [stretch-fit sizing](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size), attempting to match the size of the box’s [margin box](https://www.w3.org/TR/css-box-4/#margin-box) to the size of its [containing block](https://www.w3.org/TR/css-display-3/#containing-block). See [§ 6.1 Stretch-fit Sizing: filling the containing block](#stretch-fit-sizing).

<a id="valdef-width-fit-content"></a>fit-content  
<a id="ref-for-valdef-width-stretch①"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-max-content"></a>

Essentially fit-content(stretch) i.e. min([max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content), max([min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [stretch](#valdef-width-stretch))).

<a id="valdef-width-contain"></a>contain  
<a id="ref-for-contain-fit-sizing①"></a>

<a id="ref-for-contain-fit-sizing"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

If the box has a [preferred aspect ratio](#preferred-aspect-ratio), applies [contain-fit sizing](#contain-fit-sizing), attempting to fit into the box’s constraints while maintaining its <a id="ref-for-preferred-aspect-ratio①"></a>preferred aspect ratio insofar as possible. See [§ 6.2 Contain-fit Sizing: stretching while maintaining an aspect ratio](#contain-fit-sizing).

<a id="ref-for-preferred-aspect-ratio②"></a>

<a id="ref-for-stretch-fit-size①"></a>

If the box has no [preferred aspect ratio](#preferred-aspect-ratio), applies [stretch-fit sizing](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

## <a id="ratios"></a>4.  Aspect Ratios

<a id="ref-for-natural-aspect-ratio"></a>

Images often have a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), which the CSS layout algorithms attempt to preserve as they resize the element.

<a id="ref-for-propdef-aspect-ratio"></a>

The [aspect-ratio](#propdef-aspect-ratio) property allows specifying this behavior for non-replaced elements, and for altering the effective aspect ratio of replaced elements.

<a id="ref-for-replaced-element"></a>

<a id="ref-for-preferred-aspect-ratio③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-cce77cb6"></a> We are still working through the details of this section. If there is any behavior specified here that would cause [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) with a [preferred aspect ratio](#preferred-aspect-ratio) to behave differently than they would under the requirements of the [CSS2](https://www.w3.org/TR/CSS2/visudet.html), [Flex Layout](https://www.w3.org/TR/css-flexbox-1/), and [Grid Layout](https://www.w3.org/TR/css-grid-1/) specs combined (without this specification in effect), <strong>this is an error and should be <a href="https://github.com/w3c/csswg-drafts/issues">reported</a> to the CSSWG</strong>.

<a id="ref-for-propdef-aspect-ratio①"></a>

### <a id="aspect-ratio"></a>4.1.  Preferred Aspect Ratios: the [aspect-ratio](#propdef-aspect-ratio) property



| Field               | Definition                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-aspect-ratio"></a>aspect-ratio                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-ratio-value"></a><a id="ref-for-comb-any"></a>auto [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                         |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-inline-box"></a>all elements except [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box) and internal ruby or table boxes                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword or a pair of numbers                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                            |



<a id="ref-for-valdef-width-auto"></a>

This property sets a <a id="preferred-aspect-ratio"></a>preferred aspect ratio for the box, which will be used in the calculation of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) sizes and some other layout functions.

<a id="valdef-aspect-ratio-auto"></a>auto

<a id="ref-for-content-box"></a>

<a id="ref-for-preferred-aspect-ratio④"></a>

<a id="ref-for-natural-aspect-ratio①"></a>

<a id="ref-for-replaced-element①"></a>

[Replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) use that aspect ratio; otherwise the box has no [preferred aspect ratio](#preferred-aspect-ratio). Size calculations involving the aspect ratio work with the [content box](https://www.w3.org/TR/css-box-4/#content-box) dimensions always.

<a id="ref-for-ratio-value①"></a>

<a id="valdef-aspect-ratio-ratio"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)

<a id="ref-for-propdef-box-sizing"></a>

<a id="ref-for-preferred-aspect-ratio⑤"></a>

The box’s [preferred aspect ratio](#preferred-aspect-ratio) is the specified ratio of width / height. Size calculations involving the aspect ratio work with the dimensions of the box specified by [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing).

<a id="ref-for-ratio-value②"></a>

<a id="ref-for-degenerate-ratio"></a>

<a id="ref-for-valdef-aspect-ratio-auto"></a>

If the [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) is [degenerate](https://www.w3.org/TR/css-values-4/#degenerate-ratio), the property instead behaves as [auto](#valdef-aspect-ratio-auto).

<a id="ref-for-ratio-value③"></a>

<a id="valdef-aspect-ratio-auto--ratio"></a>auto &#x26;&#x26; [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)

<a id="ref-for-content-box①"></a>

<a id="ref-for-natural-aspect-ratio②"></a>

<a id="ref-for-replaced-element②"></a>

<a id="ref-for-preferred-aspect-ratio⑥"></a>

<a id="ref-for-ratio-value④"></a>

<a id="ref-for-valdef-aspect-ratio-auto①"></a>

If both [auto](#valdef-aspect-ratio-auto) and a [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) are specified together, the [preferred aspect ratio](#preferred-aspect-ratio) is the specified ratio of width / height unless it is a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), in which case that aspect ratio is used instead. In all cases, size calculations involving the aspect ratio work with the [content box](https://www.w3.org/TR/css-box-4/#content-box) dimensions always.

<a id="ref-for-ratio-value⑤"></a>

<a id="ref-for-degenerate-ratio①"></a>

<a id="ref-for-valdef-aspect-ratio-auto②"></a>

If the [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) is [degenerate](https://www.w3.org/TR/css-values-4/#degenerate-ratio), the property instead behaves as [auto](#valdef-aspect-ratio-auto).

<a id="ref-for-preferred-aspect-ratio⑦"></a>

<a id="ref-for-replaced-element③"></a>

<a id="ref-for-non-replaced"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-propdef-justify-self"></a>

<a id="ref-for-valdef-justify-self-stretch"></a>

<a id="ref-for-valdef-self-position-start"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Having a [preferred aspect ratio](#preferred-aspect-ratio) does not make a box into a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element); layout rules specific to <a id="ref-for-replaced-element④"></a>replaced elements do not generally apply to [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) boxes with a <a id="ref-for-preferred-aspect-ratio⑧"></a>preferred aspect ratio. For example, a <a id="ref-for-non-replaced①"></a>non-replaced [absolutely-positioned](https://www.w3.org/TR/css-position-3/#absolute-position) box treats [justify-self: normal](https://www.w3.org/TR/css-align-3/#propdef-justify-self) as [stretch](https://www.w3.org/TR/css-align-3/#valdef-justify-self-stretch), not as [start](https://drafts.csswg.org/css-align-3/#valdef-self-position-start) ([CSS Box Alignment 3 §6.1.2 Absolutely-Positioned Boxes](https://www.w3.org/TR/css-align-3/#justify-abspos)), even if it has a <a id="ref-for-preferred-aspect-ratio⑨"></a>preferred aspect ratio

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ada2dbb9"></a> CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95a95149"></a> This example sets each item in the grid to render as a square, determining the number of items and their widths by the available space.
>
> ```text
> <ul>
>   <li>…
>   <li>…
>   <li>…
>   <li>…
> </ul>
> ```
>
> ```text
> ul {
>   display: grid;
>   grid-template-columns: repeat(auto-fill, minmax(12em, 1fr));
> }
> li {
>   aspect-ratio: 1/1;
>   overflow: auto;
> }
> ```
<a id="ref-for-the-iframe-element"></a>

<a id="ref-for-propdef-aspect-ratio②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2993b4a0"></a> This example uses the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element’s `width` and `height` attributes to set the [aspect-ratio](#propdef-aspect-ratio) property, giving the iframe an aspect ratio to use for sizing so that it behaves exactly like an image with that aspect ratio.
>
> ```text
> <iframe
>   src="https://www.youtube.com/embed/0Gr1XSyxZy0"
>   width=560
>   height=315>
> ```
>
> ```text
> @supports (aspect-ratio: attr(width number) / 1) {
>   iframe {
>     aspect-ratio: attr(width number) / attr(height number);
>     width: 100%;
>     height: auto;
>   }
> }
> ```
<a id="ref-for-natural-dimensions"></a>

<a id="ref-for-natural-width"></a>

<a id="ref-for-natural-height"></a>

<a id="ref-for-preferred-aspect-ratio①⓪"></a>

If a replaced element’s only [natural dimension](https://www.w3.org/TR/css-images-3/#natural-dimensions) is a [natural width](https://www.w3.org/TR/css-images-3/#natural-width) or a [natural height](https://www.w3.org/TR/css-images-3/#natural-height), giving it a [preferred aspect ratio](#preferred-aspect-ratio) also gives it an <a id="ref-for-natural-dimensions①"></a>natural height or width, whichever was missing, by transferring the existing size through the <a id="ref-for-preferred-aspect-ratio①①"></a>preferred aspect ratio.

### <a id="aspect-ratio-automatic"></a>4.2.  Effects of Preferred Aspect Ratio on Automatic Sizes

<a id="ref-for-preferred-aspect-ratio①②"></a>

<a id="ref-for-automatic-size"></a>

<a id="ref-for-replaced-element⑤"></a>

<a id="ref-for-natural-aspect-ratio③"></a>

<a id="ref-for-natural-size"></a>

<a id="ref-for-preferred-size"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-ratio-dependent-axis"></a>

When a box has a [preferred aspect ratio](#preferred-aspect-ratio), its [automatic sizes](https://www.w3.org/TR/css-sizing-3/#automatic-size) are calculated the same as for a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) and no [natural size](https://www.w3.org/TR/css-images-3/#natural-size) in that axis, see e.g. [CSS2 § 10](https://www.w3.org/TR/CSS2/visudet.html) and [CSS Flexible Box Model Level 1 § 9.2](https://www.w3.org/TR/css-flexbox-1/#algo-main-item). The axis in which the [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) calculation depends on this aspect ratio is called the <a id="ratio-dependent-axis"></a>ratio-dependent axis, and the resulting size is [definite](https://www.w3.org/TR/css-sizing-3/#definite) if its input sizes are also <a id="ref-for-definite①"></a>definite. The opposite axis (on which the [ratio-dependent axis](#ratio-dependent-axis) size depends) is the <a id="ratio-determining-axis"></a>ratio-determining axis.

<a id="ref-for-preferred-aspect-ratio①③"></a>

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-propdef-height②"></a>

<a id="ref-for-preferred-size①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [preferred aspect ratio](#preferred-aspect-ratio) only ever has an effect if at least one of the box’s sizes is [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size). If neither [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) nor [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) is an <a id="ref-for-automatic-size②"></a>automatic size, it can have no effect on its [preferred sizes](https://www.w3.org/TR/css-sizing-3/#preferred-size).

<a id="ref-for-preferred-size②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-36f34fa3"></a> When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio.

#### <a id="aspect-ratio-margin-collapse"></a>4.2.1.  Margin-collapsing

<a id="ref-for-block-axis"></a>

<a id="ref-for-ratio-dependent-axis①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-block-size①"></a>

<a id="ref-for-valdef-width-auto①"></a>

For the purpose of margin collapsing ([CSS 2 §8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)), if the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the [ratio-dependent axis](#ratio-dependent-axis), it is not considered to have a [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

### <a id="aspect-ratio-minimum"></a>4.3.  Automatic Content-based Minimum Sizes

<a id="ref-for-automatic-minimum-size"></a>

<a id="ref-for-ratio-dependent-axis②"></a>

<a id="ref-for-preferred-aspect-ratio①④"></a>

<a id="ref-for-replaced-element⑥"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-max-width"></a>

In order to avoid unintentional overflow, the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) in the [ratio-dependent axis](#ratio-dependent-axis) of a box with a [preferred aspect ratio](#preferred-aspect-ratio) that is neither a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) nor a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) is its [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) capped by its [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-514390d4"></a> In the following example, the box is as wide as the container (as usual), and its height is as tall as needed to contain its content but at least as tall as it is wide.
>
> ```text
> div {
>   aspect-ratio: 1/1;
>   /* 'width' and 'height' both default to 'auto' */
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~ |
> +----------+  +----------+  | ~~~~~~~~ |
>                             | ~~~~~~   |
>                             +----------+
> ```
>
> <a id="ref-for-propdef-overflow"></a>
>
> When [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is specified, however, even the box with excess content maintains the 1:1 aspect ratio (and handles overflow by becoming scrollable instead, as usual).
>
> ```text
> div {
>   overflow: auto;
>   aspect-ratio: 1/1;
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~^|
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~v|
> +----------+  +----------+  +----------+
> ```
>
> <a id="ref-for-propdef-min-height②"></a>
>
> Overriding the [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) property also maintains the 1:1 aspect ratio, but will result in content overflowing the box if it is not otherwise handled.
>
> ```text
> div {
>   aspect-ratio: 1/1;
>   min-height: 0;
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~ |
> +----------+  +----------+  +-~~~~~~~~-+
>                               ~~~~~~    
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63716969"></a> This automatic minimum operates in both axes. Consider this example:
>
> ```text
> <div style="height: 100px; aspect-ratio: 1/1;">
>   <span style="display: inline-block; width: 50px;"></span>
>   <span style="display: inline-block; width: 150px;"></span>
> </div>
> ```
>
> <a id="ref-for-propdef-width③"></a>
>
> <a id="ref-for-valdef-width-auto②"></a>
>
> <a id="ref-for-propdef-min-width②"></a>
>
> The [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) of the container, being [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), resolves through the aspect ratio to 100px. However, its [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), being <a id="ref-for-valdef-width-auto③"></a>auto, resolves to 150px. The resulting width of the container is thus 150px. To ignore the contents when sizing the container, <a id="ref-for-propdef-min-width③"></a>min-width: 0 can be specified.

### <a id="aspect-ratio-size-transfers"></a>4.4.  Min/Max Size Transfers

<a id="ref-for-preferred-aspect-ratio①⑤"></a>

<a id="ref-for-indefinite"></a>

<a id="ref-for-min-width"></a>

<a id="ref-for-max-width①"></a>

<a id="ref-for-preferred-size③"></a>

Sizing constraints in either axis (the <var>origin</var> axis) are transferred through the [preferred aspect ratio](#preferred-aspect-ratio) and applied to any [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) [minimum](https://www.w3.org/TR/css-sizing-3/#min-width), [maximum](https://www.w3.org/TR/css-sizing-3/#max-width), or [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) size in the other axis (the <var>destination</var> axis) as follows:

- <a id="ref-for-definite②"></a>

  <a id="ref-for-min-width①"></a>

  <a id="ref-for-preferred-size④"></a>

  <a id="ref-for-max-width②"></a>

  First, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) is converted and transferred from the <var>origin</var> to <var>destination</var> axis. This transferred minimum is capped by any <a id="ref-for-definite③"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the <var>destination</var> axis.

- <a id="ref-for-definite④"></a>

  <a id="ref-for-max-width③"></a>

  <a id="ref-for-preferred-size⑤"></a>

  <a id="ref-for-min-width②"></a>

  Then, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) is converted and transferred from the <var>origin</var> to <var>destination</var>. This transferred maximum is floored by any <a id="ref-for-definite⑤"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) in the <var>destination</var> axis as well as by the transferred minimum, if any.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Thus, any definite sizes are completely unaffected by a transferred constraint; and a transferred minimum will never cause an element to exceed a definite preferred/maximum size, nor will a transferred maximum cause an element to violate its preferred/minimum size.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis. (This is the principle that drove the contents of the [constraint table in CSS2 Section 10.4](https://www.w3.org/TR/CSS2/visudet.html#min-max-widths).)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b98c11d1"></a> In the following example:
>
> ```text
> <div id=container style="height: 100px; float: left;">
>   <div id=item style="height: 100%; aspect-ratio: 1/1;">content</div>
> </div>
> ```
>
> Since the height of the `#item` is a percentage that resolves against a definite container, the width of the item resolves to 100px for both its intrinsic size contributions as well as for final layout, so the container also sizes to a width of 100px.
>
> ```text
> <div id=container style="height: auto; float: left;">
>   <div id=item style="height: 100%; aspect-ratio: 1/1;">content</div>
> </div>
> ```
>
> <a id="ref-for-behave-as-auto"></a>
>
> <a id="ref-for-automatic-size③"></a>
>
> <a id="ref-for-ratio-dependent-axis③"></a>
>
> <a id="ref-for-intrinsic-size-contribution"></a>
>
> In this next example, the percentage height of the item cannot be resolved and [behaves as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto) (see [CSS 2 §10.5 Content height: the 'height' property](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)). Since both axes now have an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size), the height becomes the [ratio-dependent axis](#ratio-dependent-axis). Calculating the [intrinsic size contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution) of the box produces a width derived from its content, and a height calculated from that width and the aspect ratio, yielding a square box (and a container) sized to the width of the word “content”.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-0273742f"></a> This section might not be written correctly. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;6071&#x3E;](https://github.com/w3c/csswg-drafts/issues/6071)

## <a id="intrinsic"></a>5.  Intrinsic Size Determination

### <a id="intrinsic-sizes"></a>5.1.  Intrinsic Sizes

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9②"></a> [CSS Sizing 3 §5.1 Intrinsic Sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizes)

### <a id="intrinsic-size-override"></a>5.2.  Overriding Contained Intrinsic Sizes: the contain-intrinsic-\* properties



| Field               | Definition                                                                                                                                                                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain-intrinsic-width"></a>contain-intrinsic-width, <a id="propdef-contain-intrinsic-height"></a>contain-intrinsic-height, <a id="propdef-contain-intrinsic-block-size"></a>contain-intrinsic-block-size, <a id="propdef-contain-intrinsic-inline-size"></a>contain-intrinsic-inline-size                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-all"></a><a id="ref-for-length-value"></a><a id="ref-for-comb-one②"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-3/#length-value) <a id="ref-for-comb-one③"></a>\| auto [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) <a id="ref-for-length-value①"></a>\<length\> |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                    |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-size-containment"></a>elements with [size containment](https://www.w3.org/TR/css-contain-1/#size-containment)                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value②"></a>as specified, with [\<length\>](https://www.w3.org/TR/css-values-3/#length-value) values computed                                                                                                                                                                                                    |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                                                  |



<a id="ref-for-size-containment①"></a>

<a id="ref-for-explicit-intrinsic-inner-size"></a>

These properties allow elements with [size containment](https://www.w3.org/TR/css-contain-1/#size-containment) to specify an <a id="explicit-intrinsic-inner-size"></a>explicit intrinsic inner size, causing the box to size as if its in-flow content totals to a width and height matching the specified [explicit intrinsic inner size](#explicit-intrinsic-inner-size) (rather than sizing as if it were empty).

<a id="ref-for-explicit-intrinsic-inner-size①"></a>

<a id="ref-for-grid-container"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is not always equivalent to laying out as if the element had one child of the specified [explicit intrinsic inner size](#explicit-intrinsic-inner-size). For example, a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) with one child of the specified size would still size according to the specified grid, usually ending up with a larger content size than specified.

Values are defined as:

<a id="valdef-contain-intrinsic-width-none"></a>none

<a id="ref-for-explicit-intrinsic-inner-size②"></a>

The corresponding axis does not have an [explicit intrinsic inner size](#explicit-intrinsic-inner-size).

<a id="ref-for-length-value③"></a>

<a id="valdef-contain-intrinsic-width-length"></a>[\<length\>](https://www.w3.org/TR/css-values-3/#length-value)

<a id="ref-for-explicit-intrinsic-inner-size③"></a>

<a id="ref-for-length-value④"></a>

The corresponding axis has an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) of the specified [\<length\>](https://www.w3.org/TR/css-values-3/#length-value).

<a id="ref-for-length-value⑤"></a>

<a id="valdef-contain-intrinsic-width-auto--length"></a>auto &#x26;&#x26; [\<length\>](https://www.w3.org/TR/css-values-3/#length-value)

<a id="ref-for-explicit-intrinsic-inner-size④"></a>

<a id="ref-for-last-remembered"></a>

<a id="ref-for-length-value⑥"></a>

The corresponding axis has an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) of its [last remembered size](#last-remembered). If a <a id="ref-for-last-remembered①"></a>last remembered size does not yet exist, instead use the specified [\<length\>](https://www.w3.org/TR/css-values-3/#length-value).

<a id="ref-for-explicit-intrinsic-inner-size⑤"></a>

<a id="ref-for-size-containment②"></a>

If an element has an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) in an axis, then after laying out the element as normal for [size containment](https://www.w3.org/TR/css-contain-1/#size-containment), the size of the contents in that axis are instead treated as being the <a id="ref-for-explicit-intrinsic-inner-size⑥"></a>explicit intrinsic inner size instead of what was calculated in layout, and layout is performed again if necessary. (If it has an <a id="ref-for-explicit-intrinsic-inner-size⑦"></a>explicit intrinsic inner size in both axises, this implies the first layout can be skipped.)

<a id="ref-for-logical-property-group"></a>

These four properties are part of a [logical property group](https://www.w3.org/TR/css-logical-1/#logical-property-group).

<a id="ref-for-size-containment③"></a>

<a id="ref-for-propdef-height③"></a>

<a id="ref-for-explicit-intrinsic-inner-size⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An element with [size containment](https://www.w3.org/TR/css-contain-1/#size-containment) is laid out as if it had no contents [\[CSS-CONTAIN-1\]](#biblio-css-contain-1), which in many cases this will cause the element to collapse to zero inner height. This can be corrected with an explicit [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) chosen to show the expected contents, but that can have unintended effects in some layout systems, such as Flex and Grid Layout, which treat an explicit <a id="ref-for-propdef-height④"></a>height as a stronger command than an implicit content-based height. The element thus might lay out substantially differently than it would have were it simply filled with content up to that height. Providing an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) for the element preserves the performance benefits of ignoring its contents for layout while still allowing it to size as if it had content.



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain-intrinsic-size"></a>contain-intrinsic-size                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-comb-all①"></a><a id="ref-for-length-value⑦"></a><a id="ref-for-comb-one④"></a>\[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-3/#length-value) <a id="ref-for-comb-one⑤"></a>\| auto [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) <a id="ref-for-length-value⑧"></a>\<length\> \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>Applies to:&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                 |



<a id="ref-for-propdef-contain-intrinsic-size"></a>

<a id="ref-for-propdef-contain-intrinsic-width"></a>

<a id="ref-for-propdef-contain-intrinsic-height"></a>

[contain-intrinsic-size](#propdef-contain-intrinsic-size) is a shorthand property that sets the [contain-intrinsic-width](#propdef-contain-intrinsic-width) and [contain-intrinsic-height](#propdef-contain-intrinsic-height) properties.

<a id="ref-for-propdef-contain-intrinsic-width①"></a>

<a id="ref-for-propdef-contain-intrinsic-height①"></a>

The first value represents the [contain-intrinsic-width](#propdef-contain-intrinsic-width) value, and the second represents the [contain-intrinsic-height](#propdef-contain-intrinsic-height) value. If only one value is given, it applies to both properties.

#### <a id="last-remembered"></a>5.2.1.  Last Remembered Size

<a id="ref-for-size-containment④"></a>

[Size containment](https://www.w3.org/TR/css-contain-1/#size-containment) is very valuable for ensuring a page can render efficiently, restricting the scope of layout work that can happen as a result of an element changing its rendering. However, it’s also very restrictive for the author, requiring them to correctly predict what the size of the element will be; if this guess is incorrect, even slightly, it can cause unsightly scrollbars or accidentally-hidden content.

<a id="ref-for-propdef-contain-intrinsic-size①"></a>

<a id="ref-for-size-containment⑤"></a>

The [contain-intrinsic-size: auto](#propdef-contain-intrinsic-size) value allows a middle-ground: if an element is ever <em>not</em> [size-contained](https://www.w3.org/TR/css-contain-1/#size-containment), this value causes the element to remember its size (calculated as normal by layout); then, if the element gains <a id="ref-for-size-containment⑥"></a>size containment later, it will use the remembered size, offering the performance benefits of <a id="ref-for-size-containment⑦"></a>size containment while <em>probably</em> sizing accurately to its contents.

<a id="ref-for-last-remembered②"></a>

The [last remembered size](#last-remembered) of an element is determined by:

- <a id="ref-for-propdef-contain-intrinsic-size②"></a>

  <a id="ref-for-size-containment⑧"></a>

  <a id="ref-for-last-remembered③"></a>

  At the time that ResizeObserver events are determined and delivered, if an element has [contain-intrinsic-size: auto](#propdef-contain-intrinsic-size) but does not have [size containment](https://www.w3.org/TR/css-contain-1/#size-containment), record its current inner dimensions as its [last remembered size](#last-remembered).

<a id="ref-for-last-remembered④"></a>

<a id="ref-for-size-containment⑨"></a>

An element might not have a [last remembered size](#last-remembered), if it has never been rendered without [size containment](https://www.w3.org/TR/css-contain-1/#size-containment). (In this case, it will instead use the fallback value provided along with auto.)

<a id="ref-for-propdef-overflow①"></a>

#### <a id="cis-scrollbars"></a>5.2.2.  Interaction With [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow)

<a id="ref-for-propdef-contain-intrinsic-size③"></a>

<a id="ref-for-propdef-overflow②"></a>

The [contain-intrinsic-size](#propdef-contain-intrinsic-size) property provides an estimate of how large the author expects the content of an element to be, but this estimate is not actual content and does not represent anything that needs to be shown to the user. Therefore, an element with [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) must not generate scrollbars as a consequence of <a id="ref-for-propdef-contain-intrinsic-size④"></a>contain-intrinsic-size.

<a id="ref-for-propdef-contain-intrinsic-size⑤"></a>

However, if [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a size large enough that the element would generate scrollbars if it contained actual content of that size, then the element must be <em>sized</em> as if it generated those scrollbar(s) in accordance with such hypothetical content.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b57541eb"></a> In the following example code:
>
> ```text
> div {
>   width: max-content;
>   contain-intrinsic-size: 100px 100px;
>   overflow: auto;
> }
> ```
>
> <a id="ref-for-propdef-contain-intrinsic-size⑥"></a>
>
> The element ends up being 100px wide and 100px tall: [contain-intrinsic-size](#propdef-contain-intrinsic-size) provides the max-content width, and also the height.
>
> If the element then ended up with content that was 150px tall, it would show a vertical scrollbar; if the scrollbar is not overlay, it will take up some of that 100px width, leaving a smaller amount (roughly 84px, typically) for the content to flow into. (See [CSS Overflow 3 §3.2 Scrollbars and Layout](https://www.w3.org/TR/css-overflow-3/#scrollbar-layout).)
>
> <a id="ref-for-propdef-contain-intrinsic-size⑦"></a>
>
> Even though there’s now less than 100px of horizontal space available for the content, it will not generate a horizontal scrollbar just because [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a 100px width; that would only happen if the actual content had something unbreakable and wider than the remaining space.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-577a9f87"></a> In contrast, in the following example code:
>
> ```text
> div {
>   width: max-content;
>   contain-intrinsic-size: 100px 100px;
>   height: 50px;
>   overflow: auto;
> }
> ```
>
> <a id="ref-for-propdef-contain-intrinsic-size⑧"></a>
>
> The element has a fixed 50px height, but [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a 100px “estimated content height”. The element thus assumes that it will need a vertical scrollbar when it’s filled with actual content, resulting in a max-content width a little more than 100px (roughly 116px, typically), to accommodate the estimated 100px of max-content width from <a id="ref-for-propdef-contain-intrinsic-size⑨"></a>contain-intrinsic-size, and as well as the vertical scrollbar width (roughly 16px, typically).
>
> However, even though the element reserves space on the assumption of needing a scrollbar, it will not actually generate one unless the actual content overflows: if it ends up containing content that’s less than 50px tall, no vertical scrollbar will be generated at all, but the element will still be 116px wide.

### <a id="intrinsic-contribution"></a>5.3.  Intrinsic Size Contributions

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9③"></a> [CSS Sizing 3 §5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution)

<a id="ref-for-propdef-min-intrinsic-sizing"></a>

### <a id="intrinsic-contribution-override"></a>5.4.  Zeroing Min-Content Size Contributions: the [min-intrinsic-sizing](#propdef-min-intrinsic-sizing) property



| Field               | Definition                                                                                                                                                                            |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-min-intrinsic-sizing"></a>min-intrinsic-sizing                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any①"></a><a id="ref-for-comb-one⑥"></a>legacy [\|](https://www.w3.org/TR/css-values-4/#comb-one) zero-if-scroll [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) zero-if-extrinsic |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | legacy                                                                                                                                                                                |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-inline-box①"></a>all elements except [inline boxes](https://www.w3.org/TR/css-display-3/#inline-box)                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                          |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                              |



> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b731f1ac"></a> This property seriously needs some name bikeshedding.

<a id="ref-for-min-content-contribution"></a>

<a id="ref-for-non-replaced②"></a>

This property defines whether the [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of a [non-replaced](https://www.w3.org/TR/css-display-3/#non-replaced) box is “compressed” under certain circumstances. Values have the following meanings:

<a id="valdef-min-intrinsic-size-legacy"></a>legacy  
<a id="ref-for-min-content-contribution①"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is handled as normal.

<a id="valdef-min-intrinsic-size-zero-if-scroll"></a>zero-if-scroll  
<a id="ref-for-scroll-container①"></a>

<a id="ref-for-min-content-contribution②"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is “compressed” if it is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<a id="valdef-min-intrinsic-size-zero-if-extrinsic"></a>zero-if-extrinsic  
<a id="ref-for-max-width④"></a>

<a id="ref-for-preferred-size⑥"></a>

<a id="ref-for-extrinsic-sizing"></a>

<a id="ref-for-min-content-contribution③"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is “compressed” if has an [extrinsic](https://www.w3.org/TR/css-sizing-3/#extrinsic-sizing) [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum](https://www.w3.org/TR/css-sizing-3/#max-width) size.

<a id="ref-for-replaced-element⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the default behavior of most [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element).

<a id="ref-for-scroll-container②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3141cd7d"></a> The following rule will make all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) essentially ignore their contents when passing up their size contributions (unless they specifically requested a content-based size):
>
> ```text
> *, ::before, ::after { min-intrinsic-size: zero-if-scroll; }
> ```
>
> <a id="ref-for-scroll-container③"></a>
>
> <a id="ref-for-min-content①"></a>
>
> This prevents the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) from blowing up the size of its ancestors if it contains large items such as a table or long lines of unbreakable text. Meanwhile, it allows boxes that are not scroll containers to continue influencing the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) of their ancestors.

<a id="ref-for-valdef-min-intrinsic-size-zero-if-scroll"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The behavior of [zero-if-scroll](#valdef-min-intrinsic-size-zero-if-scroll) would have been a better default, but due to Web-compat, it cannot be the initial value. :(

<a id="ref-for-min-content-contribution④"></a>

<a id="ref-for-valdef-width-min-content①"></a>

<a id="ref-for-valdef-width-max-content①"></a>

<a id="ref-for-valdef-width-fit-content①"></a>

<a id="ref-for-sizing-property"></a>

The “compressed” [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is calculated by pretending the box were empty, except when factoring in sizing constraints imposed by explicit [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content), and [fit-content](#valdef-width-fit-content) values of the [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property).

## <a id="extrinsic"></a>6.  Extrinsic Size Determination

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9④"></a> [CSS Sizing 3 §4 Extrinsic Size Determination](https://www.w3.org/TR/css-sizing-3/#extrinsic)

### <a id="stretch-fit-sizing"></a>6.1.  Stretch-fit Sizing: filling the containing block

<a id="ref-for-containing-block①"></a>

Stretch-fit sizing tries to set the box’s used size to the length necessary to make its outer size as close to filling the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) as possible while still respecting the constraints imposed by min-height/min-width/max-height/max-width.

<a id="ref-for-automatic-size④"></a>

<a id="ref-for-self-alignment-properties"></a>

<a id="ref-for-valdef-width-stretch②"></a>

<a id="ref-for-alignment-container"></a>

Formally, its behavior is the same as specifying an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) together with a [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) value of [stretch](#valdef-width-stretch) (in the relevant axis), except that the resulting box, which can end up not exactly fitting its [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container), can be subsequently aligned by its actual <a id="ref-for-self-alignment-properties①"></a>self-alignment property value.

<a id="ref-for-formatting-context"></a>

<a id="ref-for-self-alignment-properties②"></a>

<a id="ref-for-stretch-fit-size②"></a>

<a id="ref-for-propdef-box-sizing①"></a>

<a id="ref-for-block-level-box"></a>

<a id="ref-for-margin"></a>

<a id="ref-for-sizing-property①"></a>

<a id="ref-for-initial-value"></a>

Additionally, in [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context) and axes in which the relevant [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) does not apply (such as the block axis in Block Layout, or the main axis in Flex Layout), in cases where a percentage size in that axis would resolve to a definite value, a [stretch-fit size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size) causes the box to attempt to fill its containing block—behaving as 100% but applying the resulting size to its margin box instead of the box indicated by [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing). For this purpose, auto margins are treated as zero, and furthermore, for [block-level boxes](https://www.w3.org/TR/css-display-3/#block-level-box) in particular, if its block-start/block-end [margin](https://www.w3.org/TR/css-box-4/#margin) would be adjoining to its parent’s block-start/block-end <a id="ref-for-margin①"></a>margin if its parent’s [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property) all had their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value), then its block-start/block-end <a id="ref-for-margin②"></a>margin is treated as zero.

<a id="ref-for-valdef-align-self-stretch"></a>

<a id="ref-for-automatic-size⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Consequently, if neither [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) alignment applies nor percentage sizing can resolve, then the box will resolve to its [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size).

<a id="ref-for-block-box"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-174ae518"></a> For example, given the following HTML representing two [block boxes](https://www.w3.org/TR/css-display-3/#block-box):
>
> ```text
> <div class="outer">
>   <div class="inner"></div>
> </div>
> ```
>
> <a id="ref-for-outer-size"></a>
>
> <a id="ref-for-inner-size"></a>
>
> In the following case, the [outer height](https://www.w3.org/TR/css-sizing-3/#outer-size) of the inner box will exactly match the height of the outer box (200px), but its [inner height](https://www.w3.org/TR/css-sizing-3/#inner-size) will be 20px less, to account for its margins.
>
> ```text
> .outer { height: 200px; border: solid; }
> .inner { height: stretch; margin: 10px; }
> ```
>
> <a id="ref-for-valdef-width-auto④"></a>
>
> In the following case, the height of the inner box will exactly match the height of the outer box (200px). The top margins will collapse, but the bottom margins do not collapse (because the bottom margin of a box is not adjoining to the bottom margin of a parent with a non-[auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) height, see [CSS 2 §8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)), and therefore the inner box’s bottom margin will be truncated.
>
> ```text
> .outer { height: 200px; margin: 0; }
> .inner { height: stretch; margin: 10px; }
> ```
<a id="ref-for-propdef-width④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6aa0dfe9"></a> Similarly, [width: stretch](https://www.w3.org/TR/css-sizing-3/#propdef-width) causes the box to fill its container, being 20px narrower than the width of "some more text" (due to the 10px margin):
>
> ```text
> <div class="outer">
>   <div class="inner">text</div>
> </div>
> some more text
> ```
>
> ```text
> .outer { float: left; margin: 0; }
> .inner { width: stretch; margin: 10px; }
> ```
<a id="ref-for-behave-as-auto①"></a>

<a id="ref-for-propdef-height⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3a83f468"></a> On the other hand, in this example the container’s height is indefinite, which would cause a percentage height on the child to [behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto), so [height: stretch](https://www.w3.org/TR/css-sizing-3/#propdef-height) <a id="ref-for-behave-as-auto②"></a>behaves as auto as well.
>
> ```text
> .outer { height: auto; margin: 0; }
> .inner { height: stretch; margin: 10px; }
> ```
### <a id="contain-fit-sizing"></a>6.2.  Contain-fit Sizing: stretching while maintaining an aspect ratio

<a id="ref-for-preferred-aspect-ratio①⑥"></a>

<a id="ref-for-valdef-object-fit-contain"></a>

<a id="ref-for-propdef-object-fit"></a>

<a id="ref-for-propdef-background-size"></a>

Contain-fit sizing essentially applies stretch-fit sizing, but reduces the size of the box in one axis to maintain the box’s [preferred aspect ratio](#preferred-aspect-ratio), similar to the [contain](https://www.w3.org/TR/css-images-3/#valdef-object-fit-contain) keyword of the [object-fit](https://www.w3.org/TR/css-images-3/#propdef-object-fit) and [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) properties.

First, a target rectangle is determined:

1.  <a id="ref-for-stretch-fit-size③"></a>

    The initial target rectangle is the size of the box’s containing block, with any indefinite size assumed as infinity. If both dimensions are indefinite, the initial target rectangle is set to match the outer edges of the box were it [stretch-fit sized](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

2.  <a id="ref-for-propdef-min-height③"></a>

    <a id="ref-for-propdef-min-width④"></a>

    <a id="ref-for-propdef-max-height②"></a>

    <a id="ref-for-propdef-max-width②"></a>

    <a id="ref-for-valdef-max-width-none"></a>

    Next, if the box has a non-[none](https://www.w3.org/TR/css-sizing-3/#valdef-max-width-none) [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) or [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), the target rectangle is clamped in the affected dimension to less than or equal to the “maximum size” of the box’s margin box, i.e. the size its margin box would be if the box was sized at its max-width/height. (Note that, consistent with normal [box-sizing rules](https://www.w3.org/TR/CSS2/visuren.html), this “maximum size” is floored by the effects of the box’s [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height).)

3.  <a id="ref-for-preferred-aspect-ratio①⑦"></a>

    Last, the target rectangle is reduced in one dimension by the minimum necessary for it to match the box’s [preferred aspect ratio](#preferred-aspect-ratio).

The contain-fit size in each dimension is the size that would result from stretch-fitting into the target rectangle.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6e598fc0"></a> Copy whatever stretch-fit ends up doing wrt margin collapsing.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9d61aaed"></a> If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to \#2 that applies the relevant minimums.

### <a id="percentage-sizing"></a>6.3.  Percentage Sizing

…

## <a id="changes"></a> Changes

### <a id="changes-2020-05"></a> Recent Changes

Significant changes since the [20 October 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-4-20201020/) include:

- <a id="ref-for-propdef-min-intrinsic-sizing①"></a>

  Drafted [min-intrinsic-sizing](#propdef-min-intrinsic-sizing) property, to better control the min-content contributions of scroll containers. ([Issue 1865](https://github.com/w3c/csswg-drafts/issues/1865), [Issue 4585](https://github.com/w3c/csswg-drafts/issues/4585))

- <a id="ref-for-propdef-contain-intrinsic-size①⓪"></a>

  Added longhands to [contain-intrinsic-size](#propdef-contain-intrinsic-size) for controlling each axis independently. ([Issue 5432](https://github.com/w3c/csswg-drafts/issues/5432))

- <a id="ref-for-propdef-contain-intrinsic-size①①"></a>

  Drafted auto value to [contain-intrinsic-size](#propdef-contain-intrinsic-size) to allow “remembering” the previously-calculated size. ([Issue 5668](https://github.com/w3c/csswg-drafts/issues/5668), [Issue 5815](https://github.com/w3c/csswg-drafts/issues/5815))

- <a id="ref-for-propdef-aspect-ratio③"></a>

  Defined handling of degenerate ratios in [aspect-ratio](#propdef-aspect-ratio). ([Issue 5557](https://github.com/w3c/csswg-drafts/issues/5557))

- <a id="ref-for-propdef-aspect-ratio④"></a>

  Defined how [aspect-ratio](#propdef-aspect-ratio) impacts a replaced element’s natural sizes. ([Issue 5306](https://github.com/w3c/csswg-drafts/issues/5306))

- Fixed some errors in the [§ 4.4 Min/Max Size Transfers](#aspect-ratio-size-transfers) section, aligning the behavior to not conflict with behavior defined by CSS2 / CSS Flex Layout / etc. ([Issue 6071](https://github.com/w3c/csswg-drafts/issues/6071))

Significant changes since the [26 May 2020 First Public Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-4-20200526/) include:

- <a id="ref-for-ratio-determining-axis"></a>

  Define [ratio-determining axis](#ratio-determining-axis) as a term.

- Define that min/max sizing constraints are transferred across an aspect-ratio, ([Issue 5257](https://github.com/w3c/csswg-drafts/issues/5257))

  > <a id="ref-for-preferred-aspect-ratio①⑧"></a>
  >
  > Additionally, sizing constraints in either axis (the <var>origin</var> axis) are transferred through the [preferred aspect ratio](#preferred-aspect-ratio) to the other axis (the <var>destination</var> axis) as follows:
  >
  > - <a id="ref-for-definite⑥"></a>
  >
  >   <a id="ref-for-min-width③"></a>
  >
  >   <a id="ref-for-preferred-size⑦"></a>
  >
  >   <a id="ref-for-max-width⑤"></a>
  >
  >   First, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) is converted and transferred from the <var>origin</var> to <var>destination</var> axis. This transferred minimum is capped by any <a id="ref-for-definite⑦"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the <var>destination</var> axis.
  >
  > - <a id="ref-for-definite⑧"></a>
  >
  >   <a id="ref-for-max-width⑥"></a>
  >
  >   <a id="ref-for-preferred-size⑧"></a>
  >
  >   <a id="ref-for-min-width④"></a>
  >
  >   Then, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) is converted and transferred from the <var>origin</var> to <var>destination</var>. This transferred maximum is floored by any <a id="ref-for-definite⑨"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) in the <var>destination</var> axis as well as by the transferred minimum, if any.
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note: The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis.

- <a id="ref-for-propdef-aspect-ratio⑤"></a>

  <a id="ref-for-replaced-element⑧"></a>

  <a id="ref-for-natural-size①"></a>

  Clarify that [aspect-ratio](#propdef-aspect-ratio) on a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) with only one [natural size](https://www.w3.org/TR/css-images-3/#natural-size) determines the other dimension. ([Issue 5306](https://github.com/w3c/csswg-drafts/issues/5306))

  > <a id="ref-for-preferred-aspect-ratio①⑨"></a>
  >
  > If a replaced element’s only natural dimension is a natural width or a natural height, giving it a [preferred aspect ratio](#preferred-aspect-ratio) also gives it a natural height or width, whichever was missing, by transferring the existing size through the <a id="ref-for-preferred-aspect-ratio②⓪"></a>preferred aspect ratio.

- <a id="ref-for-propdef-aspect-ratio⑥"></a>

  Define that [aspect-ratio](#propdef-aspect-ratio) inhibits margin self-collapsing ([Issue 5328](https://github.com/w3c/csswg-drafts/issues/5328))

  > <a id="ref-for-block-axis①"></a>
  >
  > <a id="ref-for-ratio-dependent-axis④"></a>
  >
  > <a id="ref-for-computed-value①"></a>
  >
  > <a id="ref-for-propdef-block-size②"></a>
  >
  > <a id="ref-for-valdef-width-auto⑤"></a>
  >
  > For the purpose of margin collapsing ([CSS 2 §8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)), if the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the [ratio-dependent axis](#ratio-dependent-axis), it is not considered to have a [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

### <a id="additions-L3"></a> Additions Since Level 3

- <a id="ref-for-valdef-width-contain①"></a>

  <a id="ref-for-valdef-width-fit-content②"></a>

  <a id="ref-for-valdef-width-stretch③"></a>

  Added [stretch](#valdef-width-stretch), [fit-content](#valdef-width-fit-content), and [contain](#valdef-width-contain) keywords for sizing properties.

- <a id="ref-for-propdef-aspect-ratio⑦"></a>

  Added [aspect-ratio](#propdef-aspect-ratio) property.

- <a id="ref-for-propdef-contain-intrinsic-size①②"></a>

  Added [contain-intrinsic-size](#propdef-contain-intrinsic-size) property.

## <a id="acknowledgments"></a> Acknowledgments

Special thanks go to Aaron Gustafson, L. David Baron for their contributions to this module.

## <a id="priv-sec"></a> Privacy and Security Considerations

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

- [aspect-ratio](#propdef-aspect-ratio), in §4.1
- [auto](#valdef-aspect-ratio-auto), in §4.1
- [auto &#x26;&#x26; \<length\>](#valdef-contain-intrinsic-width-auto--length), in §5.2
- [auto &#x26;&#x26; \<ratio\>](#valdef-aspect-ratio-auto--ratio), in §4.1
- [contain](#valdef-width-contain), in §3.2
- [contain-fit sizing](#contain-fit-sizing), in §6.1
- [contain-intrinsic-block-size](#propdef-contain-intrinsic-block-size), in §5.2
- [contain-intrinsic-height](#propdef-contain-intrinsic-height), in §5.2
- [contain-intrinsic-inline-size](#propdef-contain-intrinsic-inline-size), in §5.2
- [contain-intrinsic-size](#propdef-contain-intrinsic-size), in §5.2
- [contain-intrinsic-width](#propdef-contain-intrinsic-width), in §5.2
- [explicit intrinsic inner size](#explicit-intrinsic-inner-size), in §5.2
- [fit-content](#valdef-width-fit-content), in §3.2
- [last remembered size](#last-remembered), in §5.2
- [legacy](#valdef-min-intrinsic-size-legacy), in §5.4
- [\<length\>](#valdef-contain-intrinsic-width-length), in §5.2
- [min-intrinsic-sizing](#propdef-min-intrinsic-sizing), in §5.4
- [none](#valdef-contain-intrinsic-width-none), in §5.2
- [preferred aspect ratio](#preferred-aspect-ratio), in §4.1
- [\<ratio\>](#valdef-aspect-ratio-ratio), in §4.1
- [ratio-dependent axis](#ratio-dependent-axis), in §4.2
- [ratio-determining axis](#ratio-determining-axis), in §4.2
- [stretch](#valdef-width-stretch), in §3.2
- [zero-if-extrinsic](#valdef-min-intrinsic-size-zero-if-extrinsic), in §5.4
- [zero-if-scroll](#valdef-min-intrinsic-size-zero-if-scroll), in §5.4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-align-3\] defines the following terms:
  - <a id="term-for-alignment-container"></a>alignment container
  - <a id="term-for-propdef-justify-self"></a>justify-self
  - <a id="term-for-self-alignment-properties"></a>self-alignment properties
  - <a id="term-for-valdef-self-position-start"></a>start
  - <a id="term-for-valdef-justify-self-stretch"></a>stretch (for justify-self)
- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background-size"></a>background-size
- \[css-box-4\] defines the following terms:
  - <a id="term-for-content-box"></a>content box
  - <a id="term-for-margin"></a>margin
  - <a id="term-for-margin-box"></a>margin box
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-initial-value"></a>initial value
- \[CSS-CONTAIN-1\] defines the following terms:
  - <a id="term-for-size-containment"></a>size containment
- \[css-display-3\] defines the following terms:
  - <a id="term-for-block-box"></a>block box
  - <a id="term-for-block-level-box"></a>block-level box
  - <a id="term-for-containing-block"></a>containing block
  - <a id="term-for-formatting-context"></a>formatting context
  - <a id="term-for-inline-box"></a>inline box
  - <a id="term-for-non-replaced"></a>non-replaced
  - <a id="term-for-replaced-element"></a>replaced element
- \[css-grid-2\] defines the following terms:
  - <a id="term-for-grid-container"></a>grid container
- \[css-images-3\] defines the following terms:
  - <a id="term-for-valdef-object-fit-contain"></a>contain
  - <a id="term-for-natural-aspect-ratio"></a>natural aspect ratio
  - <a id="term-for-natural-dimensions"></a>natural dimension
  - <a id="term-for-natural-height"></a>natural height
  - <a id="term-for-natural-size"></a>natural size
  - <a id="term-for-natural-width"></a>natural width
  - <a id="term-for-propdef-object-fit"></a>object-fit
- \[css-logical-1\] defines the following terms:
  - <a id="term-for-propdef-block-size"></a>block-size
  - <a id="term-for-propdef-inline-size"></a>inline-size
  - <a id="term-for-logical-property-group"></a>logical property group
  - <a id="term-for-propdef-max-block-size"></a>max-block-size
  - <a id="term-for-propdef-max-inline-size"></a>max-inline-size
  - <a id="term-for-propdef-min-block-size"></a>min-block-size
  - <a id="term-for-propdef-min-inline-size"></a>min-inline-size
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-scroll-container"></a>scroll container
- \[css-position-3\] defines the following terms:
  - <a id="term-for-absolute-position"></a>absolutely-positioned
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-valdef-width-auto"></a>auto
  - <a id="term-for-automatic-minimum-size"></a>automatic minimum size
  - <a id="term-for-automatic-size"></a>automatic size
  - <a id="term-for-behave-as-auto"></a>behave as auto
  - <a id="term-for-behave-as-auto①"></a>behaves as auto
  - <a id="term-for-propdef-box-sizing"></a>box-sizing
  - <a id="term-for-definite"></a>definite
  - <a id="term-for-extrinsic-sizing"></a>extrinsic sizing
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-indefinite"></a>indefinite
  - <a id="term-for-inner-size"></a>inner height
  - <a id="term-for-intrinsic-size-contribution"></a>intrinsic size contribution
  - <a id="term-for-valdef-width-max-content"></a>max-content
  - <a id="term-for-max-width"></a>maximum size
  - <a id="term-for-valdef-width-min-content"></a>min-content
  - <a id="term-for-min-content-contribution"></a>min-content contribution
  - <a id="term-for-min-content"></a>min-content size
  - <a id="term-for-min-width"></a>minimum size
  - <a id="term-for-valdef-max-width-none"></a>none
  - <a id="term-for-outer-size"></a>outer height
  - <a id="term-for-preferred-size"></a>preferred size
  - <a id="term-for-sizing-property"></a>sizing property
  - <a id="term-for-stretch-fit-size"></a>stretch-fit size
  - <a id="term-for-propdef-width"></a>width
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="term-for-length-value"></a>\<length\>
- \[css-values-4\] defines the following terms:
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-ratio-value"></a>\<ratio\>
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-degenerate-ratio"></a>degenerate ratio
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-max-height"></a>max-height
  - <a id="term-for-propdef-max-width"></a>max-width
  - <a id="term-for-propdef-min-height"></a>min-height
  - <a id="term-for-propdef-min-width"></a>min-width
- \[CSS3COL\] defines the following terms:
  - <a id="term-for-propdef-column-width"></a>column-width
- \[HTML\] defines the following terms:
  - <a id="term-for-the-iframe-element"></a>iframe

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 22 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 19 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 22 December 2020. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 11 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3col"></a>\[CSS3COL\]  
Håkon Wium Lie; Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 12 February 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; et al. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 19 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                           | Initial                   | Applies to                                                        | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value                                |
|---------------------|---------------------------------------------------------------------------------|---------------------------|-------------------------------------------------------------------|---------------------------|---------------------------|---------------------------|-----------------|-----------------------------------------------|
| <strong><span><a id="ref-for-propdef-aspect-ratio⑧"></a></span><a href="#propdef-aspect-ratio">aspect-ratio</a>&#xA;      </strong> | auto \|\| \<ratio\>                                                             | auto                      | all elements except inline boxes and internal ruby or table boxes | no                        | n/a                       | by computed value         | per grammar     | specified keyword or a pair of numbers        |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-block-size"></a></span><a href="#propdef-contain-intrinsic-block-size">contain-intrinsic-block-size</a>&#xA;      </strong> | none \| \<length\> \| auto &#x26;&#x26; \<length\>            | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-height②"></a></span><a href="#propdef-contain-intrinsic-height">contain-intrinsic-height</a>&#xA;      </strong> | none \| \<length\> \| auto &#x26;&#x26; \<length\>            | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-inline-size"></a></span><a href="#propdef-contain-intrinsic-inline-size">contain-intrinsic-inline-size</a>&#xA;      </strong> | none \| \<length\> \| auto &#x26;&#x26; \<length\>            | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-size①③"></a></span><a href="#propdef-contain-intrinsic-size">contain-intrinsic-size</a>&#xA;      </strong> | \[ none \| \<length\> \| auto &#x26;&#x26; \<length\> \]{1,2} | see individual properties | see individual properties                                         | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                     |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-width②"></a></span><a href="#propdef-contain-intrinsic-width">contain-intrinsic-width</a>&#xA;      </strong> | none \| \<length\> \| auto &#x26;&#x26; \<length\>            | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed |
| <strong><span><a id="ref-for-propdef-min-intrinsic-sizing②"></a></span><a href="#propdef-min-intrinsic-sizing">min-intrinsic-sizing</a>&#xA;      </strong> | legacy \| zero-if-scroll \|\| zero-if-extrinsic                                 | legacy                    | all elements except inline boxes                                  | no                        | n/a                       | discrete                  | per grammar     | as specified                                  |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is a diff spec over [CSS Sizing Level 3](https://www.w3.org/TR/css-sizing-3/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR. [↵](#issue-f7037e82)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 §2 Terminology](https://www.w3.org/TR/css-sizing-3/#terms) [↵](#issue-d41d8cd9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 §3 Specifying Box Sizes](https://www.w3.org/TR/css-sizing-3/#specifying-sizes) [↵](#issue-d41d8cd9%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add shorthands. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;820&#x3E;](https://github.com/w3c/csswg-drafts/issues/820) [↵](#issue-0e53565a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We are still working through the details of this section. If there is any behavior specified here that would cause [replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) with a [preferred aspect ratio](#preferred-aspect-ratio) to behave differently than they would under the requirements of the [CSS2](https://www.w3.org/TR/CSS2/visudet.html), [Flex Layout](https://www.w3.org/TR/css-flexbox-1/), and [Grid Layout](https://www.w3.org/TR/css-grid-1/) specs combined (without this specification in effect), <strong>this is an error and should be <a href="https://github.com/w3c/csswg-drafts/issues">reported</a> to the CSSWG</strong>. [↵](#issue-cce77cb6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here. [↵](#issue-ada2dbb9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio. [↵](#issue-36f34fa3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section might not be written correctly. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;6071&#x3E;](https://github.com/w3c/csswg-drafts/issues/6071) [↵](#issue-0273742f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 §5.1 Intrinsic Sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizes) [↵](#issue-d41d8cd9%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 §5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution) [↵](#issue-d41d8cd9%E2%91%A2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This property seriously needs some name bikeshedding. [↵](#issue-b731f1ac)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 §4 Extrinsic Size Determination](https://www.w3.org/TR/css-sizing-3/#extrinsic) [↵](#issue-d41d8cd9%E2%91%A3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Copy whatever stretch-fit ends up doing wrt margin collapsing. [↵](#issue-6e598fc0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to \#2 that applies the relevant minimums. [↵](#issue-9d61aaed)
