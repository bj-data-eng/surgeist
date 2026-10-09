Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Compositing and Blending Level 1](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Compositing and Blending Level 1

Source snapshot: https://www.w3.org/TR/2024/CRD-compositing-1-20240321/

Snapshot SHA-256: 0fb91119308e706a5e4df368a91fa3b43cd5fe8cd8f192418b2df0a30ba8a62b

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 6 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions, 1 already-readable table. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# Compositing and Blending Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

<a id="ref-for-dom-context-2d-globalcompositeoperation"></a>

## <a id="abstract"></a>Abstract

<a id="ref-for-backdrop"></a>

Compositing describes how shapes of different elements are combined into a single image. There are various possible approaches for compositing. Previous versions of SVG and CSS used [Simple Alpha Compositing](https://www.w3.org/TR/SVG11/masking.html#SimpleAlphaBlending). In this model, each element is rendered into its own buffer and is then merged with its [backdrop](#backdrop) using the Porter Duff source-over operator. This specification will define a new compositing model that expands upon the Simple Alpha Compositing model by offering:

- additional Porter Duff compositing operators
- advanced blending modes which allow control of how colors mix in the areas where shapes overlap
- compositing groups

In addition, this specification will define CSS properties for blending and group isolation and the properties of the <code><a href="https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-globalcompositeoperation">globalCompositeOperation</a></code> attribute. [CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “compositing” in the title, like this: “\[compositing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcompositing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="introduction"></a>1. Introduction

<em>This subsection is non-normative.</em>  
The first part of this document describes the properties used to control the compositing in CSS. The second part will describe the algorithms of Porter Duff compositing and blending.

## <a id="reading-this-document"></a>2. Reading This Document

Each section of this document is normative unless otherwise specified.

### <a id="module-interactions"></a>2.1. Module interactions

This specification defines a set of CSS properties that affect the visual rendering of elements to which those properties are applied; these effects are applied after elements have been sized and positioned according to the [Visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) from [\[CSS21\]](#biblio-css21). Some values of these properties result in the creation of a [containing block](https://www.w3.org/TR/CSS2/visuren.html#containing-block), and/or the creation of a [stacking context](https://www.w3.org/tr/css21/visuren.html#x43).

<a id="ref-for-propdef-background-blend-mode"></a>

The [background-blend-mode](#propdef-background-blend-mode) property also builds upon the properties defined in the [CSS Backgrounds and Borders](https://www.w3.org/TR/css3-background/#placement) module [\[CSS3BG\]](#biblio-css3bg).

This specification also enhances the rules as specified in [Section 14.2 Simple alpha compositing](https://www.w3.org/TR/2003/REC-SVG11-20030114/masking.html#SimpleAlphaBlending) of [\[SVG11\]](#biblio-svg11) and [simple alpha compositing](https://www.w3.org/TR/css-color-4/#alpha) of [\[CSS-COLOR-4\]](#biblio-css-color-4).

<a id="ref-for-dom-context-2d-globalcompositeoperation①"></a>

This module also extends the <code><a href="https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-globalcompositeoperation">globalCompositeOperation</a></code> attribute.

### <a id="values"></a>2.2. Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#biblio-css21). Value types not defined in this specification are defined in CSS Level 2 Revision 1 \[CSS21\]. Other CSS modules may expand the definitions of these value types: for example [\[CSS-COLOR-4\]](#biblio-css-color-4), when combined with this module, expands the definition of the \<color\> value type as used in this specification.

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [inherit](https://www.w3.org/TR/CSS21/cascade.html#value-def-inherit) keyword as their property value. For readability it has not been repeated explicitly.

## <a id="csscompositingandblending"></a>3. Specifying Blending in CSS 

### <a id="compositingandblendingorder"></a>3.1. Order of graphical operations

The compositing model must follow the [SVG compositing](https://www.w3.org/TR/SVG11/render.html#Introduction) model [\[SVG11\]](#biblio-svg11): first any filter effect is applied, then any clipping, masking, blending and compositing.

### <a id="csscompositingrules_CSS"></a>3.2. Behavior specific to HTML

Everything in CSS that creates a [stacking context](https://www.w3.org/TR/CSS21/zindex.html) must be considered an [‘isolated’ group](#isolatedgroups). HTML elements themselves should not create groups.

An element that has blending applied, must blend with all the underlying content of the [stacking context](https://www.w3.org/tr/css21/visuren.html#x43) [\[CSS21\]](#biblio-css21) that that element belongs to.

The root element for an HTML document is the [document element](https://dom.spec.whatwg.org/#document-element).

### <a id="csscompositingrules_SVG"></a>3.3. Behavior specific to SVG

By default, every element must create a [non-isolated group](#isolatedgroups).

However, certain operations in SVG will create [isolated groups](#isolatedgroups). If one of the following features is used, the group must become isolated:

- opacity
- filters
- 3D transforms (2D transforms must NOT cause isolation)
- blending
- masking

The root element for SVG is the [SVG element](https://www.w3.org/TR/SVG11/struct.html#NewDocument).

### <a id="csskeywords"></a>3.4. CSS Properties

<a id="ref-for-propdef-mix-blend-mode"></a>

#### <a id="mix-blend-mode"></a>3.4.1. The [mix-blend-mode](#propdef-mix-blend-mode) property

The blend mode defines the formula that must be used to mix the colors with the backdrop. This behavior is described in more detail in [Blending](#blending).

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mix-blend-mode"></a>mix-blend-mode                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-ltblendmodegt"></a>[\<blend-mode\>](#ltblendmodegt)                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG11/intro.html#TermContainerElement), [graphics elements](https://www.w3.org/TR/SVG11/intro.html#TermGraphicsElement) and [graphics referencing elements](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermGraphicsReferencingElement). [\[SVG11\]](#biblio-svg11) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                        |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                          |

<a id="ref-for-ltblendmodegt①"></a>

The syntax of the property of [\<blend-mode\>](#ltblendmodegt) is given with:

<a id="ltblendmodegt"></a>

<a id="ref-for-valdef-blend-mode-normal"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-valdef-blend-mode-darken"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-valdef-blend-mode-multiply"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-valdef-blend-mode-color-burn"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-valdef-blend-mode-lighten"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-valdef-blend-mode-screen"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-valdef-blend-mode-color-dodge"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-valdef-blend-mode-overlay"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-valdef-blend-mode-soft-light"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-valdef-blend-mode-hard-light"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-valdef-blend-mode-difference"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-valdef-blend-mode-exclusion"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-valdef-blend-mode-hue"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-valdef-blend-mode-saturation"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-valdef-blend-mode-color"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-valdef-blend-mode-luminosity"></a>

```text
<blend-mode> =
  normal |
  darken | multiply | color-burn |
  lighten | screen | color-dodge |
  overlay | soft-light | hard-light |
  difference | exclusion |
  hue | saturation | color | luminosity
```
<a id="ref-for-valdef-blend-mode-normal①"></a>

Applying a blendmode other than [normal](#valdef-blend-mode-normal) to the element must establish a new [stacking context](https://www.w3.org/tr/css21/visuren.html#x43) [\[CSS21\]](#biblio-css21). This group must then be blended and composited with the [stacking context](https://www.w3.org/tr/css21/visuren.html#x43) that contains the element.

Tests

- Blending_in_a_group_with_filter.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/Blending_in_a_group_with_filter.html)
- Blending_in_a_group_with_opacity.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/Blending_in_a_group_with_opacity.html)
- Text_with_SVG_background.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/Text_with_SVG_background.html)
- [inheritance.html](https://wpt.fyi/results/css/compositing/inheritance.html) [(live test)](http://wpt.live/css/compositing/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/inheritance.html)
- line-with-svg-background.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/line-with-svg-background.html)
- [mix-blend-mode-in-svg-image.html](https://wpt.fyi/results/css/compositing/svg/mix-blend-mode-in-svg-image.html) [(live test)](http://wpt.live/css/compositing/svg/mix-blend-mode-in-svg-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/svg/mix-blend-mode-in-svg-image.html)
- [mix-blend-mode-svg-rectangle.html](https://wpt.fyi/results/css/compositing/svg/mix-blend-mode-svg-rectangle.html) [(live test)](http://wpt.live/css/compositing/svg/mix-blend-mode-svg-rectangle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/svg/mix-blend-mode-svg-rectangle.html)
- text-with-svg-background.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/text-with-svg-background.html)
- [mix-blend-mode-animation.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-animation.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-animation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-animation.html)
- [mix-blend-mode-blended-element-interposed.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-interposed.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-interposed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-interposed.html)
- [mix-blend-mode-blended-element-overflow-hidden-and-border-radius.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-hidden-and-border-radius.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-hidden-and-border-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-hidden-and-border-radius.html)
- [mix-blend-mode-blended-element-overflow-scroll.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-scroll.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-overflow-scroll.html)
- [mix-blend-mode-blended-element-with-transparent-pixels.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-with-transparent-pixels.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-with-transparent-pixels.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-element-with-transparent-pixels.html)
- [mix-blend-mode-blended-with-3D-transform.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-3D-transform.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-3D-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-3D-transform.html)
- [mix-blend-mode-blended-with-transform-and-perspective.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-transform-and-perspective.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-transform-and-perspective.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blended-with-transform-and-perspective.html)
- [mix-blend-mode-blending-with-sibling.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-blending-with-sibling.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-blending-with-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-blending-with-sibling.html)
- [mix-blend-mode-border-image.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-border-image.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-border-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-border-image.html)
- [mix-blend-mode-both-parent-and-blended-with-3D-transform.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-both-parent-and-blended-with-3D-transform.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-both-parent-and-blended-with-3D-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-both-parent-and-blended-with-3D-transform.html)
- [mix-blend-mode-canvas-parent.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-canvas-parent.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-canvas-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-canvas-parent.html)
- [mix-blend-mode-canvas-sibling.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-canvas-sibling.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-canvas-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-canvas-sibling.html)
- [mix-blend-mode-creates-stacking-context.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-creates-stacking-context.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-creates-stacking-context.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-creates-stacking-context.html)
- [mix-blend-mode-filter.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-filter.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-filter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-filter.html)
- [mix-blend-mode-iframe-parent.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-iframe-parent.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-iframe-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-iframe-parent.html)
- [mix-blend-mode-iframe-sibling.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-iframe-sibling.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-iframe-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-iframe-sibling.html)
- [mix-blend-mode-image.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-image.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-image.html)
- [mix-blend-mode-intermediate-element-overflow-hidden-and-border-radius.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-intermediate-element-overflow-hidden-and-border-radius.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-intermediate-element-overflow-hidden-and-border-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-intermediate-element-overflow-hidden-and-border-radius.html)
- [mix-blend-mode-mask.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-mask.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-mask.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-mask.html)
- [mix-blend-mode-overflowing-child-of-blended-element.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child-of-blended-element.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child-of-blended-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child-of-blended-element.html)
- [mix-blend-mode-overflowing-child.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-overflowing-child.html)
- [mix-blend-mode-paragraph-background-image.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-paragraph-background-image.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-paragraph-background-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-paragraph-background-image.html)
- [mix-blend-mode-paragraph.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-paragraph.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-paragraph.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-paragraph.html)
- [mix-blend-mode-parent-element-overflow-hidden-and-border-radius.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-hidden-and-border-radius.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-hidden-and-border-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-hidden-and-border-radius.html)
- [mix-blend-mode-parent-element-overflow-scroll-blended-position-fixed.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll-blended-position-fixed.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll-blended-position-fixed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll-blended-position-fixed.html)
- [mix-blend-mode-parent-element-overflow-scroll.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-element-overflow-scroll.html)
- mix-blend-mode-parent-with-3D-transform-and-transition.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-3D-transform-and-transition.html)
- [mix-blend-mode-parent-with-3D-transform.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-3D-transform.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-3D-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-3D-transform.html)
- [mix-blend-mode-parent-with-border-radius.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-border-radius.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-border-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-border-radius.html)
- [mix-blend-mode-parent-with-text.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-text.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-text.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parent-with-text.html)
- [mix-blend-mode-parsing.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-parsing.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-parsing.html)
- [mix-blend-mode-plus-lighter-basic.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-basic.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-basic.html)
- [mix-blend-mode-plus-lighter-svg-basic.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg-basic.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg-basic.html)
- [mix-blend-mode-plus-lighter-svg.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter-svg.html)
- [mix-blend-mode-plus-lighter.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-plus-lighter.html)
- [mix-blend-mode-root-element-group.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-root-element-group.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-root-element-group.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-root-element-group.html)
- [mix-blend-mode-rotated-clip.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-rotated-clip.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-rotated-clip.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-rotated-clip.html)
- [mix-blend-mode-script.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-script.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-script.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-script.html)
- mix-blend-mode-sibling-with-3D-transform-and-transition.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-sibling-with-3D-transform-and-transition.html)
- [mix-blend-mode-sibling-with-3D-transform.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-sibling-with-3D-transform.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-sibling-with-3D-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-sibling-with-3D-transform.html)
- [mix-blend-mode-simple.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-simple.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-simple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-simple.html)
- [mix-blend-mode-stacking-context-001.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-001.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-001.html)
- [mix-blend-mode-stacking-context-creates-isolation.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-creates-isolation.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-creates-isolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-stacking-context-creates-isolation.html)
- [mix-blend-mode-svg.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-svg.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-svg.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-svg.html)
- [mix-blend-mode-video-sibling.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-video-sibling.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-video-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-video-sibling.html)
- [mix-blend-mode-video.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-video.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-video.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-video.html)
- [mix-blend-mode-with-transform-and-preserve-3D.html](https://wpt.fyi/results/css/compositing/mix-blend-mode/mix-blend-mode-with-transform-and-preserve-3D.html) [(live test)](http://wpt.live/css/compositing/mix-blend-mode/mix-blend-mode-with-transform-and-preserve-3D.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/mix-blend-mode/mix-blend-mode-with-transform-and-preserve-3D.html)
- [mix-blend-mode-computed.html](https://wpt.fyi/results/css/compositing/parsing/mix-blend-mode-computed.html) [(live test)](http://wpt.live/css/compositing/parsing/mix-blend-mode-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/mix-blend-mode-computed.html)
- [mix-blend-mode-invalid.html](https://wpt.fyi/results/css/compositing/parsing/mix-blend-mode-invalid.html) [(live test)](http://wpt.live/css/compositing/parsing/mix-blend-mode-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/mix-blend-mode-invalid.html)
- [mix-blend-mode-valid.html](https://wpt.fyi/results/css/compositing/parsing/mix-blend-mode-valid.html) [(live test)](http://wpt.live/css/compositing/parsing/mix-blend-mode-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/mix-blend-mode-valid.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-duck-on-lime"></a>
>
> Given the following sample markup:
>
> ```text
> <body>
>   <img src="ducky.png"/>
> </body>
> ```
>
> And the following style rule:
>
> ```text
> body { background-color: lime; }
> ```
>
> ... will produce the following result:
>
> ![example of an image of duck with the lime color of the document](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/ducky_normal.png)
>
> Partially transparent image on a lime backdrop
>
> If we change the style rule to include blending:
>
> ```text
> body { background-color: lime; }
> img { mix-blend-mode: multiply; }
> ```
>
> ... the output will be the image blending with the lime background of the \<body\> element.
>
> ![example of an image of duck blending with the lime color of the document](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/ducky_multiply.png)
>
> Blending of a transparent image on a lime backdrop.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-svg-screen-mode"></a>
>
> Given the following svg code:
>
> ```text
> <svg>
>   <circle cx="40" cy="40" r="40" fill="red"/>
>   <circle cx="80" cy="40" r="40" fill="lime"/>
>   <circle cx="60" cy="80" r="40" fill="blue"/>
> </svg>
> ```
>
> And the following style rule:
>
> ```text
> circle { mix-blend-mode: screen; }
> ```
>
> ... the output will be blending of the 3 circles. Each circle is rendered from bottom to top. Where the elements overlap, the blend mode produces a change in color.
>
> ![example of 3 circles blending with a screen blend mode](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/screen_example.svg)
>
> Example of 3 circles blending with a screen blend mode

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-opacity-stacking"></a>
>
> In the following style sheet and document fragment:
>
> ```text
> body { background-color: lime; }
> div { background-color: red; width: 200px; opacity: .95}
> img { mix-blend-mode: difference; }
> 
> <body>
>   <div>
>      <img src="ducky.png"/>
>   </div>
> </body>
> ```
>
> <a id="ref-for-propdef-opacity"></a>
>
> ... the [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) on the \<div\> element is causing the creation of a stacking context. This causes the creation of a new group so the image doesn’t blend with the color of the \<body\>.
>
> ![example of an image of duck blending with the lime color of the document](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/ducky_difference.png)
>
> Example of blending within a stacking context.
>
> Note how the image is not blending with the lime color.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-overlay-blend"></a>
>
> Given the following sample markup:
>
> <a id="ref-for-valdef-blend-mode-overlay①"></a>
>
> ```text
> <body>
>    <div>
>      <p>overlay blending on text</p>
>    </div>
> </body>
> ```
>
> And the following style rule:
>
> ```text
> div { background-image: url('texture.png'); }
> @font-face {
>   font-family: "Mythos Std";
>   src: url("http://myfontvendor.com/mythos.otf");
> }
> p {
>   mix-blend-mode: overlay;
>   font-family: "Mythos Std"
> }
> ```
>
> ![example of text that has an overlay blend on top of a texture](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/overlay_text.png)
>
> Text with a blend overlay on top of an image.

<a id="ref-for-propdef-isolation"></a>

#### <a id="isolation"></a>3.4.2. The [isolation](#propdef-isolation) property

<a id="ref-for-propdef-isolation①"></a>

In SVG, this defines whether an element is isolated or not.  
For CSS, setting [isolation](#propdef-isolation) to isolate will turn the element into a stacking context.

By default, elements use the auto keyword which implies that they are not isolated. However operations that cause the creation of stacking context [\[CSS21\]](#biblio-css21) must cause a group to be isolated. These operations are described in ['behavior specific to HTML'](#csscompositingrules_CSS) and ['behavior specific to SVG'](#csscompositingrules_SVG).

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-isolation"></a>isolation                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-isolated-propid"></a>[\<isolation-mode\>](#isolated-propid)                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG11/intro.html#TermContainerElement), [graphics elements](https://www.w3.org/TR/SVG11/intro.html#TermGraphicsElement) and [graphics referencing elements](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermGraphicsReferencingElement). [\[SVG11\]](#biblio-svg11) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                        |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                          |

<a id="ref-for-isolated-propid①"></a>

The syntax of the property of [\<isolation-mode\>](#isolated-propid) is given with:

<a id="isolated-propid"></a>

<a id="ref-for-comb-one①⑤"></a>

```text
<isolation-mode> = auto | isolate
```
Tests

- [inheritance.html](https://wpt.fyi/results/css/compositing/inheritance.html) [(live test)](http://wpt.live/css/compositing/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/inheritance.html)
- [isolation-no-interpolation.html](https://wpt.fyi/results/css/compositing/isolation/animation/isolation-no-interpolation.html) [(live test)](http://wpt.live/css/compositing/isolation/animation/isolation-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/isolation/animation/isolation-no-interpolation.html)
- blend-isolation.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/isolation/blend-isolation.html)
- [isolation-computed.html](https://wpt.fyi/results/css/compositing/parsing/isolation-computed.html) [(live test)](http://wpt.live/css/compositing/parsing/isolation-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/isolation-computed.html)
- [isolation-invalid.html](https://wpt.fyi/results/css/compositing/parsing/isolation-invalid.html) [(live test)](http://wpt.live/css/compositing/parsing/isolation-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/isolation-invalid.html)
- [isolation-valid.html](https://wpt.fyi/results/css/compositing/parsing/isolation-valid.html) [(live test)](http://wpt.live/css/compositing/parsing/isolation-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/isolation-valid.html)

<a id="ref-for-the-img-element"></a>

<a id="img_isolation"></a> In CSS, a background image or the content of an [img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element) must always be rendered into an isolated group.  
For instance, if you link to an SVG file through the <a id="ref-for-the-img-element①"></a>img tag, the artwork of that SVG will not blend with the backdrop of the content.

<a id="ref-for-propdef-mask"></a>

In SVG, [mask](https://www.w3.org/TR/css-masking-1/#propdef-mask) always creates an isolated group.

<a id="ref-for-propdef-background-blend-mode①"></a>

#### <a id="background-blend-mode"></a>3.4.3. The [background-blend-mode](#propdef-background-blend-mode) property

Defines the blending mode of each background layer.

Each background layer must blend with the element’s background layer that is below it and the element’s background color. Background layers must not blend with the content that is behind the element, instead they must act as if they are rendered into an isolated group.

<a id="ref-for-propdef-background-blend-mode②"></a>

The description of the [background-blend-mode](#propdef-background-blend-mode) property is as follows:

| Field               | Definition                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-blend-mode"></a>background-blend-mode                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-ltblendmodegt②"></a>[\<blend-mode\>](#ltblendmodegt)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | All HTML elements                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                   |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                     |

Tests

- [inheritance.html](https://wpt.fyi/results/css/compositing/inheritance.html) [(live test)](http://wpt.live/css/compositing/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/inheritance.html)
- [background-blend-mode-gradient-image.html](https://wpt.fyi/results/css/compositing/background-blending/background-blend-mode-gradient-image.html) [(live test)](http://wpt.live/css/compositing/background-blending/background-blend-mode-gradient-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/background-blending/background-blend-mode-gradient-image.html)
- [background-blend-mode-plus-lighter.html](https://wpt.fyi/results/css/compositing/background-blending/background-blend-mode-plus-lighter.html) [(live test)](http://wpt.live/css/compositing/background-blending/background-blend-mode-plus-lighter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/background-blending/background-blend-mode-plus-lighter.html)
- [background-blend-mode-computed-multiple.html](https://wpt.fyi/results/css/compositing/parsing/background-blend-mode-computed-multiple.html) [(live test)](http://wpt.live/css/compositing/parsing/background-blend-mode-computed-multiple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/background-blend-mode-computed-multiple.html)
- [background-blend-mode-computed.html](https://wpt.fyi/results/css/compositing/parsing/background-blend-mode-computed.html) [(live test)](http://wpt.live/css/compositing/parsing/background-blend-mode-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/background-blend-mode-computed.html)
- [background-blend-mode-invalid.html](https://wpt.fyi/results/css/compositing/parsing/background-blend-mode-invalid.html) [(live test)](http://wpt.live/css/compositing/parsing/background-blend-mode-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/background-blend-mode-invalid.html)
- [background-blend-mode-valid.html](https://wpt.fyi/results/css/compositing/parsing/background-blend-mode-valid.html) [(live test)](http://wpt.live/css/compositing/parsing/background-blend-mode-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/parsing/background-blend-mode-valid.html)

<a id="ref-for-propdef-background-blend-mode③"></a>

<a id="ref-for-propdef-background-image"></a>

The [background-blend-mode](#propdef-background-blend-mode) list must be applied in the same order as [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) [\[CSS3BG\]](#biblio-css3bg). This means that the first element in the list will apply to the layer that is on top. If a property doesn’t have enough comma-separated values to match the number of layers, the UA must calculate its used value by repeating the list of values until there are enough.

<a id="ref-for-propdef-background"></a>

<a id="ref-for-propdef-background-blend-mode④"></a>

If the [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) [\[CSS3BG\]](#biblio-css3bg) shorthand is used, the [background-blend-mode](#propdef-background-blend-mode) property for that element must be reset to its initial value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-background-blend"></a>
>
> Given the following sample markup:
>
> ```text
> <body>
>    <div></div>
> </body>
> ```
>
> And the following style rule:
>
> ```text
> body { background-color: lime; }
> div {
>   width: 200px;
>   height: 200px;
>   background-size: 200px 200px;
>   background-repeat:no-repeat;
>   background-image: linear-gradient(to right, #000000 0%,#ffffff 100%), url('ducky.png');
>   background-blend-mode: difference, normal;
> }
> ```
>
> ![example of div that has an image of a duck and a gradient that is blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/ducky_background_blend.png)
>
> Blending of 2 background images.
>
> <a id="ref-for-the-body-element"></a>
>
> Note that the gradient is not blending with the color of [body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element). Instead it retains its original color.

## <a id="canvascompositingandblending"></a>4. Specifying Compositing and Blending in Canvas 2D

<a id="ref-for-dom-context-2d-globalcompositeoperation②"></a>

The [canvas 2d](https://html.spec.whatwg.org/multipage/canvas.html#2dcontext) context defines the <code><a href="https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-globalcompositeoperation">globalCompositeOperation</a></code> attribute that is used to set the current compositing and blending operator.

This property takes the following value:

<a id="propdef-mix"></a>‘globalCompositeOperation’  

| Column 1              | Column 2                                                                                                        |
|-----------------------|-----------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | <a id="ref-for-compositemode"></a><a id="ref-for-ltblendmodegt③"></a> [\<blend-mode\>](#ltblendmodegt) \| [\<composite-mode\>](#compositemode) |
| <em>Initial:</em>   | source-over                                                                                                     |

<a id="ref-for-compositemode①"></a>

The syntax of the property of [\<composite-mode\>](#compositemode) is given with:

<a id="compositemode"></a>

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

```text
<composite-mode> = clear | copy | source-over | destination-over | source-in |    destination-in | source-out | destination-out | source-atop |    destination-atop | xor | lighter
```
Tests

- [canvas-composite-modes.html](https://wpt.fyi/results/css/compositing/canvas-composite-modes.html) [(live test)](http://wpt.live/css/compositing/canvas-composite-modes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/canvas-composite-modes.html)

## <a id="whatiscompositing"></a>5. Introduction to compositing

Compositing is the combining of a graphic element with its backdrop.

In the model described in this specification there are two steps to the overall compositing operation - [Porter-Duff compositing](#advancedcompositing) and [blending](#blending). The blending step determines how the colors from the graphic element and the backdrop interact.

<a id="ref-for-backdrop①"></a>

Typically, the blending step is performed first, followed by the Porter-Duff compositing step. In the blending step, the resultant color from the mix of the element and the the [backdrop](#backdrop) is calculated. The graphic element’s color is replaced with this resultant color. The graphic element is then composited with the <a id="ref-for-backdrop②"></a>backdrop using the specified compositing operator.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Shape is defined by the mathematical description of the shape. A particular point is either inside the shape or it is not. There are no gradations.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Opacity is described using an alpha value, stored alongside the color value for each particular point. The alpha value is between 0 and 1, inclusive. A value of 0 means that the pixel has no coverage at that point, and is therefore transparent; i.e. there is no color contribution from any geometry because the geometry does not overlap this pixel. A value of 1 means that the pixel is fully opaque; the geometry completely overlaps the pixel.

### <a id="simplealphacompositing"></a>5.1. Simple alpha compositing

The formula for simple alpha compositing is

```text
co = Cs x αs + Cb x αb x (1 - αs)
```
Where

- co: the premultiplied pixel value after compositing

- Cs: the color value of the source graphic element being composited

- αs: the alpha value of the source graphic element being composited

- <a id="ref-for-backdrop③"></a>

  Cb: the color value of the [backdrop](#backdrop)

- <a id="ref-for-backdrop④"></a>

  αb: the alpha value of the [backdrop](#backdrop)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All values are between 0 and 1 inclusive.

The pixel value after compositing (co) is given by adding the contributions from the source graphic element \[Cs x αs\] and the backdrop \[Cb x αb x (1 - αs)\]. For both the graphic element and the backdrop, the color values are multiplied by the alpha to determine the amount of color that contributes. With zero alpha meaning that the color does not contribute and partial alpha means that some percentage of the color contributes. The contribution of the backdrop is further reduced based on the opacity of the graphic element. Conceptually, (1 - αs) of the backdrop shows through the graphic element, meaning that if the graphic element is fully opaque (αs=1) then no backdrop shows through.

The simple alpha compositing formula listed above gives a resultant color which is the result of the weighted average of the backdrop color and graphic element color, with the weighting determined by the backdrop and graphic element alphas. The resultant alpha value of the composite is simply the sum of the contributed alpha of the composited elements. The formula for the resultant alpha of the composite is

```text
αo = αs + αb x (1 - αs)
```
Where

- αo: the alpha value of the composite

- αs: the alpha value of the graphic element being composited

- <a id="ref-for-backdrop⑤"></a>

  αb: the alpha value of the [backdrop](#backdrop)

> <strong data-conversion-semantic="note">Note</strong>
>
> Often, it can be more efficient to store a `pre-multiplied value` for the color and opacity. The pre-multiplied value is given by
>
> ```text
> cs = Cs x αs
> ```
>
> with
>
> - cs: the pre-multiplied value
> - Cs: the color value
> - αs: the alpha value
>
> Thus the formula for simple alpha compositing using pre-multiplied values becomes
>
> ```text
> co = cs + cb x (1 - αs)
> ```
>
> To extract the color component of a pre-multiplied value, the formula is reversed:
>
> ```text
> Co = co / αo
> ```
#### <a id="simplealphacompositingexamples"></a>5.1.1. Examples of simple alpha compositing

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-basic-alpha"></a>
>
> ![Simple box showing alpha compositing](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/simple_box.svg)
>
> This describes the most basic case. It consists of 1 shape that is filled with a solid color (α = 1). The shape is composited with an empty background. The empty background has no effect on the resultant composite.
>
> ```text
> Cs = RGB(1,0,0)
> αs = 1
> Cb = RGB(0,0,0)
> αb = 0
> 
> co = Cs x αs + Cb x αb x (1 - αs)
> co = RGB(1,0,0) x 1 + RGB(0,0,0) x 0 x (1 - 1)
> co = RGB(1,0,0) x 1
> co = RGB(1,0,0)
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-shapes-intersect"></a>
>
> ![simple shape](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/two_box.svg)
>
> This is a more complex example. There is no transparency, but the 2 shapes intersect.
>
> Applying the compositing formula in the area of intersection, gives:
>
> ```text
> Cs = RGB(0,0,1)
> αs = 1
> Cb = RGB(1,0,0)
> αb = 1
> 
> co = Cs x αs + Cb x αb x (1 - αs)
> co = RGB(0,0,1) x 1 + RGB(1,0,0) x 1 x (1 - 1)
> co = RGB(0,0,1) x 1 + RGB(1,0,0) x 1 x 0
> co = RGB(0,0,1) x 1
> co = RGB(0,0,1)
> ```
>
> Calculating the alpha of the resultant composite
>
> ```text
> αo = αs + αb x (1 - αs)
> αo = 1 + 1 x (1 - 1)
> αo = 1
> ```
>
> Calculating the color component of the resultant composite
>
> ```text
> Co = co / αo
> Co = RGB(0, 0, 1) / 1
> Co = RGB(0, 0, 1)
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-transparent-over-opaque"></a>
>
> ![interaction of a solid box with a transparent box on top](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/two_box_transparency.svg)
>
> <a id="ref-for-backdrop⑥"></a>
>
> This is an example where the shape has some transparency, but the [backdrop](#backdrop) is fully opaque.
>
> Applying the compositing formula in the area of intersection, gives:
>
> ```text
> Cs = RGB(0,0,1)
> αs = 0.5
> Cb = RGB(1,0,0)
> αb = 1
> 
> co = Cs x αs + Cb x αb x (1 - αs)
> co = RGB(0,0,1) x 0.5 + RGB(1,0,0) x 1 x (1 - 0.5)
> co = RGB(0,0,1) x 0.5 + RGB(1,0,0) x 0.5
> co = RGB(0.5,0,0.5)
> ```
>
> Calculating the alpha of the resultant composite
>
> ```text
> αo = αs + αb x (1 - αs)
> αo = 0.5 + 1 x (1 - 0.5)
> αo = 1
> ```
>
> Calculating the color component of the resultant composite
>
> ```text
> Co = co / αo
> Co = RGB(0.5, 0, 0.5) / 1
> Co = RGB(0.5, 0, 0.5)
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-two-transparent"></a>
>
> ![interaction of 2 transparent boxes](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/two_box_transparency_both.svg)
>
> <a id="ref-for-backdrop⑦"></a>
>
> Figure 4 shows an example where both the shape and the [backdrop](#backdrop) are transparent.
>
> Applying the compositing formula in the area of intersection, gives:
>
> ```text
> Cs = RGB(0,0,1)
> αs = 0.5
> Cb = RGB(1,0,0)
> αb = 0.5
> 
> co = Cs x αs + Cb x αb x (1 - αs)
> co = RGB(0,0,1) x 0.5 + RGB(1,0,0) x 0.5 x (1 - 0.5)
> co = RGB(0,0,1) x 0.5 + RGB(1,0,0) x 0.25
> co = RGB(0.25, 0, 0.5)
> ```
>
> Calculating the alpha of the resultant composite
>
> ```text
> αo = αs + αb x (1 - αs)
> αo = 0.5 + 0.5 x (1 - 0.5)
> αo = 0.75
> ```
>
> Calculating the color component of the resultant composite
>
> ```text
> Co = co / αo
> Co = RGB(0.25, 0, 0.5) / 0.75
> Co = RGB(0.33, 0, 0.66)
> ```
## <a id="generalformula"></a>6. General Formula for Compositing and Blending

The general formula for compositing and blending which allows for selection of the compositing operator and blending function comprises two steps. The terms used in these functions will be described in detail in the following sections.

Apply the blend in place

```text
Cs = (1 - αb) x Cs + αb x B(Cb, Cs)
```
Composite

```text
Co = αs x Fa x Cs + αb x Fb x Cb
```
Where:

- <strong>Cs</strong>: is the source color
- <strong>Cb</strong>: is the backdrop color
- <strong>αs</strong>: is the source alpha
- <strong>αb</strong>: is the backdrop alpha
- <strong>B(Cb, Cs)</strong>: is the mixing function
- <strong>Fa</strong>: is defined by the Porter Duff operator in use
- <strong>Fb</strong>: is defined by the Porter Duff operator in use

## <a id="backdropCalc"></a>7. Backdrop calculation

The <a id="backdrop"></a>backdrop is the content behind the element and is what the element is composited with. This means that the backdrop is the result of compositing all previous elements.

### <a id="backdropexamples"></a>7.1. Examples of backdrop calculation

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-simple-backdrop-calculation"></a>
>
> ![example of a simple backdrop calculation](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/simple_backdrop.svg)
>
> This example has 2 simple shapes. The backdrop for the blue shape includes the bottom right corner of the red shape . The dotted line shows the area that is examined during compositing of the blue shape.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-backdrop-with-alpha"></a>
>
> ![example of a backdrop with alpha](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/simple_backdrop_alpha.svg)
>
> The shape in the backdrop has an alpha value. The alpha value of the backdrop shape is preserved when the backdrop is calculated.

## <a id="groups"></a>8. Compositing Groups

Compositing groups allow more control over the interaction of compositing with the backdrop. Groups can be used to specify how a compositing effect within a group will interact with the content that is already in the scene (the backdrop).

Compositing groups may be made up of any number of elements, and may contain other compositing groups.

The default properties of a compositing group shall cause no visual difference compared to having no group. See [Group Invariance](#groupinvariance).

A compositing group is rendered by first compositing the elements of the group onto the initial backdrop. The result of this is a single element containing color and alpha information. This element is then composited onto the group backdrop. Steps shall be taken to ensure the group backdrop makes only a single contribution to the final composite.

<a id="initialbackdrop"></a>initial backdrop  
The initial backdrop is the backdrop used for compositing the group’s first element. This will be the same as the group backdrop in a non-isolated group, or a fully transparent backdrop for an isolated group.

<a id="groupbackdrop"></a>group backdrop  
The group backdrop is the result of compositing all elements up to but not including the first element in the group.

### <a id="groupinvariance"></a>8.1. Group invariance

An important property of simple alpha compositing is its group invariance. This behavior is preserved in the more complex model described in this specification. Adding or removing grouping with default attributes shall not show visual differences.

```text
so: A + B + C = A + (B + C) = (A + B) + C
```
<a id="ref-for-valdef-blend-mode-normal②"></a>

When adding attributes to the group such as isolate, blending modes other than [normal](#valdef-blend-mode-normal) or Porter Duff compositing operators other than source-over, groups may no longer be invariant.

### <a id="isolatedgroups"></a>8.2. Isolated Groups

In an isolated group, the initial backdrop shall be black and fully transparent.

In this instance, the initial backdrop is different than the group backdrop. The only interaction with the group backdrop shall occur when the group’s computed color, shape and alpha are composited with it.

See '[Isolated groups and Porter Duff modes](#groupcompositing)' for a description of the effect of isolated groups on compositing. See '[Effect of group isolation on blending](#isolationblending)' for a description of the effect of isolated groups on blending.

### <a id="pagebackdrop"></a>8.3. The Root Element Group

The [isolated group](#isolatedgroups) for the root element is the root element group. All other elements and groups are composited into this group. The background of the root element (if specified) is painted into the root element group, and any filter, clip-path, mask and and opacity is then applied, before compositing into the [root group](#rootgroup), if present.

Tests

- [root-element-background-image-transparency-001.html](https://wpt.fyi/results/css/compositing/root-element-background-image-transparency-001.html) [(live test)](http://wpt.live/css/compositing/root-element-background-image-transparency-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-background-image-transparency-001.html)
- [root-element-background-image-transparency-002.html](https://wpt.fyi/results/css/compositing/root-element-background-image-transparency-002.html) [(live test)](http://wpt.live/css/compositing/root-element-background-image-transparency-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-background-image-transparency-002.html)
- [root-element-background-image-transparency-003.html](https://wpt.fyi/results/css/compositing/root-element-background-image-transparency-003.html) [(live test)](http://wpt.live/css/compositing/root-element-background-image-transparency-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-background-image-transparency-003.html)
- [root-element-background-image-transparency-004.html](https://wpt.fyi/results/css/compositing/root-element-background-image-transparency-004.html) [(live test)](http://wpt.live/css/compositing/root-element-background-image-transparency-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-background-image-transparency-004.html)
- [root-element-background-transparency.html](https://wpt.fyi/results/css/compositing/root-element-background-transparency.html) [(live test)](http://wpt.live/css/compositing/root-element-background-transparency.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-background-transparency.html)
- [root-element-blend-mode.html](https://wpt.fyi/results/css/compositing/root-element-blend-mode.html) [(live test)](http://wpt.live/css/compositing/root-element-blend-mode.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-blend-mode.html)
- [root-element-filter-background-clip-text-crash.html](https://wpt.fyi/results/css/compositing/root-element-filter-background-clip-text-crash.html) [(live test)](http://wpt.live/css/compositing/root-element-filter-background-clip-text-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-filter-background-clip-text-crash.html)
- [root-element-filter.html](https://wpt.fyi/results/css/compositing/root-element-filter.html) [(live test)](http://wpt.live/css/compositing/root-element-filter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-filter.html)
- [root-element-opacity-change.html](https://wpt.fyi/results/css/compositing/root-element-opacity-change.html) [(live test)](http://wpt.live/css/compositing/root-element-opacity-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-opacity-change.html)
- [root-element-opacity.html](https://wpt.fyi/results/css/compositing/root-element-opacity.html) [(live test)](http://wpt.live/css/compositing/root-element-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/root-element-opacity.html)

### <a id="rootgroup"></a>8.4. The Root Group

The root group encompasses the entire canvas and contains (or is below) the root element group of the root element of a web page.

> <strong data-conversion-semantic="note">Note</strong>
>
> Browsers often use an infinite [white, 100% opaque](https://www.w3.org/TR/css-color-4/#sample) root group, for final compositing, but this is not required.

## <a id="advancedcompositing"></a>9. Advanced compositing features

[Simple alpha compositing](#simplealphacompositing) uses the [source-over](#porterduffcompositingoperators_srcover) Porter Duff compositing operator.

Porter Duff compositing is based on a model of a pixel in which two shapes (source and destination) may contribute to the final color of the pixel. The pixel is divided into 4 sub-pixel regions and each region represents a possible combination of source and destination. [\[PORTERDUFF\]](#biblio-porterduff)

The four regions are:

Source Only  
Where only the source contributes to the pixel color

Destination only  
where only the destination contributes to the pixel color

Both  
Source and Destination – where both the source and destination may combine to define the pixel color

None  
No source or Destination – where neither make a contribution to the final pixel color

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Destination is synonymous with backdrop. The term destination is used in this section as this is considered the standard when working with Porter Duff compositing. Additionally, the compositing operators use destination in their names.

![overview of the regions affected by porter duff](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_regions.svg)

The contribution from each region to the final pixel color is defined by the coverage of the shape at that pixel, and the operator in use. Coverage is specified in terms of alpha. Full alpha (1) implies full coverage, while zero alpha (0) implies no coverage. This means that the area of each region within the sub-pixel is dependent on the coverage of each shape contributing to the pixel. The area of each region can be calculated with the following equations:

|                  |                     |
|------------------|---------------------|
| Both             | αs x αb             |
| Source only      | αs x (1 – αb)       |
| Destination only | αb x (1 – αs)       |
| None             | (1 – αs) x (1 – αb) |

The figure above represents coverage of 0.5 for both source and destination.

```text
Both = 0.5 x 0.5 = 0.25
Source Only = 0.5 (1 – 0.5) = 0.25
Destination Only = 0.5(1 – 0.5) = 0.25
None = (1 – 0.5)(1 – 0.5) = 0.25
```
Therefore, the coverage of each region is 0.25 in this example.

### <a id="porterduffcompositingoperators"></a>9.1. The Porter Duff Compositing Operators

The landmark paper by Thomas Porter and Tom Duff, who worked for Lucasfilm, defined the algebra of compositing and developed the twelve "Porter Duff" operators. These operators control the results of mixing the four sub-pixel regions formed by the overlapping of graphical objects that have an alpha or pixel coverage channel/value. The operators use all practical combinations of the four regions.

There are 12 basic Porter Duff operators, satisfying all possible combinations of source and destination.

From the geometric representation of each operator, the contribution of each shape can be seen to be expressed as a fraction of the total coverage of the output. For example, in source over, the possible contribution of source is full (1) and the possible contribution of destination is whatever is remaining (1 – αs). This is modified by the coverage of source and destination to give the equation for the final coverage of the pixel:

```text
αo = αs x 1 + αb x (1 – αs)
```
The fractional terms Fa (1 in this example) and Fb (1 – αs in this example) are defined for each operator and specify the fraction of the shapes that may contribute to the final pixel value. The general form of the equation for coverage is:

```text
αs x Fa + αb x Fb
```
and incorporating color gives the general Porter Duff equation

```text
co = αs x Fa x Cs + αb x Fb x Cb
```
Where:

- co is the output color pre-multiplied with the output alpha \[0 \<= co \<= 1\]
- αs is the coverage of the source Fa is defined by the operator and controls inclusion of the source Cs is the color of the source (not multiplied by alpha)
- αb is the coverage of the destination Fb is defined by the operator and controls inclusion of the destination Cb is the color of the destination (not multiplied by alpha)

#### <a id="porterduffcompositingoperators_clear"></a>9.1.1. Clear

No regions are enabled.

![example of porter duff clear](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_clr.svg)

```text
Fa = 0; Fb = 0
co = 0
αo = 0
```
#### <a id="porterduffcompositingoperators_src"></a>9.1.2. Copy

Only the source will be present.

![example of porter duff copy](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_src.svg)

```text
Fa = 1; Fb = 0
co = αs x Cs
αo = αs
```
#### <a id="porterduffcompositingoperators_dst"></a>9.1.3. Destination

Only the destination will be present.

![example of porter duff destination](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_dst.svg)

```text
Fa = 0; Fb = 1
co = αb x Cb
αo = αb
```
#### <a id="porterduffcompositingoperators_srcover"></a>9.1.4. Source Over

Source is placed over the destination.

![example of porter duff source over](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_src-over.svg)

```text
Fa = 1; Fb = 1 – αs
co = αs x Cs + αb x Cb x (1 – αs)
αo = αs + αb x (1 – αs)
```
#### <a id="porterduffcompositingoperators_dstover"></a>9.1.5. Destination Over

Destination is placed over the source.

![example of porter duff destination over](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_dst-over.svg)

```text
Fa = 1 – αb; Fb = 1
co = αs x Cs x (1 – αb) + αb x Cb
αo = αs x (1 – αb) + αb
```
#### <a id="porterduffcompositingoperators_srcin"></a>9.1.6. Source In

The source that overlaps the destination, replaces the destination.

![example of porter duff source in](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_src-in.svg)

```text
Fa = αb; Fb = 0
co = αs x Cs x αb
αo = αs x αb
```
#### <a id="porterduffcompositingoperators_dstin"></a>9.1.7. Destination In

Destination which overlaps the source, replaces the source.

![example of porter duff destination in ](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_dst-in.svg)

```text
Fa = 0; Fb = αs
co = αb x Cb x αs
αo = αb x αs
```
#### <a id="porterduffcompositingoperators_srcout"></a>9.1.8. Source Out

Source is placed, where it falls outside of the destination.

![example of porter duff source out](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_src-out.svg)

```text
Fa = 1 – αb; Fb = 0
co = αs x Cs x (1 – αb)
αo = αs x (1 – αb)
```
#### <a id="porterduffcompositingoperators_dstout"></a>9.1.9. Destination Out

Destination is placed, where it falls outside of the source.

![example of porter duff destination out](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_dst-out.svg)

```text
Fa = 0; Fb = 1 – αs
co = αb x Cb x (1 – αs)
αo = αb x (1 – αs)
```
#### <a id="porterduffcompositingoperators_srcatop"></a>9.1.10. Source Atop

Source which overlaps the destination, replaces the destination. Destination is placed elsewhere.

![example of porter duff source atop](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_src-atop.svg)

```text
Fa = αb; Fb = 1 – αs
co = αs x Cs x αb + αb x Cb x (1 – αs)
αo = αs x αb + αb x (1 – αs)
```
#### <a id="porterduffcompositingoperators_dstatop"></a>9.1.11. Destination Atop

Destination which overlaps the source replaces the source. Source is placed elsewhere.

![example of porter duff destination atop](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_dst-atop.svg)

```text
Fa = 1 - αb; Fb = αs
co = αs x Cs x (1 - αb) + αb x Cb x αs
αo = αs x (1 - αb) + αb x αs
```
#### <a id="porterduffcompositingoperators_xor"></a>9.1.12. XOR

The non-overlapping regions of source and destination are combined.

![example of porter duff xor](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/PD_xor.svg)

```text
Fa = 1 - αb; Fb = 1 – αs
co = αs x Cs x (1 - αb) + αb x Cb x (1 – αs)
αo = αs x (1 - αb) + αb x (1 – αs)
```
#### <a id="porterduffcompositingoperators_plus"></a>9.1.13. Lighter

Display the sum of the source image and destination image. It is defined in the Porter Duff paper as the plus operator [\[PORTERDUFF\]](#biblio-porterduff).

```text
Fa = 1; Fb = 1
co = αs x Cs + αb x Cb;
αo = αs + αb
```
### <a id="groupcompositing"></a>9.2. Group compositing behavior with Porter Duff modes

When compositing the elements within an isolated group, the elements are composited over a transparent black initial backdrop. If the bottom element in the group uses a Porter Duff compositing operator which is dependent on the backdrop, such as [destination](#porterduffcompositingoperators_dst), [source-in](#porterduffcompositingoperators_srcin), [destination-in](#porterduffcompositingoperators_dstin), [destination-out](#porterduffcompositingoperators_dstout) or [source-atop](#porterduffcompositingoperators_srcatop), then the result of the composite will be empty. Subsequent elements within the group are composited with the result of the first composite.

## <a id="blending"></a>10. Blending

<a id="ref-for-backdrop⑧"></a>

Blending is the aspect of compositing that calculates the mixing of colors where the source element and [backdrop](#backdrop) overlap.  
Conceptually, the colors in the source element are blended in place with the backdrop. After blending, the modified source element is composited with the backdrop. In practice, this is usually all performed in one step.  
The blending calculations must not use pre-multiplied color values.

The "mixing" formula is defined as:

```text
Cm = B(Cb, Cs)
```
with:

- Cm: the result color after blending

- B: the formula that does the blending

- <a id="ref-for-backdrop⑨"></a>

  Cb: the [backdrop](#backdrop) color

- Cs: the source color

The result of the mixing formula must be clamped to the minimum and maximum values of the color range.

The result of the mixing function is modulated by the backdrop alpha. A fully opaque backdrop allows the mixing function to be fully realized. A transparent backdrop will cause the final result to be a weighted average between the source color and mixed color with the weight controlled by the backdrop alpha. The value of the new color becomes:

```text
Cr = (1 - αb) x Cs + αb x B(Cb, Cs)
```
with:

- Cr: the result color

- B: the formula that does the blending

- Cs: the source color

- <a id="ref-for-backdrop①⓪"></a>

  Cb: the [backdrop](#backdrop) color

- <a id="ref-for-backdrop①①"></a>

  αb: the [backdrop](#backdrop) alpha

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-blend-with-opacity"></a>
>
> ![example of blending with opacity](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/blend_background_opacity.svg)
>
> This example has a red rectangle with a blending mode that is placed on top of a set of green rectangles that have different levels of opacity.
>
> <a id="ref-for-backdrop①②"></a>
>
> Note how the top rectangle shifts more toward red as the opacity of the [backdrop](#backdrop) gets smaller.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The following formula gives the color value in the area where the source and backdrop intersects and then composites with the specified Porter Duff compositing formula. For simple alpha blending, the formula thus becomes:

```text
simple alpha compositing:
    co = cs + cb x (1 - αs)
written as non-premultiplied:
    αo x Co = αs x Cs + (1 - αs) x αb x Cb
now substitute the result of blending for Cs:
    αo x Co = αs x ((1 - αb) x Cs + αb x B(Cb, Cs)) + (1 - αs) x αb x Cb
            = αs x (1 - αb) x Cs + αs x αb x B(Cb, Cs) + (1 - αs) x αb x Cb
```
### <a id="blendingseparable"></a>10.1. Separable blend modes

<a id="ref-for-backdrop①③"></a>

A blend mode is termed separable if each component of the result color is completely determined by the corresponding components of the constituent [backdrop](#backdrop) and source colors — that is, if the mixing formula is applied <strong>separately</strong> to each set of corresponding components.

Each of the following blend modes will apply the blending function B(Cb, Cs) on each of the color components. For simplicity, all the examples in this chapter use [source-over](#porterduffcompositingoperators_srcover) compositing.

#### <a id="blendingnormal"></a>10.1.1. <a id="valdef-blend-mode-normal"></a>normal blend mode

This is the default attribute which specifies no blending. The blending formula simply selects the source color.

```text
B(Cb, Cs) = Cs
```
![example of normal blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/normal.png)

#### <a id="blendingmultiply"></a>10.1.2. <a id="valdef-blend-mode-multiply"></a>multiply blend mode

The source color is multiplied by the destination color and replaces the destination.

The resultant color is always at least as dark as either the source or destination color. Multiplying any color with black results in black. Multiplying any color with white preserves the original color.

```text
B(Cb, Cs) = Cb x Cs
```
![example of multiply blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/multiply.png)

#### <a id="blendingscreen"></a>10.1.3. <a id="valdef-blend-mode-screen"></a>screen blend mode

<a id="ref-for-backdrop①④"></a>

Multiplies the complements of the [backdrop](#backdrop) and source color values, then complements the result.

The result color is always at least as light as either of the two constituent colors. Screening any color with white produces white; screening with black leaves the original color unchanged. The effect is similar to projecting multiple photographic slides simultaneously onto a single screen.

```text
B(Cb, Cs) = 1 - [(1 - Cb) x (1 - Cs)]
          = Cb + Cs -(Cb x Cs)
```
![example of screen blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/screen.png)

#### <a id="blendingoverlay"></a>10.1.4. <a id="valdef-blend-mode-overlay"></a>overlay blend mode

<a id="ref-for-backdrop①⑤"></a>

Multiplies or screens the colors, depending on the [backdrop](#backdrop) color value.

<a id="ref-for-backdrop①⑥"></a>

Source colors overlay the [backdrop](#backdrop) while preserving its highlights and shadows. The <a id="ref-for-backdrop①⑦"></a>backdrop color is not replaced but is mixed with the source color to reflect the lightness or darkness of the <a id="ref-for-backdrop①⑧"></a>backdrop.

```text
B(Cb, Cs) = HardLight(Cs, Cb)
```
<a id="ref-for-valdef-blend-mode-hard-light①"></a>

Overlay is the inverse of the [hard-light](#valdef-blend-mode-hard-light) blend mode. See the definition of <a id="ref-for-valdef-blend-mode-hard-light②"></a>hard-light for the formula.

![example of overlay blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/overlay.png)

#### <a id="blendingdarken"></a>10.1.5. <a id="valdef-blend-mode-darken"></a>darken blend mode

<a id="ref-for-backdrop①⑨"></a>

Selects the darker of the [backdrop](#backdrop) and source colors.

<a id="ref-for-backdrop②⓪"></a>

The [backdrop](#backdrop) is replaced with the source where the source is darker; otherwise, it is left unchanged.

```text
B(Cb, Cs) = min(Cb, Cs)
```
![example of darken blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/darken.png)

#### <a id="blendinglighten"></a>10.1.6. <a id="valdef-blend-mode-lighten"></a>lighten blend mode

<a id="ref-for-backdrop②①"></a>

Selects the lighter of the [backdrop](#backdrop) and source colors.

<a id="ref-for-backdrop②②"></a>

The [backdrop](#backdrop) is replaced with the source where the source is lighter; otherwise, it is left unchanged.

```text
B(Cb, Cs) = max(Cb, Cs)
```
The result must be rounded down if it exceeds the range.

![example of lighten blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/lighten.png)

#### <a id="blendingcolordodge"></a>10.1.7. <a id="valdef-blend-mode-color-dodge"></a>color-dodge blend mode

<a id="ref-for-backdrop②③"></a>

Brightens the [backdrop](#backdrop) color to reflect the source color. Painting with black produces no changes.

```text
if(Cb == 0)
    B(Cb, Cs) = 0
else if(Cs == 1)
    B(Cb, Cs) = 1
else
    B(Cb, Cs) = min(1, Cb / (1 - Cs))
```
![example of color dodge blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/colordodge.png)

#### <a id="blendingcolorburn"></a>10.1.8. <a id="valdef-blend-mode-color-burn"></a>color-burn blend mode

<a id="ref-for-backdrop②④"></a>

Darkens the [backdrop](#backdrop) color to reflect the source color. Painting with white produces no change.

```text
if(Cb == 1)
    B(Cb, Cs) = 1
else if(Cs == 0)
    B(Cb, Cs) = 0
else
    B(Cb, Cs) = 1 - min(1, (1 - Cb) / Cs)
```
![example of color burn blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/colorburn.png)

#### <a id="blendinghardlight"></a>10.1.9. <a id="valdef-blend-mode-hard-light"></a>hard-light blend mode

<a id="ref-for-backdrop②⑤"></a>

Multiplies or screens the colors, depending on the source color value. The effect is similar to shining a harsh spotlight on the [backdrop](#backdrop).

```text
if(Cs <= 0.5)
    B(Cb, Cs) = Multiply(Cb, 2 x Cs)
else
    B(Cb, Cs) = Screen(Cb, 2 x Cs -1)
```
<a id="ref-for-valdef-blend-mode-multiply①"></a>

<a id="ref-for-valdef-blend-mode-screen①"></a>

See the definition of [multiply](#valdef-blend-mode-multiply) and [screen](#valdef-blend-mode-screen) for their formulas.

![example of hard light blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/hardlight.png)

#### <a id="blendingsoftlight"></a>10.1.10. <a id="valdef-blend-mode-soft-light"></a>soft-light blend mode

<a id="ref-for-backdrop②⑥"></a>

Darkens or lightens the colors, depending on the source color value. The effect is similar to shining a diffused spotlight on the [backdrop](#backdrop).

```text
    if(Cs <= 0.5)
        B(Cb, Cs) = Cb - (1 - 2 x Cs) x Cb x (1 - Cb)
    else
        B(Cb, Cs) = Cb + (2 x Cs - 1) x (D(Cb) - Cb)
with
    if(Cb <= 0.25)
        D(Cb) = ((16 * Cb - 12) x Cb + 4) x Cb
    else
        D(Cb) = sqrt(Cb)
```
![example of soft light blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/softlight.png)

#### <a id="blendingdifference"></a>10.1.11. <a id="valdef-blend-mode-difference"></a>difference blend mode

Subtracts the darker of the two constituent colors from the lighter color.

<a id="ref-for-backdrop②⑦"></a>

Painting with white inverts the [backdrop](#backdrop) color; painting with black produces no change.

```text
B(Cb, Cs) = | Cb - Cs |
```
![example of difference blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/difference.png)

#### <a id="blendingexclusion"></a>10.1.12. <a id="valdef-blend-mode-exclusion"></a>exclusion blend mode

<a id="ref-for-backdrop②⑧"></a>

Produces an effect similar to that of the Difference mode but lower in contrast. Painting with white inverts the [backdrop](#backdrop) color; painting with black produces no change

```text
B(Cb, Cs) = Cb + Cs - 2 x Cb x Cs
```
![example of exclusion blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/exclusion.png)

### <a id="blendingnonseparable"></a>10.2. Non-separable blend modes

Nonseparable blend modes consider all color components in combination as opposed to the separable ones that look at each component individually. All of these blend modes conceptually entail the following steps:

1.  <a id="ref-for-backdrop②⑨"></a>

    Convert the [backdrop](#backdrop) and source colors from the blending color space to an intermediate hue-saturation-luminosity representation.

2.  <a id="ref-for-backdrop③⓪"></a>

    Create a new color from some combination of hue, saturation, and luminosity components selected from the [backdrop](#backdrop) and source colors.

3.  Convert the result back to the original color space.

The nonseparable blend mode formulas make use of several auxiliary functions:

```text
    Lum(C) = 0.3 x Cred + 0.59 x Cgreen + 0.11 x Cblue

    ClipColor(C)
        L = Lum(C)
        n = min(Cred, Cgreen, Cblue)
        x = max(Cred, Cgreen, Cblue)
        if(n < 0)
            C = L + (((C - L) × L) / (L - n))

        if(x > 1)
            C = L + (((C - L) × (1 - L)) / (x - L))

        return C

    SetLum(C, l)
        d = l - Lum(C)
        Cred = Cred + d
        Cgreen = Cgreen + d
        Cblue = Cblue + d
        return ClipColor(C)

    Sat(C) = max(Cred, Cgreen, Cblue) - min(Cred, Cgreen, Cblue)

The subscripts min, mid, and max in the next function refer to the color
components having the minimum, middle, and maximum values upon entry to the function.

    SetSat(C, s)
        if(Cmax > Cmin)
            Cmid = (((Cmid - Cmin) x s) / (Cmax - Cmin))
            Cmax = s
        else
            Cmid = Cmax = 0
        Cmin = 0
        return C;
```
#### <a id="blendinghue"></a>10.2.1. <a id="valdef-blend-mode-hue"></a>hue blend mode

<a id="ref-for-backdrop③①"></a>

Creates a color with the hue of the source color and the saturation and luminosity of the [backdrop](#backdrop) color.

```text
B(Cb, Cs) = SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))
```
![example of hue blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/hue.png)

#### <a id="blendingsaturation"></a>10.2.2. <a id="valdef-blend-mode-saturation"></a>saturation blend mode

<a id="ref-for-backdrop③②"></a>

Creates a color with the saturation of the source color and the hue and luminosity of the [backdrop](#backdrop) color. Painting with this mode in an area of the <a id="ref-for-backdrop③③"></a>backdrop that is a pure gray (no saturation) produces no change.

```text
B(Cb, Cs) = SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))
```
![example of saturation blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/saturation.png)

#### <a id="blendingcolor"></a>10.2.3. <a id="valdef-blend-mode-color"></a>color blend mode

<a id="ref-for-backdrop③④"></a>

Creates a color with the hue and saturation of the source color and the luminosity of the [backdrop](#backdrop) color. This preserves the gray levels of the <a id="ref-for-backdrop③⑤"></a>backdrop and is useful for coloring monochrome images or tinting color images.

```text
B(Cb, Cs) = SetLum(Cs, Lum(Cb))
```
![example of color blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/color.png)

#### <a id="blendingluminosity"></a>10.2.4. <a id="valdef-blend-mode-luminosity"></a>luminosity blend mode

<a id="ref-for-backdrop③⑥"></a>

Creates a color with the luminosity of the source color and the hue and saturation of the [backdrop](#backdrop) color. This produces an inverse effect to that of the Color mode.

```text
B(Cb, Cs) = SetLum(Cb, Lum(Cs))
```
![example of luminosity blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/luminosity.png)

### <a id="isolationblending"></a>10.3. Effect of group isolation on blending

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In the following example, the elements used to construct the paper airplane are within a group. Each of these elements has their blend mode set to multiply.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-isolated-group"></a> The airplane on the left is a normal group, the airplane on the right is an [isolated group](#isolatedgroups).
>
> In the isolated group, the elements within the group are composited onto an empty initial backdrop, this stops the elements within the group multiplying with the backdrop. In the normal group, the elements within the group are composited onto the initial backdrop containing the land and sky. Therefore the elements of the airplane multiply with the land and sky. In both instances, the result of the group composite is composited onto the land and sky using the normal mix-blend-mode (the default mix-blend-mode applied to the group).
>
> ![example of isolated group blending](https://www.w3.org/TR/2024/CRD-compositing-1-20240321/examples/isolate_blend_example.png)

## <a id="privacy"></a> Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a> Security Considerations

It is important that the timing to the blending and compositing operations is independent of the source and destination pixel. Operations must be implemented in such a way that they always take the same amount of time regardless of the pixel values.

> <strong data-conversion-semantic="note">Note</strong>
>
> If this rule is not followed, an attacker could infer information and mount a timing attack.
>
> A timing attack is a method of obtaining information about content that is otherwise protected, based on studying the amount of time it takes for an operation to occur. If, for example, red pixels took longer to draw than green pixels, one might be able to reconstruct a rough image of the element being rendered, without ever having access to the content of the element.

## <a id="changes"></a>Changes

The following changes were made relative to the [Candidate Recommendation of 13 January 2015](https://www.w3.org/TR/2015/CR-compositing-1-20150113/):

- Added Privacy and Security sections

- Updated Animatable: no to Animation type: discrete

- Defined root elements for HTML and SVG, and the root element group

- Clarified that root groups are not necessarily opaque white

- Better definitions for root group and isolating groups

- Consistently use the multiplication symbol × rather than x or \*

- <a id="ref-for-valdef-color-lime"></a>

  <a id="ref-for-valdef-color-green"></a>

  Corrected examples that used [green](https://www.w3.org/TR/css-color-4/#valdef-color-green) where [lime](https://www.w3.org/TR/css-color-4/#valdef-color-lime) was meant

- Corrected initial value of background-blend-mode

- Assorted markup and linking fixes

The following changes were made relative to the [Candidate Recommendation of 20 February 2014](https://www.w3.org/TR/2014/CR-compositing-1-20140220/):

- force isolation of SVG images embedded as \<img\> no longer at risk

- <a id="ref-for-compositemode②"></a>

  removed destination as an option for [\<composite-mode\>](#compositemode)

The following changes were made relative to the [Last Call Working draft of 7 January 2014](https://www.w3.org/TR/2013/WD-compositing-1-20131010/):

- unneeded normative and informative references were removed

The following changes were made relative to the [Last Call Working draft of 10 October 2013](https://www.w3.org/TR/2013/WD-compositing-1-20131010/):

- knockout was removed from the non-normative section
- removed paragraph on SVG from simple alpha compositing
- updated abstract to include CSS and clarify intent
- removed clip-to-self and its references
- Changed section 5-10 to be normative + clarified notes/examples in those sections

The following changes were made relative to the [Working Draft of 2013-06-25](https://www.w3.org/TR/2013/WD-compositing-1-20130625/):

- clipping was removed as one of the operators that creates an isolated group in SVG
- background-blend-mode was changed so it matches repeating behavior of other background syntaxes.
- The mix-composite property was removed
- all open issues were resolved

Tests

- [opacity-and-transform-animation-crash.html](https://wpt.fyi/results/css/compositing/opacity-and-transform-animation-crash.html) [(live test)](http://wpt.live/css/compositing/opacity-and-transform-animation-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/compositing/opacity-and-transform-animation-crash.html)

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

- [backdrop](#backdrop), in § 7
- [background-blend-mode](#propdef-background-blend-mode), in § 3.4.3
- [\<blend-mode\>](#ltblendmodegt), in § 3.4.1
- [color](#valdef-blend-mode-color), in § 10.2.3
- [color-burn](#valdef-blend-mode-color-burn), in § 10.1.8
- [color-dodge](#valdef-blend-mode-color-dodge), in § 10.1.7
- [\<composite-mode\>](#compositemode), in § 4
- [darken](#valdef-blend-mode-darken), in § 10.1.5
- [difference](#valdef-blend-mode-difference), in § 10.1.11
- [exclusion](#valdef-blend-mode-exclusion), in § 10.1.12
- [hard-light](#valdef-blend-mode-hard-light), in § 10.1.9
- [hue](#valdef-blend-mode-hue), in § 10.2.1
- [isolation](#propdef-isolation), in § 3.4.2
- [\<isolation-mode\>](#isolated-propid), in § 3.4.2
- [lighten](#valdef-blend-mode-lighten), in § 10.1.6
- [luminosity](#valdef-blend-mode-luminosity), in § 10.2.4
- [mix-blend-mode](#propdef-mix-blend-mode), in § 3.4.1
- [multiply](#valdef-blend-mode-multiply), in § 10.1.2
- [normal](#valdef-blend-mode-normal), in § 10.1.1
- [overlay](#valdef-blend-mode-overlay), in § 10.1.4
- [saturation](#valdef-blend-mode-saturation), in § 10.2.2
- [screen](#valdef-blend-mode-screen), in § 10.1.3
- [soft-light](#valdef-blend-mode-soft-light), in § 10.1.10

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-COLOR-4\] defines the following terms:
  - <a id="45dae945"></a>green
  - <a id="f8e15dd7"></a>lime
  - <a id="3b7558dc"></a>opacity
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="5b43edf4"></a>mask
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="4eb9d37e"></a>\|
- \[CSS3BG\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="5ced56d0"></a>background-image
- \[HTML\] defines the following terms:
  - <a id="2f0492ac"></a>body
  - <a id="33db4476"></a>globalCompositeOperation
  - <a id="f0811ff8"></a>img

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 18 December 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 19 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

### <a id="informative"></a>Informative References

<a id="biblio-porterduff"></a>\[PORTERDUFF\]  
Thomas Porter; Tom Duff. Compositing digital images. July 1984.

## <a id="property-index"></a>Property Index

| Name                | Value              | Initial | Applies to                                                                                                             | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value | Media  |
|---------------------|--------------------|---------|------------------------------------------------------------------------------------------------------------------------|------|-------|----------------|-----------------|----------------|--------|
| <strong><span><a id="ref-for-propdef-background-blend-mode⑤"></a></span><a href="#propdef-background-blend-mode">background-blend-mode</a>&#xA;      </strong> | \<blend-mode\>#    | normal  | All HTML elements                                                                                                      | no   | N/A   | discrete       | per grammar     | as specified   | visual |
| <strong><span><a id="ref-for-propdef-isolation②"></a></span><a href="#propdef-isolation">isolation</a>&#xA;      </strong> | \<isolation-mode\> | auto    | All elements. In SVG, it applies to container elements, graphics elements and graphics referencing elements. \[SVG11\] | no   | N/A   | discrete       | per grammar     | as specified   | visual |
| <strong><span><a id="ref-for-propdef-mix-blend-mode①"></a></span><a href="#propdef-mix-blend-mode">mix-blend-mode</a>&#xA;      </strong> | \<blend-mode\>     | normal  | All elements. In SVG, it applies to container elements, graphics elements and graphics referencing elements. \[SVG11\] | no   | N/A   | discrete       | per grammar     | as specified   | visual |

