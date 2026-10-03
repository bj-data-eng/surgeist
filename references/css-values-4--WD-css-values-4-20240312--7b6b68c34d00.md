Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 4](https://www.w3.org/TR/2024/WD-css-values-4-20240312/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 4

Source snapshot: https://www.w3.org/TR/2024/WD-css-values-4-20240312/

Snapshot SHA-256: 7b6b68c34d00d7f6945e66e4e2efa913266299f597d5825393cae85dac1cc78c

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 5 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Values and Units Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-values” in the title, like this: “\[css-values\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-values%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<a id="ref-for-length-value"></a>

The value definition field of each CSS property can contain keywords, data types (which appear between \< and \>), and information on how they can be combined. Generic data types ([\<length\>](#length-value) being the most widely used) that can be used by many properties are described in this specification, while more specific data types (e.g., \<spacing-limit\>) are described in the corresponding modules.

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the data type definitions in [\[CSS2\]](#biblio-css2) sections [1.4.2.1](https://www.w3.org/TR/CSS2/about.html#value-defs), [4.3](https://www.w3.org/TR/CSS2/syndata.html#values), and [A.2](https://www.w3.org/TR/CSS2/aural.html#aural-intro).

## <a id="value-defs"></a>2.  Value Definition Syntax

The <a id="css-value-definition-syntax"></a>value definition syntax described here is used to define the set of valid values for CSS properties (and the valid syntax of many other parts of CSS). A value so described can have one or more components.

### <a id="component-types"></a>2.1.  Component Value Types

Component value types are designated in several ways:

1.  <a id="ref-for-disc"></a>

    [Keyword](#keywords) values (such as auto, [disc](https://www.w3.org/TR/css-counter-styles-3/#disc), etc.), which appear literally, without quotes (e.g. `auto`).

2.  <a id="ref-for-length-value①"></a>

    <a id="ref-for-percentage-value"></a>

    <a id="ref-for-numeric-data-types"></a>

    Basic data types, which appear between \< and \> (e.g., [\<length\>](#length-value), [\<percentage\>](#percentage-value), etc.). For [numeric data types](#numeric-data-types), this type notation can annotate any range restrictions using the [bracketed range notation](#numeric-ranges) described below.

3.  <a id="ref-for-propdef-border-width"></a>

    <a id="ref-for-propdef-background-attachment"></a>

    Property value ranges, which represent the same pattern of values as a property bearing the same name. These are written as the property name, surrounded by single quotes, between \< and \>, e.g., [\<'border-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width), [\<'background-attachment'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-attachment), etc.

    <a id="ref-for-valdef-all-inherit"></a>

    <a id="ref-for-mult-comma"></a>

    <a id="ref-for-identifier-value"></a>

    <a id="ref-for-integer-value"></a>

    <a id="ref-for-identifier-value①"></a>

    <a id="ref-for-integer-value①"></a>

    <a id="ref-for-identifier-value②"></a>

    <a id="ref-for-integer-value②"></a>

    These types <em>do not</em> include [CSS-wide keywords](#common-keywords) such as [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit). Additionally, if the property’s value grammar is a [comma-separated repetition](#mult-comma), the corresponding type does not include the top-level <a id="ref-for-mult-comma①"></a>comma-separated list multiplier. (E.g. if a property named pairing is defined as \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \]#, then \<'pairing'\> is equivalent to \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \], not \[ [\<custom-ident\>](#identifier-value) [\<integer\>](#integer-value)? \]#.)&#x5C;

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Why remove the multiplier?
    > <a id="ref-for-coordinating-list-property"></a>
    >
    > The top-level multiplier is ripped out of these value types because top-level comma-separated repetitions are mostly used for [coordinating list properties](#coordinating-list-property), and when a shorthand combines several such properties, it needs the unmultiplied grammar so it can construct its <em>own</em> comma-separated repetition.
    >
    > Without this special treatment, every such longhand would have to be defined with an ad-hoc production just for the inner value, which makes the grammars harder to understand overall.

4.  <a id="ref-for-funcdef-calc"></a>

    <a id="ref-for-functional-notation"></a>

    Functional notations and their arguments. These are written as the function’s name, followed by an empty parentheses pair, between \< and \>, e.g. [\<calc()\>](#funcdef-calc), and references the correspondingly-named [functional notation](#functional-notation).

5.  <a id="ref-for-value-def-border-width"></a>

    <a id="ref-for-propdef-border-width①"></a>

    Other non-terminals. These are written as the name of the non-terminal between \< and \>, as in \<spacing-limit\>. Notice the distinction between [\<border-width\>](https://www.w3.org/TR/CSS21/box.html#value-def-border-width) and [\<'border-width'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width): the latter represents the grammar of the <a id="ref-for-propdef-border-width②"></a>border-width property, the former requires an explicit expansion elsewhere. The definition of a non-terminal is typically located near its first appearance in the specification.

Some property value definitions also include the slash (/), the comma (,), and/or parentheses as literals. These represent their corresponding tokens. Other non-keyword literal characters that may appear in a component value, such as “+”, must be written enclosed in single quotes.

<strong><dfn><span><a id="comb-comma"></a></span>Commas</dfn> specified in the grammar are implicitly omissible</strong> in some circumstances, when used to separate optional terms in the grammar. Within a top-level list in a property or other CSS value, or a function’s argument list, a comma specified in the grammar must be omitted if:

- all items preceding the comma have been omitted
- all items following the comma have been omitted
- multiple commas would be adjacent (ignoring [white space](https://www.w3.org/TR/css-syntax/#whitespace)/comments), due to the items between the commas being omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c02dd41d"></a> For example, if a function can accept three arguments in order, but all of them are optional, the grammar can be written like:
>
> <a id="ref-for-mult-opt"></a>
>
> <a id="ref-for-comb-comma"></a>
>
> <a id="ref-for-mult-opt①"></a>
>
> <a id="ref-for-comb-comma①"></a>
>
> <a id="ref-for-mult-opt②"></a>
>
> ```text
> example( first? , second? , third? )
> ```
>
> Given this grammar, writing example(first, second, third) is valid, as is example(first, second) or example(first, third) or example(second). However, example(first, , third) is invalid, as one of those commas are no longer separating two options; similarly, example(,second) and example(first,) are invalid. example(first second) is also invalid, as commas are still required to actually separate the options.
>
> If commas were not implicitly omittable, the grammar would have to be much more complicated to properly express the ways that the arguments can be omitted, greatly obscuring the simplicity of the feature.

<a id="ref-for-propdef-border-color"></a>

All CSS properties also accept the [CSS-wide keyword values](#common-keywords) as the sole component of their property value. For readability these are not listed explicitly in the property value syntax definitions. For example, the full value definition of [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color) under [CSS Cascading and Inheritance Level 3](#biblio-css-cascade-3) is `<color>{1,4} | inherit | initial | unset` (even though it is listed as `<color>{1,4}`).

<a id="ref-for-propdef-background"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This implies that, in general, combining these keywords with other component values in the same declaration results in an invalid declaration. For example, [background: url(corner.png) no-repeat, inherit;](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) is invalid.

### <a id="component-combinators"></a>2.2.  Component Value Combinators

Component values can be arranged into property values as follows:

- Juxtaposing components means that all of them must occur, in the given order.
- A double ampersand (<a id="comb-all"></a>&#x26;&#x26;) separates two or more components, all of which must occur, in any order.
- A double bar (<a id="comb-any"></a>\|\|) separates two or more options: one or more of them must occur, in any order.
- A bar (<a id="comb-one"></a>\|) separates two or more alternatives: exactly one of them must occur.
- Brackets (\[ \]) are for grouping.

Juxtaposition is stronger than the double ampersand, the double ampersand is stronger than the double bar, and the double bar is stronger than the bar. Thus, the following lines are equivalent:

```text
  a b   |   c ||   d &&   e f
[ a b ] | [ c || [ d && [ e f ]]]
```
For reorderable combinators (\|\|, &#x26;&#x26;), ordering of the grammar does not matter: components in the same grouping may be interleaved in any order. Thus, the following lines are equivalent:

```text
a || b || c
b || a || c
```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Combinators are <em>not</em> associative, so grouping is significant. For example, a \|\| b \|\| c and a \|\| \[ b \|\| c \] are distinct grammars: the first allows a value like b a c, but the second does not.

### <a id="component-multipliers"></a>2.3.  Component Value Multipliers

Every type, keyword, or bracketed group may be followed by one of the following modifiers:

- An asterisk (<a id="mult-zero-plus"></a>\*) indicates that the preceding type, word, or group occurs zero or more times.
- A plus (<a id="mult-one-plus"></a>+) indicates that the preceding type, word, or group occurs one or more times.
- A question mark (<a id="mult-opt"></a>?) indicates that the preceding type, word, or group is optional (occurs zero or one times).
- A single number in curly braces (<a id="mult-num"></a>{<var>A</var>}) indicates that the preceding type, word, or group occurs <var>A</var> times.
- A comma-separated pair of numbers in curly braces (<a id="mult-num-range"></a>{<var>A</var>,<var>B</var>}) indicates that the preceding type, word, or group occurs at least <var>A</var> and at most <var>B</var> times. The <var>B</var> may be omitted ({<var>A</var>,}) to indicate that there must be at least <var>A</var> repetitions, with no upper bound on the number of repetitions.
- A hash mark (<a id="mult-comma"></a>\#) indicates that the preceding type, word, or group occurs one or more times, separated by comma tokens (which may optionally be surrounded by [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments). It may optionally be followed by the curly brace forms, above, to indicate precisely how many times the repetition occurs, like \<length\>#{1,4}.
- An exclamation point (<a id="mult-req"></a>!) after a group indicates that the group is required and must produce at least one value; even if the grammar of the items within the group would otherwise allow the entire contents to be omitted, at least one component value must not be omitted.

The + and \# multipliers may be stacked as +#; similarly, the \# and ? multipliers may be stacked as \#?. These stacks each represent the later multiplier applied to the result of the earlier multiplier. (These same stacks can be represented using grouping, but in complex grammars this can push the number of brackets beyond readability.)

<a id="ref-for-user-agent"></a>

For repeated component values (indicated by \*, +, or \#), [UAs](https://www.w3.org/TR/css-2023/#user-agent) must support at least 20 repetitions of the component. If a property value contains more than the supported number of repetitions, the declaration must be ignored as if it were invalid.

### <a id="combinator-multiplier-patterns"></a>2.4.  Combinator and Multiplier Patterns

<a id="ref-for-component-value"></a>

There are a small set of common ways to combine multiple independent [component values](https://www.w3.org/TR/css-syntax-3/#component-value) in particular numbers and orders. In particular, it’s common to want to express that, from a set of component value, the author must select zero or more, one or more, or all of them, and in either the order specified in the grammar or in any order.

All of these can be easily expressed using simple patterns of [combinators](#component-combinators) and [multipliers](#component-multipliers):

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

<strong>Column 2 (header cell):</strong>

in order

<strong>Column 3 (header cell):</strong>

any order

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

zero or more

<strong>Column 2 (data cell):</strong>

`A? B? C?`

<strong>Column 3 (data cell):</strong>

`A? || B? || C?`

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

one or more

<strong>Column 2 (data cell):</strong>

`[ A? B? C? ]!`

<strong>Column 3 (data cell):</strong>

`A || B || C`

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

all

<strong>Column 2 (data cell):</strong>

` A B C  `

<strong>Column 3 (data cell):</strong>

`A && B && C`

Note that all of the "any order" possibilities are expressed using combinators, while the "in order" possibilities are all variants on juxtaposition.

### <a id="component-whitespace"></a>2.5.  Component Values and White Space

Unless otherwise specified, [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments may appear before, after, and/or between components combined using the above [combinators](#component-combinators) and [multipliers](#component-multipliers).

<a id="ref-for-typedef-dimension-token"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In many cases, spaces will in fact be <em>required</em> between components in order to distinguish them from each other. For example, the value 1em2em would be parsed as a single [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) with the number 1 and the identifier em2em, which is an invalid unit. In this case, a space would be required before the 2 to get this parsed as the two lengths 1em and 2em.

### <a id="component-functions"></a>2.6.  Functional Notation Definitions

<a id="ref-for-functional-notation①"></a>

The syntax of a [functional notation](#functional-notation) is defined as a sequence of:

1.  <a id="ref-for-typedef-function-token"></a>

    The function’s name written as an identifier followed by an open parenthesis (such as example(), or the [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token) production to indicate a function with an arbitrary name.

2.  <a id="ref-for-css-value-definition-syntax"></a>

    The function’s arguments, if any, expressed using the [value definition syntax](#css-value-definition-syntax).

3.  A literal closing parenthesis.

The function’s arguments are considered <em>implicitly grouped</em>, as if surrounded by brackets (\[ ... \]).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0cdbfbca"></a> For example, a grammar like:
>
> <a id="ref-for-comb-comma②"></a>
>
> ```text
> example( <length> , <length> )
> ```
>
> <a id="ref-for-length-value②"></a>
>
> will match a function whose name is "example" and whose arguments match "[\<length\>](#length-value) , <a id="ref-for-length-value③"></a>\<length\>".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-99e5c2a2"></a> For example, the Selectors grammar defines pseudo-classes generically, allowing any possibly function name after the initial colon:
>
> <a id="ref-for-comb-one"></a>
>
> ```text
> <pseudo-class-selector> = ':' <ident-token> | ':' <function-token> <any-value> ')'
> ```
>
> <a id="ref-for-typedef-any-value"></a>
>
> This represents <em>any</em> function name, with [\<any-value\>](https://www.w3.org/TR/css-syntax-3/#typedef-any-value) as the function arguments.

<a id="ref-for-functional-notation②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-885ed9bc"></a> Since the [functional notation](#functional-notation) <em>implicitly groups</em> its contents, the effect of any combinator inside it is scoped to the function’s argument. For example, the <a id="ref-for-functional-notation③"></a>functional notation syntax definition example( foo \| bar ) is equivalent to example( \[ foo \| bar \] ).

### <a id="value-examples"></a>2.7.  Property Value Examples

Below are some examples of properties with their corresponding value definition fields

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-78954521"></a>
>
> <a id="propvalues"></a>
>
> <strong>Table 2 — structured row/cell transcription</strong>
>
> <strong>Row 1</strong>
>
> <strong>Column 1 (header cell):</strong>
>
> Property
>
> <strong>Column 2 (header cell):</strong>
>
> Value definition field
>
> <strong>Column 3 (header cell):</strong>
>
> Example value
>
> <strong>Row 2</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-orphans"></a>
>
> [orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans)
>
> <strong>Column 2 (data cell):</strong>
>
> \<integer\>
>
> <strong>Column 3 (data cell):</strong>
>
> 3
>
> <strong>Row 3</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-text-align"></a>
>
> [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)
>
> <strong>Column 2 (data cell):</strong>
>
> left \| right \| center \| justify
>
> <strong>Column 3 (data cell):</strong>
>
> <a id="ref-for-valdef-text-align-center"></a>
>
> [center](https://www.w3.org/TR/css-text-4/#valdef-text-align-center)
>
> <strong>Row 4</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-padding-top"></a>
>
> [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)
>
> <strong>Column 2 (data cell):</strong>
>
> \<length\> \| \<percentage\>
>
> <strong>Column 3 (data cell):</strong>
>
> 5%
>
> <strong>Row 5</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-outline-color"></a>
>
> [outline-color](https://www.w3.org/TR/css-ui-4/#propdef-outline-color)
>
> <strong>Column 2 (data cell):</strong>
>
> \<color\> \| invert
>
> <strong>Column 3 (data cell):</strong>
>
> \#fefefe
>
> <strong>Row 6</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-text-decoration"></a>
>
> [text-decoration](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration)
>
> <strong>Column 2 (data cell):</strong>
>
> none \| underline \|\| overline \|\| line-through \|\| blink
>
> <strong>Column 3 (data cell):</strong>
>
> overline underline
>
> <strong>Row 7</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-font-family"></a>
>
> [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family)
>
> <strong>Column 2 (data cell):</strong>
>
> \[ \<family-name\> \| \<generic-family\> \]#
>
> <strong>Column 3 (data cell):</strong>
>
> "Gill Sans", Futura, sans-serif
>
> <strong>Row 8</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-border-width③"></a>
>
> [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)
>
> <strong>Column 2 (data cell):</strong>
>
> \[ \<length\> \| thick \| medium \| thin \]{1,4}
>
> <strong>Column 3 (data cell):</strong>
>
> 2px medium 4px
>
> <strong>Row 9</strong>
>
> <strong>Column 1 (data cell):</strong>
>
> <a id="ref-for-propdef-box-shadow"></a>
>
> [box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow)
>
> <strong>Column 2 (data cell):</strong>
>
> \[ inset? &#x26;&#x26; \<length\>{2,4} &#x26;&#x26; \<color\>? \]# \| none
>
> <strong>Column 3 (data cell):</strong>
>
> 3px 3px rgba(50%, 50%, 50%, 50%), lemonchiffon 0 0 4px inset

### <a id="production-blocks"></a>2.8.  Non-Terminal Definitions and Grammar Production Blocks

<a id="ref-for-typedef-position"></a>

<a id="ref-for-funcdef-calc①"></a>

The precise grammar of non-terminals, like [\<position\>](#typedef-position) or [\<calc()\>](#funcdef-calc), is often specified in a <a id="css-grammar-production-block"></a>CSS grammar production block. These are conventionally represented in a preformatted block of definitions like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b4765280"></a> The \<foo\> syntax is defined as follows:
>
> <a id="ref-for-comb-one①"></a>
>
> <a id="ref-for-comb-one②"></a>
>
> ```text
> <foo> = keyword | <bar> |
>         some-really-long-pattern-of-stuff
> <bar> = <length>
> ```
<a id="ref-for-css-value-definition-syntax①"></a>

Each definition starts on its own line, and consists of the non-terminal to be defined, followed by an `=`, followed by the fragment of [value definition syntax](#css-value-definition-syntax) to which it expands. A definition can stretch across multiple lines, and terminates before the next line that starts a new grammar production or at the end of the grammar production block (whichever comes first).

<a id="ref-for-css-value-definition-syntax②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3cb12b0e"></a> In the above example, the \<foo\> definition covers two lines. The third line starts a new definition for \<bar\>. (A naked `=` is never valid in [value definition syntax](#css-value-definition-syntax), so it’s unambiguous when a new line starts a fresh definition.)

## <a id="combining-values"></a>3.  Combining Values: Interpolation, Addition, and Accumulation

<a id="ref-for-computed-value"></a>

Some procedures, for example [transitions](https://www.w3.org/TR/css-transitions/) and [animations](https://www.w3.org/TR/css-animations/), <a id="combine"></a>combine two CSS property values. The following combining operations—​on the two [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var> yielding the <a id="ref-for-computed-value①"></a>computed value <var>V<sub>result</sub></var>—​are defined. For operations that are not commutative (for example, matrix multiplication, or accumulation of mismatched transform lists) <var>V<sub>A</sub></var> represents the first term of the operation and <var>V<sub>B</sub></var> represents the second.

<a id="interpolation"></a>interpolation  
Given two property values <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var>, produces an intermediate value <var>V<sub>result</sub></var> at a distance of <var>p</var> along the interval between <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var> such that <var>p</var> = 0 produces <var>V<sub>A</sub></var> and <var>p</var> = 1 produces <var>V</var><sub>B</sub>.

<a id="ref-for-easing-function"></a>

The range of <var>p</var> is (−∞, ∞) due to the effect of [timing functions](https://www.w3.org/TR/css-easing-1/#easing-function). As a result, this procedure must also define extrapolation behavior for <var>p</var> outside \[0, 1\].

<a id="addition"></a>addition  
Given two property values <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var>, returns the sum of the two properties, <var>V</var><sub>result</sub>.

<a id="ref-for-addition"></a>

<a id="ref-for-interpolation"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While [addition](#addition) can often be expressed in terms of the same weighted sum function used to define [interpolation](#interpolation), this is not always the case. For example, interpolation of transform matrices involves decomposing and interpolating the matrix components whilst addition relies on matrix multiplication.

<a id="ref-for-addition①"></a>

If a value type does not define a specific procedure for [addition](#addition) or is defined as <a id="not-additive"></a>not additive, its <a id="ref-for-addition②"></a>addition operation is simply <var>V<sub>result</sub></var> = <var>V<sub>B</sub></var>.

<a id="accumulation"></a>accumulation  
Given two property values <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var>, returns the result, <var>V<sub>result</sub></var>, of combining the two operands such that <var>V<sub>B</sub></var> is treated as a <em>delta</em> from <var>V<sub>A</sub></var>.

<a id="ref-for-accumulation"></a>

<a id="ref-for-addition③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For many types of animation such as numbers or lengths, [accumulation](#accumulation) is defined to be identical to [addition](#addition).
> <a id="ref-for-addition④"></a>
>
> <a id="ref-for-accumulation①"></a>
>
> A common case where the definitions differ is for list-based types where [addition](#addition) may be defined as appending to a list whilst [accumulation](#accumulation) may be defined as component-based addition. For example, the filter list values blur(2) and blur(3), when <a id="ref-for-addition⑤"></a>added together would produce blur(2) blur(3), but when <a id="ref-for-accumulation②"></a>accumulated would produce blur(5).

<a id="ref-for-accumulation③"></a>

<a id="ref-for-addition⑥"></a>

If a value type does not define a specific procedure for [accumulation](#accumulation), its <a id="ref-for-accumulation④"></a>accumulation operation is identical to [addition](#addition).

<a id="ref-for-computed-value②"></a>

<a id="ref-for-length-value④"></a>

<a id="ref-for-canonical-unit"></a>

These operations are only defined on [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value). (As a result, it is not necessary to define, for example, how to add a [\<length\>](#length-value) value of 15pt with 5em since such values will be resolved to their [canonical unit](#canonical-unit) before being passed to any of the above procedures.)

### <a id="combining-range"></a>3.1.  Range Checking

<a id="ref-for-easing-function①"></a>

<a id="ref-for-math-function"></a>

Interpolation can result in a value outside the valid range for a property, even if all of the inputs to interpolation are valid; this especially happens when <var>p</var> is outside the \[0, 1\] range, but some [easing functions](https://www.w3.org/TR/css-easing-1/#easing-function) can cause this to occur even within that range. If the final result <em>after</em> interpolation, addition, and accumulation is out-of-range for the target context the value is being used in, it does not cause the declaration to be invalid. Instead, the value must be clamped to the range allowed in the target context, exactly the same as [math functions](#math-function) (see [§ 10.12 Range Checking](#calc-range)).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even if interpolation results in an out-of-range value, addition/accumulation might "correct" the result and bring it back into range. Thus, clamping is only applied to the <em>final</em> result of applying all interpolation-related operations.

## <a id="textual-values"></a>4.  Textual Data Types

<a id="ref-for-string-value"></a>

<a id="ref-for-url-value"></a>

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value③"></a>

The <a id="css-textual-data-types"></a>textual data types include various keywords and identifiers as well as strings ([\<string\>](#string-value)) and URLs ([\<url\>](#url-value)). Aside from the casing of [pre-defined keywords](#keywords) or as explicitly defined for a given property, no normalization is performed, not even Unicode normalization: the [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a property are exactly the provided Unicode values after parsing (which includes character set conversion and [escaping](https://www.w3.org/TR/css-syntax-3/#escaping)). [\[UNICODE\]](#biblio-unicode) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3)

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-css-css-identifier"></a>

CSS <a id="css-css-identifier"></a>identifiers, generically denoted by <a id="typedef-ident"></a>\<ident\>, consist of a sequence of characters conforming to the [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token) grammar. [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3) Identifiers cannot be quoted; otherwise they would be interpreted as strings. CSS properties accept two classes of [identifiers](#css-css-identifier): [pre-defined keywords](#keywords) and [author-defined identifiers](#custom-idents).

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-identifier-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<ident\>](#typedef-ident) production is not meant for property value definitions—​[\<custom-ident\>](#identifier-value) should be used instead. It is provided as a convenience for defining other syntactic constructs.

<a id="ref-for-interpolation①"></a>

<a id="ref-for-discrete"></a>

<a id="ref-for-not-additive"></a>

All textual data types [interpolate](#interpolation) as [discrete](https://www.w3.org/TR/web-animations-1/#discrete) and are [not additive](#not-additive).

### <a id="keywords"></a>4.1.  Pre-defined Keywords

<a id="ref-for-css-css-identifier①"></a>

<a id="ref-for-ascii-case-insensitive"></a>

In the value definition fields, <a id="css-keyword"></a>keywords with a pre-defined meaning appear literally. Keywords are [identifiers](#css-css-identifier) and are interpreted [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive) (i.e., \[a-z\] and \[A-Z\] are equivalent).

<a id="ref-for-propdef-border-collapse"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9d0ae504"></a> For example, here is the value definition for the [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) property:
>
> ```text
> Value: collapse | separate
> ```
>
> And here is an example of its use:
>
> ```text
> table { border-collapse: separate }
> ```
<a id="ref-for-valdef-all-initial"></a>

<a id="ref-for-valdef-all-inherit①"></a>

<a id="ref-for-valdef-all-unset"></a>

#### <a id="common-keywords"></a>4.1.1.  CSS-wide keywords: [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial), [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) and [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset)

As defined [above](#component-types), all properties accept the <a id="css-wide-keywords"></a>CSS-wide keywords, which represent value computations common to all CSS properties. These keywords are normatively defined in the [CSS Cascading and Inheritance Module](https://www.w3.org/TR/css-cascade/#defaulting-keywords).

Other CSS specifications can define additional CSS-wide keywords.

<a id="ref-for-identifier-value④"></a>

### <a id="custom-idents"></a>4.2.  Unprefixed Author-defined Identifiers: the [\<custom-ident\>](#identifier-value) type

<a id="ref-for-css-css-identifier②"></a>

<a id="ref-for-string-is"></a>

Some properties accept arbitrary author-defined identifiers as a component value. This generic data type is denoted by <a id="identifier-value"></a>\<custom-ident\>, and represents any valid CSS [identifier](#css-css-identifier) that would not be misinterpreted as a pre-defined keyword in that property’s value definition. Such identifiers are fully case-sensitive (meaning they’re compared using the "[identical to](https://infra.spec.whatwg.org/#string-is)" operation), even in the ASCII range (e.g. example and EXAMPLE are two different, unrelated user-defined identifiers).

<a id="ref-for-css-wide-keywords"></a>

<a id="ref-for-identifier-value⑤"></a>

<a id="ref-for-ascii-case-insensitive①"></a>

The [CSS-wide keywords](#css-wide-keywords) are not valid [\<custom-ident\>](#identifier-value)s. The default keyword is reserved and is also not a valid <a id="ref-for-identifier-value⑥"></a>\<custom-ident\>. Specifications using <a id="ref-for-identifier-value⑦"></a>\<custom-ident\> must specify clearly what other keywords are excluded from <a id="ref-for-identifier-value⑧"></a>\<custom-ident\>, if any—​for example by saying that any pre-defined keywords in that property’s value definition are excluded. Excluded keywords are excluded in all [ASCII case permutations](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-identifier-value⑨"></a>

When parsing positionally-ambiguous keywords in a property value, a [\<custom-ident\>](#identifier-value) production can only claim the keyword if no other unfulfilled production can claim it.

<a id="ref-for-propdef-animation"></a>

<a id="ref-for-propdef-animation-timing-function"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-in"></a>

<a id="ref-for-typedef-easing-function"></a>

<a id="ref-for-valdef-cubic-bezier-easing-function-ease-out"></a>

<a id="ref-for-identifier-value①⓪"></a>

<a id="ref-for-propdef-animation-name"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-950e3618"></a> For example, the shorthand declaration [animation: ease-in ease-out](https://www.w3.org/TR/css-animations-1/#propdef-animation) is equivalent to the longhand declarations [animation-timing-function: ease-in; animation-name: ease-out;](https://www.w3.org/TR/css-animations-1/#propdef-animation-timing-function). [ease-in](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-easing-function-ease-in) is claimed by the [\<easing-function\>](https://www.w3.org/TR/css-easing-1/#typedef-easing-function) production belonging to <a id="ref-for-propdef-animation-timing-function①"></a>animation-timing-function, leaving [ease-out](https://www.w3.org/TR/css-easing-1/#valdef-cubic-bezier-easing-function-ease-out) to be claimed by the [\<custom-ident\>](#identifier-value) production belonging to [animation-name](https://www.w3.org/TR/css-animations-1/#propdef-animation-name).

<a id="ref-for-identifier-value①①"></a>

<a id="ref-for-typedef-dashed-ident"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When designing grammars with [\<custom-ident\>](#identifier-value), the <a id="ref-for-identifier-value①②"></a>\<custom-ident\> should always be “positionally unambiguous”, so that it’s impossible to conflict with any keyword values in the property. Such conflicts can alternatively be avoided by using [\<dashed-ident\>](#typedef-dashed-ident).

<a id="ref-for-typedef-dashed-ident①"></a>

### <a id="dashed-idents"></a>4.3.  Prefixed Author-defined Identifiers: the [\<dashed-ident\>](#typedef-dashed-ident) type

<a id="ref-for-user-agent①"></a>

Some contexts accept <em>both</em> author-defined identifiers <em>and</em> CSS-defined identifiers. If not handled carefully, this can result in difficulties adding new CSS-defined values; [UAs](https://www.w3.org/TR/css-2023/#user-agent) have to study existing usage and gamble that there are sufficiently few author-defined identifiers in use matching the new CSS-defined one, so giving the new value a special CSS-defined meaning won’t break existing pages.

<a id="ref-for-typedef-dashed-ident②"></a>

While there are many legacy cases in CSS that mix these two values spaces in exactly this fraught way, the [\<dashed-ident\>](#typedef-dashed-ident) type is meant to be an easy way to distinguish author-defined identifiers from CSS-defined identifiers.

<a id="ref-for-typedef-dashed-ident③"></a>

<a id="ref-for-identifier-value①③"></a>

The <a id="typedef-dashed-ident"></a>[\<dashed-ident\>](#typedef-dashed-ident) production is a [\<custom-ident\>](#identifier-value), with all the case-sensitivity that implies, with the additional restriction that it must start with two dashes (U+002D HYPHEN-MINUS).

<a id="ref-for-typedef-dashed-ident④"></a>

[\<dashed-ident\>](#typedef-dashed-ident)s are reserved solely for use as author-defined names. CSS will never define a <a id="ref-for-typedef-dashed-ident⑤"></a>\<dashed-ident\> for its own use.

<a id="ref-for-custom-property"></a>

<a id="ref-for-typedef-dashed-ident⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a91b3e3f"></a> For example, [custom properties](https://www.w3.org/TR/css-variables-1/#custom-property) need to be distinguishable from CSS-defined properties, as new properties are added to CSS regularly. To allow this, <a id="ref-for-custom-property①"></a>custom property names are required to be [\<dashed-ident\>](#typedef-dashed-ident)s, as in this example:
>
> ```text
> .foo {
>   --fg-color: blue;
> }
> ```
<a id="ref-for-typedef-dashed-ident⑦"></a>

<a id="ref-for-at-ruledef-profile"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-340c7d34"></a> [\<dashed-ident\>](#typedef-dashed-ident)s are also used in the [@color-profile](https://www.w3.org/TR/css-color-5/#at-ruledef-profile) rule, to separate author-defined color profiles from pre-defined ones like device-cmyk, and allow CSS to define more pre-defined (but overridable) profiles in the future without fear of clashing with author-defined profiles:
>
> ```text
> @color-profile --foo { src: url(https://example.com/foo.icc); }
> .foo {
>   color: color(--foo 1 0 .5 / .2);
> }
> ```
<a id="ref-for-typedef-dashed-ident⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-32974307"></a> CSS will use [\<dashed-ident\>](#typedef-dashed-ident) more in the future, as more author-controlled syntax is added. CSS authoring tools, such as preprocessors that turn custom syntax into standard CSS, <em>should</em> use <a id="ref-for-typedef-dashed-ident⑨"></a>\<dashed-ident\> as well, to avoid clashing with future CSS design.
>
> For example, if a CSS preprocessor added a new "custom" at-rule, it <em>shouldn’t</em> spell it @custom, as this would clash with a future official @custom rule added by CSS. Instead, it should use @--custom, which is guaranteed to never clash with anything defined by CSS.
>
> Even better, it should use @--library1-custom, so that if Library2 adds their own "custom" at-rule (spelled @--library2-custom), there’s no possibility of clash. Ideally this prefix should be customizable, if allowed by the tooling, so authors can manually avoid clashes on their own.

<a id="ref-for-string-value①"></a>

### <a id="strings"></a>4.4.  Quoted Strings: the [\<string\>](#string-value) type

<a id="ref-for-string"></a>

<a id="ref-for-typedef-string-token"></a>

[Strings](https://infra.spec.whatwg.org/#string) are denoted by <a id="string-value"></a>\<string\>. When written literally, they consist of a sequence of characters delimited by double quotes or single quotes, corresponding to the [\<string-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-string-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9204e1e4"></a> Double quotes cannot occur inside double quotes, unless [escaped](https://www.w3.org/TR/CSS2/syndata.html#escaped-characters) (as `"\""` or as `"\22"`). Analogously for single quotes (`'\''` or `'\27'`).
>
> ```text
> content: "this is a 'string'.";
> content: "this is a \"string\".";
> content: 'this is a "string".';
> content: 'this is a \'string\'.'
> ```
It is possible to break strings over several lines, for aesthetic or other reasons, but in such a case the newline itself has to be escaped with a backslash (&#x5C;). The newline is subsequently removed from the string. For instance, the following two selectors are exactly the same:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4f7429ed"></a>
>
> Example(s):
>
> ```text
> a[title="a not s\
> o very long title"] {/*...*/}
> a[title="a not so very long title"] {/*...*/}
> ```
Since a string cannot directly represent a newline, to include a newline in a string, use the escape "&#x5C;A". (Hexadecimal A is the line feed character in Unicode (U+000A), but represents the generic notion of "newline" in CSS.)

<a id="ref-for-url-value①"></a>

### <a id="urls"></a>4.5.  Resource Locators: the [\<url\>](#url-value) type

<a id="ref-for-url-value②"></a>

<a id="ref-for-concept-url"></a>

The [\<url\>](#url-value) type, written with the <a id="funcdef-url"></a>url() and <a id="funcdef-src"></a>src() functions, represents a [URL](https://url.spec.whatwg.org/#concept-url), which is a pointer to a resource.

<a id="ref-for-url-value③"></a>

The syntax of [\<url\>](#url-value) is:

<a id="url-value"></a>

<a id="ref-for-funcdef-url"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-funcdef-src"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-url-token"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-typedef-url-modifier①"></a>

<a id="ref-for-mult-zero-plus①"></a>

```text
<url> = <url()> | <src()>

<url()> = url( <string> <url-modifier>* ) | <url-token>
<src()> = src( <string> <url-modifier>* )
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1dcaf1ec"></a> This example shows a URL being used as a background image:
>
> ```text
> body { background: url("http://www.example.com/pinkish.gif") }
> ```
<a id="ref-for-funcdef-url①"></a>

<a id="ref-for-consume-a-url-token"></a>

<a id="ref-for-typedef-url-token①"></a>

<a id="ref-for-funcdef-src①"></a>

<a id="ref-for-funcdef-var"></a>

For legacy reasons, a [url()](#funcdef-url) can be written without quotation marks around the URL itself, in which case it is [specially-parsed](https://www.w3.org/TR/css-syntax-3/#consume-a-url-token) as a [\<url-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-url-token) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). Because of this special parsing, <a id="ref-for-funcdef-url②"></a>url() is only able to specify its URL literally; [src()](#funcdef-src) lacks this special parsing rule, and so its URL can be provided by functions, such as [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a2ee15a6"></a> For example, the following declarations are identical:
>
> ```text
> background: url("http://www.example.com/pinkish.gif");
> background: url(http://www.example.com/pinkish.gif);
> ```
>
> And these have the same meaning as well:
>
> ```text
> background: src("http://www.example.com/pinkish.gif");
> --foo: "http://www.example.com/pinkish.gif";
> background: src(var(--foo));
> ```
>
> But this does <em>not</em> work:
>
> ```text
> --foo: "http://www.example.com/pinkish.gif";
> background: url(var(--foo));
> ```
>
> ...because the unescaped "(" in the value causes a parse error, so the entire declaration is thrown out as invalid.

<a id="ref-for-funcdef-url③"></a>

<a id="ref-for-typedef-url-modifier②"></a>

<a id="ref-for-string-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The unquoted [url()](#funcdef-url) syntax cannot accept a [\<url-modifier\>](#typedef-url-modifier) argument and has extra escaping requirements: parentheses, [whitespace](https://www.w3.org/TR/css-syntax/#whitespace) characters, single quotes (') and double quotes (") appearing in a URL must be escaped with a backslash, e.g. url(open&#x5C;(parens), url(close&#x5C;)parens). (In quoted [\<string\>](#string-value) <a id="ref-for-funcdef-url④"></a>url()s, only newlines and the character used to quote the string need to be escaped.) Depending on the type of URL, it might also be possible to write these characters as URL-escapes (e.g. url(open%28parens) or url(close%29parens)) as described in [\[URL\]](#biblio-url).
>
> <a id="ref-for-funcdef-url⑤"></a>
>
> The precise requirements for parsing the unquoted [url()](#funcdef-url) syntax are normatively defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-url-value④"></a>

<a id="ref-for-string-value⑤"></a>

<a id="ref-for-funcdef-url⑥"></a>

Some CSS contexts (such as [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import)) also allow a [\<url\>](#url-value) to be represented by a bare [\<string\>](#string-value), without the function wrapper. In such cases the string behaves identically to a [url()](#funcdef-url) function containing that string.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae720903"></a> For example, the following statements act identically:
>
> ```text
> @import url("base-theme.css");
> @import "base-theme.css";
> ```
#### <a id="relative-urls"></a>4.5.1.  Relative URLs

In order to create modular style sheets that are not dependent on the absolute location of a resource, authors should use relative URLs. Relative URLs (as defined in [\[URL\]](#biblio-url)) are resolved to full URLs using a base URL. RFC 3986, section 3, defines the normative algorithm for this process. For CSS style sheets, the base URL is that of the style sheet itself, not that of the styled source document. Style sheets embedded within a document have the base URL associated with their container.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For HTML documents, the [base URL is mutable](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#dynamic-changes-to-base-urls).

<a id="ref-for-url-value⑤"></a>

<a id="ref-for-user-agent②"></a>

When a [\<url\>](#url-value) appears in the computed value of a property, it is resolved to an absolute URL, as described in the preceding paragraph. The computed value of a URL that the [UA](https://www.w3.org/TR/css-2023/#user-agent) cannot resolve to an absolute URL is the specified value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b514611a"></a> For example, suppose the following rule:
>
> ```text
> body { background: url("tile.png") }
> ```
>
> is located in a style sheet designated by the URL:
>
> ```text
> http://www.example.org/style/basic.css
> ```
>
> The background of the source document’s `<body>` will be tiled with whatever image is described by the resource designated by the URL:
>
> ```text
> http://www.example.org/style/tile.png
> ```
>
> The same image will be used regardless of the URL of the source document containing the `<body>`.

##### <a id="local-urls"></a>4.5.1.1.  Fragment URLs

<a id="ref-for-url-value⑥"></a>

To enable element ID references to work in CSS regardless of base URL changes or shadow DOM, [\<url\>](#url-value)s have special behavior when they contain only a fragment.

<a id="ref-for-url-value⑦"></a>

<a id="ref-for-css-tree-scoped-reference"></a>

<a id="ref-for-concept-url-fragment"></a>

If a [\<url\>](#url-value)’s value starts with a U+0023 NUMBER SIGN (`#`) character, then the URL additionally has its <a id="url-local-url-flag"></a>local url flag set, and is a [tree-scoped reference](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-reference) for the URL’s [fragment](https://url.spec.whatwg.org/#concept-url-fragment).

<a id="ref-for-url-value⑧"></a>

<a id="ref-for-url-local-url-flag"></a>

When matching a [\<url\>](#url-value) with the [local url flag](#url-local-url-flag) set:

- <a id="ref-for-css-tree-scoped-reference①"></a>

  <a id="ref-for-css-tree-scoped-name"></a>

  <a id="ref-for-concept-tree-order"></a>

  <a id="ref-for-concept-node-tree"></a>

  <a id="ref-for-concept-url-fragment①"></a>

  if the URL’s fragment is an element ID reference (rather than, say, a media fragment), resolve it as a [tree-scoped reference](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-reference) with the tree’s IDs as the associated [tree-scoped names](https://drafts.csswg.org/css-scoping-1/#css-tree-scoped-name): specifically, resolve to the first element in [tree order](https://dom.spec.whatwg.org/#concept-tree-order) among the associated [node tree](https://dom.spec.whatwg.org/#concept-node-tree)'s descendants with the URL’s [fragment](https://url.spec.whatwg.org/#concept-url-fragment) as its ID. (And, as usual for <a id="ref-for-css-tree-scoped-reference②"></a>tree-scoped references, continuing up to the host’s tree if needed.)

  If no such element is found, the URL fails to resolve.

- otherwise, resolve the fragment against the current document.

<a id="ref-for-find-a-potential-indicated-element"></a>

<a id="ref-for-document"></a>

<a id="ref-for-shadowroot"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f83e4845"></a> Possibly reference [find a potential indicated element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#find-a-potential-indicated-element), but that is defined specifically for <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>s, not <code><a href="https://dom.spec.whatwg.org/#shadowroot">ShadowRoot</a></code>s.

<a id="ref-for-concept-node-tree①"></a>

<a id="ref-for-the-base-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that such fragments will resolve against the contents of the current document (or whichever [node tree](https://dom.spec.whatwg.org/#concept-node-tree) the stylesheet lives in, if shadow DOM is involved) regardless of how such relative URLs would resolve elsewhere (ignoring, for example, <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-base-element">base</a></code> elements changing the base URL, or relative URLs in linked stylesheets resolving against the stylesheet’s URL).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-08e9b99a"></a> In the following example, `#anchor` will resolve against `http://example.com/` whereas `#image` will resolve against the elements in the HTML document itself:
>
> ```text
> <!DOCTYPE html>
> <base href="http://example.com/">
> ...
> <a href="#anchor" style="background-image: url(#image)">link</a>
> ```
<a id="ref-for-funcdef-url⑦"></a>

<a id="ref-for-url-local-url-flag①"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) set, it must serialize as just the fragment.

#### <a id="url-empty"></a>4.5.2.  Empty URLs

<a id="ref-for-url-value⑨"></a>

<a id="ref-for-funcdef-url⑧"></a>

If the value of the [\<url\>](#url-value) is the empty string (like url("") or [url()](#funcdef-url)), the url must resolve to an invalid resource (similar to what the url about:invalid does).

Its computed value is url("") or src(""), whichever was specified, and it must serialize as such.

<a id="ref-for-funcdef-url⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This matches the behavior of empty urls for embedded resources elsewhere in the web platform, and avoids excess traffic re-requesting the stylesheet or host document due to editing mistakes leaving the [url()](#funcdef-url) value empty, which are almost certain to be invalid resources for whatever the <a id="ref-for-funcdef-url①⓪"></a>url() shows up in. Linking on the web platform <em>does</em> allow empty urls, so if/when CSS gains some functionality to control hyperlinks, this restriction can be relaxed in those contexts.

#### <a id="url-modifiers"></a>4.5.3.  URL Modifiers

<a id="ref-for-url-value①⓪"></a>

<a id="ref-for-typedef-url-modifier③"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="ref-for-functional-notation④"></a>

[\<url\>](#url-value)s support specifying additional <a id="typedef-url-modifier"></a>\<url-modifier\>s, which change the meaning or the interpretation of the URL somehow. A [\<url-modifier\>](#typedef-url-modifier) is either an [\<ident\>](#typedef-ident) or a [functional notation](#functional-notation).

<a id="ref-for-typedef-url-modifier④"></a>

This specification does not define any [\<url-modifier\>](#typedef-url-modifier)s, but other specs may do so.

<a id="ref-for-url-value①①"></a>

<a id="ref-for-funcdef-url①①"></a>

<a id="ref-for-typedef-url-modifier⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [\<url\>](#url-value) that is either unquoted or not wrapped in [url()](#funcdef-url) notation cannot accept any [\<url-modifier\>](#typedef-url-modifier)s.

#### <a id="url-processing"></a>4.5.4.  URL Processing Model

<a id="ref-for-concept-url①"></a>

<a id="ref-for-url-value①②"></a>

<a id="ref-for-cssstylesheet"></a>

<a id="ref-for-requestdestination"></a>

<a id="ref-for-concept-response"></a>

To <a id="fetch-a-style-resource"></a>fetch a style resource from a [url](https://url.spec.whatwg.org/#concept-url) or [\<url\>](#url-value) <var>urlValue</var>, given a <code><a href="https://www.w3.org/TR/cssom-1/#cssstylesheet">CSSStyleSheet</a></code> <var>sheet</var>, a string <var>destination</var> matching a <code><a href="https://fetch.spec.whatwg.org/#requestdestination">RequestDestination</a></code>, a "no-cors" or "cors" <var>corsMode</var>, and an algorithm <var>processResponse</var> accepting a [response](https://fetch.spec.whatwg.org/#concept-response) and a null, failure or byte stream:

1.  <a id="ref-for-relevant-settings-object"></a>

    Let <var>environmentSettings</var> be <var>sheet</var>’s [relevant settings object](https://html.spec.whatwg.org/multipage/webappapis.html#relevant-settings-object).

2.  <a id="ref-for-concept-css-style-sheet-stylesheet-base-url"></a>

    <a id="ref-for-api-base-url"></a>

    Let <var>base</var> be <var>sheet</var>’s [stylesheet base URL](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-stylesheet-base-url) if it is not null, otherwise <var>environmentSettings</var>’s [API base URL](https://html.spec.whatwg.org/multipage/webappapis.html#api-base-url). [\[CSSOM\]](#biblio-cssom)

3.  <a id="ref-for-concept-url-parser"></a>

    <a id="ref-for-concept-url②"></a>

    Let <var>parsedUrl</var> be the result of the [URL parser](https://url.spec.whatwg.org/#concept-url-parser) steps with <var>urlValue</var>’s [url](https://url.spec.whatwg.org/#concept-url) and <var>base</var>. If the algorithm returns an error, return.

4.  <a id="ref-for-concept-request"></a>

    <a id="ref-for-concept-request-url"></a>

    <a id="ref-for-concept-request-destination"></a>

    <a id="ref-for-concept-request-mode"></a>

    <a id="ref-for-concept-request-origin"></a>

    <a id="ref-for-concept-settings-object-origin"></a>

    <a id="ref-for-concept-request-credentials-mode"></a>

    <a id="ref-for-concept-request-use-url-credentials-flag"></a>

    <a id="ref-for-concept-request-client"></a>

    <a id="ref-for-concept-request-referrer"></a>

    <a id="ref-for-api-base-url①"></a>

    Let <var>req</var> be a new [request](https://fetch.spec.whatwg.org/#concept-request) whose [url](https://fetch.spec.whatwg.org/#concept-request-url) is <var>parsedUrl</var>, whose [destination](https://fetch.spec.whatwg.org/#concept-request-destination) is <var>destination</var>, [mode](https://fetch.spec.whatwg.org/#concept-request-mode) is <var>corsMode</var>, [origin](https://fetch.spec.whatwg.org/#concept-request-origin) is <var>environmentSettings</var>’s [origin](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-origin), [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) is "same-origin", [use-url-credentials flag](https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag) is set, [client](https://fetch.spec.whatwg.org/#concept-request-client) is <var>environmentSettings</var>, and whose [referrer](https://fetch.spec.whatwg.org/#concept-request-referrer) is <var>environmentSettings</var>’s [API base URL](https://html.spec.whatwg.org/multipage/webappapis.html#api-base-url).

5.  Apply any <a id="url-request-modifier-steps"></a>URL request modifier steps that apply to this request.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This specification does not define any URL request modification steps, but other specs may do so.

6.  <a id="ref-for-concept-request-mode①"></a>

    <a id="ref-for-concept-request-referrer①"></a>

    <a id="ref-for-concept-css-style-sheet-location"></a>

    If <var>req</var>’s [mode](https://fetch.spec.whatwg.org/#concept-request-mode) is "cors", set <var>req</var>’s [referrer](https://fetch.spec.whatwg.org/#concept-request-referrer) to <var>sheet</var>’s [location](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-location). [\[CSSOM\]](#biblio-cssom)

7.  <a id="ref-for-concept-css-style-sheet-origin-clean-flag"></a>

    <a id="ref-for-request-initiator-type"></a>

    If <var>sheet</var>’s [origin-clean flag](https://www.w3.org/TR/cssom-1/#concept-css-style-sheet-origin-clean-flag) is set, set <var>req</var>’s [initiator type](https://fetch.spec.whatwg.org/#request-initiator-type) to "css". [\[CSSOM\]](#biblio-cssom)

8.  <a id="ref-for-concept-fetch"></a>

    <a id="ref-for-process-response-end-of-body"></a>

    [Fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>req</var>, with [processresponseconsumebody](https://fetch.spec.whatwg.org/#process-response-end-of-body) set to <var>processResponse</var>.

<a id="ref-for-concept-url③"></a>

<a id="ref-for-concept-url-parser①"></a>

When interpreting [URLs](https://url.spec.whatwg.org/#concept-url) expressed in CSS, the [URL parser’s](https://url.spec.whatwg.org/#concept-url-parser) <var>encoding</var> argument must be omitted (i.e. use the default, UTF-8), regardless of the stylesheet encoding.

<a id="ref-for-string-percent-encode-after-encoding"></a>

<a id="ref-for-concept-url④"></a>

<a id="ref-for-code-point"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In other words, a URL written in CSS will always [percent-encode](https://url.spec.whatwg.org/#string-percent-encode-after-encoding) non-ASCII codepoints using UTF-8 in the [URL](https://url.spec.whatwg.org/#concept-url) object (and thus whenever using the <a id="ref-for-concept-url⑤"></a>URL value for e.g. network requests), regardless of the stylesheet’s own encoding. Note that this occurs [after decoding the stylesheet](https://www.w3.org/TR/css-syntax-3/#input-byte-stream) into Unicode [code points](https://infra.spec.whatwg.org/#code-point).

## <a id="numeric-types"></a>5.  Numeric Data Types

<a id="ref-for-specified-value①"></a>

<a id="ref-for-computed-value④"></a>

Numeric data types are used to represent quantities, indexes, positions, and other such values. Although many syntactic variations can exist in expressing the quantity (numeric aspect) in a given numeric value, the [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) do not distinguish these variations: they represent the value’s abstract quantity, not its syntactic representation.

<a id="ref-for-integer-value③"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-dimension"></a>

<a id="ref-for-length-value⑤"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-time-value"></a>

<a id="ref-for-frequency-value"></a>

<a id="ref-for-resolution-value"></a>

The <a id="numeric-data-types"></a>numeric data types include [\<integer\>](#integer-value), [\<number\>](#number-value), [\<percentage\>](#percentage-value), and various [dimensions](#dimension) including [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<frequency\>](#frequency-value), and [\<resolution\>](#resolution-value).

<a id="ref-for-dimension①"></a>

<a id="ref-for-valdef-flex-fr"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While general-purpose [dimensions](#dimension) are defined here, some other modules define additional data types (e.g. [\[css-grid-1\]](#biblio-css-grid-1) introduces [fr](https://www.w3.org/TR/css-grid-2/#valdef-flex-fr) units) whose usage is more localized.

<a id="ref-for-implementation-defined"></a>

The precision and supported range of numeric values in CSS is [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined), and can vary based on the property or other context a value is used in. However, within the CSS specifications, infinite precision and range is assumed. When a value cannot be explicitly supported due to range/precision limitations, it must be converted to the closest value supported by the implementation, but how the implementation defines "closest" is <a id="ref-for-implementation-defined①"></a>implementation-defined as well.

<a id="ref-for-angle-value①"></a>

If an [\<angle\>](#angle-value) must be converted due to exceeding the implementation-defined range of supported values, it must be clamped to the nearest supported multiple of 360deg.

### <a id="numeric-ranges"></a>5.1.  Range Restrictions and Range Definition Notation

<a id="ref-for-integer-value④"></a>

<a id="ref-for-angle-value②"></a>

Properties can restrict numeric values to some range. If the value is outside the allowed range, then unless otherwise specified, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore). Range restrictions can be annotated in the numeric type notation using <a id="css-bracketed-range-notation"></a>CSS bracketed range notation—​<code><c->&#x5B;</c-><var>min</var><c->,</c-><var>max</var><c->&#x5D;</c-></code>—​within the angle brackets, after the identifying keyword, indicating a closed range between (and including) <var>min</var> and <var>max</var>. For example, [\<integer \[0,10\]\>](#integer-value) indicates an integer between 0 and 10, inclusive, while [\<angle \[0,180deg\]\>](#angle-value) indicates an angle between 0deg and 180deg (expressed in any unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS values generally do not allow open ranges; thus only square-bracket notation is used.

<a id="ref-for-user-agent③"></a>

<a id="ref-for-length-value⑥"></a>

CSS theoretically supports infinite precision and infinite ranges for all value types; however in reality implementations have finite capacity. [UAs](https://www.w3.org/TR/css-2023/#user-agent) should support reasonably useful ranges and precisions. Range extremes that are ideally unlimited are indicated using ∞ or −∞ as appropriate. For example, [\<length \[0,∞\]\>](#length-value) indicates a non-negative length.

<a id="ref-for-css-bracketed-range-notation"></a>

If no range is indicated, either by using the [bracketed range notation](#css-bracketed-range-notation) or in the property description, then `[−∞,∞]` is assumed.

<a id="ref-for-time-value①"></a>

Values of −∞ or ∞ must be written without units, even if the value type uses units. Values of 0 <em>can</em> be written without units, even if the value type doesn’t allow “unitless zeroes” (such as [\<time\>](#time-value)).

<a id="ref-for-css-bracketed-range-notation①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At the time of writing, the [bracketed range notation](#css-bracketed-range-notation) is new; thus in most CSS specifications any range limitations are described only in prose. (For example, “Negative values are not allowed” or “Negative values are invalid” indicate a `[0,∞]` range.) This does not make them any less binding.

<a id="ref-for-integer-value⑤"></a>

### <a id="integers"></a>5.2.  Integers: the [\<integer\>](#integer-value) type

Integer values are denoted by <a id="integer-value"></a>\<integer\>.

<a id="ref-for-typedef-number-token"></a>

When written literally, an <a id="integer"></a>integer is one or more decimal digits 0 through 9 and corresponds to a subset of the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the CSS Syntax Module [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). The first digit of an integer may be immediately preceded by - or + to indicate the integer’s sign.

Unless otherwise specified, in the CSS specifications <a id="css-round-to-the-nearest-integer"></a>rounding to the nearest integer requires rounding in the direction of +∞ when the fractional portion is exactly 0.5. (For example, 1.5 rounds to 2, while -1.5 rounds to -1.)

<a id="ref-for-integer-value⑥"></a>

#### <a id="combine-integers"></a>5.2.1.  Computation and Combination of [\<integer\>](#integer-value)

<a id="ref-for-computed-value⑤"></a>

<a id="ref-for-integer-value⑦"></a>

Unless otherwise specified, the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a specified [\<integer\>](#integer-value) is the specified abstract integer.

<a id="ref-for-interpolation②"></a>

<a id="ref-for-integer-value⑧"></a>

<a id="ref-for-number-value①"></a>

<a id="ref-for-css-round-to-the-nearest-integer"></a>

[Interpolation](#interpolation) of [\<integer\>](#integer-value) is defined as <var>V</var><sub>result</sub> = round((1 - <var>p</var>) × <var>V<sub>A</sub></var> + <var>p</var> × <var>V<sub>B</sub></var>); that is, interpolation happens in the real number space as for [\<number\>](#number-value)s, and the result is converted to an <a id="ref-for-integer-value⑨"></a>\<integer\> by [rounding to the nearest integer](#css-round-to-the-nearest-integer).

<a id="ref-for-addition⑦"></a>

<a id="ref-for-integer-value①⓪"></a>

[Addition](#addition) of [\<integer\>](#integer-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>A</sub></var> + <var>V<sub>B</sub></var>

<a id="ref-for-number-value②"></a>

### <a id="numbers"></a>5.3.  Real Numbers: the [\<number\>](#number-value) type

Number values are denoted by <a id="number-value"></a>\<number\>, and represent real numbers, possibly with a fractional component.

<a id="ref-for-integer"></a>

<a id="ref-for-typedef-number-token①"></a>

When written literally, a <a id="number"></a>number is either an [integer](#integer), or zero or more decimal digits followed by a dot (.) followed by one or more decimal digits; optionally, it can be concluded by the letter “e” or “E” followed by an integer indicating the base-ten exponent in [scientific notation](https://en.wikipedia.org/wiki/Scientific_notation). It corresponds to the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). As with integers, the first character of a number may be immediately preceded by - or + to indicate the number’s sign.

<a id="ref-for-number"></a>

<a id="ref-for-number-value③"></a>

<a id="ref-for-zero-value"></a>

<a id="ref-for-typedef-number-token②"></a>

The value <a id="zero-value"></a>\<zero\> represents a literal [number](#number) with the value 0. Expressions that merely evaluate to a [\<number\>](#number-value) with the value 0 (for example, calc(0)) do not match [\<zero\>](#zero-value); only literal [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s do.

<a id="ref-for-number-value④"></a>

#### <a id="combine-numbers"></a>5.3.1.  Computation and Combination of [\<number\>](#number-value)

<a id="ref-for-computed-value⑥"></a>

<a id="ref-for-number-value⑤"></a>

Unless otherwise specified, the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a specified [\<number\>](#number-value) is the specified abstract number.

<a id="ref-for-interpolation③"></a>

<a id="ref-for-number-value⑥"></a>

[Interpolation](#interpolation) of [\<number\>](#number-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>A</sub></var> + <var>p</var> × <var>V<sub>B</sub></var>

<a id="ref-for-addition⑧"></a>

<a id="ref-for-number-value⑦"></a>

[Addition](#addition) of [\<number\>](#number-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>A</sub></var> + <var>V<sub>B</sub></var>

<a id="ref-for-dimension②"></a>

### <a id="dimensions"></a>5.4.  Numbers with Units: [dimension](#dimension) values

The general term <a id="dimension"></a>dimension refers to a number with a unit attached to it; and is denoted by <a id="typedef-dimension"></a>\<dimension\>.

<a id="ref-for-dimension③"></a>

<a id="ref-for-number①"></a>

<a id="ref-for-css-css-identifier③"></a>

<a id="ref-for-typedef-dimension-token①"></a>

<a id="ref-for-ascii-case-insensitive②"></a>

When written literally, a [dimension](#dimension) is a [number](#number) immediately followed by a unit identifier, which is an [identifier](#css-css-identifier). It corresponds to the [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). Like keywords, unit identifiers are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-length-value⑦"></a>

<a id="ref-for-time-value②"></a>

<a id="ref-for-frequency-value①"></a>

<a id="ref-for-resolution-value①"></a>

CSS uses [\<dimension\>](#typedef-dimension)s to specify distances ([\<length\>](#length-value)), durations ([\<time\>](#time-value)), frequencies ([\<frequency\>](#frequency-value)), resolutions ([\<resolution\>](#resolution-value)), and other quantities.

#### <a id="compat"></a>5.4.1.  Compatible Units

<a id="ref-for-computed-value⑦"></a>

<a id="ref-for-px"></a>

<a id="ref-for-in"></a>

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-em"></a>

<a id="ref-for-canonical-unit①"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) [\[CSSOM\]](#biblio-cssom), <a id="compatible-units"></a>compatible units (those related by a static multiplicative factor, like the 96:1 factor between [px](#px) and [in](#in), or the computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) factor between [em](#em) and <a id="ref-for-px①"></a>px) are converted into a single <a id="canonical-unit"></a>canonical unit. Each group of compatible units defines which among them is the [canonical unit](#canonical-unit) that will be used for serialization.

<a id="ref-for-used-value"></a>

<a id="ref-for-compatible-units"></a>

<a id="ref-for-dimension④"></a>

When serializing [resolved values](https://www.w3.org/TR/cssom-1/#resolved-values) that are [used values](https://www.w3.org/TR/css-cascade-5/#used-value), all value types (percentages, numbers, keywords, etc.) that represent lengths are considered [compatible](#compatible-units) with lengths. Likewise any future API that returns <a id="ref-for-used-value①"></a>used values must consider any values that represent distances/durations/frequencies/etc. as <a id="ref-for-compatible-units①"></a>compatible with the relevant class of [dimensions](#dimension), and canonicalize accordingly.

#### <a id="combine-dimensions"></a>5.4.2.  Combination of Dimensions

<a id="ref-for-interpolation④"></a>

<a id="ref-for-compatible-units②"></a>

<a id="ref-for-dimension⑤"></a>

<a id="ref-for-length-value⑧"></a>

[Interpolation](#interpolation) of [compatible](#compatible-units) [dimensions](#dimension) (for example, two [\<length\>](#length-value) values) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>A</sub></var> + <var>p</var> × <var>V<sub>B</sub></var>

<a id="ref-for-addition⑨"></a>

<a id="ref-for-compatible-units③"></a>

<a id="ref-for-dimension⑥"></a>

[Addition](#addition) of [compatible](#compatible-units) [dimensions](#dimension) is defined as <var>V<sub>result</sub></var> = <var>V<sub>A</sub></var> + <var>V<sub>B</sub></var>

<a id="ref-for-percentage-value②"></a>

### <a id="percentages"></a>5.5.  Percentages: the [\<percentage\>](#percentage-value) type

Percentage values are denoted by <a id="percentage-value"></a>\<percentage\>, and indicates a value that is some fraction of another reference value.

<a id="ref-for-number②"></a>

<a id="ref-for-typedef-percentage-token"></a>

When written literally, a <a id="percentage"></a>percentage consists of a [number](#number) immediately followed by a percent sign %. It corresponds to the [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-containing-block"></a>

Percentage values are always relative to another quantity, for example a length. Each property that allows percentages also defines the quantity to which the percentage refers. This quantity can be a value of another property for the same element, the value of a property for an ancestor element, a measurement of the formatting context (e.g., the width of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block)), or something else.

<a id="ref-for-percentage-value③"></a>

#### <a id="combine-percentages"></a>5.5.1.  Computation and Combination of [\<percentage\>](#percentage-value)

<a id="ref-for-propdef-font-size①"></a>

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-length-value⑨"></a>

<a id="ref-for-computed-value⑧"></a>

Unless otherwise specified (such as in [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), which computes its [\<percentage\>](#percentage-value) values to [\<length\>](#length-value)), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a percentage is the specified percentage.

<a id="ref-for-interpolation⑤"></a>

<a id="ref-for-percentage-value⑤"></a>

[Interpolation](#interpolation) of [\<percentage\>](#percentage-value) is defined as <var>V</var><sub>result</sub> = (1 - <var>p</var>) × <var>V<sub>A</sub></var> + <var>p</var> × <var>V<sub>B</sub></var>

<a id="ref-for-addition①⓪"></a>

<a id="ref-for-percentage-value⑥"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined as <var>V<sub>result</sub></var> = <var>V<sub>A</sub></var> + <var>V<sub>B</sub></var>

### <a id="mixed-percentages"></a>5.6.  Mixing Percentages and Dimensions

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-dimension⑦"></a>

<a id="ref-for-component-value①"></a>

<a id="ref-for-funcdef-calc②"></a>

In cases where a [\<percentage\>](#percentage-value) can represent the same quantity as a [dimension](#dimension) in the same [component value](https://www.w3.org/TR/css-syntax-3/#component-value) position, and can therefore be combined with them in a [calc()](#funcdef-calc) expression, the following convenience notations may be used in the property grammar:

<a id="typedef-length-percentage"></a>\<length-percentage\>  
<a id="ref-for-length-value①⓪"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-length-value①①"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#length-value">&lt;length&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<length\>](#length-value).

<a id="typedef-frequency-percentage"></a>\<frequency-percentage\>  
<a id="ref-for-frequency-value②"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-frequency-value③"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#frequency-value">&lt;frequency&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<frequency\>](#frequency-value).

<a id="typedef-angle-percentage"></a>\<angle-percentage\>  
<a id="ref-for-angle-value③"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-angle-value④"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#angle-value">&lt;angle&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to an [\<angle\>](#angle-value).

<a id="typedef-time-percentage"></a>\<time-percentage\>  
<a id="ref-for-time-value③"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-time-value④"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#time-value">&lt;time&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<time\>](#time-value).

<a id="ref-for-propdef-width"></a>

<a id="ref-for-length-value①②"></a>

<a id="ref-for-percentage-value①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-10c2940e"></a> For example, the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property can accept a [\<length\>](#length-value) or a [\<percentage\>](#percentage-value), both representing a measure of distance. This means that <a id="ref-for-propdef-width①"></a>width: calc(500px + 50%); is allowed—​both values are converted to absolute lengths and added. If the containing block is 1000px wide, then <a id="ref-for-propdef-width②"></a>width: 50%; is equivalent to <a id="ref-for-propdef-width③"></a>width: 500px, and <a id="ref-for-propdef-width④"></a>width: calc(50% + 500px) thus ends up equivalent to <a id="ref-for-propdef-width⑤"></a>width: calc(500px + 500px) or <a id="ref-for-propdef-width⑥"></a>width: 1000px.
>
> <a id="ref-for-funcdef-hsl"></a>
>
> <a id="ref-for-percentage-value①⑦"></a>
>
> <a id="ref-for-funcdef-calc③"></a>
>
> On the other hand, the second and third arguments of the [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) function can only be expressed as [\<percentage\>](#percentage-value)s. Although [calc()](#funcdef-calc) productions are allowed in their place, they can only combine percentages with themselves, as in calc(10% + 20%).

<a id="ref-for-percentage-value①⑧"></a>

<a id="ref-for-compatible-units④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifications should never alternate [\<percentage\>](#percentage-value) in place of a dimension in a grammar unless they are [compatible](#compatible-units).

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-percentage-value①⑨"></a>

<a id="ref-for-funcdef-calc④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: More \<<var>type</var>-percentage\> productions can be added in the future as needed. A \<number-percentage\> will never be added, as [\<number\>](#number-value) and [\<percentage\>](#percentage-value) can’t be combined in [calc()](#funcdef-calc).

#### <a id="combine-mixed"></a>5.6.1.  Computation and Combination of Percentage and Dimension Mixes

<a id="ref-for-computed-value⑨"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a percentage-dimension mix is defined as

- a computed dimension if the percentage component is zero or is defined specifically to compute to a dimension value

- a computed percentage if the dimension component is zero

- a [computed calc() expression](#calc-computed-value) otherwise

<a id="ref-for-interpolation⑥"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-typedef-frequency-percentage"></a>

<a id="ref-for-typedef-angle-percentage"></a>

<a id="ref-for-typedef-time-percentage"></a>

[Interpolation](#interpolation) of percentage-dimension value combinations (e.g. [\<length-percentage\>](#typedef-length-percentage), [\<frequency-percentage\>](#typedef-frequency-percentage), [\<angle-percentage\>](#typedef-angle-percentage), [\<time-percentage\>](#typedef-time-percentage) or equivalent notations) is defined as

- <a id="ref-for-length-value①③"></a>

  <a id="ref-for-interpolation⑦"></a>

  equivalent to [interpolation](#interpolation) of [\<length\>](#length-value) if both <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var> are pure <a id="ref-for-length-value①④"></a>\<length\> values

- <a id="ref-for-percentage-value②⓪"></a>

  <a id="ref-for-interpolation⑧"></a>

  equivalent to [interpolation](#interpolation) of [\<percentage\>](#percentage-value) if both <var>V<sub>A</sub></var> and <var>V<sub>B</sub></var> are pure <a id="ref-for-percentage-value②①"></a>\<percentage\> values

- <a id="ref-for-percentage-value②②"></a>

  <a id="ref-for-time-value⑤"></a>

  <a id="ref-for-angle-value⑤"></a>

  <a id="ref-for-frequency-value④"></a>

  <a id="ref-for-length-value①⑤"></a>

  <a id="ref-for-interpolation⑨"></a>

  <a id="ref-for-funcdef-calc⑤"></a>

  equivalent to converting both values into a [calc()](#funcdef-calc) expression representing the sum of the dimension type and a percentage (each possibly zero) and [interpolating](#interpolation) each component individually (as a [\<length\>](#length-value)/[\<frequency\>](#frequency-value)/[\<angle\>](#angle-value)/[\<time\>](#time-value) and as a [\<percentage\>](#percentage-value), respectively)

<a id="ref-for-addition①①"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-interpolation①⓪"></a>

[Addition](#addition) of [\<percentage\>](#percentage-value) is defined the same as [interpolation](#interpolation) except by <a id="ref-for-addition①②"></a>adding each component rather than <a id="ref-for-interpolation①①"></a>interpolating it.

<a id="ref-for-ratio-value"></a>

### <a id="ratios"></a>5.7.  Ratios: the [\<ratio\>](#ratio-value) type

Ratio values are denoted by <a id="ratio-value"></a>\<ratio\>, and represent the ratio of two numeric values. It most often represents an aspect ratio, relating a width (first) to a height (second).

When written literally, a <a id="ratio"></a>ratio has the syntax:

<a id="ref-for-ratio-value①"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-number-value①⓪"></a>

<a id="ref-for-mult-opt③"></a>

```text
<ratio> = <number [0,∞]> [ / <number [0,∞]> ]?
```
<a id="ref-for-number-value①①"></a>

<a id="ref-for-ratio-value②"></a>

The second [\<number\>](#number-value) is optional, defaulting to 1. However, [\<ratio\>](#ratio-value) is always serialized with both components.

<a id="ref-for-ratio-value③"></a>

The computed value of a [\<ratio\>](#ratio-value) is the pair of numbers provided.

<a id="ref-for-ratio-value④"></a>

If either number in the [\<ratio\>](#ratio-value) is 0 or infinite, it represents a <a id="degenerate-ratio"></a>degenerate ratio (and, generally, won’t do anything).

<a id="ref-for-ratio-value⑤"></a>

If two [\<ratio\>](#ratio-value)s need to be compared, divide the first number by the second, and compare the results. For example, 3/2 is less than 2/1, because it resolves to 1.5 while the second resolves to 2. (In other words, “tall” aspect ratios are less than “wide” aspect ratios.)

<a id="ref-for-ratio-value⑥"></a>

#### <a id="combine-ratio"></a>5.7.1.  Combination of [\<ratio\>](#ratio-value)

<a id="ref-for-ratio-value⑦"></a>

The interpolation of a [\<ratio\>](#ratio-value) is defined by converting each <a id="ref-for-ratio-value⑧"></a>\<ratio\> to a number by dividing the first value by the second (so a ratio of 3 / 2 would become 1.5), taking the logarithm of that result (so the 1.5 would become approximately 0.176), then interpolating those values. The result during the interpolation is converted back to a <a id="ref-for-ratio-value⑨"></a>\<ratio\> by inverting the logarithm, then interpreting the result as a <a id="ref-for-ratio-value①⓪"></a>\<ratio\> with the result as the first value and 1 as the second value.

<a id="ref-for-ratio-value①①"></a>

<a id="ref-for-degenerate-ratio"></a>

If either [\<ratio\>](#ratio-value) is [degenerate](#degenerate-ratio), the values cannot be interpolated.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4a376f3"></a> For example, halfway through a linear interpolation from 5 / 1 to 3 / 2, the result is approximately the ratio 2.73 / 1 (roughly 11 / 4, slightly taller than a 3 / 1 ratio):
>
> ```text
> start  = log(5);   // ≈ 0.69897
> end    = log(1.5); // ≈ 0.17609
> interp = 0.69897*.5 + 0.17609*.5; // ≈ 0.43753
> final  = 10^interp; // ≈ 2.73
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Interpolating over the logarithm of the ratio means the results are scale-independent (5 / 1 to 300 / 200 would give the same results as above), that they’re symmetrical over "wide" and "tall" variants (interpolating from 1 / 5 to 2 / 3 would give a ratio approximately equal to 1 / 2.73 at the halfway point), and that they’re symmetrical over whether the width is fixed and the height is based on the ratio or vice versa. These properties are not shared by many other possible interpolation strategies.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Due to the properties of logarithms, any log can be used; the example here uses base-10 log, but if, say, the natural log and e was used, the intermediate results would be different but the final result would be the same.

<a id="ref-for-ratio-value①②"></a>

Addition of [\<ratio\>](#ratio-value)s is not possible.

<a id="ref-for-length-value①⑥"></a>

## <a id="lengths"></a>6.  Distance Units: the [\<length\>](#length-value) type

<a id="ref-for-dimension⑧"></a>

Lengths refer to distance measurements and are denoted by <a id="length-value"></a>\<length\> in the property definitions. A length is a [dimension](#dimension).

<a id="ref-for-number-value①②"></a>

<a id="ref-for-length-value①⑦"></a>

<a id="ref-for-propdef-line-height"></a>

For zero lengths the unit identifier is optional (i.e. can be syntactically represented as the [\<number\>](#number-value) 0). However, if a 0 could be parsed as either a <a id="ref-for-number-value①③"></a>\<number\> or a [\<length\>](#length-value) in a property (such as [line-height](https://drafts.csswg.org/css2/#propdef-line-height)), it must parse as a <a id="ref-for-number-value①④"></a>\<number\>.

Properties may restrict the length value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore).

While some properties allow negative length values, this may complicate the formatting and there may be implementation-specific limits. If a negative length value is allowed but cannot be supported, it must be converted to the nearest value that can be supported.

<a id="ref-for-used-value②"></a>

<a id="ref-for-actual-value"></a>

In cases where the [used](https://www.w3.org/TR/css-cascade-5/#used-value) length cannot be supported, user agents must approximate it in the [actual](https://www.w3.org/TR/css-cascade-5/#actual-value) value.

<a id="ref-for-relative-length"></a>

<a id="ref-for-absolute-length"></a>

<a id="ref-for-specified-value②"></a>

<a id="ref-for-computed-value①⓪"></a>

<a id="ref-for-canonical-unit②"></a>

<a id="ref-for-px②"></a>

There are two types of length units: [relative](#relative-length) and [absolute](#absolute-length). The [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) of a length (<a id="specified-length"></a>specified length) is represented by its quantity and its unit. The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a length (<a id="computed-length"></a>computed length) is the specified length resolved to an <a id="ref-for-absolute-length①"></a>absolute length, and its unit is not distinguished: it can be represented by any <a id="ref-for-absolute-length②"></a>absolute length unit (but will be serialized using its [canonical unit](#canonical-unit), [px](#px)).

<a id="ref-for-implementation-defined②"></a>

<a id="ref-for-length-value①⑧"></a>

<a id="ref-for-propdef-border-width④"></a>

While the exact supported precision of numeric values, and how they are rounded to match that precision, is generally [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined), [\<length\>](#length-value)s in [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width) and a few other properties are rounded in a specific fashion to ensure reasonable visual display. (This algorithm is called by individual properties explicitly.)

<a id="ref-for-length-value①⑨"></a>

To <a id="snap-a-length-as-a-border-width"></a>snap a length as a border width given a [\<length\>](#length-value) <var>len</var>:

1.  Assert: <var>len</var> is non-negative.

2.  <a id="ref-for-device-pixel"></a>

    If <var>len</var> is an integer number of [device pixels](#device-pixel), do nothing.

3.  <a id="ref-for-device-pixel①"></a>

    If <var>len</var> is greater than zero, but less than 1 [device pixel](#device-pixel), round <var>len</var> up to 1 <a id="ref-for-device-pixel②"></a>device pixel.

4.  <a id="ref-for-device-pixel③"></a>

    If <var>len</var> is greater than 1 [device pixel](#device-pixel), round it down to the nearest integer number of <a id="ref-for-device-pixel④"></a>device pixels.

### <a id="relative-lengths"></a>6.1.  Relative Lengths

<a id="relative-length"></a>Relative length units specify a length relative to another length. Style sheets that use relative units can more easily scale from one output environment to another.

The relative units are:

<strong>Table 3 — structured row/cell transcription</strong>

Informative Summary of Relative Units

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

unit

<strong>Column 2 (header cell):</strong>

relative to

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-em①"></a>

[em](#em)

<strong>Column 2 (data cell):</strong>

font size of the element

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-ex"></a>

[ex](#ex)

<strong>Column 2 (data cell):</strong>

x-height of the element’s font

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-cap"></a>

[cap](#cap)

<strong>Column 2 (data cell):</strong>

cap height (the nominal height of capital letters) of the element’s font

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-ch"></a>

[ch](#ch)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-advance-measure"></a>

typical [character advance](#length-advance-measure) of a narrow glyph in the element’s font, as represented by the “0” (ZERO, U+0030) glyph

<strong>Row 6</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-ic"></a>

[ic](#ic)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-advance-measure①"></a>

typical [character advance](#length-advance-measure) of a fullwidth glyph in the element’s font, as represented by the “水” (CJK water ideograph, U+6C34) glyph

<strong>Row 7</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-rem"></a>

[rem](#rem)

<strong>Column 2 (data cell):</strong>

font size of the root element

<strong>Row 8</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-lh"></a>

[lh](#lh)

<strong>Column 2 (data cell):</strong>

line height of the element

<strong>Row 9</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-rlh"></a>

[rlh](#rlh)

<strong>Column 2 (data cell):</strong>

line height of the root element

<strong>Row 10</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vw"></a>

[vw](#vw)

<strong>Column 2 (data cell):</strong>

1% of viewport’s width

<strong>Row 11</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vh"></a>

[vh](#vh)

<strong>Column 2 (data cell):</strong>

1% of viewport’s height

<strong>Row 12</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vi"></a>

[vi](#vi)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-inline-axis"></a>

1% of viewport’s size in the root element’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis)

<strong>Row 13</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vb"></a>

[vb](#vb)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-block-axis"></a>

1% of viewport’s size in the root element’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis)

<strong>Row 14</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vmin"></a>

[vmin](#vmin)

<strong>Column 2 (data cell):</strong>

1% of viewport’s smaller dimension

<strong>Row 15</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-vmax"></a>

[vmax](#vmax)

<strong>Column 2 (data cell):</strong>

1% of viewport’s larger dimension

<a id="ref-for-computed-value①①"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-em②"></a>

<a id="ref-for-rem①"></a>

<a id="ref-for-ex①"></a>

<a id="ref-for-rex"></a>

<a id="ref-for-cap①"></a>

<a id="ref-for-rcap"></a>

<a id="ref-for-ch①"></a>

<a id="ref-for-rch"></a>

<a id="ref-for-ic①"></a>

<a id="ref-for-ric"></a>

<a id="ref-for-lh①"></a>

<a id="ref-for-rlh①"></a>

#### <a id="font-relative-lengths"></a>6.1.1.  Font-relative Lengths: the [em](#em), [rem](#rem), [ex](#ex), [rex](#rex), [cap](#cap), [rcap](#rcap), [ch](#ch), [rch](#rch), [ic](#ic), [ric](#ric), [lh](#lh), [rlh](#rlh) units

The <a id="font-relative-length"></a>font-relative lengths refer to the font metrics either of the element on which they are used (for the <a id="local-font-relative-lengths"></a>local font-relative lengths) or of the root element (for the <a id="root-font-relative-lengths"></a>root font-relative lengths).

![The word 'Sphinx' annotated with various font metrics: ascender height, to the top of the h's serif; cap height, to the visually approximate top of the S; the x height, to the visually approximate top of the x; the baseline, along the bottom of S, h, i, n, and x; and the descender height, to the bottom fo the p.](https://www.w3.org/TR/2024/WD-css-values-4-20240312/images/Typography_Line_Terms.svg)

Common typographic metrics

<a id="em"></a>em  
<a id="ref-for-propdef-font-size②"></a>

Equal to the computed value of the [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) property of the element on which it is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-83bc8a19"></a> The rule:
> ```text
> h1 { line-height: 1.2em }
> ```
>
> means that the line height of `h1` elements will be 20% greater than the font size of `h1` element. On the other hand:
>
> ```text
> h1 { font-size: 1.2em }
> ```
>
> means that the font size of `h1` elements will be 20% greater than the computed font size inherited by `h1` elements.

<a id="rem"></a>rem  
<a id="ref-for-em③"></a>

Equal to the computed value of the [em](#em) unit on the root element.

<a id="ex"></a>ex  
<a id="ref-for-user-agent④"></a>

<a id="ref-for-ex②"></a>

Equal to the used x-height of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font) [\[CSS3-FONTS\]](#biblio-css3-fonts). The x-height is so called because it is often equal to the height of the lowercase "x". However, an [ex](#ex) is defined even for fonts that do not contain an "x". The x-height of a font can be found in different ways. Some fonts contain reliable metrics for the x-height. If reliable font metrics are not available, [UAs](https://www.w3.org/TR/css-2023/#user-agent) may determine the x-height from the height of a lowercase glyph. One possible heuristic is to look at how far the glyph for the lowercase "o" extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the x-height, a value of 0.5em must be assumed.

<a id="rex"></a>rex  
<a id="ref-for-ex③"></a>

Equal to the value of the [ex](#ex) unit on the root element.

<a id="cap"></a>cap  
<a id="ref-for-user-agent⑤"></a>

<a id="ref-for-cap②"></a>

Equal to the used cap-height of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font) [\[CSS3-FONTS\]](#biblio-css3-fonts). The cap-height is so called because it is approximately equal to the height of a capital Latin letter. However, a [cap](#cap) is defined even for fonts that do not contain Latin letters. The cap-height of a font can be found in different ways. Some fonts contain reliable metrics for the cap-height. If reliable font metrics are not available, [UAs](https://www.w3.org/TR/css-2023/#user-agent) may determine the cap-height from the height of an uppercase glyph. One possible heuristic is to look at how far the glyph for the uppercase “O” extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the cap-height, the font’s ascent must be used.

<a id="rcap"></a>rcap  
<a id="ref-for-cap③"></a>

Equal to the value of the [cap](#cap) unit on the root element.

<a id="ch"></a>ch  
<a id="ref-for-length-advance-measure②"></a>

Represents the typical [advance measure](#length-advance-measure) of European alphanumeric characters, and measured as the used <a id="ref-for-length-advance-measure③"></a>advance measure of the “0” (ZERO, U+0030) glyph in the font used to render it. (The <a id="length-advance-measure"></a>advance measure of a glyph is its advance width or height, whichever is in the inline axis of the element.)

<a id="ref-for-length-advance-measure④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This measurement is an approximation (and in monospace fonts, an exact measure) of a single narrow glyph’s [advance measure](#length-advance-measure), thus allowing measurements based on an expected glyph count.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The advance measure of a glyph depends on writing-mode and text-orientation as well as font settings, text-transform, and any other properties that affect glyph selection or orientation.

<a id="ref-for-ch②"></a>

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl"></a>

<a id="ref-for-valdef-writing-mode-vertical-lr"></a>

<a id="ref-for-propdef-text-orientation"></a>

<a id="ref-for-valdef-text-orientation-upright"></a>

In the cases where it is impossible or impractical to determine the measure of the “0” glyph, it must be assumed to be 0.5em wide by 1em tall. Thus, the [ch](#ch) unit falls back to 0.5em in the general case, and to 1em when it would be typeset upright (i.e. [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) is [vertical-rl](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-rl) or [vertical-lr](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-lr) and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation) is [upright](https://www.w3.org/TR/css-writing-modes-4/#valdef-text-orientation-upright)).

<a id="rch"></a>rch  
<a id="ref-for-ch③"></a>

Equal to the value of the [ch](#ch) unit on the root element.

<a id="ic"></a>ic  
<a id="ref-for-length-advance-measure⑤"></a>

Represents the typical [advance measure](#length-advance-measure) of CJK letters, and measured as the used <a id="ref-for-length-advance-measure⑥"></a>advance measure of the “水” (CJK water ideograph, U+6C34) glyph found in the font used to render it.

<a id="ref-for-length-advance-measure⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This measurement is a typically an exact measure (in the few fonts with proportional fullwidth glyphs, an approximation) of a single [fullwidth](http://unicode.org/reports/tr11/#Definitions) glyph’s [advance measure](#length-advance-measure), thus allowing measurements based on an expected glyph count.

In the cases where it is impossible or impractical to determine the ideographic advance measure, it must be assumed to be 1em.

<a id="ric"></a>ric  
<a id="ref-for-ic②"></a>

Equal to the value of the [ic](#ic) unit on the root element.

<a id="lh"></a>lh  
<a id="ref-for-valdef-line-height-normal"></a>

<a id="ref-for-propdef-line-height①"></a>

Equal to the computed value of the [line-height](https://drafts.csswg.org/css2/#propdef-line-height) property of the element on which it is used, converting [normal](https://www.w3.org/TR/css-inline-3/#valdef-line-height-normal) to an absolute length by using only the metrics of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font).

<a id="rlh"></a>rlh  
<a id="ref-for-lh②"></a>

Equal to the value of the [lh](#lh) unit on the root element.

<a id="ref-for-propdef-height"></a>

<a id="ref-for-lh③"></a>

<a id="ref-for-rlh②"></a>

<a id="ref-for-propdef-max-lines"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Setting the [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of an element using either the [lh](#lh) or the [rlh](#rlh) units does not enable authors to control the actual number of lines in that element. These units only enable length calculations based on the theoretical size of an ideal empty line; the size of actual lines boxes may differ based on their content. In cases where an author wants to limit the number of actual lines in an element, the [max-lines](https://www.w3.org/TR/css-overflow-4/#propdef-max-lines) property can be used instead.

<a id="ref-for-font-relative-length"></a>

<a id="ref-for-propdef-font"></a>

<a id="ref-for-propdef-line-height②"></a>

<a id="ref-for-lh④"></a>

<a id="ref-for-rlh③"></a>

When used in the value of any font-\* property on the element they refer to, the [font-relative lengths](#font-relative-length) resolve against the computed metrics of the parent element—​or against the computed metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) and [line-height](https://drafts.csswg.org/css2/#propdef-line-height) properties, if the element has no parent. Similarly, when [lh](#lh) or [rlh](#rlh) units are used in the value of the <a id="ref-for-propdef-line-height③"></a>line-height property or font-\* properties on the element they refer to, they resolve against the computed <a id="ref-for-propdef-line-height④"></a>line-height and font metrics of the parent element—​or the computed metrics corresponding to the initial values of the <a id="ref-for-propdef-font①"></a>font and <a id="ref-for-propdef-line-height⑤"></a>line-height properties, if the element has no parent. (The other font-relative lengths continue to resolve against the element’s own metrics when used in <a id="ref-for-propdef-line-height⑥"></a>line-height.)

<a id="ref-for-media-query"></a>

<a id="ref-for-font-relative-length①"></a>

<a id="ref-for-propdef-font②"></a>

<a id="ref-for-propdef-line-height⑦"></a>

<a id="ref-for-root-font-relative-lengths"></a>

When used outside the context of an element (such as in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), the [font-relative lengths](#font-relative-length) units refer to the metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) and [line-height](https://drafts.csswg.org/css2/#propdef-line-height) properties. Similarly, when specified in a document with no root element, the [root font-relative lengths](#root-font-relative-lengths) are resolved assuming the initial values of the <a id="ref-for-propdef-font③"></a>font and <a id="ref-for-propdef-line-height⑧"></a>line-height properties.

<a id="ref-for-ch④"></a>

<a id="ref-for-ic③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Font-relative units such as [ch](#ch) and [ic](#ic) can trigger font downloads, if a required font is not yet loaded.

<a id="ref-for-font-relative-length②"></a>

The [font-relative lengths](#font-relative-length) are calculated in the absence of shaping.

<a id="ref-for-used-value③"></a>

<a id="ref-for-font-relative-length③"></a>

<a id="ref-for-media-query①"></a>

Some user-agents allow users to apply additional restrictions to font sizes in a document, such as setting minimum font sizes to ensure readability. Such restrictions must be applied to the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the affected properties only; they <em>must not</em> affect the resolution of [font-relative lengths](#font-relative-length) used in properties. However, in other contexts (such as in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), to the extent that they would impact the used font metrics, such restrictions <em>do</em> affect the resolution of <a id="ref-for-font-relative-length④"></a>font-relative lengths.

<a id="ref-for-propdef-font-size③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In general, respecting a user’s preferences, like minimum font sizes, is desirable; it’s useful for a media query like (min-width: 40em) to use the actual font size the document will be displayed in. However, having these preferences affect font-relative lengths <em>in properties on an element</em> was found to not be Web-compatible; too many pages expect these units to be exact multiples of the specified [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), rather than the <em>actual</em> font-size after applying user preferences.

<a id="ref-for-propdef-line-height⑨"></a>

<a id="ref-for-lh⑤"></a>

<a id="ref-for-rlh④"></a>

<a id="ref-for-implementation-defined③"></a>

Some user-agents apply restrictions to the [line-height](https://drafts.csswg.org/css2/#propdef-line-height) values on form controls. These must have no effect on the [lh](#lh) and [rlh](#rlh) units. The effect on their descendants, however, is [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined).

#### <a id="viewport-relative-lengths"></a>6.1.2.  Viewport-percentage Lengths: the \*vw, \*vh, \*vi, \*vb, \*vmin, \*vmax units

<a id="ref-for-continuous-media"></a>

<a id="ref-for-page-area"></a>

<a id="ref-for-paged-media"></a>

The <a id="viewport-percentage-lengths"></a>viewport-percentage lengths are relative to the size of the [initial containing block](https://www.w3.org/TR/CSS2/visudet.html#containing-block-details)—​which is itself based on the size of either the viewport (for [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media)) or the [page area](https://www.w3.org/TR/css-page-3/#page-area) (for [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media)). When the height or width of the initial containing block is changed, they are scaled accordingly.

##### <a id="viewport-variants"></a>6.1.2.1.  The Large, Small, and Dynamic Viewport Sizes

<a id="ref-for-viewport-percentage-lengths"></a>

There are four variants of the [viewport-percentage length](#viewport-percentage-lengths) units, corresponding to three (possibly identical) notions of the viewport size.

large viewport  
<a id="ref-for-user-agent⑥"></a>

The <a id="large-viewport-percentage-units"></a>large viewport-percentage units (lv\*) and <a id="default-viewport-percentage-units"></a>default viewport-percentage units (v\*) are defined with respect to the <a id="large-viewport-size"></a>large viewport size: the viewport sized assuming any [UA](https://www.w3.org/TR/css-2023/#user-agent) interfaces that are dynamically expanded and retracted to be retracted. This allows authors to size content such that it is guaranteed to fill the viewport, noting that such content might be hidden behind such interfaces when they are expanded.

<a id="ref-for-large-viewport-percentage-units"></a>

The sizes of the [large viewport-percentage units](#large-viewport-percentage-units) are fixed (and therefore stable) unless the viewport itself is resized.

<a id="ref-for-large-viewport-percentage-units①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-18205b29"></a> For example, on phones, where screen real-estate is at a premium, browsers will often hide part or all of the title and address bar once the user starts scrolling the page. The [large viewport-percentage units](#large-viewport-percentage-units) are sized relative to this larger everything-retracted space, so content using these units will fill the entire visible page when these UI elements are hidden. However, when these retractable elements are shown, they can obscure content that is sized or positioned using these units.

small viewport  
<a id="ref-for-user-agent⑦"></a>

The <a id="small-viewport-percentage-units"></a>small viewport-percentage units (sv\*) are defined with respect to the <a id="small-viewport-size"></a>small viewport size: the viewport sized assuming any [UA](https://www.w3.org/TR/css-2023/#user-agent) interfaces that are dynamically expanded and retracted to be expanded. This allows authors to size content such that it can fit within the viewport even when such interfaces are present, noting that such content might not fill the viewport when such interfaces are retracted.

<a id="ref-for-small-viewport-percentage-units"></a>

The sizes of the [small viewport-percentage units](#small-viewport-percentage-units) are fixed (and therefore stable) unless the viewport itself is resized.

<a id="ref-for-propdef-height①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1ac496e"></a> An element that is sized as [height: 100svh](https://www.w3.org/TR/css-sizing-3/#propdef-height), for example, will fill the screen perfectly, without any of its content being obscured, when all the dynamic UI elements of the UA are shown.
> <a id="ref-for-small-viewport-percentage-units①"></a>
>
> Once those UI elements start being hidden, however, there will be extra space around the element. The [small viewport-percentage units](#small-viewport-percentage-units) units are thus “safer” in general, but might not produce the most attractive layout once the user starts interacting with the page.

dynamic viewport  
<a id="ref-for-user-agent⑧"></a>

The <a id="dynamic-viewport-percentage-units"></a>dynamic viewport-percentage units (dv\*) are defined with respect to the <a id="dynamic-viewport-size"></a>dynamic viewport size: the viewport sized with dynamic consideration of any [UA](https://www.w3.org/TR/css-2023/#user-agent) interfaces that are dynamically expanded and retracted. This allows authors to size content such that it can exactly fit within the viewport whether or not such interfaces are present.

<a id="ref-for-dynamic-viewport-percentage-units"></a>

The sizes of the [dynamic viewport-percentage units](#dynamic-viewport-percentage-units) <em>are not stable</em> even while the viewport itself is unchanged. Using these units can cause content to resize e.g. while the user scrolls the page. Depending on usage, this can be disturbing to the user and/or costly in terms of performance.

<a id="ref-for-dynamic-viewport-percentage-units①"></a>

The UA is not required to animate the [dynamic viewport-percentage units](#dynamic-viewport-percentage-units) while expanding and retracting any relevant interfaces, and may instead calculate the units as if the relevant interface was fully expanded or retracted during the UI animation. (It is recommended that UAs assume the fully-retracted size for this duration.)

<a id="ref-for-viewport-percentage-lengths①"></a>

<a id="ref-for-initial-containing-block"></a>

<a id="ref-for-large-viewport-size"></a>

<a id="ref-for-small-viewport-size"></a>

Whether the expansion/retraction of a particular interface (A) changes the sizes of all of the [viewport-percentage lengths](#viewport-percentage-lengths) (and the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block)) simultaneously or (B) contributes to the differences between the [large viewport size](#large-viewport-size) and [small viewport size](#small-viewport-size) is largely UA-dependent. However:

- Changes in interface that happen as a result of scrolling or other frequent page interactions that would disturb the user if they resulted in substantial layout changes must be categorized as the latter (B).

- Changes in interface that have a sufficiently steady state that re-laying out the document into the adjusted space would be beneficial to the user must be categorized as the former (A).

- <a id="ref-for-viewport-percentage-lengths②"></a>

  Additionally, UAs may have some dynamically-shown interfaces that intentionally overlay content and do not cause any shifts in layout—​and therefore have no effect on any of the [viewport-percentage lengths](#viewport-percentage-lengths). (Typically on-screen keyboards will fit into this category.)

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-propdef-scrollbar-gutter"></a>

<a id="ref-for-root-element"></a>

<a id="ref-for-computed-value①②"></a>

<a id="ref-for-viewport-percentage-lengths③"></a>

<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-media-query②"></a>

In all cases, if the value of [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) or [scrollbar-gutter](https://www.w3.org/TR/css-overflow-3/#propdef-scrollbar-gutter) on the [root element](https://www.w3.org/TR/css-display-3/#root-element) in either axis would cause scrollbars to appear (or space to be reserved for them) unconditionally (for example, <a id="ref-for-propdef-overflow①"></a>overflow: scroll, but not <a id="ref-for-propdef-overflow②"></a>overflow: auto), the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [viewport-percentage lengths](#viewport-percentage-lengths) in that axis are reduced in accordance with the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block). Otherwise, and always in the case of [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query), the <a id="ref-for-viewport-percentage-lengths④"></a>viewport-percentage lengths are sized assuming that scrollbars do not exist (even if this diverges from the <a id="ref-for-initial-containing-block②"></a>initial containing block).

<a id="ref-for-propdef-overflow③"></a>

<a id="ref-for-the-body-element-2"></a>

<a id="ref-for-root-element①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The value of [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) on [the body element](https://html.spec.whatwg.org/multipage/dom.html#the-body-element-2) can sometimes affect the presence of scrollbars on the [root element](https://www.w3.org/TR/css-display-3/#root-element). This <em>does not</em> affect the size of viewport units, however.

##### <a id="viewport-relative-units"></a>6.1.2.2.  The Various Viewport-relative Units

<a id="ref-for-viewport-percentage-lengths⑤"></a>

The [viewport-percentage length](#viewport-percentage-lengths) units are:

<a id="vw"></a>vw  
<a id="svw"></a>svw  
<a id="lvw"></a>lvw  
<a id="dvw"></a>dvw  
<a id="ref-for-dynamic-viewport-size"></a>

<a id="ref-for-small-viewport-size①"></a>

<a id="ref-for-large-viewport-size①"></a>

Equal to 1% of the width of the [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), <a id="ref-for-large-viewport-size②"></a>large viewport size, and [dynamic viewport size](#dynamic-viewport-size), respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6068ee5d"></a> In the example below, if the width of the viewport is 200mm, the font size of `h1` elements will be 16mm (i.e. (8×200mm)/100).
> ```text
> h1 { font-size: 8vw }
> ```
<a id="vh"></a>vh  
<a id="svh"></a>svh  
<a id="lvh"></a>lvh  
<a id="dvh"></a>dvh  
<a id="ref-for-dynamic-viewport-size①"></a>

<a id="ref-for-small-viewport-size②"></a>

<a id="ref-for-large-viewport-size③"></a>

Equal to 1% of the height of the [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), <a id="ref-for-large-viewport-size④"></a>large viewport size, and [dynamic viewport size](#dynamic-viewport-size), respectively.

<a id="vi"></a>vi  
<a id="svi"></a>svi  
<a id="lvi"></a>lvi  
<a id="dvi"></a>dvi  
<a id="ref-for-inline-axis①"></a>

<a id="ref-for-dynamic-viewport-size②"></a>

<a id="ref-for-small-viewport-size③"></a>

<a id="ref-for-large-viewport-size⑤"></a>

Equal to 1% of the size of the [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), <a id="ref-for-large-viewport-size⑥"></a>large viewport size, and [dynamic viewport size](#dynamic-viewport-size) (respectively) in the box’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="vb"></a>vb  
<a id="svb"></a>svb  
<a id="lvb"></a>lvb  
<a id="dvb"></a>dvb  
<a id="ref-for-block-axis①"></a>

<a id="ref-for-dynamic-viewport-size③"></a>

<a id="ref-for-small-viewport-size④"></a>

<a id="ref-for-large-viewport-size⑦"></a>

Equal to 1% of the size of the initial containing block [large viewport size](#large-viewport-size), [small viewport size](#small-viewport-size), <a id="ref-for-large-viewport-size⑧"></a>large viewport size, and [dynamic viewport size](#dynamic-viewport-size) (respectively) in the box’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

<a id="vmin"></a>vmin  
<a id="svmin"></a>svmin  
<a id="lvmin"></a>lvmin  
<a id="dvmin"></a>dvmin  
Equal to the smaller of \*vw or \*vh.

<a id="vmax"></a>vmax  
<a id="svmax"></a>svmax  
<a id="lvmax"></a>lvmax  
<a id="dvmax"></a>dvmax  
Equal to the larger of \*vw or \*vh.

<a id="ref-for-initial-containing-block③"></a>

<a id="ref-for-continuous-media①"></a>

<a id="ref-for-large-viewport-percentage-units②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The original (unprefixed) viewport units [were defined](https://www.w3.org/TR/css-values-3/#viewport-relative-lengths) relative to the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block), which in [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media) always matched the (singular) viewport size. The dynamism of browser chrome shifting in and out during scrolling was invented later, and following Safari’s lead, most UAs mapped these units to the larger size. Defining it this way is prettier in many cases, but can also block critical content (such as toolbars, headers, and footers) in others. It’s therefore not entirely clear whether this was the best mapping, and thus earlier editions of this specifications allowed UAs to choose the mapping of these default units. However at this point the mapping to the [large viewport-percentage units](#large-viewport-percentage-units) is presumed to be required for Web compatibility.

<a id="ref-for-media-query③"></a>

<a id="ref-for-propdef-writing-mode①"></a>

In situations where there is no element or it hasn’t yet been styled (such as when evaluating [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), the \*vi and \*vb units use the initial value of the [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) property to determine which axis they correspond to.

<a id="ref-for-cm"></a>

<a id="ref-for-mm"></a>

<a id="ref-for-Q"></a>

<a id="ref-for-in①"></a>

<a id="ref-for-pt"></a>

<a id="ref-for-pc"></a>

<a id="ref-for-px③"></a>

### <a id="absolute-lengths"></a>6.2.  Absolute Lengths: the [cm](#cm), [mm](#mm), [Q](#Q), [in](#in), [pt](#pt), [pc](#pc), [px](#px) units

<a id="ref-for-anchor-unit"></a>

<a id="ref-for-in②"></a>

<a id="ref-for-cm①"></a>

<a id="ref-for-mm①"></a>

<a id="ref-for-pt①"></a>

<a id="ref-for-pc①"></a>

<a id="ref-for-Q①"></a>

<a id="ref-for-px④"></a>

The <a id="absolute-length"></a>absolute length units are fixed in relation to each other and [anchored](#anchor-unit) to some physical measurement. They are mainly useful when the output environment is known. The absolute units consist of the <a id="physical-unit"></a>physical units ([in](#in), [cm](#cm), [mm](#mm), [pt](#pt), [pc](#pc), [Q](#Q)) and the <a id="visual-angle-unit"></a>visual angle unit (pixel unit) ([px](#px)):

| unit                  | name                | equivalence         |
|-----------------------|---------------------|---------------------|
| <a id="cm"></a>cm | centimeters         | 1cm = 96px/2.54     |
| <a id="mm"></a>mm | millimeters         | 1mm = 1/10th of 1cm |
| <a id="Q"></a>Q  | quarter-millimeters | 1Q = 1/40th of 1cm  |
| <a id="in"></a>in | inches              | 1in = 2.54cm = 96px |
| <a id="pc"></a>pc | picas               | 1pc = 1/6th of 1in  |
| <a id="pt"></a>pt | points              | 1pt = 1/72nd of 1in |
| <a id="px"></a>px | pixels              | 1px = 1/96th of 1in |

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-afc1f57e"></a>
>
> ```text
> h1 { margin: 0.5in }      /* inches  */
> h2 { line-height: 3cm }   /* centimeters */
> h3 { word-spacing: 4mm }  /* millimeters */
> h3 { letter-spacing: 1Q } /* quarter-millimeters */
> h4 { font-size: 12pt }    /* points */
> h4 { font-size: 1pc }     /* picas */
> p  { font-size: 12px }    /* px */
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Lengths in publishing contexts are sometimes written like `2p3`, indicating a length of 2 picas and 3 points. These can be written in CSS as calc(2pc + 3pt) (see [§ 10.1 Basic Arithmetic: calc()](#calc-func)).

<a id="ref-for-compatible-units⑤"></a>

<a id="ref-for-px⑤"></a>

<a id="ref-for-canonical-unit③"></a>

All of the absolute length units are [compatible](#compatible-units), and [px](#px) is their [canonical unit](#canonical-unit).

For a CSS device, these dimensions are <a id="anchor-unit"></a>anchored either

1.  <a id="ref-for-physical-unit"></a>

    by relating the [physical units](#physical-unit) to their physical measurements, or

2.  <a id="ref-for-reference-pixel"></a>

    <a id="ref-for-visual-angle-unit"></a>

    by relating the [pixel unit](#visual-angle-unit) to the [reference pixel](#reference-pixel).

<a id="ref-for-anchor-unit①"></a>

<a id="ref-for-physical-unit①"></a>

<a id="ref-for-visual-angle-unit①"></a>

<a id="ref-for-device-pixel⑤"></a>

For print media at typical viewing distances, the [anchor unit](#anchor-unit) should be one of the [physical units](#physical-unit) (inches, centimeters, etc). For screen media (including high-resolution devices), low-resolution devices, and devices with unusual viewing distances, it is recommended instead that the <a id="ref-for-anchor-unit②"></a>anchor unit be the [pixel unit](#visual-angle-unit). For such devices it is recommended that the <a id="ref-for-visual-angle-unit②"></a>pixel unit refer to the whole number of [device pixels](#device-pixel) that best approximates the reference pixel.

<a id="ref-for-anchor-unit③"></a>

<a id="ref-for-visual-angle-unit③"></a>

<a id="ref-for-physical-unit②"></a>

<a id="ref-for-device-pixel⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the [anchor unit](#anchor-unit) is the [pixel unit](#visual-angle-unit), the [physical units](#physical-unit) might not match their physical measurements. Alternatively if the <a id="ref-for-anchor-unit④"></a>anchor unit is a <a id="ref-for-physical-unit③"></a>physical unit, the <a id="ref-for-visual-angle-unit④"></a>pixel unit might not map to a whole number of [device pixels](#device-pixel).

<a id="ref-for-visual-angle-unit⑤"></a>

<a id="ref-for-physical-unit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This definition of the [pixel unit](#visual-angle-unit) and the [physical units](#physical-unit) differs from the earlier editions of CSS1 and CSS2. In particular, in previous versions of CSS the <a id="ref-for-visual-angle-unit⑥"></a>pixel unit and the <a id="ref-for-physical-unit⑤"></a>physical units were not related by a fixed ratio: the <a id="ref-for-physical-unit⑥"></a>physical units were always tied to their physical measurements while the <a id="ref-for-visual-angle-unit⑦"></a>pixel unit would vary to most closely match the reference pixel. (This unfortunate change was made because too much existing content relies on the assumption of 96dpi, and breaking that assumption broke the content.)

<a id="ref-for-ascii-case-insensitive③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Units are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) and serialize as lowercase, for example 1Q serializes as 1q.

<a id="ref-for-device-pixel⑦"></a>

The <a id="reference-pixel"></a>reference pixel is the visual angle of one pixel on a device with a [device pixel](#device-pixel) density of 96dpi and a distance from the reader of an arm’s length. For a nominal arm’s length of 28 inches, the visual angle is therefore about 0.0213 degrees. For reading at arm’s length, 1px thus corresponds to about 0.26 mm (1/96 inch).

The image below illustrates the effect of viewing distance on the size of a reference pixel: a reading distance of 71 cm (28 inches) results in a reference pixel of 0.26 mm, while a reading distance of 3.5 m (12 feet) results in a reference pixel of 1.3 mm.

![This diagram illustrates how the definition of a pixel depends on the users distance from the viewing surface (paper or screen). The image depicts the user looking at two planes, one 28 inches (71 cm) from the user, the second 140 inches (3.5 m) from the user. An expanding cone is projected from the user's eye onto each plane. Where the cone strikes the first plane, the projected pixel is 0.26 mm high. Where the cone strikes the second plane, the projected pixel is 1.4 mm high.](https://www.w3.org/TR/2024/WD-css-values-4-20240312/images/pixel1.png)

Showing that pixels must become larger if the viewing distance increases

This second image illustrates the effect of a device’s resolution on the pixel unit: an area of 1px by 1px is covered by a single dot in a low-resolution device (e.g. a typical computer display), while the same area is covered by 16 dots in a higher resolution device (such as a printer).

![This diagram illustrates the relationship between the reference pixel and device pixels (called "dots" below). The image depicts a high resolution (large dot density) laser printer output on the left and a low resolution monitor screen on the right. For the laser printer, one square reference pixel is implemented by 16 dots. For the monitor screen, one square reference pixel is implemented by a single dot.](https://www.w3.org/TR/2024/WD-css-values-4-20240312/images/pixel2.png)

Showing that more device pixels (dots) are needed to cover a 1px by 1px area on a high-resolution device than on a lower-resolution one (of the same approximate viewing distance)

A <a id="device-pixel"></a>device pixel is the smallest unit of area on the device output capable of displaying its full range of colors. For typical color screens, it’s a square or somewhat rectangular region containing a red, green, and blue subpixel. Many non-traditional outputs exist that can blur this definition, such as by displaying some colors at higher resolutions. Such devices still expose some equivalent notion of "device pixel", however.

## <a id="other-units"></a>7.  Other Quantities

<a id="ref-for-angle-value⑥"></a>

<a id="ref-for-deg"></a>

<a id="ref-for-grad"></a>

<a id="ref-for-rad"></a>

<a id="ref-for-turn"></a>

### <a id="angles"></a>7.1.  Angle Units: the [\<angle\>](#angle-value) type and [deg](#deg), [grad](#grad), [rad](#rad), [turn](#turn) units

<a id="ref-for-typedef-dimension①"></a>

Angle values are [\<dimension\>](#typedef-dimension)s denoted by <a id="angle-value"></a>\<angle\>. The angle unit identifiers are:

<a id="deg"></a>deg  
Degrees. There are 360 degrees in a full circle.

<a id="grad"></a>grad  
Gradians, also known as "gons" or "grades". There are 400 gradians in a full circle.

<a id="rad"></a>rad  
Radians. There are 2π radians in a full circle.

<a id="turn"></a>turn  
Turns. There is 1 turn in a full circle.

For example, a right angle is 90deg or 100grad or 0.25turn or approximately 1.57rad.

<a id="ref-for-angle-value⑦"></a>

<a id="ref-for-compatible-units⑥"></a>

<a id="ref-for-deg①"></a>

<a id="ref-for-canonical-unit④"></a>

All [\<angle\>](#angle-value) units are [compatible](#compatible-units), and [deg](#deg) is their [canonical unit](#canonical-unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> By convention, when an angle denotes a direction in CSS, it is typically interpreted as a <a id="bearing-angle"></a>bearing angle, where 0deg is "up" or "north" on the screen, and larger angles are more clockwise (so 90deg is "right" or "east").
>
> <a id="ref-for-funcdef-linear-gradient"></a>
>
> <a id="ref-for-angle-value⑧"></a>
>
> For example, in the [linear-gradient()](https://www.w3.org/TR/css-images-4/#funcdef-linear-gradient) function, the [\<angle\>](#angle-value) that determines the direction of the gradient is interpreted as a bearing angle.

<a id="ref-for-angle-value⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For legacy reasons, some uses of [\<angle\>](#angle-value) allow a bare 0 to mean 0deg. This is not true in general, however, and will not occur in future uses of the <a id="ref-for-angle-value①⓪"></a>\<angle\> type.

<a id="ref-for-time-value⑥"></a>

<a id="ref-for-s"></a>

<a id="ref-for-ms"></a>

### <a id="time"></a>7.2.  Duration Units: the [\<time\>](#time-value) type and [s](#s), [ms](#ms) units

<a id="ref-for-dimension⑨"></a>

Time values are [dimensions](#dimension) denoted by <a id="time-value"></a>\<time\>. The time unit identifiers are:

<a id="s"></a>s  
Seconds.

<a id="ms"></a>ms  
Milliseconds. There are 1000 milliseconds in a second.

<a id="ref-for-time-value⑦"></a>

<a id="ref-for-compatible-units⑦"></a>

<a id="ref-for-s①"></a>

<a id="ref-for-canonical-unit⑤"></a>

All [\<time\>](#time-value) units are [compatible](#compatible-units), and [s](#s) is their [canonical unit](#canonical-unit).

Properties may restrict the time value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore).

<a id="ref-for-frequency-value⑤"></a>

<a id="ref-for-Hz"></a>

<a id="ref-for-kHz"></a>

### <a id="frequency"></a>7.3.  Frequency Units: the [\<frequency\>](#frequency-value) type and [Hz](#Hz), [kHz](#kHz) units

<a id="ref-for-dimension①⓪"></a>

Frequency values are [dimensions](#dimension) denoted by <a id="frequency-value"></a>\<frequency\>. The frequency unit identifiers are:

<a id="Hz"></a>Hz  
Hertz. It represents the number of occurrences per second.

<a id="kHz"></a>kHz  
KiloHertz. A kiloHertz is 1000 Hertz.

For example, when representing sound pitches, 200Hz (or 200hz) is a bass sound, and 6kHz (or 6khz) is a treble sound.

<a id="ref-for-frequency-value⑥"></a>

<a id="ref-for-compatible-units⑧"></a>

<a id="ref-for-Hz①"></a>

<a id="ref-for-canonical-unit⑥"></a>

All [\<frequency\>](#frequency-value) units are [compatible](#compatible-units), and [hz](#Hz) is their [canonical unit](#canonical-unit).

<a id="ref-for-ascii-case-insensitive④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Units are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) and serialize as lowercase, for example 1Hz serializes as 1hz.

<a id="ref-for-resolution-value②"></a>

<a id="ref-for-dpi"></a>

<a id="ref-for-dpcm"></a>

<a id="ref-for-dppx"></a>

### <a id="resolution"></a>7.4.  Resolution Units: the [\<resolution\>](#resolution-value) type and [dpi](#dpi), [dpcm](#dpcm), [dppx](#dppx) units

<a id="ref-for-dimension①①"></a>

Resolution units are [dimensions](#dimension) denoted by <a id="resolution-value"></a>\<resolution\>. The resolution unit identifiers are:

<a id="dpi"></a>dpi  
Dots per inch.

<a id="dpcm"></a>dpcm  
Dots per centimeter.

<a id="dppx"></a>dppx  
<a id="x"></a>x  
<a id="ref-for-px⑥"></a>

Dots per [px](#px) unit.

<a id="ref-for-resolution-value③"></a>

<a id="ref-for-in③"></a>

<a id="ref-for-cm②"></a>

<a id="ref-for-px⑦"></a>

<a id="ref-for-propdef-image-resolution"></a>

The [\<resolution\>](#resolution-value) unit represents the size of a single "dot" in a graphical representation by indicating how many of these dots fit in a CSS [in](#in), [cm](#cm), or [px](#px). For uses, see e.g. the resolution media query in [\[MEDIAQ\]](#biblio-mediaq) or the [image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution) property defined in [\[CSS3-IMAGES\]](#biblio-css3-images).

<a id="ref-for-resolution-value④"></a>

<a id="ref-for-compatible-units⑨"></a>

<a id="ref-for-canonical-unit⑦"></a>

All [\<resolution\>](#resolution-value) units are [compatible](#compatible-units), and dppx is their [canonical unit](#canonical-unit).

<a id="ref-for-resolution-value⑤"></a>

The allowed range of [\<resolution\>](#resolution-value) values <em>always</em> excludes negative values, in addition to any explicit ranges that might be specified.

<a id="ref-for-in④"></a>

<a id="ref-for-px⑧"></a>

<a id="ref-for-propdef-image-resolution①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that due to the 1:96 fixed ratio of CSS [in](#in) to CSS [px](#px), 1dppx is equivalent to 96dpi. This corresponds to the default resolution of images displayed in CSS: see [image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution).

<a id="ref-for-px⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33e77fee"></a> The following @media rule uses Media Queries [\[MEDIAQ\]](#biblio-mediaq) to assign some special style rules to devices that use two or more device pixels per CSS [px](#px) unit:
>
> ```text
> @media (min-resolution: 2dppx) { ... }
> ```
## <a id="defined-elsewhere"></a>8.  Data Types Defined Elsewhere

Some data types are defined in their own modules. This example talks about some of the most common ones used across several specifications.

<a id="ref-for-typedef-color"></a>

### <a id="colors"></a>8.1.  Colors: the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) type

<a id="ref-for-typedef-color①"></a>

The [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) data type is defined in [\[CSS-COLOR-4\]](#biblio-css-color-4). UAs must interpret <a id="ref-for-typedef-color②"></a>\<color\> as defined therein.

<a id="ref-for-typedef-color③"></a>

#### <a id="combine-colors"></a>8.1.1.  Combination of [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

<a id="ref-for-interpolation①②"></a>

<a id="ref-for-typedef-color④"></a>

[Interpolation](#interpolation) of [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) is defined in [CSS Color 4 §  12. Color Interpolation](https://www.w3.org/TR/css-color-4/#interpolation). Interpolation is done between premultiplied colors, as defined in [CSS Color 4 § 12.3 Interpolating with Alpha](https://www.w3.org/TR/css-color-4/#interpolation-alpha).

<a id="ref-for-typedef-color⑤"></a>

<a id="ref-for-not-additive①"></a>

The [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) type is [not additive](#not-additive).

<a id="ref-for-typedef-color⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the CSS WG is interested to [hear](https://github.com/w3c/csswg-drafts/issues/new) use-cases for addition of [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color), and may consider making <a id="ref-for-typedef-color⑦"></a>\<color\> additive in the future.

<a id="ref-for-typedef-image"></a>

### <a id="images"></a>8.2.  Images: the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) type

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-url-value①③"></a>

The [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) data type is defined in [\[CSS3-IMAGES\]](#biblio-css3-images). UAs that support CSS Images Level 3 or its successor must interpret <a id="ref-for-typedef-image②"></a>\<image\> as defined therein. UAs that do not yet support CSS Images Level 3 must interpret <a id="ref-for-typedef-image③"></a>\<image\> as [\<url\>](#url-value).

<a id="ref-for-typedef-image④"></a>

#### <a id="combine-images"></a>8.2.1.  Combination of [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)

<a id="ref-for-typedef-image⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Interpolation of [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) is defined in [CSS Images 3 § 6 Interpolation](https://www.w3.org/TR/css-images-3/#interpolation).

<a id="ref-for-not-additive②"></a>

Images are [not additive](#not-additive).

<a id="ref-for-typedef-position①"></a>

### <a id="position"></a>8.3.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-propdef-background-position"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of a object area (e.g. background image) inside a positioning area (e.g. background positioning area). It is computed and interpreted as specified for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position). [\[CSS3-BACKGROUND\]](#biblio-css3-background)

<a id="ref-for-typedef-position③"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

```text
<position> = [
  [ left | center | right | top | bottom | <length-percentage> ]
|
  [ left | center | right ] && [ top | center | bottom ]
|
  [ left | center | right | <length-percentage> ]
  [ top | center | bottom | <length-percentage> ]
|
  [ [ left | right ] <length-percentage> ] &&
  [ [ top | bottom ] <length-percentage> ]
]
```
<a id="ref-for-propdef-background-position①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) property also accepts a three-value syntax. This has been disallowed generically because it creates parsing ambiguities when combined with other length or percentage components in a property value.

<a id="ref-for-typedef-position④"></a>

#### <a id="position-parsing"></a>8.3.1.  Parsing [\<position\>](#typedef-position)

<a id="ref-for-length-value②⓪"></a>

<a id="ref-for-percentage-value②④"></a>

<a id="ref-for-typedef-position⑤"></a>

When specified in a grammar alongside other keywords, [\<length\>](#length-value)s, or [\<percentage\>](#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position⑥"></a>

<a id="ref-for-length-value②①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2622a7d7"></a> For example, [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) \<position\> \<length\>?. A value such as left 50px will be parsed as a 2-value [\<position\>](#typedef-position), with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑦"></a>\<position\> followed by a [\<length\>](#length-value).

<a id="ref-for-typedef-position⑧"></a>

#### <a id="position-serialization"></a>8.3.2.  Serializing [\<position\>](#typedef-position)

<a id="ref-for-specified-value③"></a>

<a id="ref-for-typedef-position⑨"></a>

When serializing the [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) of a [\<position\>](#typedef-position):

If only one component is specified:  
- <a id="ref-for-valdef-background-position-center"></a>

  The implied [center](https://www.w3.org/TR/css-backgrounds-3/#valdef-background-position-center) keyword is added, and a 2-component value is serialized.

If two components are specified:  
- Keywords are serialized as keywords.

- <a id="ref-for-typedef-length-percentage⑥"></a>

  [\<length-percentage\>](#typedef-length-percentage)s are serialized as <a id="ref-for-typedef-length-percentage⑦"></a>\<length-percentage\>s.

- Components are serialized horizontal first, then vertical.

If four components are specified:  
- Keywords and offsets are both serialized.

- Components are serialized horizontal first, then vertical.

<a id="ref-for-typedef-position①⓪"></a>

<a id="ref-for-length-value②②"></a>

<a id="ref-for-propdef-transform-origin①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<position\>](#typedef-position) values are never serialized as a single value, even when a single value would produce the same behavior, to avoid causing parsing ambiguities in some grammars where a <a id="ref-for-typedef-position①①"></a>\<position\> is placed next to a [\<length\>](#length-value), such as [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin).

<a id="ref-for-computed-value①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) are always serialized as two offsets (without keywords) because the <a id="ref-for-computed-value①④"></a>computed value does not preserve syntactic distinctions.

<a id="ref-for-typedef-position①②"></a>

#### <a id="combine-positions"></a>8.3.3.  Combination of [\<position\>](#typedef-position)

<a id="ref-for-interpolation①③"></a>

<a id="ref-for-typedef-position①③"></a>

<a id="ref-for-typedef-length-percentage⑧"></a>

[Interpolation](#interpolation) of [\<position\>](#typedef-position) is defined as the independent interpolation of each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](#typedef-length-percentage).

<a id="ref-for-addition①③"></a>

<a id="ref-for-typedef-position①④"></a>

<a id="ref-for-typedef-length-percentage⑨"></a>

[Addition](#addition) of [\<position\>](#typedef-position) is likewise defined as the independent <a id="ref-for-addition①④"></a>addition each component (x, y) normalized as an offset from the top left corner as a [\<length-percentage\>](#typedef-length-percentage).

## <a id="functional-notations"></a>9.  Functional Notations

<a id="ref-for-typedef-function-token①"></a>

<a id="ref-for-ascii-case-insensitive⑤"></a>

A <a id="functional-notation"></a>functional notation is a type of component value that can represent more complex types or invoke special processing. The syntax starts with the name of the function immediately followed by a left parenthesis (i.e. a [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token)) followed by the argument(s) to the notation followed by a right parenthesis. Like keywords, function names are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). [White space](https://www.w3.org/TR/css-syntax/#whitespace) is allowed, but optional, immediately inside the parentheses. Functions can take multiple arguments, which are formatted similarly to a CSS property value. See [§ 2.6 Functional Notation Definitions](#component-functions).

<a id="ref-for-functional-notation⑤"></a>

<a id="ref-for-funcdef-rgba"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some legacy [functional notations](#functional-notation), such as [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), use commas unnecessarily, but generally commas are only used to separate items in a list, or pieces of a grammar that would be ambiguous otherwise. If a comma is used to separate arguments, [white space](https://www.w3.org/TR/css-syntax/#whitespace) is optional before and after the comma.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3fbb1c9d"></a>
>
> ```text
> background: url(http://www.example.org/image);
> color: rgb(100, 200, 50 );
> content: counter(list-item) ". ";
> width: calc(50% - 2em);
> ```
<a id="ref-for-math-function①"></a>

<a id="ref-for-functional-notation⑥"></a>

<a id="ref-for-typedef-color⑧"></a>

The [math functions](#math-function) are defined below. Other [functional notations](#functional-notation) are defined in their own modules; for example the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) functions are defined in [\[CSS-COLOR-4\]](#biblio-css-color-4) and [\[CSS-COLOR-5\]](#biblio-css-color-5).

## <a id="math"></a>10.  Mathematical Expressions<a id="calc-notation"></a>

<a id="ref-for-funcdef-calc⑥"></a>

<a id="ref-for-funcdef-clamp"></a>

<a id="ref-for-funcdef-sin"></a>

The <a id="math-function"></a>math functions ([calc()](#funcdef-calc), [clamp()](#funcdef-clamp), [sin()](#funcdef-sin), and others defined in this chapter) allow numeric CSS values to be written as mathematical expressions.

<a id="ref-for-math-function②"></a>

A [math function](#math-function) represents a numeric value, one of:

- <a id="ref-for-length-value②③"></a>

  [\<length\>](#length-value),

- <a id="ref-for-frequency-value⑦"></a>

  [\<frequency\>](#frequency-value),

- <a id="ref-for-angle-value①①"></a>

  [\<angle\>](#angle-value),

- <a id="ref-for-time-value⑧"></a>

  [\<time\>](#time-value),

- <a id="ref-for-typedef-flex"></a>

  [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex),

- <a id="ref-for-resolution-value⑥"></a>

  [\<resolution\>](#resolution-value),

- <a id="ref-for-percentage-value②⑤"></a>

  [\<percentage\>](#percentage-value),

- <a id="ref-for-number-value①⑤"></a>

  [\<number\>](#number-value),

- <a id="ref-for-integer-value①①"></a>

  [\<integer\>](#integer-value)

<a id="ref-for-typedef-length-percentage①⓪"></a>

...or the [\<length-percentage\>](#typedef-length-percentage)/etc mixed types, and can be used wherever such a value would be valid.

<a id="ref-for-funcdef-calc⑦"></a>

### <a id="calc-func"></a>10.1.  Basic Arithmetic: [calc()](#funcdef-calc)

<a id="ref-for-math-function③"></a>

The <a id="funcdef-calc"></a>calc() function is a [math function](#math-function) that allows basic arithmetic to be performed on numerical values, using addition (+), subtraction (-), multiplication (\*), division (/), and parentheses.

<a id="ref-for-funcdef-calc⑧"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="ref-for-calc-calculation"></a>

A [calc()](#funcdef-calc) function contains a single <a id="calc-calculation"></a>calculation, which is a sequence of values interspersed with operators, and possibly grouped by parentheses (matching the [\<calc-sum\>](#typedef-calc-sum) grammar), which represents the result of evaluating the expression using standard operator precedence rules (\* and / bind tighter than + and -, and operators are otherwise evaluated left-to-right). The <a id="ref-for-funcdef-calc⑨"></a>calc() function represents the result of its contained [calculation](#calc-calculation).

<a id="ref-for-calc-calculation①"></a>

<a id="ref-for-math-function④"></a>

<a id="ref-for-funcdef-var①"></a>

<a id="ref-for-length-value②④"></a>

Components of a [calculation](#calc-calculation) can be literal values (such as 5px), other [math functions](#math-function), or other expressions, such as [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var), that evaluate to a valid argument type (like [\<length\>](#length-value)).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1daf921b"></a>
>
> <a id="ref-for-math-function⑤"></a>
>
> <a id="ref-for-propdef-box-sizing"></a>
>
> [Math functions](#math-function) can be used to combine value that use different units. In this example the author wants the <em>margin box</em> of each section to take up 1/3 of the space, so they start with 100%/3, then subtract the element’s borders and margins. ([box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) can automatically achieve this effect for borders and padding, but a <a id="ref-for-math-function⑥"></a>math function is needed if you want to include margins.)
>
> ```css
> section {
>   float: left;
>   margin: 1em; border: solid 1px;
>   width: calc(100% / 3 - 2 * 1em - 2 * 1px);
> }
> ```
>
> Similarly, in this example the gradient will show a color transition only in the first and last 20px of the element:
>
> ```css
> .fade {
>   background-image: linear-gradient(silver 0%, white 20px,
>                                     white calc(100% - 20px), silver 100%);
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a73c5015"></a>
>
> <a id="ref-for-math-function⑦"></a>
>
> <a id="ref-for-propdef-font-size④"></a>
>
> [Math functions](#math-function) can also be useful just to express values in a more natural, readable fashion, rather than as an obscure decimal. For example, the following sets the [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) so that exactly 35em fits within the viewport, ensuring that roughly the same amount of text always fills the screen no matter the screen size.
>
> ```css
> :root {
>   font-size: calc(100vw / 35);
> }
> ```
>
> <a id="ref-for-descdef-font-face-font-size"></a>
>
> Functionality-wise, this is identical to just writing [font-size: 2.857vw](https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size), but then the intent (that 35em fills the viewport) is much less clear to someone reading the code; the later reader will have to reverse the math themselves to figure out that 2.857 is meant to approximate 100/35.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-baa620d3"></a>
>
> Standard mathematical precedence rules for the operators apply: calc(2 + 3 \* 4) is equal to 14, not 20.
>
> Parentheses can be used to manipulate precedence: calc((2 + 3) \* 4) is instead equal to 20.
>
> <a id="ref-for-funcdef-calc①⓪"></a>
>
> <a id="ref-for-funcdef-var②"></a>
>
> Parentheses and nesting additional [calc()](#funcdef-calc) functions are equivalent; the preceding expression could equivalently have been written as calc(calc(2 + 3) \* 4). This can be useful when building up values piecemeal via [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var), such as in the following example:
>
> ```css
> .aspect-ratio-box {
>   --ar: calc(16 / 9);
>   --w: calc(100% / 3);
>   --h: calc(var(--w) / var(--ar));
>   width: var(--w);
>   height: var(--h);
> }
> ```
>
> <a id="ref-for-propdef-width⑦"></a>
>
> <a id="ref-for-funcdef-calc①①"></a>
>
> Although --ar <em>could</em> have been written as simply --ar: (16 / 9);, --w is used both on its own (in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)) and as a [calc()](#funcdef-calc) component (in --h), so it has to be written as a full <a id="ref-for-funcdef-calc①②"></a>calc() function itself.

<a id="ref-for-funcdef-min"></a>

<a id="ref-for-funcdef-max"></a>

<a id="ref-for-funcdef-clamp①"></a>

### <a id="comp-func"></a>10.2.  Comparison Functions: [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp)

<a id="ref-for-funcdef-min①"></a>

<a id="ref-for-funcdef-max①"></a>

<a id="ref-for-funcdef-clamp②"></a>

<a id="ref-for-calc-calculation②"></a>

The comparison functions of [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) compare multiple [calculations](#calc-calculation) and represent the value of one of them.

<a id="ref-for-calc-calculation③"></a>

The <a id="funcdef-min"></a>min() or <a id="funcdef-max"></a>max() functions contain one or more comma-separated [calculations](#calc-calculation), and represent the smallest (most negative) or largest (most positive) of them, respectively.

<a id="ref-for-calc-calculation④"></a>

The <a id="funcdef-clamp"></a>clamp() function takes three [calculations](#calc-calculation)—​a minimum value, a central value, and a maximum value—​and represents its central calculation, clamped according to its min and max calculations, favoring the min calculation if it conflicts with the max. (That is, given clamp(MIN, VAL, MAX), it represents exactly the same value as max(MIN, min(VAL, MAX))).

Either the min or max calculations (or even both) can instead be the keyword <a id="valdef-clamp-none"></a>none, which indicates the value is <em>not</em> clamped from that side. (That is, clamp(MIN, VAL, none) is equivalent to max(MIN, VAL), clamp(none, VAL, MAX) is equivalent to min(VAL, MAX), and clamp(none, VAL, none) is equivalent to just calc(VAL).)

<a id="ref-for-calc-calculation⑤"></a>

<a id="ref-for-number-value①⑥"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-percentage-value②⑥"></a>

<a id="ref-for-css-consistent-type"></a>

For all three functions, the argument [calculations](#calc-calculation) can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have a [consistent type](#css-consistent-type) or else the function is invalid; the result’s type will be the <a id="ref-for-css-consistent-type①"></a>consistent type.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-67532822"></a>
>
> <a id="ref-for-funcdef-min②"></a>
>
> <a id="ref-for-funcdef-max②"></a>
>
> <a id="ref-for-funcdef-clamp③"></a>
>
> <a id="ref-for-propdef-font-size⑤"></a>
>
> [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) can be used to make sure a value doesn’t exceed a "safe" limit: For example, "responsive type" that sets [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) with viewport units might still want a minimum size to ensure readability:
>
> ```css
> .type {
>   /* Set font-size to 10x the average of vw and vh,
>      but don’t let it go below 12px. */
>   font-size: max(10 * (1vw + 1vh) / 2, 12px);
> }
> ```
>
> <a id="ref-for-funcdef-calc①③"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Full math expressions are allowed in each of the arguments; there’s no need to nest a [calc()](#funcdef-calc) inside! You can also provide more than two arguments, if you have multiple constraints to apply.

<a id="ref-for-funcdef-min③"></a>

<a id="ref-for-funcdef-max③"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-funcdef-clamp④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-50cf5aee"></a> An occasional point of confusion when using [min()](#funcdef-min)/[max()](#funcdef-max) is that you use <a id="ref-for-funcdef-max④"></a>max() to impose a minimum value on something (that is, properties like [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) effectively use <a id="ref-for-funcdef-max⑤"></a>max()), and <a id="ref-for-funcdef-min④"></a>min() to impose a maximum value on something; it’s easy to accidentally reach for the opposite function and try to use <a id="ref-for-funcdef-min⑤"></a>min() to add a minimum size. Using [clamp()](#funcdef-clamp) can make the code read more naturally, as the value is nestled between its minimum and maximum:
>
> ```css
> .type {
>   /* Force the font-size to stay between 12px and 100px */
>   font-size: clamp(12px, 10 * (1vw + 1vh) / 2, 100px);
> }
> ```
>
> Or, if you only wanted to impose a minimum size, but allow the font-size to grow as large as it wants:
>
> ```css
> .type {
>   /* Force the font-size to be at least 12px */
>   font-size: clamp(12px, 10 * (1vw + 1vh) / 2, none);
> }
> ```
<a id="ref-for-funcdef-clamp⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that [clamp()](#funcdef-clamp), matching CSS conventions elsewhere, has its minimum value "win" over its maximum value if the two are in the "wrong order". That is, clamp(100px, ..., 50px) will resolve to 100px, exceeding its stated "max" value.
>
> <a id="ref-for-funcdef-clamp⑥"></a>
>
> <a id="ref-for-funcdef-min⑥"></a>
>
> <a id="ref-for-funcdef-max⑥"></a>
>
> If alternate resolution mechanics are desired they can be achieved by combining [clamp()](#funcdef-clamp) with [min()](#funcdef-min) or [max()](#funcdef-max):
>
> To have MAX win over MIN:  
> <a id="ref-for-funcdef-clamp⑦"></a>
>
> clamp(min(MIN, MAX), VAL, MAX). If you want to avoid repeating the MAX calculation, you can just reverse the nesting of functions that [clamp()](#funcdef-clamp) is defined against—​min(MAX, max(MIN, VAL)).
>
> To have MAX and MIN "swap" when they’re in the wrong order:  
> clamp(min(MIN, MAX), VAL, max(MIN, MAX)). Unfortunately, there’s no easy way to do this without repeating the MIN and MAX terms.

<a id="ref-for-funcdef-round"></a>

<a id="ref-for-funcdef-mod"></a>

<a id="ref-for-funcdef-rem"></a>

### <a id="round-func"></a>10.3.  Stepped Value Functions: [round()](#funcdef-round), [mod()](#funcdef-mod), and [rem()](#funcdef-rem)

<a id="ref-for-funcdef-round①"></a>

<a id="ref-for-funcdef-mod①"></a>

<a id="ref-for-funcdef-rem①"></a>

The stepped-value functions, [round()](#funcdef-round), [mod()](#funcdef-mod), and [rem()](#funcdef-rem), all transform a given value according to another "step value", in different ways.

<a id="ref-for-typedef-rounding-strategy"></a>

<a id="ref-for-calc-calculation⑥"></a>

<a id="ref-for-number-value①⑦"></a>

<a id="ref-for-typedef-dimension③"></a>

<a id="ref-for-percentage-value②⑦"></a>

<a id="ref-for-css-consistent-type②"></a>

The <a id="funcdef-round"></a>round([\<rounding-strategy\>](#typedef-rounding-strategy)?, A, B?) function contains an optional rounding strategy, and two [calculations](#calc-calculation) A and B, and returns the value of A, rounded according to the rounding strategy, to the nearest integer multiple of B either above or below A. The argument <a id="ref-for-calc-calculation⑦"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have a [consistent type](#css-consistent-type) or else the function is invalid; the result’s type will be the <a id="ref-for-css-consistent-type③"></a>consistent type.

<a id="ref-for-funcdef-round②"></a>

<a id="ref-for-typedef-rounding-strategy①"></a>

If A is exactly equal to an integer multiple of B, [round()](#funcdef-round) resolves to A exactly (preserving whether A is 0⁻ or 0⁺, if relevant). Otherwise, there are two integer multiples of B that are potentially "closest" to A, <var>lower B</var> which is closer to −∞ and <var>upper B</var> which is closer to +∞. The following <a id="typedef-rounding-strategy"></a>[\<rounding-strategy\>](#typedef-rounding-strategy)s dictate how to choose between them:

<a id="valdef-rounding-strategy-nearest"></a>nearest  
Choose whichever of <var>lower B</var> and <var>upper B</var> that has the smallest absolute difference from A. If both have an equal difference (A is exactly between the two values), choose <var>upper B</var>.

<a id="valdef-rounding-strategy-up"></a>up  
Choose <var>upper B</var>.

<a id="valdef-rounding-strategy-down"></a>down  
Choose <var>lower B</var>.

<a id="valdef-rounding-strategy-to-zero"></a>to-zero  
Choose whichever of <var>lower B</var> and <var>upper B</var> that has the smallest absolute difference from 0.

If <var>lower B</var> would be zero, it is specifically equal to 0⁺; if <var>upper B</var> would be zero, it is specifically equal to 0⁻.

<a id="ref-for-typedef-rounding-strategy②"></a>

<a id="ref-for-valdef-rounding-strategy-nearest"></a>

<a id="ref-for-css-round-to-the-nearest-integer①"></a>

<a id="ref-for-cssnumericvalue-type"></a>

<a id="ref-for-number-value①⑧"></a>

If [\<rounding-strategy\>](#typedef-rounding-strategy) is omitted, it defaults to [nearest](#valdef-rounding-strategy-nearest). (Aka [rounding to the nearest integer](#css-round-to-the-nearest-integer).) If the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of A matches [\<number\>](#number-value), then B may be omitted, and defaults to 1; omitting B is otherwise invalid.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1a929fd0"></a> CSSOM needs to specify how it rounds, and it’s probably good for CSS functions to round the same way by default. What behavior should be used? [\[Issue \#5689\]](https://github.com/w3c/csswg-drafts/issues/5689)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1c1bb50d"></a> Unlike languages like JavaScript which have a natural "precision" to round to (integers), CSS values have no such precision because values can be written in many different compatible units. As such, the precision has to be given explicitly; to round a width to the nearest 50px, one can write round(var(--width), 50px).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: JavaScript and other programming languages sometimes separate out the rounding strategies into separate rounding functions. JS’s `Math.floor()` is equivalent to CSS’s round(down, ...); JS’s `Math.ceil()` is equivalent to CSS’s round(up, ...); JS’s `Math.trunc()` is equivalent to CSS’s round(to-zero, ...); and JS’s `Math.round()` is equivalent to CSS’s round(nearest, ...), or just round(...).

<a id="ref-for-typedef-rounding-strategy③"></a>

<a id="ref-for-propdef-block-step-size"></a>

<a id="ref-for-valdef-rounding-strategy-to-zero"></a>

<a id="ref-for-valdef-rounding-strategy-down"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<rounding-strategy\>](#typedef-rounding-strategy) keywords are the same as the keywords in [block-step-size](https://www.w3.org/TR/css-rhythm-1/#propdef-block-step-size) and have the same behavior. (<a id="ref-for-propdef-block-step-size①"></a>block-step-size just lacks [to-zero](#valdef-rounding-strategy-to-zero); since block sizes are always non-negative, <a id="ref-for-valdef-rounding-strategy-to-zero①"></a>to-zero and [down](#valdef-rounding-strategy-down) would be identical.)

<a id="ref-for-calc-calculation⑧"></a>

<a id="ref-for-number-value①⑨"></a>

<a id="ref-for-typedef-dimension④"></a>

<a id="ref-for-percentage-value②⑧"></a>

<a id="ref-for-determine-the-type-of-a-calculation"></a>

<a id="ref-for-cssnumericvalue-type①"></a>

The modulus functions <a id="funcdef-mod"></a>mod(A, B) and <a id="funcdef-rem"></a>rem(A, B) similarly contain two [calculations](#calc-calculation) A and B, and return the difference between A and the nearest integer multiple of B either above or below A. The argument <a id="ref-for-calc-calculation⑨"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have the <em>same</em> [type](#determine-the-type-of-a-calculation), or else the function is invalid; the result will have the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the arguments.

The two functions are very similar, and in fact return identical results if both arguments are positive or both are negative: the value of the function is equal to the value of A shifted by the integer multiple of B that brings the value <a id="between-zero-and-b"></a>between zero and B. (Specifically, the range includes zero and excludes B. More specifically, if B is positive the range starts at 0⁺, and if B is negative it starts at 0⁻.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1588859b"></a> For example, mod(18px, 5px) resolves to the value 3px, because subtracting 5px \* 3 from 18px yields 3px, which is the only such value between 0px and 3px.
>
> Similarly, mod(-140deg, -90deg) resolves to the value -50deg, because adding -90deg \* 1 to -140deg yields -50deg, which is the only such value between 0deg and -90deg.
>
> <a id="ref-for-funcdef-rem②"></a>
>
> Evaluating either of these examples with [rem()](#funcdef-rem) yields the exact same results.

<a id="ref-for-funcdef-mod②"></a>

<a id="ref-for-between-zero-and-b"></a>

<a id="ref-for-funcdef-rem③"></a>

Their behavior diverges if the A value and the B step are on opposite sides of zero: [mod()](#funcdef-mod) (short for “modulus”) continues to choose the integer multiple of B that puts the value [between zero and B](#between-zero-and-b), as above (guaranteeing that the result will either be zero or share the sign of B, not A), while [rem()](#funcdef-rem) (short for "remainder") chooses the integer multiple of B that puts the value <a id="ref-for-between-zero-and-b①"></a>between zero and -B, avoiding changing the sign of the value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3a2ef59d"></a> For example, mod(-18px, 5px) resolves to the value 2px: adding 5px \* 4 to -18px yields 2px, which is between 0px and 5px.
>
> On the other hand, rem(-18px, 5px) resolves to the value -3px: adding 5px \* 3 to -18px yields -3px, which has the same sign as -18px but is between 0px and -5px.
>
> Similarly, mod(140deg, -90deg) resolves to the value -40deg (adding -90deg \* 2 to 140deg, bringing it to between 0deg and -90deg), but rem(140deg, -90deg) resolves to the value 50deg.

<a id="ref-for-funcdef-mod③"></a>

<a id="ref-for-funcdef-rem④"></a>

When should I choose [mod()](#funcdef-mod) vs [rem()](#funcdef-rem)?

<a id="ref-for-funcdef-mod④"></a>

Typically, users of this operation are in control of the step value (B), and are modifying an unknown value A. As a result, it’s <em>usually</em> more expected that the result is between 0 and B, regardless of A’s sign, meaning [mod()](#funcdef-mod) should be chosen.

For example, if an author wants to know whether a length is an even or odd number of pixels, mod(A, 2px) will return either 0px or 1px (assuming the value is a whole number of pixels to begin with), regardless of the value of a. rem(A, 2px), on the other hand, will return 0px if A is an even number of pixels, but will return <em>either</em> 1px or -1px if it’s odd, depending on whether A is positive or negative.

<a id="ref-for-funcdef-rem⑤"></a>

The opposite situation does sometimes occur, however, and so [rem()](#funcdef-rem) is provided to cater to that. As well, <a id="ref-for-funcdef-rem⑥"></a>rem() is the behavior of JavaScript’s `%` operator, so if an exact match between CSS and JS code is desired, <a id="ref-for-funcdef-rem⑦"></a>rem() can be useful.

<a id="ref-for-funcdef-mod⑤"></a>

<a id="ref-for-funcdef-rem⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [mod()](#funcdef-mod) and [rem()](#funcdef-rem) can also be defined directly in terms of other functions: mod(A, B) is equivalent to calc(A - sign(B)\*round(down, A\*sign(B), B)) (a hacky way to say "round(down) when B is positive, round(up) when B is negative), while rem(A, B) is equivalent to calc(A - round(to-zero, A, B)). (These expressions don’t always handle 0⁺ and 0⁻ correctly, though, because 0⁻ semantics aren’t commutative for addition.)

#### <a id="round-infinities"></a>10.3.1.  Argument Ranges

In round(A, B), if B is 0, the result is NaN. If A and B are both infinite, the result is NaN.

If A is infinite but B is finite, the result is the same infinity.

<a id="ref-for-typedef-rounding-strategy④"></a>

If A is finite but B is infinite, the result depends on the [\<rounding-strategy\>](#typedef-rounding-strategy) and the sign of A:

<a id="ref-for-valdef-rounding-strategy-nearest①"></a>

[nearest](#valdef-rounding-strategy-nearest)

<a id="ref-for-valdef-rounding-strategy-to-zero②"></a>

[to-zero](#valdef-rounding-strategy-to-zero)

If A is positive or 0⁺, return 0⁺. Otherwise, return 0⁻.

<a id="ref-for-valdef-rounding-strategy-up"></a>

[up](#valdef-rounding-strategy-up)

If A is positive (not zero), return +∞. If A is 0⁺, return 0⁺. Otherwise, return 0⁻.

<a id="ref-for-valdef-rounding-strategy-down①"></a>

[down](#valdef-rounding-strategy-down)

If A is negative (not zero), return −∞. If A is 0⁻, return 0⁻. Otherwise, return 0⁺.

In mod(A, B) or rem(A, B), if B is 0, the result is NaN. If A is infinite, the result is NaN.

In mod(A, B) only, if B is infinite and A has opposite sign to B (including an oppositely-signed zero), the result is NaN.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All other "infinite B" cases are valid, and just return A immediately.

<a id="ref-for-funcdef-sin①"></a>

<a id="ref-for-funcdef-cos"></a>

<a id="ref-for-funcdef-tan"></a>

<a id="ref-for-funcdef-asin"></a>

<a id="ref-for-funcdef-acos"></a>

<a id="ref-for-funcdef-atan"></a>

<a id="ref-for-funcdef-atan2"></a>

### <a id="trig-funcs"></a>10.4.  Trigonometric Functions: [sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), and [atan2()](#funcdef-atan2)

<a id="ref-for-funcdef-sin②"></a>

<a id="ref-for-funcdef-cos①"></a>

<a id="ref-for-funcdef-tan①"></a>

<a id="ref-for-funcdef-asin①"></a>

<a id="ref-for-funcdef-acos①"></a>

<a id="ref-for-funcdef-atan①"></a>

<a id="ref-for-funcdef-atan2①"></a>

The trigonometric functions—​[sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), and [atan2()](#funcdef-atan2)—​compute the various basic trigonometric relationships.

<a id="ref-for-calc-calculation①⓪"></a>

<a id="ref-for-number-value②⓪"></a>

<a id="ref-for-angle-value①②"></a>

<a id="ref-for-css-make-a-type-consistent"></a>

<a id="ref-for-funcdef-sin③"></a>

<a id="ref-for-funcdef-cos②"></a>

<a id="ref-for-funcdef-tan②"></a>

<a id="ref-for-math-function⑧"></a>

The <a id="funcdef-sin"></a>sin(A), <a id="funcdef-cos"></a>cos(A), and <a id="funcdef-tan"></a>tan(A) functions all contain a single [calculation](#calc-calculation) which must resolve to either a [\<number\>](#number-value) or an [\<angle\>](#angle-value), and compute their corresponding function by interpreting the result of their argument as radians. (That is, sin(45deg), sin(.125turn), and sin(3.14159 / 4) all represent the same value, approximately .707.) They all represent a <a id="ref-for-number-value②①"></a>\<number\>, with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation①①"></a>calculation’s type. [sin()](#funcdef-sin) and [cos()](#funcdef-cos) will always return a number between −1 and 1, while [tan()](#funcdef-tan) can return any number between −∞ and +∞. (See [§ 10.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle ∞.)

<a id="ref-for-calc-calculation①②"></a>

<a id="ref-for-number-value②②"></a>

<a id="ref-for-angle-value①③"></a>

<a id="ref-for-css-make-a-type-consistent①"></a>

<a id="ref-for-funcdef-asin②"></a>

<a id="ref-for-funcdef-acos②"></a>

<a id="ref-for-funcdef-atan②"></a>

The <a id="funcdef-asin"></a>asin(A), <a id="funcdef-acos"></a>acos(A), and <a id="funcdef-atan"></a>atan(A) functions are the "arc" or "inverse" trigonometric functions, representing the inverse function to their corresponding "normal" trig functions. All of them contain a single [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and compute their corresponding function, interpreting their result as a number of radians, representing an [\<angle\>](#angle-value) with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation①③"></a>calculation’s type. The angle returned by [asin()](#funcdef-asin) must be normalized to the range \[-90deg, 90deg\]; the angle returned by [acos()](#funcdef-acos) to the range \[0deg, 180deg\]; and the angle returned by [atan()](#funcdef-atan) to the range \[-90deg, 90deg\].

<a id="ref-for-calc-calculation①④"></a>

<a id="ref-for-number-value②③"></a>

<a id="ref-for-typedef-dimension⑤"></a>

<a id="ref-for-percentage-value②⑨"></a>

<a id="ref-for-css-consistent-type④"></a>

<a id="ref-for-angle-value①④"></a>

<a id="ref-for-css-make-a-type-consistent②"></a>

The <a id="funcdef-atan2"></a>atan2(A, B) function contains two comma-separated [calculations](#calc-calculation), A and B. A and B can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have a [consistent type](#css-consistent-type) or else the function is invalid. The function returns the [\<angle\>](#angle-value) between the positive X-axis and the point (B,A), with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation①⑤"></a>calculation’s type. The returned angle must be normalized to the interval (-180deg, 180deg\] (that is, greater than -180deg, and less than or equal to 180deg).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: atan2(Y, X) is <em>generally</em> equivalent to atan(Y / X), but it gives a better answer when the point in question may include negative components. atan2(1, -1), corresponding to the point (-1, 1), returns 135deg, distinct from atan2(-1, 1), corresponding to the point (1, -1), which returns -45deg. In contrast, atan(1 / -1) and atan(-1 / 1) both return-45deg, because the internal calculation resolves to -1 for both.

#### <a id="trig-infinities"></a>10.4.1.  Argument Ranges

<a id="ref-for-math-function⑨"></a>

In sin(A), cos(A), or tan(A), if A is infinite, the result is NaN. (See [§ 10.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle NaN.)

In sin(A) or tan(A), if A is 0⁻, the result is 0⁻.

<a id="ref-for-implementation-defined④"></a>

<a id="ref-for-funcdef-tan③"></a>

In tan(A), if A is one of the asymptote values (such as 90deg, 270deg, etc), the numeric result is [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined). If an implementation is capable of exactly representing these inputs, it <em>should</em> return +∞ for the asymptotes at `90deg + N*360deg`, and −∞ for the asymptotes at `-90deg + N*360deg`, but implementations are not required to be able to exactly represent these inputs (and if they can’t, will return whatever the correct numeric answer is for the closest approximation to the input they are capable of representing). Authors <em>must not</em> rely on [tan()](#funcdef-tan) returning any particular value for these inputs.

<a id="ref-for-implementation-defined⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why are these [implementation-defined](https://infra.spec.whatwg.org/#implementation-defined)?
>
> The tangent function is <em>discontinuous</em> at its asymptotes: it approaches infinity from one side <em>and</em> negative infinity from the other side, and isn’t defined at the exact values of the asymptote.
>
> Further, whether or not the asymptotic values are exactly representable in implementations depends on how they internally store and manipulate angles; when written in degrees the values are simple (90deg, etc), but in radians the values are transcendental (pi / 2, etc) and cannot be exactly represented. So, even defining a specific behavior for these values is difficult; if an implementation uses radians internally, it would have to do some fuzzy matching to return the defined value when the input is <em>sufficiently close</em> to the asymptote.
>
> The other major language for the Web, JavaScript, exposes these functions as taking radians only, so it can’t hit the exact asymptotes either (and this true for most other computer languages, too). Authors writing code in JS, then, can’t rely on any specific behavior for these values either, and it’s unlikely that their needs in CSS are significantly different.
>
> <a id="ref-for-funcdef-atan③"></a>
>
> The suggested behavior for implementations that can exactly represent the asymptote values preserves round-tripping with the [atan()](#funcdef-atan) function: tan(atan(X)) and atan(tan(X)) will both return (approximately) X for all possible X values, given this definition. It also means that within the supported output range of <a id="ref-for-funcdef-atan④"></a>atan(), the function is continuous.

In asin(A) or acos(A), if A is less than -1 or greater than 1, the result is NaN.

In acos(A), if A is exactly 1, the result is 0.

In asin(A) or atan(A), if A is 0⁻, the result is 0⁻.

In atan(A), if A is +∞, the result is 90deg; if A is −∞, the result is -90deg.

In atan2(Y, X), the following table gives the results for all unusual argument combinations:

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (data cell; column span 2):</strong>

<strong>Column 3 (header cell; column span 6):</strong>

X

<strong>Row 2</strong>

<strong>Column 1 (data cell; column span 2):</strong>

<strong>Column 3 (header cell):</strong>

−∞

<strong>Column 4 (header cell):</strong>

-finite

<strong>Column 5 (header cell):</strong>

0⁻

<strong>Column 6 (header cell):</strong>

0⁺

<strong>Column 7 (header cell):</strong>

+finite

<strong>Column 8 (header cell):</strong>

+∞

<strong>Row 3</strong>

<strong>Column 1 (header cell; row span 6):</strong>

Y

<strong>Column 2 (header cell):</strong>

−∞

<strong>Column 3 (data cell):</strong>

-135deg

<strong>Column 4 (data cell):</strong>

-90deg

<strong>Column 5 (data cell):</strong>

-90deg

<strong>Column 6 (data cell):</strong>

-90deg

<strong>Column 7 (data cell):</strong>

-90deg

<strong>Column 8 (data cell):</strong>

-45deg

<strong>Row 4</strong>

<strong>Column 2 (header cell):</strong>

-finite

<strong>Column 3 (data cell):</strong>

-180deg

<strong>Column 4 (data cell):</strong>

(normal)

<strong>Column 5 (data cell):</strong>

-90deg

<strong>Column 6 (data cell):</strong>

-90deg

<strong>Column 7 (data cell):</strong>

(normal)

<strong>Column 8 (data cell):</strong>

0⁻deg

<strong>Row 5</strong>

<strong>Column 2 (header cell):</strong>

0⁻

<strong>Column 3 (data cell):</strong>

-180deg

<strong>Column 4 (data cell):</strong>

-180deg

<strong>Column 5 (data cell):</strong>

-180deg

<strong>Column 6 (data cell):</strong>

0⁻deg

<strong>Column 7 (data cell):</strong>

0⁻deg

<strong>Column 8 (data cell):</strong>

0⁻deg

<strong>Row 6</strong>

<strong>Column 2 (header cell):</strong>

0⁺

<strong>Column 3 (data cell):</strong>

180deg

<strong>Column 4 (data cell):</strong>

180deg

<strong>Column 5 (data cell):</strong>

180deg

<strong>Column 6 (data cell):</strong>

0⁺deg

<strong>Column 7 (data cell):</strong>

0⁺deg

<strong>Column 8 (data cell):</strong>

0⁺deg

<strong>Row 7</strong>

<strong>Column 2 (header cell):</strong>

+finite

<strong>Column 3 (data cell):</strong>

180deg

<strong>Column 4 (data cell):</strong>

(normal)

<strong>Column 5 (data cell):</strong>

90deg

<strong>Column 6 (data cell):</strong>

90deg

<strong>Column 7 (data cell):</strong>

(normal)

<strong>Column 8 (data cell):</strong>

0⁺deg

<strong>Row 8</strong>

<strong>Column 2 (header cell):</strong>

+∞

<strong>Column 3 (data cell):</strong>

135deg

<strong>Column 4 (data cell):</strong>

90deg

<strong>Column 5 (data cell):</strong>

90deg

<strong>Column 6 (data cell):</strong>

90deg

<strong>Column 7 (data cell):</strong>

90deg

<strong>Column 8 (data cell):</strong>

45deg

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All of these behaviors are intended to match the "standard" definitions of these functions as implemented by most programming languages, in particular as implemented in JS.

<a id="ref-for-funcdef-pow"></a>

<a id="ref-for-funcdef-sqrt"></a>

<a id="ref-for-funcdef-hypot"></a>

<a id="ref-for-funcdef-log"></a>

<a id="ref-for-funcdef-exp"></a>

### <a id="exponent-funcs"></a>10.5.  Exponential Functions: [pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), [exp()](#funcdef-exp)

<a id="ref-for-funcdef-pow①"></a>

<a id="ref-for-funcdef-sqrt①"></a>

<a id="ref-for-funcdef-hypot①"></a>

<a id="ref-for-funcdef-log①"></a>

<a id="ref-for-funcdef-exp①"></a>

The exponential functions—​[pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), and [exp()](#funcdef-exp)—​compute various exponential functions with their arguments.

<a id="ref-for-calc-calculation①⑥"></a>

<a id="ref-for-number-value②④"></a>

<a id="ref-for-css-consistent-type⑤"></a>

The <a id="funcdef-pow"></a>pow(A, B) function contains two comma-separated [calculations](#calc-calculation) A and B, both of which must resolve to [\<number\>](#number-value)s, and returns the result of raising A to the power of B, returning the value as a <a id="ref-for-number-value②⑤"></a>\<number\>. The input <a id="ref-for-calc-calculation①⑦"></a>calculations must have a [consistent type](#css-consistent-type) or else the function is invalid; the result’s type will be the <a id="ref-for-css-consistent-type⑥"></a>consistent type.

<a id="ref-for-calc-calculation①⑧"></a>

<a id="ref-for-number-value②⑥"></a>

<a id="ref-for-css-make-a-type-consistent③"></a>

<a id="ref-for-funcdef-sqrt②"></a>

The <a id="funcdef-sqrt"></a>sqrt(A) function contains a single [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and returns the square root of the value as a <a id="ref-for-number-value②⑦"></a>\<number\>, with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation①⑨"></a>calculation’s type. (sqrt(X) and pow(X, .5) are basically equivalent, differing only in some error-handling; [sqrt()](#funcdef-sqrt) is a common enough function that it is provided as a convenience.)

<a id="ref-for-calc-calculation②⓪"></a>

<a id="ref-for-number-value②⑧"></a>

<a id="ref-for-typedef-dimension⑥"></a>

<a id="ref-for-percentage-value③⓪"></a>

<a id="ref-for-css-consistent-type⑦"></a>

The <a id="funcdef-hypot"></a>hypot(A, …) function contains one or more comma-separated [calculations](#calc-calculation), and returns the length of an N-dimensional vector with components equal to each of the <a id="ref-for-calc-calculation②①"></a>calculations. (That is, the square root of the sum of the squares of its arguments.) The argument <a id="ref-for-calc-calculation②②"></a>calculations can resolve to any [\<number\>](#number-value), [\<dimension\>](#typedef-dimension), or [\<percentage\>](#percentage-value), but must have a [consistent type](#css-consistent-type) or else the function is invalid; the result’s type will be the <a id="ref-for-css-consistent-type⑧"></a>consistent type.

<a id="ref-for-funcdef-hypot②"></a>

<a id="ref-for-funcdef-pow②"></a>

<a id="ref-for-funcdef-sqrt③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Why does [hypot()](#funcdef-hypot) allow dimensions (values with units), but [pow()](#funcdef-pow) and [sqrt()](#funcdef-sqrt) only work on numbers?
>
> You are allowed to write expressions like hypot(30px, 40px), which resolves to 50px, but you aren’t allowed to write the expression sqrt(pow(30px, 2) + pow(40px, 2)), despite the two being equivalent in most mathematical systems.
>
> There are two reasons for this: numeric precision in the exponents, and clashing expectations from authors.
>
> <a id="ref-for-cssnumericvalue-type②"></a>
>
> <a id="ref-for-cssnumericvalue-match"></a>
>
> <a id="ref-for-length-value②⑤"></a>
>
> First, numerical precision. For a [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) to [match](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) a CSS production like [\<length\>](#length-value), it needs to have a single unit with its exponent set to exactly 1. Theoretically, expressions like pow(pow(30px, 3), 1/3) should result in exactly that: the inner pow(30px, 3) would resolve to a value of 27000 with a <a id="ref-for-cssnumericvalue-type③"></a>type of «\[ "length" → 3 \]» (aka <a id="ref-for-length-value②⑥"></a>\<length\>³), and then the pow(X, 1/3) would cube-root the value back down to 30 and multiply the exponent by 1/3, giving «\[ "length" → 1 \]», which <a id="ref-for-cssnumericvalue-match①"></a>matches <a id="ref-for-length-value②⑦"></a>\<length\>.
>
> <a id="ref-for-math-function①⓪"></a>
>
> In the realm of pure mathematics, that’s guaranteed to work out; in the real-world of computers using binary floating-point arithmetic, in some cases the powers might not exactly cancel out, leaving you with an invalid [math function](#math-function) for confusing, hard-to-track-down reasons. (For a JS example, evaluate `Math.pow(Math.pow(30, 10/3), .1+.1+.1)`; the result is not exactly 30, because `.1+.1+.1` is not exactly 3/10. Instead, `(10/3) * (.1 + .1 + .1)` is <em>slightly greater</em> than 1.)
>
> <a id="ref-for-length-value②⑧"></a>
>
> Requiring authors to cast their value down into a number, do all the math on the raw number, then finally send it back to the desired unit, while inconvenient, ensures that numerical precision won’t bite anyone: calc(pow(pow(30px / 1px, 3), 1/3) \* 1px) is guaranteed to resolve to a [\<length\>](#length-value), with a value that, if not exactly 30, is at least very close to 30, even if numerical precision actually prevents the powers from exactly canceling.
>
> <a id="ref-for-canonical-unit⑧"></a>
>
> Second, clashing expectations. It’s not uncommon for authors to expect pow(30px, 2) to result in 900px (such as in [this Sass issue](https://github.com/sass/sass/issues/684)); that is, just squaring the numerical value and leaving the unit alone. This, however, means the result is dependent on what unit you’re expressing the argument in; if 1em is 16px, then pow(1em, 2) would give 1em, while pow(16px, 2) would give 256px, or 16em, which are very different values for what should otherwise be identical input arguments! This sort of input dependency is troublesome for CSS, which generally allows values to be [canonicalized](#canonical-unit) freely; it also makes more complex expressions like pow(2em + 10px, 2) difficult to interpret.
>
> Again, requiring authors to cast their value down into a number and then back up again into the desired unit sidesteps these issues; pow(30, 2) is indeed 900, and the author can interpret that however they wish.
>
> ------------------------------------------------------------------------
>
> <a id="ref-for-funcdef-hypot③"></a>
>
> On the other hand, [hypot()](#funcdef-hypot) doesn’t suffer from these problems. Numerical precision in units isn’t a concern, as the inputs and output all have the same type. The result isn’t unit-dependent, either, due to the nature of the operation; hypot(3em, 4em) and hypot(48px, 64px) both result in the same length when 1em equals 16px: 5em or 80px. Thus it’s fine to let author use dimensions directly in <a id="ref-for-funcdef-hypot④"></a>hypot().

<a id="ref-for-calc-calculation②③"></a>

<a id="ref-for-number-value②⑨"></a>

<a id="ref-for-css-make-a-type-consistent④"></a>

The <a id="funcdef-log"></a>log(A, B?) function contains one or two [calculations](#calc-calculation) (representing the value to be logarithmed, and the base of the logarithm, defaulting to e), which must resolve to [\<number\>](#number-value)s, and returns the logarithm base B of the value A, as a <a id="ref-for-number-value③⓪"></a>\<number\> with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation②④"></a>calculation’s type.

<a id="ref-for-calc-calculation②⑤"></a>

<a id="ref-for-number-value③①"></a>

<a id="ref-for-css-make-a-type-consistent⑤"></a>

The <a id="funcdef-exp"></a>exp(A) function contains one [calculation](#calc-calculation) which must resolve to a [\<number\>](#number-value), and returns the same value as pow(e, A) as a <a id="ref-for-number-value③②"></a>\<number\> with the return type [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation②⑥"></a>calculation’s type.

<a id="ref-for-funcdef-pow③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bebe620e"></a> The [pow()](#funcdef-pow) function can be useful for strategies like [CSS Modular Scale](https://www.modularscale.com/), which relates all the font-sizes on a page to each other by a fixed ratio.
>
> These sizes can be easily written into custom properties like:
>
> ```css
> :root {
>   --h6: calc(1rem * pow(1.5, -1));
>   --h5: calc(1rem * pow(1.5, 0));
>   --h4: calc(1rem * pow(1.5, 1));
>   --h3: calc(1rem * pow(1.5, 2));
>   --h2: calc(1rem * pow(1.5, 3));
>   --h1: calc(1rem * pow(1.5, 4));
> }
> ```
>
> ...rather than writing out the values in pre-calculated numbers like 5.0625rem (what calc(1rem \* pow(1.5, 4)) resolves to) which have less clear provenance when encountered in a stylesheet.

<a id="ref-for-funcdef-hypot⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d7f26d9d"></a> With a single argument, [hypot()](#funcdef-hypot) gives the absolute value of its input; hypot(2em) and hypot(-2em) both resolve to 2em.
>
> With more arguments, it gives the size of the main diagonal of a box whose side lengths are given by the arguments. This can be useful for transform-related things, giving the distance that an element will actually travel when it’s translated by a particular X, Y, and Z amount.
>
> For example, hypot(30px, 40px) resolves to 50px, which is indeed the distance between an element’s starting and ending positions when it’s translated by a translate(30px, 40px) transform. If an author wanted elements to get smaller as they moved further away from their starting point (drawing some sort of word cloud, for example), they could then use this distance in their scaling factor calculations.

<a id="ref-for-funcdef-log②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d939ff73"></a> With a single argument, [log()](#funcdef-log) provides the “natural log” of its argument, or the log base e, same as JavaScript.
>
> If one instead wants log base 10 (to, for example, count the number of digits in a value) or log base 2 (counting the number of bits in a value), log(X, 10) or log(X, 2) provide those values.

#### <a id="exponent-infinities"></a>10.5.1.  Argument Ranges

In pow(A, B), if A is negative and finite, and B is finite, B must be an integer, or else the result is NaN.

If A or B are infinite or 0, the following tables give the results:

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (data cell):</strong>

<strong>Column 2 (header cell):</strong>

A is −∞

<strong>Column 3 (header cell):</strong>

A is 0⁻

<strong>Column 4 (header cell):</strong>

A is 0⁺

<strong>Column 5 (header cell):</strong>

A is +∞

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

B is −finite

<strong>Column 2 (data cell):</strong>

0⁻ if B is an odd integer, 0⁺ otherwise

<strong>Column 3 (data cell):</strong>

−∞ if B is an odd integer, +∞ otherwise

<strong>Column 4 (data cell):</strong>

+∞

<strong>Column 5 (data cell):</strong>

0⁺

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

B is 0

<strong>Column 2 (data cell; column span 4):</strong>

always 1

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

B is +finite

<strong>Column 2 (data cell):</strong>

−∞ if B is an odd integer, +∞ otherwise

<strong>Column 3 (data cell):</strong>

0⁻ if B is an odd integer, 0⁺ otherwise

<strong>Column 4 (data cell):</strong>

0⁺

<strong>Column 5 (data cell):</strong>

+∞

|         | A is \< -1   | A is -1       | -1 \< A \< 1 | A is 1        | A is \> 1    |
|---------|--------------|---------------|--------------|---------------|--------------|
| B is +∞ | result is +∞ | result is NaN | result is 0⁺ | result is NaN | result is +∞ |
| B is −∞ | result is 0⁺ | result is NaN | result is +∞ | result is NaN | result is 0⁺ |

In sqrt(A), if A is +∞, the result is +∞. If A is 0⁻, the result is 0⁻. If A is less than 0, the result is NaN.

In hypot(A, …), if any of the inputs are infinite, the result is +∞.

In log(A, B), if B is 1 or negative, <strong data-conversion-semantic="note">Note:</strong> B values <em>between</em> 0 and 1, or greater than 1, are valid. the result is NaN. If A is negative, the result is NaN. If A is 0⁺ or 0⁻, the result is −∞. If A is 1, the result is 0⁺. If A is +∞, the result is +∞.

In exp(A), if A is +∞, the result is +∞. If A is −∞, the result is 0⁺.

<a id="ref-for-math-function①①"></a>

(See [§ 10.9 Type Checking](#calc-type-checking) for details on how [math functions](#math-function) handle NaN and infinities.)

> <strong data-conversion-semantic="note">Note</strong>
>
> All of these behaviors are intended to match the "standard" definitions of these functions as implemented by most programming languages, in particular as implemented in JS.
>
> The only divergences from the behavior of the equivalent JS functions are that NaN is "infectious" in <em>every</em> function, forcing the function to return NaN if any argument calculation is NaN.
>
> Details of the JS Behavior
>
> There are two cases in JS where a NaN is not "infectious" to the math function it finds itself in:
>
> - `Math.hypot(Infinity, NaN)` will return `Infinity`.
>
> - `Math.pow(NaN, 0)` will return `1`.
>
> The logic appears to be that, if you replace the NaN with <em>any</em> Number, the return value will be the same. However, this logic is not applied consistently to the `Math` functions: `Math.max(Infinity, NaN)` returns `NaN`, not `Infinity`; the same is true of `Math.min(-Infinity, NaN)`.
>
> <a id="ref-for-calc-calculation②⑦"></a>
>
> Because this is an error corner case, JS isn’t consistent on the matter, and NaN recognition/handling of [calculations](#calc-calculation) is likely done at a higher CSS level rather than in the internal math functions anyway, consistency in CSS was chosen to be more important, so all functions were defined to have "infectious" NaN.

<a id="ref-for-funcdef-abs"></a>

<a id="ref-for-funcdef-sign"></a>

### <a id="sign-funcs"></a>10.6.  Sign-Related Functions: [abs()](#funcdef-abs), [sign()](#funcdef-sign)

<a id="ref-for-funcdef-abs①"></a>

<a id="ref-for-funcdef-sign①"></a>

The sign-related functions—​[abs()](#funcdef-abs) and [sign()](#funcdef-sign)—​compute various functions related to the sign of their argument.

<a id="ref-for-calc-calculation②⑧"></a>

<a id="ref-for-cssnumericvalue-type④"></a>

The <a id="funcdef-abs"></a>abs(A) function contains one [calculation](#calc-calculation) A, and returns the absolute value of A, as the same [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) as the input: if A’s numeric value is positive or 0⁺, just A again; otherwise -1 \* A.

<a id="ref-for-calc-calculation②⑨"></a>

<a id="ref-for-number-value③③"></a>

<a id="ref-for-css-make-a-type-consistent⑥"></a>

The <a id="funcdef-sign"></a>sign(A) function contains one [calculation](#calc-calculation) A, and returns -1 if A’s numeric value is negative, +1 if A’s numeric value is positive, 0⁺ if A’s numeric value is 0⁺, and 0⁻ if A’s numeric value is 0⁻. The return type is a [\<number\>](#number-value), [made consistent](#css-make-a-type-consistent) with the input <a id="ref-for-calc-calculation③⓪"></a>calculation’s type.

<a id="ref-for-propdef-background-position②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Both of these functions operate on the fully simplified/resolved form of their arguments, which may give unintuitive results at first glance. In particular, an expression like 10% might be positive <em>or</em> negative once it’s resolved, depending on what value it’s resolved against. For example, in [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) positive percentages resolve to a negative length, and vice versa, if the background image is larger than the background area. Thus sign(10%) might return 1 <em>or</em> -1, depending on how the percentage is resolved! (Or even 0, if it’s resolved against a zero length.)

### <a id="calc-keywords"></a>10.7.  Numeric Keywords

<a id="ref-for-calc-calculation③①"></a>

<a id="ref-for-determine-the-type-of-a-calculation①"></a>

Keywords in [calculations](#calc-calculation) provide access to values that are difficult or impossible to represent as literals. Each keyword defines its value, its [type](#determine-the-type-of-a-calculation), and when it can be resolved.

<a id="ref-for-valdef-calc-e"></a>

<a id="ref-for-valdef-calc-pi"></a>

#### <a id="calc-constants"></a>10.7.1.  Numeric Constants: [e](#valdef-calc-e), [pi](#valdef-calc-pi)

While the trigonometric and exponential functions handle many complex numeric operations, some reasonable calculations must be put together more manually, and many times these include well-known constants, such as <i>e</i> and <i>π</i>.

Rather than require authors to manually type out several digits of these constants, a few of them are provided directly:

<a id="valdef-calc-e"></a>e  
the base of the natural logarithm, approximately equal to 2.7182818284590452354.

<a id="valdef-calc-pi"></a>pi  
the ratio of a circle’s circumference to its diameter, approximately equal to 3.1415926535897932.

<a id="ref-for-number-value③④"></a>

Both of these keywords are [\<number\>](#number-value)s, and resolve at parse time.

<a id="ref-for-propdef-animation-name①"></a>

<a id="ref-for-propdef-line-height①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These keywords are only usable within a calculation, such as calc(pow(e, pi) - pi), or min(pi, 5, e). If used outside of a calculation, they’re treated like any other keyword: [animation-name: pi;](https://www.w3.org/TR/css-animations-1/#propdef-animation-name) refers to an animation named "pi"; [line-height: e;](https://drafts.csswg.org/css2/#propdef-line-height) is invalid (<em>not</em> similar to <a id="ref-for-propdef-line-height①①"></a>line-height: 2.7, but <a id="ref-for-propdef-line-height①②"></a>line-height: calc(e); is).

<a id="ref-for-valdef-calc-infinity"></a>

<a id="ref-for-valdef-calc--infinity"></a>

<a id="ref-for-valdef-calc-nan"></a>

#### <a id="calc-error-constants"></a>10.7.2.  Degenerate Numeric Constants: [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), [NaN](#valdef-calc-nan)

<a id="ref-for-calc-calculation③②"></a>

<a id="ref-for-css-infinity"></a>

<a id="ref-for-css-nan"></a>

When a [calculation](#calc-calculation) or a subtree of a <a id="ref-for-calc-calculation③③"></a>calculation becomes [infinite](#css-infinity) or [NaN](#css-nan), representing it with a numeric value is no longer possible. To aid in serialization of these degenerate values, the following additional math constants are defined:

<a id="valdef-calc-infinity"></a>infinity  
the value positive infinity (+∞)

<a id="valdef-calc--infinity"></a>-infinity  
the value negative infinity (−∞)

<a id="valdef-calc-nan"></a>NaN  
the value NaN

<a id="ref-for-number-value③⑤"></a>

All of these keywords are [\<number\>](#number-value)s, and resolve at parse time.

<a id="ref-for-ascii-case-insensitive⑥"></a>

<a id="ref-for-valdef-calc-nan①"></a>

As usual for CSS keywords, these are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). <strong data-conversion-semantic="note">Note:</strong> Thus, calc(InFiNiTy) is perfectly valid. However, [NaN](#valdef-calc-nan) must be serialized with this canonical casing.

<a id="ref-for-number-value③⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As these keywords are [\<number\>](#number-value)s, to get an infinite length, for example, requires an expression like calc(infinity \* 1px).

<a id="ref-for-valdef-calc-infinity①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These constants are defined <em>mostly</em> to make serialization of infinite/NaN values simpler and more obvious, but <em>can</em> be used to indicate a "largest possible value", since an infinite value gets clamped to the allowed range. It’s rare for this to be reasonable, but when it is, using [infinity](#valdef-calc-infinity) is clearer in its intent than just putting an enormous number in one’s stylesheet.

#### <a id="calc-variables"></a>10.7.3.  Numeric Variables

<a id="ref-for-calc-calculation③④"></a>

<a id="ref-for-relative-color"></a>

<a id="ref-for-number-value③⑦"></a>

Other specifications can define additional keywords which are usable in [calculations](#calc-calculation) in certain contexts. For example, [relative color](https://www.w3.org/TR/css-color-5/#relative-color) syntax defines a number of color-channel keywords representing the value of each color channel as a [\<number\>](#number-value).

Each specifications defining such keywords must define for each keyword:

- its value

- <a id="ref-for-determine-the-type-of-a-calculation②"></a>

  <a id="ref-for-number-value③⑧"></a>

  <a id="ref-for-length-value②⑨"></a>

  its [type](#determine-the-type-of-a-calculation) ([\<number\>](#number-value), [\<length\>](#length-value), etc)

- when it resolves (parse time, computed-value time, or used-value time)

### <a id="calc-syntax"></a>10.8.  Syntax

<a id="ref-for-math-function①②"></a>

The syntax of a [math function](#math-function) is:

<a id="ref-for-funcdef-calc①④"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="ref-for-funcdef-min⑦"></a>

<a id="ref-for-typedef-calc-sum②"></a>

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-funcdef-max⑦"></a>

<a id="ref-for-typedef-calc-sum③"></a>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-funcdef-clamp⑧"></a>

<a id="ref-for-typedef-calc-sum④"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-calc-sum⑤"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-typedef-calc-sum⑥"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-funcdef-round③"></a>

<a id="ref-for-typedef-rounding-strategy⑤"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-typedef-calc-sum⑦"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-typedef-calc-sum⑧"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-funcdef-mod⑥"></a>

<a id="ref-for-typedef-calc-sum⑨"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-typedef-calc-sum①⓪"></a>

<a id="ref-for-funcdef-rem⑨"></a>

<a id="ref-for-typedef-calc-sum①①"></a>

<a id="ref-for-comb-comma⑧"></a>

<a id="ref-for-typedef-calc-sum①②"></a>

<a id="ref-for-funcdef-sin④"></a>

<a id="ref-for-typedef-calc-sum①③"></a>

<a id="ref-for-funcdef-cos③"></a>

<a id="ref-for-typedef-calc-sum①④"></a>

<a id="ref-for-funcdef-tan④"></a>

<a id="ref-for-typedef-calc-sum①⑤"></a>

<a id="ref-for-funcdef-asin③"></a>

<a id="ref-for-typedef-calc-sum①⑥"></a>

<a id="ref-for-funcdef-acos③"></a>

<a id="ref-for-typedef-calc-sum①⑦"></a>

<a id="ref-for-funcdef-atan⑤"></a>

<a id="ref-for-typedef-calc-sum①⑧"></a>

<a id="ref-for-funcdef-atan2②"></a>

<a id="ref-for-typedef-calc-sum①⑨"></a>

<a id="ref-for-comb-comma⑨"></a>

<a id="ref-for-typedef-calc-sum②⓪"></a>

<a id="ref-for-funcdef-pow④"></a>

<a id="ref-for-typedef-calc-sum②①"></a>

<a id="ref-for-comb-comma①⓪"></a>

<a id="ref-for-typedef-calc-sum②②"></a>

<a id="ref-for-funcdef-sqrt④"></a>

<a id="ref-for-typedef-calc-sum②③"></a>

<a id="ref-for-funcdef-hypot⑥"></a>

<a id="ref-for-typedef-calc-sum②④"></a>

<a id="ref-for-mult-comma④"></a>

<a id="ref-for-funcdef-log③"></a>

<a id="ref-for-typedef-calc-sum②⑤"></a>

<a id="ref-for-comb-comma①①"></a>

<a id="ref-for-typedef-calc-sum②⑥"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-funcdef-exp②"></a>

<a id="ref-for-typedef-calc-sum②⑦"></a>

<a id="ref-for-funcdef-abs②"></a>

<a id="ref-for-typedef-calc-sum②⑧"></a>

<a id="ref-for-funcdef-sign②"></a>

<a id="ref-for-typedef-calc-sum②⑨"></a>

<a id="typedef-calc-sum"></a>

<a id="ref-for-typedef-calc-product"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-typedef-calc-product①"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="typedef-calc-product"></a>

<a id="ref-for-typedef-calc-value"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-typedef-calc-value①"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="typedef-calc-value"></a>

<a id="ref-for-number-value③⑨"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-typedef-dimension⑦"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-percentage-value③①"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-typedef-calc-keyword"></a>

<a id="ref-for-comb-one③⑥"></a>

<a id="ref-for-typedef-calc-sum③⓪"></a>

<a id="typedef-calc-keyword"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-comb-one③⑨"></a>

<a id="ref-for-comb-one④⓪"></a>

<a id="ref-for-typedef-rounding-strategy⑥"></a>

<a id="ref-for-comb-one④①"></a>

<a id="ref-for-comb-one④②"></a>

<a id="ref-for-comb-one④③"></a>

```text
<calc()>  = calc( <calc-sum> )
<min()>   = min( <calc-sum># )
<max()>   = max( <calc-sum># )
<clamp()> = clamp( [ <calc-sum> | none ], <calc-sum>, [ <calc-sum> | none ] )
<round()> = round( <rounding-strategy>?, <calc-sum>, <calc-sum>? )
<mod()>   = mod( <calc-sum>, <calc-sum> )
<rem()>   = rem( <calc-sum>, <calc-sum> )
<sin()>   = sin( <calc-sum> )
<cos()>   = cos( <calc-sum> )
<tan()>   = tan( <calc-sum> )
<asin()>  = asin( <calc-sum> )
<acos()>  = acos( <calc-sum> )
<atan()>  = atan( <calc-sum> )
<atan2()> = atan2( <calc-sum>, <calc-sum> )
<pow()>   = pow( <calc-sum>, <calc-sum> )
<sqrt()>  = sqrt( <calc-sum> )
<hypot()> = hypot( <calc-sum># )
<log()>   = log( <calc-sum>, <calc-sum>? )
<exp()>   = exp( <calc-sum> )
<abs()>   = abs( <calc-sum> )
<sign()>  = sign( <calc-sum> )
<calc-sum> = <calc-product> [ [ '+' | '-' ] <calc-product> ]*
<calc-product> = <calc-value> [ [ '*' | '/' ] <calc-value> ]*
<calc-value> = <number> | <dimension> | <percentage> |
               <calc-keyword> | ( <calc-sum> )
<calc-keyword> = e | pi | infinity | -infinity | NaN
<rounding-strategy> = nearest | up | down | to-zero
```
<a id="ref-for-typedef-calc-keyword①"></a>

<a id="ref-for-relative-color①"></a>

In some contexts, additional [\<calc-keyword\>](#typedef-calc-keyword) values can be defined to be valid. (For example, in [relative color](https://www.w3.org/TR/css-color-5/#relative-color) syntax, appropriate channel keywords are allowed.)

<a id="ref-for-whitespace"></a>

In addition, [whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace) is required on both sides of the + and - operators. (The \* and / operators can be used without white space around them.)

<a id="ref-for-typedef-calc-sum③①"></a>

Several of the math functions above have additional constraints on what their [\<calc-sum\>](#typedef-calc-sum) arguments can contain. Check the definitions of the individual functions for details.

<a id="ref-for-calc-calculation③⑤"></a>

<a id="ref-for-typedef-calc-value②"></a>

<a id="ref-for-funcdef-min⑧"></a>

UAs must support [calculations](#calc-calculation) of at least 32 [\<calc-value\>](#typedef-calc-value) terms and at least 32 levels of nesting (parentheses and/or functions). For functions that support an arbitrary number of arguments (such as [min()](#funcdef-min)), it must also support at least 32 arguments. If a <a id="ref-for-calc-calculation③⑥"></a>calculation contains more than the supported number of terms, arguments, or nesting it must be treated as if it were invalid.

### <a id="calc-type-checking"></a>10.9.  Type Checking

<a id="ref-for-math-function①③"></a>

<a id="ref-for-length-value③⓪"></a>

<a id="ref-for-number-value④⓪"></a>

<a id="ref-for-calc-calculation③⑦"></a>

A [math function](#math-function) can be many possible types, such as [\<length\>](#length-value), [\<number\>](#number-value), etc., depending on the [calculations](#calc-calculation) it contains, as defined below. It can be used anywhere a value of that type is allowed.

<a id="ref-for-propdef-width⑧"></a>

<a id="ref-for-length-value③①"></a>

<a id="ref-for-math-function①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-40312766"></a> For example, the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property accepts [\<length\>](#length-value) values, so a [math function](#math-function) that resolves to a <a id="ref-for-length-value③②"></a>\<length\>, such as calc(5px + 1em), can be used in <a id="ref-for-propdef-width⑨"></a>width.

<a id="ref-for-math-function①⑤"></a>

<a id="ref-for-number-value④①"></a>

<a id="ref-for-integer-value①②"></a>

<a id="ref-for-css-round-to-the-nearest-integer②"></a>

Additionally, [math functions](#math-function) that resolve to [\<number\>](#number-value) can be used in any place that only accepts [\<integer\>](#integer-value); the value is [rounded to the nearest integer](#css-round-to-the-nearest-integer) as it resolves.

Operators form sub-expressions, which gain types based on their arguments.

<a id="ref-for-length-value③③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In previous versions of this specification, multiplication and division were limited in what arguments they could take, to avoid producing more complex intermediate results (such as 1px \* 1em, which is [\<length\>](#length-value)²) and to make division-by-zero detectable at parse time. This version now relaxes those restrictions.

<a id="ref-for-calc-calculation③⑧"></a>

To <a id="determine-the-type-of-a-calculation"></a>determine the type of a [calculation](#calc-calculation):

- <a id="ref-for-cssnumericvalue-add-two-types"></a>

  <a id="ref-for-calc-calculation③⑨"></a>

  <a id="ref-for-cssnumericvalue-type⑤"></a>

  At a + or - sub-expression, attempt to [add the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of the left and right arguments. If this returns failure, the entire [calculation’s](#calc-calculation) type is failure. Otherwise, the sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned type.

- <a id="ref-for-cssnumericvalue-multiply-two-types"></a>

  <a id="ref-for-cssnumericvalue-type⑥"></a>

  At a \* sub-expression, [multiply the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) of the left and right arguments. The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the returned result.

- <a id="ref-for-cssnumericvalue-type⑦"></a>

  <a id="ref-for-cssnumericvalue-invert-a-type"></a>

  At a / sub-expression, let <var>left type</var> be the result of finding the [types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of its left argument, and <var>right type</var> be the result of finding the <a id="ref-for-cssnumericvalue-type⑧"></a>types of its right argument and then [inverting](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-invert-a-type) it.

  <a id="ref-for-cssnumericvalue-type⑨"></a>

  <a id="ref-for-cssnumericvalue-multiply-two-types①"></a>

  The sub-expression’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is the result of [multiplying](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) the <var>left type</var> and <var>right type</var>.

- <a id="ref-for-cssnumericvalue-type①⓪"></a>

  <a id="ref-for-cssnumericvalue-percent-hint"></a>

  Anything else is a terminal value, whose [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined based on its CSS type. (Unless otherwise specified, the type’s associated [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) is null.)

  <a id="ref-for-number-value④②"></a>

  [\<number\>](#number-value)

  <a id="ref-for-integer-value①③"></a>

  [\<integer\>](#integer-value)

  <a id="ref-for-cssnumericvalue-type①①"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ \]» (empty map)

  <a id="ref-for-length-value③④"></a>

  [\<length\>](#length-value)

  <a id="ref-for-cssnumericvalue-type①②"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "length" → 1 \]»

  <a id="ref-for-angle-value①⑤"></a>

  [\<angle\>](#angle-value)

  <a id="ref-for-cssnumericvalue-type①③"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "angle" → 1 \]»

  <a id="ref-for-time-value⑨"></a>

  [\<time\>](#time-value)

  <a id="ref-for-cssnumericvalue-type①④"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "time" → 1 \]»

  <a id="ref-for-frequency-value⑧"></a>

  [\<frequency\>](#frequency-value)

  <a id="ref-for-cssnumericvalue-type①⑤"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "frequency" → 1 \]»

  <a id="ref-for-resolution-value⑦"></a>

  [\<resolution\>](#resolution-value)

  <a id="ref-for-cssnumericvalue-type①⑥"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "resolution" → 1 \]»

  <a id="ref-for-typedef-flex①"></a>

  [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex)

  <a id="ref-for-cssnumericvalue-type①⑦"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "flex" → 1 \]»

  <a id="ref-for-typedef-calc-keyword②"></a>

  [\<calc-keyword\>](#typedef-calc-keyword)

  <a id="ref-for-cssnumericvalue-type①⑧"></a>

  the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is as defined by the keyword

  <a id="ref-for-percentage-value③②"></a>

  [\<percentage\>](#percentage-value)

  <a id="ref-for-math-function①⑥"></a>

  <a id="ref-for-calc-calculation④⓪"></a>

  <a id="ref-for-percentage-value③③"></a>

  <a id="ref-for-propdef-width①⓪"></a>

  <a id="ref-for-length-value③⑤"></a>

  <a id="ref-for-number-value④③"></a>

  <a id="ref-for-cssnumericvalue-type①⑨"></a>

  <a id="ref-for-cssnumericvalue-percent-hint①"></a>

  If, in the context in which the [math function](#math-function) containing this [calculation](#calc-calculation) is placed, [\<percentage\>](#percentage-value)s are resolved relative to another type of value (such as in [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), where <a id="ref-for-percentage-value③④"></a>\<percentage\> is resolved against a [\<length\>](#length-value)), and that other type is <em>not</em> [\<number\>](#number-value), the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is determined as the other type, but with a [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) set to that other type.

  <a id="ref-for-cssnumericvalue-type②⓪"></a>

  Otherwise, the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "percent" → 1 \]».

  anything else

  <a id="ref-for-calc-calculation④①"></a>

  The [calculation’s](#calc-calculation) type is failure.

<a id="ref-for-cssnumericvalue-type②①"></a>

<a id="ref-for-cssnumericvalue-percent-hint②"></a>

A value <a id="css-contain-a-percentage"></a>contains a percentage if its [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is «\[ "percent" → 1 \]», or its type’s [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) is non-null.

<a id="ref-for-cssnumericvalue-add-two-types①"></a>

<a id="ref-for-css-consistent-type⑨"></a>

Two or more calculations have a <a id="css-consistent-type"></a>consistent type if [adding the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) doesn’t result in failure. The [consistent type](#css-consistent-type) is the result of the type addition.

To <a id="css-make-a-type-consistent"></a>make a type <var>base</var> consistent with another type <var>input</var>:

1.  <a id="ref-for-cssnumericvalue-percent-hint③"></a>

    If both <var>base</var> and <var>input</var> have different non-null [percent hints](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint), they can’t be made consistent. Return failure.

2.  <a id="ref-for-cssnumericvalue-percent-hint④"></a>

    If <var>base</var> has a null [percent hint](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-percent-hint) set <var>base</var>’s <a id="ref-for-cssnumericvalue-percent-hint⑤"></a>percent hint to <var>input</var>’s <a id="ref-for-cssnumericvalue-percent-hint⑥"></a>percent hint.

3.  Return <var>base</var>.

<a id="ref-for-math-function①⑦"></a>

<a id="ref-for-cssnumericvalue-type②②"></a>

<a id="ref-for-calc-calculation④②"></a>

[Math functions](#math-function) themselves have [types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type), according to their contained [calculations](#calc-calculation):

<a id="ref-for-funcdef-calc①⑤"></a>

[calc()](#funcdef-calc)

<a id="ref-for-funcdef-abs③"></a>

[abs()](#funcdef-abs)

<a id="ref-for-cssnumericvalue-type②③"></a>

<a id="ref-for-calc-calculation④③"></a>

The [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) of its contained [calculation](#calc-calculation).

<a id="ref-for-funcdef-min⑨"></a>

[min()](#funcdef-min)

<a id="ref-for-funcdef-max⑧"></a>

[max()](#funcdef-max)

<a id="ref-for-funcdef-clamp⑨"></a>

[clamp()](#funcdef-clamp)

<a id="ref-for-funcdef-hypot⑦"></a>

[hypot()](#funcdef-hypot)

<a id="ref-for-funcdef-round④"></a>

[round()](#funcdef-round)

<a id="ref-for-funcdef-mod⑦"></a>

[mod()](#funcdef-mod)

<a id="ref-for-funcdef-rem①⓪"></a>

[rem()](#funcdef-rem)

<a id="ref-for-cssnumericvalue-add-two-types②"></a>

<a id="ref-for-calc-calculation④④"></a>

The result of [adding the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-add-two-types) of its comma-separated [calculations](#calc-calculation).

<a id="ref-for-funcdef-asin④"></a>

[asin()](#funcdef-asin)

<a id="ref-for-funcdef-acos④"></a>

[acos()](#funcdef-acos)

<a id="ref-for-funcdef-atan⑥"></a>

[atan()](#funcdef-atan)

<a id="ref-for-funcdef-atan2③"></a>

[atan2()](#funcdef-atan2)

«\[ "angle" → 1 \]».

<a id="ref-for-funcdef-sign③"></a>

[sign()](#funcdef-sign)

<a id="ref-for-funcdef-sin⑤"></a>

[sin()](#funcdef-sin)

<a id="ref-for-funcdef-cos④"></a>

[cos()](#funcdef-cos)

<a id="ref-for-funcdef-tan⑤"></a>

[tan()](#funcdef-tan)

<a id="ref-for-funcdef-pow⑤"></a>

[pow()](#funcdef-pow)

<a id="ref-for-funcdef-sqrt⑤"></a>

[sqrt()](#funcdef-sqrt)

<a id="ref-for-funcdef-log④"></a>

[log()](#funcdef-log)

<a id="ref-for-funcdef-exp③"></a>

[exp()](#funcdef-exp)

«\[ \]» (empty map).

<a id="ref-for-cssnumericvalue-type②④"></a>

<a id="ref-for-math-function①⑧"></a>

For each of the above, if the [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is failure, the [math function](#math-function) is invalid.

<a id="ref-for-math-function①⑨"></a>

<a id="ref-for-number-value④④"></a>

<a id="ref-for-length-value③⑥"></a>

<a id="ref-for-angle-value①⑥"></a>

<a id="ref-for-time-value①⓪"></a>

<a id="ref-for-frequency-value⑨"></a>

<a id="ref-for-resolution-value⑧"></a>

<a id="ref-for-typedef-flex②"></a>

<a id="ref-for-percentage-value③⑤"></a>

<a id="ref-for-cssnumericvalue-type②⑤"></a>

<a id="ref-for-cssnumericvalue-match②"></a>

A [math function](#math-function) resolves to [\<number\>](#number-value), [\<length\>](#length-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<frequency\>](#frequency-value), [\<resolution\>](#resolution-value), [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex), or [\<percentage\>](#percentage-value) according to which of those productions its [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match). (These categories are mutually exclusive.) If it can’t <a id="ref-for-cssnumericvalue-match③"></a>match any of these, the <a id="ref-for-math-function②⓪"></a>math function is invalid.

<a id="ref-for-math-function②①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Algebraic simplifications do not affect the validity of a [math function](#math-function) or its resolved type. For example, calc(5px - 5px + 10s) and calc(0 \* 5px + 10s) are both invalid due to the attempt to add a length and a time.

<a id="ref-for-percentage-value③⑥"></a>

<a id="ref-for-number-value④⑤"></a>

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-typedef-dimension⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that [\<percentage\>](#percentage-value)s relative to [\<number\>](#number-value)s, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), are not <em>combinable</em> with those numbers—​<a id="ref-for-propdef-opacity①"></a>opacity: calc(.25 + 25%) is invalid. Allowing this causes significant problems with "unit algebra" (allowing multiplication/division of [\<dimension\>](#typedef-dimension)s), and in every case so far, doesn’t provide any new functionality. (For example, <a id="ref-for-propdef-opacity②"></a>opacity: 25% is identical to <a id="ref-for-propdef-opacity③"></a>opacity: .25; it’s just a trivial syntax transform.) You can still perform other operations with them, such as <a id="ref-for-propdef-opacity④"></a>opacity: calc(100% / 3);, which is valid.

<a id="ref-for-typedef-number-token③"></a>

<a id="ref-for-number-value④⑥"></a>

<a id="ref-for-integer-value①④"></a>

<a id="ref-for-length-value③⑦"></a>

<a id="ref-for-math-function②②"></a>

<a id="ref-for-propdef-width①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s are always interpreted as [\<number\>](#number-value)s or [\<integer\>](#integer-value)s, "unitless 0" [\<length\>](#length-value)s aren’t supported in [math functions](#math-function). That is, [width: calc(0 + 5px);](https://www.w3.org/TR/css-sizing-3/#propdef-width) is invalid, because it’s trying to add a <a id="ref-for-number-value④⑦"></a>\<number\> to a <a id="ref-for-length-value③⑧"></a>\<length\>, even though both <a id="ref-for-propdef-width①②"></a>width: 0; and <a id="ref-for-propdef-width①③"></a>width: 5px; are valid.

<a id="ref-for-number-value④⑧"></a>

<a id="ref-for-length-value③⑨"></a>

<a id="ref-for-propdef-line-height①③"></a>

<a id="ref-for-propdef-tab-size"></a>

<a id="ref-for-funcdef-calc①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although there are a few properties in which a bare [\<number\>](#number-value) becomes a [\<length\>](#length-value) at used-value time (specifically, [line-height](https://drafts.csswg.org/css2/#propdef-line-height) and [tab-size](https://www.w3.org/TR/css-text-4/#propdef-tab-size)), <a id="ref-for-number-value④⑨"></a>\<number\>s never become "length-like" in [calc()](#funcdef-calc). They always stay as <a id="ref-for-number-value⑤⓪"></a>\<number\>s.

<a id="ref-for-length-value④⓪"></a>

<a id="ref-for-number-value⑤①"></a>

<a id="ref-for-px①⓪"></a>

<a id="ref-for-math-function②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In Quirks Mode [\[QUIRKS\]](#biblio-quirks), some properties that would normally only accept [\<length\>](#length-value)s are defined to also accept [\<number\>](#number-value)s, interpreting them as [px](#px) lengths. Like unitless zeroes, this has no effect on the parsing or behavior of [math functions](#math-function), though a <a id="ref-for-math-function②④"></a>math function that resolves to a <a id="ref-for-number-value⑤②"></a>\<number\> value might become valid in Quirks Mode (and have its result interpreted as a <a id="ref-for-px①①"></a>px length).

#### <a id="calc-ieee"></a>10.9.1.  Infinities, NaN, and Signed Zero

<a id="ref-for-math-function②⑤"></a>

[Math functions](#math-function) follow IEEE-754 semantics, which means they recognize the concepts of positive and negative zero, positive and negative infinity, and NaN (not a number).

<a id="ref-for-calculation-tree"></a>

<a id="ref-for-math-function②⑥"></a>

However, these concepts are only retained within a [calculation tree](#calculation-tree); if a <a id="top-level-calculation"></a>top-level calculation (a [math function](#math-function) not nested directly inside of another <a id="ref-for-math-function②⑦"></a>math function) would result in one of these special values, they’re instead "censored" into a standard representable value, as defined below.

<a id="css-signed-zero"></a>Signed zeros (indicated here as 0⁺ or 0⁻) can not be written directly in CSS; 0, +0 and -0 all produce the standard "unsigned" zero, which is considered positive (0⁺) for the purposes of these rules.

<a id="ref-for-css-signed-zero"></a>

[Signed zeroes](#css-signed-zero) are produced in the following ways:

- Negative zero (0⁻) can be produced by a multiplication or division that produces zero with exactly one negative argument (such as -5 \* 0 or 1 / -infinity).

- 0⁻ + 0⁻ or 0⁻ - 0⁺ produces 0⁻. All other additions or subtractions that would produce a zero produce 0⁺.

- Multiplying or dividing 0⁻ with a positive number (including 0⁺) produces a negative result (either 0⁻ or −∞), while multiplying or dividing 0⁻ with a negative number produces a positive result.

  (In other words, multiplying or dividing with 0⁻ follows standard sign rules.)

- When comparing 0⁺ and 0⁻, 0⁻ is less than 0⁺. For example, min(0⁺, 0⁻) must produce 0⁻, max(0⁺, 0⁻) must produce 0⁺, and clamp(0⁺, 0⁻, 1) must produce 0⁺.

- <a id="ref-for-math-function②⑧"></a>

  Certain argument combinations in [math functions](#math-function) are defined to produce 0⁻ (for example, round(-1, infinity)). All other operations that produce a zero produce positive zero (0⁺).

<a id="ref-for-css-signed-zero①"></a>

<a id="ref-for-top-level-calculation"></a>

[Signed zeroes](#css-signed-zero) do not escape a [top-level calculation](#top-level-calculation); they’re censored into the "unsigned" zero.

<a id="ref-for-valdef-calc-infinity②"></a>

<a id="ref-for-valdef-calc--infinity①"></a>

<a id="css-infinity"></a>Infinities (indicated here as +∞ or −∞) can be written directly using the [math constants](#calc-error-constants) [infinity](#valdef-calc-infinity) and [-infinity](#valdef-calc--infinity), or produced as a result of some calculations:

- Dividing a value by zero produces either +∞ or −∞, according to the standard sign rules.

- Adding or subtracting ±∞ to anything produces the appropriate infinity.

- Multiplying any value by ±∞ produces the appropriate infinity.

- Dividing any value by ±∞ produces zero.

- <a id="ref-for-math-function②⑨"></a>

  <a id="ref-for-css-infinity①"></a>

  Certain argument combinations in [math functions](#math-function) are defined to produce [infinities](#css-infinity) (for example, pow(0, -1) produces +∞).

<a id="ref-for-css-nan①"></a>

<a id="ref-for-css-infinity②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The rules for producing [NaN](#css-nan), below, supersede the above rules for producing [infinities](#css-infinity).

<a id="ref-for-css-infinity③"></a>

<a id="ref-for-top-level-calculation①"></a>

[Infinities](#css-infinity) do not escape a [top-level calculation](#top-level-calculation); they’re clamped to the minimum or maximum value allowed in the context, as defined in [§ 10.12 Range Checking](#calc-range).

<a id="ref-for-valdef-calc-nan②"></a>

<a id="css-nan"></a>NaN (short for "not a number") is the result of certain operations that don’t have a well-defined value. It can be written directly using the [math constants](#calc-error-constants) [NaN](#valdef-calc-nan), or produced as a result of some calculations:

- Dividing zero by zero, dividing ±∞ by ±∞, multiplying 0 by ±∞, adding +∞ to −∞, or subtracting two infinities of the same sign produces NaN.

  These rules override any other result, if there’s a conflict. For example, 0 / 0 is NaN, not +∞.

- <a id="ref-for-math-function③⓪"></a>

  <a id="ref-for-css-nan②"></a>

  Certain argument combinations in [math functions](#math-function) are defined to produce [NaN](#css-nan) (for example, asin(2) produces NaN).

- Any operation with at least one NaN argument produces NaN.

<a id="ref-for-css-nan③"></a>

<a id="ref-for-top-level-calculation②"></a>

[NaN](#css-nan) does not escape a [top-level calculation](#top-level-calculation); it’s censored into a zero value

<a id="ref-for-top-level-calculation③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-07a5380d"></a> For example, calc(-5 \* 0) produces an unsigned zero—​the calculation resolves to 0⁻, but as it’s a [top-level calculation](#top-level-calculation), it’s then censored to an unsigned zero.
>
> <a id="ref-for-top-level-calculation④"></a>
>
> On the other hand, calc(1 / calc(-5 \* 0)) produces −∞, same as calc(1 / (-5 \* 0))—​the inner calc resolves to 0⁻, and as it’s not a [top-level calculation](#top-level-calculation), it passes it up unchanged to the outer calc to produce −∞. If it was censored into an unsigned zero, it would instead produce +∞.

### <a id="calc-internal"></a>10.10.  Internal Representation

<a id="ref-for-css-internal-representation"></a>

<a id="ref-for-math-function③①"></a>

<a id="ref-for-calc-calculation④⑤"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of a [math function](#math-function) is a <a id="calculation-tree"></a>calculation tree: a tree where the branch nodes are <a id="calculation-tree-operator-nodes"></a>operator nodes corresponding either to <a id="ref-for-math-function③②"></a>math functions (such as Min, Cos, Sqrt, etc) or to operators in a [calculation](#calc-calculation) (Sum, Product, Negate, and Invert, the <a id="calculation-tree-calc-operator-nodes"></a>calc-operator nodes), and the leaf nodes are either numeric values (such as numbers, dimensions, and percentages) or non-<a id="ref-for-math-function③③"></a>math functions that resolve to a numeric type.

<a id="ref-for-math-function③④"></a>

<a id="ref-for-calculation-tree①"></a>

[Math functions](#math-function) are turned into [calculation trees](#calculation-tree) depending on the function:

calc()

<a id="ref-for-css-internal-representation①"></a>

<a id="ref-for-funcdef-calc①⑦"></a>

<a id="ref-for-parse-a-calculation"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of a [calc()](#funcdef-calc) function is the result of [parsing a calculation](#parse-a-calculation) from its argument.

<a id="ref-for-math-function③⑤"></a>

any other [math function](#math-function)

<a id="ref-for-css-internal-representation②"></a>

<a id="ref-for-calculation-tree-operator-nodes"></a>

<a id="ref-for-parse-a-calculation①"></a>

The [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) is an [operator node](#calculation-tree-operator-nodes) with the same name as the function, whose children are the result of [parsing a calculation](#parse-a-calculation) from each of the function’s arguments, in the order they appear.

<a id="ref-for-calc-calculation④⑥"></a>

<a id="ref-for-component-value②"></a>

<a id="ref-for-calculation-tree②"></a>

To <a id="parse-a-calculation"></a>parse a calculation, given a [calculation](#calc-calculation) <var>values</var> represented as a list of [component values](https://www.w3.org/TR/css-syntax-3/#component-value), and returning a [calculation tree](#calculation-tree):

1.  <a id="ref-for-typedef-whitespace-token"></a>

    Discard any [\<whitespace-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-whitespace-token)s from <var>values</var>.

2.  <a id="ref-for-typedef-delim-token"></a>

    An item in <var>values</var> is an “operator” if it’s a [\<delim-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-delim-token) with the value "+", "-", "\*", or "/". Otherwise, it’s a “value”.

3.  Collect children into Product and Invert nodes.

    For every consecutive run of value items in <var>values</var> separated by "\*" or "/" operators:

    1.  For each "/" operator in the run, replace its right-hand value item <var>rhs</var> with an Invert node containing <var>rhs</var> as its child.

    2.  Replace the entire run with a Product node containing the value items of the run as its children.

4.  Collect children into Sum and Negate nodes.

    1.  For each "-" operator item in <var>values</var>, replace its right-hand value item <var>rhs</var> with a Negate node containing <var>rhs</var> as its child.

    2.  <a id="ref-for-simple-block"></a>

        If <var>values</var> has only one item, and it is a Product node or a parenthesized [simple block](https://www.w3.org/TR/css-syntax-3/#simple-block), replace <var>values</var> with that item.

        Otherwise, replace <var>values</var> with a Sum node containing the value items of <var>values</var> as its children.

5.  At this point <var>values</var> is a tree of Sum, Product, Negate, and Invert nodes, with other types of values at the leaf nodes. Process the leaf nodes.

    For every leaf node <var>leaf</var> in <var>values</var>:

    1.  <a id="ref-for-simple-block①"></a>

        <a id="ref-for-parse-a-calculation②"></a>

        If <var>leaf</var> is a parenthesized [simple block](https://www.w3.org/TR/css-syntax-3/#simple-block), replace <var>leaf</var> with the result of [parsing a calculation](#parse-a-calculation) from <var>leaf</var>’s contents.

    2.  <a id="ref-for-math-function③⑥"></a>

        <a id="ref-for-css-internal-representation③"></a>

        If <var>leaf</var> is a [math function](#math-function), replace <var>leaf</var> with the [internal representation](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of that math function.

6.  <a id="ref-for-simplify-a-calculation-tree"></a>

    Return the result of [simplifying a calculation tree](#simplify-a-calculation-tree) from <var>values</var>.

#### <a id="calc-simplification"></a>10.10.1.  Simplification

<a id="ref-for-css-internal-representation④"></a>

<a id="ref-for-math-function③⑦"></a>

[Internal representations](https://drafts.css-houdini.org/css-typed-om-1/#css-internal-representation) of [math functions](#math-function) are eagerly simplified to the extent possible, using standard algebraic simplifications (distributing multiplication over sums, combining similar units, etc.).

<a id="ref-for-at-font-face-rule"></a>

<a id="ref-for-math-function③⑧"></a>

<a id="ref-for-specified-value④"></a>

When used in non-property contexts (such as in [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) descriptors, for example), [math functions](#math-function) are simplified as if they were [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value).

To <a id="simplify-a-calculation-tree"></a>simplify a calculation tree <var>root</var>:

1.  If <var>root</var> is a numeric value:

    1.  <a id="ref-for-canonical-unit⑨"></a>

        If <var>root</var> is a percentage that will be resolved against another value, and there is enough information available to resolve it, do so, and express the resulting numeric value in the appropriate [canonical unit](#canonical-unit). Return the value.

    2.  <a id="ref-for-canonical-unit①⓪"></a>

        If <var>root</var> is a dimension that is not expressed in its [canonical unit](#canonical-unit), and there is enough information available to convert it to the <a id="ref-for-canonical-unit①①"></a>canonical unit, do so, and return the value.

    3.  <a id="ref-for-typedef-calc-keyword③"></a>

        <a id="ref-for-simplify-a-calculation-tree①"></a>

        If <var>root</var> is a [\<calc-keyword\>](#typedef-calc-keyword) that can be resolved, return what it resolves to, [simplified](#simplify-a-calculation-tree).

    4.  Otherwise, return <var>root</var>.

2.  If <var>root</var> is any other leaf node (not an operator node):

    1.  <a id="ref-for-canonical-unit①②"></a>

        If there is enough information available to determine its numeric value, return its value, expressed in the value’s [canonical unit](#canonical-unit).

    2.  Otherwise, return <var>root</var>.

3.  <a id="ref-for-calculation-tree-operator-nodes①"></a>

    <a id="ref-for-simplify-a-calculation-tree②"></a>

    <a id="ref-for-calc-calculation④⑦"></a>

    At this point, <var>root</var> is an [operator node](#calculation-tree-operator-nodes). [Simplify](#simplify-a-calculation-tree) all the [calculation](#calc-calculation) children of <var>root</var>.

4.  <a id="ref-for-calculation-tree-operator-nodes②"></a>

    <a id="ref-for-calculation-tree-calc-operator-nodes"></a>

    <a id="ref-for-calc-calculation④⑧"></a>

    <a id="ref-for-canonical-unit①③"></a>

    If <var>root</var> is an [operator node](#calculation-tree-operator-nodes) that’s not one of the [calc-operator nodes](#calculation-tree-calc-operator-nodes), and all of its [calculation](#calc-calculation) children are numeric values with enough information to compute the operation <var>root</var> represents, return the result of running <var>root</var>’s operation using its children, expressed in the result’s [canonical unit](#canonical-unit).

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > If a percentage is left at this point, it will <em>usually</em> block simplification of the node, since it needs to be resolved against another value using information not currently available. (Otherwise, it would have been converted to a different value in an earlier step.) This includes operations such as "min", since percentages might resolve against a negative basis, and thus end up with an opposite comparative relationship than the raw percentage value would seem to indicate.
    >
    > <a id="ref-for-propdef-opacity⑤"></a>
    >
    > However, "raw" percentages—​ones which do not resolve against another value, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)—​might not block simplification.

5.  If <var>root</var> is a Min or Max node, attempt to <em>partially</em> simplify it:

    1.  <a id="ref-for-list-iterate"></a>

        [For each](https://infra.spec.whatwg.org/#list-iterate) node <var>child</var> of <var>root</var>’s children:

        If <var>child</var> is a numeric value with enough information to compare magnitudes with another child of the same unit (see note in previous step), and there are other children of <var>root</var> that are numeric values with the same unit, combine all such children with the appropriate operator per <var>root</var>, and replace <var>child</var> with the result, removing all other child nodes involved.

    2.  If <var>root</var> has only one child, return the child.

        Otherwise, return <var>root</var>.

6.  If <var>root</var> is a Negate node:

    1.  If <var>root</var>’s child is a numeric value, return an equivalent numeric value, but with the value negated (0 - value).

    2.  If <var>root</var>’s child is a Negate node, return the child’s child.

    3.  Return <var>root</var>.

7.  If <var>root</var> is an Invert node:

    1.  If <var>root</var>’s child is a number (not a percentage or dimension) return the reciprocal of the child’s value.

    2.  If <var>root</var>’s child is an Invert node, return the child’s child.

    3.  Return <var>root</var>.

8.  If <var>root</var> is a Sum node:

    1.  For each of <var>root</var>’s children that are Sum nodes, replace them with their children.

    2.  For each set of <var>root</var>’s children that are numeric values with identical units, remove those children and replace them with a single numeric value containing the sum of the removed nodes, and with the same unit.

        (E.g. combine numbers, combine percentages, combine px values, etc.)

    3.  If <var>root</var> has only a single child at this point, return the child. Otherwise, return <var>root</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Zero-valued terms cannot be simply removed from a Sum; they can only be combined with other values that have identical units. (This is because the mere presence of a unit, even with a zero value, can sometimes imply a change in behavior.)

9.  If <var>root</var> is a Product node:

    1.  For each of <var>root</var>’s children that are Product nodes, replace them with their children.

    2.  If <var>root</var> has multiple children that are numbers (not percentages or dimensions), remove them and replace them with a single number containing the product of the removed nodes.

    3.  If <var>root</var> contains only two children, one of which is a number (not a percentage or dimension) and the other of which is a Sum whose children are all numeric values, multiply all of the Sum’s children by the number, then return the Sum.

    4.  <a id="ref-for-cssnumericvalue-multiply-two-types②"></a>

        <a id="ref-for-cssnumericvalue-invert-a-type①"></a>

        <a id="ref-for-cssnumericvalue-match④"></a>

        <a id="ref-for-math-function③⑨"></a>

        <a id="ref-for-canonical-unit①④"></a>

        If <var>root</var> contains only numeric values and/or Invert nodes containing numeric values, and [multiplying the types](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-multiply-two-types) of all the children (noting that the type of an Invert node is the [inverse](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-invert-a-type) of its child’s type) results in a type that [matches](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-match) any of the types that a [math function](#math-function) can resolve to, return the result of multiplying all the values of the children (noting that the value of an Invert node is the reciprocal of its child’s value), expressed in the result’s [canonical unit](#canonical-unit).

    5.  Return <var>root</var>.

### <a id="calc-computed-value"></a>10.11.  Computed Value

<a id="ref-for-computed-value①⑤"></a>

<a id="ref-for-math-function④⓪"></a>

<a id="ref-for-calculation-tree③"></a>

<a id="ref-for-simplify-a-calculation-tree③"></a>

<a id="ref-for-em④"></a>

<a id="ref-for-px①②"></a>

The [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [math function](#math-function) is its [calculation tree](#calculation-tree) [simplified](#simplify-a-calculation-tree), using all the information available at <a id="ref-for-computed-value①⑥"></a>computed value time. (Such as the [em](#em) to [px](#px) ratio, how to resolve percentages in some properties, etc.)

<a id="ref-for-math-function④①"></a>

Where percentages are not resolved at computed-value time, they are not resolved in [math functions](#math-function), e.g. calc(100% - 100% + 1px) resolves to calc(0% + 1px), not to 1px. If there are special rules for computing percentages in a value (e.g. [the height property](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)), they apply whenever a <a id="ref-for-math-function④②"></a>math function contains percentages.

<a id="ref-for-calculation-tree④"></a>

<a id="ref-for-used-value④"></a>

<a id="ref-for-math-function④③"></a>

The [calculation tree](#calculation-tree) is again simplified at [used value](https://www.w3.org/TR/css-cascade-5/#used-value) time; with <a id="ref-for-used-value⑤"></a>used value time information, a [math function](#math-function) always simplifies down to a single numeric value.

<a id="ref-for-propdef-font-size⑥"></a>

<a id="ref-for-computed-value①⑦"></a>

<a id="ref-for-font-relative-length⑤"></a>

<a id="ref-for-propdef-background-position③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-023dad93"></a> For example, whereas [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) computes percentage values at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time so that [font-relative length](#font-relative-length) units can be computed, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) has layout-dependent behavior for percentage values, and thus does not resolve percentages until used-value time.
>
> <a id="ref-for-propdef-background-position④"></a>
>
> <a id="ref-for-funcdef-calc①⑧"></a>
>
> <a id="ref-for-propdef-font-size⑦"></a>
>
> Due to this, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) computation preserves the percentage in a [calc()](#funcdef-calc) whereas [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) will compute such expressions directly into a length.

<a id="ref-for-valdef-width-auto"></a>

Given the complexities of width and height calculations on table cells and table elements, math expressions mixing both percentages and non-zero lengths for widths and heights on table columns, table column groups, table rows, table row groups, and table cells in both auto and fixed layout tables MUST be treated as if [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) had been specified.

### <a id="calc-range"></a>10.12.  Range Checking

<a id="ref-for-math-function④④"></a>

<a id="ref-for-top-level-calculation⑤"></a>

<a id="ref-for-computed-value①⑧"></a>

<a id="ref-for-used-value⑥"></a>

<a id="ref-for-specified-value⑤"></a>

Parse-time range-checking of values is not performed within [math functions](#math-function), and therefore out-of-range values do not cause the declaration to become invalid. However, the value resulting from a [top-level calculation](#top-level-calculation) must be clamped to the range allowed in the target context. Clamping is performed on [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) to the extent possible, and also on [used values](https://www.w3.org/TR/css-cascade-5/#used-value) if computation was unable to sufficiently simplify the expression to allow range-checking. (Clamping is not performed on [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value).)

<a id="ref-for-funcdef-calc①⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This requires all contexts accepting [calc()](#funcdef-calc) to define their allowable values as a closed (not open) interval.

<a id="ref-for-propdef-animation-iteration-count"></a>

<a id="ref-for-math-function④⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By definition, ±∞ are outside the allowed range for any property, and will clamp to the minimum/maximum value allowed. Even properties that can explicitly represent infinity as a keyword value, such as [animation-iteration-count](https://www.w3.org/TR/css-animations-1/#propdef-animation-iteration-count), will end up clamping ±∞, as [math functions](#math-function) can’t resolve to keyword values; the <em>numeric</em> part of the property’s syntax still has a minimum/maximum value.

<a id="ref-for-math-function④⑥"></a>

<a id="ref-for-number-value⑤③"></a>

<a id="ref-for-integer-value①⑤"></a>

<a id="ref-for-computed-value①⑨"></a>

<a id="ref-for-used-value⑦"></a>

<a id="ref-for-css-round-to-the-nearest-integer③"></a>

Additionally, if a [math function](#math-function) that resolves to [\<number\>](#number-value) is used somewhere that only accepts [\<integer\>](#integer-value), the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and [used value](https://www.w3.org/TR/css-cascade-5/#used-value) are [rounded to the nearest integer](#css-round-to-the-nearest-integer), in the same manner as clamping, above.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ec14dee2"></a> Since widths smaller than 0px are not allowed, these three declarations are equivalent:
>
> ```text
> width: calc(5px - 10px);
> width: calc(-5px);
> width: 0px;
> ```
>
> <a id="ref-for-propdef-width①④"></a>
>
> <a id="ref-for-funcdef-calc②⓪"></a>
>
> Note however that [width: -5px](https://www.w3.org/TR/css-sizing-3/#propdef-width) is not equivalent to <a id="ref-for-propdef-width①⑤"></a>width: calc(-5px)! Out-of-range values <em>outside</em> [calc()](#funcdef-calc) are syntactically invalid, and cause the entire declaration to be dropped.

### <a id="calc-serialize"></a>10.13.  Serialization

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f5bc4b00"></a> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html).

To <a id="serialize-a-math-function"></a>serialize a math function <var>fn</var>:

1.  <a id="ref-for-calculation-tree⑤"></a>

    <a id="ref-for-computed-value②⓪"></a>

    If the root of the [calculation tree](#calculation-tree) <var>fn</var> represents is a numeric value (number, percentage, or dimension), and the serialization being produced is of a [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) or later, then clamp the value to the range allowed for its context (if necessary), then serialize the value as normal and return the result.

2.  If <var>fn</var> represents an infinite or NaN value:

    1.  <a id="ref-for-string①"></a>

        Let <var>s</var> be the [string](https://infra.spec.whatwg.org/#string) "calc(".

    2.  <a id="ref-for-valdef-calc-infinity③"></a>

        <a id="ref-for-valdef-calc--infinity②"></a>

        <a id="ref-for-valdef-calc-nan③"></a>

        Serialize the keyword [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), or [NaN](#valdef-calc-nan), as appropriate to represent the value, and append it to <var>s</var>.

    3.  <a id="ref-for-cssnumericvalue-type②⑥"></a>

        <a id="ref-for-number-value⑤④"></a>

        <a id="ref-for-canonical-unit①⑤"></a>

        <a id="ref-for-px①③"></a>

        <a id="ref-for-length-value④①"></a>

        If <var>fn</var>’s [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) is anything other than «\[ \]» (empty, representing a [\<number\>](#number-value)), append " \* " to <var>s</var>. Create a numeric value in the [canonical unit](#canonical-unit) for <var>fn</var>’s <a id="ref-for-cssnumericvalue-type②⑦"></a>type (such as [px](#px) for [\<length\>](#length-value)), with a value of 1. Serialize this numeric value and append it to <var>s</var>.

    4.  Return <var>s</var>.

3.  <a id="ref-for-calculation-tree⑥"></a>

    <a id="ref-for-calculation-tree-calc-operator-nodes①"></a>

    If the [calculation tree’s](#calculation-tree) root node is a numeric value, or a [calc-operator node](#calculation-tree-calc-operator-nodes), let <var>s</var> be a string initially containing "calc(".

    Otherwise, let <var>s</var> be a string initially containing the name of the root node, lowercased (such as "sin" or "max"), followed by a "(" (open parenthesis).

4.  <a id="ref-for-serialize-a-calculation-tree"></a>

    <a id="ref-for-string-concatenate"></a>

    For each child of the root node, [serialize the calculation tree](#serialize-a-calculation-tree). If a result of this serialization starts with a "(" (open parenthesis) and ends with a ")" (close parenthesis), remove those characters from the result. [Concatenate](https://infra.spec.whatwg.org/#string-concatenate) all of the results using ", " (comma followed by space), then append the result to <var>s</var>.

5.  Append ")" (close parenthesis) to <var>s</var>.

6.  Return <var>s</var>.

To <a id="serialize-a-calculation-tree"></a>serialize a calculation tree:

1.  <a id="ref-for-calculation-tree⑦"></a>

    Let <var>root</var> be the root node of the [calculation tree](#calculation-tree).

2.  <a id="ref-for-math-function④⑦"></a>

    If <var>root</var> is a numeric value, or a non-[math function](#math-function), serialize <var>root</var> per the normal rules for it and return the result.

3.  <a id="ref-for-serialize-a-math-function"></a>

    <a id="ref-for-calc-calculation④⑨"></a>

    If <var>root</var> is anything but a Sum, Negate, Product, or Invert node, [serialize a math function](#serialize-a-math-function) for the function corresponding to the node type, treating the node’s children as the function’s comma-separated [calculation](#calc-calculation) arguments, and return the result.

4.  <a id="ref-for-string②"></a>

    If <var>root</var> is a Negate node, let <var>s</var> be a [string](https://infra.spec.whatwg.org/#string) initially containing "(-1 \* ".

    <a id="ref-for-serialize-a-calculation-tree①"></a>

    [Serialize](#serialize-a-calculation-tree) <var>root</var>’s child, and append it to <var>s</var>.

    Append ")" to <var>s</var>, then return it.

5.  <a id="ref-for-string③"></a>

    If <var>root</var> is an Invert node, let <var>s</var> be a [string](https://infra.spec.whatwg.org/#string) initially containing "(1 / ".

    <a id="ref-for-serialize-a-calculation-tree②"></a>

    [Serialize](#serialize-a-calculation-tree) <var>root</var>’s child, and append it to <var>s</var>.

    Append ")" to <var>s</var>, then return it.

6.  <a id="ref-for-string④"></a>

    If <var>root</var> is a Sum node, let <var>s</var> be a [string](https://infra.spec.whatwg.org/#string) initially containing "(".

    <a id="ref-for-sort-a-calculations-children"></a>

    [Sort root’s children](#sort-a-calculations-children).

    <a id="ref-for-serialize-a-calculation-tree③"></a>

    [Serialize](#serialize-a-calculation-tree) <var>root</var>’s first child, and append it to <var>s</var>.

    <a id="ref-for-list-iterate①"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>child</var> of <var>root</var> beyond the first:

    1.  <a id="ref-for-serialize-a-calculation-tree④"></a>

        If <var>child</var> is a Negate node, append " - " to <var>s</var>, then [serialize](#serialize-a-calculation-tree) the Negate’s child and append the result to <var>s</var>.

    2.  If <var>child</var> is a negative numeric value, append " - " to <var>s</var>, then serialize the negation of <var>child</var> as normal and append the result to <var>s</var>.

    3.  <a id="ref-for-serialize-a-calculation-tree⑤"></a>

        Otherwise, append " + " to <var>s</var>, then [serialize](#serialize-a-calculation-tree) <var>child</var> and append the result to <var>s</var>.

    Finally, append ")" to <var>s</var> and return it.

7.  <a id="ref-for-string⑤"></a>

    If <var>root</var> is a Product node, let <var>s</var> be a [string](https://infra.spec.whatwg.org/#string) initially containing "(".

    <a id="ref-for-sort-a-calculations-children①"></a>

    [Sort root’s children](#sort-a-calculations-children).

    <a id="ref-for-serialize-a-calculation-tree⑥"></a>

    [Serialize](#serialize-a-calculation-tree) <var>root</var>’s first child, and append it to <var>s</var>.

    <a id="ref-for-list-iterate②"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) <var>child</var> of <var>root</var> beyond the first:

    1.  <a id="ref-for-serialize-a-calculation-tree⑦"></a>

        If <var>child</var> is an Invert node, append " / " to <var>s</var>, then [serialize](#serialize-a-calculation-tree) the Invert’s child and append the result to <var>s</var>.

    2.  <a id="ref-for-serialize-a-calculation-tree⑧"></a>

        Otherwise, append " \* " to <var>s</var>, then [serialize](#serialize-a-calculation-tree) <var>child</var> and append the result to <var>s</var>.

    Finally, append ")" to <var>s</var> and return it.

To <a id="sort-a-calculations-children"></a>sort a calculation’s children <var>nodes</var>:

1.  Let <var>ret</var> be an empty list.

2.  If <var>nodes</var> contains a number, remove it from <var>nodes</var> and append it to <var>ret</var>.

3.  If <var>nodes</var> contains a percentage, remove it from <var>nodes</var> and append it to <var>ret</var>.

4.  <a id="ref-for-ascii-case-insensitive⑦"></a>

    If <var>nodes</var> contains any dimensions, remove them from <var>nodes</var>, sort them by their units, ordered [ASCII case-insensitively](https://infra.spec.whatwg.org/#ascii-case-insensitive), and append them to <var>ret</var>.

5.  If <var>nodes</var> still contains any items, append them to <var>ret</var> in the same order.

6.  Return <var>ret</var>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c3db2475"></a> For example, calc(20px + 30px) would serialize as calc(50px) as a specified value, or as 50px as a computed value.
>
> <a id="ref-for-funcdef-calc②①"></a>
>
> A value like calc(20px + 0%) would serialize as calc(0% + 20px), maintaining both terms in the serialized value. (It’s important to maintain zero-valued terms, so the [calc()](#funcdef-calc) doesn’t suddenly "change shape" in the middle of a transition when one of the values happens to have a zero value temporarily. This also removes the need to "pick a unit" when all the terms are zero.)
>
> <a id="ref-for-em⑤"></a>
>
> <a id="ref-for-funcdef-calc②②"></a>
>
> A value like calc(20px + 2em) would serialize as calc(2em + 20px) as a specified value (maintaining both units as they’re incompatible at specified-value time, but sorting them alphabetically), or as something like 52px as a computed value ([em](#em) values are converted to absolute lengths at computed-value time, so assuming 1em = 16px, they combine into 52px, which then drops the [calc()](#funcdef-calc) wrapper.)

<a id="ref-for-at-font-face-rule①"></a>

<a id="ref-for-math-function④⑧"></a>

<a id="ref-for-specified-value⑥"></a>

When used in non-property contexts (such as in [@font-face](https://www.w3.org/TR/css-fonts-5/#at-font-face-rule) descriptors, for example), [math functions](#math-function) are simplified as if they were [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value).

See [\[CSSOM\]](#biblio-cssom) for further information on serialization.

### <a id="combine-math"></a>10.14.  Combination of Math Functions

<a id="ref-for-interpolation①④"></a>

<a id="ref-for-math-function④⑨"></a>

<a id="ref-for-simplify-a-calculation-tree④"></a>

[Interpolation](#interpolation) of [math functions](#math-function), with each other or with numeric values and other numeric-valued functions, is defined as V<sub>result</sub> = calc((1 - p) \* V<sub>A</sub> + p \* V<sub>B</sub>). ([Simplification](#simplify-a-calculation-tree) of the value might then reduce the expression to a smaller, simpler form.)

<a id="ref-for-addition①⑤"></a>

<a id="ref-for-math-function⑤⓪"></a>

<a id="ref-for-simplify-a-calculation-tree⑤"></a>

[Addition](#addition) of [math functions](#math-function), with each other or with numeric values and other numeric-valued functions, is defined as V<sub>result</sub> = calc(V<sub>A</sub> + V<sub>B</sub>). ([Simplification](#simplify-a-calculation-tree) of the value might then reduce the expression to a smaller, simpler form.)

## <a id="linked-properties"></a> Appendix A: Coordinating List-Valued Properties

<a id="ref-for-shorthand-property"></a>

Some list-valued properties have coordinated effects: each item in their value list applies to a distinct effect, and corresponding entries in each property’s list all refer to the same effect. Often the coordinating values can also be specified together as a single entry in a list-valued [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property).

A typical example is the list-valued background-\* properties, which can specify [multiple background image layers](https://www.w3.org/TR/css-backgrounds-3/#layering). For each property controlling how the image is sized, tiled, placed, etc., the <var>N</var>th item in its list describes some effect that applies to the <var>N</var>th background image.

<a id="ref-for-coordinated-value-list"></a>

A <a id="coordinating-list-property"></a>coordinating list property group creates a <a id="coordinated-value-list"></a>coordinated value list, which has, for each entry, a value from each property in the group; these are used together to define a single effect, such as a background image layer or an animation. The [coordinated value list](#coordinated-value-list) is assembled as follows:

- <a id="ref-for-coordinated-value-list①"></a>

  <a id="ref-for-coordinating-list-property①"></a>

  <a id="ref-for-propdef-background-image"></a>

  The length of the [coordinated value list](#coordinated-value-list) is determined by the number of items specified in one particular [coordinating list property](#coordinating-list-property), the <a id="coordinating-list-base-property"></a>coordinating list base property. (In the case of backgrounds, this is the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property.)

- <a id="ref-for-coordinated-value-list②"></a>

  <a id="ref-for-used-value⑧"></a>

  <a id="ref-for-coordinating-list-property②"></a>

  The <var>N</var>th value of the [coordinated value list](#coordinated-value-list) is constructed by collecting the <var>N</var>th [use value](https://www.w3.org/TR/css-cascade-5/#used-value) of each [coordinating list property](#coordinating-list-property)

- <a id="ref-for-coordinating-list-property③"></a>

  <a id="ref-for-used-value⑨"></a>

  If a [coordinating list property](#coordinating-list-property) has too many values specified, excess values at the end of its list are not [used](https://www.w3.org/TR/css-cascade-5/#used-value).

- <a id="ref-for-coordinating-list-property④"></a>

  <a id="ref-for-used-value①⓪"></a>

  If a [coordinating list property](#coordinating-list-property) has too few values specified, its value list is repeated to add more [used values](https://www.w3.org/TR/css-cascade-5/#used-value).

- <a id="ref-for-computed-value②①"></a>

  <a id="ref-for-coordinating-list-property⑤"></a>

  The [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [coordinating list properties](#coordinating-list-property) are not affected by such truncation or repetition.

## <a id="iana"></a> Appendix B: IANA Considerations

### <a id="about-invalid"></a> Registration for the `about:invalid` URL scheme

This sections defines and registers the `about:invalid` URL, in accordance with the registration procedure defined in [\[RFC6694\]](#biblio-rfc6694).

The official record of this registration can be found at [http&#x3A;&#x2F;&#x2F;www&#x2E;iana&#x2E;org&#x2F;assignments&#x2F;about-uri-tokens&#x2F;about-uri-tokens&#x2E;xhtml](http://www.iana.org/assignments/about-uri-tokens/about-uri-tokens.xhtml)&#x2E;

|                           |                                                                                                                                                                                                             |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Registered Token          | `invalid`                                                                                                                                                                                           |
| Intended Usage            | The `about:invalid` URL references a non-existent document with a generic error condition. It can be used when a URL is necessary, but the default value shouldn’t be resolvable as any type of document. |
| Contact/Change controller | CSS WG \<<www-style@w3.org>\> (on behalf of W3C)                                                                                                                                                            |
| Specification             | [CSS Values and Units Module Level 3](https://www.w3.org/TR/css3-values/)                                                                                                                                   |

## <a id="deprecated-quirky-length"></a> Appendix C: Quirky Lengths

<a id="ref-for-concept-document-quirks"></a>

<a id="ref-for-typedef-quirky-length"></a>

<a id="ref-for-length-value④②"></a>

When CSS is being parsed in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), <a id="typedef-quirky-length"></a>[\<quirky-length\>](#typedef-quirky-length) is a type of [\<length\>](#length-value) that is only valid in certain properties:

- <a id="ref-for-propdef-background-position⑤"></a>

  [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)

- <a id="ref-for-propdef-border-spacing"></a>

  [border-spacing](https://www.w3.org/TR/CSS21/tables.html#propdef-border-spacing)

- <a id="ref-for-propdef-border-top-width"></a>

  [border-top-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width)

- <a id="ref-for-propdef-border-right-width"></a>

  [border-right-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width)

- <a id="ref-for-propdef-border-bottom-width"></a>

  [border-bottom-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width)

- <a id="ref-for-propdef-border-left-width"></a>

  [border-left-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width)

- <a id="ref-for-propdef-border-width⑤"></a>

  [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)

- <a id="ref-for-propdef-bottom"></a>

  [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)

- <a id="ref-for-propdef-clip"></a>

  [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip)

- <a id="ref-for-propdef-font-size⑧"></a>

  [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size)

- <a id="ref-for-propdef-height②"></a>

  [height](https://www.w3.org/TR/css-sizing-3/#propdef-height)

- <a id="ref-for-propdef-left"></a>

  [left](https://www.w3.org/TR/css-position-3/#propdef-left)

- <a id="ref-for-propdef-letter-spacing"></a>

  [letter-spacing](https://www.w3.org/TR/css-text-4/#propdef-letter-spacing)

- <a id="ref-for-propdef-margin-right"></a>

  [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right)

- <a id="ref-for-propdef-margin-left"></a>

  [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left)

- <a id="ref-for-propdef-margin-top"></a>

  [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top)

- <a id="ref-for-propdef-margin-bottom"></a>

  [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom)

- <a id="ref-for-propdef-margin"></a>

  [margin](https://www.w3.org/TR/css-box-4/#propdef-margin)

- <a id="ref-for-propdef-max-height"></a>

  [max-height](https://www.w3.org/TR/css-sizing-3/#propdef-max-height)

- <a id="ref-for-propdef-max-width"></a>

  [max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width)

- <a id="ref-for-propdef-min-height"></a>

  [min-height](https://www.w3.org/TR/css-sizing-3/#propdef-min-height)

- <a id="ref-for-propdef-min-width①"></a>

  [min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width)

- <a id="ref-for-propdef-padding-top①"></a>

  [padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)

- <a id="ref-for-propdef-padding-right"></a>

  [padding-right](https://www.w3.org/TR/css-box-4/#propdef-padding-right)

- <a id="ref-for-propdef-padding-bottom"></a>

  [padding-bottom](https://www.w3.org/TR/css-box-4/#propdef-padding-bottom)

- <a id="ref-for-propdef-padding-left"></a>

  [padding-left](https://www.w3.org/TR/css-box-4/#propdef-padding-left)

- <a id="ref-for-propdef-padding"></a>

  [padding](https://www.w3.org/TR/css-box-4/#propdef-padding)

- <a id="ref-for-propdef-right"></a>

  [right](https://www.w3.org/TR/css-position-3/#propdef-right)

- <a id="ref-for-propdef-text-indent"></a>

  [text-indent](https://www.w3.org/TR/css-text-4/#propdef-text-indent)

- <a id="ref-for-propdef-top"></a>

  [top](https://www.w3.org/TR/css-position-3/#propdef-top)

- <a id="ref-for-propdef-vertical-align"></a>

  [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align)

- <a id="ref-for-propdef-width①⑥"></a>

  [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)

- <a id="ref-for-propdef-word-spacing"></a>

  [word-spacing](https://www.w3.org/TR/css-text-4/#propdef-word-spacing)

<a id="ref-for-propdef-background①"></a>

<a id="ref-for-functional-notation⑦"></a>

<a id="ref-for-funcdef-calc②③"></a>

<a id="ref-for-funcdef-basic-shape-rect"></a>

<a id="ref-for-propdef-clip①"></a>

It is <em>not</em> valid in properties that include or reference these properties, such as the [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) shorthand, or inside [functional notations](#functional-notation) such as [calc()](#funcdef-calc), except that they must be allowed in [rect()](https://www.w3.org/TR/css-shapes-1/#funcdef-basic-shape-rect) in the [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip) property.

<a id="ref-for-typedef-quirky-length①"></a>

<a id="ref-for-length-value④③"></a>

<a id="ref-for-at-ruledef-supports"></a>

<a id="ref-for-dom-css-supports-conditiontext"></a>

Additionally, while [\<quirky-length\>](#typedef-quirky-length) must be valid as a [\<length\>](#length-value) when parsing the affected properties in the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule, it is <em>not</em> valid for those properties when used in the <code><a href="https://www.w3.org/TR/css-conditional-3/#dom-css-supports-conditiontext">CSS.supports()</a></code> method.

<a id="ref-for-typedef-quirky-length②"></a>

<a id="ref-for-typedef-number-token④"></a>

<a id="ref-for-px①④"></a>

A [\<quirky-length\>](#typedef-quirky-length) is syntactically identical to a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), and is interpreted as a [px](#px) length with the same value.

<a id="ref-for-px①⑤"></a>

(In other words, Quirks Mode allows all [px](#px) lengths in the affected properties to be written without a unit, similar to unitless zero lengths.)

## <a id="acknowledgments"></a> Acknowledgments

Firstly, the editors would like to thank all of the contributors to the [previous level](https://www.w3.org/TR/css-values-3/#acknowledgments) of this module.

Secondly, we would like to acknowledge Anthony Frehner, Emilio Cobos Álvarez, Koji Ishii, Noam Rosenthal, and Xidorn Quan for their comments and suggestions, which have improved Level 4.

## <a id="changes"></a> Changes

### <a id="changes-recent"></a> Recent Changes

(This is a subset of [Additions Since Level 3](#additions-L3).)

Substantial changes since [18 December 2023 WD](https://www.w3.org/TR/2023/WD-css-values-4-20231218/):

- <a id="ref-for-valdef-clamp-none"></a>

  <a id="ref-for-funcdef-clamp①⓪"></a>

  Added the [none](#valdef-clamp-none) values to [clamp()](#funcdef-clamp), ([Issue 9713](https://github.com/w3c/csswg-drafts/issues/9713))

- Generally fixed how type inference handles percentages. ([Issue 10017](https://github.com/w3c/csswg-drafts/issues/10017))

- <a id="ref-for-viewport-percentage-lengths⑥"></a>

  <a id="ref-for-propdef-overflow④"></a>

  <a id="ref-for-propdef-scrollbar-gutter①"></a>

  <a id="ref-for-initial-containing-block④"></a>

  Restored dependency of [viewport-percentage lengths](#viewport-percentage-lengths) on [overflow: scroll](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) and added one on [scrollbar-gutter](https://www.w3.org/TR/css-overflow-3/#propdef-scrollbar-gutter) to make it possible for 100 of these units to actually match the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block). ([Issue 6026](https://github.com/w3c/csswg-drafts/issues/6026))

- <a id="ref-for-funcdef-round⑤"></a>

  <a id="ref-for-number-value⑤⑤"></a>

  Allow B to be omitted in [round()](#funcdef-round) if the A’s type is [\<number\>](#number-value). ([Issue 9668](https://github.com/w3c/csswg-drafts/issues/9668))

Substantial changes since [27 October 2023 WD](https://www.w3.org/TR/2023/WD-css-values-4-20231027/):

- <a id="ref-for-default-viewport-percentage-units"></a>

  <a id="ref-for-large-viewport-percentage-units③"></a>

  Pinned the [default viewport-percentage units](#default-viewport-percentage-units) to the [large viewport-percentage units](#large-viewport-percentage-units)—​despite violation of the “avoid dataloss by default principle”—​given existing interoperability and presumed Web-compat restriction. ([Issue 6452](https://github.com/w3c/csswg-drafts/issues/6454))

- <a id="ref-for-css-grammar-production-block"></a>

  Added an explicit definition for the [CSS grammar production block](#css-grammar-production-block) convention. ([Issue 2921](https://github.com/w3c/csswg-drafts/issues/2921))

- Clarified character encoding of percent-encoded URLs. ([Issue 9301](https://github.com/w3c/csswg-drafts/issues/9301))

Substantial changes since [6 April 2023 WD](https://www.w3.org/TR/2023/WD-css-values-4-20230406/):

- <a id="ref-for-funcdef-mix"></a>

  Punted [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix) to [\[css-values-5\]](#biblio-css-values-5). ([Issue 9343](https://github.com/w3c/csswg-drafts/issues/9343))

- Color type defined to be non-additive. ([Issue 8576](https://github.com/w3c/csswg-drafts/issues/8576))

- <a id="ref-for-math-function⑤①"></a>

  Non-property contexts treat [math functions](#math-function) as specified values. ([Issue 7964](https://github.com/w3c/csswg-drafts/issues/7964))

- Specified that URLs from CSS are always transmitted as UTF-8. ([Issue 9301](https://github.com/w3c/csswg-drafts/issues/9301))

- <a id="ref-for-addition①⑥"></a>

  <a id="ref-for-accumulation⑤"></a>

  Fixed [addition](#addition)/[accumulation](#accumulation) to use the \*second\* value, when the two values aren’t additive/accumulative. ([Issue 9070](https://github.com/w3c/csswg-drafts/issues/9070))

- <a id="ref-for-font-relative-length⑥"></a>

  Specified that [font-relative lengths](#font-relative-length) are always resolved against the parent element when used in a font-\* property. ([Issue 8169](https://github.com/w3c/csswg-drafts/issues/8169))

- <a id="ref-for-funcdef-min①⓪"></a>

  <a id="ref-for-funcdef-max⑨"></a>

  Simplify away single-argument [min()](#funcdef-min) and [max()](#funcdef-max) functions. ([Issue 9559](https://github.com/w3c/csswg-drafts/issues/9559))

Substantial changes since [19 October 2022 WD](https://www.w3.org/TR/2022/WD-css-values-4-20221019/):

- <a id="ref-for-functional-notation⑧"></a>

  Added [§ 2.6 Functional Notation Definitions](#component-functions) to formally define the way that [functional notation](#functional-notation) syntaxes are defined. ([Issue 2921](https://github.com/w3c/csswg-drafts/issues/2921))

- <a id="ref-for-snap-a-length-as-a-border-width"></a>

  Added algorithm for [snap as a border width](#snap-a-length-as-a-border-width), to reflect the interoperable rules for rendering consistent stroke widths. ([Issue 5210](https://github.com/w3c/csswg-drafts/issues/5210))

- <a id="ref-for-funcdef-mix①"></a>

  <a id="ref-for-computed-value②②"></a>

  Clarified grammar and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix). ([Issue 8096](https://github.com/w3c/csswg-drafts/issues/8096))

- <a id="ref-for-funcdef-tan⑥"></a>

  Undefined the behavior of [tan()](#funcdef-tan) at the asymptote values. ([Issue 8527](https://github.com/w3c/csswg-drafts/issues/8527))

- <a id="ref-for-resolution-value⑨"></a>

  Specified that negative [\<resolution\>](#resolution-value) values are out-of-range by definition. ([Issue 8532](https://github.com/w3c/csswg-drafts/issues/8532))

- <a id="ref-for-funcdef-mix②"></a>

  Clarified that fully omitted [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix) arguments are valid. ([Issue 8556](https://github.com/w3c/csswg-drafts/issues/8556))

- <a id="ref-for-top-level-calculation⑥"></a>

  Clarified that range clamping happens specifically to [top-level calculations](#top-level-calculation) (rather than the unclear term "expressions"). ([Issue 8158](https://github.com/w3c/csswg-drafts/issues/8158))

- <a id="ref-for-url-value①④"></a>

  <a id="ref-for-funcdef-src②"></a>

  <a id="ref-for-funcdef-url①②"></a>

  Added formal definitions for [\<url()\>](#funcdef-url) and [\<src()\>](#funcdef-src) (in addition to [\<url\>](#url-value)).

- Rephrased fragment-only URLs in terms of tree-scoped references. ([Issue 3320](https://github.com/w3c/csswg-drafts/issues/3320))

Substantial changes since [16 December 2021 WD](https://www.w3.org/TR/2021/WD-css-values-4-20211216/):

- <a id="ref-for-concept-node-tree②"></a>

  <a id="ref-for-url-local-url-flag②"></a>

  <a id="ref-for-funcdef-url①③"></a>

  Changed resolution of a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) to reference the current [node tree](https://dom.spec.whatwg.org/#concept-node-tree) (regardless of document base URL modifications). ([Issue 3320](https://github.com/w3c/csswg-drafts/issues/3320))

- <a id="ref-for-math-function⑤②"></a>

  <a id="ref-for-valdef-calc-nan④"></a>

  Switched censoring of [NaN](#valdef-calc-nan) that escapes a [math function](#math-function) from infinity to zero. ([Issue 7067](https://github.com/w3c/csswg-drafts/issues/7067))

- Added [Appendix A: Coordinating List-Valued Properties](#linked-properties) to allow this property pattern to be easily referenced. ([Issue 7164](https://github.com/w3c/csswg-drafts/issues/7164))

- <a id="ref-for-funcdef-mix③"></a>

  Restricted [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix) to be the sole value of a declaration. ([Issue 6700](https://github.com/w3c/csswg-drafts/issues/6700))

- Updated to match latest Fetch terminology. ([Fetch PR 1413](https://github.com/whatwg/fetch/pull/1413), [CSS PR 7160](https://github.com/w3c/csswg-drafts/pull/7160))

- <a id="ref-for-font-relative-length⑦"></a>

  Clarified that the [font-relative lengths](#font-relative-length) are calculated without text shaping.

- Defined serialization of empty urls to be `url("")`. ([Issue 6447](https://github.com/w3c/csswg-drafts/issues/6447))

- <a id="ref-for-specified-value⑦"></a>

  <a id="ref-for-typedef-position①⑤"></a>

  Defined serialization of [\<position\>](#typedef-position) [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value). ([Issue 2274](https://github.com/w3c/csswg-drafts/issues/2274))

- <a id="ref-for-number③"></a>

  Fixed definition of [numbers](#number) to allow decimals in combination with scientific notation, as originally intended and as defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). ([Issue 7248](https://github.com/w3c/csswg-drafts/issues/7248))

- <a id="ref-for-cssnumericvalue-type②⑧"></a>

  Corrected various functions to return an empty map for their [type](https://drafts.css-houdini.org/css-typed-om-1/#cssnumericvalue-type) instead of «\[ "number" → 1 \]». ([Issue 7486](https://github.com/w3c/csswg-drafts/issues/7486))

- <a id="ref-for-rlh⑤"></a>

  <a id="ref-for-lh⑥"></a>

  <a id="ref-for-propdef-line-height①④"></a>

  Clarified effect of special UA restrictions on [line-height](https://drafts.csswg.org/css2/#propdef-line-height) on [lh](#lh) and [rlh](#rlh). ([Issue 3257](https://github.com/w3c/csswg-drafts/issues/3257))

- Defined `<function()>` notation to refer to functional notations. ([Issue 5728](https://github.com/w3c/csswg-drafts/issues/5728))

Substantial changes since [16 October 2021 WD](https://www.w3.org/TR/2021/WD-css-values-4-20211016/):

- <a id="ref-for-writing-mode"></a>

  Switched \*vi and \*vb units to resolve against the computed [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element itself. ([Issue 6873](https://github.com/w3c/csswg-drafts/issues/6873))

- Added [§ 4.5.4 URL Processing Model](#url-processing) to define integration with CORS, etc. ([Issue 562](https://github.com/w3c/csswg-drafts/issues/562))

- <a id="ref-for-viewport-percentage-lengths⑦"></a>

  Fixed the inverted assignment of [viewport-percentage length](#viewport-percentage-lengths) behaviors to types of interface changes (A vs. B).

  > - Changes in interface that happen as a result of scrolling or other frequent page interactions that would disturb the user if they resulted in substantial layout changes must be categorized as the ~~former (A)~~ <u>latter (B)</u> .
  > - Changes in interface that have a sufficiently steady state that re-laying out the document into the adjusted space would be beneficial to the user must be categorized as the ~~latter (B)~~ <u>former (A)</u> .

- <a id="ref-for-funcdef-calc②④"></a>

  Defined minimum number of [calc()](#funcdef-calc) terms, arguments, and nesting as 32. ([Issue 3462](https://github.com/w3c/csswg-drafts/issues/3462))

- <a id="ref-for-valdef-calc-nan⑤"></a>

  Defined that mod(-0, infinity) returns [NaN](#valdef-calc-nan). ([Issue 4723](https://github.com/w3c/csswg-drafts/issues/4723))

- <a id="ref-for-funcdef-attr"></a>

  <a id="ref-for-funcdef-toggle"></a>

  Deferred [toggle()](https://drafts.csswg.org/css-values-5/#funcdef-toggle) and [attr()](https://drafts.csswg.org/css-values-5/#funcdef-attr) to Level 5.

Changes since [30 September 2021 WD](https://www.w3.org/TR/2021/WD-css-values-4-20210930/):

- <a id="ref-for-ric①"></a>

  <a id="ref-for-rch①"></a>

  <a id="ref-for-rcap①"></a>

  <a id="ref-for-rex①"></a>

  Added [rex](#rex), [rcap](#rcap), [rch](#rch), and [ric](#ric) units.

- <a id="ref-for-funcdef-mix④"></a>

  <a id="ref-for-funcdef-toggle①"></a>

  Switched [toggle()](https://drafts.csswg.org/css-values-5/#funcdef-toggle) to use semicolons, matching with [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix). ([Issue 6701](https://github.com/w3c/csswg-drafts/issues/6701))

- <a id="ref-for-funcdef-calc②⑤"></a>

  Fixed some wording errors in the definition of [calc()](#funcdef-calc). ([Issue 6506](https://github.com/w3c/csswg-drafts/issues/6506))

- <a id="ref-for-typedef-quirky-length③"></a>

  Imported definition of [\<quirky-length\>](#typedef-quirky-length) from [\[QUIRKS\]](#biblio-quirks). ([Issue 6100](https://github.com/w3c/csswg-drafts/issues/6100))

Changes since [7 July 2021 WD](https://www.w3.org/TR/2021/WD-css-values-4-20210715/):

- <a id="ref-for-funcdef-mix⑤"></a>

  Added [mix()](https://drafts.csswg.org/css-values-5/#funcdef-mix) notation for representing interpolated values.

- <a id="ref-for-length-value④④"></a>

  <a id="ref-for-percentage-value③⑦"></a>

  <a id="ref-for-number-value⑤⑥"></a>

  <a id="ref-for-integer-value①⑥"></a>

  Defined generically the computation of [\<integer\>](#integer-value), [\<number\>](#number-value), [\<percentage\>](#percentage-value), and [\<length\>](#length-value).

- <a id="ref-for-valdef-width-auto①"></a>

  Clarified that only non-zero lengths create a percentage+length mix that switches table cells to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) sizing.

Changes since [11 November 2020 WD](https://www.w3.org/TR/2020/WD-css-values-4-20201111/):

- Updated interpolation of colors to reference [\[CSS-COLOR-4\]](#biblio-css-color-4) instead of [\[CSS-COLOR-3\]](#biblio-css-color-3).

- <a id="ref-for-dynamic-viewport-percentage-units②"></a>

  <a id="ref-for-dvmax"></a>

  <a id="ref-for-dvmin"></a>

  <a id="ref-for-dvb"></a>

  <a id="ref-for-dvi"></a>

  <a id="ref-for-dvw"></a>

  <a id="ref-for-dvh"></a>

  <a id="ref-for-large-viewport-percentage-units④"></a>

  <a id="ref-for-lvmax"></a>

  <a id="ref-for-lvmin"></a>

  <a id="ref-for-lvb"></a>

  <a id="ref-for-lvi"></a>

  <a id="ref-for-lvw"></a>

  <a id="ref-for-lvh"></a>

  <a id="ref-for-small-viewport-percentage-units②"></a>

  <a id="ref-for-svmax"></a>

  <a id="ref-for-svmin"></a>

  <a id="ref-for-svb"></a>

  <a id="ref-for-svi"></a>

  <a id="ref-for-svw"></a>

  <a id="ref-for-svh"></a>

  Added the [svh](#svh), [svw](#svw), [svi](#svi), [svb](#svb), [svmin](#svmin), and [svmax](#svmax) [small viewport-percentage units](#small-viewport-percentage-units); [lvh](#lvh), [lvw](#lvw), [lvi](#lvi), [lvb](#lvb), [lvmin](#lvmin), and [lvmax](#lvmax) [large viewport-percentage units](#large-viewport-percentage-units); and [dvh](#dvh), [dvw](#dvw), [dvi](#dvi), [dvb](#dvb), [dvmin](#dvmin), and [dvmax](#dvmax) [dynamic viewport-percentage units](#dynamic-viewport-percentage-units). ([Issue 4329](https://github.com/w3c/csswg-drafts/issues/4329) and [Issue 6113](https://github.com/w3c/csswg-drafts/issues/6113))

- <a id="ref-for-angle-value①⑦"></a>

  Clamped excessively large [\<angle\>](#angle-value) values to multiples of 360deg. ([Issue 6105](https://github.com/w3c/csswg-drafts/issues/6105))

- Added back [rules on range-checking combined values](#combining-range) lost during move from the [CSS Transitions](https://www.w3.org/TR/css-transitions-1/) specification. ([Issue 6097](https://github.com/w3c/csswg-drafts/issues/6097))

- <a id="ref-for-font-relative-length⑧"></a>

  <a id="ref-for-propdef-font-size⑨"></a>

  Specified that UA-imposed minimum font sizes apply to the used [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) and not to resolution of [font-relative lengths](#font-relative-length). ([Issue 5858](https://github.com/w3c/csswg-drafts/issues/5858))

- <a id="ref-for-funcdef-max①⓪"></a>

  <a id="ref-for-funcdef-min①①"></a>

  Clarified how [min()](#funcdef-min) and [max()](#funcdef-max) percentages can partially simplify. ([Issue 6293](https://github.com/w3c/csswg-drafts/issues/6298))

### <a id="additions-L3"></a> Additions Since Level 3

Changes since [CSS Values and Units Level 3](https://www.w3.org/TR/css-values-3/):

- Explicitly undefined numeric precision/range.
- Added rules for interpolation per value type, and their clarified computed values.
- Updated interpolation of colors to reference [\[CSS-COLOR-4\]](#biblio-css-color-4).

Additions since [CSS Values and Units Level 3](https://www.w3.org/TR/css-values-3/):

- <a id="ref-for-typedef-dashed-ident①⓪"></a>

  Defined the [\<dashed-ident\>](#typedef-dashed-ident) type.

- <a id="ref-for-ratio-value①③"></a>

  Defined the [\<ratio\>](#ratio-value) type.

- <a id="ref-for-url-value①⑤"></a>

  <a id="ref-for-funcdef-src③"></a>

  Added [src()](#funcdef-src) to the [\<url\>](#url-value) type.

- <a id="ref-for-rlh⑥"></a>

  <a id="ref-for-lh⑦"></a>

  <a id="ref-for-cap④"></a>

  <a id="ref-for-ic④"></a>

  <a id="ref-for-vb①"></a>

  <a id="ref-for-vi①"></a>

  Added the [vi](#vi), [vb](#vb), [ic](#ic), [cap](#cap), [lh](#lh) and [rlh](#rlh) length units.

- <a id="ref-for-dynamic-viewport-percentage-units③"></a>

  <a id="ref-for-dvmax①"></a>

  <a id="ref-for-dvmin①"></a>

  <a id="ref-for-dvb①"></a>

  <a id="ref-for-dvi①"></a>

  <a id="ref-for-dvw①"></a>

  <a id="ref-for-dvh①"></a>

  <a id="ref-for-small-viewport-percentage-units③"></a>

  <a id="ref-for-svmax①"></a>

  <a id="ref-for-svmin①"></a>

  <a id="ref-for-svb①"></a>

  <a id="ref-for-svi①"></a>

  <a id="ref-for-svw①"></a>

  <a id="ref-for-svh①"></a>

  Added the [svh](#svh), [svw](#svw), [svi](#svi), [svb](#svb), [svmin](#svmin), and [svmax](#svmax) [small viewport-percentage units](#small-viewport-percentage-units) and [dvh](#dvh), [dvw](#dvw), [dvi](#dvi), [dvb](#dvb), [dvmin](#dvmin), and [dvmax](#dvmax) [dynamic viewport-percentage units](#dynamic-viewport-percentage-units).

- <a id="ref-for-dppx①"></a>

  <a id="ref-for-x"></a>

  Added the [x](#x) alias to [dppx](#dppx).

- <a id="ref-for-funcdef-clamp①①"></a>

  <a id="ref-for-funcdef-max①①"></a>

  <a id="ref-for-funcdef-min①②"></a>

  Added [min()](#funcdef-min), [max()](#funcdef-max), and [clamp()](#funcdef-clamp) [comparison functions](#comp-func).

- <a id="ref-for-funcdef-sign④"></a>

  <a id="ref-for-funcdef-abs④"></a>

  <a id="ref-for-funcdef-exp④"></a>

  <a id="ref-for-funcdef-log⑤"></a>

  <a id="ref-for-funcdef-hypot⑧"></a>

  <a id="ref-for-funcdef-sqrt⑥"></a>

  <a id="ref-for-funcdef-pow⑥"></a>

  <a id="ref-for-funcdef-atan2④"></a>

  <a id="ref-for-funcdef-atan⑦"></a>

  <a id="ref-for-funcdef-acos⑤"></a>

  <a id="ref-for-funcdef-asin⑤"></a>

  <a id="ref-for-funcdef-tan⑦"></a>

  <a id="ref-for-funcdef-cos⑤"></a>

  <a id="ref-for-funcdef-sin⑥"></a>

  <a id="ref-for-funcdef-rem①①"></a>

  <a id="ref-for-funcdef-mod⑧"></a>

  <a id="ref-for-funcdef-round⑥"></a>

  Added [round()](#funcdef-round), [mod()](#funcdef-mod), [rem()](#funcdef-rem), [sin()](#funcdef-sin), [cos()](#funcdef-cos), [tan()](#funcdef-tan), [asin()](#funcdef-asin), [acos()](#funcdef-acos), [atan()](#funcdef-atan), [atan2()](#funcdef-atan2), [pow()](#funcdef-pow), [sqrt()](#funcdef-sqrt), [hypot()](#funcdef-hypot), [log()](#funcdef-log), [exp()](#funcdef-exp), [abs()](#funcdef-abs), [sign()](#funcdef-sign) math functions.

- <a id="ref-for-funcdef-calc②⑥"></a>

  <a id="ref-for-valdef-calc-nan⑥"></a>

  <a id="ref-for-valdef-calc--infinity③"></a>

  <a id="ref-for-valdef-calc-infinity④"></a>

  <a id="ref-for-valdef-calc-pi①"></a>

  <a id="ref-for-valdef-calc-e①"></a>

  Added [e](#valdef-calc-e), [pi](#valdef-calc-pi), [infinity](#valdef-calc-infinity), [-infinity](#valdef-calc--infinity), [NaN](#valdef-calc-nan) constants for use in [calc()](#funcdef-calc).

- <a id="ref-for-dimension①②"></a>

  <a id="ref-for-funcdef-calc②⑦"></a>

  Added [unit algebra](#calc-type-checking) to [calc()](#funcdef-calc), allowing multiplication and division of [dimensions](#dimension).

- <a id="ref-for-integer-value①⑦"></a>

  A non-integer in a calc() automatically rounds to the nearest integer when used where an [\<integer\>](#integer-value) is required.

- <a id="ref-for-math-function⑤③"></a>

  Defined [serialization](#calc-serialize) of [math functions](#math-function).

- <a id="ref-for-propdef-background②"></a>

  <a id="ref-for-coordinating-list-property⑥"></a>

  Added a genericized definition of [coordinating list property groups](#coordinating-list-property), to make it easier to reference the coordinating behavior of the [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) properties.

## <a id="security"></a> Security Considerations

This specification presents no new security considerations.

<a id="ref-for-funcdef-url①④"></a>

<a id="ref-for-funcdef-src④"></a>

<a id="ref-for-url-value①⑥"></a>

This specification defines the [url()](#funcdef-url) and [src()](#funcdef-src) functions ([\<url\>](#url-value)), which allow CSS to make network requests. Depending on what features they are used in, these can potentially expose whether or not the user has access to resources on a network, and expose information about their contents (such as the rules within a style sheet, the size of an image, the metrics of a font). They can also allow exfiltrating data via URL.

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-viewport-percentage-lengths⑧"></a>

<a id="ref-for-font-relative-length⑨"></a>

This specification defines units that expose the user’s screen size (the [viewport-percentage lengths](#viewport-percentage-lengths)), default font size, and potentially some information about which fonts are available on the user’s system (the [font-relative lengths](#font-relative-length)).

<a id="ref-for-funcdef-url①⑤"></a>

<a id="ref-for-funcdef-src⑤"></a>

<a id="ref-for-url-value①⑦"></a>

This specification defines the [url()](#funcdef-url) and [src()](#funcdef-src) functions ([\<url\>](#url-value)), which allow CSS to make network requests. Depending on what features they are used in, these can potentially expose whether or not the user has access to resources on a network, and expose information about their contents (such as the rules within a style sheet, the size of an image, the metrics of a font). They can also allow exfiltrating data via URL.

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

- [!](#mult-req), in § 2.3
- [\#](#mult-comma), in § 2.3
- [&#x26;&#x26;](#comb-all), in § 2.2
- [\*](#mult-zero-plus), in § 2.3
- [+](#mult-one-plus), in § 2.3
- [,](#comb-comma), in § 2.1
- [?](#mult-opt), in § 2.3
- [\|](#comb-one), in § 2.2
- [\|\|](#comb-any), in § 2.2
- [{A}](#mult-num), in § 2.3
- [{A,B}](#mult-num-range), in § 2.3
- [abs()](#funcdef-abs), in § 10.6
- [absolute length](#absolute-length), in § 6.2
- [absolute length unit](#absolute-length), in § 6.2
- [accumulate](#accumulation), in § 3
- [accumulation](#accumulation), in § 3
- [accumulation procedure](#accumulation), in § 3
- [acos()](#funcdef-acos), in § 10.4
- [add](#addition), in § 3
- [addition](#addition), in § 3
- [addition procedure](#addition), in § 3
- [advance measure](#length-advance-measure), in § 6.1.1
- [anchor](#anchor-unit), in § 6.2
- [anchor unit](#anchor-unit), in § 6.2
- [\<angle\>](#angle-value), in § 7.1
- [\<angle-percentage\>](#typedef-angle-percentage), in § 5.6
- [asin()](#funcdef-asin), in § 10.4
- [atan()](#funcdef-atan), in § 10.4
- [atan2()](#funcdef-atan2), in § 10.4
- [bearing angle](#bearing-angle), in § 7.1
- [between zero and B](#between-zero-and-b), in § 10.3
- [bracketed range notation](#css-bracketed-range-notation), in § 5.1
- [calc()](#funcdef-calc), in § 10.1
- [\<calc-keyword\>](#typedef-calc-keyword), in § 10.8
- [calc-operator nodes](#calculation-tree-calc-operator-nodes), in § 10.10
- [\<calc-product\>](#typedef-calc-product), in § 10.8
- [\<calc-sum\>](#typedef-calc-sum), in § 10.8
- [calculation](#calc-calculation), in § 10.1
- [calculation tree](#calculation-tree), in § 10.10
- [\<calc-value\>](#typedef-calc-value), in § 10.8
- [canonical](#canonical-unit), in § 5.4.1
- [canonical unit](#canonical-unit), in § 5.4.1
- [cap](#cap), in § 6.1.1
- [ch](#ch), in § 6.1.1
- [clamp()](#funcdef-clamp), in § 10.2
- [cm](#cm), in § 6.2
- [combine](#combine), in § 3
- [compatible](#compatible-units), in § 5.4.1
- [compatible units](#compatible-units), in § 5.4.1
- [computed length](#computed-length), in § 6
- [consistent](#css-consistent-type), in § 10.9
- [consistent type](#css-consistent-type), in § 10.9
- [contain a percentage](#css-contain-a-percentage), in § 10.9
- [coordinated value list](#coordinated-value-list), in § Unnumbered section
- [coordinating list base property](#coordinating-list-base-property), in § Unnumbered section
- [coordinating list property](#coordinating-list-property), in § Unnumbered section
- [coordinating list property group](#coordinating-list-property), in § Unnumbered section
- [cos()](#funcdef-cos), in § 10.4
- [CSS bracketed range notation](#css-bracketed-range-notation), in § 5.1
- [CSS grammar production block](#css-grammar-production-block), in § 2.8
- [CSS ident](#css-css-identifier), in § 4
- [CSS identifier](#css-css-identifier), in § 4
- [CSS-wide keywords](#css-wide-keywords), in § 4.1.1
- [\<custom-ident\>](#identifier-value), in § 4.2
- [\<dashed-ident\>](#typedef-dashed-ident), in § 4.3
- [default viewport-percentage units](#default-viewport-percentage-units), in § 6.1.2.1
- [deg](#deg), in § 7.1
- [degenerate ratio](#degenerate-ratio), in § 5.7
- [determine the type of a calculation](#determine-the-type-of-a-calculation), in § 10.9
- [device pixel](#device-pixel), in § 6.2
- [\<dimension\>](#typedef-dimension), in § 5.4
- [dimension](#dimension), in § 5.4
- [down](#valdef-rounding-strategy-down), in § 10.3
- [dpcm](#dpcm), in § 7.4
- [dpi](#dpi), in § 7.4
- [dppx](#dppx), in § 7.4
- [dvb](#dvb), in § 6.1.2.2
- [dvh](#dvh), in § 6.1.2.2
- [dvi](#dvi), in § 6.1.2.2
- [dvmax](#dvmax), in § 6.1.2.2
- [dvmin](#dvmin), in § 6.1.2.2
- [dvw](#dvw), in § 6.1.2.2
- [dynamic viewport-percentage units](#dynamic-viewport-percentage-units), in § 6.1.2.1
- [dynamic viewport size](#dynamic-viewport-size), in § 6.1.2.1
- [e](#valdef-calc-e), in § 10.7.1
- [em](#em), in § 6.1.1
- [ex](#ex), in § 6.1.1
- [exp()](#funcdef-exp), in § 10.5
- [fetch a style resource](#fetch-a-style-resource), in § 4.5.4
- [font-relative lengths](#font-relative-length), in § 6.1.1
- [\<frequency\>](#frequency-value), in § 7.3
- [\<frequency-percentage\>](#typedef-frequency-percentage), in § 5.6
- [functional notation](#functional-notation), in § 9
- [grad](#grad), in § 7.1
- [hypot()](#funcdef-hypot), in § 10.5
- [Hz](#Hz), in § 7.3
- [ic](#ic), in § 6.1.1
- [\<ident\>](#typedef-ident), in § 4
- [ident](#css-css-identifier), in § 4
- [identifier](#css-css-identifier), in § 4
- [in](#in), in § 6.2
- [infinite](#css-infinity), in § 10.9.1
- [-infinity](#valdef-calc--infinity), in § 10.7.2
- infinity
  - [dfn for CSS](#css-infinity), in § 10.9.1
  - [value for calc()](#valdef-calc-infinity), in § 10.7.2
- [\<integer\>](#integer-value), in § 5.2
- [integer](#integer), in § 5.2
- [interpolate](#interpolation), in § 3
- [interpolation](#interpolation), in § 3
- [interpolation procedure](#interpolation), in § 3
- [keyword](#css-keyword), in § 4.1
- [kHz](#kHz), in § 7.3
- [large viewport-percentage units](#large-viewport-percentage-units), in § 6.1.2.1
- [large viewport size](#large-viewport-size), in § 6.1.2.1
- [\<length\>](#length-value), in § 6
- [\<length-percentage\>](#typedef-length-percentage), in § 5.6
- [lh](#lh), in § 6.1.1
- [local font-relative lengths](#local-font-relative-lengths), in § 6.1.1
- [local url flag](#url-local-url-flag), in § 4.5.1.1
- [log()](#funcdef-log), in § 10.5
- [lvb](#lvb), in § 6.1.2.2
- [lvh](#lvh), in § 6.1.2.2
- [lvi](#lvi), in § 6.1.2.2
- [lvmax](#lvmax), in § 6.1.2.2
- [lvmin](#lvmin), in § 6.1.2.2
- [lvw](#lvw), in § 6.1.2.2
- [made consistent](#css-make-a-type-consistent), in § 10.9
- [make a type consistent](#css-make-a-type-consistent), in § 10.9
- [make consistent](#css-make-a-type-consistent), in § 10.9
- [math function](#math-function), in § 10
- [max()](#funcdef-max), in § 10.2
- [min()](#funcdef-min), in § 10.2
- [mm](#mm), in § 6.2
- [mod()](#funcdef-mod), in § 10.3
- [ms](#ms), in § 7.2
- NaN
  - [dfn for CSS](#css-nan), in § 10.9.1
  - [value for calc()](#valdef-calc-nan), in § 10.7.2
- [nearest](#valdef-rounding-strategy-nearest), in § 10.3
- [negative infinity](#css-infinity), in § 10.9.1
- [negative zero](#css-signed-zero), in § 10.9.1
- [none](#valdef-clamp-none), in § 10.2
- [not additive](#not-additive), in § 3
- [\<number\>](#number-value), in § 5.3
- [number](#number), in § 5.3
- [numeric data types](#numeric-data-types), in § 5
- [operator nodes](#calculation-tree-operator-nodes), in § 10.10
- [parse a calculation](#parse-a-calculation), in § 10.10
- [parsing a calculation](#parse-a-calculation), in § 10.10
- [pc](#pc), in § 6.2
- [\<percentage\>](#percentage-value), in § 5.5
- [percentage](#percentage), in § 5.5
- [percentage-containing](#css-contain-a-percentage), in § 10.9
- [physical unit](#physical-unit), in § 6.2
- [pi](#valdef-calc-pi), in § 10.7.1
- [pixel unit](#visual-angle-unit), in § 6.2
- [\<position\>](#typedef-position), in § 8.3
- [positive infinity](#css-infinity), in § 10.9.1
- [positive zero](#css-signed-zero), in § 10.9.1
- [pow()](#funcdef-pow), in § 10.5
- [pt](#pt), in § 6.2
- [px](#px), in § 6.2
- [Q](#Q), in § 6.2
- [\<quirky-length\>](#typedef-quirky-length), in § Unnumbered section
- [rad](#rad), in § 7.1
- [\<ratio\>](#ratio-value), in § 5.7
- [ratio](#ratio), in § 5.7
- [rcap](#rcap), in § 6.1.1
- [rch](#rch), in § 6.1.1
- [reference pixel](#reference-pixel), in § 6.2
- [relative length](#relative-length), in § 6.1
- [relative length unit](#relative-length), in § 6.1
- [rem](#rem), in § 6.1.1
- [rem()](#funcdef-rem), in § 10.3
- [\<resolution\>](#resolution-value), in § 7.4
- [rex](#rex), in § 6.1.1
- [ric](#ric), in § 6.1.1
- [rlh](#rlh), in § 6.1.1
- [root font-relative lengths](#root-font-relative-lengths), in § 6.1.1
- [round()](#funcdef-round), in § 10.3
- [\<rounding-strategy\>](#typedef-rounding-strategy), in § 10.3
- [round to the nearest integer](#css-round-to-the-nearest-integer), in § 5.2
- [s](#s), in § 7.2
- [serialize a calculation tree](#serialize-a-calculation-tree), in § 10.13
- [serialize a math function](#serialize-a-math-function), in § 10.13
- [serialize the calculation tree](#serialize-a-calculation-tree), in § 10.13
- [serializing a calculation tree](#serialize-a-calculation-tree), in § 10.13
- [serializing the calculation tree](#serialize-a-calculation-tree), in § 10.13
- [sign()](#funcdef-sign), in § 10.6
- [signed zero](#css-signed-zero), in § 10.9.1
- [simplify](#simplify-a-calculation-tree), in § 10.10.1
- [simplify a calculation tree](#simplify-a-calculation-tree), in § 10.10.1
- [simplifying a calculation tree](#simplify-a-calculation-tree), in § 10.10.1
- [sin()](#funcdef-sin), in § 10.4
- [small viewport-percentage units](#small-viewport-percentage-units), in § 6.1.2.1
- [small viewport size](#small-viewport-size), in § 6.1.2.1
- [snap a length as a border width](#snap-a-length-as-a-border-width), in § 6
- [snap as a border width](#snap-a-length-as-a-border-width), in § 6
- [sort a calculation’s children](#sort-a-calculations-children), in § 10.13
- [specified length](#specified-length), in § 6
- [sqrt()](#funcdef-sqrt), in § 10.5
- [src()](#funcdef-src), in § 4.5
- [\<string\>](#string-value), in § 4.4
- [svb](#svb), in § 6.1.2.2
- [svh](#svh), in § 6.1.2.2
- [svi](#svi), in § 6.1.2.2
- [svmax](#svmax), in § 6.1.2.2
- [svmin](#svmin), in § 6.1.2.2
- [svw](#svw), in § 6.1.2.2
- [tan()](#funcdef-tan), in § 10.4
- [textual data types](#css-textual-data-types), in § 4
- [\<time\>](#time-value), in § 7.2
- [\<time-percentage\>](#typedef-time-percentage), in § 5.6
- [top-level calculation](#top-level-calculation), in § 10.9.1
- [to-zero](#valdef-rounding-strategy-to-zero), in § 10.3
- [turn](#turn), in § 7.1
- [up](#valdef-rounding-strategy-up), in § 10.3
- [\<url\>](#url-value), in § 4.5
- [url()](#funcdef-url), in § 4.5
- [\<url-modifier\>](#typedef-url-modifier), in § 4.5.3
- [URL request modifier steps](#url-request-modifier-steps), in § 4.5.4
- [value accumulation](#accumulation), in § 3
- [value addition](#addition), in § 3
- [value definition syntax](#css-value-definition-syntax), in § 2
- [value interpolation](#interpolation), in § 3
- [vb](#vb), in § 6.1.2.2
- [vh](#vh), in § 6.1.2.2
- [vi](#vi), in § 6.1.2.2
- [viewport-percentage lengths](#viewport-percentage-lengths), in § 6.1.2
- [visual angle unit](#visual-angle-unit), in § 6.2
- [vmax](#vmax), in § 6.1.2.2
- [vmin](#vmin), in § 6.1.2.2
- [vw](#vw), in § 6.1.2.2
- [x](#x), in § 7.4
- [\<zero\>](#zero-value), in § 5.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-2023\] defines the following terms:
  - <a id="861626b1"></a>ua
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="1e12dba3"></a>animation
  - <a id="5e7cfa4f"></a>animation-iteration-count
  - <a id="04beed9b"></a>animation-name
  - <a id="79dcf364"></a>animation-timing-function
- \[CSS-BOX-4\] defines the following terms:
  - <a id="253362bb"></a>margin
  - <a id="ba64c9f5"></a>margin-bottom
  - <a id="836161df"></a>margin-left
  - <a id="76c02e00"></a>margin-right
  - <a id="58404105"></a>margin-top
  - <a id="a3a070bd"></a>padding
  - <a id="c8473aa4"></a>padding-bottom
  - <a id="c22cb630"></a>padding-left
  - <a id="0a9e7084"></a>padding-right
  - <a id="f72f0a02"></a>padding-top
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="4f75e4ec"></a>orphans
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
  - <a id="0948355d"></a>actual value
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="762bad34"></a>initial
  - <a id="980ac56a"></a>shorthand property
  - <a id="d5e08d9c"></a>specified value
  - <a id="7c39b465"></a>unset
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="5bd3632a"></a>hsl()
  - <a id="3b7558dc"></a>opacity
  - <a id="f3226176"></a>rgba()
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
  - <a id="2b36ebcb"></a>@color-profile
  - <a id="b786b3c1"></a>relative color
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="a5d6c9d2"></a>@supports
  - <a id="087858ba"></a>supports(conditionText)
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="a9efccbc"></a>disc
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="6b4fc208"></a>containing block
  - <a id="e26aa9bf"></a>initial containing block
  - <a id="8b4f8a45"></a>root element
- \[CSS-EASING-1\] defines the following terms:
  - <a id="c7d3b8b7"></a>\<easing-function\>
  - <a id="2f7ff51f"></a>ease-in
  - <a id="056b67db"></a>ease-out
  - <a id="887acc42"></a>easing function
  - <a id="e444e13f"></a>timing function
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="11a985a9"></a>font
  - <a id="7066562d"></a>font-family
  - <a id="297dfe3a"></a>font-size
- \[CSS-FONTS-5\] defines the following terms:
  - <a id="b24ce65e"></a>@font-face
  - <a id="ee74cde8"></a>font-size
- \[CSS-GRID-2\] defines the following terms:
  - <a id="b1382fd7"></a>\<flex\>
  - <a id="ca502323"></a>fr
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="5663abb5"></a>image-resolution
  - <a id="dcb1125e"></a>linear-gradient()
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="80a12c2a"></a>normal
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="e97d95b6"></a>clip
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="91e8e72e"></a>scrollbar-gutter
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="072f08da"></a>max-lines
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="1fe10a71"></a>page area
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="f411d42d"></a>bottom
  - <a id="ebcbc56d"></a>left
  - <a id="a5bae6ee"></a>right
  - <a id="f99d4ae2"></a>top
- \[CSS-RHYTHM-1\] defines the following terms:
  - <a id="03e2765f"></a>block-step-size
- \[CSS-SCOPING-1\] defines the following terms:
  - <a id="778f8dbd"></a>tree-scoped name
  - <a id="c304f669"></a>tree-scoped reference
- \[CSS-SHAPES-1\] defines the following terms:
  - <a id="4f29b6e5"></a>rect()
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="54a1fea8"></a>box-sizing
  - <a id="5ad01cca"></a>height
  - <a id="2d68423f"></a>max-height
  - <a id="f4066072"></a>max-width
  - <a id="f12a84ab"></a>min-height
  - <a id="4ed06a81"></a>min-width
  - <a id="49731d1d"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="4920620f"></a>\<any-value\>
  - <a id="f9309bf7"></a>\<delim-token\>
  - <a id="87de393a"></a>\<dimension-token\>
  - <a id="a6331414"></a>\<function-token\>
  - <a id="446c663e"></a>\<ident-token\>
  - <a id="eebbfe3d"></a>\<number-token\>
  - <a id="8a73a2e3"></a>\<percentage-token\>
  - <a id="17fe01a1"></a>\<string-token\>
  - <a id="30679d85"></a>\<url-token\>
  - <a id="5c2618b5"></a>\<whitespace-token\>
  - <a id="267b6766"></a>component value
  - <a id="cdce34f5"></a>consume a url token
  - <a id="b3fe0edf"></a>simple block
  - <a id="fa522ba4"></a>whitespace
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="36e5f32e"></a>text-align
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="291965b2"></a>center
  - <a id="02bd7a8b"></a>letter-spacing
  - <a id="beb6807a"></a>tab-size
  - <a id="1e5ca230"></a>text-indent
  - <a id="a22cd86a"></a>word-spacing
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="4d38e4c5"></a>text-decoration
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="31452ed5"></a>transform-origin
- \[CSS-TYPED-OM-1\] defines the following terms:
  - <a id="6c3d0564"></a>add two types
  - <a id="9a34f8c4"></a>internal representation
  - <a id="5fcfec98"></a>invert a type
  - <a id="2fafca4a"></a>match
  - <a id="76d3715b"></a>multiply two types
  - <a id="0cac57ed"></a>percent hint
  - <a id="d7d4206f"></a>type
- \[CSS-UI-4\] defines the following terms:
  - <a id="6ed19243"></a>outline-color
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="57699934"></a>attr()
  - <a id="ceb4e9b9"></a>mix()
  - <a id="2ef1aa7d"></a>toggle()
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="5550667d"></a>custom property
  - <a id="3beec8c9"></a>var()
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="a6eb24bb"></a>inline axis
  - <a id="8664e85f"></a>text-orientation
  - <a id="cec0d4db"></a>upright
  - <a id="35f596e9"></a>vertical-lr
  - <a id="ee88ce59"></a>vertical-rl
  - <a id="eb6008ce"></a>writing mode
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="17f0c439"></a>\<border-width\>
  - <a id="29ca9f85"></a>border-collapse
  - <a id="fe0abd77"></a>border-spacing
- \[CSS22\] defines the following terms:
  - <a id="ac54fbff"></a>line-height
- \[CSS3-BACKGROUND\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="8218676f"></a>background-attachment
  - <a id="5ced56d0"></a>background-image
  - <a id="f2249e38"></a>background-position
  - <a id="bb899432"></a>border-bottom-width
  - <a id="65b3a7bc"></a>border-color
  - <a id="aaf0980c"></a>border-left-width
  - <a id="47e9abf9"></a>border-right-width
  - <a id="051410c7"></a>border-top-width
  - <a id="064303ba"></a>border-width
  - <a id="c48eaa20"></a>box-shadow
  - <a id="44e4312c"></a>center
- \[CSS3-IMAGES\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
- \[CSSOM\] defines the following terms:
  - <a id="102f21b5"></a>CSSStyleSheet
  - <a id="d321b0e7"></a>location
  - <a id="0c5a4350"></a>origin-clean flag
  - <a id="c9f23acd"></a>stylesheet base url
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document
  - <a id="1cd7ff31"></a>ShadowRoot
  - <a id="c7d8d91b"></a>node tree
  - <a id="fd11cdcd"></a>quirks mode
  - <a id="25538777"></a>tree order
- \[FETCH\] defines the following terms:
  - <a id="b4c5b3f7"></a>RequestDestination
  - <a id="857d5516"></a>client
  - <a id="902380f7"></a>credentials mode
  - <a id="3ae34c95"></a>destination
  - <a id="a33db89a"></a>fetch
  - <a id="7b0b4fa0"></a>initiator type
  - <a id="cb98f71f"></a>mode
  - <a id="f78d5b5c"></a>origin
  - <a id="08b51e40"></a>processresponseconsumebody
  - <a id="358f1dbd"></a>referrer
  - <a id="55213b5b"></a>request
  - <a id="ee7bba09"></a>response
  - <a id="dc1cd39b"></a>url
  - <a id="44d0e8a7"></a>use-url-credentials flag
- \[HTML\] defines the following terms:
  - <a id="1b5b1c0c"></a>api base url
  - <a id="c847111b"></a>base
  - <a id="24905be8"></a>find a potential indicated element
  - <a id="43ac8374"></a>origin
  - <a id="9c4c1e66"></a>relevant settings object
  - <a id="c51ad875"></a>the body element
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ascii case-insensitive
  - <a id="915aff5e"></a>code point
  - <a id="4a3bf5fb"></a>concatenate
  - <a id="16d07e10"></a>for each
  - <a id="91c016b5"></a>identical to
  - <a id="860300d4"></a>implementation-defined
  - <a id="0698d556"></a>string
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="9e462db5"></a>continuous media
  - <a id="3ea2fcbb"></a>media query
  - <a id="23af89d0"></a>paged media
- \[URL\] defines the following terms:
  - <a id="ed948033"></a>fragment
  - <a id="96bd090d"></a>percent-encode after encoding
  - <a id="dcffbccd"></a>url
  - <a id="ca3ca4ae"></a>url parser
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="2cec4673"></a>discrete

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-2023"></a>\[CSS-2023\]  
Chris Lilley; et al. [CSS Snapshot 2023](https://www.w3.org/TR/css-2023/). 7 December 2023. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2023&#x2F;](https://www.w3.org/TR/css-2023/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 28 June 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
David Baron; Elika Etemad; Chris Lilley. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 27 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 6 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 1 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 3 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-scoping-1"></a>\[CSS-SCOPING-1\]  
Tab Atkins Jr.; Elika Etemad. [CSS Scoping Module Level 1](https://www.w3.org/TR/css-scoping-1/). 3 April 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-scoping-1&#x2F;](https://www.w3.org/TR/css-scoping-1/)

<a id="biblio-css-shapes-1"></a>\[CSS-SHAPES-1\]  
Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 15 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 20 October 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-typed-om-1"></a>\[CSS-TYPED-OM-1\]  
Shane Stephens; Tab Atkins Jr.; Naina Raisinghani. [CSS Typed OM Level 1](https://www.w3.org/TR/css-typed-om-1/). 10 April 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-typed-om-1&#x2F;](https://www.w3.org/TR/css-typed-om-1/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
[CSS Values and Units Module Level 5](https://drafts.csswg.org/css-values-5/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-5&#x2F;](https://drafts.csswg.org/css-values-5/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-css3-background"></a>\[CSS3-BACKGROUND\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 19 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3-fonts"></a>\[CSS3-FONTS\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-unicode"></a>\[UNICODE\]  
[The Unicode Standard](https://www.unicode.org/versions/latest/). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;unicode&#x2E;org&#x2F;versions&#x2F;latest&#x2F;](https://www.unicode.org/versions/latest/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-3"></a>\[CSS-CASCADE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css-color-3"></a>\[CSS-COLOR-3\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 18 January 2022. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-rhythm-1"></a>\[CSS-RHYTHM-1\]  
Koji Ishii; Elika Etemad. [CSS Rhythmic Sizing](https://www.w3.org/TR/css-rhythm-1/). 2 March 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-rhythm-1&#x2F;](https://www.w3.org/TR/css-rhythm-1/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 3 September 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-quirks"></a>\[QUIRKS\]  
Simon Pieters. [Quirks Mode Standard](https://quirks.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;quirks&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://quirks.spec.whatwg.org/)

<a id="biblio-rfc6694"></a>\[RFC6694\]  
S. Moonesamy, Ed.. [The "about" URI Scheme](https://www.rfc-editor.org/rfc/rfc6694). August 2012. Informational. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc6694](https://www.rfc-editor.org/rfc/rfc6694)

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Possibly reference [find a potential indicated element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#find-a-potential-indicated-element), but that is defined specifically for <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>s, not <code><a href="https://dom.spec.whatwg.org/#shadowroot">ShadowRoot</a></code>s. [↵](#issue-f83e4845)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> CSSOM needs to specify how it rounds, and it’s probably good for CSS functions to round the same way by default. What behavior should be used? [\[Issue \#5689\]](https://github.com/w3c/csswg-drafts/issues/5689) [↵](#issue-1a929fd0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is still [under discussion](https://lists.w3.org/Archives/Member/w3c-css-wg/2016AprJun/0239.html). [↵](#issue-f5bc4b00)

CanIUse

<b>Support:</b>Android Browser2.1+Baidu Browser13.18+Blackberry Browser7+Chrome4+Chrome for Android122+Edge12+Firefox3.6+Firefox for Android123+IE11+IE Mobile10+KaiOS Browser2.5+Opera11.6+Opera MiniAllOpera Mobile12+QQ Browser13.1+Safari5+Safari on iOS6.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=rem) as of 2024-03-07

CanIUse

<b>Support:</b>Android Browser4.4+Baidu Browser13.18+Blackberry Browser10+Chrome27+Chrome for Android122+Edge12+Firefox2+Firefox for Android123+IE (limited)9+IE Mobile10+KaiOS Browser2.5+Opera15+Opera MiniNoneOpera Mobile73+QQ Browser13.1+Safari7+Safari on iOS7.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=ch-unit) as of 2024-03-07

CanIUse

<b>Support:</b>Android Browser4.4+Baidu Browser13.18+Blackberry Browser10+Chrome26+Chrome for Android122+Edge16+Firefox19+Firefox for Android123+IE (limited)9+IE Mobile (limited)10+KaiOS Browser2.5+Opera15+Opera MiniNoneOpera Mobile73+QQ Browser13.1+Safari6.1+Safari on iOS8+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=viewport-units) as of 2024-03-07

CanIUse

<b>Support:</b>Android Browser122+Baidu Browser13.18+Blackberry Browser10+Chrome26+Chrome for Android122+Edge12+Firefox16+Firefox for Android123+IE (limited)9+IE Mobile10+KaiOS Browser2.5+Opera15+Opera MiniNoneOpera Mobile73+QQ Browser13.1+Safari6.1+Safari on iOS7.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=calc) as of 2024-03-07
