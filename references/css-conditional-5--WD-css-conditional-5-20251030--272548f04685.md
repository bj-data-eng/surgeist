Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Conditional Rules Module Level 5

Source snapshot: https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/

Snapshot SHA-256: 272548f0468544b6834bce6b5bbdb1203d3dade2b3480edfc7aed40b066d0f4a

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 16 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Conditional Rules Module Level 5

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-at-ruledef-when"></a>

<a id="ref-for-at-ruledef-else"></a>

<a id="ref-for-supports-queries"></a>

<a id="ref-for-at-ruledef-supports"></a>

This module contains the features of CSS for conditional processing of parts of style sheets, based on capabilities of the processor or the environment the style sheet is being applied in. It includes and extends the functionality of CSS Conditional 4 [\[css-conditional-4\]](#biblio-css-conditional-4), adding the generalized conditional rule [@when](#at-ruledef-when) and the chained conditional rule [@else](#at-ruledef-else), as well as introducing font processing queries to the [supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) syntax used in [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rules, and container queries.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-conditional” in the title, like this: “\[css-conditional\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-conditional%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="introduction"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-092ad31b"></a> This is currently an early draft of the things that are <em>new</em> in level 5. The features in Level 3 and Level 4 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and [\[css-conditional-4\]](#biblio-css-conditional-4) and have not yet been copied here.

<a id="ref-for-at-ruledef-supports①"></a>

<a id="ref-for-supports-queries①"></a>

<a id="ref-for-at-rule"></a>

CSS Conditional Level 5 extends the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule and [supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) syntax to allow testing for custom support conditions as well as supported [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) and font technologies.

<a id="ref-for-at-ruledef-when①"></a>

<a id="ref-for-conditional-rule-chain"></a>

It also adds an [@when](#at-ruledef-when) rule, which generalizes the concept of a conditional rule. Anything that can be expressed in an existing conditional rule can be expressed in <a id="ref-for-at-ruledef-when②"></a>@when by wrapping it in an appropriate function to declare what kind of condition it is. This allow authors to easily combine multiple types of queries, such as media queries and supports queries, in a single boolean expression. Without this, authors must rely on nesting separate conditional rules, which is harder to read and write, presupposes the conditions are to be conjoined with the “and” boolean relation (with no easy way to indicate anything else), and restricts their utility in the proposed [conditional rule chains](#conditional-rule-chain).

<a id="ref-for-at-ruledef-else①"></a>

<a id="ref-for-conditional-rule-chain①"></a>

It also adds [@else](#at-ruledef-else) rules, which immediately follow other conditional rules and automatically qualify their conditions as the inverse of the immediately preceding rule’s conditions, such that only the first matching rule in a [conditional rule chain](#conditional-rule-chain) is applied.

It also adds Container Queries. They are conceptually similar to Media Queries, but allow testing aspects of elements within the document (such as box dimensions or computed styles), rather than on the document as a whole.

<a id="ref-for-at-ruledef-supports②"></a>

## <a id="at-supports-ext"></a>2.  Extensions to the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule

<a id="ref-for-typedef-supports-feature"></a>

This level of the specification extends the [\<supports-feature\>](#typedef-supports-feature) syntax as follows:

<a id="typedef-supports-feature"></a>

<a id="ref-for-typedef-supports-selector-fn"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-supports-font-tech-fn"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-typedef-supports-font-format-fn"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-typedef-supports-at-rule-fn"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-typedef-supports-decl"></a>

<a id="typedef-supports-decl"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-supports-condition-name"></a>

<a id="typedef-supports-font-tech-fn"></a>

<a id="ref-for-font-tech-values"></a>

<a id="typedef-supports-font-format-fn"></a>

<a id="ref-for-font-format-values"></a>

<a id="typedef-supports-at-rule-fn"></a>

<a id="ref-for-typedef-at-keyword-token"></a>

```text
<supports-feature> = <supports-selector-fn>
                   | <supports-font-tech-fn> | <supports-font-format-fn>
                   | <supports-at-rule-fn>
           | <supports-decl>
<supports-decl> = ( [ <declaration> | <supports-condition-name> ] )
<supports-font-tech-fn> = font-tech( <font-tech> )
<supports-font-format-fn> = font-format( <font-format> )
<supports-at-rule-fn> = at-rule( <at-keyword-token> )
```
<a id="ref-for-typedef-supports-condition-name①"></a>

[\<supports-condition-name\>](#typedef-supports-condition-name)

<a id="ref-for-dfn-supports-condition-name"></a>

The result is true if the UA [supports the named condition](#dfn-supports-condition-name). If the name is not recognized, the result is false.

<a id="ref-for-typedef-supports-font-tech-fn①"></a>

[\<supports-font-tech-fn\>](#typedef-supports-font-tech-fn)

<a id="ref-for-dfn-support-font-tech"></a>

The result is true if the UA [supports the font tech](#dfn-support-font-tech) provided as an argument to the function.

<a id="ref-for-typedef-supports-font-format-fn①"></a>

[\<supports-font-format-fn\>](#typedef-supports-font-format-fn)

<a id="ref-for-dfn-support-font-format"></a>

The result is true if the UA [supports the font format](#dfn-support-font-format) provided as an argument to the function.

<a id="ref-for-typedef-supports-at-rule-fn①"></a>

[\<supports-at-rule-fn\>](#typedef-supports-at-rule-fn)

<a id="ref-for-dfn-support-at-rule"></a>

The result is true if the UA [supports the at-rule](#dfn-support-at-rule) provided as an argument to the function.

### <a id="support-definition-ext"></a>2.1.  Extensions to the definition of support

#### <a id="support-definition-ext-fonts"></a>2.1.1.  Font techs and formats

Tests

- [CSS-supports-L5.html](https://wpt.fyi/results/css/css-conditional/js/CSS-supports-L5.html) [(live test)](http://wpt.live/css/css-conditional/js/CSS-supports-L5.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/CSS-supports-L5.html)

A CSS processor is considered to <a id="dfn-support-font-tech"></a>support a font tech when it is capable of utilizing the specified [CSS Fonts 4 § 11.1 Font tech](https://www.w3.org/TR/css-fonts-4/#font-tech-definitions) in layout and rendering.

<a id="ref-for-string-value"></a>

A CSS processor is considered to <a id="dfn-support-font-format"></a>support a font format when it is capable of utilizing the specified [CSS Fonts 4 § 11.2 Font formats](https://www.w3.org/TR/css-fonts-4/#font-format-definitions) in layout and rendering, and this format is not specified as a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value).

#### <a id="support-definition-at-rules"></a>2.1.2.  At-rules

Tests

- [supports-at-rule.html](https://wpt.fyi/results/css/css-conditional/js/supports-at-rule.html) [(live test)](http://wpt.live/css/css-conditional/js/supports-at-rule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/js/supports-at-rule.html)

<a id="ref-for-at-rule①"></a>

A CSS processor is considered to <a id="dfn-support-at-rule"></a>support an at-rule if it would accept an [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) beginning with the specified at-keyword within any context.

<a id="ref-for-at-ruledef-charset"></a>

<a id="ref-for-at-rule②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [@charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset) is not a valid [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule), it is not considered to be supported under this definition.

#### <a id="support-definition-supports-condition-name"></a>2.1.3.  Named conditions

<a id="ref-for-named-supports-condition"></a>

A CSS processor is considered to <a id="dfn-supports-condition-name"></a>support a named condition when the related [named supports condition](#named-supports-condition) returns true.

<a id="ref-for-at-ruledef-when③"></a>

## <a id="when-rule"></a>3.  Generalized Conditional Rules: the [@when](#at-ruledef-when) rule

<a id="ref-for-conditional-group-rule"></a>

<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-supports③"></a>

The <a id="at-ruledef-when"></a>@when at-rule is a [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) that generalizes the individual <a id="ref-for-conditional-group-rule①"></a>conditional group rules such as [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media) and [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). It is defined as:

<a id="ref-for-typedef-boolean-condition"></a>

<a id="ref-for-typedef-rule-list"></a>

```text
@when <boolean-condition> {
  <rule-list>
}
```
<a id="ref-for-typedef-boolean-condition①"></a>

<a id="ref-for-funcdef-media"></a>

<a id="ref-for-funcdef-supports"></a>

Where <a id="typedef-boolean-condition"></a>[\<boolean-condition\>](#typedef-boolean-condition) is a boolean algebra a la [Media Queries 4 § 3 Syntax](https://www.w3.org/TR/mediaqueries-4/#mq-syntax), but with [media()](#funcdef-media) and [supports()](#funcdef-supports) functions as leaves.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e3cd55e5"></a> Define "boolean algebra, with X as leaves" in a generic way in Conditional, so all the conditional rules can reference it directly, rather than having to redefine boolean algebra on their own.

<a id="ref-for-funcdef-media①"></a>

<a id="ref-for-funcdef-supports①"></a>

The [media()](#funcdef-media) and [supports()](#funcdef-supports) functions are defined as:

<a id="funcdef-media"></a>

<a id="ref-for-typedef-mf-plain"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-mf-boolean"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-mf-range"></a>

<a id="funcdef-supports"></a>

```text
media() = media( [ <mf-plain> | <mf-boolean> | <mf-range> ] )
supports() = supports( <declaration> )
```
<a id="ref-for-funcdef-media②"></a>

<a id="ref-for-funcdef-supports②"></a>

A [media()](#funcdef-media) or [supports()](#funcdef-supports) function is associated with the boolean result that its contained condition is associated with.

<a id="ref-for-at-ruledef-else②"></a>

## <a id="else-rule"></a>4.  Chained Conditionals: the [@else](#at-ruledef-else) rule

<a id="ref-for-conditional-group-rule②"></a>

Usually, [conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) are independent; each one has a separate condition evaluated without direct reference to any other rule, and decides whether or not to apply its contained rules based solely on its condition.

This is fine for simple conditions, but makes it difficult to write a collection of conditionals that are meant to be mutually exclusive: authors have to very carefully craft their conditions to not activate when the other rules are meant to, and make sure the collection of conditionals don’t accidentally <em>all</em> exclude some situation which is then left unstyled.

<a id="ref-for-conditional-group-rule③"></a>

<a id="ref-for-conditional-rule-chain②"></a>

The <a id="at-ruledef-else"></a>@else rule is a [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) used to form [conditional rule chains](#conditional-rule-chain), which associate multiple <a id="ref-for-conditional-group-rule④"></a>conditional group rules and guarantee that only the first one that matches will evaluate its condition as true. It is defined as:

<a id="ref-for-typedef-boolean-condition②"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-rule-list①"></a>

```text
@else <boolean-condition>? {
  <rule-list>
}
```
<a id="ref-for-at-ruledef-else③"></a>

<a id="ref-for-at-ruledef-when④"></a>

<a id="ref-for-typedef-boolean-condition③"></a>

[@else](#at-ruledef-else) is interpreted identically to [@when](#at-ruledef-when). If its [\<boolean-condition\>](#typedef-boolean-condition) is omitted, it’s treated as having a condition that’s always true.

<a id="ref-for-conditional-group-rule⑤"></a>

<a id="ref-for-at-ruledef-else④"></a>

A <a id="conditional-rule-chain"></a>conditional rule chain is a series of consecutive [conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule), starting with a <a id="ref-for-conditional-group-rule⑥"></a>conditional group rule other than [@else](#at-ruledef-else), followed by zero or more <a id="ref-for-at-ruledef-else⑤"></a>@else rules. There cannot be anything between the successive <a id="ref-for-conditional-group-rule⑦"></a>conditional group rules other than whitespace and/or comments; any other token “breaks” the chain.

<a id="ref-for-at-ruledef-else⑥"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a37c2e08"></a> Should we require that only the last [@else](#at-ruledef-else) in a chain can have an omitted condition? It’s not uncommon for me, when debugging code, to short-circuit an if-else chain by setting one of them to "true"; I presume that would be similarly useful in CSS? It’s still pretty easy to see you’ve done something wrong if you omit the condition accidentally.

<a id="ref-for-conditional-rule-chain③"></a>

<a id="ref-for-conditional-group-rule⑧"></a>

Within a [conditional rule chain](#conditional-rule-chain), the conditions of each [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) are evaluated in order. If one of them is true, the conditions of all <em>following</em> <a id="ref-for-conditional-group-rule⑨"></a>conditional group rules in the chain evaluate to false, regardless of their stated condition.

<a id="ref-for-at-ruledef-else⑦"></a>

<a id="ref-for-conditional-rule-chain④"></a>

An [@else](#at-ruledef-else) rule that is not part of a [conditional rule chain](#conditional-rule-chain) is invalid and must be ignored.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-042e36fb"></a> For example, here’s a (somewhat silly) conditional chain:
>
> ```css
> @when media(width >= 400px) and media(pointer: fine) and supports(display: flex) {
>   /* A */
> } @else supports(caret-color: pink) and supports(background: double-rainbow()) {
>   /* B */
> } @else {
>   /* C */
> }
> ```
>
> Exactly one of the preceding rules will be chosen, even though the second rule doesn’t exclude large widths, fine points, or flexbox support, and the last rule doesn’t specify anything at all.
>
> <a id="ref-for-conditional-rule-chain⑤"></a>
>
> To achieve the same result without [conditional rule chains](#conditional-rule-chain), you’d need to write:
>
> ```css
> @media (width >= 400px) and (pointer: fine) {
>   @supports (display: flex) {
>   /* A */
>   }
>   @supports not (display: flex) {
>   @supports (caret-color: pink) and (background: double-rainbow()) {
>     /* B */
>   }
>   @supports not ((caret-color: pink) and (background: double-rainbow())) {
>     /* C */
>   }
>   }
> }
> @media not ((width >= 400px) and (pointer: fine)) {
>   @supports (caret-color: pink) and (background: double-rainbow()) {
>   /* B */
>   }
>   @supports not ((caret-color: pink) and (background: double-rainbow())) {
>   /* C */
>   }
> }
> ```
>
> This is simultaneously hard to read, requires significant duplication of both conditions and contents, and is <em>very</em> difficult to write correctly. If the conditions got any more complicated (which is not unusual in real-world content), the example would get <em>significantly</em> worse.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e1965372"></a> In this example, three different color font technologies are tested, in order of preference, plus a monochrome fallback. The most capable, COLRv1, supports both gradients and font variations; the next best choice, SVG, supports gradients while the least capable, COLRv0, supports flat color fill only.
>
> The fallback has no test condition, so will always be chosen unless one of the earlier conditions succeeds.
>
> ```css
> @when font-tech(color-COLRv1) and font-tech(variations) {
>   @font-face { font-family: icons; src: url(icons-gradient-var.woff2); }
> }
> @else font-tech(color-SVG) {
>   @font-face { font-family: icons; src: url(icons-gradient.woff2); }
> }
> @else font-tech(color-COLRv0) {
>   @font-face { font-family: icons; src: url(icons-flat.woff2); }
> }
> @else {
>   @font-face { font-family: icons; src: url(icons-fallback.woff2); }
> }
> ```
>
> Notice that in this example, the variable color font is only downloaded if COLRv1 is supported and font variations are also supported.
>
> <a id="ref-for-at-ruledef-when⑤"></a>
>
> <a id="ref-for-at-ruledef-else⑧"></a>
>
> Notice too that only one of the available options will be downloaded; this would not be the case without [@when](#at-ruledef-when) and [@else](#at-ruledef-else), as the next example shows.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a1e1a2b3"></a> In this example, although it appears that the fallback will not be used if COLRv1 is supported, in fact both fonts will be downloaded, which wastes bandwidth if it is not used.
>
> The fallback might still be used for some characters; for example, if the color font supports only Latin, while the fallback supports Latin and Greek.
>
> ```css
> @font-face { font-family: icons; src: url(icons-fallback.woff2);
> @supports font-tech(color-COLRv1) {
>   @font-face { font-family: icons; src: url(icons-gradient-var.woff2); }
> }
> ```
## <a id="container-queries"></a>5.  Container Queries

<a id="ref-for-media-query"></a>

<a id="ref-for-container-query"></a>

While [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query) provide a method to query aspects of the user agent or device environment that a document is being displayed in (such as viewport dimensions or user preferences), [container queries](#container-query) allow testing aspects of elements within the document (such as box dimensions or computed styles).

<a id="ref-for-container-style-query"></a>

<a id="ref-for-query-container"></a>

<a id="ref-for-container-size-query"></a>

<a id="ref-for-container-scroll-state-query"></a>

<a id="ref-for-propdef-container-type"></a>

<a id="ref-for-propdef-container"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-flat-tree"></a>

<a id="ref-for-at-ruledef-container"></a>

<a id="ref-for-conditional-group-rule①⓪"></a>

By default, all elements are <a id="query-container"></a>query containers for the purpose of [container style queries](#container-style-query), and can be established as [query containers](#query-container) for [container size queries](#container-size-query) and [container scroll-state queries](#container-scroll-state-query) by specifying the additional query types using the [container-type](#propdef-container-type) property (or the [container](#propdef-container) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property)). Style rules applying to a <a id="ref-for-query-container①"></a>query container’s [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) descendants can be conditioned by querying against it, using the [@container](#at-ruledef-container) [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule).

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
>   grid-template: 'img content' auto / auto 1fr;
>   }
> }
> ```
>
> Media objects in the main and sidebar areas will each respond to their own container context.

<a id="ref-for-selectordef-part"></a>

<a id="ref-for-selectordef-slotted"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-query-container②"></a>

<a id="ref-for-flat-tree①"></a>

<a id="ref-for-originating-element"></a>

For the [::part()](https://www.w3.org/TR/css-shadow-parts-1/#selectordef-part) and [::slotted()](https://drafts.csswg.org/css-scoping-1/#selectordef-slotted) [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element) selectors, which represent real elements in the DOM tree, [query containers](#query-container) can be established by [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree) ancestors of those elements. For other <a id="ref-for-pseudo-element①"></a>pseudo-elements, <a id="ref-for-query-container③"></a>query containers can be established by inclusive <a id="ref-for-flat-tree②"></a>flat tree ancestors of their [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

> <strong data-conversion-semantic="note">Note</strong>
>
> It follows that:
>
> - <a id="ref-for-selectordef-before"></a>
>
>   <a id="ref-for-selectordef-after"></a>
>
>   <a id="ref-for-selectordef-marker"></a>
>
>   <a id="ref-for-selectordef-backdrop"></a>
>
>   [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before), [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after), [::marker](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker), and [::backdrop](https://www.w3.org/TR/css-position-4/#selectordef-backdrop) query their originating elements
>
> - <a id="ref-for-selectordef-first-letter"></a>
>
>   <a id="ref-for-selectordef-first-line"></a>
>
>   <a id="ref-for-fictional-tag-sequence"></a>
>
>   [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) and [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) query their originating elements, even if the [fictional tag sequence](https://www.w3.org/TR/css-pseudo-4/#fictional-tag-sequence) may push the `::first-line` past other elements for the purpose of inheritance and rendering
>
> - <a id="ref-for-selectordef-slotted①"></a>
>
>   [::slotted()](https://drafts.csswg.org/css-scoping-1/#selectordef-slotted) selectors can query containers inside the shadow tree, including the slot itself
>
> - ::slotted()::before selectors can query the slotted shadow host child
>
> - <a id="ref-for-selectordef-part①"></a>
>
>   [::part()](https://www.w3.org/TR/css-shadow-parts-1/#selectordef-part) selectors can query containers inside the shadow tree
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
>   width: 100px;
>   container-type: inline-size;
>   }
>   @container (inline-size < 150px) {
>   #inner::before {
>     content: "BEFORE";
>   }
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
>   <style>
>     #container {
>     width: 100px;
>     container-type: inline-size;
>     }
>     @container (inline-size < 150px) {
>     ::slotted(span) {
>       color: green;
>     }
>     }
>   </style>
>   <div id=container>
>     <slot />
>   </div>
>   </template>
>   <span id=slotted>Green</span>
> </div>
> ```
<a id="ref-for-propdef-container-type①"></a>

### <a id="container-type"></a>5.1.  Creating Query Containers: the [container-type](#propdef-container-type) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-container-type"></a>container-type

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one⑦"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ \[ size <a id="ref-for-comb-one⑧"></a>\| inline-size \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) scroll-state \]

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

<a id="ref-for-propdef-container-type②"></a>

<a id="ref-for-query-container④"></a>

<a id="ref-for-container-query①"></a>

<a id="ref-for-container-style-query①"></a>

The [container-type](#propdef-container-type) property establishes the element as a [query container](#query-container) for certain types of queries. For size [container queries](#container-query), which require certain types of containment, elements are explicitly made <a id="ref-for-query-container⑤"></a>query containers through this property. For other types of <a id="ref-for-query-container⑥"></a>query containers any element can be a <a id="ref-for-query-container⑦"></a>query container, such as for [container style queries](#container-style-query).

Values have the following meanings:

<a id="valdef-container-type-size"></a>size  
<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-principal-box"></a>

<a id="ref-for-size-containment"></a>

<a id="ref-for-style-containment"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-container-size-query①"></a>

<a id="ref-for-query-container⑧"></a>

Establishes a [query container](#query-container) for [container size queries](#container-size-query) on both the [inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) and [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Applies [style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and [size containment](https://www.w3.org/TR/css-contain-2/#size-containment) to the [principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<a id="valdef-container-type-inline-size"></a>inline-size  
<a id="ref-for-independent-formatting-context①"></a>

<a id="ref-for-principal-box①"></a>

<a id="ref-for-inline-size-containment"></a>

<a id="ref-for-style-containment①"></a>

<a id="ref-for-inline-axis①"></a>

<a id="ref-for-container-size-query②"></a>

<a id="ref-for-query-container⑨"></a>

Establishes a [query container](#query-container) for [container size queries](#container-size-query) on the container’s own [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Applies [style containment](https://www.w3.org/TR/css-contain-2/#style-containment) and [inline-size containment](https://www.w3.org/TR/css-contain-3/#inline-size-containment) to the [principal box](https://www.w3.org/TR/css-display-4/#principal-box), and establishes an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

<a id="valdef-container-type-scroll-state"></a>scroll-state  
<a id="ref-for-container-scroll-state-query①"></a>

<a id="ref-for-query-container①⓪"></a>

Establishes a [query container](#query-container) for [container scroll-state queries](#container-scroll-state-query)

<a id="valdef-container-type-normal"></a>normal  
<a id="ref-for-container-style-query②"></a>

<a id="ref-for-container-scroll-state-query②"></a>

<a id="ref-for-container-size-query③"></a>

<a id="ref-for-query-container①①"></a>

The element is not a [query container](#query-container) for any [container size queries](#container-size-query) or [container scroll-state queries](#container-scroll-state-query), but remains a <a id="ref-for-query-container①②"></a>query container for [container style queries](#container-style-query).

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
> <a id="ref-for-query-container①③"></a>
>
> The 40em value used in the query condition is relative to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant [query container](#query-container).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-76326f8c"></a> Containers can also expose computed style values for querying. This can be useful for toggling behavior across multiple properties:
>
> ```css
> @container style(--cards: small) {
>   article {
>   border: thin solid silver;
>   border-radius: 0.5em;
>   padding: 1em;
>   }
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9f75dbe5"></a> Containers can also expose state that depends on scroll offset. This example styles a descendant of a sticky positioned element when it is stuck to the top edge:
>
> ```css
> #sticky {
>   container-type: scroll-state;
>   position: sticky;
> }
> @container scroll-state(stuck: top) {
>   #sticky-child {
>   background-color: lime;
>   }
> }
> ```
<a id="ref-for-propdef-container-name"></a>

### <a id="container-name"></a>5.2.  Naming Query Containers: the [container-name](#propdef-container-name) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-container-name"></a>container-name

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-comb-one⑨"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

[all elements](https://www.w3.org/TR/css-pseudo/#generated-content)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-css-css-identifier"></a>

<a id="ref-for-valdef-container-name-none"></a>

the keyword [none](#valdef-container-name-none), or an ordered list of [identifiers](https://www.w3.org/TR/css-values-4/#css-css-identifier)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

not animatable

Tests

- [container-ident-function.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-ident-function.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-ident-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-ident-function.html)
- [container-ident-function.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-ident-function.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-ident-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-ident-function.html)

<a id="ref-for-propdef-container-name①"></a>

<a id="ref-for-at-ruledef-container①"></a>

<a id="ref-for-query-container①④"></a>

<a id="ref-for-css-tree-scoped-name"></a>

The [container-name](#propdef-container-name) property specifies a list of <a id="query-container-name"></a>query container names. These names can be used by [@container](#at-ruledef-container) rules to filter which [query containers](#query-container) are targeted. Container names are not [tree-scoped names](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-name).

<a id="valdef-container-name-none"></a>none

<a id="ref-for-query-container-name"></a>

<a id="ref-for-query-container①⑤"></a>

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
> <a id="example-63042722"></a> In some cases, we want to query aspects of a specific container, even if it’s not the nearest ancestor container. For example, we might want to query the height of a main content area, and the width of a more nested inline-container.
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
>
> It is also possible to query for a container only based on its name.
>
> ```css
> @container my-page-layout {
>   .card { padding: 1em; }
> }
> ```
<a id="ref-for-propdef-container①"></a>

### <a id="container-shorthand"></a>5.3.  Creating Named Containers: the [container](#propdef-container) shorthand

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-container"></a>container

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-propdef-container-type③"></a>

<a id="ref-for-propdef-container-name②"></a>

[\<'container-name'\>](#propdef-container-name) \[ / [\<'container-type'\>](#propdef-container-type) \][?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

Tests

- [animation-container-size.html](https://wpt.fyi/results/css/css-conditional/container-queries/animation-container-size.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/animation-container-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/animation-container-size.html)
- [animation-container-type-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/animation-container-type-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/animation-container-type-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/animation-container-type-dynamic.html)
- [animation-nested-animation.html](https://wpt.fyi/results/css/css-conditional/container-queries/animation-nested-animation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/animation-nested-animation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/animation-nested-animation.html)
- [animation-nested-transition.html](https://wpt.fyi/results/css/css-conditional/container-queries/animation-nested-transition.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/animation-nested-transition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/animation-nested-transition.html)
- [aspect-ratio-feature-evaluation.html](https://wpt.fyi/results/css/css-conditional/container-queries/aspect-ratio-feature-evaluation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/aspect-ratio-feature-evaluation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/aspect-ratio-feature-evaluation.html)
- [at-container-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/at-container-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/at-container-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/at-container-parsing.html)
- [at-container-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/at-container-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/at-container-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/at-container-serialization.html)
- [at-container-style-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/at-container-style-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/at-container-style-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/at-container-style-parsing.html)
- [at-container-style-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/at-container-style-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/at-container-style-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/at-container-style-serialization.html)
- [auto-scrollbars.html](https://wpt.fyi/results/css/css-conditional/container-queries/auto-scrollbars.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/auto-scrollbars.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/auto-scrollbars.html)
- [backdrop-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/backdrop-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/backdrop-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/backdrop-invalidation.html)
- [calc-evaluation.html](https://wpt.fyi/results/css/css-conditional/container-queries/calc-evaluation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/calc-evaluation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/calc-evaluation.html)
- [canvas-as-container-001.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-001.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-001.html)
- [canvas-as-container-002.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-002.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-002.html)
- [canvas-as-container-003.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-003.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-003.html)
- [canvas-as-container-004.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-004.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-004.html)
- [canvas-as-container-005.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-005.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-005.html)
- [canvas-as-container-006.html](https://wpt.fyi/results/css/css-conditional/container-queries/canvas-as-container-006.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/canvas-as-container-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/canvas-as-container-006.html)
- [change-display-in-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/change-display-in-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/change-display-in-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/change-display-in-container.html)
- [chrome-legacy-skip-recalc.html](https://wpt.fyi/results/css/css-conditional/container-queries/chrome-legacy-skip-recalc.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/chrome-legacy-skip-recalc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/chrome-legacy-skip-recalc.html)
- [column-spanner-in-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/column-spanner-in-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/column-spanner-in-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/column-spanner-in-container.html)
- [conditional-container-status.html](https://wpt.fyi/results/css/css-conditional/container-queries/conditional-container-status.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/conditional-container-status.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/conditional-container-status.html)
- [container-computed.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-computed.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-computed.html)
- [container-for-cue.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-for-cue.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-for-cue.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-for-cue.html)
- [container-for-shadow-dom.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-for-shadow-dom.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-for-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-for-shadow-dom.html)
- [container-inheritance.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-inheritance.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-inheritance.html)
- [container-inner-at-rules.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-inner-at-rules.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-inner-at-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-inner-at-rules.html)
- [container-inside-multicol-with-table.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-inside-multicol-with-table.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-inside-multicol-with-table.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-inside-multicol-with-table.html)
- [container-longhand-animation-type.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-longhand-animation-type.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-longhand-animation-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-longhand-animation-type.html)
- [container-name-computed.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-name-computed.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-name-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-name-computed.html)
- [container-name-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-name-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-name-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-name-invalidation.html)
- [container-name-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-name-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-name-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-name-parsing.html)
- [container-name-tree-scoped.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-name-tree-scoped.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-name-tree-scoped.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-name-tree-scoped.html)
- [container-nested.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-nested.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-nested.html)
- [container-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-parsing.html)
- [container-selection-unknown-features.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-selection-unknown-features.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-selection-unknown-features.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-selection-unknown-features.html)
- [container-selection.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-selection.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-selection.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-selection.html)
- [container-size-invalidation-after-load.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-size-invalidation-after-load.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-size-invalidation-after-load.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-size-invalidation-after-load.html)
- [container-size-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-size-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-size-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-size-invalidation.html)
- [container-size-nested-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-size-nested-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-size-nested-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-size-nested-invalidation.html)
- [container-size-shadow-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-size-shadow-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-size-shadow-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-size-shadow-invalidation.html)
- [container-type-computed.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-type-computed.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-type-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-type-computed.html)
- [container-type-containment.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-type-containment.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-type-containment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-type-containment.html)
- [container-type-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-type-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-type-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-type-invalidation.html)
- [container-type-layout-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-type-layout-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-type-layout-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-type-layout-invalidation.html)
- [container-type-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-type-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-type-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-type-parsing.html)
- [container-units-animation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-animation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-animation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-animation.html)
- [container-units-basic.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-basic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-basic.html)
- [container-units-computational-independence.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-computational-independence.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-computational-independence.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-computational-independence.html)
- [container-units-content-box.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-content-box.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-content-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-content-box.html)
- [container-units-gradient-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-gradient-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-gradient-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-gradient-invalidation.html)
- [container-units-gradient.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-gradient.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-gradient.html)
- [container-units-in-at-container-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-in-at-container-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-in-at-container-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-in-at-container-dynamic.html)
- [container-units-in-at-container-fallback.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-in-at-container-fallback.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-in-at-container-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-in-at-container-fallback.html)
- [container-units-in-at-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-in-at-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-in-at-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-in-at-container.html)
- [container-units-ineligible-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-ineligible-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-ineligible-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-ineligible-container.html)
- [container-units-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-invalidation.html)
- [container-units-media-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-media-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-media-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-media-queries.html)
- [container-units-rule-cache.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-rule-cache.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-rule-cache.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-rule-cache.html)
- [container-units-selection.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-selection.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-selection.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-selection.html)
- [container-units-shadow.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-shadow.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-shadow.html)
- [container-units-sharing-via-rule-node.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-sharing-via-rule-node.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-sharing-via-rule-node.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-sharing-via-rule-node.html)
- [container-units-small-viewport-fallback.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-small-viewport-fallback.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-small-viewport-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-small-viewport-fallback.html)
- [container-units-svglength.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-svglength.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-svglength.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-svglength.html)
- [container-units-typed-om.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-typed-om.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-typed-om.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-typed-om.html)
- [counters-flex-circular.html](https://wpt.fyi/results/css/css-conditional/container-queries/counters-flex-circular.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/counters-flex-circular.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/counters-flex-circular.html)
- [counters-in-container-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/counters-in-container-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/counters-in-container-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/counters-in-container-dynamic.html)
- [counters-in-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/counters-in-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/counters-in-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/counters-in-container.html)
- [br-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/br-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/br-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/br-crash.html)
- [canvas-as-container-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/canvas-as-container-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/canvas-as-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/canvas-as-container-crash.html)
- [chrome-bug-1289718-000-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-000-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-000-crash.html)
- [chrome-bug-1289718-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1289718-001-crash.html)
- [chrome-bug-1346969-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1346969-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1346969-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1346969-crash.html)
- [chrome-bug-1362391-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1362391-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1362391-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1362391-crash.html)
- [chrome-bug-1429955-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1429955-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1429955-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1429955-crash.html)
- [chrome-bug-1505250-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-1505250-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-1505250-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-1505250-crash.html)
- [chrome-bug-346264227-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-346264227-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-346264227-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-346264227-crash.html)
- [chrome-bug-372358471-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-bug-372358471-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-bug-372358471-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-bug-372358471-crash.html)
- [chrome-custom-highlight-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-custom-highlight-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-custom-highlight-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-custom-highlight-crash.html)
- [chrome-layout-root-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-layout-root-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-layout-root-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-layout-root-crash.html)
- [chrome-quotes-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-quotes-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-quotes-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-quotes-crash.html)
- [chrome-remove-insert-evaluator-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/chrome-remove-insert-evaluator-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/chrome-remove-insert-evaluator-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/chrome-remove-insert-evaluator-crash.html)
- [columns-in-table-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/columns-in-table-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/columns-in-table-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/columns-in-table-001-crash.html)
- [columns-in-table-002-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/columns-in-table-002-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/columns-in-table-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/columns-in-table-002-crash.html)
- [container-in-canvas-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/container-in-canvas-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/container-in-canvas-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/container-in-canvas-crash.html)
- [container-type-change-chrome-legacy-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/container-type-change-chrome-legacy-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/container-type-change-chrome-legacy-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/container-type-change-chrome-legacy-crash.html)
- [dialog-backdrop-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/dialog-backdrop-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/dialog-backdrop-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/dialog-backdrop-crash.html)
- [dirty-rowgroup-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/dirty-rowgroup-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/dirty-rowgroup-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/dirty-rowgroup-crash.html)
- [flex-in-columns-000-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/flex-in-columns-000-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/flex-in-columns-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/flex-in-columns-000-crash.html)
- [flex-in-columns-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/flex-in-columns-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/flex-in-columns-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/flex-in-columns-001-crash.html)
- [flex-in-columns-002-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/flex-in-columns-002-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/flex-in-columns-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/flex-in-columns-002-crash.html)
- [flex-in-columns-003-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/flex-in-columns-003-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/flex-in-columns-003-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/flex-in-columns-003-crash.html)
- [focus-inside-content-visibility-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/focus-inside-content-visibility-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/focus-inside-content-visibility-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/focus-inside-content-visibility-crash.html)
- [force-sibling-style-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/force-sibling-style-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/force-sibling-style-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/force-sibling-style-crash.html)
- [grid-in-columns-000-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/grid-in-columns-000-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/grid-in-columns-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/grid-in-columns-000-crash.html)
- [grid-in-columns-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/grid-in-columns-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/grid-in-columns-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/grid-in-columns-001-crash.html)
- [grid-in-columns-002-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/grid-in-columns-002-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/grid-in-columns-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/grid-in-columns-002-crash.html)
- [grid-in-columns-003-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/grid-in-columns-003-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/grid-in-columns-003-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/grid-in-columns-003-crash.html)
- [iframe-init-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/iframe-init-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/iframe-init-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/iframe-init-crash.html)
- [inline-multicol-inside-container-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/inline-multicol-inside-container-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/inline-multicol-inside-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/inline-multicol-inside-container-crash.html)
- [inline-with-columns-000-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/inline-with-columns-000-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/inline-with-columns-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/inline-with-columns-000-crash.html)
- [inline-with-columns-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/inline-with-columns-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/inline-with-columns-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/inline-with-columns-001-crash.html)
- [input-column-group-container-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/input-column-group-container-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/input-column-group-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/input-column-group-container-crash.html)
- [input-placeholder-inline-size-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/input-placeholder-inline-size-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/input-placeholder-inline-size-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/input-placeholder-inline-size-crash.html)
- [marker-gcs-after-disconnect-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/marker-gcs-after-disconnect-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/marker-gcs-after-disconnect-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/marker-gcs-after-disconnect-crash.html)
- [math-block-container-child-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/math-block-container-child-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/math-block-container-child-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/math-block-container-child-crash.html)
- [mathml-container-type-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/mathml-container-type-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/mathml-container-type-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/mathml-container-type-crash.html)
- [orthogonal-replaced-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/orthogonal-replaced-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/orthogonal-replaced-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/orthogonal-replaced-crash.html)
- [pseudo-container-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/pseudo-container-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/pseudo-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/pseudo-container-crash.html)
- [remove-dom-child-change-style.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/remove-dom-child-change-style.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/remove-dom-child-change-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/remove-dom-child-change-style.html)
- [reversed-ol-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/reversed-ol-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/reversed-ol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/reversed-ol-crash.html)
- [size-change-during-transition-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/size-change-during-transition-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/size-change-during-transition-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/size-change-during-transition-crash.html)
- [svg-layout-root-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/svg-layout-root-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/svg-layout-root-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/svg-layout-root-crash.html)
- [svg-resource-in-container-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/svg-resource-in-container-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/svg-resource-in-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/svg-resource-in-container-crash.html)
- [svg-text-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/svg-text-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/svg-text-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/svg-text-crash.html)
- [table-in-columns-000-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-000-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-000-crash.html)
- [table-in-columns-001-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-001-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-001-crash.html)
- [table-in-columns-002-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-002-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-002-crash.html)
- [table-in-columns-003-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-003-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-003-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-003-crash.html)
- [table-in-columns-004-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-004-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-004-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-004-crash.html)
- [table-in-columns-005-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/table-in-columns-005-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/table-in-columns-005-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/table-in-columns-005-crash.html)
- [top-layer-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/top-layer-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/top-layer-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/top-layer-crash.html)
- [top-layer-nested-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/crashtests/top-layer-nested-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/crashtests/top-layer-nested-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/crashtests/top-layer-nested-crash.html)
- [custom-layout-container-001.https.html](https://wpt.fyi/results/css/css-conditional/container-queries/custom-layout-container-001.https.html) [(live test)](https://wpt.live/css/css-conditional/container-queries/custom-layout-container-001.https.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/custom-layout-container-001.https.html)
- [custom-property-style-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/custom-property-style-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/custom-property-style-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/custom-property-style-queries.html)
- [custom-property-style-query-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/custom-property-style-query-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/custom-property-style-query-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/custom-property-style-query-change.html)
- [deep-nested-inline-size-containers.html](https://wpt.fyi/results/css/css-conditional/container-queries/deep-nested-inline-size-containers.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/deep-nested-inline-size-containers.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/deep-nested-inline-size-containers.html)
- [dialog-backdrop-create.html](https://wpt.fyi/results/css/css-conditional/container-queries/dialog-backdrop-create.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/dialog-backdrop-create.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/dialog-backdrop-create.html)
- [dialog-backdrop-remove.html](https://wpt.fyi/results/css/css-conditional/container-queries/dialog-backdrop-remove.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/dialog-backdrop-remove.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/dialog-backdrop-remove.html)
- [display-contents-dynamic-style-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/display-contents-dynamic-style-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/display-contents-dynamic-style-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/display-contents-dynamic-style-queries.html)
- [display-contents.html](https://wpt.fyi/results/css/css-conditional/container-queries/display-contents.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/display-contents.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/display-contents.html)
- [display-in-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/display-in-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/display-in-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/display-in-container.html)
- [display-none.html](https://wpt.fyi/results/css/css-conditional/container-queries/display-none.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/display-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/display-none.html)
- [fieldset-legend-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/fieldset-legend-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/fieldset-legend-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/fieldset-legend-change.html)
- [flex-basis-with-container-type.html](https://wpt.fyi/results/css/css-conditional/container-queries/flex-basis-with-container-type.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/flex-basis-with-container-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/flex-basis-with-container-type.html)
- [font-relative-calc-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/font-relative-calc-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/font-relative-calc-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/font-relative-calc-dynamic.html)
- [font-relative-units-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/font-relative-units-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/font-relative-units-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/font-relative-units-dynamic.html)
- [font-relative-units.html](https://wpt.fyi/results/css/css-conditional/container-queries/font-relative-units.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/font-relative-units.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/font-relative-units.html)
- [fragmented-container-001.html](https://wpt.fyi/results/css/css-conditional/container-queries/fragmented-container-001.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/fragmented-container-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/fragmented-container-001.html)
- [get-animations.html](https://wpt.fyi/results/css/css-conditional/container-queries/get-animations.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/get-animations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/get-animations.html)
- [grid-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/grid-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/grid-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/grid-container.html)
- [grid-item-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/grid-item-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/grid-item-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/grid-item-container.html)
- [idlharness.html](https://wpt.fyi/results/css/css-conditional/container-queries/idlharness.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/idlharness.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/idlharness.html)
- [iframe-in-container-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/iframe-in-container-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/iframe-in-container-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/iframe-in-container-invalidation.html)
- [iframe-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/iframe-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/iframe-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/iframe-invalidation.html)
- [ineligible-containment.html](https://wpt.fyi/results/css/css-conditional/container-queries/ineligible-containment.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/ineligible-containment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/ineligible-containment.html)
- [inheritance-from-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/inheritance-from-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inheritance-from-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inheritance-from-container.html)
- [inline-size-and-min-width.html](https://wpt.fyi/results/css/css-conditional/container-queries/inline-size-and-min-width.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inline-size-and-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inline-size-and-min-width.html)
- [inline-size-bfc-floats.html](https://wpt.fyi/results/css/css-conditional/container-queries/inline-size-bfc-floats.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inline-size-bfc-floats.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inline-size-bfc-floats.html)
- [inline-size-containment-vertical-rl.html](https://wpt.fyi/results/css/css-conditional/container-queries/inline-size-containment-vertical-rl.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inline-size-containment-vertical-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inline-size-containment-vertical-rl.html)
- [inline-size-containment.html](https://wpt.fyi/results/css/css-conditional/container-queries/inline-size-containment.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inline-size-containment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inline-size-containment.html)
- [inner-first-line-non-matching.html](https://wpt.fyi/results/css/css-conditional/container-queries/inner-first-line-non-matching.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/inner-first-line-non-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/inner-first-line-non-matching.html)
- [layout-dependent-focus.html](https://wpt.fyi/results/css/css-conditional/container-queries/layout-dependent-focus.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/layout-dependent-focus.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/layout-dependent-focus.html)
- [multicol-container-001.html](https://wpt.fyi/results/css/css-conditional/container-queries/multicol-container-001.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/multicol-container-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/multicol-container-001.html)
- [multicol-inside-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/multicol-inside-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/multicol-inside-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/multicol-inside-container.html)
- [nested-query-containers.html](https://wpt.fyi/results/css/css-conditional/container-queries/nested-query-containers.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/nested-query-containers.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/nested-query-containers.html)
- [nested-size-style-container-invalidation.html](https://wpt.fyi/results/css/css-conditional/container-queries/nested-size-style-container-invalidation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/nested-size-style-container-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/nested-size-style-container-invalidation.html)
- [never-match-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/never-match-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/never-match-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/never-match-container.html)
- [no-layout-containment-abspos-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-abspos-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-abspos-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-abspos-dynamic.html)
- [no-layout-containment-abspos.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-abspos.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-abspos.html)
- [no-layout-containment-baseline.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-baseline.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-baseline.html)
- [no-layout-containment-fixedpos-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-fixedpos-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-fixedpos-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-fixedpos-dynamic.html)
- [no-layout-containment-fixedpos.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-fixedpos.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-fixedpos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-fixedpos.html)
- [no-layout-containment-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-scroll.html)
- [no-layout-containment-subgrid-crash.html](https://wpt.fyi/results/css/css-conditional/container-queries/no-layout-containment-subgrid-crash.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/no-layout-containment-subgrid-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/no-layout-containment-subgrid-crash.html)
- [orthogonal-wm-container-query.html](https://wpt.fyi/results/css/css-conditional/container-queries/orthogonal-wm-container-query.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/orthogonal-wm-container-query.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/orthogonal-wm-container-query.html)
- [percentage-padding-orthogonal.html](https://wpt.fyi/results/css/css-conditional/container-queries/percentage-padding-orthogonal.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/percentage-padding-orthogonal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/percentage-padding-orthogonal.html)
- [pseudo-elements-001.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-001.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-001.html)
- [pseudo-elements-002.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-002.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-002.html)
- [pseudo-elements-002b.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-002b.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-002b.html)
- [pseudo-elements-003.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-003.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-003.html)
- [pseudo-elements-004.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-004.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-004.html)
- [pseudo-elements-005.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-005.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-005.html)
- [pseudo-elements-006.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-006.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-006.html)
- [pseudo-elements-007.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-007.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-007.html)
- [pseudo-elements-008.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-008.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-008.html)
- [pseudo-elements-009.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-009.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-009.html)
- [pseudo-elements-010.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-010.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-010.html)
- [pseudo-elements-011.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-011.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-011.html)
- [pseudo-elements-012.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-012.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-012.html)
- [pseudo-elements-013.html](https://wpt.fyi/results/css/css-conditional/container-queries/pseudo-elements-013.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/pseudo-elements-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/pseudo-elements-013.html)
- [query-content-box.html](https://wpt.fyi/results/css/css-conditional/container-queries/query-content-box.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/query-content-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/query-content-box.html)
- [query-evaluation-style.html](https://wpt.fyi/results/css/css-conditional/container-queries/query-evaluation-style.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/query-evaluation-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/query-evaluation-style.html)
- [query-evaluation.html](https://wpt.fyi/results/css/css-conditional/container-queries/query-evaluation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/query-evaluation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/query-evaluation.html)
- [reattach-container-with-dirty-child.html](https://wpt.fyi/results/css/css-conditional/container-queries/reattach-container-with-dirty-child.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/reattach-container-with-dirty-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/reattach-container-with-dirty-child.html)
- [registered-color-style-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/registered-color-style-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/registered-color-style-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/registered-color-style-queries.html)
- [resize-while-content-visibility-hidden.html](https://wpt.fyi/results/css/css-conditional/container-queries/resize-while-content-visibility-hidden.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/resize-while-content-visibility-hidden.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/resize-while-content-visibility-hidden.html)
- [at-container-scrollable-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-scrollable-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-scrollable-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-scrollable-parsing.html)
- [at-container-scrollable-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-scrollable-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-scrollable-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-scrollable-serialization.html)
- [at-container-snapped-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-snapped-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-snapped-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-snapped-parsing.html)
- [at-container-snapped-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-snapped-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-snapped-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-snapped-serialization.html)
- [at-container-stuck-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-stuck-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-stuck-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-stuck-parsing.html)
- [at-container-stuck-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-stuck-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-stuck-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-stuck-serialization.html)
- [container-type-scroll-state-computed.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-computed.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-computed.html)
- [container-type-scroll-state-containment.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-containment.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-containment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-containment.html)
- [container-type-scroll-state-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/container-type-scroll-state-parsing.html)
- [scroll-state-initially-scrollable.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-initially-scrollable.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-initially-scrollable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-initially-scrollable.html)
- [scroll-state-initially-snapped.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-initially-snapped.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-initially-snapped.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-initially-snapped.html)
- [scroll-state-initially-stuck.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-initially-stuck.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-initially-stuck.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-initially-stuck.html)
- [scroll-state-scrollable-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-change.html)
- [scroll-state-scrollable-container-type-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-container-type-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-container-type-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-container-type-change.html)
- [scroll-state-scrollable-layout-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change.html)
- [scroll-state-scrollable-wm.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-wm.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-wm.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-wm.html)
- [scroll-state-snapped-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-change.html)
- [scroll-state-snapped-container-type-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-container-type-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-container-type-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-container-type-change.html)
- [scroll-state-snapped-layout-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-layout-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-layout-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-layout-change.html)
- [scroll-state-snapped-none.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-none.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-none.html)
- [scroll-state-snapped-snap-changing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-snap-changing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-snap-changing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-snap-changing.html)
- [scroll-state-snapped-wm.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-wm.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-wm.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-wm.html)
- [scroll-state-stuck-container-type-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-container-type-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-container-type-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-container-type-change.html)
- [scroll-state-stuck-layout-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-layout-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-layout-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-layout-change.html)
- [scroll-state-stuck-writing-direction.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-writing-direction.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-writing-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-writing-direction.html)
- [scroll-state-target-query-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-target-query-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-target-query-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-target-query-change.html)
- [scrollbar-container-units-block.html](https://wpt.fyi/results/css/css-conditional/container-queries/scrollbar-container-units-block.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scrollbar-container-units-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scrollbar-container-units-block.html)
- [scrollbar-container-units-inline.html](https://wpt.fyi/results/css/css-conditional/container-queries/scrollbar-container-units-inline.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scrollbar-container-units-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scrollbar-container-units-inline.html)
- [sibling-layout-dependency.html](https://wpt.fyi/results/css/css-conditional/container-queries/sibling-layout-dependency.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/sibling-layout-dependency.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/sibling-layout-dependency.html)
- [size-container-no-principal-box.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-container-no-principal-box.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-container-no-principal-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-container-no-principal-box.html)
- [size-container-with-quotes.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-container-with-quotes.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-container-with-quotes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-container-with-quotes.html)
- [size-container-writing-mode-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-container-writing-mode-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-container-writing-mode-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-container-writing-mode-change.html)
- [size-feature-evaluation.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-feature-evaluation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-feature-evaluation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-feature-evaluation.html)
- [style-change-in-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-change-in-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-change-in-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-change-in-container.html)
- [style-container-for-shadow-dom.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-container-for-shadow-dom.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-container-for-shadow-dom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-container-for-shadow-dom.html)
- [style-container-invalidation-inheritance.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-container-invalidation-inheritance.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-container-invalidation-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-container-invalidation-inheritance.html)
- [style-not-sharing-float.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-not-sharing-float.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-not-sharing-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-not-sharing-float.html)
- [style-query-document-element.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-query-document-element.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-query-document-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-query-document-element.html)
- [style-query-no-cycle.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-query-no-cycle.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-query-no-cycle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-query-no-cycle.html)
- [style-query-with-unknown-width.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-query-with-unknown-width.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-query-with-unknown-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-query-with-unknown-width.html)
- [svg-foreignobject-child-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/svg-foreignobject-child-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/svg-foreignobject-child-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/svg-foreignobject-child-container.html)
- [svg-foreignobject-no-size-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/svg-foreignobject-no-size-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/svg-foreignobject-no-size-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/svg-foreignobject-no-size-container.html)
- [svg-g-no-size-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/svg-g-no-size-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/svg-g-no-size-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/svg-g-no-size-container.html)
- [svg-root-size-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/svg-root-size-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/svg-root-size-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/svg-root-size-container.html)
- [table-inside-container-changing-display.html](https://wpt.fyi/results/css/css-conditional/container-queries/table-inside-container-changing-display.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/table-inside-container-changing-display.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/table-inside-container-changing-display.html)
- [top-layer-dialog-backdrop.html](https://wpt.fyi/results/css/css-conditional/container-queries/top-layer-dialog-backdrop.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/top-layer-dialog-backdrop.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/top-layer-dialog-backdrop.html)
- [top-layer-dialog-container.html](https://wpt.fyi/results/css/css-conditional/container-queries/top-layer-dialog-container.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/top-layer-dialog-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/top-layer-dialog-container.html)
- [top-layer-dialog.html](https://wpt.fyi/results/css/css-conditional/container-queries/top-layer-dialog.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/top-layer-dialog.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/top-layer-dialog.html)
- [top-layer-nested-dialog.html](https://wpt.fyi/results/css/css-conditional/container-queries/top-layer-nested-dialog.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/top-layer-nested-dialog.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/top-layer-nested-dialog.html)
- [transition-scrollbars.html](https://wpt.fyi/results/css/css-conditional/container-queries/transition-scrollbars.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/transition-scrollbars.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/transition-scrollbars.html)
- [transition-style-change-event-002.html](https://wpt.fyi/results/css/css-conditional/container-queries/transition-style-change-event-002.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/transition-style-change-event-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/transition-style-change-event-002.html)
- [transition-style-change-event.html](https://wpt.fyi/results/css/css-conditional/container-queries/transition-style-change-event.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/transition-style-change-event.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/transition-style-change-event.html)
- [unsupported-axis.html](https://wpt.fyi/results/css/css-conditional/container-queries/unsupported-axis.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/unsupported-axis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/unsupported-axis.html)
- [viewport-units-dynamic.html](https://wpt.fyi/results/css/css-conditional/container-queries/viewport-units-dynamic.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/viewport-units-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/viewport-units-dynamic.html)
- [viewport-units.html](https://wpt.fyi/results/css/css-conditional/container-queries/viewport-units.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/viewport-units.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/viewport-units.html)
- [whitespace-update-after-removal.html](https://wpt.fyi/results/css/css-conditional/container-queries/whitespace-update-after-removal.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/whitespace-update-after-removal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/whitespace-update-after-removal.html)

<a id="ref-for-propdef-container②"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-container-type④"></a>

<a id="ref-for-propdef-container-name③"></a>

<a id="ref-for-initial-value"></a>

The [container](#propdef-container) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both [container-type](#propdef-container-type) and [container-name](#propdef-container-name) in the same declaration. If <a id="ref-for-propdef-container-type⑤"></a>\<'container-type'\> is omitted, it is reset to its [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-propdef-container-type⑥"></a>

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

### <a id="container-rule"></a>5.4.  Container Queries: the [@container](#at-ruledef-container) rule

<a id="ref-for-conditional-group-rule①①"></a>

<a id="ref-for-container-size-query④"></a>

<a id="ref-for-container-style-query③"></a>

<a id="ref-for-typedef-block-contents"></a>

<a id="ref-for-at-ruledef-container③"></a>

<a id="ref-for-container-query②"></a>

<a id="ref-for-query-container①⑥"></a>

The <a id="at-ruledef-container"></a>@container rule is a [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) whose condition contains a <a id="container-query"></a>container query, which is a boolean combination of [container size queries](#container-size-query) and/or [container style queries](#container-style-query). Style declarations within the [\<block-contents\>](https://drafts.csswg.org/css-syntax-3/#typedef-block-contents) block of an [@container](#at-ruledef-container) rule are [filtered](https://www.w3.org/TR/css-cascade-4/#filtering) by its condition to only match when the [container query](#container-query) is true for their element’s [query container](#query-container).

<a id="ref-for-at-ruledef-container④"></a>

The syntax of the [@container](#at-ruledef-container) rule is:

<a id="ref-for-typedef-container-condition"></a>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-typedef-block-contents①"></a>

```text
@container <container-condition># {
  <block-contents>
}
```
where:

<a id="typedef-container-condition"></a>

<a id="ref-for-typedef-container-condition①"></a>

<a id="ref-for-typedef-container-name"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-typedef-container-query"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-mult-req"></a>

<a id="typedef-container-name"></a>

<a id="ref-for-typedef-container-name①"></a>

<a id="ref-for-identifier-value③"></a>

<a id="typedef-container-query"></a>

<a id="ref-for-typedef-container-query①"></a>

<a id="ref-for-typedef-query-in-parens"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-typedef-query-in-parens①"></a>

<a id="ref-for-typedef-query-in-parens②"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-typedef-query-in-parens③"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="typedef-query-in-parens"></a>

<a id="ref-for-typedef-query-in-parens④"></a>

<a id="ref-for-typedef-container-query②"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-size-feature"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-typedef-style-query"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-typedef-scroll-state-query"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="typedef-style-query"></a>

<a id="ref-for-typedef-style-query①"></a>

<a id="ref-for-typedef-style-in-parens"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-typedef-style-in-parens①"></a>

<a id="ref-for-typedef-style-in-parens②"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-typedef-style-in-parens③"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-typedef-style-feature"></a>

<a id="typedef-style-in-parens"></a>

<a id="ref-for-typedef-style-in-parens④"></a>

<a id="ref-for-typedef-style-query②"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-typedef-style-feature①"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="typedef-style-feature"></a>

<a id="ref-for-typedef-style-feature②"></a>

<a id="ref-for-typedef-style-feature-plain"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-typedef-style-feature-boolean"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-style-range"></a>

<a id="typedef-style-feature-plain"></a>

<a id="ref-for-typedef-style-feature-plain①"></a>

<a id="ref-for-typedef-style-feature-name"></a>

<a id="ref-for-typedef-style-feature-value"></a>

<a id="typedef-style-feature-boolean"></a>

<a id="ref-for-typedef-style-feature-boolean①"></a>

<a id="ref-for-typedef-style-feature-name①"></a>

<a id="typedef-style-range"></a>

<a id="ref-for-typedef-style-range①"></a>

<a id="ref-for-typedef-style-range-value"></a>

<a id="ref-for-typedef-mf-comparison"></a>

<a id="ref-for-typedef-style-range-value①"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-typedef-style-range-value②"></a>

<a id="ref-for-typedef-mf-lt"></a>

<a id="ref-for-typedef-style-range-value③"></a>

<a id="ref-for-typedef-mf-lt①"></a>

<a id="ref-for-typedef-style-range-value④"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-typedef-style-range-value⑤"></a>

<a id="ref-for-typedef-mf-gt"></a>

<a id="ref-for-typedef-style-range-value⑥"></a>

<a id="ref-for-typedef-mf-gt①"></a>

<a id="ref-for-typedef-style-range-value⑦"></a>

<a id="typedef-style-range-value"></a>

<a id="ref-for-typedef-style-range-value⑧"></a>

<a id="ref-for-typedef-custom-property-name"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-typedef-style-feature-value①"></a>

<a id="typedef-scroll-state-query"></a>

<a id="ref-for-typedef-scroll-state-query①"></a>

<a id="ref-for-typedef-scroll-state-in-parens"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-typedef-scroll-state-in-parens①"></a>

<a id="ref-for-typedef-scroll-state-in-parens②"></a>

<a id="ref-for-mult-zero-plus④"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-typedef-scroll-state-in-parens③"></a>

<a id="ref-for-mult-zero-plus⑤"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-scroll-state-feature"></a>

<a id="typedef-scroll-state-in-parens"></a>

<a id="ref-for-typedef-scroll-state-in-parens④"></a>

<a id="ref-for-typedef-scroll-state-query②"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-scroll-state-feature①"></a>

<a id="ref-for-comb-one③⓪"></a>

```text
<container-condition> = [ <container-name>? <container-query>? ]!
<container-name> = <custom-ident>
<container-query> = not <query-in-parens>
          | <query-in-parens> [ [ and <query-in-parens> ]* | [ or <query-in-parens> ]* ]
<query-in-parens> = ( <container-query> )
          | ( <size-feature> )
          | style( <style-query> )
          | scroll-state( <scroll-state-query> )
          | <general-enclosed>

<style-query>     = not <style-in-parens>
          | <style-in-parens> [ [ and <style-in-parens> ]* | [ or <style-in-parens> ]* ]
          | <style-feature>
<style-in-parens> = ( <style-query> )
          | ( <style-feature> )
          | <general-enclosed>
<style-feature> = <style-feature-plain> | <style-feature-boolean> | <style-range>
<style-feature-plain> = <style-feature-name> : <style-feature-value>
<style-feature-boolean> = <style-feature-name>
<style-range> = <style-range-value> <mf-comparison> <style-range-value>
    | <style-range-value> <mf-lt> <style-range-value> <mf-lt> <style-range-value>
    | <style-range-value> <mf-gt> <style-range-value> <mf-gt> <style-range-value>
<style-range-value> = <custom-property-name> | <style-feature-value>

<scroll-state-query>     = not <scroll-state-in-parens>
             | <scroll-state-in-parens> [ [ and <scroll-state-in-parens> ]* | [ or <scroll-state-in-parens> ]* ]
             | <scroll-state-feature>
<scroll-state-in-parens> = ( <scroll-state-query> )
             | ( <scroll-state-feature> )
             | <general-enclosed>
```
Tests

- [container-ident-function.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-ident-function.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-ident-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-ident-function.html)
- [container-ident-function.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-ident-function.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-ident-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-ident-function.html)
- [multiple-size-containers-comma-separated-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/multiple-size-containers-comma-separated-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/multiple-size-containers-comma-separated-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/multiple-size-containers-comma-separated-queries.html)
- [multiple-style-containers-comma-separated-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/multiple-style-containers-comma-separated-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/multiple-style-containers-comma-separated-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/multiple-style-containers-comma-separated-queries.html)
- [query-container-name.html](https://wpt.fyi/results/css/css-conditional/container-queries/query-container-name.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/query-container-name.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/query-container-name.html)
- [at-container-scrolled-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html)
- [multiple-scroll-state-containers-comma-separated-queries.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/multiple-scroll-state-containers-comma-separated-queries.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/multiple-scroll-state-containers-comma-separated-queries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/multiple-scroll-state-containers-comma-separated-queries.html)
- [scroll-state-query-with-var.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-query-with-var.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-query-with-var.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-query-with-var.html)
- [size-query-with-var.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-query-with-var.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-query-with-var.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-query-with-var.html)

<a id="ref-for-valdef-container-name-none②"></a>

<a id="ref-for-valdef-media-not①"></a>

<a id="ref-for-identifier-value④"></a>

The keywords [none](#valdef-container-name-none), and, [not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and or are excluded from the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) above.

<a id="ref-for-query-container①⑦"></a>

<a id="ref-for-container-feature"></a>

<a id="ref-for-typedef-container-query③"></a>

<a id="ref-for-typedef-container-condition②"></a>

<a id="ref-for-typedef-container-name②"></a>

<a id="ref-for-query-container-name②"></a>

For each element, the [query container](#query-container) to be queried is selected from among the element’s ancestor <a id="ref-for-query-container①⑧"></a>query containers that are established as a valid <a id="ref-for-query-container①⑨"></a>query container for all the [container features](#container-feature) in the [\<container-query\>](#typedef-container-query). If the <a id="ref-for-typedef-container-query④"></a>\<container-query\> contains unknown or unsupported <a id="ref-for-container-feature①"></a>container features, no <a id="ref-for-query-container②⓪"></a>query container will be selected for that [\<container-condition\>](#typedef-container-condition). The [\<container-name\>](#typedef-container-name) filters the set of <a id="ref-for-query-container②①"></a>query containers considered to just those with a matching [query container name](#query-container-name).

<a id="ref-for-query-container②②"></a>

<a id="ref-for-container-feature②"></a>

<a id="ref-for-typedef-container-query⑤"></a>

<a id="ref-for-container-query③"></a>

<a id="ref-for-typedef-container-name③"></a>

Once an eligible [query container](#query-container) has been selected for an element, each [container feature](#container-feature) in the [\<container-query\>](#typedef-container-query) is evaluated against that <a id="ref-for-query-container②③"></a>query container. If no ancestor is an eligible <a id="ref-for-query-container②④"></a>query container, then the [container query](#container-query) is unknown for that element. As with media queries, \<general-enclosed\> evaluates to unknown. If the <a id="ref-for-typedef-container-query⑥"></a>\<container-query\> is omitted, the <a id="ref-for-query-container②⑤"></a>query container is eligible as long as the [\<container-name\>](#typedef-container-name) matches.

<a id="ref-for-container-query④"></a>

<a id="ref-for-typedef-container-condition③"></a>

<a id="ref-for-query-container②⑥"></a>

<a id="ref-for-valdef-custom-media-true"></a>

<a id="ref-for-valdef-custom-media-false"></a>

If a [container query](#container-query) includes multiple [\<container-condition\>](#typedef-container-condition)s, each condition will select it’s own [query container](#query-container), and evaluate independently. A <a id="ref-for-container-query⑤"></a>container query is [true](https://www.w3.org/TR/mediaqueries-5/#valdef-custom-media-true) if <em>any</em> of its component <a id="ref-for-typedef-container-condition④"></a>\<container-condition\>s are <a id="ref-for-valdef-custom-media-true①"></a>true, and [false](https://www.w3.org/TR/mediaqueries-5/#valdef-custom-media-false) only if <em>all</em> of its component <a id="ref-for-typedef-container-condition⑤"></a>\<container-condition\>s are <a id="ref-for-valdef-custom-media-false①"></a>false.

<a id="ref-for-media-query①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cba8e10c"></a> As with [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query), we can string together multiple queries in a single condition:
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
>
> We can also combine multiple conditions into a list, with each condition evaluating against a different container:
>
> ```css
> @container card (inline-size > 30em), style(--large: true) {
>   /* styles */
> }
> ```
>
> <a id="ref-for-descdef-container-inline-size①"></a>
>
> <a id="ref-for-container-style-query⑤"></a>
>
> The styles above will be applied if there is an ancestor container named "card" that meets the [inline-size](#descdef-container-inline-size) condition <em>or</em> the nearest style container meets the [style](#container-style-query) condition.

<a id="ref-for-container-query⑥"></a>

Style rules defined on an element inside multiple nested [container queries](#container-query) apply when all of the wrapping <a id="ref-for-container-query⑦"></a>container queries are true for that element.

<a id="ref-for-container-query⑧"></a>

<a id="ref-for-typedef-container-condition⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Nested [container queries](#container-query) can evaluate in relation to different containers, so it is not always possible to merge the individual [\<container-condition\>](#typedef-container-condition)s into a single query.

<a id="ref-for-container-query⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b320e2d0"></a> Using a single comma-separated [container query](#container-query), we can query multiple containers:
>
> ```css
> @container card (inline-size > 30em), style(--responsive: true) {
>   /* styles */
> }
> ```
>
> <a id="ref-for-descdef-container-inline-size②"></a>
>
> <a id="ref-for-container-style-query⑥"></a>
>
> The styles above will apply for an element inside <em>either</em> a container named "card" that meets the [inline-size](#descdef-container-inline-size) condition, <em>or</em> a container meeting the [style](#container-style-query) condition.
>
> In order to require that <em>all</em> conditions are met while querying multiple containers, we would need to nest multiple queries:
>
> ```css
> @container card (inline-size > 30em) {
>   @container style(--responsive: true) {
>   /* styles */
>   }
> }
> ```
>
> <a id="ref-for-descdef-container-inline-size③"></a>
>
> <a id="ref-for-container-style-query⑦"></a>
>
> The styles above will only be applied if there is <em>both</em> an ancestor container named "card" that meets the [inline-size](#descdef-container-inline-size) condition, <em>and</em> an ancestor container meeting the [style](#container-style-query) condition.

<a id="ref-for-at-rule③"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-at-font-face-rule"></a>

<a id="ref-for-at-ruledef-layer"></a>

<a id="ref-for-container-query①⓪"></a>

Global, name-defining [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule) such as [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) or [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) or [@layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer) that are defined inside [container queries](#container-query) are not constrained by the <a id="ref-for-container-query①①"></a>container query conditions.

### <a id="animated-containers"></a>5.5.  Animated Containers

<a id="ref-for-container-query①②"></a>

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
>   background-color: skyblue;
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
>   <div id=inner>Inner</div>
>   </div>
>   <div id=sibling>Sibling</div>
> </main>
> ```
<a id="ref-for-computed-value①"></a>

<a id="ref-for-container-query-length"></a>

<a id="ref-for-style-change-event②"></a>

Changes in [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) caused by [container query length](#container-query-length) units must also be part of a [style change event](https://www.w3.org/TR/css-transitions-1/#style-change-event).

## <a id="container-features"></a>6.  Container Features

<a id="ref-for-query-container②⑦"></a>

A <a id="container-feature"></a>container feature queries a specific aspect of a [query container](#query-container).

<a id="ref-for-container-feature③"></a>

<a id="ref-for-media-feature"></a>

<a id="ref-for-boolean-context"></a>

[Container features](#container-feature) use the same rules as [media features](https://www.w3.org/TR/mediaqueries-5/#media-feature) when evaluating in a [boolean context](https://www.w3.org/TR/mediaqueries-5/#boolean-context).

### <a id="size-container"></a>6.1.  Size Container Features

<a id="ref-for-query-container②⑧"></a>

<a id="ref-for-principal-box②"></a>

<a id="ref-for-typedef-size-feature①"></a>

<a id="ref-for-typedef-size-feature②"></a>

<a id="ref-for-media-feature①"></a>

<a id="ref-for-size-features"></a>

<a id="ref-for-container-size-query⑤"></a>

<a id="ref-for-css-feature-queries"></a>

<a id="ref-for-at-ruledef-supports④"></a>

A <a id="container-size-query"></a>container size query allows querying the size of the [query container](#query-container)’s [principal box](https://www.w3.org/TR/css-display-4/#principal-box). It is a boolean combination of individual <a id="size-features"></a>size features ([\<size-feature\>](#typedef-size-feature)) that each query a single, specific dimensional feature of the <a id="ref-for-query-container②⑨"></a>query container. The syntax of a <a id="typedef-size-feature"></a>[\<size-feature\>](#typedef-size-feature) is the same as for a [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value. [\[mediaqueries-5\]](#biblio-mediaqueries-5) The boolean syntax and logic combining [size features](#size-features) into a [size query](#container-size-query) is the same as for [CSS feature queries](https://www.w3.org/TR/css-conditional-3/#css-feature-queries). (See [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). [\[css-conditional-3\]](#biblio-css-conditional-3))

Tests

- [size-container-auto-height.html](https://wpt.fyi/results/css/css-conditional/container-queries/size-container-auto-height.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/size-container-auto-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/size-container-auto-height.html)
- [var-evaluation.html](https://wpt.fyi/results/css/css-conditional/container-queries/var-evaluation.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/var-evaluation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/var-evaluation.html)

<a id="ref-for-query-container③⓪"></a>

<a id="ref-for-principal-box③"></a>

<a id="ref-for-layout-containment-box"></a>

<a id="ref-for-container-size-query⑥"></a>

<a id="ref-for-size-features①"></a>

If the [query container](#query-container) does not have a [principal box](https://www.w3.org/TR/css-display-4/#principal-box), or the principal box is not a [layout containment box](https://www.w3.org/TR/css-contain-2/#layout-containment-box), or the <a id="ref-for-query-container③①"></a>query container does not support [container size queries](#container-size-query) on the relevant axes, then the result of evaluating the [size feature](#size-features) is unknown.

<a id="ref-for-relative-length"></a>

<a id="ref-for-container-query-length①"></a>

<a id="ref-for-custom-property"></a>

<a id="ref-for-container-query①③"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-query-container③②"></a>

[Relative length](https://www.w3.org/TR/css-values-4/#relative-length) units (including [container query length](#container-query-length) units) and [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) in [container query](#container-query) conditions are evaluated based on the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [query container](#query-container).

Tree counting functions ([CSS Values 5 § 9 Tree Counting Functions: the sibling-count() and sibling-index() notations](https://www.w3.org/TR/css-values-5/#tree-counting)) are evaluated against the container element.

<a id="ref-for-media-query②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from the handling of relative units in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

<a id="ref-for-custom-property①"></a>

<a id="ref-for-size-features②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) substitution results in an invalid value for the [size feature](#size-features), it is handled the same as other invalid feature values, and the result of the <a id="ref-for-size-features③"></a>size feature is unknown.

<a id="ref-for-query-container③③"></a>

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
> <a id="ref-for-query-container③④"></a>
>
> The 40em value used in the query condition is relative to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) on the relevant [query container](#query-container):
>
> - For any h2 inside aside, the query condition will be true above 640px.
>
> - For any h2 inside main, the query condition will be true above 960px.

<a id="ref-for-query-container③⑤"></a>

<a id="ref-for-funcdef-var"></a>

<a id="ref-for-computed-value④"></a>

<a id="ref-for-custom-property②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5df63e7c"></a> Similarly, [query containers](#query-container) will evaluate [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var)-based queries relative to their own [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [custom property](https://www.w3.org/TR/css-variables-1/#custom-property):
>
> ```css
> aside, main {
>   container-type: inline-size;
> }
> 
> aside { --query: 300px; }
> main { --query: 500px; }
> 
> @container (width > var(--query)) {
>   h2 { font-size: 1.5em; }
> }
> ```
>
> <a id="ref-for-computed-value⑤"></a>
>
> <a id="ref-for-custom-property③"></a>
>
> <a id="ref-for-query-container③⑥"></a>
>
> The var(--query) value used in the query condition is substituted with the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the --query [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) on the relevant [query container](#query-container):
>
> - For any h2 inside aside, the query condition will be true above 300px.
>
> - For any h2 inside main, the query condition will be true above 500px.

<a id="ref-for-descdef-container-width"></a>

#### <a id="width"></a>6.1.1.  Width: the [width](#descdef-container-width) feature

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-width"></a>width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container⑤"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

range

<a id="ref-for-descdef-container-width①"></a>

<a id="ref-for-container-feature④"></a>

<a id="ref-for-width"></a>

<a id="ref-for-query-container③⑦"></a>

<a id="ref-for-content-box"></a>

The [width](#descdef-container-width) [container feature](#container-feature) queries the [width](https://www.w3.org/TR/css-sizing-3/#width) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-descdef-container-height"></a>

#### <a id="height"></a>6.1.2.  Height: the [height](#descdef-container-height) feature

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-height"></a>height

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container⑥"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value①"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

range

<a id="ref-for-descdef-container-height①"></a>

<a id="ref-for-container-feature⑤"></a>

<a id="ref-for-height"></a>

<a id="ref-for-query-container③⑧"></a>

<a id="ref-for-content-box①"></a>

The [height](#descdef-container-height) [container feature](#container-feature) queries the [height](https://www.w3.org/TR/css-sizing-3/#height) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-descdef-container-inline-size④"></a>

#### <a id="inline-size"></a>6.1.3.  Inline-size: the [inline-size](#descdef-container-inline-size) feature

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-inline-size"></a>inline-size

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container⑦"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value②"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

range

<a id="ref-for-descdef-container-inline-size⑤"></a>

<a id="ref-for-container-feature⑥"></a>

<a id="ref-for-size"></a>

<a id="ref-for-query-container③⑨"></a>

<a id="ref-for-content-box②"></a>

<a id="ref-for-inline-axis②"></a>

The [inline-size](#descdef-container-inline-size) [container feature](#container-feature) queries the [size](https://www.w3.org/TR/css-sizing-3/#size) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④⓪"></a>query container’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="ref-for-descdef-container-block-size"></a>

#### <a id="block-size"></a>6.1.4.  Block-size: the [block-size](#descdef-container-block-size) feature

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-block-size"></a>block-size

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container⑧"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value③"></a>

[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

range

<a id="ref-for-descdef-container-block-size①"></a>

<a id="ref-for-container-feature⑦"></a>

<a id="ref-for-size①"></a>

<a id="ref-for-query-container④①"></a>

<a id="ref-for-content-box③"></a>

<a id="ref-for-block-axis①"></a>

The [block-size](#descdef-container-block-size) [container feature](#container-feature) queries the [size](https://www.w3.org/TR/css-sizing-3/#size) of the [query container](#query-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-query-container④②"></a>query container’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

<a id="ref-for-descdef-container-aspect-ratio"></a>

#### <a id="aspect-ratio"></a>6.1.5.  Aspect-ratio: the [aspect-ratio](#descdef-container-aspect-ratio) feature

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-aspect-ratio"></a>aspect-ratio

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container⑨"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-ratio-value"></a>

[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

range

<a id="ref-for-descdef-container-aspect-ratio①"></a>

<a id="ref-for-container-feature⑧"></a>

<a id="ref-for-descdef-container-width②"></a>

<a id="ref-for-descdef-container-height②"></a>

The [aspect-ratio](#descdef-container-aspect-ratio) [container feature](#container-feature) is defined as the ratio of the value of the [width](#descdef-container-width) <a id="ref-for-container-feature⑨"></a>container feature to the value of the [height](#descdef-container-height) <a id="ref-for-container-feature①⓪"></a>container feature.

<a id="ref-for-descdef-container-orientation"></a>

#### <a id="orientation"></a>6.1.6.  Orientation: the [orientation](#descdef-container-orientation) feature

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-orientation"></a>orientation

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container①⓪"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one③①"></a>

portrait [\|](https://www.w3.org/TR/css-values-4/#comb-one) landscape

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

discrete

<a id="valdef-container-orientation-portrait"></a>portrait  
<a id="ref-for-descdef-container-width③"></a>

<a id="ref-for-descdef-container-height③"></a>

<a id="ref-for-valdef-container-orientation-portrait"></a>

<a id="ref-for-container-feature①①"></a>

<a id="ref-for-descdef-container-orientation①"></a>

The [orientation](#descdef-container-orientation) [container feature](#container-feature) is [portrait](#valdef-container-orientation-portrait) when the value of the [height](#descdef-container-height) <a id="ref-for-container-feature①②"></a>container feature is greater than or equal to the value of the [width](#descdef-container-width) <a id="ref-for-container-feature①③"></a>container feature.

<a id="valdef-container-orientation-landscape"></a>landscape  
<a id="ref-for-valdef-container-orientation-landscape"></a>

<a id="ref-for-descdef-container-orientation②"></a>

Otherwise [orientation](#descdef-container-orientation) is [landscape](#valdef-container-orientation-landscape).

### <a id="style-container"></a>6.2.  Style Container Features

<a id="ref-for-computed-value⑥"></a>

<a id="ref-for-query-container④③"></a>

<a id="ref-for-typedef-style-feature③"></a>

<a id="ref-for-declaration"></a>

<a id="ref-for-typedef-style-feature-name②"></a>

<a id="ref-for-typedef-style-range②"></a>

<a id="ref-for-typedef-style-feature-name③"></a>

<a id="ref-for-supported-css-property"></a>

<a id="ref-for-typedef-custom-property-name①"></a>

<a id="ref-for-typedef-style-feature-value②"></a>

<a id="ref-for-typedef-declaration-value"></a>

<a id="ref-for-typedef-mf-lt②"></a>

<a id="ref-for-typedef-mf-gt②"></a>

<a id="ref-for-typedef-mf-eq"></a>

A <a id="container-style-query"></a>container style query allows querying the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [query container](#query-container). It is a boolean combination of individual <a id="style-features"></a>style features ([\<style-feature\>](#typedef-style-feature)) that each query a single, specific property of the <a id="ref-for-query-container④④"></a>query container. The syntax of a <a id="ref-for-typedef-style-feature④"></a>\<style-feature\> is either the same as for a valid [declaration](https://www.w3.org/TR/css-syntax-3/#declaration)[\[CSS-SYNTAX-3\]](#biblio-css-syntax-3), a [\<style-feature-name\>](#typedef-style-feature-name) or a valid <a id="style-range"></a>style range([\<style-range\>](#typedef-style-range)). The <a id="typedef-style-feature-name"></a>[\<style-feature-name\>](#typedef-style-feature-name) can be either a [supported CSS property](https://www.w3.org/TR/cssom-1/#supported-css-property) or a valid [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name). The <a id="typedef-style-feature-value"></a>[\<style-feature-value\>](#typedef-style-feature-value) production matches any valid [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) as long as it doesn’t contain [\<mf-lt\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-lt), [\<mf-gt\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-gt) and [\<mf-eq\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-eq) tokens.

Tests

- [style-query-registered-custom-rem-change.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-query-registered-custom-rem-change.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-query-registered-custom-rem-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-query-registered-custom-rem-change.html)
- [style-query-unset-on-root.html](https://wpt.fyi/results/css/css-conditional/container-queries/style-query-unset-on-root.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/style-query-unset-on-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/style-query-unset-on-root.html)

<a id="ref-for-typedef-style-feature-plain②"></a>

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-query-container④⑤"></a>

A [\<style-feature-plain\>](#typedef-style-feature-plain) evaluates to true if the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the given property on the [query container](#query-container) matches the given value (which is also <a id="ref-for-computed-value⑧"></a>computed with respect to the <a id="ref-for-query-container④⑥"></a>query container), and false otherwise.

<a id="ref-for-style-features"></a>

<a id="ref-for-typedef-style-feature-boolean②"></a>

<a id="ref-for-computed-value⑨"></a>

<a id="ref-for-initial-value①"></a>

<a id="ref-for-css-property"></a>

A [style feature](#style-features) without a value ([\<style-feature-boolean\>](#typedef-style-feature-boolean)) evaluates to true if the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is different from the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) for the given [property](https://www.w3.org/TR/css-cascade-5/#css-property).

<a id="ref-for-typedef-style-range③"></a>

To <a id="evaluate-a-style-range"></a>evaluate a [\<style-range\>](#typedef-style-range), the following steps needs to be performed:

1.  <a id="ref-for-typedef-style-range-value⑨"></a>

    <a id="ref-for-typedef-custom-property-name②"></a>

    <a id="ref-for-funcdef-var①"></a>

    If [\<style-range-value\>](#typedef-style-range-value) is a [\<custom-property-name\>](https://www.w3.org/TR/css-variables-1/#typedef-custom-property-name), it needs to be substituted as if the <a id="ref-for-typedef-custom-property-name③"></a>\<custom-property-name\> was wrapped inside a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var).

2.  <a id="ref-for-arbitrary-substitution-function"></a>

    <a id="ref-for-typedef-style-range-value①⓪"></a>

    Substitute [arbitrary substitution function](https://www.w3.org/TR/css-values-5/#arbitrary-substitution-function) within [\<style-range-value\>](#typedef-style-range-value).

3.  <a id="ref-for-typedef-style-range-value①①"></a>

    <a id="ref-for-number-value"></a>

    <a id="ref-for-percentage-value"></a>

    <a id="ref-for-length-value④"></a>

    <a id="ref-for-angle-value"></a>

    <a id="ref-for-time-value"></a>

    <a id="ref-for-frequency-value"></a>

    <a id="ref-for-resolution-value"></a>

    Parse [\<style-range-value\>](#typedef-style-range-value) to [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), [\<time\>](https://www.w3.org/TR/css-values-4/#time-value), [\<frequency\>](https://www.w3.org/TR/css-values-4/#frequency-value) or [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value). If this cannot be done, evaluate to false.

4.  <a id="ref-for-typedef-style-range-value①②"></a>

    <a id="ref-for-length-value⑤"></a>

    If each [\<style-range-value\>](#typedef-style-range-value) from the range have the same type, compute each and evaluate the comparison; a unitless zero must be treated as being a zero-[\<length\>](https://www.w3.org/TR/css-values-4/#length-value) when compared against a <a id="ref-for-length-value⑥"></a>\<length\> .

5.  Otherwise evaluate to false.

<a id="ref-for-style-features①"></a>

<a id="ref-for-container-style-query⑧"></a>

<a id="ref-for-css-feature-queries①"></a>

<a id="ref-for-at-ruledef-supports⑤"></a>

The boolean syntax and logic combining [style features](#style-features) into a [style query](#container-style-query) is the same as for [CSS feature queries](https://www.w3.org/TR/css-conditional-3/#css-feature-queries). (See [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports). [\[css-conditional-3\]](#biblio-css-conditional-3))

<a id="ref-for-style-features②"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-longhand"></a>

[Style features](#style-features) that query a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are true if the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) match for each of its [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand), and false otherwise.

<a id="ref-for-cascade-dependent-keyword"></a>

<a id="ref-for-valdef-all-revert"></a>

<a id="ref-for-valdef-all-revert-layer"></a>

<a id="ref-for-style-features③"></a>

<a id="ref-for-container-style-query⑨"></a>

[Cascade-dependent keywords](https://drafts.csswg.org/css-cascade-5/#cascade-dependent-keyword), such as [revert](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert) and [revert-layer](https://www.w3.org/TR/css-cascade-5/#valdef-all-revert-layer), are invalid as values in a [style feature](#style-features), and cause the [container style query](#container-style-query) to be false.

<a id="ref-for-css-wide-keywords"></a>

<a id="ref-for-computed-value①①"></a>

<a id="ref-for-query-container④⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The remaining non-cascade-dependent [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) are [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) with respect to the [query container](#query-container), the same as other values.

### <a id="scroll-state-container"></a>6.3.  Scroll State Container Features

<a id="ref-for-typedef-scroll-state-feature②"></a>

<a id="ref-for-query-container④⑧"></a>

<a id="ref-for-typedef-scroll-state-feature③"></a>

<a id="ref-for-media-feature②"></a>

A <a id="container-scroll-state-query"></a>container scroll-state query allows querying a container for state that depends on scroll position. It is a boolean combination of individual <a id="scroll-state-feature"></a>scroll-state features ([\<scroll-state-feature\>](#typedef-scroll-state-feature)) that each query a single feature of the [query container](#query-container). The syntax of a <a id="typedef-scroll-state-feature"></a>[\<scroll-state-feature\>](#typedef-scroll-state-feature) is the same as for a [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature): a feature name, a comparator, and a value.

<a id="ref-for-scroll-state-feature"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-scrollport"></a>

[Scroll-state features](#scroll-state-feature) can either match state of the scroller itself, or an element that is affected by the scroll position of an ancestor [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport). An example of the former is the scrollable feature, snapped the latter.

#### <a id="updating-scroll-state"></a>6.3.1.  Updating Scroll State

Scroll state may cause layout cycles since queried scroll state may cause style changes, which may lead to scroll state changes as a result of layout.

<a id="ref-for-valdef-container-type-scroll-state"></a>

<a id="ref-for-query-container④⑨"></a>

<a id="ref-for-run-snapshot-post-layout-state-steps"></a>

To avoid such layout cycles, [scroll-state](#valdef-container-type-scroll-state) [query containers](#query-container) update their current state as part of [run snapshot post-layout state steps](https://www.w3.org/TR/cssom-view-1/#run-snapshot-post-layout-state-steps) which is only run at specific points in the [HTML event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model).

<a id="ref-for-run-snapshot-post-layout-state-steps①"></a>

<a id="ref-for-valdef-container-type-scroll-state①"></a>

<a id="ref-for-query-container⑤⓪"></a>

When asked to [run snapshot post-layout state steps](https://www.w3.org/TR/cssom-view-1/#run-snapshot-post-layout-state-steps), update the current state of every [scroll-state](#valdef-container-type-scroll-state) [query container](#query-container). This snapshotted state will be used for any style and layout updates until the next time these steps are run.

<a id="ref-for-descdef-container-stuck"></a>

#### <a id="stuck"></a>6.3.2.  Sticky positioning: the [stuck](#descdef-container-stuck) feature

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-stuck"></a>stuck

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container①①"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one③②"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one③③"></a>\| right <a id="ref-for-comb-one③④"></a>\| bottom <a id="ref-for-comb-one③⑤"></a>\| left <a id="ref-for-comb-one③⑥"></a>\| block-start <a id="ref-for-comb-one③⑦"></a>\| inline-start <a id="ref-for-comb-one③⑧"></a>\| block-end <a id="ref-for-comb-one③⑨"></a>\| inline-end

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [scroll-state-stuck-pseudo.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-pseudo.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-stuck-pseudo.html)

<a id="ref-for-descdef-container-stuck①"></a>

<a id="ref-for-container-feature①④"></a>

<a id="ref-for-valdef-position-sticky"></a>

<a id="ref-for-sticky-view-rectangle"></a>

<a id="ref-for-query-container⑤①"></a>

<a id="ref-for-sticky-position"></a>

The [stuck](#descdef-container-stuck) [container feature](#container-feature) queries whether a [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) positioned container is visually shifted to stay inside the [sticky view rectangle](https://www.w3.org/TR/css-position-3/#sticky-view-rectangle) for the given edge. The logical edges map to physical based on the direction and writing-mode of the [query container](#query-container). None of the values match if the <a id="ref-for-query-container⑤②"></a>query container is not [sticky positioned](https://www.w3.org/TR/css-position-3/#sticky-position).

It is possible for two values from opposite axes to match at the same time, but not for opposite edges along the same axis.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5e3347d6"></a> May match:
>
> ```css
> @container scroll-state((stuck: top) and (stuck: left)) { ... }
> ```
>
> Will never match:
>
> ```css
> @container scroll-state((stuck: left) and (stuck: right)) { ... }
> ```
<a id="valdef-container-stuck-none"></a>none  
<a id="ref-for-valdef-position-sticky①"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is not shifted in any direction.

<a id="valdef-container-stuck-top"></a>top  
<a id="ref-for-valdef-position-sticky②"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the top edge.

<a id="valdef-container-stuck-right"></a>right  
<a id="ref-for-valdef-position-sticky③"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the right edge.

<a id="valdef-container-stuck-bottom"></a>bottom  
<a id="ref-for-valdef-position-sticky④"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the bottom edge.

<a id="valdef-container-stuck-left"></a>left  
<a id="ref-for-valdef-position-sticky⑤"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the left edge.

<a id="valdef-container-stuck-block-start"></a>block-start  
<a id="ref-for-block-start"></a>

<a id="ref-for-valdef-position-sticky⑥"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge.

<a id="valdef-container-stuck-inline-start"></a>inline-start  
<a id="ref-for-inline-start"></a>

<a id="ref-for-valdef-position-sticky⑦"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge.

<a id="valdef-container-stuck-block-end"></a>block-end  
<a id="ref-for-block-end"></a>

<a id="ref-for-valdef-position-sticky⑧"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge.

<a id="valdef-container-stuck-inline-end"></a>inline-end  
<a id="ref-for-inline-end"></a>

<a id="ref-for-valdef-position-sticky⑨"></a>

The [sticky](https://www.w3.org/TR/css-position-3/#valdef-position-sticky) container is shifted to stay inside the [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge.

<a id="ref-for-descdef-container-snapped"></a>

#### <a id="snapped"></a>6.3.3.  Scroll snapping: the [snapped](#descdef-container-snapped) feature

<strong>Table 11 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-snapped"></a>snapped

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container①②"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one④⓪"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) x <a id="ref-for-comb-one④①"></a>\| y <a id="ref-for-comb-one④②"></a>\| block <a id="ref-for-comb-one④③"></a>\| inline <a id="ref-for-comb-one④④"></a>\| both

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [scroll-state-snapped-both.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-both.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-both.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-both.html)
- [scroll-state-snapped-pseudo.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-pseudo.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-snapped-pseudo.html)

<a id="ref-for-descdef-container-snapped①"></a>

<a id="ref-for-container-feature①⑤"></a>

<a id="ref-for-snap-target"></a>

<a id="ref-for-scroll-snap-container"></a>

<a id="ref-for-eventdef-snapevent-scrollsnapchanging"></a>

The [snapped](#descdef-container-snapped) [container feature](#container-feature) queries whether a [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) is, or would be, snapped to its [scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container), in the given axis. That is, it matches any <a id="ref-for-snap-target①"></a>snap target that the <code><a href="https://www.w3.org/TR/css-scroll-snap-2/#eventdef-snapevent-scrollsnapchanging">scrollsnapchanging</a></code> event is fired for.

<a id="valdef-container-snapped-none"></a>none  
<a id="ref-for-snap-target②"></a>

<a id="ref-for-query-container⑤③"></a>

The [query container](#query-container) is not a [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target).

<a id="valdef-container-snapped-x"></a>x  
<a id="ref-for-scroll-container①"></a>

<a id="ref-for-snap-target③"></a>

<a id="ref-for-query-container⑤④"></a>

<a id="ref-for-valdef-container-snapped-x"></a>

<a id="ref-for-container-feature①⑥"></a>

<a id="ref-for-descdef-container-snapped②"></a>

[snapped](#descdef-container-snapped) [container feature](#container-feature) matches [x](#valdef-container-snapped-x) if the [query container](#query-container) is a horizontal [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<a id="valdef-container-snapped-y"></a>y  
<a id="ref-for-scroll-container②"></a>

<a id="ref-for-snap-target④"></a>

<a id="ref-for-query-container⑤⑤"></a>

<a id="ref-for-valdef-container-snapped-y"></a>

<a id="ref-for-container-feature①⑦"></a>

<a id="ref-for-descdef-container-snapped③"></a>

[snapped](#descdef-container-snapped) [container feature](#container-feature) matches [y](#valdef-container-snapped-y) if the [query container](#query-container) is a vertical [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<a id="valdef-container-snapped-block"></a>block  
<a id="ref-for-scroll-snap-container①"></a>

<a id="ref-for-scroll-container③"></a>

<a id="ref-for-snap-target⑤"></a>

<a id="ref-for-query-container⑤⑥"></a>

<a id="ref-for-valdef-container-snapped-block"></a>

<a id="ref-for-container-feature①⑧"></a>

<a id="ref-for-descdef-container-snapped④"></a>

[snapped](#descdef-container-snapped) [container feature](#container-feature) matches [block](#valdef-container-snapped-block) if the [query container](#query-container) is a [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container). in the block direction of the [scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

<a id="valdef-container-snapped-inline"></a>inline  
<a id="ref-for-scroll-snap-container②"></a>

<a id="ref-for-scroll-container④"></a>

<a id="ref-for-snap-target⑥"></a>

<a id="ref-for-query-container⑤⑦"></a>

<a id="ref-for-valdef-container-snapped-inline"></a>

<a id="ref-for-container-feature①⑨"></a>

<a id="ref-for-descdef-container-snapped⑤"></a>

[snapped](#descdef-container-snapped) [container feature](#container-feature) matches [inline](#valdef-container-snapped-inline) if the [query container](#query-container) is a [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in the inline direction of the [scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

<a id="valdef-container-snapped-both"></a>both  
<a id="ref-for-scroll-snap-container③"></a>

<a id="ref-for-scroll-container⑤"></a>

<a id="ref-for-snap-target⑦"></a>

<a id="ref-for-query-container⑤⑧"></a>

<a id="ref-for-valdef-container-snapped-both"></a>

<a id="ref-for-container-feature②⓪"></a>

<a id="ref-for-descdef-container-snapped⑥"></a>

[snapped](#descdef-container-snapped) [container feature](#container-feature) matches [both](#valdef-container-snapped-both) if the [query container](#query-container) is a [snap target](https://www.w3.org/TR/css-scroll-snap-2/#snap-target) for its [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in both directions of the [scroll snap container](https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-container).

<a id="ref-for-descdef-container-scrollable"></a>

#### <a id="scrollable"></a>6.3.4.  Scrollable: the [scrollable](#descdef-container-scrollable) feature

<strong>Table 12 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-scrollable"></a>scrollable

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container①③"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one④⑤"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one④⑥"></a>\| right <a id="ref-for-comb-one④⑦"></a>\| bottom <a id="ref-for-comb-one④⑧"></a>\| left <a id="ref-for-comb-one④⑨"></a>\| block-start <a id="ref-for-comb-one⑤⓪"></a>\| inline-start <a id="ref-for-comb-one⑤①"></a>\| block-end <a id="ref-for-comb-one⑤②"></a>\| inline-end <a id="ref-for-comb-one⑤③"></a>\| x <a id="ref-for-comb-one⑤④"></a>\| y <a id="ref-for-comb-one⑤⑤"></a>\| block <a id="ref-for-comb-one⑤⑥"></a>\| inline

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [scroll-state-scrollable-axis.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-axis.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-axis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-axis.html)
- [scroll-state-scrollable-body-001.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-001.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-001.html)
- [scroll-state-scrollable-body-002.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-002.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-body-002.html)
- [scroll-state-scrollable-layout-change-002.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change-002.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-layout-change-002.html)
- [scroll-state-scrollable-pseudo.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-pseudo.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-pseudo.html)
- [scroll-state-scrollable-root.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-root.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrollable-root.html)

<a id="ref-for-descdef-container-scrollable①"></a>

<a id="ref-for-container-feature②①"></a>

<a id="ref-for-scroll-container⑥"></a>

<a id="ref-for-scrollable-overflow-rectangle"></a>

<a id="ref-for-valdef-overflow-hidden"></a>

The [scrollable](#descdef-container-scrollable) [container feature](#container-feature) queries whether a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has clipped [scrollable overflow rectangle](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-rectangle) content in the given direction which is reachable through user initiated scrolling. That is, <a id="ref-for-descdef-container-scrollable②"></a>scrollable does not match for a [hidden](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-hidden) container, nor for a negative scrollable overflow region.

<a id="ref-for-query-container⑤⑨"></a>

<a id="ref-for-scroll-container⑦"></a>

The logical values map to physical based on the direction and writing-mode of the [query container](#query-container). None of the values match if the container is not a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

<a id="valdef-container-scrollable-none"></a>none  
<a id="ref-for-scrollable-overflow"></a>

<a id="ref-for-scroll-container⑧"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) does not have [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in any direction.

<a id="valdef-container-scrollable-top"></a>top  
<a id="ref-for-scrollable-overflow①"></a>

<a id="ref-for-scroll-container⑨"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the top edge.

<a id="valdef-container-scrollable-right"></a>right  
<a id="ref-for-scrollable-overflow②"></a>

<a id="ref-for-scroll-container①⓪"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the right edge.

<a id="valdef-container-scrollable-bottom"></a>bottom  
<a id="ref-for-scrollable-overflow③"></a>

<a id="ref-for-scroll-container①①"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the bottom edge.

<a id="valdef-container-scrollable-left"></a>left  
<a id="ref-for-scrollable-overflow④"></a>

<a id="ref-for-scroll-container①②"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the left edge.

<a id="valdef-container-scrollable-block-start"></a>block-start  
<a id="ref-for-block-start①"></a>

<a id="ref-for-scrollable-overflow⑤"></a>

<a id="ref-for-scroll-container①③"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge.

<a id="valdef-container-scrollable-inline-start"></a>inline-start  
<a id="ref-for-inline-start①"></a>

<a id="ref-for-scrollable-overflow⑥"></a>

<a id="ref-for-scroll-container①④"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) edge.

<a id="valdef-container-scrollable-block-end"></a>block-end  
<a id="ref-for-block-end①"></a>

<a id="ref-for-scrollable-overflow⑦"></a>

<a id="ref-for-scroll-container①⑤"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edge.

<a id="valdef-container-scrollable-inline-end"></a>inline-end  
<a id="ref-for-inline-end①"></a>

<a id="ref-for-scrollable-overflow⑧"></a>

<a id="ref-for-scroll-container①⑥"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) past the [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) edge.

<a id="valdef-container-scrollable-x"></a>x  
<a id="ref-for-scrollable-overflow⑨"></a>

<a id="ref-for-scroll-container①⑦"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has horizontally [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow).

<a id="valdef-container-scrollable-y"></a>y  
<a id="ref-for-scrollable-overflow①⓪"></a>

<a id="ref-for-scroll-container①⑧"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has vertically [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow).

<a id="valdef-container-scrollable-block"></a>block  
<a id="ref-for-scrollable-overflow①①"></a>

<a id="ref-for-scroll-container①⑨"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in its block direction.

<a id="valdef-container-scrollable-inline"></a>inline  
<a id="ref-for-scrollable-overflow①②"></a>

<a id="ref-for-scroll-container②⓪"></a>

The [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) has [scrollable overflow](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow) in its inline direction.

<a id="ref-for-descdef-container-scrolled"></a>

#### <a id="scrolled"></a>6.3.5.  Scrolled: the [scrolled](#descdef-container-scrolled) feature

<strong>Table 13 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="descdef-container-scrolled"></a>scrolled

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

For:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-at-ruledef-container①④"></a>

[@container](#at-ruledef-container)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

Value:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one⑤⑦"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) top <a id="ref-for-comb-one⑤⑧"></a>\| right <a id="ref-for-comb-one⑤⑨"></a>\| bottom <a id="ref-for-comb-one⑥⓪"></a>\| left <a id="ref-for-comb-one⑥①"></a>\| block-start <a id="ref-for-comb-one⑥②"></a>\| inline-start <a id="ref-for-comb-one⑥③"></a>\| block-end <a id="ref-for-comb-one⑥④"></a>\| inline-end <a id="ref-for-comb-one⑥⑤"></a>\| x <a id="ref-for-comb-one⑥⑥"></a>\| y <a id="ref-for-comb-one⑥⑦"></a>\| block <a id="ref-for-comb-one⑥⑧"></a>\| inline

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

Type:

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [scroll-state-scrolled-arrow-key-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-arrow-key-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-arrow-key-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-arrow-key-scroll.html)
- [scroll-state-scrolled-home-end-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-home-end-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-home-end-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-home-end-scroll.html)
- [scroll-state-scrolled-hv.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-hv.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-hv.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-hv.html)
- [scroll-state-scrolled-keyboard-scroll-on-body.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-body.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-body.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-body.html)
- [scroll-state-scrolled-keyboard-scroll-on-root.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-root.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-keyboard-scroll-on-root.html)
- [scroll-state-scrolled-mouse-drag-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-mouse-drag-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-mouse-drag-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-mouse-drag-scroll.html)
- [scroll-state-scrolled-multiple-scrollers.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-multiple-scrollers.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-multiple-scrollers.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-multiple-scrollers.html)
- [scroll-state-scrolled-programmatic-absolute-scrolls.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-absolute-scrolls.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-absolute-scrolls.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-absolute-scrolls.html)
- [scroll-state-scrolled-programmatic-relative-scrolls.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-relative-scrolls.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-relative-scrolls.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-programmatic-relative-scrolls.html)
- [scroll-state-scrolled-pu-pd-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-pu-pd-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-pu-pd-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-pu-pd-scroll.html)
- [scroll-state-scrolled-scrollbar-button-clicks.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-button-clicks.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-button-clicks.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-button-clicks.html)
- [scroll-state-scrolled-scrollbar-track-clicks.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-track-clicks.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-track-clicks.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-scrollbar-track-clicks.html)
- [scroll-state-scrolled-spacebar-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-spacebar-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-spacebar-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-spacebar-scroll.html)
- [scroll-state-scrolled-user-touch-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-user-touch-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-user-touch-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-user-touch-scroll.html)
- [scroll-state-scrolled-wheel-scroll.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wheel-scroll.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wheel-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wheel-scroll.html)
- [scroll-state-scrolled-wm.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wm.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wm.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/scroll-state-scrolled-wm.html)
- [at-container-scrolled-parsing.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-scrolled-parsing.html)
- [at-container-scrolled-serialization.html](https://wpt.fyi/results/css/css-conditional/container-queries/scroll-state/at-container-scrolled-serialization.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/scroll-state/at-container-scrolled-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/scroll-state/at-container-scrolled-serialization.html)

<a id="ref-for-query-container⑥⓪"></a>

<a id="ref-for-scroll-container②①"></a>

<a id="ref-for-descdef-container-scrolled①"></a>

<a id="ref-for-container-feature②②"></a>

<a id="ref-for-relative-scroll"></a>

For a [query container](#query-container) that is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), the [scrolled](#descdef-container-scrolled) [container feature](#container-feature) queries the direction of its most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll). The logical values map to physical based on the direction and writing-mode of the <a id="ref-for-query-container⑥①"></a>query container. None of the values match if the container is not a <a id="ref-for-scroll-container②②"></a>scroll container.

<a id="valdef-container-scrolled-none"></a>none  
<a id="ref-for-relative-scroll①"></a>

<a id="ref-for-query-container⑥②"></a>

The [query container](#query-container) has not had a [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) yet.

<a id="valdef-container-scrolled-top"></a>top  
<a id="ref-for-relative-scroll②"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was upwards.

<a id="valdef-container-scrolled-right"></a>right  
<a id="ref-for-relative-scroll③"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was to the right.

<a id="valdef-container-scrolled-bottom"></a>bottom  
<a id="ref-for-relative-scroll④"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was downwards.

<a id="valdef-container-scrolled-left"></a>left  
<a id="ref-for-relative-scroll⑤"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was to the left.

<a id="valdef-container-scrolled-block-start"></a>block-start  
<a id="ref-for-block-start②"></a>

<a id="ref-for-relative-scroll⑥"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) direction.

<a id="valdef-container-scrolled-inline-start"></a>inline-start  
<a id="ref-for-inline-start②"></a>

<a id="ref-for-relative-scroll⑦"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) direction.

<a id="valdef-container-scrolled-block-end"></a>block-end  
<a id="ref-for-block-end②"></a>

<a id="ref-for-relative-scroll⑧"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) direction.

<a id="valdef-container-scrolled-inline-end"></a>inline-end  
<a id="ref-for-inline-end②"></a>

<a id="ref-for-relative-scroll⑨"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was towards the [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) direction.

<a id="valdef-container-scrolled-x"></a>x  
<a id="ref-for-relative-scroll①⓪"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the horizontal direction.

<a id="valdef-container-scrolled-y"></a>y  
<a id="ref-for-relative-scroll①①"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the vertical direction.

<a id="valdef-container-scrolled-block"></a>block  
<a id="ref-for-relative-scroll①②"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the block direction.

<a id="valdef-container-scrolled-inline"></a>inline  
<a id="ref-for-relative-scroll①③"></a>

The most recent [relative scroll](https://drafts.csswg.org/css-scroll-snap-1/#relative-scroll) was in the inline direction.

## <a id="container-lengths"></a>7.  Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units

<a id="ref-for-query-container⑥③"></a>

<a id="ref-for-container-query-length②"></a>

<a id="container-query-length"></a>Container query length units specify a length relative to the dimensions of a [query container](#query-container). Style sheets that use [container query length](#container-query-length) units can more easily move components from one <a id="ref-for-query-container⑥④"></a>query container to another.

<a id="ref-for-container-query-length③"></a>

The [container query length](#container-query-length) units are:

<strong>Table 14 — structured row/cell transcription</strong>

Informative Summary of Container Units

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

unit

<strong>Column 2 (header cell):</strong>

relative to

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

cqw

<strong>Column 2 (data cell):</strong>

<a id="ref-for-width①"></a>

<a id="ref-for-query-container⑥⑤"></a>

1% of a [query container](#query-container)’s [width](https://www.w3.org/TR/css-sizing-3/#width)

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

cqh

<strong>Column 2 (data cell):</strong>

<a id="ref-for-height①"></a>

<a id="ref-for-query-container⑥⑥"></a>

1% of a [query container](#query-container)’s [height](https://www.w3.org/TR/css-sizing-3/#height)

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

cqi

<strong>Column 2 (data cell):</strong>

<a id="ref-for-inline-size"></a>

<a id="ref-for-query-container⑥⑦"></a>

1% of a [query container](#query-container)’s [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size)

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

cqb

<strong>Column 2 (data cell):</strong>

<a id="ref-for-block-size"></a>

<a id="ref-for-query-container⑥⑧"></a>

1% of a [query container](#query-container)’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size)

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

cqmin

<strong>Column 2 (data cell):</strong>

The smaller value of cqi or cqb

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

cqmax

<strong>Column 2 (data cell):</strong>

The larger value of cqi or cqb

Tests

- [container-units-auto.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-auto.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-auto.html)
- [container-units-selection-pseudo.html](https://wpt.fyi/results/css/css-conditional/container-queries/container-units-selection-pseudo.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/container-units-selection-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/container-units-selection-pseudo.html)

<a id="ref-for-container-query-length④"></a>

<a id="ref-for-container-size-query⑦"></a>

<a id="ref-for-query-container⑥⑨"></a>

<a id="ref-for-small-viewport-size"></a>

For each element, [container query length](#container-query-length) units are evaluated as [container size queries](#container-size-query) on the relevant axis (or axes) described by the unit. The [query container](#query-container) for each axis is the nearest ancestor container that accepts <a id="ref-for-container-size-query⑧"></a>container size queries on that axis. If no eligible <a id="ref-for-query-container⑦⓪"></a>query container is available, then use the [small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) for that axis.

<a id="ref-for-query-container⑦①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In some cases cqi and cqb units on the same element will evaluate in relation to different [query containers](#query-container). Similarly, cqmin and cqmax units represent the larger or smaller of the cqi and cqb units, even when those dimensions come from different <a id="ref-for-query-container⑦②"></a>query containers.

<a id="ref-for-computed-value①②"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-container-query-length⑤"></a>

<a id="ref-for-query-container⑦③"></a>

<a id="ref-for-container-query①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a4252068"></a> Authors can ensure that [container query length](#container-query-length) units have an appropriate [query container](#query-container) by applying them inside a [container query](#container-query) that relies on the same container-type. Custom fallback values can be defined outside the <a id="ref-for-container-query①⑤"></a>container query:
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
<a id="ref-for-at-ruledef-supports-condition"></a>

## <a id="supports-condition-rule"></a>8.  Defining Custom Support Queries: the [@supports-condition](#at-ruledef-supports-condition) rule

<a id="ref-for-at-rule④"></a>

<a id="ref-for-conditional-group-rule①②"></a>

<a id="ref-for-supports-queries②"></a>

The <a id="at-ruledef-supports-condition"></a>@supports-condition [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) is a [conditional group rule](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) that allows authors to define and name a [supports query](https://www.w3.org/TR/css-conditional-3/#supports-queries) for later reuse, creating a <a id="named-supports-condition"></a>named supports condition. This enables complex or frequently-used feature queries to be referenced by name, improving maintainability and readability.

<a id="ref-for-typedef-supports-condition-name②"></a>

<a id="ref-for-typedef-block-contents②"></a>

```text
@supports-condition <supports-condition-name> {
  <block-contents>
}
```
Tests

- [at-supports-selector-details-content-before.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-details-content-before.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-details-content-before.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-details-content-before.html)
- [at-supports-selector-details-content.html](https://wpt.fyi/results/css/css-conditional/at-supports-selector-details-content.html) [(live test)](http://wpt.live/css/css-conditional/at-supports-selector-details-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/at-supports-selector-details-content.html)

<a id="ref-for-typedef-supports-condition-name③"></a>

<a id="ref-for-typedef-extension-name"></a>

Where <a id="typedef-supports-condition-name"></a>[\<supports-condition-name\>](#typedef-supports-condition-name) is an [\<extension-name\>](https://drafts.csswg.org/css-extensions-1/#typedef-extension-name) that defines the name of the supports query.

Anything inside the block is evaluated to test whether the user agent supports the features used. The contents do not have any effect on the document’s rendering.

<a id="ref-for-at-ruledef-supports⑥"></a>

<a id="ref-for-at-ruledef-when⑥"></a>

Once defined, the named supports condition can be used in subsequent [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) or [@when](#at-ruledef-when) conditions.

<a id="ref-for-at-ruledef-supports-condition①"></a>

If multiple [@supports-condition](#at-ruledef-supports-condition) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ebaec0db"></a> For example, we can define a supports query checking multiple properties at once:
>
> ```css
> @supports-condition --thicker-underlines {
>   text-decoration-thickness: 0.2em;
>   text-underline-offset: 0.3em;
> }
> 
> /* Equivalent to (text-decoration-thickness: 0.2em) and (text-underline-offset: 0.3em) */
> @supports (--thicker-underlines) {
>   a {
>     text-decoration: underline;
>     text-decoration-thickness: 0.2em;
>     text-underline-offset: 0.3em;
>   }
> }
> ```
<a id="ref-for-at-ruledef-supports-condition②"></a>

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-at-ruledef-namespace"></a>

<a id="ref-for-at-ruledef-charset①"></a>

[@supports-condition](#at-ruledef-supports-condition) rules are allowed before [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import) and [@namespace](https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace) rules (after the [@charset](https://www.w3.org/TR/css-syntax-3/#at-ruledef-charset) rule, if any).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63d89a93"></a> As support queries can contain arbitrary declarations, they can be used to detect support for complex features such as nesting:
>
> ```css
> @supports-condition --nesting {
>   & { }
> }
> 
> @import url("nested-styles.css") supports(--nesting);
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2391b075"></a> Support queries can also be used to detect support for at-rules:
>
> ```css
> @supports-condition --stuck-container-feature {
>   @container scroll-state(stuck: top) { }
> }
> 
> @supports (--stuck-container-feature) {
>   div { border-color: navy; }
> }
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4ac0bea1"></a> The name of the at-rule is under discussion. Alternatives include @supports-query, @supports-test, and @custom-supports. The name should be consistent with the one chosen for custom media queries.

## <a id="apis"></a>9. APIs

### <a id="the-csscontainerrule-interface"></a>9.1.  The `CSSContainerRule` interface

<a id="ref-for-csscontainerrule"></a>

<a id="ref-for-at-ruledef-container①⑤"></a>

The <code><a href="#csscontainerrule">CSSContainerRule</a></code> interface represents an [@container](#at-ruledef-container) rule.

<a id="ref-for-Exposed"></a>

<a id="csscontainerrule"></a>

<a id="ref-for-cssconditionrule"></a>

<a id="ref-for-cssomstring"></a>

<a id="dom-csscontainerrule-containername"></a>

<a id="ref-for-cssomstring①"></a>

<a id="dom-csscontainerrule-containerquery"></a>

```text
[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
  readonly attribute CSSOMString containerName;
  readonly attribute CSSOMString containerQuery;
};
```
`conditionText` of type `CSSOMString` (CSSContainerRule-specific definition for attribute on CSSConditionRule)

The `conditionText` attribute (defined on the `CSSConditionRule` parent rule), on getting, must return a value as follows:

<a id="ref-for-typedef-container-name④"></a>

<a id="ref-for-at-ruledef-container①⑥"></a>

The [@container](#at-ruledef-container) rule has an associated [\<container-name\>](#typedef-container-name)

The result of getting the `containerName` and `containerQuery` attributes, joined by a single whitespace.

Otherwise

The result of getting the `containerQuery` attribute.

`containerName` of type `CSSOMString`

The `containerName` attribute, on getting, must return a value as follows:

<a id="ref-for-typedef-container-name⑤"></a>

<a id="ref-for-at-ruledef-container①⑦"></a>

The [@container](#at-ruledef-container) rule has an associated [\<container-name\>](#typedef-container-name)

<a id="ref-for-typedef-container-name⑥"></a>

The result of serializing that [\<container-name\>](#typedef-container-name).

Otherwise

An empty string.

`containerQuery` of type `CSSOMString`

<a id="ref-for-typedef-container-query⑦"></a>

The `containerQuery` attribute, on getting, must return the [\<container-query\>](#typedef-container-query) that was specified, without any logical simplifications, so that the returned query will evaluate to the same result as the specified query in any conformant implementation of this specification (including implementations that implement future extensions allowed by the \<general-enclosed\> extensibility mechanism in this specification). In other words, token stream simplifications are allowed (such as reducing whitespace to a single space or omitting it in cases where it is known to be optional), but logical simplifications (such as removal of unneeded parentheses, or simplification based on evaluating results) are not allowed.

<a id="ref-for-dom-window-matchmedia"></a>

<a id="ref-for-mediaquerylist"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2e3d6538"></a> Container Queries should have a `matchContainer` method. This will be modeled on <code><a href="https://www.w3.org/TR/cssom-view-1/#dom-window-matchmedia">matchMedia()</a></code> and the <code><a href="https://www.w3.org/TR/cssom-view-1/#mediaquerylist">MediaQueryList</a></code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to `resizeObserver`, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205)

### <a id="the-csssupportscondition-interface"></a>9.2.  The `CSSSupportsConditionRule` interface

<a id="ref-for-csssupportsconditionrule"></a>

<a id="ref-for-at-ruledef-supports-condition③"></a>

The <code><a href="#csssupportsconditionrule">CSSSupportsConditionRule</a></code> interface represents an [@supports-condition](#at-ruledef-supports-condition) rule.

<a id="ref-for-Exposed①"></a>

<a id="csssupportsconditionrule"></a>

<a id="ref-for-cssgroupingrule"></a>

<a id="ref-for-cssomstring②"></a>

<a id="dom-csssupportsconditionrule-name"></a>

```text
[Exposed=Window]
interface CSSSupportsConditionRule : CSSGroupingRule {
  readonly attribute CSSOMString name;
};
```
`name` of type `CSSOMString`  
<a id="ref-for-named-supports-condition①"></a>

This attribute is the name of the [named supports condition](#named-supports-condition).

## <a id="security"></a>Security Considerations

No security issues have been raised against this document

## <a id="privacy"></a>Privacy Considerations

The font-tech() and font-format() functions may provide information about the user’s software such as its version and whether it is running with non-default settings that enable or disable certain features.

This information can also be determined through other APIs. However, the features in this specification are one of the ways this information is exposed on the Web.

This information can also, in aggregate, be used to improve the accuracy of [fingerprinting](https://www.w3.org/2001/tag/doc/unsanctioned-tracking/) of the user.

## <a id="changes"></a> Changes

### <a id="changes-20241105"></a> Changes since the [Working Draft of 5 November 2024](https://www.w3.org/TR/2024/WD-css-conditional-5-20241105/) 

- Clarified that the last @supports-condition in document order wins ([\#12973](https://github.com/w3c/csswg-drafts/issues/12973))

- <a id="ref-for-at-rule⑤"></a>

  <a id="ref-for-supports-queries③"></a>

  Extended [supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express [at-rule](https://www.w3.org/TR/css-syntax-3/#at-rule) capabilities via at-rule(). ([\#2463](https://github.com/w3c/csswg-drafts/issues/2463), [\#6966](https://github.com/w3c/csswg-drafts/issues/6966), [\#11116](https://github.com/w3c/csswg-drafts/issues/11116), [\#11117](https://github.com/w3c/csswg-drafts/issues/11117))

- <a id="ref-for-csssupportsconditionrule①"></a>

  <a id="ref-for-at-ruledef-supports-condition④"></a>

  Added [@supports-condition](#at-ruledef-supports-condition) at-rule and related <code><a href="#csssupportsconditionrule">CSSSupportsConditionRule</a></code> interface. ([\#12622](https://github.com/w3c/csswg-drafts/issues/12622))

- Clarified that container-names are not tree-scoped ([\#12090](https://github.com/w3c/csswg-drafts/issues/12090))

- Defined direction feature for scroll-state() queries ([\#6400](https://github.com/w3c/csswg-drafts/issues/6400#issuecomment-3200664309))

- Clarified that 0 and 0px are equivalent in conditions ([\#12236](https://github.com/w3c/csswg-drafts/issues/12236#issuecomment-3204826253))

- Defined a range syntax for style container queries ([\#8376](https://github.com/w3c/csswg-drafts/issues/8376#issuecomment-2773374483))

- Explicitly allow tree-counting functions ([\#10982](https://github.com/w3c/csswg-drafts/issues/10982))

- Dimensional query containers no longer apply layout containment ([\#10544](https://github.com/w3c/csswg-drafts/pull/10544))

- Added "both" value for snapped query ([\#11181](https://github.com/w3c/csswg-drafts/issues/11181))

- Added axis keywords for overflowing ([\#11183](https://github.com/w3c/csswg-drafts/issues/11183))

- Renamed overflowing to scrollable ([\#11182](https://github.com/w3c/csswg-drafts/issues/11182))

- <a id="ref-for-typedef-container-query⑧"></a>

  Made [\<container-query\>](#typedef-container-query) optional ([\#9192](https://github.com/w3c/csswg-drafts/issues/9192#issuecomment-1789850349))

### <a id="changes-20240723"></a> Changes since the [Working Draft of 23 July 2024](https://www.w3.org/TR/2024/WD-css-conditional-5-20240723/) 

- <a id="ref-for-valdef-container-stuck-none"></a>

  Added [none](#valdef-container-stuck-none)-keywords to scroll-state() features ([\#10874](https://github.com/w3c/csswg-drafts/pull/10874))

- Added container-type:scroll-state, and scroll-state() queries for stuck, snapped, and scrollable features ([\#6402](https://github.com/w3c/csswg-drafts/issues/6402#issuecomment-1812973013), [\#10784](https://github.com/w3c/csswg-drafts/issues/10784#issuecomment-2379901508), [\#10796](https://github.com/w3c/csswg-drafts/issues/10796#issuecomment-2379885032))

- Corrected example (there is no container-type:style)

- Specified that container queries use the flat tree ([\#5984](https://github.com/w3c/csswg-drafts/issues/5984#issuecomment-2112977366))

### <a id="changes-20211221"></a> Changes since the [First Public Working Draft of 21 December 2021](https://www.w3.org/TR/2021/WD-css-conditional-5-20211221/) 

- Moved container queries to this specification, from CSS Contain 3 ([\#10433](https://github.com/w3c/csswg-drafts/issues/10433))
- Imported the definitions of \<font-format\> and \<font-tech\> from CSS Fonts 4, rather than duplicating them in this specification ([\#8110](https://github.com/w3c/csswg-drafts/issues/8110))
- Updated to use the new parsing algorithm names and block production names
- Corrected a typo in the grammar of \<font-format\>
- Corrected extra spaces in the font-tech and font-format productions ([\#7369](https://github.com/w3c/csswg-drafts/issues/7369))

### <a id="changes-from-L4"></a> Additions since Level 4

- <a id="ref-for-at-ruledef-else⑨"></a>

  <a id="ref-for-at-ruledef-when⑦"></a>

  Added [@when](#at-ruledef-when) and [@else](#at-ruledef-else).

- <a id="ref-for-supports-queries④"></a>

  Extended [supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express font capabilities via font-tech() and font-format().

- <a id="ref-for-supports-queries⑤"></a>

  Extended [supports queries](https://www.w3.org/TR/css-conditional-3/#supports-queries) to express at-rule capabilities via at-rule().

- Moved Container Queries from [\[CSS-CONTAIN-3\]](#biblio-css-contain-3) to this specification. (See also the [CSS Containment 3 § A Changes](https://www.w3.org/TR/css-contain-3/#changes) for more information on the evolution of this feature.)

- <a id="ref-for-csssupportsconditionrule②"></a>

  <a id="ref-for-at-ruledef-supports-condition⑤"></a>

  Added [@supports-condition](#at-ruledef-supports-condition) at-rule and related <code><a href="#csssupportsconditionrule">CSSSupportsConditionRule</a></code> interface.

## <a id="acknowledgments"></a>Acknowledgments

<a id="ref-for-at-ruledef-when⑧"></a>

<a id="ref-for-at-ruledef-else①⓪"></a>

The [@when](#at-ruledef-when) and [@else](#at-ruledef-else) rules are based on a proposal by Tab Atkins.

Comments and previous work from Adam Argyle, Amelia Bellamy-Royds, Anders Hartvoll Ruud, Brian Kardell, Chris Coyier, Christopher Kirk-Nielsen, David Herron, Eric Portis, Ethan Marcotte, Florian Rivoal, Geoff Graham, Gregory Wild-Smith, Ian Kilpatrick, Jen Simmons, Kenneth Rohde Christiansen, Lea Verou, Martin Auswöger, Martine Dowden, Mike Riethmuller, Morten Stenshorne, Nicole Sullivan, Rune Lillesveen, Scott Jehl Scott Kellum, Stacy Kvernmo, Theresa O’Connor, Una Kravets, and many others have contributed to this specification.

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

- [aspect-ratio](#descdef-container-aspect-ratio), in § 6.1.5
- block
  - [value for @container/scrollable](#valdef-container-scrollable-block), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-block), in § 6.3.5
  - [value for @container/snapped](#valdef-container-snapped-block), in § 6.3.3
- block-end
  - [value for @container/scrollable](#valdef-container-scrollable-block-end), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-block-end), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-block-end), in § 6.3.2
- [block-size](#descdef-container-block-size), in § 6.1.4
- block-start
  - [value for @container/scrollable](#valdef-container-scrollable-block-start), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-block-start), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-block-start), in § 6.3.2
- [\<boolean-condition\>](#typedef-boolean-condition), in § 3
- [both](#valdef-container-snapped-both), in § 6.3.3
- bottom
  - [value for @container/scrollable](#valdef-container-scrollable-bottom), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-bottom), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-bottom), in § 6.3.2
- [conditional rule chain](#conditional-rule-chain), in § 4
- [@container](#at-ruledef-container), in § 5.4
- [container](#propdef-container), in § 5.3
- [\<container-condition\>](#typedef-container-condition), in § 5.4
- [container feature](#container-feature), in § 6
- [\<container-name\>](#typedef-container-name), in § 5.4
- [container-name](#propdef-container-name), in § 5.2
- [containerName](#dom-csscontainerrule-containername), in § 9.1
- [\<container-query\>](#typedef-container-query), in § 5.4
- [container query](#container-query), in § 5.4
- [containerQuery](#dom-csscontainerrule-containerquery), in § 9.1
- [container query length](#container-query-length), in § 7
- [container query length unit](#container-query-length), in § 7
- [container scroll-state query](#container-scroll-state-query), in § 6.3
- [container size query](#container-size-query), in § 6.1
- [container style query](#container-style-query), in § 6.2
- [container-type](#propdef-container-type), in § 5.1
- [CSSContainerRule](#csscontainerrule), in § 9.1
- [CSSSupportsConditionRule](#csssupportsconditionrule), in § 9.2
- [\<custom-ident\>](#valdef-container-name-custom-ident), in § 5.2
- [@else](#at-ruledef-else), in § 4
- [evaluate a \<style-range\>](#evaluate-a-style-range), in § 6.2
- [height](#descdef-container-height), in § 6.1.2
- inline
  - [value for @container/scrollable](#valdef-container-scrollable-inline), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-inline), in § 6.3.5
  - [value for @container/snapped](#valdef-container-snapped-inline), in § 6.3.3
- inline-end
  - [value for @container/scrollable](#valdef-container-scrollable-inline-end), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-inline-end), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-inline-end), in § 6.3.2
- inline-size
  - [descriptor for @container](#descdef-container-inline-size), in § 6.1.3
  - [value for container-type](#valdef-container-type-inline-size), in § 5.1
- inline-start
  - [value for @container/scrollable](#valdef-container-scrollable-inline-start), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-inline-start), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-inline-start), in § 6.3.2
- [landscape](#valdef-container-orientation-landscape), in § 6.1.6
- left
  - [value for @container/scrollable](#valdef-container-scrollable-left), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-left), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-left), in § 6.3.2
- [media()](#funcdef-media), in § 3
- [name](#dom-csssupportsconditionrule-name), in § 9.2
- [named supports condition](#named-supports-condition), in § 8
- none
  - [value for @container/scrollable](#valdef-container-scrollable-none), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-none), in § 6.3.5
  - [value for @container/snapped](#valdef-container-snapped-none), in § 6.3.3
  - [value for @container/stuck](#valdef-container-stuck-none), in § 6.3.2
  - [value for container-name](#valdef-container-name-none), in § 5.2
- [normal](#valdef-container-type-normal), in § 5.1
- [orientation](#descdef-container-orientation), in § 6.1.6
- [portrait](#valdef-container-orientation-portrait), in § 6.1.6
- [query container](#query-container), in § 5
- [query container name](#query-container-name), in § 5.2
- [\<query-in-parens\>](#typedef-query-in-parens), in § 5.4
- right
  - [value for @container/scrollable](#valdef-container-scrollable-right), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-right), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-right), in § 6.3.2
- [scrollable](#descdef-container-scrollable), in § 6.3.4
- [scrolled](#descdef-container-scrolled), in § 6.3.5
- [scroll-state](#valdef-container-type-scroll-state), in § 5.1
- [\<scroll-state-feature\>](#typedef-scroll-state-feature), in § 6.3
- [scroll-state feature](#scroll-state-feature), in § 6.3
- [\<scroll-state-in-parens\>](#typedef-scroll-state-in-parens), in § 5.4
- [\<scroll-state-query\>](#typedef-scroll-state-query), in § 5.4
- [size](#valdef-container-type-size), in § 5.1
- [\<size-feature\>](#typedef-size-feature), in § 6.1
- [size features](#size-features), in § 6.1
- [snapped](#descdef-container-snapped), in § 6.3.3
- [stuck](#descdef-container-stuck), in § 6.3.2
- [\<style-feature\>](#typedef-style-feature), in § 5.4
- [\<style-feature-boolean\>](#typedef-style-feature-boolean), in § 5.4
- [\<style-feature-name\>](#typedef-style-feature-name), in § 6.2
- [\<style-feature-plain\>](#typedef-style-feature-plain), in § 5.4
- [style features](#style-features), in § 6.2
- [\<style-feature-value\>](#typedef-style-feature-value), in § 6.2
- [\<style-in-parens\>](#typedef-style-in-parens), in § 5.4
- [\<style-query\>](#typedef-style-query), in § 5.4
- [\<style-range\>](#typedef-style-range), in § 5.4
- [style range](#style-range), in § 6.2
- [\<style-range-value\>](#typedef-style-range-value), in § 5.4
- [support a font format](#dfn-support-font-format), in § 2.1.1
- [support a font tech](#dfn-support-font-tech), in § 2.1.1
- [support a named condition](#dfn-supports-condition-name), in § 2.1.3
- [support an at-rule](#dfn-support-at-rule), in § 2.1.2
- [supports()](#funcdef-supports), in § 3
- [\<supports-at-rule-fn\>](#typedef-supports-at-rule-fn), in § 2
- [@supports-condition](#at-ruledef-supports-condition), in § 8
- [\<supports-condition-name\>](#typedef-supports-condition-name), in § 8
- [\<supports-decl\>](#typedef-supports-decl), in § 2
- [\<supports-feature\>](#typedef-supports-feature), in § 2
- [\<supports-font-format-fn\>](#typedef-supports-font-format-fn), in § 2
- [\<supports-font-tech-fn\>](#typedef-supports-font-tech-fn), in § 2
- top
  - [value for @container/scrollable](#valdef-container-scrollable-top), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-top), in § 6.3.5
  - [value for @container/stuck](#valdef-container-stuck-top), in § 6.3.2
- [@when](#at-ruledef-when), in § 3
- [width](#descdef-container-width), in § 6.1.1
- x
  - [value for @container/scrollable](#valdef-container-scrollable-x), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-x), in § 6.3.5
  - [value for @container/snapped](#valdef-container-snapped-x), in § 6.3.3
- y
  - [value for @container/scrollable](#valdef-container-scrollable-y), in § 6.3.4
  - [value for @container/scrolled](#valdef-container-scrolled-y), in § 6.3.5
  - [value for @container/snapped](#valdef-container-snapped-y), in § 6.3.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="7177b17d"></a>@keyframes
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f5cb4"></a>content box
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
  - <a id="08b3934a"></a>@layer
  - <a id="66620709"></a>cascade-dependent keyword
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="8f27be0f"></a>longhand property
  - <a id="81a52293"></a>property
  - <a id="b45bd8fa"></a>revert
  - <a id="529d0525"></a>revert-layer
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>@media
  - <a id="a5d6c9d2"></a>@supports
  - <a id="ac8ed0be"></a>CSSConditionRule
  - <a id="f1a41224"></a>conditional group rule
  - <a id="6e0a7174"></a>CSS feature queries
  - <a id="be011220"></a>supports queries
- \[CSS-CONDITIONAL-4\] defines the following terms:
  - <a id="3d20c9fd"></a>\<supports-selector-fn\>
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="cb02fa9d"></a>layout containment box
  - <a id="3e4b15e8"></a>size containment
  - <a id="3dc278d0"></a>style containment
- \[CSS-CONTAIN-3\] defines the following terms:
  - <a id="18b2519e"></a>inline-size containment
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="57aa8824"></a>independent formatting context
  - <a id="7a605ac8"></a>principal box
- \[CSS-EXTENSIONS-1\] defines the following terms:
  - <a id="e9f46f56"></a>\<extension-name\>
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="a4687703"></a>\<font-format\>
  - <a id="b89378df"></a>\<font-tech\>
  - <a id="297dfe3a"></a>font-size
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="b24ce65e"></a>@font-face
- \[CSS-NAMESPACES-3\] defines the following terms:
  - <a id="bf0e8186"></a>@namespace
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="d9cfd0c5"></a>hidden
  - <a id="a3cabdb1"></a>scroll container
  - <a id="e3488cd0"></a>scrollable overflow
  - <a id="b1f927bc"></a>scrollable overflow rectangle
  - <a id="700ea31d"></a>scrollport
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="804439eb"></a>sticky
  - <a id="0363c82a"></a>sticky position
  - <a id="01926727"></a>sticky view rectangle
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="5d904ee5"></a>::backdrop
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
  - <a id="2d843062"></a>::file-selector-button
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="b6b63ba4"></a>::marker
  - <a id="4b84fe49"></a>::placeholder
  - <a id="cb456c77"></a>fictional tag sequence
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="c90d9be6"></a>::slotted()
  - <a id="22109b0e"></a>flat tree
  - <a id="778f8dbd"></a>tree-scoped name
- \[CSS-SCROLL-SNAP-1\] defines the following terms:
  - <a id="a307a40e"></a>relative scroll
  - <a id="552105cb"></a>scroll snap container
- \[CSS-SCROLL-SNAP-2\] defines the following terms:
  - <a id="50b46a3e"></a>scrollsnapchanging
  - <a id="47951dd8"></a>snap target
- \[CSS-SHADOW-PARTS-1\] defines the following terms:
  - <a id="0626d1bc"></a>::part()
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="fde87e72"></a>height
  - <a id="d8595fdb"></a>size
  - <a id="88d82030"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="1edd7994"></a>\<at-keyword-token\>
  - <a id="b845e85c"></a>\<block-contents\>
  - <a id="04853566"></a>\<declaration-value\>
  - <a id="a8cb81d7"></a>\<rule-list\>
  - <a id="fb5475b9"></a>@charset
  - <a id="b29fecf5"></a>at-rule
  - <a id="e2112c66"></a>declaration
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="2f6588ea"></a>style change event
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="81b3af3e"></a>!
  - <a id="c297b070"></a>\#
  - <a id="ef9f8297"></a>\*
  - <a id="af4a190d"></a>+
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="c2268b97"></a>\<frequency\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="ee68e69a"></a>\<ratio\>
  - <a id="9108b09d"></a>\<resolution\>
  - <a id="1d798932"></a>\<string\>
  - <a id="7aaa7c88"></a>\<time\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="eefce2af"></a>em
  - <a id="ce7ea0de"></a>identifier
  - <a id="5d6143d2"></a>relative length
  - <a id="a959f189"></a>small viewport size
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="4ae03ef7"></a>arbitrary substitution function
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="310b3140"></a>\<custom-property-name\>
  - <a id="5550667d"></a>custom property
  - <a id="3beec8c9"></a>var()
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="ecef1eb5"></a>block size
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="a6eb24bb"></a>inline axis
  - <a id="18bb1084"></a>inline size
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
- \[CSS2\] defines the following terms:
  - <a id="fb01f680"></a>line-height
- \[CSSOM-1\] defines the following terms:
  - <a id="697c30aa"></a>CSSGroupingRule
  - <a id="9d357000"></a>CSSOMString
  - <a id="79414121"></a>supported CSS property
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="3a71fc5d"></a>MediaQueryList
  - <a id="fa5fd853"></a>matchMedia(query)
  - <a id="279b2092"></a>run snapshot post-layout state steps
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="2ae11dd2"></a>\<mf-boolean\>
  - <a id="64e5a7f2"></a>\<mf-comparison\>
  - <a id="ad2ab4e6"></a>\<mf-eq\>
  - <a id="44194fb2"></a>\<mf-gt\>
  - <a id="9db9ed47"></a>\<mf-lt\>
  - <a id="e518251c"></a>\<mf-plain\>
  - <a id="f92b7f82"></a>\<mf-range\>
  - <a id="fe1a9628"></a>boolean context
  - <a id="a8e43bb7"></a>false
  - <a id="0a000463"></a>media feature
  - <a id="3ea2fcbb"></a>media query
  - <a id="8a490d77"></a>not
  - <a id="0e4f830d"></a>true
- \[SELECTORS-4\] defines the following terms:
  - <a id="7b5d8638"></a>originating element
  - <a id="4d06fa38"></a>pseudo-element
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="61189f1f"></a>effect value
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-conditional-4"></a>\[CSS-CONDITIONAL-4\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 4](https://www.w3.org/TR/css-conditional-4/). 4 September 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-4&#x2F;](https://www.w3.org/TR/css-conditional-4/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-contain-3"></a>\[CSS-CONTAIN-3\]  
Tab Atkins Jr.; Florian Rivoal; Miriam Suzanne. [CSS Containment Module Level 3](https://www.w3.org/TR/css-contain-3/). 18 August 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-3&#x2F;](https://www.w3.org/TR/css-contain-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-extensions-1"></a>\[CSS-EXTENSIONS-1\]  
[CSS Extensions Module Level 1](https://drafts.csswg.org/css-extensions-1/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-extensions-1&#x2F;](https://drafts.csswg.org/css-extensions-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 6 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-namespaces-3"></a>\[CSS-NAMESPACES-3\]  
Elika Etemad. [CSS Namespaces Module Level 3](https://www.w3.org/TR/css-namespaces-3/). 20 March 2014. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-namespaces-3&#x2F;](https://www.w3.org/TR/css-namespaces-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-scroll-snap-1"></a>\[CSS-SCROLL-SNAP-1\]  
Matt Rakow; et al. [CSS Scroll Snap Module Level 1](https://www.w3.org/TR/css-scroll-snap-1/). 11 March 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-1&#x2F;](https://www.w3.org/TR/css-scroll-snap-1/)

<a id="biblio-css-scroll-snap-2"></a>\[CSS-SCROLL-SNAP-2\]  
Elika Etemad; Tab Atkins Jr.; Adam Argyle. [CSS Scroll Snap Module Level 2](https://www.w3.org/TR/css-scroll-snap-2/). 23 July 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scroll-snap-2&#x2F;](https://www.w3.org/TR/css-scroll-snap-2/)

<a id="biblio-css-shadow-parts-1"></a>\[CSS-SHADOW-PARTS-1\]  
Tab Atkins Jr.; Fergal Daly. [CSS Shadow Parts](https://www.w3.org/TR/css-shadow-parts-1/). 15 November 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shadow-parts-1&#x2F;](https://www.w3.org/TR/css-shadow-parts-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-mediaqueries-4"></a>\[MEDIAQUERIES-4\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-position-4"></a>\[CSS-POSITION-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://www.w3.org/TR/css-position-4/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-4&#x2F;](https://www.w3.org/TR/css-position-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

## <a id="property-index"></a>Property Index

<strong>Table 15 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Applies to

<strong>Column 5 (header cell; scope col):</strong>

Inh.

<strong>Column 6 (header cell; scope col):</strong>

%ages

<strong>Column 7 (header cell; scope col):</strong>

Anim­ation type

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-container③"></a>

[container](#propdef-container)

<strong>Column 2 (data cell):</strong>

\<'container-name'\> \[ / \<'container-type'\> \]?

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-container-name⑤"></a>

[container-name](#propdef-container-name)

<strong>Column 2 (data cell):</strong>

none \| \<custom-ident\>+

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none, or an ordered list of identifiers

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-container-type⑦"></a>

[container-type](#propdef-container-type)

<strong>Column 2 (data cell):</strong>

normal \| \[ \[ size \| inline-size \] \|\| scroll-state \]

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

all elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

not animatable

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

<a id="ref-for-at-ruledef-container①⑧"></a>

### <a id="container-descriptor-table"></a>[@container](#at-ruledef-container) Descriptors

<strong>Table 16 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Type

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-aspect-ratio②"></a>

[aspect-ratio](#descdef-container-aspect-ratio)

<strong>Column 2 (data cell):</strong>

\<ratio\>

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

range

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-block-size②"></a>

[block-size](#descdef-container-block-size)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

range

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-height④"></a>

[height](#descdef-container-height)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

range

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-inline-size⑥"></a>

[inline-size](#descdef-container-inline-size)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

range

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-orientation③"></a>

[orientation](#descdef-container-orientation)

<strong>Column 2 (data cell):</strong>

portrait \| landscape

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

discrete

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-scrollable③"></a>

[scrollable](#descdef-container-scrollable)

<strong>Column 2 (data cell):</strong>

none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end \| x \| y \| block \| inline

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

discrete

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-scrolled②"></a>

[scrolled](#descdef-container-scrolled)

<strong>Column 2 (data cell):</strong>

none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end \| x \| y \| block \| inline

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

discrete

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-snapped⑦"></a>

[snapped](#descdef-container-snapped)

<strong>Column 2 (data cell):</strong>

none \| x \| y \| block \| inline \| both

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

discrete

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-stuck②"></a>

[stuck](#descdef-container-stuck)

<strong>Column 2 (data cell):</strong>

none \| top \| right \| bottom \| left \| block-start \| inline-start \| block-end \| inline-end

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

discrete

<strong>Row 11</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-descdef-container-width④"></a>

[width](#descdef-container-width)

<strong>Column 2 (data cell):</strong>

\<length\>

<strong>Column 3 (data cell):</strong>

<strong>Column 4 (data cell):</strong>

range

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface CSSContainerRule : CSSConditionRule {
  readonly attribute CSSOMString containerName;
  readonly attribute CSSOMString containerQuery;
};

[Exposed=Window]
interface CSSSupportsConditionRule : CSSGroupingRule {
  readonly attribute CSSOMString name;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is currently an early draft of the things that are <em>new</em> in level 5. The features in Level 3 and Level 4 are still defined in [\[css-conditional-3\]](#biblio-css-conditional-3) and [\[css-conditional-4\]](#biblio-css-conditional-4) and have not yet been copied here. [↵](#issue-092ad31b)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define "boolean algebra, with X as leaves" in a generic way in Conditional, so all the conditional rules can reference it directly, rather than having to redefine boolean algebra on their own. [↵](#issue-e3cd55e5)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should we require that only the last [@else](#at-ruledef-else) in a chain can have an omitted condition? It’s not uncommon for me, when debugging code, to short-circuit an if-else chain by setting one of them to "true"; I presume that would be similarly useful in CSS? It’s still pretty easy to see you’ve done something wrong if you omit the condition accidentally. [↵](#issue-a37c2e08)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The name of the at-rule is under discussion. Alternatives include @supports-query, @supports-test, and @custom-supports. The name should be consistent with the one chosen for custom media queries. [↵](#issue-4ac0bea1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Container Queries should have a `matchContainer` method. This will be modeled on <code><a href="https://www.w3.org/TR/cssom-view-1/#dom-window-matchmedia">matchMedia()</a></code> and the <code><a href="https://www.w3.org/TR/cssom-view-1/#mediaquerylist">MediaQueryList</a></code> interface, but applied to Elements rather than the Window. When measuring layout sizes, it behaves Similar to `resizeObserver`, but it provides the additional Container Query syntax and features. [\[Issue \#6205\]](https://github.com/w3c/csswg-drafts/issues/6205) [↵](#issue-2e3d6538)
