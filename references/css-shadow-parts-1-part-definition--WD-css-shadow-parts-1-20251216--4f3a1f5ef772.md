Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Shadow Parts Module Level 1](https://www.w3.org/TR/2025/WD-css-shadow-parts-1-20251216/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Shadow Parts Module Level 1

Source snapshot: https://www.w3.org/TR/2025/WD-css-shadow-parts-1-20251216/

Snapshot SHA-256: 4f3a1f5ef7725f5684ed09ac30619eac9271cfa1af0fa19c2b4cfd08e8f2bc54

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

<a id="ref-for-concept-shadow-tree②"></a>

# <a id="title"></a>CSS Shadow Parts Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-selectordef-part"></a>

<a id="ref-for-element-shadow-host"></a>

<a id="ref-for-concept-shadow-tree"></a>

This specification defines the [::part()](#selectordef-part) pseudo-element on [shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host), allowing <a id="ref-for-element-shadow-host①"></a>shadow hosts to selectively expose chosen elements from their [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) to the outside page for styling purposes.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-shadow-parts” in the title, like this: “\[css-shadow-parts\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-shadow-parts%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

Shadow DOM allows authors to separate their page into "components", subtrees of markup whose details are only relevant to the component itself, not the outside page. This reduces the chance of a style meant for one part of the page accidentally over-applying and making a different part of the page look wrong. However, this styling barrier also makes it harder for a page to interact with its components when it actually <em>wants</em> to do so.

<a id="ref-for-selectordef-part①"></a>

<a id="ref-for-concept-shadow-tree①"></a>

<a id="ref-for-custom-property"></a>

This specification defines the [::part()](#selectordef-part) pseudo-element, which allows an author to style specific, purposely exposed elements in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) from the outside page’s context. In combination with [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property), which let the outside page pass particular values (such as theme colors) into the component for it to do with as it will, these pseudo-elements allow components and the outside page to interact in safe, powerful ways, maintaining encapsulation without surrendering all control.

Tests

General tests for shadow parts

- [all-hosts.html](https://wpt.fyi/results/css/css-shadow-parts/all-hosts.html) [(live test)](http://wpt.live/css/css-shadow-parts/all-hosts.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/all-hosts.html)
- [animation-part.html](https://wpt.fyi/results/css/css-shadow-parts/animation-part.html) [(live test)](http://wpt.live/css/css-shadow-parts/animation-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/animation-part.html)
- [chaining-invalid-selector.html](https://wpt.fyi/results/css/css-shadow-parts/chaining-invalid-selector.html) [(live test)](http://wpt.live/css/css-shadow-parts/chaining-invalid-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/chaining-invalid-selector.html)
- [complex-matching.html](https://wpt.fyi/results/css/css-shadow-parts/complex-matching.html) [(live test)](http://wpt.live/css/css-shadow-parts/complex-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/complex-matching.html)
- [complex-non-matching.html](https://wpt.fyi/results/css/css-shadow-parts/complex-non-matching.html) [(live test)](http://wpt.live/css/css-shadow-parts/complex-non-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/complex-non-matching.html)
- [different-host.html](https://wpt.fyi/results/css/css-shadow-parts/different-host.html) [(live test)](http://wpt.live/css/css-shadow-parts/different-host.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/different-host.html)
- [double-forward.html](https://wpt.fyi/results/css/css-shadow-parts/double-forward.html) [(live test)](http://wpt.live/css/css-shadow-parts/double-forward.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/double-forward.html)
- [grouping-with-checked.html](https://wpt.fyi/results/css/css-shadow-parts/grouping-with-checked.html) [(live test)](http://wpt.live/css/css-shadow-parts/grouping-with-checked.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/grouping-with-checked.html)
- [grouping-with-disabled.html](https://wpt.fyi/results/css/css-shadow-parts/grouping-with-disabled.html) [(live test)](http://wpt.live/css/css-shadow-parts/grouping-with-disabled.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/grouping-with-disabled.html)
- [host-stylesheet.html](https://wpt.fyi/results/css/css-shadow-parts/host-stylesheet.html) [(live test)](http://wpt.live/css/css-shadow-parts/host-stylesheet.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/host-stylesheet.html)
- [inner-host.html](https://wpt.fyi/results/css/css-shadow-parts/inner-host.html) [(live test)](http://wpt.live/css/css-shadow-parts/inner-host.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/inner-host.html)
- [interaction-with-nested-pseudo-class.html](https://wpt.fyi/results/css/css-shadow-parts/interaction-with-nested-pseudo-class.html) [(live test)](http://wpt.live/css/css-shadow-parts/interaction-with-nested-pseudo-class.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/interaction-with-nested-pseudo-class.html)
- [interaction-with-placeholder.html](https://wpt.fyi/results/css/css-shadow-parts/interaction-with-placeholder.html) [(live test)](http://wpt.live/css/css-shadow-parts/interaction-with-placeholder.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/interaction-with-placeholder.html)
- [interaction-with-pseudo-elements.html](https://wpt.fyi/results/css/css-shadow-parts/interaction-with-pseudo-elements.html) [(live test)](http://wpt.live/css/css-shadow-parts/interaction-with-pseudo-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/interaction-with-pseudo-elements.html)
- [invalidation-complex-selector-forward.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-complex-selector-forward.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-complex-selector-forward.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-complex-selector-forward.html)
- [invalidation-complex-selector.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-complex-selector.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-complex-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-complex-selector.html)
- [invalidation-part-pseudo.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-part-pseudo.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-part-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-part-pseudo.html)
- [multiple-parts.html](https://wpt.fyi/results/css/css-shadow-parts/multiple-parts.html) [(live test)](http://wpt.live/css/css-shadow-parts/multiple-parts.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/multiple-parts.html)
- [part-after-combinator-invalidation.html](https://wpt.fyi/results/css/css-shadow-parts/part-after-combinator-invalidation.html) [(live test)](http://wpt.live/css/css-shadow-parts/part-after-combinator-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/part-after-combinator-invalidation.html)
- [part-mutation-pseudo.html](https://wpt.fyi/results/css/css-shadow-parts/part-mutation-pseudo.html) [(live test)](http://wpt.live/css/css-shadow-parts/part-mutation-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/part-mutation-pseudo.html)
- [part-nested-pseudo.html](https://wpt.fyi/results/css/css-shadow-parts/part-nested-pseudo.html) [(live test)](http://wpt.live/css/css-shadow-parts/part-nested-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/part-nested-pseudo.html)
- [precedence-part-vs-part.html](https://wpt.fyi/results/css/css-shadow-parts/precedence-part-vs-part.html) [(live test)](http://wpt.live/css/css-shadow-parts/precedence-part-vs-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/precedence-part-vs-part.html)
- [pseudo-classes-after-part.html](https://wpt.fyi/results/css/css-shadow-parts/pseudo-classes-after-part.html) [(live test)](http://wpt.live/css/css-shadow-parts/pseudo-classes-after-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/pseudo-classes-after-part.html)
- [pseudo-elements-after-part.html](https://wpt.fyi/results/css/css-shadow-parts/pseudo-elements-after-part.html) [(live test)](http://wpt.live/css/css-shadow-parts/pseudo-elements-after-part.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/pseudo-elements-after-part.html)
- [serialization.html](https://wpt.fyi/results/css/css-shadow-parts/serialization.html) [(live test)](http://wpt.live/css/css-shadow-parts/serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/serialization.html)
- [simple-forward-shorthand.html](https://wpt.fyi/results/css/css-shadow-parts/simple-forward-shorthand.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-forward-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-forward-shorthand.html)
- [simple-forward.html](https://wpt.fyi/results/css/css-shadow-parts/simple-forward.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-forward.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-forward.html)
- [simple-important.html](https://wpt.fyi/results/css/css-shadow-parts/simple-important.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-important.html)
- [simple-important-important.html](https://wpt.fyi/results/css/css-shadow-parts/simple-important-important.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-important-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-important-important.html)
- [simple-important-inline.html](https://wpt.fyi/results/css/css-shadow-parts/simple-important-inline.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-important-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-important-inline.html)
- [simple.html](https://wpt.fyi/results/css/css-shadow-parts/simple.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple.html)
- [simple-important.html](https://wpt.fyi/results/css/css-shadow-parts/simple-important.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-important.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-important.html)
- [simple-inline.html](https://wpt.fyi/results/css/css-shadow-parts/simple-inline.html) [(live test)](http://wpt.live/css/css-shadow-parts/simple-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/simple-inline.html)
- [style-sharing.html](https://wpt.fyi/results/css/css-shadow-parts/style-sharing.html) [(live test)](http://wpt.live/css/css-shadow-parts/style-sharing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/style-sharing.html)

------------------------------------------------------------------------

### <a id="motivation"></a>1.1. Motivation

For custom elements to be fully useful and as capable as built-in elements it should be possible for parts of them to be styled from outside. Exactly what can be styled from outside should be controlled by the element author. Also, it should be possible for a custom element to present a stable "API" for styling. That is, the selector used to style a part of a custom element should not expose or require knowledge of the internal details of the element. The custom element author should be able to change the internal details of the element while leaving the selectors untouched.

The previous proposed method for styling inside the shadow tree, the \>\>\> combinator, turned out to be <em>too powerful</em> for its own good; it exposed too much of a component’s internal structure to scrutiny, defeating some of the encapsulation benefits that using Shadow DOM brings. For this, and other performance-related reasons, the \>\>\> combinator was eventually dropped.

<a id="ref-for-custom-property①"></a>

<a id="ref-for-element-shadow-host②"></a>

This left us with using [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) as the only way to style into a shadow tree: the component would advertise that it uses certain <a id="ref-for-custom-property②"></a>custom properties to style its internals, and the outer page could then set those properties as it wished on the [shadow host](https://dom.spec.whatwg.org/#element-shadow-host), letting inheritance push the values down to where they were needed. This works very well for many simple theming use-cases.

<a id="ref-for-custom-property③"></a>

<a id="ref-for-hover-pseudo"></a>

However, there are some cases where this falls down. If a component wishes to allow arbitrary styling of something in its shadow tree, the only way to do so is to define hundreds of [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) (one per CSS property they wish to allow control of), which is obviously ridiculous for both usability and performance reasons. The situation is compounded if authors wish to style the component differently based on pseudo-classes like [:hover](https://www.w3.org/TR/selectors-4/#hover-pseudo); the component needs to duplicate the <a id="ref-for-custom-property④"></a>custom properties used for each pseudo-class (and each combination, like :hover:focus, resulting in a combinatorial explosion). This makes the usability and performance problems even worse.

<a id="ref-for-selectordef-part②"></a>

<a id="ref-for-custom-property⑤"></a>

We introduce [::part()](#selectordef-part) to handle this case much more elegantly and performantly. Rather than bundling everything into [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) names, the functionality lives in selectors and style rule syntax, like it’s meant to. This is far more usable for both component authors and component users, should have much better performance, and allows for better encapsulation/API surface.

<a id="ref-for-selectordef-part③"></a>

<a id="ref-for-custom-property⑥"></a>

<a id="ref-for-shadow-root-part-element-map"></a>

It’s important to note that [::part()](#selectordef-part) offers <em>absolutely zero new theoretical power</em>. It is not a rehash of the \>\>\> combinator, it is simply a more convenient and consistent syntax for something authors can already do with [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property). By separating out the explicitly "published" parts of an element (the [part element map](#shadow-root-part-element-map)) from the sub-parts that it merely happens to contain, it also helps with encapsulation, as authors can use <a id="ref-for-selectordef-part④"></a>::part() without fear of accidental over-styling.

## <a id="exposing"></a>2. Exposing a Shadow Element:

Elements in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) may be exported for styling by stylesheets outside the tree using the part and exportparts attributes.

<a id="ref-for-ordered-set"></a>

Each element has a <a id="element-part-name-list"></a>part name list which is an [ordered set](https://infra.spec.whatwg.org/#ordered-set) of tokens.

<a id="ref-for-list"></a>

<a id="ref-for-tuple"></a>

<a id="ref-for-string"></a>

Each element has a <a id="element-forwarded-part-name-list"></a>forwarded part name list which is a [list](https://infra.spec.whatwg.org/#list) of [tuples](https://infra.spec.whatwg.org/#tuple) containing a [string](https://infra.spec.whatwg.org/#string) for the inner part being forwarded and a <a id="ref-for-string①"></a>string giving the name it will be exposed as.

<a id="ref-for-concept-shadow-root"></a>

<a id="ref-for-string②"></a>

<a id="ref-for-ordered-set①"></a>

Each [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root) can be thought of as having a <a id="shadow-root-part-element-map"></a>part element map with keys that are [strings](https://infra.spec.whatwg.org/#string) and values that are [ordered sets](https://infra.spec.whatwg.org/#ordered-set) of elements.

<a id="ref-for-shadow-root-part-element-map①"></a>

The [part element map](#shadow-root-part-element-map) is described only as part of the algorithm for calculating style in this spec. It is not exposed via the DOM, as calculating it may be expensive and exposing it could allow access to elements inside closed shadow roots.

<a id="ref-for-shadow-root-part-element-map②"></a>

<a id="ref-for-element-part-name-list"></a>

<a id="ref-for-element-forwarded-part-name-list"></a>

[Part element maps](#shadow-root-part-element-map) are affected by the addition and removal of elements and changes to the [part name lists](#element-part-name-list) and [forwarded part name lists](#element-forwarded-part-name-list) of elements in the DOM.

<a id="ref-for-shadow-root-part-element-map③"></a>

To <a id="calculate-the-part-element-map"></a>calculate the [part element map](#shadow-root-part-element-map) of a shadow root, <var>outerRoot</var>:

1.  <a id="ref-for-concept-tree-descendant"></a>

    For each [descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>el</var> within <var>outerRoot</var>:

    1.  <a id="ref-for-element-part-name-list①"></a>

        <a id="ref-for-list-append"></a>

        <a id="ref-for-shadow-root-part-element-map④"></a>

        For each <var>name</var> in <var>el</var>’s [part name list](#element-part-name-list), [append](https://infra.spec.whatwg.org/#list-append) <var>el</var> to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>name</var>\].

    2.  <a id="ref-for-element-shadow-host③"></a>

        If <var>el</var> is a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host) itself then let <var>innerRoot</var> be its shadow root.

    3.  <a id="ref-for-calculate-the-part-element-map"></a>

        <a id="ref-for-shadow-root-part-element-map⑤"></a>

        [Calculate](#calculate-the-part-element-map) <var>innerRoot</var>’s [part element map](#shadow-root-part-element-map).

    4.  <a id="ref-for-element-forwarded-part-name-list①"></a>

        For each <var>innerName</var>/<var>outerName</var> in <var>el</var>’s [forwarded part name list](#element-forwarded-part-name-list):

        1.  If <var>innerName</var> is an ident:

            1.  <a id="ref-for-shadow-root-part-element-map⑥"></a>

                Let <var>innerParts</var> be <var>innerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>innerName</var>\]

            2.  <a id="ref-for-list-append①"></a>

                <a id="ref-for-shadow-root-part-element-map⑦"></a>

                [Append](https://infra.spec.whatwg.org/#list-append) the elements in <var>innerParts</var> to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>outerName</var>\]

        2.  If <var>innerName</var> is a pseudo-element name:

            1.  <a id="ref-for-list-append②"></a>

                <a id="ref-for-shadow-root-part-element-map⑧"></a>

                [Append](https://infra.spec.whatwg.org/#list-append) <var>innerRoot</var>’s pseudo-element(s) with that name to <var>outerRoot</var>’s [part element map](#shadow-root-part-element-map)\[<var>outerName</var>\].

<a id="ref-for-element-attrdef-html-global-part"></a>

### <a id="part-attr"></a>2.1. Naming a Shadow Element: the <code><a href="#element-attrdef-html-global-part">part</a></code> attribute

<a id="ref-for-concept-shadow-tree③"></a>

Any element in a shadow tree can have a <a id="element-attrdef-html-global-part"></a>`part` attribute. This is used to expose the element outside of the [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree).

Tests

- [invalidation-change-part-name-forward.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-change-part-name-forward.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-change-part-name-forward.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-change-part-name-forward.html)
- [invalidation-change-part-name.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-change-part-name.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-change-part-name.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-change-part-name.html)

The part attribute is parsed as a space-separated list of tokens representing the part names of this element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It’s okay to give a part multiple names. The "part name" should be considered similar to a class, not an id or tagname.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a2e8f894"></a>
>
> ```text
> <style>
>   c-e::part(textspan) { color: red; }
> </style>
> 
> <template id="c-e-template">
>   <span part="textspan">This text will be red</span>
> </template>
> <c-e></c-e>
> <script>
>   // Add template as custom element c-e
>   ...
> </script>
> ```
<a id="ref-for-element-attrdef-html-global-exportparts"></a>

### <a id="exportparts-attr"></a>2.2. Forwarding a Shadow Element: the <code><a href="#element-attrdef-html-global-exportparts">exportparts</a></code> attribute

Any element in a shadow tree can have a <a id="element-attrdef-html-global-exportparts"></a>`exportparts` attribute. If the element is a shadow host, this is used to allow styling of parts from hosts inside the <a id="ref-for-concept-shadow-tree④"></a>shadow tree by rules outside this the <a id="ref-for-concept-shadow-tree⑤"></a>shadow tree (as if they were elements in the same tree as the host, named by a part attribute).

Tests

- [both-part-and-exportparts.html](https://wpt.fyi/results/css/css-shadow-parts/both-part-and-exportparts.html) [(live test)](http://wpt.live/css/css-shadow-parts/both-part-and-exportparts.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/both-part-and-exportparts.html)
- [exportparts-different-scope.html](https://wpt.fyi/results/css/css-shadow-parts/exportparts-different-scope.html) [(live test)](http://wpt.live/css/css-shadow-parts/exportparts-different-scope.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/exportparts-different-scope.html)
- [exportparts-layered.html](https://wpt.fyi/results/css/css-shadow-parts/exportparts-layered.html) [(live test)](http://wpt.live/css/css-shadow-parts/exportparts-layered.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/exportparts-layered.html)
- [exportparts-multiple.html](https://wpt.fyi/results/css/css-shadow-parts/exportparts-multiple.html) [(live test)](http://wpt.live/css/css-shadow-parts/exportparts-multiple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/exportparts-multiple.html)
- [invalidation-change-exportparts-forward.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-change-exportparts-forward.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-change-exportparts-forward.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-change-exportparts-forward.html)

The exportparts attribute is parsed as a comma-separated list of part mappings. Each part mapping is one of:

`innerIdent : outerIdent`  
<a id="ref-for-element-forwarded-part-name-list②"></a>

Adds `innerIdent`/`outerIdent` to el’s [forwarded part name list](#element-forwarded-part-name-list).

`ident`  
<a id="ref-for-element-forwarded-part-name-list③"></a>

Adds `ident`/`ident` to el’s [forwarded part name list](#element-forwarded-part-name-list).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is shorthand for `ident : ident`.

`::ident : outerIdent`  
<a id="ref-for-fully-styleable"></a>

<a id="ref-for-element-forwarded-part-name-list④"></a>

If `::ident` is the name of a [fully styleable pseudo-element](https://www.w3.org/TR/css-pseudo-4/#fully-styleable), adds `::ident`/`outerIdent` to el’s [forward part name list](#element-forwarded-part-name-list). Otherwise, does nothing.

anything else  
Ignored for error-recovery / future compatibility.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It’s okay to map a sub-part to several names.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f835643"></a>
>
> ```text
> <style>
>   c-e::part(textspan) { color: red; }
> </style>
> 
> <template id="c-e-outer-template">
>   <c-e-inner exportparts="innerspan: textspan"></c-e-inner>
> </template>
> 
> <template id="c-e-inner-template">
>   <span part="innerspan">
>     This text will be red because the containing shadow
>     host forwards innerspan to the document as "textspan"
>     and the document style matches it.
>   </span>
>   <span part="textspan">
>     This text will not be red because textspan in the document style
>     cannot match against the part inside the inner custom element
>     if it is not forwarded.
>   </span>
> </template>
> 
> <c-e></c-e>
> <script>
>   // Add template as custom elements c-e-inner, c-e-outer
>   ...
> </script>
> ```
<a id="ref-for-fully-styleable①"></a>

<a id="ref-for-element-attrdef-html-global-exportparts①"></a>

<a id="ref-for-selectordef-part⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b11a0a88"></a> For example, a [fully styleable pseudo-element](https://www.w3.org/TR/css-pseudo-4/#fully-styleable) can be used in the <code><a href="#element-attrdef-html-global-exportparts">exportparts</a></code> attribute, to masquerade as a [::part()](#selectordef-part) for the component it’s in:
>
> ```text
> <template id=custom-element-template>
>   <p exportparts="::before : preceding-text, ::after : following-text">
>     Main text.
> </template>
> ```
>
> An element using that template can use a selector like x-component::part(preceding-text) to target the p::before pseudo-element in its shadow, so users of the component don’t need to know that the preceding text is implemented as a pseudo-element.

<a id="ref-for-selectordef-part⑥"></a>

## <a id="part"></a>3. Selecting a Shadow Element: the [::part()](#selectordef-part) pseudo-element

<a id="ref-for-element-attrdef-html-global-part①"></a>

The <a id="selectordef-part"></a>::part() pseudo-element allows you to select elements that have been exposed via a <code><a href="#element-attrdef-html-global-part">part</a></code> attribute. The syntax is:

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-mult-one-plus"></a>

```text
::part() = ::part( <ident>+ )
```
Tests

- [host-part-001.html](https://wpt.fyi/results/css/css-shadow-parts/host-part-001.html) [(live test)](http://wpt.live/css/css-shadow-parts/host-part-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/host-part-001.html)
- [host-part-002.html](https://wpt.fyi/results/css/css-shadow-parts/host-part-002.html) [(live test)](http://wpt.live/css/css-shadow-parts/host-part-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/host-part-002.html)
- [host-part-003.html](https://wpt.fyi/results/css/css-shadow-parts/host-part-003.html) [(live test)](http://wpt.live/css/css-shadow-parts/host-part-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/host-part-003.html)
- [host-part-nesting.html](https://wpt.fyi/results/css/css-shadow-parts/host-part-nesting.html) [(live test)](http://wpt.live/css/css-shadow-parts/host-part-nesting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/host-part-nesting.html)
- [multiple-scopes.html](https://wpt.fyi/results/css/css-shadow-parts/multiple-scopes.html) [(live test)](http://wpt.live/css/css-shadow-parts/multiple-scopes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/multiple-scopes.html)

<a id="ref-for-selectordef-part⑦"></a>

<a id="ref-for-originating-element"></a>

<a id="ref-for-element-shadow-host④"></a>

The [::part()](#selectordef-part) pseudo-element only matches anything when the [originating element](https://www.w3.org/TR/selectors-4/#originating-element) is a [shadow host](https://dom.spec.whatwg.org/#element-shadow-host).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c617aa59"></a> For example, if you have a custom button that contains a "label" element that is exposed for styling (via `part="label"`), you can select it with x-button::part(label).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-72ff019b"></a> Part names act similarly to classes: multiple elements can have the same part name, and a single element can have multiple part names.
>
> A tabstrip control might have multiple elements with `part="tab"`, all of which are selected by ::part(tab).
>
> If a single tab is active at a time, it can be specially indicated with `part="tab active"` and then selected by ::part(tab active) (or ::part(active tab), as order doesn’t matter).

<a id="ref-for-selectordef-part⑧"></a>

<a id="ref-for-fully-styleable②"></a>

<a id="ref-for-originating-element①"></a>

<a id="ref-for-concept-shadow-root①"></a>

<a id="ref-for-shadow-root-part-element-map⑨"></a>

<a id="ref-for-map-exists"></a>

<a id="ref-for-typedef-ident①"></a>

The [::part()](#selectordef-part) pseudo-element is a [fully styleable pseudo-element](https://www.w3.org/TR/css-pseudo-4/#fully-styleable). If the [originating element’s](https://www.w3.org/TR/selectors-4/#originating-element) [shadow root’s](https://dom.spec.whatwg.org/#concept-shadow-root) [part element map](#shadow-root-part-element-map) [contains](https://infra.spec.whatwg.org/#map-exists) the specified [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident), <a id="ref-for-selectordef-part⑨"></a>::part() represents the elements keyed to that ident; if multiple idents are provided and the <a id="ref-for-shadow-root-part-element-map①⓪"></a>part element map contains them all, it represents the intersection of the elements keyed to each ident. Otherwise, it matches nothing.

<a id="ref-for-selectordef-part①⓪"></a>

<a id="ref-for-originating-element②"></a>

[::part()](#selectordef-part) pseudo-elements inherit according to their position in the [originating element’s](https://www.w3.org/TR/selectors-4/#originating-element) shadow tree.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-11b18654"></a> For example, x-panel::part(confirm-button)::part(label) never matches anything. This is because doing so would expose more structural information than is intended.
>
> <a id="ref-for-shadow-root-part-element-map①①"></a>
>
> If the `<x-panel>`’s internal confirm button had used something like `part="label => confirm-label"` to forward the button’s internal parts up into the panel’s own [part element map](#shadow-root-part-element-map), then a selector like x-panel::part(confirm-label) would select just the one button’s label, ignoring any other labels.

<a id="ref-for-element"></a>

## <a id="idl"></a>4. Extensions to the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> Interface

<a id="ref-for-element①"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-PutForwards"></a>

<a id="ref-for-dom-domtokenlist-value"></a>

<a id="ref-for-domtokenlist"></a>

<a id="dom-element-part"></a>

```text
partial interface Element {
  [SameObject, PutForwards=value] readonly attribute DOMTokenList part;
};
```
Tests

- [idlharness.html](https://wpt.fyi/results/css/css-shadow-parts/idlharness.html) [(live test)](http://wpt.live/css/css-shadow-parts/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/idlharness.html)
- [invalidation-change-part-name-idl-domtokenlist.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-change-part-name-idl-domtokenlist.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-change-part-name-idl-domtokenlist.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-change-part-name-idl-domtokenlist.html)
- [invalidation-change-part-name-idl-setter.html](https://wpt.fyi/results/css/css-shadow-parts/invalidation-change-part-name-idl-setter.html) [(live test)](http://wpt.live/css/css-shadow-parts/invalidation-change-part-name-idl-setter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/invalidation-change-part-name-idl-setter.html)
- [part-name-idl.html](https://wpt.fyi/results/css/css-shadow-parts/part-name-idl.html) [(live test)](http://wpt.live/css/css-shadow-parts/part-name-idl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shadow-parts/part-name-idl.html)

The part attribute’s getter must return a DOMTokenList object whose associated element is the context object and whose associated attribute’s local name is part. The token set of this particular DOMTokenList object are also known as the element’s parts.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-853df044"></a> Define this as a superglobal in the DOM spec. [\[w3c/csswg-drafts Issue \#3424\]](https://github.com/w3c/csswg-drafts/issues/3424)

## <a id="parsing"></a>5. Microsyntaxes for parsing

### <a id="parsing-mapping"></a>5.1. Rules for parsing part mappings

<a id="ref-for-tuple①"></a>

A <a id="valid-part-mapping"></a>valid part mapping is a [tuple](https://infra.spec.whatwg.org/#tuple) of tokens separated by a U+003A COLON character and any number of space characters before or after the U+003A COLON The tokens must not contain U+003A COLON or U+002C COMMA characters.

The rules for parsing a part mapping are as follows:

1.  Let <var>input</var> be the string being parsed.

2.  Let <var>position</var> be a pointer into <var>input</var>, initially pointing at the start of the string.

3.  <a id="ref-for-collect-a-sequence-of-code-points"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters

4.  <a id="ref-for-collect-a-sequence-of-code-points①"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters, and let <var>first token</var> be the result.

5.  If <var>first token</var> is empty then return error.

6.  <a id="ref-for-collect-a-sequence-of-code-points②"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

7.  <a id="ref-for-tuple②"></a>

    If the end of the <var>input</var> has been reached, return the [tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>first token</var>)

8.  If character at <var>position</var> is not a U+003A COLON character, return error.

9.  Consume the U+003A COLON character.

10. <a id="ref-for-collect-a-sequence-of-code-points③"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

11. <a id="ref-for-collect-a-sequence-of-code-points④"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are not space characters or U+003A COLON characters. and let <var>second token</var> be the result.

12. If <var>second token</var> is empty then return error.

13. <a id="ref-for-collect-a-sequence-of-code-points⑤"></a>

    [Collect a sequence of code points](https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points) that are space characters.

14. If <var>position</var> is not past the end of <var>input</var> then return error.

15. <a id="ref-for-tuple③"></a>

    Return the [tuple](https://infra.spec.whatwg.org/#tuple) (<var>first token</var>, <var>second token</var>).

### <a id="parsing-mapping-list"></a>5.2. Rules for parsing a list of part mappings

A <a id="valid-list-of-part-mappings"></a>valid list of part mappings is a number of valid part mappings separated by a U+002C COMMA character and any number of space characters before or after the U+002C COMMA

The rules for parsing a list of part mappings are as follow:

1.  Let <var>input</var> be the string being parsed.

2.  <a id="ref-for-split-on-commas"></a>

    [Split the string <var>input</var> on commas](https://infra.spec.whatwg.org/#split-on-commas). Let <var>unparsed mappings</var> be the resulting list of strings.

3.  <a id="ref-for-list①"></a>

    <a id="ref-for-tuple④"></a>

    Let <var>mappings</var> be an initially empty [list](https://infra.spec.whatwg.org/#list) of [tuples](https://infra.spec.whatwg.org/#tuple) of tokens. This <a id="ref-for-list②"></a>list will be the result of this algorithm.

4.  For each string <var>unparsed mapping</var> in <var>unparsed mappings</var>, run the following substeps:

    1.  If <var>unparsed mapping</var> is empty or contains only space characters, continue to the next iteration of the loop.

    2.  Let <var>mapping</var> be the result of parsing <var>unparsed mapping</var> using the rules for parsing part mappings.

    3.  If <var>mapping</var> is an error then continue to the next iteration of the loop. This allows clients to skip over new syntax that is not understood.

    4.  Append <var>mapping</var> to <var>mappings</var>.

## <a id="priv"></a>6.  Privacy Considerations

This specification defines new ways to target styling of elements on the page, which are already fully styleable in other ways. As such, it introduces no new privacy considerations.

## <a id="sec"></a>7.  Security Considerations

<a id="ref-for-concept-shadow-tree⑥"></a>

As [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) are intentionally not a security boundary, merely a convenience for page authors, exposing them to selectors in this way introduces no new security considerations.

## <a id="changes"></a>8.  Changes

Changes since the [First Public Working Draft](https://www.w3.org/TR/2018/WD-css-shadow-parts-1-20181115/) of 15 November 2018

- <a id="ref-for-selectordef-part①①"></a>

  Add support for multiple names in [::part()](#selectordef-part)

- <a id="ref-for-element-forwarded-part-name-list⑤"></a>

  Renamed 'part name map' to [forwarded part name list](#element-forwarded-part-name-list)

- Restructured 'part element list' algorithm

- <a id="ref-for-selectordef-part①②"></a>

  <a id="ref-for-fully-styleable③"></a>

  Moved various [::part()](#selectordef-part) details to [fully styleable pseudo-element](https://www.w3.org/TR/css-pseudo-4/#fully-styleable)

- Added Web Platform Tests coverage

- Minor editorial improvements

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

- [calculate the part element map](#calculate-the-part-element-map), in § 2
- [exportparts](#element-attrdef-html-global-exportparts), in § 2.2
- [forwarded part name list](#element-forwarded-part-name-list), in § 2
- [::part()](#selectordef-part), in § 3
- part
  - [attribute for Element](#dom-element-part), in § 4
  - [element-attr for html-global](#element-attrdef-html-global-part), in § 2.1
- [part element map](#shadow-root-part-element-map), in § 2
- [part name list](#element-part-name-list), in § 2
- [valid list of part mappings](#valid-list-of-part-mappings), in § 5.2
- [valid part mapping](#valid-part-mapping), in § 5.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="5c9e608d"></a>fully styleable pseudo-elements
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="af4a190d"></a>+
  - <a id="dcecfc13"></a>\<ident\>
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="5550667d"></a>custom property
- \[DOM\] defines the following terms:
  - <a id="141bbada"></a>DOMTokenList
  - <a id="296f3551"></a>Element
  - <a id="da8b8e0e"></a>descendant
  - <a id="58a8f724"></a>shadow host
  - <a id="3fcc582f"></a>shadow root
  - <a id="19f9e9df"></a>shadow tree
  - <a id="ff74a37a"></a>value
- \[INFRA\] defines the following terms:
  - <a id="53275e46"></a>append
  - <a id="d24334b4"></a>collect a sequence of code points
  - <a id="a326add7"></a>contain
  - <a id="649608b9"></a>list
  - <a id="692595fe"></a>ordered set
  - <a id="b29084c6"></a>split a string on commas
  - <a id="0698d556"></a>string
  - <a id="0e8de730"></a>tuple
- \[SELECTORS-4\] defines the following terms:
  - <a id="b725b7bc"></a>:hover
  - <a id="7b5d8638"></a>originating element
- \[WEBIDL\] defines the following terms:
  - <a id="21ecf38f"></a>PutForwards
  - <a id="a5c91173"></a>SameObject

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

## <a id="idl-index"></a>IDL Index

```text
partial interface Element {
  [SameObject, PutForwards=value] readonly attribute DOMTokenList part;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define this as a superglobal in the DOM spec. [\[w3c/csswg-drafts Issue \#3424\]](https://github.com/w3c/csswg-drafts/issues/3424) [↵](#issue-853df044)
