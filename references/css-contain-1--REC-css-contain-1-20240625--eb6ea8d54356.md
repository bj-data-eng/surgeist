Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Containment Module Level 1](https://www.w3.org/TR/2024/REC-css-contain-1-20240625/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Containment Module Level 1

Source snapshot: https://www.w3.org/TR/2024/REC-css-contain-1-20240625/

Snapshot SHA-256: eb6ea8d5435697776878bee6f58e0284c73f4ecd682ee43381d99f97bc69f56b

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 2 source tables are presented as readable Markdown tables or explicit labeled layouts: 2 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Containment Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-contain"></a>

This CSS module describes the [contain](#propdef-contain) property, which indicates that the element’s subtree is independent of the rest of the page. This enables heavy optimizations by user agents when used well.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a W3C Recommendation using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). A W3C Recommendation is a specification that, after extensive consensus-building, is endorsed by W3C and its Members, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/policies/patent-policy/#sec-Requirements) for implementations.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-contain” in the title, like this: “\[css-contain\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-contain%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

This document was published by the CSS Working Group as a Recommendation using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes).

W3C recommends the wide deployment of this specification as a standard for the Web.

## <a id="intro"></a>1.  Introduction

Efficiently rendering a website relies on the user agent being able to detect what parts of the page are being displayed, which parts might affect the currently-displayed section, and what can be ignored.

There are various heuristics that can be used to guess when a given sub-tree is independent of the rest of the page in some manner, but they’re fragile, so innocuous changes to a page may inadvertently make it fail such heuristic tests, causing rendering to fall into a slow code path. There are also many things that would be good to isolate which are difficult or impossible to detect in a heuristic manner.

<a id="ref-for-propdef-contain①"></a>

To alleviate these problems and allow strong, predictable isolation of a subtree from the rest of the page, this specification defines a [contain](#propdef-contain) property.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-contain②"></a>

## <a id="contain-property"></a>2.  Strong Containment: the [contain](#propdef-contain) property

| Field               | Definition                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain"></a>contain                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) strict <a id="ref-for-comb-one①"></a>\| content <a id="ref-for-comb-one②"></a>\| \[ size [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) layout <a id="ref-for-comb-any①"></a>\|\| paint \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | See [below](#contain-applies)                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-contain-paint"></a><a id="ref-for-valdef-contain-layout"></a><a id="ref-for-valdef-contain-size"></a><a id="ref-for-valdef-contain-none"></a>the keyword [none](#valdef-contain-none) or one or more of [size](#valdef-contain-size), [layout](#valdef-contain-layout), [paint](#valdef-contain-paint)                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                                                                |

User agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-contain③"></a>

The [contain](#propdef-contain) property allows an author to indicate that an element and its contents are, as much as possible, <em>independent</em> of the rest of the document tree. This allows user agents to utilize much stronger optimizations when rendering a page using <a id="ref-for-propdef-contain④"></a>contain properly, and allows authors to be confident that their page won’t accidentally fall into a slow code path due to an innocuous change.

<a id="valdef-contain-none"></a>none  
This value indicates that the property has no effect. The element renders as normal, with no containment effects applied.

<a id="valdef-contain-strict"></a>strict  
<a id="ref-for-containment"></a>

This value computes to size layout paint, and thus turns on all forms of [containment](#containment) for the element.

<a id="valdef-contain-content"></a>content  
<a id="ref-for-size-containment"></a>

<a id="ref-for-containment①"></a>

This value computes to layout paint, and thus turns on all forms of [containment](#containment) <em>except</em> [size containment](#size-containment) for the element.

<a id="ref-for-propdef-contain⑤"></a>

<a id="ref-for-size-containment①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [contain: content](#propdef-contain) is reasonably "safe" to apply widely; its effects are fairly minor in practice, and most content won’t run afoul of its restrictions. However, because it doesn’t apply [size containment](#size-containment), the element can still respond to the size of its contents, which can cause layout-invalidation to percolate further up the tree than desired. Use <a id="ref-for-propdef-contain⑥"></a>contain: strict when possible, to gain as much containment as you can.

<a id="valdef-contain-size"></a>size  
<a id="ref-for-size-containment-box"></a>

<a id="ref-for-size-containment②"></a>

The value turns on [size containment](#size-containment) for the element. This ensures that the [containment box](#size-containment-box) can be laid out without needing to examine its descendants.

<a id="valdef-contain-layout"></a>layout  
<a id="ref-for-layout-containment-box"></a>

<a id="ref-for-layout-containment"></a>

This value turns on [layout containment](#layout-containment) for the element. This ensures that the [containment box](#layout-containment-box) is <em>totally opaque</em> for layout purposes; nothing outside can affect its internal layout, and vice versa.

<a id="valdef-contain-paint"></a>paint  
<a id="ref-for-paint-containment-box"></a>

<a id="ref-for-paint-containment"></a>

This value turns on [paint containment](#paint-containment) for the element. This ensures that the descendants of the [containment box](#paint-containment-box) don’t display outside its bounds, so if an element is off-screen or otherwise not visible, its descendants are also guaranteed to be not visible.

<a id="ref-for-propdef-contain⑦"></a>

<a id="ref-for-elementdef-svg"></a>

<a id="contain-applies"></a>This property generally applies to all elements (including [CSS Pseudo-Elements 4 § 4.1 Generated Content Pseudo-elements: ::before and ::after](https://www.w3.org/TR/css-pseudo-4/#generated-content)), although some types of containment have no effect on some elements, as detailed in [§ 3 Types of Containment](#containment-types). In addition, in the case of [\[SVG2\]](#biblio-svg2), the [contain](#propdef-contain) property only applies to <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code> elements that have an associated CSS layout box.

<a id="ref-for-propdef-contain⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ace3db03"></a> [contain](#propdef-contain) is useful when used widely on a page, particularly when a page contains a lot of "widgets" which are all independent.
>
> For example, assume a micropost social network had markup something like this:
>
> ```text
> <body>
>   <aside>...</aside>
>   <section>
>     <h2>Messages</h2>
>     <article>
>       Lol, check out this dog: images.example.com/jsK3jkl
>     </article>
>     <article>
>       I had a ham sandwich today. #goodtimes
>     </article>
>     <article>
>       I have political opinions that you need to hear!
>     </article>
>     …
>   </section>
> </body>
> ```
>
> <a id="ref-for-propdef-contain⑨"></a>
>
> There are probably a <em>lot</em> of messages displayed on the site, but each is independent and won’t affect anything else on the site. As such, each can be marked with [contain: content](#propdef-contain) to communicate this to the user agent, so it can optimize the page and skip a lot of computation for messages that are off-screen. If the size of each message is known ahead of time, <a id="ref-for-propdef-contain①⓪"></a>contain: strict can be applied to communicate further restrictions.

<a id="ref-for-used-value"></a>

<a id="ref-for-propdef-contain①①"></a>

<a id="ref-for-the-html-element"></a>

<a id="ref-for-the-body-element"></a>

<a id="ref-for-valdef-contain-none①"></a>

<a id="ref-for-the-body-element①"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-canvas-background"></a>

Additionally, when the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the [contain](#propdef-contain) property on either the HTML <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> elements is anything other than [none](#valdef-contain-none), propagation of properties from the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), the viewport, or the [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background), is disabled. Notably, this affects:

- <a id="ref-for-propdef-writing-mode"></a>

  <a id="ref-for-propdef-direction"></a>

  <a id="ref-for-propdef-text-orientation"></a>

  [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) (see [CSS Writing Modes 3 § 8 The Principal Writing Mode](https://www.w3.org/TR/css-writing-modes-3/#principal-flow))

- <a id="ref-for-propdef-overflow"></a>

  [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) and its longhands (see [CSS Overflow 3 § 3.3 Overflow Viewport Propagation](https://www.w3.org/TR/css-overflow-3/#overflow-propagation))

- <a id="ref-for-propdef-background"></a>

  [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) and its longhands (see [CSS Backgrounds 3 § 2.11.2 The Canvas Background and the HTML \<body\> Element](https://www.w3.org/TR/css-backgrounds-3/#body-background))

<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-canvas-background①"></a>

<a id="ref-for-the-html-element①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Propagation to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), the viewport, or the [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background), of properties set on the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> element itself is unaffected.

## <a id="containment-types"></a>3.  Types of Containment

<a id="ref-for-containment②"></a>

There are several varieties of <a id="containment"></a>containment that an element can be subject to, restricting the effects that its descendants can have on the rest of the page in various ways. [Containment](#containment) enables much more powerful optimizations by user agents, and helps authors compose their page out of functional units, as it limits how widely a given change can affect a document.

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Specification authors introducing new properties or mechanisms
	need to consider whether and how the various types of containment
	affect what they are introducing,
	and include in their specification any effect not described here.</strong>

### <a id="containment-size"></a>3.1.  Size Containment

<a id="ref-for-principal-box"></a>

Giving an element <a id="size-containment"></a>size containment makes its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) a <a id="size-containment-box"></a>size containment box and has the following effects:

1.  <a id="ref-for-intrinsic-size"></a>

    <a id="ref-for-size-containment-box①"></a>

    <a id="ref-for-sizing-as-if-empty"></a>

    The [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) of the [size containment box](#size-containment-box) are determined as if the element had no content, following the same logic as when [sizing as if empty](#sizing-as-if-empty).

    <a id="ref-for-valdef-width-min-content"></a>

    <a id="ref-for-valdef-width-max-content"></a>

    <a id="ref-for-grid-track"></a>

    <a id="ref-for-fit-content-size"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This affects explicit invocations of the [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content) or [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content) keywords, as well as any calculation that depends on these measurement, such as sizing [grid tracks](https://www.w3.org/TR/css-grid-2/#grid-track) into which a size contained item is placed, or if [fit-content sizing](https://www.w3.org/TR/css-sizing-3/#fit-content-size) the containment box’s parent.

2.  <a id="ref-for-size-containment-box②"></a>

    Laying out a [size containment box](#size-containment-box) and its content is conceptually done in two phases:

    <a id="sizing-as-if-empty"></a>Sizing as if empty  
    <a id="ref-for-selectordef-marker"></a>

    <a id="ref-for-selectordef-after"></a>

    <a id="ref-for-selectordef-before"></a>

    <a id="ref-for-size-containment-box③"></a>

    <a id="ref-for-propdef-height"></a>

    <a id="ref-for-propdef-width"></a>

    <a id="ref-for-used-value①"></a>

    The [used](https://www.w3.org/TR/css-cascade-5/#used-value) [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of the [containment box](#size-containment-box) are determined as if performing a normal layout of the box, except that it is treated as having no content—​not even through pseudo elements such as [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), or [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker).

    <a id="ref-for-replaced-element"></a>

    <a id="ref-for-natural-dimensions"></a>

    <a id="ref-for-natural-aspect-ratio"></a>

    [Replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) must be treated as having a [natural](https://www.w3.org/TR/css-images-3/#natural-dimensions) width and height of 0 and no [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio).

    <a id="ref-for-natural-aspect-ratio①"></a>

    <a id="ref-for-propdef-aspect-ratio"></a>

    <a id="ref-for-preferred-aspect-ratio"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Size containment only suppresses the [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), so properties like [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) which affect that [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio) directly are honored.

    <a id="ref-for-size-containment-box④"></a>

    All CSS properties of the [size containment box](#size-containment-box) are taken into account as they would be when performing layout normally. Other specifications may make specific exemptions.

    <a id="ref-for-sizing-property"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Even when the element’s [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property) specify an intrinsic size, this does not necessarily make the element zero-sized: properties set on the element itself continue to be taken into account, which can cause it to be larger.

    <a id="laying-out-in-place"></a>Laying out in-place  
    <a id="ref-for-size-containment-box⑤"></a>

    The [containment box](#size-containment-box)'s content (including any pseudo-elements) must then be laid out into the now fixed-size <a id="ref-for-size-containment-box⑥"></a>containment box normally.

    <a id="ref-for-size-containment③"></a>

    <a id="ref-for-layout-containment①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: [Size containment](#size-containment) does not suppress baseline alignment. See [layout containment](#layout-containment) for that.

3.  <a id="ref-for-size-containment-box⑦"></a>

    <a id="ref-for-monolithic"></a>

    [Size containment boxes](#size-containment-box) are [monolithic](https://www.w3.org/TR/css-break-3/#monolithic) (See [CSS Fragmentation 3 § 4.1 Possible Break Points](https://www.w3.org/TR/css-break-3/#possible-breaks)).

<a id="ref-for-propdef-aspect-ratio①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eadb9c00"></a> Given the following markup and style, the image would be sized to 100px by 100px, as the aspect ratio set by the [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) property takes effect.
>
> ```text
> img {
>   width: 100px;
>   aspect-ratio: 1/1;
>   contain: size;
> }
> 
> <img src="https://www.example.com/300x100.jpg">
> ```
>
> <a id="ref-for-propdef-aspect-ratio②"></a>
>
> <a id="ref-for-natural-aspect-ratio②"></a>
>
> <a id="ref-for-natural-height"></a>
>
> If the [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) property had not been declared, the image would have been 100px by 0px, as its [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) is suppressed, and its [natural height](https://www.w3.org/TR/css-images-3/#natural-height) is treated as 0.

<a id="ref-for-size-containment④"></a>

However, giving an element [size containment](#size-containment) has no effect if any of the following are true:

- <a id="ref-for-principal-box①"></a>

  <a id="ref-for-propdef-display"></a>

  if the element does not generate a [principal box](https://www.w3.org/TR/css-display-3/#principal-box) (as is the case with [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) or <a id="ref-for-propdef-display①"></a>display: none)

- <a id="ref-for-inner-display-type"></a>

  <a id="ref-for-valdef-display-table"></a>

  if its [inner display type](https://www.w3.org/TR/css-display-3/#inner-display-type) is [table](https://www.w3.org/TR/css-display-3/#valdef-display-table)

- <a id="ref-for-principal-box②"></a>

  <a id="ref-for-internal-table-box"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal table box](https://www.w3.org/TR/css-display-3/#internal-table-box)

- <a id="ref-for-principal-box③"></a>

  <a id="ref-for-internal-ruby-box"></a>

  <a id="ref-for-atomic-inline"></a>

  <a id="ref-for-inline-level"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal ruby box](https://www.w3.org/TR/css-display-3/#internal-ruby-box) or a [non-atomic](https://www.w3.org/TR/css-display-3/#atomic-inline) [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) box

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-height①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Internal table boxes, which do not include table captions, are excluded, because the table layout algorithm does not allow boxes to become smaller than their inflow content. Sizing a table cell as if it was empty and then laying out its content inside without changing the size is effectively an undefined operation. Manually setting the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) or [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) properties to 0 cannot make it smaller than its content. This concern does not apply to table captions, which are perfectly capable of having a fixed size that is independent of their content.

#### <a id="containment-size-opt"></a>3.1.1.  Possible Size-Containment Optimizations

<em>This section is non-normative.</em>

<a id="ref-for-size-containment⑤"></a>

<a id="ref-for-size-containment-box⑧"></a>

By itself, [size containment](#size-containment) does not offer much optimization opportunity. Its primary benefit on its own is that tools which want to lay out the [containment box](#size-containment-box)'s contents based on the <a id="ref-for-size-containment-box⑨"></a>containment box's size (such as a JS library implementing the "container query" concept) can do so without fear of "infinite loops", where having a child’s size respond to the size of the <a id="ref-for-size-containment-box①⓪"></a>containment box causes the <a id="ref-for-size-containment-box①①"></a>containment box's size to change as well, possibly triggering <em>further</em> changes in how the child sizes itself and possibly thus more changes to the <a id="ref-for-size-containment-box①②"></a>containment box's size, ad infinitum.

<a id="ref-for-layout-containment②"></a>

When paired with [layout containment](#layout-containment), though, possible optimizations that can be enabled include (but are not limited to):

1.  <a id="ref-for-size-containment-box①③"></a>

    When the style or contents of a descendant of the [containment box](#size-containment-box) is changed, calculating what part of the DOM tree is "dirtied" and might need to be re-laid out can stop at the <a id="ref-for-size-containment-box①④"></a>containment box.

2.  <a id="ref-for-size-containment-box①⑤"></a>

    <a id="ref-for-laying-out-in-place"></a>

    When laying out the page, if the [containment box](#size-containment-box) is off-screen or obscured, the layout of its contents (i.e. "[laying out in-place](#laying-out-in-place)") can be delayed or done at a lower priority.

### <a id="containment-layout"></a>3.2.  Layout Containment

<a id="ref-for-principal-box④"></a>

Giving an element <a id="layout-containment"></a>layout containment makes its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) a <a id="layout-containment-box"></a>layout containment box and has the following effects:

1.  <a id="ref-for-layout-containment-box①"></a>

    <a id="ref-for-establish-an-independent-formatting-context"></a>

    The [layout containment box](#layout-containment-box) [establishes an independent formatting context](https://www.w3.org/TR/css-display-3/#establish-an-independent-formatting-context).

2.  <a id="ref-for-fragmentation-container"></a>

    <a id="ref-for-fragmentation-context"></a>

    <a id="ref-for-layout-containment③"></a>

    <a id="ref-for-layout-containment-box②"></a>

    <a id="ref-for-fragmented-flow"></a>

    <a id="ref-for-fragmentation"></a>

    If at least one [fragmentation container](https://www.w3.org/TR/css-break-3/#fragmentation-container) of a [fragmentation context](https://www.w3.org/TR/css-break-3/#fragmentation-context) has [layout containment](#layout-containment), or if at least one <a id="ref-for-fragmentation-container①"></a>fragmentation container of a <a id="ref-for-fragmentation-context①"></a>fragmentation context is a descendant of [layout containment box](#layout-containment-box) <strong>and</strong> at least one subsequent <a id="ref-for-fragmentation-container②"></a>fragmentation container of the same <a id="ref-for-fragmentation-context②"></a>fragmentation context is not a descendant of that same element with layout containment, then the first <a id="ref-for-layout-containment-box③"></a>layout containment box which is either a <a id="ref-for-fragmentation-container③"></a>fragmentation container itself or is an ancestor of a <a id="ref-for-fragmentation-container④"></a>fragmentation container must “trap” the remainder of the [fragmented flow](https://www.w3.org/TR/css-break-3/#fragmented-flow): [fragmentation](https://www.w3.org/TR/css-break-3/#fragmentation) must not continue past the <a id="ref-for-layout-containment④"></a>layout containment boundary, and the last <a id="ref-for-fragmentation-container⑤"></a>fragmentation container within the first <a id="ref-for-layout-containment⑤"></a>layout containment boundary is treated as if it is the last <a id="ref-for-fragmentation-container⑥"></a>fragmentation container in its <a id="ref-for-fragmentation-context③"></a>fragmentation context.

    <a id="ref-for-fragmentation-container⑦"></a>

    <a id="ref-for-fragmentation-context④"></a>

    <a id="ref-for-fragmented-flow①"></a>

    If subsequent [fragmentation containers](https://www.w3.org/TR/css-break-3/#fragmentation-container) in the [fragmentation context](https://www.w3.org/TR/css-break-3/#fragmentation-context) are only generated when more content remains in the [fragmented flow](https://www.w3.org/TR/css-break-3/#fragmented-flow), then they are not generated. If they would exist regardless, they remain part of the <a id="ref-for-fragmentation-context⑤"></a>fragmentation context, but do not receive any content from the <a id="ref-for-fragmented-flow②"></a>fragmented flow.

    <a id="ref-for-selectordef-nth-fragment"></a>

    <a id="ref-for-layout-containment⑥"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: At the time of writing, no stable specification is affected by this point. Only specifications that would enable some (but not all) fragmentation containers of a fragmentation context to be layout-contained (or descendants of a layout contained element) are concerned. This is not the case of [\[CSS-PAGE-3\]](#biblio-css-page-3) nor of [\[CSS-MULTICOL-1\]](#biblio-css-multicol-1). This requirement is nonetheless included because several mechanisms that would make this a possibility have been considered (e.g.: [\[CSS-REGIONS-1\]](#biblio-css-regions-1), [::nth-fragment()](https://www.w3.org/TR/css-overflow-4/#selectordef-nth-fragment), a hypothetical selector for individual columns of a multicol…), and the guarantees that layout containment is intended to offer would not be realized if such mechanisms did not abide by this rule. \[CSS-REGIONS-1\] has details over how [layout containment](#layout-containment) affects regions.

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-2b10faa8"></a>
    > ```text
    > <article>Lorem ipsum…</article>
    > <div id=a></div>
    > <aside>
    >   <div id=b></div>
    >   <div id=c></div>
    > </aside>
    > <aside>
    >   <div id=d></div>
    >   <div id=e></div>
    > </aside>
    > <div id=f></div>
    > ```
    >
    > ```text
    > article {flow-into: foo;}
    > #a, #b, #c, #d, #e, #f {flow-from: foo;}
    > aside {contain: layout}
    > ```
    >
    > <a id="ref-for-layout-containment-box④"></a>
    >
    > In this [\[CSS-REGIONS-1\]](#biblio-css-regions-1) example, content can flow from `#a` to `#b`, from `#b` to `#c`. However as `#c` is the last fragment container in the first [layout containment box](#layout-containment-box) it traps all the remaining content, and nothing gets flowed into `#d`, `#e`, or `#f`.

3.  <a id="ref-for-propdef-overflow①"></a>

    <a id="ref-for-valdef-overflow-visible"></a>

    <a id="ref-for-valdef-overflow-clip"></a>

    <a id="ref-for-ink-overflow"></a>

    If the computed value of the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property is either [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible) or [clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip) or a combination thereof, any overflow must be treated as [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow).

4.  <a id="ref-for-layout-containment-box⑤"></a>

    <a id="ref-for-absolute-positioning-containing-block"></a>

    <a id="ref-for-fixed-positioning-containing-block"></a>

    The [layout containment box](#layout-containment-box) establishes an [absolute positioning containing block](https://www.w3.org/TR/css-position-3/#absolute-positioning-containing-block) and a [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block).

5.  <a id="ref-for-layout-containment-box⑥"></a>

    <a id="ref-for-x43"></a>

    The [layout containment box](#layout-containment-box) creates a [stacking context](https://www.w3.org/TR/CSS21/visuren.html#x43).

6.  <a id="ref-for-forced-break"></a>

    <a id="ref-for-layout-containment-box⑦"></a>

    [Forced breaks](https://www.w3.org/TR/css-break-3/#forced-break) are allowed within [layout containment boxes](#layout-containment-box) but do not propagate to the parent as otherwise described in [CSS Fragmentation 3 § 3.1 Breaks Between Boxes: the break-before and break-after properties](https://www.w3.org/TR/css-break-3/#break-between).

    <a id="ref-for-forced-break①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This introduces the previously non-existent possibility that [forced breaks](https://www.w3.org/TR/css-break-3/#forced-break) may occur between a box and its container (See [CSS Fragmentation 3 § 4.1 Possible Break Points](https://www.w3.org/TR/css-break-3/#possible-breaks)).

7.  <a id="ref-for-propdef-vertical-align"></a>

    <a id="ref-for-layout-containment-box⑧"></a>

    For the purpose of the [vertical-align](https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align) property, or any other property whose effects need to relate the position of the [layout containment box](#layout-containment-box)'s baseline to something other than its descendants, the <a id="ref-for-layout-containment-box⑨"></a>containment box is treated as having no baseline.

<a id="ref-for-layout-containment⑦"></a>

However, giving an element [layout containment](#layout-containment) has no effect if any of the following are true:

- <a id="ref-for-principal-box⑤"></a>

  <a id="ref-for-propdef-display②"></a>

  if the element does not generate a [principal box](https://www.w3.org/TR/css-display-3/#principal-box) (as is the case with [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) or <a id="ref-for-propdef-display③"></a>display: none)

- <a id="ref-for-principal-box⑥"></a>

  <a id="ref-for-internal-table-box①"></a>

  <a id="ref-for-valdef-display-table-cell"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal table box](https://www.w3.org/TR/css-display-3/#internal-table-box) other than [table-cell](https://www.w3.org/TR/css-display-3/#valdef-display-table-cell)

- <a id="ref-for-principal-box⑦"></a>

  <a id="ref-for-internal-ruby-box①"></a>

  <a id="ref-for-atomic-inline①"></a>

  <a id="ref-for-inline-level①"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal ruby box](https://www.w3.org/TR/css-display-3/#internal-ruby-box) or a [non-atomic](https://www.w3.org/TR/css-display-3/#atomic-inline) [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) box

#### <a id="containment-layout-opt"></a>3.2.1.  Possible Layout-Containment Optimizations

<em>This section is non-normative.</em>

<a id="ref-for-layout-containment⑧"></a>

Possible optimizations that can be enabled by [layout containment](#layout-containment) include (but are not limited to):

1.  <a id="ref-for-layout-containment-box①⓪"></a>

    When laying out the page, the contents of separate [containment boxes](#layout-containment-box) can be laid out in parallel, as they’re guaranteed not to affect each other.

2.  <a id="ref-for-layout-containment-box①①"></a>

    When laying out the page, if the [containment box](#layout-containment-box) is off-screen or obscured and the layout of the visible parts of the screen do not depend on the size of the <a id="ref-for-layout-containment-box①②"></a>containment box (for example, if the <a id="ref-for-layout-containment-box①③"></a>containment box is near the end of a block container, and you’re viewing the beginning of the block container), the layout of the <a id="ref-for-layout-containment-box①④"></a>containment box' contents can be delayed or done at a lower priority.

    <a id="ref-for-size-containment⑥"></a>

    (When paired with [size containment](#size-containment), this optimization can be applied more liberally.)

### <a id="containment-paint"></a>3.3.  Paint Containment

<a id="ref-for-principal-box⑧"></a>

Giving an element <a id="paint-containment"></a>paint containment makes its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) a <a id="paint-containment-box"></a>paint containment box and has the following effects:

1.  <a id="ref-for-ink-overflow①"></a>

    <a id="ref-for-scrollable-overflow"></a>

    <a id="ref-for-padding-edge"></a>

    <a id="ref-for-paint-containment-box①"></a>

    <a id="ref-for-corner-clipping"></a>

    <a id="ref-for-propdef-overflow②"></a>

    <a id="ref-for-propdef-resize"></a>

    <a id="ref-for-propdef-text-overflow"></a>

    The contents of the element including any [ink](https://www.w3.org/TR/css-overflow-3/#ink-overflow) or [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) must be clipped to the [padding edge](https://www.w3.org/TR/CSS2/box.html#padding-edge) of the [paint containment box](#paint-containment-box), taking [corner clipping](https://drafts.csswg.org/css-backgrounds-3/#corner-clipping) into account. This does not include the creation of any mechanism to access or indicate the presence of the clipped content; nor does it inhibit the creation of any such mechanism through other properties, such as [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [resize](https://www.w3.org/TR/css-ui-4/#propdef-resize), or [text-overflow](https://www.w3.org/TR/css-ui-3/#propdef-text-overflow).

    <a id="ref-for-overflow-clip-edge"></a>

    <a id="ref-for-padding-edge①"></a>

    <a id="ref-for-propdef-overflow-clip-margin"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The next level of this specification [\[CSS-CONTAIN-2\]](#biblio-css-contain-2) refines this effect to apply to the [overflow clip edge](https://www.w3.org/TR/css-overflow-4/#overflow-clip-edge) rather than the [padding edge](https://www.w3.org/TR/CSS2/box.html#padding-edge), in order to take the new [overflow-clip-margin](https://www.w3.org/TR/css-overflow-4/#propdef-overflow-clip-margin) property into account. For implementations that do not support <a id="ref-for-propdef-overflow-clip-margin①"></a>overflow-clip-margin, the effect is identical.

    <a id="ref-for-propdef-overflow-x"></a>

    <a id="ref-for-propdef-overflow-y"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: The behavior is described in this paragraph is equivalent to changing [overflow-x: visible](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-x) into <a id="ref-for-propdef-overflow-x①"></a>overflow-x: clip and [overflow-y: visible](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-y) into <a id="ref-for-propdef-overflow-y①"></a>overflow-y: clip at used value time, while leaving other values of <a id="ref-for-propdef-overflow-x②"></a>overflow-x and <a id="ref-for-propdef-overflow-y②"></a>overflow-y unchanged.

2.  <a id="ref-for-paint-containment-box②"></a>

    <a id="ref-for-absolute-positioning-containing-block①"></a>

    <a id="ref-for-fixed-positioning-containing-block①"></a>

    The [paint containment box](#paint-containment-box) establishes an [absolute positioning containing block](https://www.w3.org/TR/css-position-3/#absolute-positioning-containing-block) and a [fixed positioning containing block](https://www.w3.org/TR/css-position-3/#fixed-positioning-containing-block).

3.  <a id="ref-for-paint-containment-box③"></a>

    <a id="ref-for-x43①"></a>

    The [paint containment box](#paint-containment-box) creates a [stacking context](https://www.w3.org/TR/CSS21/visuren.html#x43).

4.  <a id="ref-for-paint-containment-box④"></a>

    <a id="ref-for-establish-an-independent-formatting-context①"></a>

    The [paint containment box](#paint-containment-box) [establishes an independent formatting context](https://www.w3.org/TR/css-display-3/#establish-an-independent-formatting-context).

<a id="ref-for-paint-containment①"></a>

However, giving an element [paint containment](#paint-containment) has no effect if any of the following are true:

- <a id="ref-for-principal-box⑨"></a>

  <a id="ref-for-propdef-display④"></a>

  if the element does not generate a [principal box](https://www.w3.org/TR/css-display-3/#principal-box) (as is the case with [display: contents](https://www.w3.org/TR/css-display-3/#propdef-display) or <a id="ref-for-propdef-display⑤"></a>display: none)

- <a id="ref-for-principal-box①⓪"></a>

  <a id="ref-for-internal-table-box②"></a>

  <a id="ref-for-valdef-display-table-cell①"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal table box](https://www.w3.org/TR/css-display-3/#internal-table-box) other than [table-cell](https://www.w3.org/TR/css-display-3/#valdef-display-table-cell)

- <a id="ref-for-principal-box①①"></a>

  <a id="ref-for-internal-ruby-box②"></a>

  <a id="ref-for-atomic-inline②"></a>

  <a id="ref-for-inline-level②"></a>

  if its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) is an [internal ruby box](https://www.w3.org/TR/css-display-3/#internal-ruby-box) or a [non-atomic](https://www.w3.org/TR/css-display-3/#atomic-inline) [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) box

#### <a id="containment-paint-opt"></a>3.3.1.  Possible Paint-Containment Optimizations

<em>This section is non-normative.</em>

<a id="ref-for-paint-containment②"></a>

Possible optimizations that can be enabled by [paint containment](#paint-containment) include (but are not limited to):

1.  <a id="ref-for-paint-containment-box⑤"></a>

    If the [containment box](#paint-containment-box) is off-screen or obscured, the UA can usually skip trying to paint its contents, as they’re guaranteed to be off-screen/obscured as well.

    <a id="ref-for-funcdef-filter-blur"></a>

    <a id="ref-for-paint-containment③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Some paint effects such as the [blur()](https://www.w3.org/TR/filter-effects-1/#funcdef-filter-blur) filter from [\[FILTER-EFFECTS-1\]](#biblio-filter-effects-1) have non local effects. The user agent needs to keep track of these, as it may need to repaint parts of an element with such a filter when its descendents change, even if they have [paint containment](#paint-containment) and could otherwise be skipped.

2.  <a id="ref-for-propdef-overflow③"></a>

    <a id="ref-for-propdef-resize①"></a>

    <a id="ref-for-propdef-text-overflow①"></a>

    Unless the clipped content is made accessible via a separate mechanism such as the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [resize](https://www.w3.org/TR/css-ui-4/#propdef-resize), or [text-overflow](https://www.w3.org/TR/css-ui-3/#propdef-text-overflow) properties, the UA can reserve "canvas" space for the box exactly the box’s size. (In similar, scrollable, situations, like <a id="ref-for-propdef-overflow④"></a>overflow: hidden, it’s possible to scroll to the currently-clipped content, so UAs often predictively overpaint somewhat so there’s something to see as soon as the scroll happens, rather than a frame later.)

3.  Because they are guaranteed to be stacking contexts, scrolling elements can be painted into a single GPU layer.

## <a id="privacy"></a>4.  Privacy Considerations<a id="priv-sec"></a>

There are no known privacy impacts of the features in this specification.

## <a id="security"></a>5.  Security Considerations

There are no known security impacts of the features in this specification.

Like any other CSS specification, it affects the rendering of the document, but does not introduce any special ability to present content in a misleading way that was not previously available through other CSS modules and that isn’t inherent to the act of formatting the document.

## <a id="changes"></a>Appendix A. Changes

This appendix is <em>informative</em>.

### <a id="2022-10-25-changes"></a>Changes from the [Recommendation of 25 October 2022](https://www.w3.org/TR/2022/REC-css-contain-1-20221025/)

Three proposed corrections were formally incorporated into the normative text:

<a id="c1"></a>Proposed Correction 1:  
<a id="ref-for-valdef-contain-content"></a>

<a id="ref-for-valdef-contain-strict"></a>

<a id="ref-for-propdef-contain①②"></a>

A minor adjustment was made to the way the computed value of the [contain](#propdef-contain) property is determined: the shortcut values ([strict](#valdef-contain-strict) and [content](#valdef-contain-content)), instead of computing to themselves, compute to the corresponding keywords. Given that the effect is identical, this allows implementations not to store the precise syntax through which this was achieved. Also, thanks to the shortest-serialization principle, this makes this unimportant difference non observable via serialization.

<a id="c2"></a>Proposed Correction 2:  
The description of size containment was somewhat ambiguous, causing implementers to have some doubts about the precise intended effects in certain cases. It was replaced with a more precise description, in order to clarify what is meant without changing the intended behavior.

<a id="c3"></a>Proposed Correction 3:  
<a id="ref-for-the-body-element③"></a>

<a id="ref-for-the-body-element②"></a>

<a id="ref-for-the-html-element②"></a>

The CSS Working Group had forgotten to consider the effects of containment on the HTML <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> elements, particularly in consideration of the fact that for legacy reasons, some properties can propagate outwards from the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element. The text added addresses this oversight.

An [implementation report](https://drafts.csswg.org/css-contain-1/implementation-report-2022-09) details their implementation in multiple engines.

### <a id="old-changes"></a> Earlier Changes<a id="2020-12-22-changes"></a><a id="2019-11-21-changes"></a><a id="2019-04-30-changes"></a><a id="2018-11-08-changes"></a><a id="2018-05-24-changes"></a><a id="2017-08-08-changes"></a><a id="2017-04-19-changes"></a><a id="fpwd-changes"></a>

Details about earlier changes to this specification can be found in [the Changes section of its previous publication](https://www.w3.org/TR/2022/REC-css-contain-1-20221025/).

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

- [contain](#propdef-contain), in § 2
- [containment](#containment), in § 3
- [content](#valdef-contain-content), in § 2
- [Laying out in-place](#laying-out-in-place), in § 3.1
- [layout](#valdef-contain-layout), in § 2
- [layout containment](#layout-containment), in § 3.2
- [layout containment box](#layout-containment-box), in § 3.2
- [none](#valdef-contain-none), in § 2
- [paint](#valdef-contain-paint), in § 2
- [paint containment](#paint-containment), in § 3.3
- [paint containment box](#paint-containment-box), in § 3.3
- [size](#valdef-contain-size), in § 2
- [size containment](#size-containment), in § 3.1
- [size containment box](#size-containment-box), in § 3.1
- [Sizing as if empty](#sizing-as-if-empty), in § 3.1
- [strict](#valdef-contain-strict), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="fd42d148"></a>canvas background
  - <a id="dcf09107"></a>corner clipping
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="99ae4c34"></a>forced break
  - <a id="8a4e9d7f"></a>fragmentation
  - <a id="a40115fc"></a>fragmentation container
  - <a id="7a1e79aa"></a>fragmentation context
  - <a id="3485678f"></a>fragmented flow
  - <a id="4ea821fb"></a>monolithic
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="1a2b1083"></a>used value
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="f2938b9c"></a>atomic inline
  - <a id="2ccfe434"></a>display
  - <a id="8ae53990"></a>establishes an independent formatting context
  - <a id="e26aa9bf"></a>initial containing block
  - <a id="4f918eb5"></a>inline-level
  - <a id="b7bfe5ca"></a>inner display type
  - <a id="9aa7b5d0"></a>internal ruby box
  - <a id="f4938aa4"></a>internal table box
  - <a id="93f98063"></a>principal box
  - <a id="299e10e4"></a>replaced element
  - <a id="3559c120"></a>table
  - <a id="73d7ce97"></a>table-cell
- \[CSS-GRID-2\] defines the following terms:
  - <a id="b4a14210"></a>grid track
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="ffedca23"></a>natural aspect ratio
  - <a id="487e1aa9"></a>natural dimension
  - <a id="b9cef6bf"></a>natural height
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="d9b4880c"></a>clip
  - <a id="00d2e365"></a>ink overflow
  - <a id="add377f4"></a>overflow
  - <a id="4ed1ab05"></a>overflow-x
  - <a id="e55d9f25"></a>overflow-y
  - <a id="e3488cd0"></a>scrollable overflow
  - <a id="855a7562"></a>visible
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="bd452b8b"></a>::nth-fragment()
  - <a id="c26bd888"></a>overflow clip edge
  - <a id="555f477c"></a>overflow-clip-margin
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="8bf8e632"></a>absolute positioning containing block
  - <a id="1febb260"></a>fixed positioning containing block
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
  - <a id="b6b63ba4"></a>::marker
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="f15ee6fc"></a>fit-content size
  - <a id="5ad01cca"></a>height
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="8cdc912e"></a>max-content
  - <a id="d3da3539"></a>min-content
  - <a id="2ac08cff"></a>sizing property
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="d1cbb104"></a>aspect-ratio
  - <a id="b03f7c8f"></a>preferred aspect ratio
- \[CSS-UI-3\] defines the following terms:
  - <a id="87d8bda0"></a>text-overflow
- \[CSS-UI-4\] defines the following terms:
  - <a id="c034f069"></a>resize
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="8664e85f"></a>text-orientation
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="8e0289d3"></a>padding edge
  - <a id="a50c2771"></a>stacking context
  - <a id="5a3d5e86"></a>vertical-align
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="a775a088"></a>blur()
- \[HTML\] defines the following terms:
  - <a id="2f0492ac"></a>body
  - <a id="c3dd181e"></a>html
- \[SVG2\] defines the following terms:
  - <a id="45c278c3"></a>svg

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 3 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-regions-1"></a>\[CSS-REGIONS-1\]  
Rossen Atanassov; Alan Stearns. [CSS Regions Module Level 1](https://www.w3.org/TR/css-regions-1/). 9 October 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-regions-1&#x2F;](https://www.w3.org/TR/css-regions-1/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                          | Initial | Applies to | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value                                         |
|---------------------|----------------------------------------------------------------|---------|------------|------|-------|----------------|-----------------|--------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-contain①③"></a></span><a href="#propdef-contain">contain</a>&#xA;      </strong> | none \| strict \| content \| \[ size \|\| layout \|\| paint \] | none    | See below  | no   | n/a   | not animatable | per grammar     | the keyword none or one or more of size, layout, paint |

