Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [CSS Scoping Module Level 1](https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/).

Original copyright notice: Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Scoping Module Level 1

Source snapshot: https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/

Snapshot SHA-256: 0742453932fc6d802845b59084c51007069c7ddb4b689b59017bb546637c2140

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Scoping Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2014 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

## <a id="abstract"></a>Abstract

This specification defines various scoping/encapsulation mechanisms for CSS, including scoped styles and the [@scope](#at-ruledef-scope) rule, Shadow DOM selectors, and page/region-based styling. [CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, in speech, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of
   its publication. Other documents may supersede this document. A list of
   current W3C publications and the latest revision of this technical report
   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports
   index at http&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document is a <b>First Public Working Draft</b>.

Publication as a First Public Working Draft does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

The ([archived](http://lists.w3.org/Archives/Public/www-style/)) public mailing list <www-style@w3.org> (see [instructions](https://www.w3.org/Mail/Request)) is preferred for discussion of this specification. When sending e-mail, please put the text “css-scoping” in the subject, preferably like this: “\[css-scoping\] <em>…summary of comment…</em>”

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) (part of the [Style Activity](https://www.w3.org/Style/)).

This document was produced by a group operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

## <a id="contents"></a>Table of Contents

## <a id="intro"></a>1  Introduction

...

## <a id="scope"></a>2  Scoped Styles

[Scoped](https://www.w3.org/TR/css3-cascade/#scoped) style rules apply only within a subtree of a document, rather than matching against the entire document. Scoping has two primary effects:

- The selector of the [scoped](https://www.w3.org/TR/css3-cascade/#scoped) style rule is restricted to match only elements within scope. See [Scoped Selectors](https://www.w3.org/TR/selectors4/#scoping) in [\[SELECTORS4\]](#selectors4).
- The cascade prioritizes scoped rules over unscoped ones, regardless of specificity. See [Cascading by Scope](https://www.w3.org/TR/css-cascade/#cascade-scope) in [\[CSS3CASCADE\]](#css3cascade).

### <a id="scoping-mechanisms"></a>2.1  Scoping Mechanisms

Style rules can be scoped using constructs defined in the document language or using the [@scope](#at-ruledef-scope) rule in CSS.

#### <a id="scoping-markup"></a>2.1.1  Document Markup for Scoping

Document languages may define a mechanism for a stylesheet to be scoped to some element in the document. For example, in HTML, a [style](https://www.w3.org/TR/html5/document-metadata.html#the-style-element) element with a scoped attribute defines a stylesheet that is scoped to the [style](https://www.w3.org/TR/html5/document-metadata.html#the-style-element) element’s parent element. [\[HTML\]](#html)

#### <a id="scope-atrule"></a>2.1.2  CSS Syntax for Scoping: the [@scope](#at-ruledef-scope) rule

The <a id="at-ruledef-scope"></a>@scope at-rule allows authors to create scoped style rules using CSS syntax. The syntax of the [@scope](#at-ruledef-scope) rule is:

```text
@scope <selector> {
  <stylesheet>
}
```
where the elements matched by the [\<selector\>](https://www.w3.org/TR/selectors4/#ltselector) are [scoping roots](http://dev.w3.org/csswg/selectors-4/#scoping-root) for the style rules in [\<stylesheet\>](https://www.w3.org/TR/css3-syntax/#typedef-stylesheet), and selectors of style rules scoped by [@scope](#at-ruledef-scope) are [scope-contained](http://dev.w3.org/csswg/selectors-4/#scope-contained-) to their [scoping root](http://dev.w3.org/csswg/selectors-4/#scoping-root).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5f568f10"></a> This rule makes it very easy for authors to create scoped style sheets, which could affect the optimization strategies for implementing scoped styles.

If multiple elements match the [\<selector\>](https://www.w3.org/TR/selectors4/#ltselector), the [\<stylesheet\>](https://www.w3.org/TR/css3-syntax/#typedef-stylesheet) is effectively duplicated and scoped independently to each one. Authors should avoid using overly-generic selectors as it can have confusing interactions with the cascade.

> <strong data-conversion-semantic="example">Example</strong>
>
> A scoped stylesheet is attached not only to the outermost scoping element, but to all matching elements. For example, given the style sheet below
>
> ```text
> @scope div {
>   span {
>     color: blue;
>   }
> }
> @scope section {
>   span {
>     color: orange;
>   }
> }
> ```
>
> and the following document fragment
>
> ```text
> <div>
>   <section>
>     <div>
>       <span>text</span>
>     </div>
>   </section>
> </div>
> ```
>
> the text will be blue.

[@scope](#at-ruledef-scope) rules can be nested. In this case, just as with the nested style rules, the selector of an outer [@scope](#at-ruledef-scope) scope-contains the selector of the inner one.

The specificity of selectors inside the [@scope](#at-ruledef-scope) rule is calculated locally: the selector specifying the scoping element is ignored. However, because scoped styles override non-scoped styles, style rules inside the [@scope](#at-ruledef-scope) will override rules outside of it.

> <strong data-conversion-semantic="example">Example</strong>
>
> In the following example, the text would be green:
>
> ```text
>   @scope aside {
>     p { color: green; }
>   }
>   aside#sidebar p { color: red; }
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5c7571b6"></a> If multiple [@scope](#at-ruledef-scope) rules apply to an element, should they be cascaded by specificity?

### <a id="scoping-context"></a>2.2  Querying the Scoping Context

#### <a id="scope-pseudo"></a>2.2.1  Selecting the Scoping Root: [:scope](https://www.w3.org/TR/selectors4/#scope-pseudo) pseudo-class

In a scoped stylesheet, the [:scope](https://www.w3.org/TR/selectors4/#scope-pseudo) pseudo-class, defined in [\[SELECTORS4\]](#selectors4), matches the [scoping root](http://dev.w3.org/csswg/selectors-4/#scoping-root).

#### <a id="scope-content-pseudo"></a>2.2.2  Selecting Outside the Scope: :scope-context() pseudo-class

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-aecbd194"></a> This would be defined similarly to [:host-context()](#selectordef-host-context), but matching the ancestors of the [scoping root](http://dev.w3.org/csswg/selectors-4/#scoping-root) instead.
>
> However, since for scoped stylesheets you may want the ability to match complex selectors against the outside tree, rather than a single compound selector, we may want to instead use a more general mechanism that doesn’t syntactically invert the order of tree elements.
>
> Possible ideas:
>
> ```text
> :scope-context(<selector>) div {...}
> scope(<selector>) div {...}
> \scope <selector>\ div {...}
> <selector> \scope\ div {...}
> ```
>
> This functionality would replace @global, which is a poor excuse for a selector.

## <a id="shadow-dom"></a>3  Shadow Encapsulation

The Shadow DOM spec augments the DOM with several new concepts, several of which are relevant to CSS.

A <a id="shadow-tree"></a>shadow tree is a document fragment that can be attached to any element in the DOM. The root of the [shadow tree](#shadow-tree) is a <a id="shadow-root"></a>shadow root, a non-element node which is associated with a [shadow host](#shadow-host). An element can have any number of [shadow trees](#shadow-tree), which are ordered by creation time. The most recently-created [shadow tree](#shadow-tree) on an element is the <a id="active-shadow-tree"></a>active shadow tree for that element.

An element with a [shadow tree](#shadow-tree) is a <a id="shadow-host"></a>shadow host. It is the <a id="host-element0"></a>host element for its shadow trees.

The descendants of a [shadow host](#shadow-host) must not generate boxes in the formatting tree. Instead, the contents of the [active shadow tree](#active-shadow-tree) generate boxes as if they were the contents of the element instead.

In several instances in shadow DOM, elements don’t have element parents (instead, they may have a [shadow root](#shadow-root) as parent, or something else). An element without a parent, or whose parent is not an element, is called a <a id="top-level-element"></a>top-level element.

While the children of a [shadow host](#shadow-host) do not generate boxes normally, they can be explicitly pulled into a [shadow tree](#shadow-tree) and forced to render normally. This is done by assigning the elements to a <a id="distribution-list"></a>distribution list. An element with a [distribution list](#distribution-list) is an <a id="insertion-point"></a>insertion point.

This specification does not define how to assign elements to a [distribution list](#distribution-list), instead leaving that to the Shadow DOM spec. At the time this spec is written, however, only content elements in a [shadow tree](#shadow-tree) can have [distribution lists](#distribution-list).

An [insertion point](#insertion-point) must not generate any boxes. Instead, the elements in its [distribution list](#distribution-list) generate boxes as normal, as if they all replaced the [insertion point](#insertion-point) in-place. <strong data-conversion-semantic="note">Note:</strong> (Akin to the behavior of [display-box: contents](http://dev.w3.org/csswg/css-display-3/#propdef-display-box).)

### <a id="selectors-data-model"></a>3.1  Shadow DOM Selection Model

Elements in the [element tree](http://dev.w3.org/csswg/selectors-4/#element-tree) additionally have zero or more [shadow trees](#shadow-tree) and zero or one [distribution lists](#distribution-list).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The "descendants" of an element are based on the children of the element, which does not include the [shadow trees](#shadow-tree) or [distribution lists](#distribution-list) of the element.

When a selector is matched against a [shadow tree](#shadow-tree), the [initial selector match list](http://dev.w3.org/csswg/selectors-4/#initial-selector-match-list) is the [shadow host](#shadow-host), followed by all the [top-level elements](#top-level-element) of the [shadow tree](#shadow-tree) and their descendants, ordered by a pre-order traversal.

#### <a id="host-element"></a>3.1.1  Host Elements in a Shadow Tree

A [shadow host](#shadow-host) is outside of the [shadow trees](#shadow-tree) it hosts, but it is sometimes useful to be able to style it from inside the [shadow tree](#shadow-tree) context.

For the purpose of Selectors, a [host element](#host-element0) also appears in each of its [shadow trees](#shadow-tree), with the contents of the [shadow tree](#shadow-tree) treated as its children. If an element has multiple [shadow trees](#shadow-tree), it appears in each [shadow tree’s](#shadow-tree) context independently; each [shadow tree](#shadow-tree) sees <em>itself</em> as the contents of the [host element](#host-element0), not the other [shadow trees](#shadow-tree).

The [host element](#host-element0) is not selectable by <strong>any means</strong> except for the [:host](#selectordef-host0) and [:host-context()](#selectordef-host-context) pseudo-classes. That is, in this context the [shadow host](#shadow-host) has no tagname, ID, classes, or attributes, and the only additional information is has is that the [:host](#selectordef-host0) pseudo-class matches it. In particular, the [host element](#host-element0) isn’t matched by the \* selector either.

Why is the shadow host so weird?

The [shadow host](#shadow-host) lives outside the [shadow tree](#shadow-tree), and its markup is in control of the page author, not the component author.

It would not be very good if a component used a particular class name internally in a [shadow tree](#shadow-tree), and the page author using the component accidentally <em>also</em> used the the same class name and put it on the [host element](#host-element0). Such a situation would result in accidental styling that is impossible for the component author to predict, and confusing for the page author to debug.

However, there are still some reasonable use-cases for letting a stylesheet in a [shadow tree](#shadow-tree) style its [host element](#host-element0). So, to allow this situation but prevent accidental styling, the [host element](#host-element0) appears but is completely featureless and unselectable except through [:host](#selectordef-host0).

### <a id="selectors"></a>3.2  Shadow DOM Selectors

Shadow DOM defines a few new selectors to help select elements in useful way related to Shadow DOM.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-dea392ae"></a> This section is still under discussion. Feedback and advice on intuitive syntax for the following functionality would be appreciated.

#### <a id="host-selector"></a>3.2.1  Selecting Into the Light: the [:host](#selectordef-host0), [:host()](#selectordef-host), and [:host-context()](#selectordef-host-context) pseudo-classes

The <a id="selectordef-host0"></a>:host pseudo-class, when evaluated in the context of a [shadow tree](#shadow-tree), matches the [shadow tree’s](#shadow-tree) [host element](#host-element0). In any other context, it matches nothing.

The <a id="selectordef-host"></a>:host() function pseudo-class has the syntax:

```text
:host( <compound-selector> )
```
When evaluated in the context of a [shadow tree](#shadow-tree), it matches the [shadow tree’s](#shadow-tree) [host element](#host-element0) if the [host element](#host-element0), in its normal context, matches the selector argument. In any other context, it matches nothing.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, say you had a component with a [shadow tree](#shadow-tree) like the following:
>
> ```text
>   <x-foo class="foo">
>     <"shadow tree">
>       <div class="foo">...</div>
>     </>
>   </x-foo>
> ```
>
> For a stylesheet within the [shadow tree](#shadow-tree):
>
> - [:host](#selectordef-host0) matches the `<x-foo>` element.
> - x-foo matches nothing.
> - .foo matches only the `<div>` element.
> - .foo:host matches nothing
> - :host(.foo) matches the `<x-foo>` element.

Ordinary, selectors within a [shadow tree](#shadow-tree) can’t see elements outside the [shadow tree](#shadow-tree) at all. Sometimes, however, it’s useful to select an ancestor that lies somewhere outside the shadow tree, above it in the document.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, a group of components can define a handful of color themes they they know how to respond to. Page authors could opt into a particular theme by adding a specific class to the components, or higher up in the document.

The <a id="selectordef-host-context"></a>:host-context() functional pseudo-class tests whether there is an ancestor, outside the [shadow tree](#shadow-tree), which matches a particular selector. Its syntax is:

```text
:host-context( <compound-selector> )
```
When evaluated in the context of a [shadow tree](#shadow-tree), the [:host-context()](#selectordef-host-context) pseudo-class matches the [host element](#host-element0), if the [host element](#host-element0) or one of its ancestors matches the provided [\<compound-selector\>](https://www.w3.org/TR/selectors4/#ltcompound-selector). For the purpose of this pseudo-class, the "ancestor" of an element is:

if the element is distributed to a [distribution list](#distribution-list)  
the content element it is ultimately distributed to.

if the element is a top-most element in a shadow tree  
the [host element](#host-element0)

otherwise  
the element’s parent, if it has one.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that the selector pierces through shadow boundaries on the way up, looking for elements that match its argument, until it reaches the document root.

#### <a id="shadow-pseudoelement"></a>3.2.2  Selecting Into the Dark: the [::shadow](#selectordef-shadow) pseudo-element

If an element has at least one [shadow tree](#shadow-tree), the <a id="selectordef-shadow"></a>::shadow pseudo-element matches the [shadow roots](#shadow-root) themselves. In HTML, the [shadow root](#shadow-root) is represented by ShadowRoot objects.

The [::shadow](#selectordef-shadow) pseudo-element must not generate boxes, unless specified otherwise in another specification. However, for the purpose of Selectors, the [::shadow](#selectordef-shadow) pseudo-element is considered to be the root of the [shadow tree](#shadow-tree), with the [top-level elements](#top-level-element) in the [shadow tree](#shadow-tree) the direct children of the [::shadow](#selectordef-shadow) pseudo-element.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, say you had a component with a [shadow tree](#shadow-tree) like the following:
>
> ```text
>   <x-foo>
>     <"shadow tree">
>       <div>
>         <span id="not-top">...</span>
>       </div>
>       <span id="top">...</span>
>     </>
>   </x-foo>
> ```
>
> For a stylesheet in the outer document, x-foo::shadow \> span matches \#top, but not \#not-top, because it’s not a [top-level element](#top-level-element) in the [shadow tree](#shadow-tree).
>
> If one wanted to target \#not-top, one way to do it would be with x-foo::shadow \> div \> span. However, this introduces a strong dependency on the internal structure of the component; in most cases, it’s better to use the descendant combinator, like x-foo::shadow span, to select all the elements of some type in the [shadow tree](#shadow-tree).

> <strong data-conversion-semantic="example">Example</strong>
>
> If an element has multiple [shadow trees](#shadow-tree), a [::shadow](#selectordef-shadow) pseudo-element selects <em>all</em> of the corresponding [shadow roots](#shadow-root).
>
> Similarly, inside of a [shadow tree](#shadow-tree), a selector like :host::shadow div selects the [div](https://www.w3.org/TR/html5/grouping-content.html#the-div-element) elements in <em>all</em> the [shadow trees](#shadow-tree) on the element, not just the one containing that selector.

#### <a id="content-combinator"></a>3.2.3  Selecting Shadow-Projected Content: the [::content](#selectordef-content) pseudo-element

The <a id="selectordef-content"></a>::content pseudo-element matches the [distribution list](#distribution-list) itself, on elements that have one.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1b6118ae"></a> [::content](#selectordef-content) is a confusingly general name for something that is specific to the projected content of a shadow tree.

The [::content](#selectordef-content) pseudo-element must not generate boxes, unless specified otherwise in another specification. However, for the purpose of Selectors, the [::content](#selectordef-content) pseudo-element is considered to be the parent of the elements in the [distribution list](#distribution-list).

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, say you had a component with both children and a shadow tree, like the following:
>
> ```text
>   <x-foo>
>     <div id="one" class="foo">...</div>
>     <div id="two">...</div>
>     <div id="three" class="foo">
>       <div id="four">...</div>
>     </div>
>     <"shadow tree">
>       <div id="five">...</div>
>       <div id="six">...</div>
>       <content select=".foo"></content>
>     </"shadow tree">
>   </x-foo>
> ```
>
> For a stylesheet within the [shadow tree](#shadow-tree), a selector like ::content div selects \#one, \#three, and \#four, as they’re the elements distributed by the sole content element, but not \#two.
>
> If only the [top-level elements](#top-level-element) distributed the content element are desired, a [child combinator](https://www.w3.org/TR/selectors4/#child-combinator) can be used, like ::content \> div, which will exclude \#four as it’s not treated as a child of the [::content](#selectordef-content) pseudo-element.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Note that a selector like ::content div is equivalent to \*::content div, where the \* selects many more elements that just the content element. However, since only the content element has a [distribution list](#distribution-list), it’s the only element that has a [::content](#selectordef-content) pseudo-element as well.

#### <a id="deep-combinator"></a>3.2.4  Selecting Through Shadows: the [/deep/](#selectordef-deep) combinator

When a <a id="selectordef-deep"></a>/deep/ combinator is encountered in a selector, replace every element in the [selector match list](http://dev.w3.org/csswg/selectors-4/#selector-match-list) with every element reachable from the original element by traversing any number of child lists or shadow trees.

> <strong data-conversion-semantic="example">Example</strong>
>
> For example, say you had a component with a [shadow tree](#shadow-tree) like the following:
>
> ```text
>   <x-foo>
>     <"shadow tree">
>       <div>
>         <span id="not-top">...</span>
>       </div>
>       <span id="top">...</span>
>       <x-bar>
>         <"shadow tree">
>           <span id="nested">...</span>
>         </>
>       </x-bar>
>     </>
>   </x-foo>
> ```
>
> For a stylesheet in the outer document, the selector x-foo /deep/ span selects all three of `<span>` elements: \#top, \#not-top, <em>and</em> \#nested.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-250ee37f"></a> This is basically a super-descendant combinator. If the descendant combinator had a real glyph, it would potentially be interesting to just double it. Maybe we can give the descendant combinator a pseudonym of \>\>, as it itself is a super-child combinator? Then [/deep/](#selectordef-deep) could be spelled \>\>\>

### <a id="shadow-cascading"></a>3.3  Shadow Cascading &#x26; Inheritance

#### <a id="cascading"></a>3.3.1  Cascading

To address the desired cascading behavior of rules targetting elements in shadow roots, this specification extends the [cascade order](http://dev.w3.org/csswg/css-cascade/#cascading) defined in the Cascade specification. [\[CSS3CASCADE\]](#css3cascade)

An additional cascade criteria must be added, between Origin and Scope, called Shadow Tree.

- When comparing two declarations, if one of them is in a [shadow tree](#shadow-tree) and the other is in a document that contains that [shadow tree](#shadow-tree), then for normal rules the declaration from the outer document wins, and for important rules the declaration from the [shadow tree](#shadow-tree) wins.
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This is the <em>opposite</em> of how scoped styles work.
- When comparing two declarations, if both are in [shadow trees](#shadow-tree) with the same [host element](#host-element0), then for normal rules the declaration from the [shadow tree](#shadow-tree) that was created most recently wins, and for important rules the declaration from the [shadow tree](#shadow-tree) that was created less recently wins.

When calculating [Order of Appearance](http://dev.w3.org/csswg/css-cascade/#cascade-order), the tree of trees, defined by the Shadow DOM specification, is used to calculate ordering.

#### <a id="inheritance"></a>3.3.2  Inheritance

The [top-level elements](#top-level-element) of a [shadow tree](#shadow-tree) inherit from their [host element](#host-element0).

The elements in a [distribution list](#distribution-list) inherit from the parent of the content element they are ultimately distributed to, rather than from their normal parent.

## <a id="fragment-scoping"></a>4  Fragmented Styling

Fragmented content can be styled differently based on which line, column, page, region, etc. it appears in. This is done by using an appropriate <a id="fragment-pseudo-element"></a>fragment pseudo-element, which allows targetting individual fragments of an element rather than the entire element.

> <strong data-conversion-semantic="example">Example</strong>
>
> In our example, the designer wants to make text flowing into \#region1 dark blue and bold. This design can be expressed as shown below.
>
> ```text
> #region1::region p {
>   color: #0C3D5F;
>   font-weight: bold;
> }
> ```
>
> The ::region pseudo-element is followed by a p relative selector in this example. The color and font-weight declarations will apply to any fragments of paragraphs that are displayed in \#region1. The following figure shows how the rendering changes if we apply this styling specific to \#region1. Note how less text fits into this box now that the [font-weight](https://www.w3.org/TR/css-fonts-3/#propdef-font-weight) is bold instead of normal.
>
> ![Illustrate how changing region styling affects the flow of content.](https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/images/region-styling.png)
>
> Different rendering with a different region styling

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This feature is an extension of ::first-line styling.

### <a id="the-region-pseudo-element"></a>4.1  Region-based Styling: the ::region pseudo-element

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f778a2c0"></a> Extend this to specify:
>
> - \<region-element-selector\>::region
> - \<paginated-element-selector\>::page(\<page-selector\>)
> - \<multicol-element\>::column(\<AnB\>)
> - \<fragmented-element-selector\>::nth-fragment(\<AnB\>)
> - ::first-line

A ::region pseudo-element represents a relationship between a selector that matches a CSS Region, and a relative selector that matches some named flow content. This allows style declarations to be applied to fragments of named flow content flowing into particular regions.

```text
<region selector>::region <content selector>  {
    ... CSS styling declarations ...
}
```
When the ::region pseudo-element is appended to a [selector](https://www.w3.org/TR/css3-selectors/#selector-syntax) that matches one or more CSS Regions, this creates a 'flow fragment' selector. The flow fragment selector specifies which range of elements in the flow can be matched by the relative selector. The relative selector can match elements in the range(s) (see [\[DOM\]](#dom)) of the named flow that are displayed fully or partially in the selected region(s).

Elements that are fully or partially in the flow fragment range may match the relative selector. However, the style declarations only apply to the fragment of the element that is displayed in the corresponding region(s).

Only a limited list of properties apply to a ::region pseudo-element:

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9b00b7af"></a> Either this list should be all functionally inheritable properties, or all properties. Why is it a seemingly-arbitrary subset of all properties, including box properties?

1.  [font properties](https://www.w3.org/TR/CSS2/fonts.html)
2.  [color property](https://www.w3.org/TR/CSS2/colors.html)
3.  [opacity property](https://www.w3.org/TR/css3-color/#transparency)
4.  [background property](https://www.w3.org/TR/css3-background/#backgrounds)
5.  [word-spacing](https://www.w3.org/TR/css3-text/#word-spacing)
6.  [letter-spacing](https://www.w3.org/TR/css3-text/#letter-spacing)
7.  [text-decoration](https://www.w3.org/TR/css-text-decor-3/#text-decoration)
8.  [text-transform](https://www.w3.org/TR/css3-text/#text-transform)
9.  [line-height](https://www.w3.org/TR/CSS21/visudet.html#propdef-line-height)
10. [alignment and justification properties](https://www.w3.org/TR/css3-text/#justification)
11. [border properties](https://www.w3.org/TR/css3-background/#borders)
12. [rounded corner properties](https://www.w3.org/TR/css3-background/#corners)
13. [border images properties](https://www.w3.org/TR/css3-background/#border-images)
14. [margin properties](https://www.w3.org/TR/CSS2/box.html#margin-properties)
15. [padding properties](https://www.w3.org/TR/CSS2/box.html#padding-properties)
16. [text-shadow](https://www.w3.org/TR/css-text-decor-3/#text-shadow)
17. [box-shadow](https://www.w3.org/TR/css3-background/#box-shadow)
18. [box-decoration-break](https://www.w3.org/TR/css3-break/#box-decoration-break)
19. [width](https://www.w3.org/TR/CSS21/visudet.html#propdef-width)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="region-style-example"></a>
>
> In the following example, the named flow “article-flow” flows into “region-1” and “region-2”.
>
> ```text
> <style>
>   #div-1 {
>     flow-into: article-flow;
>   }
> 
>   #region-1, #region-2 {
>     flow-from: article-flow;
>   }
> 
>   /* region styling */
>   #region-1::region p  {
>     margin-right: 5em;
>   }
> </style>
> 
> <body>
>   <div id="div-1">
>       <p id="p-1">...</p>
>       <p id="p-2">...</p>
>   </div>
>   <div id="region-1"></div>
>   <div id="region-2"></div>
> </body>
> ```
>
> <a id="region_styling_illustration"></a>
>
> <a id="region_styling_img_2"></a>
>
> ![Example showing how a named flow content fits into regions to illustrate region styling.](https://www.w3.org/TR/2014/WD-css-scoping-1-20140403/images/region-styling-2.png)
>
> -  div div-1
> -  paragraph p-1
> -  paragraph p-2
> -  range of flow that fits into region-1
> -  range of flow that fits into region-2
>
> The region styling applies to flow content that fits in region-1. The relative selector matches p-1 and p-2 because these paragraphs flow into region-1. Only the fragment of p-2 that flows into region-1 is styled with the pseudo-element.

All of the selectors in a ::region pseudo-element contribute to its [specificity](https://www.w3.org/TR/css3-selectors/#specificity). So the specificity of the ::region pseudo-element in the example above would combine the id selector’s specificity with the specificity of the type selector, resulting in a specificity of 101.

Selectors that match a given element or element fragment (as described above), participate in the [CSS Cascading order](css2--cascade.html--c7aff33e6f0d.md#cascading-order) as defined in [\[CSS21\]](#css21).

Region styling does not apply to nested regions. For example, if a region A receives content from a flow that contains region B, the content that flows into B does not receive the region styling specified for region A.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5bff6297"></a> We’ll need some way to query the styles of a fragment in a particular region. `getComputedStyle()` isn’t enough, because an element can exist in multiple regions, for example, with each fragment receiving different styles.

## <a id="conformance"></a> Conformance

### <a id="conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#rfc2119)

Examples in this specification are introduced with the words "for example" or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> This is an example of an informative example.

Informative notes begin with the word "Note" and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

### <a id="conformance-classes"></a> Conformance classes

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

### <a id="partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

### <a id="experimental"></a> Experimental implementations

To avoid clashes with future CSS features, the CSS2.1 specification reserves a [prefixed syntax](https://www.w3.org/TR/CSS21/syndata.html#vendor-keywords) for proprietary and experimental extensions to CSS.

Prior to a specification reaching the Candidate Recommendation stage in the W3C process, all implementations of a CSS feature are considered experimental. The CSS Working Group recommends that implementations use a vendor-prefixed syntax for such features, including those in W3C Working Drafts. This avoids incompatibilities with future changes in the draft.

### <a id="testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](http://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="references"></a> References

### <a id="normative"></a> Normative References

<a id="css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](css2--REC-CSS2-20110607--1e43327015ed.md). 7 June 2011. W3C Recommendation. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2011&#x2F;REC-CSS2-20110607](css2--REC-CSS2-20110607--1e43327015ed.md)

<a id="css3cascade"></a>\[CSS3CASCADE\]  
Håkon Wium Lie; Elika J. Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/2013/CR-css-cascade-3-20131003/). 3 October 2013. W3C Candidate Recommendation. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2013&#x2F;CR-css-cascade-3-20131003&#x2F;](https://www.w3.org/TR/2013/CR-css-cascade-3-20131003/)

<a id="dom"></a>\[DOM\]  
Anne van Kesteren; Aryeh Gregor; Ms2ger. [DOM Living Standard](https://dom.spec.whatwg.org/). WHATWG Living Standard. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](http://www.ietf.org/rfc/rfc2119.txt). URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;ietf&#x2E;org&#x2F;rfc&#x2F;rfc2119&#x2E;txt](http://www.ietf.org/rfc/rfc2119.txt)

### <a id="informative"></a> Informative References

<a id="html"></a>\[HTML\]  
Ian Hickson. [HTML](https://www.whatwg.org/specs/web-apps/current-work/multipage/). Living Standard. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;whatwg&#x2E;org&#x2F;specs&#x2F;web-apps&#x2F;current-work&#x2F;multipage&#x2F;](https://www.whatwg.org/specs/web-apps/current-work/multipage/)

<a id="selectors4"></a>\[SELECTORS4\]  
Elika J. Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/2013/WD-selectors4-20130502/). 2 May 2013. W3C Working Draft. (Work in progress.) URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;2013&#x2F;WD-selectors4-20130502&#x2F;](https://www.w3.org/TR/2013/WD-selectors4-20130502/)

## <a id="index"></a> Index

- active shadow tree, [3](#active-shadow-tree)
- ::content, [3.2.3](#selectordef-content)
- /deep/, [3.2.4](#selectordef-deep)
- distribution list, [3](#distribution-list)
- fragment pseudo-element, [4](#fragment-pseudo-element)
- :host(), [3.2.1](#selectordef-host)
- :host, [3.2.1](#selectordef-host0)
- :host-context(), [3.2.1](#selectordef-host-context)
- host element, [3](#host-element0)
- insertion point, [3](#insertion-point)
- @scope, [2.1.2](#at-ruledef-scope)
- ::shadow, [3.2.2](#selectordef-shadow)
- shadow host, [3](#shadow-host)
- shadow root, [3](#shadow-root)
- shadow tree, [3](#shadow-tree)
- top-level element, [3](#top-level-element)

## <a id="property-index"></a> Property index

No properties defined.

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This rule makes it very easy for authors to create scoped style sheets, which could affect the optimization strategies for implementing scoped styles. [↵](#issue-5f568f10)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If multiple [@scope](#at-ruledef-scope) rules apply to an element, should they be cascaded by specificity? [↵](#issue-5c7571b6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This would be defined similarly to [:host-context()](#selectordef-host-context), but matching the ancestors of the [scoping root](http://dev.w3.org/csswg/selectors-4/#scoping-root) instead.
>
> However, since for scoped stylesheets you may want the ability to match complex selectors against the outside tree, rather than a single compound selector, we may want to instead use a more general mechanism that doesn’t syntactically invert the order of tree elements.
>
> Possible ideas:
>
> ```text
> :scope-context(<selector>) div {...}
> scope(<selector>) div {...}
> \scope <selector>\ div {...}
> <selector> \scope\ div {...}
> ```
>
> This functionality would replace @global, which is a poor excuse for a selector.
>
> [↵](#issue-aecbd194)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is still under discussion. Feedback and advice on intuitive syntax for the following functionality would be appreciated. [↵](#issue-dea392ae)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [::content](#selectordef-content) is a confusingly general name for something that is specific to the projected content of a shadow tree. [↵](#issue-1b6118ae)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is basically a super-descendant combinator. If the descendant combinator had a real glyph, it would potentially be interesting to just double it. Maybe we can give the descendant combinator a pseudonym of \>\>, as it itself is a super-child combinator? Then [/deep/](#selectordef-deep) could be spelled \>\>\> [↵](#issue-250ee37f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Extend this to specify:
>
> - \<region-element-selector\>::region
> - \<paginated-element-selector\>::page(\<page-selector\>)
> - \<multicol-element\>::column(\<AnB\>)
> - \<fragmented-element-selector\>::nth-fragment(\<AnB\>)
> - ::first-line
>
> [↵](#issue-f778a2c0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Either this list should be all functionally inheritable properties, or all properties. Why is it a seemingly-arbitrary subset of all properties, including box properties? [↵](#issue-9b00b7af)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We’ll need some way to query the styles of a fragment in a particular region. `getComputedStyle()` isn’t enough, because an element can exist in multiple regions, for example, with each fragment receiving different styles. [↵](#issue-5bff6297)
