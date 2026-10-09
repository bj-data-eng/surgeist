Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Containment Module Level 3](https://www.w3.org/TR/2022/WD-css-contain-3-20220818/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Containment Module Level 3

Source snapshot: https://www.w3.org/TR/2022/WD-css-contain-3-20220818/

Snapshot SHA-256: 7e6b410172bfecf8b5fcccca2c5d6862ea87b81f20c34cd508934199f5373ec7

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 13 source tables are presented as readable Markdown tables or explicit labeled layouts: 13 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Containment Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-contain"></a>

This CSS module describes the [contain](https://www.w3.org/TR/css-contain-1/#propdef-contain) property, which indicates that the element’s subtree is independent of the rest of the page. This enables heavy optimizations by user agents when used well.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-contain” in the title, like this: “\[css-contain\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-contain%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2016550a"></a> This is a diff spec over [CSS Containment Level 2](https://www.w3.org/TR/css-contain-2/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 2 as a reference. We will merge the Level 2 text into this draft once it reaches CR.

### <a id="interaction"></a>1.1.  Module Interactions

This document defines new features not present in earlier specifications. In addition, it aims to replace and supersede [\[CSS-CONTAIN-1\]](#biblio-css-contain-1) once stable.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-contain①"></a>

## <a id="contain-property"></a>2.  Strong Containment: the [contain](https://www.w3.org/TR/css-contain-1/#propdef-contain) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9"></a> [CSS Containment 2 § 2 Strong Containment: the contain property](https://drafts.csswg.org/css-contain-2/#contain-property)

| Field               | Definition                                                                                                                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-contain②"></a>[contain](https://www.w3.org/TR/css-contain-1/#propdef-contain)                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a><a id="ref-for-comb-any"></a>layout [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) style <a id="ref-for-comb-any①"></a>\|\| paint <a id="ref-for-comb-any②"></a>\|\| \[ size [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-size \] |

<a id="valdef-contain-inline-size"></a>inline-size  
<a id="ref-for-principal-box"></a>

<a id="ref-for-inline-size"></a>

<a id="ref-for-inline-size-containment"></a>

This value turns on [inline-size containment](#inline-size-containment) for the element. This prevents the [inline-size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) from directly depending on its contents.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There can still be indirect dependencies, see [§ 3.1 Inline-Size Containment](#containment-inline-size).

## <a id="containment-types"></a>3.  Types of Containment

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①"></a> [CSS Containment 2 § 3 Types of Containment](https://drafts.csswg.org/css-contain-2/#containment-types)

### <a id="containment-inline-size"></a>3.1.  Inline-Size Containment

<a id="ref-for-size-containment"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-principal-box①"></a>

<a id="ref-for-intrinsic-size"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-fragmentation"></a>

Giving an element <a id="inline-size-containment"></a>inline-size containment applies [size containment](https://www.w3.org/TR/css-contain-1/#size-containment) to the [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) sizing of its [principal box](https://www.w3.org/TR/css-display-3/#principal-box). This means the <a id="ref-for-inline-axis①"></a>inline-axis [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) of the <a id="ref-for-principal-box②"></a>principal box are determined as if the element had no content. However, content continues to impact the box’s [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) <a id="ref-for-intrinsic-size①"></a>intrinsic sizes as usual, and the box is allowed to [fragment](https://www.w3.org/TR/css-break-3/#fragmentation) normally in the <a id="ref-for-block-axis①"></a>block axis.

<a id="ref-for-block-axis②"></a>

<a id="ref-for-intrinsic-size②"></a>

<a id="ref-for-formatting-context"></a>

<a id="ref-for-inline-size①"></a>

<a id="ref-for-block-size"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases, a box’s [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) can impact layout in the parent [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context) in ways that affect the box’s [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) (e.g. by triggering scrollbars on an ancestor element), creating a dependency of the box’s <a id="ref-for-inline-size②"></a>inline size on its own content. If this changed <a id="ref-for-inline-size③"></a>inline size results in a different [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size), that new <a id="ref-for-block-size①"></a>block size can loop into further impacting the parent formatting context, but not in a way that reverts it to the previously-problematic layout.
>
> <a id="ref-for-block-size②"></a>
>
> For example, if scrollbars were introduced, they are not then removed, even if the consequent [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) is small enough to not need them; or if a box’s logical height collides with a lower-placed float and is cleared down to where it also has more available inline space and thus becomes short enough to not have collided, it is not them moved back up to its previous problematic size and position.
>
> <a id="ref-for-inline-size-containment①"></a>
>
> <a id="ref-for-inline-size④"></a>
>
> <a id="ref-for-inline-axis②"></a>
>
> <a id="ref-for-intrinsic-size③"></a>
>
> <a id="ref-for-block-size③"></a>
>
> Thus, although [inline-size containment](#inline-size-containment) prevents the box’s content from directly affecting its [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) through its [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size), its <a id="ref-for-inline-size⑤"></a>inline size can still indirectly depend on its contents by their effect on its [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ac484479"></a> In general, the relationship between an element’s inline size and it’s block size is unpredictable and non-monotonic, with the block size capable of shifting up and down arbitrarily as the inline size is changed. Infinite cycles are prevented by ensuring that layout does not revert to a previous (known-problematic) state, even if a naive analysis of the constraints would allow for such; in other words, layout always “moves forward”. We believe that current CSS layout specifications incorporate such rules, but to the extent that they don’t, please [inform the CSSWG](https://github.com/w3c/csswg-drafts/issues) so that these errors can be corrected.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-45b76066"></a> Consider this example, where float placement creates a dependency of block sizes on inline sizes:
>
> ```markup
> <section style="width: 200px; border: solid; display: flow-root;">
>   <div style="float: left; width: 50px; height: 80px; background: blue;"></div>
>   <div style="float: right; width: 50px; height: 80px; background: blue;"></div>
>   <div style="float: left; width: 160px; height: 80px; background: navy;"></div>
> 
>   <article style="border: solid orangered; display: flow-root; min-width: min-content">
>     <div style="background: orange; aspect-ratio: 1/1;">
>       Article
>     </div>
>   </article>
> </section>
> ```
>
> Article
>
> The block layout algorithm will first place the floating boxes, with the first two sitting in the left and right corners of the container, and the third, being too wide to fit between, being pushed below them.
>
> <a id="ref-for-propdef-display"></a>
>
> The following `article` will then be laid out. Because it is [display: flow-root](https://www.w3.org/TR/css-display-3/#propdef-display), it cannot intersect any floats, and thus must take them into account when figuring out how to size and position itself.
>
> <a id="ref-for-min-content"></a>
>
> <a id="ref-for-propdef-aspect-ratio"></a>
>
> The layout engine first attempts to place the `article` flush with the top of the container, resulting a 100px width, plenty wide enough to accommodate its [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content). However, due to the [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) of its child, this would cause the `article` to be 100px tall as well, which would intersect the third float 80px below, so this layout opportunity is discarded.
>
> <a id="ref-for-propdef-min-width"></a>
>
> It then attempts to position the `article` flush with the top of the third float, in the narrow 40px-wide space to its right. However, since the `article`’s [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) makes it too large to fit in the 40px-wide space beside the third float, it shifts below that one as well, forming a 200px square below all the floated boxes.
>
> Article
>
> <a id="ref-for-propdef-min-width①"></a>
>
> <a id="ref-for-inline-size-containment②"></a>
>
> If the [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) is removed from the `article`, or if [inline-size containment](#inline-size-containment) is added to either the `article` or `header` (causing <a id="ref-for-propdef-min-width②"></a>min-width: min-content to resolve to zero), then the `article` will fit as a 40px square next to the final floated `div` (possibly with some of its content overflowing).
>
> At this point, the width and height of the `article` (40px each) <em>would</em> fit back in the first considered space, flush with the top of the container. However, the box is not returned to the previous position, because the layout engine knows already that this position would result in an invalid layout.

<a id="ref-for-inline-size-containment③"></a>

Giving an element [inline-size containment](#inline-size-containment) has no effect if any of the following are true:

- <a id="ref-for-principal-box③"></a>

  <a id="ref-for-propdef-display①"></a>

  if the element does not generate a [principal box](https://www.w3.org/TR/css-display-3/#principal-box) (as is the case with [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) or <a id="ref-for-propdef-display②"></a>display: none)

- <a id="ref-for-inner-display-type"></a>

  <a id="ref-for-valdef-display-table"></a>

  if its [inner display type](https://www.w3.org/TR/css-display-3/#inner-display-type) is [table](https://www.w3.org/TR/css-display-3/#valdef-display-table)

- <a id="ref-for-principal-box④"></a>

  <a id="ref-for-internal-table-box"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal table box](https://www.w3.org/TR/css-display-3/#internal-table-box)

- <a id="ref-for-principal-box⑤"></a>

  <a id="ref-for-internal-ruby-box"></a>

  <a id="ref-for-atomic-inline"></a>

  <a id="ref-for-inline-level"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal ruby box](https://www.w3.org/TR/css-display-3/#internal-ruby-box) or a [non-atomic](https://www.w3.org/TR/css-display-3/#atomic-inline) [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) box

## <a id="container-queries"></a>4.  Container Queries

<a id="ref-for-media-query"></a>

<a id="ref-for-container-query"></a>

While [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query) provide a method to query aspects of the user agent or device environment that a document is being displayed in (such as viewport dimensions or user preferences), [container queries](#container-query) allow testing aspects of elements within the document (such as box dimensions or computed styles).

<a id="ref-for-query-container"></a>

<a id="ref-for-container-style-query"></a>

<a id="ref-for-container-size-query"></a>

<a id="ref-for-propdef-container-type"></a>

<a id="ref-for-propdef-container"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-concept-shadow-including-descendant"></a>

<a id="ref-for-at-ruledef-container"></a>

<a id="ref-for-conditional-group-rule"></a>

By default, all elements are [query containers](#query-container) for the purpose of [container style queries](#container-style-query), and can be established as <a id="ref-for-query-container①"></a>query containers for [container size queries](#container-size-query) by specifying the additional query types using the [container-type](#propdef-container-type) property (or the [container](#propdef-container) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property)). Style rules applying to a <a id="ref-for-query-container②"></a>query container’s [shadow-including descendants](https://dom.spec.whatwg.org/#concept-shadow-including-descendant) can be conditioned by querying against it, using the [@container](#at-ruledef-container) [conditional group rule](https://www.w3.org/TR/css3-conditional/#conditional-group-rule).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-93e9e452"></a> For example, we can define the main content area and sidebar as containers, and then describe a .media-object that changes from vertical to horizontal layout depending on the size of its container:
>
> ```css
> main, aside {
>   container: my-layout / inline-size;
> }
> 
> .media-object {
>   display: grid;
>   grid-template: 'img' auto 'content' auto / 100%;
> }
> 
> @container my-layout (inline-size > 45em) {
>   .media-object {
>     grid-template: 'img content' auto / auto 1fr;
>   }
> }
> ```
>
> Media objects in the main and sidebar areas will each respond to their own container context.

<a id="ref-for-concept-shadow-including-inclusive-ancestor"></a>

<a id="ref-for-ultimate-originating-element"></a>

For selectors with pseudo elements, query containers can be established by the [shadow-including inclusive ancestors](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of the [ultimate originating element](https://www.w3.org/TR/selectors-4/#ultimate-originating-element).

> <strong data-conversion-semantic="note">Note</strong>
>
> It follows that:
>
> - Pseudo elements themselves can not be query containers
>
> - <a id="ref-for-selectordef-before"></a>
>
>   <a id="ref-for-selectordef-after"></a>
>
>   <a id="ref-for-selectordef-marker"></a>
>
>   <a id="ref-for-css-pe-backdrop"></a>
>
>   [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker), and [::backdrop](https://fullscreen.spec.whatwg.org/#css-pe-backdrop) query their originating elements
>
> - <a id="ref-for-selectordef-first-letter"></a>
>
>   <a id="ref-for-selectordef-first-line"></a>
>
>   <a id="ref-for-fictional-tag-sequence"></a>
>
>   [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) and [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) query their originating elements, even if the [fictional tag sequence](https://www.w3.org/TR/css-pseudo-4/#fictional-tag-sequence) may push the `::first-line` past other elements for the purpose of inheritance and rendering
>
> - Multiple pseudo elements do not allow pseudo elements to be query containers for other pseudo elements. E.g., the host, but not the `::part()`, can be the query container for `::before` in `host::part()::before`. Similarly, `::before` can not be the query container for the `::marker` in `div::before::marker`
>
> - <a id="ref-for-selectordef-slotted"></a>
>
>   [::slotted()](https://drafts.csswg.org/css-scoping-1/#selectordef-slotted) selectors can query containers inside the shadow tree, including the slot itself
>
> - <a id="ref-for-selectordef-part"></a>
>
>   [::part()](https://www.w3.org/TR/css-shadow-parts-1/#selectordef-part) selectors can query its originating host, but not internal query containers inside the shadow tree
>
> - <a id="ref-for-selectordef-placeholder"></a>
>
>   <a id="ref-for-selectordef-file-selector-button"></a>
>
>   [::placeholder](https://www.w3.org/TR/css-pseudo-4/#selectordef-placeholder) and [::file-selector-button](https://www.w3.org/TR/css-pseudo-4/#selectordef-file-selector-button) can query the input element, but do not expose any internal containers if the input element is implemented using a shadow tree

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-13f6f0fa"></a> A ::before selector querying the size of the originating element:
>
> ```html
> <style>
>   #container {
>     width: 100px;
>     container-type: inline-size;
>   }
>   @container (inline-size < 150px) {
>     #inner::before {
>       content: "BEFORE";
>     }
>   }
> </style>
> <div id=container>
>   <span id=inner></span>
> </div>
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c6405bc1"></a> A ::slotted() selector for styling a shadow host child can query a container in the shadow tree:
>
> ```html
> <div id=host style="width:200px">
>   <template shadowroot=open>
>     <style>
>       #container {
>         width: 100px;
>         container-type: inline-size;
>       }
>       @container (inline-size < 150px) {
>         ::slotted(span) {
>           color: green;
>         }
>       }
>     </style>
>     <div id=container>
>       <slot />
>     </div>
>   </template>
>   <span id=slotted>Green</span>
> </div>
> ```
<a id="ref-for-propdef-container-type①"></a>

### <a id="container-type"></a>4.1.  Creating Query Containers: the [container-type](#propdef-container-type) property

| Field               | Definition                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-container-type"></a>container-type                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①"></a><a id="ref-for-comb-any③"></a>normal [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ size [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-size \]                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-container-type-inline-size"></a><a id="ref-for-valdef-container-type-size"></a><a id="ref-for-valdef-container-type-normal"></a>the keyword [normal](#valdef-container-type-normal) or one or more of [size](#valdef-container-type-size), [inline-size](#valdef-container-type-inline-size) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                        |

<a id="ref-for-propdef-container-type②"></a>

<a id="ref-for-container-query①"></a>

<a id="ref-for-container-size-query①"></a>

<a id="ref-for-style-rule"></a>

The [container-type](#propdef-container-type) property establishes the element as a <a id="query-container"></a>query container for the purpose of [container queries](#container-query) that require explicit containment (such as [container size queries](#container-size-query)), allowing [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) styling its descendants to query various aspects of its sizing and layout, and respond accordingly.

<a id="ref-for-query-container③"></a>

<a id="ref-for-container-query②"></a>

<a id="ref-for-container-style-query①"></a>

<a id="ref-for-propdef-container-type③"></a>

Unless otherwise noted, all elements are [query containers](#query-container) for the purpose [container queries](#container-query) that do no require explicit containment (such as [container style queries](#container-style-query)), reagardless of the specified [container-type](#propdef-container-type).

Values have the following meanings:

<a id="valdef-container-type-size"></a>size  
<a id="ref-for-principal-box⑥"></a>

<a id="ref-for-size-containment①"></a>

<a id="ref-for-style-containment"></a>

<a id="ref-for-layout-containment"></a>

<a id="ref-for-block-axis③"></a>

<a id="ref-for-inline-axis③"></a>

<a id="ref-for-container-size-query②"></a>

<a id="ref-for-query-container④"></a>

Establishes a [query container](#query-container) for [container size queries](#container-size-query) on both the [inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) and [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Applies [layout containment](https://www.w3.org/TR/css-contain-1/#layout-containment), [style containment](https://drafts.csswg.org/css-contain-2/#style-containment), and [size containment](https://www.w3.org/TR/css-contain-1/#size-containment) to the [principal box](https://www.w3.org/TR/css-display-3/#principal-box).

<a id="valdef-container-type-inline-size"></a>inline-size  
<a id="ref-for-principal-box⑦"></a>

<a id="ref-for-inline-size-containment④"></a>

<a id="ref-for-style-containment①"></a>

<a id="ref-for-layout-containment①"></a>

<a id="ref-for-inline-axis④"></a>

<a id="ref-for-container-size-query③"></a>

<a id="ref-for-query-container⑤"></a>

Establishes a [query container](#query-container) for [container size queries](#container-size-query) on the container’s own [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Applies [layout containment](https://www.w3.org/TR/css-contain-1/#layout-containment), [style containment](https://drafts.csswg.org/css-contain-2/#style-containment), and [inline-size containment](#inline-size-containment) to the [principal box](https://www.w3.org/TR/css-display-3/#principal-box).

<a id="valdef-container-type-normal"></a>normal  
<a id="ref-for-container-style-query②"></a>

<a id="ref-for-container-size-query④"></a>

<a id="ref-for-query-container⑥"></a>

The element is not a [query container](#query-container) for any [container size queries](#container-size-query), but remains a <a id="ref-for-query-container⑦"></a>query container for [container style queries](#container-style-query).

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-propdef-line-height"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5708b886"></a> For example, authors can create container-responsive typography, adjusting [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), and other typographic concerns based on the size of a container:
>
> ```css
> aside, main {
>   container-type: inline-size;
> }
> 
> h2 { font-size: 1.2em; }
> 
> @container (width > 40em) {
>   h2 { font-size: 1.5em; }
> }
> ```
>
> <a id="ref-for-computed-value"></a>
>
> <a id="ref-for-propdef-font-size①"></a>
>
> <a id="ref-for-query-container⑧"></a>
>
> The 40em value used in the query condition is relative to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant [query container](#query-container).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1144456a"></a> Containers can also expose computed style values for querying. This can be useful for toggling behavior across multiple properties:
>
> ```css
> section {
>   container-type: style;
> }
> 
> @container (--cards: small) {
>   article {
>     border: thin solid silver;
>     border-radius: 0.5em;
>     padding: 1em;
>   }
> }
> ```
<a id="ref-for-propdef-container-name"></a>

### <a id="container-name"></a>4.2.  Naming Query Containers: the [container-name](#propdef-container-name) property

| Field               | Definition                                                                                                                                                                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-container-name"></a>container-name                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-one-plus"></a><a id="ref-for-identifier-value"></a><a id="ref-for-comb-one②"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-css-css-identifier"></a><a id="ref-for-valdef-container-name-none"></a>the keyword [none](#valdef-container-name-none), or an ordered list of [identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier)                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                                                  |

<a id="ref-for-propdef-container-name①"></a>

<a id="ref-for-at-ruledef-container①"></a>

<a id="ref-for-query-container⑨"></a>

The [container-name](#propdef-container-name) property specifies a list of <a id="query-container-name"></a>query container names. These names can be used by [@container](#at-ruledef-container) rules to filter which [query containers](#query-container) are targeted.

<a id="valdef-container-name-none"></a>none

<a id="ref-for-query-container-name"></a>

<a id="ref-for-query-container①⓪"></a>

The [query container](#query-container) has no [query container name](#query-container-name).

<a id="ref-for-identifier-value①"></a>

<a id="valdef-container-name-custom-ident"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

<a id="ref-for-identifier-value②"></a>

<a id="ref-for-valdef-media-not"></a>

<a id="ref-for-valdef-container-name-none①"></a>

<a id="ref-for-css-css-identifier①"></a>

<a id="ref-for-query-container-name①"></a>

Specifies a [query container name](#query-container-name) as an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier). The keywords [none](#valdef-container-name-none), and, [not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and or are excluded from this [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-44473254"></a> In some cases, we want to query aspects of a specific container, even if it’s not the nearest ancestor container. For example, we might want to query the height of a main content area, and the width of a more nested inline-container.
>
> ```css
> main {
>   container-type: size;
>   container-name: my-page-layout;
> }
> 
> .my-component {
>   container-type: inline-size;
>   container-name: my-component-library;
> }
> 
> @container my-page-layout (block-size > 12em) {
>   .card { margin-block: 2em; }
> }
> 
> @container my-component-library (inline-size > 30em) {
>   .card { margin-inline: 2em; }
> }
> ```
<a id="ref-for-propdef-container①"></a>

### <a id="container-shorthand"></a>4.3.  Creating Named Containers: the [container](#propdef-container) shorthand

| Field               | Definition                                                                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-container"></a>container                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-propdef-container-type④"></a><a id="ref-for-propdef-container-name②"></a>[\<'container-name'\>](#propdef-container-name) \[ / [\<'container-type'\>](#propdef-container-type) \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                       |

<a id="ref-for-propdef-container②"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-container-type⑤"></a>

<a id="ref-for-propdef-container-name③"></a>

<a id="ref-for-initial-value"></a>

The [container](#propdef-container) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both [container-type](#propdef-container-type) and [container-name](#propdef-container-name) in the same declaration. If <a id="ref-for-propdef-container-type⑥"></a>\<'container-type'\> is omitted, it is reset to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-propdef-container-type⑦"></a>

<a id="ref-for-propdef-container-name④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4e7fa3f"></a> We can define both a [container-type](#propdef-container-type) and [container-name](#propdef-container-name) using the shorthand syntax:
>
> ```css
> main {
>   container: my-layout / size;
> }
> 
> .grid-item {
>   container: my-component / inline-size;
> }
> ```
<a id="ref-for-at-ruledef-container②"></a>

### <a id="container-rule"></a>4.4.  Container Queries: the [@container](#at-ruledef-container) rule

<a id="ref-for-conditional-group-rule①"></a>

<a id="ref-for-container-size-query⑤"></a>

<a id="ref-for-container-style-query③"></a>

<a id="ref-for-typedef-stylesheet"></a>

<a id="ref-for-at-ruledef-container③"></a>

<a id="ref-for-container-query③"></a>

<a id="ref-for-query-container①①"></a>

The <a id="at-ruledef-container"></a>@container rule is a [conditional group rule](https://www.w3.org/TR/css3-conditional/#conditional-group-rule) whose condition is a <a id="container-query"></a>container query, which is a boolean combination of [container size queries](#container-size-query) and/or [container style queries](#container-style-query). Style declarations within the [\<stylesheet\>](https://www.w3.org/TR/css-syntax-3/#typedef-stylesheet) block of an [@container](#at-ruledef-container) rule are [filtered](https://www.w3.org/TR/css-cascade-4/#filtering) by its condition to only match when the [container query](#container-query) is true for their element’s [query container](#query-container).

<a id="ref-for-at-ruledef-container④"></a>

The syntax of the [@container](#at-ruledef-container) rule is:

<a id="ref-for-typedef-container-name"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-container-condition"></a>

<a id="ref-for-typedef-stylesheet①"></a>

```text
@container [ <container-name> ]? <container-condition> {
  <stylesheet>
}
```
where:

<a id="typedef-container-name"></a>

<a id="ref-for-typedef-container-name①"></a>

<a id="ref-for-identifier-value③"></a>

<a id="typedef-container-condition"></a>

<a id="ref-for-typedef-container-condition①"></a>

<a id="ref-for-typedef-query-in-parens"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-typedef-query-in-parens①"></a>

<a id="ref-for-typedef-query-in-parens②"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-query-in-parens③"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="typedef-query-in-parens"></a>

<a id="ref-for-typedef-query-in-parens④"></a>

<a id="ref-for-typedef-container-condition②"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-size-feature"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-style-query"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-general-enclosed"></a>

<a id="typedef-style-query"></a>

<a id="ref-for-typedef-style-query①"></a>

<a id="ref-for-typedef-style-condition"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-style-feature"></a>

<a id="typedef-style-condition"></a>

<a id="ref-for-typedef-style-condition①"></a>

<a id="ref-for-typedef-style-in-parens"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-style-in-parens①"></a>

<a id="ref-for-typedef-style-in-parens②"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-typedef-style-in-parens③"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="typedef-style-in-parens"></a>

<a id="ref-for-typedef-style-in-parens④"></a>

<a id="ref-for-typedef-style-condition②"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-typedef-style-feature①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-general-enclosed①"></a>

```text
<container-name> = <custom-ident>
<container-condition> = not <query-in-parens>
                      | <query-in-parens> [ [ and <query-in-parens> ]* | [ or <query-in-parens> ]* ]
<query-in-parens>     = ( <container-condition> )
                      | ( <size-feature> )
                      | style( <style-query> )
                      | <general-enclosed>

<style-query>         = <style-condition> | <style-feature>
<style-condition>     = not <style-in-parens>
                      | <style-in-parens> [ [ and <style-in-parens> ]* | [ or <style-in-parens> ]* ]
<style-in-parens>     = ( <style-condition> )
                      | ( <style-feature> )
                      | <general-enclosed>
```
<a id="ref-for-valdef-container-name-none②"></a>

<a id="ref-for-valdef-media-not①"></a>

<a id="ref-for-identifier-value④"></a>

The keywords [none](#valdef-container-name-none), and, [not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and or are excluded from the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) above.

<a id="ref-for-query-container①②"></a>

<a id="ref-for-container-feature"></a>

<a id="ref-for-typedef-container-condition③"></a>

<a id="ref-for-typedef-container-name②"></a>

<a id="ref-for-query-container-name②"></a>

For each element, the [query container](#query-container) to be queried is selected from among the element’s ancestor <a id="ref-for-query-container①③"></a>query containers that are established as a valid <a id="ref-for-query-container①④"></a>query container for all the [container features](#container-feature) in the [\<container-condition\>](#typedef-container-condition). The optional [\<container-name\>](#typedef-container-name) filters the set of <a id="ref-for-query-container①⑤"></a>query containers considered to just those with a matching [query container name](#query-container-name).

<a id="ref-for-query-container①⑥"></a>

<a id="ref-for-container-feature①"></a>

<a id="ref-for-typedef-container-condition④"></a>

<a id="ref-for-container-query④"></a>

Once an eligible [query container](#query-container) has been selected for an element, each [container feature](#container-feature) in the [\<container-condition\>](#typedef-container-condition) is evaluated against that <a id="ref-for-query-container①⑦"></a>query container. If no ancestor is an eligible <a id="ref-for-query-container①⑧"></a>query container, then the [container query](#container-query) is unknown for that element.

<a id="ref-for-media-query①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e1075629"></a> As with [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query), we can string together multiple conditions in a single query list:
>
> ```css
> @container card (inline-size > 30em) and style(--responsive: true) {
>   /* styles */
> }
> ```
>
> <a id="ref-for-descdef-container-inline-size"></a>
>
> <a id="ref-for-container-style-query④"></a>
>
> The styles above will only be applied if there is an ancestor container named "card" that meets both the [inline-size](#descdef-container-inline-size) and [style](#container-style-query) conditions.

<a id="ref-for-container-query⑤"></a>

Style rules defined on an element inside multiple nested [container queries](#container-query) apply when all of the wrapping <a id="ref-for-container-query⑥"></a>container queries are true for that element.

<a id="ref-for-container-query⑦"></a>

<a id="ref-for-typedef-container-condition⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Nested [container queries](#container-query) can evaluate in relation to different containers, so it is not always possible to merge the individual [\<container-condition\>](#typedef-container-condition)s into a single query.

<a id="ref-for-container-query⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9cdb6b15"></a> While it is not possible to query multiple containers in a single [container query](#container-query), that can be achieved by nesting multiple queries:
>
> ```css
> @container card (inline-size > 30em) {
>   @container style(--responsive: true) {
>     /* styles */
>   }
> }
> ```
>
> <a id="ref-for-descdef-container-inline-size①"></a>
>
> <a id="ref-for-container-style-query⑤"></a>
>
> The styles above will only be applied if there is an ancestor container named "card" that meets the [inline-size](#descdef-container-inline-size) condition, as well as an ancestor container meeting [style](#container-style-query) condition.

<a id="ref-for-at-rule"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-at-font-face-rule"></a>

<a id="ref-for-at-ruledef-layer"></a>

<a id="ref-for-container-query⑨"></a>

Global, name-defining [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) such as [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) or [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) or [@layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer) that are defined inside [container queries](#container-query) are not constrained by the <a id="ref-for-container-query①⓪"></a>container query conditions.

### <a id="animated-containers"></a>4.5.  Animated Containers

<a id="ref-for-container-query①①"></a>

<a id="ref-for-style-change-event"></a>

<a id="ref-for-effect-value"></a>

A change in the evaluation of a [container query](#container-query) must be part of a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event), even when the change occurred because of [animation effects](https://www.w3.org/TR/web-animations-1/#effect-value).

<a id="ref-for-style-change-event①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cfbdbffb"></a> A transition on a sibling element can indirectly affect the size of a container, triggering [style change events](https://www.w3.org/TR/css-transitions-1/#style-change-event) whenever container queries change their evaluation as a result:
>
> ```css
> main {
>   display: flex;
>   width: 300px;
> }
> 
> #container {
>   container-type: inline-size;
>   flex: 1;
> }
> 
> /* Resolved width is initially 200px, but changes as the transition
>    on #sibling progresses. */
> #inner {
>   transition: 1s background-color;
>   background-color: tomato;
> }
> 
> /* When this container query starts (or stops) applying, a transition
>    must start on background-color on #inner. */
> @container (width <= 150px) {
>   #inner {
>     background-color: skyblue;
>   }
> }
> 
> #sibling {
>   width: 100px;
>   transition: width 1s;
> }
> 
> #sibling:hover {
>   width: 200px;
> }
> ```
>
> ```html
> <main>
>   <div id=container>
>     <div id=inner>Inner</div>
>   </div>
>   <div id=sibling>Sibling</div>
> </main>
> ```
<a id="ref-for-computed-value①"></a>

<a id="ref-for-container-query-length"></a>

<a id="ref-for-style-change-event②"></a>

Changes in [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) caused by [container query length](#container-query-length) units must also be part of a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event).

## <a id="container-features"></a>5.  Container Features

<a id="ref-for-query-container①⑨"></a>

A <a id="container-feature"></a>container feature queries a specific aspect of a [query container](#query-container).

### <a id="size-container"></a>5.1.  Size Container Features

<a id="ref-for-query-container②⓪"></a>

<a id="ref-for-principal-box⑧"></a>

<a id="ref-for-typedef-size-feature①"></a>

<a id="ref-for-typedef-size-feature②"></a>

<a id="ref-for-media-feature"></a>

<a id="ref-for-size-features"></a>

<a id="ref-for-container-size-query⑥"></a>

<a id="ref-for-css-feature-queries"></a>

<a id="ref-for-at-ruledef-supports"></a>

A <a id="container-size-query"></a>container size query allows querying the size of the [query container](#query-container)’s [principal box](https://www.w3.org/TR/css-display-3/#principal-box). It is a boolean combination of individual <a id="size-features"></a>size features ([\<size-feature\>](#typedef-size-feature)) that each query a single, specific dimensional feature of the <a id="ref-for-query-container②①"></a>query container. The syntax of a <a id="typedef-size-feature"></a>[\<size-feature\>](#typedef-size-feature) is the same as for a [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value. [\[mediaqueries-5\]](#biblio-mediaqueries-5) The boolean syntax and logic combining [size features](#size-features) into a [size query](#container-size-query) is the same as for [CSS feature queries](https://www.w3.org/TR/css3-conditional/#css-feature-queries). (See [@supports](https://www.w3.org/TR/css3-conditional/#at-ruledef-supports). [\[CSS-CONDITIONAL-3\]](#biblio-css-conditional-3))

<a id="ref-for-query-container②②"></a>

<a id="ref-for-principal-box⑨"></a>

<a id="ref-for-layout-containment-box"></a>

<a id="ref-for-container-size-query⑦"></a>

<a id="ref-for-size-features①"></a>

If the [query container](#query-container) does not have a [principal box](https://www.w3.org/TR/css-display-3/#principal-box), or the principal box is not a [layout containment box](https://drafts.csswg.org/css-contain-2/#layout-containment-box), or the <a id="ref-for-query-container②③"></a>query container does not support [container size queries](#container-size-query) on the relevant axes, then the result of evaluating the [size feature](#size-features) is unknown.

<a id="ref-for-relative-length"></a>

<a id="ref-for-container-query-length①"></a>

<a id="ref-for-container-query①②"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-query-container②④"></a>

[Relative length](https://www.w3.org/TR/css-values-4/#relative-length) units (including [container query length](#container-query-length) units) in [container query](#container-query) conditions are evaluated based on the the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [query container](#query-container).

<a id="ref-for-media-query②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from the handling of relative units in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

<a id="ref-for-query-container②⑤"></a>

<a id="ref-for-em"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-031e4923"></a> For example, [query containers](#query-container) with different font-sizes will evaluate [em](https://www.w3.org/TR/css-values-4/#em)-based queries relative to their own font sizes:
>
> ```css
> aside, main {
>   container-type: inline-size;
> }
> 
> aside { font-size: 16px; }
> main { font-size: 24px; }
> 
> @container (width > 40em) {
>   h2 { font-size: 1.5em; }
> }
> ```
>
> <a id="ref-for-computed-value③"></a>
>
> <a id="ref-for-propdef-font-size②"></a>
>
> <a id="ref-for-query-container②⑥"></a>
>
> The 40em value used in the query condition is relative to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant [query container](#query-container):
>
> - For any h2 inside aside, the query condition will be true above 640px.
>
> - For any h2 inside main, the query condition will be true above 960px.

<a id="ref-for-descdef-container-width"></a>

#### <a id="width"></a>5.1.1.  Width: the [width](#descdef-container-width) feature

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-width"></a>width                                                          |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container⑤"></a>[@container](#at-ruledef-container)                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:&#xA;      </strong> | range                                                                             |

<a id="ref-for-descdef-container-width①"></a>

<a id="ref-for-container-feature②"></a>

<a id="ref-for-width"></a>

<a id="ref-for-query-container②⑦"></a>

<a id="ref-for-content-box"></a>

The [width](#descdef-container-width) [container feature](#container-feature) queries the [width](https://www.w3.org/TR/css-sizing-3/#width) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-descdef-container-height"></a>

#### <a id="height"></a>5.1.2.  Height: the [height](#descdef-container-height) feature

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-height"></a>height                                                         |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container⑥"></a>[@container](#at-ruledef-container)                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value①"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:&#xA;      </strong> | range                                                                             |

<a id="ref-for-descdef-container-height①"></a>

<a id="ref-for-container-feature③"></a>

<a id="ref-for-height"></a>

<a id="ref-for-query-container②⑧"></a>

<a id="ref-for-content-box①"></a>

The [height](#descdef-container-height) [container feature](#container-feature) queries the [height](https://www.w3.org/TR/css-sizing-3/#height) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-descdef-container-inline-size②"></a>

#### <a id="inline-size"></a>5.1.3.  Inline-size: the [inline-size](#descdef-container-inline-size) feature

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-inline-size"></a>inline-size                                                    |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container⑦"></a>[@container](#at-ruledef-container)                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:&#xA;      </strong> | range                                                                             |

<a id="ref-for-descdef-container-inline-size③"></a>

<a id="ref-for-container-feature④"></a>

<a id="ref-for-size"></a>

<a id="ref-for-query-container②⑨"></a>

<a id="ref-for-content-box②"></a>

<a id="ref-for-inline-axis⑤"></a>

The [inline-size](#descdef-container-inline-size) [container feature](#container-feature) queries the [size](https://www.w3.org/TR/css-sizing-3/#size) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container③⓪"></a>query container’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="ref-for-descdef-container-block-size"></a>

#### <a id="block-size"></a>5.1.4.  Block-size: the [block-size](#descdef-container-block-size) feature

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-block-size"></a>block-size                                                     |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container⑧"></a>[@container](#at-ruledef-container)                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-length-value③"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong>Type:&#xA;      </strong> | range                                                                             |

<a id="ref-for-descdef-container-block-size①"></a>

<a id="ref-for-container-feature⑤"></a>

<a id="ref-for-size①"></a>

<a id="ref-for-query-container③①"></a>

<a id="ref-for-content-box③"></a>

<a id="ref-for-block-axis④"></a>

The [block-size](#descdef-container-block-size) [container feature](#container-feature) queries the [size](https://www.w3.org/TR/css-sizing-3/#size) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container③②"></a>query container’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

<a id="ref-for-descdef-container-aspect-ratio"></a>

#### <a id="aspect-ratio"></a>5.1.5.  Aspect-ratio: the [aspect-ratio](#descdef-container-aspect-ratio) feature

| Field               | Definition                                                                      |
|---------------------|---------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-aspect-ratio"></a>aspect-ratio                                                 |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container⑨"></a>[@container](#at-ruledef-container)                          |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-ratio-value"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) |
| <strong>Type:&#xA;      </strong> | range                                                                           |

<a id="ref-for-descdef-container-aspect-ratio①"></a>

<a id="ref-for-container-feature⑥"></a>

<a id="ref-for-descdef-container-width②"></a>

<a id="ref-for-descdef-container-height②"></a>

The [aspect-ratio](#descdef-container-aspect-ratio) [container feature](#container-feature) is defined as the ratio of the value of the [width](#descdef-container-width) <a id="ref-for-container-feature⑦"></a>container feature to the value of the [height](#descdef-container-height) <a id="ref-for-container-feature⑧"></a>container feature.

<a id="ref-for-descdef-container-orientation"></a>

#### <a id="orientation"></a>5.1.6.  Orientation: the [orientation](#descdef-container-orientation) feature

| Field               | Definition                                                                               |
|---------------------|------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-container-orientation"></a>orientation                                                           |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-container①⓪"></a>[@container](#at-ruledef-container)                                   |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one①③"></a>portrait [\|](https://www.w3.org/TR/css-values-4/#comb-one) landscape |
| <strong>Type:&#xA;      </strong> | discrete                                                                                 |

<a id="valdef-container-orientation-portrait"></a>portrait  
<a id="ref-for-descdef-container-width③"></a>

<a id="ref-for-descdef-container-height③"></a>

<a id="ref-for-valdef-container-orientation-portrait"></a>

<a id="ref-for-container-feature⑨"></a>

<a id="ref-for-descdef-container-orientation①"></a>

The [orientation](#descdef-container-orientation) [container feature](#container-feature) is [portrait](#valdef-container-orientation-portrait) when the value of the [height](#descdef-container-height) <a id="ref-for-container-feature①⓪"></a>container feature is greater than or equal to the value of the [width](#descdef-container-width) <a id="ref-for-container-feature①①"></a>container feature.

<a id="valdef-container-orientation-landscape"></a>landscape  
<a id="ref-for-valdef-container-orientation-landscape"></a>

<a id="ref-for-descdef-container-orientation②"></a>

Otherwise [orientation](#descdef-container-orientation) is [landscape](#valdef-container-orientation-landscape).

### <a id="style-container"></a>5.2.  Style Container Features

<a id="ref-for-computed-value④"></a>

<a id="ref-for-query-container③③"></a>

<a id="ref-for-typedef-style-feature②"></a>

<a id="ref-for-typedef-style-feature③"></a>

<a id="ref-for-declaration"></a>

<a id="ref-for-style-features"></a>

<a id="ref-for-container-style-query⑥"></a>

<a id="ref-for-css-feature-queries①"></a>

<a id="ref-for-at-ruledef-supports①"></a>

A <a id="container-style-query"></a>container style query allows querying the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [query container](#query-container). It is a boolean combination of individual <a id="style-features"></a>style features ([\<style-feature\>](#typedef-style-feature)) that each query a single, specific property of the <a id="ref-for-query-container③④"></a>query container. The syntax of a <a id="typedef-style-feature"></a>[\<style-feature\>](#typedef-style-feature) is the same as for a [declaration](https://www.w3.org/TR/css-syntax-3/#declaration) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3), and its query is true if the <a id="ref-for-computed-value⑤"></a>computed value of the given property on the <a id="ref-for-query-container③⑤"></a>query container matches the given value (which is also <a id="ref-for-computed-value⑥"></a>computed with respect to the <a id="ref-for-query-container③⑥"></a>query container), unknown if the property or its value is invalid or unsupported, and false otherwise. The boolean syntax and logic combining [style features](#style-features) into a [style query](#container-style-query) is the same as for [CSS feature queries](https://www.w3.org/TR/css3-conditional/#css-feature-queries). (See [@supports](https://www.w3.org/TR/css3-conditional/#at-ruledef-supports). [\[CSS-CONDITIONAL-3\]](#biblio-css-conditional-3))

<a id="ref-for-style-features①"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-longhand"></a>

[Style features](#style-features) that query a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are true if the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) match for each of its [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand), and false otherwise.

<a id="ref-for-cascade-dependent-keyword"></a>

<a id="ref-for-valdef-all-revert"></a>

<a id="ref-for-valdef-all-revert-layer"></a>

<a id="ref-for-style-features②"></a>

<a id="ref-for-container-style-query⑦"></a>

[Cascade-dependent keywords](https://drafts.csswg.org/css-cascade-5/#cascade-dependent-keyword), such as [revert](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert) and [revert-layer](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert-layer), are invalid as values in a [style feature](#style-features), and cause the [container style query](#container-style-query) to be false.

<a id="ref-for-css-wide-keywords①"></a>

<a id="ref-for-computed-value⑧"></a>

<a id="ref-for-query-container③⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The remaining non-cascade-dependent [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) are [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) with respect to the [query container](#query-container), the same as other values.

## <a id="container-lengths"></a>6.  Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units

<a id="ref-for-query-container③⑧"></a>

<a id="ref-for-container-query-length②"></a>

<a id="container-query-length"></a>Container query length units specify a length relative to the dimensions of a [query container](#query-container). Style sheets that use [container query length](#container-query-length) units can more easily move components from one <a id="ref-for-query-container③⑨"></a>query container to another.

<a id="ref-for-container-query-length③"></a>

The [container query length](#container-query-length) units are:

| unit  | relative to                                                                                                                                               |
|-------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| cqw   | <a id="ref-for-width①"></a><a id="ref-for-query-container④⓪"></a>1% of a [query container](#query-container)’s [width](https://www.w3.org/TR/css-sizing-3/#width)                    |
| cqh   | <a id="ref-for-height①"></a><a id="ref-for-query-container④①"></a>1% of a [query container](#query-container)’s [height](https://www.w3.org/TR/css-sizing-3/#height)                  |
| cqi   | <a id="ref-for-inline-size⑥"></a><a id="ref-for-query-container④②"></a>1% of a [query container](#query-container)’s [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) |
| cqb   | <a id="ref-for-block-size④"></a><a id="ref-for-query-container④③"></a>1% of a [query container](#query-container)’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size)   |
| cqmin | The smaller value of cqi or cqb                                                                                                                           |
| cqmax | The larger value of cqi or cqb                                                                                                                            |

Informative Summary of Container Units

<a id="ref-for-container-query-length④"></a>

<a id="ref-for-container-size-query⑧"></a>

<a id="ref-for-query-container④④"></a>

<a id="ref-for-small-viewport-size"></a>

For each element, [container query length](#container-query-length) units are evaluated as [container size queries](#container-size-query) on the relevant axis (or axes) described by the unit. The [query container](#query-container) for each axis is the nearest ancestor container that accepts <a id="ref-for-container-size-query⑨"></a>container size queries on that axis. If no eligible <a id="ref-for-query-container④⑤"></a>query container is available, then use the [small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) for that axis.

<a id="ref-for-query-container④⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases cqi and cqb units on the same element will evaluate in relation to different [query containers](#query-container). Similarly, cqmin and cqmax units represent the larger or smaller of the cqi and cqb units, even when those dimensions come from different <a id="ref-for-query-container④⑦"></a>query containers.

<a id="ref-for-computed-value⑨"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-container-query-length⑤"></a>

<a id="ref-for-query-container④⑧"></a>

<a id="ref-for-container-query①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a4252068"></a> Authors can ensure that [container query length](#container-query-length) units have an appropriate [query container](#query-container) by applying them inside a [container query](#container-query) that relies on the same container-type. Custom fallback values can be defined outside the <a id="ref-for-container-query①④"></a>container query:
>
> ```css
> /* The fallback value does not rely on containment */
> h2 { font-size: 1.2em; }
> 
> @container (inline-size >= 0px) {
>   /* only applies when an inline-size container is available */
>   h2 { font-size: calc(1.2em + 1cqi); }
> }
> ```
## <a id="apis"></a>7. APIs

### <a id="the-csscontainerrule-interface"></a>7.1.  The `CSSContainerRule` interface

<a id="ref-for-csscontainerrule"></a>

<a id="ref-for-at-ruledef-container①①"></a>

The <code><a href="#csscontainerrule">CSSContainerRule</a></code> interface represents a [@container](#at-ruledef-container) rule.

<a id="ref-for-Exposed"></a>

<a id="csscontainerrule"></a>

<a id="ref-for-cssconditionrule"></a>

```text
[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
};
```
`conditionText` of type `CSSOMString` (CSSContainerRule-specific definition for attribute on CSSConditionRule)  
<a id="ref-for-typedef-general-enclosed②"></a>

The `conditionText` attribute (defined on the `CSSConditionRule` parent rule), on getting, must return the condition that was specified, without any logical simplifications, so that the returned condition will evaluate to the same result as the specified condition in any conformant implementation of this specification (including implementations that implement future extensions allowed by the [\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) extensibility mechanism in this specification). In other words, token stream simplifications are allowed (such as reducing whitespace to a single space or omitting it in cases where it is known to be optional), but logical simplifications (such as removal of unneeded parentheses, or simplification based on evaluating results) are not allowed.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-fe7a1c72"></a> Add CSSOM API for CSSContainerRule [\[Issue \#7033\]](https://github.com/w3c/csswg-drafts/issues/7033)

<a id="ref-for-dom-window-matchmedia"></a>

<a id="ref-for-mediaquerylist"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2e3d6538"></a> Container Queries should have a `matchContainer` method. This will be modeled on <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-matchmedia">matchMedia()</a></code> and the <code><a href="https://www.w3.org/TR/cssom-view/#mediaquerylist">MediaQueryList</a></code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to `resizeObserver`, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205)

<a id="ref-for-propdef-content-visibility"></a>

## <a id="content-visibility"></a>8. Suppressing An Element’s Contents Entirely: the [content-visibility](https://drafts.csswg.org/css-contain-2/#propdef-content-visibility) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9②"></a> [CSS Containment 2 § 4 Suppressing An Element’s Contents Entirely: the content-visibility property](https://drafts.csswg.org/css-contain-2/#content-visibility)

## <a id="priv-sec"></a>9. Privacy and Security Considerations

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9③"></a> [CSS Containment 2 § 5 Privacy and Security Considerations](https://drafts.csswg.org/css-contain-2/#priv-sec)

## <a id="changes"></a>Appendix A. Changes

This appendix is <em>informative</em>.

### <a id="changes-2021-12"></a> Changes since the 21 December 2021 First Public Working Draft

Significant changes since the [21 December 2021 First Public Working Draft](https://www.w3.org/TR/2021/WD-css-contain-3-20211221/) include:

- <a id="ref-for-propdef-container-name⑤"></a>

  Allow the computed value of [container-name](#propdef-container-name) to include duplicate identifiers. ([Issue 7181](https://github.com/w3c/csswg-drafts/issues/7181))

- <a id="ref-for-propdef-container-name⑥"></a>

  <a id="ref-for-propdef-container③"></a>

  Make the [\<'container-name'\>](#propdef-container-name) in the [container](#propdef-container) shorthand required. ([Issue 7142](https://github.com/w3c/csswg-drafts/issues/7142))

- <a id="ref-for-shorthand-property③"></a>

  <a id="ref-for-container-style-query⑧"></a>

  Clarify handling of [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) in [container style queries](#container-style-query). ([Issue 7095](https://github.com/w3c/csswg-drafts/issues/7095))

- <a id="ref-for-cascade-dependent-keyword①"></a>

  <a id="ref-for-style-features③"></a>

  <a id="ref-for-container-style-query⑨"></a>

  [Cascade-dependent keywords](https://drafts.csswg.org/css-cascade-5/#cascade-dependent-keyword) are not allowed as values in a [style feature](#style-features), and cause the [container style query](#container-style-query) to be false. ([Issue 7080](https://github.com/w3c/csswg-drafts/issues/7080))

- <a id="ref-for-propdef-container-type⑧"></a>

  <a id="ref-for-valdef-container-type-style"></a>

  Change the initial value of [container-type](#propdef-container-type) to be [style](https://drafts.csswg.org/css-contain-3/#valdef-container-type-style). ([Issue 6393](https://github.com/w3c/csswg-drafts/issues/6393))

- <a id="ref-for-propdef-container-type⑨"></a>

  Remove the block-size value from [container-type](#propdef-container-type), since single-axis block-size containment is not currently possible. ([Issue 1031](https://github.com/w3c/csswg-drafts/issues/1031))

- <a id="ref-for-string-value"></a>

  <a id="ref-for-propdef-container-name⑦"></a>

  <a id="ref-for-identifier-value⑤"></a>

  Remove the [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) option from the [container-name](#propdef-container-name) syntax. Container names must be [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)s. ([Issue 6405](https://github.com/w3c/csswg-drafts/issues/6405))

- <a id="ref-for-propdef-container-name⑧"></a>

  <a id="ref-for-propdef-container-type①⓪"></a>

  <a id="ref-for-propdef-container④"></a>

  Reverse the order of [\<'container-name'\>](#propdef-container-name) and [\<'container-type'\>](#propdef-container-type) in the [container](#propdef-container) shorthand property, with both being optional. ([Issue 6393](https://github.com/w3c/csswg-drafts/issues/6393))

- <a id="ref-for-typedef-general-enclosed③"></a>

  <a id="ref-for-typedef-container-condition⑥"></a>

  Allow [\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) syntax in [\<container-condition\>](#typedef-container-condition)s, for the sake of forward compatability. ([Issue 6396](https://github.com/w3c/csswg-drafts/issues/6396))

- <a id="ref-for-typedef-size-feature③"></a>

  Remove the size function syntax from [\<size-feature\>](#typedef-size-feature) queries. ([Issue 6870](https://github.com/w3c/csswg-drafts/issues/6870))

- <a id="ref-for-query-container④⑨"></a>

  Update the [query container](#query-container) selection process to account for necessary container-types, and removed the explicit type-selection syntax. ([Issue 6644](https://github.com/w3c/csswg-drafts/issues/6644))

- Remove state query features, which have been deferred. ([Issue 6402](https://github.com/w3c/csswg-drafts/issues/6402))

- Clarify container selection around pseudo-elements and the shadow-DOM. ([Issue 5984](https://github.com/w3c/csswg-drafts/issues/5984) and [Issue 6711](https://github.com/w3c/csswg-drafts/issues/6711))

### <a id="l3-changes"></a> Changes from [CSS Containment Level 2](https://www.w3.org/TR/css-contain-2/) 

- <a id="ref-for-inline-size-containment⑤"></a>

  Introduces [inline-size containment](#inline-size-containment).

- <a id="ref-for-container-query①⑤"></a>

  Defines the terms, properties, units, and at-rule needed for [Container Queries](#container-query)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9④"></a> [CSS Containment 2 §  Changes](https://drafts.csswg.org/css-contain-2/#changes)

## <a id="acknowledgments"></a>Acknowledgments

Comments and previous work from Adam Argyle, Amelia Bellamy-Royds, Anders Hartvoll Ruud, Brian Kardell, Chris Coyier, Christopher Kirk-Nielsen, David Herron, Elika J. Etemad (fantasai), Eric Portis, Ethan Marcotte, Geoff Graham, Gregory Wild-Smith, Ian Kilpatrick, Jen Simmons, Kenneth Rohde Christiansen, L. David Baron, Lea Verou, Martin Auswöger, Martine Dowden, Mike Riethmuller, Morten Stenshorne, Nicole Sullivan, Rune Lillesveen, Scott Jehl Scott Kellum, Stacy Kvernmo, Theresa O’Connor, Una Kravets, and many others have contributed to this specification.

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

- [aspect-ratio](#descdef-container-aspect-ratio), in § 5.1.5
- [block-size](#descdef-container-block-size), in § 5.1.4
- [@container](#at-ruledef-container), in § 4.4
- [container](#propdef-container), in § 4.3
- [\<container-condition\>](#typedef-container-condition), in § 4.4
- [container feature](#container-feature), in § 5
- [\<container-name\>](#typedef-container-name), in § 4.4
- [container-name](#propdef-container-name), in § 4.2
- [container query](#container-query), in § 4.4
- [container query length](#container-query-length), in § 6
- [container size query](#container-size-query), in § 5.1
- [container style query](#container-style-query), in § 5.2
- [container-type](#propdef-container-type), in § 4.1
- [CSSContainerRule](#csscontainerrule), in § 7.1
- [\<custom-ident\>](#valdef-container-name-custom-ident), in § 4.2
- [height](#descdef-container-height), in § 5.1.2
- inline-size
  - [descriptor for @container](#descdef-container-inline-size), in § 5.1.3
  - [value for contain](#valdef-contain-inline-size), in § 2
  - [value for container-type](#valdef-container-type-inline-size), in § 4.1
- [inline-size containment](#inline-size-containment), in § 3.1
- [landscape](#valdef-container-orientation-landscape), in § 5.1.6
- [none](#valdef-container-name-none), in § 4.2
- [normal](#valdef-container-type-normal), in § 4.1
- [orientation](#descdef-container-orientation), in § 5.1.6
- [portrait](#valdef-container-orientation-portrait), in § 5.1.6
- [query container](#query-container), in § 4.1
- [query container name](#query-container-name), in § 4.2
- [\<query-in-parens\>](#typedef-query-in-parens), in § 4.4
- [size](#valdef-container-type-size), in § 4.1
- [\<size-feature\>](#typedef-size-feature), in § 5.1
- [size features](#size-features), in § 5.1
- [\<style-condition\>](#typedef-style-condition), in § 4.4
- [\<style-feature\>](#typedef-style-feature), in § 5.2
- [style features](#style-features), in § 5.2
- [\<style-in-parens\>](#typedef-style-in-parens), in § 4.4
- [\<style-query\>](#typedef-style-query), in § 4.4
- [width](#descdef-container-width), in § 5.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="term-for-at-ruledef-keyframes"></a>@keyframes
- \[CSS-BOX-4\] defines the following terms:
  - <a id="term-for-content-box"></a>content box
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="term-for-fragmentation"></a>fragmentation
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="term-for-at-ruledef-layer"></a>@layer
  - <a id="term-for-cascade-dependent-keyword"></a>cascade-dependent keyword
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-initial-value"></a>initial value
  - <a id="term-for-longhand"></a>longhand property
  - <a id="term-for-valdef-all-revert"></a>revert
  - <a id="term-for-valdef-all-revert-layer"></a>revert-layer
  - <a id="term-for-shorthand-property"></a>shorthand
  - <a id="term-for-shorthand-property①"></a>shorthand property
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="term-for-at-ruledef-supports"></a>@supports
  - <a id="term-for-cssconditionrule"></a>CSSConditionRule
  - <a id="term-for-conditional-group-rule"></a>conditional group rule
  - <a id="term-for-css-feature-queries"></a>css feature queries
- \[CSS-CONTAIN-1\] defines the following terms:
  - <a id="term-for-propdef-contain"></a>contain
  - <a id="term-for-layout-containment"></a>layout containment
  - <a id="term-for-size-containment"></a>size containment
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="term-for-propdef-content-visibility"></a>content-visibility
  - <a id="term-for-layout-containment-box"></a>layout containment box
  - <a id="term-for-style-containment"></a>style containment
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="term-for-atomic-inline"></a>atomic inline
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-formatting-context"></a>formatting context
  - <a id="term-for-inline-level"></a>inline-level
  - <a id="term-for-inner-display-type"></a>inner display type
  - <a id="term-for-internal-ruby-box"></a>internal ruby box
  - <a id="term-for-internal-table-box"></a>internal table box
  - <a id="term-for-principal-box"></a>principal box
  - <a id="term-for-valdef-display-table"></a>table
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="term-for-propdef-font-size"></a>font-size
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="term-for-at-font-face-rule"></a>@font-face
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
  - <a id="term-for-selectordef-file-selector-button"></a>::file-selector-button
  - <a id="term-for-selectordef-first-letter"></a>::first-letter
  - <a id="term-for-selectordef-first-line"></a>::first-line
  - <a id="term-for-selectordef-marker"></a>::marker
  - <a id="term-for-selectordef-placeholder"></a>::placeholder
  - <a id="term-for-fictional-tag-sequence"></a>fictional tag sequence
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="term-for-selectordef-slotted"></a>::slotted()
- \[CSS-SHADOW-PARTS-1\] defines the following terms:
  - <a id="term-for-selectordef-part"></a>::part()
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="term-for-height"></a>height
  - <a id="term-for-intrinsic-size"></a>intrinsic size
  - <a id="term-for-min-content"></a>min-content size
  - <a id="term-for-propdef-min-width"></a>min-width
  - <a id="term-for-size"></a>size
  - <a id="term-for-width"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="term-for-propdef-aspect-ratio"></a>aspect-ratio
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="term-for-typedef-stylesheet"></a>\<stylesheet\>
  - <a id="term-for-at-rule"></a>at-rule
  - <a id="term-for-declaration"></a>declaration
  - <a id="term-for-style-rule"></a>style rule
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="term-for-style-change-event"></a>style change event
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="term-for-mult-zero-plus"></a>\*
  - <a id="term-for-mult-one-plus"></a>+
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-ratio-value"></a>\<ratio\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-em"></a>em
  - <a id="term-for-css-css-identifier"></a>identifier
  - <a id="term-for-relative-length"></a>relative length
  - <a id="term-for-small-viewport-size"></a>small viewport size
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
  - <a id="term-for-block-size"></a>block size
  - <a id="term-for-block-axis①"></a>block-axis
  - <a id="term-for-inline-axis"></a>inline axis
  - <a id="term-for-inline-size"></a>inline size
  - <a id="term-for-inline-axis①"></a>inline-axis
  - <a id="term-for-inline-size①"></a>inline-size
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-line-height"></a>line-height
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="term-for-mediaquerylist"></a>MediaQueryList
  - <a id="term-for-dom-window-matchmedia"></a>matchMedia(query)
- \[DOM\] defines the following terms:
  - <a id="term-for-concept-shadow-including-descendant"></a>shadow-including descendant
  - <a id="term-for-concept-shadow-including-inclusive-ancestor"></a>shadow-including inclusive ancestor
- \[FULLSCREEN\] defines the following terms:
  - <a id="term-for-css-pe-backdrop"></a>::backdrop
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="term-for-typedef-general-enclosed"></a>\<general-enclosed\>
  - <a id="term-for-media-feature"></a>media feature
  - <a id="term-for-media-query"></a>media query
  - <a id="term-for-valdef-media-not"></a>not
- \[SELECTORS-4\] defines the following terms:
  - <a id="term-for-ultimate-originating-element"></a>ultimate originating element
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="term-for-effect-value"></a>effect value
- \[WEBIDL\] defines the following terms:
  - <a id="term-for-Exposed"></a>Exposed

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
Dean Jackson; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
CSS Cascading and Inheritance Level 4 URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
David Baron; Elika Etemad; Chris Lilley. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 22 December 2020. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
CSS Containment Module Level 2 URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-contain-2&#x2F;](https://drafts.csswg.org/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 7 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 18 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 21 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-shadow-parts-1"></a>\[CSS-SHADOW-PARTS-1\]  
Tab Atkins Jr.; Fergal Daly. [CSS Shadow Parts](https://www.w3.org/TR/css-shadow-parts-1/). 15 November 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shadow-parts-1&#x2F;](https://www.w3.org/TR/css-shadow-parts-1/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-fullscreen"></a>\[FULLSCREEN\]  
Philip Jägenstedt. [Fullscreen API Standard](https://fullscreen.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fullscreen&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fullscreen.spec.whatwg.org/)

## <a id="property-index"></a>Property Index

| Name                | Value                                              | Initial                   | Applies to                | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value                                         |
|---------------------|----------------------------------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|-----------------|--------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-container⑤"></a></span><a href="#propdef-container">container</a>&#xA;      </strong> | \<'container-name'\> \[ / \<'container-type'\> \]? | see individual properties | see individual properties | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                              |
| <strong><span><a id="ref-for-propdef-container-name⑨"></a></span><a href="#propdef-container-name">container-name</a>&#xA;      </strong> | none \| \<custom-ident\>+                          | none                      | all elements              | no                        | n/a                       | not animatable            | per grammar     | the keyword none, or an ordered list of identifiers    |
| <strong><span><a id="ref-for-propdef-container-type①①"></a></span><a href="#propdef-container-type">container-type</a>&#xA;      </strong> | normal \|\| \[ size \| inline-size \]              | normal                    | all elements              | no                        | n/a                       | not animatable            | per grammar     | the keyword normal or one or more of size, inline-size |

<a id="ref-for-at-ruledef-container①②"></a>

### <a id="container-descriptor-table"></a>[@container](#at-ruledef-container) Descriptors

| Name                | Value                 | Initial | Type     |
|---------------------|-----------------------|---------|----------|
| <strong><span><a id="ref-for-descdef-container-aspect-ratio②"></a></span><a href="#descdef-container-aspect-ratio">aspect-ratio</a>&#xA;      </strong> | \<ratio\>             |         | range    |
| <strong><span><a id="ref-for-descdef-container-block-size②"></a></span><a href="#descdef-container-block-size">block-size</a>&#xA;      </strong> | \<length\>            |         | range    |
| <strong><span><a id="ref-for-descdef-container-height④"></a></span><a href="#descdef-container-height">height</a>&#xA;      </strong> | \<length\>            |         | range    |
| <strong><span><a id="ref-for-descdef-container-inline-size④"></a></span><a href="#descdef-container-inline-size">inline-size</a>&#xA;      </strong> | \<length\>            |         | range    |
| <strong><span><a id="ref-for-descdef-container-orientation③"></a></span><a href="#descdef-container-orientation">orientation</a>&#xA;      </strong> | portrait \| landscape |         | discrete |
| <strong><span><a id="ref-for-descdef-container-width④"></a></span><a href="#descdef-container-width">width</a>&#xA;      </strong> | \<length\>            |         | range    |

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is a diff spec over [CSS Containment Level 2](https://www.w3.org/TR/css-contain-2/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 2 as a reference. We will merge the Level 2 text into this draft once it reaches CR. [↵](#issue-2016550a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Containment 2 § 2 Strong Containment: the contain property](https://drafts.csswg.org/css-contain-2/#contain-property) [↵](#issue-d41d8cd9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Containment 2 § 3 Types of Containment](https://drafts.csswg.org/css-contain-2/#containment-types) [↵](#issue-d41d8cd9%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> In general, the relationship between an element’s inline size and it’s block size is unpredictable and non-monotonic, with the block size capable of shifting up and down arbitrarily as the inline size is changed. Infinite cycles are prevented by ensuring that layout does not revert to a previous (known-problematic) state, even if a naive analysis of the constraints would allow for such; in other words, layout always “moves forward”. We believe that current CSS layout specifications incorporate such rules, but to the extent that they don’t, please [inform the CSSWG](https://github.com/w3c/csswg-drafts/issues) so that these errors can be corrected. [↵](#issue-ac484479)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add CSSOM API for CSSContainerRule [\[Issue \#7033\]](https://github.com/w3c/csswg-drafts/issues/7033) [↵](#issue-fe7a1c72)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Container Queries should have a `matchContainer` method. This will be modeled on <code><a href="https://www.w3.org/TR/cssom-view/#dom-window-matchmedia">matchMedia()</a></code> and the <code><a href="https://www.w3.org/TR/cssom-view/#mediaquerylist">MediaQueryList</a></code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to `resizeObserver`, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205) [↵](#issue-2e3d6538)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Containment 2 § 4 Suppressing An Element’s Contents Entirely: the content-visibility property](https://drafts.csswg.org/css-contain-2/#content-visibility) [↵](#issue-d41d8cd9%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Containment 2 § 5 Privacy and Security Considerations](https://drafts.csswg.org/css-contain-2/#priv-sec) [↵](#issue-d41d8cd9%E2%91%A2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Containment 2 §  Changes](https://drafts.csswg.org/css-contain-2/#changes) [↵](#issue-d41d8cd9%E2%91%A3)
