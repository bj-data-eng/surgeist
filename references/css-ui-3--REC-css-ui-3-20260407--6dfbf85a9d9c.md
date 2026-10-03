Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Basic User Interface Module Level 3 (CSS3 UI)

Source snapshot: https://www.w3.org/TR/2026/REC-css-ui-3-20260407/

Snapshot SHA-256: 6dfbf85a9d9c6d682d9c0621872021e8a57216737acacd5dee31c2936d7da67e

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 14 source tables are presented as readable Markdown tables or explicit labeled layouts: 12 ordinary table conversions, 2 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Live HTML/CSS demonstrations are represented by static source code and text, not equivalent browser appearance. Incidental whitespace in sample-display elements may collapse as in HTML; exact source markup is retained, and true preformatted/code blocks stay literal.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Basic User Interface Module Level 3 (CSS3 UI)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This specification describes user interface related properties and values that are proposed for CSS level 3 to style HTML and XML (including XHTML). It includes and extends user interface related features from the properties and values of CSS level 2 revision 1. It uses various properties and values to style basic user interface elements in a document.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a Recommendation using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). It includes [candidate corrections.](https://www.w3.org/policies/process/20250818/#candidate-correction).

A W3C Recommendation is a specification that, after extensive consensus-building, is endorsed by W3C and its Members, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/policies/patent-policy/#sec-Requirements) for implementations.

Candidate corrections are marked in the document.

W3C recommends the wide deployment of this specification as a standard for the Web.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-ui” in the title, like this: “\[css-ui\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-ui%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

This module describes CSS properties which enable authors to style user interface related properties and values.

[Section 2.1 of CSS1](https://www.w3.org/TR/REC-CSS1#anchor-pseudo-classes) [\[CSS1\]](#biblio-css1) and [Chapter 18 of CSS2](https://www.w3.org/TR/CSS2/ui.html) [\[CSS2\]](#biblio-css2) introduced several user interface related properties and values. [User Interface for CSS3 (16 February 2000)](https://www.w3.org/TR/2000/WD-css3-userint-20000216) introduced several new user interface related features.

This specification incorporates, extends, and supersedes them.

### <a id="purpose"></a>1.1. Purpose

The purpose of this specification is to achieve the following objectives:

- Extend the user interface features in CSS2.1.
- Provide additional CSS mechanisms to augment or replace other dynamic presentation related features in HTML.

## <a id="interaction"></a>2. Module Interactions

This document defines new features not present in earlier specifications. In addition, it replaces and supersedes the following:

- [Section 18.1](https://www.w3.org/TR/CSS2/ui.html#cursor-props), [section 18.4](https://www.w3.org/TR/CSS2/ui.html#dynamic-outlines), and Information on the stacking of outlines defined in [Appendix E](https://www.w3.org/TR/CSS2/zindex.html) of Cascading Style Sheets, level 2, revision 1 [\[CSS2\]](#biblio-css2)
- [User Interface for CSS3 (16 February 2000)](https://www.w3.org/TR/2000/WD-css3-userint-20000216)

### <a id="values"></a>2.1. Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-animation-type"></a>

<a id="ref-for-computed-value"></a>

<a id="c3"></a> Candidate Correction 3: [\[CSS-CASCADE-4\]](#biblio-css-cascade-4) and [\[WEB-ANIMATIONS-1\]](#biblio-web-animations-1) have refined how to define properties, with precise definitions for the concepts underlying [animation type](https://www.w3.org/TR/web-animations-1/#animation-type) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value). The definitions of properties throughout this specification are updated to use this terminology. In many cases, this is arguably editorial, but in some it defines previously unclear or ambiguous behavior.

## <a id="box-model"></a>3. Box Model addition

<a id="ref-for-propdef-box-sizing"></a>

### <a id="box-sizing"></a>3.1. Changing the Box Model: the [box-sizing](#propdef-box-sizing) property

| Field               | Definition                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-sizing"></a>box-sizing                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>content-box [\|](https://www.w3.org/TR/css-values-4/#comb-one) border-box |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | content-box                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements that accept width or height                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <u>discrete</u>                                                                              |

<a id="valdef-box-sizing-content-box"></a>content-box  
This is the behavior of width and height as specified by CSS2.1. The specified width and height (and respective min/max properties) apply to the width and height respectively of the content box of the element. The padding and border of the element are laid out and drawn outside the specified width and height.

<a id="valdef-box-sizing-border-box"></a>border-box  
<a id="4dfcfd3c0"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-width"></a>

Length and percentages values for width and height (and respective min/max properties) on this element determine the border box of the element. That is, any padding or border specified on the element is laid out and drawn inside this specified width and height. The content width and height are calculated by subtracting the border and padding widths of the respective sides from the specified [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) and [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height) properties. As the content width and height [cannot be negative](https://www.w3.org/TR/CSS2/visudet.html#the-width-property) ([\[CSS2\]](#biblio-css2), section 10.2), this computation is floored at 0. Used values, as exposed for instance through getComputedStyle(), also refer to the border box.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the behavior of width and height as commonly implemented by legacy HTML user agents for replaced elements and input elements.

<a id="ref-for-the-width-property"></a>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-propdef-box-sizing①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In contrast to the length and percentage values, the [auto](https://www.w3.org/TR/CSS2/visudet.html#the-width-property) value of the [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) and [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height) properties (as well as other keyword values introduced by later specifications, unless otherwise specified) is not influenced by the [box-sizing](#propdef-box-sizing) property, and always sets the size of the content box.

<a id="ref-for-propdef-box-sizing②"></a>

The following terms, whose definitions vary based on the computed value of [box-sizing](#propdef-box-sizing) are introduced:

|                     | <a id="ref-for-propdef-box-sizing③"></a>[box-sizing: content-box](#propdef-box-sizing)                           | <a id="ref-for-propdef-box-sizing④"></a>[box-sizing: border-box](#propdef-box-sizing)                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|---------------------|---------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><dfn><span><a id="min-inner-width"></a></span>min inner width</dfn>&#xA;      </strong> | <a id="ref-for-propdef-min-width"></a>[min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)   | <a id="ref-for-propdef-border-right-width"></a><a id="ref-for-propdef-border-left-width"></a><a id="ref-for-propdef-padding-right"></a><a id="ref-for-propdef-padding-left"></a><a id="ref-for-propdef-min-width①"></a>max(0, [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) − [padding-left](https://www.w3.org/TR/CSS2/box.html#propdef-padding-left) − [padding-right](https://www.w3.org/TR/CSS2/box.html#propdef-padding-right) − [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width) − [border-right-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width))   |
| <strong><dfn><span><a id="max-inner-width"></a></span>max inner width</dfn>&#xA;      </strong> | <a id="ref-for-propdef-max-width"></a>[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width)   | <a id="ref-for-propdef-border-right-width①"></a><a id="ref-for-propdef-border-left-width①"></a><a id="ref-for-propdef-padding-right①"></a><a id="ref-for-propdef-padding-left①"></a><a id="ref-for-propdef-max-width①"></a>max(0, [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) − [padding-left](https://www.w3.org/TR/CSS2/box.html#propdef-padding-left) − [padding-right](https://www.w3.org/TR/CSS2/box.html#propdef-padding-right) − [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width) − [border-right-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width))   |
| <strong><dfn><span><a id="min-inner-height"></a></span>min inner height</dfn>&#xA;      </strong> | <a id="ref-for-propdef-min-height"></a>[min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) | <a id="ref-for-propdef-border-bottom-width"></a><a id="ref-for-propdef-border-top-width"></a><a id="ref-for-propdef-padding-bottom"></a><a id="ref-for-propdef-padding-top"></a><a id="ref-for-propdef-min-height①"></a>max(0, [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) − [padding-top](https://www.w3.org/TR/CSS2/box.html#propdef-padding-top) − [padding-bottom](https://www.w3.org/TR/CSS2/box.html#propdef-padding-bottom) − [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width) − [border-bottom-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width)) |
| <strong><dfn><span><a id="max-inner-height"></a></span>max inner height</dfn>&#xA;      </strong> | <a id="ref-for-propdef-max-height"></a>[max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) | <a id="ref-for-propdef-border-bottom-width①"></a><a id="ref-for-propdef-border-top-width①"></a><a id="ref-for-propdef-padding-bottom①"></a><a id="ref-for-propdef-padding-top①"></a><a id="ref-for-propdef-max-height①"></a>max(0, [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) − [padding-top](https://www.w3.org/TR/CSS2/box.html#propdef-padding-top) − [padding-bottom](https://www.w3.org/TR/CSS2/box.html#propdef-padding-bottom) − [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width) − [border-bottom-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width)) |

<a id="ref-for-propdef-box-sizing⑤"></a>

The [Visual formatting model details](https://www.w3.org/TR/CSS2/visudet.html) of [\[CSS2\]](#biblio-css2) are written assuming [box-sizing: content-box](#propdef-box-sizing). The following disambiguations are made to clarify the behavior for all values of <a id="ref-for-propdef-box-sizing⑥"></a>box-sizing:

1.  <a id="ref-for-propdef-padding-left②"></a>

    <a id="ref-for-propdef-border-left-width②"></a>

    <a id="ref-for-the-width-property①"></a>

    <a id="ref-for-propdef-width②"></a>

    <a id="ref-for-content-width"></a>

    In [10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth), the second “width” in the following phrase is to be interpreted as [content width](https://drafts.csswg.org/css2/#content-width): “If [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) is not [auto](https://www.w3.org/TR/CSS2/visudet.html#the-width-property) and [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width) + [padding-left](https://www.w3.org/TR/CSS2/box.html#propdef-padding-left) + <a id="ref-for-propdef-width③"></a>width + \[...\]”

2.  <a id="ref-for-propdef-width④"></a>

    <a id="ref-for-propdef-padding-left③"></a>

    <a id="ref-for-propdef-border-left-width③"></a>

    <a id="ref-for-propdef-margin-left"></a>

    <a id="ref-for-position-props"></a>

    <a id="ref-for-content-width①"></a>

    In [10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width), “width” is to be interpreted as [content width](https://drafts.csswg.org/css2/#content-width) in the following equation: “[left](https://www.w3.org/TR/CSS2/visuren.html#position-props) + [margin-left](https://www.w3.org/TR/CSS2/box.html#propdef-margin-left) + [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width) + [padding-left](https://www.w3.org/TR/CSS2/box.html#propdef-padding-left) + [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) + \[...\]”

3.  <a id="ref-for-max-inner-height"></a>

    <a id="ref-for-min-inner-height"></a>

    <a id="ref-for-max-inner-width"></a>

    <a id="ref-for-min-inner-width"></a>

    <a id="ref-for-content-height"></a>

    <a id="ref-for-content-width②"></a>

    In [10.4](https://www.w3.org/TR/CSS2/visudet.html#min-max-widths), “width”, “height”, “min-width”, “max-width”, “min-height” and “max-height” are respectively to be interpreted as [content width](https://drafts.csswg.org/css2/#content-width), [content height](https://drafts.csswg.org/css2/#content-height), [min inner width](#min-inner-width), [max inner width](#max-inner-width), [min inner height](#min-inner-height) and [max inner height](#max-inner-height) in the following phrases:

    1.  “The tentative used width is calculated \[...\]”

    2.  <a id="ref-for-propdef-width⑤"></a>

        <a id="ref-for-propdef-max-width②"></a>

        “If the tentative used width is greater than [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), the rules above are applied again, but this time using the computed value of <a id="ref-for-propdef-max-width③"></a>max-width as the computed value for [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width).”

    3.  <a id="ref-for-propdef-width⑥"></a>

        <a id="ref-for-propdef-min-width②"></a>

        “If the resulting width is smaller than [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), the rules above are applied again, but this time using the value of <a id="ref-for-propdef-min-width③"></a>min-width as the computed value for [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width).”

    4.  “Select from the table the resolved height and width values for the appropriate constraint violation. Take the max-width and max-height as max(min, max) so that min ≤ max holds true. In this table w and h stand for the results of the width and height computations \[...\]”

    5.  All instances of these words in the table

    6.  <a id="ref-for-propdef-width⑦"></a>

        “Then apply the rules under "Calculating widths and margins" above, as if [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) were computed as this value.”

4.  <a id="ref-for-propdef-height②"></a>

    <a id="ref-for-propdef-padding-top②"></a>

    <a id="ref-for-propdef-border-top-width②"></a>

    <a id="ref-for-propdef-margin-top"></a>

    <a id="ref-for-position-props①"></a>

    <a id="ref-for-content-height①"></a>

    In [10.6.4](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height), “height” is to be interpreted as [content height](https://drafts.csswg.org/css2/#content-height) in the following equation: “[top](https://www.w3.org/TR/CSS2/visuren.html#position-props) + [margin-top](https://www.w3.org/TR/CSS2/box.html#propdef-margin-top) + [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width) + [padding-top](https://www.w3.org/TR/CSS2/box.html#propdef-padding-top) + [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height) + \[...\]”

5.  <a id="ref-for-max-inner-height①"></a>

    <a id="ref-for-min-inner-height①"></a>

    <a id="ref-for-content-height②"></a>

    <a id="ref-for-content-width③"></a>

    In [10.7](https://www.w3.org/TR/CSS2/visudet.html#min-max-heights), “width”, “height”, “min-height” and “max-height” are respectively to be interpreted as [content width](https://drafts.csswg.org/css2/#content-width), [content height](https://drafts.csswg.org/css2/#content-height), [min inner height](#min-inner-height) and [max inner height](#max-inner-height) in the following phrases:

    1.  “The tentative used height is calculated \[...\]”

    2.  <a id="ref-for-propdef-height③"></a>

        <a id="ref-for-propdef-max-height②"></a>

        “If this tentative height is greater than [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), the rules above are applied again, but this time using the value of <a id="ref-for-propdef-max-height③"></a>max-height as the computed value for [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height).”

    3.  <a id="ref-for-propdef-height④"></a>

        <a id="ref-for-propdef-min-height②"></a>

        “If the resulting height is smaller than [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), the rules above are applied again, but this time using the value of <a id="ref-for-propdef-min-height③"></a>min-height as the computed value for [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height).”

    4.  “\[...\] use the algorithm under Minimum and maximum widths above to find the used width and height. Then apply the rules under "Computing heights and margins" above, using the resulting width and height as if they were the computed values.”

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7c24a6b2"></a>
>
> Example(s):
>
> #### <a id="box-sizing-example"></a>Using box-sizing to evenly share space
>
> This example uses box-sizing to evenly horizontally split two divs with fixed size borders inside a div container, which would otherwise require additional markup.
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
> <a id="ref-for-propdef-box-sizing⑦"></a>
>
> This div should occupy the left half.
>
> This div should occupy the right half.
>
> The two divs above should appear side by side, each (including borders) 50% of the content width of their container. If instead they are stacked one on top of the other then your browser does not support [box-sizing](#propdef-box-sizing).

## <a id="outline-props"></a>4. Outline properties

At times, style sheet authors may want to create outlines around visual objects such as buttons, active form fields, image maps, etc., to make them stand out. Outlines differ from borders in the following ways:

1.  Outlines do not take up space.
2.  Outlines may be non-rectangular.
3.  UAs often render outlines on elements in the :focus state.

The outline properties control the style of these dynamic outlines.

The stacking of the rendering of these outlines is explicitly left up to implementations to provide a better user experience per platform. This supersedes the stacking of outlines as defined in [Appendix E of CSS 2.1](https://www.w3.org/TR/CSS2/zindex.html) [\[CSS2\]](#biblio-css2).

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong>
Keyboard users,
in particular people with disabilities
who may not be able to interact with the page in any other fashion,
depend on the outline being visible
on elements in the :focus state,
thus authors must not make the outline invisible on such elements
without making sure an alternative highlighting mechanism is provided.
</strong>

The rendering of applying transforms to outlines is left explicitly undefined in CSS3-UI.

<a id="ref-for-propdef-outline"></a>

### <a id="outline"></a>4.1. Outlines Shorthand: the [outline](#propdef-outline) property

| Field               | Definition                                                                                                                                                                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-outline"></a>outline                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-outline-width"></a><a id="ref-for-propdef-outline-style"></a><a id="ref-for-comb-any"></a><a id="ref-for-propdef-outline-color"></a>\[ [\<'outline-color'\>](#propdef-outline-color) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'outline-style'\>](#propdef-outline-style) <a id="ref-for-comb-any①"></a>\|\| [\<'outline-width'\>](#propdef-outline-width) \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                              |

<a id="ref-for-propdef-outline-width①"></a>

### <a id="outline-width"></a>4.2. Outline Thickness: the [outline-width](#propdef-outline-width) property

<a id="ref-for-snap-a-length-as-a-border-width"></a>

<a id="ref-for-propdef-outline-width②"></a>

<a id="c2"></a> Candidate Correction 2: [\[CSS-VALUES-4\]](#biblio-css-values-4) has introduced the concept of [snapping as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width) as a type of rounding applied to some lengths to ensure reasonable visual display. Define [outline-width](#propdef-outline-width) to use this concept, for consistency with other border-like things.

<a id="ref-for-propdef-outline-width③"></a>

<a id="ref-for-propdef-outline-style①"></a>

<a id="ref-for-propdef-column-rule-width"></a>

<a id="ref-for-propdef-border-width"></a>

<a id="c4"></a> Candidate Correction 4: Remove the special-case where [outline-width](#propdef-outline-width) computes to 0 based on [outline-style: none](#propdef-outline-style), keeping things consistent with a similar change being applied to [column-rule-width](https://www.w3.org/TR/css-gaps-1/#propdef-column-rule-width) and [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width). (See [Issue 11494](https://github.com/w3c/csswg-drafts/issues/11494).)

| Field               | Definition                                                                                                                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-outline-width"></a>outline-width                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-line-width"></a>[\<line-width\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-line-width)                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-line-style-none"></a><a id="ref-for-snap-a-length-as-a-border-width①"></a> absolute length <u>, [snapped as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width)</u> ~~; 0 if the outline style is [none](https://www.w3.org/TR/css-backgrounds-3/#valdef-line-style-none).~~ |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | ~~length~~ <u>by computed value</u>                                                                                                                                                                                                                                       |

<a id="ref-for-propdef-outline-style②"></a>

### <a id="outline-style"></a>4.3. Outline Patterns: the [outline-style](#propdef-outline-style) property

| Field               | Definition                                                                                                                                                                      |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-outline-style"></a>outline-style                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-style"></a><a id="ref-for-comb-one①"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) \<[border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style)\> |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <u>discrete</u>                                                                                                                                                                 |

<a id="ref-for-propdef-outline-color①"></a>

### <a id="outline-color"></a>4.4. Outline Colors: the [outline-color](#propdef-outline-color) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-outline-color"></a>outline-color                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②"></a><a id="ref-for-typedef-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) [\|](https://www.w3.org/TR/css-values-4/#comb-one) invert                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | invert                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color①"></a><a id="ref-for-propdef-color"></a><a id="ref-for-valdef-color-currentcolor"></a><a id="ref-for-valdef-outline-color-invert"></a>The computed value for [invert](#valdef-outline-color-invert) is <a id="ref-for-valdef-outline-color-invert①"></a>invert; the computed value of currentColor is currentColor (See [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor)); see the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property for other [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) values. |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | ~~color~~ <u>by computed value</u>                                                                                                                                                                                                                                                                                                                                                                                                                                        |

The outline created with the outline properties is drawn "over" a box, i.e., the outline is always on top, and doesn’t influence the position or size of the box, or of any other boxes. Therefore, displaying or suppressing outlines does not cause reflow.

Outlines may be non-rectangular. For example, if the element is broken across several lines, the outline should be an outline or minimum set of outlines that encloses all the element’s boxes.

Each part of the outline should be fully connected rather than open on some sides (as borders on inline elements are when lines are broken).

<a id="ref-for-border-edge"></a>

<a id="ref-for-propdef-border-radius"></a>

The parts of the outline are not required to be rectangular. To the extent that the outline follows the [border edge](https://www.w3.org/TR/CSS2/box.html#border-edge), it should follow the [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) curve.

The position of the outline may be affected by descendant boxes.

User agents should use an algorithm for determining the outline that encloses a region appropriate for conveying the concept of focus to the user.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define the exact position or shape of the outline, but it is typically drawn immediately outside the border box.

<a id="ref-for-propdef-outline-width④"></a>

<a id="ref-for-propdef-border-width①"></a>

The [outline-width](#propdef-outline-width) property accepts the same values as [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width) ([CSS Backgrounds 3 § 3.3 Line Thickness: the border-width properties](https://www.w3.org/TR/css-backgrounds-3/#border-width)).

<a id="ref-for-propdef-outline-style③"></a>

<a id="ref-for-propdef-border-style①"></a>

<a id="ref-for-propdef-outline-color②"></a>

The [outline-style](#propdef-outline-style) property accepts the same values as [border-style](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-style) ([CSS Backgrounds 3 § 3.2 Line Patterns: the border-style properties](https://www.w3.org/TR/css-backgrounds-3/#border-style)), except that hidden is not a legal outline style. In addition, in CSS3, <a id="ref-for-propdef-outline-style④"></a>outline-style accepts the value auto. The auto value permits the user agent to render a custom outline style, typically a style which is either a user interface default for the platform, or perhaps a style that is richer than can be described in detail in CSS, e.g. a rounded edge outline with semi-translucent outer pixels that appears to glow. As such, this specification does not define how the [outline-color](#propdef-outline-color) is incorporated or used (if at all) when rendering auto style outlines. User agents may treat auto as solid.

<a id="ref-for-propdef-outline-color③"></a>

<a id="ref-for-valdef-outline-color-invert②"></a>

The [outline-color](#propdef-outline-color) property accepts all colors, as well as the keyword <a id="valdef-outline-color-invert"></a>invert. [Invert](#valdef-outline-color-invert) is expected to perform a color inversion on the pixels on the screen. This is a common trick to ensure the focus border is visible, regardless of color background.

<a id="ref-for-valdef-outline-color-invert③"></a>

Conformant UAs may ignore the [invert](#valdef-outline-color-invert) value on platforms that do not support color inversion of the pixels on the screen.

<a id="ref-for-valdef-outline-color-invert④"></a>

<a id="ref-for-propdef-outline-color④"></a>

If the UA does not support the [invert](#valdef-outline-color-invert) value then it must reject that value at parse-time, and the initial value of the [outline-color](#propdef-outline-color) property is the currentColor keyword.

<a id="ref-for-propdef-outline①"></a>

<a id="ref-for-propdef-outline-style⑤"></a>

<a id="ref-for-propdef-outline-width⑤"></a>

<a id="ref-for-propdef-outline-color⑤"></a>

The [outline](#propdef-outline) property is a shorthand property, and sets all three of [outline-style](#propdef-outline-style), [outline-width](#propdef-outline-width), and [outline-color](#propdef-outline-color).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The outline is the same on all sides. In contrast to borders, there are no outline-top or outline-left etc. properties.

This specification does not define how multiple overlapping outlines are drawn, or how outlines are drawn for boxes that are partially obscured behind other elements.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d84e3271"></a>
>
> Example(s):
>
> Here’s an example of drawing a thick outline around a BUTTON element:
>
> ```text
> button { outline: thick solid }
> ```
Graphical user interfaces may use outlines around elements to tell the user which element on the page has the focus. These outlines are in addition to any borders, and switching outlines on and off should not cause the document to reflow. The focus is the subject of user interaction in a document (e.g. for entering text or selecting a button).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c74f9dfd"></a>
>
> Example(s):
>
> For example, to draw a thick black line around an element when it has the focus, and a thick red line when it is active, the following rules can be used:
>
> ```text
> :focus  { outline: thick solid black }
> :active { outline: thick solid red }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since the outline does not affect formatting (i.e., no space is left for it in the box model), it may well overlap other elements on the page.

<a id="ref-for-propdef-outline-offset"></a>

### <a id="outline-offset"></a>4.5. Offsetting the Outline: the [outline-offset](#propdef-outline-offset) property

<a id="ref-for-border-edge①"></a>

By default, the outline is drawn starting just outside the [border edge](https://www.w3.org/TR/CSS2/box.html#border-edge). However, it is possible to offset the outline and draw it beyond the <a id="ref-for-border-edge②"></a>border edge.

| Field               | Definition                                                                                                                                              |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-outline-offset"></a>outline-offset                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a> ~~[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value in absolute units (px or physical).~~ <u>absolute length</u> |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | ~~length~~ <u>by computed value</u>                                                                                                                     |

<a id="ref-for-propdef-outline-offset①"></a>

<a id="ref-for-border-edge③"></a>

If the computed value of [outline-offset](#propdef-outline-offset) is anything other than 0, then the outline is outset from the [border edge](https://www.w3.org/TR/CSS2/box.html#border-edge) by that amount.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9002bdd7"></a>
>
> Example(s):
>
> For example, to leave 2 pixels of space between a focus outline and the element that has the focus or is active, the following rule can be used:
>
> ```text
> :focus,:active  { outline-offset: 2px }
> ```
<a id="ref-for-propdef-outline-width⑥"></a>

<a id="negative-offset"></a>Negative values must cause the outline to shrink into the border box. Both the height and the width of outside of the shape drawn by the outline should not become smaller than twice the computed value of the [outline-width](#propdef-outline-width) property, to make sure that an outline can be rendered even with large negative values. User agents should apply this constraint independently in each dimension. If the outline is drawn as multiple disconnected shapes, this constraint applies to each shape separately.

## <a id="resizing-and-overflow"></a>5. Resizing &#x26; Overflow

CSS2.1 provides a mechanism for controlling the appearance of a scrolling mechanism (e.g. scrollbars) on block container elements. This specification adds to that a mechanism for controlling user resizability of elements as well as the ability to specify text overflow behavior.

<a id="ref-for-propdef-resize"></a>

### <a id="resize"></a>5.1. Resizing Boxes: the [resize](#propdef-resize) property

<a id="ref-for-propdef-resize①"></a>

The [resize](#propdef-resize) property allows the author to specify whether or not an element is resizable by the user, and if so, along which axis/axes.

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-propdef-resize②"></a>

<a id="c1"></a> Candidate Correction 1: Now that [overflow: visible](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is no longer the only value of the <a id="ref-for-propdef-overflow①"></a>overflow property which doesn’t cause the element to become a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), adjust what elements to which the [resize](#propdef-resize) property applies to to match the original intent, rather than the literal text.

| Field               | Definition                                                                                                                                                                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-resize"></a>resize                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) both <a id="ref-for-comb-one④"></a>\| horizontal <a id="ref-for-comb-one⑤"></a>\| vertical                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-scroll-container①"></a><a id="ref-for-propdef-overflow②"></a> elements ~~with [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) other than visible,~~ <u>that are [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container)</u> and optionally replaced elements such as images, videos, and iframes |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | ~~as~~ specified <u>keyword</u>                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <u>discrete</u>                                                                                                                                                                                                                                                                                                  |

none  
The UA does not present a resizing mechanism on the element, and the user is given no direct manipulation mechanism to resize the element.

both  
The UA presents a bidirectional resizing mechanism to allow the user to adjust both the height and the width of the element.

horizontal  
The UA presents a unidirectional horizontal resizing mechanism to allow the user to adjust only the width of the element.

vertical  
The UA presents a unidirectional vertical resizing mechanism to allow the user to adjust only the height of the element.

<a id="ref-for-propdef-overflow③"></a>

<a id="ref-for-propdef-resize③"></a>

Currently it is possible to control the appearance of the scrolling mechanism (if any) on an element using the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property (e.g. `overflow: scroll` vs. `overflow: hidden` etc.). The purpose of the [resize](#propdef-resize) property is to allow control over the appearance and function of the resizing mechanism (e.g. a resize box or widget) on the element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The resizing mechanism is NOT the same as the scrolling mechanism, nor is it related to any UA mechanism for zooming. The scrolling mechanism allows the user to determine which portion of the contents of an element is shown. The resizing mechanism allows the user to determine the size of the element.

<a id="ref-for-propdef-resize④"></a>

<a id="ref-for-propdef-overflow④"></a>

<a id="ref-for-valdef-overflow-visible"></a>

<a id="ref-for-scroll-container②"></a>

<a id="ref-for-propdef-overflow⑤"></a>

The [resize](#propdef-resize) property applies to elements ~~whose computed [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) value is something other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible).~~ <u>that are [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container).</u> UAs may also apply it, regardless of the value of the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property, to:

- <a id="ref-for-canvas"></a>

  <a id="ref-for-the-object-element"></a>

  <a id="ref-for-elementdef-svg"></a>

  <a id="ref-for-the-picture-element"></a>

  <a id="ref-for-video"></a>

  <a id="ref-for-the-img-element"></a>

  Replaced elements representing images or videos, such as <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/media.html#video">video</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-picture-element">picture</a></code>, <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code>, <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element">object</a></code>, or <code><a href="https://html.spec.whatwg.org/multipage/canvas.html#canvas">canvas</a></code>.

- <a id="ref-for-the-iframe-element"></a>

  The <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element.

<a id="ref-for-propdef-resize⑤"></a>

The effect of the [resize](#propdef-resize) property on generated content is undefined. Implementations should not apply the <a id="ref-for-propdef-resize⑥"></a>resize property to generated content.

<a id="ref-for-propdef-resize⑦"></a>

<a id="ref-for-csspseudoelement"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the [resize](#propdef-resize) property may apply to generated content in the future if there is implementation of the <code><a href="https://www.w3.org/TR/css-pseudo-4/#csspseudoelement">CSSPseudoElement</a></code> interface (See [\[css-pseudo-4\]](#biblio-css-pseudo-4)).

<a id="ref-for-propdef-width⑧"></a>

<a id="ref-for-propdef-height⑤"></a>

When an element is resized by the user, the user agent sets the [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) and [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height) properties to px unit length values of the size indicated by the user, in the element’s [style attribute](https://www.w3.org/TR/css-style-attr/#style-attribute) DOM, replacing existing property declaration(s), if any, without !important, if any.

If an element is resized in only one dimension, only the corresponding property is set, not both.

<a id="ref-for-position-props②"></a>

The precise direction of resizing (i.e. altering the top left of the element or altering the bottom right) may depend on a number of CSS layout factors including whether the element is absolutely positioned, whether it is positioned using the [right](https://www.w3.org/TR/CSS2/visuren.html#position-props) and <a id="ref-for-position-props③"></a>bottom properties, whether the language of the element is right-to-left etc. The UA should consider the direction of resizing (as determined by CSS layout), as well as platform conventions and constraints when deciding how to convey the resizing mechanism to the user.

<a id="ref-for-propdef-min-width④"></a>

<a id="ref-for-propdef-max-width④"></a>

<a id="ref-for-propdef-min-height④"></a>

<a id="ref-for-propdef-max-height④"></a>

The user agent must allow the user to resize the element with no other constraints than what is imposed by [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), and [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height).

<a id="ref-for-propdef-width⑨"></a>

<a id="ref-for-propdef-height⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There may be situations where user attempts to resize an element appear to be overridden or ignored, e.g. because of !important cascading declarations that supersede that element’s [style attribute](https://www.w3.org/TR/css-style-attr/#style-attribute) [width](https://www.w3.org/TR/CSS2/visudet.html#propdef-width) and [height](https://www.w3.org/TR/CSS2/visudet.html#propdef-height) properties in the DOM.

<a id="ref-for-propdef-resize⑧"></a>

Changes to the computed value of an element’s [resize](#propdef-resize) property do not reset changes to the [style attribute](https://www.w3.org/TR/css-style-attr/#style-attribute) made due to user resizing of that element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0da0f3ee"></a>
>
> Example(s):
>
> For example, to make iframes scrollable <em>and</em> resizable, the following rule can be used:
>
> ```text
> iframe,object[type^="text/"],
> object[type$="+xml"],object[type="application/xml"] {
>   overflow:auto;
>   resize:both;
> }
> ```
<a id="ref-for-propdef-text-overflow"></a>

### <a id="text-overflow"></a>5.2.  Overflow Ellipsis: the [text-overflow](#propdef-text-overflow) property

| Field               | Definition                                                                          |
|---------------------|-------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-text-overflow"></a>text-overflow                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑥"></a>clip [\|](https://www.w3.org/TR/css-values-4/#comb-one) ellipsis |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | clip                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | ~~as~~ specified <u>keyword</u>                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <u>discrete</u>                                                                     |

<a id="ref-for-end"></a>

<a id="ref-for-propdef-overflow⑥"></a>

<a id="ref-for-valdef-overflow-visible①"></a>

This property specifies rendering when inline content overflows its [end](https://www.w3.org/TR/css-writing-modes-4/#end) line box edge in the inline progression direction of its block container element ("the block") that has [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible).

Text can overflow for example when it is prevented from wrapping (e.g. due to `white-space: nowrap` or a single word is too long to fit). Values have the following meanings:

<a id="overflow-clip"></a>clip  
Clip inline content that overflows its block container element. Characters may be only partially rendered.

<a id="overflow-ellipsis"></a>ellipsis  
Render an ellipsis character (U+2026) to represent clipped inline content. Implementations may substitute a more language, script, or writing-mode appropriate ellipsis character, or three dots "..." if the ellipsis character is unavailable.

The term "character" is used in this property definition for better readability and means "grapheme cluster" [\[UAX29\]](#biblio-uax29) for implementation purposes.

<a id="ref-for-end①"></a>

For the ellipsis value implementations must hide characters and [atomic inline-level elements](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) at the [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge of the line as necessary to fit the ellipsis, and place the ellipsis immediately adjacent to the <a id="ref-for-end②"></a>end edge of the remaining inline content. The first character or [atomic inline-level element](https://www.w3.org/TR/CSS2/visuren.html#inline-boxes) on a line must be clipped rather than ellipsed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7153ff61"></a>
>
> #### <a id="bidi-ellipsis"></a>Bidi ellipsis examples
>
> These examples demonstrate which characters get hidden to make room for the ellipsis in a bidi situation: those visually at the end edge of the line.
>
> Sample CSS:
>
> ```text
> div {
>   font-family: monospace;
>   white-space: pre;
>   overflow: hidden;
>   width: 9ch;
>   text-overflow: ellipsis;
> }
> ```
>
> Sample HTML fragments, renderings, and your browser:
>
>
> These are static transcriptions of the source’s live browser demonstration. The “browser” text below is the source content, not a measured rendering. Original demonstration HTML/CSS is included so clipping, direction, and line-breaking are not lost. See the [source demonstration](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/#example-7153ff61).
>
> **Demonstration stylesheet from the source**
>
> ```css
> .awesome-table td {padding:5px}
> .awesome-table {color:#000;background:#fff;margin: auto;}
> ```
>
> **Example 1**
>
> **HTML**
>
> `<div>שלום 123456</div>`
>
> **Reference rendering**
>
> 123456 ם…
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family:monospace">123456 ם…</div>
> ```
>
> **Your Browser**
>
> שלום 123456
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family: monospace; white-space: pre; overflow: hidden; width: 9ch; text-overflow: ellipsis">שלום 123456</div>
> ```
>
> **Example 2**
>
> **HTML**
>
> `<div dir=rtl>שלום 123456</div>`
>
> **Reference rendering**
>
> …456 שלום
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="font-family:monospace">…456 שלום</div>
> ```
>
> **Your Browser**
>
> שלום 123456
>
> **Original browser-demonstration HTML**
>
> ```html
> <div dir="rtl" style="font-family: monospace; white-space: pre; overflow: hidden; width: 9ch; text-overflow: ellipsis">שלום 123456</div>
> ```
>

#### <a id="ellipsing-details"></a>ellipsing details

- <a id="ref-for-propdef-text-overflow①"></a>

  Ellipsing only affects rendering and must not affect layout nor dispatching of pointer events: The UA should dispatch any pointer event on the ellipsis to the elided element, as if [text-overflow](#propdef-text-overflow) had been none.

- The ellipsis is styled and baseline-aligned according to the block.

- Ellipsing occurs after relative positioning and other graphical transformations.

- If there is insufficient space for the ellipsis, then clip the rendering of the ellipsis itself (on the same side that neutral characters on the line would have otherwise been clipped with the text-overflow:clip value).

#### <a id="ellipsis-interaction"></a>user interaction with ellipsis

- <a id="ref-for-propdef-text-overflow②"></a>

  When the user is interacting with content (e.g. editing, selecting, scrolling), the user agent may treat [text-overflow: ellipsis](#propdef-text-overflow) as <a id="ref-for-propdef-text-overflow③"></a>text-overflow: clip.

- Selecting the ellipsis should select the ellipsed text. If all of the ellipsed text is selected, UAs should show selection of the ellipsis. Behavior of partially-selected ellipsed text is up to the UA.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d8da6cec"></a>
>
> Example(s):
>
> #### <a id="text-overflow-examples"></a>text-overflow examples
>
> These examples demonstrate setting the text-overflow of a block container element that has text which overflows its dimensions:
>
> sample CSS for a div:
>
> ```text
> div {
>   font-family:Helvetica,sans-serif; line-height:1.1;
>   width:3.1em; padding:.2em; border:solid .1em black; margin:1em 0;
> }
> ```
>
> sample HTML fragments, renderings, and your browser:
>
>
> These are static transcriptions of the source’s live browser demonstration. The “browser” text below is the source content, not a measured rendering. Original demonstration HTML/CSS is included so clipping, direction, and line-breaking are not lost. See the [source demonstration](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/#example-d8da6cec).
>
> **Demonstration stylesheet from the source**
>
> ```css
> .awesome-table td {padding:5px}
> .awesome-table {color:#000;background:#fff;margin: auto;}
> ```
>
> **Example 1**
>
> **HTML**
>
> ```text
> <div>
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> First, a box with text drawing outside of it.
>
> ![First, a box with text drawing outside of it.](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/images/cssisawesome.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em; font-family:Helvetica,sans-serif; line-height:1.1;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 2**
>
> **HTML**
>
> ```text
> <div style="text-overflow:clip; overflow:hidden">
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> Second, a similar box with the text clipped outside the box.
>
> ![Second, a similar box with the text clipped outside the box.](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/images/cssisaweso.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em; font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:clip;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 3**
>
> **HTML**
>
> ```text
> <div style="text-overflow:ellipsis; overflow:hidden">
> CSS IS AWESOME, YES
> </div>
> ```
>
> **sample rendering**
>
> Third, a similar box with an ellipsis representing the clipped text.
>
> ![Third, a similar box with an ellipsis representing the clipped text.](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/images/cssisaw.png)
>
> **your browser**
>
> CSS IS AWESOME, YES
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em;  font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:ellipsis;">CSS IS AWESOME, YES</div>
> ```
>
> **Example 4**
>
> **HTML**
>
> ```text
> <div style="text-overflow:ellipsis; overflow:hidden">
> NESTED
>   <p>PARAGRAPH</p>
> WON'T ELLIPSE.
> </div>
> ```
>
> **sample rendering**
>
> Fourth, a box with a nested paragraph demonstrating anonymous block boxes equivalency and non-inheritance into a nested element.
>
> ![Fourth, a box with a nested paragraph demonstrating anonymous block boxes equivalency and non-inheritance into a nested element.](https://www.w3.org/TR/2026/REC-css-ui-3-20260407/images/nes.png)
>
> **your browser**
>
> NESTED
>
> PARAGRAPH
>
> WON’T ELLIPSE.
>
> **Original browser-demonstration HTML**
>
> ```html
> <div style="width:3.1em; border:solid .1em black; margin:1em 0; padding:.2em;  font-family:Helvetica,sans-serif; line-height:1.1; overflow:hidden;text-overflow:ellipsis;">
>          NESTED
> 	
>          <p>PARAGRAPH</p>
>          
> WON’T ELLIPSE.
>         </div>
> ```
>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-physical-left"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the side of the line that the ellipsis is placed depends on the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) of the block. E.g. an overflow hidden right-to-left (`direction: rtl`) block clips inline content on the [left](https://www.w3.org/TR/css-writing-modes-3/#physical-left) side, thus would place a text-overflow ellipsis on the <a id="ref-for-physical-left①"></a>left to represent that clipped content.

#### <a id="ellipsis-scrolling"></a>ellipsis interaction with scrolling interfaces

This section applies to elements with text-overflow other than text-overflow:clip (non-clip text-overflow) and overflow:scroll.

When an element with non-clip text-overflow has overflow of scroll in the inline progression dimension of the text, and the browser provides a mechanism for scrolling (e.g. a scrollbar on the element, or a touch interface to swipe-scroll, etc.), there are additional implementation details that provide a better user experience:

When an element is scrolled (e.g. by the user, DOM manipulation), more of the element’s content is shown. The value of text-overflow should not affect whether more of the element’s content is shown or not. If a non-clip text-overflow is set, then as more content is scrolled into view, implementations should show whatever additional content fits, only truncating content which would otherwise be clipped (or is necessary to make room for the ellipsis/string), until the element is scrolled far enough to display the edge of the content at which point that content should be displayed rather than an ellipsis/string.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6a5a6d1e"></a>
>
> Example(s):
>
> This example uses text-overflow on an element with overflow scroll to demonstrate the above described behavior.
>
> sample CSS:
>
> ```text
> div.crawlbar {
>   text-overflow: ellipsis;
>   height: 2em;
>   overflow: scroll;
>   white-space: nowrap;
>   width: 15em;
>   border:1em solid black;
> }
> ```
>
> sample HTML fragment:
>
> ```text
> <div class="crawlbar">
> CSS is awesome, especially when you can scroll
> to see extra text instead of just
> having it overlap other text by default.
> </div>
> ```
>
> demonstration of sample CSS and HTML:
>
> CSS is awesome, especially when you can scroll to see extra text instead of just having it overlap other text by default.

While the content is being scrolled, implementations may adjust their rendering of ellipses (e.g. align to the box edge rather than line edge).

## <a id="pointing-keyboard"></a>6. Pointing Devices and Keyboards

### <a id="pointer-interaction"></a>6.1. Pointer interaction

<a id="ref-for-propdef-cursor"></a>

#### <a id="cursor"></a>6.1.1. Styling the Cursor: the [cursor](#propdef-cursor) property

| Field               | Definition                                                                                                                                                                                                                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-cursor"></a>cursor                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-cursor-predefined"></a><a id="ref-for-mult-zero-plus"></a><a id="ref-for-comb-comma"></a><a id="ref-for-typedef-cursor-cursor-image"></a>\[[\<cursor-image\>](#typedef-cursor-cursor-image)[,](https://www.w3.org/TR/css-values-4/#comb-comma)\][\*](https://www.w3.org/TR/css-values-4/#mult-zero-plus) [\<cursor-predefined\>](#typedef-cursor-predefined) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified, except with any relative URLs converted to absolute                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <u>discrete</u>                                                                                                                                                                                                                                                                                 |

<a id="ref-for-border-edge④"></a>

This property specifies the type of cursor to be displayed for the pointing device when the cursor’s hotspot is within the element’s [border edge](https://www.w3.org/TR/CSS2/box.html#border-edge).

<a id="ref-for-border-edge⑤"></a>

<a id="ref-for-propdef-border-radius①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As per [CSS Backgrounds 3 § 4.1 Curve Radii: the border-radius properties](https://www.w3.org/TR/css-backgrounds-3/#border-radius), the [border edge](https://www.w3.org/TR/CSS2/box.html#border-edge) is affected by [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius).

In the case of overlapping elements, which element determines the type of cursor is based on hit testing: the element determining the cursor is the one that would receive a click initiated from this position.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The specifics of hit testing are out of scope of this specification. Hit testing will hopefully be defined in a future revision of CSS or HTML.

User agents may ignore the cursor property over native user-agent controls such as scrollbars, resizers, or other native UI widgets e.g. those that may be used inside some user agent specific implementations of form elements. User agents may also ignore the cursor property and display a cursor of their choice to indicate various states of the UA’s user interface, such as a busy cursor when the page is not responding, or a text cursor when the user is performing text selection.

<a id="ref-for-propdef-cursor①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[HTML\]](#biblio-html) defines [special handling of image maps](https://html.spec.whatwg.org/multipage/rendering.html#image-maps-2) for the [cursor](#propdef-cursor) property.

Values have the following meanings:

<a id="ref-for-typedef-cursor-cursor-image①"></a>

[\<cursor-image\>](#typedef-cursor-cursor-image)

<a id="ref-for-propdef-cursor②"></a>

The first (optional) component of the [cursor](#propdef-cursor) property is a list of image-based cursors. If the user agent cannot handle the first cursor of a list of cursors, it must attempt to handle the second, etc. If the user agent cannot handle any of these author-defined cursors, it must use the keyword-based cursor at the end of the list.

<a id="ref-for-typedef-cursor-cursor-image②"></a>

A [\<cursor-image\>](#typedef-cursor-cursor-image) has the following syntax:

<a id="typedef-cursor-cursor-image"></a>

<a id="ref-for-url-value"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-mult-opt"></a>

```text
<cursor-image> = <url> <number>{2}?
```
<a id="ref-for-url-value①"></a>

<a id="ref-for-typedef-image"></a>

The user agent retrieves the cursor from the resource designated by the URL. Conforming user agents may, instead of [\<url\>](https://www.w3.org/TR/css-values-4/#url-value), support [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) which is a superset.

The UA must support the following image file formats:

- PNG, as defined in [\[PNG\]](#biblio-png)

- <a id="ref-for-natural-size"></a>

  SVG, as defined in [\[SVG11\]](#biblio-svg11), in [secure static mode](https://www.w3.org/TR/SVG2/conform.html#secure-static-mode) [\[SVG2\]](#biblio-svg2), if it has a [natural size](https://www.w3.org/TR/css-images-3/#natural-size).

- <a id="ref-for-propdef-background-image"></a>

  <a id="ref-for-typedef-image①"></a>

  any other non-animated image file format that they support for [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) in other properties, such as the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property

In addition, the UA should support the following image file formats:

- <a id="ref-for-natural-size①"></a>

  SVG, as defined in [\[SVG11\]](#biblio-svg11), in [secure animated mode](https://www.w3.org/TR/SVG2/conform.html#secure-animated-mode) [\[SVG2\]](#biblio-svg2), if it has a [natural size](https://www.w3.org/TR/css-images-3/#natural-size).

- <a id="ref-for-propdef-background-image①"></a>

  <a id="ref-for-typedef-image②"></a>

  any other animated image file format that they support for [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) in other properties, such as the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property

<a id="ref-for-natural-size②"></a>

The UA may also support additional file formats, including SVG, as defined in [\[SVG11\]](#biblio-svg11), in secure static mode or secure animated mode [\[SVG2\]](#biblio-svg2), even if it does not have a [natural size](https://www.w3.org/TR/css-images-3/#natural-size).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSS Working group initially intended support for all SVG, naturally sized or not. Support for non-naturally sized SVG was downgraded from mandatory to optional due to lack of implementations.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing this specification (spring 2015), the only file formats supported for cursors in common desktop browsers are the .ico and .cur file formats, as designed by Microsoft. For compatibility with legacy content, UAs are encouraged to support these, even though the lack of an open specification makes it impossible to have a normative requirement about these formats. Some information on these formats can be found [on Wikipedia](https://en.wikipedia.org/wiki/ICO_%28file_format%29).

<a id="ref-for-default-object-size"></a>

The [default object size](https://www.w3.org/TR/css-images-3/#default-object-size) for cursor images is a UA-defined size that should be based on the size of a typical cursor on the UA’s operating system.

<a id="ref-for-concrete-object-size"></a>

<a id="ref-for-default-sizing-algorithm"></a>

<a id="ref-for-natural-aspect-ratio"></a>

The [concrete object size](https://www.w3.org/TR/css-images-3/#concrete-object-size) is determined using the [default sizing algorithm](https://www.w3.org/TR/css-images-3/#default-sizing-algorithm). If an operating system is <strong>incapable</strong> of rendering a cursor above a given size, cursors larger than that size must be shrunk to within the OS-supported size bounds, while maintaining the cursor image’s [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), if any.

<a id="ref-for-number-value①"></a>

The optional pair of [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) values give the X and Y coordinates of the exact position within the image which is the pointer position (i.e., the hotspot), as offsets from the left/top of the image.

<a id="ref-for-typedef-image③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define how the coordinate systems of the various types of [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) are established, and defers these definitions to [\[CSS4-IMAGES\]](#biblio-css4-images).

If the values are omitted, then the natural hotspot defined inside the image resource itself is used. If it has no natural hotspot, the top left corner of the image is used, as if 0 0 were provided.

If the X or Y coordinates of the hotspot (whether specified explicitly or taken from the image) fall outside of the cursor image, they must be clamped (independently) to fit.

<a id="ref-for-typedef-cursor-predefined①"></a>

<a id="valdef-cursor-cursor-predefined"></a>[\<cursor-predefined\>](#typedef-cursor-predefined)

<a id="ref-for-url-value②"></a>

<a id="ref-for-typedef-cursor-predefined②"></a>

The mandatory [\<cursor-predefined\>](#typedef-cursor-predefined) keyword specifies a predefined cursor to use, or the <em>fallback</em> cursor to be used if [\<url\>](https://www.w3.org/TR/css-values-4/#url-value)s were provided and none of them can be successfully used.

<a id="ref-for-typedef-cursor-predefined③"></a>

See [§ 6.1.1.1 Predefined Cursors](#predefined-cursors) for the full set of [\<cursor-predefined\>](#typedef-cursor-predefined) keywords and their meanings.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-07f62ed2"></a> Example: cursor fallback
>
> Here is an example of using several cursor values.
>
> ```text
> :link,:visited {
>     cursor: url(example.svg#linkcursor),
>             url(hyper.cur),
>             url(hyper.png) 2 3,
>             pointer;
> }
> ```
>
> <a id="ref-for-valdef-cursor-pointer"></a>
>
> This example sets the cursor on all hyperlinks (whether visited or not) to an external [SVG cursor](https://www.w3.org/TR/SVG11/interact.html#CursorElement) ([\[SVG11\]](#biblio-svg11), section 16.8.3). User agents that don’t support SVG cursors would simply skip to the next value and attempt to use the "hyper.cur" cursor. If that cursor format was also not supported, the UA could attempt to use the "hyper.png" cursor with the explicit hotspot. Finally if the UA does not support any of those image cursor formats, the UA would skip to the last value and render the [pointer](#valdef-cursor-pointer) cursor.

##### <a id="predefined-cursors"></a>6.1.1.1.  Predefined Cursors

<a id="ref-for-typedef-cursor-predefined④"></a>

The [\<cursor-predefined\>](#typedef-cursor-predefined) production encompasses a broad selection of predefined cursors, present on most operating systems. Its syntax is:

<a id="typedef-cursor-predefined"></a>

<a id="ref-for-typedef-cursor-predefined⑤"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

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

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-comb-one②⑤"></a>

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

<a id="ref-for-comb-one③⑥"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-comb-one③⑨"></a>

<a id="ref-for-comb-one④⓪"></a>

<a id="ref-for-comb-one④①"></a>

```text
<cursor-predefined> = auto | default | none |
  context-menu | help | pointer | progress | wait |
  cell | crosshair | text | vertical-text |
  alias | copy | move | no-drop | not-allowed | grab | grabbing |
  e-resize | n-resize | ne-resize | nw-resize | s-resize | se-resize | sw-resize | w-resize |
  ew-resize | ns-resize | nesw-resize | nwse-resize |
  col-resize | row-resize |
  all-scroll |
  zoom-in | zoom-out
```
<a id="ref-for-typedef-cursor-predefined⑥"></a>

The [\<cursor-predefined\>](#typedef-cursor-predefined) keywords have the following meanings and likely renderings:

general purpose cursors  
<a id="valdef-cursor-auto"></a>auto  
<a id="ref-for-valdef-cursor-default"></a>

<a id="ref-for-valdef-cursor-text"></a>

<a id="ref-for-valdef-cursor-auto"></a>

The UA determines the cursor to display based on the current context: [auto](#valdef-cursor-auto) behaves as [text](#valdef-cursor-text) over selectable text or editable elements, and [default](#valdef-cursor-default) otherwise.

<a id="valdef-cursor-default"></a>default  
The platform-dependent default cursor. Often rendered as an arrow.

<a id="valdef-cursor-none"></a>none  
No cursor is rendered for the element.

links and status cursors  
<a id="valdef-cursor-context-menu"></a>context-menu  
A context menu is available for the object under the cursor. Often rendered as an arrow with a small menu-like graphic next to it.

<a id="valdef-cursor-help"></a>help  
Help is available for the object under the cursor. Often rendered as a question mark or a balloon.

<a id="valdef-cursor-pointer"></a>pointer  
The cursor is a pointer that indicates a link.

<a id="valdef-cursor-progress"></a>progress  
<a id="ref-for-valdef-cursor-wait"></a>

A progress indicator. The program is performing some processing, but is different from [wait](#valdef-cursor-wait) in that the user may still interact with the program. Often rendered as a spinning beach ball, or an arrow with a watch or hourglass.

<a id="valdef-cursor-wait"></a>wait  
Indicates that the program is busy and the user should wait. Often rendered as a watch or hourglass.

selection cursors  
<a id="valdef-cursor-cell"></a>cell  
Indicates that a cell or set of cells may be selected. Often rendered as a thick plus-sign with a dot in the middle.

<a id="valdef-cursor-crosshair"></a>crosshair  
A simple crosshair (e.g., short line segments resembling a "+" sign). Often used to indicate a two dimensional bitmap selection mode.

<a id="valdef-cursor-text"></a>text  
<a id="ref-for-valdef-cursor-vertical-text"></a>

Indicates text that may be selected. Often rendered as a vertical I-beam. User agents may automatically display a horizontal I-beam/cursor (e.g. same as the [vertical-text](#valdef-cursor-vertical-text) keyword) for vertical text, or for that matter, any angle of I-beam/cursor for text that is rendered at any particular angle.

<a id="valdef-cursor-vertical-text"></a>vertical-text  
Indicates vertical-text that may be selected. Often rendered as a horizontal I-beam.

drag and drop cursors  
<a id="valdef-cursor-alias"></a>alias  
Indicates an alias of/shortcut to something is to be created. Often rendered as an arrow with a small curved arrow next to it.

<a id="valdef-cursor-copy"></a>copy  
Indicates something is to be copied. Often rendered as an arrow with a small plus sign next to it.

<a id="valdef-cursor-move"></a>move  
Indicates something is to be moved.

<a id="valdef-cursor-no-drop"></a>no-drop  
Indicates that the dragged item cannot be dropped at the current cursor location. Often rendered as a hand or pointer with a small circle with a line through it.

<a id="valdef-cursor-not-allowed"></a>not-allowed  
Indicates that the requested action will not be carried out. Often rendered as a circle with a line through it.

<a id="valdef-cursor-grab"></a>grab  
Indicates that something can be grabbed (dragged to be moved). Often rendered as the backside of an open hand.

<a id="valdef-cursor-grabbing"></a>grabbing  
Indicates that something is being grabbed (dragged to be moved). Often rendered as the backside of a hand with fingers closed mostly out of view.

resizing and scrolling cursors  
<a id="valdef-cursor-e-resize"></a>e-resize  
<a id="valdef-cursor-n-resize"></a>n-resize  
<a id="valdef-cursor-ne-resize"></a>ne-resize  
<a id="valdef-cursor-nw-resize"></a>nw-resize  
<a id="valdef-cursor-s-resize"></a>s-resize  
<a id="valdef-cursor-se-resize"></a>se-resize  
<a id="valdef-cursor-sw-resize"></a>sw-resize  
<a id="valdef-cursor-w-resize"></a>w-resize  
<a id="ref-for-valdef-cursor-se-resize"></a>

Indicates that some edge is to be moved. For example, the [se-resize](#valdef-cursor-se-resize) cursor is used when the movement starts from the south-east corner of the box.

<a id="valdef-cursor-ew-resize"></a>ew-resize  
<a id="valdef-cursor-ns-resize"></a>ns-resize  
<a id="valdef-cursor-nesw-resize"></a>nesw-resize  
<a id="valdef-cursor-nwse-resize"></a>nwse-resize  
Indicates a bidirectional resize cursor.

<a id="valdef-cursor-col-resize"></a>col-resize  
Indicates that the item/column can be resized horizontally. Often rendered as arrows pointing left and right with a vertical bar separating them.

<a id="valdef-cursor-row-resize"></a>row-resize  
Indicates that the item/row can be resized vertically. Often rendered as arrows pointing up and down with a horizontal bar separating them.

<a id="valdef-cursor-all-scroll"></a>all-scroll  
Indicates that the something can be scrolled in any direction. Often rendered as arrows pointing up, down, left, and right with a dot in the middle.

<a id="zooming-cursors"></a>zooming cursors  
<a id="valdef-cursor-zoom-in"></a>zoom-in  
<a id="valdef-cursor-zoom-out"></a>zoom-out  
<a id="ref-for-valdef-cursor-zoom-out"></a>

<a id="ref-for-valdef-cursor-zoom-in"></a>

Indicates that something can be zoomed (magnified) in or out, and often rendered as a magnifying glass with a "+" or "-" in the center of the glass, for [zoom-in](#valdef-cursor-zoom-in) and [zoom-out](#valdef-cursor-zoom-out) respectively.

##### <a id="canvas_cursor"></a>6.1.1.2. Cursor of the canvas

<a id="ref-for-propdef-display"></a>

The document [canvas](https://www.w3.org/TR/CSS2/intro.html#the-canvas) is the infinite surface over which the document is rendered [\[CSS2\]](#biblio-css2). Since no element corresponds to the canvas, in order to allow styling of the cursor when not over any element, the canvas cursor re-uses the root element’s cursor. However, if no boxes are generated for the root element (for example, if the root element has [display: none](https://www.w3.org/TR/CSS2/visuren.html#propdef-display)), then the canvas cursor is the platform-dependent default cursor.

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-display①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An element might be invisible, but still generate boxes. For example, if the element has [visibility: hidden](https://www.w3.org/TR/CSS2/visufx.html#propdef-visibility) but not [display: none](https://www.w3.org/TR/CSS2/visuren.html#propdef-display), boxes are generated for it and its cursor is used for the canvas.

### <a id="insertion-caret"></a>6.2. Insertion caret

<a id="ref-for-propdef-caret-color"></a>

#### <a id="caret-color"></a>6.2.1. Coloring the Insertion Caret: the [caret-color](#propdef-caret-color) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-caret-color"></a>caret-color                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color②"></a><a id="ref-for-comb-one④②"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color③"></a><a id="ref-for-propdef-color①"></a><a id="ref-for-valdef-color-currentcolor①"></a><a id="ref-for-valdef-caret-color-auto"></a>The computed value for [auto](#valdef-caret-color-auto) is <a id="ref-for-valdef-caret-color-auto①"></a>auto; the computed value of currentColor is currentColor (See [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor)); see the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property for other [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) values. |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | ~~color~~ <u>by computed value</u>                                                                                                                                                                                                                                                                                                                                                                                                                                |

<a id="valdef-caret-color-auto"></a>auto

User agents should use currentColor. User agents may automatically adjust the color of caret to ensure good visibility and contrast with the surrounding content, possibly based on the currentColor, background, shadows, etc.

<a id="ref-for-typedef-color④"></a>

[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

The insertion caret is colored with the specified color.

The caret is a visible indicator of the insertion point in an element where text (and potentially other content) is inserted by the user. This property controls the color of that visible indicator.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: caret shape and blinking is outside the scope of this feature and thus unspecified.

<a id="ref-for-propdef-cursor③"></a>

<a id="ref-for-valdef-cursor-auto①"></a>

<a id="ref-for-valdef-cursor-text①"></a>

<a id="ref-for-valdef-cursor-vertical-text①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: UAs might have additional things that count as “carets”. For example, some UAs can show a “navigation caret”, which acts similarly to an insertion caret but can be moved around in non-editable text, and is functionally a caret. On the other hand, the cursor image shown when hovering over text when the [cursor](#propdef-cursor) property is [auto](#valdef-cursor-auto), or when hovering over an element where the <a id="ref-for-propdef-cursor④"></a>cursor property is [text](#valdef-cursor-text) or [vertical-text](#valdef-cursor-vertical-text), though it sometimes resembles a caret, is not a caret (it’s a cursor).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dfd794ec"></a> Example: a textarea with `caret-color:#00aacc;`

### <a id="keyboard"></a>6.3. Keyboard control

#### <a id="input-method-editor"></a>6.3.1. Obsolete: the ime-mode property

"ime-mode" is a property somewhat implemented in some browsers, that is problematic and officially obsoleted by this specification.

User agents should not support the ime-mode property.

Authors must not use the ime-mode property.

Users may use the ime-mode property only for repair use-cases where they have to work around bad sites and legacy implementations, e.g. with a user style sheet rule like:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a7900f43"></a>
>
> Example: user preference
>
> ```text
> input[type=password] {
>   ime-mode: auto !important;
> }
> ```
This example CSS may be placed into a user style sheet file to force password input fields to behave in a default manner.

This specification deliberately does not attempt to document the functionality of legacy ime-mode implementations nor what they specifically support because it does not make sense to pursue or recommend any such path.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: there are several [\[HTML\]](#biblio-html) features which authors should use to provide information to user agents that allow them to provide a better input user experience:
>
> - The global `lang` attribute
> - The `inputmode`, `pattern`, and `type` attributes of the input element

------------------------------------------------------------------------

## <a id="acknowledgments"></a>Appendix A: Acknowledgments

This appendix is <em>informative</em>.

This specification was edited and written for the most part by Tantek Çelik from 1999 to the present, first while representing Microsoft, then as an Invited Expert, and most recently while representing Mozilla.

<a id="ref-for-propdef-box-sizing⑧"></a>

Thanks to Florian Rivoal, working on this specification on behalf of Bloomberg, for his recent work documenting issues from www-style emails, proposing resolutions &#x26; changes, and in particular for researching &#x26; writing greatly improved details for the [box-sizing](#propdef-box-sizing) property.

Thanks to feedback and contributions from Rossen Atanassov, Tab Atkins, L. David Baron, Bert Bos, Matthew Brealey, Rick Byers, Ada Chan, James Craig, Michael Cooper, Axel Dahmen, Michael Day, Micah Dubinko, Elika E., Steve Falkenburg, Andrew Fedoniouk, Al Gilman, Ian Hickson, Bjoern Hoehrmann, Alan Hogan, David Hyatt, Richard Ishida, Sho Kuwamoto, Yves Lafon, Stuart Langridge, Susan Lesch, Peter Linss, Kang-Hao Lu, Masayuki Nakano, Mats Palmgren, Brad Pettit, Chris Rebert, François Remy, Andrey Rybka, Simon Sapin, Alexander Savenkov, Sebastian Schnitzenbaumer, Lea Verou, Etan Wexler, David Woolley, Frank Yan, Boris Zbarsky, and Domel.

## <a id="changes"></a>Appendix B: Changes

This appendix is <em>informative</em>.

### <a id="changes-2018-06-21"></a> Changes from the [21 June 2018 Recommendation (REC)](https://www.w3.org/TR/2018/REC-css-ui-3-20180621/)

- <a id="ref-for-valdef-overflow-clip"></a>

  <a id="ref-for-valdef-overflow-visible②"></a>

  <a id="ref-for-propdef-overflow⑦"></a>

  <a id="ref-for-scroll-container③"></a>

  [Candidate Correction 1:](#c1) Referred to [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) instead of [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) values other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible), so [clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip) is included.

- <a id="ref-for-snap-a-length-as-a-border-width②"></a>

  <a id="ref-for-propdef-outline-width⑦"></a>

  [Candidate Correction 2:](#c2) Let computed value of [outline-width](#propdef-outline-width) be [snapped as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width).

- [Candidate Correction 3:](#c3) Updated and clarified computed value and animation type definitions.

- <a id="ref-for-propdef-outline-style⑥"></a>

  <a id="ref-for-propdef-outline-width⑧"></a>

  [Candidate Correction 4:](#c4) Removed the special case of [outline-width](#propdef-outline-width) having a computed value of 0 based on [outline-style: none](#propdef-outline-style).

- Update to modern terminology for consistency with other specifications, replacing relevant uses of terms 'intrinsic width/height/aspect ratio/size' to 'natural'.

- Removed "Media" entries from property definitions. Its meaning has never been defined properly, and it does not add useful information.

- Added [Value Definitions](#values) section, for consistency with other CSS specifications.

- <a id="ref-for-propdef-cursor⑤"></a>

  Rearranged the description of the syntax of the [cursor](#propdef-cursor) property for better readability.

- Added Web Platform Tests coverage.

- Split privacy and security considerations into separate sections.

- Update references to other CSS modules to more contemporary ones.

- Various other editorial improvements.

### <a id="changes-2021-03-16"></a> Changes from the [2 March 2017 Candidate Recommendation (CR)](https://www.w3.org/TR/2017/CR-css-ui-3-20170302/)

- Updated references to latest versions

- Editorial Clarification about the resize property

- Move (at risk) directional focus navigation properties from level 3 to level 4

- <a id="ref-for-propdef-cursor⑥"></a>

  Added informative link to HTML about special handling of [cursor](#propdef-cursor) over image maps

- Clarified (as a SHOULD) the implications of text-overflow on pointer events to capture implementor consensus ([corresponding test](https://github.com/web-platform-tests/wpt/commit/b749ca84fe5474adb4473c35a3da5788e5b6cfd7#diff-8667b2cb07cf6ee064b9f2a74e221e8f)).

- Clarified that UAs may ignore the cursor property to reflect the UA’s UI state

- Allowed, but stopped requiring support for SVG images without natural sizes for cursors ([corresponding test update](https://github.com/web-platform-tests/wpt/commit/92770f655298aa72b0c0ee9238377d6b04d2e3e6)).

- <a id="ref-for-valdef-cursor-text②"></a>

  <a id="ref-for-propdef-cursor⑦"></a>

  Aligned the spec with implementations, and make [cursor: auto](#propdef-cursor) look like [text](#valdef-cursor-text) over <strong>selectable</strong> text, and over editable elements ([corresponding tests](https://github.com/web-platform-tests/wpt/commit/34c61eff5eab4ebe9ff271e46658f73f18858c4f)).

## <a id="privacy"></a>Appendix C: Privacy Considerations

The W3C TAG is developing a [Self-Review Questionnaire: Security and Privacy](https://w3ctag.github.io/security-questionnaire/) for editors of specifications to informatively answer.

Per the [Questions to Consider](https://w3ctag.github.io/security-questionnaire/#questions)

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

6.  Does this specification allow an origin access to a user’s location?

    No.

7.  Does this specification allow an origin access to sensors on a user’s device?

    No.

8.  Does this specification allow an origin access to aspects of a user’s local computing environment?

    No.

9.  Does this specification allow an origin access to other devices?

    No.

10. Does this specification expose temporary identifiers to the web?

    No.

11. Does this specification distinguish between behavior in first-party and third-party contexts?

    No.

12. How should this specification work in the context of a user agent’s "incognito" mode?

    No differently.

13. Does this specification persist data to a user’s local device?

    No.

## <a id="security"></a>Appendix D: Security Considerations

This appendix is <em>informative</em>.

The W3C TAG is developing a [Self-Review Questionnaire: Security and Privacy](https://w3ctag.github.io/security-questionnaire/) for editors of specifications to informatively answer.

Per the [Questions to Consider](https://w3ctag.github.io/security-questionnaire/#questions)

1.  Does this specification enable new script execution/loading mechanisms?

    <a id="ref-for-propdef-cursor⑧"></a>

    <a id="ref-for-typedef-image④"></a>

    Yes to loading, but not to execution. The [cursor](#propdef-cursor) property accepts [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) values which may include URLs to be loaded. These may be SVG documents which may contain scripts, but this specification requires that scripts must not be run.

2.  Does this specification allow an origin some measure of control over a user agent’s native UI?

    <a id="ref-for-propdef-cursor⑨"></a>

    <a id="ref-for-propdef-caret-color①"></a>

    <a id="ref-for-propdef-outline-style⑦"></a>

    <a id="ref-for-propdef-outline②"></a>

    Yes. The [cursor](#propdef-cursor) and [caret-color](#propdef-caret-color) properties enable the page to change the display of the cursor and text insertion caret of the user agent’s native UI. In addition the [outline-style](#propdef-outline-style) property’s auto value (and thus [outline](#propdef-outline) shorthand) enable the page to potentially display a native focused element outline presentation around any element.

3.  Does this specification allow downgrading default security characteristics?

    No.

## <a id="default-style-sheet"></a>Appendix E: Default style sheet additions for HTML

This appendix is <em>informative</em>.

Potential additions to the base style sheet to express HTML form controls, and a few dynamic presentation attributes:

```text
:enabled:focus {
  outline: 2px inset;
}

button,
input[type=button],
input[type=reset],
input[type=submit],
input[type=checkbox],
input[type=radio],
textarea,
input,
input[type=text],
input[type=password],
input[type=image] {
  display: inline-block;
}

input[type=button],
input[type=reset],
input[type=submit],
input[type=checkbox],
input[type=radio],
input,
input[type=text],
input[type=password],
input[type=image] {
  white-space: nowrap;
}

button {
  /* white space handling of BUTTON tags in particular */
  white-space:normal;
}

input[type=reset]:lang(en) {
/* default content of HTML input type=reset button, per language */
  content: "Reset";
}

input[type=submit]:lang(en) {
/* default content of HTML input type=submit button, per language */
  content: "Submit";
}

/* UAs should use language-specific Reset/Submit rules for others. */

input[type=button],
input[type=reset][value],
input[type=submit][value] {
/* text content/labels of HTML "input" buttons */
  content: attr(value);
}

textarea {
/* white space handling of TEXTAREA tags in particular */
  white-space:pre-wrap;
  resize: both;
}

input[type=hidden] {
/* appearance of the HTML hidden text field in particular */
  display: none !important;
}

input[type=image] {
  content: attr(src,url);
  border: none;
}

select[size] {
/* HTML4/XHTML1 <select> w/ size more than 1 - appearance of list */
  display: inline-block;
  height: attr(size,em);
}

select, select[size=1] {
/* HTML4/XHTML1 <select> without size, or size=1 - popup-menu */
  display: inline-block;
  height: 1em;
  overflow: hidden;
}

select[size]:active {
/* active HTML <select> w/ size more than 1 - appearance of active list */
  display: inline-block;
}

optgroup, option {
  display: block;
  white-space: nowrap;
}

optgroup[label], option[label] {
  content: attr(label);
}

option[selected]::before {
  display: inline;
  content: check;
}

  /* Though FRAME resizing is not directly addressed by this specification,
     the following rules may provide an approximation of reasonable behavior. */

/*

frame {
  resize:both;
}
frame[noresize] {
  resize:none
}

*/
```
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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [alias](#valdef-cursor-alias), in § 6.1.1.1
- [all-scroll](#valdef-cursor-all-scroll), in § 6.1.1.1
- auto
  - [value for caret-color](#valdef-caret-color-auto), in § 6.2.1
  - [value for cursor](#valdef-cursor-auto), in § 6.1.1.1
- [border-box](#valdef-box-sizing-border-box), in § 3.1
- [box-sizing](#propdef-box-sizing), in § 3.1
- [caret-color](#propdef-caret-color), in § 6.2.1
- [cell](#valdef-cursor-cell), in § 6.1.1.1
- [clip](#overflow-clip), in § 5.2
- [col-resize](#valdef-cursor-col-resize), in § 6.1.1.1
- [content-box](#valdef-box-sizing-content-box), in § 3.1
- [context-menu](#valdef-cursor-context-menu), in § 6.1.1.1
- [copy](#valdef-cursor-copy), in § 6.1.1.1
- [crosshair](#valdef-cursor-crosshair), in § 6.1.1.1
- [cursor](#propdef-cursor), in § 6.1.1
- [\<cursor-image\>](#typedef-cursor-cursor-image), in § 6.1.1
- \<cursor-predefined\>
  - [(type)](#typedef-cursor-predefined), in § 6.1.1.1
  - [value for cursor](#valdef-cursor-cursor-predefined), in § 6.1.1
- [default](#valdef-cursor-default), in § 6.1.1.1
- [ellipsis](#overflow-ellipsis), in § 5.2
- [e-resize](#valdef-cursor-e-resize), in § 6.1.1.1
- [ew-resize](#valdef-cursor-ew-resize), in § 6.1.1.1
- [grab](#valdef-cursor-grab), in § 6.1.1.1
- [grabbing](#valdef-cursor-grabbing), in § 6.1.1.1
- [help](#valdef-cursor-help), in § 6.1.1.1
- [invert](#valdef-outline-color-invert), in § 4.4
- [max inner height](#max-inner-height), in § 3.1
- [max inner width](#max-inner-width), in § 3.1
- [min inner height](#min-inner-height), in § 3.1
- [min inner width](#min-inner-width), in § 3.1
- [move](#valdef-cursor-move), in § 6.1.1.1
- [ne-resize](#valdef-cursor-ne-resize), in § 6.1.1.1
- [nesw-resize](#valdef-cursor-nesw-resize), in § 6.1.1.1
- [no-drop](#valdef-cursor-no-drop), in § 6.1.1.1
- [none](#valdef-cursor-none), in § 6.1.1.1
- [not-allowed](#valdef-cursor-not-allowed), in § 6.1.1.1
- [n-resize](#valdef-cursor-n-resize), in § 6.1.1.1
- [ns-resize](#valdef-cursor-ns-resize), in § 6.1.1.1
- [nw-resize](#valdef-cursor-nw-resize), in § 6.1.1.1
- [nwse-resize](#valdef-cursor-nwse-resize), in § 6.1.1.1
- [outline](#propdef-outline), in § 4.1
- [outline-color](#propdef-outline-color), in § 4.4
- [outline-offset](#propdef-outline-offset), in § 4.5
- [outline-style](#propdef-outline-style), in § 4.3
- [outline-width](#propdef-outline-width), in § 4.2
- [pointer](#valdef-cursor-pointer), in § 6.1.1.1
- [progress](#valdef-cursor-progress), in § 6.1.1.1
- [resize](#propdef-resize), in § 5.1
- [row-resize](#valdef-cursor-row-resize), in § 6.1.1.1
- [se-resize](#valdef-cursor-se-resize), in § 6.1.1.1
- [s-resize](#valdef-cursor-s-resize), in § 6.1.1.1
- [sw-resize](#valdef-cursor-sw-resize), in § 6.1.1.1
- [text](#valdef-cursor-text), in § 6.1.1.1
- [text-overflow](#propdef-text-overflow), in § 5.2
- [vertical-text](#valdef-cursor-vertical-text), in § 6.1.1.1
- [wait](#valdef-cursor-wait), in § 6.1.1.1
- [w-resize](#valdef-cursor-w-resize), in § 6.1.1.1
- [zoom-in](#valdef-cursor-zoom-in), in § 6.1.1.1
- [zoom-out](#valdef-cursor-zoom-out), in § 6.1.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="5747d295"></a>\<line-width\>
  - <a id="5ced56d0"></a>background-image
  - <a id="bb899432"></a>border-bottom-width
  - <a id="aaf0980c"></a>border-left-width
  - <a id="3a4a9318"></a>border-radius
  - <a id="47e9abf9"></a>border-right-width
  - <a id="b6bb4b13"></a>border-style
  - <a id="051410c7"></a>border-top-width
  - <a id="064303ba"></a>border-width
  - <a id="1c96a88c"></a>none
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="bcdf9b19"></a>color
  - <a id="a42c65ac"></a>currentcolor
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-GAPS-1\] defines the following terms:
  - <a id="96286e4e"></a>column-rule-width
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="e9e2f325"></a>concrete object size
  - <a id="c82a1380"></a>default object size
  - <a id="af7c36a6"></a>default sizing algorithm
  - <a id="ffedca23"></a>natural aspect ratio
  - <a id="c0cc78c8"></a>natural size
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="d9b4880c"></a>clip
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
  - <a id="855a7562"></a>visible
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="b81d0a11"></a>CSSPseudoElement
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="ef9f8297"></a>\*
  - <a id="8cd4f032"></a>,
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="699488a8"></a>\<url\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4f460096"></a>snap as a border width
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="8a5584f2"></a>left
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="e112902f"></a>end
- \[CSS2\] defines the following terms:
  - <a id="2e21b9cc"></a>auto
  - <a id="c6866721"></a>border edge
  - <a id="f42088a1"></a>bottom
  - <a id="ddc96bb0"></a>content height
  - <a id="e295344d"></a>content width
  - <a id="ddbfe589"></a>display
  - <a id="b43f2834"></a>height
  - <a id="faacc054"></a>left
  - <a id="4f2fe4c4"></a>margin-left
  - <a id="7196e5b8"></a>margin-top
  - <a id="0f0ab49f"></a>max-height
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
  - <a id="a716e62e"></a>padding-bottom
  - <a id="c2154181"></a>padding-left
  - <a id="f1a194a8"></a>padding-right
  - <a id="db86fe38"></a>padding-top
  - <a id="7818f443"></a>right
  - <a id="e4729cc6"></a>top
  - <a id="1b9ee727"></a>visibility
  - <a id="49c5c029"></a>width
- \[HTML\] defines the following terms:
  - <a id="0fc40460"></a>canvas
  - <a id="87fcd40c"></a>iframe
  - <a id="f0811ff8"></a>img
  - <a id="99b5cef6"></a>object
  - <a id="29399d44"></a>picture
  - <a id="aa7bbf63"></a>video
- \[SVG2\] defines the following terms:
  - <a id="45c278c3"></a>svg
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="d376a2a1"></a>animation type

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 31 March 2026. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 25 March 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-png"></a>\[PNG\]  
Chris Lilley; et al. [Portable Network Graphics (PNG) Specification (Third Edition)](https://www.w3.org/TR/png-3/). 24 June 2025. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;png-3&#x2F;](https://www.w3.org/TR/png-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-uax29"></a>\[UAX29\]  
Josh Hadley. [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/tr29-47.html). 17 August 2025. Unicode Standard Annex \#29. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;reports&#x2F;tr29&#x2F;tr29-47&#x2E;html](https://www.unicode.org/reports/tr29/tr29-47.html)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-gaps-1"></a>\[CSS-GAPS-1\]  
Kevin Babbitt. [CSS Gap Decorations Module Level 1](https://www.w3.org/TR/css-gaps-1/). 27 February 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-gaps-1&#x2F;](https://www.w3.org/TR/css-gaps-1/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css1"></a>\[CSS1\]  
Håkon Wium Lie; Bert Bos. [Cascading Style Sheets, level 1](https://www.w3.org/TR/CSS1/). 13 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS1&#x2F;](https://www.w3.org/TR/CSS1/)

<a id="biblio-css4-images"></a>\[CSS4-IMAGES\]  
Elika Etemad; Tab Atkins Jr.; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 30 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                       | Initial                   | Applies to                                                                                                                                | Inh. | %ages | Anim­ation type            | Canonical order | Com­puted value                                                                                                                                                     |
|---------------------|-----------------------------------------------------------------------------|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------|------|-------|---------------------------|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-box-sizing⑨"></a></span><a href="#propdef-box-sizing">box-sizing</a>&#xA;      </strong> | content-box \| border-box                                                   | content-box               | all elements that accept width or height                                                                                                  | no   | N/A   | discrete                  | per grammar     | specified value                                                                                                                                                    |
| <strong><span><a id="ref-for-propdef-caret-color②"></a></span><a href="#propdef-caret-color">caret-color</a>&#xA;      </strong> | auto \| \<color\>                                                           | auto                      | all elements                                                                                                                              | yes  | N/A   | colorby computed value    | per grammar     | The computed value for auto is auto; the computed value of currentColor is currentColor (See currentcolor); see the color property for other \<color\> values.     |
| <strong><span><a id="ref-for-propdef-cursor①⓪"></a></span><a href="#propdef-cursor">cursor</a>&#xA;      </strong> | \[\<cursor-image\>,\]\* \<cursor-predefined\>                               | auto                      | all elements                                                                                                                              | yes  | N/A   | discrete                  | per grammar     | as specified, except with any relative URLs converted to absolute                                                                                                  |
| <strong><span><a id="ref-for-propdef-outline③"></a></span><a href="#propdef-outline">outline</a>&#xA;      </strong> | \[ \<'outline-color'\> \|\| \<'outline-style'\> \|\| \<'outline-width'\> \] | see individual properties | all elements                                                                                                                              | no   | N/A   | see individual properties | per grammar     | see individual properties                                                                                                                                          |
| <strong><span><a id="ref-for-propdef-outline-color⑥"></a></span><a href="#propdef-outline-color">outline-color</a>&#xA;      </strong> | \<color\> \| invert                                                         | invert                    | all elements                                                                                                                              | no   | N/A   | colorby computed value    | per grammar     | The computed value for invert is invert; the computed value of currentColor is currentColor (See currentcolor); see the color property for other \<color\> values. |
| <strong><span><a id="ref-for-propdef-outline-offset②"></a></span><a href="#propdef-outline-offset">outline-offset</a>&#xA;      </strong> | \<length\>                                                                  | 0                         | all elements                                                                                                                              | no   | N/A   | lengthby computed value   | per grammar     | \<length\> value in absolute units (px or physical).absolute length                                                                                                |
| <strong><span><a id="ref-for-propdef-outline-style⑧"></a></span><a href="#propdef-outline-style">outline-style</a>&#xA;      </strong> | auto \| \<border-style\>                                                    | none                      | all elements                                                                                                                              | no   | N/A   | discrete                  | per grammar     | as specified                                                                                                                                                       |
| <strong><span><a id="ref-for-propdef-outline-width⑨"></a></span><a href="#propdef-outline-width">outline-width</a>&#xA;      </strong> | \<line-width\>                                                              | medium                    | all elements                                                                                                                              | no   | N/A   | lengthby computed value   | per grammar     | absolute length, snapped as a border width; 0 if the outline style is none.                                                                                        |
| <strong><span><a id="ref-for-propdef-resize⑨"></a></span><a href="#propdef-resize">resize</a>&#xA;      </strong> | none \| both \| horizontal \| vertical                                      | none                      | elements with overflow other than visible,that are scroll containers and optionally replaced elements such as images, videos, and iframes | no   | N/A   | discrete                  | per grammar     | as specified keyword                                                                                                                                               |
| <strong><span><a id="ref-for-propdef-text-overflow④"></a></span><a href="#propdef-text-overflow">text-overflow</a>&#xA;      </strong> | clip \| ellipsis                                                            | clip                      | block containers                                                                                                                          | no   | N/A   | discrete                  | per grammar     | as specified keyword                                                                                                                                               |

