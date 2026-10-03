Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Containment Module Level 2](https://www.w3.org/TR/2022/WD-css-contain-2-20220917/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Containment Module Level 2

Source snapshot: https://www.w3.org/TR/2022/WD-css-contain-2-20220917/

Snapshot SHA-256: d26c8b725efe73df0f4ec215f44903d5128b3d61e74654f92f0d37be3631bfb9

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 3 source tables are presented as readable Markdown tables or explicit labeled layouts: 3 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Containment Module Level 2

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-contain"></a>

This CSS module describes the [contain](#propdef-contain) property, which indicates that the element’s subtree is independent of the rest of the page. This enables heavy optimizations by user agents when used well.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-contain” in the title, like this: “\[css-contain\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-contain%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

Efficiently rendering a website relies on the user agent being able to detect what parts of the page are being displayed, which parts might affect the currently-displayed section, and what can be ignored.

There are various heuristics that can be used to guess when a given sub-tree is independent of the rest of the page in some manner, but they’re fragile, so innocuous changes to a page may inadvertently make it fail such heuristic tests, causing rendering to fall into a slow code path. There are also many things that would be good to isolate which are difficult or impossible to detect in a heuristic manner.

<a id="ref-for-propdef-contain①"></a>

To alleviate these problems and allow strong, predictable isolation of a subtree from the rest of the page, this specification defines a [contain](#propdef-contain) property.

<a id="ref-for-propdef-content-visibility"></a>

To allow even further optimization of off-screen contents, this spec also defines a [content-visibility](#propdef-content-visibility) property, enabling the user agent to skip an element’s layout and painting <em>entirely</em> when not needed.

### <a id="interaction"></a>1.1.  Module Interactions

This document defines new features not present in earlier specifications. In addition, it aims to replace and supersede [\[CSS-CONTAIN-1\]](#biblio-css-contain-1) once stable.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-contain②"></a>

## <a id="contain-property"></a>2.  Strong Containment: the [contain](#propdef-contain) property

| Field               | Definition                                                                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain"></a>contain                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) strict <a id="ref-for-comb-one①"></a>\| content <a id="ref-for-comb-one②"></a>\| \[ size [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) layout <a id="ref-for-comb-any①"></a>\|\| style <a id="ref-for-comb-any②"></a>\|\| paint \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | See [below](#contain-applies)                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-contain-paint"></a><a id="ref-for-valdef-contain-layout"></a><a id="ref-for-valdef-contain-size"></a><a id="ref-for-valdef-contain-none"></a>the keyword [none](#valdef-contain-none) or one or more of [size](#valdef-contain-size), [layout](#valdef-contain-layout), [paint](#valdef-contain-paint)                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                                                                                              |

User agents are expected to support this property on all media, including non-visual ones.

<a id="ref-for-propdef-contain③"></a>

The [contain](#propdef-contain) property allows an author to indicate that an element and its contents are, as much as possible, <em>independent</em> of the rest of the document tree. This allows user agents to utilize much stronger optimizations when rendering a page using <a id="ref-for-propdef-contain④"></a>contain properly, and allows authors to be confident that their page won’t accidentally fall into a slow code path due to an innocuous change.

<a id="valdef-contain-none"></a>none  
This value indicates that the property has no effect. The element renders as normal, with no containment effects applied.

<a id="valdef-contain-strict"></a>strict  
<a id="ref-for-containment"></a>

This value computes to size layout paint style, and thus turns on all forms of [containment](#containment) for the element.

<a id="valdef-contain-content"></a>content  
<a id="ref-for-size-containment"></a>

<a id="ref-for-containment①"></a>

This value computes to layout paint style, and thus turns on all forms of [containment](#containment) <em>except</em> [size containment](#size-containment) for the element.

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

<a id="valdef-contain-style"></a>style  
<a id="ref-for-style-containment"></a>

This value turns on [style containment](#style-containment) for the element. This ensures that, for properties which can have effects on more than just an element and its descendants, those effects don’t escape the element.

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

<a id="ref-for-containment②"></a>

<a id="ref-for-the-html-element"></a>

<a id="ref-for-the-body-element"></a>

<a id="ref-for-the-body-element①"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-canvas-background"></a>

Additionally, when any [containments](#containment) are active on either the HTML <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> elements, propagation of properties from the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), the viewport, or the [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background), is disabled. Notably, this affects:

- <a id="ref-for-propdef-writing-mode"></a>

  <a id="ref-for-propdef-direction"></a>

  <a id="ref-for-propdef-text-orientation"></a>

  [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) (see [CSS Writing Modes 3 § 8 The Principal Writing Mode](https://www.w3.org/TR/css-writing-modes-3/#principal-flow))

- <a id="ref-for-propdef-overflow"></a>

  [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) and its longhands (see [CSS Overflow 3 § 3.5 Overflow Viewport Propagation](https://www.w3.org/TR/css-overflow-3/#overflow-propagation))

- <a id="ref-for-propdef-background"></a>

  [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) and its longhands (see [CSS Backgrounds 3 § 2.11.2 The Canvas Background and the HTML \<body\> Element](https://www.w3.org/TR/css-backgrounds-3/#body-background))

<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-canvas-background①"></a>

<a id="ref-for-the-html-element①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Propagation to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), the viewport, or the [canvas background](https://www.w3.org/TR/css-backgrounds-3/#canvas-background), of properties set on the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-html-element">html</a></code> element itself is unaffected.

<a id="ref-for-propdef-contain①①"></a>

<a id="ref-for-containment③"></a>

<a id="ref-for-layout-containment①"></a>

<a id="ref-for-propdef-content-visibility①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Several properties beyond [contain](#propdef-contain) can turn on various [containments](#containment) for an element. These do not affect the value of <a id="ref-for-propdef-contain①②"></a>contain; an element can have <a id="ref-for-propdef-contain①③"></a>contain: none but still have [layout containment](#layout-containment) turned on by [content-visibility](#propdef-content-visibility), for example.

## <a id="containment-types"></a>3.  Types of Containment

<a id="ref-for-containment④"></a>

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

    <a id="ref-for-used-value"></a>

    The [used](https://www.w3.org/TR/css-cascade-5/#used-value) [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of the [containment box](#size-containment-box) are determined as if performing a normal layout of the box, except that it is treated as having no content—not even through pseudo elements such as [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), or [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker).

    <a id="ref-for-replaced-element"></a>

    <a id="ref-for-natural-dimensions"></a>

    <a id="ref-for-natural-aspect-ratio"></a>

    [Replaced elements](https://www.w3.org/TR/css-display-3/#replaced-element) must be treated as having an [natural](https://www.w3.org/TR/css-images-3/#natural-dimensions) width and height of 0 and no [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio).

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

    <a id="ref-for-layout-containment②"></a>

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

<a id="ref-for-layout-containment③"></a>

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

    <a id="ref-for-layout-containment④"></a>

    <a id="ref-for-layout-containment-box②"></a>

    <a id="ref-for-fragmented-flow"></a>

    <a id="ref-for-fragmentation"></a>

    If at least one [fragmentation container](https://www.w3.org/TR/css-break-3/#fragmentation-container) of a [fragmentation context](https://www.w3.org/TR/css-break-3/#fragmentation-context) has [layout containment](#layout-containment), or if at least one <a id="ref-for-fragmentation-container①"></a>fragmentation container of a <a id="ref-for-fragmentation-context①"></a>fragmentation context is a descendant of [layout containment box](#layout-containment-box) <strong>and</strong> at least one subsequent <a id="ref-for-fragmentation-container②"></a>fragmentation container of the same <a id="ref-for-fragmentation-context②"></a>fragmentation context is not a descendant of that same element with layout containment, then the first <a id="ref-for-layout-containment-box③"></a>layout containment box which is either a <a id="ref-for-fragmentation-container③"></a>fragmentation container itself or is an ancestor of a <a id="ref-for-fragmentation-container④"></a>fragmentation container must “trap” the remainder of the [fragmented flow](https://www.w3.org/TR/css-break-3/#fragmented-flow): [fragmentation](https://www.w3.org/TR/css-break-3/#fragmentation) must not continue past the <a id="ref-for-layout-containment⑤"></a>layout containment boundary, and the last <a id="ref-for-fragmentation-container⑤"></a>fragmentation container within the first <a id="ref-for-layout-containment⑥"></a>layout containment boundary is treated as if it is the last <a id="ref-for-fragmentation-container⑥"></a>fragmentation container in its <a id="ref-for-fragmentation-context③"></a>fragmentation context.

    <a id="ref-for-fragmentation-container⑦"></a>

    <a id="ref-for-fragmentation-context④"></a>

    <a id="ref-for-fragmented-flow①"></a>

    If subsequent [fragmentation containers](https://www.w3.org/TR/css-break-3/#fragmentation-container) in the [fragmentation context](https://www.w3.org/TR/css-break-3/#fragmentation-context) are only generated when more content remains in the [fragmented flow](https://www.w3.org/TR/css-break-3/#fragmented-flow), then they are not generated. If they would exist regardless, they remain part of the <a id="ref-for-fragmentation-context⑤"></a>fragmentation context, but do not receive any content from the <a id="ref-for-fragmented-flow②"></a>fragmented flow.

    <a id="ref-for-selectordef-nth-fragment"></a>

    <a id="ref-for-layout-containment⑦"></a>

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

    The [layout containment box](#layout-containment-box) creates a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

6.  <a id="ref-for-forced-break"></a>

    <a id="ref-for-layout-containment-box⑦"></a>

    [Forced breaks](https://www.w3.org/TR/css-break-3/#forced-break) are allowed within [layout containment boxes](#layout-containment-box) but do not propagate to the parent as otherwise described in [CSS Fragmentation 3 § 3.1 Breaks Between Boxes: the break-before and break-after properties](https://www.w3.org/TR/css-break-3/#break-between).

    <a id="ref-for-forced-break①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This introduces the previously non-existent possibility that [forced breaks](https://www.w3.org/TR/css-break-3/#forced-break) may occur between a box and its container (See [CSS Fragmentation 3 § 4.1 Possible Break Points](https://www.w3.org/TR/css-break-3/#possible-breaks)).

7.  <a id="ref-for-propdef-vertical-align"></a>

    <a id="ref-for-layout-containment-box⑧"></a>

    For the purpose of the [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property, or any other property whose effects need to relate the position of the [layout containment box](#layout-containment-box)'s baseline to something other than its descendants, the <a id="ref-for-layout-containment-box⑨"></a>containment box is treated as having no baseline.

<a id="ref-for-layout-containment⑧"></a>

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

<a id="ref-for-layout-containment⑨"></a>

Possible optimizations that can be enabled by [layout containment](#layout-containment) include (but are not limited to):

1.  <a id="ref-for-layout-containment-box①⓪"></a>

    When laying out the page, the contents of separate [containment boxes](#layout-containment-box) can be laid out in parallel, as they’re guaranteed not to affect each other.

2.  <a id="ref-for-layout-containment-box①①"></a>

    When laying out the page, if the [containment box](#layout-containment-box) is off-screen or obscured and the layout of the visible parts of the screen do not depend on the size of the <a id="ref-for-layout-containment-box①②"></a>containment box (for example, if the <a id="ref-for-layout-containment-box①③"></a>containment box is near the end of a block container, and you’re viewing the beginning of the block container), the layout of the <a id="ref-for-layout-containment-box①④"></a>containment box' contents can be delayed or done at a lower priority.

    <a id="ref-for-size-containment⑥"></a>

    (When paired with [size containment](#size-containment), this optimization can be applied more liberally.)

### <a id="containment-style"></a>3.3.  Style Containment

Giving an element <a id="style-containment"></a>style containment has the following effects:

1.  <a id="ref-for-propdef-counter-increment"></a>

    <a id="ref-for-propdef-counter-set"></a>

    <a id="ref-for-property-scoped-to-a-sub-tree"></a>

    The [counter-increment](https://www.w3.org/TR/CSS2/generate.html#propdef-counter-increment) and [counter-set](https://www.w3.org/TR/css-lists-3/#propdef-counter-set) properties must be [scoped to the element’s sub-tree](#property-scoped-to-a-sub-tree) and create a new counter.

2.  <a id="ref-for-propdef-content"></a>

    <a id="ref-for-valdef-content-open-quote"></a>

    <a id="ref-for-valdef-content-close-quote"></a>

    <a id="ref-for-valdef-content-no-open-quote"></a>

    <a id="ref-for-valdef-content-no-close-quote"></a>

    <a id="ref-for-property-scoped-to-a-sub-tree①"></a>

    The effects of the [content](https://www.w3.org/TR/CSS2/generate.html#propdef-content) property’s [open-quote](https://www.w3.org/TR/css-content-3/#valdef-content-open-quote), [close-quote](https://www.w3.org/TR/css-content-3/#valdef-content-close-quote), [no-open-quote](https://www.w3.org/TR/css-content-3/#valdef-content-no-open-quote) and [no-close-quote](https://www.w3.org/TR/css-content-3/#valdef-content-no-close-quote) must be [scoped to the element’s sub-tree](#property-scoped-to-a-sub-tree).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This implies that the depth of quote nesting in the subtree is unchanged and starts at the value that its context normally implies, but that changes to the depth of quote nesting by these values inside the subtree do not affect the depth of quote nesting outside the subtree.

<a id="ref-for-style-containment①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[CSS-REGIONS-1\]](#biblio-css-regions-1) has normative requirements on how [style containment](#style-containment) affects regions.

A <a id="property-scoped"></a>scoped property has its effects scoped to a particular element or subtree.

- If <a id="property-scoped-to-an-element"></a>scoped to an element, it must act as if the scoping element was the root of the document for the purpose of evaluating the property’s effects: any uses of the property outside the scoping element must have no effect on the uses of the property on or in the scoping element, and vice versa.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: “Scoping to an element” is currently unused. It is defined as an extension point for future specifications to use.

- If <a id="property-scoped-to-a-sub-tree"></a>scoped to a sub-tree, it’s the same, except the scoping element itself is counted as "outside" the tree, like the rest of the document, and the effects of the property on that element are unaffected by scoping. When considering the effects of the scoped property on elements <em>inside</em> the subtree, the element at the base of the subtree is treated as if it was the root of the document.

<a id="ref-for-propdef-counter-increment①"></a>

<a id="ref-for-propdef-content①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6932a400"></a> As [counter-increment](https://www.w3.org/TR/CSS2/generate.html#propdef-counter-increment) is scoped to an element’s subtree, the first use of it within the subtree acts as if the named counter were set to 0 at the scoping element, regardless of whether the counter had been used outside the scoping element. Any increments made within the subtree have no effect on counters of the same name outside the scoping element. However, the counter() and counters() value of the [content](https://www.w3.org/TR/CSS2/generate.html#propdef-content) property is not itself scoped, and can refer to counters established outside of the subtree. Therefore, the following code results in “`1 1.2`” being displayed:
>
> ```text
> <div></div>
> ```
>
> ```text
> div {
>   contain: style;
>   counter-increment: n;
> }
> div::before, div::after {
>   content: counters(n, '.') " ";
> }
> div::after {
>   counter-increment: n 2;
> }
> ```
#### <a id="containment-style-opt"></a>3.3.1.  Possible Style-Containment Optimizations

<em>This section is non-normative.</em>

<a id="ref-for-style-containment②"></a>

Possible optimizations that can be enabled by [style containment](#style-containment) include (but are not limited to):

1.  <a id="ref-for-style-containment③"></a>

    Whenever a property is changed on a descendant of an element with [style containment](#style-containment), calculating what part of the DOM tree is "dirtied" and might need to have its style recalculated can stop at the element with <a id="ref-for-style-containment④"></a>style containment.

### <a id="containment-paint"></a>3.4.  Paint Containment

<a id="ref-for-principal-box⑧"></a>

Giving an element <a id="paint-containment"></a>paint containment makes its [principal box](https://www.w3.org/TR/css-display-3/#principal-box) a <a id="paint-containment-box"></a>paint containment box and has the following effects:

1.  <a id="ref-for-ink-overflow①"></a>

    <a id="ref-for-scrollable-overflow"></a>

    <a id="ref-for-overflow-clip-edge"></a>

    <a id="ref-for-paint-containment-box①"></a>

    <a id="ref-for-propdef-overflow②"></a>

    <a id="ref-for-propdef-resize"></a>

    <a id="ref-for-propdef-text-overflow"></a>

    The contents of the element including any [ink](https://www.w3.org/TR/css-overflow-3/#ink-overflow) or [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) must be clipped to the [overflow clip edge](https://www.w3.org/TR/css-overflow-3/#overflow-clip-edge) of the [paint containment box](#paint-containment-box), taking \[\[css-backgrounds-3#corner clipping\|corner clipping\]\] into account. This does not include the creation of any mechanism to access or indicate the presence of the clipped content; nor does it inhibit the creation of any such mechanism through other properties, such as [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [resize](https://www.w3.org/TR/css-ui-3/#propdef-resize), or [text-overflow](https://www.w3.org/TR/css-ui-3/#propdef-text-overflow).

    <a id="ref-for-propdef-overflow-clip-margin"></a>

    <a id="ref-for-paint-containment①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This clipping shape respects [overflow-clip-margin](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-clip-margin), allowing an element with [paint containment](#paint-containment) to still slightly overflow its normal bounds.

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

    The [paint containment box](#paint-containment-box) creates a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

4.  <a id="ref-for-paint-containment-box④"></a>

    <a id="ref-for-establish-an-independent-formatting-context①"></a>

    The [paint containment box](#paint-containment-box) [establishes an independent formatting context](https://www.w3.org/TR/css-display-3/#establish-an-independent-formatting-context).

<a id="ref-for-paint-containment②"></a>

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

#### <a id="containment-paint-opt"></a>3.4.1.  Possible Paint-Containment Optimizations

<em>This section is non-normative.</em>

<a id="ref-for-paint-containment③"></a>

Possible optimizations that can be enabled by [paint containment](#paint-containment) include (but are not limited to):

1.  <a id="ref-for-paint-containment-box⑤"></a>

    If the [containment box](#paint-containment-box) is off-screen or obscured, the UA can usually skip trying to paint its contents, as they’re guaranteed to be off-screen/obscured as well.

    <a id="ref-for-funcdef-filter-blur"></a>

    <a id="ref-for-paint-containment④"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Some paint effects such as the [blur()](https://www.w3.org/TR/filter-effects-1/#funcdef-filter-blur) filter from [\[FILTER-EFFECTS-1\]](#biblio-filter-effects-1) have non local effects. The user agent needs to keep track of these, as it may need to repaint parts of an element with such a filter when its descendents change, even if they have [paint containment](#paint-containment) and could otherwise be skipped.

2.  <a id="ref-for-propdef-overflow③"></a>

    <a id="ref-for-propdef-resize①"></a>

    <a id="ref-for-propdef-text-overflow①"></a>

    Unless the clipped content is made accessible via a separate mechanism such as the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [resize](https://www.w3.org/TR/css-ui-3/#propdef-resize), or [text-overflow](https://www.w3.org/TR/css-ui-3/#propdef-text-overflow) properties, the UA can reserve "canvas" space for the box exactly the box’s size. (In similar, scrollable, situations, like <a id="ref-for-propdef-overflow④"></a>overflow: hidden, it’s possible to scroll to the currently-clipped content, so UAs often predictively overpaint somewhat so there’s something to see as soon as the scroll happens, rather than a frame later.)

3.  Because they are guaranteed to be stacking contexts, scrolling elements can be painted into a single GPU layer.

<a id="ref-for-propdef-content-visibility②"></a>

## <a id="content-visibility"></a>4. Suppressing An Element’s Contents Entirely: the [content-visibility](#propdef-content-visibility) property

| Field               | Definition                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-content-visibility"></a>content-visibility                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one③"></a>visible [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto <a id="ref-for-comb-one④"></a>\| hidden |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | visible                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-layout-containment①⓪"></a>elements for which [layout containment](#layout-containment) can apply                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                  |

<a id="ref-for-propdef-content-visibility③"></a>

<a id="ref-for-containment⑤"></a>

The [content-visibility](#propdef-content-visibility) property controls whether or not an element renders its contents at all, along with forcing a strong set of [containments](#containment), allowing user agents to potentially omit large swathes of layout and rendering work until it becomes needed. It has the following values:

<a id="valdef-content-visibility-visible"></a>visible  
No effect. The element’s contents are laid out and rendered as normal.

<a id="valdef-content-visibility-hidden"></a>hidden  
<a id="ref-for-skips-its-contents"></a>

The element [skips its contents](#skips-its-contents).

<a id="ref-for-skips-its-contents①"></a>

The [skipped contents](#skips-its-contents) <em>must not</em> be accessible to user-agent features, such as find-in-page, tab-order navigation, etc., nor be selectable or focusable.

<a id="ref-for-propdef-display⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is similar to giving the contents [display: none](https://www.w3.org/TR/css-display-3/#propdef-display).

<a id="valdef-content-visibility-auto"></a>auto  
<a id="ref-for-layout-containment①①"></a>

<a id="ref-for-style-containment⑤"></a>

<a id="ref-for-paint-containment⑤"></a>

Turns on [layout containment](#layout-containment), [style containment](#style-containment), and [paint containment](#paint-containment) for the element.

<a id="ref-for-relevant-to-the-user"></a>

<a id="ref-for-skips-its-contents②"></a>

If the element is not [relevant to the user](#relevant-to-the-user), it also [skips its contents](#skips-its-contents).

<a id="ref-for-valdef-content-visibility-hidden"></a>

<a id="ref-for-skips-its-contents③"></a>

Unlike [hidden](#valdef-content-visibility-hidden), the [skipped contents](#skips-its-contents) <em>must</em> still be available as normal to user-agent features such as find-in-page, tab order navigation, etc., and must be focusable and selectable as normal.

<a id="ref-for-used-value①"></a>

<a id="ref-for-propdef-contain①④"></a>

<a id="ref-for-layout-containment①②"></a>

<a id="ref-for-style-containment⑥"></a>

<a id="ref-for-paint-containment⑥"></a>

<a id="ref-for-size-containment⑦"></a>

<a id="ref-for-flat-tree"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-propdef-visibility"></a>

<a id="ref-for-propdef-pointer-events"></a>

When an element <a id="skips-its-contents"></a>skips its contents, the user agent must change the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the [contain](#propdef-contain) property so as to turn on [layout containment](#layout-containment), [style containment](#style-containment), [paint containment](#paint-containment), and [size containment](#size-containment). Further, its <a id="element-contents"></a>contents (the [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) descendants of the element, including both text and elements, or the replaced content of a [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element)) are not painted (as if they had [visibility: hidden](https://www.w3.org/TR/css-display-3/#propdef-visibility)) and do not respond to hit-testing (as if they had [pointer-events: none](https://drafts.csswg.org/css-ui-4/#propdef-pointer-events)), and to a large extent do not update their styles at all unless explicitly requested by script (see [§ 4.4 Restrictions and Clarifications](#cv-notes) for details).

<a id="ref-for-skips-its-contents④"></a>

<a id="ref-for-containment⑥"></a>

The user agent <em>should</em> additionally avoid as much layout/rendering work as possible for [skipped contents](#skips-its-contents); the combination of heavy [containment](#containment) and making the contents invisible and untouchable enables heavy optimizations. If rendering work is done at some point, the user-agent should retain the previously computed layout state if possible, to allow the <a id="ref-for-skips-its-contents⑤"></a>skipped contents to be displayed quickly at a later moment.

<a id="ref-for-propdef-content-visibility④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> If an element has a value other than [content-visibility: visible](#propdef-content-visibility), then the following properties hold:
>
> - <a id="ref-for-layout-containment①③"></a>
>
>   [layout containment](#layout-containment) ensures that the user-agent is able to omit layout work in skipped subtrees, since the results of such layouts will not affect elements outside of the container element.
>
> - <a id="ref-for-style-containment⑦"></a>
>
>   [style containment](#style-containment) ensures that counters do not have to be processed in skipped subtrees, since they do not affect counters outside of the container element.
>
> - <a id="ref-for-paint-containment⑦"></a>
>
>   <a id="ref-for-propdef-content-visibility⑤"></a>
>
>   [paint containment](#paint-containment) ensures that ink overflow of painted contents is clipped; this, in turn, means that user-agent can reliably determine when the visible portion of the element approaches the viewport (and, for [content-visibility: auto](#propdef-content-visibility), start painting it).
>
> - <a id="ref-for-size-containment⑧"></a>
>
>   [size containment](#size-containment) ensures that the user-agent is able to omit layout in skipped subtrees, since the results of such layouts will not affect the container element’s size.
>
> <a id="ref-for-propdef-content-visibility⑥"></a>
>
> <a id="ref-for-layout-containment①④"></a>
>
> <a id="ref-for-style-containment⑧"></a>
>
> <a id="ref-for-paint-containment⑧"></a>
>
> <a id="ref-for-skips-its-contents⑥"></a>
>
> <a id="ref-for-containment⑦"></a>
>
> Note that in the [content-visibility: auto](#propdef-content-visibility) case, [layout containment](#layout-containment), [style containment](#style-containment), and [paint containment](#paint-containment) persist even if the element is not [skipped](#skips-its-contents). This is done to prevent layout changes that would be incurred by [containment](#containment) changes as a result of an element entering and exiting the <a id="ref-for-skips-its-contents⑦"></a>skipped state.

An element is <a id="relevant-to-the-user"></a>relevant to the user if <strong>any</strong> of the following conditions are true:

- <a id="ref-for-paint-containment-box⑥"></a>

  <a id="ref-for-overflow-clip-edge①"></a>

  The element is "on-screen": its [paint containment box](#paint-containment-box)'s [overflow clip edge](https://www.w3.org/TR/css-overflow-3/#overflow-clip-edge) intersects with the viewport, or a user-agent defined margin around the viewport.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This margin is meant to allow the user agent to begin preparing for an element to be in the viewport soon. A margin of 50% is suggested as a reasonable default.

- <a id="ref-for-element-contents"></a>

  Either the element or its [contents](#element-contents) are focused, as described in the [focus](https://html.spec.whatwg.org/multipage/interaction.html#focus) section of the HTML spec.

- <a id="ref-for-element-contents①"></a>

  Either the element or its [contents](#element-contents) are selected, as described in [CSS Pseudo-Elements 4 § 3 Highlight Pseudo-elements](https://www.w3.org/TR/css-pseudo-4/#highlight-pseudos).

- <a id="ref-for-element-contents②"></a>

  <a id="ref-for-top-layer"></a>

  Either the element or its [contents](#element-contents) are placed in the [top layer](https://fullscreen.spec.whatwg.org/#top-layer).

<a id="ref-for-propdef-content-visibility⑦"></a>

### <a id="using-cv-hidden"></a>4.1.  Using [content-visibility: hidden](#propdef-content-visibility)

<em>This section is non-normative.</em>

<a id="ref-for-propdef-content-visibility⑧"></a>

[content-visibility: hidden](#propdef-content-visibility) lays powerful restrictions onto an element, and so should be used with caution. It also enables some very useful scenarios, often improving on existing techniques, a few of which are outlined here.

1.  <a id="ref-for-propdef-position"></a>

    <a id="ref-for-dom-element-getboundingclientrect"></a>

    If a page needs to take some measurements of elements or text which aren’t themselves going to be rendered, commonly this is done by positioning the stuff-to-be-measured off-screen, using something like [position: absolute; left: -100000px;](https://www.w3.org/TR/css-position-3/#propdef-position), then calling an API like <code><a href="https://www.w3.org/TR/cssom-view/#dom-element-getboundingclientrect">getBoundingClientRect()</a></code>.

    <a id="ref-for-propdef-left"></a>

    Unfortunately, even though the page never intends to display this content, the user agent will still have to do full styling, layout, and rendering for the content, <em>just in case</em> it affects what’s shown on screen. The author also can’t, without further work, guarantee that the content <em>won’t</em> accidentally show up on-screen; even a very negative [left](https://www.w3.org/TR/css-position-3/#propdef-left) value (like above) might not be enough, depending on the content.

    <a id="ref-for-propdef-content-visibility⑨"></a>

    <a id="ref-for-skips-its-contents⑧"></a>

    Wrapping this content in a [content-visibility: hidden](#propdef-content-visibility) container solves all of these problems. If the wrapper has no border, background, etc, then it and its [skipped contents](#skips-its-contents) are guaranteed to never render anything to the screen, no matter how big they get. Because the contents are <a id="ref-for-skips-its-contents⑨"></a>skipped, the user agent can also avoid styling or laying them out until absolutely necessary, when script finally asks for it.

2.  A "single-page app" often consists of several independent panes or "views", of which only one is displayed at a time.

    If the author wants to avoid paying styling/layout/rendering/etc cost for the inactive views, they can remove them from the document entirely, or at minimum apply display:none to them. Unfortunately, this means that when the view <em>does</em> need to be displayed, all of the styling/layout/rendering/etc work needs to be done all at once, potentially causing a noticeable delay before the view actually shows up.

    Alternately, the view can just be positioned off-screen. This means it’ll be immediately ready when it’s time to be used, but it incurs the cost of styling/layout/rendering all the time, which might be significant, especially if there are a number of inactive views. The inactive views also might still show up to accessibility tooling, confusing users of screen-readers, people using Ctrl-F to find-in-page, etc.

    <a id="ref-for-propdef-content-visibility①⓪"></a>

    <a id="ref-for-skips-its-contents①⓪"></a>

    [content-visibility: hidden](#propdef-content-visibility) improves on both of these options. Because the contents are [skipped](#skips-its-contents), the user agent isn’t spending time on them when they’re not active. They’re also not visible to screen readers, find-in-page, and other tools. And because user agents <em>should</em> preserve previous styling/layout work if possible, if the view was displayed before, re-rendering it might be very fast.

3.  <a id="ref-for-propdef-visibility①"></a>

    If an author wants to make an element "invisible", but still show up in the page for layout purposes, one option is [visibility: hidden](https://www.w3.org/TR/css-display-3/#propdef-visibility). However, descendants of a <a id="ref-for-propdef-visibility②"></a>visibility: hidden element can set <a id="ref-for-propdef-visibility③"></a>visibility: visible and start showing up again, which isn’t always intuitive or expected.

    <a id="ref-for-propdef-content-visibility①①"></a>

    [content-visibility: hidden](#propdef-content-visibility) performs a very similar purpose, but descendants can’t turn it "off" and start displaying; they stay "hidden" until the ancestor turns it off.

    <a id="ref-for-propdef-content-visibility①②"></a>

    <a id="ref-for-containment⑧"></a>

    <a id="ref-for-propdef-visibility④"></a>

    Because [content-visibility: hidden](#propdef-content-visibility) also applies many [containment](#containment) values to the container, it’s not always quite as usable as [visibility: hidden](https://www.w3.org/TR/css-display-3/#propdef-visibility) would be, but when its restrictions are acceptable, it can be a more reliable, more consistent way to hide an element’s contents.

<a id="ref-for-propdef-content-visibility①③"></a>

### <a id="using-cv-auto"></a>4.2.  Using [content-visibility: auto](#propdef-content-visibility)

<em>This section is non-normative.</em>

<a id="ref-for-propdef-content-visibility①④"></a>

<a id="ref-for-valdef-content-visibility-hidden①"></a>

<a id="ref-for-propdef-display⑦"></a>

<a id="ref-for-relevant-to-the-user①"></a>

<a id="ref-for-skips-its-contents①①"></a>

[content-visibility: auto](#propdef-content-visibility) is a more complex value than [hidden](#valdef-content-visibility-hidden); rather than being similar to [display: none](https://www.w3.org/TR/css-display-3/#propdef-display), it adaptively hides/displays an element’s contents as they become [relevant to the user](#relevant-to-the-user). It also doesn’t hide its [skipped contents](#skips-its-contents) from the user agent, so screen readers, find-in-page, and other tools can still interact with it.

<a id="ref-for-containment⑨"></a>

<a id="ref-for-containment①⓪"></a>

<a id="ref-for-propdef-content-visibility①⑤"></a>

It is best to think of it as <em>an upgrade to <a href="#containment">containment</a></em>: if an author has a large amount of content to display that will often be off-screen (such as a long scrollable list), and that content is okay with heavy [containment](#containment), they should consider using [content-visibility: auto](#propdef-content-visibility) to apply all of the <a id="ref-for-containment①①"></a>containments at once. This also strongly hints to the user agent that it’s acceptable to skip work on the contents (possibly causing a small delay when they do come on-screen) because it’s more important to have a large amount of content in the document and most of it won’t be seen anyway.

<a id="ref-for-propdef-content-visibility①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [content-visibility: auto](#propdef-content-visibility) can thus be used <em>instead of</em> complicated "virtual list" techniques, at least in many cases.

<a id="ref-for-propdef-content-visibility①⑦"></a>

<a id="ref-for-skips-its-contents①②"></a>

<a id="ref-for-relevant-to-the-user②"></a>

Because [content-visibility: auto](#propdef-content-visibility) only causes the element to [skip](#skips-its-contents) its contents when none of it is [relevant to the user](#relevant-to-the-user), it’s best to use at a reasonably fine granularity.

<a id="ref-for-propdef-content-visibility①⑧"></a>

<a id="ref-for-skips-its-contents①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40f1ea8c"></a> For example, on Twitter, applying [content-visibility: auto](#propdef-content-visibility) to the entire timeline wouldn’t accomplish much—it’s always on-screen, and so it will never [skip its contents](#skips-its-contents).
>
> <a id="ref-for-propdef-content-visibility①⑨"></a>
>
> <a id="ref-for-skips-its-contents①④"></a>
>
> Instead, [content-visibility](#propdef-content-visibility) should be applied to <em>individual tweets</em>, allowing each of them to be [skipped](#skips-its-contents) as they go off-screen.

<a id="ref-for-propdef-content-visibility②⓪"></a>

<a id="ref-for-size-containment⑨"></a>

<a id="ref-for-skips-its-contents①⑤"></a>

Because [content-visibility: auto](#propdef-content-visibility) imposes [size containment](#size-containment) when the element [skips its contents](#skips-its-contents), if the element depends on its contents to determine its size the layout of the page (or at least, the scrollbar position) can "jump around" as elements go off-screen and start <a id="ref-for-skips-its-contents①⑥"></a>skipping.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6da2c0d3"></a> This can be fixed in a number of ways:
>
> - making the element fixed-size
>
> - carefully arranging a layout such as Grid to size the element without depending on its contents
>
> - <a id="ref-for-propdef-contain-intrinsic-size"></a>
>
>   using [contain-intrinsic-size](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size) to set an <em>estimate</em> of the element’s size
>
> - <a id="ref-for-propdef-contain-intrinsic-size①"></a>
>
>   <a id="ref-for-skips-its-contents①⑦"></a>
>
>   using [contain-intrinsic-size: auto](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size) to automatically "snapshot" the <em>exact</em> size of the element from the last time it was rendered, before it was [skipped](#skips-its-contents) (along with providing an estimate of the size to be used before it’s rendered and can have its size snapshotted)
>
> <a id="ref-for-propdef-contain-intrinsic-size②"></a>
>
> <a id="ref-for-skips-its-contents①⑧"></a>
>
> For example, on Twitter, the average tweet is approximately 200px tall, so [contain-intrinsic-size: auto 500px 200px](https://www.w3.org/TR/css-sizing-4/#propdef-contain-intrinsic-size) will ensure that the scrollbar thumb is in approximately the correct size and position even when preceding or following tweets are [skipped](#skips-its-contents), while still allowing the tweets to size according to their contents when they’re on-screen. As long as the tweets have all been viewed at least once (and haven’t changed size while they were <a id="ref-for-skips-its-contents①⑨"></a>skipped), their sizes will be <em>exactly</em> correct while they’re <a id="ref-for-skips-its-contents②⓪"></a>skipped, so the scrollbar thumb will be as well; only freshly-loaded tweets (such as those loaded at the top of the timeline while you are scrolling further down) will be forced to rely on the 200px height estimate.

<a id="ref-for-propdef-content-visibility②①"></a>

<a id="ref-for-eventdef-element-contentvisibilityautostatechanged"></a>

### <a id="content-visibility-auto-state-changed"></a>4.3. Detecting [content-visibility: auto](#propdef-content-visibility) state changes: the [contentvisibilityautostatechanged](#eventdef-element-contentvisibilityautostatechanged) event

<a id="ref-for-propdef-content-visibility②②"></a>

<a id="ref-for-relevant-to-the-user③"></a>

The <a id="eventdef-element-contentvisibilityautostatechanged"></a>`contentvisibilityautostatechanged` event is fired on an element with [content-visibility: auto](#propdef-content-visibility) style when the rendering state changes and the element either becomes or stops being [relevant to the user](#relevant-to-the-user).

This event is dispatched by posting a task at the time when the state change occurs.

<a id="ref-for-Exposed"></a>

<a id="contentvisibilityautostatechangedevent"></a>

<a id="ref-for-event"></a>

<a id="dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent"></a>

<a id="ref-for-idl-DOMString"></a>

<a id="dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent-type-eventinitdict-type"></a>

<a id="ref-for-dictdef-contentvisibilityautostatechangedeventinit"></a>

<a id="dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent-type-eventinitdict-eventinitdict"></a>

<a id="ref-for-idl-boolean"></a>

<a id="ref-for-dom-contentvisibilityautostatechangedevent-skipped"></a>

<a id="dictdef-contentvisibilityautostatechangedeventinit"></a>

<a id="ref-for-dictdef-eventinit"></a>

<a id="ref-for-idl-boolean①"></a>

<a id="ref-for-dom-contentvisibilityautostatechangedeventinit-skipped"></a>

```text
[Exposed=Window]
interface ContentVisibilityAutoStateChangedEvent : Event {
  constructor(DOMString type, optional ContentVisibilityAutoStateChangedEventInit eventInitDict = {});
  readonly attribute boolean skipped;
};
dictionary ContentVisibilityAutoStateChangedEventInit : EventInit {
  boolean skipped = false;
};
```
Description of ContentVisibilityAutoStateChangedEvent attributes:

<a id="ref-for-idl-boolean②"></a>

<a id="dom-contentvisibilityautostatechangedevent-skipped"></a>`skipped`, of type [boolean](https://webidl.spec.whatwg.org/#idl-boolean), readonly

<a id="ref-for-skips-its-contents②①"></a>

Set to true if target changed state to [skip its contents](#skips-its-contents), and false otherwise.

Description of ContentVisibilityAutoStateChangedEventInit members:

<a id="ref-for-idl-boolean③"></a>

<a id="dom-contentvisibilityautostatechangedeventinit-skipped"></a>`skipped`, of type [boolean](https://webidl.spec.whatwg.org/#idl-boolean), defaulting to `false`

<a id="ref-for-dom-contentvisibilityautostatechangedevent-skipped①"></a>

See the description of the <code><a href="#dom-contentvisibilityautostatechangedevent-skipped">skipped</a></code> attribute.

<a id="ref-for-propdef-content-visibility②③"></a>

<a id="ref-for-skips-its-contents②②"></a>

Note that elements in [content-visibility: auto](#propdef-content-visibility) subtrees remain semantically relevant even for elements that [skip its contents](#skips-its-contents). This means that it is inappropriate to use this signal to indefinitely skip DOM updates in the subtree that is skipped. Instead, it should be used to deprioritize updates, but ensure that the content remains semantically relevant and reasonably up-to-date. This is particularly important for assistive technologies which consume this content even when the ancestor is set to <a id="ref-for-skips-its-contents②③"></a>skip its contents.

### <a id="cv-notes"></a>4.4. Restrictions and Clarifications

1.  <a id="ref-for-intersectionobserver"></a>

    <a id="ref-for-skips-its-contents②④"></a>

    <a id="ref-for-intersectionobserver-intersection-root"></a>

    From the perspective of an <code><a href="https://www.w3.org/TR/intersection-observer/#intersectionobserver">IntersectionObserver</a></code>, the [skipped contents](#skips-its-contents) of an element are never intersecting the [intersection root](https://www.w3.org/TR/intersection-observer/#intersectionobserver-intersection-root). This is true even if both the root and the target elements are in the <a id="ref-for-skips-its-contents②⑤"></a>skipped contents.

2.  <a id="ref-for-resizeobserver"></a>

    <a id="ref-for-skips-its-contents②⑥"></a>

    From the perspective of a <code><a href="https://www.w3.org/TR/resize-observer-1/#resizeobserver">ResizeObserver</a></code>, the [skipped contents](#skips-its-contents) of an element never change their size. If these elements become non-skipped later, the resize observation will be delivered if the new size differs from the last size used to notify the resize observer.

3.  <a id="ref-for-skips-its-contents②⑦"></a>

    <a id="termref-for-update-the-rendering"></a>

    If an element starts or stops [skipping its contents](#skips-its-contents), this change happens after the requestAnimationFrame callbacks of the frame that renders the effects of the change have run. Specifically, such changes will take effect between steps 13 and 14 of [Update the Rendering](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering) step of the Processing Model (between “run the animation frame callbacks” and “run the update intersection observations steps”).

    <a id="ref-for-skips-its-contents②⑧"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Determining the viewport intersection of the element can be done with an internal version of an IntersectionObserver. However, since the observations from this are dispatched at step 14 of Update the Rendering, any changes to the [skipped](#skips-its-contents) (and thus painted) state will not be visible to the user until the next frame’s processing. For this reason, updating the <a id="ref-for-skips-its-contents②⑨"></a>skipped state, including containment adjustments, is deferred to that frame as well. This ensures that script accessing, for example, the containment value of the element between these two events (internal intersection observation and <a id="ref-for-skips-its-contents③⓪"></a>skipped state update) will retrieve values consistent with current painted state and not cause any forced layouts.

4.  <a id="ref-for-propdef-content-visibility②④"></a>

    The initial determination of visibility for [content-visibility: auto](#propdef-content-visibility) must happen in the same frame that determined an existence of a new <a id="ref-for-propdef-content-visibility②⑤"></a>content-visibility: auto element.

    <a id="ref-for-propdef-content-visibility②⑥"></a>

    <a id="ref-for-skips-its-contents③①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > When an element first gains [content-visibility: auto](#propdef-content-visibility), it may or may not be positioned on screen. The determination of this state and thus determination of whether this element is [skipped](#skips-its-contents) must happen in the same frame. If it does not, then there is a possibility of producing blank content in the element’s place since visibility check and <a id="ref-for-skips-its-contents③②"></a>skipped state update would be deferred to the next frame.

5.  <a id="ref-for-dom-element-scrollintoview"></a>

    <a id="ref-for-propdef-content-visibility②⑦"></a>

    <a id="ref-for-skips-its-contents③③"></a>

    <a id="ref-for-size-containment①⓪"></a>

    For the purposes of scrolling operations, such as <code><a href="https://drafts.csswg.org/cssom-view-1/#dom-element-scrollintoview">scrollIntoView()</a></code>, an element with [content-visibility: auto](#propdef-content-visibility) that is [skipping its contents](#skips-its-contents) has its size and location determined <em>with</em> [size containment](#size-containment) still active.

    <a id="ref-for-skips-its-contents③④"></a>

    <a id="ref-for-size-containment①①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Once it’s scrolled into view, the element will no longer [skip its contents](#skips-its-contents), and so might not have [size containment](#size-containment); if this changes the element’s size, it might not align in the viewport exactly as requested.

    <a id="ref-for-skips-its-contents③⑤"></a>

    <a id="ref-for-propdef-content-visibility②⑧"></a>

    If an element is not available to user-agent features (for example, if it is [skipped](#skips-its-contents) due to a [content-visibility: hidden](#propdef-content-visibility) ancestor), then scrolling operations must not scroll to it at all, as if it did not have a layout box.

6.  <a id="ref-for-propdef-content-visibility②⑨"></a>

    <a id="ref-for-skips-its-contents③⑥"></a>

    <a id="ref-for-relevant-to-the-user④"></a>

    If an element with [content-visibility: auto](#propdef-content-visibility) that is [skipping its contents](#skips-its-contents) is focused (or its contents are), it becomes [relevant to the user](#relevant-to-the-user) (and thus stops <a id="ref-for-skips-its-contents③⑦"></a>skipping its contents) <em>before</em> it is scrolled into view due to the focusing.

    <a id="ref-for-dom-window-focus"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Thus, unlike the previous point, the element <em>will</em> be correctly sized and aligned in the viewport. This is consistent with the order of the steps for the <code><a href="https://html.spec.whatwg.org/multipage/interaction.html#dom-window-focus">focus()</a></code> method.

7.  <a id="ref-for-the-iframe-element"></a>

    <a id="ref-for-skips-its-contents③⑧"></a>

    <a id="ref-for-update-the-rendering"></a>

    If an <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> [skips its contents](#skips-its-contents) or is part of an element’s <a id="ref-for-skips-its-contents③⑨"></a>skipped contents, the user agent should entirely skip the [Update The Rendering](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering) step in the iframe’s event loop, if possible.

    <a id="ref-for-the-iframe-element①"></a>

    <a id="ref-for-skips-its-contents④⓪"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: At the moment the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> starts being [skipped](#skips-its-contents), it needs to run that step at least once to remove the painted output.

8.  <a id="ref-for-skips-its-contents④①"></a>

    <a id="ref-for-dom-innertext"></a>

    [Skipped contents](#skips-its-contents) do not contribute to the result of <code><a href="https://html.spec.whatwg.org/multipage/dom.html#dom-innertext">innerText</a></code>.

9.  <a id="ref-for-skips-its-contents④②"></a>

    While an element is [skipped](#skips-its-contents), CSS transitions and animations on the element do not update:

    - New animations are not created even if newly-applied style would start one.

    - Existing animations do not advance in their timeline.

    - Running animations on the element do not end.

    <a id="ref-for-skips-its-contents④③"></a>

    <a id="ref-for-style-change-event"></a>

    If script queries the style of a [skipped](#skips-its-contents) element (causing a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event)) such that knowing the state of animations or transitions is required to return correct information, animation and transitions are sampled according to the styles at the time of that <a id="ref-for-style-change-event①"></a>style change event:

    [CSS Animations 2 § 4 Animation Events](https://drafts.csswg.org/css-animations-2/#events) and [CSS Transitions 2 § 5 Transition Events](https://drafts.csswg.org/css-transitions-2/#transition-events) defines what objects are created and what events are fired, with what data, when an animation or transition is updated,.

    <a id="ref-for-skips-its-contents④④"></a>

    When an element stops being [skipped](#skips-its-contents), animations and transitions are sampled and then resume advancing on their timelines as normal from that point.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Overall, this is similar to the behavior of transitions/animations when a background tab is brought back to the foreground, allowing user agents to skip as much unnecessary animation work as possible without overly disrupting the animations when they become relevant again.

10. <a id="ref-for-skips-its-contents④⑤"></a>

    <a id="ref-for-style-change-event②"></a>

    While an element is [skipped](#skips-its-contents), it must not start any transitions, even if a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event) affects its computed styles.

    <a id="ref-for-skips-its-contents④⑥"></a>

    <a id="ref-for-style-change-event③"></a>

    When an element stops being [skipped](#skips-its-contents), it must not start any transitions as a result of the [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event) associated with it no longer being <a id="ref-for-skips-its-contents④⑦"></a>skipped.

    <a id="ref-for-valdef-display-none"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is similar to an element switching from display:none to a non-[none](https://www.w3.org/TR/css-display-3/#valdef-display-none) value—even though the styles are <em>technically</em> changing in that case (from their initial values to their "proper" values from the cascade), no transitions are started.

11. <a id="ref-for-propdef-content-visibility③⓪"></a>

    <a id="ref-for-top-layer①"></a>

    <a id="ref-for-propdef-display⑧"></a>

    If an element has an ancestor with [content-visibility: hidden](#propdef-content-visibility), and it is placed in the [top layer](https://fullscreen.spec.whatwg.org/#top-layer), it does not generate any boxes, as if it were [display: none](https://www.w3.org/TR/css-display-3/#propdef-display).

    <a id="ref-for-skips-its-contents④⑧"></a>

    <a id="ref-for-propdef-content-visibility③①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: An element [skipped](#skips-its-contents) for other reasons, such as having a [content-visibility: auto](#propdef-content-visibility) ancestor, will still generate boxes as normal, and might thus become un-<a id="ref-for-skips-its-contents④⑨"></a>skipped.

### <a id="cv-a11y"></a>4.5. Accessibility Implications

<a id="ref-for-skips-its-contents⑤⓪"></a>

<a id="ref-for-propdef-content-visibility③②"></a>

<a id="ref-for-propdef-display⑨"></a>

If a user agent exposes some form of "accessibility tree", akin to the DOM tree but specialized for accessibility use-cases such as screen-readers (thus providing the positions/etc of elements relevant to accessibility APIs, such as focusable elements), then the [skipped contents](#skips-its-contents) of [content-visibility: hidden](#propdef-content-visibility) elements must similarly be "skipped" (omitted) in the accessibility tree (similar to how [display: none](https://www.w3.org/TR/css-display-3/#propdef-display) elements are omitted in all views of the document).

<a id="ref-for-skips-its-contents⑤①"></a>

<a id="ref-for-propdef-content-visibility③③"></a>

[Skipped contents](#skips-its-contents) of [content-visibility: auto](#propdef-content-visibility) elements must not expose the fact that a user is interacting with the page via the accessibility tree, rather than via rendering visually to the screen. In particular, if a user agent uses <a id="ref-for-propdef-content-visibility③④"></a>content-visibility: auto to avoid doing layout and painting work on off-screen content for displaying to the screen, it must similarly avoid doing that work on off-screen content for representing in the accessibility tree. If this is not possible (for example, if the user agent’s representation of a focusable element in the accessibility tree requires knowledge of its exact position, and thus requires a full layout to be done on it and surrounding contents), then the user agent <em>must</em> omit the <a id="ref-for-skips-its-contents⑤②"></a>skipped contents from the accessibility tree entirely.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This requirement is intended to protect users utilizing accessibility tooling from being identified and profiled as such via observation of timing channels; if a user agent can skip significant amounts of work when rendering visually, but has to do all of the work when rendering to an accessibility tree, then an author can tell how a user is interacting with the page by observing the timing of layout operations.

### <a id="cv-examples"></a>4.6. Examples

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1f50d3e3"></a>
>
> ```text
> <style>
> .sv {
>   content-visibility: auto;
>   min-height: 50px;
> }
> </style>
> 
> <div class=sv>
>   ... some content goes here ...
> </div>
> ```
>
> <a id="ref-for-propdef-content-visibility③⑤"></a>
>
> <a id="ref-for-skips-its-contents⑤③"></a>
>
> The .sv element’s [content-visibility: auto](#propdef-content-visibility) value lets the user-agent manage whether the element is [skipped](#skips-its-contents). Specifically when this element is near the viewport, the user-agent will begin painting the element. When the element moves away from the viewport, it will stop being painted. In addition, the user-agent should skip as much of the rendering work as possible when the element is <a id="ref-for-skips-its-contents⑤④"></a>skipped.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-52a4682e"></a>
>
> ```text
> <style>
> .sv {
>   content-visibility: hidden;
> }
> </style>
> 
> <div class=sv>
>   ... some content goes here ...
> </div>
> ```
>
> <a id="ref-for-skips-its-contents⑤⑤"></a>
>
> <a id="ref-for-element-contents③"></a>
>
> <a id="ref-for-propdef-content-visibility③⑥"></a>
>
> In this case, the element is [skipped](#skips-its-contents) regardless of viewport intersection. This means that the only way to have the [contents](#element-contents) painted is via script updating the value to remove [content-visibility](#propdef-content-visibility) or change its value. As before, the user-agent should skip as much of the rendering in the <a id="ref-for-element-contents④"></a>contents as possible.
>
> <a id="ref-for-element-contents⑤"></a>
>
> <a id="ref-for-propdef-content-visibility③⑦"></a>
>
> <a id="ref-for-propdef-display①⓪"></a>
>
> An additional effect of skipping rendering is that the layout state of the [contents](#element-contents) can be preserved by the user-agent, so that removing the [content-visibility](#propdef-content-visibility) property in the future will cause the <a id="ref-for-element-contents⑥"></a>contents to be rendered quicker than if they were hidden with [display: none](https://www.w3.org/TR/css-display-3/#propdef-display) or similar.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a89bb171"></a>
>
> ```text
> <style>
> body {
>   margin: 0;
> }
> .sv {
>   content-visibility: hidden;
>   position: relative;
>   left: 10px;
>   top: 20px;
> }
> #child {
>   position: relative;
>   left: 1px;
>   top: 2px;
>   width: 100px;
>   height: 200px;
> }
> </style>
> 
> <div id=target class=sv>
>   <div id=child></div>
>   ... some other content goes here ...
> </div>
> <script>
>   ...
>   // This will force rendering work, including layout,
>   // if the UA previously avoided it.
>   target.firstElementChild.getBoundingClientRect();
>   ...
> </script>
> ```
>
> <a id="ref-for-skips-its-contents⑤⑥"></a>
>
> <a id="ref-for-element-contents⑦"></a>
>
> <a id="ref-for-dom-element-getboundingclientrect①"></a>
>
> Similarly to the last example, the element is [skipped](#skips-its-contents). The user-agent should avoid as much rendering work as possible. However, in this example, at some point script accesses a layout value in the element’s [contents](#element-contents). In this situation, the user-agent cannot avoid rendering work and has to process any previously skipped rendering work in order to return a correct value to the caller. In this example, the result of <code><a href="https://www.w3.org/TR/cssom-view/#dom-element-getboundingclientrect">getBoundingClientRect()</a></code> is a rect positioned at (11, 22) with a size 100x200.
>
> Note that repeated calls to the same layout value should not cause any additional rendering work, since the user-agent should retain the last updated rendering state.
>
> Also note that this situation in which rendering work is required is not unique. There may be other situations in which the user-agent cannot avoid rendering work.

## <a id="privacy"></a>5.  Privacy Considerations<a id="priv-sec"></a>

There are no known privacy impacts of the features in this specification.

## <a id="security"></a>6.  Security Considerations

There are no known security impacts of the features in this specification.

Like any other CSS specification, it affects the rendering of the document, but does not introduce any special ability to present content in a misleading way that was not previously available through other CSS modules and that isn’t inherent to the act of formatting the document.

## <a id="changes"></a>Appendix A. Changes

This appendix is <em>informative</em>.

### <a id="changes-since-2020-12-16"></a> Changes from [2020-12-16 Working Draft](https://www.w3.org/TR/2020/WD-css-contain-2-20201216/) 

- <a id="ref-for-style-containment⑨"></a>

  <a id="ref-for-valdef-contain-strict"></a>

  <a id="ref-for-valdef-contain-content"></a>

  Included [style containment](#style-containment) in the [strict](#valdef-contain-strict) and [content](#valdef-contain-content) keywords.

- <a id="ref-for-overflow-clip-edge②"></a>

  <a id="ref-for-border-edge"></a>

  <a id="ref-for-relevant-to-the-user⑤"></a>

  Use the [overflow clip edge](https://www.w3.org/TR/css-overflow-3/#overflow-clip-edge) rather than the [border edge](https://www.w3.org/TR/css-box-4/#border-edge) when deciding if the element is “on screen”, as part of determining if it is [relevant to the user](#relevant-to-the-user)

- Define how animations and transitions act on skipped elements

- Remove "at risk" marker from style containment

- Disable propagation from the HTML body element when containment is on

- <a id="ref-for-valdef-contain-strict①"></a>

  <a id="ref-for-valdef-contain-content①"></a>

  Made style containment part of [strict](#valdef-contain-strict) and [content](#valdef-contain-content)

- Clarify that scrollIntoView() doesn’t scroll to children of a content-visibility:hidden element

- <a id="ref-for-propdef-content-visibility③⑧"></a>

  Defined that elements having an ancestor with [content-visibility: hidden](#propdef-content-visibility) don’t generate boxes in the top layer

- Defined that being in the top-layer makes an element relevant to the user

- Noted that paint effects with non-local effects can limit certain optimization opportunities.

- Add ContentVisibilityAutoStateChanged event

### <a id="changes-since-2020-06-03"></a> Changes from [2020-06-03 Working Draft](https://www.w3.org/TR/2020/WD-css-contain-2-20200603/) 

- Changed how the computed values of the contain property are determined

- <a id="ref-for-propdef-contain①⑤"></a>

  Fixed a syntax error in the note about [contain: content](#propdef-contain)

- Change terminology: replace "containing box" with "containment box" (in sync with the same improvements made to Level 1)

- Editorial improvements and clarifications to size and paint containment (in sync with the same improvements made to Level 1)

- Be explicit that size containment suppresses natural aspect ratio (in sync with the same improvements made to Level 1)

- Change the animation type of content-visibility from discrete to not animatable

- <a id="ref-for-propdef-content-visibility③⑨"></a>

  Add constraint on the timing of the initial determination of visibility for [content-visibility: auto](#propdef-content-visibility) in [§ 4.4 Restrictions and Clarifications](#cv-notes)

### <a id="changes-since-2019-11-11"></a> Changes from [2019-11-11 Working Draft](https://www.w3.org/TR/2019/WD-css-contain-2-20191111/) 

- <a id="ref-for-paint-containment⑨"></a>

  <a id="ref-for-propdef-overflow-clip-margin①"></a>

  Define the interaction between [paint containment](#paint-containment) and [overflow-clip-margin](https://www.w3.org/TR/css-overflow-3/#propdef-overflow-clip-margin)

- <a id="ref-for-propdef-content-visibility④⓪"></a>

  Add the [content-visibility](#propdef-content-visibility) property

### <a id="l1-changes"></a> Changes from [CSS Containment Level 1](https://www.w3.org/TR/css-contain-1/) 

- <a id="ref-for-style-containment①⓪"></a>

  Restored [style containment](#style-containment), which had been dropped from Level 1

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

- [auto](#valdef-content-visibility-auto), in § 4
- [constructor(type)](#dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent), in § 4.3
- [constructor(type, eventInitDict)](#dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent), in § 4.3
- [contain](#propdef-contain), in § 2
- [containment](#containment), in § 3
- [content](#valdef-contain-content), in § 2
- [contents](#element-contents), in § 4
- [content-visibility](#propdef-content-visibility), in § 4
- [contentvisibilityautostatechanged](#eventdef-element-contentvisibilityautostatechanged), in § 4.3
- [ContentVisibilityAutoStateChangedEvent](#contentvisibilityautostatechangedevent), in § 4.3
- [ContentVisibilityAutoStateChangedEventInit](#dictdef-contentvisibilityautostatechangedeventinit), in § 4.3
- [ContentVisibilityAutoStateChangedEvent(type)](#dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent), in § 4.3
- [ContentVisibilityAutoStateChangedEvent(type, eventInitDict)](#dom-contentvisibilityautostatechangedevent-contentvisibilityautostatechangedevent), in § 4.3
- [hidden](#valdef-content-visibility-hidden), in § 4
- [Laying out in-place](#laying-out-in-place), in § 3.1
- [layout](#valdef-contain-layout), in § 2
- [layout containment](#layout-containment), in § 3.2
- [layout containment box](#layout-containment-box), in § 3.2
- [none](#valdef-contain-none), in § 2
- [paint](#valdef-contain-paint), in § 2
- [paint containment](#paint-containment), in § 3.4
- [paint containment box](#paint-containment-box), in § 3.4
- [relevant to the user](#relevant-to-the-user), in § 4
- [scoped](#property-scoped), in § 3.3
- [scoped properties](#property-scoped), in § 3.3
- [scoped property](#property-scoped), in § 3.3
- [scoped to an element](#property-scoped-to-an-element), in § 3.3
- [scoped to an element’s sub-tree](#property-scoped-to-a-sub-tree), in § 3.3
- [scoped to a sub-tree](#property-scoped-to-a-sub-tree), in § 3.3
- [scoped to the element](#property-scoped-to-an-element), in § 3.3
- [scoped to the element’s sub-tree](#property-scoped-to-a-sub-tree), in § 3.3
- [scoped to the sub-tree](#property-scoped-to-a-sub-tree), in § 3.3
- [size](#valdef-contain-size), in § 2
- [size containment](#size-containment), in § 3.1
- [size containment box](#size-containment-box), in § 3.1
- [Sizing as if empty](#sizing-as-if-empty), in § 3.1
- [skip](#skips-its-contents), in § 4
- [skip its contents](#skips-its-contents), in § 4
- skipped
  - [attribute for ContentVisibilityAutoStateChangedEvent](#dom-contentvisibilityautostatechangedevent-skipped), in § 4.3
  - [dict-member for ContentVisibilityAutoStateChangedEventInit](#dom-contentvisibilityautostatechangedeventinit-skipped), in § 4.3
- [skipped contents](#skips-its-contents), in § 4
- [skipping its contents](#skips-its-contents), in § 4
- [skips its contents](#skips-its-contents), in § 4
- [strict](#valdef-contain-strict), in § 2
- [style](#valdef-contain-style), in § 2
- [style containment](#style-containment), in § 3.3
- [visible](#valdef-content-visibility-visible), in § 4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-canvas-background"></a>canvas background
- \[css-box-4\] defines the following terms:
  - <a id="term-for-border-edge"></a>border edge
- \[css-break-3\] defines the following terms:
  - <a id="term-for-forced-break"></a>forced break
  - <a id="term-for-fragmentation"></a>fragmentation
  - <a id="term-for-fragmentation-container"></a>fragmentation container
  - <a id="term-for-fragmentation-context"></a>fragmentation context
  - <a id="term-for-fragmented-flow"></a>fragmented flow
  - <a id="term-for-monolithic"></a>monolithic
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-used-value"></a>used value
- \[css-content-3\] defines the following terms:
  - <a id="term-for-valdef-content-close-quote"></a>close-quote
  - <a id="term-for-valdef-content-no-close-quote"></a>no-close-quote
  - <a id="term-for-valdef-content-no-open-quote"></a>no-open-quote
  - <a id="term-for-valdef-content-open-quote"></a>open-quote
- \[css-display-3\] defines the following terms:
  - <a id="term-for-atomic-inline"></a>atomic inline
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-establish-an-independent-formatting-context"></a>establishes an independent formatting context
  - <a id="term-for-initial-containing-block"></a>initial containing block
  - <a id="term-for-inline-level"></a>inline-level
  - <a id="term-for-inner-display-type"></a>inner display type
  - <a id="term-for-internal-ruby-box"></a>internal ruby box
  - <a id="term-for-internal-table-box"></a>internal table box
  - <a id="term-for-valdef-display-none"></a>none
  - <a id="term-for-principal-box"></a>principal box
  - <a id="term-for-replaced-element"></a>replaced element
  - <a id="term-for-valdef-display-table"></a>table
  - <a id="term-for-valdef-display-table-cell"></a>table-cell
  - <a id="term-for-propdef-visibility"></a>visibility
- \[css-grid-2\] defines the following terms:
  - <a id="term-for-grid-track"></a>grid track
- \[css-images-3\] defines the following terms:
  - <a id="term-for-natural-aspect-ratio"></a>natural aspect ratio
  - <a id="term-for-natural-dimensions"></a>natural dimension
  - <a id="term-for-natural-height"></a>natural height
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-propdef-vertical-align"></a>vertical-align
- \[css-lists-3\] defines the following terms:
  - <a id="term-for-propdef-counter-set"></a>counter-set
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-valdef-overflow-clip"></a>clip
  - <a id="term-for-ink-overflow"></a>ink overflow
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-overflow-clip-edge"></a>overflow clip edge
  - <a id="term-for-propdef-overflow-clip-margin"></a>overflow-clip-margin
  - <a id="term-for-propdef-overflow-x"></a>overflow-x
  - <a id="term-for-propdef-overflow-y"></a>overflow-y
  - <a id="term-for-scrollable-overflow"></a>scrollable overflow
  - <a id="term-for-valdef-overflow-visible"></a>visible
- \[css-overflow-4\] defines the following terms:
  - <a id="term-for-selectordef-nth-fragment"></a>::nth-fragment()
- \[css-position-3\] defines the following terms:
  - <a id="term-for-absolute-positioning-containing-block"></a>absolute positioning containing block
  - <a id="term-for-fixed-positioning-containing-block"></a>fixed positioning containing block
  - <a id="term-for-propdef-left"></a>left
  - <a id="term-for-propdef-position"></a>position
- \[css-pseudo-4\] defines the following terms:
  - <a id="term-for-selectordef-after"></a>::after
  - <a id="term-for-selectordef-before"></a>::before
  - <a id="term-for-selectordef-marker"></a>::marker
- \[css-scoping-1\] defines the following terms:
  - <a id="term-for-flat-tree"></a>flat tree
- \[css-sizing-3\] defines the following terms:
  - <a id="term-for-fit-content-size"></a>fit-content size
  - <a id="term-for-propdef-height"></a>height
  - <a id="term-for-intrinsic-size"></a>intrinsic size
  - <a id="term-for-valdef-width-max-content"></a>max-content
  - <a id="term-for-valdef-width-min-content"></a>min-content
  - <a id="term-for-sizing-property"></a>sizing property
  - <a id="term-for-propdef-width"></a>width
- \[css-sizing-4\] defines the following terms:
  - <a id="term-for-propdef-aspect-ratio"></a>aspect-ratio
  - <a id="term-for-propdef-contain-intrinsic-size"></a>contain-intrinsic-size
  - <a id="term-for-preferred-aspect-ratio"></a>preferred aspect ratio
- \[css-transitions-1\] defines the following terms:
  - <a id="term-for-style-change-event"></a>style change event
- \[css-ui-3\] defines the following terms:
  - <a id="term-for-propdef-resize"></a>resize
  - <a id="term-for-propdef-text-overflow"></a>text-overflow
- \[css-ui-4\] defines the following terms:
  - <a id="term-for-propdef-pointer-events"></a>pointer-events
- \[css-values-4\] defines the following terms:
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[css-writing-modes-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-propdef-text-orientation"></a>text-orientation
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-content"></a>content
  - <a id="term-for-propdef-counter-increment"></a>counter-increment
  - <a id="term-for-x43"></a>stacking context
- \[cssom-view-1\] defines the following terms:
  - <a id="term-for-dom-element-getboundingclientrect"></a>getBoundingClientRect()
  - <a id="term-for-dom-element-scrollintoview"></a>scrollIntoView()
- \[DOM\] defines the following terms:
  - <a id="term-for-event"></a>Event
  - <a id="term-for-dictdef-eventinit"></a>EventInit
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="term-for-funcdef-filter-blur"></a>blur()
- \[FULLSCREEN\] defines the following terms:
  - <a id="term-for-top-layer"></a>top layer
- \[HTML\] defines the following terms:
  - <a id="term-for-the-body-element"></a>body
  - <a id="term-for-dom-window-focus"></a>focus()
  - <a id="term-for-the-html-element"></a>html
  - <a id="term-for-the-iframe-element"></a>iframe
  - <a id="term-for-dom-innertext"></a>innerText
  - <a id="term-for-update-the-rendering"></a>update the rendering
- \[intersection-observer\] defines the following terms:
  - <a id="term-for-intersectionobserver"></a>IntersectionObserver
  - <a id="term-for-intersectionobserver-intersection-root"></a>intersection root
- \[resize-observer-1\] defines the following terms:
  - <a id="term-for-resizeobserver"></a>ResizeObserver
- \[SVG2\] defines the following terms:
  - <a id="term-for-elementdef-svg"></a>svg
- \[WEBIDL\] defines the following terms:
  - <a id="term-for-idl-DOMString"></a>DOMString
  - <a id="term-for-Exposed"></a>Exposed
  - <a id="term-for-idl-boolean"></a>boolean

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 21 April 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 22 December 2020. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 23 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 1 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 31 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Pieters. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 17 March 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fullscreen"></a>\[FULLSCREEN\]  
Philip Jägenstedt. [Fullscreen API Standard](https://fullscreen.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fullscreen&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fullscreen.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-intersection-observer"></a>\[INTERSECTION-OBSERVER\]  
Stefan Zager; Emilio Cobos Álvarez. [Intersection Observer](https://www.w3.org/TR/intersection-observer/). 6 July 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;intersection-observer&#x2F;](https://www.w3.org/TR/intersection-observer/)

<a id="biblio-resize-observer-1"></a>\[RESIZE-OBSERVER-1\]  
Aleks Totic; Greg Whitworth. [Resize Observer](https://www.w3.org/TR/resize-observer-1/). 11 February 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;resize-observer-1&#x2F;](https://www.w3.org/TR/resize-observer-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 12 October 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 13 June 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-regions-1"></a>\[CSS-REGIONS-1\]  
Rossen Atanassov; Alan Stearns. [CSS Regions Module Level 1](https://www.w3.org/TR/css-regions-1/). 9 October 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-regions-1&#x2F;](https://www.w3.org/TR/css-regions-1/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                     | Initial | Applies to                                      | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value                                         |
|---------------------|---------------------------------------------------------------------------|---------|-------------------------------------------------|------|-------|----------------|-----------------|--------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-contain①⑥"></a></span><a href="#propdef-contain">contain</a>&#xA;      </strong> | none \| strict \| content \| \[ size \|\| layout \|\| style \|\| paint \] | none    | See below                                       | no   | n/a   | not animatable | per grammar     | the keyword none or one or more of size, layout, paint |
| <strong><span><a id="ref-for-propdef-content-visibility④①"></a></span><a href="#propdef-content-visibility">content-visibility</a>&#xA;      </strong> | visible \| auto \| hidden                                                 | visible | elements for which layout containment can apply | no   | n/a   | not animatable | per grammar     | as specified                                           |

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface ContentVisibilityAutoStateChangedEvent : Event {
  constructor(DOMString type, optional ContentVisibilityAutoStateChangedEventInit eventInitDict = {});
  readonly attribute boolean skipped;
};
dictionary ContentVisibilityAutoStateChangedEventInit : EventInit {
  boolean skipped = false;
};

```