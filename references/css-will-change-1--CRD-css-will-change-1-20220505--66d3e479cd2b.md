Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Will Change Module Level 1](https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/).

Original copyright notice: Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Will Change Module Level 1

Source snapshot: https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/

Snapshot SHA-256: 66d3e479cd2b49c5998afa78cb8a571235c04fa42c3a47a053494d296b1bd9f4

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 2 source tables are presented as readable Markdown tables or explicit labeled layouts: 2 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Will Change Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2022 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-propdef-will-change"></a>

This document defines the [will-change](#propdef-will-change) CSS property, which allows an author to inform the UA ahead of time of what kinds of changes they are likely to make to an element. This allows the UA to optimize how they handle the element ahead of time, performing potentially-expensive work preparing for an animation before the animation actually begins.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-will-change” in the title, like this: “\[css-will-change\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-will-change%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

Modern CSS renderers perform a number of complex optimizations in order to render webpages quickly and efficiently. Unfortunately, employing these optimizations often has a non-trivial start-up cost, which can have a negative impact on the responsiveness of a page.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d1b18fd8"></a> For example, when using CSS 3D Transforms to move an element around the screen, the element and its contents might be promoted to a “layer”, where they can render independently from the rest of the page and be composited in later. This isolates the rendering of the content so that the rest of the page doesn’t have to be rerendered if the element’s transform is the only thing that changes between frames, and often provides significant speed benefits.
>
> <a id="ref-for-propdef-transform"></a>
>
> However, setting up the element in a fresh layer is a relatively expensive operation, which can delay the start of a [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) animation by a noticeable fraction of a second.

<a id="ref-for-propdef-will-change①"></a>

The [will-change](#propdef-will-change) property defined in this specification allows an author to declare ahead-of-time what properties are likely to change in the future, so the UA can set up the appropriate optimizations some time before they’re needed. This way, when the actual change happens, the page updates in a snappy manner.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-will-change②"></a>

### <a id="using"></a>1.2.  Using [will-change](#propdef-will-change) Well

<a id="ref-for-propdef-will-change③"></a>

The [will-change](#propdef-will-change) property, like all performance hints, can be somewhat difficult to learn how to use “properly”, particularly since it has very little, if any, effect an author can directly detect. However, there are several simple “Dos and Don’ts” which hopefully will help develop a good intuition about how to use <a id="ref-for-propdef-will-change④"></a>will-change well.

<a id="ref-for-propdef-will-change⑤"></a>

#### <a id="dont-global"></a> Don’t Spam [will-change](#propdef-will-change) Across Too Many Properties or Elements

<a id="ref-for-propdef-will-change⑥"></a>

A common initial response to seeing [will-change](#propdef-will-change) is to assume that code like this is a good idea:

```text
* { will-change: transform, opacity /* , ... */; }
```
After all, this tells the browser to go ahead and optimize everything, which has to be good right?

<a id="ref-for-propdef-will-change⑦"></a>

Wrong. The browser <em>already</em> tries as hard as it can to optimize everything. Telling it to do so explicitly doesn’t help anything, and in fact has the capacity to do a lot of harm; some of the stronger optimizations that are likely to be tied to [will-change](#propdef-will-change) end up using a lot of a machine’s resources, and when overused like this can cause the page to slow down or even crash.

<a id="ref-for-propdef-will-change⑧"></a>

In addition, [will-change](#propdef-will-change) does have <strong>some</strong> side-effects, and it’s very unlikely that pages actually want all those side-effects on every element.

<a id="ref-for-propdef-will-change⑨"></a>

#### <a id="css-sparingly"></a> Use [will-change](#propdef-will-change) Sparingly In Stylesheets

<a id="ref-for-propdef-will-change①⓪"></a>

Using [will-change](#propdef-will-change) directly in a stylesheet implies that the targeted elements are always a few moments away from changing. This is <em>usually</em> not what you actually mean; instead, <a id="ref-for-propdef-will-change①①"></a>will-change should usually be flipped on and off via scripting before and after the change occurs (see [Don’t Waste Resources On Elements That Have Stopped Changing](#dont-waste)). However, there are some common circumstances in which it is appropriate to use <a id="ref-for-propdef-will-change①②"></a>will-change directly in a stylesheet.

<a id="ref-for-propdef-will-change①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-831888c0"></a> For example, specifying [will-change](#propdef-will-change) for a small number of persistent UI elements in a page which should react snappily to the user is appropriate:
>
> ```text
> body > .sidebar {
>    will-change: transform;
>    /* Will use 'transform' to slide it out
>       when the user requests. */
> }
> ```
>
> Because this is limited to a small number of elements, the fact that the optimization is rarely actually used doesn’t hurt very much.

<a id="ref-for-propdef-will-change①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1ea11531"></a> Sometimes an element really <em>does</em> change a property nearly constantly. Perhaps it responds to the user’s mouse movements, or just regularly takes some action that causes an animation. In this case, just declaring the [will-change](#propdef-will-change) value in the stylesheet is fine, as it accurately describes that the element will regularly/constantly change, and so should be kept optimized.
>
> ```text
> .cats-flying-around-the-screen {
>   will-change: left, top;
> }
> ```
<a id="ref-for-propdef-will-change①⑤"></a>

#### <a id="give-time"></a> Give [will-change](#propdef-will-change) Sufficient Time To Work

<a id="ref-for-propdef-will-change①⑥"></a>

Another common bad pattern is to apply [will-change](#propdef-will-change) to an element <em>immediately</em> before starting the animation or property change that it’s meant to help with. Unfortunately, most of those optimizations need time to be applied, and so they don’t have enough time to set-up when this is done, and the <a id="ref-for-propdef-will-change①⑦"></a>will-change has little to no effect. Instead, find some way to predict at least slightly ahead of time that something will change, and set <a id="ref-for-propdef-will-change①⑧"></a>will-change <em>then</em>.

<a id="ref-for-propdef-will-change①⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8e9e08f2"></a> For example, if an element is going to change when a user clicks on it, setting [will-change](#propdef-will-change) on hover will usually give at least 200 milliseconds for the optimizations to be set up, as human reaction time is relatively slow. This can be done either via scripting, or rather simply with a CSS rule:
>
> ```text
> .element { transition: opacity .2s; opacity: 1; }
> .element:hover { will-change: opacity; }
> .element:active { opacity: .3; }
> ```
>
> However, a rule like that is useless if the effect is going to happen on hover. In cases like these, it is often still possible to find some way to predict the action before it occurs. For example, hovering an ancestor may give enough lead time:
>
> ```text
> .element { transition: opacity .2s; opacity: 1; }
> .container:hover > .element { will-change: opacity; }
> .element:hover { opacity: .3; }
> ```
#### <a id="dont-waste"></a> Don’t Waste Resources On Elements That Have Stopped Changing

<a id="ref-for-propdef-will-change②⓪"></a>

Because the optimizations browsers use for changing some properties are expensive, browsers remove them and revert to normal behavior as soon as they can in normal circumstances. However, [will-change](#propdef-will-change) will generally override this behavior, maintaining the optimizations for much longer than the browser would otherwise do.

<a id="ref-for-propdef-will-change②①"></a>

As such, whenever you add [will-change](#propdef-will-change) to an element, especially via scripting, don’t forget to <em>remove</em> it after the element is done changing, so the browser can recover whatever resources the optimizations are claiming.

<a id="ref-for-propdef-will-change②②"></a>

## <a id="will-change"></a>2.  Hinting at Future Behavior: the [will-change](#propdef-will-change) property

| Field               | Definition                                                                                                                                                                                                                   |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-will-change"></a>will-change                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-typedef-animateable-feature"></a><a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<animateable-feature\>](#typedef-animateable-feature)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                                                                                                                                               |

<a id="typedef-animateable-feature"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-identifier-value"></a>

```text
<animateable-feature> = scroll-position | contents | <custom-ident>
```
<a id="ref-for-propdef-will-change②③"></a>

The [will-change](#propdef-will-change) property provides a rendering hint to the user agent, stating what kinds of changes the author expects to perform on the element. This allows the user agent to perform ahead-of-time any optimizations necessary for rendering those changes smoothly, avoiding “jank” when the author does begin changing or animating that feature.

<a id="ref-for-propdef-will-change②④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Different browsers can use the information from [will-change](#propdef-will-change) in different ways, and even a single browser might use it in different ways at different time. For example, a browser that promotes elements to their own “GPU layer” when they have <a id="ref-for-propdef-will-change②⑤"></a>will-change: transform specified might avoid doing that when there are <em>too many</em> elements declaring that, to avoid exhausting GPU memory.

Values have the following meanings:

<a id="valdef-will-change-auto"></a>auto

Expresses no particular intent; the user agent should apply whatever heuristics and optimizations it normally does.

<a id="valdef-will-change-scroll-position"></a>scroll-position

Indicates that the author expects to animate or change the scroll position of the element in the near future.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c0594e8a"></a> For example, browsers often only render the content in the "scroll window" on a scrollable element, and some of the content past that window, balancing memory and time savings from the skipped rendering against making scrolling look nice. A browser might take this value as a signal to expand the range of content around the scroll window that is rendered, so that longer/faster scrolls can be done smoothly.

<a id="valdef-will-change-contents"></a>contents

Indicates that the author expects to animate or change something about the element’s contents in the near future.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5dd10a2f"></a> For example, browsers often “cache” rendering of elements over time, because most things don’t change very often, or only change their position. However, if an element <em>does</em> change its contents continually, producing and maintaining this cache is a waste of time. A browser might take this value as a signal to cache less aggressively on the element, or avoid caching at all and just continually re-render the element from scratch.
>
> This value is mostly intended to help browsers optimize JS-based animations of content, which change aspects of an element’s contents many times per second. This kind of optimization, when possible, is already done automatically by browsers when declarative animations are used.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value more-or-less applies to the entire subtree of the element its declared on, as it indicates the browser should count on \*any\* of the descendants changing in some way. Using this on an element “high up” in your document might be very bad for your page’s performance; try to only use this on elements near the “bottom” of your document tree, containing as little of the document as possible.

<a id="ref-for-identifier-value①"></a>

<a id="valdef-will-change-custom-ident"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-identifier-value②"></a>

If the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the name of a built-in CSS property, it indicates that the author expects to animate or change the property with the given name on the element in the near future. If the property given is a shorthand, it indicates the expectation for all the longhands the shorthand expands to.

<a id="ref-for-propdef-will-change②⑥"></a>

<a id="ref-for-propdef-background"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-03a1e696"></a> For example, setting [will-change: background;](#propdef-will-change) is identical to setting <a id="ref-for-propdef-will-change②⑦"></a>will-change: background-image, background-position, ... for all the properties that [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) expands into.

<a id="ref-for-identifier-value③"></a>

<a id="ref-for-valdef-will-change-auto"></a>

<a id="ref-for-valdef-will-change-scroll-position"></a>

<a id="ref-for-valdef-will-change-contents"></a>

The [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) production used here excludes the keywords will-change, none, all, [auto](#valdef-will-change-auto), [scroll-position](#valdef-will-change-scroll-position), and [contents](#valdef-will-change-contents), in addition to the keywords normally excluded from <a id="ref-for-identifier-value④"></a>\<custom-ident\>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that most properties will have no effect when specified, as the user agent doesn’t perform any special optimizations for changes in most properties. It is still <em>safe</em> to specify them, though; it’ll simply have no effect.

Specifying a custom property must have no effect, which means that effects that happen through custom properties do not count for the rules below that are conditioned on any non-initial value of a property causing something.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifying a value that’s not recognized as a property is fine; it simply has no effect. This allows you to safely specify <em>new</em> properties that exist in some user agents without negatively affecting down-level user agents that don’t know about that property.

<a id="ref-for-propdef-transform①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7225dcbc"></a> For example, browsers often handle elements with [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) set to a non-initial value very differently from normal elements, perhaps rendering them to their own “GPU layer” or using other mechanisms to make it easier to quickly make the sort of transformations that <a id="ref-for-propdef-transform②"></a>transform can produce. A browser might take a value of <a id="ref-for-propdef-transform③"></a>transform as a signal that it should go ahead and promote the element to its own layer immediately, before the element starts to be transformed, to avoid any delay involved in rerendering the old and new layers.

<a id="ref-for-propdef-will-change②⑧"></a>

If any non-initial value of a property would create a stacking context on the element, specifying that property in [will-change](#propdef-will-change) must create a stacking context on the element.

<a id="ref-for-propdef-will-change②⑨"></a>

If any non-initial value of a property would cause the element to generate a containing block for absolutely positioned elements, specifying that property in [will-change](#propdef-will-change) must cause the element to generate a containing block for absolutely positioned elements.

<a id="ref-for-propdef-will-change③⓪"></a>

If any non-initial value of a property would cause the element to generate a containing block for fixed positioned elements, specifying that property in [will-change](#propdef-will-change) must cause the element to generate a containing block for fixed positioned elements.

<a id="ref-for-propdef-will-change③①"></a>

If any non-initial value of a property would cause rendering differences on the element (such as using a different anti-aliasing strategy for text), the user agent should use that alternate rendering when the property is specified in [will-change](#propdef-will-change), to avoid sudden rendering differences when the property is eventually changed.

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-propdef-will-change③②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e1300843"></a> For example, setting [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) to any value other than 1 creates a stacking context on the element. Thus, setting [will-change: opacity](#propdef-will-change) also creates a stacking context, even if <a id="ref-for-propdef-opacity①"></a>opacity is <em>currently</em> still equal to 1.

<a id="ref-for-propdef-will-change③③"></a>

The [will-change](#propdef-will-change) property has no <em>direct</em> effect on the element it is specified on, beyond the creation of stacking contexts and containing blocks as specified above. It is solely a rendering hint to the user agent, allowing it set up potentially-expensive optimizations for certain types of changes before the changes actually start occurring.

## <a id="security"></a>3. Security Considerations

No Security concerns have been raised against this document

## <a id="privacy"></a>4. Privacy Considerations

No Privacy concerns have been raised against this document

## <a id="acks"></a>5. Acknowledgements

Thanks to Benoit Girard for originally suggesting the will-animate property, and doing a lot of the initial design work.

## <a id="changes"></a>6. Changes

Since the [03 December 2015 CR](https://www.w3.org/TR/2015/CR-css-will-change-1-20151203/):

- Added Security and Privacy sections

- Clarified that unknown values are fine, and have no effect

- Specified that ASCII Case-Insensitive matching is used against property names

- Changed the animation type of the will-change property to not animatable

- Dropped the "Media:" entry from propdef tables, as with other CSS specifications

- Minor editorial clarifications, markup improvements

Since the [April 29 2014 Working Draft](https://www.w3.org/TR/2014/WD-css-will-change-1-20140429/):

- <a id="ref-for-propdef-will-change③④"></a>

  Added an explanatory section giving guidance on how to use [will-change](#propdef-will-change) well.

- Specified the behavior of shorthands

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

- [\<animateable-feature\>](#typedef-animateable-feature), in § 2
- [auto](#valdef-will-change-auto), in § 2
- [contents](#valdef-will-change-contents), in § 2
- [\<custom-ident\>](#valdef-will-change-custom-ident), in § 2
- [scroll-position](#valdef-will-change-scroll-position), in § 2
- [will-change](#propdef-will-change), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background"></a>background
- \[css-color-4\] defines the following terms:
  - <a id="term-for-propdef-opacity"></a>opacity
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-propdef-transform"></a>transform
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-identifier-value"></a>\<custom-ident\>
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-comb-one"></a>\|
- \[INFRA\] defines the following terms:
  - <a id="term-for-ascii-case-insensitive"></a>ascii case-insensitive

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 15 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                            | Initial | Applies to   | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value  |
|---------------------|----------------------------------|---------|--------------|------|-------|----------------|-----------------|-----------------|
| <strong><span><a id="ref-for-propdef-will-change③⑤"></a></span><a href="#propdef-will-change">will-change</a>&#xA;      </strong> | auto \| \<animateable-feature\># | auto    | all elements | no   | n/a   | not animatable | per grammar     | specified value |

