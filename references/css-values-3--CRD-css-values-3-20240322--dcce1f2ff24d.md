Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Values and Units Module Level 3](https://www.w3.org/TR/2024/CRD-css-values-3-20240322/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Values and Units Module Level 3

Source snapshot: https://www.w3.org/TR/2024/CRD-css-values-3-20240322/

Snapshot SHA-256: dcce1f2ff24d66a05373696e07dda59727303a421ee4593eb433092029df0094

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 5 source tables are presented as readable Markdown tables or explicit labeled layouts: 3 ordinary table conversions, 2 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Values and Units Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module describes the common values and units that CSS properties accept and the syntax used for describing them in CSS property definitions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

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
    > The top-level multiplier is ripped out of these value types because top-level comma-separated repetitions are mostly used for [coordinating list properties](https://www.w3.org/TR/css-values-4/#coordinating-list-property), and when a shorthand combines several such properties, it needs the unmultiplied grammar so it can construct its <em>own</em> comma-separated repetition.
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

|                     | in order          | any order         |
|---------------------|-------------------|-------------------|
| <strong>zero or more &#xA;      </strong> | <code>A?&#x20;B?&#x20;C?</code> | <code>A?&#x20;&#x7C;&#x7C;&#x20;B?&#x20;&#x7C;&#x7C;&#x20;C?</code> |
| <strong>one or more &#xA;      </strong> | <code><c->&#x5B;</c->&#x20;A?&#x20;B?&#x20;C?&#x20;<c->&#x5D;</c->!</code> | <code>A&#x20;&#x7C;&#x7C;&#x20;B&#x20;&#x7C;&#x7C;&#x20;C</code> |
| <strong>all &#xA;      </strong> | <code>A&#x20;B&#x20;C&#x20;</code> | <code>A&#x20;&amp;&amp;&#x20;B&#x20;&amp;&amp;&#x20;C</code> |

Note that all of the "any order" possibilities are expressed using combinators, while the "in order" possibilities are all variants on juxtaposition.

### <a id="component-whitespace"></a>2.5.  Component Values and White Space

Unless otherwise specified, [white space](https://www.w3.org/TR/css-syntax/#whitespace) and/or comments may appear before, after, and/or between components combined using the above [combinators](#component-combinators) and [multipliers](#component-multipliers).

<a id="ref-for-typedef-dimension-token"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In many cases, spaces will in fact be <em>required</em> between components in order to distinguish them from each other. For example, the value 1em2em would be parsed as a single [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) with the number 1 and the identifier em2em, which is an invalid unit. In this case, a space would be required before the 2 to get this parsed as the two lengths 1em and 2em.

### <a id="value-examples"></a>2.6.  Property Value Examples

Below are some examples of properties with their corresponding value definition fields

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-78954521"></a>
>
> <a id="propvalues"></a>
>
> | Property                                                                                              | Value definition field                                                                                         | Example value                                                                           |
> |-------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
> | <a id="ref-for-propdef-orphans"></a>[orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans)                      | \<integer\>                                                                                                    | 3                                                                                       |
> | <a id="ref-for-propdef-text-align"></a>[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align)                 | left \| right \| center \| justify                                                                             | <a id="ref-for-valdef-text-align-center"></a>[center](https://www.w3.org/TR/css-text-4/#valdef-text-align-center) |
> | <a id="ref-for-propdef-padding-top"></a>[padding-top](https://www.w3.org/TR/css-box-4/#propdef-padding-top)                | \<length\> \| \<percentage\>                                                                                   | 5%                                                                                      |
> | <a id="ref-for-propdef-outline-color"></a>[outline-color](https://www.w3.org/TR/css-ui-4/#propdef-outline-color)             | \<color\> \| invert                                                                                            | \#fefefe                                                                                |
> | <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration) | none \| underline \|\| overline \|\| line-through \|\| blink                                                   | overline underline                                                                      |
> | <a id="ref-for-propdef-font-family"></a>[font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family)              | \[ \<family-name\> \| \<generic-family\> \]#                                                                   | "Gill Sans", Futura, sans-serif                                                         |
> | <a id="ref-for-propdef-border-width③"></a>[border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width)      | \[ \<length\> \| thick \| medium \| thin \]{1,4}                                                               | 2px medium 4px                                                                          |
> | <a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://www.w3.org/TR/css-backgrounds-3/#propdef-box-shadow)          | \[ inset? &#x26;&#x26; \<length\>{2,4} &#x26;&#x26; \<color\>? \]# \| none | 3px 3px rgba(50%, 50%, 50%, 50%), lemonchiffon 0 0 4px inset                            |

## <a id="textual-values"></a>3.  Textual Data Types

<a id="ref-for-string-value"></a>

<a id="ref-for-url-value"></a>

The <a id="css-textual-data-types"></a>textual data types include various keywords and identifiers as well as strings ([\<string\>](#string-value)) and URLs ([\<url\>](#url-value)).

<a id="ref-for-typedef-ident-token"></a>

<a id="ref-for-css-css-identifier"></a>

CSS <a id="css-css-identifier"></a>identifiers, generically denoted by <a id="typedef-ident"></a>\<ident\>, consist of a sequence of characters conforming to the [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token) grammar. [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3) Identifiers cannot be quoted; otherwise they would be interpreted as strings. CSS properties accept two classes of [identifiers](#css-css-identifier): [pre-defined keywords](#keywords) and [author-defined identifiers](#custom-idents).

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-identifier-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<ident\>](#typedef-ident) production is not meant for property value definitions—​[\<custom-ident\>](#identifier-value) should be used instead. It is provided as a convenience for defining other syntactic constructs.

### <a id="keywords"></a>3.1.  Pre-defined Keywords

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

#### <a id="common-keywords"></a>3.1.1.  CSS-wide keywords: [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial), [inherit](https://www.w3.org/TR/css-cascade-5/#valdef-all-inherit) and [unset](https://www.w3.org/TR/css-cascade-5/#valdef-all-unset)

As defined [above](#component-types), all properties accept the <a id="css-wide-keywords"></a>CSS-wide keywords, which represent value computations common to all CSS properties. These keywords are normatively defined in the [CSS Cascading and Inheritance Module](https://www.w3.org/TR/css-cascade/#defaulting-keywords).

Other CSS specifications can define additional CSS-wide keywords.

<a id="ref-for-identifier-value④"></a>

### <a id="custom-idents"></a>3.2.  Author-defined Identifiers: the [\<custom-ident\>](#identifier-value) type

<a id="ref-for-css-css-identifier②"></a>

Some properties accept arbitrary author-defined identifiers as a component value. This generic data type is denoted by <a id="identifier-value"></a>\<custom-ident\>, and represents any valid [CSS identifier](#css-css-identifier) that would not be misinterpreted as a pre-defined keyword in that property’s value definition. Such identifiers are fully case-sensitive, even in the ASCII range (e.g. example and EXAMPLE are two different, unrelated user-defined identifiers).

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

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When designing grammars with [\<custom-ident\>](#identifier-value), the <a id="ref-for-identifier-value①②"></a>\<custom-ident\> should always be “positionally unambiguous”, so that it’s impossible to conflict with any keyword values in the property.

<a id="ref-for-string-value①"></a>

### <a id="strings"></a>3.3.  Quoted Strings: the [\<string\>](#string-value) type

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

### <a id="urls"></a>3.4.  Resource Locators: the [\<url\>](#url-value) type

<a id="ref-for-functional-notation①"></a>

<a id="ref-for-url-value②"></a>

The <a id="funcdef-url"></a>url() [functional notation](#functional-notation), denoted by [\<url\>](#url-value), represents a <a id="url"></a>URL, which is a pointer to a resource. The typical syntax of a <a id="ref-for-url-value③"></a>\<url\> is:

<a id="url-value"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-typedef-url-modifier"></a>

<a id="ref-for-mult-zero-plus"></a>

```text
<url> = url( <string> <url-modifier>* )
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1dcaf1ec"></a> This example shows a URL being used as a background image:
>
> ```text
> body { background: url("http://www.example.com/pinkish.gif") }
> ```
<a id="ref-for-funcdef-url"></a>

<a id="ref-for-consume-a-url-token"></a>

<a id="ref-for-typedef-url-token"></a>

A [url()](#funcdef-url) can alternatively be written be written without quotation marks around the URL value, in which case it is [specially-parsed](https://www.w3.org/TR/css-syntax-3/#consume-a-url-token) as a [\<url-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-url-token); see [CSS Syntax 3 § 4.3.6 Consume a url token](https://www.w3.org/TR/css-syntax-3/#consume-url-token). [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2e04cf61"></a> For example, the following declarations are identical:
>
> ```text
> background: url("http://www.example.com/pinkish.gif");
> background: url(http://www.example.com/pinkish.gif);
> ```
<a id="ref-for-funcdef-url①"></a>

<a id="ref-for-typedef-url-modifier①"></a>

<a id="ref-for-string-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The unquoted [url()](#funcdef-url) syntax cannot accept a [\<url-modifier\>](#typedef-url-modifier) argument and has extra escaping requirements: parentheses, [whitespace](https://www.w3.org/TR/css-syntax/#whitespace) characters, single quotes (') and double quotes (") appearing in a URL must be escaped with a backslash, e.g. url(open&#x5C;(parens), url(close&#x5C;)parens). (In quoted [\<string\>](#string-value) <a id="ref-for-funcdef-url②"></a>url()s, only newlines and the character used to quote the string need to be escaped.) Depending on the type of URL, it might also be possible to write these characters as URL-escapes (e.g. url(open%28parens) or url(close%29parens)) as described in [\[URL\]](#biblio-url).

<a id="ref-for-at-ruledef-import"></a>

<a id="ref-for-url-value④"></a>

<a id="ref-for-string-value④"></a>

<a id="ref-for-funcdef-url③"></a>

Some CSS contexts (such as [@import](https://www.w3.org/TR/css-cascade-5/#at-ruledef-import)) also allow a [\<url\>](#url-value) to be represented by a bare [\<string\>](#string-value), without the function wrapper. In such cases the string behaves identically to a [url()](#funcdef-url) function containing that string.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae720903"></a> For example, the following statements act identically:
>
> ```text
> @import url("base-theme.css");
> @import "base-theme.css";
> ```
#### <a id="relative-urls"></a>3.4.1.  Relative URLs

In order to create modular style sheets that are not dependent on the absolute location of a resource, authors should use relative URLs. Relative URLs (as defined in [\[URL\]](#biblio-url)) are resolved to full URLs using a base URL. RFC 3986, section 3, defines the normative algorithm for this process. For CSS style sheets, the base URL is that of the style sheet itself, not that of the styled source document. Style sheets embedded within a document have the base URL associated with their container.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For HTML documents, the [base URL is mutable](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#dynamic-changes-to-base-urls).

<a id="ref-for-url-value⑤"></a>

<a id="ref-for-user-agent①"></a>

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

##### <a id="local-urls"></a>3.4.1.1.  Fragment URLs

To work around some common eccentricities in browser URL handling, CSS has special behavior for fragment-only urls.

<a id="ref-for-funcdef-url④"></a>

If a [url()](#funcdef-url)’s value starts with a U+0023 NUMBER SIGN (`#`) character, parse it as per normal for URLs, but additionally set the <a id="url-local-url-flag"></a>local url flag of the <a id="ref-for-funcdef-url⑤"></a>url().

<a id="ref-for-funcdef-url⑥"></a>

<a id="ref-for-url-local-url-flag"></a>

When matching a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) set, ignore everything but the URL’s fragment, and resolve that fragment against the current document that relative URLs are resolved against. This reference must always be treated as same-document (rather than cross-document).

<a id="ref-for-funcdef-url⑦"></a>

<a id="ref-for-url-local-url-flag①"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) a [url()](#funcdef-url) with the [local url flag](#url-local-url-flag) set, it must serialize as just the fragment.

> <strong data-conversion-semantic="note">Note</strong>
>
> What “browser eccentricities”?
>
> <a id="ref-for-the-base-element"></a>
>
> <a id="ref-for-dom-history-pushstate"></a>
>
> Theoretically, browsers should re-resolve any relative URLs, including fragment-only URLs, whenever the document’s base URL changes (such as through mutation of the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-base-element">base</a></code> element, or calling <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#dom-history-pushstate">pushState()</a></code>). In many cases they don’t, however, and so without special handling, fragment-only URLs will suddenly become cross-document references (pointing at the previous base URL) and break in many of the places they’re used.
>
> Since fragment-only URLs express a clear semantic of wanting to refer to the current document regardless of what its current URL is, this hack preserves the expected behavior at least in these cases.

#### <a id="url-empty"></a>3.4.2.  Empty URLs

<a id="ref-for-url-value⑥"></a>

<a id="ref-for-funcdef-url⑧"></a>

If the value of the [\<url\>](#url-value) is the empty string (like url("") or [url()](#funcdef-url)), the url must resolve to an invalid resource (similar to what the url about:invalid does).

Its computed value is url(""), and it must serialize as such.

<a id="ref-for-funcdef-url⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This matches the behavior of empty urls for embedded resources elsewhere in the web platform, and avoids excess traffic re-requesting the stylesheet or host document due to editing mistakes leaving the [url()](#funcdef-url) value empty, which are almost certain to be invalid resources for whatever the <a id="ref-for-funcdef-url①⓪"></a>url() shows up in. Linking on the web platform <em>does</em> allow empty urls, so if/when CSS gains some functionality to control hyperlinks, this restriction can be relaxed in those contexts.

#### <a id="url-modifiers"></a>3.4.3.  URL Modifiers

<a id="ref-for-funcdef-url①①"></a>

<a id="ref-for-typedef-url-modifier②"></a>

<a id="ref-for-typedef-ident①"></a>

<a id="ref-for-functional-notation②"></a>

The [url()](#funcdef-url) function supports specifying additional <a id="typedef-url-modifier"></a>\<url-modifier\>s, which change the meaning or the interpretation of the URL somehow. A [\<url-modifier\>](#typedef-url-modifier) is either an [\<ident\>](#typedef-ident) or a [functional notation](#functional-notation).

<a id="ref-for-typedef-url-modifier③"></a>

This specification does not define any [\<url-modifier\>](#typedef-url-modifier)s, but other specs may do so.

<a id="ref-for-url-value⑦"></a>

<a id="ref-for-funcdef-url①②"></a>

<a id="ref-for-typedef-url-modifier④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [\<url\>](#url-value) that is either unquoted or not wrapped in [url()](#funcdef-url) notation cannot accept any [\<url-modifier\>](#typedef-url-modifier)s.

## <a id="numeric-types"></a>4.  Numeric Data Types

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value"></a>

Numeric data types are used to represent quantities, indexes, positions, and other such values. Although many syntactic variations can exist in expressing the quantity (numeric aspect) in a given numeric value, the [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) do not distinguish these variations: they represent the value’s abstract quantity, not its syntactic representation.

<a id="ref-for-integer-value③"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-dimension"></a>

<a id="ref-for-length-value②"></a>

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

### <a id="numeric-ranges"></a>4.1.  Range Restrictions and Range Definition Notation

<a id="ref-for-integer-value④"></a>

<a id="ref-for-angle-value①"></a>

Properties can restrict numeric values to some range. If the value is outside the allowed range, then unless otherwise specified, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore). Range restrictions can be annotated in the numeric type notation using <a id="css-bracketed-range-notation"></a>CSS bracketed range notation—​<code><c->&#x5B;</c-><var>min</var><c->,</c-><var>max</var><c->&#x5D;</c-></code>—​within the angle brackets, after the identifying keyword, indicating a closed range between (and including) <var>min</var> and <var>max</var>. For example, [\<integer \[0,10\]\>](#integer-value) indicates an integer between 0 and 10, inclusive, while [\<angle \[0,180deg\]\>](#angle-value) indicates an angle between 0deg and 180deg (expressed in any unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS values generally do not allow open ranges; thus only square-bracket notation is used.

<a id="ref-for-user-agent②"></a>

<a id="ref-for-length-value③"></a>

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

### <a id="integers"></a>4.2.  Integers: the [\<integer\>](#integer-value) type

Integer values are denoted by <a id="integer-value"></a>\<integer\>.

<a id="ref-for-typedef-number-token"></a>

When written literally, an <a id="integer"></a>integer is one or more decimal digits 0 through 9 and corresponds to a subset of the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the CSS Syntax Module [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). The first digit of an integer may be immediately preceded by - or + to indicate the integer’s sign.

<a id="ref-for-number-value①"></a>

### <a id="numbers"></a>4.3.  Real Numbers: the [\<number\>](#number-value) type

Number values are denoted by <a id="number-value"></a>\<number\>, and represent real numbers, possibly with a fractional component.

<a id="ref-for-integer"></a>

<a id="ref-for-typedef-number-token①"></a>

When written literally, a <a id="number"></a>number is either an [integer](#integer), or zero or more decimal digits followed by a dot (.) followed by one or more decimal digits; optionally, it can be concluded by the letter “e” or “E” followed by an integer indicating the base-ten exponent in [scientific notation](https://en.wikipedia.org/wiki/Scientific_notation). It corresponds to the [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). As with integers, the first character of a number may be immediately preceded by - or + to indicate the number’s sign.

<a id="ref-for-dimension②"></a>

### <a id="dimensions"></a>4.4.  Numbers with Units: [dimension](#dimension) values

The general term <a id="dimension"></a>dimension refers to a number with a unit attached to it; and is denoted by <a id="typedef-dimension"></a>\<dimension\>.

<a id="ref-for-dimension③"></a>

<a id="ref-for-number"></a>

<a id="ref-for-css-css-identifier③"></a>

<a id="ref-for-typedef-dimension-token①"></a>

<a id="ref-for-ascii-case-insensitive②"></a>

When written literally, a [dimension](#dimension) is a [number](#number) immediately followed by a unit identifier, which is an [identifier](#css-css-identifier). It corresponds to the [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). Like keywords, unit identifiers are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-typedef-dimension"></a>

<a id="ref-for-length-value④"></a>

<a id="ref-for-time-value②"></a>

<a id="ref-for-frequency-value①"></a>

<a id="ref-for-resolution-value①"></a>

CSS uses [\<dimension\>](#typedef-dimension)s to specify distances ([\<length\>](#length-value)), durations ([\<time\>](#time-value)), frequencies ([\<frequency\>](#frequency-value)), resolutions ([\<resolution\>](#resolution-value)), and other quantities.

#### <a id="compat"></a>4.4.1.  Compatible Units

<a id="ref-for-computed-value①"></a>

<a id="ref-for-px"></a>

<a id="ref-for-in"></a>

<a id="ref-for-propdef-font-size"></a>

<a id="ref-for-em"></a>

<a id="ref-for-canonical-unit"></a>

When [serializing](https://www.w3.org/TR/cssom-1/#serializing-css-values) [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) [\[CSSOM\]](#biblio-cssom), <a id="compatible-units"></a>compatible units (those related by a static multiplicative factor, like the 96:1 factor between [px](#px) and [in](#in), or the computed [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) factor between [em](#em) and <a id="ref-for-px①"></a>px) are converted into a single <a id="canonical-unit"></a>canonical unit. Each group of compatible units defines which among them is the [canonical unit](#canonical-unit) that will be used for serialization.

<a id="ref-for-used-value"></a>

<a id="ref-for-compatible-units"></a>

<a id="ref-for-dimension④"></a>

When serializing [resolved values](https://www.w3.org/TR/cssom-1/#resolved-values) that are [used values](https://www.w3.org/TR/css-cascade-5/#used-value), all value types (percentages, numbers, keywords, etc.) that represent lengths are considered [compatible](#compatible-units) with lengths. Likewise any future API that returns <a id="ref-for-used-value①"></a>used values must consider any values that represent distances/durations/frequencies/etc. as <a id="ref-for-compatible-units①"></a>compatible with the relevant class of [dimensions](#dimension), and canonicalize accordingly.

<a id="ref-for-percentage-value②"></a>

### <a id="percentages"></a>4.5.  Percentages: the [\<percentage\>](#percentage-value) type

Percentage values are denoted by <a id="percentage-value"></a>\<percentage\>, and indicates a value that is some fraction of another reference value.

<a id="ref-for-number①"></a>

<a id="ref-for-typedef-percentage-token"></a>

When written literally, a <a id="percentage"></a>percentage consists of a [number](#number) immediately followed by a percent sign %. It corresponds to the [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) production in the [CSS Syntax Module](https://www.w3.org/TR/css-syntax/) [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3).

<a id="ref-for-containing-block"></a>

Percentage values are always relative to another quantity, for example a length. Each property that allows percentages also defines the quantity to which the percentage refers. This quantity can be a value of another property for the same element, the value of a property for an ancestor element, a measurement of the formatting context (e.g., the width of a [containing block](https://www.w3.org/TR/css-display-3/#containing-block)), or something else.

### <a id="mixed-percentages"></a>4.6.  Mixing Percentages and Dimensions

<a id="ref-for-percentage-value③"></a>

<a id="ref-for-dimension⑤"></a>

<a id="ref-for-component-value①"></a>

<a id="ref-for-funcdef-calc①"></a>

In cases where a [\<percentage\>](#percentage-value) can represent the same quantity as a [dimension](#dimension) in the same [component value](https://www.w3.org/TR/css-syntax-3/#component-value) position, and can therefore be combined with them in a [calc()](#funcdef-calc) expression, the following convenience notations may be used in the property grammar:

<a id="typedef-length-percentage"></a>\<length-percentage\>  
<a id="ref-for-length-value⑤"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-length-value⑥"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#length-value">&lt;length&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<length\>](#length-value).

<a id="typedef-frequency-percentage"></a>\<frequency-percentage\>  
<a id="ref-for-frequency-value②"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-frequency-value③"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#frequency-value">&lt;frequency&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<frequency\>](#frequency-value).

<a id="typedef-angle-percentage"></a>\<angle-percentage\>  
<a id="ref-for-angle-value②"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-angle-value③"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#angle-value">&lt;angle&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to an [\<angle\>](#angle-value).

<a id="typedef-time-percentage"></a>\<time-percentage\>  
<a id="ref-for-time-value③"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-time-value④"></a>

Equivalent to <code><c->&#x5B;</c-> <a href="#time-value">&lt;time&gt;</a> <a href="#comb-one">|</a> <a href="#percentage-value">&lt;percentage&gt;</a> <c->&#x5D;</c-></code>, where the [\<percentage\>](#percentage-value) will resolve to a [\<time\>](#time-value).

<a id="ref-for-propdef-width"></a>

<a id="ref-for-length-value⑦"></a>

<a id="ref-for-percentage-value①②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-10c2940e"></a> For example, the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property can accept a [\<length\>](#length-value) or a [\<percentage\>](#percentage-value), both representing a measure of distance. This means that <a id="ref-for-propdef-width①"></a>width: calc(500px + 50%); is allowed—​both values are converted to absolute lengths and added. If the containing block is 1000px wide, then <a id="ref-for-propdef-width②"></a>width: 50%; is equivalent to <a id="ref-for-propdef-width③"></a>width: 500px, and <a id="ref-for-propdef-width④"></a>width: calc(50% + 500px) thus ends up equivalent to <a id="ref-for-propdef-width⑤"></a>width: calc(500px + 500px) or <a id="ref-for-propdef-width⑥"></a>width: 1000px.
>
> <a id="ref-for-funcdef-hsl"></a>
>
> <a id="ref-for-percentage-value①③"></a>
>
> <a id="ref-for-funcdef-calc②"></a>
>
> On the other hand, the second and third arguments of the [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) function can only be expressed as [\<percentage\>](#percentage-value)s. Although [calc()](#funcdef-calc) productions are allowed in their place, they can only combine percentages with themselves, as in calc(10% + 20%).

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-compatible-units②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Specifications should never alternate [\<percentage\>](#percentage-value) in place of a dimension in a grammar unless they are [compatible](#compatible-units).

<a id="ref-for-number-value②"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-funcdef-calc③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: More \<<var>type</var>-percentage\> productions can be added in the future as needed. A \<number-percentage\> will never be added, as [\<number\>](#number-value) and [\<percentage\>](#percentage-value) can’t be combined in [calc()](#funcdef-calc).

<a id="ref-for-length-value⑧"></a>

## <a id="lengths"></a>5.  Distance Units: the [\<length\>](#length-value) type

<a id="ref-for-dimension⑥"></a>

Lengths refer to distance measurements and are denoted by <a id="length-value"></a>\<length\> in the property definitions. A length is a [dimension](#dimension).

<a id="ref-for-number-value③"></a>

<a id="ref-for-length-value⑨"></a>

<a id="ref-for-propdef-line-height"></a>

For zero lengths the unit identifier is optional (i.e. can be syntactically represented as the [\<number\>](#number-value) 0). However, if a 0 could be parsed as either a <a id="ref-for-number-value④"></a>\<number\> or a [\<length\>](#length-value) in a property (such as [line-height](https://drafts.csswg.org/css2/#propdef-line-height)), it must parse as a <a id="ref-for-number-value⑤"></a>\<number\>.

Properties may restrict the length value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore).

While some properties allow negative length values, this may complicate the formatting and there may be implementation-specific limits. If a negative length value is allowed but cannot be supported, it must be converted to the nearest value that can be supported.

<a id="ref-for-used-value②"></a>

<a id="ref-for-actual-value"></a>

In cases where the [used](https://www.w3.org/TR/css-cascade-5/#used-value) length cannot be supported, user agents must approximate it in the [actual](https://www.w3.org/TR/css-cascade-5/#actual-value) value.

<a id="ref-for-relative-length"></a>

<a id="ref-for-absolute-length"></a>

There are two types of length units: [relative](#relative-length) and [absolute](#absolute-length).

### <a id="relative-lengths"></a>5.1.  Relative Lengths

<a id="relative-length"></a>Relative length units specify a length relative to another length. Style sheets that use relative units can more easily scale from one output environment to another.

The relative units are:

| unit                             | relative to                                                                                                           |
|----------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| <a id="ref-for-em①"></a>[em](#em)     | font size of the element                                                                                              |
| <a id="ref-for-ex"></a>[ex](#ex)     | x-height of the element’s font                                                                                        |
| <a id="ref-for-ch"></a>[ch](#ch)     | <a id="ref-for-length-advance-measure"></a>[character advance](#length-advance-measure) of the “0” (ZERO, U+0030) glyph in the element’s font |
| <a id="ref-for-rem"></a>[rem](#rem)   | font size of the root element                                                                                         |
| <a id="ref-for-vw"></a>[vw](#vw)     | 1% of viewport’s width                                                                                                |
| <a id="ref-for-vh"></a>[vh](#vh)     | 1% of viewport’s height                                                                                               |
| <a id="ref-for-vmin"></a>[vmin](#vmin) | 1% of viewport’s smaller dimension                                                                                    |
| <a id="ref-for-vmax"></a>[vmax](#vmax) | 1% of viewport’s larger dimension                                                                                     |

Informative Summary of Relative Units

<a id="ref-for-computed-value②"></a>

Child elements do not inherit the relative values as specified for their parent; they inherit the [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="ref-for-em②"></a>

<a id="ref-for-ex①"></a>

<a id="ref-for-ch①"></a>

<a id="ref-for-rem①"></a>

#### <a id="font-relative-lengths"></a>5.1.1.  Font-relative Lengths: the [em](#em), [ex](#ex), [ch](#ch), [rem](#rem) units

<a id="ref-for-rem②"></a>

The <a id="font-relative-length"></a>font-relative lengths refer to the font metrics of the element on which they are used—​or, in the case of [rem](#rem), the metrics of the root element.

<a id="em"></a>em  
<a id="ref-for-propdef-font-size①"></a>

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

<a id="ex"></a>ex unit  
<a id="ref-for-user-agent③"></a>

<a id="ref-for-ex②"></a>

Equal to the used x-height of the [first available font](https://www.w3.org/TR/css3-fonts/#first-available-font) [\[CSS3-FONTS\]](#biblio-css3-fonts). The x-height is so called because it is often equal to the height of the lowercase "x". However, an [ex](#ex) is defined even for fonts that do not contain an "x". The x-height of a font can be found in different ways. Some fonts contain reliable metrics for the x-height. If reliable font metrics are not available, [UAs](https://www.w3.org/TR/css-2023/#user-agent) may determine the x-height from the height of a lowercase glyph. One possible heuristic is to look at how far the glyph for the lowercase "o" extends below the baseline, and subtract that value from the top of its bounding box. In the cases where it is impossible or impractical to determine the x-height, a value of 0.5em must be assumed.

<a id="ch"></a>ch unit  
<a id="ref-for-length-advance-measure①"></a>

Represents the typical [advance measure](#length-advance-measure) of European alphanumeric characters, and measured as the used <a id="ref-for-length-advance-measure②"></a>advance measure of the “0” (ZERO, U+0030) glyph in the font used to render it. (The <a id="length-advance-measure"></a>advance measure of a glyph is its advance width or height, whichever is in the inline axis of the element.)

<a id="ref-for-length-advance-measure③"></a>

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

<a id="ref-for-media-query"></a>

<a id="ref-for-propdef-font"></a>

<a id="ref-for-propdef-font-size②"></a>

When used outside the context of an element (such as in [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query)), these units refer to the computed font metrics corresponding to the initial values of the [font](https://www.w3.org/TR/css-fonts-4/#propdef-font) property. When used in the value of the [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) property on the element they refer to, these units refer to the computed font metrics of the parent element (or the computed font metrics corresponding to the initial values of the <a id="ref-for-propdef-font①"></a>font property, if the element has no parent).

<a id="ref-for-font-relative-length"></a>

The [font-relative lengths](#font-relative-length) are calculated in the absence of shaping.

<a id="ref-for-vw①"></a>

<a id="ref-for-vh①"></a>

<a id="ref-for-vmin①"></a>

<a id="ref-for-vmax①"></a>

#### <a id="viewport-relative-lengths"></a>5.1.2.  Viewport-percentage Lengths: the [vw](#vw), [vh](#vh), [vmin](#vmin), [vmax](#vmax) units

<a id="ref-for-continuous-media"></a>

<a id="ref-for-page-area"></a>

<a id="ref-for-paged-media"></a>

The <a id="viewport-percentage-lengths"></a>viewport-percentage lengths are relative to the size of the [initial containing block](https://www.w3.org/TR/CSS2/visudet.html#containing-block-details)—​which is itself based on the size of either the viewport (for [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media)) or the [page area](https://www.w3.org/TR/css-page-3/#page-area) (for [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media)). When the height or width of the initial containing block is changed, they are scaled accordingly. However, any scrollbars are assumed not to exist.

<a id="ref-for-paged-media①"></a>

For [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), the exact definition of the viewport-percentage lengths is deferred to [\[CSS3PAGE\]](#biblio-css3page).

<a id="vw"></a>vw unit  
Equal to 1% of the width of the initial containing block.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6068ee5d"></a> In the example below, if the width of the viewport is 200mm, the font size of `h1` elements will be 16mm (i.e. (8×200mm)/100).
> ```text
> h1 { font-size: 8vw }
> ```
<a id="vh"></a>vh unit  
Equal to 1% of the height of the initial containing block.

<a id="vmin"></a>vmin unit  
<a id="ref-for-vh②"></a>

<a id="ref-for-vw②"></a>

Equal to the smaller of [vw](#vw) or [vh](#vh).

<a id="vmax"></a>vmax unit  
<a id="ref-for-vh③"></a>

<a id="ref-for-vw③"></a>

Equal to the larger of [vw](#vw) or [vh](#vh).

<a id="ref-for-cm"></a>

<a id="ref-for-mm"></a>

<a id="ref-for-Q"></a>

<a id="ref-for-in①"></a>

<a id="ref-for-pt"></a>

<a id="ref-for-pc"></a>

<a id="ref-for-px②"></a>

### <a id="absolute-lengths"></a>5.2.  Absolute Lengths: the [cm](#cm), [mm](#mm), [Q](#Q), [in](#in), [pt](#pt), [pc](#pc), [px](#px) units

<a id="ref-for-anchor-unit"></a>

<a id="ref-for-in②"></a>

<a id="ref-for-cm①"></a>

<a id="ref-for-mm①"></a>

<a id="ref-for-pt①"></a>

<a id="ref-for-pc①"></a>

<a id="ref-for-Q①"></a>

<a id="ref-for-px③"></a>

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
> Note: Lengths in publishing contexts are sometimes written like `2p3`, indicating a length of 2 picas and 3 points. These can be written in CSS as calc(2pc + 3pt) (see [§ 8.1 Mathematical Expressions: calc()](#calc-notation)).

<a id="ref-for-compatible-units③"></a>

<a id="ref-for-px④"></a>

<a id="ref-for-canonical-unit①"></a>

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

<a id="ref-for-device-pixel"></a>

For print media at typical viewing distances, the [anchor unit](#anchor-unit) should be one of the [physical units](#physical-unit) (inches, centimeters, etc). For screen media (including high-resolution devices), low-resolution devices, and devices with unusual viewing distances, it is recommended instead that the <a id="ref-for-anchor-unit②"></a>anchor unit be the [pixel unit](#visual-angle-unit). For such devices it is recommended that the <a id="ref-for-visual-angle-unit②"></a>pixel unit refer to the whole number of [device pixels](#device-pixel) that best approximates the reference pixel.

<a id="ref-for-anchor-unit③"></a>

<a id="ref-for-visual-angle-unit③"></a>

<a id="ref-for-physical-unit②"></a>

<a id="ref-for-device-pixel①"></a>

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

<a id="ref-for-device-pixel②"></a>

The <a id="reference-pixel"></a>reference pixel is the visual angle of one pixel on a device with a [device pixel](#device-pixel) density of 96dpi and a distance from the reader of an arm’s length. For a nominal arm’s length of 28 inches, the visual angle is therefore about 0.0213 degrees. For reading at arm’s length, 1px thus corresponds to about 0.26 mm (1/96 inch).

The image below illustrates the effect of viewing distance on the size of a reference pixel: a reading distance of 71 cm (28 inches) results in a reference pixel of 0.26 mm, while a reading distance of 3.5 m (12 feet) results in a reference pixel of 1.3 mm.

![This diagram illustrates how the definition of a pixel depends on the users distance from the viewing surface (paper or screen). The image depicts the user looking at two planes, one 28 inches (71 cm) from the user, the second 140 inches (3.5 m) from the user. An expanding cone is projected from the user's eye onto each plane. Where the cone strikes the first plane, the projected pixel is 0.26 mm high. Where the cone strikes the second plane, the projected pixel is 1.4 mm high.](https://www.w3.org/TR/2024/CRD-css-values-3-20240322/images/pixel1.png)

Showing that pixels must become larger if the viewing distance increases

This second image illustrates the effect of a device’s resolution on the pixel unit: an area of 1px by 1px is covered by a single dot in a low-resolution device (e.g. a typical computer display), while the same area is covered by 16 dots in a higher resolution device (such as a printer).

![This diagram illustrates the relationship between the reference pixel and device pixels (called “dots” below). The image depicts a high resolution (large dot density) laser printer output on the left and a low resolution monitor screen on the right. For the laser printer, one square reference pixel is implemented by 16 dots. For the monitor screen, one square reference pixel is implemented by a single dot.](https://www.w3.org/TR/2024/CRD-css-values-3-20240322/images/pixel2.png)

Showing that more device pixels (dots) are needed to cover a 1px by 1px area on a high-resolution device than on a lower-resolution one (of the same approximate viewing distance)

A <a id="device-pixel"></a>device pixel is the smallest unit of area on the device output capable of displaying its full range of colors. For typical color screens, it’s a square or somewhat rectangular region containing a red, green, and blue subpixel. Many non-traditional outputs exist that can blur this definition, such as by displaying some colors at higher resolutions. Such devices still expose some equivalent notion of "device pixel", however.

## <a id="other-units"></a>6.  Other Quantities

<a id="ref-for-angle-value④"></a>

<a id="ref-for-deg"></a>

<a id="ref-for-grad"></a>

<a id="ref-for-rad"></a>

<a id="ref-for-turn"></a>

### <a id="angles"></a>6.1.  Angle Units: the [\<angle\>](#angle-value) type and [deg](#deg), [grad](#grad), [rad](#rad), [turn](#turn) units

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

<a id="ref-for-angle-value⑤"></a>

<a id="ref-for-compatible-units④"></a>

<a id="ref-for-deg①"></a>

<a id="ref-for-canonical-unit②"></a>

All [\<angle\>](#angle-value) units are [compatible](#compatible-units), and [deg](#deg) is their [canonical unit](#canonical-unit).

> <strong data-conversion-semantic="note">Note</strong>
>
> By convention, when an angle denotes a direction in CSS, it is typically interpreted as a <a id="bearing-angle"></a>bearing angle, where 0deg is "up" or "north" on the screen, and larger angles are more clockwise (so 90deg is "right" or "east").
>
> <a id="ref-for-funcdef-linear-gradient"></a>
>
> <a id="ref-for-angle-value⑥"></a>
>
> For example, in the [linear-gradient()](https://www.w3.org/TR/css-images-4/#funcdef-linear-gradient) function, the [\<angle\>](#angle-value) that determines the direction of the gradient is interpreted as a bearing angle.

<a id="ref-for-angle-value⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For legacy reasons, some uses of [\<angle\>](#angle-value) allow a bare 0 to mean 0deg. This is not true in general, however, and will not occur in future uses of the <a id="ref-for-angle-value⑧"></a>\<angle\> type.

<a id="ref-for-time-value⑤"></a>

<a id="ref-for-s"></a>

<a id="ref-for-ms"></a>

### <a id="time"></a>6.2.  Duration Units: the [\<time\>](#time-value) type and [s](#s), [ms](#ms) units

<a id="ref-for-dimension⑦"></a>

Time values are [dimensions](#dimension) denoted by <a id="time-value"></a>\<time\>. The time unit identifiers are:

<a id="s"></a>s  
Seconds.

<a id="ms"></a>ms  
Milliseconds. There are 1000 milliseconds in a second.

<a id="ref-for-time-value⑥"></a>

<a id="ref-for-compatible-units⑤"></a>

<a id="ref-for-s①"></a>

<a id="ref-for-canonical-unit③"></a>

All [\<time\>](#time-value) units are [compatible](#compatible-units), and [s](#s) is their [canonical unit](#canonical-unit).

Properties may restrict the time value to some range. If the value is outside the allowed range, the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore).

<a id="ref-for-frequency-value④"></a>

<a id="ref-for-Hz"></a>

<a id="ref-for-kHz"></a>

### <a id="frequency"></a>6.3.  Frequency Units: the [\<frequency\>](#frequency-value) type and [Hz](#Hz), [kHz](#kHz) units

<a id="ref-for-dimension⑧"></a>

Frequency values are [dimensions](#dimension) denoted by <a id="frequency-value"></a>\<frequency\>. The frequency unit identifiers are:

<a id="Hz"></a>Hz  
Hertz. It represents the number of occurrences per second.

<a id="kHz"></a>kHz  
KiloHertz. A kiloHertz is 1000 Hertz.

For example, when representing sound pitches, 200Hz (or 200hz) is a bass sound, and 6kHz (or 6khz) is a treble sound.

<a id="ref-for-frequency-value⑤"></a>

<a id="ref-for-compatible-units⑥"></a>

<a id="ref-for-Hz①"></a>

<a id="ref-for-canonical-unit④"></a>

All [\<frequency\>](#frequency-value) units are [compatible](#compatible-units), and [hz](#Hz) is their [canonical unit](#canonical-unit).

<a id="ref-for-ascii-case-insensitive④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Units are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) and serialize as lowercase, for example 1Hz serializes as 1hz.

<a id="ref-for-resolution-value②"></a>

<a id="ref-for-dpi"></a>

<a id="ref-for-dpcm"></a>

<a id="ref-for-dppx"></a>

### <a id="resolution"></a>6.4.  Resolution Units: the [\<resolution\>](#resolution-value) type and [dpi](#dpi), [dpcm](#dpcm), [dppx](#dppx) units

<a id="ref-for-dimension⑨"></a>

Resolution units are [dimensions](#dimension) denoted by <a id="resolution-value"></a>\<resolution\>. The resolution unit identifiers are:

<a id="dpi"></a>dpi  
Dots per inch.

<a id="dpcm"></a>dpcm  
Dots per centimeter.

<a id="dppx"></a>dppx  
<a id="ref-for-px⑤"></a>

Dots per [px](#px) unit.

<a id="ref-for-resolution-value③"></a>

<a id="ref-for-in③"></a>

<a id="ref-for-cm②"></a>

<a id="ref-for-px⑥"></a>

<a id="ref-for-propdef-image-resolution"></a>

The [\<resolution\>](#resolution-value) unit represents the size of a single "dot" in a graphical representation by indicating how many of these dots fit in a CSS [in](#in), [cm](#cm), or [px](#px). For uses, see e.g. the resolution media query in [\[MEDIAQ\]](#biblio-mediaq) or the [image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution) property defined in [\[CSS3-IMAGES\]](#biblio-css3-images).

<a id="ref-for-resolution-value④"></a>

<a id="ref-for-compatible-units⑦"></a>

<a id="ref-for-canonical-unit⑤"></a>

All [\<resolution\>](#resolution-value) units are [compatible](#compatible-units), and dppx is their [canonical unit](#canonical-unit).

<a id="ref-for-resolution-value⑤"></a>

The allowed range of [\<resolution\>](#resolution-value) values <em>always</em> excludes negative values, in addition to any explicit ranges that might be specified.

<a id="ref-for-in④"></a>

<a id="ref-for-px⑦"></a>

<a id="ref-for-propdef-image-resolution①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that due to the 1:96 fixed ratio of CSS [in](#in) to CSS [px](#px), 1dppx is equivalent to 96dpi. This corresponds to the default resolution of images displayed in CSS: see [image-resolution](https://www.w3.org/TR/css-images-4/#propdef-image-resolution).

<a id="ref-for-px⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33e77fee"></a> The following @media rule uses Media Queries [\[MEDIAQ\]](#biblio-mediaq) to assign some special style rules to devices that use two or more device pixels per CSS [px](#px) unit:
>
> ```text
> @media (min-resolution: 2dppx) { ... }
> ```
## <a id="defined-elsewhere"></a>7.  Data Types Defined Elsewhere

Some data types are defined in their own modules. This example talks about some of the most common ones used across several specifications.

<a id="ref-for-typedef-color"></a>

### <a id="colors"></a>7.1.  Colors: the [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) type

<a id="ref-for-typedef-color①"></a>

The [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) data type is defined in [\[CSS3COLOR\]](#biblio-css3color). UAs that support CSS Color Level 3 or its successor must interpret <a id="ref-for-typedef-color②"></a>\<color\> as defined therein.

<a id="ref-for-typedef-image"></a>

### <a id="images"></a>7.2.  Images: the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) type

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-url-value⑧"></a>

The [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) data type is defined in [\[CSS3-IMAGES\]](#biblio-css3-images). UAs that support CSS Images Level 3 or its successor must interpret <a id="ref-for-typedef-image②"></a>\<image\> as defined therein. UAs that do not yet support CSS Images Level 3 must interpret <a id="ref-for-typedef-image③"></a>\<image\> as [\<url\>](#url-value).

<a id="ref-for-typedef-position"></a>

### <a id="position"></a>7.3.  2D Positioning: the [\<position\>](#typedef-position) type

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-propdef-background-position"></a>

The <a id="typedef-position"></a>[\<position\>](#typedef-position) value specifies the position of a object area (e.g. background image) inside a positioning area (e.g. background positioning area). It is computed and interpreted as specified for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position). [\[CSS3-BACKGROUND\]](#biblio-css3-background)

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-typedef-length-percentage④"></a>

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

The canonical order when serializing is the horizontal component followed by the vertical component.

<a id="ref-for-length-value①⓪"></a>

<a id="ref-for-percentage-value①⑥"></a>

<a id="ref-for-typedef-position③"></a>

When specified in a grammar alongside other keywords, [\<length\>](#length-value)s, or [\<percentage\>](#percentage-value)s, [\<position\>](#typedef-position) is <em>greedily</em> parsed; it consumes as many components as possible.

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-typedef-position④"></a>

<a id="ref-for-length-value①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2622a7d7"></a> For example, [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) defines a 3D position as (effectively) \<position\> \<length\>?. A value such as left 50px will be parsed as a 2-value [\<position\>](#typedef-position), with an omitted z-component; on the other hand, a value such as top 50px will be parsed as a single-value <a id="ref-for-typedef-position⑤"></a>\<position\> followed by a [\<length\>](#length-value).

## <a id="functional-notations"></a>8.  Functional Notations

<a id="ref-for-typedef-function-token"></a>

<a id="ref-for-ascii-case-insensitive⑤"></a>

A <a id="functional-notation"></a>functional notation is a type of component value that can represent more complex types or invoke special processing. The syntax starts with the name of the function immediately followed by a left parenthesis (i.e. a [\<function-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-function-token)) followed by the argument(s) to the notation followed by a right parenthesis. Like keywords, function names are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). [White space](https://www.w3.org/TR/css-syntax/#whitespace) is allowed, but optional, immediately inside the parentheses. Functions can take multiple arguments, which are formatted similarly to a CSS property value.

<a id="ref-for-functional-notation③"></a>

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
<a id="ref-for-funcdef-calc④"></a>

### <a id="calc-notation"></a>8.1.  Mathematical Expressions: [calc()](#funcdef-calc)

<a id="ref-for-selectordef-adjacent"></a>

<a id="ref-for-x"></a>

The <a id="funcdef-calc"></a>calc() function allows a numeric CSS component value to be written as a mathematical expression using addition ([+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent)), subtraction (-), multiplication ([\*](https://www.w3.org/TR/selectors-3/#x)), and/or division (/).

<a id="ref-for-funcdef-calc⑤"></a>

The [calc()](#funcdef-calc) expression represents the result of the mathematical calculation it contains, which is evaluated using standard operator precedence rules (\* and / bind tighter than + and -, and operators are otherwise evaluated left-to-right).

<a id="ref-for-length-value①②"></a>

<a id="ref-for-frequency-value⑥"></a>

<a id="ref-for-angle-value⑨"></a>

<a id="ref-for-time-value⑦"></a>

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-number-value⑥"></a>

<a id="ref-for-integer-value⑥"></a>

<a id="ref-for-funcdef-calc⑥"></a>

It can be used wherever [\<length\>](#length-value), [\<frequency\>](#frequency-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<percentage\>](#percentage-value), [\<number\>](#number-value), or [\<integer\>](#integer-value) values are allowed. Components of a [calc()](#funcdef-calc) expression can be literal values or or <a id="ref-for-funcdef-calc⑦"></a>calc() expressions.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ec5d11f6"></a>
>
> ```text
> section {
>   float: left;
>   margin: 1em; border: solid 1px;
>   width: calc(100%/3 - 2*1em - 2*1px);
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e56942d8"></a>
>
> ```text
> p {
>   margin: calc(1rem - 2px) calc(1rem - 1px);
> }
> ```
<a id="ref-for-propdef-font-size③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f3f3fe7"></a> The following sets the [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) so that exactly 40em fits within the viewport, ensuring that roughly the same amount of text always fills the screen no matter the screen size.
>
> ```text
> :root {
>   font-size: calc(100vw / 40);
> }
> ```
>
> <a id="ref-for-rem③"></a>
>
> If the rest of the design is specified using the [rem](#rem) unit, the entire layout will scale to match the viewport width.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-816e5203"></a> The following example stacks two background images, with the first perfectly centered and the second offset 20px to the bottom and to the left of the first.
>
> ```text
> .foo {
>   background: url(top.png), url(bottom.png);
>   background-repeat: no-repeat;
>   background-position: 50% 50%, calc(50% + 20px) calc(50% + 20px);
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cac7a502"></a> This example shows how to place color-stops on a gradient an equal distance from either end.
>
> ```text
> .foo {
>   background-image: linear-gradient(to right, silver,
>                                               white 50px,
>                                               white calc(100% - 50px),
>                                               silver);
> }
> ```
#### <a id="calc-syntax"></a>8.1.1.  Syntax

<a id="ref-for-funcdef-calc⑧"></a>

The syntax of a [calc()](#funcdef-calc) function is:

<a id="ref-for-funcdef-calc⑨"></a>

<a id="ref-for-typedef-calc-sum"></a>

<a id="typedef-calc-sum"></a>

<a id="ref-for-typedef-calc-product"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-typedef-calc-product①"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="typedef-calc-product"></a>

<a id="ref-for-typedef-calc-value"></a>

<a id="ref-for-typedef-calc-value①"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-typedef-calc-number-value"></a>

<a id="ref-for-mult-zero-plus②"></a>

<a id="typedef-calc-value"></a>

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-typedef-dimension②"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-percentage-value①⑧"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-calc-sum①"></a>

<a id="typedef-calc-number-sum"></a>

<a id="ref-for-typedef-calc-number-product"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-calc-number-product①"></a>

<a id="ref-for-mult-zero-plus③"></a>

<a id="typedef-calc-number-product"></a>

<a id="ref-for-typedef-calc-number-value①"></a>

<a id="ref-for-typedef-calc-number-value②"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-typedef-calc-number-value③"></a>

<a id="ref-for-mult-zero-plus④"></a>

<a id="typedef-calc-number-value"></a>

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-typedef-calc-number-sum"></a>

```text
<calc()> = calc( <calc-sum> )
<calc-sum> = <calc-product> [ [ '+' | '-' ] <calc-product> ]*
<calc-product> = <calc-value> [ '*' <calc-value> | '/' <calc-number-value> ]*
<calc-value> = <number> | <dimension> | <percentage> | ( <calc-sum> )
<calc-number-sum> = <calc-number-product> [ [ '+' | '-' ] <calc-number-product> ]*
<calc-number-product> = <calc-number-value> [ '*' <calc-number-value> | '/' <calc-number-value> ]*
<calc-number-value> = <number> | ( <calc-number-sum> )
```
<a id="ref-for-selectordef-adjacent①"></a>

<a id="ref-for-x①"></a>

In addition, [white space](https://www.w3.org/TR/css-syntax/#whitespace) is required on both sides of the [+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent) and - operators. (The [\*](https://www.w3.org/TR/selectors-3/#x) and / operators can be used without white space around them.)

<a id="ref-for-funcdef-calc①⓪"></a>

UAs must support [calc()](#funcdef-calc) expressions of at least 20 terms, where each `NUMBER`, `DIMENSION`, or `PERCENTAGE` is a term. If a <a id="ref-for-funcdef-calc①①"></a>calc() expression contains more than the supported number of terms, it must be treated as if it were invalid.

#### <a id="calc-type-checking"></a>8.1.2.  Type Checking

<a id="ref-for-length-value①③"></a>

<a id="ref-for-frequency-value⑦"></a>

<a id="ref-for-angle-value①⓪"></a>

<a id="ref-for-time-value⑧"></a>

<a id="ref-for-percentage-value①⑨"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-integer-value⑦"></a>

<a id="ref-for-resolved-type"></a>

<a id="ref-for-typedef-number-token②"></a>

<a id="ref-for-typedef-dimension-token②"></a>

<a id="ref-for-cm③"></a>

<a id="ref-for-deg②"></a>

A math expression has a <a id="resolved-type"></a>resolved type, which is one of [\<length\>](#length-value), [\<frequency\>](#frequency-value), [\<angle\>](#angle-value), [\<time\>](#time-value), [\<percentage\>](#percentage-value), [\<number\>](#number-value), or [\<integer\>](#integer-value). The [resolved type](#resolved-type) must be valid for where the expression is placed; otherwise, the expression is invalid. The <a id="ref-for-resolved-type①"></a>resolved type of the expression is determined by the types of the values it contains. [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s are of type <a id="ref-for-number-value①⓪"></a>\<number\> or <a id="ref-for-integer-value⑧"></a>\<integer\>. A [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token)’s type is given by its unit ([cm](#cm) is <a id="ref-for-length-value①④"></a>\<length\>, [deg](#deg) is <a id="ref-for-angle-value①①"></a>\<angle\>, etc.).

<a id="ref-for-typedef-number-token③"></a>

<a id="ref-for-number-value①①"></a>

<a id="ref-for-integer-value⑨"></a>

<a id="ref-for-length-value①⑤"></a>

<a id="ref-for-funcdef-calc①②"></a>

<a id="ref-for-propdef-width⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token)s are always interpreted as [\<number\>](#number-value)s or [\<integer\>](#integer-value)s, "unitless 0" [\<length\>](#length-value)s aren’t supported in [calc()](#funcdef-calc). That is, [width: calc(0 + 5px);](https://www.w3.org/TR/css-sizing-3/#propdef-width) is invalid, even though both <a id="ref-for-propdef-width⑧"></a>width: 0; and <a id="ref-for-propdef-width⑨"></a>width: 5px; are valid.

<a id="ref-for-number-value①②"></a>

<a id="ref-for-typedef-percentage-token①"></a>

<a id="ref-for-propdef-width①⓪"></a>

<a id="ref-for-length-value①⑥"></a>

<a id="ref-for-percentage-value②⓪"></a>

<a id="ref-for-funcdef-calc①③"></a>

If percentages are accepted in the context in which the expression is placed, and they are defined to be relative to another type besides [\<number\>](#number-value), a [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) is treated as that type. For example, in the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property, percentages have the [\<length\>](#length-value) type. A percentage only has the [\<percentage\>](#percentage-value) type if in that context <a id="ref-for-percentage-value②①"></a>\<percentage\> values are not used-value compatible with any other type. If percentages are not normally allowed in place of the [calc()](#funcdef-calc), then a <a id="ref-for-funcdef-calc①④"></a>calc() expression containing percentages is invalid in that context.

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-number-value①③"></a>

<a id="ref-for-propdef-opacity"></a>

<a id="ref-for-funcdef-calc①⑤"></a>

<a id="ref-for-typedef-dimension③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that [\<percentage\>](#percentage-value)s relative to [\<number\>](#number-value)s, such as in [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), are not allowed in [calc()](#funcdef-calc). Allowing this would cause significant problems with "unit algebra" (allowing multiplication/division of [\<dimension\>](#typedef-dimension)s), and in every case so far, doesn’t provide any new functionality. (For example, <a id="ref-for-propdef-opacity①"></a>opacity: 25% is identical to <a id="ref-for-propdef-opacity②"></a>opacity: .25; it’s just a trivial syntax transform.)

<a id="ref-for-number-value①④"></a>

<a id="ref-for-length-value①⑦"></a>

<a id="ref-for-propdef-line-height①"></a>

<a id="ref-for-propdef-tab-size"></a>

<a id="ref-for-funcdef-calc①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although there are a few properties in which a bare [\<number\>](#number-value) becomes a [\<length\>](#length-value) at used-value time (specifically, [line-height](https://drafts.csswg.org/css2/#propdef-line-height) and [tab-size](https://www.w3.org/TR/css-text-4/#propdef-tab-size)), <a id="ref-for-number-value①⑤"></a>\<number\>s never become "length-like" in [calc()](#funcdef-calc). They always stay as <a id="ref-for-number-value①⑥"></a>\<number\>s.

Operators form sub-expressions, which gain types based on their arguments. To make expressions simpler, operators have restrictions on the types they accept. At each operator, the types of the left and right argument are checked for these restrictions. If compatible, the type resolves as described below (the following ignores precedence rules on the operators for simplicity):

- <a id="ref-for-integer-value①⓪"></a>

  <a id="ref-for-number-value①⑦"></a>

  <a id="ref-for-selectordef-adjacent②"></a>

  At [+](https://www.w3.org/TR/selectors-4/#selectordef-adjacent) or -, check that both sides have the same type, or that one side is a [\<number\>](#number-value) and the other is an [\<integer\>](#integer-value). If both sides are the same type, resolve to that type. If one side is a <a id="ref-for-number-value①⑧"></a>\<number\> and the other is an <a id="ref-for-integer-value①①"></a>\<integer\>, resolve to <a id="ref-for-number-value①⑨"></a>\<number\>.

- <a id="ref-for-integer-value①②"></a>

  <a id="ref-for-number-value②⓪"></a>

  <a id="ref-for-x②"></a>

  At [\*](https://www.w3.org/TR/selectors-3/#x), check that at least one side is [\<number\>](#number-value). If both sides are [\<integer\>](#integer-value), resolve to <a id="ref-for-integer-value①③"></a>\<integer\>. Otherwise, resolve to the type of the other side.

- <a id="ref-for-integer-value①④"></a>

  <a id="ref-for-number-value②①"></a>

  At /, check that the right side is [\<number\>](#number-value). If the left side is [\<integer\>](#integer-value), resolve to <a id="ref-for-number-value②②"></a>\<number\>. Otherwise, resolve to the type of the left side.

If an operator does not pass the above checks, the expression is invalid. Also, division by zero is invalid. This includes both dividing by the literal number zero, as well as any numeric expression that evaluates to zero (as purely-numeric expressions can be evaluated without any additional information at parse time).

<a id="ref-for-funcdef-calc①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Algebraic simplifications do not affect the validity of the [calc()](#funcdef-calc) expression or its resolved type. For example, calc(5px - 5px + 10s) and calc(0 \* 5px + 10s) are both invalid due to the attempt to add a length and a time.

#### <a id="calc-computed-value"></a>8.1.3.  Computed Value

<a id="ref-for-funcdef-calc①⑧"></a>

The computed value of a [calc()](#funcdef-calc) expression is the expression with all components computed.

<a id="ref-for-funcdef-calc①⑨"></a>

Where percentages are not resolved at computed-value time, they are not resolved in [calc()](#funcdef-calc) expressions, e.g. calc(100% - 100% + 1em) resolves to calc(1em + 0%), not to 1em. If there are special rules for computing percentages in a value (e.g. [the height property](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)), they apply whenever a <a id="ref-for-funcdef-calc②⓪"></a>calc() expression contains percentages.

<a id="ref-for-propdef-font-size④"></a>

<a id="ref-for-computed-value③"></a>

<a id="ref-for-font-relative-length①"></a>

<a id="ref-for-propdef-background-position②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-023dad93"></a> For example, whereas [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) computes percentage values at [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) time so that [font-relative length](#font-relative-length) units can be computed, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) has layout-dependent behavior for percentage values, and thus does not resolve percentages until used-value time.
>
> <a id="ref-for-propdef-background-position③"></a>
>
> <a id="ref-for-funcdef-calc②①"></a>
>
> <a id="ref-for-propdef-font-size⑤"></a>
>
> Due to this, [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) computation preserves the percentage in a [calc()](#funcdef-calc) whereas [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) will compute such expressions directly into a length.

<a id="ref-for-valdef-width-auto"></a>

Given the complexities of width and height calculations on table cells and table elements, math expressions involving percentages for widths and heights on table columns, table column groups, table rows, table row groups, and table cells in both auto and fixed layout tables MAY be treated as if [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) had been specified.

#### <a id="calc-range"></a>8.1.4.  Range Checking

<a id="ref-for-funcdef-calc②②"></a>

<a id="ref-for-computed-value④"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-specified-value①"></a>

Parse-time range-checking of values is not performed within [calc()](#funcdef-calc), and therefore out-of-range values do not cause the declaration to become invalid. However, the value resulting from an expression must be clamped to the range allowed in the target context. Clamping is performed on [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value) to the extent possible, and also on [used values](https://www.w3.org/TR/css-cascade-5/#used-value) if computation was unable to sufficiently simplify the expression to allow range-checking. (Clamping is not performed on [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value).)

<a id="ref-for-funcdef-calc②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This requires all contexts accepting [calc()](#funcdef-calc) to define their allowable values as a closed (not open) interval.

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
> <a id="ref-for-propdef-width①①"></a>
>
> <a id="ref-for-funcdef-calc②④"></a>
>
> Note however that [width: -5px](https://www.w3.org/TR/css-sizing-3/#propdef-width) is not equivalent to <a id="ref-for-propdef-width①②"></a>width: calc(-5px)! Out-of-range values <em>outside</em> [calc()](#funcdef-calc) are syntactically invalid, and cause the entire declaration to be dropped.

#### <a id="calc-serialize"></a>8.1.5.  Serialization

<a id="ref-for-funcdef-calc②⑤"></a>

The serialization of [calc()](#funcdef-calc) values is undefined in this level.

## <a id="iana"></a> Appendix A: IANA Considerations

### <a id="about-invalid"></a> Registration for the `about:invalid` URL scheme

This sections defines and registers the `about:invalid` URL, in accordance with the registration procedure defined in [\[RFC6694\]](#biblio-rfc6694).

The official record of this registration can be found at [http&#x3A;&#x2F;&#x2F;www&#x2E;iana&#x2E;org&#x2F;assignments&#x2F;about-uri-tokens&#x2F;about-uri-tokens&#x2E;xhtml](http://www.iana.org/assignments/about-uri-tokens/about-uri-tokens.xhtml)&#x2E;

|                           |                                                                                                                                                                                                             |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Registered Token          | `invalid`                                                                                                                                                                                           |
| Intended Usage            | The `about:invalid` URL references a non-existent document with a generic error condition. It can be used when a URL is necessary, but the default value shouldn’t be resolvable as any type of document. |
| Contact/Change controller | CSS WG \<<www-style@w3.org>\> (on behalf of W3C)                                                                                                                                                            |
| Specification             | [CSS Values and Units Module Level 3](https://www.w3.org/TR/css3-values/)                                                                                                                                   |

## <a id="acknowledgments"></a> Acknowledgments

Comments and suggestions from Giovanni Campagna, Christoph Päper, Keith Rarick, Alex Mogilevsky, Ian Hickson, David Baron, Edward Welbourne, Boris Zbarsky, Björn Höhrmann and Michael Day improved this module.

## <a id="changes"></a> Changes

Changes since the [1 December 2022 Candidate Recommenation Snapshot](https://www.w3.org/TR/2022/CR-css-values-3-20221201/) are:

- Editorial synchronization with [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/).

- <a id="ref-for-css-value-definition-syntax"></a>

  Added definition for \<func()\> notation to the [value definition syntax](#css-value-definition-syntax).

- Clarified stacking of multipliers.

- <a id="ref-for-resolution-value⑥"></a>

  Clarified that [\<resolution\>](#resolution-value) cannot be negative.

Changes since the [6 June 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-values-3-20190606/) are:

- <a id="ref-for-funcdef-attr"></a>

  Dropped the [attr()](https://drafts.csswg.org/css-values-5/#funcdef-attr) function, since it was punted to Level 5.

- Specified serialization of empty urls to be `url("")`. ([Issue 6447](https://github.com/w3c/csswg-drafts/issues/6447))

- <a id="ref-for-font-relative-length②"></a>

  Clarified that the [font-relative lengths](#font-relative-length) are calculated without text shaping. ([Issue 5498](https://github.com/w3c/csswg-drafts/issues/5498))

- <a id="ref-for-device-pixel③"></a>

  Added a definition for [device pixel](#device-pixel). ([Issue 7287](https://github.com/w3c/csswg-drafts/issues/7287))

- <a id="ref-for-css-wide-keywords①"></a>

  Cleaned up interaction of this specification and [\[CSS-CASCADE-3\]](#biblio-css-cascade-3) in defining the [CSS-wide keywords](#css-wide-keywords). ([Issue 7439](https://github.com/w3c/csswg-drafts/issues/7439))

- <a id="ref-for-funcdef-calc②⑥"></a>

  Removed definition of \<length-number\>, since it is impossible to combine them with [calc()](#funcdef-calc). ([Issue 2789](https://github.com/w3c/csswg-drafts/issues/2789))

- <a id="ref-for-number②"></a>

  Fixed definition of [numbers](#number) to allow decimals in combination with scientific notation, as originally intended and as defined in [\[CSS-SYNTAX-3\]](#biblio-css-syntax-3). ([Issue 7248](https://github.com/w3c/csswg-drafts/issues/7248))

- Editorial improvements.

Changes since the [31 January 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-values-3-20190131/) are:

- <a id="ref-for-css-value-definition-syntax①"></a>

  <a id="ref-for-css-bracketed-range-notation②"></a>

  Added the new [bracketed range notation](#css-bracketed-range-notation) to the CSS [value definition syntax](#css-value-definition-syntax). This has no normative implications on implementations, just allows more routine annotation of ranges in future CSS specifications. ([Issue 355](https://github.com/w3c/csswg-drafts/issues/355))

Changes since the [14 August 2018 Candidate Recommendation](https://www.w3.org/TR/2018/CR-css-values-3-20180814/) are:

- Defined \<'property'\> syntax to refer to the property without any top-level \#-multiplier, to make the notation usable with common list-valued property patterns. ([Issue 3146](https://github.com/w3c/csswg-drafts/issues/3146))

- Clarified that numeric values outside the allowed range are not ignored if a more specific spec defines other handling. ([Issue 3270](https://github.com/w3c/csswg-drafts/issues/3270))

  > Properties may restrict numeric values to some range. If the value is outside the allowed range, <u>unless otherwise specified,</u> the declaration is invalid and must be [ignored](https://www.w3.org/TR/CSS2/conform.html#ignore).

- <a id="ref-for-propdef-background-position④"></a>

  Fixed some [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) examples to be less confusing. ([Issue 3482](https://github.com/w3c/csswg-drafts/issues/3482#issuecomment-453257033))

A [Disposition of Comments](http://drafts.csswg.org/css-values-3/issues-cr-2018) is available.

Changes since the [29 September 2016 Candidate Recommendation](https://www.w3.org/TR/2016/CR-css-values-3-20160929/) are:

- Removed [consideration of scrollbars](#change-2012-vwh-scrollbars) in computing viewport units due to lack of implementations. ([Issue 15](https://drafts.csswg.org/css-values-3/issues-cr-2016#issue-15))

- <a id="ref-for-propdef-object-position"></a>

  <a id="ref-for-typedef-position⑥"></a>

  Inlined the [\<position\>](#typedef-position) definition and dropped the three-value syntaxes to allow for unambiguous combination in complex grammars. This effectively removes that syntax from [object-position](https://www.w3.org/TR/css-images-3/#propdef-object-position), but allows <a id="ref-for-typedef-position⑦"></a>\<position\> to be re-used e.g. in [\[CSS-TRANSFORMS-1\]](#biblio-css-transforms-1) for 3D positions. (See [discussion](https://lists.w3.org/Archives/Public/www-style/2017Feb/0052.html).)

- Reverted previous change to allow zero angles to drop their unit; this will instead be special-cased where needed for backwards-compatibility. (See [discussion](https://lists.w3.org/Archives/Public/www-style/2017Apr/0027.html))

- <a id="ref-for-funcdef-calc②⑦"></a>

  Defined that range checking, and any resulting clamping, of [calc()](#funcdef-calc) values is performed both at computed time and at used time. ([Issue \#434](https://github.com/w3c/csswg-drafts/issues/434))

- <a id="ref-for-funcdef-calc②⑧"></a>

  Fixed grammar error that disallowed numeric expressions as denominators in [calc()](#funcdef-calc). ([Issue 12](https://drafts.csswg.org/css-values-3/issues-cr-2016#issue-12))

- Defined handling of font-relative units outside the context of an element. ([Issue 9](https://drafts.csswg.org/css-values-3/issues-cr-2016#issue-9))

- <a id="ref-for-length-value①⑧"></a>

  <a id="ref-for-number-value②③"></a>

  Defined that 0 parses as [\<number\>](#number-value) if it’s ambiguous whether it’s a <a id="ref-for-number-value②④"></a>\<number\> or a [\<length\>](#length-value). ([Issue 489](https://github.com/w3c/csswg-drafts/issues/489))

- <a id="ref-for-funcdef-url①③"></a>

  Defined empty [url()](#funcdef-url)s to refer to an invalid URL, rather than resolving to the URL of the style sheet. ([Issue 2211](https://github.com/w3c/csswg-drafts/issues/2211))

- <a id="ref-for-funcdef-calc②⑨"></a>

  <a id="ref-for-number-value②⑤"></a>

  Removed (unused) ability for percentages to be treated as a [\<number\>](#number-value) type in [calc()](#funcdef-calc). ([Issue 1463](https://github.com/w3c/csswg-drafts/issues/1463))

- Clarified that high-resolution screens should anchor on device pixels, not physical units. ([Issue 8](https://drafts.csswg.org/css-values-3/issues-cr-2016#issue-8))

- <a id="ref-for-funcdef-url①④"></a>

  Clarified definition of [url()](#funcdef-url) to normatively say that it accepts unquoted syntax.

- <a id="ref-for-funcdef-url①⑤"></a>

  Defined that fragment-only [url()](#funcdef-url) are specially handled to always be page-local links, regardless of base-url shenanigans. (See [§ 3.4.1.1 Fragment URLs](#local-urls).)

- Defined attr() parsing in terms of the Syntax spec, not CSS2.1 grammar.

A [Disposition of Comments](https://drafts.csswg.org/css-values-3/issues-cr-2016) is available.

Changes since the [11 June 2015 Candidate Recommendation](https://www.w3.org/TR/2015/CR-css-values-3-20150611/) are:

- <a id="ref-for-funcdef-toggle"></a>

  Dropped [toggle()](https://drafts.csswg.org/css-values-5/#funcdef-toggle) for lack of implementations.

- Allow zero angles to be represented as 0. (Change due to Web-compatibility constraints in transform and gradient syntaxes.)

- Defined [special handling](#local-urls) for fragment URLs.

- <a id="ref-for-url-value⑨"></a>

  Defined an empty [\<url\>](#url-value) resolves to an invalid resource.

- <a id="ref-for-canonical-unit⑥"></a>

  <a id="ref-for-compatible-units⑧"></a>

  Defined [compatible units](#compatible-units) and [canonical units](#canonical-unit) for serialization.

- <a id="ref-for-funcdef-url①⑥"></a>

  Defined case-sensitivity of [url()](#funcdef-url) attribute argument to match attribute selectors.

- <a id="ref-for-css-css-identifier④"></a>

  <a id="ref-for-typedef-ident②"></a>

  Added definition of [\<ident\>](#typedef-ident) notation to definition of [identifiers](#css-css-identifier).

- <a id="ref-for-percentage-value②③"></a>

  <a id="ref-for-length-value①⑨"></a>

  <a id="ref-for-typedef-length-percentage⑤"></a>

  Added [\<length-percentage\>](#typedef-length-percentage) as a shorthand for [\<length\>](#length-value) \| [\<percentage\>](#percentage-value), along with equivalent productions for angles, numbers, times, and frequencies.

- <a id="ref-for-funcdef-calc③⓪"></a>

  <a id="ref-for-percentage-value②④"></a>

  Allowed [\<percentage\>](#percentage-value)s inside [calc()](#funcdef-calc) to resolve as their own type, if they occur in some (as yet theoretical) context where they are not compatible with any other type.

- Various clarifications and editorial improvements.

Changes since the [30 July 2013 Candidate Recommendation](https://www.w3.org/TR/2013/CR-css3-values-20130730/) are:

- Specified that, in the absence of font information, 1ch equal .5em.

- <a id="ref-for-Q②"></a>

  Added [Q](#Q) unit.

- <a id="ref-for-identifier-value①③"></a>

  Relaxed unnecessary restrictions on [\<custom-ident\>](#identifier-value). Require specs referencing it to be clear about excluded keywords, because the new rule isn’t as simple.

- Clarified relative URL resolution for embedded style sheets.

- Clarified {<var>A</var>} variant of {<var>A</var>,<var>B</var>} notation.

- Added notation for restricting the length of comma-separated lists specified with the \# notation.

- <a id="ref-for-funcdef-toggle①"></a>

  Clarified handling of [toggle()](https://drafts.csswg.org/css-values-5/#funcdef-toggle) when used in shorthand declarations.

- Clarified that stringing together reorderable combinations allows interleaving.

- Changed syntax references from the 2.1 grammar to the Syntax spec.

A [Disposition of Comments](https://drafts.csswg.org/css-values-3/issues-cr-2013) is available.

Changes since the [28 August 2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-values-20120828/) are:

- <a id="ref-for-funcdef-attr①"></a>

  Corrected `wqname` in the [attr()](https://drafts.csswg.org/css-values-5/#funcdef-attr) syntax to `qname`

- <a id="ref-for-funcdef-attr②"></a>

  Made undefined namespace prefixes in [attr()](https://drafts.csswg.org/css-values-5/#funcdef-attr) invalidate the function.

- <a id="ref-for-valdef-overflow-auto"></a>

  <a id="ref-for-propdef-overflow"></a>

  <a id="change-2012-vwh-scrollbars"></a> Per [WG resolution](https://lists.w3.org/Archives/Public/www-style/2013Jan/0616.html), made [viewport-percentage units](#viewport-relative-lengths) respect scrollbars on the viewport unless [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is [auto](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-auto) (in which case they ignore the presence of scrollbars).

- <a id="ref-for-paged-media②"></a>

  Deferred exact definition of [viewport-percentage units](#viewport-relative-lengths) in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media) to [CSS Paged Media](https://www.w3.org/TR/css3-page/).

- <a id="ref-for-identifier-value①④"></a>

  Added back the [\<custom-ident\>](#identifier-value) term as a convenience notation, so that other specs can refer to it.

Changes since the [4 April 2013 Candidate Recommendation](https://www.w3.org/TR/2013/CR-css3-values-20130404/) are:

- <a id="ref-for-css-wide-keywords②"></a>

  Noted that the list of [CSS-wide keywords](#css-wide-keywords) may be expanded by other specs.

- <a id="ref-for-ex③"></a>

  Clarified definition of [ex](#ex) to refer to the “first available font”.

- <a id="ref-for-string-value⑤"></a>

  <a id="ref-for-valdef-attr-url"></a>

  <a id="ref-for-valdef-attr-string"></a>

  <a id="ref-for-funcdef-attr③"></a>

  Specified that [attr()](https://drafts.csswg.org/css-values-5/#funcdef-attr) with [string](https://drafts.csswg.org/css-values-5/#valdef-attr-string) or [url](https://drafts.csswg.org/css-values-5/#valdef-attr-url) types doesn’t reparse the attribute contents, just takes the value literally as the value of a [\<string\>](#string-value).

## <a id="security"></a> Security Considerations

This specification presents no new security considerations.

<a id="ref-for-funcdef-url①⑦"></a>

<a id="ref-for-url-value①⓪"></a>

This specification defines the [url()](#funcdef-url) function ([\<url\>](#url-value)), which allows CSS to make network requests. Depending on what features they are used in, these can potentially expose whether or not the user has access to resources on a network, and expose information about their contents (such as the rules within a style sheet, the size of an image, the metrics of a font). They can also allow exfiltrating data via URL.

## <a id="privacy"></a> Privacy Considerations

<a id="ref-for-viewport-percentage-lengths"></a>

<a id="ref-for-font-relative-length③"></a>

This specification introduces units that expose the user’s screen size (the [viewport-percentage lengths](#viewport-percentage-lengths)), default font size, and potentially some information about which fonts are available on the user’s system (the [font-relative lengths](#font-relative-length)).

<a id="ref-for-funcdef-url①⑧"></a>

<a id="ref-for-url-value①①"></a>

This specification defines the [url()](#funcdef-url) function ([\<url\>](#url-value)), which allows CSS to make network requests. Depending on what features they are used in, these can potentially expose whether or not the user has access to resources on a network, and expose information about their contents (such as the rules within a style sheet, the size of an image, the metrics of a font). They can also allow exfiltrating data via URL.

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
- [absolute length](#absolute-length), in § 5.2
- [absolute length unit](#absolute-length), in § 5.2
- [advance measure](#length-advance-measure), in § 5.1.1
- [anchor](#anchor-unit), in § 5.2
- [anchor unit](#anchor-unit), in § 5.2
- [\<angle\>](#angle-value), in § 6.1
- [\<angle-percentage\>](#typedef-angle-percentage), in § 4.6
- [bearing angle](#bearing-angle), in § 6.1
- [bracketed range notation](#css-bracketed-range-notation), in § 4.1
- [calc()](#funcdef-calc), in § 8.1
- [\<calc-number-product\>](#typedef-calc-number-product), in § 8.1.1
- [\<calc-number-sum\>](#typedef-calc-number-sum), in § 8.1.1
- [\<calc-number-value\>](#typedef-calc-number-value), in § 8.1.1
- [\<calc-product\>](#typedef-calc-product), in § 8.1.1
- [\<calc-sum\>](#typedef-calc-sum), in § 8.1.1
- [\<calc-value\>](#typedef-calc-value), in § 8.1.1
- [canonical](#canonical-unit), in § 4.4.1
- [canonical unit](#canonical-unit), in § 4.4.1
- [ch](#ch), in § 5.1.1
- [ch unit](#ch), in § 5.1.1
- [cm](#cm), in § 5.2
- [compatible](#compatible-units), in § 4.4.1
- [compatible units](#compatible-units), in § 4.4.1
- [CSS bracketed range notation](#css-bracketed-range-notation), in § 4.1
- [CSS ident](#css-css-identifier), in § 3
- [CSS identifier](#css-css-identifier), in § 3
- [CSS value definition syntax](#css-value-definition-syntax), in § 2
- [CSS-wide keywords](#css-wide-keywords), in § 3.1.1
- [\<custom-ident\>](#identifier-value), in § 3.2
- [deg](#deg), in § 6.1
- [device pixel](#device-pixel), in § 5.2
- [\<dimension\>](#typedef-dimension), in § 4.4
- [dimension](#dimension), in § 4.4
- [dpcm](#dpcm), in § 6.4
- [dpi](#dpi), in § 6.4
- [dppx](#dppx), in § 6.4
- [em](#em), in § 5.1.1
- [em unit](#em), in § 5.1.1
- [ex](#ex), in § 5.1.1
- [ex unit](#ex), in § 5.1.1
- [font-relative lengths](#font-relative-length), in § 5.1.1
- [\<frequency\>](#frequency-value), in § 6.3
- [\<frequency-percentage\>](#typedef-frequency-percentage), in § 4.6
- [functional notation](#functional-notation), in § 8
- [grad](#grad), in § 6.1
- [Hz](#Hz), in § 6.3
- [\<ident\>](#typedef-ident), in § 3
- [ident](#css-css-identifier), in § 3
- [identifier](#css-css-identifier), in § 3
- [in](#in), in § 5.2
- [\<integer\>](#integer-value), in § 4.2
- [integer](#integer), in § 4.2
- [keyword](#css-keyword), in § 3.1
- [kHz](#kHz), in § 6.3
- [\<length\>](#length-value), in § 5
- [\<length-percentage\>](#typedef-length-percentage), in § 4.6
- [local url flag](#url-local-url-flag), in § 3.4.1.1
- [mm](#mm), in § 5.2
- [ms](#ms), in § 6.2
- [\<number\>](#number-value), in § 4.3
- [number](#number), in § 4.3
- [numeric data types](#numeric-data-types), in § 4
- [pc](#pc), in § 5.2
- [\<percentage\>](#percentage-value), in § 4.5
- [percentage](#percentage), in § 4.5
- [physical unit](#physical-unit), in § 5.2
- [pixel unit](#visual-angle-unit), in § 5.2
- [\<position\>](#typedef-position), in § 7.3
- [pt](#pt), in § 5.2
- [px](#px), in § 5.2
- [Q](#Q), in § 5.2
- [rad](#rad), in § 6.1
- [reference pixel](#reference-pixel), in § 5.2
- [relative length](#relative-length), in § 5.1
- [relative length unit](#relative-length), in § 5.1
- [rem](#rem), in § 5.1.1
- [rem unit](#rem), in § 5.1.1
- [\<resolution\>](#resolution-value), in § 6.4
- [resolved type](#resolved-type), in § 8.1.2
- [s](#s), in § 6.2
- [\<string\>](#string-value), in § 3.3
- [textual data types](#css-textual-data-types), in § 3
- [\<time\>](#time-value), in § 6.2
- [\<time-percentage\>](#typedef-time-percentage), in § 4.6
- [turn](#turn), in § 6.1
- [\<url\>](#url-value), in § 3.4
- [URL](#url), in § 3.4
- [url()](#funcdef-url), in § 3.4
- [\<url-modifier\>](#typedef-url-modifier), in § 3.4.3
- [value definition syntax](#css-value-definition-syntax), in § 2
- [vh](#vh), in § 5.1.2
- [viewport-percentage lengths](#viewport-percentage-lengths), in § 5.1.2
- [visual angle unit](#visual-angle-unit), in § 5.2
- [vmax](#vmax), in § 5.1.2
- [vmin](#vmin), in § 5.1.2
- [vw](#vw), in § 5.1.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-2023\] defines the following terms:
  - <a id="861626b1"></a>ua
- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="1e12dba3"></a>animation
  - <a id="04beed9b"></a>animation-name
  - <a id="79dcf364"></a>animation-timing-function
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f0a02"></a>padding-top
- \[CSS-BREAK-3\] defines the following terms:
  - <a id="4f75e4ec"></a>orphans
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="3eef835e"></a>@import
  - <a id="0948355d"></a>actual value
  - <a id="8c8e51b4"></a>computed value
  - <a id="d0dc95c3"></a>inherit
  - <a id="762bad34"></a>initial
  - <a id="d5e08d9c"></a>specified value
  - <a id="7c39b465"></a>unset
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="5bd3632a"></a>hsl()
  - <a id="3b7558dc"></a>opacity
  - <a id="f3226176"></a>rgba()
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-COUNTER-STYLES-3\] defines the following terms:
  - <a id="a9efccbc"></a>disc
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="6b4fc208"></a>containing block
- \[CSS-EASING-1\] defines the following terms:
  - <a id="c7d3b8b7"></a>\<easing-function\>
  - <a id="2f7ff51f"></a>ease-in
  - <a id="056b67db"></a>ease-out
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="11a985a9"></a>font
  - <a id="7066562d"></a>font-family
  - <a id="297dfe3a"></a>font-size
- \[CSS-GRID-2\] defines the following terms:
  - <a id="ca502323"></a>fr
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="5663abb5"></a>image-resolution
  - <a id="dcb1125e"></a>linear-gradient()
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="0bcf441e"></a>auto
  - <a id="add377f4"></a>overflow
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="49731d1d"></a>width
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="87de393a"></a>\<dimension-token\>
  - <a id="a6331414"></a>\<function-token\>
  - <a id="446c663e"></a>\<ident-token\>
  - <a id="eebbfe3d"></a>\<number-token\>
  - <a id="8a73a2e3"></a>\<percentage-token\>
  - <a id="17fe01a1"></a>\<string-token\>
  - <a id="30679d85"></a>\<url-token\>
  - <a id="267b6766"></a>component value
  - <a id="cdce34f5"></a>consume a url token
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="36e5f32e"></a>text-align
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="291965b2"></a>center
  - <a id="beb6807a"></a>tab-size
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="4d38e4c5"></a>text-decoration
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="31452ed5"></a>transform-origin
- \[CSS-UI-4\] defines the following terms:
  - <a id="6ed19243"></a>outline-color
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="b1f72e36"></a>coordinating list property
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="57699934"></a>attr()
  - <a id="8ac10bc6"></a>string
  - <a id="2ef1aa7d"></a>toggle()
  - <a id="d10808c7"></a>url
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="8664e85f"></a>text-orientation
  - <a id="cec0d4db"></a>upright
  - <a id="35f596e9"></a>vertical-lr
  - <a id="ee88ce59"></a>vertical-rl
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="17f0c439"></a>\<border-width\>
  - <a id="29ca9f85"></a>border-collapse
- \[CSS22\] defines the following terms:
  - <a id="ac54fbff"></a>line-height
- \[CSS3-BACKGROUND\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="8218676f"></a>background-attachment
  - <a id="f2249e38"></a>background-position
  - <a id="65b3a7bc"></a>border-color
  - <a id="064303ba"></a>border-width
  - <a id="c48eaa20"></a>box-shadow
- \[CSS3-IMAGES\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="534310cb"></a>object-position
- \[CSS3PAGE\] defines the following terms:
  - <a id="1fe10a71"></a>page area
- \[HTML\] defines the following terms:
  - <a id="c847111b"></a>base
  - <a id="87806adf"></a>pushState(data, unused, url)
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ascii case-insensitive
  - <a id="0698d556"></a>string
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="9e462db5"></a>continuous media
  - <a id="3ea2fcbb"></a>media query
  - <a id="23af89d0"></a>paged media
- \[SELECTORS-3\] defines the following terms:
  - <a id="dfd67b05"></a>\*
- \[SELECTORS-4\] defines the following terms:
  - <a id="3c7eec47"></a>+

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-2023"></a>\[CSS-2023\]  
Chris Lilley; et al. [CSS Snapshot 2023](https://www.w3.org/TR/css-2023/). 7 December 2023. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2023&#x2F;](https://www.w3.org/TR/css-2023/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 29 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-counter-styles-3"></a>\[CSS-COUNTER-STYLES-3\]  
Tab Atkins Jr.. [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/). 27 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-counter-styles-3&#x2F;](https://www.w3.org/TR/css-counter-styles-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-css3-background"></a>\[CSS3-BACKGROUND\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3-fonts"></a>\[CSS3-FONTS\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css3-images"></a>\[CSS3-IMAGES\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css3color"></a>\[CSS3COLOR\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 18 January 2022. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

<a id="biblio-css3page"></a>\[CSS3PAGE\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-url"></a>\[URL\]  
Anne van Kesteren. [URL Standard](https://url.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;url&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://url.spec.whatwg.org/)

### <a id="informative"></a>Informative References

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-cascade-3"></a>\[CSS-CASCADE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 3](https://www.w3.org/TR/css-cascade-3/). 11 February 2021. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-3&#x2F;](https://www.w3.org/TR/css-cascade-3/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 13 February 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-easing-1"></a>\[CSS-EASING-1\]  
Brian Birtles; Dean Jackson; Matt Rakow. [CSS Easing Functions Level 1](https://www.w3.org/TR/css-easing-1/). 13 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-easing-1&#x2F;](https://www.w3.org/TR/css-easing-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 3 September 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 19 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
[CSS Values and Units Module Level 5](https://drafts.csswg.org/css-values-5/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-values-5&#x2F;](https://drafts.csswg.org/css-values-5/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaq"></a>\[MEDIAQ\]  
Florian Rivoal; Tab Atkins Jr.. [Media Queries Level 4](https://www.w3.org/TR/mediaqueries-4/). 25 December 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-4&#x2F;](https://www.w3.org/TR/mediaqueries-4/)

<a id="biblio-rfc6694"></a>\[RFC6694\]  
S. Moonesamy, Ed.. [The "about" URI Scheme](https://www.rfc-editor.org/rfc/rfc6694). August 2012. Informational. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;rfc-editor&#x2E;org&#x2F;rfc&#x2F;rfc6694](https://www.rfc-editor.org/rfc/rfc6694)
