Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Cascading and Inheritance Level 6

Source snapshot: https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/

Snapshot SHA-256: 0cc42c0d3579cacf79c4549bc3302d31a2c847a325693585c21cafd92632263d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Cascading and Inheritance Level 6

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes how to collate style rules and assign values to all properties on all elements. By way of cascading and inheritance, values are propagated for all properties on all elements.

New in this level is [§ 2.5 Scoping Styles: the @scope rule](#scoped-styles).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-cascade” in the title, like this: “\[css-cascade\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-cascade%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction and Missing Sections<a id="filtering"></a><a id="fragments"></a><a id="stages-examples"></a><a id="actual"></a><a id="used"></a><a id="computed"></a><a id="cascaded"></a><a id="declared"></a><a id="specified"></a><a id="value-stages"></a><a id="all-shorthand"></a><a id="aliasing"></a><a id="shorthand"></a><a id="content-type"></a><a id="import-processing"></a><a id="conditional-import"></a><a id="at-import"></a><a id="defaulting"></a><a id="initial-values"></a><a id="inheriting"></a><a id="defaulting-keywords"></a><a id="initial"></a><a id="inherit"></a><a id="inherit-initial"></a><a id="default"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a783701e"></a> This is a diff spec over [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 5 as a reference. We will merge the Level 5 text into this draft once it reaches CR.

## <a id="cascading"></a>2.  Cascading

<a id="ref-for-declared-value"></a>

<a id="ref-for-declaration"></a>

<a id="ref-for-cascaded-value"></a>

The <a id="cascade"></a>cascade takes an unordered list of [declared values](https://www.w3.org/TR/css-cascade-5/#declared-value) for a given property on a given element, sorts them by their [declaration’s](https://www.w3.org/TR/css-syntax-3/#declaration) precedence as determined below, and outputs a single [cascaded value](https://www.w3.org/TR/css-cascade-5/#cascaded-value).

### <a id="cascade-sort"></a>2.1.  Cascade Sorting Order

<a id="ref-for-cssstyledeclaration-declarations"></a>

The cascade sorts [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations) according to the following criteria, in descending order of precedence:

<a id="cascade-origin"></a>Origin and Importance  
<a id="ref-for-important"></a>

<a id="ref-for-declaration①"></a>

<a id="ref-for-origin"></a>

The [origin](#origin) of a [declaration](https://www.w3.org/TR/css-syntax-3/#declaration) is based on where it comes from and its [importance](#important) is whether or not it is declared with !important (see [below](#importance)). The precedence of the various <a id="ref-for-origin①"></a>origins is, in descending order:

1.  Transition declarations [\[css-transitions-1\]](#biblio-css-transitions-1)

2.  <a id="ref-for-cascade-origin-ua"></a>

    <a id="ref-for-important①"></a>

    [Important](#important) [user agent](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua) declarations

3.  <a id="ref-for-cascade-origin-user"></a>

    <a id="ref-for-important②"></a>

    [Important](#important) [user](https://www.w3.org/TR/css-cascade-5/#cascade-origin-user) declarations

4.  <a id="ref-for-cascade-origin-author"></a>

    <a id="ref-for-important③"></a>

    [Important](#important) [author](https://www.w3.org/TR/css-cascade-5/#cascade-origin-author) declarations

5.  Animation declarations [\[css-animations-1\]](#biblio-css-animations-1)

6.  <a id="ref-for-cascade-origin-author①"></a>

    <a id="ref-for-normal"></a>

    [Normal](#normal) [author](https://www.w3.org/TR/css-cascade-5/#cascade-origin-author) declarations

7.  <a id="ref-for-cascade-origin-user①"></a>

    <a id="ref-for-normal①"></a>

    [Normal](#normal) [user](https://www.w3.org/TR/css-cascade-5/#cascade-origin-user) declarations

8.  <a id="ref-for-cascade-origin-ua①"></a>

    <a id="ref-for-normal②"></a>

    [Normal](#normal) [user agent](https://www.w3.org/TR/css-cascade-5/#cascade-origin-ua) declarations

<a id="ref-for-origin②"></a>

Declarations from [origins](#origin) earlier in this list win over declarations from later <a id="ref-for-origin③"></a>origins.

<a id="cascade-context"></a>Context  
<a id="ref-for-concept-shadow-tree"></a>

<a id="ref-for-tree-context"></a>

<a id="ref-for-cssstyledeclaration-declarations①"></a>

A document language can provide for blending [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations) sourced from different <a id="encapsulation-contexts"></a>encapsulation contexts, such as the nested [tree contexts](https://drafts.csswg.org/css-scoping-1/#tree-context) of [shadow trees](https://dom.spec.whatwg.org/#concept-shadow-tree) in the [\[DOM\]](#biblio-dom).

<a id="ref-for-encapsulation-contexts"></a>

<a id="ref-for-normal③"></a>

<a id="ref-for-important④"></a>

<a id="ref-for-tree-context①"></a>

<a id="ref-for-concept-shadow-including-tree-order"></a>

When comparing two declarations that are sourced from different [encapsulation contexts](#encapsulation-contexts), then for [normal](#normal) rules the declaration from the outer context wins, and for [important](#important) rules the declaration from the inner context wins. For this purpose, [\[DOM\]](#biblio-dom) [tree contexts](https://drafts.csswg.org/css-scoping-1/#tree-context) are considered to be nested in [shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order).

<a id="ref-for-normal④"></a>

<a id="ref-for-encapsulation-contexts①"></a>

<a id="ref-for-important⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This effectively means that [normal](#normal) declarations belonging to an [encapsulation context](#encapsulation-contexts) can set defaults that are easily overridden by the outer context, while [important](#important) declarations belonging to an <a id="ref-for-encapsulation-contexts②"></a>encapsulation context can enforce requirements that cannot be overridden by the outer context.

<a id="style-attr"></a>The Style Attribute  
<a id="ref-for-cssstyledeclaration-declarations②"></a>

<a id="ref-for-important⑥"></a>

<a id="ref-for-normal⑤"></a>

Separately for [normal](#normal) and [important](#important) [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations), declarations that are attached directly to an element (such as the [contents of a style attribute](https://www.w3.org/TR/css-style-attr/#interpret)) rather than indirectly mapped by means of a style rule selector take precedence over declarations the same <a id="ref-for-important⑦"></a>importance that are mapped via style rule.

<a id="cascade-layering"></a>Layers  
<a id="ref-for-cascade-layers"></a>

<a id="ref-for-encapsulation-contexts③"></a>

<a id="ref-for-origin④"></a>

<a id="ref-for-cssstyledeclaration-declarations③"></a>

[Declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations) within each [origin](#origin) and [context](#encapsulation-contexts) can be explicitly assigned to a [cascade layer](https://www.w3.org/TR/css-cascade-5/#cascade-layers). For the purpose of this step, any declaration not assigned to an explicit layer is added to an implicit final layer.

<a id="ref-for-normal⑥"></a>

<a id="ref-for-cascade-layers①"></a>

<a id="ref-for-important⑧"></a>

Cascade layers (like declarations) are sorted by order of appearance, see [§ 2.4.1 Layer Ordering](#layer-ordering). When comparing declarations that belong to different layers, then for [normal](#normal) rules the declaration whose [cascade layer](https://www.w3.org/TR/css-cascade-5/#cascade-layers) is latest in the layer order wins, and for [important](#important) rules the declaration whose <a id="ref-for-cascade-layers②"></a>cascade layer is earliest wins.

<a id="ref-for-normal⑦"></a>

<a id="ref-for-important⑨"></a>

<a id="ref-for-origin⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This follows the same logic used for precedence of [normal](#normal) and [important](#important) [origins](#origin), thus the !important flag maintains the same “override” purpose in both settings.

<a id="cascade-specificity"></a>Specificity  
The [Selectors module](https://www.w3.org/TR/selectors/#specificity) [\[SELECT\]](#biblio-select) describes how to compute the specificity of a selector. Each declaration has the same specificity as the style rule it appears in. The declaration with the highest specificity wins.

<a id="cascade-proximity"></a><a id="scope-proximity"></a>Scope Proximity  
<a id="ref-for-selector-subject"></a>

<a id="ref-for-scoped-style-rules"></a>

<a id="ref-for-scoping-root"></a>

When comparing declarations that appear in style rules with different [scoping roots](https://www.w3.org/TR/selectors-4/#scoping-root), then the declaration with the fewest generational or sibling-element hops between the <a id="ref-for-scoping-root①"></a>scoping root and the [scoped style rule](#scoped-style-rules) [subject](https://www.w3.org/TR/selectors-4/#selector-subject) wins. For this purpose, style rules without a <a id="ref-for-scoping-root②"></a>scoping root are considered to have infinite proximity hops.

<a id="cascade-order"></a>Order of Appearance  
The last declaration in document order wins. For this purpose:

- Style sheets are ordered as in [final CSS style sheets](https://drafts.csswg.org/cssom/#documentorshadowroot-final-css-style-sheets).

- <a id="ref-for-at-ruledef-import"></a>

  Declarations from [imported style sheets](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) are ordered as if their style sheets were substituted in place of the <a id="ref-for-at-ruledef-import①"></a>@import rule.

- Declarations from style sheets independently linked by the originating document are treated as if they were concatenated in linking order, as determined by the host document language.

- Declarations from style attributes are ordered according to the document order of the element the style attribute appears on, and are all placed after any style sheets. [\[CSSSTYLEATTR\]](#biblio-cssstyleattr)

<a id="ref-for-declared-value①"></a>

The <a id="output-of-the-cascade"></a>output of the cascade is a (potentially empty) sorted list of [declared values](https://www.w3.org/TR/css-cascade-5/#declared-value) for each property on each element.

### <a id="cascading-origins"></a>2.2.  Cascading Origins

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9"></a> [CSS Cascading 5 § 6.2 Cascading Origins](https://www.w3.org/TR/css-cascade-5/#cascading-origins)

<a id="origin"></a>cascade origin

### <a id="importance"></a>2.3.  Important Declarations: the !important annotation

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①"></a> [CSS Cascading 5 § 6.3 Important Declarations: the !important annotation](https://www.w3.org/TR/css-cascade-5/#importance)

<a id="important"></a>important <a id="normal"></a>normal

### <a id="layering"></a>2.4.  Cascade Layers

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9②"></a> [CSS Cascading 5 § 6.4 Cascade Layers](https://www.w3.org/TR/css-cascade-5/#layering)

#### <a id="layer-ordering"></a>2.4.1.  Layer Ordering

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9③"></a> [CSS Cascading 5 § 6.4.3 Layer Ordering](https://www.w3.org/TR/css-cascade-5/#layer-ordering)

<a id="ref-for-at-ruledef-scope"></a>

### <a id="scoped-styles"></a>2.5.  Scoping Styles: the [@scope](#at-ruledef-scope) rule<a id="scope-atrule"></a>

<a id="ref-for-scope"></a>

A <a id="scope"></a>scope is a subtree or fragment of a document, which can be used by selectors for more targeted matching. A [scope](#scope) is formed by determining:

- <a id="ref-for-scoping-root③"></a>

  <a id="ref-for-boundary-point-node"></a>

  The [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) [node](https://dom.spec.whatwg.org/#boundary-point-node), which acts as the upper bound of the scope, and optionally:

- The <a id="scoping-limit"></a>scoping limit elements, which act as the lower bounds.

An element is <a id="in-scope"></a>in scope if:

- <a id="ref-for-concept-tree-inclusive-descendant"></a>

  <a id="ref-for-scoping-root④"></a>

  It is an [inclusive descendant](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant) of the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root), and

- <a id="ref-for-concept-tree-inclusive-descendant①"></a>

  <a id="ref-for-scoping-limit"></a>

  It is not an [inclusive descendant](https://dom.spec.whatwg.org/#concept-tree-inclusive-descendant) of a [scoping limit](#scoping-limit).

<a id="ref-for-shadow-host"></a>

<a id="ref-for-concept-shadow-tree①"></a>

<a id="ref-for-scope①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In contrast to [Shadow Encapsulation](https://www.w3.org/TR/css-scoping-1/#shadow-dom), which describes a persistent one-to-one relationship in the DOM between a [shadow host](https://www.w3.org/TR/css-scoping-1/#shadow-host) and its nested [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), multiple overlapping [scopes](#scope) can be defined in relation to the same elements.

<a id="ref-for-block-at-rule"></a>

<a id="ref-for-scoping-root⑤"></a>

<a id="ref-for-scoping-limit①"></a>

<a id="ref-for-style-rule"></a>

Scoped styles are described in CSS using the <a id="at-ruledef-scope"></a>@scope [block at-rule](https://www.w3.org/TR/css-syntax-3/#block-at-rule), which declares a [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) and optional [scoping limits](#scoping-limit) associated with a set of [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule).

<a id="ref-for-at-ruledef-scope①"></a>

<a id="ref-for-scoping-root⑥"></a>

<a id="ref-for-scoping-limit②"></a>

<a id="ref-for-in-scope"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f310c4e4"></a> For example, an author might have wide-reaching color-scheme scopes, which overlap more narrowly-scoped design patterns such as a media object. The selectors in the [@scope](#at-ruledef-scope) rule establish [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) and optional [scoping limit](#scoping-limit) elements, while the nested selectors only match elements that are [in a resulting scope](#in-scope):
>
> ```text
> @scope (.light-scheme) {
>   /* Only match links inside a light-scheme */
>   a { color: darkmagenta; }
> }
> 
> @scope (.dark-scheme) {
>   /* Only match links inside a dark-scheme */
>   a { color: plum; }
> }
> 
> @scope (.media-object) {
>   /* Only match author images inside a media-object */
>   .author-image { border-radius: 50%; }
> }
> ```
<a id="ref-for-scoping-limit③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b0418ef2"></a> By providing [scoping limits](#scoping-limit), an author can limit matching more deeply nested descendants. For example:
>
> ```text
> @scope (.media-object) to (.content > *) {
>   img { border-radius: 50%; }
>   .content { padding: 1em; }
> }
> ```
>
> The img selector will only match image tags that are in a DOM fragment starting with any .media-object, and including all descendants up to any intervening children of the .content class.

<a id="ref-for-scoped-selector"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-acd126a1"></a> Should scoping limits be added to the definition of [scoped selectors](https://www.w3.org/TR/selectors-4/#scoped-selector)?

<a id="ref-for-at-ruledef-scope②"></a>

#### <a id="scope-effects"></a>2.5.1.  Effects of [@scope](#at-ruledef-scope)

<a id="ref-for-at-ruledef-scope③"></a>

<a id="ref-for-at-rule"></a>

<a id="ref-for-style-rule①"></a>

The [@scope](#at-ruledef-scope) [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) has three primary effects on the [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) it contains:

- <a id="ref-for-style-rule②"></a>

  <a id="ref-for-at-ruledef-scope④"></a>

  <a id="ref-for-typedef-rule-list"></a>

  <a id="ref-for-scoped-style-rules①"></a>

  The [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) in an [@scope](#at-ruledef-scope) [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list) are [scoped style rules](#scoped-style-rules).

- <a id="ref-for-scope-pseudo"></a>

  <a id="ref-for-at-ruledef-scope⑤"></a>

  <a id="ref-for-scoping-root⑦"></a>

  <a id="ref-for-featureless"></a>

  <a id="ref-for-shadow-host①"></a>

  <a id="ref-for-selectordef-"></a>

  <a id="ref-for-typedef-scope-start"></a>

  The [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) selector is defined to match the [@scope](#at-ruledef-scope) rule’s [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root), including the [featureless](https://www.w3.org/TR/selectors-4/#featureless) [shadow host](https://www.w3.org/TR/css-scoping-1/#shadow-host) when that host is the <a id="ref-for-scoping-root⑧"></a>scoping root. The [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selector is defined to represent the selector representing the <a id="ref-for-scoping-root⑨"></a>scoping root (the [\<scope-start\>](#typedef-scope-start) selector), or else <a id="ref-for-scope-pseudo①"></a>:scope if no selector was specified.

- <a id="ref-for-cascade"></a>

  <a id="ref-for-scope-proximity"></a>

  <a id="ref-for-scoping-root①⓪"></a>

  <a id="ref-for-selector-subject①"></a>

  <a id="ref-for-scoped-style-rules②"></a>

  The [cascade](#cascade) prioritizes declarations with a [more proximate](#scope-proximity) [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root), regardless of specificity or order of appearance by applying <a id="ref-for-scope-proximity①"></a>scope proximity between the <a id="ref-for-scoping-root①①"></a>scoping root and the [subject](https://www.w3.org/TR/selectors-4/#selector-subject) of each [scoped style rule](#scoped-style-rules).

<a id="ref-for-at-ruledef-scope⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike [Nesting](https://www.w3.org/TR/css-nesting/), selectors within an [@scope](#at-ruledef-scope) rule do not acquire the specificity of any parent selector(s) in the <a id="ref-for-at-ruledef-scope⑦"></a>@scope prelude.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-47f85a12"></a> The following selectors have the same specificity (0,0,1):
>
> ```text
> @scope (#hero) {
>   img { border-radius: 50%; }
> }
> 
> :where(#hero) img { border-radius: 50%; }
> ```
>
> <a id="ref-for-the-img-element"></a>
>
> <a id="ref-for-scope-proximity②"></a>
>
> The additional specificity of the \#hero selector is not applied to the specificity of the scoped selector. However, since one <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> selector is scoped, that selector is weighted more strongly in the cascade with the application of [scope proximity](#scope-proximity).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-463550a5"></a> Many existing tools implement "scoped styles" by applying a unique class or attribute to every element in a given scope or "single file component." In this example there are two scopes (`main-component` and `sub-component`) and every element is marked as part of one or both scopes using the `data-scope` attribute:
>
> ```html
> <section data-scope="main-component">
>   <p data-scope="main-component">...<p>
> 
>   <!-- sub-component root is in both scopes -->
>   <section data-scope="main-component sub-component">
>     <!-- children are only in the inner scope -->
>     <p data-scope="sub-component">...<p>
>   </section>
> </section>
> ```
>
> Those custom scope attributes are then appended to every single selector in CSS:
>
> ```text
> p[data-scope~='main-component'] { color: red; }
> p[data-scope~='sub-component'] { color: blue; }
> 
> /* both sections are part of the outer scope */
> section[data-scope~='main-component'] { background: snow; }
> 
> /* the inner section is also part of the inner scope */
> section[data-scope~='sub-component'] { color: ghostwhite; }
> ```
>
> <a id="ref-for-at-ruledef-scope⑧"></a>
>
> <a id="ref-for-scoping-root①②"></a>
>
> Using the [@scope](#at-ruledef-scope) rule, authors and tools can replicate similar behavior with the unique attribute or class applied only to the [scoping roots](https://www.w3.org/TR/selectors-4/#scoping-root):
>
> ```html
> <section data-scope="main-component">
>   <p>...<p>
>   <section data-scope="sub-component">
>     <p>...<p>
>   </section>
> </section>
> ```
>
> Then the class or attribute can be used for establishing both upper and lower boundaries. Elements matched by a lower boundary selector are excluded from the resulting scope, which allows authors to create non-overlapping scopes by default:
>
> ```text
> @scope ([data-scope='main-component']) to ([data-scope]) {
>   p { color: red; }
> 
>   /* only the outer section is part of the outer scope */
>   section { background: snow; }
> }
> 
> @scope ([data-scope='sub-component']) to ([data-scope]) {
>   p { color: blue; }
> 
>   /* the inner section is only part of the inner scope */
>   section { color: ghostwhite; }
> }
> ```
>
> However, authors can use the child combinator and universal selector to create scope boundaries that overlap, such that the inner scope root is part of both scopes:
>
> ```text
> @scope ([data-scope='main-component']) to ([data-scope] > *) {
>   p { color: red; }
> 
>   /* both sections are part of the outer scope */
>   section { background: snow; }
> }
> ```
<a id="ref-for-at-ruledef-scope⑨"></a>

#### <a id="scope-syntax"></a>2.5.2.  Syntax of [@scope](#at-ruledef-scope)

<a id="ref-for-at-ruledef-scope①⓪"></a>

The syntax of the [@scope](#at-ruledef-scope) rule is:

<a id="ref-for-typedef-scope-start①"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-scope-end"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-rule-list①"></a>

```text
@scope [(<scope-start>)]? [to (<scope-end>)]? {
  <rule-list>
}
```
where:

- <a id="ref-for-typedef-scope-start②"></a>

  <a id="ref-for-typedef-selector-list"></a>

  <a id="ref-for-x15"></a>

  <a id="ref-for-scoping-root①③"></a>

  <a id="typedef-scope-start"></a>[\<scope-start\>](#typedef-scope-start) is a [\<selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-selector-list) [selector](https://www.w3.org/TR/CSS21/syndata.html#x15) used to identify the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root)(s).

- <a id="ref-for-typedef-scope-end①"></a>

  <a id="ref-for-typedef-selector-list①"></a>

  <a id="ref-for-x15①"></a>

  <a id="ref-for-scoping-limit④"></a>

  <a id="typedef-scope-end"></a>[\<scope-end\>](#typedef-scope-end) is a [\<selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-selector-list) [selector](https://www.w3.org/TR/CSS21/syndata.html#x15) used to identify any [scoping limits](#scoping-limit).

- <a id="ref-for-typedef-rule-list②"></a>

  <a id="ref-for-scoped-style-rules③"></a>

  the [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list) represents the [scoped style rules](#scoped-style-rules).

<a id="ref-for-x22"></a>

<a id="ref-for-scoping-root①④"></a>

<a id="ref-for-scoping-limit⑤"></a>

<a id="ref-for-typedef-scope-start③"></a>

<a id="ref-for-typedef-scope-end②"></a>

[Pseudo-elements](https://www.w3.org/TR/CSS21/selector.html#x22) cannot be [scoping roots](https://www.w3.org/TR/selectors-4/#scoping-root) or [scoping limits](#scoping-limit); they are invalid both within [\<scope-start\>](#typedef-scope-start) and [\<scope-end\>](#typedef-scope-end).

#### <a id="scoped-rules"></a>2.5.3.  Scoped Style Rules

<a id="scoped-style-rules"></a>Scoped style rules differ from non-scoped rules in the following ways:

- <a id="ref-for-in-scope①"></a>

  <a id="ref-for-selector-subject②"></a>

  Their selectors can only match elements that are [in scope](#in-scope). (This only applies to the [subject](https://www.w3.org/TR/selectors-4/#selector-subject); the rest of the selector can match unrestricted.)

- <a id="ref-for-typedef-relative-selector-list"></a>

  <a id="ref-for-typedef-selector-list②"></a>

  <a id="ref-for-relative-selector"></a>

  <a id="ref-for-scope-pseudo②"></a>

  They accept a [\<relative-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-relative-selector-list) as their prelude (rather than just a [\<selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-selector-list)). Such [relative selectors](https://www.w3.org/TR/selectors-4/#relative-selector) are relative to [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo).

- <a id="ref-for-typedef-relative-selector-list①"></a>

  <a id="ref-for-combinator"></a>

  <a id="ref-for-contain-the-nesting-selector"></a>

  <a id="ref-for-scope-pseudo③"></a>

  <a id="ref-for-relative-selector①"></a>

  <a id="ref-for-selector-subject③"></a>

  <a id="ref-for-in-scope②"></a>

  Any selector in the [\<relative-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-relative-selector-list) that does not start with a [combinator](https://www.w3.org/TR/CSS21/selector.html#combinator) but does [contain the nesting selector](https://drafts.csswg.org/css-nesting-1/#contain-the-nesting-selector) or the [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) selector, is interpreted as a non-[relative selector](https://www.w3.org/TR/selectors-4/#relative-selector) (but the [subject](https://www.w3.org/TR/selectors-4/#selector-subject) must still be [in scope](#in-scope) to match).

<a id="ref-for-scoped-style-rules④"></a>

<a id="ref-for-relative-selector②"></a>

<a id="ref-for-scoping-root①⑤"></a>

<a id="ref-for-descendant-combinator"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-66991251"></a> By default, selectors in a [scoped style rule](#scoped-style-rules) are [relative selectors](https://www.w3.org/TR/selectors-4/#relative-selector), with the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) and [descendant combinator](https://www.w3.org/TR/selectors-4/#descendant-combinator) implied at the start. The following selectors will match the same elements:
>
> ```text
> @scope (#my-component) {
>   p { color: green; }
>   :scope p { color: green; }
> }
> ```
>
> Authors can adjust the implied relationship by adding an explicit combinator:
>
> ```text
> @scope (#my-component) {
>   > p { color: green; }
>   :scope > p { color: green; }
> }
> ```
>
> <a id="ref-for-scoping-root①⑥"></a>
>
> <a id="ref-for-scope-pseudo④"></a>
>
> <a id="ref-for-selectordef-①"></a>
>
> Authors can also target or explicitly position the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) in a selector by including either [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) or [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) in a given selector:
>
> ```text
> @scope (#my-component) {
>   :scope { border: thin solid; }
>   & { border: thin solid; }
> 
>   main :scope p { color: green; }
>   main & p { color: green; }
> }
> ```
>
> <a id="ref-for-scope-pseudo⑤"></a>
>
> <a id="ref-for-selectordef-②"></a>
>
> <a id="ref-for-scoping-root①⑦"></a>
>
> While the [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) or [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selectors can both refer to the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root), they have otherwise different meanings in this context:
>
> Differences in selector matching  
> <a id="ref-for-scope-pseudo⑥"></a>
>
> <a id="ref-for-scoping-root①⑧"></a>
>
> <a id="ref-for-selectordef-③"></a>
>
> <a id="ref-for-typedef-scope-start④"></a>
>
> The [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) selector will only match the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) itself, while the [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selector is able to match any element that is matched by the [\<scope-start\>](#typedef-scope-start) selector list.
>
> Differences in selector specificity  
> <a id="ref-for-scope-pseudo⑦"></a>
>
> <a id="ref-for-selectordef-④"></a>
>
> <a id="ref-for-typedef-scope-start⑤"></a>
>
> The [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) selector has a specificity equal to other pseudo-classes, while the [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selector has the specificity equal to the most specific selector in [\<scope-start\>](#typedef-scope-start).

#### <a id="scope-limits"></a>2.5.4.  Identifying Scoping Roots and Limits

<a id="ref-for-at-ruledef-scope①①"></a>

<a id="ref-for-scope②"></a>

A [@scope](#at-ruledef-scope) rule produces one or more [scopes](#scope) as follows:

<a id="ref-for-scoping-root①⑨"></a>

Finding the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root)(s)

<a id="ref-for-typedef-scope-start⑥"></a>

<a id="ref-for-scope③"></a>

<a id="ref-for-scoping-root②⓪"></a>

<a id="ref-for-parent-element"></a>

<a id="ref-for-cssstyledeclaration-owner-node"></a>

<a id="ref-for-at-ruledef-scope①②"></a>

<a id="ref-for-concept-node-tree"></a>

<a id="ref-for-concept-shadow-tree②"></a>

<a id="ref-for-shadow-host②"></a>

<a id="ref-for-concept-tree-root"></a>

<a id="ref-for-scope-pseudo⑧"></a>

<a id="ref-for-selectordef-⑤"></a>

For each element matched by [\<scope-start\>](#typedef-scope-start), create a [scope](#scope) using that element as the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root). If no <a id="ref-for-typedef-scope-start⑦"></a>\<scope-start\> is specified, the <a id="ref-for-scoping-root②①"></a>scoping root is the [parent element](https://dom.spec.whatwg.org/#parent-element) of the [owner node](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-owner-node) of the stylesheet where the [@scope](#at-ruledef-scope) rule is defined. (If no such element exists and the containing [node tree](https://dom.spec.whatwg.org/#concept-node-tree) is a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree), then the <a id="ref-for-scoping-root②②"></a>scoping root is the [shadow host](https://www.w3.org/TR/css-scoping-1/#shadow-host). Otherwise, the <a id="ref-for-scoping-root②③"></a>scoping root is the [root](https://dom.spec.whatwg.org/#concept-tree-root) of the containing <a id="ref-for-concept-node-tree①"></a>node tree.) Any [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) or [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selectors in <a id="ref-for-typedef-scope-start⑧"></a>\<scope-start\> are interpreted as defined for its outer context.

<a id="ref-for-scoping-limit⑥"></a>

Finding any [scoping limits](#scoping-limit)

<a id="ref-for-scope④"></a>

<a id="ref-for-scoping-root②④"></a>

<a id="ref-for-scoping-limit⑦"></a>

<a id="ref-for-in-scope③"></a>

<a id="ref-for-typedef-scope-end③"></a>

<a id="ref-for-scope-pseudo⑨"></a>

<a id="ref-for-selectordef-⑥"></a>

<a id="ref-for-scoped-style-rules⑤"></a>

For each [scope](#scope) created by a [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root), its [scoping limits](#scoping-limit) are set to all elements that are [in scope](#in-scope) and that match [\<scope-end\>](#typedef-scope-end), interpreting [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) and [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) exactly as in [scoped style rules](#scoped-style-rules).

<a id="ref-for-the-style-element"></a>

<a id="ref-for-typedef-scope-start⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-52419898"></a> Authors can establish local scoping for <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-style-element">style</a></code> elements by leaving out the [\<scope-start\>](#typedef-scope-start) selector. For example:
>
> ```html
> <div>
>   <style>
>     @scope {
>       p { color: red; }
>     }
>   </style>
>   <p>this is red</p>
> </div>
> <p>not red</p>
> ```
>
> That would be equivalent to:
>
> ```html
> <div id="foo">
>   <style>
>     @scope (#foo) {
>       p { color: red; }
>     }
>   </style>
>   <p>this is red</p>
> </div>
> <p>not red</p>
> ```
<a id="ref-for-scoping-limit⑧"></a>

<a id="ref-for-scope-pseudo①⓪"></a>

<a id="ref-for-scoping-root②⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-132cb7e9"></a> [Scoping limits](#scoping-limit) can use the [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) pseudo-class to require a specific relationship to the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root):
>
> ```text
> /* .content is only a limit when it is a direct child of the :scope */
> @scope (.media-object) to (:scope > .content) { ... }
> ```
>
> <a id="ref-for-scoping-limit⑨"></a>
>
> <a id="ref-for-scoping-root②⑥"></a>
>
> <a id="ref-for-scope-pseudo①①"></a>
>
> [Scoping limits](#scoping-limit) can also reference elements outside their [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) by using [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo). For example:
>
> ```text
> /* .content is only a limit when the :scope is inside .sidebar */
> @scope (.media-object) to (.sidebar :scope .content) { ... }
> ```
#### <a id="scope-scope"></a>2.5.5.  Scope Nesting

<a id="ref-for-at-ruledef-scope①③"></a>

<a id="ref-for-scope⑤"></a>

<a id="ref-for-scoped-selector①"></a>

[@scope](#at-ruledef-scope) rules can be nested. In this case, just as with the nested style rules, the prelude selectors of the inner <a id="ref-for-at-ruledef-scope①④"></a>@scope (those defining its [scope](#scope)) are [scoped by](https://www.w3.org/TR/selectors-4/#scoped-selector) the selectors of the outer one.

<a id="ref-for-scope⑥"></a>

<a id="ref-for-scoped-style-rules⑥"></a>

<a id="ref-for-at-ruledef-scope①⑤"></a>

<a id="ref-for-scoping-root②⑦"></a>

<a id="ref-for-scope-proximity③"></a>

<a id="ref-for-selector-subject④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The resulting [scope](#scope) for further nested [scoped style rules](#scoped-style-rules) is practically constrained by both the outer and inner [@scope](#at-ruledef-scope) rules, but the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) is defined by the innermost <a id="ref-for-at-ruledef-scope①⑥"></a>@scope. Since [scope proximity](#scope-proximity) is measured between a <a id="ref-for-scoped-style-rules⑦"></a>scoped style rule [subject](https://www.w3.org/TR/selectors-4/#selector-subject) and <a id="ref-for-scoping-root②⑧"></a>scoping root, only the innermost <a id="ref-for-at-ruledef-scope①⑦"></a>@scope matters for determining <a id="ref-for-scope-proximity④"></a>scope proximity of [nested @scope rules](#scope-scope).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a2203bb7"></a> Should the scope proximity calculation be impacted by nesting scopes? [\[Issue \#10795\]](https://github.com/w3c/csswg-drafts/issues/10795)

<a id="ref-for-at-ruledef-scope①⑧"></a>

<a id="ref-for-typedef-scope-start①⓪"></a>

<a id="ref-for-relative-selector③"></a>

<a id="ref-for-typedef-scope-end④"></a>

<a id="ref-for-scoped-style-rules⑧"></a>

<a id="ref-for-scoping-root②⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-49f2905a"></a> When nesting [@scope](#at-ruledef-scope) rules inside other <a id="ref-for-at-ruledef-scope①⑨"></a>@scope rules, or inside other selectors, the [\<scope-start\>](#typedef-scope-start) selector is [relative to](https://www.w3.org/TR/selectors-4/#relative-selector) the nesting context, while the [\<scope-end\>](#typedef-scope-end) and any [scoped style rules](#scoped-style-rules) are <a id="ref-for-relative-selector④"></a>relative to the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) For example, the following code:
>
> ```text
> @scope (.parent-scope) {
>   @scope (:scope > .child-scope) to (:scope .limit) {
>     :scope .content {
>       color: red;
>     }
>   }
> }
> ```
>
> is equivalent to:
>
> ```text
> @scope (.parent-scope > .child-scope) to (.parent-scope > .child-scope .limit) {
>   .parent-scope > .child-scope .content {
>     color: red;
>   }
> }
> ```
<a id="ref-for-at-rule①"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-at-font-face-rule"></a>

<a id="ref-for-at-ruledef-layer"></a>

<a id="ref-for-at-ruledef-scope②⓪"></a>

<a id="ref-for-style-rule③"></a>

<a id="ref-for-scoped-style-rules⑨"></a>

Global name-defining [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) such as [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) or [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) or [@layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer) that are defined inside [@scope](#at-ruledef-scope) are valid, but are not scoped or otherwise affected by the enclosing <a id="ref-for-at-ruledef-scope②①"></a>@scope rule. However, any [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) contained by such rules (e.g. within <a id="ref-for-at-ruledef-layer①"></a>@layer) are [scoped](#scoped-style-rules).

### <a id="preshint"></a>2.6.  Precedence of Non-CSS Presentational Hints

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9④"></a> [CSS Cascading 5 § 6.4 Cascade Layers](https://www.w3.org/TR/css-cascade-5/#layering)

## <a id="cssom"></a>3. CSSOM

### <a id="the-cssscoperule-interface"></a>3.1.  The `CSSScopeRule` interface

<a id="ref-for-cssscoperule"></a>

<a id="ref-for-at-ruledef-scope②②"></a>

The <code><a href="#cssscoperule">CSSScopeRule</a></code> interface represents the [@scope](#at-ruledef-scope) rule:

<a id="ref-for-Exposed"></a>

<a id="cssscoperule"></a>

<a id="ref-for-cssgroupingrule"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-cssscoperule-start"></a>

<a id="ref-for-cssomstring①"></a>

<a id="dom-cssscoperule-end"></a>

```text
[Exposed=Window]
interface CSSScopeRule : CSSGroupingRule {
  readonly attribute CSSOMString? start;
  readonly attribute CSSOMString? end;
};
```
`start` of type `CSSOMString`  
<a id="ref-for-typedef-scope-start①①"></a>

The `start` attribute returns the result of serializing the [\<scope-start\>](#typedef-scope-start) of the rule (without the enclosing parentheses), or null if there is no <a id="ref-for-typedef-scope-start①②"></a>\<scope-start\>.

`end` of type `CSSOMString`  
<a id="ref-for-typedef-scope-end⑤"></a>

The `end` attribute returns the result of serializing the [\<scope-end\>](#typedef-scope-end) of the rule (without the enclosing parentheses), or null if there is no <a id="ref-for-typedef-scope-end⑥"></a>\<scope-end\>.

## <a id="changes"></a>4.  Changes

This appendix is <em>informative</em>.

### <a id="changes-since-2023-03"></a>4.1.  Changes since the 21 March 2023 Working Draft

Significant changes since the [21 March 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-cascade-6-20230321/) include:

- <a id="ref-for-scope-pseudo①②"></a>

  <a id="ref-for-featureless①"></a>

  <a id="ref-for-shadow-host③"></a>

  <a id="ref-for-scoping-root③⓪"></a>

  The [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) selector can match the [featureless](https://www.w3.org/TR/selectors-4/#featureless) [shadow host](https://www.w3.org/TR/css-scoping-1/#shadow-host) when that host is the [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) element. ([Issue 9025](https://github.com/w3c/csswg-drafts/issues/9025))

- \<scope-start\> and \<scope-end\> selectors are unforgiving. ([Issue 10042](https://github.com/w3c/csswg-drafts/issues/10042))

- <a id="ref-for-at-ruledef-scope②③"></a>

  <a id="ref-for-shadow-host④"></a>

  <a id="ref-for-concept-shadow-root"></a>

  A [@scope](#at-ruledef-scope) rule without \<scope-start\> scopes to the [shadow host](https://www.w3.org/TR/css-scoping-1/#shadow-host) instead of the [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root). ([Issue 9178](https://github.com/w3c/csswg-drafts/issues/9178))

- <a id="ref-for-scope-proximity⑤"></a>

  <a id="ref-for-scoping-root③①"></a>

  <a id="ref-for-scoped-style-rules①⓪"></a>

  <a id="ref-for-selector-subject⑤"></a>

  Clarified that [scope proximity](#scope-proximity) is a single measurement of the steps between a single [scoping root](https://www.w3.org/TR/selectors-4/#scoping-root) and [scoped style rule](#scoped-style-rules) [subject](https://www.w3.org/TR/selectors-4/#selector-subject) ([Issue 10795](https://github.com/w3c/csswg-drafts/issues/10795) has been opened to discuss this futher).

- Removed strong scope proximity. ([Issue 6790](https://github.com/w3c/csswg-drafts/issues/6790))

- Removed the scoped descendant combinator (deferred). ([Issue 8628](https://github.com/w3c/csswg-drafts/issues/8628))

- <a id="ref-for-cssscoperule①"></a>

  Added the <code><a href="#cssscoperule">CSSScopeRule</a></code> interface. ([Issue 8626](https://github.com/w3c/csswg-drafts/issues/8626))

### <a id="changes-2022-08"></a>4.2.  Changes since the 21 December 2021 First Public Working Draft

Significant changes since the [21 December 2021 First Public Working Draft](https://www.w3.org/TR/2021/WD-css-cascade-6-20211221/) include:

- <a id="ref-for-at-ruledef-scope②④"></a>

  <a id="ref-for-scope-pseudo①③"></a>

  <a id="ref-for-selectordef-⑦"></a>

  Clarified [@scope](#at-ruledef-scope) effects on nested [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) and [&#x26;](https://www.w3.org/TR/css-nesting-1/#selectordef-) selectors. ([Issue 8377](https://github.com/w3c/csswg-drafts/issues/8377))

- <a id="ref-for-at-ruledef-scope②⑤"></a>

  Removed [@scope](#at-ruledef-scope) prelude from specificity calculation. ([Issue 8500](https://github.com/w3c/csswg-drafts/issues/8500))

- <a id="ref-for-at-rule②"></a>

  <a id="ref-for-at-ruledef-scope②⑥"></a>

  Specified how name-defining [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) behave in [@scope](#at-ruledef-scope). ([Issue 6895](https://github.com/w3c/csswg-drafts/issues/6895))

- Added implicit scopes by making \<scope-start\> optional. ([Issue 6606](https://github.com/w3c/csswg-drafts/issues/6606))

- <a id="ref-for-x22①"></a>

  <a id="ref-for-at-ruledef-scope②⑦"></a>

  Disallowed [pseudo-elements](https://www.w3.org/TR/CSS21/selector.html#x22) in the [@scope](#at-ruledef-scope) prelude. ([Issue 7382](https://github.com/w3c/csswg-drafts/issues/7382))

- Removed selector scoping notation. ([Issue 7709](https://github.com/w3c/csswg-drafts/issues/7709))

- <a id="ref-for-scoping-limit①⓪"></a>

  <a id="ref-for-scope⑦"></a>

  [Scoping limit](#scoping-limit) elements are excluded from the resulting [scope](#scope). ([Issue 6577](https://github.com/w3c/csswg-drafts/issues/6577))

### <a id="additions-l5"></a>4.3.  Additions Since Level 5

The following features have been added since [Level 5](https://www.w3.org/TR/css-cascade-5/):

- <a id="ref-for-scope⑧"></a>

  <a id="ref-for-typedef-scope-start①③"></a>

  <a id="ref-for-typedef-scope-end⑦"></a>

  The definition of a [scope](#scope), as described by a combination of [\<scope-start\>](#typedef-scope-start) and [\<scope-end\>](#typedef-scope-end) selectors.

- The in-scope (:in()) pseudo-class for selecting with lower-boundaries

- <a id="ref-for-at-ruledef-scope②⑧"></a>

  The [@scope](#at-ruledef-scope) rule for creating scoped stylesheets

- <a id="ref-for-scope-proximity⑥"></a>

  The definition of [scope proximity](#scope-proximity) in the cascade

### <a id="additions-l4"></a>4.4.  Additions Since Level 4

The following features have been added since [Level 4](https://www.w3.org/TR/css-cascade-4/):

- <a id="ref-for-cascade-layers③"></a>

  <a id="ref-for-cascade①"></a>

  Added [cascade layers](https://www.w3.org/TR/css-cascade-5/#cascade-layers) to the [cascade](#cascade) sort criteria (and defined style attributes as a distinct step of the <a id="ref-for-cascade②"></a>cascade sort criteria so that they interact appropriately).

- <a id="ref-for-at-ruledef-layer②"></a>

  Introduced the [@layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer) rule for defining cascade layers.

- <a id="ref-for-at-ruledef-import②"></a>

  Added layer/layer() option to [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) definition.

- <a id="ref-for-valdef-all-revert-layer"></a>

  Introduced the [revert-layer](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert-layer) keyword for rolling back values to previous layers.

### <a id="additions-l3"></a>4.5.  Additions Since Level 3

The following features have been added since [Level 3](https://www.w3.org/TR/css-cascade-3/):

- <a id="ref-for-valdef-all-revert"></a>

  Introduced [revert](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert) keyword, for rolling back the cascade.

- <a id="ref-for-funcdef-supports"></a>

  <a id="ref-for-at-ruledef-import③"></a>

  Introduced [supports()](https://www.w3.org/TR/css-conditional-5/#funcdef-supports) syntax for supports-conditional [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) rules.

- <a id="ref-for-encapsulation-contexts④"></a>

  <a id="ref-for-cascade③"></a>

  Added [encapsulation context](#encapsulation-contexts) to the [cascade](#cascade) sort criteria to accommodate Shadow DOM. [\[DOM\]](#biblio-dom)

- Defined the property two aliasing mechanisms CSS uses to support legacy syntaxes. See [CSS Cascading 4 § 3.1 Property Aliasing](https://www.w3.org/TR/css-cascade-4/#aliasing).

### <a id="changes-2"></a>4.6.  Additions Since Level 2

The following features have been added since [Level 2](https://www.w3.org/TR/CSS2/cascade.html):

- <a id="ref-for-propdef-all"></a>

  The [all](https://www.w3.org/TR/css-cascade-5/#propdef-all) shorthand

- <a id="ref-for-valdef-all-initial"></a>

  The [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) keyword

- <a id="ref-for-valdef-all-unset"></a>

  The [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset) keyword

- <a id="ref-for-cascade④"></a>

  Incorporation of animations and transitions into the [cascade](#cascade).

## <a id="acknowledgments"></a>Acknowledgments

David Baron, Tantek Çelik, Keith Grant, Giuseppe Gurgone, Theresa O’Connor, Florian Rivoal, Noam Rosenthal, Simon Sapin, Jen Simmons, Nicole Sullivan, Lea Verou, and Boris Zbarsky contributed to this specification.

## <a id="privacy"></a>5.  Privacy Considerations

- User preferences and UA defaults expressed via application of style rules are exposed by the cascade process, and can be inferred from the computed styles they apply to a document.

## <a id="security"></a>6.  Security Considerations

- The cascade process does not distinguish between same-origin and cross-origin stylesheets, enabling the content of cross-origin stylesheets to be inferred from the computed styles they apply to a document.

- <a id="ref-for-at-ruledef-import④"></a>

  <a id="ref-for-cors-protocol"></a>

  The [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) rule does not apply the [CORS protocol](https://fetch.spec.whatwg.org/#cors-protocol) to loading cross-origin stylesheets, instead allowing them to be freely imported and applied.

- <a id="ref-for-at-ruledef-import⑤"></a>

  The [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) rule assumes that resources without [`Content-Type` metadata](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#content-type) (or any same-origin file if the host document is in quirks mode) are `text/css`, potentially allowing arbitrary files to be imported into the page and interpreted as CSS, potentially allowing sensitive data to be inferred from the computed styles they apply to a document.

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

- [cascade](#cascade), in § 2
- [cascade origin](#origin), in § 2.2
- [context](#encapsulation-contexts), in § 2.1
- [CSSScopeRule](#cssscoperule), in § 3.1
- [encapsulation contexts](#encapsulation-contexts), in § 2.1
- [end](#dom-cssscoperule-end), in § 3.1
- [importance](#important), in § 2.3
- [important](#important), in § 2.3
- [in scope](#in-scope), in § 2.5
- [normal](#normal), in § 2.3
- [origin](#origin), in § 2.2
- [output of the cascade](#output-of-the-cascade), in § 2.1
- [@scope](#at-ruledef-scope), in § 2.5
- [scope](#scope), in § 2.5
- [Scoped style rules](#scoped-style-rules), in § 2.5.3
- [\<scope-end\>](#typedef-scope-end), in § 2.5.2
- [Scope Proximity](#scope-proximity), in § 2.1
- [\<scope-start\>](#typedef-scope-start), in § 2.5.2
- [scoping limit](#scoping-limit), in § 2.5
- [start](#dom-cssscoperule-start), in § 3.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="7177b17d"></a>@keyframes
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
  - <a id="08b3934a"></a>@layer
  - <a id="d3b48763"></a>all
  - <a id="515ba43c"></a>author origin
  - <a id="5b7444b2"></a>cascade layers
  - <a id="9b9f041e"></a>cascaded value
  - <a id="92922499"></a>declared value
  - <a id="762bad34"></a>initial
  - <a id="b45bd8fa"></a>revert
  - <a id="529d0525"></a>revert-layer
  - <a id="7c39b465"></a>unset
  - <a id="e308a45f"></a>user origin
  - <a id="a9479ee7"></a>user-agent origin
- \[CSS-CONDITIONAL-5\] defines the following terms:
  - <a id="a43be3f5"></a>supports()
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="b24ce65e"></a>@font-face
- \[CSS-NESTING-1\] defines the following terms:
  - <a id="97abc909"></a>&#x26;
  - <a id="5140976e"></a>contain the nesting selector
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="ce2e3c84"></a>shadow host
  - <a id="cfd2c803"></a>tree context
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="a8cb81d7"></a>\<rule-list\>
  - <a id="b29fecf5"></a>at-rule
  - <a id="1157d267"></a>block at-rule
  - <a id="e2112c66"></a>declaration
  - <a id="87d90aed"></a>style rule
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="569c7f9f"></a>?
- \[CSS21\] defines the following terms:
  - <a id="615e2aa3"></a>combinator
  - <a id="2161cf2b"></a>pseudo-elements
  - <a id="7c931380"></a>selector
- \[CSSOM-1\] defines the following terms:
  - <a id="697c30aa"></a>CSSGroupingRule
  - <a id="9d357000"></a>CSSOMString
  - <a id="9852f862"></a>declarations
  - <a id="451490eb"></a>owner node
- \[DOM\] defines the following terms:
  - <a id="f9d909f7"></a>inclusive descendant
  - <a id="d462b34f"></a>node
  - <a id="c7d8d91b"></a>node tree
  - <a id="5afeceea"></a>parent element
  - <a id="f7960529"></a>root
  - <a id="3fcc582f"></a>shadow root
  - <a id="19f9e9df"></a>shadow tree
  - <a id="fefa5851"></a>shadow-including tree order
- \[FETCH\] defines the following terms:
  - <a id="a2a30ba8"></a>cors protocol
- \[HTML\] defines the following terms:
  - <a id="f0811ff8"></a>img
  - <a id="ba920583"></a>style
- \[SELECTORS-4\] defines the following terms:
  - <a id="9fca4587"></a>:scope
  - <a id="0af7e6f0"></a>\<relative-selector-list\>
  - <a id="1fd14124"></a>\<selector-list\>
  - <a id="003a1eca"></a>descendant combinator
  - <a id="09a6f82f"></a>featureless
  - <a id="09bedad5"></a>relative selector
  - <a id="dc4e5911"></a>scoped selector
  - <a id="f177dc2c"></a>scoping root
  - <a id="aa4e3328"></a>subject
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/css-conditional-5/). 23 July 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-5&#x2F;](https://www.w3.org/TR/css-conditional-5/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 6 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-nesting-1"></a>\[CSS-NESTING-1\]  
Tab Atkins Jr.; Adam Argyle. [CSS Nesting Module](https://www.w3.org/TR/css-nesting-1/). 14 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-nesting-1&#x2F;](https://www.w3.org/TR/css-nesting-1/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-cssstyleattr"></a>\[CSSSTYLEATTR\]  
Tantek Çelik; Elika Etemad. [CSS Style Attributes](https://www.w3.org/TR/css-style-attr/). 7 November 2013. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-style-attr&#x2F;](https://www.w3.org/TR/css-style-attr/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-select"></a>\[SELECT\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface CSSScopeRule : CSSGroupingRule {
  readonly attribute CSSOMString? start;
  readonly attribute CSSOMString? end;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is a diff spec over [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 5 as a reference. We will merge the Level 5 text into this draft once it reaches CR. [↵](#issue-a783701e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Cascading 5 § 6.2 Cascading Origins](https://www.w3.org/TR/css-cascade-5/#cascading-origins) [↵](#issue-d41d8cd9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Cascading 5 § 6.3 Important Declarations: the !important annotation](https://www.w3.org/TR/css-cascade-5/#importance) [↵](#issue-d41d8cd9%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Cascading 5 § 6.4 Cascade Layers](https://www.w3.org/TR/css-cascade-5/#layering) [↵](#issue-d41d8cd9%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Cascading 5 § 6.4.3 Layer Ordering](https://www.w3.org/TR/css-cascade-5/#layer-ordering) [↵](#issue-d41d8cd9%E2%91%A2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should scoping limits be added to the definition of [scoped selectors](https://www.w3.org/TR/selectors-4/#scoped-selector)? [↵](#issue-acd126a1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should the scope proximity calculation be impacted by nesting scopes? [\[Issue \#10795\]](https://github.com/w3c/csswg-drafts/issues/10795) [↵](#issue-a2203bb7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Cascading 5 § 6.4 Cascade Layers](https://www.w3.org/TR/css-cascade-5/#layering) [↵](#issue-d41d8cd9%E2%91%A3)
