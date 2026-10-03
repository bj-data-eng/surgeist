Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Nesting Module Level 1](https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Nesting Module Level 1

Source snapshot: https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/

Snapshot SHA-256: b29b0db74b96af295efcd7cd94f34366dd099ebb905dc5529c61409184f6fa9c

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Nesting Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module introduces the ability to nest one style rule inside another, with the selector of the child rule relative to the selector of the parent rule. This increases the modularity and maintainability of CSS stylesheets.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-nesting” in the title, like this: “\[css-nesting\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-nesting%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

This module describes support for nesting a style rule within another style rule, allowing the inner rule’s selector to reference the elements matched by the outer rule. This feature allows related styles to be aggregated into a single structure within the CSS document, improving readability and maintainability.

Tests

General tests for nesting

- [block-skipping.html](https://wpt.fyi/results/css/css-nesting/block-skipping.html) [(live test)](http://wpt.live/css/css-nesting/block-skipping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/block-skipping.html)
- [delete-other-rule-crash.html](https://wpt.fyi/results/css/css-nesting/delete-other-rule-crash.html) [(live test)](http://wpt.live/css/css-nesting/delete-other-rule-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/delete-other-rule-crash.html)
- [double-parent-pseudo-in-placeholder-crash.html](https://wpt.fyi/results/css/css-nesting/double-parent-pseudo-in-placeholder-crash.html) [(live test)](http://wpt.live/css/css-nesting/double-parent-pseudo-in-placeholder-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/double-parent-pseudo-in-placeholder-crash.html)
- [has-nesting.html](https://wpt.fyi/results/css/css-nesting/has-nesting.html) [(live test)](http://wpt.live/css/css-nesting/has-nesting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/has-nesting.html)
- [invalidation-001.html](https://wpt.fyi/results/css/css-nesting/invalidation-001.html) [(live test)](http://wpt.live/css/css-nesting/invalidation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/invalidation-001.html)
- [invalidation-002.html](https://wpt.fyi/results/css/css-nesting/invalidation-002.html) [(live test)](http://wpt.live/css/css-nesting/invalidation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/invalidation-002.html)
- [invalidation-003.html](https://wpt.fyi/results/css/css-nesting/invalidation-003.html) [(live test)](http://wpt.live/css/css-nesting/invalidation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/invalidation-003.html)
- [invalidation-004.html](https://wpt.fyi/results/css/css-nesting/invalidation-004.html) [(live test)](http://wpt.live/css/css-nesting/invalidation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/invalidation-004.html)
- [nested-error-recovery.html](https://wpt.fyi/results/css/css-nesting/nested-error-recovery.html) [(live test)](http://wpt.live/css/css-nesting/nested-error-recovery.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nested-error-recovery.html)
- [nesting-basic.html](https://wpt.fyi/results/css/css-nesting/nesting-basic.html) [(live test)](http://wpt.live/css/css-nesting/nesting-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nesting-basic.html)
- [nesting-revert-rule.tentative.html](https://wpt.fyi/results/css/css-nesting/nesting-revert-rule.tentative.html) [(live test)](http://wpt.live/css/css-nesting/nesting-revert-rule.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nesting-revert-rule.tentative.html)
- [parent-pseudo-in-placeholder-crash.html](https://wpt.fyi/results/css/css-nesting/parent-pseudo-in-placeholder-crash.html) [(live test)](http://wpt.live/css/css-nesting/parent-pseudo-in-placeholder-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/parent-pseudo-in-placeholder-crash.html)
- [pseudo-part-crash.html](https://wpt.fyi/results/css/css-nesting/pseudo-part-crash.html) [(live test)](http://wpt.live/css/css-nesting/pseudo-part-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/pseudo-part-crash.html)
- [pseudo-where-crash.html](https://wpt.fyi/results/css/css-nesting/pseudo-where-crash.html) [(live test)](http://wpt.live/css/css-nesting/pseudo-where-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/pseudo-where-crash.html)
- [supports-is-consistent.html](https://wpt.fyi/results/css/css-nesting/supports-is-consistent.html) [(live test)](http://wpt.live/css/css-nesting/supports-is-consistent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/supports-is-consistent.html)

------------------------------------------------------------------------

### <a id="placement"></a>1.1.  Module Interactions

This module introduces new parser rules that extend the [\[CSS21\]](#biblio-css21) parser model. It introduces selectors that extend the [\[SELECTORS-4\]](#biblio-selectors-4) module. It extends and modifies some IDL and algorithms defined in the [\[CSSOM-1\]](#biblio-cssom-1) module.

### <a id="values"></a>1.2.  Values

This specification does not define any new properties or values.

## <a id="explainer"></a>2.  Explainer

<em>This section is non-normative.</em>

Imagine you have some CSS that you’d like to write in a more compact way.

```text
.foo {
  color: green;
}
.foo .bar {
  font-size: 1.4rem;
}
```
With Nesting, you can write such code as:

```text
.foo {
  color: green;
  .bar {
    font-size: 1.4rem;
  }
}
```
If you’ve been nesting styles in Sass or other CSS preprocessors, you will find this very familiar.

You can nest any rules inside of a parent style rule:

```text
main {
  div { ... }
  .bar { ... }
  #baz { ...}
  :has(p) { ... }
  ::backdrop { ... }
  [lang|="zh"] { ... }
  * { ... }
}
```
<a id="ref-for-descendant-combinator"></a>

By default, the child rule’s selector is assumed to connect to the parent rule by a [descendant combinator](https://www.w3.org/TR/selectors-4/#descendant-combinator), but you can start the nested selector with any combinator to change that:

```text
main {
  + article { ... }
  > p { ... }
  ~ main { ... }
}
```
<a id="ref-for-selectordef-"></a>

The new [&#x26;](#selectordef-) selector lets you refer to the elements matched by the parent selector explictly, so the previous examples could have been written as:

```text
main {
  & + article { ... }
  & > p { ... }
  & ~ main { ... }
}
```
<a id="ref-for-selectordef-①"></a>

But you can place the [&#x26;](#selectordef-) in other locations within the nested selector, to indicate other types of relationships between the parent and child rule. For example, this CSS:

```text
ul {
  padding-left: 1em;
}
.component ul {
  padding-left: 0;
}
```
Can be rewritten using Nesting as:

```text
ul {
  padding-left: 1em;
  .component & {
    padding-left: 0;
  }
}
```
<a id="ref-for-selectordef-②"></a>

Again, the [&#x26;](#selectordef-) gives you a way to say “this is where I want the nested selector to go”.

It’s also handy when you don’t want a space between your selectors. For example:

```text
a {
  color: blue;
  &:hover {
    color: lightblue;
  }
}
```
<a id="ref-for-selectordef-③"></a>

Such code yields the same result as `a:hover {`. Without the [&#x26;](#selectordef-), you’d get `a :hover {`—​notice the space between `a` and `:hover`—​which would fail to style your hover link.

You can nest more than one layer deep—​nesting CSS inside already-nested CSS—​in as many levels as you desire. You can mix Nesting with Container Queries, Supports Queries, Media Queries, and/or Cascade Layers however you want. (Nearly) anything can go inside of anything.

## <a id="nesting"></a>3. Nesting Style Rules

Style rules can be nested inside of other styles rules. These <a id="nested-style-rule"></a>nested style rules act exactly like ordinary style rules—​associating properties with elements via selectors—​but they "inherit" their parent rule’s selector context, allowing them to further build on the parent’s selector without having to repeat it, possibly multiple times.

<a id="ref-for-nested-style-rule"></a>

<a id="ref-for-relative-selector"></a>

A [nested style rule](#nested-style-rule) is exactly like a normal style rule, except that it can use [relative selectors](https://www.w3.org/TR/selectors-4/#relative-selector), which are implicitly relative to the elements matched by the parent rule.

Tests

- [implicit-nesting.html](https://wpt.fyi/results/css/css-nesting/implicit-nesting.html) [(live test)](http://wpt.live/css/css-nesting/implicit-nesting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/implicit-nesting.html)
- [implicit-nesting-ident.html](https://wpt.fyi/results/css/css-nesting/implicit-nesting-ident.html) [(live test)](http://wpt.live/css/css-nesting/implicit-nesting-ident.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/implicit-nesting-ident.html)
- [implicit-nesting-ident-recovery.html](https://wpt.fyi/results/css/css-nesting/implicit-nesting-ident-recovery.html) [(live test)](http://wpt.live/css/css-nesting/implicit-nesting-ident-recovery.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/implicit-nesting-ident-recovery.html)
- [implicit-parent-insertion-crash.html](https://wpt.fyi/results/css/css-nesting/implicit-parent-insertion-crash.html) [(live test)](http://wpt.live/css/css-nesting/implicit-parent-insertion-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/implicit-parent-insertion-crash.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-82c7d228"></a> That is, a nested style rule like:
>
> ```css
> .foo {
>   color: red;
> 
>   a {
>     color: blue;
>   }
> }
> ```
>
> is valid, and equivalent to:
>
> ```css
> .foo {
>   color: red;
> }
> .foo a {
>   color: blue;
> }
> ```
>
> <a id="ref-for-nesting-selector"></a>
>
> <a id="ref-for-relative-selector①"></a>
>
> The nested rule can also use the [nesting selector](#nesting-selector) to directly refer to the parent rule’s matched elements, or use [relative selector](https://www.w3.org/TR/selectors-4/#relative-selector) syntax to specify relationships other than "descendant".
>
> ```css
> .foo {
>   color: red;
> 
>   &:hover {
>     color: blue;
>   }
> }
> 
> /* equivalent to: */
> 
> .foo { color: red; }
> .foo:hover { color: blue; }
> ```
>
> ```css
> .foo {
>   color: red;
> 
>   + .bar {
>     color: blue;
>   }
> }
> 
> /* equivalent to: */
> 
> .foo { color: red; }
> .foo + .bar { color: blue; }
> ```
Tests

- [nested-rule-cssom-invalidation.html](https://wpt.fyi/results/css/css-nesting/nested-rule-cssom-invalidation.html) [(live test)](http://wpt.live/css/css-nesting/nested-rule-cssom-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nested-rule-cssom-invalidation.html)

### <a id="syntax"></a>3.1. Syntax

<a id="ref-for-style-rule"></a>

<a id="ref-for-nested-style-rule①"></a>

<a id="ref-for-at-rule"></a>

<a id="ref-for-cssstyledeclaration-declarations"></a>

The contents of [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) now accepts [nested style rules](#nested-style-rule) and [at-rules](https://www.w3.org/TR/css-syntax-3/#at-rule), in addition to the existing [declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations).

<a id="ref-for-nested-style-rule②"></a>

[Nested style rules](#nested-style-rule) differ from non-nested rules in the following ways:

- <a id="ref-for-nested-style-rule③"></a>

  <a id="ref-for-typedef-relative-selector-list"></a>

  <a id="ref-for-typedef-selector-list"></a>

  <a id="ref-for-relative-selector②"></a>

  <a id="ref-for-nesting-selector①"></a>

  A [nested style rule](#nested-style-rule) accepts a [\<relative-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-relative-selector-list) as its prelude (rather than just a [\<selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-selector-list)). Any [relative selectors](https://www.w3.org/TR/selectors-4/#relative-selector) are relative to the elements represented by the [nesting selector](#nesting-selector).

- <a id="ref-for-typedef-relative-selector-list①"></a>

  <a id="ref-for-selector-combinator"></a>

  <a id="ref-for-contain-the-nesting-selector"></a>

  <a id="ref-for-relative-selector③"></a>

  If a selector in the [\<relative-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-relative-selector-list) does not start with a [combinator](https://www.w3.org/TR/selectors-4/#selector-combinator) but does [contain the nesting selector](#contain-the-nesting-selector), it is interpreted as a non-[relative selector](https://www.w3.org/TR/selectors-4/#relative-selector).

The precise details of how nested style rules are parsed are defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-nested-style-rule④"></a>

An invalid [nested style rule](#nested-style-rule) is ignored, along with its contents, but does not invalidate its parent rule.

<a id="ref-for-relative-selector④"></a>

<a id="ref-for-nesting-selector②"></a>

Nested rules with [relative selectors](https://www.w3.org/TR/selectors-4/#relative-selector) include the specificity of their implied [nesting selector](#nesting-selector). For example, .foo { \> .bar {...}} and .foo { &#x26; \> .bar {...}} have the same specificity for their inner rule.

<a id="ref-for-simple"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Some CSS-generating tools that preprocess nesting will concatenate selectors as strings, allowing authors to build up a <em>single</em> [simple selector](https://www.w3.org/TR/selectors-4/#simple) across nesting levels. This is sometimes used with hierarchical name patterns like [BEM](https://getbem.com/introduction/) to reduce repetition across a file, when the selectors themselves have significant repetition internally.
>
> For example, if one component uses the class .foo, and a nested component uses .fooBar, you could write this in [Sass](https://sass-lang.com/) as:
>
> ```css
> .foo {
>   color: blue;
>   &Bar { color: red; }
> }
> /* In Sass, this is equivalent to
>   .foo { color: blue; }
>   .fooBar { color: red; }
> */
> ```
>
> This is not allowed in CSS, as nesting is not a syntax transformation, but rather matches on the actual elements the parent selector matches.
>
> <a id="ref-for-valdef-caret-shape-bar"></a>
>
> It is also true that the selector &#x26;Bar is invalid in CSS in the first place, as the [Bar](https://www.w3.org/TR/css-ui-4/#valdef-caret-shape-bar) part is a type selector, which must come first in the compound selector. (That is, it must be written as Bar&#x26;.) So, luckily, there is no overlap between CSS Nesting and the preprocessor syntax.

<a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

<a id="ref-for-typedef-delim-token"></a>

A selector is said to <a id="contain-the-nesting-selector"></a>contain the nesting selector if, when it was [parsed](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) as any type of selector, a [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token) with the value "&#x26;" (U+0026 AMPERSAND) was encountered.

<a id="ref-for-selectordef-④"></a>

<a id="ref-for-contain-the-nesting-selector①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is phrased in this explicit manner so as to catch cases like :is(:unknown(&#x26;), .bar), where an unknown selector (which, being unknown, we have no way of knowing whether the argument is <em>meant</em> to be parsed as a selector or not) is the only part of the selector that contains an [&#x26;](#selectordef-). As that <em>might</em> be a perfectly valid selector that’s only supported by newer browsers, and we don’t want parsing to be dependent on unrelated versioning issues, we treat it as still [containing the nesting selector](#contain-the-nesting-selector).

<a id="ref-for-typedef-forgiving-selector-list"></a>

<a id="ref-for-contain-the-nesting-selector②"></a>

If a [\<forgiving-selector-list\>](https://www.w3.org/TR/selectors-4/#typedef-forgiving-selector-list) has an item that [contains the nesting selector](#contain-the-nesting-selector) but is invalid, that item is preserved exactly as-is rather than being discarded. (This does not change the matching behavior of the selector—​an invalid selector still fails to match anything—​just the serialization of the selector.)

Tests

- [nest-containing-forgiving.html](https://wpt.fyi/results/css/css-nesting/nest-containing-forgiving.html) [(live test)](http://wpt.live/css/css-nesting/nest-containing-forgiving.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nest-containing-forgiving.html)
- [parsing.html](https://wpt.fyi/results/css/css-nesting/parsing.html) [(live test)](http://wpt.live/css/css-nesting/parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/parsing.html)

<a id="ref-for-selectordef-⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-03f974e2"></a> The preceding paragraph needs to move to Selectors when we move [&#x26;](#selectordef-) itself to Selectors; I’m monkey-patching for convenience here.

### <a id="syntax-examples"></a>3.2.  Examples

<a id="ref-for-relative-selector⑤"></a>

```css
/* & can be used on its own */
.foo {
  color: blue;
  & > .bar { color: red; }
  > .baz { color: green; }
}
/* equivalent to
  .foo { color: blue; }
  .foo > .bar { color: red; }
  .foo > .baz { color: green; }
*/


/* or in a compound selector,
   refining the parent’s selector */
.foo {
  color: blue;
  &.bar { color: red; }
}
/* equivalent to
  .foo { color: blue; }
  .foo.bar { color: red; }
*/

/* multiple selectors in the list are all
   relative to the parent */
.foo, .bar {
  color: blue;
  + .baz, &.qux { color: red; }
}
/* equivalent to
  .foo, .bar { color: blue; }
  :is(.foo, .bar) + .baz,
  :is(.foo, .bar).qux { color: red; }
*/

/* & can be used multiple times in a single selector */
.foo {
  color: blue;
  & .bar & .baz & .qux { color: red; }
}
/* equivalent to
  .foo { color: blue; }
  .foo .bar .foo .baz .foo .qux { color: red; }
*/

/* & doesn’t have to be at the beginning of the selector */

.foo {
  color: red;
  .parent & {
    color: blue;
  }
}
/* equivalent to
  .foo { color: red; }
  .parent .foo { color: blue; }
*/

.foo {
  color: red;
  :not(&) {
    color: blue;
  }
}
/* equivalent to
  .foo { color: red; }
  :not(.foo) { color: blue; }
*/

/* But if you use a relative selector,
  an initial & is implied automatically */

.foo {
  color: red;
  + .bar + & { color: blue; }
}

/* equivalent to
  .foo { color: red; }
  .foo + .bar + .foo { color: blue; }
*/

/* Somewhat silly, but & can be used all on its own, as well. */
.foo {
  color: blue;
  & { padding: 2ch; }
}
/* equivalent to
  .foo { color: blue; }
  .foo { padding: 2ch; }

  // or

  .foo {
    color: blue;
    padding: 2ch;
  }
*/

/* Again, silly, but can even be doubled up. */
.foo {
  color: blue;
  && { padding: 2ch; }
}
/* equivalent to
  .foo { color: blue; }
  .foo.foo { padding: 2ch; }
*/

/* The parent selector can be arbitrarily complicated */
.error, #404 {
  &:hover > .baz { color: red; }
}
/* equivalent to
  :is(.error, #404):hover > .baz { color: red; }
*/

.ancestor .el {
  .other-ancestor & { color: red; }
}
/* equivalent to
  .other-ancestor :is(.ancestor .el) { color: red; }

/* As can the nested selector */
.foo {
  & :is(.bar, &.baz) { color: red; }
}
/* equivalent to
  .foo :is(.bar, .foo.baz) { color: red; }
*/

/* Multiple levels of nesting "stack up" the selectors */
figure {
  margin: 0;

  > figcaption {
    background: hsl(0 0% 0% / 50%);

    > p {
      font-size: .9rem;
    }
  }
}
/* equivalent to
  figure { margin: 0; }
  figure > figcaption { background: hsl(0 0% 0% / 50%); }
  figure > figcaption > p { font-size: .9rem; }
*/

/* Example usage with Cascade Layers */
@layer base {
  html {
    block-size: 100%;

    body {
      min-block-size: 100%;
    }
  }
}
/* equivalent to
  @layer base {
    html { block-size: 100%; }
    html body { min-block-size: 100%; }
  }
*/

/* Example nesting Cascade Layers */
@layer base {
  html {
    block-size: 100%;

    @layer support {
      body {
        min-block-size: 100%;
      }
    }
  }
}
/* equivalent to
  @layer base {
    html { block-size: 100%; }
  }
  @layer base.support {
    html body { min-block-size: 100%; }
  }
*/

/* Example usage with Scoping */
@scope (.card) to (> header) {
  :scope {
    inline-size: 40ch;
    aspect-ratio: 3/4;

    > header {
      border-block-end: 1px solid white;
    }
  }
}
/* equivalent to
  @scope (.card) to (> header) {
    :scope { inline-size: 40ch; aspect-ratio: 3/4; }
    :scope > header { border-block-end: 1px solid white; }
  }
*/

/* Example nesting Scoping */
.card {
  inline-size: 40ch;
  aspect-ratio: 3/4;

  @scope (&) to (> header > *) {
    :scope > header {
      border-block-end: 1px solid white;
    }
  }
}

/* equivalent to
  .card { inline-size: 40ch; aspect-ratio: 3/4; }
  @scope (.card) to (> header > *) {
    :scope > header { border-block-end: 1px solid white; }
  }
*/
```
### <a id="conditionals"></a>3.3. Nesting Other At-Rules

<a id="ref-for-nested-style-rule⑤"></a>

<a id="ref-for-style-rule①"></a>

In addition to [nested style rules](#nested-style-rule), this specification allows <a id="nested-group-rules"></a>nested group rules inside of [style rules](https://www.w3.org/TR/css-syntax-3/#style-rule): any at-rule whose body contains <a id="ref-for-style-rule②"></a>style rules can be nested inside of a <a id="ref-for-style-rule③"></a>style rule as well, unless otherwise specified.

<a id="ref-for-nested-group-rules"></a>

<a id="ref-for-typedef-block-contents"></a>

<a id="ref-for-typedef-rule-list"></a>

When nested in this way, the contents of a [nested group rule](#nested-group-rules)’s block are parsed as [\<block-contents\>](https://drafts.csswg.org/css-syntax-3/#typedef-block-contents) rather than [\<rule-list\>](https://www.w3.org/TR/css-syntax-3/#typedef-rule-list):

- <a id="ref-for-style-rule④"></a>

  <a id="ref-for-nested-style-rule⑥"></a>

  <a id="ref-for-nesting-selector③"></a>

  [Style rules](https://www.w3.org/TR/css-syntax-3/#style-rule) are [nested style rules](#nested-style-rule), with their [nesting selector](#nesting-selector) taking its definition from the nearest ancestor <a id="ref-for-style-rule⑤"></a>style rule.

- <a id="ref-for-nested-declarations-rule"></a>

  Properties can be directly used, acting as if they were nested in a [nested declarations rule](#nested-declarations-rule).

<a id="ref-for-nested-group-rules①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Specifically, these rules are capable of being [nested group rules](#nested-group-rules):
>
> - <a id="ref-for-conditional-group-rule"></a>
>
>   <a id="ref-for-at-ruledef-container"></a>
>
>   <a id="ref-for-at-ruledef-media"></a>
>
>   <a id="ref-for-at-ruledef-supports"></a>
>
>   all the [conditional group rules](https://www.w3.org/TR/css-conditional-3/#conditional-group-rule) ([@container](https://www.w3.org/TR/css-conditional-5/#at-ruledef-container), [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media), [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports))
>
> - <a id="ref-for-at-ruledef-layer"></a>
>
>   [@layer](https://www.w3.org/TR/css-cascade-5/#at-ruledef-layer)
>
> - <a id="ref-for-at-ruledef-scope"></a>
>
>   [@scope](https://www.w3.org/TR/css-cascade-6/#at-ruledef-scope)

Tests

- [conditional-properties.html](https://wpt.fyi/results/css/css-nesting/conditional-properties.html) [(live test)](http://wpt.live/css/css-nesting/conditional-properties.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/conditional-properties.html)
- [conditional-rules.html](https://wpt.fyi/results/css/css-nesting/conditional-rules.html) [(live test)](http://wpt.live/css/css-nesting/conditional-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/conditional-rules.html)
- [invalid-inner-rules.html](https://wpt.fyi/results/css/css-nesting/invalid-inner-rules.html) [(live test)](http://wpt.live/css/css-nesting/invalid-inner-rules.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/invalid-inner-rules.html)
- [supports-rule.html](https://wpt.fyi/results/css/css-nesting/supports-rule.html) [(live test)](http://wpt.live/css/css-nesting/supports-rule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/supports-rule.html)

<a id="ref-for-nested-group-rules②"></a>

The meanings and behavior of such [nested group rules](#nested-group-rules) is otherwise unchanged, unless otherwise specified.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-81212ace"></a> For example, the following conditional nestings are valid:
>
> ```css
> /* Properties can be directly used */
> .foo {
>   display: grid;
> 
>   @media (orientation: landscape) {
>     grid-auto-flow: column;
>   }
> }
> 
> /* equivalent to: */
> .foo {
>   display: grid;
> }
> @media (orientation: landscape) {
>   .foo {
>     grid-auto-flow: column
>   }
> }
> 
> /* and also equivalent to the unnested: */
> .foo { display: grid; }
> 
> @media (orientation: landscape) {
>   .foo {
>     grid-auto-flow: column;
>   }
> }
> 
> /* Conditionals can be further nested */
> .foo {
>   display: grid;
> 
>   @media (orientation: landscape) {
>     grid-auto-flow: column;
> 
>     @media (min-width > 1024px) {
>       max-inline-size: 1024px;
>     }
>   }
> }
> 
> /* equivalent to */
> .foo { display: grid; }
> 
> @media (orientation: landscape) {
>   .foo {
>     grid-auto-flow: column;
>   }
> }
> 
> @media (orientation: landscape) and (min-width > 1024px) {
>   .foo {
>     max-inline-size: 1024px;
>   }
> }
> 
> /* Example nesting Cascade Layers */
> html {
>   @layer base {
>     block-size: 100%;
> 
>     @layer support {
>       & body {
>         min-block-size: 100%;
>       }
>     }
>   }
> }
> 
> /* equivalent to */
> @layer base {
>   html { block-size: 100%; }
> }
> @layer base.support {
>   html body { min-block-size: 100%; }
> }
> 
> /* Example nesting Scoping */
> .card {
>   inline-size: 40ch;
>   aspect-ratio: 3/4;
> 
>   @scope (&) {
>     :scope {
>       border: 1px solid white;
>     }
>   }
> }
> 
> /* equivalent to */
> .card { inline-size: 40ch; aspect-ratio: 3/4; }
> @scope (.card) {
>   :scope { border-block-end: 1px solid white; }
> }
> ```
<a id="ref-for-nested-declarations-rule①"></a>

Runs of consecutive directly-nested properties are automatically wrapped in [nested declarations rules](#nested-declarations-rule). (This is observable in the CSSOM.)

Tests

- [nesting-layer.html](https://wpt.fyi/results/css/css-nesting/nesting-layer.html) [(live test)](http://wpt.live/css/css-nesting/nesting-layer.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nesting-layer.html)

<a id="ref-for-at-ruledef-scope①"></a>

#### <a id="nesting-at-scope"></a>3.3.1.  Nested [@scope](https://www.w3.org/TR/css-cascade-6/#at-ruledef-scope) Rules

<a id="ref-for-at-ruledef-scope②"></a>

<a id="ref-for-nested-group-rules③"></a>

<a id="ref-for-selectordef-⑥"></a>

<a id="ref-for-typedef-scope-start"></a>

When the [@scope](https://www.w3.org/TR/css-cascade-6/#at-ruledef-scope) rule is a [nested group rule](#nested-group-rules), an [&#x26;](#selectordef-) in the [\<scope-start\>](https://www.w3.org/TR/css-cascade-6/#typedef-scope-start) selector refers to the elements matched by the nearest ancestor style rule.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d2d35473"></a> That is, the following code:
>
> ```text
> .parent {
>   color: blue;
> 
>   @scope (& > .scope) to (& .limit) {
>     & .content {
>       color: red;
>     }
>   }
> }
> ```
>
> is equivalent to:
>
> ```text
> .parent { color: blue; }
> @scope (.parent > .scope) to (:where(:scope) .limit) {
>   :where(:scope) .content {
>     color: red;
>   }
> }
> ```
<a id="ref-for-selectordef-⑦"></a>

<a id="ref-for-at-ruledef-scope③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [&#x26;](#selectordef-) selector behaves like `:where(:scope)` in [@scope](https://www.w3.org/TR/css-cascade-6/#at-ruledef-scope) rules.

### <a id="mixing"></a>3.4. Mixing Nesting Rules and Declarations

<a id="ref-for-nested-style-rule⑦"></a>

<a id="ref-for-nested-group-rules④"></a>

<a id="ref-for-nested-declarations-rule②"></a>

When a style rule contains both declarations and [nested style rules](#nested-style-rule) or [nested group rules](#nested-group-rules), all three can be arbitrarily mixed. Declarations coming after or between rules are implicitly wrapped in [nested declarations rules](#nested-declarations-rule), to preserve their order relative to the other rules.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-de3f70c3"></a> For example, in the following code:
>
> ```css
> article {
>   color: green;
>   & { color: blue; }
>   color: red;
> }
> 
> /* equivalent to */
> article { color: green; }
> :is(article) { color: blue; }
> article { color: red; }
> 
> /* NOT equivalent to */
> article { color: green; }
> article { color: red; }
> :is(article) { color: blue; }
> ```
<a id="ref-for-nested-style-rule⑧"></a>

<a id="ref-for-nested-group-rules⑤"></a>

For the purpose of determining the [Order Of Appearance](https://www.w3.org/TR/css-cascade-4/#cascade-sort), [nested style rules](#nested-style-rule) and [nested group rules](#nested-group-rules) are considered to come <em>after</em> their parent rule.

For example:

```css
article {
  color: blue;
  & { color: red; }
}
```
<a id="ref-for-propdef-color"></a>

Both declarations have the same specificity (0,0,1), but the nested rule is considered to come <em>after</em> its parent rule, so the [color: red](https://www.w3.org/TR/css-color-4/#propdef-color) declarations wins the cascade.

On the other hand, in this example:

```css
article {
  color: blue;
  :where(&) { color: red; }
}
```
<a id="ref-for-where-pseudo"></a>

<a id="ref-for-nesting-selector④"></a>

<a id="ref-for-propdef-color①"></a>

The [:where()](https://www.w3.org/TR/selectors-4/#where-pseudo) pseudoclass reduces the specificity of the [nesting selector](#nesting-selector) to 0, so the [color: red](https://www.w3.org/TR/css-color-4/#propdef-color) declaration now has a specificity of (0,0,0), and loses to the <a id="ref-for-propdef-color②"></a>color: blue declaration before "Order Of Appearance" comes into consideration.

<a id="ref-for-nested-declarations-rule③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While one <em>can</em> freely intermix declarations and nested rules, it’s harder to read and somewhat confusing to do so, since the later properties are automatically wrapped in a [nested declarations rule](#nested-declarations-rule) that doesn’t appear in the source text. For readability’s sake, it’s recommended that authors put all their properties first in a style rule, before any nested rules. (This also happens to act slightly better in older user agents: due to specifics of how parsing and error-recovery work, properties appearing after nested rules can get skipped.)

<a id="ref-for-selectordef-⑧"></a>

## <a id="nest-selector"></a>4. Nesting Selector: the [&#x26;](#selectordef-) selector

<a id="ref-for-nested-style-rule⑨"></a>

When using a [nested style rule](#nested-style-rule), one must be able to refer to the elements matched by the parent rule; that is, after all, <em>the entire point of nesting</em>. To accomplish that, this specification defines a new selector, the <a id="nesting-selector"></a>nesting selector, written as <a id="selectordef-"></a>&#x26; (U+0026 AMPERSAND).

<a id="ref-for-nested-style-rule①⓪"></a>

<a id="ref-for-nesting-selector⑤"></a>

<a id="ref-for-scope-pseudo"></a>

When used in the selector of a [nested style rule](#nested-style-rule), the [nesting selector](#nesting-selector) represents the elements matched by the parent rule. When used in any other context, it represents the same elements as [:scope](https://www.w3.org/TR/selectors-4/#scope-pseudo) in that context (unless otherwise defined).

Tests

- [top-level-is-scope.html](https://wpt.fyi/results/css/css-nesting/top-level-is-scope.html) [(live test)](http://wpt.live/css/css-nesting/top-level-is-scope.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/top-level-is-scope.html)

<a id="ref-for-nesting-selector⑥"></a>

<a id="ref-for-matches-pseudo"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The [nesting selector](#nesting-selector) can be desugared by replacing it with the parent style rule’s selector, wrapped in an [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) selector. For example,
>
> ```css
> a, b {
>   & c { color: blue; }
> }
> ```
>
> is equivalent to
>
> ```css
> :is(a, b) c { color: blue; }
> ```
<a id="ref-for-nesting-selector⑦"></a>

<a id="ref-for-matches-pseudo①"></a>

The [nesting selector](#nesting-selector) cannot represent pseudo-elements (identical to the behavior of the [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) pseudo-class).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7145ff1e"></a> For example, in the following style rule:
>
> ```css
> .foo, .foo::before, .foo::after {
>   color: red;
> 
>   &:hover { color: blue; }
> }
> ```
>
> <a id="ref-for-selectordef-⑨"></a>
>
> the [&#x26;](#selectordef-) only represents the elements matched by .foo; in other words, it’s equivalent to:
>
> ```css
> .foo, .foo::before, .foo::after {
>   color: red;
> }
> .foo:hover {
>   color: blue;
> }
> ```
<a id="ref-for-matches-pseudo②"></a>

<a id="ref-for-selectordef-①⓪"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-62c479a8"></a> We’d like to relax this restriction, but need to do so simultaneously for both [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) and [&#x26;](#selectordef-), since they’re intentionally built on the same underlying mechanisms. ([Issue 7433](https://github.com/w3c/csswg-drafts/issues/7433))

<a id="ref-for-specificity"></a>

<a id="ref-for-nesting-selector⑧"></a>

<a id="ref-for-matches-pseudo③"></a>

The [specificity](https://www.w3.org/TR/selectors-4/#specificity) of the [nesting selector](#nesting-selector) is equal to the largest specificity among the complex selectors in the parent style rule’s selector list (identical to the behavior of [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo)), or zero if no such selector list exists.

Tests

- [top-level-parent-pseudo-specificity.html](https://wpt.fyi/results/css/css-nesting/top-level-parent-pseudo-specificity.html) [(live test)](http://wpt.live/css/css-nesting/top-level-parent-pseudo-specificity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/top-level-parent-pseudo-specificity.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a71fa710"></a> For example, given the following style rules:
>
> ```css
> #a, b {
>   & c { color: blue; }
> }
> .foo c { color: red; }
> ```
>
> Then in a DOM structure like
>
> ```html
> <b class=foo>
>   <c>Blue text</c>
> </b>
> ```
>
> <a id="ref-for-selectordef-①①"></a>
>
> The text will be blue, rather than red. The specificity of the [&#x26;](#selectordef-) is the larger of the specificities of \#a (\[1,0,0\]) and b (\[0,0,1\]), so it’s \[1,0,0\], and the entire &#x26; c selector thus has specificity \[1,0,1\], which is larger than the specificity of .foo c (\[0,1,1\]).
>
> <a id="ref-for-propdef-color③"></a>
>
> Notably, this is <em>different</em> than the result you’d get if the nesting were manually expanded out into non-nested rules, since the [color: blue](https://www.w3.org/TR/css-color-4/#propdef-color) declaration would then be matching due to the b c selector (\[0,0,2\]) rather than \#a c (\[1,0,1\]).

> <strong data-conversion-semantic="note">Note</strong>
>
> Why is the specificity different than non-nested rules?
>
> <a id="ref-for-nesting-selector⑨"></a>
>
> <a id="ref-for-matches-pseudo④"></a>
>
> The [nesting selector](#nesting-selector) intentionally uses the same specificity rules as the [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) pseudoclass, which just uses the largest specificity among its arguments, rather than tracking <em>which</em> selector actually matched.
>
> This is required for performance reasons; if a selector has multiple possible specificities, depending on how precisely it was matched, it makes selector matching much more complicated and slower.
>
> <a id="ref-for-selectordef-①②"></a>
>
> <a id="ref-for-matches-pseudo⑤"></a>
>
> That skirts the question, tho: why <em>do</em> we define [&#x26;](#selectordef-) in terms of [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo)? Some non-browser implementations of Nesting-like functionality do <em>not</em> desugar to <a id="ref-for-matches-pseudo⑥"></a>:is(), largely because they predate the introduction of <a id="ref-for-matches-pseudo⑦"></a>:is() as well. Instead, they desugar directly; however, this comes with its own <em>significant</em> problems, as some (reasonably common) cases can accidentally produce <em>massive</em> selectors, due to the exponential explosion of possibilities.
>
> ```css
> .a1, .a2, .a3 {
>   .b1, .b2, .b3 {
>     .c1, .c2, .c3 {
>       ...;
>     }
>   }
> }
> 
> /* naively desugars to */
> .a1 .b1 .c1,
> .a1 .b1 .c2,
> .a1 .b1 .c3,
> .a1 .b2 .c1,
> .a1 .b2 .c2,
> .a1 .b2 .c3,
> .a1 .b3 .c1,
> .a1 .b3 .c2,
> .a1 .b3 .c3,
> .a2 .b1 .c1,
> .a2 .b1 .c2,
> .a2 .b1 .c3,
> .a2 .b2 .c1,
> .a2 .b2 .c2,
> .a2 .b2 .c3,
> .a2 .b3 .c1,
> .a2 .b3 .c2,
> .a2 .b3 .c3,
> .a3 .b1 .c1,
> .a3 .b1 .c2,
> .a3 .b1 .c3,
> .a3 .b2 .c1,
> .a3 .b2 .c2,
> .a3 .b2 .c3,
> .a3 .b3 .c1,
> .a3 .b3 .c2,
> .a3 .b3 .c3 {...}
> ```
>
> Here, three levels of nesting, each with three selectors in their lists, produced 27 desugared selectors. Adding more selectors to the lists, adding more levels of nesting, or making the nested rules more complex can make a relatively small rule expand into multiple megabytes of selectors (or much, much more!).
>
> Some CSS tools avoid the worst of this by heuristically discarding some variations, so they don’t have to output as much but are still <em>probably</em> correct, but that’s not an option available to UAs.
>
> <a id="ref-for-matches-pseudo⑧"></a>
>
> Desugaring with [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) instead eliminates this problem entirely, at the cost of making specificity slightly less useful, which was judged a reasonable trade-off.

<a id="ref-for-nesting-selector①⓪"></a>

<a id="ref-for-featureless"></a>

The [nesting selector](#nesting-selector) is capable of matching [featureless](https://www.w3.org/TR/selectors-4/#featureless) elements, if they were matched by the parent rule.

<a id="ref-for-nesting-selector①①"></a>

<a id="ref-for-compound"></a>

<a id="ref-for-type-selector"></a>

While the position of a [nesting selector](#nesting-selector) in a [compound selector](https://www.w3.org/TR/selectors-4/#compound) does not make a difference in its behavior (that is, &#x26;.foo and .foo&#x26; match the same elements), the existing rule that a [type selector](https://www.w3.org/TR/selectors-4/#type-selector), if present, must be first in the <a id="ref-for-compound①"></a>compound selector continues to apply (that is, &#x26;div is illegal, and must be written div&#x26; instead).

Tests

- [contextually-invalid-selectors-001.html](https://wpt.fyi/results/css/css-nesting/contextually-invalid-selectors-001.html) [(live test)](http://wpt.live/css/css-nesting/contextually-invalid-selectors-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/contextually-invalid-selectors-001.html)
- [contextually-invalid-selectors-002.html](https://wpt.fyi/results/css/css-nesting/contextually-invalid-selectors-002.html) [(live test)](http://wpt.live/css/css-nesting/contextually-invalid-selectors-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/contextually-invalid-selectors-002.html)
- [contextually-invalid-selectors-003.html](https://wpt.fyi/results/css/css-nesting/contextually-invalid-selectors-003.html) [(live test)](http://wpt.live/css/css-nesting/contextually-invalid-selectors-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/contextually-invalid-selectors-003.html)
- [host-nesting-001.html](https://wpt.fyi/results/css/css-nesting/host-nesting-001.html) [(live test)](http://wpt.live/css/css-nesting/host-nesting-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/host-nesting-001.html)
- [host-nesting-002.html](https://wpt.fyi/results/css/css-nesting/host-nesting-002.html) [(live test)](http://wpt.live/css/css-nesting/host-nesting-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/host-nesting-002.html)
- [host-nesting-003.html](https://wpt.fyi/results/css/css-nesting/host-nesting-003.html) [(live test)](http://wpt.live/css/css-nesting/host-nesting-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/host-nesting-003.html)
- [host-nesting-004.html](https://wpt.fyi/results/css/css-nesting/host-nesting-004.html) [(live test)](http://wpt.live/css/css-nesting/host-nesting-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/host-nesting-004.html)
- [host-nesting-005.html](https://wpt.fyi/results/css/css-nesting/host-nesting-005.html) [(live test)](http://wpt.live/css/css-nesting/host-nesting-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/host-nesting-005.html)
- [nesting-type-selector.html](https://wpt.fyi/results/css/css-nesting/nesting-type-selector.html) [(live test)](http://wpt.live/css/css-nesting/nesting-type-selector.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nesting-type-selector.html)

## <a id="nested-declarations"></a>5.  The Nested Declarations Rule

For somewhat-technical reasons, it’s important to be able to distinguish properties that appear at the start of a style rule’s contents from those that appear interspersed with other rules.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b6fcbb18"></a> For example, in the following two rules:
>
> ```css
> .foo {
>   color: red;
>   @media (...) {...}
>   background: blue;
> }
> ```
>
> <a id="ref-for-propdef-color④"></a>
>
> <a id="ref-for-propdef-background"></a>
>
> <a id="ref-for-dom-cssstylerule-style"></a>
>
> <a id="ref-for-dom-cssgroupingrule-cssrules"></a>
>
> We need to treat the [color: red](https://www.w3.org/TR/css-color-4/#propdef-color) and [background: blue](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) slightly differently. In particular, in the CSSOM, the <a id="ref-for-propdef-color⑤"></a>color: red is exposed in the style rule’s <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssstylerule-style">style</a></code> attribute, while the <a id="ref-for-propdef-background①"></a>background: blue needs to instead show up in its <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssgroupingrule-cssrules">cssRules</a></code> list.

<a id="ref-for-style-rule⑥"></a>

<a id="ref-for-selectordef-①③"></a>

To accomplish this, CSS parsing <em>automatically</em> wraps such properties in a special child rule to contain them. However, if we were to wrap them in a [style rule](https://www.w3.org/TR/css-syntax-3/#style-rule) with an [&#x26;](#selectordef-) selector, it would have somewhat unfortunate behavior:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a4638964"></a> For example, in
>
> ```css
> .foo, .foo::before {
>   color: red;
>   & {
>     background: blue;
>   }
> }
> ```
>
> <a id="ref-for-propdef-background②"></a>
>
> <a id="ref-for-selectordef-①④"></a>
>
> the nested rule <em>does not</em> apply the [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) property to the .foo::before elements, because the [&#x26;](#selectordef-) can’t represent pseudo-elements.

<a id="ref-for-css-rule"></a>

<a id="ref-for-at-ruledef-media①"></a>

<a id="ref-for-dom-cssstylerule-style①"></a>

Similarly, child declarations in nested non-style rules need to be exposed as [rules](https://drafts.csswg.org/css-syntax-3/#css-rule) in some way, because these sorts of rules (like [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)) have never had <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssstylerule-style">style</a></code> properties. These run into the same problems as above.

To address all of these issue, we instead wrap runs of consecutive directly-nested properties in a <a id="nested-declarations-rule"></a>nested declarations rule.

<a id="ref-for-nested-declarations-rule④"></a>

<a id="ref-for-nested-style-rule①①"></a>

<a id="ref-for-selectordef-①⑤"></a>

Unless otherwise specified, a [nested declarations rule](#nested-declarations-rule) is a [nested style rule](#nested-style-rule), and acts identically to any other style rule. It matches the exact same elements and pseudo-elements as its parent style rule, with the same specificity behavior. <strong data-conversion-semantic="note">Note:</strong> (This is <em>similar to</em> being a style rule with an [&#x26;](#selectordef-) selector, but slightly more powerful, as explained above.)

<a id="ref-for-nested-declarations-rule⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why does the [nested declarations rule](#nested-declarations-rule) exist?
>
> <a id="ref-for-nested-group-rules⑥"></a>
>
> <a id="ref-for-selectordef-①⑥"></a>
>
> Originally, this specification grouped all declarations in style rules together, "moving" them from their original location to act as if they were placed at the front of the rule. It also automatically wrapped raw declarations inside of [nested group rules](#nested-group-rules) in plain style rules, using the [&#x26;](#selectordef-) selector.
>
> <a id="ref-for-nested-declarations-rule⑥"></a>
>
> There are two major reasons we switched to instead use the [nested declarations rule](#nested-declarations-rule).
>
> <a id="ref-for-nested-group-rules⑦"></a>
>
> <a id="ref-for-nested-declarations-rule⑦"></a>
>
> <a id="ref-for-at-ruledef-media②"></a>
>
> First, using an &#x26; {...} rule to implicitly wrap declarations in a [nested group rule](#nested-group-rules) also changed the behavior. As shown in the example following this note, it breaks cases where the parent style rule contains pseudo-elements, and even when that’s not the case, it potentially changes the specificity behavior of the nested declarations. Switching to the [nested declarations rule](#nested-declarations-rule) avoids these problems, making the behavior of nested [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)/etc identical to the behavior of \*non\*-nested <a id="ref-for-at-ruledef-media③"></a>@media/etc.
>
> <a id="ref-for-nested-declarations-rule⑧"></a>
>
> Second, there are some details of future CSS features (notably, "mixins") that simply won’t work correctly if interleaved declarations are automatically moved to the front of the style rule. We need to keep their relative order with other rules, and in order to actually make that representable in the CSSOM, that means they have to be wrapped in some kind of rule. The same issues as the previous paragraph apply if we just use a normal &#x26; {...} rule, so the [nested declarations rule](#nested-declarations-rule) lets us do so without side effects.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cef45fb3"></a> For example, in the following stylesheet snippet:
>
> ```css
> .foo, .foo::before, .foo::after {
>   color: black;
>   @media (prefers-color-scheme: dark) {
>     & {
>       color: white;
>     }
>   }
> }
> ```
>
> <a id="ref-for-selectordef-before"></a>
>
> <a id="ref-for-selectordef-after"></a>
>
> <a id="ref-for-selectordef-①⑦"></a>
>
> In a darkmode page, the .foo element would have its text color changed to white, but its [::before](https://www.w3.org/TR/css-pseudo-4/#selectordef-before) and [::after](https://www.w3.org/TR/css-pseudo-4/#selectordef-after) pseudos would remain black, because the [&#x26;](#selectordef-) selector can’t represent pseudo-elements.
>
> However, it was instead written as:
>
> ```css
> .foo, .foo::before, .foo::after {
>   color: black;
>   @media (prefers-color-scheme: dark) {
>     color: white;
>   }
> }
> ```
>
> <a id="ref-for-propdef-color⑥"></a>
>
> <a id="ref-for-nested-declarations-rule⑨"></a>
>
> Then the [color: white](https://www.w3.org/TR/css-color-4/#propdef-color) is implicitly wrapped in a [nested declarations rule](#nested-declarations-rule), which is guaranteed to match <em>exactly</em> the same as its parent style rule, so the element <em>and</em> its pseudo-elements would all have white text in a darkmode page.

<a id="ref-for-nested-declarations-rule①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3bca626"></a> Declarations interleaved with rules get implicitly wrapped in a [nested declarations rule](#nested-declarations-rule), which makes them part of a separate style rule. For example, given this CSS:
>
> ```css
> .foo {
>   color: black;
>   @media (...) {...}
>   background: silver;
> }
> ```
>
> <a id="ref-for-dom-cssstylerule-style②"></a>
>
> <a id="ref-for-propdef-color⑦"></a>
>
> If the .foo rule’s CSSOM object is examined, its <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssstylerule-style">style</a></code> attribute will contain only one declaration: the [color: black](https://www.w3.org/TR/css-color-4/#propdef-color) one.
>
> <a id="ref-for-propdef-background③"></a>
>
> <a id="ref-for-nested-declarations-rule①①"></a>
>
> The [background: silver](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) declaration will instead be found in the implicitly-created [nested declarations child rule](#nested-declarations-rule), at `fooRule.cssRules[1].style`.

Tests

- mixed-declarations-rules.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/mixed-declarations-rules.html)
- [nested-declarations-cssom-whitespace.html](https://wpt.fyi/results/css/css-nesting/nested-declarations-cssom-whitespace.html) [(live test)](http://wpt.live/css/css-nesting/nested-declarations-cssom-whitespace.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nested-declarations-cssom-whitespace.html)
- [nested-declarations-cssom.html](https://wpt.fyi/results/css/css-nesting/nested-declarations-cssom.html) [(live test)](http://wpt.live/css/css-nesting/nested-declarations-cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nested-declarations-cssom.html)
- [nested-declarations-matching.html](https://wpt.fyi/results/css/css-nesting/nested-declarations-matching.html) [(live test)](http://wpt.live/css/css-nesting/nested-declarations-matching.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/nested-declarations-matching.html)

## <a id="cssom"></a>6. CSSOM

<a id="ref-for-cssstylerule"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\[CSSOM-1\]](#biblio-cssom-1) now defines that <code><a href="https://www.w3.org/TR/cssom-1/#cssstylerule">CSSStyleRule</a></code> can have child rules.

<a id="ref-for-relative-selector⑥"></a>

<a id="ref-for-nested-style-rule①②"></a>

<a id="ref-for-nesting-selector①②"></a>

When serializing a [relative selector](https://www.w3.org/TR/selectors-4/#relative-selector) in a [nested style rule](#nested-style-rule), the selector must be absolutized, with the implied [nesting selector](#nesting-selector) inserted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-724b564c"></a> For example, the selector \> .foo will serialize as &#x26; \> .foo.

Tests

- [cssom.html](https://wpt.fyi/results/css/css-nesting/cssom.html) [(live test)](http://wpt.live/css/css-nesting/cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/cssom.html)
- [set-selector-text.html](https://wpt.fyi/results/css/css-nesting/set-selector-text.html) [(live test)](http://wpt.live/css/css-nesting/set-selector-text.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/set-selector-text.html)

<a id="ref-for-cssnesteddeclarations"></a>

### <a id="the-cssnestrule"></a>6.1. The <code><a href="#cssnesteddeclarations">CSSNestedDeclarations</a></code> Interface

<a id="ref-for-cssnesteddeclarations①"></a>

<a id="ref-for-nested-declarations-rule①②"></a>

The <code><a href="#cssnesteddeclarations">CSSNestedDeclarations</a></code> interface represents a [nested declarations rule](#nested-declarations-rule).

<a id="ref-for-Exposed"></a>

<a id="cssnesteddeclarations"></a>

<a id="ref-for-cssrule"></a>

<a id="ref-for-SameObject"></a>

<a id="ref-for-PutForwards"></a>

<a id="ref-for-cssstyleproperties"></a>

<a id="ref-for-dom-cssnesteddeclarations-style"></a>

```text
[Exposed=Window]
interface CSSNestedDeclarations : CSSRule {
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};
```
<a id="ref-for-cssstyleproperties①"></a>

The <a id="dom-cssnesteddeclarations-style"></a>`style` attribute must return a <code><a href="https://drafts.csswg.org/cssom-1/#cssstyleproperties">CSSStyleProperties</a></code> object for the rule, with the following properties:

<a id="ref-for-cssstyledeclaration-computed-flag"></a>

[computed flag](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-computed-flag)

Unset

<a id="ref-for-cssstyledeclaration-readonly-flag"></a>

[readonly flag](https://drafts.csswg.org/cssom-1/#cssstyledeclaration-readonly-flag)

Unset

<a id="ref-for-cssstyledeclaration-declarations①"></a>

[declarations](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-declarations)

<a id="ref-for-concept-declarations-specified-order"></a>

The declared declarations in the rule, in [specified order](https://www.w3.org/TR/cssom-1/#concept-declarations-specified-order).

<a id="ref-for-cssstyledeclaration-parent-css-rule"></a>

[parent CSS rule](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-parent-css-rule)

<a id="ref-for-this"></a>

[this](https://webidl.spec.whatwg.org/#this)

<a id="ref-for-cssstyledeclaration-owner-node"></a>

[owner node](https://www.w3.org/TR/cssom-1/#cssstyledeclaration-owner-node)

Null

<a id="ref-for-cssnesteddeclarations②"></a>

<a id="ref-for-serialize-a-css-rule"></a>

<a id="ref-for-css-declaration-block"></a>

<a id="ref-for-serialize-a-css-declaration-block"></a>

The <code><a href="#cssnesteddeclarations">CSSNestedDeclarations</a></code> rule [serializes](https://www.w3.org/TR/cssom-1/#serialize-a-css-rule) as if its [declaration block](https://www.w3.org/TR/cssom-1/#css-declaration-block) had been [serialized](https://www.w3.org/TR/cssom-1/#serialize-a-css-declaration-block) directly.

Tests

- [serialize-group-rules-with-decls.html](https://wpt.fyi/results/css/css-nesting/serialize-group-rules-with-decls.html) [(live test)](http://wpt.live/css/css-nesting/serialize-group-rules-with-decls.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-nesting/serialize-group-rules-with-decls.html)

<a id="ref-for-nested-declarations-rule①③"></a>

<a id="ref-for-dom-cssgroupingrule-insertrule"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that multiple adjacent [nested declarations rules](#nested-declarations-rule) (which is possible to create with e.g. <code><a href="https://www.w3.org/TR/cssom-1/#dom-cssgroupingrule-insertrule">insertRule</a></code>) will collapse into a single rule when serialized and parsed again.

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

## <a id="changes"></a>7. Changes

Significant changes since the [Feb 14, 2023 Working Draft](https://www.w3.org/TR/2023/WD-css-nesting-1-20230214/):

- <a id="ref-for-typedef-scope-start①"></a>

  <a id="ref-for-at-ruledef-scope④"></a>

  The [\<scope-start\>](https://www.w3.org/TR/css-cascade-6/#typedef-scope-start) selector of a [@scope](https://www.w3.org/TR/css-cascade-6/#at-ruledef-scope) rule no longer acts as the parent rule for nesting. ([Issue 9740](https://github.com/w3c/csswg-drafts/issues/9740))

- <a id="ref-for-nesting-selector①③"></a>

  Clarified that the [nesting selector](#nesting-selector) is allowed to match featureless elements.

- Switched &#x26;div back to being invalid; now that Syntax does "infinite lookahead", we no longer need to allow it. Plus, doing so avoids a clash with preprocessors. ([Issue 8662](https://github.com/w3c/csswg-drafts/issues/8662))

- CSSOM now defines that CSSStyleRule is a CSSGroupingRule subclass, so the manual definition of the `cssRules` attribute and related machinery was removed. ([Issue 8940](https://github.com/w3c/csswg-drafts/issues/8940))

- Clarified the effect of the <em>implied</em> nesting selector on specificity. ([Issue 9069](https://github.com/w3c/csswg-drafts/issues/9069))

- Declarations intermixed with rules (or all declarations in nested group rules) are now automatically wrapped in `@nest` rules. (Also the `@nest` rule was added.) ([Issue 8738](https://github.com/w3c/csswg-drafts/issues/8738))

- <a id="ref-for-nested-declarations-rule①④"></a>

  Replaced `@nest` with [nested declarations rules](#nested-declarations-rule). ([Issue 10234](https://github.com/w3c/csswg-drafts/issues/10234))

- Added Web Platform Tests coverage.

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

- [&#x26;](#selectordef-), in § 4
- [contain the nesting selector](#contain-the-nesting-selector), in § 3.1
- [CSSNestedDeclarations](#cssnesteddeclarations), in § 6.1
- [nested declarations rule](#nested-declarations-rule), in § 5
- [nested group rules](#nested-group-rules), in § 3.3
- [nested style rule](#nested-style-rule), in § 3
- [nesting selector](#nesting-selector), in § 4
- [nesting style rule](#nested-style-rule), in § 3
- [style](#dom-cssnesteddeclarations-style), in § 6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="db6870d5"></a>background
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="08b3934a"></a>@layer
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="a308d237"></a>\<scope-start\>
  - <a id="aeedf61e"></a>@scope
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="bcdf9b19"></a>color
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="4397147f"></a>@media
  - <a id="a5d6c9d2"></a>@supports
  - <a id="f1a41224"></a>conditional group rule
- \[CSS-CONDITIONAL-5\] defines the following terms:
  - <a id="d56fefb4"></a>@container
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="70503bd6"></a>::after
  - <a id="7c6f51b7"></a>::before
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="b845e85c"></a>\<block-contents\>
  - <a id="f9309bf7"></a>\<delim-token\>
  - <a id="a8cb81d7"></a>\<rule-list\>
  - <a id="b29fecf5"></a>at-rule
  - <a id="67800454"></a>parse
  - <a id="4fcdc12a"></a>rule
  - <a id="87d90aed"></a>style rule
- \[CSS-UI-4\] defines the following terms:
  - <a id="1279e2aa"></a>bar
- \[CSSOM-1\] defines the following terms:
  - <a id="0f78dbdd"></a>CSSRule
  - <a id="f9bcb636"></a>CSSStyleProperties
  - <a id="d293a05b"></a>CSSStyleRule
  - <a id="70899879"></a>computed flag
  - <a id="ff26bbb9"></a>CSS declaration block
  - <a id="c6758e4f"></a>cssRules
  - <a id="9852f862"></a>declarations
  - <a id="3f8f9b90"></a>insertRule(rule)
  - <a id="451490eb"></a>owner node
  - <a id="56031446"></a>parent CSS rule
  - <a id="6eabcb67"></a>readonly flag
  - <a id="4bdd675c"></a>serialize a CSS declaration block
  - <a id="3f2d4635"></a>serialize a CSS rule
  - <a id="ec9c26d7"></a>specified order
  - <a id="5f56bd13"></a>style
- \[SELECTORS-4\] defines the following terms:
  - <a id="e94d3b11"></a>:is()
  - <a id="9fca4587"></a>:scope
  - <a id="c77f7ef7"></a>:where()
  - <a id="b7d040a0"></a>\<forgiving-selector-list\>
  - <a id="0af7e6f0"></a>\<relative-selector-list\>
  - <a id="1fd14124"></a>\<selector-list\>
  - <a id="cc53c5e6"></a>combinator
  - <a id="31c47bc6"></a>compound selector
  - <a id="003a1eca"></a>descendant combinator
  - <a id="09a6f82f"></a>featureless
  - <a id="09bedad5"></a>relative selector
  - <a id="76e5295f"></a>simple selector
  - <a id="ba72ce8f"></a>specificity
  - <a id="15e81c46"></a>type selector
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed
  - <a id="21ecf38f"></a>PutForwards
  - <a id="a5c91173"></a>SameObject
  - <a id="4013a022"></a>this

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 6 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
Chris Lilley; et al. [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/css-conditional-5/). 30 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-5&#x2F;](https://www.w3.org/TR/css-conditional-5/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 20 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface CSSNestedDeclarations : CSSRule {
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleProperties style;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The preceding paragraph needs to move to Selectors when we move [&#x26;](#selectordef-) itself to Selectors; I’m monkey-patching for convenience here. [↵](#issue-03f974e2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We’d like to relax this restriction, but need to do so simultaneously for both [:is()](https://www.w3.org/TR/selectors-4/#matches-pseudo) and [&#x26;](#selectordef-), since they’re intentionally built on the same underlying mechanisms. ([Issue 7433](https://github.com/w3c/csswg-drafts/issues/7433)) [↵](#issue-62c479a8)
